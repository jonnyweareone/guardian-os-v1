//! Alerts page

use cosmic::widget::{self, button, column, container, row, text, Space};
use cosmic::{Element, Theme};
use cosmic::iced::{Color, Length};

use crate::app::{GuardianSettings, Message};

pub fn view(app: &GuardianSettings) -> Element<Message> {
    let alerts = app.alerts();
    
    // Count by severity
    let critical_count = alerts.iter().filter(|a| a.severity.as_deref() == Some("critical")).count();
    let high_count = alerts.iter().filter(|a| a.severity.as_deref() == Some("high")).count();
    let new_count = alerts.iter().filter(|a| a.status.as_deref() == Some("new")).count();
    
    let header = row![
        text("Alerts").size(24),
        Space::with_width(20),
        if critical_count > 0 {
            badge("critical", critical_count)
        } else {
            Space::with_width(0).into()
        },
        if high_count > 0 {
            badge("high", high_count)
        } else {
            Space::with_width(0).into()
        },
        Space::with_width(Length::Fill),
        text(format!("{} new", new_count)).size(14),
        Space::with_width(10),
        button::text("Refresh")
            .on_press(Message::RefreshAlerts),
    ]
    .align_items(cosmic::iced::Alignment::Center)
    .padding(20);
    
    let alert_cards: Vec<Element<Message>> = alerts.iter().map(|alert| {
        let (r, g, b) = alert.severity_color();
        let severity = alert.severity.as_deref().unwrap_or("low");
        let child_name = alert.child_name().unwrap_or("Unknown");
        let is_new = alert.status.as_deref() == Some("new");
        
        let time_ago = alert.created_at
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
            .unwrap_or_else(|| "Unknown time".to_string());
        
        let severity_icon = match severity {
            "critical" => "🔴",
            "high" => "🟠",
            "medium" => "🟡",
            _ => "⚪",
        };
        
        container(
            row![
                // Severity indicator
                container(
                    text(severity_icon).size(24)
                )
                .width(40)
                .center_x(Length::Fill),
                
                // Content
                column![
                    row![
                        text(&alert.title).size(16),
                        Space::with_width(10),
                        if is_new {
                            container(text("NEW").size(10))
                                .padding([2, 6])
                                .style(cosmic::theme::Container::Primary)
                                .into()
                        } else {
                            Space::with_width(0).into()
                        },
                    ],
                    Space::with_height(5),
                    text(alert.description.as_deref().unwrap_or("")).size(14),
                    Space::with_height(5),
                    row![
                        text(format!("Child: {}", child_name)).size(12),
                        Space::with_width(15),
                        text(format!("• {}", time_ago)).size(12),
                        Space::with_width(15),
                        text(format!("• Type: {}", alert.alert_type)).size(12),
                    ],
                ]
                .width(Length::Fill),
                
                // Actions
                column![
                    button::text("Dismiss")
                        .on_press(Message::DismissAlert(alert.id.clone()))
                        .style(cosmic::theme::Button::Standard),
                ]
                .align_items(cosmic::iced::Alignment::End),
            ]
            .spacing(15)
            .align_items(cosmic::iced::Alignment::Center)
        )
        .padding(15)
        .style(cosmic::theme::Container::Card)
        .into()
    }).collect();
    
    let alert_list = if alert_cards.is_empty() {
        container(
            column![
                text("✅ All Clear!").size(32),
                Space::with_height(15),
                text("No active alerts. Your family is safe.").size(16),
            ]
            .align_items(cosmic::iced::Alignment::Center)
        )
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
    } else {
        widget::scrollable(
            column::with_children(alert_cards)
                .spacing(10)
                .padding(20)
        )
        .into()
    };
    
    column![
        header,
        alert_list,
    ]
    .into()
}

fn badge<'a>(severity: &str, count: usize) -> Element<'a, Message> {
    let (r, g, b) = match severity {
        "critical" => (0.9, 0.1, 0.1),
        "high" => (0.9, 0.4, 0.1),
        _ => (0.5, 0.5, 0.5),
    };
    
    container(
        text(format!("{} {}", count, severity)).size(12)
    )
    .padding([4, 10])
    .style(cosmic::theme::Container::custom(move |_| {
        cosmic::iced_style::container::Style {
            background: Some(Color::from_rgb(r, g, b).into()),
            text_color: Some(Color::WHITE),
            border: cosmic::iced::Border {
                radius: 12.0.into(),
                ..Default::default()
            },
            ..Default::default()
        }
    }))
    .into()
}
