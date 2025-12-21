// Guardian OS - Protection Setup Page
// Shows daemon status and DNS configuration on first boot

use crate::fl;
use cosmic::{
    Apply, Element,
    iced::Length,
    widget::{self},
};
use std::sync::Arc;

#[derive(Clone, Debug, Default)]
pub struct Page {
    status: SetupStatus,
    dns_configured: bool,
    daemon_running: bool,
    filtering_level: FilteringLevel,
    error_message: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub enum SetupStatus {
    #[default]
    Checking,
    Configuring,
    Complete,
    Error,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum FilteringLevel {
    Strict,      // Block adult, social media, gaming, etc.
    #[default]
    Moderate,    // Block adult content, allow most else
    Permissive,  // Minimal blocking, monitoring only
}

impl FilteringLevel {
    pub fn description(&self) -> &'static str {
        match self {
            FilteringLevel::Strict => "Blocks adult content, social media, gaming sites, and videos. Best for younger children.",
            FilteringLevel::Moderate => "Blocks adult content and dangerous sites. Allows social media and videos with monitoring.",
            FilteringLevel::Permissive => "Minimal blocking. Monitors activity and sends reports. Best for teenagers.",
        }
    }
    
    pub fn label(&self) -> &'static str {
        match self {
            FilteringLevel::Strict => "Strict",
            FilteringLevel::Moderate => "Moderate", 
            FilteringLevel::Permissive => "Permissive",
        }
    }
}

#[derive(Clone, Debug)]
pub enum Message {
    CheckStatus,
    StatusResult(Arc<DaemonStatus>),
    SetFilteringLevel(FilteringLevel),
    ConfigureDns,
    DnsConfigured(bool),
    RestartDaemon,
    DaemonRestarted(bool),
}

#[derive(Clone, Debug)]
pub struct DaemonStatus {
    pub running: bool,
    pub dns_configured: bool,
    pub device_registered: bool,
    pub policy_synced: bool,
    pub last_sync: Option<String>,
}

impl From<Message> for super::Message {
    fn from(message: Message) -> Self {
        super::Message::GuardianProtection(message)
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
        fl!("guardian-protection-page")
    }

    fn completed(&self) -> bool {
        self.status == SetupStatus::Complete
    }

    fn init(&mut self) -> cosmic::Task<super::Message> {
        // Check daemon status on page load
        cosmic::Task::future(async {
            let status = check_daemon_status().await;
            Message::StatusResult(Arc::new(status))
        }).map(super::Message::GuardianProtection)
    }

    fn view(&self) -> Element<'_, super::Message> {
        let spacing = cosmic::theme::spacing();
        
        let title = widget::text::title3(fl!("guardian-protection-page", "title"))
            .apply(widget::container)
            .center_x(Length::Fill);

        let status_icon = match self.status {
            SetupStatus::Checking => "🔄",
            SetupStatus::Configuring => "⚙️",
            SetupStatus::Complete => "✅",
            SetupStatus::Error => "❌",
        };

        let status_text = match self.status {
            SetupStatus::Checking => fl!("guardian-protection-page", "checking"),
            SetupStatus::Configuring => fl!("guardian-protection-page", "configuring"),
            SetupStatus::Complete => fl!("guardian-protection-page", "complete"),
            SetupStatus::Error => fl!("guardian-protection-page", "error"),
        };

        let status_row = widget::row::with_capacity(2)
            .spacing(spacing.space_s)
            .push(widget::text::body(status_icon))
            .push(widget::text::body(status_text))
            .apply(widget::container)
            .center_x(Length::Fill);

        // Status indicators
        let daemon_status = widget::row::with_capacity(2)
            .spacing(spacing.space_xs)
            .push(widget::text::body(if self.daemon_running { "✓" } else { "○" }))
            .push(widget::text::body(fl!("guardian-protection-page", "daemon-status")));

        let dns_status = widget::row::with_capacity(2)
            .spacing(spacing.space_xs)
            .push(widget::text::body(if self.dns_configured { "✓" } else { "○" }))
            .push(widget::text::body(fl!("guardian-protection-page", "dns-status")));

        let status_list = widget::column::with_capacity(2)
            .spacing(spacing.space_xxs)
            .push(daemon_status)
            .push(dns_status)
            .apply(widget::container)
            .padding(spacing.space_s);

        // Filtering level selector (only show when complete)
        let filtering_section = if self.status == SetupStatus::Complete {
            let level_description = widget::text::body(self.filtering_level.description())
                .apply(widget::container)
                .padding(spacing.space_xs);

            let level_buttons = widget::row::with_capacity(3)
                .spacing(spacing.space_xs)
                .push(
                    widget::button::text(FilteringLevel::Strict.label())
                        .on_press(Message::SetFilteringLevel(FilteringLevel::Strict).into())
                        .class(if self.filtering_level == FilteringLevel::Strict {
                            cosmic::theme::Button::Suggested
                        } else {
                            cosmic::theme::Button::Standard
                        })
                )
                .push(
                    widget::button::text(FilteringLevel::Moderate.label())
                        .on_press(Message::SetFilteringLevel(FilteringLevel::Moderate).into())
                        .class(if self.filtering_level == FilteringLevel::Moderate {
                            cosmic::theme::Button::Suggested
                        } else {
                            cosmic::theme::Button::Standard
                        })
                )
                .push(
                    widget::button::text(FilteringLevel::Permissive.label())
                        .on_press(Message::SetFilteringLevel(FilteringLevel::Permissive).into())
                        .class(if self.filtering_level == FilteringLevel::Permissive {
                            cosmic::theme::Button::Suggested
                        } else {
                            cosmic::theme::Button::Standard
                        })
                )
                .apply(widget::container)
                .center_x(Length::Fill);

            Some(
                widget::column::with_capacity(3)
                    .spacing(spacing.space_s)
                    .push(widget::text::body(fl!("guardian-protection-page", "filtering-level")))
                    .push(level_buttons)
                    .push(level_description)
            )
        } else {
            None
        };

        // Error message
        let error_widget = self.error_message.as_ref().map(|msg| {
            widget::text::body(msg)
                .apply(widget::container)
                .center_x(Length::Fill)
        });

        // Build the page
        let mut column = widget::column::with_capacity(8)
            .spacing(spacing.space_m)
            .push(title)
            .push(widget::vertical_space().height(spacing.space_s))
            .push(status_row)
            .push(status_list);

        if let Some(filtering) = filtering_section {
            column = column.push(filtering);
        }

        if let Some(error) = error_widget {
            column = column.push(error);
        }

        column.into()
    }
}

impl Page {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn update(&mut self, message: Message) -> cosmic::Task<super::Message> {
        match message {
            Message::CheckStatus => {
                self.status = SetupStatus::Checking;
                return cosmic::Task::future(async {
                    let status = check_daemon_status().await;
                    Message::StatusResult(Arc::new(status))
                }).map(super::Message::GuardianProtection);
            }
            
            Message::StatusResult(status) => {
                self.daemon_running = status.running;
                self.dns_configured = status.dns_configured;
                
                if status.running && status.dns_configured {
                    self.status = SetupStatus::Complete;
                } else if !status.running {
                    // Try to start daemon
                    self.status = SetupStatus::Configuring;
                    return cosmic::Task::future(async {
                        let success = start_daemon().await;
                        Message::DaemonRestarted(success)
                    }).map(super::Message::GuardianProtection);
                } else if !status.dns_configured {
                    // Configure DNS
                    self.status = SetupStatus::Configuring;
                    return cosmic::Task::future(async {
                        let success = configure_dns().await;
                        Message::DnsConfigured(success)
                    }).map(super::Message::GuardianProtection);
                }
            }
            
            Message::SetFilteringLevel(level) => {
                self.filtering_level = level.clone();
                // Update policy via daemon
                return cosmic::Task::future(async move {
                    let _ = set_filtering_level(&level).await;
                    Message::CheckStatus
                }).map(super::Message::GuardianProtection);
            }
            
            Message::ConfigureDns => {
                self.status = SetupStatus::Configuring;
                return cosmic::Task::future(async {
                    let success = configure_dns().await;
                    Message::DnsConfigured(success)
                }).map(super::Message::GuardianProtection);
            }
            
            Message::DnsConfigured(success) => {
                self.dns_configured = success;
                if success {
                    return cosmic::Task::done(Message::CheckStatus.into());
                } else {
                    self.status = SetupStatus::Error;
                    self.error_message = Some(fl!("guardian-protection-page", "dns-error"));
                }
            }
            
            Message::RestartDaemon => {
                return cosmic::Task::future(async {
                    let success = start_daemon().await;
                    Message::DaemonRestarted(success)
                }).map(super::Message::GuardianProtection);
            }
            
            Message::DaemonRestarted(success) => {
                self.daemon_running = success;
                if success {
                    return cosmic::Task::done(Message::CheckStatus.into());
                } else {
                    self.status = SetupStatus::Error;
                    self.error_message = Some(fl!("guardian-protection-page", "daemon-error"));
                }
            }
        }
        cosmic::Task::none()
    }
}

// Helper functions to interact with guardian-daemon
async fn check_daemon_status() -> DaemonStatus {
    // Check if daemon is running via systemd
    let running = tokio::process::Command::new("systemctl")
        .args(["is-active", "--quiet", "guardian-daemon"])
        .status()
        .await
        .map(|s| s.success())
        .unwrap_or(false);

    // Check DNS configuration
    let dns_configured = check_dns_config().await;

    // Check device registration
    let device_registered = std::path::Path::new("/etc/guardian/device.conf").exists();

    DaemonStatus {
        running,
        dns_configured,
        device_registered,
        policy_synced: running && device_registered,
        last_sync: None,
    }
}

async fn check_dns_config() -> bool {
    // Check if resolv.conf points to Guardian DNS or if systemd-resolved is configured
    if let Ok(content) = tokio::fs::read_to_string("/etc/resolv.conf").await {
        // Check for common safe DNS servers (CleanBrowsing, OpenDNS Family, etc.)
        content.contains("185.228.168") || // CleanBrowsing
        content.contains("208.67.222.123") || // OpenDNS FamilyShield
        content.contains("127.0.0.1") // Local DNS proxy
    } else {
        false
    }
}

async fn configure_dns() -> bool {
    // Configure DNS via guardian-daemon CLI
    tokio::process::Command::new("guardian-cli")
        .args(["dns", "configure"])
        .status()
        .await
        .map(|s| s.success())
        .unwrap_or(false)
}

async fn start_daemon() -> bool {
    tokio::process::Command::new("systemctl")
        .args(["start", "guardian-daemon"])
        .status()
        .await
        .map(|s| s.success())
        .unwrap_or(false)
}

async fn set_filtering_level(level: &FilteringLevel) -> bool {
    let level_str = match level {
        FilteringLevel::Strict => "strict",
        FilteringLevel::Moderate => "moderate",
        FilteringLevel::Permissive => "permissive",
    };
    
    tokio::process::Command::new("guardian-cli")
        .args(["policy", "set-level", level_str])
        .status()
        .await
        .map(|s| s.success())
        .unwrap_or(false)
}
