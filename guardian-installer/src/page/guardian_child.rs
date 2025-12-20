// Copyright 2024 Guardian Network Solutions
// SPDX-License-Identifier: GPL-3.0-only

use crate::fl;
use cosmic::{
    Apply, Element,
    iced::Length,
    widget::{self},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

// Supabase configuration - Guardian OS project
const SUPABASE_URL: &str = "https://gkyspvcafyttfhyjryyk.supabase.co";
const SUPABASE_ANON_KEY: &str = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJzdXBhYmFzZSIsInJlZiI6ImdreXNwdmNhZnl0dGZoeWpyeXlrIiwicm9sZSI6ImFub24iLCJpYXQiOjE3MzQyNzQwMjYsImV4cCI6MjA0OTg1MDAyNn0.example";

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum ViewMode {
    #[default]
    SelectChild,
    CreateChild,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Child {
    pub id: String,
    pub parent_id: String,
    pub name: String,
    pub birth_date: Option<String>,
    pub avatar_url: Option<String>,
}

#[derive(Clone, Debug, Default)]
pub struct Page {
    mode: ViewMode,
    // Auth context (passed from guardian_auth page)
    pub access_token: Option<String>,
    pub parent_id: Option<String>,
    // Children list
    children: Vec<Child>,
    loading: bool,
    error_message: Option<String>,
    // Selected child
    pub selected_child: Option<Child>,
    selected_index: Option<usize>,
    // Create child form
    new_child_name: String,
    new_child_birth_date: String,
    // Device claim
    device_claimed: bool,
    device_id: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
struct CreateChildRequest {
    parent_id: String,
    name: String,
    birth_date: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
struct ClaimDeviceRequest {
    parent_id: String,
    child_id: String,
    hostname: String,
    device_key: String,
}

#[derive(Clone, Debug, Deserialize)]
struct DeviceResponse {
    id: String,
}

#[derive(Clone, Debug)]
pub enum Message {
    SetMode(ViewMode),
    LoadChildren,
    ChildrenLoaded(Arc<Result<Vec<Child>, String>>),
    SelectChild(usize),
    SetNewChildName(String),
    SetNewChildBirthDate(String),
    CreateChild,
    ChildCreated(Arc<Result<Child, String>>),
    ClaimDevice,
    DeviceClaimed(Arc<Result<String, String>>),
}

impl From<Message> for super::Message {
    fn from(message: Message) -> Self {
        super::Message::GuardianChild(message)
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
        fl!("guardian-child-page")
    }

    fn completed(&self) -> bool {
        self.selected_child.is_some() && self.device_claimed
    }

    fn init(&mut self) -> cosmic::Task<super::Message> {
        // Trigger loading children when page opens
        cosmic::Task::done(Message::LoadChildren.into())
    }

    fn view(&self) -> Element<'_, super::Message> {
        let spacing = cosmic::theme::spacing();
        
        match self.mode {
            ViewMode::SelectChild => self.view_select_child(spacing),
            ViewMode::CreateChild => self.view_create_child(spacing),
        }
    }
}

impl Page {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_auth_context(&mut self, access_token: String, parent_id: String) {
        self.access_token = Some(access_token);
        self.parent_id = Some(parent_id);
    }

    fn view_select_child(&self, spacing: &cosmic::theme::Spacing) -> Element<'_, super::Message> {
        let description = widget::text::body(fl!("guardian-child-page", "select-description"))
            .apply(widget::container)
            .center_x(Length::Fill);

        // Children list
        let children_list = if self.loading {
            widget::text::body(fl!("loading"))
                .apply(widget::container)
                .center_x(Length::Fill)
                .into()
        } else if self.children.is_empty() {
            widget::text::body(fl!("guardian-child-page", "no-children"))
                .apply(widget::container)
                .center_x(Length::Fill)
                .into()
        } else {
            let mut list = widget::column::with_capacity(self.children.len())
                .spacing(spacing.space_xs);

            for (index, child) in self.children.iter().enumerate() {
                let is_selected = self.selected_index == Some(index);
                
                let child_button = widget::button::text(&child.name)
                    .on_press(Message::SelectChild(index).into())
                    .class(if is_selected {
                        cosmic::theme::Button::Suggested
                    } else {
                        cosmic::theme::Button::Standard
                    })
                    .width(Length::Fill);

                list = list.push(child_button);
            }

            list.into()
        };

        // Error message
        let error_widget = self.error_message.as_ref().map(|msg| {
            widget::text::body(msg)
                .class(cosmic::theme::Text::Destructive)
                .apply(widget::container)
                .center_x(Length::Fill)
        });

        // Create new child button
        let create_button = widget::button::standard(fl!("guardian-child-page", "create-new"))
            .on_press(Message::SetMode(ViewMode::CreateChild).into())
            .apply(widget::container)
            .center_x(Length::Fill);

        // Claim device button (only if child selected)
        let claim_button = if self.selected_child.is_some() && !self.device_claimed {
            Some(
                widget::button::suggested(fl!("guardian-child-page", "claim-device"))
                    .on_press_maybe((!self.loading).then_some(Message::ClaimDevice.into()))
                    .apply(widget::container)
                    .center_x(Length::Fill)
            )
        } else if self.device_claimed {
            Some(
                widget::text::body(fl!("guardian-child-page", "device-claimed"))
                    .class(cosmic::theme::Text::Success)
                    .apply(widget::container)
                    .center_x(Length::Fill)
            )
        } else {
            None
        };

        let mut column = widget::column::with_capacity(8)
            .spacing(spacing.space_s)
            .push(description)
            .push(widget::vertical_space().height(spacing.space_m))
            .push(children_list);

        if let Some(error) = error_widget {
            column = column.push(error);
        }

        column = column
            .push(widget::vertical_space().height(spacing.space_m))
            .push(create_button);

        if let Some(claim) = claim_button {
            column = column.push(widget::vertical_space().height(spacing.space_s));
            column = column.push(claim);
        }

        column.into()
    }

    fn view_create_child(&self, spacing: &cosmic::theme::Spacing) -> Element<'_, super::Message> {
        let description = widget::text::body(fl!("guardian-child-page", "create-description"))
            .apply(widget::container)
            .center_x(Length::Fill);

        // Name input
        let name_input = widget::text_input(fl!("child-name-placeholder"), &self.new_child_name)
            .label(fl!("child-name"))
            .on_input(|value| Message::SetNewChildName(value).into());

        // Birth date input
        let birth_date_input = widget::text_input(fl!("birth-date-placeholder"), &self.new_child_birth_date)
            .label(fl!("birth-date"))
            .on_input(|value| Message::SetNewChildBirthDate(value).into());

        // Error message
        let error_widget = self.error_message.as_ref().map(|msg| {
            widget::text::body(msg)
                .class(cosmic::theme::Text::Destructive)
                .apply(widget::container)
                .center_x(Length::Fill)
        });

        // Back button
        let back_button = widget::button::standard(fl!("back"))
            .on_press(Message::SetMode(ViewMode::SelectChild).into());

        // Create button
        let can_create = !self.new_child_name.is_empty() && !self.loading;
        let create_button = widget::button::suggested(fl!("guardian-child-page", "create"))
            .on_press_maybe(can_create.then_some(Message::CreateChild.into()));

        let button_row = widget::row::with_capacity(2)
            .spacing(spacing.space_s)
            .push(back_button)
            .push(widget::horizontal_space())
            .push(create_button);

        let mut column = widget::column::with_capacity(8)
            .spacing(spacing.space_s)
            .push(description)
            .push(widget::vertical_space().height(spacing.space_m))
            .push(name_input)
            .push(birth_date_input);

        if let Some(error) = error_widget {
            column = column.push(error);
        }

        column = column
            .push(widget::vertical_space().height(spacing.space_m))
            .push(button_row);

        column.into()
    }

    pub fn update(&mut self, message: Message) -> cosmic::Task<super::Message> {
        match message {
            Message::SetMode(mode) => {
                self.mode = mode;
                self.error_message = None;
            }
            Message::LoadChildren => {
                if let (Some(token), Some(parent_id)) = (&self.access_token, &self.parent_id) {
                    self.loading = true;
                    let token = token.clone();
                    let parent_id = parent_id.clone();

                    return cosmic::Task::future(async move {
                        let result = fetch_children(&token, &parent_id).await;
                        Message::ChildrenLoaded(Arc::new(result))
                    });
                }
            }
            Message::ChildrenLoaded(result) => {
                self.loading = false;
                match Arc::into_inner(result).unwrap() {
                    Ok(children) => {
                        self.children = children;
                        self.error_message = None;
                    }
                    Err(error) => {
                        self.error_message = Some(error);
                    }
                }
            }
            Message::SelectChild(index) => {
                if let Some(child) = self.children.get(index) {
                    self.selected_child = Some(child.clone());
                    self.selected_index = Some(index);
                }
            }
            Message::SetNewChildName(name) => {
                self.new_child_name = name;
            }
            Message::SetNewChildBirthDate(date) => {
                self.new_child_birth_date = date;
            }
            Message::CreateChild => {
                if let (Some(token), Some(parent_id)) = (&self.access_token, &self.parent_id) {
                    self.loading = true;
                    let token = token.clone();
                    let parent_id = parent_id.clone();
                    let name = self.new_child_name.clone();
                    let birth_date = if self.new_child_birth_date.is_empty() {
                        None
                    } else {
                        Some(self.new_child_birth_date.clone())
                    };

                    return cosmic::Task::future(async move {
                        let result = create_child(&token, &parent_id, &name, birth_date.as_deref()).await;
                        Message::ChildCreated(Arc::new(result))
                    });
                }
            }
            Message::ChildCreated(result) => {
                self.loading = false;
                match Arc::into_inner(result).unwrap() {
                    Ok(child) => {
                        self.children.push(child.clone());
                        self.selected_child = Some(child);
                        self.selected_index = Some(self.children.len() - 1);
                        self.mode = ViewMode::SelectChild;
                        self.new_child_name.clear();
                        self.new_child_birth_date.clear();
                        self.error_message = None;
                    }
                    Err(error) => {
                        self.error_message = Some(error);
                    }
                }
            }
            Message::ClaimDevice => {
                if let (Some(token), Some(parent_id), Some(child)) = 
                    (&self.access_token, &self.parent_id, &self.selected_child) 
                {
                    self.loading = true;
                    let token = token.clone();
                    let parent_id = parent_id.clone();
                    let child_id = child.id.clone();
                    let hostname = get_hostname();
                    let device_key = generate_device_key();

                    return cosmic::Task::future(async move {
                        let result = claim_device(&token, &parent_id, &child_id, &hostname, &device_key).await;
                        Message::DeviceClaimed(Arc::new(result))
                    });
                }
            }
            Message::DeviceClaimed(result) => {
                self.loading = false;
                match Arc::into_inner(result).unwrap() {
                    Ok(device_id) => {
                        self.device_id = Some(device_id);
                        self.device_claimed = true;
                        self.error_message = None;
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

async fn fetch_children(access_token: &str, parent_id: &str) -> Result<Vec<Child>, String> {
    let client = reqwest::Client::new();
    
    let response = client
        .get(format!("{}/rest/v1/children?parent_id=eq.{}&select=*", SUPABASE_URL, parent_id))
        .header("apikey", SUPABASE_ANON_KEY)
        .header("Authorization", format!("Bearer {}", access_token))
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if response.status().is_success() {
        response
            .json::<Vec<Child>>()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))
    } else {
        Err(format!("Failed to fetch children: {}", response.status()))
    }
}

async fn create_child(
    access_token: &str, 
    parent_id: &str, 
    name: &str, 
    birth_date: Option<&str>
) -> Result<Child, String> {
    let client = reqwest::Client::new();
    
    let response = client
        .post(format!("{}/rest/v1/children", SUPABASE_URL))
        .header("apikey", SUPABASE_ANON_KEY)
        .header("Authorization", format!("Bearer {}", access_token))
        .header("Content-Type", "application/json")
        .header("Prefer", "return=representation")
        .json(&CreateChildRequest {
            parent_id: parent_id.to_string(),
            name: name.to_string(),
            birth_date: birth_date.map(|s| s.to_string()),
        })
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if response.status().is_success() {
        let children: Vec<Child> = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))?;
        
        children.into_iter().next()
            .ok_or_else(|| "No child returned".to_string())
    } else {
        Err(format!("Failed to create child: {}", response.status()))
    }
}

async fn claim_device(
    access_token: &str,
    parent_id: &str,
    child_id: &str,
    hostname: &str,
    device_key: &str,
) -> Result<String, String> {
    let client = reqwest::Client::new();
    
    let response = client
        .post(format!("{}/rest/v1/devices", SUPABASE_URL))
        .header("apikey", SUPABASE_ANON_KEY)
        .header("Authorization", format!("Bearer {}", access_token))
        .header("Content-Type", "application/json")
        .header("Prefer", "return=representation")
        .json(&ClaimDeviceRequest {
            parent_id: parent_id.to_string(),
            child_id: child_id.to_string(),
            hostname: hostname.to_string(),
            device_key: device_key.to_string(),
        })
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if response.status().is_success() {
        let devices: Vec<DeviceResponse> = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))?;
        
        devices.into_iter().next()
            .map(|d| d.id)
            .ok_or_else(|| "No device returned".to_string())
    } else {
        Err(format!("Failed to claim device: {}", response.status()))
    }
}

fn get_hostname() -> String {
    std::fs::read_to_string("/etc/hostname")
        .unwrap_or_else(|_| "guardian-device".to_string())
        .trim()
        .to_string()
}

fn generate_device_key() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();
    
    format!("guardian-{:x}", timestamp)
}
