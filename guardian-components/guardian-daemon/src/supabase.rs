//! Supabase client for Guardian Daemon
//! 
//! Handles device registration, activation, heartbeat, and policy sync

use anyhow::{Result, Context};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::{info, debug, error, warn};
use chrono::{DateTime, Utc};

const SUPABASE_URL: &str = "https://gkyspvcafyttfhyjryyk.supabase.co";
const SUPABASE_ANON_KEY: &str = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJzdXBhYmFzZSIsInJlZiI6ImdreXNwdmNhZnl0dGZoeWpyeXlrIiwicm9sZSI6ImFub24iLCJpYXQiOjE3NjYxMDIzMzQsImV4cCI6MjA4MTY3ODMzNH0.Ns5N9Y9uZgWqdhnYiX5IrubOO-Xopl2urBDR1AVD7FI";

/// Supabase client for Guardian OS
pub struct SupabaseClient {
    client: Client,
    anon_key: String,
}

impl SupabaseClient {
    pub fn new(anon_key: &str) -> Self {
        Self {
            client: Client::new(),
            anon_key: if anon_key.is_empty() { 
                SUPABASE_ANON_KEY.to_string() 
            } else { 
                anon_key.to_string() 
            },
        }
    }

    fn headers(&self) -> Vec<(&str, String)> {
        vec![
            ("apikey", self.anon_key.clone()),
            ("Authorization", format!("Bearer {}", self.anon_key)),
            ("Content-Type", "application/json".to_string()),
        ]
    }

    // ============ Device Registration ============

    /// Register a new device via Edge Function
    pub async fn register_device(&self, registration: &DeviceRegistration) -> Result<DeviceRegistrationResponse> {
        let resp = self.client
            .post(&format!("{}/functions/v1/device-register", SUPABASE_URL))
            .header("Authorization", format!("Bearer {}", self.anon_key))
            .header("Content-Type", "application/json")
            .json(registration)
            .send()
            .await
            .context("Failed to call device-register function")?;

        if !resp.status().is_success() {
            let error_text = resp.text().await.unwrap_or_default();
            anyhow::bail!("Device registration failed: {}", error_text);
        }

        let response: DeviceRegistrationResponse = resp.json().await
            .context("Failed to parse registration response")?;
        
        info!("Device registered with code: {}", response.device_code);
        Ok(response)
    }

    /// Check device activation status
    pub async fn check_activation(&self, device_code: &str) -> Result<DeviceStatusResponse> {
        let resp = self.client
            .get(&format!(
                "{}/functions/v1/device-status?device_code={}",
                SUPABASE_URL, device_code
            ))
            .header("Authorization", format!("Bearer {}", self.anon_key))
            .send()
            .await
            .context("Failed to check activation status")?;

        if !resp.status().is_success() {
            let error_text = resp.text().await.unwrap_or_default();
            anyhow::bail!("Activation check failed: {}", error_text);
        }

        resp.json().await.context("Failed to parse status response")
    }

    // ============ Heartbeat & Status ============

    /// Send device heartbeat with system status
    pub async fn send_heartbeat(&self, heartbeat: &DeviceHeartbeat) -> Result<()> {
        let payload = serde_json::json!({
            "last_seen": Utc::now().to_rfc3339(),
            "ip_address": heartbeat.ip_address,
            "cpu_percent": heartbeat.cpu_percent,
            "memory_percent": heartbeat.memory_percent,
            "disk_percent": heartbeat.disk_percent,
            "active_app": heartbeat.active_app,
            "screen_locked": heartbeat.screen_locked
        });

        let resp = self.client
            .patch(&format!(
                "{}/rest/v1/devices?id=eq.{}",
                SUPABASE_URL, heartbeat.device_id
            ))
            .header("apikey", &self.anon_key)
            .header("Authorization", format!("Bearer {}", self.anon_key))
            .header("Content-Type", "application/json")
            .header("Prefer", "return=minimal")
            .json(&payload)
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            warn!("Heartbeat failed: {} - {}", status, text);
        } else {
            debug!("Heartbeat sent for device {}", heartbeat.device_id);
        }

        Ok(())
    }

    // ============ Commands ============

    /// Get pending commands for this device
    pub async fn get_pending_commands(&self, device_id: &str) -> Result<Vec<PendingCommand>> {
        let resp = self.client
            .get(&format!(
                "{}/rest/v1/device_commands?device_id=eq.{}&status=eq.pending&select=id,command,payload,created_at&order=created_at.asc",
                SUPABASE_URL, device_id
            ))
            .header("apikey", &self.anon_key)
            .header("Authorization", format!("Bearer {}", self.anon_key))
            .send()
            .await?;

        if !resp.status().is_success() {
            return Ok(vec![]);
        }

        resp.json().await.unwrap_or_else(|_| vec![]).pipe(Ok)
    }

    /// Acknowledge receipt of a command
    pub async fn acknowledge_command(&self, command_id: &str) -> Result<()> {
        self.client
            .patch(&format!(
                "{}/rest/v1/device_commands?id=eq.{}",
                SUPABASE_URL, command_id
            ))
            .header("apikey", &self.anon_key)
            .header("Authorization", format!("Bearer {}", self.anon_key))
            .header("Content-Type", "application/json")
            .json(&serde_json::json!({
                "status": "acknowledged",
                "acknowledged_at": Utc::now().to_rfc3339()
            }))
            .send()
            .await?;

        debug!("Command {} acknowledged", command_id);
        Ok(())
    }

    /// Mark command as completed
    pub async fn complete_command(&self, command_id: &str, result: serde_json::Value) -> Result<()> {
        self.client
            .patch(&format!(
                "{}/rest/v1/device_commands?id=eq.{}",
                SUPABASE_URL, command_id
            ))
            .header("apikey", &self.anon_key)
            .header("Authorization", format!("Bearer {}", self.anon_key))
            .header("Content-Type", "application/json")
            .json(&serde_json::json!({
                "status": "completed",
                "result": result,
                "completed_at": Utc::now().to_rfc3339()
            }))
            .send()
            .await?;

        debug!("Command {} completed", command_id);
        Ok(())
    }

    // ============ Policies ============

    /// Get all policies for a child
    pub async fn get_child_policies(&self, child_id: &str) -> Result<ChildPolicies> {
        // Get screen time policy
        let screen_time: Option<ScreenTimePolicy> = self.client
            .get(&format!(
                "{}/rest/v1/screen_time_policies?child_id=eq.{}&select=*",
                SUPABASE_URL, child_id
            ))
            .header("apikey", &self.anon_key)
            .header("Authorization", format!("Bearer {}", self.anon_key))
            .send()
            .await?
            .json::<Vec<ScreenTimePolicy>>()
            .await?
            .into_iter()
            .next();

        // Get DNS policy
        let dns_profile: Option<DnsPolicy> = self.client
            .get(&format!(
                "{}/rest/v1/dns_policies?child_id=eq.{}&select=*",
                SUPABASE_URL, child_id
            ))
            .header("apikey", &self.anon_key)
            .header("Authorization", format!("Bearer {}", self.anon_key))
            .send()
            .await?
            .json::<Vec<DnsPolicy>>()
            .await?
            .into_iter()
            .next();

        // Get app policies
        let app_policies: Vec<AppPolicy> = self.client
            .get(&format!(
                "{}/rest/v1/app_policies?child_id=eq.{}&select=*",
                SUPABASE_URL, child_id
            ))
            .header("apikey", &self.anon_key)
            .header("Authorization", format!("Bearer {}", self.anon_key))
            .send()
            .await?
            .json()
            .await
            .unwrap_or_default();

        Ok(ChildPolicies {
            screen_time,
            dns_profile,
            app_policies,
        })
    }

    // ============ Activity Logging ============

    /// Log activity to Supabase
    pub async fn log_activity(&self, device_id: &str, child_id: Option<&str>, activity_type: &str, data: serde_json::Value) -> Result<()> {
        let payload = serde_json::json!({
            "device_id": device_id,
            "child_id": child_id,
            "activity_type": activity_type,
            "data": data,
            "timestamp": Utc::now().to_rfc3339()
        });

        self.client
            .post(&format!("{}/rest/v1/activity_logs", SUPABASE_URL))
            .header("apikey", &self.anon_key)
            .header("Authorization", format!("Bearer {}", self.anon_key))
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await?;

        debug!("Activity logged: {}", activity_type);
        Ok(())
    }

    /// Create an alert
    pub async fn create_alert(&self, alert: &AlertCreate) -> Result<()> {
        self.client
            .post(&format!("{}/rest/v1/alerts", SUPABASE_URL))
            .header("apikey", &self.anon_key)
            .header("Authorization", format!("Bearer {}", self.anon_key))
            .header("Content-Type", "application/json")
            .json(alert)
            .send()
            .await?;

        info!("Alert created: {} - {}", alert.trigger_type, alert.summary);
        Ok(())
    }
}

impl Default for SupabaseClient {
    fn default() -> Self {
        Self::new(SUPABASE_ANON_KEY)
    }
}

// Helper trait
trait Pipe: Sized {
    fn pipe<F, R>(self, f: F) -> R where F: FnOnce(Self) -> R {
        f(self)
    }
}
impl<T> Pipe for T {}

// ============ Types ============

#[derive(Debug, Serialize)]
pub struct DeviceHeartbeat {
    pub device_id: String,
    pub ip_address: Option<String>,
    pub cpu_percent: Option<f32>,
    pub memory_percent: Option<f32>,
    pub disk_percent: Option<f32>,
    pub active_app: Option<String>,
    pub screen_locked: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct PendingCommand {
    pub id: String,
    pub command: String,
    pub payload: Option<serde_json::Value>,
    pub created_at: String,
}

#[derive(Debug, Serialize)]
pub struct DeviceRegistration {
    pub os_version: String,
    pub device_type: Option<String>,
    pub hardware_info: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct DeviceRegistrationResponse {
    pub success: bool,
    pub device_code: String,
    pub device_id: String,
    pub activation_url: String,
    pub poll_interval_seconds: u32,
}

#[derive(Debug, Deserialize)]
pub struct DeviceStatusResponse {
    pub activated: bool,
    #[serde(default)]
    pub device_id: Option<String>,
    #[serde(default)]
    pub family_id: Option<String>,
    #[serde(default)]
    pub child_id: Option<String>,
    #[serde(default)]
    pub child_name: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ChildPolicies {
    pub screen_time: Option<ScreenTimePolicy>,
    pub dns_profile: Option<DnsPolicy>,
    pub app_policies: Vec<AppPolicy>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ScreenTimePolicy {
    pub id: String,
    pub child_id: String,
    pub enabled: Option<bool>,
    pub weekday_limit_mins: Option<i32>,
    pub weekend_limit_mins: Option<i32>,
    pub bedtime_enabled: Option<bool>,
    pub bedtime_start: Option<String>,
    pub bedtime_end: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DnsPolicy {
    pub id: String,
    pub child_id: String,
    pub enabled: Option<bool>,
    pub enforce_safe_search: Option<bool>,
    pub blocked_categories: Option<Vec<String>>,
    pub blocked_domains: Option<Vec<String>>,
    pub allowed_domains: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AppPolicy {
    pub id: String,
    pub child_id: String,
    pub app_id: String,
    pub app_name: String,
    pub policy: Option<String>,
    pub daily_limit_mins: Option<i32>,
}

#[derive(Debug, Serialize)]
pub struct AlertCreate {
    pub family_id: String,
    pub child_id: String,
    pub alert_tier: String,
    pub trigger_type: String,
    pub risk_score: f32,
    pub summary: String,
    pub risk_factors: Vec<String>,
    pub recommended_action: Option<String>,
}
