//! Devices management page

use cosmic::widget::{self, button, column, container, row, text, Space};
use cosmic::{Element, Theme};
use cosmic::iced::Length;

use crate::app::{GuardianSettings, Message};

pub fn view(app: &GuardianSettings) -> Element<Message> {
    let header = row![
        text("Devices").size(24),
        Space::with_width(Length::Fill),
        button::text("Refresh")
            .on_press(Message::RefreshDevices),
    ]
    .padding(20);
    
    let devices = app.devices();
    
    let device_cards: Vec<Element<Message>> = devices.iter().map(|device| {
        let status_icon = if device.is_online() { "🟢" } else { "⚫" };
        let status_text = if device.is_online() { "Online" } else { "Offline" };
        let name = device.name.as_deref().unwrap_or("Unknown Device");
        let device_type = device.device_type.as_deref().unwrap_or("desktop");
        let child_name = device.child_name().unwrap_or("Unassigned");
        
        let type_icon = match device_type {
            "laptop" => "💻",
            "tablet" => "📱",
            _ => "🖥️",
        };
        
        let last_seen = device.last_seen_at
            .map(|dt| {
                let now = chrono::Utc::now();
                let diff = now - dt;
                if diff.num_minutes() < 1 {
                    "Just now".to_string()
                } else if diff.num_hours() < 1 {
                    format!("{} mins ago", diff.num_minutes())
                } else if diff.num_hours() < 24 {
                    format!("{} hours ago", diff.num_hours())
                } else {
                    format!("{} days ago", diff.num_days())
                }
            })
            .unwrap_or_else(|| "Never".to_string());
        
        container(
            column![
                row![
                    text(type_icon).size(32),
                    Space::with_width(15),
                    column![
                        text(name).size(18),
                        row![
                            text(status_icon),
                            Space::with_width(5),
                            text(status_text).size(14),
                            Space::with_width(10),
                            text("•"),
                            Space::with_width(10),
                            text(format!("Last seen: {}", last_seen)).size(14),
                        ],
                    ],
                    Space::with_width(Length::Fill),
                    column![
                        text("Assigned to:").size(12),
                        text(child_name).size(14),
                    ]
                    .align_items(cosmic::iced::Alignment::End),
                ]
                .align_items(cosmic::iced::Alignment::Center),
                
                Space::with_height(15),
                
                // Action buttons
                row![
                    button::text("Lock Device")
                        .on_press(Message::SendCommand(device.id.clone(), "lock".to_string()))
                        .style(cosmic::theme::Button::Standard),
                    Space::with_width(10),
                    button::text("Send Message")
                        .on_press(Message::SendCommand(device.id.clone(), "message".to_string()))
                        .style(cosmic::theme::Button::Standard),
                    Space::with_width(10),
                    button::text("Update Policies")
                        .on_press(Message::SendCommand(device.id.clone(), "update_policies".to_string()))
                        .style(cosmic::theme::Button::Suggested),
                ],
                
                // Version info
                Space::with_height(10),
                row![
                    text(format!("Guardian OS {}", 
                        device.daemon_version.as_deref().unwrap_or("?")
                    )).size(12),
                    Space::with_width(10),
                    text(format!("• {}", 
                        device.os_version.as_deref().unwrap_or("Unknown OS")
                    )).size(12),
                ],
            ]
            .spacing(5)
        )
        .padding(20)
        .style(cosmic::theme::Container::Card)
        .into()
    }).collect();
    
    let device_list = if device_cards.is_empty() {
        container(
            column![
                text("No devices registered").size(18),
                Space::with_height(10),
                text("Devices running Guardian OS will appear here once activated.").size(14),
            ]
            .align_items(cosmic::iced::Alignment::Center)
        )
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
    } else {
        widget::scrollable(
            column::with_children(device_cards)
                .spacing(15)
                .padding(20)
        )
        .into()
    };
    
    column![
        header,
        device_list,
    ]
    .into()
}
