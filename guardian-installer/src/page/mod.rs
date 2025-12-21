// Guardian OS Post-Install Wizard
// Runs on FIRST BOOT after installation
// Simple customization - device already registered during install

use cosmic::{Element, widget};
use indexmap::IndexMap;
use std::any::{Any, TypeId};

pub mod appearance;
pub mod keyboard;
pub mod language;
pub mod layout;
pub mod welcome;
pub mod wifi;

// Guardian auth pages - ONLY used if device wasn't registered during install
pub mod guardian_auth;
pub mod guardian_child;
pub mod guardian_sync;
pub mod user;

/// Application modes
pub enum AppMode {
    /// Normal first-boot wizard (device already registered during install)
    /// Simple customization only
    PostInstall,
    
    /// Fallback mode if device wasn't registered during installation
    /// Full Guardian auth flow
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
        // Simple customization wizard
        AppMode::PostInstall => {
            // 1. Welcome to Guardian OS
            pages.insert(
                TypeId::of::<welcome::Page>(),
                Box::new(welcome::Page::new()),
            );

            // 2. WiFi (connect for updates)
            pages.insert(
                TypeId::of::<wifi::Page>(),
                Box::new(wifi::Page::default()),
            );

            // 3. Appearance - theme, accent colours
            pages.insert(
                TypeId::of::<appearance::Page>(),
                Box::new(appearance::Page::new()),
            );

            // 4. Layout - panel position, dock
            pages.insert(
                TypeId::of::<layout::Page>(),
                Box::new(layout::Page::default()),
            );
        }
        
        // ============================================
        // UNREGISTERED DEVICE (Fallback - shouldn't normally happen)
        // ============================================
        // Full Guardian registration flow
        // This runs if installation happened without Guardian auth
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
