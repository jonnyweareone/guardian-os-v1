//! Guardian Wizard - First-boot setup for Guardian OS
//!
//! This wizard runs on first boot to:
//! 1. Register the device with Guardian backend
//! 2. Display activation code for parent to approve
//! 3. Wait for parent approval
//! 4. Configure device for assigned child
//! 5. Start Guardian Daemon

mod api;
mod pages;
mod state;

use iced::{
    Application, Command, Element, Settings, Theme,
    widget::{button, column, container, image, row, text, text_input, Space},
    Length, Alignment,
};
use tracing::{info, error};

use api::GuardianApi;
use state::{WizardState, WizardPage};

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
    CreateFamily,
    FamilyCreated(Result<api::FamilyInfo, String>),
    
    // Child flow
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
                let api = self.api.clone();
                let email = self.state.email.clone();
                let password = self.state.password.clone();
                Command::perform(
                    async move { api.login(&email, &password).await },
                    |result| Message::LoginResult(result.map_err(|e| e.to_string())),
                )
            }
            
            Message::LoginResult(result) => {
                match result {
                    Ok(auth) => {
                        self.state.access_token = Some(auth.access_token);
                        self.state.next_page();
                    }
                    Err(e) => {
                        self.state.error = Some(format!("Login failed: {}", e));
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
            
            Message::CreateFamily => {
                // TODO: Implement create family flow
                Command::none()
            }
            
            Message::FamilyCreated(_) => Command::none(),
            
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
            // Logo would go here
            text("Welcome to Guardian OS")
                .size(40),
            Space::with_height(20),
            text("AI-powered safety for your family")
                .size(20),
            Space::with_height(40),
            text("Guardian OS keeps your children safe online while respecting their privacy.")
                .size(16),
            Space::with_height(20),
            text("• Smart screen time management")
                .size(14),
            text("• AI-powered content filtering")
                .size(14),
            text("• Real-time safety alerts")
                .size(14),
            text("• Works offline")
                .size(14),
            Space::with_height(40),
            button(text("Get Started"))
                .on_press(Message::NextPage)
                .padding(15),
        ]
        .spacing(10)
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
                        text("Set up a new family or\nadd this device to your family")
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
                .password()
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
                button(text("Sign In"))
                    .on_press(Message::Login),
            ],
            Space::with_height(20),
            text("Don't have an account?")
                .size(14),
            button(text("Create Account"))
                .on_press(Message::GoToPage(WizardPage::ParentLogin)), // TODO: SignUp page
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
            text("Or scan this QR code with the Guardian app:")
                .size(14),
            // TODO: QR code image
            Space::with_height(30),
            text("⏳ Waiting for parent approval...")
                .size(16),
        ]
        .spacing(10)
        .align_items(Alignment::Center)
        .into()
    }
    
    fn view_complete(&self) -> Element<Message> {
        column![
            text("🎉 All Set!")
                .size(40),
            Space::with_height(30),
            text("Guardian OS is now protecting this device.")
                .size(20),
            Space::with_height(20),
            if let Some(ref child_id) = self.state.child_id {
                text(format!("This device is assigned to: {}", child_id))
                    .size(16)
            } else {
                text("Device activated successfully")
                    .size(16)
            },
            Space::with_height(40),
            text("What's next:")
                .size(18),
            Space::with_height(10),
            text("• Screen time limits will be enforced")
                .size(14),
            text("• Content filtering is active")
                .size(14),
            text("• Activity reports are being sent to parents")
                .size(14),
            Space::with_height(40),
            button(text("Finish Setup"))
                .on_press(Message::Complete)
                .padding(15),
        ]
        .spacing(5)
        .align_items(Alignment::Center)
        .into()
    }
}
