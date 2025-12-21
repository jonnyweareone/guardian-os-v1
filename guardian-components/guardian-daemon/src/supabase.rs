//! Supabase client for Guardian OS v1.1.0
//! 
//! Handles device registration, activation, and data sync with Supabase backend
//! Uses Edge Functions for device lifecycle management

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
    device_code: Option<String>,
}

impl SupabaseClient {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            device_code: None,
        }
    }

    pub fn with_device_code(device_code: &str) -> Self {
        Self {
            client: Client::new(),
            device_code: Some(device_code.to_string()),
        }
    }

    pub fn set_device_code(&mut self, code: &str) {
        self.device_code = Some(code.to_string());
    }

    // ============ Edge Function Calls ============

    /// Register a new device via Edge Function
    /// Returns device code for activation
    pub async fn register_device(&self, registration: &DeviceRegistration) -> Result<DeviceRegistrationResponse> {
        let resp = self.client
            .post(&format!("{}/functions/v1/device-register", SUPABASE_URL))
            .header("Authorization", format!("Bearer {}", SUPABASE_ANON_KEY))
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

    /// Check device activation status via Edge Function
    pub async fn check_activation(&self, device_code: &str) -> Result<DeviceStatusResponse> {
        let resp = self.client
            .get(&format!(
                "{}/functions/v1/device-status?device_code={}",
                SUPABASE_URL, device_code
            ))
            .header("Authorization", format!("Bearer {}", SUPABASE_ANON_KEY))
            .send()
            .await
            .context("Failed to check activation status")?;

        if !resp.status().is_success() {
            let error_text = resp.text().await.unwrap_or_default();
            anyhow::bail!("Activation check failed: {}", error_text);
        }

        let response: DeviceStatusResponse = resp.json().await
            .context("Failed to parse status response")?;
        
        Ok(response)
    }

    /// Sync data to cloud via Edge Function
    pub async fn sync_data(&self, payload: &SyncPayload) -> Result<SyncResponse> {
        let device_code = self.device_code.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Device code not set"))?;

        let resp = self.client
            .post(&format!("{}/functions/v1/device-sync", SUPABASE_URL))
            .header("Authorization", format!("Bearer {}", SUPABASE_ANON_KEY))
            .header("x-device-id", device_code)
            .header("Content-Type", "application/json")
            .json(payload)
            .send()
            .await
            .context("Failed to sync data")?;

        if resp.status().as_u16() == 403 {
            warn!("Device not activated, sync rejected");
            anyhow::bail!("Device not activated");
        }

        if !resp.status().is_success() {
            let error_text = resp.text().await.unwrap_or_default();
            anyhow::bail!("Sync failed: {}", error_text);
        }

        let response: SyncResponse = resp.json().await
            .context("Failed to parse sync response")?;
        
        debug!("Sync complete: {:?}", response.results);
        Ok(response)
    }

    // ============ Direct REST API Calls (for simple operations) ============

    /// Update device last_seen (heartbeat)
    pub async fn heartbeat(&self) -> Result<()> {
        let device_code = self.device_code.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Device code not set"))?;

        self.client
            .patch(&format!(
                "{}/rest/v1/devices?device_code=eq.{}",
                SUPABASE_URL, device_code
            ))
            .header("apikey", SUPABASE_ANON_KEY)
            .header("Authorization", format!("Bearer {}", SUPABASE_ANON_KEY))
            .header("Content-Type", "application/json")
            .json(&serde_json::json!({
                "last_seen": Utc::now().to_rfc3339()
            }))
            .send()
            .await?;

        Ok(())
    }

    /// Mark a remote command as completed
    pub async fn complete_command(&self, command_id: &str, result: serde_json::Value) -> Result<()> {
        let device_code = self.device_code.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Device code not set"))?;

        self.client
            .patch(&format!(
                "{}/rest/v1/device_commands?id=eq.{}",
                SUPABASE_URL, command_id
            ))
            .header("apikey", SUPABASE_ANON_KEY)
            .header("Authorization", format!("Bearer {}", SUPABASE_ANON_KEY))
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

    /// Get approved/blocked contacts for local cache
    pub async fn get_contacts(&self, family_id: &str) -> Result<Vec<ContactInfo>> {
        let resp = self.client
            .get(&format!(
                "{}/rest/v1/contacts?family_id=eq.{}&or=(is_approved.eq.true,is_blocked.eq.true)&select=contact_hash,platform,is_approved,is_blocked",
                SUPABASE_URL, family_id
            ))
            .header("apikey", SUPABASE_ANON_KEY)
            .header("Authorization", format!("Bearer {}", SUPABASE_ANON_KEY))
            .send()
            .await?;

        let contacts: Vec<ContactInfo> = resp.json().await?;
        Ok(contacts)
    }
}

// ============ Request/Response Types ============

#[derive(Debug, Serialize)]
pub struct DeviceRegistration {
    pub os_version: String,
    pub device_type: Option<String>,
    pub hardware_info: Option<HardwareInfo>,
}

#[derive(Debug, Serialize)]
pub struct HardwareInfo {
    pub cpu: Option<String>,
    pub ram_gb: Option<u32>,
    pub gpu: Option<String>,
    pub hostname: Option<String>,
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
    pub device_code: Option<String>,
    #[serde(default)]
    pub message: Option<String>,
    #[serde(default)]
    pub device: Option<DeviceInfo>,
    #[serde(default)]
    pub family: Option<FamilyInfo>,
    #[serde(default)]
    pub children: Vec<ChildInfo>,
    #[serde(default)]
    pub assigned_child: Option<ChildInfo>,
    #[serde(default)]
    pub contacts: Vec<ContactInfo>,
    #[serde(default)]
    pub sync_endpoint: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DeviceInfo {
    pub id: String,
    pub code: String,
    pub name: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct FamilyInfo {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ChildInfo {
    pub id: String,
    pub name: String,
    pub age_tier: String,
    pub trust_score: Option<f64>,
    #[serde(default)]
    pub date_of_birth: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ContactInfo {
    pub contact_hash: String,
    pub platform: String,
    pub is_approved: bool,
    pub is_blocked: bool,
}

// ============ Sync Payload Types ============

#[derive(Debug, Serialize)]
pub struct SyncPayload {
    pub device_code: String,
    pub os_version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub topic_summaries: Option<Vec<TopicSummarySync>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub browsing_summary: Option<BrowsingSummarySync>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alerts: Option<Vec<AlertSync>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contacts: Option<Vec<ContactSync>>,
}

#[derive(Debug, Serialize)]
pub struct TopicSummarySync {
    pub child_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contact_hash: Option<String>,
    pub period_start: String,
    pub period_end: String,
    pub topics: Vec<TopicEntry>,
    pub message_count: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ai_summary: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub risk_events: Option<Vec<RiskEvent>>,
}

#[derive(Debug, Serialize)]
pub struct TopicEntry {
    pub topic: String,
    pub percentage: f32,
    pub description: String,
    pub risk_level: String, // "safe", "caution", "high", "critical"
}

#[derive(Debug, Serialize)]
pub struct RiskEvent {
    pub timestamp: String,
    pub event_type: String,
    pub details: String,
    pub severity: f32,
}

#[derive(Debug, Serialize)]
pub struct BrowsingSummarySync {
    pub child_id: String,
    pub date: String, // YYYY-MM-DD
    pub top_domains: Vec<DomainEntry>,
    pub categories: Vec<CategoryEntry>,
    pub blocked_count: u32,
    pub vpn_attempts: u32,
    pub total_queries: u32,
}

#[derive(Debug, Serialize)]
pub struct DomainEntry {
    pub domain: String,
    pub count: u32,
    pub category: String,
}

#[derive(Debug, Serialize)]
pub struct CategoryEntry {
    pub category: String,
    pub percentage: f32,
}

#[derive(Debug, Serialize)]
pub struct AlertSync {
    pub child_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contact_hash: Option<String>,
    pub alert_tier: String, // "digest", "note", "elevated", "high", "critical", "emergency"
    pub trigger_type: String,
    pub risk_score: f32,
    pub summary: String,
    pub risk_factors: Vec<String>,
    pub recommended_action: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub replay_available: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub replay_hours: Option<u32>,
}

#[derive(Debug, Serialize)]
pub struct ContactSync {
    pub contact_hash: String,
    pub platform: String,
    pub risk_score: f32,
    pub inferred_tags: Vec<String>,
    pub child_interactions: Vec<ChildInteraction>,
}

#[derive(Debug, Serialize)]
pub struct ChildInteraction {
    pub child_id: String,
    pub message_count: u32,
    pub child_risk_score: f32,
}

#[derive(Debug, Deserialize)]
pub struct SyncResponse {
    pub success: bool,
    pub results: SyncResults,
    pub timestamp: String,
}

#[derive(Debug, Deserialize)]
pub struct SyncResults {
    pub topic_summaries: u32,
    pub browsing_summary: bool,
    pub alerts: u32,
    pub contacts: u32,
    #[serde(default)]
    pub errors: Vec<String>,
}

impl Default for SupabaseClient {
    fn default() -> Self {
        Self::new()
    }
}
