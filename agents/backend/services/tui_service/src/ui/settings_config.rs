use ratatui::layout::Rect;
use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use crate::app::App;
use crate::config::{config_key_scope, SettingScope};
use crate::workspace::SettingsSection;

use super::{section_active, section_block, truncate};

pub(crate) fn render_settings_config_section(f: &mut Frame, app: &App, area: Rect) {
    let config_title = if let Some(ref key) = app.settings_edit_config_key {
        let scope = config_key_scope(key);
        format!("Config overrides (editing {key} [{}])", scope.label())
    } else {
        "Config overrides (editable; session only)".to_string()
    };
    let config_block = section_block(&config_title, section_active(app, SettingsSection::Config));
    let config_value_width = area.width.saturating_sub(20) as usize;
    let mut config_lines = Vec::new();
    for idx in 0..app.config_key_count() {
        if let Some((key, value)) = app.config_key_value_at(idx) {
            let active = section_active(app, SettingsSection::Config)
                && app.settings_config_key_index == idx;
            let scope = config_key_scope(&key);
            let key_style = if active {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Cyan)
            };
            let scope_style = match scope {
                SettingScope::Editable => Style::default().fg(Color::Green),
                SettingScope::EnvOnly => Style::default().fg(Color::DarkGray),
                SettingScope::BuiltIn => Style::default().fg(Color::Yellow),
            };
            let value_style = if active {
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
            config_lines.push(Line::from(vec![
                Span::styled(format!("{key}: "), key_style),
                Span::styled(format!("[{}] ", scope.label()), scope_style),
                Span::styled(truncate(&value, config_value_width.max(20)), value_style),
            ]));
        }
    }
    if app.settings_edit_config_key.is_some() && app.settings_add_symbol_input.is_some() {
        let key = app.settings_edit_config_key.as_deref().unwrap_or("CONFIG");
        let buf = app.settings_add_symbol_input.as_deref().unwrap_or("");
        let config_edit = Paragraph::new(vec![
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
                "[Enter] save  [Esc] cancel",
                Style::default().fg(Color::DarkGray),
            )),
        ])
        .block(config_block);
        f.render_widget(config_edit, area);
    } else {
        f.render_widget(Paragraph::new(config_lines).block(config_block), area);
    }
}
