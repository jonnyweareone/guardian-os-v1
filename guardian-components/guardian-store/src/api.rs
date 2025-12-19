//! Supabase API client for Guardian Store

use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::catalog::AppEntry;
use crate::ratings::AgeRating;

const SUPABASE_URL: &str = "https://gkyspvcafyttfhyjryyk.supabase.co";
const SUPABASE_ANON_KEY: &str = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJzdXBhYmFzZSIsInJlZiI6ImdreXNwdmNhZnl0dGZoeWpyeXlrIiwicm9sZSI6ImFub24iLCJpYXQiOjE3NjYxMDIzMzQsImV4cCI6MjA4MTY3ODMzNH0.Ns5N9Y9uZgWqdhnYiX5IrubOO-Xopl2urBDR1AVD7FI";

/// API client for Guardian Store
pub struct StoreApi {
    client: Client,
    access_token: Option<String>,
}

impl StoreApi {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            access_token: None,
        }
    }
    
    fn auth_header(&self) -> String {
        match &self.access_token {
            Some(token) => format!("Bearer {}", token),
            None => format!("Bearer {}", SUPABASE_ANON_KEY),
        }
    }
    
    /// Get Guardian-approved apps from catalog
    pub async fn get_approved_apps(&self) -> Result<Vec<AppCatalogEntry>> {
        let resp = self.client
            .get(&format!(
                "{}/rest/v1/app_catalog?guardian_approved=eq.true&order=name.asc",
                SUPABASE_URL
            ))
            .header("apikey", SUPABASE_ANON_KEY)
            .header("Authorization", self.auth_header())
            .send()
            .await?;
        
        let apps: Vec<AppCatalogEntry> = resp.json().await?;
        Ok(apps)
    }
    
    /// Get apps for a specific age rating
    pub async fn get_apps_for_rating(&self, max_rating: &AgeRating) -> Result<Vec<AppCatalogEntry>> {
        let max_age = max_rating.min_age();
        
        let resp = self.client
            .get(&format!(
                "{}/rest/v1/app_catalog?min_age=lte.{}&order=name.asc",
                SUPABASE_URL, max_age
            ))
            .header("apikey", SUPABASE_ANON_KEY)
            .header("Authorization", self.auth_header())
            .send()
            .await?;
        
        let apps: Vec<AppCatalogEntry> = resp.json().await?;
        Ok(apps)
    }
    
    /// Create an app request (child -> parent)
    pub async fn create_app_request(&self, app_id: &str, child_id: &str, family_id: &str) -> Result<()> {
        self.client
            .post(&format!("{}/rest/v1/app_requests", SUPABASE_URL))
            .header("apikey", SUPABASE_ANON_KEY)
            .header("Authorization", self.auth_header())
            .header("Content-Type", "application/json")
            .json(&serde_json::json!({
                "app_id": app_id,
                "child_id": child_id,
                "family_id": family_id,
                "status": "pending",
            }))
            .send()
            .await?;
        Ok(())
    }
    
    /// Get pending app requests for a family
    pub async fn get_pending_requests(&self, family_id: &str) -> Result<Vec<AppRequest>> {
        let resp = self.client
            .get(&format!(
                "{}/rest/v1/app_requests?family_id=eq.{}&status=eq.pending&order=created_at.desc",
                SUPABASE_URL, family_id
            ))
            .header("apikey", SUPABASE_ANON_KEY)
            .header("Authorization", self.auth_header())
            .send()
            .await?;
        
        let requests: Vec<AppRequest> = resp.json().await?;
        Ok(requests)
    }
    
    /// Approve or deny an app request
    pub async fn update_request_status(&self, request_id: &str, approved: bool) -> Result<()> {
        let status = if approved { "approved" } else { "denied" };
        
        self.client
            .patch(&format!(
                "{}/rest/v1/app_requests?id=eq.{}",
                SUPABASE_URL, request_id
            ))
            .header("apikey", SUPABASE_ANON_KEY)
            .header("Authorization", self.auth_header())
            .header("Content-Type", "application/json")
            .json(&serde_json::json!({
                "status": status,
                "actioned_at": chrono::Utc::now().to_rfc3339(),
            }))
            .send()
            .await?;
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppCatalogEntry {
    pub id: String,
    pub flatpak_id: String,
    pub name: String,
    pub summary: Option<String>,
    pub description: Option<String>,
    pub icon_url: Option<String>,
    pub developer: Option<String>,
    pub categories: Option<Vec<String>>,
    pub min_age: Option<i32>,
    pub pegi_rating: Option<String>,
    pub esrb_rating: Option<String>,
    pub guardian_approved: bool,
}

impl AppCatalogEntry {
    pub fn to_app_entry(&self) -> AppEntry {
        let rating = if let Some(age) = self.min_age {
            AgeRating::from_appstream_age(age as u8)
        } else if let Some(ref pegi) = self.pegi_rating {
            AgeRating::from_pegi(pegi)
        } else if let Some(ref esrb) = self.esrb_rating {
            AgeRating::from_esrb(esrb)
        } else {
            AgeRating::Unrated
        };
        
        AppEntry {
            id: self.flatpak_id.clone(),
            name: self.name.clone(),
            summary: self.summary.clone(),
            description: self.description.clone(),
            developer: self.developer.clone(),
            icon_url: self.icon_url.clone(),
            categories: self.categories.clone().unwrap_or_default(),
            rating,
            guardian_approved: self.guardian_approved,
            flatpak_ref: Some(self.flatpak_id.clone()),
            homepage: None,
            version: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppRequest {
    pub id: String,
    pub app_id: String,
    pub child_id: String,
    pub family_id: String,
    pub status: String,
    pub created_at: Option<String>,
    pub actioned_at: Option<String>,
}
