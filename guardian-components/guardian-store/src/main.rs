//! Guardian Store - Family-safe App Store for Guardian OS
//!
//! This is a parental-control-aware app store that:
//! - Displays age ratings (PEGI, ESRB) for all apps
//! - Filters apps based on child's age
//! - Requires parent PIN for app installation
//! - Supports app request workflow (child requests → parent approves)
//! - Shows "Guardian Approved" curated section
//! - Integrates with guardian-daemon for policy enforcement

mod api;
mod app;
mod catalog;
mod daemon;
mod ratings;

use cosmic::app::Settings;
use cosmic::iced::Size;

fn main() -> cosmic::iced::Result {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("warn")).init();
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    let settings = Settings::default()
        .size(Size::new(1000.0, 700.0))
        .antialiasing(true);

    cosmic::app::run::<app::GuardianStore>(settings, ())
}
