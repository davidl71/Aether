//! Settings tab: backend service status, config options (read-only), symbol list (watchlist) with add/remove.

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
    Frame,
};

use crate::app::App;

pub fn render_settings(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4),
            Constraint::Min(4),
            Constraint::Length(4),
            Constraint::Min(4),
        ])
        .split(area);

    // 1) Backend services (system.health)
    let backends_block = Block::default()
        .title(" Backend services (system.health) ")
        .borders(Borders::ALL);
    let backend_rows: Vec<Row> = if app.backend_health.is_empty() {
        vec![Row::new(vec![
            Cell::from("No backends reported yet (connect to NATS)"),
            Cell::from(""),
            Cell::from(""),
        ])]
    } else {
        let mut names: Vec<_> = app.backend_health.keys().collect();
        names.sort();
        names
            .into_iter()
            .map(|id| {
                let h = app.backend_health.get(id).unwrap();
                let status_style = match h.status.as_str() {
                    "ok" => Style::default().fg(Color::Green),
                    "error" | "disabled" => Style::default().fg(Color::Red),
                    _ => Style::default().fg(Color::Yellow),
                };
                Row::new([
                    Cell::from(id.clone()),
                    Cell::from(h.status.clone()).style(status_style),
                    Cell::from(h.updated_at.clone()),
                ])
            })
            .collect()
    };
    let backend_table = Table::new(
        backend_rows,
        [Constraint::Length(12), Constraint::Length(10), Constraint::Min(8)],
    )
    .header(
        Row::new(["Backend", "Status", "Updated"])
            .style(Style::default().add_modifier(Modifier::BOLD | Modifier::UNDERLINED)),
    )
    .block(backends_block);
    f.render_widget(backend_table, chunks[0]);

    // 2) Config (read-only)
    let config_block = Block::default()
        .title(" Config (read-only; edit file or env to change) ")
        .borders(Borders::ALL);
    let config_lines = vec![
        Line::from(vec![
            Span::raw("NATS_URL: "),
            Span::styled(
                truncate(&app.config.nats_url, 50),
                Style::default().fg(Color::Cyan),
            ),
        ]),
        Line::from(vec![
            Span::raw("BACKEND_ID: "),
            Span::styled(
                app.config.backend_id.as_str(),
                Style::default().fg(Color::Cyan),
            ),
            Span::raw("  TICK_MS: "),
            Span::raw(app.config.tick_ms.to_string()),
            Span::raw("  SNAPSHOT_TTL_SECS: "),
            Span::raw(app.config.snapshot_ttl_secs.to_string()),
            Span::raw("  SPLIT_PANE: "),
            Span::raw(if app.config.split_pane { "true" } else { "false" }),
        ]),
    ];
    f.render_widget(
        Paragraph::new(config_lines).block(config_block),
        chunks[1],
    );

    // 3) Symbols (watchlist) — add (a), remove (Del), reset override (r)
    let watchlist = app.watchlist();
    let override_note = if app.watchlist_override.is_some() {
        " (override; r = reset to config)"
    } else {
        " (edit config / WATCHLIST to persist)"
    };
    let symbols_block = Block::default()
        .title(format!(" Symbols / watchlist{} ", override_note))
        .borders(Borders::ALL);

    if app.settings_add_symbol_input.is_some() {
        let buf = app.settings_add_symbol_input.as_deref().unwrap_or("");
        let line = Line::from(vec![
            Span::raw("Add symbol: "),
            Span::styled(
                format!("{buf}_"),
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
            ),
            Span::raw("  [Enter] confirm  [Esc] cancel"),
        ]);
        f.render_widget(
            Paragraph::new(line).block(symbols_block),
            chunks[2],
        );
    } else if watchlist.is_empty() {
        let line = Line::from(Span::styled(
            "No symbols. Press 'a' to add (in-memory), or set WATCHLIST / config strategy.symbols.",
            Style::default().fg(Color::DarkGray),
        ));
        f.render_widget(
            Paragraph::new(line).block(symbols_block),
            chunks[2],
        );
    } else {
        let mut spans = vec![];
        for (i, sym) in watchlist.iter().enumerate() {
            if i > 0 {
                spans.push(Span::raw(" "));
            }
            let selected = i == app.settings_symbol_index;
            if selected {
                spans.push(
                    Span::styled(
                        format!("[{}]", sym),
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    )
                );
            } else {
                spans.push(Span::styled(sym.as_str(), Style::default().fg(Color::Cyan)));
            }
        }
        spans.push(Span::raw("  ↑↓ select  a add  Del remove  r reset to config"));
        f.render_widget(
            Paragraph::new(Line::from(spans)).block(symbols_block),
            chunks[2],
        );
    }

    // Hint line
    let hint = Line::from(Span::styled(
        " 8 = Settings  ↑↓ section/symbol  a add symbol  Del remove  r reset watchlist ",
        Style::default().fg(Color::DarkGray),
    ));
    f.render_widget(Paragraph::new(hint), chunks[3]);
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}…", &s[..max.saturating_sub(1)])
    }
}
