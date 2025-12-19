//! Guardian Wizard - First-boot setup for Guardian OS
//!
//! This wizard runs on first boot to:
//! 1. Register the device with Guardian backend
//! 2. Authenticate parent user
//! 3. Fetch family and children data
//! 4. Allow parent to select which child will use this device
//! 5. Activate device and start Guardian Daemon

mod api;
mod pages;
mod state;

use iced::{
    Application, Command, Element, Settings, Theme,
    widget::{button, column, container, row, text, text_input, scrollable, Space},
    Length, Alignment,
};
use tracing::{info, error};

use api::GuardianApi;
use state::{WizardState, WizardPage, ChildData};

fn main() -> iced::Result {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();
    
    info!("Guardian Wizard starting...");
    
    GuardianWizard::run(Settings {
        window: iced::window::Settings {
            size: iced::Size::new(800.0, 600.0),
            resizable: false,
            decorations: true,
            ..Default::default()
        },
        ..Default::default()
    })
}

/// Main wizard application
struct GuardianWizard {
    state: WizardState,
    api: GuardianApi,
}

#[derive(Debug, Clone)]
enum Message {
    // Navigation
    NextPage,
    PrevPage,
    GoToPage(WizardPage),
    
    // Device registration
    RegisterDevice,
    DeviceRegistered(Result<api::DeviceInfo, String>),
    
    // Activation polling
    CheckActivation,
    ActivationStatus(Result<api::ActivationStatus, String>),
    
    // Parent flow
    EmailChanged(String),
    PasswordChanged(String),
    NameChanged(String),
    Login,
    LoginResult(Result<api::AuthResult, String>),
    
    // Family & Children
    FetchFamily,
    FamilyLoaded(Result<api::FamilyInfo, String>),
    FetchChildren,
    ChildrenLoaded(Result<Vec<api::ChildInfo>, String>),
    SelectChild(ChildData),
    ConfirmChildSelection,
    DeviceActivated(Result<(), String>),
    
    // Child join flow
    FamilyCodeChanged(String),
    JoinFamily,
    JoinResult(Result<api::FamilyInfo, String>),
    
    // Finish
    Complete,
    StartDaemon,
    DaemonStarted(Result<(), String>),
}

impl Application for GuardianWizard {
    type Executor = iced::executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        let state = WizardState::new();
        let api = GuardianApi::new();
        
        (
            Self { state, api },
            Command::perform(async {}, |_| Message::RegisterDevice),
        )
    }

    fn title(&self) -> String {
        "Guardian OS Setup".to_string()
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::NextPage => {
                self.state.next_page();
                Command::none()
            }
            
            Message::PrevPage => {
                self.state.prev_page();
                Command::none()
            }
            
            Message::GoToPage(page) => {
                self.state.current_page = page;
                Command::none()
            }
            
            Message::RegisterDevice => {
                let api = self.api.clone();
                Command::perform(
                    async move { api.register_device().await },
                    |result| Message::DeviceRegistered(result.map_err(|e| e.to_string())),
                )
            }
            
            Message::DeviceRegistered(result) => {
                match result {
                    Ok(device_info) => {
                        info!("Device registered: {}", device_info.device_id);
                        self.state.device_id = Some(device_info.device_id);
                        self.state.activation_code = device_info.activation_code;
                        self.state.hardware_id = Some(device_info.hardware_id);
                    }
                    Err(e) => {
                        error!("Device registration failed: {}", e);
                        self.state.error = Some(format!("Registration failed: {}", e));
                    }
                }
                Command::none()
            }
            
            Message::CheckActivation => {
                if let Some(ref device_id) = self.state.device_id {
                    let api = self.api.clone();
                    let device_id = device_id.clone();
                    Command::perform(
                        async move { api.check_activation(&device_id).await },
                        |result| Message::ActivationStatus(result.map_err(|e| e.to_string())),
                    )
                } else {
                    Command::none()
                }
            }
            
            Message::ActivationStatus(result) => {
                match result {
                    Ok(status) => {
                        if status.activated {
                            self.state.activated = true;
                            self.state.family_id = status.family_id;
                            self.state.child_id = status.child_id;
                            self.state.current_page = WizardPage::Complete;
                        }
                    }
                    Err(e) => {
                        error!("Activation check failed: {}", e);
                    }
                }
                // Continue polling if not activated
                if !self.state.activated {
                    Command::perform(
                        tokio::time::sleep(std::time::Duration::from_secs(3)),
                        |_| Message::CheckActivation,
                    )
                } else {
                    Command::none()
                }
            }
            
            Message::EmailChanged(email) => {
                self.state.email = email;
                Command::none()
            }
            
            Message::PasswordChanged(password) => {
                self.state.password = password;
                Command::none()
            }
            
            Message::NameChanged(name) => {
                self.state.name = name;
                Command::none()
            }
            
            Message::Login => {
                self.state.loading = true;
                self.state.error = None;
                let api = self.api.clone();
                let email = self.state.email.clone();
                let password = self.state.password.clone();
                Command::perform(
                    async move { api.login(&email, &password).await },
                    |result| Message::LoginResult(result.map_err(|e| e.to_string())),
                )
            }
            
            Message::LoginResult(result) => {
                self.state.loading = false;
                match result {
                    Ok(auth) => {
                        info!("Login successful for user: {}", auth.user_id);
                        self.state.access_token = Some(auth.access_token.clone());
                        self.state.user_id = Some(auth.user_id.clone());
                        self.api.set_access_token(auth.access_token);
                        // Immediately fetch family data
                        return Command::perform(async {}, |_| Message::FetchFamily);
                    }
                    Err(e) => {
                        self.state.error = Some(format!("Login failed: {}", e));
                    }
                }
                Command::none()
            }
            
            Message::FetchFamily => {
                self.state.loading = true;
                if let Some(ref user_id) = self.state.user_id {
                    let api = self.api.clone();
                    let user_id = user_id.clone();
                    Command::perform(
                        async move { api.fetch_family_for_user(&user_id).await },
                        |result| Message::FamilyLoaded(result.map_err(|e| e.to_string())),
                    )
                } else {
                    self.state.error = Some("No user ID available".to_string());
                    Command::none()
                }
            }
            
            Message::FamilyLoaded(result) => {
                match result {
                    Ok(family) => {
                        info!("Family loaded: {} ({})", family.name, family.id);
                        self.state.family_id = Some(family.id.clone());
                        self.state.family_name = Some(family.name);
                        // Now fetch children
                        let api = self.api.clone();
                        let family_id = family.id;
                        return Command::perform(
                            async move { api.fetch_children(&family_id).await },
                            |result| Message::ChildrenLoaded(result.map_err(|e| e.to_string())),
                        );
                    }
                    Err(e) => {
                        self.state.loading = false;
                        self.state.error = Some(format!("Failed to load family: {}", e));
                    }
                }
                Command::none()
            }
            
            Message::FetchChildren => {
                if let Some(ref family_id) = self.state.family_id {
                    let api = self.api.clone();
                    let family_id = family_id.clone();
                    Command::perform(
                        async move { api.fetch_children(&family_id).await },
                        |result| Message::ChildrenLoaded(result.map_err(|e| e.to_string())),
                    )
                } else {
                    Command::none()
                }
            }
            
            Message::ChildrenLoaded(result) => {
                self.state.loading = false;
                match result {
                    Ok(children) => {
                        info!("Loaded {} children", children.len());
                        self.state.children = children.into_iter().map(|c| ChildData {
                            id: c.id,
                            name: c.name,
                            date_of_birth: c.date_of_birth,
                            avatar_url: c.avatar_url,
                        }).collect();
                        // Navigate to child selection page
                        self.state.current_page = WizardPage::SelectChild;
                    }
                    Err(e) => {
                        self.state.error = Some(format!("Failed to load children: {}", e));
                    }
                }
                Command::none()
            }
            
            Message::SelectChild(child) => {
                info!("Selected child: {} ({})", child.name, child.id);
                self.state.selected_child = Some(child.clone());
                self.state.child_id = Some(child.id);
                Command::none()
            }
            
            Message::ConfirmChildSelection => {
                // Activate device for selected child
                if let (Some(device_id), Some(child_id), Some(family_id)) = (
                    &self.state.device_id,
                    &self.state.child_id,
                    &self.state.family_id,
                ) {
                    self.state.loading = true;
                    let api = self.api.clone();
                    let device_id = device_id.clone();
                    let child_id = child_id.clone();
                    let family_id = family_id.clone();
                    Command::perform(
                        async move { 
                            api.activate_device_for_child(&device_id, &child_id, &family_id).await 
                        },
                        |result| Message::DeviceActivated(result.map_err(|e| e.to_string())),
                    )
                } else {
                    self.state.error = Some("Missing device, child, or family ID".to_string());
                    Command::none()
                }
            }
            
            Message::DeviceActivated(result) => {
                self.state.loading = false;
                match result {
                    Ok(()) => {
                        info!("Device activated successfully!");
                        self.state.activated = true;
                        self.state.current_page = WizardPage::Complete;
                    }
                    Err(e) => {
                        self.state.error = Some(format!("Activation failed: {}", e));
                    }
                }
                Command::none()
            }
            
            Message::FamilyCodeChanged(code) => {
                self.state.family_code = code;
                Command::none()
            }
            
            Message::JoinFamily => {
                // TODO: Implement join family flow
                Command::none()
            }
            
            Message::JoinResult(_) => Command::none(),
            
            Message::Complete => {
                self.state.save_config().ok();
                Command::perform(async {}, |_| Message::StartDaemon)
            }
            
            Message::StartDaemon => {
                Command::perform(
                    async {
                        // Start guardian-daemon via systemd
                        let output = tokio::process::Command::new("systemctl")
                            .args(["--user", "enable", "--now", "guardian-daemon"])
                            .output()
                            .await;
                        
                        match output {
                            Ok(o) if o.status.success() => Ok(()),
                            Ok(o) => Err(String::from_utf8_lossy(&o.stderr).to_string()),
                            Err(e) => Err(e.to_string()),
                        }
                    },
                    |result| Message::DaemonStarted(result),
                )
            }
            
            Message::DaemonStarted(result) => {
                match result {
                    Ok(()) => {
                        info!("Guardian daemon started successfully");
                        // Close wizard
                        std::process::exit(0);
                    }
                    Err(e) => {
                        error!("Failed to start daemon: {}", e);
                        self.state.error = Some(format!("Failed to start daemon: {}", e));
                    }
                }
                Command::none()
            }
        }
    }

    fn view(&self) -> Element<Message> {
        let content: Element<Message> = match self.state.current_page {
            WizardPage::Welcome => self.view_welcome(),
            WizardPage::UserType => self.view_user_type(),
            WizardPage::ParentLogin => self.view_parent_login(),
            WizardPage::SelectChild => self.view_select_child(),
            WizardPage::ChildJoin => self.view_child_join(),
            WizardPage::WaitingActivation => self.view_waiting_activation(),
            WizardPage::Complete => self.view_complete(),
        };
        
        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .padding(40)
            .into()
    }
}

impl GuardianWizard {
    fn view_welcome(&self) -> Element<Message> {
        column![
            // Guardian OS Logo
            text("🛡️").size(80),
            Space::with_height(10),
            text("Welcome to Guardian OS")
                .size(42),
            Space::with_height(10),
            text("The safe computing platform for families")
                .size(22),
            Space::with_height(40),
            
            // Description
            container(
                column![
                    text("Guardian OS is a complete Linux operating system designed to keep children safe online. Built on Pop!_OS with COSMIC desktop, it provides a beautiful, modern computing experience with powerful parental controls built in from the ground up.")
                        .size(15),
                ]
                .padding(20)
                .align_items(Alignment::Center)
            )
            .width(600),
            
            Space::with_height(30),
            
            // Features grid
            text("Key Features")
                .size(20),
            Space::with_height(15),
            
            row![
                column![
                    text("🕐 Screen Time").size(16),
                    text("Smart scheduling and").size(13),
                    text("daily limits").size(13),
                ]
                .align_items(Alignment::Center)
                .width(150),
                
                Space::with_width(20),
                
                column![
                    text("🔒 Content Filter").size(16),
                    text("AI-powered safe").size(13),
                    text("browsing protection").size(13),
                ]
                .align_items(Alignment::Center)
                .width(150),
                
                Space::with_width(20),
                
                column![
                    text("📱 Parent App").size(16),
                    text("Monitor and manage").size(13),
                    text("from anywhere").size(13),
                ]
                .align_items(Alignment::Center)
                .width(150),
                
                Space::with_width(20),
                
                column![
                    text("🔐 Privacy First").size(16),
                    text("Local processing,").size(13),
                    text("no cloud tracking").size(13),
                ]
                .align_items(Alignment::Center)
                .width(150),
            ],
            
            Space::with_height(40),
            
            button(
                text("Get Started")
                    .size(18)
            )
            .on_press(Message::NextPage)
            .padding([15, 40]),
        ]
        .spacing(5)
        .align_items(Alignment::Center)
        .into()
    }
    
    fn view_user_type(&self) -> Element<Message> {
        column![
            text("Who will use this computer?")
                .size(30),
            Space::with_height(40),
            row![
                button(
                    column![
                        text("👨‍👩‍👧 I'm a Parent")
                            .size(20),
                        Space::with_height(10),
                        text("Set up this device for\none of your children")
                            .size(14),
                    ]
                    .align_items(Alignment::Center)
                    .padding(30)
                )
                .on_press(Message::GoToPage(WizardPage::ParentLogin)),
                Space::with_width(40),
                button(
                    column![
                        text("👧 I'm a Child")
                            .size(20),
                        Space::with_height(10),
                        text("Join your family using\na code from your parent")
                            .size(14),
                    ]
                    .align_items(Alignment::Center)
                    .padding(30)
                )
                .on_press(Message::GoToPage(WizardPage::ChildJoin)),
            ],
        ]
        .spacing(10)
        .align_items(Alignment::Center)
        .into()
    }
    
    fn view_parent_login(&self) -> Element<Message> {
        let login_button = if self.state.loading {
            button(text("Signing in..."))
        } else {
            button(text("Sign In")).on_press(Message::Login)
        };
        
        column![
            text("Parent Sign In")
                .size(30),
            Space::with_height(20),
            text("Sign in to your Guardian account")
                .size(16),
            Space::with_height(30),
            text_input("Email", &self.state.email)
                .on_input(Message::EmailChanged)
                .padding(10)
                .width(300),
            Space::with_height(10),
            text_input("Password", &self.state.password)
                .on_input(Message::PasswordChanged)
                .secure(true)
                .padding(10)
                .width(300),
            Space::with_height(20),
            if let Some(ref error) = self.state.error {
                text(error).style(iced::theme::Text::Color(iced::Color::from_rgb(0.8, 0.2, 0.2)))
            } else {
                text("")
            },
            Space::with_height(20),
            row![
                button(text("Back"))
                    .on_press(Message::PrevPage),
                Space::with_width(20),
                login_button,
            ],
        ]
        .spacing(5)
        .align_items(Alignment::Center)
        .into()
    }
    
    fn view_select_child(&self) -> Element<Message> {
        let family_name = self.state.family_name.clone()
            .unwrap_or_else(|| "Your Family".to_string());
        
        // Build list of child buttons
        let children_list: Element<Message> = if self.state.children.is_empty() {
            column![
                text("No children found in your family.")
                    .size(16),
                Space::with_height(10),
                text("Please add children in the Guardian app first.")
                    .size(14),
            ]
            .align_items(Alignment::Center)
            .into()
        } else {
            let mut children_col = column![].spacing(10).align_items(Alignment::Center);
            
            for child in &self.state.children {
                let is_selected = self.state.selected_child.as_ref()
                    .map(|s| s.id == child.id)
                    .unwrap_or(false);
                
                let child_clone = child.clone();
                let age_str = child.age()
                    .map(|a| format!("{} years old", a))
                    .unwrap_or_default();
                
                let btn_content = row![
                    // Avatar placeholder
                    text("👤").size(32),
                    Space::with_width(15),
                    column![
                        text(&child.name).size(18),
                        text(&age_str).size(14),
                    ],
                ]
                .align_items(Alignment::Center)
                .padding(15);
                
                let btn = if is_selected {
                    button(btn_content)
                        .style(iced::theme::Button::Primary)
                        .on_press(Message::SelectChild(child_clone))
                        .width(300)
                } else {
                    button(btn_content)
                        .on_press(Message::SelectChild(child_clone))
                        .width(300)
                };
                
                children_col = children_col.push(btn);
            }
            
            scrollable(children_col)
                .height(250)
                .into()
        };
        
        // Confirm button (only enabled if child selected)
        let confirm_btn = if self.state.selected_child.is_some() && !self.state.loading {
            button(text("Continue"))
                .on_press(Message::ConfirmChildSelection)
                .padding(15)
        } else if self.state.loading {
            button(text("Activating...")).padding(15)
        } else {
            button(text("Select a child")).padding(15)
        };
        
        column![
            text("Who will use this device?")
                .size(30),
            Space::with_height(10),
            text(format!("Family: {}", family_name))
                .size(16),
            Space::with_height(30),
            text("Select the child who will use this computer:")
                .size(16),
            Space::with_height(20),
            children_list,
            Space::with_height(20),
            if let Some(ref error) = self.state.error {
                text(error).style(iced::theme::Text::Color(iced::Color::from_rgb(0.8, 0.2, 0.2)))
            } else if let Some(ref child) = self.state.selected_child {
                text(format!("Selected: {}", child.display_name()))
            } else {
                text("")
            },
            Space::with_height(20),
            row![
                button(text("Back"))
                    .on_press(Message::GoToPage(WizardPage::ParentLogin)),
                Space::with_width(20),
                confirm_btn,
            ],
        ]
        .spacing(5)
        .align_items(Alignment::Center)
        .into()
    }
    
    fn view_child_join(&self) -> Element<Message> {
        let activation_code = self.state.activation_code.clone()
            .unwrap_or_else(|| "Loading...".to_string());
        
        column![
            text("Join Your Family")
                .size(30),
            Space::with_height(20),
            text("Show this code to your parent:")
                .size(16),
            Space::with_height(30),
            container(
                text(&activation_code)
                    .size(48)
            )
            .padding(30)
            .style(iced::theme::Container::Box),
            Space::with_height(30),
            text("Your parent will enter this code in the Guardian app")
                .size(14),
            text("to approve this device and set up your profile.")
                .size(14),
            Space::with_height(30),
            text("Waiting for parent approval...")
                .size(16),
            // TODO: Add spinner
            Space::with_height(20),
            button(text("Back"))
                .on_press(Message::PrevPage),
        ]
        .spacing(5)
        .align_items(Alignment::Center)
        .into()
    }
    
    fn view_waiting_activation(&self) -> Element<Message> {
        let activation_code = self.state.activation_code.clone()
            .unwrap_or_else(|| "------".to_string());
        
        column![
            text("Waiting for Activation")
                .size(30),
            Space::with_height(30),
            text("Enter this code in the Guardian parent app:")
                .size(16),
            Space::with_height(20),
            container(
                text(&activation_code)
                    .size(56)
            )
            .padding(40)
            .style(iced::theme::Container::Box),
            Space::with_height(30),
            Space::with_height(30),
            text("⏳ Waiting for parent approval...")
                .size(16),
        ]
        .spacing(10)
        .align_items(Alignment::Center)
        .into()
    }
    
    fn view_complete(&self) -> Element<Message> {
        let child_name = self.state.selected_child.as_ref()
            .map(|c| c.name.clone())
            .unwrap_or_else(|| "your child".to_string());
        
        column![
            text("🎉").size(60),
            Space::with_height(10),
            text("Welcome to Guardian OS!")
                .size(36),
            Space::with_height(20),
            text(format!("This device is now set up for {}", child_name))
                .size(20),
            Space::with_height(30),
            
            container(
                column![
                    text("What happens now:")
                        .size(18),
                    Space::with_height(15),
                    row![
                        text("✓").size(16),
                        Space::with_width(10),
                        text("Screen time limits are active and will be enforced").size(14),
                    ],
                    Space::with_height(8),
                    row![
                        text("✓").size(16),
                        Space::with_width(10),
                        text("Content filtering is protecting web browsing").size(14),
                    ],
                    Space::with_height(8),
                    row![
                        text("✓").size(16),
                        Space::with_width(10),
                        text("App usage is being monitored").size(14),
                    ],
                    Space::with_height(8),
                    row![
                        text("✓").size(16),
                        Space::with_width(10),
                        text("Parents can manage settings from the Guardian app").size(14),
                    ],
                ]
                .align_items(Alignment::Start)
                .padding(25)
            )
            .style(iced::theme::Container::Box),
            
            Space::with_height(40),
            button(
                text("Start Using Guardian OS")
                    .size(18)
            )
            .on_press(Message::Complete)
            .padding([15, 40]),
        ]
        .spacing(5)
        .align_items(Alignment::Center)
        .into()
    }
}
