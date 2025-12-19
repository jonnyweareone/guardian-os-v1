//! Guardian Settings - Parental Control Panel for COSMIC Desktop
//!
//! This application provides a parent-facing interface to:
//! - View family members and devices
//! - Configure screen time limits per child
//! - Set content filtering rules
//! - View activity reports and alerts
//! - Manage app permissions

mod api;
mod app;
mod config;
mod pages;
mod widgets;

use cosmic::app::{Core, Settings, Task};
use cosmic::iced::Size;
use cosmic::Application;

fn main() -> cosmic::iced::Result {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    let settings = Settings::default()
        .size(Size::new(900.0, 700.0))
        .antialiasing(true);

    cosmic::app::run::<app::GuardianSettings>(settings, ())
}
