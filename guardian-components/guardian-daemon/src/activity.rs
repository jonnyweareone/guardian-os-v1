//! Activity logging - structures for tracking user activity

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Types of activity that can be tracked
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ActivityType {
    /// Application usage (which app is active)
    AppUsage,
    /// URL visited (from browser integration)
    UrlVisit,
    /// Screen time accumulation
    ScreenTime,
    /// User login event
    Login,
    /// User logout event
    Logout,
    /// App installed
    AppInstalled,
    /// App launched
    AppLaunched,
    /// Search query (if enabled)
    SearchQuery,
    /// File accessed (if enabled)
    FileAccess,
}

/// A single activity event to be logged and synced
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityEvent {
    /// Unique identifier for this event
    pub id: String,
    
    /// Device that generated this event
    pub device_id: String,
    
    /// Child profile associated with this event (if any)
    pub child_id: Option<String>,
    
    /// Type of activity
    pub activity_type: ActivityType,
    
    /// When the activity occurred
    pub timestamp: DateTime<Utc>,
    
    /// Activity-specific data (JSON)
    pub data: serde_json::Value,
    
    /// Whether this event has been synced to the server
    pub synced: bool,
}

/// URL visit event data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UrlVisitData {
    pub url: String,
    pub title: Option<String>,
    pub domain: String,
    pub browser: String,
    pub duration_secs: Option<u64>,
    pub ai_classification: Option<UrlClassification>,
}

/// AI classification result for a URL
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UrlClassification {
    pub safe: bool,
    pub category: String,
    pub confidence: f32,
    pub blocked: bool,
    pub reason: Option<String>,
}

/// App usage event data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppUsageData {
    pub app_id: String,
    pub app_name: String,
    pub window_title: Option<String>,
    pub duration_secs: u64,
    pub category: Option<String>,
}

/// Daily activity summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailySummary {
    pub date: String,
    pub device_id: String,
    pub child_id: Option<String>,
    pub total_screen_time_minutes: u64,
    pub app_breakdown: Vec<AppTimeSummary>,
    pub url_count: u64,
    pub blocked_attempts: u64,
    pub categories: Vec<CategoryTimeSummary>,
}

/// Time spent in a specific app
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppTimeSummary {
    pub app_id: String,
    pub app_name: String,
    pub minutes: u64,
    pub percentage: f32,
}

/// Time spent in a content category
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryTimeSummary {
    pub category: String,
    pub minutes: u64,
    pub percentage: f32,
}

impl ActivityEvent {
    /// Create a new URL visit event
    pub fn url_visit(
        device_id: &str,
        child_id: Option<&str>,
        url: &str,
        title: Option<&str>,
        browser: &str,
    ) -> Self {
        let domain = url::Url::parse(url)
            .map(|u| u.host_str().unwrap_or("unknown").to_string())
            .unwrap_or_else(|_| "unknown".to_string());
        
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            device_id: device_id.to_string(),
            child_id: child_id.map(|s| s.to_string()),
            activity_type: ActivityType::UrlVisit,
            timestamp: Utc::now(),
            data: serde_json::json!({
                "url": url,
                "title": title,
                "domain": domain,
                "browser": browser,
            }),
            synced: false,
        }
    }
    
    /// Create a new app usage event
    pub fn app_usage(
        device_id: &str,
        child_id: Option<&str>,
        app_id: &str,
        app_name: &str,
        duration_secs: u64,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            device_id: device_id.to_string(),
            child_id: child_id.map(|s| s.to_string()),
            activity_type: ActivityType::AppUsage,
            timestamp: Utc::now(),
            data: serde_json::json!({
                "app_id": app_id,
                "app_name": app_name,
                "duration_secs": duration_secs,
            }),
            synced: false,
        }
    }
}
