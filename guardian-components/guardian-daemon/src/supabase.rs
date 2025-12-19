//! Supabase client for Guardian OS
//! 
//! Handles authentication and policy sync with Supabase backend

use anyhow::{Result, Context};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::{info, debug, error};
use chrono::{DateTime, Utc, NaiveTime};

const SUPABASE_URL: &str = "https://gkyspvcafyttfhyjryyk.supabase.co";

/// Supabase client for Guardian OS
pub struct SupabaseClient {
    client: Client,
    anon_key: String,
    access_token: Option<String>,
}

impl SupabaseClient {
    pub fn new(anon_key: &str) -> Self {
        Self {
            client: Client::new(),
            anon_key: anon_key.to_string(),
            access_token: None,
        }
    }

    /// Set the user's access token (after login)
    pub fn set_access_token(&mut self, token: &str) {
        self.access_token = Some(token.to_string());
    }

    /// Get authorization header
    fn auth_header(&self) -> String {
        match &self.access_token {
            Some(token) => format!("Bearer {}", token),
            None => format!("Bearer {}", self.anon_key),
        }
    }

    /// Register a new device with activation code
    pub async fn register_device(&self, device: &DeviceRegistration) -> Result<DeviceRecord> {
        let resp = self.client
            .post(&format!("{}/rest/v1/devices", SUPABASE_URL))
            .header("apikey", &self.anon_key)
            .header("Authorization", self.auth_header())
            .header("Content-Type", "application/json")
            .header("Prefer", "return=representation")
            .json(device)
            .send()
            .await
            .context("Failed to register device")?;

        if !resp.status().is_success() {
            let error_text = resp.text().await.unwrap_or_default();
            anyhow::bail!("Device registration failed: {}", error_text);
        }

        let records: Vec<DeviceRecord> = resp.json().await?;
        records.into_iter().next()
            .ok_or_else(|| anyhow::anyhow!("No device record returned"))
    }

    /// Check device activation status
    pub async fn check_activation(&self, device_id: &str) -> Result<Option<ActivationStatus>> {
        let resp = self.client
            .get(&format!(
                "{}/rest/v1/devices?id=eq.{}&select=id,status,activated_at,child_id,family_id",
                SUPABASE_URL, device_id
            ))
            .header("apikey", &self.anon_key)
            .header("Authorization", self.auth_header())
            .send()
            .await?;

        let records: Vec<ActivationStatus> = resp.json().await?;
        Ok(records.into_iter().next())
    }

    /// Get child's safety policies
    pub async fn get_child_policies(&self, child_id: &str) -> Result<ChildPolicies> {
        // Fetch screen time policy
        let screen_time = self.get_screen_time_policy(child_id).await?;
        
        // Fetch DNS/content policy
        let dns_profile = self.get_dns_profile(child_id).await?;
        
        // Fetch app policies
        let app_policies = self.get_app_policies(child_id).await?;

        Ok(ChildPolicies {
            child_id: child_id.to_string(),
            screen_time,
            dns_profile,
            app_policies,
        })
    }

    async fn get_screen_time_policy(&self, child_id: &str) -> Result<Option<ScreenTimePolicy>> {
        let resp = self.client
            .get(&format!(
                "{}/rest/v1/screen_time_policies?child_id=eq.{}",
                SUPABASE_URL, child_id
            ))
            .header("apikey", &self.anon_key)
            .header("Authorization", self.auth_header())
            .send()
            .await?;

        let records: Vec<ScreenTimePolicy> = resp.json().await?;
        Ok(records.into_iter().next())
    }

    async fn get_dns_profile(&self, child_id: &str) -> Result<Option<DnsProfile>> {
        let resp = self.client
            .get(&format!(
                "{}/rest/v1/dns_profiles?child_id=eq.{}",
                SUPABASE_URL, child_id
            ))
            .header("apikey", &self.anon_key)
            .header("Authorization", self.auth_header())
            .send()
            .await?;

        let records: Vec<DnsProfile> = resp.json().await?;
        Ok(records.into_iter().next())
    }

    async fn get_app_policies(&self, child_id: &str) -> Result<Vec<AppPolicy>> {
        let resp = self.client
            .get(&format!(
                "{}/rest/v1/app_policies?child_id=eq.{}",
                SUPABASE_URL, child_id
            ))
            .header("apikey", &self.anon_key)
            .header("Authorization", self.auth_header())
            .send()
            .await?;

        let records: Vec<AppPolicy> = resp.json().await?;
        Ok(records)
    }

    /// Send heartbeat
    pub async fn send_heartbeat(&self, heartbeat: &DeviceHeartbeat) -> Result<()> {
        let resp = self.client
            .post(&format!("{}/rest/v1/device_heartbeats", SUPABASE_URL))
            .header("apikey", &self.anon_key)
            .header("Authorization", self.auth_header())
            .header("Content-Type", "application/json")
            .json(heartbeat)
            .send()
            .await?;

        if !resp.status().is_success() {
            let error = resp.text().await.unwrap_or_default();
            error!("Heartbeat failed: {}", error);
        }

        // Also update device last_seen
        self.client
            .patch(&format!(
                "{}/rest/v1/devices?id=eq.{}",
                SUPABASE_URL, heartbeat.device_id
            ))
            .header("apikey", &self.anon_key)
            .header("Authorization", self.auth_header())
            .header("Content-Type", "application/json")
            .json(&serde_json::json!({
                "last_seen_at": Utc::now().to_rfc3339(),
                "ip_address": heartbeat.ip_address,
            }))
            .send()
            .await?;

        Ok(())
    }

    /// Log app session
    pub async fn log_app_session(&self, session: &AppSession) -> Result<()> {
        self.client
            .post(&format!("{}/rest/v1/app_sessions", SUPABASE_URL))
            .header("apikey", &self.anon_key)
            .header("Authorization", self.auth_header())
            .header("Content-Type", "application/json")
            .json(session)
            .send()
            .await?;
        Ok(())
    }

    /// Log URL visit
    pub async fn log_url(&self, url_log: &UrlLog) -> Result<()> {
        self.client
            .post(&format!("{}/rest/v1/url_logs", SUPABASE_URL))
            .header("apikey", &self.anon_key)
            .header("Authorization", self.auth_header())
            .header("Content-Type", "application/json")
            .json(url_log)
            .send()
            .await?;
        Ok(())
    }

    /// Create alert
    pub async fn create_alert(&self, alert: &AlertCreate) -> Result<()> {
        self.client
            .post(&format!("{}/rest/v1/alerts", SUPABASE_URL))
            .header("apikey", &self.anon_key)
            .header("Authorization", self.auth_header())
            .header("Content-Type", "application/json")
            .json(alert)
            .send()
            .await?;
        Ok(())
    }

    /// Update daily screen time summary
    pub async fn update_screen_time_daily(&self, summary: &ScreenTimeDailySummary) -> Result<()> {
        // Upsert - update if exists, insert if not
        self.client
            .post(&format!("{}/rest/v1/screen_time_daily", SUPABASE_URL))
            .header("apikey", &self.anon_key)
            .header("Authorization", self.auth_header())
            .header("Content-Type", "application/json")
            .header("Prefer", "resolution=merge-duplicates")
            .json(summary)
            .send()
            .await?;
        Ok(())
    }

    /// Check for pending device commands
    pub async fn get_pending_commands(&self, device_id: &str) -> Result<Vec<DeviceCommand>> {
        let resp = self.client
            .get(&format!(
                "{}/rest/v1/device_commands?device_id=eq.{}&acknowledged_at=is.null&order=issued_at.asc",
                SUPABASE_URL, device_id
            ))
            .header("apikey", &self.anon_key)
            .header("Authorization", self.auth_header())
            .send()
            .await?;

        let commands: Vec<DeviceCommand> = resp.json().await?;
        Ok(commands)
    }

    /// Acknowledge a device command
    pub async fn acknowledge_command(&self, command_id: &str) -> Result<()> {
        self.client
            .patch(&format!(
                "{}/rest/v1/device_commands?id=eq.{}",
                SUPABASE_URL, command_id
            ))
            .header("apikey", &self.anon_key)
            .header("Authorization", self.auth_header())
            .header("Content-Type", "application/json")
            .json(&serde_json::json!({
                "acknowledged_at": Utc::now().to_rfc3339()
            }))
            .send()
            .await?;
        Ok(())
    }

    /// Mark command as executed
    pub async fn complete_command(&self, command_id: &str, result: serde_json::Value) -> Result<()> {
        self.client
            .patch(&format!(
                "{}/rest/v1/device_commands?id=eq.{}",
                SUPABASE_URL, command_id
            ))
            .header("apikey", &self.anon_key)
            .header("Authorization", self.auth_header())
            .header("Content-Type", "application/json")
            .json(&serde_json::json!({
                "executed_at": Utc::now().to_rfc3339(),
                "result": result
            }))
            .send()
            .await?;
        Ok(())
    }
}

// ============ Data Types ============

#[derive(Debug, Serialize)]
pub struct DeviceRegistration {
    pub hardware_id: String,
    pub name: Option<String>,
    pub device_type: String,
    pub os_version: String,
    pub daemon_version: String,
}

#[derive(Debug, Deserialize)]
pub struct DeviceRecord {
    pub id: String,
    pub hardware_id: String,
    pub activation_code: Option<String>,
    pub status: String,
    pub family_id: Option<String>,
    pub child_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ActivationStatus {
    pub id: String,
    pub status: String,
    pub activated_at: Option<DateTime<Utc>>,
    pub child_id: Option<String>,
    pub family_id: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ChildPolicies {
    pub child_id: String,
    pub screen_time: Option<ScreenTimePolicy>,
    pub dns_profile: Option<DnsProfile>,
    pub app_policies: Vec<AppPolicy>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ScreenTimePolicy {
    pub id: String,
    pub child_id: String,
    pub weekday_limit_mins: Option<i32>,
    pub weekend_limit_mins: Option<i32>,
    pub earliest_start: Option<String>,
    pub latest_end: Option<String>,
    pub bedtime_enabled: Option<bool>,
    pub bedtime_time: Option<String>,
    pub bedtime_grace_mins: Option<i32>,
    pub break_reminder_enabled: Option<bool>,
    pub break_after_mins: Option<i32>,
    pub break_duration_mins: Option<i32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DnsProfile {
    pub id: String,
    pub child_id: String,
    pub filter_level: Option<String>,
    pub block_adult: Option<bool>,
    pub block_gambling: Option<bool>,
    pub block_social_media: Option<bool>,
    pub block_gaming: Option<bool>,
    pub block_streaming: Option<bool>,
    pub blocked_domains: Option<Vec<String>>,
    pub allowed_domains: Option<Vec<String>>,
    pub enforce_safe_search: Option<bool>,
    pub enforce_youtube_restricted: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AppPolicy {
    pub id: String,
    pub child_id: String,
    pub app_id: String,
    pub app_name: Option<String>,
    pub policy: Option<String>,
    pub daily_limit_mins: Option<i32>,
}

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

#[derive(Debug, Serialize)]
pub struct AppSession {
    pub device_id: String,
    pub child_id: Option<String>,
    pub app_id: String,
    pub app_name: Option<String>,
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
    pub duration_secs: Option<i32>,
    pub session_date: String,
}

#[derive(Debug, Serialize)]
pub struct UrlLog {
    pub device_id: String,
    pub child_id: Option<String>,
    pub url: String,
    pub domain: String,
    pub title: Option<String>,
    pub duration_secs: Option<i32>,
    pub category: Option<String>,
    pub risk_score: Option<f32>,
    pub flagged: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct AlertCreate {
    pub family_id: String,
    pub child_id: Option<String>,
    pub device_id: Option<String>,
    pub alert_type: String,
    pub severity: String,
    pub title: String,
    pub description: Option<String>,
    pub evidence: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
pub struct ScreenTimeDailySummary {
    pub child_id: String,
    pub device_id: Option<String>,
    pub date: String,
    pub total_mins: i32,
    pub gaming_mins: i32,
    pub education_mins: i32,
    pub entertainment_mins: i32,
    pub social_mins: i32,
    pub productivity_mins: i32,
    pub other_mins: i32,
    pub top_apps: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct DeviceCommand {
    pub id: String,
    pub device_id: String,
    pub command: String,
    pub payload: Option<serde_json::Value>,
    pub issued_at: DateTime<Utc>,
}
