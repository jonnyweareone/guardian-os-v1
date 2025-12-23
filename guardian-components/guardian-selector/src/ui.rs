//! Terminal UI for child selection

use crate::Child;
use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};
use std::io::stdout;

pub fn show_selector(children: &[Child]) -> Result<Child> {
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

    let mut state = ListState::default();
    state.select(Some(0));

    let result = loop {
        terminal.draw(|frame| {
            render_ui(frame, children, &mut state);
        })?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Up | KeyCode::Char('k') => {
                    let i = state.selected().unwrap_or(0);
                    state.select(Some(if i == 0 { children.len() - 1 } else { i - 1 }));
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    let i = state.selected().unwrap_or(0);
                    state.select(Some(if i >= children.len() - 1 { 0 } else { i + 1 }));
                }
                KeyCode::Enter => {
                    let selected = state.selected().unwrap_or(0);
                    break children[selected].clone();
                }
                KeyCode::Char('q') | KeyCode::Esc => {
                    cleanup_terminal()?;
                    std::process::exit(0);
                }
                _ => {}
            }
        }
    };

    cleanup_terminal()?;
    Ok(result)
}

fn render_ui(frame: &mut Frame, children: &[Child], state: &mut ListState) {
    let area = frame.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(2),
            Constraint::Min(10),
            Constraint::Length(3),
        ])
        .split(area);

    // Title
    let title = Paragraph::new("🛡️ Guardian OS")
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(Block::default());
    frame.render_widget(title, chunks[0]);

    // Subtitle
    let subtitle = Paragraph::new("Who's using this device?")
        .style(Style::default().fg(Color::White))
        .alignment(Alignment::Center);
    frame.render_widget(subtitle, chunks[1]);

    // Child list
    let items: Vec<ListItem> = children
        .iter()
        .map(|child| {
            let mode_icon = match child.experience_mode.as_str() {
                "kiosk" => "🎮",
                "desktop_supervised" => "💻",
                "desktop_trusted" => "🔓",
                _ => "👤",
            };
            let unlock_icon = match child.unlock_method.as_str() {
                "ask_parent" => "📱",
                "face_id" => "👤",
                "pin" => "🔢",
                "auto" => "✨",
                _ => "❓",
            };
            ListItem::new(format!("  {}  {}  {}  ({})", mode_icon, child.name, unlock_icon, child.experience_mode))
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(" Select Profile "))
        .highlight_style(Style::default().bg(Color::Blue).fg(Color::White).add_modifier(Modifier::BOLD))
        .highlight_symbol("▶ ");

    frame.render_stateful_widget(list, chunks[2], state);

    // Help
    let help = Paragraph::new("↑↓ Navigate  •  Enter Select  •  Q Quit")
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);
    frame.render_widget(help, chunks[3]);
}

fn cleanup_terminal() -> Result<()> {
    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}
