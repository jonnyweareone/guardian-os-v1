//! Configuration for Guardian DNS

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsConfig {
    /// Path this config was loaded from
    #[serde(skip)]
    pub config_path: PathBuf,
    
    /// Server configuration
    pub server: ServerConfig,
    
    /// Safe search configuration
    pub safesearch: SafeSearchConfig,
    
    /// Blocking configuration
    pub blocking: BlockingConfig,
    
    /// VPN detection configuration
    pub vpn_detection: VpnDetectionConfig,
    
    /// Logging configuration
    pub logging: LoggingConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// Address to listen on
    #[serde(default = "default_listen_addr")]
    pub listen_addr: String,
    
    /// Upstream DNS servers
    #[serde(default = "default_upstream")]
    pub upstream_dns: Vec<String>,
    
    /// Cache size (number of entries)
    #[serde(default = "default_cache_size")]
    pub cache_size: usize,
    
    /// Cache TTL in seconds
    #[serde(default = "default_cache_ttl")]
    pub cache_ttl: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafeSearchConfig {
    /// Enable safe search enforcement
    #[serde(default = "default_true")]
    pub enabled: bool,
    
    /// Enforce Google safe search
    #[serde(default = "default_true")]
    pub google: bool,
    
    /// Enforce Bing safe search
    #[serde(default = "default_true")]
    pub bing: bool,
    
    /// YouTube restriction level: "strict", "moderate", or "off"
    #[serde(default = "default_youtube_mode")]
    pub youtube: String,
    
    /// Enforce DuckDuckGo safe search
    #[serde(default = "default_true")]
    pub duckduckgo: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockingConfig {
    /// Categories to always block
    #[serde(default = "default_always_block")]
    pub always_block: Vec<String>,
    
    /// URLs of blocklists to download
    #[serde(default = "default_blocklists")]
    pub blocklists: Vec<String>,
    
    /// Custom domains to block
    #[serde(default)]
    pub custom_block: Vec<String>,
    
    /// Domains to never block (whitelist)
    #[serde(default)]
    pub whitelist: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VpnDetectionConfig {
    /// Enable VPN detection
    #[serde(default = "default_true")]
    pub enabled: bool,
    
    /// Block known VPN domains
    #[serde(default = "default_true")]
    pub block_vpn_domains: bool,
    
    /// Block DNS-over-HTTPS providers
    #[serde(default = "default_true")]
    pub block_doh: bool,
    
    /// Alert parent on VPN attempt
    #[serde(default = "default_true")]
    pub alert_on_attempt: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Enable query logging
    #[serde(default = "default_true")]
    pub enabled: bool,
    
    /// Days to retain logs
    #[serde(default = "default_retention_days")]
    pub retention_days: u32,
    
    /// Path to query log database
    #[serde(default = "default_log_path")]
    pub log_path: PathBuf,
}

// Default value functions
fn default_listen_addr() -> String {
    "127.0.0.1:53".to_string()
}

fn default_upstream() -> Vec<String> {
    vec![
        "1.1.1.3".to_string(),  // Cloudflare Family (blocks malware + adult)
        "1.0.0.3".to_string(),
    ]
}

fn default_cache_size() -> usize {
    10000
}

fn default_cache_ttl() -> u64 {
    300
}

fn default_true() -> bool {
    true
}

fn default_youtube_mode() -> String {
    "moderate".to_string()
}

fn default_always_block() -> Vec<String> {
    vec![
        "adult".to_string(),
        "malware".to_string(),
        "gambling".to_string(),
    ]
}

fn default_blocklists() -> Vec<String> {
    vec![
        "https://raw.githubusercontent.com/StevenBlack/hosts/master/hosts".to_string(),
    ]
}

fn default_retention_days() -> u32 {
    30
}

fn default_log_path() -> PathBuf {
    PathBuf::from("/var/lib/guardian/dns/queries.db")
}

impl Default for DnsConfig {
    fn default() -> Self {
        Self {
            config_path: PathBuf::from("/etc/guardian/dns.toml"),
            server: ServerConfig {
                listen_addr: default_listen_addr(),
                upstream_dns: default_upstream(),
                cache_size: default_cache_size(),
                cache_ttl: default_cache_ttl(),
            },
            safesearch: SafeSearchConfig {
                enabled: true,
                google: true,
                bing: true,
                youtube: default_youtube_mode(),
                duckduckgo: true,
            },
            blocking: BlockingConfig {
                always_block: default_always_block(),
                blocklists: default_blocklists(),
                custom_block: vec![],
                whitelist: vec![],
            },
            vpn_detection: VpnDetectionConfig {
                enabled: true,
                block_vpn_domains: true,
                block_doh: true,
                alert_on_attempt: true,
            },
            logging: LoggingConfig {
                enabled: true,
                retention_days: default_retention_days(),
                log_path: default_log_path(),
            },
        }
    }
}

impl DnsConfig {
    /// Load configuration from file or use defaults
    pub fn load() -> Result<Self> {
        let config_paths = [
            PathBuf::from("/etc/guardian/dns.toml"),
            dirs::config_dir()
                .map(|p| p.join("guardian/dns.toml"))
                .unwrap_or_default(),
            PathBuf::from("dns.toml"),
        ];
        
        for path in &config_paths {
            if path.exists() {
                let content = std::fs::read_to_string(path)
                    .with_context(|| format!("Failed to read config from {:?}", path))?;
                let mut config: DnsConfig = toml::from_str(&content)
                    .with_context(|| format!("Failed to parse config from {:?}", path))?;
                config.config_path = path.clone();
                return Ok(config);
            }
        }
        
        // No config found, use defaults
        tracing::warn!("No configuration file found, using defaults");
        Ok(Self::default())
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
}
