// Guardian OS Post-Install Wizard
// Runs on FIRST BOOT after installation

use cosmic::{Element, widget};
use indexmap::IndexMap;
use std::any::{Any, TypeId};

pub mod appearance;
pub mod guardian_auth;
pub mod guardian_child;
pub mod guardian_protection;
pub mod guardian_sync;
pub mod keyboard;
pub mod language;
pub mod layout;
pub mod user;
pub mod welcome;
pub mod wifi;

/// Application modes
pub enum AppMode {
    /// Normal first-boot wizard (device already registered during install)
    PostInstall,
    
    /// Fallback mode if device wasn't registered during installation
    UnregisteredDevice,
}

/// Build the page list based on mode
#[inline]
pub fn pages(mode: AppMode) -> IndexMap<TypeId, Box<dyn Page>> {
    let mut pages: IndexMap<TypeId, Box<dyn Page>> = IndexMap::new();
    
    match mode {
        // ============================================
        // NORMAL POST-INSTALL (Device already registered)
        // ============================================
        // Flow: Welcome → WiFi → Protection Setup → Appearance → Layout
        AppMode::PostInstall => {
            // 1. Welcome to Guardian OS
            pages.insert(
                TypeId::of::<welcome::Page>(),
                Box::new(welcome::Page::new()),
            );

            // 2. WiFi (connect for policy sync)
            pages.insert(
                TypeId::of::<wifi::Page>(),
                Box::new(wifi::Page::default()),
            );

            // 3. Guardian Protection Setup
            // Shows daemon status, DNS config, filtering level
            pages.insert(
                TypeId::of::<guardian_protection::Page>(),
                Box::new(guardian_protection::Page::new()),
            );

            // 4. Appearance - theme, accent colours
            pages.insert(
                TypeId::of::<appearance::Page>(),
                Box::new(appearance::Page::new()),
            );

            // 5. Layout - panel position, dock
            pages.insert(
                TypeId::of::<layout::Page>(),
                Box::new(layout::Page::default()),
            );
        }
        
        // ============================================
        // UNREGISTERED DEVICE (Fallback)
        // ============================================
        // Flow: Welcome → WiFi → Auth → Child → Protection → Appearance → Layout
        AppMode::UnregisteredDevice => {
            pages.insert(
                TypeId::of::<welcome::Page>(),
                Box::new(welcome::Page::new()),
            );

            pages.insert(
                TypeId::of::<wifi::Page>(),
                Box::new(wifi::Page::default()),
            );

            // Guardian auth flow (fallback)
            pages.insert(
                TypeId::of::<guardian_auth::Page>(),
                Box::new(guardian_auth::Page::new()),
            );

            pages.insert(
                TypeId::of::<guardian_child::Page>(),
                Box::new(guardian_child::Page::new()),
            );

            pages.insert(
                TypeId::of::<guardian_sync::Page>(),
                Box::new(guardian_sync::Page::new()),
            );

            // Protection setup (after auth)
            pages.insert(
                TypeId::of::<guardian_protection::Page>(),
                Box::new(guardian_protection::Page::new()),
            );

            // Appearance customization
            pages.insert(
                TypeId::of::<appearance::Page>(),
                Box::new(appearance::Page::new()),
            );

            pages.insert(
                TypeId::of::<layout::Page>(),
                Box::new(layout::Page::default()),
            );
        }
    }

    pages
}

#[derive(Clone, Debug)]
pub enum Message {
    Appearance(appearance::Message),
    GuardianAuth(guardian_auth::Message),
    GuardianChild(guardian_child::Message),
    GuardianProtection(guardian_protection::Message),
    GuardianSync(guardian_sync::Message),
    Keyboard(keyboard::Message),
    Language(language::Message),
    Layout(layout::Message),
    SetTheme(cosmic::Theme),
    User(user::Message),
    Welcome(welcome::Message),
    WiFi(wifi::Message),
}

impl From<Message> for super::Message {
    fn from(message: Message) -> Self {
        super::Message::PageMessage(message)
    }
}

pub trait Page {
    fn as_any(&mut self) -> &mut dyn Any;

    fn title(&self) -> String;

    fn init(&mut self) -> cosmic::Task<Message> {
        cosmic::Task::none()
    }

    fn apply_settings(&mut self) -> cosmic::Task<Message> {
        cosmic::Task::none()
    }

    fn open(&mut self) -> cosmic::Task<Message> {
        cosmic::Task::none()
    }

    fn width(&self) -> f32 {
        640.0
    }

    fn completed(&self) -> bool {
        true
    }

    fn optional(&self) -> bool {
        false
    }

    fn skippable(&self) -> bool {
        false
    }

    fn dialog(&self) -> Option<Element<'_, Message>> {
        None
    }

    fn view(&self) -> Element<'_, Message> {
        widget::text::body("TODO").into()
    }
}
