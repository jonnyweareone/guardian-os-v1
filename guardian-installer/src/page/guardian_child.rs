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

const SUPABASE_URL: &str = "https://gkyspvcafyttfhyjryyk.supabase.co";
const SUPABASE_ANON_KEY: &str = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJzdXBhYmFzZSIsInJlZiI6ImdreXNwdmNhZnl0dGZoeWpyeXlrIiwicm9sZSI6ImFub24iLCJpYXQiOjE3MzQyNzQwMjYsImV4cCI6MjA0OTg1MDAyNn0.example";

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Child {
    pub id: String,
    pub parent_id: String,
    pub name: String,
}

#[derive(Clone, Debug, Default)]
pub struct Page {
    pub access_token: Option<String>,
    pub parent_id: Option<String>,
    children: Vec<Child>,
    loading: bool,
    error_message: Option<String>,
    pub selected_child: Option<Child>,
    selected_index: Option<usize>,
    new_child_name: String,
    pub device_claimed: bool,
    pub device_id: Option<String>,
}

#[derive(Clone, Debug)]
pub enum Message {
    LoadChildren,
    ChildrenLoaded(Arc<Result<Vec<Child>, String>>),
    SelectChild(usize),
    SetNewChildName(String),
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
        cosmic::Task::done(Message::LoadChildren.into())
    }

    fn view(&self) -> Element<'_, super::Message> {
        let spacing = cosmic::theme::spacing();
        
        let description = widget::text::body(fl!("guardian-child-page", "select-description"))
            .apply(widget::container)
            .center_x(Length::Fill);

        let children_list: Element<'_, super::Message> = if self.loading {
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

        let error_widget = self.error_message.as_ref().map(|msg| {
            widget::text::body(msg)
                .apply(widget::container)
                .center_x(Length::Fill)
        });

        let name_input = widget::text_input(fl!("child-name-placeholder"), &self.new_child_name)
            .on_input(|value| Message::SetNewChildName(value).into());

        let can_create = !self.new_child_name.is_empty() && !self.loading;
        let create_button = widget::button::standard(fl!("guardian-child-page", "create-new"))
            .on_press_maybe(can_create.then_some(Message::CreateChild.into()));

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
                    .apply(widget::container)
                    .center_x(Length::Fill)
            )
        } else {
            None
        };

        let mut column = widget::column::with_capacity(10)
            .spacing(spacing.space_s)
            .push(description)
            .push(widget::vertical_space().height(spacing.space_m))
            .push(children_list);

        if let Some(error) = error_widget {
            column = column.push(error);
        }

        column = column
            .push(widget::vertical_space().height(spacing.space_m))
            .push(name_input)
            .push(create_button);

        if let Some(claim) = claim_button {
            column = column.push(widget::vertical_space().height(spacing.space_s));
            column = column.push(claim);
        }

        column.into()
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

    pub fn update(&mut self, message: Message) -> cosmic::Task<super::Message> {
        match message {
            Message::LoadChildren => {
                if let (Some(token), Some(parent_id)) = (&self.access_token, &self.parent_id) {
                    self.loading = true;
                    let token = token.clone();
                    let parent_id = parent_id.clone();

                    return cosmic::Task::future(async move {
                        let result = fetch_children(&token, &parent_id).await;
                        Message::ChildrenLoaded(Arc::new(result))
                    }).map(super::Message::GuardianChild);
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
            Message::CreateChild => {
                if let (Some(token), Some(parent_id)) = (&self.access_token, &self.parent_id) {
                    self.loading = true;
                    let token = token.clone();
                    let parent_id = parent_id.clone();
                    let name = self.new_child_name.clone();

                    return cosmic::Task::future(async move {
                        let result = create_child(&token, &parent_id, &name).await;
                        Message::ChildCreated(Arc::new(result))
                    }).map(super::Message::GuardianChild);
                }
            }
            Message::ChildCreated(result) => {
                self.loading = false;
                match Arc::into_inner(result).unwrap() {
                    Ok(child) => {
                        self.selected_child = Some(child.clone());
                        self.children.push(child);
                        self.new_child_name.clear();
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

                    return cosmic::Task::future(async move {
                        let result = claim_device(&token, &parent_id, &child_id).await;
                        Message::DeviceClaimed(Arc::new(result))
                    }).map(super::Message::GuardianChild);
                }
            }
            Message::DeviceClaimed(result) => {
                self.loading = false;
                match Arc::into_inner(result).unwrap() {
                    Ok(device_id) => {
                        self.device_claimed = true;
                        self.device_id = Some(device_id);
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

async fn fetch_children(token: &str, parent_id: &str) -> Result<Vec<Child>, String> {
    let client = reqwest::Client::new();
    
    let response = client
        .get(format!("{}/rest/v1/children?parent_id=eq.{}", SUPABASE_URL, parent_id))
        .header("apikey", SUPABASE_ANON_KEY)
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if response.status().is_success() {
        response.json::<Vec<Child>>()
            .await
            .map_err(|e| format!("Parse error: {}", e))
    } else {
        Err("Failed to fetch children".to_string())
    }
}

async fn create_child(token: &str, parent_id: &str, name: &str) -> Result<Child, String> {
    let client = reqwest::Client::new();
    
    let response = client
        .post(format!("{}/rest/v1/children", SUPABASE_URL))
        .header("apikey", SUPABASE_ANON_KEY)
        .header("Authorization", format!("Bearer {}", token))
        .header("Content-Type", "application/json")
        .header("Prefer", "return=representation")
        .json(&serde_json::json!({
            "parent_id": parent_id,
            "name": name
        }))
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if response.status().is_success() {
        let children: Vec<Child> = response.json()
            .await
            .map_err(|e| format!("Parse error: {}", e))?;
        children.into_iter().next().ok_or("No child returned".to_string())
    } else {
        Err("Failed to create child".to_string())
    }
}

async fn claim_device(token: &str, parent_id: &str, child_id: &str) -> Result<String, String> {
    let client = reqwest::Client::new();
    
    let hostname = hostname::get()
        .map(|h| h.to_string_lossy().to_string())
        .unwrap_or_else(|_| "guardian-device".to_string());

    let device_key = format!("{}-{}", child_id, uuid::Uuid::new_v4());
    
    let response = client
        .post(format!("{}/rest/v1/devices", SUPABASE_URL))
        .header("apikey", SUPABASE_ANON_KEY)
        .header("Authorization", format!("Bearer {}", token))
        .header("Content-Type", "application/json")
        .header("Prefer", "return=representation")
        .json(&serde_json::json!({
            "parent_id": parent_id,
            "child_id": child_id,
            "hostname": hostname,
            "device_key": device_key
        }))
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if response.status().is_success() {
        let devices: Vec<serde_json::Value> = response.json()
            .await
            .map_err(|e| format!("Parse error: {}", e))?;
        
        devices.first()
            .and_then(|d| d.get("id"))
            .and_then(|id| id.as_str())
            .map(|s| s.to_string())
            .ok_or("No device ID returned".to_string())
    } else {
        Err("Failed to claim device".to_string())
    }
}
