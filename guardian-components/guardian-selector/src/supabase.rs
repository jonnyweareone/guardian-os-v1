//! Supabase client for Guardian Selector

use crate::Child;
use anyhow::Result;
use serde::{Deserialize, Serialize};

pub struct SupabaseClient {
    url: String,
    anon_key: String,
    client: reqwest::Client,
}

#[derive(Debug, Deserialize)]
struct ChildRow {
    id: String,
    name: String,
    slug: String,
    experience_mode: String,
    trust_mode: String,
    unlock_method: String,
    avatar_url: Option<String>,
}

#[derive(Debug, Serialize)]
struct LoginRequestCreate {
    action: String,
    device_id: String,
    child_slug: String,
    device_name: Option<String>,
}

#[derive(Debug, Deserialize)]
struct LoginRequestResponse {
    request_id: Option<String>,
    status: Option<String>,
    error: Option<String>,
}

impl SupabaseClient {
    pub fn new(url: &str, anon_key: &str) -> Self {
        Self {
            url: url.to_string(),
            anon_key: anon_key.to_string(),
            client: reqwest::Client::new(),
        }
    }

    /// Get all children for a family
    pub async fn get_children(&self, family_id: &str) -> Result<Vec<Child>> {
        let response = self.client
            .get(format!("{}/rest/v1/children", self.url))
            .header("apikey", &self.anon_key)
            .header("Authorization", format!("Bearer {}", self.anon_key))
            .query(&[
                ("family_id", format!("eq.{}", family_id)),
                ("select", "id,name,slug,experience_mode,trust_mode,unlock_method,avatar_url".to_string()),
            ])
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow::anyhow!("Failed to fetch children: {}", error_text));
        }

        let rows: Vec<ChildRow> = response.json().await?;
        
        Ok(rows.into_iter().map(|r| Child {
            id: r.id,
            name: r.name,
            slug: r.slug,
            experience_mode: r.experience_mode,
            trust_mode: r.trust_mode,
            unlock_method: r.unlock_method,
            avatar_url: r.avatar_url,
        }).collect())
    }

    /// Create a login request (ask_parent flow)
    pub async fn create_login_request(&self, device_id: &str, child_slug: &str) -> Result<String> {
        let response = self.client
            .post(format!("{}/functions/v1/login-request", self.url))
            .header("apikey", &self.anon_key)
            .json(&LoginRequestCreate {
                action: "create".to_string(),
                device_id: device_id.to_string(),
                child_slug: child_slug.to_string(),
                device_name: hostname::get().ok().map(|h| h.to_string_lossy().to_string()),
            })
            .send()
            .await?;

        let result: LoginRequestResponse = response.json().await?;

        if let Some(error) = result.error {
            return Err(anyhow::anyhow!("Login request failed: {}", error));
        }

        result.request_id.ok_or_else(|| anyhow::anyhow!("No request ID returned"))
    }

    /// Check status of a login request
    pub async fn check_login_request(&self, request_id: &str) -> Result<String> {
        let response = self.client
            .post(format!("{}/functions/v1/login-request", self.url))
            .header("apikey", &self.anon_key)
            .json(&serde_json::json!({
                "action": "check",
                "request_id": request_id
            }))
            .send()
            .await?;

        let result: serde_json::Value = response.json().await?;
        
        Ok(result["status"].as_str().unwrap_or("pending").to_string())
    }

    /// Notify parent that session started (for auto unlock)
    pub async fn notify_session_start(&self, device_id: &str, child_id: &str) -> Result<()> {
        // This would send a notification to parents
        // For now, just log it
        tracing::info!("Session started: device={}, child={}", device_id, child_id);
        Ok(())
    }

    /// Create a session record
    pub async fn create_session(&self, device_id: &str, child_id: &str, unlock_method: &str) -> Result<String> {
        let response = self.client
            .post(format!("{}/rest/v1/device_sessions", self.url))
            .header("apikey", &self.anon_key)
            .header("Authorization", format!("Bearer {}", self.anon_key))
            .header("Content-Type", "application/json")
            .header("Prefer", "return=representation")
            .json(&serde_json::json!({
                "device_id": device_id,
                "child_id": child_id,
                "unlock_method": unlock_method
            }))
            .send()
            .await?;

        if !response.status().is_success() {
            let error = response.text().await?;
            tracing::warn!("Failed to create session: {}", error);
            return Err(anyhow::anyhow!("Failed to create session"));
        }

        let result: Vec<serde_json::Value> = response.json().await?;
        Ok(result.first()
            .and_then(|r| r["id"].as_str())
            .unwrap_or("unknown")
            .to_string())
    }

    /// Verify PIN
    pub async fn verify_pin(&self, child_id: &str, pin: &str) -> Result<bool> {
        let response = self.client
            .post(format!("{}/rest/v1/rpc/verify_child_pin", self.url))
            .header("apikey", &self.anon_key)
            .header("Authorization", format!("Bearer {}", self.anon_key))
            .json(&serde_json::json!({
                "p_child_id": child_id,
                "p_pin": pin
            }))
            .send()
            .await?;

        let result: bool = response.json().await.unwrap_or(false);
        Ok(result)
    }
}
