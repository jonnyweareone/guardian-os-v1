//! Screen time settings page

use cosmic::widget::{self, button, column, container, row, text, toggler, slider, Space};
use cosmic::{Element, Theme};
use cosmic::iced::Length;

use crate::app::{GuardianSettings, Message};

pub fn view(app: &GuardianSettings) -> Element<Message> {
    let header = row![
        text("Screen Time").size(24),
    ]
    .padding(20);
    
    // Child selector
    let child_selector = {
        let children = app.children();
        let buttons: Vec<Element<Message>> = children.iter().map(|child| {
            let is_selected = app.selected_child() == Some(&child.id);
            button::text(&child.name)
                .on_press(Message::SelectChild(child.id.clone()))
                .style(if is_selected {
                    cosmic::theme::Button::Primary
                } else {
                    cosmic::theme::Button::Standard
                })
                .into()
        }).collect();
        
        row::with_children(buttons).spacing(10)
    };
    
    // Screen time policy editor
    let policy_editor = if let Some(policy) = app.editing_screen_time() {
        let weekday_limit = policy.weekday_limit_mins.unwrap_or(120);
        let weekend_limit = policy.weekend_limit_mins.unwrap_or(180);
        let bedtime_enabled = policy.bedtime_enabled.unwrap_or(true);
        let bedtime_time = policy.bedtime_time.clone().unwrap_or_else(|| "20:30".to_string());
        
        container(
            column![
                text("Daily Limits").size(18),
                Space::with_height(15),
                
                row![
                    text("Weekday limit:"),
                    Space::with_width(Length::Fill),
                    text(format!("{} hours {} mins", weekday_limit / 60, weekday_limit % 60)),
                ],
                slider(30..=480, weekday_limit, |v| Message::UpdateWeekdayLimit(v))
                    .step(15u16),
                
                Space::with_height(15),
                
                row![
                    text("Weekend limit:"),
                    Space::with_width(Length::Fill),
                    text(format!("{} hours {} mins", weekend_limit / 60, weekend_limit % 60)),
                ],
                slider(30..=480, weekend_limit, |v| Message::UpdateWeekendLimit(v))
                    .step(15u16),
                
                Space::with_height(25),
                text("Bedtime").size(18),
                Space::with_height(15),
                
                row![
                    text("Enable bedtime:"),
                    Space::with_width(Length::Fill),
                    toggler(bedtime_enabled)
                        .on_toggle(Message::UpdateBedtimeEnabled),
                ],
                
                Space::with_height(10),
                
                if bedtime_enabled {
                    row![
                        text("Bedtime at:"),
                        Space::with_width(Length::Fill),
                        text(&bedtime_time).size(18),
                        // TODO: Time picker widget
                    ].into()
                } else {
                    Space::with_height(0).into()
                },
                
                Space::with_height(30),
                
                row![
                    button::text("Cancel")
                        .on_press(Message::LoadData),
                    Space::with_width(10),
                    button::text("Save Changes")
                        .on_press(Message::SaveScreenTime)
                        .style(cosmic::theme::Button::Suggested),
                ],
            ]
            .spacing(8)
        )
        .padding(20)
        .into()
    } else if let Some(child) = app.selected_child_data() {
        // Show current settings summary
        container(
            column![
                text(format!("Screen time for {}", child.name)).size(18),
                Space::with_height(20),
                
                text("Loading policy..."),
                
                Space::with_height(20),
                
                button::text("Edit Screen Time Settings")
                    .on_press(Message::EditScreenTime(child.id.clone()))
                    .style(cosmic::theme::Button::Suggested),
            ]
        )
        .padding(20)
        .into()
    } else {
        container(
            column![
                text("Select a child to manage their screen time").size(16),
            ]
            .align_items(cosmic::iced::Alignment::Center)
        )
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
    };
    
    column![
        header,
        container(child_selector).padding([0, 20]),
        Space::with_height(20),
        policy_editor,
    ]
    .into()
}
