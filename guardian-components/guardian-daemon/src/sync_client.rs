//! gRPC client for Guardian Sync Server

use std::sync::Arc;
use std::time::Duration;
use tonic::transport::Channel;
use tracing::{info, error, debug};
use anyhow::Result;

use crate::AppState;
use crate::activity::ActivityEvent;
use crate::rules::SafetyRules;

/// Client for communicating with Guardian Sync Server
pub struct GuardianSyncClient {
    // In production, this would be generated from .proto files
    // channel: Channel,
    server_url: String,
    connected: bool,
}

impl GuardianSyncClient {
    /// Connect to the Guardian Sync Server
    pub async fn connect(server_url: &str) -> Result<Self> {
        info!("Connecting to Guardian Sync Server at {}", server_url);
        
        // TODO: Actually connect via gRPC
        // let channel = Channel::from_shared(server_url.to_string())?
        //     .timeout(Duration::from_secs(10))
        //     .connect()
        //     .await?;
        
        Ok(Self {
            server_url: server_url.to_string(),
            connected: true,
        })
    }
    
    /// Check if connected to server
    pub fn is_connected(&self) -> bool {
        self.connected
    }
    
    /// Upload activity events to server
    pub async fn upload_activity(&self, events: &[ActivityEvent]) -> Result<Vec<String>> {
        if events.is_empty() {
            return Ok(vec![]);
        }
        
        debug!("Uploading {} activity events", events.len());
        
        // TODO: Implement actual gRPC call
        // For now, simulate success
        let synced_ids: Vec<String> = events.iter().map(|e| e.id.clone()).collect();
        
        info!("Successfully uploaded {} events", synced_ids.len());
        Ok(synced_ids)
    }
    
    /// Fetch latest safety rules from server
    pub async fn fetch_rules(&self, child_id: &str) -> Result<SafetyRules> {
        debug!("Fetching safety rules for child {}", child_id);
        
        // TODO: Implement actual gRPC call
        // For now, return default rules
        Ok(SafetyRules::default())
    }
    
    /// Send heartbeat to server
    pub async fn heartbeat(&self, device_id: &str) -> Result<()> {
        debug!("Sending heartbeat for device {}", device_id);
        
        // TODO: Implement actual gRPC call
        Ok(())
    }
    
    /// Register this device with the server
    pub async fn register_device(&self, device_info: &DeviceInfo) -> Result<String> {
        info!("Registering device: {}", device_info.hostname);
        
        // TODO: Implement actual gRPC call
        // Return a mock device ID for now
        Ok(uuid::Uuid::new_v4().to_string())
    }
    
    /// Report an alert to the server
    pub async fn report_alert(&self, alert: &Alert) -> Result<()> {
        info!("Reporting alert: {} - {}", alert.alert_type, alert.title);
        
        // TODO: Implement actual gRPC call
        Ok(())
    }
}

/// Device information for registration
#[derive(Debug, Clone)]
pub struct DeviceInfo {
    pub hostname: String,
    pub os_version: String,
    pub guardian_version: String,
    pub hardware_id: String,
}

/// Alert to send to parents
#[derive(Debug, Clone)]
pub struct Alert {
    pub alert_type: String,
    pub severity: String,
    pub title: String,
    pub description: String,
    pub child_id: Option<String>,
    pub metadata: serde_json::Value,
}

/// Run the sync loop - periodically syncs with server
pub async fn run_sync_loop(state: Arc<AppState>) -> Result<()> {
    let sync_interval = Duration::from_secs(state.config.sync_interval_secs);
    
    info!("Starting sync loop, interval: {} seconds", state.config.sync_interval_secs);
    
    loop {
        tokio::time::sleep(sync_interval).await;
        
        if let Some(ref client) = state.sync_client {
            // Upload pending activity
            match state.storage.get_unsynced_events(100).await {
                Ok(events) if !events.is_empty() => {
                    match client.upload_activity(&events).await {
                        Ok(synced_ids) => {
                            if let Err(e) = state.storage.mark_synced(&synced_ids).await {
                                error!("Failed to mark events as synced: {}", e);
                            }
                        }
                        Err(e) => {
                            error!("Failed to upload activity: {}", e);
                        }
                    }
                }
                Ok(_) => {
                    debug!("No pending events to sync");
                }
                Err(e) => {
                    error!("Failed to get unsynced events: {}", e);
                }
            }
            
            // Fetch latest rules
            if let Some(ref child_id) = state.config.child_id {
                match client.fetch_rules(child_id).await {
                    Ok(new_rules) => {
                        let mut rules = state.rules.write().await;
                        if new_rules.version > rules.version {
                            *rules = new_rules.clone();
                            if let Err(e) = state.storage.save_rules(&new_rules).await {
                                error!("Failed to cache rules: {}", e);
                            }
                            info!("Updated safety rules to version {}", new_rules.version);
                        }
                    }
                    Err(e) => {
                        error!("Failed to fetch rules: {}", e);
                    }
                }
            }
            
            // Send heartbeat
            if let Some(ref device_id) = state.config.device_id {
                if let Err(e) = client.heartbeat(device_id).await {
                    error!("Heartbeat failed: {}", e);
                }
            }
        }
    }
}
