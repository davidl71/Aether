//! Settings tab: backend health, editable config overrides, and watchlist management.

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, List, ListItem, Paragraph, Row, Table},
    Frame,
};

use crate::app::App;

pub fn render_settings(f: &mut Frame, app: &App, area: Rect) {
    let wide_layout = area.width >= 120 && area.height >= 18;
    let (health_area, config_area, symbols_area, sources_area, hint_area) = if wide_layout {
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
            .constraints([Constraint::Percentage(58), Constraint::Percentage(42)])
            .split(rows[0]);
        let bottom = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(34), Constraint::Percentage(66)])
            .split(rows[1]);
        (top[0], top[1], bottom[0], bottom[1], rows[2])
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
        (chunks[0], chunks[1], chunks[2], chunks[3], chunks[4])
    };

    let section_active = |idx: usize| app.settings_section_index == idx;
    let section_block = |title: &str, active: bool| {
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
    };

    // 1) Data flow diagram (system.health + NATS state)
    let backends_block = section_block("System health", section_active(0));

    // NATS node
    use crate::events::ConnectionState;
    let (nats_sym, nats_color, nats_label) = match app.nats_status.state {
        ConnectionState::Connected => ("●", Color::Green, "Connected"),
        ConnectionState::Starting => ("◌", Color::Yellow, "Connecting…"),
        ConnectionState::Retrying => ("⚠", Color::Red, "Retrying"),
    };

    // backend_service node (from system.health)
    let backend_entry = app.backend_health.get("backend");
    let (be_sym, be_color, be_label) = match backend_entry {
        Some(h) if h.status == "ok" => ("●", Color::Green, h.status.as_str().to_string()),
        Some(h) => ("⚠", Color::Yellow, h.status.clone()),
        None => ("?", Color::DarkGray, "no health report".to_string()),
    };

    // TWS node (from snapshot metrics)
    let tws_ok = app
        .snapshot()
        .as_ref()
        .is_some_and(|s| s.inner.metrics.tws_ok);
    let (tws_sym, tws_color) = if tws_ok {
        ("●", Color::Green)
    } else {
        ("✗", Color::DarkGray)
    };

    // Live market data source
    let live_src = app
        .live_market_data_source
        .as_ref()
        .map(|m| m.source.as_str())
        .unwrap_or("none");
    let live_priority = app
        .live_market_data_source
        .as_ref()
        .map(|m| m.priority)
        .unwrap_or(0);
    let live_age = app
        .live_market_data_source
        .as_ref()
        .map(|m| m.age_secs())
        .unwrap_or(0);
    let sym_count = app.watchlist().len();
    let yield_symbol = app
        .yield_curve
        .as_ref()
        .map(|c| c.symbol.as_str())
        .unwrap_or("—");
    let yield_pts = app.yield_curve.as_ref().map(|c| c.point_count).unwrap_or(0);
    let yield_age = app.yield_curve.as_ref().and_then(|c| {
        chrono::DateTime::parse_from_rfc3339(&c.timestamp)
            .ok()
            .map(|dt| (chrono::Utc::now() - dt.with_timezone(&chrono::Utc)).num_seconds())
    });

    let flow_lines = vec![
        Line::from(vec![
            Span::raw(" TUI -> "),
            Span::styled(nats_sym, Style::default().fg(nats_color)),
            Span::raw(" NATS "),
            Span::styled(nats_label, Style::default().fg(nats_color)),
        ]),
        Line::from(vec![
            Span::raw("  +- "),
            Span::styled(be_sym, Style::default().fg(be_color)),
            Span::raw(" backend_service "),
            Span::styled(be_label, Style::default().fg(be_color)),
        ]),
        Line::from(vec![
            Span::raw("  |  +- "),
            Span::styled(tws_sym, Style::default().fg(tws_color)),
            Span::raw(" TWS "),
            Span::styled(
                if tws_ok { "connected" } else { "not connected" },
                Style::default().fg(tws_color),
            ),
            if live_src == "tws" {
                Span::styled(" [LIVE]", Style::default().fg(Color::Green))
            } else {
                Span::raw("")
            },
        ]),
        Line::from(vec![
            Span::raw("  |  +- "),
            Span::styled(
                live_src,
                Style::default().fg(if live_src == "none" {
                    Color::DarkGray
                } else {
                    Color::Green
                }),
            ),
            Span::raw(" [p"),
            Span::raw(live_priority.to_string()),
            Span::raw("] "),
            Span::raw(if live_src == "none" { "idle" } else { "LIVE" }),
            Span::raw("  age: "),
            Span::raw(format!("{live_age}s")),
        ]),
        Line::from(vec![
            Span::raw("  |     +- symbols: "),
            Span::styled(format!("{sym_count}"), Style::default().fg(Color::Cyan)),
        ]),
        Line::from(vec![
            Span::raw("  |     +- yield: "),
            Span::styled(yield_symbol, Style::default().fg(Color::Cyan)),
            Span::raw("  "),
            Span::styled(format!("{yield_pts}pts"), Style::default().fg(Color::Cyan)),
            Span::raw("  age: "),
            Span::raw(format!(
                "{}s",
                yield_age
                    .map(|a| a.to_string())
                    .unwrap_or_else(|| "—".to_string())
            )),
        ]),
        Line::from(vec![
            Span::raw("  +- yield_curve_writer "),
            {
                let yw_entry = app.backend_health.get("yield_curve_writer");
                let (yw_sym, yw_color, yw_lbl) = match yw_entry {
                    Some(h) if h.status == "ok" => ("●", Color::Green, "ok"),
                    Some(_) => ("⚠", Color::Yellow, "degraded"),
                    None => ("?", Color::DarkGray, "not reporting"),
                };
                Span::styled(
                    format!("{} {}", yw_sym, yw_lbl),
                    Style::default().fg(yw_color),
                )
            },
        ]),
    ];

    let mut component_names: Vec<_> = app.backend_health.keys().cloned().collect();
    component_names.sort();
    let component_lines: Vec<Line> = if component_names.is_empty() {
        vec![Line::from(Span::styled(
            "No system.health components reported yet.",
            Style::default().fg(Color::DarkGray),
        ))]
    } else {
        component_names
            .into_iter()
            .filter_map(|name| {
                let state = app.backend_health.get(&name)?;
                let (sym, color) = match state.status.as_str() {
                    "ok" => ("●", Color::Green),
                    "error" | "disabled" => ("✗", Color::Red),
                    _ => ("⚠", Color::Yellow),
                };
                let mut spans = vec![
                    Span::styled(format!("{sym} "), Style::default().fg(color)),
                    Span::styled(format!("{name:<18}"), Style::default().fg(Color::Cyan)),
                    Span::styled(state.status.as_str(), Style::default().fg(color)),
                    Span::raw("  "),
                    Span::styled(
                        truncate(&state.updated_at, 19),
                        Style::default().fg(Color::DarkGray),
                    ),
                ];
                if let Some(error) = state.error.as_deref() {
                    spans.push(Span::raw("  "));
                    spans.push(Span::styled(
                        truncate(error, 28),
                        Style::default().fg(Color::Red),
                    ));
                } else if let Some(hint) = state.hint.as_deref() {
                    spans.push(Span::raw("  "));
                    spans.push(Span::styled(
                        truncate(hint, 28),
                        Style::default().fg(Color::DarkGray),
                    ));
                } else if !state.extra.is_empty() {
                    let extras = state
                        .extra
                        .iter()
                        .take(2)
                        .map(|(k, v)| format!("{k}={v}"))
                        .collect::<Vec<_>>()
                        .join(" ");
                    if !extras.is_empty() {
                        spans.push(Span::raw("  "));
                        spans.push(Span::styled(
                            truncate(&extras, 28),
                            Style::default().fg(Color::DarkGray),
                        ));
                    }
                }
                Some(Line::from(spans))
            })
            .collect()
    };
    let health_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(6), Constraint::Length(4)])
        .split(health_area);
    f.render_widget(Paragraph::new(flow_lines).block(backends_block.clone()), health_chunks[0]);
    f.render_widget(
        Paragraph::new(component_lines).block(
            Block::default()
                .title(" Components ")
                .borders(Borders::LEFT | Borders::RIGHT | Borders::BOTTOM)
                .border_style(if section_active(0) {
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::DarkGray)
                }),
        ),
        health_chunks[1],
    );

    // 2) Config (read-only)
    let config_title = if let Some(ref key) = app.settings_edit_config_key {
        format!("Config overrides (editing {})", key)
    } else {
        "Config overrides (session only)".to_string()
    };
    let config_block = section_block(&config_title, section_active(1));
    let config_value_width = config_area.width.saturating_sub(20) as usize;
    let mut config_lines = Vec::new();
    for idx in 0..=4 {
        if let Some((key, value)) = app.config_key_value_at(idx) {
            let active = section_active(1) && app.settings_config_key_index == idx;
            let key_style = if active {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Cyan)
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
        f.render_widget(config_edit, config_area);
    } else {
        f.render_widget(Paragraph::new(config_lines).block(config_block), config_area);
    }

    // 3) Symbols (watchlist) — add (a), remove (Del), reset override (r)
    let watchlist = app.watchlist();
    let override_note = if app.watchlist_override.is_some() {
        " (override; r = reset to config)"
    } else {
        " (edit config / WATCHLIST to persist)"
    };
    let symbols_block = section_block(
        &format!("Symbols / watchlist{}", override_note),
        section_active(2),
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
        f.render_widget(Paragraph::new(prompt_lines).block(symbols_block), symbols_area);
    } else if watchlist.is_empty() {
        let line = Line::from(Span::styled(
            "No symbols. Press 'a' to add (in-memory), or set WATCHLIST / config strategy.symbols.",
            Style::default().fg(Color::DarkGray),
        ));
        f.render_widget(Paragraph::new(line).block(symbols_block), symbols_area);
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
        f.render_widget(List::new(items).block(symbols_block), symbols_area);
    }

    // 4) Data sources — credential status + live tick source
    let sources_block = section_block("Data sources", section_active(3));
    let live_source = app
        .live_market_data_source
        .as_ref()
        .map(|m| m.source.to_lowercase());

    struct SourceDef {
        name: &'static str,
        priority: &'static str,
        cred_key: &'static str,
        note: &'static str,
    }
    let sources = [
        SourceDef {
            name: "yahoo",
            priority: "50",
            cred_key: "(free)",
            note: "market quotes",
        },
        SourceDef {
            name: "fmp",
            priority: "60",
            cred_key: "FMP_API_KEY",
            note: "market + fundamentals",
        },
        SourceDef {
            name: "mock",
            priority: "0",
            cred_key: "(fixture/demo)",
            note: "deterministic provider",
        },
        SourceDef {
            name: "polygon",
            priority: "70",
            cred_key: "POLYGON_API_KEY",
            note: "market quotes (WebSocket)",
        },
        SourceDef {
            name: "tase",
            priority: "—",
            cred_key: "TASE_API_KEY",
            note: "Israeli exchange",
        },
        SourceDef {
            name: "fred",
            priority: "—",
            cred_key: "FRED_API_KEY",
            note: "yield benchmarks (SOFR/Treasury)",
        },
        SourceDef {
            name: "tws",
            priority: "100",
            cred_key: "(TWS connection)",
            note: "IB broker push (highest priority)",
        },
    ];

    let configured_provider = app
        .snapshot()
        .as_ref()
        .and_then(|s| s.inner.market_data_source.as_deref().map(str::to_lowercase))
        .unwrap_or_default();

    let source_rows: Vec<Row> = sources
        .iter()
        .map(|s| {
            let has_cred = match s.name {
                "yahoo" | "tws" | "mock" => true,
                name => *app.credential_status.get(name).unwrap_or(&false),
            };
            let credential_label = match s.name {
                "yahoo" | "tws" | "mock" => s.cred_key.to_string(),
                name => match app.credential_source.get(name) {
                    Some(source) => format!("{} [{}]", s.cred_key, source),
                    None => s.cred_key.to_string(),
                },
            };
            let is_live = live_source.as_deref() == Some(s.name);
            let is_configured = configured_provider == s.name || configured_provider == "all";

            let (status_label, status_color) = if is_live {
                ("● LIVE", Color::Green)
            } else if !has_cred && s.name != "yahoo" && s.name != "tws" && s.name != "mock" {
                ("✗ no key", Color::Red)
            } else if is_configured || s.name == "yahoo" {
                ("idle", Color::DarkGray)
            } else {
                ("disabled", Color::DarkGray)
            };

            let cred_color = if has_cred || s.name == "yahoo" || s.name == "tws" || s.name == "mock" {
                Color::Green
            } else {
                Color::Red
            };

            Row::new([
                Cell::from(s.name),
                Cell::from(s.priority),
                Cell::from(credential_label).style(Style::default().fg(cred_color)),
                Cell::from(status_label).style(Style::default().fg(status_color)),
                Cell::from(s.note).style(Style::default().fg(Color::DarkGray)),
            ])
        })
        .collect();

    let sources_table = Table::new(
        source_rows,
        [
            Constraint::Length(8),
            Constraint::Length(6),
            Constraint::Length(28),
            Constraint::Length(10),
            Constraint::Min(10),
        ],
    )
    .header(
        Row::new(["Provider", "Pri", "Credential", "Status", "Purpose"])
            .style(Style::default().add_modifier(Modifier::BOLD | Modifier::UNDERLINED)),
    )
    .block(sources_block);
    f.render_widget(sources_table, sources_area);

    // 5) Hint line
    let hint_text = match app.settings_section_index {
        1 => " 0 = Settings  ↑↓ key  e/Enter edit override  Active section: Config ",
        2 => " 0 = Settings  ↑↓ symbol  a add symbol  Del remove  Active section: Symbols ",
        3 => " 0 = Settings  ↑↓ section  Active section: Data Sources (keys: keychain/env/file) ",
        _ => " 0 = Settings  ↑↓ section  Enter inspect  Active section: Backends ",
    };
    let hint = Line::from(Span::styled(
        hint_text,
        Style::default().fg(Color::DarkGray),
    ));
    f.render_widget(Paragraph::new(hint), hint_area);
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}…", &s[..max.saturating_sub(1)])
    }
}
