//! Guardian DNS - Safe browsing DNS filter for Guardian OS
//!
//! Features:
//! - Domain blocking (adult, malware, gambling)
//! - Safe search enforcement (Google, Bing, YouTube)
//! - VPN/proxy detection
//! - Browsing logs (domains only)

mod blocklist;
mod config;
mod logger;
mod safesearch;
mod server;
mod vpn_detect;

use anyhow::Result;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

pub use config::DnsConfig;
pub use server::GuardianDnsServer;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .with_target(false)
        .init();

    info!("Guardian DNS v{} starting...", env!("CARGO_PKG_VERSION"));

    // Load configuration
    let config = DnsConfig::load()?;
    info!("Configuration loaded from {:?}", config.config_path);

    // Initialize the DNS server
    let server = GuardianDnsServer::new(config).await?;

    info!("Guardian DNS listening on {}", server.listen_addr());
    info!("Upstream DNS: {:?}", server.upstream_servers());
    info!("Blocklist entries: {}", server.blocklist_count());
    info!("Safe search: {}", if server.safesearch_enabled() { "enabled" } else { "disabled" });
    info!("VPN detection: {}", if server.vpn_detection_enabled() { "enabled" } else { "disabled" });

    // Run the server
    server.run().await?;

    Ok(())
}
