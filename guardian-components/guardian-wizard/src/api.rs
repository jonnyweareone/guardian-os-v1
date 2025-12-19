//! API client for Guardian Wizard
//!
//! Communicates with Guardian Supabase backend for:
//! - Parent authentication
//! - Family/children data fetching
//! - Device registration and activation

use anyhow::{Result, Context};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::info;

#[allow(unused_imports)]
use tracing::debug;

const SUPABASE_URL: &str = "https://gkyspvcafyttfhyjryyk.supabase.co";
const SUPABASE_ANON_KEY: &str = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJzdXBhYmFzZSIsInJlZiI6ImdreXNwdmNhZnl0dGZoeWpyeXlrIiwicm9sZSI6ImFub24iLCJpYXQiOjE3NjYxMDIzMzQsImV4cCI6MjA4MTY3ODMzNH0.Ns5N9Y9uZgWqdhnYiX5IrubOO-Xopl2urBDR1AVD7FI";

#[derive(Clone)]
pub struct GuardianApi {
    client: Client,
    access_token: Option<String>,
}

impl GuardianApi {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            access_token: None,
        }
    }
    
    pub fn set_access_token(&mut self, token: String) {
        self.access_token = Some(token);
    }
    
    fn auth_header(&self) -> String {
        match &self.access_token {
            Some(token) => format!("Bearer {}", token),
            None => format!("Bearer {}", SUPABASE_ANON_KEY),
        }
    }
    
    /// Register this device with Guardian backend
    pub async fn register_device(&self) -> Result<DeviceInfo> {
        let hardware_id = get_hardware_id();
        let hostname = hostname::get()
            .map(|h| h.to_string_lossy().to_string())
            .unwrap_or_else(|_| "unknown".to_string());
        
        let device_type = detect_device_type();
        let os_version = get_os_version();
        
        info!("Registering device: {} ({})", hostname, hardware_id);
        
        let registration = DeviceRegistration {
            hardware_id: hardware_id.clone(),
            name: Some(hostname),
            device_type,
            os_version,
            daemon_version: env!("CARGO_PKG_VERSION").to_string(),
        };
        
        let resp = self.client
            .post(&format!("{}/rest/v1/devices", SUPABASE_URL))
            .header("apikey", SUPABASE_ANON_KEY)
            .header("Authorization", self.auth_header())
            .header("Content-Type", "application/json")
            .header("Prefer", "return=representation")
            .json(&registration)
            .send()
            .await
            .context("Failed to register device")?;
        
        if !resp.status().is_success() {
            let status = resp.status();
            let error_text = resp.text().await.unwrap_or_default();
            anyhow::bail!("Device registration failed ({}): {}", status, error_text);
        }
        
        let records: Vec<DeviceRecord> = resp.json().await?;
        let record = records.into_iter().next()
            .ok_or_else(|| anyhow::anyhow!("No device record returned"))?;
        
        Ok(DeviceInfo {
            device_id: record.id,
            hardware_id,
            activation_code: record.activation_code,
        })
    }
    
    /// Check if device has been activated by parent
    pub async fn check_activation(&self, device_id: &str) -> Result<ActivationStatus> {
        let resp = self.client
            .get(&format!(
                "{}/rest/v1/devices?id=eq.{}&select=id,status,activated_at,child_id,family_id",
                SUPABASE_URL, device_id
            ))
            .header("apikey", SUPABASE_ANON_KEY)
            .header("Authorization", self.auth_header())
            .send()
            .await?;
        
        let records: Vec<DeviceStatusRecord> = resp.json().await?;
        let record = records.into_iter().next()
            .ok_or_else(|| anyhow::anyhow!("Device not found"))?;
        
        Ok(ActivationStatus {
            activated: record.status == "active" && record.activated_at.is_some(),
            family_id: record.family_id,
            child_id: record.child_id,
        })
    }
    
    /// Login with email/password
    pub async fn login(&self, email: &str, password: &str) -> Result<AuthResult> {
        let resp = self.client
            .post(&format!("{}/auth/v1/token?grant_type=password", SUPABASE_URL))
            .header("apikey", SUPABASE_ANON_KEY)
            .header("Content-Type", "application/json")
            .json(&serde_json::json!({
                "email": email,
                "password": password,
            }))
            .send()
            .await?;
        
        if !resp.status().is_success() {
            let error: serde_json::Value = resp.json().await?;
            let message = error.get("error_description")
                .or_else(|| error.get("msg"))
                .and_then(|v| v.as_str())
                .unwrap_or("Login failed");
            anyhow::bail!("{}", message);
        }
        
        let auth: AuthResponse = resp.json().await?;
        
        Ok(AuthResult {
            access_token: auth.access_token,
            refresh_token: auth.refresh_token,
            user_id: auth.user.id,
        })
    }
    
    /// Sign up new account
    pub async fn signup(&self, email: &str, password: &str, full_name: &str) -> Result<AuthResult> {
        let resp = self.client
            .post(&format!("{}/auth/v1/signup", SUPABASE_URL))
            .header("apikey", SUPABASE_ANON_KEY)
            .header("Content-Type", "application/json")
            .json(&serde_json::json!({
                "email": email,
                "password": password,
                "data": {
                    "full_name": full_name,
                }
            }))
            .send()
            .await?;
        
        if !resp.status().is_success() {
            let error: serde_json::Value = resp.json().await?;
            let message = error.get("msg")
                .and_then(|v| v.as_str())
                .unwrap_or("Signup failed");
            anyhow::bail!("{}", message);
        }
        
        let auth: AuthResponse = resp.json().await?;
        
        Ok(AuthResult {
            access_token: auth.access_token,
            refresh_token: auth.refresh_token,
            user_id: auth.user.id,
        })
    }
    
    /// Fetch the family for the currently authenticated user
    pub async fn fetch_family_for_user(&self, user_id: &str) -> Result<FamilyInfo> {
        // Get family membership for this user
        let resp = self.client
            .get(&format!(
                "{}/rest/v1/family_members?select=family_id,families(id,name,invite_code)&profile_id=eq.{}",
                SUPABASE_URL, user_id
            ))
            .header("apikey", SUPABASE_ANON_KEY)
            .header("Authorization", self.auth_header())
            .send()
            .await
            .context("Failed to fetch family membership")?;
        
        if !resp.status().is_success() {
            anyhow::bail!("Failed to fetch family: {}", resp.status());
        }
        
        let records: Vec<FamilyMemberRecord> = resp.json().await?;
        let record = records.into_iter().next()
            .ok_or_else(|| anyhow::anyhow!("No family found for this user"))?;
        
        let family = record.families
            .ok_or_else(|| anyhow::anyhow!("Family data not returned"))?;
        
        Ok(FamilyInfo {
            id: family.id,
            name: family.name,
            invite_code: family.invite_code,
        })
    }
    
    /// Fetch children for a specific family
    pub async fn fetch_children(&self, family_id: &str) -> Result<Vec<ChildInfo>> {
        let resp = self.client
            .get(&format!(
                "{}/rest/v1/children?select=id,name,date_of_birth,avatar_url&family_id=eq.{}&order=name.asc",
                SUPABASE_URL, family_id
            ))
            .header("apikey", SUPABASE_ANON_KEY)
            .header("Authorization", self.auth_header())
            .send()
            .await
            .context("Failed to fetch children")?;
        
        if !resp.status().is_success() {
            anyhow::bail!("Failed to fetch children: {}", resp.status());
        }
        
        let records: Vec<ChildRecord> = resp.json().await?;
        
        Ok(records.into_iter().map(|r| ChildInfo {
            id: r.id,
            name: r.name,
            date_of_birth: r.date_of_birth,
            avatar_url: r.avatar_url,
        }).collect())
    }
    
    /// Create a new family
    pub async fn create_family(&self, name: &str) -> Result<FamilyInfo> {
        let resp = self.client
            .post(&format!("{}/rest/v1/families", SUPABASE_URL))
            .header("apikey", SUPABASE_ANON_KEY)
            .header("Authorization", self.auth_header())
            .header("Content-Type", "application/json")
            .header("Prefer", "return=representation")
            .json(&serde_json::json!({
                "name": name,
            }))
            .send()
            .await?;
        
        if !resp.status().is_success() {
            anyhow::bail!("Failed to create family");
        }
        
        let records: Vec<FamilyRecord> = resp.json().await?;
        let record = records.into_iter().next()
            .ok_or_else(|| anyhow::anyhow!("No family record returned"))?;
        
        Ok(FamilyInfo {
            id: record.id,
            name: record.name,
            invite_code: record.invite_code,
        })
    }
    
    /// Join existing family by code
    pub async fn join_family(&self, invite_code: &str) -> Result<FamilyInfo> {
        // First, find family by invite code
        let resp = self.client
            .get(&format!(
                "{}/rest/v1/families?invite_code=eq.{}",
                SUPABASE_URL, invite_code
            ))
            .header("apikey", SUPABASE_ANON_KEY)
            .header("Authorization", self.auth_header())
            .send()
            .await?;
        
        let records: Vec<FamilyRecord> = resp.json().await?;
        let family = records.into_iter().next()
            .ok_or_else(|| anyhow::anyhow!("Family not found with that code"))?;
        
        Ok(FamilyInfo {
            id: family.id,
            name: family.name,
            invite_code: family.invite_code,
        })
    }
    
    /// Assign device to a child and activate it
    pub async fn activate_device_for_child(
        &self, 
        device_id: &str, 
        child_id: &str, 
        family_id: &str
    ) -> Result<()> {
        let resp = self.client
            .patch(&format!(
                "{}/rest/v1/devices?id=eq.{}",
                SUPABASE_URL, device_id
            ))
            .header("apikey", SUPABASE_ANON_KEY)
            .header("Authorization", self.auth_header())
            .header("Content-Type", "application/json")
            .json(&serde_json::json!({
                "child_id": child_id,
                "family_id": family_id,
                "status": "active",
                "activated_at": chrono::Utc::now().to_rfc3339(),
            }))
            .send()
            .await
            .context("Failed to activate device")?;
        
        if !resp.status().is_success() {
            anyhow::bail!("Failed to activate device: {}", resp.status());
        }
        
        Ok(())
    }
}

// ============ Helper Functions ============

fn get_hardware_id() -> String {
    // Try machine-id
    if let Ok(id) = std::fs::read_to_string("/etc/machine-id") {
        return id.trim().to_string();
    }
    
    // Try DMI product UUID
    if let Ok(id) = std::fs::read_to_string("/sys/class/dmi/id/product_uuid") {
        return id.trim().to_string();
    }
    
    // Generate fallback
    uuid::Uuid::new_v4().to_string()
}

fn detect_device_type() -> String {
    // Check for laptop battery
    if std::path::Path::new("/sys/class/power_supply/BAT0").exists() {
        return "laptop".to_string();
    }
    
    // Default to desktop
    "desktop".to_string()
}

fn get_os_version() -> String {
    if let Ok(content) = std::fs::read_to_string("/etc/os-release") {
        for line in content.lines() {
            if line.starts_with("PRETTY_NAME=") {
                return line
                    .trim_start_matches("PRETTY_NAME=")
                    .trim_matches('"')
                    .to_string();
            }
        }
    }
    "Guardian OS".to_string()
}

// ============ Data Types ============

#[derive(Serialize)]
struct DeviceRegistration {
    hardware_id: String,
    name: Option<String>,
    device_type: String,
    os_version: String,
    daemon_version: String,
}

#[derive(Deserialize)]
struct DeviceRecord {
    id: String,
    activation_code: Option<String>,
}

#[derive(Deserialize)]
struct DeviceStatusRecord {
    #[allow(dead_code)]
    id: String,
    status: String,
    activated_at: Option<String>,
    family_id: Option<String>,
    child_id: Option<String>,
}

#[derive(Deserialize)]
struct AuthResponse {
    access_token: String,
    refresh_token: String,
    user: AuthUser,
}

#[derive(Deserialize)]
struct AuthUser {
    id: String,
}

#[derive(Deserialize)]
struct FamilyRecord {
    id: String,
    name: String,
    invite_code: Option<String>,
}

#[derive(Deserialize)]
struct FamilyMemberRecord {
    #[allow(dead_code)]
    family_id: String,
    families: Option<FamilyRecord>,
}

#[derive(Deserialize)]
struct ChildRecord {
    id: String,
    name: String,
    date_of_birth: Option<String>,
    avatar_url: Option<String>,
}

// ============ Public Types ============

#[derive(Debug, Clone)]
pub struct DeviceInfo {
    pub device_id: String,
    pub hardware_id: String,
    pub activation_code: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ActivationStatus {
    pub activated: bool,
    pub family_id: Option<String>,
    pub child_id: Option<String>,
}

#[derive(Debug, Clone)]
pub struct AuthResult {
    pub access_token: String,
    pub refresh_token: String,
    pub user_id: String,
}

#[derive(Debug, Clone)]
pub struct FamilyInfo {
    pub id: String,
    pub name: String,
    pub invite_code: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ChildInfo {
    pub id: String,
    pub name: String,
    pub date_of_birth: Option<String>,
    pub avatar_url: Option<String>,
}
