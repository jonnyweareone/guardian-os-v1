//! Content filter settings page

use cosmic::widget::{self, button, column, container, row, text, toggler, Space};
use cosmic::{Element, Theme};
use cosmic::iced::Length;

use crate::app::{GuardianSettings, Message};

pub fn view(app: &GuardianSettings) -> Element<Message> {
    let header = row![
        text("Content Filter").size(24),
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
    
    // DNS profile editor
    let filter_editor = if let Some(profile) = app.editing_dns() {
        let filter_level = profile.filter_level.clone().unwrap_or_else(|| "child".to_string());
        let block_adult = profile.block_adult.unwrap_or(true);
        let block_gambling = profile.block_gambling.unwrap_or(true);
        let block_social = profile.block_social_media.unwrap_or(false);
        let safe_search = profile.enforce_safe_search.unwrap_or(true);
        let blocked_domains = profile.blocked_domains.clone().unwrap_or_default();
        
        container(
            column![
                text("Filter Level").size(18),
                Space::with_height(10),
                
                row![
                    filter_button("Child", "child", &filter_level),
                    filter_button("Teen", "teen", &filter_level),
                    filter_button("Adult", "adult", &filter_level),
                    filter_button("Off", "off", &filter_level),
                ]
                .spacing(10),
                
                Space::with_height(25),
                text("Block Categories").size(18),
                Space::with_height(15),
                
                toggle_row("Adult content", block_adult, Message::ToggleBlockAdult),
                toggle_row("Gambling", block_gambling, Message::ToggleBlockGambling),
                toggle_row("Social media", block_social, Message::ToggleBlockSocialMedia),
                
                Space::with_height(25),
                text("Safe Search").size(18),
                Space::with_height(15),
                
                toggle_row("Enforce SafeSearch on Google, Bing, etc.", safe_search, Message::ToggleSafeSearch),
                
                Space::with_height(25),
                text("Blocked Websites").size(18),
                Space::with_height(10),
                
                {
                    let domain_chips: Vec<Element<Message>> = blocked_domains.iter().map(|domain| {
                        row![
                            text(domain),
                            button::icon(widget::icon::from_name("window-close-symbolic"))
                                .on_press(Message::RemoveBlockedDomain(domain.clone()))
                                .style(cosmic::theme::Button::Destructive),
                        ]
                        .spacing(5)
                        .into()
                    }).collect();
                    
                    if domain_chips.is_empty() {
                        text("No websites blocked").into()
                    } else {
                        column::with_children(domain_chips).spacing(5).into()
                    }
                },
                
                Space::with_height(30),
                
                row![
                    button::text("Cancel")
                        .on_press(Message::LoadData),
                    Space::with_width(10),
                    button::text("Save Changes")
                        .on_press(Message::SaveDnsProfile)
                        .style(cosmic::theme::Button::Suggested),
                ],
            ]
            .spacing(8)
        )
        .padding(20)
        .into()
    } else if let Some(child) = app.selected_child_data() {
        container(
            column![
                text(format!("Content filter for {}", child.name)).size(18),
                Space::with_height(20),
                
                button::text("Edit Content Filter")
                    .on_press(Message::EditDnsProfile(child.id.clone()))
                    .style(cosmic::theme::Button::Suggested),
            ]
        )
        .padding(20)
        .into()
    } else {
        container(
            text("Select a child to manage content filtering")
        )
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
    };
    
    column![
        header,
        container(child_selector).padding([0, 20]),
        Space::with_height(20),
        filter_editor,
    ]
    .into()
}

fn filter_button<'a>(label: &'a str, value: &'a str, current: &str) -> Element<'a, Message> {
    button::text(label)
        .on_press(Message::UpdateFilterLevel(value.to_string()))
        .style(if current == value {
            cosmic::theme::Button::Primary
        } else {
            cosmic::theme::Button::Standard
        })
        .into()
}

fn toggle_row<'a>(label: &'a str, value: bool, msg_fn: fn(bool) -> Message) -> Element<'a, Message> {
    row![
        text(label),
        Space::with_width(Length::Fill),
        toggler(value).on_toggle(msg_fn),
    ]
    .spacing(10)
    .into()
}
