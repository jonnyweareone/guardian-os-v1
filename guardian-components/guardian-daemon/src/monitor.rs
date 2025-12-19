//! Activity monitoring - tracks app usage, screen time, and active windows

use std::sync::Arc;
use std::time::Duration;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sysinfo::{System, ProcessExt, SystemExt};
use tokio::time::interval;
use tracing::{info, debug, warn};
use anyhow::Result;

use crate::AppState;
use crate::activity::{ActivityEvent, ActivityType};

/// Monitors system activity and enforces screen time rules
pub struct ActivityMonitor {
    state: Arc<AppState>,
    system: System,
}

impl ActivityMonitor {
    pub fn new(state: Arc<AppState>) -> Self {
        Self {
            state,
            system: System::new_all(),
        }
    }
    
    /// Main monitoring loop
    pub async fn run(&self) -> Result<()> {
        let check_interval = Duration::from_secs(
            self.state.config.screen_time_check_interval_secs
        );
        let mut ticker = interval(check_interval);
        
        info!("Activity monitor started, checking every {} seconds", 
              self.state.config.screen_time_check_interval_secs);
        
        loop {
            ticker.tick().await;
            
            // Refresh system info
            self.system.refresh_all();
            
            // Get current active window/app
            if let Some(active_app) = self.get_active_application().await {
                self.record_app_usage(&active_app).await?;
            }
            
            // Check screen time limits
            if self.state.config.screen_time_enabled {
                self.check_screen_time_limits().await?;
            }
        }
    }
    
    /// Get the currently focused application
    async fn get_active_application(&self) -> Option<ActiveApplication> {
        // This would use D-Bus to query the compositor (COSMIC/Wayland)
        // For now, return a placeholder
        // In production: query cosmic-comp via wayland protocols or D-Bus
        
        // TODO: Implement actual window tracking via:
        // - wlr-foreign-toplevel-management protocol
        // - Or COSMIC-specific D-Bus interface
        
        None
    }
    
    /// Record application usage to local storage
    async fn record_app_usage(&self, app: &ActiveApplication) -> Result<()> {
        let event = ActivityEvent {
            id: uuid::Uuid::new_v4().to_string(),
            device_id: self.state.config.device_id.clone().unwrap_or_default(),
            child_id: self.state.config.child_id.clone(),
            activity_type: ActivityType::AppUsage,
            timestamp: Utc::now(),
            data: serde_json::json!({
                "app_id": app.app_id,
                "app_name": app.name,
                "window_title": app.window_title,
                "duration_secs": app.active_duration_secs,
            }),
            synced: false,
        };
        
        self.state.storage.record_activity(&event).await?;
        debug!("Recorded app usage: {} for {} seconds", app.name, app.active_duration_secs);
        
        Ok(())
    }
    
    /// Check if screen time limits have been exceeded
    async fn check_screen_time_limits(&self) -> Result<()> {
        let rules = self.state.rules.read().await;
        
        if let Some(ref child_id) = self.state.config.child_id {
            // Get today's total screen time
            let today_usage = self.state.storage
                .get_daily_screen_time(child_id)
                .await?;
            
            // Check against limit
            if let Some(daily_limit) = rules.screen_time.daily_limit_minutes {
                let used_minutes = today_usage.as_secs() / 60;
                
                if used_minutes >= daily_limit as u64 {
                    warn!("Screen time limit reached for child {}", child_id);
                    self.enforce_screen_time_limit().await?;
                } else if used_minutes >= (daily_limit as u64 * 90 / 100) {
                    // 90% warning
                    self.show_screen_time_warning(daily_limit as u64 - used_minutes).await?;
                }
            }
        }
        
        Ok(())
    }
    
    /// Enforce screen time limit (lock screen or show blocking overlay)
    async fn enforce_screen_time_limit(&self) -> Result<()> {
        // TODO: Implement via D-Bus call to COSMIC session
        // Options:
        // 1. Lock the screen
        // 2. Show a full-screen overlay
        // 3. Log out the user
        
        info!("Enforcing screen time limit...");
        
        // Send notification first
        self.send_notification(
            "Screen Time Limit Reached",
            "Your screen time for today has ended. Please take a break!"
        ).await?;
        
        Ok(())
    }
    
    /// Show warning that screen time is almost up
    async fn show_screen_time_warning(&self, minutes_remaining: u64) -> Result<()> {
        self.send_notification(
            "Screen Time Warning",
            &format!("You have {} minutes of screen time remaining today.", minutes_remaining)
        ).await?;
        
        Ok(())
    }
    
    /// Send a desktop notification
    async fn send_notification(&self, title: &str, body: &str) -> Result<()> {
        // Use D-Bus to send notification via org.freedesktop.Notifications
        // For now, just log it
        info!("Notification: {} - {}", title, body);
        Ok(())
    }
}

/// Represents the currently active application
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveApplication {
    pub app_id: String,
    pub name: String,
    pub window_title: Option<String>,
    pub active_duration_secs: u64,
    pub pid: Option<u32>,
}
