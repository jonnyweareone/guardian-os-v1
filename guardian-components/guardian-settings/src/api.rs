//! API client for Guardian Settings

use anyhow::{Result, Context};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

const SUPABASE_URL: &str = "https://gkyspvcafyttfhyjryyk.supabase.co";
const SUPABASE_ANON_KEY: &str = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJzdXBhYmFzZSIsInJlZiI6ImdreXNwdmNhZnl0dGZoeWpyeXlrIiwicm9sZSI6ImFub24iLCJpYXQiOjE3NjYxMDIzMzQsImV4cCI6MjA4MTY3ODMzNH0.Ns5N9Y9uZgWqdhnYiX5IrubOO-Xopl2urBDR1AVD7FI";

#[derive(Clone)]
pub struct GuardianApi {
    client: Client,
    access_token: Option<String>,
}

impl GuardianApi {
    pub fn new() -> Self {
        // Try to load token from keyring
        let access_token = keyring::Entry::new("guardian-settings", "access_token")
            .ok()
            .and_then(|e| e.get_password().ok());
        
        Self {
            client: Client::new(),
            access_token,
        }
    }
    
    pub fn set_access_token(&mut self, token: &str) -> Result<()> {
        self.access_token = Some(token.to_string());
        
        // Save to keyring
        if let Ok(entry) = keyring::Entry::new("guardian-settings", "access_token") {
            entry.set_password(token)?;
        }
        
        Ok(())
    }
    
    fn auth_header(&self) -> String {
        match &self.access_token {
            Some(token) => format!("Bearer {}", token),
            None => format!("Bearer {}", SUPABASE_ANON_KEY),
        }
    }
    
    /// Get all children in the family
    pub async fn get_children(&self) -> Result<Vec<Child>> {
        let resp = self.client
            .get(&format!(
                "{}/rest/v1/children?select=*&order=name.asc",
                SUPABASE_URL
            ))
            .header("apikey", SUPABASE_ANON_KEY)
            .header("Authorization", self.auth_header())
            .send()
            .await
            .context("Failed to fetch children")?;
        
        if !resp.status().is_success() {
            let error = resp.text().await.unwrap_or_default();
            anyhow::bail!("Failed to fetch children: {}", error);
        }
        
        let children: Vec<Child> = resp.json().await?;
        Ok(children)
    }
    
    /// Get all devices in the family
    pub async fn get_devices(&self) -> Result<Vec<Device>> {
        let resp = self.client
            .get(&format!(
                "{}/rest/v1/devices?select=*,children(name)&order=name.asc",
                SUPABASE_URL
            ))
            .header("apikey", SUPABASE_ANON_KEY)
            .header("Authorization", self.auth_header())
            .send()
            .await?;
        
        if !resp.status().is_success() {
            let error = resp.text().await.unwrap_or_default();
            anyhow::bail!("Failed to fetch devices: {}", error);
        }
        
        let devices: Vec<Device> = resp.json().await?;
        Ok(devices)
    }
    
    /// Get alerts for the family
    pub async fn get_alerts(&self) -> Result<Vec<Alert>> {
        let resp = self.client
            .get(&format!(
                "{}/rest/v1/alerts?select=*,children(name)&status=neq.dismissed&order=created_at.desc&limit=50",
                SUPABASE_URL
            ))
            .header("apikey", SUPABASE_ANON_KEY)
            .header("Authorization", self.auth_header())
            .send()
            .await?;
        
        if !resp.status().is_success() {
            let error = resp.text().await.unwrap_or_default();
            anyhow::bail!("Failed to fetch alerts: {}", error);
        }
        
        let alerts: Vec<Alert> = resp.json().await?;
        Ok(alerts)
    }
    
    /// Get screen time policy for a child
    pub async fn get_screen_time_policy(&self, child_id: &str) -> Result<ScreenTimePolicy> {
        let resp = self.client
            .get(&format!(
                "{}/rest/v1/screen_time_policies?child_id=eq.{}",
                SUPABASE_URL, child_id
            ))
            .header("apikey", SUPABASE_ANON_KEY)
            .header("Authorization", self.auth_header())
            .send()
            .await?;
        
        let policies: Vec<ScreenTimePolicy> = resp.json().await?;
        policies.into_iter().next()
            .ok_or_else(|| anyhow::anyhow!("No screen time policy found"))
    }
    
    /// Update screen time policy
    pub async fn update_screen_time_policy(&self, policy: &ScreenTimePolicy) -> Result<()> {
        self.client
            .patch(&format!(
                "{}/rest/v1/screen_time_policies?id=eq.{}",
                SUPABASE_URL, policy.id
            ))
            .header("apikey", SUPABASE_ANON_KEY)
            .header("Authorization", self.auth_header())
            .header("Content-Type", "application/json")
            .json(policy)
            .send()
            .await?;
        Ok(())
    }
    
    /// Get DNS profile for a child
    pub async fn get_dns_profile(&self, child_id: &str) -> Result<DnsProfile> {
        let resp = self.client
            .get(&format!(
                "{}/rest/v1/dns_profiles?child_id=eq.{}",
                SUPABASE_URL, child_id
            ))
            .header("apikey", SUPABASE_ANON_KEY)
            .header("Authorization", self.auth_header())
            .send()
            .await?;
        
        let profiles: Vec<DnsProfile> = resp.json().await?;
        profiles.into_iter().next()
            .ok_or_else(|| anyhow::anyhow!("No DNS profile found"))
    }
    
    /// Update DNS profile
    pub async fn update_dns_profile(&self, profile: &DnsProfile) -> Result<()> {
        self.client
            .patch(&format!(
                "{}/rest/v1/dns_profiles?id=eq.{}",
                SUPABASE_URL, profile.id
            ))
            .header("apikey", SUPABASE_ANON_KEY)
            .header("Authorization", self.auth_header())
            .header("Content-Type", "application/json")
            .json(profile)
            .send()
            .await?;
        Ok(())
    }
    
    /// Send a command to a device
    pub async fn send_device_command(&self, device_id: &str, command: &str) -> Result<()> {
        self.client
            .post(&format!("{}/rest/v1/device_commands", SUPABASE_URL))
            .header("apikey", SUPABASE_ANON_KEY)
            .header("Authorization", self.auth_header())
            .header("Content-Type", "application/json")
            .json(&serde_json::json!({
                "device_id": device_id,
                "command": command,
            }))
            .send()
            .await?;
        Ok(())
    }
    
    /// Dismiss an alert
    pub async fn dismiss_alert(&self, alert_id: &str) -> Result<()> {
        self.client
            .patch(&format!(
                "{}/rest/v1/alerts?id=eq.{}",
                SUPABASE_URL, alert_id
            ))
            .header("apikey", SUPABASE_ANON_KEY)
            .header("Authorization", self.auth_header())
            .header("Content-Type", "application/json")
            .json(&serde_json::json!({
                "status": "dismissed"
            }))
            .send()
            .await?;
        Ok(())
    }
    
    /// Get screen time usage for today
    pub async fn get_screen_time_today(&self, child_id: &str) -> Result<Option<ScreenTimeDaily>> {
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        
        let resp = self.client
            .get(&format!(
                "{}/rest/v1/screen_time_daily?child_id=eq.{}&date=eq.{}",
                SUPABASE_URL, child_id, today
            ))
            .header("apikey", SUPABASE_ANON_KEY)
            .header("Authorization", self.auth_header())
            .send()
            .await?;
        
        let records: Vec<ScreenTimeDaily> = resp.json().await?;
        Ok(records.into_iter().next())
    }
    
    /// Get recent app sessions for a child
    pub async fn get_recent_sessions(&self, child_id: &str, limit: usize) -> Result<Vec<AppSession>> {
        let resp = self.client
            .get(&format!(
                "{}/rest/v1/app_sessions?child_id=eq.{}&order=started_at.desc&limit={}",
                SUPABASE_URL, child_id, limit
            ))
            .header("apikey", SUPABASE_ANON_KEY)
            .header("Authorization", self.auth_header())
            .send()
            .await?;
        
        let sessions: Vec<AppSession> = resp.json().await?;
        Ok(sessions)
    }
}

// ============ Data Types ============

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Child {
    pub id: String,
    pub family_id: String,
    pub name: String,
    pub date_of_birth: Option<String>,
    pub avatar_url: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
}

impl Child {
    pub fn age(&self) -> Option<u32> {
        self.date_of_birth.as_ref().and_then(|dob| {
            let dob = chrono::NaiveDate::parse_from_str(dob, "%Y-%m-%d").ok()?;
            let today = chrono::Local::now().date_naive();
            let age = today.years_since(dob)?;
            Some(age)
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Device {
    pub id: String,
    pub family_id: Option<String>,
    pub child_id: Option<String>,
    pub hardware_id: String,
    pub name: Option<String>,
    pub device_type: Option<String>,
    pub status: Option<String>,
    pub last_seen_at: Option<DateTime<Utc>>,
    pub os_version: Option<String>,
    pub daemon_version: Option<String>,
    pub activation_code: Option<String>,
    // Nested child data
    pub children: Option<ChildRef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChildRef {
    pub name: String,
}

impl Device {
    pub fn is_online(&self) -> bool {
        if let Some(last_seen) = self.last_seen_at {
            let now = Utc::now();
            (now - last_seen).num_minutes() < 5
        } else {
            false
        }
    }
    
    pub fn child_name(&self) -> Option<&str> {
        self.children.as_ref().map(|c| c.name.as_str())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    pub id: String,
    pub family_id: String,
    pub child_id: Option<String>,
    pub device_id: Option<String>,
    pub alert_type: String,
    pub severity: Option<String>,
    pub title: String,
    pub description: Option<String>,
    pub status: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    // Nested
    pub children: Option<ChildRef>,
}

impl Alert {
    pub fn severity_color(&self) -> (f32, f32, f32) {
        match self.severity.as_deref() {
            Some("critical") => (0.9, 0.1, 0.1),
            Some("high") => (0.9, 0.4, 0.1),
            Some("medium") => (0.9, 0.7, 0.1),
            _ => (0.5, 0.5, 0.5),
        }
    }
    
    pub fn child_name(&self) -> Option<&str> {
        self.children.as_ref().map(|c| c.name.as_str())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScreenTimeDaily {
    pub id: String,
    pub child_id: String,
    pub date: String,
    pub total_mins: Option<i32>,
    pub gaming_mins: Option<i32>,
    pub education_mins: Option<i32>,
    pub entertainment_mins: Option<i32>,
    pub social_mins: Option<i32>,
    pub productivity_mins: Option<i32>,
    pub other_mins: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSession {
    pub id: String,
    pub device_id: String,
    pub child_id: Option<String>,
    pub app_id: String,
    pub app_name: Option<String>,
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
    pub duration_secs: Option<i32>,
}
