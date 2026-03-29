//! Settings tab: backend health, editable config overrides, and watchlist management.

#[path = "settings_alpaca.rs"]
mod alpaca_section;
#[path = "settings_config.rs"]
mod config_section;
#[path = "settings_health.rs"]
mod health_section;
#[path = "settings_hint.rs"]
mod hint_section;
#[path = "settings_sources.rs"]
mod sources_section;
#[path = "settings_symbols.rs"]
mod symbols_section;

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use crate::app::App;
use crate::workspace::SettingsSection;

pub(crate) use alpaca_section::{
    alpaca_credential_key_for_row, render_settings_alpaca_section, ALPACA_CREDENTIAL_ROW_COUNT,
};
pub(crate) use config_section::render_settings_config_section;
pub(crate) use health_section::render_settings_health_section;
pub(crate) use hint_section::render_settings_hint_section;
pub(crate) use sources_section::{
    credential_key_for_sources_row, render_settings_sources_section, SOURCES_TABLE_ROW_COUNT,
};
pub(crate) use symbols_section::render_settings_symbols_section;

#[derive(Clone, Copy)]
pub(crate) struct SettingsLayout {
    pub health: Rect,
    pub config: Rect,
    pub symbols: Rect,
    pub sources: Rect,
    pub alpaca: Rect,
    pub hint: Rect,
}

pub(crate) fn settings_layout(area: Rect) -> SettingsLayout {
    let wide_layout = area.width >= 120 && area.height >= 18;
    if wide_layout {
        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(10),
                Constraint::Min(7),
                Constraint::Length(1),
            ])
            .split(area);
        let top = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(62), Constraint::Percentage(38)])
            .split(rows[0]);
        let bottom = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(38), Constraint::Percentage(62)])
            .split(rows[1]);
        let sources_alpaca = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(62), Constraint::Percentage(38)])
            .split(bottom[1]);
        SettingsLayout {
            health: top[0],
            config: top[1],
            symbols: bottom[0],
            sources: sources_alpaca[0],
            alpaca: sources_alpaca[1],
            hint: rows[2],
        }
    } else {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(9),
                Constraint::Min(3),
                Constraint::Min(5),
                Constraint::Min(4),
                Constraint::Min(5),
                Constraint::Length(1),
            ])
            .split(area);
        SettingsLayout {
            health: chunks[0],
            config: chunks[1],
            symbols: chunks[2],
            sources: chunks[3],
            alpaca: chunks[4],
            hint: chunks[5],
        }
    }
}

pub fn render_settings(f: &mut Frame, app: &App, area: Rect) {
    let SettingsLayout {
        health,
        config,
        symbols,
        sources,
        alpaca,
        hint,
    } = settings_layout(area);
    render_settings_health_section(f, app, health);
    render_settings_config_section(f, app, config);
    render_settings_symbols_section(f, app, symbols);
    render_settings_sources_section(f, app, sources);
    render_settings_alpaca_section(f, app, alpaca);
    render_settings_hint_section(f, app, hint);
    render_settings_credential_modal_if_any(f, app, area);
}

fn settings_modal_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let w = (r.width * percent_x) / 100;
    let h = (r.height * percent_y) / 100;
    let x = r.x + (r.width.saturating_sub(w)) / 2;
    let y = r.y + (r.height.saturating_sub(h)) / 2;
    Rect::new(x, y, w, h)
}

/// Centered overlay for any API key edit (Alpaca rows or Data sources rows).
fn render_settings_credential_modal_if_any(f: &mut Frame, app: &App, area: Rect) {
    if app.settings_credential_buffer.is_none() {
        return;
    }
    let key = app
        .settings_credential_edit_key
        .map(|k| k.display_name())
        .unwrap_or("API key");
    let buf = app.settings_credential_buffer.as_deref().unwrap_or("");
    let modal_area = settings_modal_rect(85, 40, area);
    let edit = Paragraph::new(vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("Edit ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                key,
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(": ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{buf}_"),
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "[Enter] save to keyring/file  [Esc] cancel",
            Style::default().fg(Color::DarkGray),
        )),
    ])
    .block(
        Block::default()
            .title(format!(" {key} — edit "))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan)),
    );
    f.render_widget(Clear, modal_area);
    f.render_widget(edit, modal_area);
}

pub(super) fn section_active(app: &App, section: SettingsSection) -> bool {
    app.settings_section == section
}

pub(super) fn section_block(title: impl Into<String>, active: bool) -> Block<'static> {
    let title = title.into();
    let border_style = if active {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::DarkGray)
    };
    let title = if active {
        format!(" ▶ {} ", title)
    } else {
        format!(" {} ", title)
    };
    Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(border_style)
}

pub(super) fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}…", &s[..max.saturating_sub(1)])
    }
}
