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
pub enum AuthMode {
    #[default]
    SignIn,
    CreateAccount,
}

#[derive(Clone, Debug, Default)]
pub struct Page {
    mode: AuthMode,
    email: String,
    password: String,
    password_confirm: String,
    full_name: String,
    password_hidden: bool,
    loading: bool,
    error_message: Option<String>,
    authenticated: bool,
    // Auth state
    pub access_token: Option<String>,
    pub parent_id: Option<String>,
    pub parent_email: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
struct SignInRequest {
    email: String,
    password: String,
}

#[derive(Clone, Debug, Serialize)]
struct SignUpRequest {
    email: String,
    password: String,
}

#[derive(Clone, Debug, Deserialize)]
struct AuthResponse {
    access_token: String,
    user: AuthUser,
}

#[derive(Clone, Debug, Deserialize)]
struct AuthUser {
    id: String,
    email: String,
}

#[derive(Clone, Debug, Deserialize)]
struct AuthError {
    error: Option<String>,
    error_description: Option<String>,
    msg: Option<String>,
}

#[derive(Clone, Debug)]
pub enum Message {
    SetMode(AuthMode),
    SetEmail(String),
    SetPassword(String),
    SetPasswordConfirm(String),
    SetFullName(String),
    TogglePasswordVisibility,
    Submit,
    AuthResult(Arc<Result<AuthResponse, String>>),
}

impl From<Message> for super::Message {
    fn from(message: Message) -> Self {
        super::Message::GuardianAuth(message)
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
        fl!("guardian-auth-page")
    }

    fn completed(&self) -> bool {
        self.authenticated
    }

    fn view(&self) -> Element<'_, super::Message> {
        let spacing = cosmic::theme::spacing();
        
        // Mode toggle
        let mode_toggle = widget::row::with_capacity(2)
            .spacing(spacing.space_s)
            .push(
                widget::button::text(fl!("sign-in"))
                    .on_press_maybe(
                        (self.mode != AuthMode::SignIn && !self.loading)
                            .then_some(Message::SetMode(AuthMode::SignIn).into())
                    )
                    .class(if self.mode == AuthMode::SignIn {
                        cosmic::theme::Button::Suggested
                    } else {
                        cosmic::theme::Button::Standard
                    })
            )
            .push(
                widget::button::text(fl!("create-account"))
                    .on_press_maybe(
                        (self.mode != AuthMode::CreateAccount && !self.loading)
                            .then_some(Message::SetMode(AuthMode::CreateAccount).into())
                    )
                    .class(if self.mode == AuthMode::CreateAccount {
                        cosmic::theme::Button::Suggested
                    } else {
                        cosmic::theme::Button::Standard
                    })
            )
            .apply(widget::container)
            .center_x(Length::Fill);

        // Description text
        let description = widget::text::body(match self.mode {
            AuthMode::SignIn => fl!("guardian-auth-page", "sign-in-description"),
            AuthMode::CreateAccount => fl!("guardian-auth-page", "create-account-description"),
        })
        .apply(widget::container)
        .center_x(Length::Fill);

        // Email input
        let email_input = widget::text_input(fl!("email-placeholder"), &self.email)
            .label(fl!("email"))
            .on_input(|value| Message::SetEmail(value).into());

        // Full name input (only for create account)
        let full_name_input = if self.mode == AuthMode::CreateAccount {
            Some(
                widget::text_input(fl!("full-name-placeholder"), &self.full_name)
                    .label(fl!("full-name"))
                    .on_input(|value| Message::SetFullName(value).into())
            )
        } else {
            None
        };

        // Password input
        let password_input = widget::secure_input(
            fl!("password-placeholder"),
            &self.password,
            Some(Message::TogglePasswordVisibility.into()),
            self.password_hidden,
        )
        .label(fl!("password"))
        .on_input(|value| Message::SetPassword(value).into());

        // Password confirm (only for create account)
        let password_confirm_input = if self.mode == AuthMode::CreateAccount {
            Some(
                widget::secure_input(
                    fl!("password-confirm-placeholder"),
                    &self.password_confirm,
                    None,
                    self.password_hidden,
                )
                .label(fl!("password-confirm"))
                .on_input(|value| Message::SetPasswordConfirm(value).into())
            )
        } else {
            None
        };

        // Error message
        let error_widget = self.error_message.as_ref().map(|msg| {
            widget::text::body(msg)
                .class(cosmic::theme::Text::Destructive)
                .apply(widget::container)
                .center_x(Length::Fill)
        });

        // Submit button
        let can_submit = self.can_submit();
        let submit_text = if self.loading {
            fl!("loading")
        } else {
            match self.mode {
                AuthMode::SignIn => fl!("sign-in"),
                AuthMode::CreateAccount => fl!("create-account"),
            }
        };
        
        let submit_button = widget::button::suggested(submit_text)
            .on_press_maybe(can_submit.then_some(Message::Submit.into()))
            .apply(widget::container)
            .center_x(Length::Fill);

        // Build the form
        let mut column = widget::column::with_capacity(10)
            .spacing(spacing.space_s)
            .push(mode_toggle)
            .push(widget::vertical_space().height(spacing.space_m))
            .push(description)
            .push(widget::vertical_space().height(spacing.space_s))
            .push(email_input);

        if let Some(input) = full_name_input {
            column = column.push(input);
        }

        column = column.push(password_input);

        if let Some(input) = password_confirm_input {
            column = column.push(input);
        }

        if let Some(error) = error_widget {
            column = column.push(error);
        }

        column = column.push(widget::vertical_space().height(spacing.space_m));
        column = column.push(submit_button);

        column.into()
    }
}

impl Page {
    pub fn new() -> Self {
        Self::default()
    }

    fn can_submit(&self) -> bool {
        if self.loading {
            return false;
        }

        let email_valid = !self.email.is_empty() && self.email.contains('@');
        let password_valid = self.password.len() >= 6;

        match self.mode {
            AuthMode::SignIn => email_valid && password_valid,
            AuthMode::CreateAccount => {
                email_valid 
                    && password_valid 
                    && self.password == self.password_confirm
                    && !self.full_name.is_empty()
            }
        }
    }

    pub fn update(&mut self, message: Message) -> cosmic::Task<super::Message> {
        match message {
            Message::SetMode(mode) => {
                self.mode = mode;
                self.error_message = None;
            }
            Message::SetEmail(email) => {
                self.email = email;
                self.error_message = None;
            }
            Message::SetPassword(password) => {
                self.password = password;
                self.error_message = None;
            }
            Message::SetPasswordConfirm(password) => {
                self.password_confirm = password;
                self.error_message = None;
            }
            Message::SetFullName(name) => {
                self.full_name = name;
                self.error_message = None;
            }
            Message::TogglePasswordVisibility => {
                self.password_hidden = !self.password_hidden;
            }
            Message::Submit => {
                self.loading = true;
                self.error_message = None;

                let mode = self.mode.clone();
                let email = self.email.clone();
                let password = self.password.clone();

                return cosmic::Task::future(async move {
                    let result = match mode {
                        AuthMode::SignIn => sign_in(&email, &password).await,
                        AuthMode::CreateAccount => sign_up(&email, &password).await,
                    };
                    Message::AuthResult(Arc::new(result))
                });
            }
            Message::AuthResult(result) => {
                self.loading = false;
                match Arc::into_inner(result).unwrap() {
                    Ok(response) => {
                        self.authenticated = true;
                        self.access_token = Some(response.access_token);
                        self.parent_id = Some(response.user.id);
                        self.parent_email = Some(response.user.email);
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

async fn sign_in(email: &str, password: &str) -> Result<AuthResponse, String> {
    let client = reqwest::Client::new();
    
    let response = client
        .post(format!("{}/auth/v1/token?grant_type=password", SUPABASE_URL))
        .header("apikey", SUPABASE_ANON_KEY)
        .header("Content-Type", "application/json")
        .json(&SignInRequest {
            email: email.to_string(),
            password: password.to_string(),
        })
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if response.status().is_success() {
        response
            .json::<AuthResponse>()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))
    } else {
        let error = response
            .json::<AuthError>()
            .await
            .map_err(|e| format!("Failed to parse error: {}", e))?;
        
        Err(error.error_description
            .or(error.msg)
            .or(error.error)
            .unwrap_or_else(|| "Authentication failed".to_string()))
    }
}

async fn sign_up(email: &str, password: &str) -> Result<AuthResponse, String> {
    let client = reqwest::Client::new();
    
    let response = client
        .post(format!("{}/auth/v1/signup", SUPABASE_URL))
        .header("apikey", SUPABASE_ANON_KEY)
        .header("Content-Type", "application/json")
        .json(&SignUpRequest {
            email: email.to_string(),
            password: password.to_string(),
        })
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if response.status().is_success() {
        response
            .json::<AuthResponse>()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))
    } else {
        let error = response
            .json::<AuthError>()
            .await
            .map_err(|e| format!("Failed to parse error: {}", e))?;
        
        Err(error.error_description
            .or(error.msg)
            .or(error.error)
            .unwrap_or_else(|| "Registration failed".to_string()))
    }
}
