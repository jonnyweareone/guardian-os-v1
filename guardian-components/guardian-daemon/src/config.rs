//! Configuration management for Guardian Daemon

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use anyhow::Result;

/// Guardian OS Supabase project
const DEFAULT_SUPABASE_URL: &str = "https://gkyspvcafyttfhyjryyk.supabase.co";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuardianConfig {
    /// Path to the config file
    #[serde(skip)]
    pub config_path: PathBuf,
    
    /// Directory for local data storage
    pub data_dir: PathBuf,
    
    /// Device ID (assigned by sync server)
    pub device_id: Option<String>,
    
    /// Hardware ID (unique to this machine)
    pub hardware_id: Option<String>,
    
    /// Child profile ID (if this device is assigned to a child)
    pub child_id: Option<String>,
    
    /// Family ID
    pub family_id: Option<String>,
    
    // ============ Supabase Config ============
    
    /// Supabase project URL
    pub supabase_url: String,
    
    /// Supabase anonymous key (public)
    pub supabase_anon_key: String,
    
    /// User access token (after activation)
    pub access_token: Option<String>,
    
    // ============ gRPC Sync Server ============
    
    /// Whether sync is enabled
    pub sync_enabled: bool,
    
    /// Guardian gRPC Sync Server URL (for file sync)
    pub sync_server_url: String,
    
    // ============ Intervals ============
    
    /// How often to sync with backend (seconds)
    pub sync_interval_secs: u64,
    
    /// How often to check screen time (seconds)
    pub screen_time_check_interval_secs: u64,
    
    /// How often to send heartbeats (seconds)
    pub heartbeat_interval_secs: u64,
    
    // ============ Feature Flags ============
    
    /// Enable URL logging
    pub url_logging_enabled: bool,
    
    /// Enable app usage tracking
    pub app_tracking_enabled: bool,
    
    /// Enable screen time enforcement
    pub screen_time_enabled: bool,
    
    /// Offline mode (enforce rules without sync)
    pub offline_mode: bool,
    
    /// Enable AI content classification
    pub ai_classification_enabled: bool,
    
    // ============ Activation ============
    
    /// Device activation code (shown to user)
    pub activation_code: Option<String>,
    
    /// Whether device is activated
    pub activated: bool,
}

impl Default for GuardianConfig {
    fn default() -> Self {
        Self {
            config_path: PathBuf::from("/etc/guardian/daemon.toml"),
            data_dir: PathBuf::from("/var/lib/guardian"),
            device_id: None,
            hardware_id: None,
            child_id: None,
            family_id: None,
            supabase_url: DEFAULT_SUPABASE_URL.to_string(),
            supabase_anon_key: String::new(), // Must be set
            access_token: None,
            sync_enabled: true,
            sync_server_url: "https://sync.gameguardian.ai:443".to_string(),
            sync_interval_secs: 60,
            screen_time_check_interval_secs: 30,
            heartbeat_interval_secs: 300, // 5 minutes
            url_logging_enabled: true,
            app_tracking_enabled: true,
            screen_time_enabled: true,
            offline_mode: false,
            ai_classification_enabled: false,
            activation_code: None,
            activated: false,
        }
    }
}

impl GuardianConfig {
    /// Load configuration from file, or create default if not exists
    pub fn load() -> Result<Self> {
        let config_paths = vec![
            PathBuf::from("/etc/guardian/daemon.toml"),
            dirs::config_dir()
                .map(|p| p.join("guardian/daemon.toml"))
                .unwrap_or_default(),
            PathBuf::from("./daemon.toml"),
        ];
        
        for path in config_paths {
            if path.exists() {
                let content = std::fs::read_to_string(&path)?;
                let mut config: GuardianConfig = toml::from_str(&content)?;
                config.config_path = path;
                
                // Generate hardware ID if not set
                if config.hardware_id.is_none() {
                    config.hardware_id = Some(generate_hardware_id());
                }
                
                return Ok(config);
            }
        }
        
        // Return default config if no file found
        let mut config = Self::default();
        config.hardware_id = Some(generate_hardware_id());
        Ok(config)
    }
    
    /// Save configuration to file
    pub fn save(&self) -> Result<()> {
        let content = toml::to_string_pretty(self)?;
        
        // Ensure parent directory exists
        if let Some(parent) = self.config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        
        std::fs::write(&self.config_path, content)?;
        Ok(())
    }
    
    /// Check if device is activated
    pub fn is_activated(&self) -> bool {
        self.activated && self.device_id.is_some() && self.family_id.is_some()
    }
    
    /// Get screen time limit for today (weekday vs weekend)
    pub fn get_daily_limit_minutes(&self) -> Option<u32> {
        // This would be populated from synced rules
        // Default to None (unlimited) until rules are synced
        None
    }
}

/// Generate a unique hardware ID for this machine
fn generate_hardware_id() -> String {
    use std::process::Command;
    
    // Try to get machine-id first (Linux standard)
    if let Ok(id) = std::fs::read_to_string("/etc/machine-id") {
        return id.trim().to_string();
    }
    
    // Try DMI product UUID
    if let Ok(id) = std::fs::read_to_string("/sys/class/dmi/id/product_uuid") {
        return id.trim().to_string();
    }
    
    // Fall back to hostname + random UUID
    let hostname = hostname::get()
        .map(|h| h.to_string_lossy().to_string())
        .unwrap_or_else(|_| "unknown".to_string());
    
    format!("{}-{}", hostname, uuid::Uuid::new_v4())
}

/// Get the Supabase anon key from environment or embedded default
pub fn get_supabase_anon_key() -> String {
    std::env::var("GUARDIAN_SUPABASE_ANON_KEY")
        .unwrap_or_else(|_| {
            // This will be baked into the binary at build time
            // In production, use a build script to embed this
            include_str!("../supabase_anon_key.txt").trim().to_string()
        })
}
