use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::app::App;
use crate::events::ConnectionState;
use crate::workspace::SettingsSection;

use super::{section_active, section_block, truncate};

#[derive(Default)]
struct BackendHealthCounts {
    total: usize,
    ok: usize,
    degraded: usize,
    error: usize,
    disabled: usize,
    stale: usize,
}

pub(crate) fn render_settings_health_section(f: &mut Frame, app: &App, area: Rect) {
    let backends_block = section_block("System health", section_active(app, SettingsSection::Health));

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
                .border_style(if section_active(app, SettingsSection::Health) {
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

fn settings_component_lines(app: &App, backend_counts: &BackendHealthCounts) -> Vec<Line<'static>> {
    let transport_state = match app.nats_status.state {
        ConnectionState::Connected => ("●", Color::Green, "transport connected"),
        ConnectionState::Starting => ("◌", Color::Yellow, "transport connecting"),
        ConnectionState::Retrying => ("⚠", Color::Red, "transport retrying"),
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
