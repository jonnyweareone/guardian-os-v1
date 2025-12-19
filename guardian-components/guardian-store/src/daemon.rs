//! Guardian daemon D-Bus client

use crate::ratings::AgeRating;

/// Client for communicating with guardian-daemon
#[derive(Clone)]
pub struct DaemonClient {
    // TODO: D-Bus connection
}

impl DaemonClient {
    pub fn new() -> Self {
        Self {}
    }
    
    /// Check if an app is allowed based on current policy
    pub async fn is_app_allowed(&self, app_id: &str, rating: &AgeRating) -> bool {
        // TODO: Query daemon via D-Bus
        // For now, allow all
        true
    }
    
    /// Get current child's allowed rating
    pub async fn get_allowed_rating(&self) -> AgeRating {
        // TODO: Query daemon via D-Bus
        AgeRating::Teen
    }
    
    /// Check if we're in parent mode
    pub async fn is_parent_mode(&self) -> bool {
        // TODO: Query daemon via D-Bus
        false
    }
    
    /// Verify parent PIN
    pub async fn verify_pin(&self, pin: &str) -> bool {
        // TODO: Verify PIN via daemon (which checks against Supabase)
        // For demo, accept "1234"
        pin == "1234"
    }
    
    /// Request app installation approval
    pub async fn request_app(&self, app_id: &str, app_name: &str) -> anyhow::Result<()> {
        // TODO: Create request via daemon -> Supabase
        log::info!("Creating app request: {} ({})", app_name, app_id);
        Ok(())
    }
    
    /// Install app (after approval)
    pub async fn install_app(&self, flatpak_ref: &str) -> anyhow::Result<()> {
        // TODO: Install via flatpak
        log::info!("Installing: {}", flatpak_ref);
        Ok(())
    }
}
