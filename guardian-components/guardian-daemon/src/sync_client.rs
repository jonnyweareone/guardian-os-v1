//! Sync client for Guardian OS v1.1.0
//! 
//! Handles data synchronization with Supabase backend via Edge Functions

use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{info, error, debug, warn};
use anyhow::Result;
use chrono::{Utc, Local, NaiveDate};

use crate::AppState;
use crate::supabase::{
    SupabaseClient, SyncPayload, TopicSummarySync, TopicEntry,
    BrowsingSummarySync, DomainEntry, CategoryEntry, 
    AlertSync, ContactSync, ChildInteraction,
    DeviceRegistration, HardwareInfo, DeviceStatusResponse,
};
use crate::config::Config;

const GUARDIAN_VERSION: &str = "1.1.0";

/// Device activation state
#[derive(Debug, Clone)]
pub enum ActivationState {
    NotRegistered,
    WaitingForActivation { device_code: String },
    Activated { family_id: String, child_id: Option<String> },
}

/// Sync client managing cloud communication
pub struct SyncClient {
    supabase: SupabaseClient,
    activation_state: Arc<RwLock<ActivationState>>,
    cached_config: Arc<RwLock<Option<DeviceStatusResponse>>>,
}

impl SyncClient {
    pub fn new() -> Self {
        Self {
            supabase: SupabaseClient::new(),
            activation_state: Arc::new(RwLock::new(ActivationState::NotRegistered)),
            cached_config: Arc::new(RwLock::new(None)),
        }
    }

    /// Register device and get activation code
    pub async fn register_device(&self) -> Result<String> {
        let hardware_info = HardwareInfo {
            cpu: get_cpu_info(),
            ram_gb: get_ram_gb(),
            gpu: get_gpu_info(),
            hostname: get_hostname(),
        };

        let registration = DeviceRegistration {
            os_version: GUARDIAN_VERSION.to_string(),
            device_type: Some(detect_device_type()),
            hardware_info: Some(hardware_info),
        };

        let response = self.supabase.register_device(&registration).await?;
        
        // Update state
        {
            let mut state = self.activation_state.write().await;
            *state = ActivationState::WaitingForActivation {
                device_code: response.device_code.clone(),
            };
        }

        info!("Device registered: {}", response.device_code);
        info!("Activation URL: {}", response.activation_url);
        
        Ok(response.device_code)
    }

    /// Poll for activation status
    pub async fn check_activation(&self, device_code: &str) -> Result<bool> {
        let status = self.supabase.check_activation(device_code).await?;
        
        if status.activated {
            let family_id = status.family.as_ref().map(|f| f.id.clone()).unwrap_or_default();
            let child_id = status.assigned_child.as_ref().map(|c| c.id.clone());
            
            // Update state
            {
                let mut state = self.activation_state.write().await;
                *state = ActivationState::Activated { 
                    family_id: family_id.clone(), 
                    child_id: child_id.clone(),
                };
            }

            // Cache config
            {
                let mut config = self.cached_config.write().await;
                *config = Some(status.clone());
            }

            // Set device code on supabase client for sync
            let mut supabase = self.supabase.clone();
            // Note: In real implementation, supabase would be Arc<RwLock<SupabaseClient>>
            
            info!("Device activated! Family: {}", family_id);
            if let Some(ref child) = status.assigned_child {
                info!("Assigned to child: {} ({})", child.name, child.age_tier);
            }
            
            Ok(true)
        } else {
            debug!("Device not yet activated");
            Ok(false)
        }
    }

    /// Get cached configuration
    pub async fn get_config(&self) -> Option<DeviceStatusResponse> {
        self.cached_config.read().await.clone()
    }

    /// Sync topic summaries to cloud
    pub async fn sync_topics(&self, summaries: Vec<TopicSummarySync>, device_code: &str) -> Result<u32> {
        let payload = SyncPayload {
            device_code: device_code.to_string(),
            os_version: GUARDIAN_VERSION.to_string(),
            topic_summaries: Some(summaries),
            browsing_summary: None,
            alerts: None,
            contacts: None,
        };

        let mut client = SupabaseClient::with_device_code(device_code);
        let response = client.sync_data(&payload).await?;
        
        Ok(response.results.topic_summaries)
    }

    /// Sync browsing summary to cloud
    pub async fn sync_browsing(&self, summary: BrowsingSummarySync, device_code: &str) -> Result<bool> {
        let payload = SyncPayload {
            device_code: device_code.to_string(),
            os_version: GUARDIAN_VERSION.to_string(),
            topic_summaries: None,
            browsing_summary: Some(summary),
            alerts: None,
            contacts: None,
        };

        let mut client = SupabaseClient::with_device_code(device_code);
        let response = client.sync_data(&payload).await?;
        
        Ok(response.results.browsing_summary)
    }

    /// Send alert to cloud (triggers push notification)
    pub async fn send_alert(&self, alert: AlertSync, device_code: &str) -> Result<()> {
        let payload = SyncPayload {
            device_code: device_code.to_string(),
            os_version: GUARDIAN_VERSION.to_string(),
            topic_summaries: None,
            browsing_summary: None,
            alerts: Some(vec![alert]),
            contacts: None,
        };

        let mut client = SupabaseClient::with_device_code(device_code);
        let response = client.sync_data(&payload).await?;
        
        if response.results.alerts > 0 {
            info!("Alert sent successfully");
        }
        
        Ok(())
    }

    /// Sync contact data to cloud
    pub async fn sync_contacts(&self, contacts: Vec<ContactSync>, device_code: &str) -> Result<u32> {
        let payload = SyncPayload {
            device_code: device_code.to_string(),
            os_version: GUARDIAN_VERSION.to_string(),
            topic_summaries: None,
            browsing_summary: None,
            alerts: None,
            contacts: Some(contacts),
        };

        let mut client = SupabaseClient::with_device_code(device_code);
        let response = client.sync_data(&payload).await?;
        
        Ok(response.results.contacts)
    }

    /// Send heartbeat
    pub async fn heartbeat(&self, device_code: &str) -> Result<()> {
        let mut client = SupabaseClient::with_device_code(device_code);
        client.heartbeat().await
    }
}

impl Clone for SyncClient {
    fn clone(&self) -> Self {
        Self {
            supabase: SupabaseClient::new(),
            activation_state: Arc::clone(&self.activation_state),
            cached_config: Arc::clone(&self.cached_config),
        }
    }
}

// ============ Sync Loop ============

/// Run the main sync loop
pub async fn run_sync_loop(state: Arc<AppState>) -> Result<()> {
    let sync_interval = Duration::from_secs(state.config.sync_interval_secs.max(60));
    let heartbeat_interval = Duration::from_secs(30);
    
    let mut heartbeat_ticker = tokio::time::interval(heartbeat_interval);
    let mut sync_ticker = tokio::time::interval(sync_interval);

    info!("Starting sync loop (sync: {}s, heartbeat: 30s)", state.config.sync_interval_secs);

    loop {
        tokio::select! {
            _ = heartbeat_ticker.tick() => {
                if let Some(ref device_code) = state.config.device_code {
                    if let Some(ref client) = state.sync_client {
                        if let Err(e) = client.heartbeat(device_code).await {
                            debug!("Heartbeat failed: {}", e);
                        }
                    }
                }
            }
            
            _ = sync_ticker.tick() => {
                if let Some(ref device_code) = state.config.device_code {
                    if let Some(ref client) = state.sync_client {
                        // Sync DNS/browsing summaries
                        if let Err(e) = sync_dns_data(&state, client, device_code).await {
                            error!("DNS sync failed: {}", e);
                        }
                        
                        // Sync any pending alerts
                        if let Err(e) = sync_pending_alerts(&state, client, device_code).await {
                            error!("Alert sync failed: {}", e);
                        }
                    }
                }
            }
        }
    }
}

/// Sync DNS query data to cloud
async fn sync_dns_data(state: &AppState, client: &SyncClient, device_code: &str) -> Result<()> {
    // Get today's DNS summary from local storage
    let today = Local::now().format("%Y-%m-%d").to_string();
    
    // In production, this would query the local DNS database
    // For now, create a placeholder summary
    let summary = BrowsingSummarySync {
        child_id: state.config.child_id.clone().unwrap_or_default(),
        date: today,
        top_domains: vec![],
        categories: vec![],
        blocked_count: 0,
        vpn_attempts: 0,
        total_queries: 0,
    };
    
    client.sync_browsing(summary, device_code).await?;
    debug!("DNS data synced");
    
    Ok(())
}

/// Sync pending alerts to cloud
async fn sync_pending_alerts(state: &AppState, client: &SyncClient, device_code: &str) -> Result<()> {
    // Get pending alerts from local storage
    // In production, this would query pending_alerts table
    
    // Placeholder - no pending alerts
    Ok(())
}

// ============ Helper Functions ============

fn get_cpu_info() -> Option<String> {
    #[cfg(target_os = "linux")]
    {
        if let Ok(contents) = std::fs::read_to_string("/proc/cpuinfo") {
            for line in contents.lines() {
                if line.starts_with("model name") {
                    if let Some(value) = line.split(':').nth(1) {
                        return Some(value.trim().to_string());
                    }
                }
            }
        }
    }
    None
}

fn get_ram_gb() -> Option<u32> {
    #[cfg(target_os = "linux")]
    {
        if let Ok(contents) = std::fs::read_to_string("/proc/meminfo") {
            for line in contents.lines() {
                if line.starts_with("MemTotal") {
                    if let Some(value) = line.split_whitespace().nth(1) {
                        if let Ok(kb) = value.parse::<u64>() {
                            return Some((kb / 1024 / 1024) as u32);
                        }
                    }
                }
            }
        }
    }
    None
}

fn get_gpu_info() -> Option<String> {
    #[cfg(target_os = "linux")]
    {
        // Try lspci for GPU info
        if let Ok(output) = std::process::Command::new("lspci")
            .args(["-v", "-s", "$(lspci | grep VGA | cut -d' ' -f1)"])
            .output()
        {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if !stdout.is_empty() {
                return Some(stdout.lines().next().unwrap_or("").to_string());
            }
        }
    }
    None
}

fn get_hostname() -> Option<String> {
    hostname::get().ok().map(|h| h.to_string_lossy().to_string())
}

fn detect_device_type() -> String {
    // Simple heuristic based on form factor
    #[cfg(target_os = "linux")]
    {
        // Check for laptop indicators
        if std::path::Path::new("/sys/class/power_supply/BAT0").exists() {
            return "laptop".to_string();
        }
    }
    "desktop".to_string()
}

impl Default for SyncClient {
    fn default() -> Self {
        Self::new()
    }
}
