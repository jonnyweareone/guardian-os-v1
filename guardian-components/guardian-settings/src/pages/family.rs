//! Family overview page

use cosmic::widget::{self, button, column, container, row, text, Space};
use cosmic::{Element, Theme};
use cosmic::iced::Length;

use crate::app::{GuardianSettings, Message};

pub fn view(app: &GuardianSettings) -> Element<Message> {
    let children = app.children();
    
    let header = row![
        text("Family Members").size(24),
        Space::with_width(Length::Fill),
        button::text("Add Child").on_press(Message::LoadData), // TODO: AddChild
    ]
    .spacing(10)
    .padding(20);
    
    let child_cards: Vec<Element<Message>> = children.iter().map(|child| {
        let is_selected = app.selected_child() == Some(&child.id);
        let age_text = child.age()
            .map(|a| format!("{} years old", a))
            .unwrap_or_else(|| "Age not set".to_string());
        
        let card = container(
            column![
                row![
                    // Avatar placeholder
                    container(text("👤").size(32))
                        .width(48)
                        .height(48)
                        .center_x(Length::Fill)
                        .center_y(Length::Fill),
                    Space::with_width(10),
                    column![
                        text(&child.name).size(18),
                        text(&age_text).size(14),
                    ]
                    .spacing(4),
                ]
                .spacing(10),
            ]
            .padding(15)
        )
        .style(if is_selected {
            cosmic::theme::Container::Primary
        } else {
            cosmic::theme::Container::Card
        });
        
        button::custom(card)
            .on_press(Message::SelectChild(child.id.clone()))
            .into()
    }).collect();
    
    let children_list = if child_cards.is_empty() {
        container(
            column![
                text("No children added yet").size(16),
                Space::with_height(10),
                text("Add a child to get started with parental controls.").size(14),
            ]
            .align_items(cosmic::iced::Alignment::Center)
        )
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
    } else {
        column::with_children(child_cards)
            .spacing(10)
            .into()
    };
    
    // Selected child details
    let details = if let Some(child) = app.selected_child_data() {
        let age_text = child.age()
            .map(|a| format!("{} years old", a))
            .unwrap_or_else(|| "Age not set".to_string());
        
        container(
            column![
                text(&child.name).size(28),
                text(&age_text).size(16),
                Space::with_height(20),
                
                text("Quick Actions").size(18),
                Space::with_height(10),
                
                row![
                    button::text("Screen Time")
                        .on_press(Message::EditScreenTime(child.id.clone())),
                    button::text("Content Filter")
                        .on_press(Message::EditDnsProfile(child.id.clone())),
                ]
                .spacing(10),
                
                Space::with_height(20),
                text("Devices").size(18),
                Space::with_height(10),
                
                // Devices assigned to this child
                {
                    let child_devices: Vec<Element<Message>> = app.devices()
                        .iter()
                        .filter(|d| d.child_id.as_ref() == Some(&child.id))
                        .map(|d| {
                            let status_icon = if d.is_online() { "🟢" } else { "⚫" };
                            let name = d.name.as_deref().unwrap_or("Unknown Device");
                            
                            row![
                                text(status_icon),
                                Space::with_width(8),
                                text(name),
                            ]
                            .spacing(5)
                            .into()
                        })
                        .collect();
                    
                    if child_devices.is_empty() {
                        text("No devices assigned").into()
                    } else {
                        column::with_children(child_devices)
                            .spacing(5)
                            .into()
                    }
                },
            ]
            .spacing(5)
        )
        .padding(20)
        .into()
    } else {
        container(
            text("Select a child to view details")
        )
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
    };
    
    column![
        header,
        row![
            container(children_list)
                .width(300)
                .height(Length::Fill)
                .padding(10),
            container(details)
                .width(Length::Fill)
                .height(Length::Fill),
        ]
    ]
    .into()
}
