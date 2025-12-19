//! Guardian Daemon - AI-powered safety service for Guardian OS
//! 
//! This daemon provides:
//! - Screen time tracking and enforcement
//! - Application usage monitoring
//! - URL/content logging (from browsers via D-Bus)
//! - Real-time sync with Guardian Cloud (Supabase + gRPC)
//! - Local enforcement when offline

mod config;
mod monitor;
mod sync_client;
mod supabase;
mod rules;
mod activity;
mod dbus;
mod storage;

use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, error, warn, Level};
use tracing_subscriber::FmtSubscriber;

pub use config::GuardianConfig;
pub use monitor::ActivityMonitor;
pub use sync_client::GuardianSyncClient;
pub use supabase::SupabaseClient;
pub use rules::SafetyRules;
pub use activity::ActivityLog;
pub use storage::LocalStorage;

/// Application state shared across all components
pub struct AppState {
    pub config: GuardianConfig,
    pub rules: RwLock<SafetyRules>,
    pub storage: LocalStorage,
    pub sync_client: Option<GuardianSyncClient>,
    pub supabase: SupabaseClient,
}

impl AppState {
    pub async fn new(config: GuardianConfig) -> anyhow::Result<Self> {
        let storage = LocalStorage::new(&config.data_dir)?;
        let rules = storage.load_rules().await.unwrap_or_default();
        
        // Initialize Supabase client
        let supabase = SupabaseClient::new(&config.supabase_anon_key);
        
        // Initialize gRPC sync client (optional - for file sync)
        let sync_client = if config.sync_enabled {
            match GuardianSyncClient::connect(&config.sync_server_url).await {
                Ok(client) => Some(client),
                Err(e) => {
                    warn!("Failed to connect to gRPC sync server: {}", e);
                    None
                }
            }
        } else {
            None
        };
        
        Ok(Self {
            config,
            rules: RwLock::new(rules),
            storage,
            sync_client,
            supabase,
        })
    }
    
    /// Sync policies from Supabase
    pub async fn sync_policies(&self) -> anyhow::Result<()> {
        if let Some(ref child_id) = self.config.child_id {
            info!("Syncing policies for child {}", child_id);
            
            let policies = self.supabase.get_child_policies(child_id).await?;
            
            // Convert to SafetyRules format
            let mut rules = self.rules.write().await;
            
            // Update screen time rules
            if let Some(st) = policies.screen_time {
                rules.screen_time.daily_limit_minutes = st.weekday_limit_mins.map(|m| m as u32);
                // TODO: Convert more fields
            }
            
            // Update content filter rules
            if let Some(dns) = policies.dns_profile {
                rules.content_filter.blocked_domains = dns.blocked_domains.unwrap_or_default();
                rules.content_filter.allowed_domains = dns.allowed_domains.unwrap_or_default();
                rules.content_filter.safe_search_enabled = dns.enforce_safe_search.unwrap_or(true);
                // TODO: Convert more fields
            }
            
            // Update app restrictions
            for app in policies.app_policies {
                match app.policy.as_deref() {
                    Some("blocked") => {
                        rules.app_restrictions.apps.push(app.app_id);
                    }
                    Some("time_limited") => {
                        if let Some(limit) = app.daily_limit_mins {
                            rules.screen_time.app_limits.push(rules::AppTimeLimit {
                                app_id: app.app_id,
                                daily_limit_minutes: limit as u32,
                            });
                        }
                    }
                    _ => {}
                }
            }
            
            rules.version += 1;
            
            // Cache rules locally
            self.storage.save_rules(&rules).await?;
            
            info!("Policies synced, version {}", rules.version);
        }
        
        Ok(())
    }
    
    /// Send heartbeat to Supabase
    pub async fn send_heartbeat(&self) -> anyhow::Result<()> {
        if let Some(ref device_id) = self.config.device_id {
            let sys = sysinfo::System::new_all();
            
            let heartbeat = supabase::DeviceHeartbeat {
                device_id: device_id.clone(),
                ip_address: get_local_ip(),
                cpu_percent: Some(sys.global_cpu_info().cpu_usage()),
                memory_percent: Some((sys.used_memory() as f32 / sys.total_memory() as f32) * 100.0),
                disk_percent: None, // TODO: Get disk usage
                active_app: None, // TODO: Get from monitor
                screen_locked: None,
            };
            
            self.supabase.send_heartbeat(&heartbeat).await?;
        }
        Ok(())
    }
    
    /// Check for and execute pending commands
    pub async fn process_commands(&self) -> anyhow::Result<()> {
        if let Some(ref device_id) = self.config.device_id {
            let commands = self.supabase.get_pending_commands(device_id).await?;
            
            for cmd in commands {
                info!("Processing command: {} ({})", cmd.command, cmd.id);
                
                // Acknowledge receipt
                self.supabase.acknowledge_command(&cmd.id).await?;
                
                // Execute command
                let result = match cmd.command.as_str() {
                    "lock" => {
                        self.lock_screen().await;
                        serde_json::json!({"success": true})
                    }
                    "message" => {
                        if let Some(payload) = &cmd.payload {
                            if let Some(msg) = payload.get("message").and_then(|m| m.as_str()) {
                                self.show_message(msg).await;
                            }
                        }
                        serde_json::json!({"success": true})
                    }
                    "update_policies" => {
                        self.sync_policies().await?;
                        serde_json::json!({"success": true})
                    }
                    "screenshot" => {
                        // TODO: Implement screenshot capture
                        serde_json::json!({"success": false, "error": "not implemented"})
                    }
                    _ => {
                        serde_json::json!({"success": false, "error": "unknown command"})
                    }
                };
                
                // Report completion
                self.supabase.complete_command(&cmd.id, result).await?;
            }
        }
        Ok(())
    }
    
    async fn lock_screen(&self) {
        // TODO: Implement via D-Bus to COSMIC session
        info!("Locking screen...");
    }
    
    async fn show_message(&self, message: &str) {
        // TODO: Show notification or overlay
        info!("Showing message: {}", message);
    }
}

/// Get local IP address
fn get_local_ip() -> Option<String> {
    use std::net::UdpSocket;
    let socket = UdpSocket::bind("0.0.0.0:0").ok()?;
    socket.connect("8.8.8.8:80").ok()?;
    socket.local_addr().ok().map(|addr| addr.ip().to_string())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .with_target(false)
        .init();
    
    info!("Guardian Daemon v{} starting...", env!("CARGO_PKG_VERSION"));
    
    // Load configuration
    let config = GuardianConfig::load()?;
    info!("Configuration loaded from {}", config.config_path.display());
    
    // Initialize application state
    let state = Arc::new(AppState::new(config).await?);
    info!("Application state initialized");
    
    // Initial policy sync
    if let Err(e) = state.sync_policies().await {
        warn!("Initial policy sync failed: {}", e);
    }
    
    // Start the activity monitor
    let monitor_state = Arc::clone(&state);
    let monitor_handle = tokio::spawn(async move {
        let monitor = ActivityMonitor::new(monitor_state);
        if let Err(e) = monitor.run().await {
            error!("Activity monitor error: {}", e);
        }
    });
    
    // Start D-Bus service for browser integration
    let dbus_state = Arc::clone(&state);
    let dbus_handle = tokio::spawn(async move {
        if let Err(e) = dbus::run_dbus_service(dbus_state).await {
            error!("D-Bus service error: {}", e);
        }
    });
    
    // Start main sync loop
    let sync_state = Arc::clone(&state);
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(
            std::time::Duration::from_secs(sync_state.config.sync_interval_secs)
        );
        
        loop {
            interval.tick().await;
            
            // Send heartbeat
            if let Err(e) = sync_state.send_heartbeat().await {
                error!("Heartbeat failed: {}", e);
            }
            
            // Check for commands
            if let Err(e) = sync_state.process_commands().await {
                error!("Command processing failed: {}", e);
            }
            
            // Sync policies periodically
            if let Err(e) = sync_state.sync_policies().await {
                error!("Policy sync failed: {}", e);
            }
        }
    });
    
    // Wait for shutdown signal
    tokio::signal::ctrl_c().await?;
    info!("Shutdown signal received, stopping Guardian Daemon...");
    
    Ok(())
}
