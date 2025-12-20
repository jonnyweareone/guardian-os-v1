// Copyright 2024 Guardian Network Solutions
// SPDX-License-Identifier: GPL-3.0-only

//! Guardian Sync enrollment page
//! 
//! Allows parents to optionally enable COSMIC settings sync.
//! This syncs desktop preferences, themes, and app settings
//! across all Guardian OS devices in the family.

use crate::fl;
use cosmic::{
    Apply, Element,
    iced::Length,
    widget::{self},
};
use std::sync::Arc;

// Guardian Sync Server configuration
const SYNC_SERVER_URL: &str = "https://sync.guardian-os.com:50051";

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum SyncChoice {
    #[default]
    Undecided,
    EnableSync,
    SkipSync,
}

#[derive(Clone, Debug, Default)]
pub struct Page {
    choice: SyncChoice,
    loading: bool,
    error_message: Option<String>,
    // Auth context from previous pages
    pub access_token: Option<String>,
    pub parent_id: Option<String>,
    pub device_id: Option<String>,
    // Sync enrollment result
    pub sync_enabled: bool,
    pub sync_token: Option<String>,
    pub encryption_key: Option<String>,
    pub account_hash: Option<String>,
}

#[derive(Clone, Debug)]
pub enum Message {
    SetChoice(SyncChoice),
    EnableSync,
    SkipSync,
    SyncResult(Arc<Result<SyncEnrollResult, String>>),
}

#[derive(Clone, Debug)]
pub struct SyncEnrollResult {
    pub sync_token: String,
    pub encryption_key: String,
    pub account_hash: String,
}

impl From<Message> for super::Message {
    fn from(message: Message) -> Self {
        super::Message::GuardianSync(message)
    }
}

impl From<Message> for crate::Message {
    fn from(message: Message) -> Self {
        crate::Message::PageMessage(message.into())
    }
}

impl super::Page for Page {
    fn as_any(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn title(&self) -> String {
        fl!("guardian-sync-page")
    }

    fn completed(&self) -> bool {
        self.choice != SyncChoice::Undecided
    }

    fn optional(&self) -> bool {
        true
    }

    fn skippable(&self) -> bool {
        true
    }

    fn view(&self) -> Element<'_, super::Message> {
        let spacing = cosmic::theme::spacing();

        // Header
        let header = widget::text::title3(fl!("guardian-sync-page", "title"))
            .apply(widget::container)
            .center_x(Length::Fill);

        // Description
        let description = widget::text::body(fl!("guardian-sync-page", "description"))
            .apply(widget::container)
            .center_x(Length::Fill);

        // Benefits list
        let benefits = widget::column::with_capacity(5)
            .spacing(spacing.space_xs)
            .push(benefit_item(fl!("guardian-sync-page", "benefit-settings")))
            .push(benefit_item(fl!("guardian-sync-page", "benefit-themes")))
            .push(benefit_item(fl!("guardian-sync-page", "benefit-reinstall")))
            .push(benefit_item(fl!("guardian-sync-page", "benefit-multidevice")))
            .push(benefit_item(fl!("guardian-sync-page", "benefit-encrypted")));

        // Privacy note
        let privacy_note = widget::text::caption(fl!("guardian-sync-page", "privacy-note"))
            .apply(widget::container)
            .center_x(Length::Fill);

        // Error message
        let error_widget = self.error_message.as_ref().map(|msg| {
            widget::text::body(msg)
                .class(cosmic::theme::Text::Destructive)
                .apply(widget::container)
                .center_x(Length::Fill)
        });

        // Action buttons
        let enable_button = if self.loading {
            widget::button::suggested(fl!("loading"))
        } else {
            widget::button::suggested(fl!("guardian-sync-page", "enable-sync"))
                .on_press(Message::EnableSync.into())
        };

        let skip_button = widget::button::standard(fl!("guardian-sync-page", "skip-sync"))
            .on_press_maybe((!self.loading).then_some(Message::SkipSync.into()));

        let buttons = widget::row::with_capacity(2)
            .spacing(spacing.space_m)
            .push(skip_button)
            .push(widget::horizontal_space())
            .push(enable_button)
            .apply(widget::container)
            .center_x(Length::Fill);

        // Success indicator if already enabled
        let status_widget = if self.sync_enabled {
            Some(
                widget::text::body(fl!("guardian-sync-page", "sync-enabled"))
                    .class(cosmic::theme::Text::Success)
                    .apply(widget::container)
                    .center_x(Length::Fill)
            )
        } else {
            None
        };

        // Build the page
        let mut column = widget::column::with_capacity(10)
            .spacing(spacing.space_s)
            .push(header)
            .push(widget::vertical_space().height(spacing.space_m))
            .push(description)
            .push(widget::vertical_space().height(spacing.space_m))
            .push(benefits)
            .push(widget::vertical_space().height(spacing.space_s))
            .push(privacy_note);

        if let Some(error) = error_widget {
            column = column.push(error);
        }

        if let Some(status) = status_widget {
            column = column.push(status);
        }

        column = column
            .push(widget::vertical_space().height(spacing.space_m))
            .push(buttons);

        column.into()
    }
}

fn benefit_item(text: String) -> Element<'static, super::Message> {
    widget::row::with_capacity(2)
        .spacing(8)
        .push(widget::text::body("✓"))
        .push(widget::text::body(text))
        .into()
}

impl Page {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_context(
        &mut self, 
        access_token: String, 
        parent_id: String,
        device_id: String,
    ) {
        self.access_token = Some(access_token);
        self.parent_id = Some(parent_id);
        self.device_id = Some(device_id);
    }

    pub fn update(&mut self, message: Message) -> cosmic::Task<super::Message> {
        match message {
            Message::SetChoice(choice) => {
                self.choice = choice;
            }
            Message::EnableSync => {
                if let (Some(token), Some(parent_id), Some(device_id)) = 
                    (&self.access_token, &self.parent_id, &self.device_id) 
                {
                    self.loading = true;
                    self.error_message = None;
                    let token = token.clone();
                    let parent_id = parent_id.clone();
                    let device_id = device_id.clone();

                    return cosmic::Task::future(async move {
                        let result = enroll_sync(&token, &parent_id, &device_id).await;
                        Message::SyncResult(Arc::new(result))
                    });
                } else {
                    self.error_message = Some(fl!("guardian-sync-page", "error-no-auth"));
                }
            }
            Message::SkipSync => {
                self.choice = SyncChoice::SkipSync;
                self.sync_enabled = false;
            }
            Message::SyncResult(result) => {
                self.loading = false;
                match Arc::into_inner(result).unwrap() {
                    Ok(enroll_result) => {
                        self.choice = SyncChoice::EnableSync;
                        self.sync_enabled = true;
                        self.sync_token = Some(enroll_result.sync_token);
                        self.encryption_key = Some(enroll_result.encryption_key);
                        self.account_hash = Some(enroll_result.account_hash);
                        self.error_message = None;

                        // Save sync config for guardian-daemon
                        if let Err(e) = save_sync_config(self) {
                            tracing::warn!("Failed to save sync config: {}", e);
                        }
                    }
                    Err(error) => {
                        self.error_message = Some(error);
                    }
                }
            }
        }
        cosmic::Task::none()
    }
}

/// Enroll device with Guardian Sync Server
async fn enroll_sync(
    access_token: &str,
    parent_id: &str,
    device_id: &str,
) -> Result<SyncEnrollResult, String> {
    // For now, use HTTP/REST endpoint
    // TODO: Switch to gRPC when guardian-sync-server is deployed
    
    let client = reqwest::Client::new();
    
    // Get device info
    let hostname = get_hostname();
    let os_version = get_os_version();
    
    // Create account hash from parent_id (consistent with sync server expectations)
    let account_hash = format!("guardian-{}", &parent_id[..8.min(parent_id.len())]);
    
    // Generate device hash
    let device_hash = format!("device-{}", device_id);
    
    // Register device with sync server
    // This is a simplified HTTP fallback - production will use gRPC
    let response = client
        .post(format!("{}/api/v1/register-device", SYNC_SERVER_URL.replace(":50051", "")))
        .header("Authorization", format!("Bearer {}", access_token))
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({
            "account_hash": account_hash,
            "device_hash": device_hash,
            "device_name": hostname,
            "os_version": os_version,
            "app_version": env!("CARGO_PKG_VERSION"),
        }))
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if response.status().is_success() {
        let data: serde_json::Value = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        Ok(SyncEnrollResult {
            sync_token: data["auth_token"].as_str().unwrap_or("").to_string(),
            encryption_key: data["encryption_key"].as_str().unwrap_or("").to_string(),
            account_hash,
        })
    } else {
        // If sync server isn't available, generate local credentials
        // This allows offline installation with sync enabled later
        tracing::warn!("Sync server not available, generating local credentials");
        
        Ok(SyncEnrollResult {
            sync_token: generate_local_token(),
            encryption_key: generate_encryption_key(),
            account_hash,
        })
    }
}

fn get_hostname() -> String {
    std::fs::read_to_string("/etc/hostname")
        .unwrap_or_else(|_| "guardian-device".to_string())
        .trim()
        .to_string()
}

fn get_os_version() -> String {
    std::fs::read_to_string("/etc/os-release")
        .ok()
        .and_then(|content| {
            content.lines()
                .find(|line| line.starts_with("VERSION="))
                .map(|line| line.trim_start_matches("VERSION=").trim_matches('"').to_string())
        })
        .unwrap_or_else(|| "Guardian OS 1.0".to_string())
}

fn generate_local_token() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();
    format!("local-sync-{:x}", timestamp)
}

fn generate_encryption_key() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    // Generate a pseudo-random key (production should use proper crypto)
    format!("{:032x}", timestamp ^ 0xDEADBEEFCAFEBABE)
}

/// Save sync configuration for guardian-daemon to use
fn save_sync_config(page: &Page) -> Result<(), Box<dyn std::error::Error>> {
    let config_dir = dirs::config_dir()
        .ok_or("Could not find config directory")?
        .join("guardian");
    
    std::fs::create_dir_all(&config_dir)?;
    
    let sync_config = serde_json::json!({
        "enabled": page.sync_enabled,
        "server_url": SYNC_SERVER_URL,
        "sync_token": page.sync_token,
        "encryption_key": page.encryption_key,
        "account_hash": page.account_hash,
        "device_id": page.device_id,
    });
    
    let config_path = config_dir.join("sync.json");
    std::fs::write(&config_path, serde_json::to_string_pretty(&sync_config)?)?;
    
    // Set restrictive permissions (owner read/write only)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&config_path, std::fs::Permissions::from_mode(0o600))?;
    }
    
    tracing::info!("Sync config saved to {:?}", config_path);
    Ok(())
}
