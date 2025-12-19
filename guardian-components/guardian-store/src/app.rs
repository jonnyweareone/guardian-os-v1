//! Main application for Guardian Store

use cosmic::app::{Core, Task};
use cosmic::iced::Subscription;
use cosmic::widget::{self, nav_bar};
use cosmic::{Application, Element};

use crate::catalog::{AppEntry, Category};
use crate::daemon::DaemonClient;
use crate::ratings::AgeRating;

/// Main application state
pub struct GuardianStore {
    core: Core,
    nav: nav_bar::Model,
    current_page: Page,
    
    // Daemon client for policy checks
    daemon: DaemonClient,
    
    // Catalog
    apps: Vec<AppEntry>,
    categories: Vec<Category>,
    search_query: String,
    search_results: Vec<AppEntry>,
    
    // Child info (from daemon)
    child_age: Option<u32>,
    allowed_rating: AgeRating,
    is_parent_mode: bool,
    
    // UI state
    selected_app: Option<AppEntry>,
    installing: Option<String>,
    install_progress: f32,
    pending_requests: Vec<AppRequest>,
    
    // PIN dialog
    pin_dialog_open: bool,
    pin_input: String,
    pin_action: Option<PinAction>,
    
    loading: bool,
    error: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Page {
    Home,
    Category(String),
    Search,
    Installed,
    Requests,
    AppDetails(String),
}

#[derive(Debug, Clone)]
pub struct AppRequest {
    pub id: String,
    pub app_id: String,
    pub app_name: String,
    pub requested_at: String,
    pub status: String,
}

#[derive(Debug, Clone)]
pub enum PinAction {
    Install(String),
    ApproveRequest(String),
}

#[derive(Debug, Clone)]
pub enum Message {
    // Navigation
    NavSelect(nav_bar::Id),
    GoHome,
    SelectCategory(String),
    SelectApp(AppEntry),
    BackToList,
    
    // Search
    SearchChanged(String),
    Search,
    
    // Catalog loading
    LoadCatalog,
    CatalogLoaded(Result<Vec<AppEntry>, String>),
    
    // Installation
    InstallApp(String),
    RequestApp(String),
    CancelInstall,
    InstallProgress(String, f32),
    InstallComplete(Result<(), String>),
    
    // Requests (parent view)
    LoadRequests,
    RequestsLoaded(Result<Vec<AppRequest>, String>),
    ApproveRequest(String),
    DenyRequest(String),
    RequestActioned(Result<(), String>),
    
    // PIN dialog
    OpenPinDialog(PinAction),
    ClosePinDialog,
    PinChanged(String),
    SubmitPin,
    PinVerified(bool),
    
    // Daemon
    DaemonConnected,
    PolicyUpdated(AgeRating, bool),
    
    // Misc
    OpenUrl(String),
    Error(String),
    ClearError,
    Tick,
}

impl Application for GuardianStore {
    type Executor = cosmic::executor::Default;
    type Flags = ();
    type Message = Message;

    const APP_ID: &'static str = "ai.guardian.store";

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    fn init(core: Core, _flags: Self::Flags) -> (Self, Task<Self::Message>) {
        let mut nav = nav_bar::Model::default();
        
        nav.insert()
            .text("Home")
            .icon(widget::icon::from_name("go-home-symbolic"))
            .data(Page::Home);
        
        nav.insert()
            .text("Installed")
            .icon(widget::icon::from_name("emblem-ok-symbolic"))
            .data(Page::Installed);
        
        nav.insert()
            .text("Requests")
            .icon(widget::icon::from_name("mail-unread-symbolic"))
            .data(Page::Requests);
        
        nav.activate_position(0);

        let app = Self {
            core,
            nav,
            current_page: Page::Home,
            daemon: DaemonClient::new(),
            apps: Vec::new(),
            categories: Vec::new(),
            search_query: String::new(),
            search_results: Vec::new(),
            child_age: None,
            allowed_rating: AgeRating::Everyone,
            is_parent_mode: false,
            selected_app: None,
            installing: None,
            install_progress: 0.0,
            pending_requests: Vec::new(),
            pin_dialog_open: false,
            pin_input: String::new(),
            pin_action: None,
            loading: true,
            error: None,
        };

        (app, Task::perform(async {}, |_| Message::LoadCatalog))
    }

    fn nav_model(&self) -> Option<&nav_bar::Model> {
        Some(&self.nav)
    }

    fn on_nav_select(&mut self, id: nav_bar::Id) -> Task<Self::Message> {
        self.nav.activate(id);
        if let Some(page) = self.nav.data::<Page>(id) {
            self.current_page = page.clone();
            self.selected_app = None;
        }
        Task::none()
    }

    fn update(&mut self, message: Self::Message) -> Task<Self::Message> {
        match message {
            Message::NavSelect(id) => self.on_nav_select(id),
            
            Message::GoHome => {
                self.current_page = Page::Home;
                self.selected_app = None;
                Task::none()
            }
            
            Message::SelectCategory(cat) => {
                self.current_page = Page::Category(cat);
                Task::none()
            }
            
            Message::SelectApp(app) => {
                self.selected_app = Some(app.clone());
                self.current_page = Page::AppDetails(app.id);
                Task::none()
            }
            
            Message::BackToList => {
                self.selected_app = None;
                self.current_page = Page::Home;
                Task::none()
            }
            
            Message::SearchChanged(query) => {
                self.search_query = query;
                Task::none()
            }
            
            Message::Search => {
                if self.search_query.is_empty() {
                    self.current_page = Page::Home;
                } else {
                    self.current_page = Page::Search;
                    let query = self.search_query.clone().to_lowercase();
                    self.search_results = self.apps.iter()
                        .filter(|app| {
                            app.name.to_lowercase().contains(&query) ||
                            app.summary.as_ref().map(|s| s.to_lowercase().contains(&query)).unwrap_or(false)
                        })
                        .filter(|app| self.is_rating_allowed(&app.rating))
                        .take(50)
                        .cloned()
                        .collect();
                }
                Task::none()
            }
            
            Message::LoadCatalog => {
                self.loading = true;
                Task::perform(
                    crate::catalog::load_catalog(),
                    |r| Message::CatalogLoaded(r.map_err(|e| e.to_string())),
                )
            }
            
            Message::CatalogLoaded(result) => {
                self.loading = false;
                match result {
                    Ok(apps) => self.apps = apps,
                    Err(e) => self.error = Some(e),
                }
                Task::none()
            }
            
            Message::InstallApp(app_id) => {
                // Check if we need PIN (child mode)
                if !self.is_parent_mode {
                    self.pin_action = Some(PinAction::Install(app_id));
                    self.pin_dialog_open = true;
                    self.pin_input.clear();
                } else {
                    self.start_install(&app_id);
                }
                Task::none()
            }
            
            Message::RequestApp(app_id) => {
                // Child requests app, parent must approve
                log::info!("App requested by child: {}", app_id);
                // TODO: Create request via Supabase API
                Task::none()
            }
            
            Message::CancelInstall => {
                self.installing = None;
                self.install_progress = 0.0;
                Task::none()
            }
            
            Message::InstallProgress(app_id, progress) => {
                if self.installing.as_ref() == Some(&app_id) {
                    self.install_progress = progress;
                }
                Task::none()
            }
            
            Message::InstallComplete(result) => {
                self.installing = None;
                self.install_progress = 0.0;
                if let Err(e) = result {
                    self.error = Some(e);
                }
                Task::none()
            }
            
            Message::LoadRequests => {
                // Load pending app requests from Supabase
                Task::none()
            }
            
            Message::RequestsLoaded(result) => {
                match result {
                    Ok(requests) => self.pending_requests = requests,
                    Err(e) => self.error = Some(e),
                }
                Task::none()
            }
            
            Message::ApproveRequest(request_id) => {
                self.pin_action = Some(PinAction::ApproveRequest(request_id));
                self.pin_dialog_open = true;
                self.pin_input.clear();
                Task::none()
            }
            
            Message::DenyRequest(request_id) => {
                // TODO: Update request status via API
                log::info!("Request denied: {}", request_id);
                Task::none()
            }
            
            Message::RequestActioned(result) => {
                if let Err(e) = result {
                    self.error = Some(e);
                }
                // Reload requests
                Task::perform(async {}, |_| Message::LoadRequests)
            }
            
            Message::OpenPinDialog(action) => {
                self.pin_action = Some(action);
                self.pin_dialog_open = true;
                self.pin_input.clear();
                Task::none()
            }
            
            Message::ClosePinDialog => {
                self.pin_dialog_open = false;
                self.pin_input.clear();
                self.pin_action = None;
                Task::none()
            }
            
            Message::PinChanged(pin) => {
                // Only allow digits, max 6
                self.pin_input = pin.chars()
                    .filter(|c| c.is_ascii_digit())
                    .take(6)
                    .collect();
                Task::none()
            }
            
            Message::SubmitPin => {
                let pin = self.pin_input.clone();
                let daemon = self.daemon.clone();
                Task::perform(
                    async move { daemon.verify_pin(&pin).await },
                    Message::PinVerified,
                )
            }
            
            Message::PinVerified(valid) => {
                if valid {
                    self.pin_dialog_open = false;
                    if let Some(action) = self.pin_action.take() {
                        match action {
                            PinAction::Install(app_id) => {
                                self.start_install(&app_id);
                            }
                            PinAction::ApproveRequest(request_id) => {
                                // TODO: Approve request
                                log::info!("Request approved: {}", request_id);
                            }
                        }
                    }
                } else {
                    self.error = Some("Incorrect PIN".to_string());
                }
                self.pin_input.clear();
                Task::none()
            }
            
            Message::DaemonConnected => {
                log::info!("Connected to guardian-daemon");
                Task::none()
            }
            
            Message::PolicyUpdated(rating, is_parent) => {
                self.allowed_rating = rating;
                self.is_parent_mode = is_parent;
                Task::none()
            }
            
            Message::OpenUrl(url) => {
                let _ = open::that(&url);
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
            
            Message::Tick => {
                Task::none()
            }
        }
    }

    fn view(&self) -> Element<Self::Message> {
        use cosmic::iced::Length;
        use cosmic::widget::{button, column, container, row, text, text_input, Space};
        
        // PIN dialog overlay
        if self.pin_dialog_open {
            return self.view_pin_dialog();
        }
        
        // Search bar
        let search_bar = row![
            text_input("Search apps...", &self.search_query)
                .on_input(Message::SearchChanged)
                .on_submit(Message::Search)
                .width(300),
            button::text("Search")
                .on_press(Message::Search),
        ]
        .spacing(10)
        .padding(15);
        
        // Main content based on page
        let content: Element<_> = match &self.current_page {
            Page::Home => self.view_home(),
            Page::Category(cat) => self.view_category(cat),
            Page::Search => self.view_search_results(),
            Page::Installed => self.view_installed(),
            Page::Requests => self.view_requests(),
            Page::AppDetails(id) => self.view_app_details(id),
        };
        
        // Error banner
        let error_banner: Element<_> = if let Some(ref error) = self.error {
            container(
                row![
                    text(error),
                    Space::with_width(Length::Fill),
                    button::text("Dismiss").on_press(Message::ClearError),
                ]
            )
            .padding(10)
            .style(cosmic::theme::Container::custom(|_| {
                cosmic::iced_style::container::Style {
                    background: Some(cosmic::iced::Color::from_rgb(0.8, 0.2, 0.2).into()),
                    text_color: Some(cosmic::iced::Color::WHITE),
                    ..Default::default()
                }
            }))
            .into()
        } else {
            Space::with_height(0).into()
        };
        
        column![
            error_banner,
            search_bar,
            content,
        ]
        .into()
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        cosmic::iced::time::every(std::time::Duration::from_secs(30))
            .map(|_| Message::Tick)
    }
}

impl GuardianStore {
    fn is_rating_allowed(&self, rating: &AgeRating) -> bool {
        rating.numeric_value() <= self.allowed_rating.numeric_value()
    }
    
    fn start_install(&mut self, app_id: &str) {
        self.installing = Some(app_id.to_string());
        self.install_progress = 0.0;
        // TODO: Actually start flatpak install
        log::info!("Starting install: {}", app_id);
    }
    
    fn view_pin_dialog(&self) -> Element<Message> {
        use cosmic::iced::Length;
        use cosmic::widget::{button, column, container, row, text, text_input, Space};
        
        let action_text = match &self.pin_action {
            Some(PinAction::Install(id)) => format!("Enter PIN to install app"),
            Some(PinAction::ApproveRequest(_)) => "Enter PIN to approve request".to_string(),
            None => "Enter PIN".to_string(),
        };
        
        let dialog = container(
            column![
                text("Parental PIN Required").size(24),
                Space::with_height(15),
                text(&action_text),
                Space::with_height(20),
                text_input("PIN", &self.pin_input)
                    .on_input(Message::PinChanged)
                    .on_submit(Message::SubmitPin)
                    .password()
                    .width(200),
                Space::with_height(20),
                row![
                    button::text("Cancel")
                        .on_press(Message::ClosePinDialog),
                    Space::with_width(10),
                    button::text("Submit")
                        .on_press(Message::SubmitPin)
                        .style(cosmic::theme::Button::Suggested),
                ],
            ]
            .align_items(cosmic::iced::Alignment::Center)
            .padding(30)
        )
        .style(cosmic::theme::Container::Card)
        .width(350);
        
        container(dialog)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .style(cosmic::theme::Container::custom(|_| {
                cosmic::iced_style::container::Style {
                    background: Some(cosmic::iced::Color::from_rgba(0.0, 0.0, 0.0, 0.5).into()),
                    ..Default::default()
                }
            }))
            .into()
    }
    
    fn view_home(&self) -> Element<Message> {
        use cosmic::iced::Length;
        use cosmic::widget::{button, column, container, row, text, scrollable, Space};
        
        if self.loading {
            return container(text("Loading apps..."))
                .center_x(Length::Fill)
                .center_y(Length::Fill)
                .into();
        }
        
        // Guardian Approved section
        let approved_apps: Vec<Element<Message>> = self.apps.iter()
            .filter(|app| app.guardian_approved)
            .filter(|app| self.is_rating_allowed(&app.rating))
            .take(6)
            .map(|app| self.view_app_card(app))
            .collect();
        
        let approved_section = if !approved_apps.is_empty() {
            column![
                text("✅ Guardian Approved").size(20),
                Space::with_height(10),
                row::with_children(approved_apps).spacing(15),
            ]
            .spacing(5)
            .into()
        } else {
            Space::with_height(0).into()
        };
        
        // Categories
        let categories = vec![
            ("Games", "applications-games-symbolic"),
            ("Education", "accessories-dictionary-symbolic"),
            ("Creativity", "applications-graphics-symbolic"),
            ("Productivity", "applications-office-symbolic"),
        ];
        
        let category_buttons: Vec<Element<Message>> = categories.iter()
            .map(|(name, icon)| {
                button::custom(
                    column![
                        widget::icon::from_name(*icon).size(48),
                        text(*name),
                    ]
                    .align_items(cosmic::iced::Alignment::Center)
                    .spacing(5)
                )
                .on_press(Message::SelectCategory(name.to_string()))
                .padding(20)
                .into()
            })
            .collect();
        
        let categories_section = column![
            text("Categories").size(20),
            Space::with_height(10),
            row::with_children(category_buttons).spacing(15),
        ];
        
        // Recently added
        let recent_apps: Vec<Element<Message>> = self.apps.iter()
            .filter(|app| self.is_rating_allowed(&app.rating))
            .take(8)
            .map(|app| self.view_app_card(app))
            .collect();
        
        let recent_section = if !recent_apps.is_empty() {
            column![
                text("Recently Added").size(20),
                Space::with_height(10),
                row::with_children(recent_apps).spacing(15),
            ]
            .into()
        } else {
            Space::with_height(0).into()
        };
        
        scrollable(
            column![
                approved_section,
                Space::with_height(30),
                categories_section,
                Space::with_height(30),
                recent_section,
            ]
            .padding(20)
        )
        .into()
    }
    
    fn view_category(&self, category: &str) -> Element<Message> {
        use cosmic::widget::{button, column, row, text, scrollable, Space};
        
        let cat_lower = category.to_lowercase();
        let apps: Vec<Element<Message>> = self.apps.iter()
            .filter(|app| {
                app.categories.iter().any(|c| c.to_lowercase().contains(&cat_lower))
            })
            .filter(|app| self.is_rating_allowed(&app.rating))
            .map(|app| self.view_app_row(app))
            .collect();
        
        column![
            row![
                button::icon(widget::icon::from_name("go-previous-symbolic"))
                    .on_press(Message::GoHome),
                Space::with_width(10),
                text(category).size(24),
            ],
            Space::with_height(20),
            scrollable(
                column::with_children(apps).spacing(10)
            ),
        ]
        .padding(20)
        .into()
    }
    
    fn view_search_results(&self) -> Element<Message> {
        use cosmic::widget::{button, column, row, text, scrollable, Space};
        
        let apps: Vec<Element<Message>> = self.search_results.iter()
            .map(|app| self.view_app_row(app))
            .collect();
        
        let content = if apps.is_empty() {
            text("No apps found").into()
        } else {
            scrollable(
                column::with_children(apps).spacing(10)
            )
            .into()
        };
        
        column![
            row![
                button::icon(widget::icon::from_name("go-previous-symbolic"))
                    .on_press(Message::GoHome),
                Space::with_width(10),
                text(format!("Search: {}", self.search_query)).size(24),
            ],
            Space::with_height(20),
            content,
        ]
        .padding(20)
        .into()
    }
    
    fn view_installed(&self) -> Element<Message> {
        use cosmic::widget::{column, text, Space};
        
        column![
            text("Installed Apps").size(24),
            Space::with_height(20),
            text("TODO: Show installed flatpak apps"),
        ]
        .padding(20)
        .into()
    }
    
    fn view_requests(&self) -> Element<Message> {
        use cosmic::widget::{button, column, container, row, text, Space};
        use cosmic::iced::Length;
        
        if self.pending_requests.is_empty() {
            return column![
                text("App Requests").size(24),
                Space::with_height(40),
                container(
                    column![
                        text("📭").size(48),
                        Space::with_height(10),
                        text("No pending requests"),
                    ]
                    .align_items(cosmic::iced::Alignment::Center)
                )
                .center_x(Length::Fill),
            ]
            .padding(20)
            .into();
        }
        
        let requests: Vec<Element<Message>> = self.pending_requests.iter()
            .map(|req| {
                container(
                    row![
                        column![
                            text(&req.app_name).size(16),
                            text(&req.requested_at).size(12),
                        ],
                        Space::with_width(Length::Fill),
                        button::text("Approve")
                            .on_press(Message::ApproveRequest(req.id.clone()))
                            .style(cosmic::theme::Button::Suggested),
                        Space::with_width(10),
                        button::text("Deny")
                            .on_press(Message::DenyRequest(req.id.clone()))
                            .style(cosmic::theme::Button::Destructive),
                    ]
                    .align_items(cosmic::iced::Alignment::Center)
                )
                .padding(15)
                .style(cosmic::theme::Container::Card)
                .into()
            })
            .collect();
        
        column![
            text("App Requests").size(24),
            Space::with_height(20),
            column::with_children(requests).spacing(10),
        ]
        .padding(20)
        .into()
    }
    
    fn view_app_details(&self, _app_id: &str) -> Element<Message> {
        use cosmic::iced::Length;
        use cosmic::widget::{button, column, container, row, text, Space};
        
        if let Some(ref app) = self.selected_app {
            let rating_badge = self.view_rating_badge(&app.rating);
            
            let install_button: Element<Message> = if self.installing.as_ref() == Some(&app.id) {
                row![
                    text(format!("Installing... {:.0}%", self.install_progress * 100.0)),
                    Space::with_width(10),
                    button::text("Cancel").on_press(Message::CancelInstall),
                ]
                .into()
            } else if self.is_parent_mode {
                button::text("Install")
                    .on_press(Message::InstallApp(app.id.clone()))
                    .style(cosmic::theme::Button::Suggested)
                    .into()
            } else {
                button::text("Request App")
                    .on_press(Message::RequestApp(app.id.clone()))
                    .style(cosmic::theme::Button::Suggested)
                    .into()
            };
            
            column![
                row![
                    button::icon(widget::icon::from_name("go-previous-symbolic"))
                        .on_press(Message::BackToList),
                    Space::with_width(10),
                    text("App Details").size(24),
                ],
                Space::with_height(20),
                row![
                    container(text("📦").size(64))
                        .width(96)
                        .height(96)
                        .center_x(Length::Fill)
                        .center_y(Length::Fill),
                    Space::with_width(20),
                    column![
                        row![
                            text(&app.name).size(28),
                            Space::with_width(10),
                            if app.guardian_approved {
                                text("✅ Guardian Approved").size(12).into()
                            } else {
                                Space::with_width(0).into()
                            },
                        ],
                        Space::with_height(5),
                        text(app.developer.as_deref().unwrap_or("Unknown developer")).size(14),
                        Space::with_height(10),
                        rating_badge,
                        Space::with_height(15),
                        install_button,
                    ],
                ],
                Space::with_height(30),
                text("Description").size(18),
                Space::with_height(10),
                text(app.description.as_deref().unwrap_or("No description available")),
            ]
            .padding(20)
            .into()
        } else {
            text("App not found").into()
        }
    }
    
    fn view_app_card(&self, app: &AppEntry) -> Element<Message> {
        use cosmic::widget::{button, column, container, text};
        
        let rating_text = app.rating.short_label();
        
        button::custom(
            container(
                column![
                    text("📦").size(48),
                    text(&app.name).size(14),
                    text(rating_text).size(10),
                ]
                .align_items(cosmic::iced::Alignment::Center)
                .spacing(5)
            )
            .padding(15)
            .width(120)
        )
        .on_press(Message::SelectApp(app.clone()))
        .into()
    }
    
    fn view_app_row(&self, app: &AppEntry) -> Element<Message> {
        use cosmic::iced::Length;
        use cosmic::widget::{button, column, container, row, text, Space};
        
        let rating_badge = self.view_rating_badge(&app.rating);
        
        button::custom(
            container(
                row![
                    text("📦").size(32),
                    Space::with_width(15),
                    column![
                        row![
                            text(&app.name).size(16),
                            Space::with_width(10),
                            if app.guardian_approved {
                                text("✅").into()
                            } else {
                                Space::with_width(0).into()
                            },
                        ],
                        text(app.summary.as_deref().unwrap_or("")).size(12),
                    ],
                    Space::with_width(Length::Fill),
                    rating_badge,
                ]
                .align_items(cosmic::iced::Alignment::Center)
            )
            .padding(10)
        )
        .on_press(Message::SelectApp(app.clone()))
        .style(cosmic::theme::Button::Standard)
        .into()
    }
    
    fn view_rating_badge(&self, rating: &AgeRating) -> Element<Message> {
        use cosmic::widget::{container, text};
        
        let (label, color) = rating.badge_info();
        
        container(text(label).size(12))
            .padding([4, 8])
            .style(cosmic::theme::Container::custom(move |_| {
                cosmic::iced_style::container::Style {
                    background: Some(cosmic::iced::Color::from_rgb(color.0, color.1, color.2).into()),
                    text_color: Some(cosmic::iced::Color::WHITE),
                    border: cosmic::iced::Border {
                        radius: 4.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                }
            }))
            .into()
    }
}
