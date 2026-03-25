//! Settings tab: backend health, editable config overrides, and watchlist management.

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
    widgets::{Block, Borders},
    Frame,
};

use crate::app::App;
use crate::workspace::SettingsSection;

pub(crate) use config_section::render_settings_config_section;
pub(crate) use health_section::render_settings_health_section;
pub(crate) use hint_section::render_settings_hint_section;
pub(crate) use sources_section::render_settings_sources_section;
pub(crate) use symbols_section::render_settings_symbols_section;

#[derive(Clone, Copy)]
pub(crate) struct SettingsLayout {
    pub health: Rect,
    pub config: Rect,
    pub symbols: Rect,
    pub sources: Rect,
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
        SettingsLayout {
            health: top[0],
            config: top[1],
            symbols: bottom[0],
            sources: bottom[1],
            hint: rows[2],
        }
    } else {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(9),
                Constraint::Min(3),
                Constraint::Min(6),
                Constraint::Min(5),
                Constraint::Length(1),
            ])
            .split(area);
        SettingsLayout {
            health: chunks[0],
            config: chunks[1],
            symbols: chunks[2],
            sources: chunks[3],
            hint: chunks[4],
        }
    }
}

pub fn render_settings(f: &mut Frame, app: &App, area: Rect) {
    let SettingsLayout {
        health,
        config,
        symbols,
        sources,
        hint,
    } = settings_layout(area);
    render_settings_health_section(f, app, health);
    render_settings_config_section(f, app, config);
    render_settings_symbols_section(f, app, symbols);
    render_settings_sources_section(f, app, sources);
    render_settings_hint_section(f, app, hint);
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
