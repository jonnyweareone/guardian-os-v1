//! D-Bus service for browser and desktop integration

use std::sync::Arc;
use zbus::{Connection, interface, fdo};
use tracing::{info, debug, error};
use anyhow::Result;

use crate::AppState;
use crate::activity::ActivityEvent;

/// D-Bus interface for Guardian Daemon
/// Browsers and other apps can report activity via this interface
pub struct GuardianDbusService {
    state: Arc<AppState>,
}

#[interface(name = "org.guardiannetwork.Daemon")]
impl GuardianDbusService {
    /// Report a URL visit from a browser extension
    async fn report_url_visit(
        &self,
        url: String,
        title: String,
        browser: String,
    ) -> fdo::Result<bool> {
        debug!("URL visit reported: {} from {}", url, browser);
        
        // Check if URL should be blocked
        let rules = self.state.rules.read().await;
        let domain = url::Url::parse(&url)
            .map(|u| u.host_str().unwrap_or("unknown").to_string())
            .unwrap_or_else(|_| "unknown".to_string());
        
        let blocked = rules.should_block_url(&url, &domain);
        
        // Record the visit
        let event = ActivityEvent::url_visit(
            self.state.config.device_id.as_deref().unwrap_or("unknown"),
            self.state.config.child_id.as_deref(),
            &url,
            Some(&title),
            &browser,
        );
        
        if let Err(e) = self.state.storage.record_activity(&event).await {
            error!("Failed to record URL visit: {}", e);
        }
        
        Ok(blocked)
    }
    
    /// Report an app launch
    async fn report_app_launch(
        &self,
        app_id: String,
        app_name: String,
    ) -> fdo::Result<bool> {
        debug!("App launch reported: {} ({})", app_name, app_id);
        
        // Check if app is allowed
        let rules = self.state.rules.read().await;
        let allowed = rules.is_app_allowed(&app_id);
        
        if !allowed {
            info!("Blocked app launch: {}", app_id);
        }
        
        Ok(allowed)
    }
    
    /// Get remaining screen time in minutes
    async fn get_remaining_screen_time(&self) -> fdo::Result<i32> {
        let rules = self.state.rules.read().await;
        
        if let (Some(ref child_id), Some(limit)) = (
            &self.state.config.child_id,
            rules.screen_time.daily_limit_minutes
        ) {
            match self.state.storage.get_daily_screen_time(child_id).await {
                Ok(used) => {
                    let used_minutes = (used.as_secs() / 60) as i32;
                    let remaining = (limit as i32) - used_minutes;
                    Ok(remaining.max(0))
                }
                Err(_) => Ok(limit as i32),
            }
        } else {
            Ok(-1) // -1 means unlimited
        }
    }
    
    /// Check if device is within allowed hours
    async fn is_within_allowed_hours(&self) -> fdo::Result<bool> {
        let rules = self.state.rules.read().await;
        Ok(rules.is_within_allowed_hours())
    }
    
    /// Get the current child profile ID
    async fn get_child_id(&self) -> fdo::Result<String> {
        Ok(self.state.config.child_id.clone().unwrap_or_default())
    }
    
    /// Trigger a rule sync from the server
    async fn sync_rules(&self) -> fdo::Result<bool> {
        if let Some(ref client) = self.state.sync_client {
            if let Some(ref child_id) = self.state.config.child_id {
                match client.fetch_rules(child_id).await {
                    Ok(new_rules) => {
                        let mut rules = self.state.rules.write().await;
                        *rules = new_rules;
                        return Ok(true);
                    }
                    Err(e) => {
                        error!("Failed to sync rules: {}", e);
                    }
                }
            }
        }
        Ok(false)
    }
}

/// Run the D-Bus service
pub async fn run_dbus_service(state: Arc<AppState>) -> Result<()> {
    info!("Starting D-Bus service on org.guardiannetwork.Daemon");
    
    let connection = Connection::session().await?;
    
    let service = GuardianDbusService { state };
    
    connection
        .object_server()
        .at("/org/guardiannetwork/Daemon", service)
        .await?;
    
    connection
        .request_name("org.guardiannetwork.Daemon")
        .await?;
    
    info!("D-Bus service started successfully");
    
    // Keep the service running
    loop {
        tokio::time::sleep(std::time::Duration::from_secs(3600)).await;
    }
}
