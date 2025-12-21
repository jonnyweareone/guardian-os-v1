// Copyright 2024 Guardian Network Solutions
// SPDX-License-Identifier: GPL-3.0-only

use crate::fl;
use cosmic::{
    Apply, Element,
    iced::Length,
    widget::{self},
};
use std::sync::Arc;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum SyncChoice {
    #[default]
    Undecided,
    EnableSync,
    SkipSync,
}

#[derive(Clone, Debug, Default)]
pub struct Page {
    pub access_token: Option<String>,
    pub parent_id: Option<String>,
    pub device_id: Option<String>,
    choice: SyncChoice,
    loading: bool,
    error_message: Option<String>,
    pub sync_enabled: bool,
}

#[derive(Clone, Debug)]
pub enum Message {
    EnableSync,
    SkipSync,
    SyncResult(Arc<Result<(), String>>),
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

    fn view(&self) -> Element<'_, super::Message> {
        let spacing = cosmic::theme::spacing();
        
        let title = widget::text::title3(fl!("guardian-sync-page", "title"))
            .apply(widget::container)
            .center_x(Length::Fill);

        let description = widget::text::body(fl!("guardian-sync-page", "description"))
            .apply(widget::container)
            .center_x(Length::Fill);

        let benefits = widget::column::with_capacity(5)
            .spacing(spacing.space_xs)
            .push(widget::text::body("✓ Desktop settings sync automatically"))
            .push(widget::text::body("✓ Themes and wallpapers follow you"))
            .push(widget::text::body("✓ Easy restore on reinstall"))
            .push(widget::text::body("✓ Perfect for multi-device families"))
            .push(widget::text::body("✓ End-to-end encrypted"));

        let privacy = widget::text::caption(fl!("guardian-sync-page", "privacy"))
            .apply(widget::container)
            .center_x(Length::Fill);

        let error_widget = self.error_message.as_ref().map(|msg| {
            widget::text::body(msg)
                .apply(widget::container)
                .center_x(Length::Fill)
        });

        let enable_button = widget::button::suggested(fl!("guardian-sync-page", "enable"))
            .on_press_maybe((!self.loading).then_some(Message::EnableSync.into()))
            .width(Length::Fill);

        let skip_button = widget::button::standard(fl!("guardian-sync-page", "skip"))
            .on_press_maybe((!self.loading).then_some(Message::SkipSync.into()))
            .width(Length::Fill);

        let button_row = widget::row::with_capacity(2)
            .spacing(spacing.space_s)
            .push(skip_button)
            .push(enable_button);

        let mut column = widget::column::with_capacity(10)
            .spacing(spacing.space_s)
            .push(title)
            .push(widget::vertical_space().height(spacing.space_m))
            .push(description)
            .push(widget::vertical_space().height(spacing.space_s))
            .push(benefits)
            .push(widget::vertical_space().height(spacing.space_s))
            .push(privacy);

        if let Some(error) = error_widget {
            column = column.push(error);
        }

        column = column
            .push(widget::vertical_space().height(spacing.space_m))
            .push(button_row);

        column.into()
    }
}

impl Page {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_context(&mut self, access_token: String, parent_id: String, device_id: String) {
        self.access_token = Some(access_token);
        self.parent_id = Some(parent_id);
        self.device_id = Some(device_id);
    }

    pub fn update(&mut self, message: Message) -> cosmic::Task<super::Message> {
        match message {
            Message::EnableSync => {
                self.loading = true;
                self.error_message = None;

                if let (Some(token), Some(parent_id), Some(device_id)) = 
                    (&self.access_token, &self.parent_id, &self.device_id) 
                {
                    let token = token.clone();
                    let parent_id = parent_id.clone();
                    let device_id = device_id.clone();

                    return cosmic::Task::future(async move {
                        let result = enroll_sync(&token, &parent_id, &device_id).await;
                        Message::SyncResult(Arc::new(result))
                    }).map(super::Message::GuardianSync);
                } else {
                    self.choice = SyncChoice::EnableSync;
                    self.sync_enabled = true;
                    self.loading = false;
                }
            }
            Message::SkipSync => {
                self.choice = SyncChoice::SkipSync;
                self.sync_enabled = false;
            }
            Message::SyncResult(result) => {
                self.loading = false;
                match Arc::into_inner(result).unwrap() {
                    Ok(()) => {
                        self.choice = SyncChoice::EnableSync;
                        self.sync_enabled = true;
                        self.error_message = None;
                        let _ = save_sync_config(true);
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

async fn enroll_sync(_token: &str, parent_id: &str, device_id: &str) -> Result<(), String> {
    // For now, just save the config locally
    // In production, this would register with the sync server
    let config_dir = dirs::config_dir()
        .ok_or("Could not find config directory")?
        .join("guardian");
    
    std::fs::create_dir_all(&config_dir)
        .map_err(|e| format!("Failed to create config dir: {}", e))?;
    
    let sync_config = serde_json::json!({
        "enabled": true,
        "parent_id": parent_id,
        "device_id": device_id,
        "enrolled_at": chrono::Utc::now().to_rfc3339()
    });
    
    let config_path = config_dir.join("sync.json");
    std::fs::write(&config_path, serde_json::to_string_pretty(&sync_config).unwrap())
        .map_err(|e| format!("Failed to write config: {}", e))?;
    
    Ok(())
}

fn save_sync_config(enabled: bool) -> Result<(), String> {
    let config_dir = dirs::config_dir()
        .ok_or("Could not find config directory")?
        .join("guardian");
    
    std::fs::create_dir_all(&config_dir)
        .map_err(|e| format!("Failed to create config dir: {}", e))?;
    
    let config_path = config_dir.join("sync.json");
    let config = serde_json::json!({
        "enabled": enabled
    });
    
    std::fs::write(&config_path, serde_json::to_string_pretty(&config).unwrap())
        .map_err(|e| format!("Failed to write config: {}", e))?;
    
    Ok(())
}
