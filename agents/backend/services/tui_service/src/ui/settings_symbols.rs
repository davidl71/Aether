use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{List, ListItem, Paragraph},
    Frame,
};

use crate::app::App;
use crate::workspace::SettingsSection;

use super::{section_active, section_block};

pub(crate) fn render_settings_symbols_section(f: &mut Frame, app: &App, area: Rect) {
    let watchlist = app.watchlist();
    let override_note = if app.watchlist_override.is_some() {
        " (override; r = reset to config)"
    } else {
        " (edit config / WATCHLIST to persist)"
    };
    let symbols_block = section_block(
        &format!("Symbols / watchlist{}", override_note),
        section_active(app, SettingsSection::Symbols),
    );

    if app.settings_add_symbol_input.is_some() && app.settings_edit_config_key.is_none() {
        let buf = app.settings_add_symbol_input.as_deref().unwrap_or("");
        let prompt_lines = vec![
            Line::from(vec![
                Span::raw("Add symbol: "),
                Span::styled(
                    format!("{buf}_"),
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(Span::styled(
                "[Enter] confirm  [Esc] cancel",
                Style::default().fg(Color::DarkGray),
            )),
        ];
        f.render_widget(Paragraph::new(prompt_lines).block(symbols_block), area);
    } else if watchlist.is_empty() {
        let line = Line::from(Span::styled(
            "No symbols. Press 'a' to add (in-memory), or set WATCHLIST / config strategy.symbols.",
            Style::default().fg(Color::DarkGray),
        ));
        f.render_widget(Paragraph::new(line).block(symbols_block), area);
    } else {
        let items: Vec<ListItem> = watchlist
            .iter()
            .enumerate()
            .map(|(i, sym)| {
                let selected = i == app.settings_symbol_index;
                let style = if selected {
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::Cyan)
                };
                ListItem::new(Line::from(vec![
                    Span::styled("[x] ", style),
                    Span::styled(sym.as_str(), style),
                ]))
            })
            .collect();
        f.render_widget(List::new(items).block(symbols_block), area);
    }
}
