// Guardian OS Installer - Page Module
// Two modes: Live Install (pre-install) and Post-Install (first boot)

use cosmic::{Element, widget};
use indexmap::IndexMap;
use std::any::{Any, TypeId};

pub mod appearance;
pub mod guardian_auth;
pub mod guardian_child;
pub mod guardian_sync;
pub mod keyboard;
pub mod language;
pub mod layout;
pub mod user;
pub mod welcome;
pub mod wifi;

/// Application modes
pub enum AppMode {
    /// Live ISO installation - Guardian auth flow + user creation
    /// This runs during the live session BEFORE installation
    NewInstall {
        create_user: bool,
    },
    /// Post-install first boot wizard - just customization
    /// This runs on FIRST BOOT after installation completes
    PostInstall,
}

/// Build the page list based on mode
#[inline]
pub fn pages(mode: AppMode) -> IndexMap<TypeId, Box<dyn Page>> {
    let mut pages: IndexMap<TypeId, Box<dyn Page>> = IndexMap::new();
    
    match mode {
        // ============================================
        // LIVE INSTALL MODE (Pre-install from live ISO)
        // ============================================
        // Flow: Welcome → WiFi → Language → Keyboard → 
        //       Guardian Auth → Child Selection → User Creation → Sync
        // After this, pop-installer handles disk partitioning
        AppMode::NewInstall { create_user } => {
            // 1. Welcome - accessibility options
            pages.insert(
                TypeId::of::<welcome::Page>(),
                Box::new(welcome::Page::new()),
            );

            // 2. WiFi connection (optional but recommended for auth)
            pages.insert(
                TypeId::of::<wifi::Page>(),
                Box::new(wifi::Page::default()),
            );

            // 3. Language - default to English UK
            pages.insert(
                TypeId::of::<language::Page>(),
                Box::new(language::Page::new()),
            );

            // 4. Keyboard layout
            pages.insert(
                TypeId::of::<keyboard::Page>(),
                Box::new(keyboard::Page::new()),
            );

            // === GUARDIAN AUTHENTICATION FLOW ===
            
            // 5. Parent Authentication (REQUIRED)
            // Parent signs in/creates Guardian account
            // This proves they have authority to set up the device
            pages.insert(
                TypeId::of::<guardian_auth::Page>(),
                Box::new(guardian_auth::Page::new()),
            );

            // 6. Child Profile Selection (REQUIRED)
            // Select existing child or create new profile
            // Links device to specific child
            pages.insert(
                TypeId::of::<guardian_child::Page>(),
                Box::new(guardian_child::Page::new()),
            );

            // 7. User Account Creation
            // Creates the local Linux user account
            // Auto-filled from child profile name
            // Limited permissions (not sudo)
            if create_user {
                pages.insert(
                    TypeId::of::<user::Page>(),
                    Box::new(user::Page::default()),
                );
            }

            // 8. Sync Enrollment (optional)
            // Enable settings sync across devices
            pages.insert(
                TypeId::of::<guardian_sync::Page>(),
                Box::new(guardian_sync::Page::new()),
            );
            
            // Note: After this wizard completes, the actual disk installation
            // is handled by pop-installer/distinst. This wizard just collects
            // the user information and Guardian credentials.
        }
        
        // ============================================
        // POST-INSTALL MODE (First boot after installation)
        // ============================================
        // Flow: Welcome → Appearance → Layout
        // Simple customization wizard - device is already set up
        AppMode::PostInstall => {
            // 1. Welcome to Guardian OS
            pages.insert(
                TypeId::of::<welcome::Page>(),
                Box::new(welcome::Page::new()),
            );

            // 2. Appearance - theme, accent colors
            pages.insert(
                TypeId::of::<appearance::Page>(),
                Box::new(appearance::Page::new()),
            );

            // 3. Layout - panel position, dock
            pages.insert(
                TypeId::of::<layout::Page>(),
                Box::new(layout::Page::default()),
            );
            
            // That's it! Device is already registered,
            // child profile is linked, daemon is running.
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
