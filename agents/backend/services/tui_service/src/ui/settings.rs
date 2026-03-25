//! Settings tab: backend health, editable config overrides, and watchlist management.

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, List, ListItem, Paragraph, Row, Table},
    Frame,
};

use crate::app::App;
use crate::config::{config_key_scope, credential_scope, SettingScope};

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

#[allow(dead_code)]
fn section_active(app: &App, idx: usize) -> bool {
    app.settings_section_index == idx
}

#[allow(dead_code)]
fn section_block(title: impl Into<String>, active: bool) -> Block<'static> {
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

pub fn render_settings(f: &mut Frame, app: &App, area: Rect) {
    let SettingsLayout {
        health: health_area,
        config: config_area,
        symbols: symbols_area,
        sources: sources_area,
        hint: hint_area,
    } = settings_layout(area);
    render_settings_health_section(f, app, health_area);
    render_settings_config_section(f, app, config_area);
    render_settings_symbols_section(f, app, symbols_area);
    render_settings_sources_section(f, app, sources_area);
    render_settings_hint_section(f, app, hint_area);
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}…", &s[..max.saturating_sub(1)])
    }
}

#[derive(Default)]
struct BackendHealthCounts {
    total: usize,
    ok: usize,
    degraded: usize,
    error: usize,
    disabled: usize,
    stale: usize,
}

fn backend_health_counts(app: &App) -> BackendHealthCounts {
    let mut counts = BackendHealthCounts::default();
    for state in app.backend_health.values() {
        counts.total += 1;
        match state.status.as_str() {
            "ok" => counts.ok += 1,
            "degraded" => counts.degraded += 1,
            "error" => counts.error += 1,
            "disabled" => counts.disabled += 1,
            _ => {}
        }
        if state
            .extra
            .get("stale")
            .map(String::as_str)
            .is_some_and(|value| value == "true")
        {
            counts.stale += 1;
        }
    }
    counts
}

#[allow(dead_code)]
pub(crate) fn render_settings_health_section(f: &mut Frame, app: &App, area: Rect) {
    let backends_block = section_block("System health", section_active(app, 0));

    use crate::events::ConnectionState;
    let (nats_sym, nats_color, nats_label) = match app.nats_status.state {
        ConnectionState::Connected => ("●", Color::Green, "Connected"),
        ConnectionState::Starting => ("◌", Color::Yellow, "Connecting…"),
        ConnectionState::Retrying => ("⚠", Color::Red, "Retrying"),
    };

    let backend_entry = app
        .backend_health
        .get("backend_service")
        .or_else(|| app.backend_health.get("backend"));
    let (be_sym, be_color, be_label) = match backend_entry {
        Some(h) if h.status == "ok" => ("●", Color::Green, h.status.as_str().to_string()),
        Some(h) => ("⚠", Color::Yellow, h.status.clone()),
        None => ("?", Color::DarkGray, "no health report".to_string()),
    };

    let tws_ok = app
        .snapshot()
        .as_ref()
        .is_some_and(|s| s.inner.metrics.tws_ok);
    let (tws_sym, tws_color) = if tws_ok {
        ("●", Color::Green)
    } else {
        ("✗", Color::DarkGray)
    };

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
    let backend_counts = backend_health_counts(app);
    let snapshot_age = app
        .snapshot()
        .as_ref()
        .map(|s| format!("{}s", s.age_secs()))
        .unwrap_or_else(|| "—".to_string());
    let flow_lines = settings_health_flow_lines(
        app,
        nats_sym.to_string(),
        nats_color,
        nats_label.to_string(),
        be_sym.to_string(),
        be_color,
        be_label,
        tws_ok,
        tws_sym.to_string(),
        tws_color,
        live_src.to_string(),
        live_priority,
        live_age,
        sym_count,
        yield_symbol.to_string(),
        yield_pts,
        yield_age,
        &backend_counts,
        snapshot_age,
    );
    let component_lines = settings_component_lines(app, &backend_counts);
    let health_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(6), Constraint::Length(4)])
        .split(area);
    f.render_widget(Paragraph::new(flow_lines).block(backends_block.clone()), health_chunks[0]);
    f.render_widget(
        Paragraph::new(component_lines).block(
            Block::default()
                .title(" Components ")
                .borders(Borders::LEFT | Borders::RIGHT | Borders::BOTTOM)
                .border_style(if section_active(app, 0) {
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::DarkGray)
                }),
        ),
        health_chunks[1],
    );
}

#[allow(dead_code)]
pub(crate) fn render_settings_config_section(f: &mut Frame, app: &App, area: Rect) {
    let config_title = if let Some(ref key) = app.settings_edit_config_key {
        let scope = config_key_scope(key);
        format!("Config overrides (editing {key} [{}])", scope.label())
    } else {
        "Config overrides (editable; session only)".to_string()
    };
    let config_block = section_block(&config_title, section_active(app, 1));
    let config_value_width = area.width.saturating_sub(20) as usize;
    let mut config_lines = Vec::new();
    for idx in 0..=9 {
        if let Some((key, value)) = app.config_key_value_at(idx) {
            let active = section_active(app, 1) && app.settings_config_key_index == idx;
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

#[allow(dead_code)]
pub(crate) fn render_settings_symbols_section(f: &mut Frame, app: &App, area: Rect) {
    let watchlist = app.watchlist();
    let override_note = if app.watchlist_override.is_some() {
        " (override; r = reset to config)"
    } else {
        " (edit config / WATCHLIST to persist)"
    };
    let symbols_block = section_block(
        &format!("Symbols / watchlist{}", override_note),
        section_active(app, 2),
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

#[allow(dead_code)]
pub(crate) fn render_settings_sources_section(f: &mut Frame, app: &App, area: Rect) {
    let sources_block = section_block(
        "Data sources (credential origin: env / keyring / file / built-in)",
        section_active(app, 3),
    );
    let source_rows = settings_source_rows(app);
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
    f.render_widget(sources_table, area);
}

#[allow(dead_code)]
pub(crate) fn render_settings_hint_section(f: &mut Frame, app: &App, area: Rect) {
    let hint_text = match app.settings_section_index {
        1 => " 0 = Settings  ↑↓ key  e/Enter edit override  Active section: Config (editable) ",
        2 => " 0 = Settings  ↑↓ symbol  a add symbol  Del remove  Active section: Symbols ",
        3 => " 0 = Settings  ↑↓ section  Active section: Data Sources (credential origin: env/keyring/file/built-in) ",
        _ => " 0 = Settings  ↑↓ section  Enter inspect  Active section: Backends ",
    };
    let hint = Line::from(Span::styled(
        hint_text,
        Style::default().fg(Color::DarkGray),
    ));
    f.render_widget(Paragraph::new(hint), area);
}

#[allow(dead_code)]
fn settings_health_flow_lines(
    app: &App,
    nats_sym: String,
    nats_color: Color,
    nats_label: String,
    be_sym: String,
    be_color: Color,
    be_label: String,
    tws_ok: bool,
    tws_sym: String,
    tws_color: Color,
    live_src: String,
    live_priority: u32,
    live_age: u64,
    sym_count: usize,
    yield_symbol: String,
    yield_pts: usize,
    yield_age: Option<i64>,
    backend_counts: &BackendHealthCounts,
    snapshot_age: String,
) -> Vec<Line<'static>> {
    let yield_age = yield_age
        .map(|a| a.to_string())
        .unwrap_or_else(|| "—".to_string());
    let transport_detail = if app.nats_status.detail.trim().is_empty() {
        "waiting for connection state".to_string()
    } else {
        truncate(&app.nats_status.detail, 42)
    };
    let yw_entry = app
        .backend_health
        .get("tws_yield_curve_daemon")
        .or_else(|| app.backend_health.get("yield_curve_writer"));
    let (yw_sym, yw_color, yw_lbl) = match yw_entry {
        Some(h) if h.status == "ok" => ("●", Color::Green, "ok"),
        Some(_) => ("⚠", Color::Yellow, "degraded"),
        None => ("?", Color::DarkGray, "not reporting"),
    };

    vec![
        Line::from(vec![
            Span::raw(" TUI -> "),
            Span::styled(nats_sym.clone(), Style::default().fg(nats_color)),
            Span::raw(" NATS transport "),
            Span::styled(nats_label, Style::default().fg(nats_color)),
            Span::raw("  detail: "),
            Span::styled(transport_detail, Style::default().fg(Color::DarkGray)),
        ]),
        Line::from(vec![
            Span::raw("  +- system.health services "),
            Span::styled(
                format!(
                    "total {} ok {} degraded {} error {} disabled {} stale {}",
                    backend_counts.total,
                    backend_counts.ok,
                    backend_counts.degraded,
                    backend_counts.error,
                    backend_counts.disabled,
                    backend_counts.stale,
                ),
                Style::default().fg(Color::Cyan),
            ),
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
                live_src.clone(),
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
            Span::raw("  |  +- snapshot age: "),
            Span::styled(snapshot_age, Style::default().fg(Color::Cyan)),
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
            Span::raw(format!("{yield_age}s")),
        ]),
        Line::from(vec![
            Span::raw("  +- tws_yield_curve_daemon "),
            Span::styled(
                format!("{} {}", yw_sym, yw_lbl),
                Style::default().fg(yw_color),
            ),
        ]),
    ]
}

#[allow(dead_code)]
fn settings_component_lines(app: &App, backend_counts: &BackendHealthCounts) -> Vec<Line<'static>> {
    let transport_state = match app.nats_status.state {
        crate::events::ConnectionState::Connected => ("●", Color::Green, "transport connected"),
        crate::events::ConnectionState::Starting => ("◌", Color::Yellow, "transport connecting"),
        crate::events::ConnectionState::Retrying => ("⚠", Color::Red, "transport retrying"),
    };
    let mut component_names: Vec<_> = app.backend_health.keys().cloned().collect();
    component_names.sort();
    let mut lines = vec![
        Line::from(vec![
            Span::styled(
                format!("{} ", transport_state.0),
                Style::default().fg(transport_state.1),
            ),
            Span::styled("NATS transport", Style::default().fg(Color::Cyan)),
            Span::raw(" "),
            Span::styled(transport_state.2, Style::default().fg(transport_state.1)),
            Span::raw("  "),
            Span::styled(app.nats_status.detail.clone(), Style::default().fg(Color::DarkGray)),
        ]),
        Line::from(vec![
            Span::styled(
                format!("  services [{} total]", backend_counts.total),
                Style::default().fg(Color::Cyan),
            ),
            Span::raw(" "),
            Span::styled(
                format!(
                    "ok {} degraded {} error {} disabled {} stale {}",
                    backend_counts.ok,
                    backend_counts.degraded,
                    backend_counts.error,
                    backend_counts.disabled,
                    backend_counts.stale,
                ),
                Style::default().fg(Color::DarkGray),
            ),
        ]),
    ];
    if component_names.is_empty() {
        lines.push(Line::from(Span::styled(
            "  No system.health service entries yet.",
            Style::default().fg(Color::DarkGray),
        )));
        return lines;
    }
    lines.extend(component_names.into_iter().filter_map(|name| {
        let state = app.backend_health.get(&name)?;
        let (sym, color) = match state.status.as_str() {
            "ok" => ("●", Color::Green),
            "error" | "disabled" => ("✗", Color::Red),
            _ => ("⚠", Color::Yellow),
        };
        let mut spans = vec![
            Span::styled(format!("  {sym} "), Style::default().fg(color)),
            Span::styled(format!("{name:<18}"), Style::default().fg(Color::Cyan)),
            Span::styled(state.status.clone(), Style::default().fg(color)),
            Span::raw("  "),
            Span::styled(
                truncate(&state.updated_at, 19),
                Style::default().fg(Color::DarkGray),
            ),
        ];
        if let Some(error) = state.error.as_deref() {
            spans.push(Span::raw("  "));
            spans.push(Span::styled(truncate(error, 28), Style::default().fg(Color::Red)));
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
    }));
    lines
}

#[allow(dead_code)]
fn settings_source_rows(app: &App) -> Vec<Row<'static>> {
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

    sources
        .iter()
        .map(|s| {
            let scope = credential_scope(s.name);
            let has_cred = match s.name {
                "yahoo" | "tws" | "mock" => true,
                name => *app.credential_status.get(name).unwrap_or(&false),
            };
            let credential_label = match s.name {
                "yahoo" | "tws" | "mock" => {
                    format!("{} [{}]", s.cred_key, scope.label())
                }
                name => match app.credential_source.get(name) {
                    Some(source) => format!("{} [{}] [{}]", s.cred_key, source, scope.label()),
                    None => format!("{} [{}]", s.cred_key, scope.label()),
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
        .collect()
}
