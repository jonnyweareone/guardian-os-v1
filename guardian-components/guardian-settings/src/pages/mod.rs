//! Page views for Guardian Settings

pub mod family;
pub mod screen_time;
pub mod content_filter;
pub mod devices;
pub mod alerts;

/// Pages in the settings app
#[derive(Debug, Clone, PartialEq)]
pub enum Page {
    Family,
    ScreenTime,
    ContentFilter,
    Devices,
    Alerts,
}
