//! Wizard state management

use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use anyhow::Result;

/// Current page in the wizard
#[derive(Debug, Clone, PartialEq)]
pub enum WizardPage {
    Welcome,
    UserType,
    ParentLogin,
    SelectChild,  // NEW: Child selection after parent login
    ChildJoin,
    WaitingActivation,
    Complete,
}

/// State of the wizard
pub struct WizardState {
    pub current_page: WizardPage,
    
    // Device info
    pub device_id: Option<String>,
    pub hardware_id: Option<String>,
    pub activation_code: Option<String>,
    pub activated: bool,
    
    // Auth
    pub email: String,
    pub password: String,
    pub name: String,
    pub access_token: Option<String>,
    pub user_id: Option<String>,
    
    // Family
    pub family_id: Option<String>,
    pub family_name: Option<String>,
    pub family_code: String,
    pub child_id: Option<String>,
    pub children: Vec<ChildData>,
    pub selected_child: Option<ChildData>,
    
    // UI state
    pub error: Option<String>,
    pub loading: bool,
}

/// Child data for display
#[derive(Debug, Clone)]
pub struct ChildData {
    pub id: String,
    pub name: String,
    pub date_of_birth: Option<String>,
    pub avatar_url: Option<String>,
}

impl ChildData {
    /// Calculate age from date of birth
    pub fn age(&self) -> Option<u32> {
        let dob = self.date_of_birth.as_ref()?;
        let birth = chrono::NaiveDate::parse_from_str(dob, "%Y-%m-%d").ok()?;
        let today = chrono::Local::now().date_naive();
        let age = today.years_since(birth)?;
        Some(age)
    }
    
    /// Get display string with name and age
    pub fn display_name(&self) -> String {
        match self.age() {
            Some(age) => format!("{} ({})", self.name, age),
            None => self.name.clone(),
        }
    }
}

impl WizardState {
    pub fn new() -> Self {
        Self {
            current_page: WizardPage::Welcome,
            device_id: None,
            hardware_id: None,
            activation_code: None,
            activated: false,
            email: String::new(),
            password: String::new(),
            name: String::new(),
            access_token: None,
            user_id: None,
            family_id: None,
            family_name: None,
            family_code: String::new(),
            child_id: None,
            children: Vec::new(),
            selected_child: None,
            error: None,
            loading: false,
        }
    }
    
    pub fn next_page(&mut self) {
        self.error = None;
        self.current_page = match self.current_page {
            WizardPage::Welcome => WizardPage::UserType,
            WizardPage::UserType => WizardPage::ParentLogin, // Default
            WizardPage::ParentLogin => WizardPage::SelectChild, // Changed: go to child selection
            WizardPage::SelectChild => WizardPage::Complete,    // After selecting child
            WizardPage::ChildJoin => WizardPage::WaitingActivation,
            WizardPage::WaitingActivation => WizardPage::Complete,
            WizardPage::Complete => WizardPage::Complete,
        };
    }
    
    pub fn prev_page(&mut self) {
        self.error = None;
        self.current_page = match self.current_page {
            WizardPage::Welcome => WizardPage::Welcome,
            WizardPage::UserType => WizardPage::Welcome,
            WizardPage::ParentLogin => WizardPage::UserType,
            WizardPage::SelectChild => WizardPage::ParentLogin,
            WizardPage::ChildJoin => WizardPage::UserType,
            WizardPage::WaitingActivation => WizardPage::UserType,
            WizardPage::Complete => WizardPage::Complete,
        };
    }
    
    /// Save configuration to daemon config file
    pub fn save_config(&self) -> Result<()> {
        let config = DaemonConfig {
            device_id: self.device_id.clone(),
            hardware_id: self.hardware_id.clone(),
            family_id: self.family_id.clone(),
            child_id: self.child_id.clone(),
            child_name: self.selected_child.as_ref().map(|c| c.name.clone()),
            activation_code: self.activation_code.clone(),
            activated: self.activated,
            access_token: self.access_token.clone(),
        };
        
        let config_dir = PathBuf::from("/etc/guardian");
        std::fs::create_dir_all(&config_dir)?;
        
        let config_path = config_dir.join("daemon.toml");
        let content = toml::to_string_pretty(&config)?;
        std::fs::write(&config_path, &content)?;
        
        // Also write to user config dir
        if let Some(config_home) = dirs::config_dir() {
            let user_config_dir = config_home.join("guardian");
            std::fs::create_dir_all(&user_config_dir)?;
            let user_config_path = user_config_dir.join("daemon.toml");
            std::fs::write(&user_config_path, &content)?;
        }
        
        Ok(())
    }
}

/// Configuration saved to daemon.toml
#[derive(Debug, Serialize, Deserialize)]
struct DaemonConfig {
    device_id: Option<String>,
    hardware_id: Option<String>,
    family_id: Option<String>,
    child_id: Option<String>,
    child_name: Option<String>,
    activation_code: Option<String>,
    activated: bool,
    access_token: Option<String>,
}

fn dirs_config_dir() -> Option<PathBuf> {
    std::env::var("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .ok()
        .or_else(|| {
            std::env::var("HOME")
                .map(|h| PathBuf::from(h).join(".config"))
                .ok()
        })
}

// Implement dirs::config_dir ourselves since we might not have the crate
mod dirs {
    use super::*;
    
    pub fn config_dir() -> Option<PathBuf> {
        dirs_config_dir()
    }
}
