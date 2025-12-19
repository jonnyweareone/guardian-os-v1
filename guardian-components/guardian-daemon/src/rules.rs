//! Safety rules - defines content filtering and screen time rules

use serde::{Deserialize, Serialize};
use chrono::NaiveTime;

/// Complete safety rules for a child profile
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SafetyRules {
    /// Child ID these rules apply to
    pub child_id: Option<String>,
    
    /// Screen time configuration
    pub screen_time: ScreenTimeRules,
    
    /// Content filtering rules
    pub content_filter: ContentFilterRules,
    
    /// App restriction rules
    pub app_restrictions: AppRestrictionRules,
    
    /// Bedtime/schedule rules
    pub schedule: ScheduleRules,
    
    /// Version for sync conflict resolution
    pub version: u64,
    
    /// Last updated timestamp
    pub updated_at: Option<String>,
}

/// Screen time limits and schedules
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ScreenTimeRules {
    /// Daily screen time limit in minutes (None = unlimited)
    pub daily_limit_minutes: Option<u32>,
    
    /// Per-app time limits
    pub app_limits: Vec<AppTimeLimit>,
    
    /// Time windows when device can be used
    pub allowed_windows: Vec<TimeWindow>,
    
    /// Apps exempt from screen time limits
    pub exempt_apps: Vec<String>,
    
    /// Whether to show warnings before limit
    pub show_warnings: bool,
    
    /// Minutes before limit to show warning
    pub warning_threshold_minutes: u32,
}

/// Time limit for a specific app
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppTimeLimit {
    pub app_id: String,
    pub daily_limit_minutes: u32,
}

/// A time window when device usage is allowed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeWindow {
    /// Days this window applies (0 = Sunday, 6 = Saturday)
    pub days: Vec<u8>,
    
    /// Start time (HH:MM format)
    pub start: String,
    
    /// End time (HH:MM format)
    pub end: String,
}

/// Content filtering configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ContentFilterRules {
    /// Age rating level (child, teen, mature)
    pub age_rating: AgeRating,
    
    /// Categories to block
    pub blocked_categories: Vec<String>,
    
    /// Specific domains to block
    pub blocked_domains: Vec<String>,
    
    /// Domains that override blocking (whitelist)
    pub allowed_domains: Vec<String>,
    
    /// Keywords to flag/block
    pub blocked_keywords: Vec<String>,
    
    /// Enable SafeSearch for search engines
    pub safe_search_enabled: bool,
    
    /// Block explicit images
    pub block_explicit_images: bool,
    
    /// AI content classification enabled
    pub ai_classification_enabled: bool,
}

/// Age-appropriate content rating
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum AgeRating {
    /// Ages 0-7: Most restrictive
    Child,
    
    /// Ages 8-12: Moderate restrictions
    #[default]
    Preteen,
    
    /// Ages 13-17: Light restrictions
    Teen,
    
    /// 18+: Minimal restrictions
    Mature,
}

/// App installation and usage restrictions
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AppRestrictionRules {
    /// Mode: allowlist or blocklist
    pub mode: AppRestrictionMode,
    
    /// Apps in the allow/block list
    pub apps: Vec<String>,
    
    /// Require approval for new app installs
    pub require_install_approval: bool,
    
    /// Categories of apps to block
    pub blocked_categories: Vec<String>,
}

/// Whether to use allowlist or blocklist for apps
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum AppRestrictionMode {
    /// Only apps in the list are allowed
    Allowlist,
    
    /// All apps allowed except those in the list
    #[default]
    Blocklist,
}

/// Schedule-based rules (bedtime, school hours, etc.)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ScheduleRules {
    /// Bedtime settings
    pub bedtime: Option<BedtimeRules>,
    
    /// School/focus time settings
    pub focus_times: Vec<FocusTime>,
}

/// Bedtime configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BedtimeRules {
    /// When bedtime starts
    pub start: String,
    
    /// When bedtime ends
    pub end: String,
    
    /// Days bedtime applies
    pub days: Vec<u8>,
    
    /// Whether to allow emergency calls/apps
    pub allow_emergency: bool,
    
    /// Apps allowed during bedtime
    pub allowed_apps: Vec<String>,
}

/// Focus/school time configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FocusTime {
    pub name: String,
    pub start: String,
    pub end: String,
    pub days: Vec<u8>,
    pub allowed_apps: Vec<String>,
    pub allowed_websites: Vec<String>,
}

impl SafetyRules {
    /// Check if a URL should be blocked
    pub fn should_block_url(&self, url: &str, domain: &str) -> bool {
        // Check whitelist first
        if self.content_filter.allowed_domains.iter()
            .any(|d| domain.ends_with(d)) {
            return false;
        }
        
        // Check blocklist
        if self.content_filter.blocked_domains.iter()
            .any(|d| domain.ends_with(d)) {
            return true;
        }
        
        // Check keywords in URL
        let url_lower = url.to_lowercase();
        if self.content_filter.blocked_keywords.iter()
            .any(|k| url_lower.contains(&k.to_lowercase())) {
            return true;
        }
        
        false
    }
    
    /// Check if an app is allowed
    pub fn is_app_allowed(&self, app_id: &str) -> bool {
        match self.app_restrictions.mode {
            AppRestrictionMode::Allowlist => {
                self.app_restrictions.apps.contains(&app_id.to_string())
            }
            AppRestrictionMode::Blocklist => {
                !self.app_restrictions.apps.contains(&app_id.to_string())
            }
        }
    }
    
    /// Check if current time is within allowed hours
    pub fn is_within_allowed_hours(&self) -> bool {
        if self.screen_time.allowed_windows.is_empty() {
            return true; // No restrictions
        }
        
        let now = chrono::Local::now();
        let current_day = now.weekday().num_days_from_sunday() as u8;
        let current_time = now.time();
        
        self.screen_time.allowed_windows.iter().any(|window| {
            if !window.days.contains(&current_day) {
                return false;
            }
            
            let start = NaiveTime::parse_from_str(&window.start, "%H:%M").ok();
            let end = NaiveTime::parse_from_str(&window.end, "%H:%M").ok();
            
            match (start, end) {
                (Some(s), Some(e)) => current_time >= s && current_time <= e,
                _ => false,
            }
        })
    }
}
