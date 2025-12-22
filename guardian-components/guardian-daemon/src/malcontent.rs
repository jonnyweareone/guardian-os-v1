//! Malcontent Integration for Guardian Daemon
//!
//! This module provides integration with GNOME's malcontent parental controls.
//! It allows the daemon to:
//! - Set app filters based on child policies
//! - Configure OARS age ratings
//! - Block/allow Flatpak installations
//! - Sync settings from Supabase to local malcontent config
//!
//! Malcontent stores its data in AccountsService at:
//! /var/lib/AccountsService/users/${username}

use anyhow::{Result, Context};
use std::process::Command;
use tracing::{info, warn, error, debug};
use serde::{Deserialize, Serialize};

/// OARS age levels (Open Age Ratings Service)
/// Maps to content intensity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OarsLevel {
    /// No content of this type
    None,
    /// Mild content (suitable for children)
    Mild,
    /// Moderate content (teens)
    Moderate,
    /// Intense content (adults only)
    Intense,
}

impl OarsLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            OarsLevel::None => "none",
            OarsLevel::Mild => "mild",
            OarsLevel::Moderate => "moderate",
            OarsLevel::Intense => "intense",
        }
    }
    
    pub fn from_pegi(pegi: u8) -> Self {
        match pegi {
            0..=3 => OarsLevel::None,
            4..=7 => OarsLevel::Mild,
            8..=12 => OarsLevel::Moderate,
            _ => OarsLevel::Intense,
        }
    }
    
    pub fn from_age(age: u8) -> Self {
        match age {
            0..=6 => OarsLevel::None,
            7..=9 => OarsLevel::Mild,
            10..=14 => OarsLevel::Moderate,
            _ => OarsLevel::Intense,
        }
    }
}

/// App filter for a user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppFilter {
    /// List of blocked app IDs (Flatpak refs or paths)
    pub blocked_apps: Vec<String>,
    /// List of explicitly allowed apps (overrides OARS)
    pub allowed_apps: Vec<String>,
    /// OARS content restrictions by category
    pub oars_restrictions: OarsRestrictions,
    /// Allow user to install Flatpaks
    pub allow_user_installation: bool,
    /// Allow installing to system repo
    pub allow_system_installation: bool,
}

impl Default for AppFilter {
    fn default() -> Self {
        Self {
            blocked_apps: Vec::new(),
            allowed_apps: Vec::new(),
            oars_restrictions: OarsRestrictions::default(),
            allow_user_installation: false,
            allow_system_installation: false,
        }
    }
}

/// OARS content type restrictions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OarsRestrictions {
    pub violence_cartoon: OarsLevel,
    pub violence_fantasy: OarsLevel,
    pub violence_realistic: OarsLevel,
    pub violence_bloodshed: OarsLevel,
    pub violence_sexual: OarsLevel,
    pub drugs_alcohol: OarsLevel,
    pub drugs_narcotics: OarsLevel,
    pub drugs_tobacco: OarsLevel,
    pub sex_nudity: OarsLevel,
    pub sex_themes: OarsLevel,
    pub sex_homosexuality: OarsLevel,
    pub sex_prostitution: OarsLevel,
    pub sex_adultery: OarsLevel,
    pub sex_appearance: OarsLevel,
    pub language_profanity: OarsLevel,
    pub language_humor: OarsLevel,
    pub language_discrimination: OarsLevel,
    pub social_chat: OarsLevel,
    pub social_info: OarsLevel,
    pub social_audio: OarsLevel,
    pub social_location: OarsLevel,
    pub social_contacts: OarsLevel,
    pub money_purchasing: OarsLevel,
    pub money_gambling: OarsLevel,
}

impl Default for OarsRestrictions {
    fn default() -> Self {
        // Default: moderate restrictions (suitable for teens)
        Self {
            violence_cartoon: OarsLevel::Moderate,
            violence_fantasy: OarsLevel::Moderate,
            violence_realistic: OarsLevel::Mild,
            violence_bloodshed: OarsLevel::None,
            violence_sexual: OarsLevel::None,
            drugs_alcohol: OarsLevel::Mild,
            drugs_narcotics: OarsLevel::None,
            drugs_tobacco: OarsLevel::Mild,
            sex_nudity: OarsLevel::None,
            sex_themes: OarsLevel::Mild,
            sex_homosexuality: OarsLevel::Moderate, // Not restricted
            sex_prostitution: OarsLevel::None,
            sex_adultery: OarsLevel::None,
            sex_appearance: OarsLevel::Mild,
            language_profanity: OarsLevel::Mild,
            language_humor: OarsLevel::Moderate,
            language_discrimination: OarsLevel::None,
            social_chat: OarsLevel::Moderate,
            social_info: OarsLevel::Mild,
            social_audio: OarsLevel::Moderate,
            social_location: OarsLevel::None,
            social_contacts: OarsLevel::Mild,
            money_purchasing: OarsLevel::None,
            money_gambling: OarsLevel::None,
        }
    }
}

impl OarsRestrictions {
    /// Create restrictions from a child's age
    pub fn from_age(age: u8) -> Self {
        let base_level = OarsLevel::from_age(age);
        
        let mut restrictions = Self::default();
        
        // Adjust based on age
        if age < 7 {
            // Very young - block most things
            restrictions.violence_cartoon = OarsLevel::Mild;
            restrictions.violence_fantasy = OarsLevel::None;
            restrictions.social_chat = OarsLevel::None;
            restrictions.money_purchasing = OarsLevel::None;
        } else if age < 10 {
            // Young child
            restrictions.violence_realistic = OarsLevel::None;
            restrictions.social_chat = OarsLevel::Mild;
        } else if age < 13 {
            // Pre-teen
            restrictions.violence_realistic = OarsLevel::Mild;
            restrictions.social_chat = OarsLevel::Moderate;
        } else if age < 16 {
            // Teen
            restrictions.violence_realistic = OarsLevel::Moderate;
            restrictions.social_chat = OarsLevel::Moderate;
            restrictions.language_profanity = OarsLevel::Moderate;
        } else {
            // Older teen - fewer restrictions
            restrictions.violence_realistic = OarsLevel::Intense;
            restrictions.social_chat = OarsLevel::Intense;
            restrictions.language_profanity = OarsLevel::Intense;
        }
        
        restrictions
    }
    
    /// Generate malcontent CLI arguments for OARS restrictions
    pub fn to_malcontent_args(&self) -> Vec<String> {
        let mut args = Vec::new();
        
        // Only include non-intense restrictions (intense = no restriction)
        macro_rules! add_restriction {
            ($category:expr, $level:expr) => {
                if $level != OarsLevel::Intense {
                    args.push(format!("{}={}", $category, $level.as_str()));
                }
            };
        }
        
        add_restriction!("violence-cartoon", self.violence_cartoon);
        add_restriction!("violence-fantasy", self.violence_fantasy);
        add_restriction!("violence-realistic", self.violence_realistic);
        add_restriction!("violence-bloodshed", self.violence_bloodshed);
        add_restriction!("violence-sexual", self.violence_sexual);
        add_restriction!("drugs-alcohol", self.drugs_alcohol);
        add_restriction!("drugs-narcotics", self.drugs_narcotics);
        add_restriction!("drugs-tobacco", self.drugs_tobacco);
        add_restriction!("sex-nudity", self.sex_nudity);
        add_restriction!("sex-themes", self.sex_themes);
        add_restriction!("sex-homosexuality", self.sex_homosexuality);
        add_restriction!("sex-prostitution", self.sex_prostitution);
        add_restriction!("sex-adultery", self.sex_adultery);
        add_restriction!("sex-appearance", self.sex_appearance);
        add_restriction!("language-profanity", self.language_profanity);
        add_restriction!("language-humor", self.language_humor);
        add_restriction!("language-discrimination", self.language_discrimination);
        add_restriction!("social-chat", self.social_chat);
        add_restriction!("social-info", self.social_info);
        add_restriction!("social-audio", self.social_audio);
        add_restriction!("social-location", self.social_location);
        add_restriction!("social-contacts", self.social_contacts);
        add_restriction!("money-purchasing", self.money_purchasing);
        add_restriction!("money-gambling", self.money_gambling);
        
        args
    }
}

/// Malcontent controller
pub struct MalcontentController {
    /// Path to malcontent-client binary
    client_path: String,
}

impl MalcontentController {
    pub fn new() -> Self {
        Self {
            client_path: "/usr/bin/malcontent-client".to_string(),
        }
    }
    
    /// Check if malcontent is available on this system
    pub fn is_available(&self) -> bool {
        std::path::Path::new(&self.client_path).exists()
    }
    
    /// Get current app filter for a user
    pub fn get_app_filter(&self, username: &str) -> Result<String> {
        let output = Command::new(&self.client_path)
            .args(["get-app-filter", username])
            .output()
            .context("Failed to run malcontent-client")?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("malcontent-client failed: {}", stderr);
        }
        
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
    
    /// Set app filter for a user
    pub fn set_app_filter(&self, username: &str, filter: &AppFilter) -> Result<()> {
        let mut args = vec!["set-app-filter".to_string(), username.to_string()];
        
        // Installation permissions
        if filter.allow_user_installation {
            args.push("--allow-user-installation".to_string());
        } else {
            args.push("--disallow-user-installation".to_string());
        }
        
        if filter.allow_system_installation {
            args.push("--allow-system-installation".to_string());
        } else {
            args.push("--disallow-system-installation".to_string());
        }
        
        // Add blocked apps
        for app in &filter.blocked_apps {
            args.push(app.clone());
        }
        
        // Add OARS restrictions
        args.extend(filter.oars_restrictions.to_malcontent_args());
        
        debug!("Running: malcontent-client {}", args.join(" "));
        
        let output = Command::new(&self.client_path)
            .args(&args)
            .output()
            .context("Failed to run malcontent-client")?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            error!("malcontent-client failed: {}", stderr);
            anyhow::bail!("Failed to set app filter: {}", stderr);
        }
        
        info!("App filter set for user {}", username);
        Ok(())
    }
    
    /// Check if a specific app is allowed for a user
    pub fn check_app_allowed(&self, username: &str, app_id: &str) -> Result<bool> {
        let output = Command::new(&self.client_path)
            .args(["check-app-filter", username, app_id])
            .output()
            .context("Failed to run malcontent-client")?;
        
        // Exit code 0 = allowed, 1 = not allowed, other = error
        Ok(output.status.code() == Some(0))
    }
    
    /// Apply child profile from Guardian to malcontent
    pub fn apply_guardian_profile(
        &self,
        username: &str,
        age: u8,
        blocked_apps: Vec<String>,
        allowed_apps: Vec<String>,
    ) -> Result<()> {
        info!("Applying Guardian profile for {} (age {})", username, age);
        
        // Generate OARS restrictions based on age
        let oars = OarsRestrictions::from_age(age);
        
        // Build filter
        let filter = AppFilter {
            blocked_apps,
            allowed_apps,
            oars_restrictions: oars,
            allow_user_installation: false, // Parent must approve
            allow_system_installation: false,
        };
        
        self.set_app_filter(username, &filter)?;
        
        info!("Guardian profile applied for {}", username);
        Ok(())
    }
    
    /// Convert Supabase app_policies to malcontent filter
    pub fn policies_to_filter(
        policies: &[super::supabase::AppPolicy],
        child_age: u8,
    ) -> AppFilter {
        let mut blocked = Vec::new();
        let mut allowed = Vec::new();
        
        for policy in policies {
            match policy.policy.as_deref() {
                Some("blocked") => {
                    // Convert app_id to Flatpak ref if needed
                    if policy.app_id.contains('/') {
                        blocked.push(policy.app_id.clone());
                    } else {
                        // Try common Flatpak patterns
                        blocked.push(format!("app/{}/x86_64/stable", policy.app_id));
                    }
                }
                Some("allowed") | Some("approved") => {
                    if policy.app_id.contains('/') {
                        allowed.push(policy.app_id.clone());
                    } else {
                        allowed.push(format!("app/{}/x86_64/stable", policy.app_id));
                    }
                }
                _ => {}
            }
        }
        
        AppFilter {
            blocked_apps: blocked,
            allowed_apps: allowed,
            oars_restrictions: OarsRestrictions::from_age(child_age),
            allow_user_installation: false,
            allow_system_installation: false,
        }
    }
}

impl Default for MalcontentController {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_oars_from_age() {
        let young = OarsRestrictions::from_age(5);
        assert_eq!(young.violence_realistic, OarsLevel::None);
        assert_eq!(young.social_chat, OarsLevel::None);
        
        let teen = OarsRestrictions::from_age(14);
        assert_eq!(teen.violence_realistic, OarsLevel::Moderate);
        assert_eq!(teen.social_chat, OarsLevel::Moderate);
    }
    
    #[test]
    fn test_oars_to_args() {
        let oars = OarsRestrictions::from_age(10);
        let args = oars.to_malcontent_args();
        
        // Should contain some restrictions
        assert!(!args.is_empty());
        
        // Check format
        for arg in &args {
            assert!(arg.contains('='));
        }
    }
    
    #[test]
    fn test_pegi_to_oars() {
        assert_eq!(OarsLevel::from_pegi(3), OarsLevel::None);
        assert_eq!(OarsLevel::from_pegi(7), OarsLevel::Mild);
        assert_eq!(OarsLevel::from_pegi(12), OarsLevel::Moderate);
        assert_eq!(OarsLevel::from_pegi(16), OarsLevel::Intense);
        assert_eq!(OarsLevel::from_pegi(18), OarsLevel::Intense);
    }
}
