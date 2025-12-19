//! Main application state and logic

use cosmic::app::{Core, Task};
use cosmic::iced::Subscription;
use cosmic::widget::{self, nav_bar};
use cosmic::{Application, ApplicationExt, Element, Theme};

use crate::api::{GuardianApi, Child, Device, Alert, ScreenTimePolicy, DnsProfile};
use crate::pages::{self, Page};

/// Main application state
pub struct GuardianSettings {
    core: Core,
    nav: nav_bar::Model,
    current_page: Page,
    
    // API client
    api: GuardianApi,
    
    // Data
    children: Vec<Child>,
    devices: Vec<Device>,
    alerts: Vec<Alert>,
    selected_child: Option<String>,
    
    // UI state
    loading: bool,
    error: Option<String>,
    
    // Edit states
    editing_screen_time: Option<ScreenTimePolicy>,
    editing_dns: Option<DnsProfile>,
}

#[derive(Debug, Clone)]
pub enum Message {
    // Navigation
    NavSelect(nav_bar::Id),
    SelectChild(String),
    
    // Data loading
    LoadData,
    ChildrenLoaded(Result<Vec<Child>, String>),
    DevicesLoaded(Result<Vec<Device>, String>),
    AlertsLoaded(Result<Vec<Alert>, String>),
    
    // Screen time
    EditScreenTime(String),  // child_id
    ScreenTimeLoaded(Result<ScreenTimePolicy, String>),
    UpdateWeekdayLimit(i32),
    UpdateWeekendLimit(i32),
    UpdateBedtime(String),
    UpdateBedtimeEnabled(bool),
    SaveScreenTime,
    ScreenTimeSaved(Result<(), String>),
    
    // Content filtering
    EditDnsProfile(String),  // child_id
    DnsProfileLoaded(Result<DnsProfile, String>),
    UpdateFilterLevel(String),
    ToggleBlockAdult(bool),
    ToggleBlockGambling(bool),
    ToggleBlockSocialMedia(bool),
    ToggleSafeSearch(bool),
    AddBlockedDomain(String),
    RemoveBlockedDomain(String),
    SaveDnsProfile,
    DnsProfileSaved(Result<(), String>),
    
    // Devices
    RefreshDevices,
    SendCommand(String, String),  // device_id, command
    CommandSent(Result<(), String>),
    
    // Alerts
    RefreshAlerts,
    DismissAlert(String),
    AlertDismissed(Result<(), String>),
    
    // Misc
    Tick,
    Error(String),
    ClearError,
}

impl Application for GuardianSettings {
    type Executor = cosmic::executor::Default;
    type Flags = ();
    type Message = Message;

    const APP_ID: &'static str = "ai.guardian.settings";

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    fn init(core: Core, _flags: Self::Flags) -> (Self, Task<Self::Message>) {
        let mut nav = nav_bar::Model::default();
        
        nav.insert()
            .text("Family")
            .icon(widget::icon::from_name("system-users-symbolic"))
            .data(Page::Family);
        
        nav.insert()
            .text("Screen Time")
            .icon(widget::icon::from_name("preferences-system-time-symbolic"))
            .data(Page::ScreenTime);
        
        nav.insert()
            .text("Content Filter")
            .icon(widget::icon::from_name("security-high-symbolic"))
            .data(Page::ContentFilter);
        
        nav.insert()
            .text("Devices")
            .icon(widget::icon::from_name("computer-symbolic"))
            .data(Page::Devices);
        
        nav.insert()
            .text("Alerts")
            .icon(widget::icon::from_name("dialog-warning-symbolic"))
            .data(Page::Alerts);
        
        nav.activate_position(0);

        let app = Self {
            core,
            nav,
            current_page: Page::Family,
            api: GuardianApi::new(),
            children: Vec::new(),
            devices: Vec::new(),
            alerts: Vec::new(),
            selected_child: None,
            loading: false,
            error: None,
            editing_screen_time: None,
            editing_dns: None,
        };

        (app, Task::perform(async {}, |_| Message::LoadData))
    }

    fn nav_model(&self) -> Option<&nav_bar::Model> {
        Some(&self.nav)
    }

    fn on_nav_select(&mut self, id: nav_bar::Id) -> Task<Self::Message> {
        self.nav.activate(id);
        if let Some(page) = self.nav.data::<Page>(id) {
            self.current_page = page.clone();
        }
        Task::none()
    }

    fn update(&mut self, message: Self::Message) -> Task<Self::Message> {
        match message {
            Message::NavSelect(id) => self.on_nav_select(id),
            
            Message::SelectChild(child_id) => {
                self.selected_child = Some(child_id);
                Task::none()
            }
            
            Message::LoadData => {
                self.loading = true;
                let api = self.api.clone();
                Task::perform(
                    async move { api.get_children().await },
                    |r| Message::ChildrenLoaded(r.map_err(|e| e.to_string())),
                )
            }
            
            Message::ChildrenLoaded(result) => {
                self.loading = false;
                match result {
                    Ok(children) => {
                        if self.selected_child.is_none() && !children.is_empty() {
                            self.selected_child = Some(children[0].id.clone());
                        }
                        self.children = children;
                        // Chain load devices
                        let api = self.api.clone();
                        Task::perform(
                            async move { api.get_devices().await },
                            |r| Message::DevicesLoaded(r.map_err(|e| e.to_string())),
                        )
                    }
                    Err(e) => {
                        self.error = Some(e);
                        Task::none()
                    }
                }
            }
            
            Message::DevicesLoaded(result) => {
                match result {
                    Ok(devices) => {
                        self.devices = devices;
                        // Chain load alerts
                        let api = self.api.clone();
                        Task::perform(
                            async move { api.get_alerts().await },
                            |r| Message::AlertsLoaded(r.map_err(|e| e.to_string())),
                        )
                    }
                    Err(e) => {
                        self.error = Some(e);
                        Task::none()
                    }
                }
            }
            
            Message::AlertsLoaded(result) => {
                match result {
                    Ok(alerts) => self.alerts = alerts,
                    Err(e) => self.error = Some(e),
                }
                Task::none()
            }
            
            Message::EditScreenTime(child_id) => {
                let api = self.api.clone();
                Task::perform(
                    async move { api.get_screen_time_policy(&child_id).await },
                    |r| Message::ScreenTimeLoaded(r.map_err(|e| e.to_string())),
                )
            }
            
            Message::ScreenTimeLoaded(result) => {
                match result {
                    Ok(policy) => self.editing_screen_time = Some(policy),
                    Err(e) => self.error = Some(e),
                }
                Task::none()
            }
            
            Message::UpdateWeekdayLimit(mins) => {
                if let Some(ref mut policy) = self.editing_screen_time {
                    policy.weekday_limit_mins = Some(mins);
                }
                Task::none()
            }
            
            Message::UpdateWeekendLimit(mins) => {
                if let Some(ref mut policy) = self.editing_screen_time {
                    policy.weekend_limit_mins = Some(mins);
                }
                Task::none()
            }
            
            Message::UpdateBedtime(time) => {
                if let Some(ref mut policy) = self.editing_screen_time {
                    policy.bedtime_time = Some(time);
                }
                Task::none()
            }
            
            Message::UpdateBedtimeEnabled(enabled) => {
                if let Some(ref mut policy) = self.editing_screen_time {
                    policy.bedtime_enabled = Some(enabled);
                }
                Task::none()
            }
            
            Message::SaveScreenTime => {
                if let Some(ref policy) = self.editing_screen_time {
                    let api = self.api.clone();
                    let policy = policy.clone();
                    Task::perform(
                        async move { api.update_screen_time_policy(&policy).await },
                        |r| Message::ScreenTimeSaved(r.map_err(|e| e.to_string())),
                    )
                } else {
                    Task::none()
                }
            }
            
            Message::ScreenTimeSaved(result) => {
                match result {
                    Ok(()) => self.editing_screen_time = None,
                    Err(e) => self.error = Some(e),
                }
                Task::none()
            }
            
            Message::EditDnsProfile(child_id) => {
                let api = self.api.clone();
                Task::perform(
                    async move { api.get_dns_profile(&child_id).await },
                    |r| Message::DnsProfileLoaded(r.map_err(|e| e.to_string())),
                )
            }
            
            Message::DnsProfileLoaded(result) => {
                match result {
                    Ok(profile) => self.editing_dns = Some(profile),
                    Err(e) => self.error = Some(e),
                }
                Task::none()
            }
            
            Message::UpdateFilterLevel(level) => {
                if let Some(ref mut profile) = self.editing_dns {
                    profile.filter_level = Some(level);
                }
                Task::none()
            }
            
            Message::ToggleBlockAdult(v) => {
                if let Some(ref mut profile) = self.editing_dns {
                    profile.block_adult = Some(v);
                }
                Task::none()
            }
            
            Message::ToggleBlockGambling(v) => {
                if let Some(ref mut profile) = self.editing_dns {
                    profile.block_gambling = Some(v);
                }
                Task::none()
            }
            
            Message::ToggleBlockSocialMedia(v) => {
                if let Some(ref mut profile) = self.editing_dns {
                    profile.block_social_media = Some(v);
                }
                Task::none()
            }
            
            Message::ToggleSafeSearch(v) => {
                if let Some(ref mut profile) = self.editing_dns {
                    profile.enforce_safe_search = Some(v);
                }
                Task::none()
            }
            
            Message::AddBlockedDomain(domain) => {
                if let Some(ref mut profile) = self.editing_dns {
                    let mut domains = profile.blocked_domains.clone().unwrap_or_default();
                    if !domains.contains(&domain) {
                        domains.push(domain);
                        profile.blocked_domains = Some(domains);
                    }
                }
                Task::none()
            }
            
            Message::RemoveBlockedDomain(domain) => {
                if let Some(ref mut profile) = self.editing_dns {
                    if let Some(ref mut domains) = profile.blocked_domains {
                        domains.retain(|d| d != &domain);
                    }
                }
                Task::none()
            }
            
            Message::SaveDnsProfile => {
                if let Some(ref profile) = self.editing_dns {
                    let api = self.api.clone();
                    let profile = profile.clone();
                    Task::perform(
                        async move { api.update_dns_profile(&profile).await },
                        |r| Message::DnsProfileSaved(r.map_err(|e| e.to_string())),
                    )
                } else {
                    Task::none()
                }
            }
            
            Message::DnsProfileSaved(result) => {
                match result {
                    Ok(()) => self.editing_dns = None,
                    Err(e) => self.error = Some(e),
                }
                Task::none()
            }
            
            Message::RefreshDevices => {
                let api = self.api.clone();
                Task::perform(
                    async move { api.get_devices().await },
                    |r| Message::DevicesLoaded(r.map_err(|e| e.to_string())),
                )
            }
            
            Message::SendCommand(device_id, command) => {
                let api = self.api.clone();
                Task::perform(
                    async move { api.send_device_command(&device_id, &command).await },
                    |r| Message::CommandSent(r.map_err(|e| e.to_string())),
                )
            }
            
            Message::CommandSent(result) => {
                if let Err(e) = result {
                    self.error = Some(e);
                }
                Task::none()
            }
            
            Message::RefreshAlerts => {
                let api = self.api.clone();
                Task::perform(
                    async move { api.get_alerts().await },
                    |r| Message::AlertsLoaded(r.map_err(|e| e.to_string())),
                )
            }
            
            Message::DismissAlert(alert_id) => {
                let api = self.api.clone();
                Task::perform(
                    async move { api.dismiss_alert(&alert_id).await },
                    |r| Message::AlertDismissed(r.map_err(|e| e.to_string())),
                )
            }
            
            Message::AlertDismissed(result) => {
                match result {
                    Ok(()) => {
                        // Refresh alerts
                        let api = self.api.clone();
                        Task::perform(
                            async move { api.get_alerts().await },
                            |r| Message::AlertsLoaded(r.map_err(|e| e.to_string())),
                        )
                    }
                    Err(e) => {
                        self.error = Some(e);
                        Task::none()
                    }
                }
            }
            
            Message::Tick => {
                // Periodic refresh
                Task::none()
            }
            
            Message::Error(e) => {
                self.error = Some(e);
                Task::none()
            }
            
            Message::ClearError => {
                self.error = None;
                Task::none()
            }
        }
    }

    fn view(&self) -> Element<Self::Message> {
        let content: Element<_> = match self.current_page {
            Page::Family => pages::family::view(self),
            Page::ScreenTime => pages::screen_time::view(self),
            Page::ContentFilter => pages::content_filter::view(self),
            Page::Devices => pages::devices::view(self),
            Page::Alerts => pages::alerts::view(self),
        };

        // Wrap in error banner if needed
        if let Some(ref error) = self.error {
            widget::column::with_children(vec![
                widget::container(
                    widget::row::with_children(vec![
                        widget::text(error).into(),
                        widget::horizontal_space().into(),
                        widget::button::text("Dismiss")
                            .on_press(Message::ClearError)
                            .into(),
                    ])
                )
                .padding(10)
                .style(cosmic::theme::Container::custom(|theme| {
                    cosmic::iced_style::container::Style {
                        background: Some(cosmic::iced::Color::from_rgb(0.8, 0.2, 0.2).into()),
                        text_color: Some(cosmic::iced::Color::WHITE),
                        ..Default::default()
                    }
                }))
                .into(),
                content,
            ])
            .into()
        } else {
            content
        }
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        // Refresh data every 30 seconds
        cosmic::iced::time::every(std::time::Duration::from_secs(30))
            .map(|_| Message::Tick)
    }
}

// Accessor methods for pages
impl GuardianSettings {
    pub fn children(&self) -> &[Child] {
        &self.children
    }
    
    pub fn devices(&self) -> &[Device] {
        &self.devices
    }
    
    pub fn alerts(&self) -> &[Alert] {
        &self.alerts
    }
    
    pub fn selected_child(&self) -> Option<&str> {
        self.selected_child.as_deref()
    }
    
    pub fn selected_child_data(&self) -> Option<&Child> {
        self.selected_child.as_ref()
            .and_then(|id| self.children.iter().find(|c| &c.id == id))
    }
    
    pub fn editing_screen_time(&self) -> Option<&ScreenTimePolicy> {
        self.editing_screen_time.as_ref()
    }
    
    pub fn editing_dns(&self) -> Option<&DnsProfile> {
        self.editing_dns.as_ref()
    }
    
    pub fn is_loading(&self) -> bool {
        self.loading
    }
}
