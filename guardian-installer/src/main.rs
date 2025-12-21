// Guardian OS Installer
// Copyright 2024 Guardian Network Solutions
// SPDX-License-Identifier: GPL-3.0-only

use std::any::TypeId;
use std::path::Path;

use cosmic::{
    Application, Apply, Element,
    app::{Core, Settings, Task},
    cosmic_theme, executor,
    iced::{Alignment, Length, Limits, Subscription},
    theme, widget,
};
use futures::{SinkExt, Stream, StreamExt};
use indexmap::IndexMap;
use tracing_subscriber::prelude::*;

mod accessibility;
mod greeter;
mod localize;

use self::page::Page;
mod page;

const GUARDIAN_SETUP_DONE_PATH: &str = ".config/guardian-setup-done";

/// Runs application with these settings
#[rustfmt::skip]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Check for disable file (used during live session)
    if let Some(file_path) = option_env!("DISABLE_IF_EXISTS") {
        if Path::new(file_path).exists() {
            return Ok(());
        }
    }

    #[allow(deprecated)]
    let home_dir = std::env::home_dir().unwrap();

    // Check if setup already completed
    if home_dir.join(GUARDIAN_SETUP_DONE_PATH).exists() {
        return Ok(());
    }

    // Setup logging
    let log_level = std::env::var("RUST_LOG")
        .ok()
        .and_then(|level| level.parse::<tracing::Level>().ok())
        .unwrap_or(tracing::Level::INFO);

    let log_format = tracing_subscriber::fmt::format()
        .pretty()
        .without_time()
        .with_line_number(true)
        .with_file(true)
        .with_target(false)
        .with_thread_names(true);

    let log_filter = tracing_subscriber::fmt::Layer::default()
        .with_writer(std::io::stderr)
        .event_format(log_format)
        .with_filter(tracing_subscriber::filter::filter_fn(move |metadata| {
            let target = metadata.target();
            metadata.level() == &tracing::Level::ERROR
                || (target.starts_with("guardian_installer")
                    && metadata.level() <= &log_level)
        }));

    tracing_subscriber::registry().with(log_filter).init();

    localize::localize();

    // Determine which mode we're running in
    let args: Vec<String> = std::env::args().collect();
    let mode = determine_mode(&args);

    let mut settings = Settings::default();
    settings = settings.size_limits(Limits::NONE.max_width(900.0).max_height(650.0));

    cosmic::app::run::<App>(settings, mode)?;

    Ok(())
}

/// Determine which mode to run based on args and environment
fn determine_mode(args: &[String]) -> page::AppMode {
    // Check for explicit mode flags
    if args.contains(&"--first-boot".to_string()) {
        return page::AppMode::PostInstall;
    }
    
    if args.contains(&"--live-install".to_string()) {
        return page::AppMode::NewInstall { create_user: true };
    }

    // Check for live session marker
    if Path::new("/etc/guardian/installer.conf").exists() {
        // Read the config to check if we're in live session
        if let Ok(content) = std::fs::read_to_string("/etc/guardian/installer.conf") {
            if content.contains("LIVE_SESSION=true") {
                return page::AppMode::NewInstall { create_user: true };
            }
        }
    }

    // Check if running as guardian-installer user (OEM mode)
    let is_installer_user = pwd::Passwd::current_user()
        .map_or(false, |user| user.name == "guardian-installer");
    
    if is_installer_user {
        return page::AppMode::NewInstall { create_user: true };
    }

    // Default: post-install first boot wizard
    page::AppMode::PostInstall
}

/// Messages that are used specifically by our [`App`].
#[derive(Clone, Debug)]
pub enum Message {
    None,
    Exit,
    Finish,
    PageMessage(page::Message),
    PageOpen(usize),
}

/// The [`App`] stores application-specific state.
pub struct App {
    core: Core,
    pages: IndexMap<TypeId, Box<dyn Page + 'static>>,
    page_i: usize,
    is_live_install: bool,
    wifi_exists: bool,
}

/// Implement [`Application`] to integrate with COSMIC.
impl Application for App {
    type Executor = executor::Default;
    type Flags = page::AppMode;
    type Message = Message;

    const APP_ID: &'static str = "com.guardian.GuardianInstaller";

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    fn init(mut core: Core, mode: Self::Flags) -> (Self, Task<Message>) {
        core.window.show_headerbar = false;
        core.window.show_close = false;
        core.window.show_maximize = false;
        core.window.show_minimize = false;

        let is_live_install = matches!(mode, page::AppMode::NewInstall { .. });

        let mut app = App {
            core,
            is_live_install,
            pages: page::pages(mode),
            page_i: 0,
            wifi_exists: true,
        };

        let tasks = app
            .pages
            .values_mut()
            .map(|page| {
                page.init()
                    .map(Message::PageMessage)
                    .map(cosmic::Action::App)
            })
            .collect::<Vec<_>>()
            .apply(Task::batch)
            .chain(app.update(Message::PageOpen(0)));

        (app, tasks)
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::None => {}

            Message::PageMessage(page_message) => {
                match page_message {
                    page::Message::SetTheme(theme) => {
                        return cosmic::command::set_theme(theme);
                    }

                    page::Message::Appearance(message) => {
                        if let Some(page) = self.pages.get_mut(&TypeId::of::<page::appearance::Page>()) {
                            return page
                                .as_any()
                                .downcast_mut::<page::appearance::Page>()
                                .unwrap()
                                .update(message)
                                .map(Message::PageMessage)
                                .map(cosmic::Action::App);
                        }
                    }

                    page::Message::Keyboard(message) => {
                        if let Some(page) = self.pages.get_mut(&TypeId::of::<page::keyboard::Page>()) {
                            return page
                                .as_any()
                                .downcast_mut::<page::keyboard::Page>()
                                .unwrap()
                                .update(message)
                                .map(Message::PageMessage)
                                .map(cosmic::Action::App);
                        }
                    }

                    page::Message::Language(message) => {
                        if let Some(page) = self.pages.get_mut(&TypeId::of::<page::language::Page>()) {
                            return page
                                .as_any()
                                .downcast_mut::<page::language::Page>()
                                .unwrap()
                                .update(message)
                                .map(Message::PageMessage)
                                .map(cosmic::Action::App);
                        }
                    }

                    page::Message::Layout(message) => {
                        if let Some(page) = self.pages.get_mut(&TypeId::of::<page::layout::Page>()) {
                            return page
                                .as_any()
                                .downcast_mut::<page::layout::Page>()
                                .unwrap()
                                .update(message)
                                .map(Message::PageMessage)
                                .map(cosmic::Action::App);
                        }
                    }

                    page::Message::User(message) => {
                        if let Some(page) = self.pages.get_mut(&TypeId::of::<page::user::Page>()) {
                            return page
                                .as_any()
                                .downcast_mut::<page::user::Page>()
                                .unwrap()
                                .update(message)
                                .map(Message::PageMessage)
                                .map(cosmic::Action::App);
                        }
                    }

                    page::Message::Welcome(message) => {
                        if let Some(page) = self.pages.get_mut(&TypeId::of::<page::welcome::Page>()) {
                            return page
                                .as_any()
                                .downcast_mut::<page::welcome::Page>()
                                .unwrap()
                                .update(message)
                                .map(Message::PageMessage)
                                .map(cosmic::Action::App);
                        }
                    }

                    page::Message::WiFi(message) => {
                        if let Some(page) = self.pages.get_mut(&TypeId::of::<page::wifi::Page>()) {
                            return page
                                .as_any()
                                .downcast_mut::<page::wifi::Page>()
                                .unwrap()
                                .update(message)
                                .map(Message::PageMessage)
                                .map(cosmic::Action::App);
                        }
                    }

                    page::Message::GuardianAuth(message) => {
                        if let Some(page) = self.pages.get_mut(&TypeId::of::<page::guardian_auth::Page>()) {
                            let auth_page = page
                                .as_any()
                                .downcast_mut::<page::guardian_auth::Page>()
                                .unwrap();
                            
                            let result = auth_page.update(message);
                            
                            // Pass auth context to child page when auth completes
                            if let (Some(token), Some(parent_id)) = 
                                (auth_page.access_token.clone(), auth_page.parent_id.clone()) 
                            {
                                if let Some(child_page) = self.pages.get_mut(&TypeId::of::<page::guardian_child::Page>()) {
                                    child_page
                                        .as_any()
                                        .downcast_mut::<page::guardian_child::Page>()
                                        .unwrap()
                                        .set_auth_context(token, parent_id);
                                }
                            }
                            
                            return result
                                .map(Message::PageMessage)
                                .map(cosmic::Action::App);
                        }
                    }

                    page::Message::GuardianChild(message) => {
                        if let Some(page) = self.pages.get_mut(&TypeId::of::<page::guardian_child::Page>()) {
                            let child_page = page
                                .as_any()
                                .downcast_mut::<page::guardian_child::Page>()
                                .unwrap();
                            
                            let result = child_page.update(message);
                            
                            // Pass context to sync page if device is claimed
                            if child_page.device_claimed {
                                if let (Some(token), Some(parent_id), Some(device_id)) = (
                                    child_page.access_token.clone(),
                                    child_page.parent_id.clone(),
                                    child_page.device_id.clone(),
                                ) {
                                    if let Some(sync_page) = self.pages.get_mut(&TypeId::of::<page::guardian_sync::Page>()) {
                                        sync_page
                                            .as_any()
                                            .downcast_mut::<page::guardian_sync::Page>()
                                            .unwrap()
                                            .set_context(token, parent_id, device_id);
                                    }
                                }
                            }
                            
                            return result
                                .map(Message::PageMessage)
                                .map(cosmic::Action::App);
                        }
                    }

                    page::Message::GuardianSync(message) => {
                        if let Some(page) = self.pages.get_mut(&TypeId::of::<page::guardian_sync::Page>()) {
                            return page
                                .as_any()
                                .downcast_mut::<page::guardian_sync::Page>()
                                .unwrap()
                                .update(message)
                                .map(Message::PageMessage)
                                .map(cosmic::Action::App);
                        }
                    }
                }
            }

            Message::PageOpen(page_i) => {
                if let Some((_, page)) = self.pages.get_index_mut(page_i) {
                    self.page_i = page_i;
                    return page
                        .open()
                        .map(Message::PageMessage)
                        .map(cosmic::Action::App);
                }
            }

            Message::Finish => {
                // Mark setup as complete
                let mark_setup_done = cosmic::Task::future(async {
                    #[allow(deprecated)]
                    let home = std::env::home_dir().unwrap();
                    _ = std::fs::File::create(home.join(GUARDIAN_SETUP_DONE_PATH));
                }).discard();

                // Apply all page settings
                let mut tasks = self
                    .pages
                    .values_mut()
                    .filter_map(|page| {
                        page.completed().then(|| {
                            page.apply_settings()
                                .map(Message::PageMessage)
                                .map(cosmic::Action::App)
                        })
                    })
                    .chain(std::iter::once(mark_setup_done))
                    .collect::<Vec<_>>()
                    .apply(Task::batch);

                // If live install mode, log out the installer user
                if self.is_live_install {
                    tasks = tasks.chain(
                        cosmic::Task::future(async {
                            _ = std::process::Command::new("loginctl")
                                .args(&["terminate-user", "guardian-installer"])
                                .status();
                        }).discard(),
                    );
                }

                return tasks.chain(cosmic::Task::done(Message::Exit.into()));
            }

            Message::Exit => {
                return cosmic::iced::exit();
            }
        }
        Task::none()
    }

    fn dialog(&self) -> Option<Element<'_, Self::Message>> {
        self.pages[self.page_i]
            .dialog()
            .map(|dialog| dialog.map(Message::PageMessage))
    }

    fn view(&self) -> Element<'_, Message> {
        let cosmic_theme::Spacing {
            space_xxs,
            space_m,
            space_l,
            space_xl,
            ..
        } = theme::active().cosmic().spacing;

        let page = &self.pages[self.page_i];

        let skip_button = page
            .optional()
            .then(|| widget::button::link(fl!("skip")).on_press(Message::PageOpen(self.page_i + 1)))
            .or_else(|| {
                page.skippable().then(|| {
                    widget::button::link(fl!("skip-setup-and-close")).on_press(Message::Finish)
                })
            });

        let mut button_row = widget::row::with_capacity(4)
            .spacing(space_xxs)
            .push_maybe(skip_button)
            .push(widget::horizontal_space());

        if let Some(page_i) = self.page_i.checked_sub(1) {
            if self.pages.get_index(page_i).is_some() {
                button_row = button_row.push(
                    widget::button::standard(fl!("back")).on_press(Message::PageOpen(page_i)),
                );
            }
        }

        if let Some(page_i) = self.page_i.checked_add(1) {
            if self.pages.get_index(page_i).is_some() {
                let mut next = widget::button::suggested(fl!("next"));
                if page.completed() {
                    next = next.on_press(Message::PageOpen(page_i));
                }
                button_row = button_row.push(next);
            } else {
                let mut finish = widget::button::suggested(fl!("finish"));
                if page.completed() {
                    finish = finish.on_press(Message::Finish);
                }
                button_row = button_row.push(finish);
            }
        }

        let title = widget::text::title2(page.title())
            .center()
            .width(Length::Fill);

        let content = page
            .view()
            .map(Message::PageMessage)
            .apply(widget::container)
            .height(Length::Fill);

        widget::column::with_capacity(7)
            .push(widget::Space::with_height(space_xl))
            .push(title)
            .push(widget::Space::with_height(space_l))
            .push(content)
            .push(widget::Space::with_height(space_m))
            .push(button_row)
            .push(widget::Space::with_height(space_l))
            .max_width(page.width())
            .width(page.width())
            .align_x(Alignment::Center)
            .apply(widget::container)
            .center_x(Length::Fill)
            .into()
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        let mut subscriptions = vec![
            cosmic_settings_subscriptions::accessibility::subscription().map(|m| {
                Message::PageMessage(page::Message::Welcome(
                    page::welcome::Message::ScreenReaderDbus(m),
                ))
            }),
        ];

        if self.wifi_exists {
            subscriptions.push(Subscription::run(network_manager_stream));
        }

        Subscription::batch(subscriptions)
    }
}

fn network_manager_stream() -> impl Stream<Item = Message> {
    use cosmic_settings_subscriptions::network_manager;
    cosmic::iced_futures::stream::channel(1, |mut output| async move {
        let conn = zbus::Connection::system().await.unwrap();
        let (tx, mut rx) = futures::channel::mpsc::channel(1);

        let watchers = std::pin::pin!(async move {
            futures::join!(
                network_manager::watch(conn.clone(), tx.clone()),
                network_manager::active_conns::watch(conn.clone(), tx.clone()),
                network_manager::wireless_enabled::watch(conn.clone(), tx.clone()),
                network_manager::watch_connections_changed(conn, tx)
            );
        });

        let forwarder = std::pin::pin!(async move {
            while let Some(message) = rx.next().await {
                _ = output
                    .send(page::Message::WiFi(page::wifi::Message::NetworkManager(message)).into())
                    .await;
            }
        });

        futures::future::select(watchers, forwarder).await;
    })
}
