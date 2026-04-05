use std::collections::HashMap;

use chrono::{DateTime, TimeDelta, Utc};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use api::{BackendHealthState, NatsTransportHealthState};

use crate::app::App;
use crate::events::ConnectionState;
use crate::workspace::{SettingsHealthFocus, SettingsSection};

use super::{section_active, section_block, truncate};

const DEFAULT_HEALTH_STALE_AFTER_SECS: i64 = 45;

fn fmt_bytes_u64(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = 1024 * 1024;
    const GB: u64 = 1024 * 1024 * 1024;
    if bytes >= GB {
        format!("{:.1}GiB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1}MiB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1}KiB", bytes as f64 / KB as f64)
    } else {
        format!("{bytes}B")
    }
}

/// One or two lines of async-nats counter text (partial fields shown when only some are set).
fn transport_nats_counter_lines(transport: &NatsTransportHealthState, width: usize) -> Vec<String> {
    let mut byte_parts: Vec<String> = Vec::new();
    if let Some(b) = transport.in_bytes {
        byte_parts.push(format!("in {}", fmt_bytes_u64(b)));
    }
    if let Some(b) = transport.out_bytes {
        byte_parts.push(format!("out {}", fmt_bytes_u64(b)));
    }

    let mut msg_parts: Vec<String> = Vec::new();
    match (transport.in_messages, transport.out_messages) {
        (Some(i), Some(o)) => msg_parts.push(format!("msgs {i}/{o}")),
        (Some(i), None) => msg_parts.push(format!("msgs_in {i}")),
        (None, Some(o)) => msg_parts.push(format!("msgs_out {o}")),
        (None, None) => {}
    }
    if let Some(c) = transport.connects {
        msg_parts.push(format!("connects {c}"));
    }

    if byte_parts.is_empty() && msg_parts.is_empty() {
        return vec!["io: —".to_string()];
    }

    let bytes_seg = byte_parts.join("  ");
    let msg_seg = msg_parts.join("  ");
    let combined = match (bytes_seg.is_empty(), msg_seg.is_empty()) {
        (true, false) => format!("io: {msg_seg}"),
        (false, true) => format!("io: {bytes_seg}"),
        (false, false) => format!("io: {bytes_seg}  {msg_seg}"),
        (true, true) => "io: —".to_string(),
    };

    let budget = width.max(32);
    if combined.len() <= budget.saturating_sub(2) || width >= 78 {
        return vec![combined];
    }

    let mut out = Vec::new();
    if !bytes_seg.is_empty() {
        out.push(format!("io: {bytes_seg}"));
    }
    if !msg_seg.is_empty() {
        out.push(format!("   {msg_seg}"));
    }
    if out.is_empty() {
        vec!["io: —".to_string()]
    } else {
        out
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

pub(crate) fn render_settings_health_section(f: &mut Frame, app: &App, area: Rect) {
    let backends_block = section_block(
        "System health",
        section_active(app, SettingsSection::Health),
    );
    let now = Utc::now();
    let stale_after = TimeDelta::seconds(DEFAULT_HEALTH_STALE_AFTER_SECS);
    let transport = app.nats_transport.effective_at(now, stale_after);
    let backends = effective_backends(app, now, stale_after);

    let (nats_sym, nats_color, nats_label) = match app.nats_status.state {
        ConnectionState::Connected => ("●", Color::Green, "Connected"),
        ConnectionState::Starting => ("◌", Color::Yellow, "Connecting…"),
        ConnectionState::Retrying => ("⚠", Color::Red, "Retrying"),
    };

    let backend_entry = backends
        .get("backend_service")
        .or_else(|| backends.get("backend"));
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
        DateTime::parse_from_rfc3339(&c.timestamp)
            .ok()
            .map(|dt| (Utc::now() - dt.with_timezone(&Utc)).num_seconds())
    });
    let backend_counts = backend_health_counts(&backends);
    let snapshot_age = app
        .snapshot()
        .as_ref()
        .map(|s| format!("{}s", s.age_secs()))
        .unwrap_or_else(|| "—".to_string());
    let content_w = area.width.saturating_sub(2) as usize;
    let flow_lines = settings_health_flow_lines(
        app,
        &transport,
        &backends,
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
        content_w,
    );
    let health_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(6), Constraint::Length(4)])
        .split(area);
    let component_lines = settings_component_lines(
        &transport,
        &backends,
        &backend_counts,
        health_chunks[1].width.saturating_sub(2) as usize,
    );
    f.render_widget(
        Paragraph::new(flow_lines).block(backends_block.clone()),
        health_chunks[0],
    );
    f.render_widget(
        Paragraph::new(component_lines).block(
            Block::default()
                .title(" Components ")
                .borders(Borders::LEFT | Borders::RIGHT | Borders::BOTTOM)
                .border_style(
                    if section_active(app, SettingsSection::Health)
                        && app.settings_health_focus == SettingsHealthFocus::Services
                    {
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::DarkGray)
                    },
                ),
        ),
        health_chunks[1],
    );
}

fn effective_backends(
    app: &App,
    now: DateTime<Utc>,
    stale_after: TimeDelta,
) -> HashMap<String, BackendHealthState> {
    app.backend_health
        .iter()
        .map(|(name, state)| (name.clone(), state.effective_at(now, stale_after)))
        .collect()
}

fn backend_health_counts(backends: &HashMap<String, BackendHealthState>) -> BackendHealthCounts {
    let mut counts = BackendHealthCounts::default();
    for state in backends.values() {
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

fn transport_summary_label(transport: &NatsTransportHealthState) -> (&'static str, Color) {
    match transport.status.as_str() {
        "ok" => ("●", Color::Green),
        "error" | "disabled" => ("✗", Color::Red),
        "degraded" => ("⚠", Color::Yellow),
        _ => ("?", Color::DarkGray),
    }
}

/// Snapshot / JetStream / KV fields from backend publisher or REST health merge (optional).
fn transport_snapshot_jetstream_kv_summary(transport: &NatsTransportHealthState) -> String {
    let conn = if transport.connection_state.is_empty() {
        "—".to_string()
    } else {
        transport.connection_state.clone()
    };
    let snap = match (
        transport.snapshot_backend_id.as_deref(),
        transport.snapshot_generated_at.as_deref(),
    ) {
        (Some(bid), Some(ts)) => {
            let age = DateTime::parse_from_rfc3339(ts)
                .ok()
                .map(|dt| (Utc::now() - dt.with_timezone(&Utc)).num_seconds());
            format!(
                "{} @ {} ({}s)",
                bid,
                truncate(ts, 19),
                age.map(|a| a.to_string())
                    .unwrap_or_else(|| "—".to_string())
            )
        }
        (Some(bid), None) => format!("{bid} (no ts)"),
        _ => "—".to_string(),
    };
    let js = match (
        transport.jetstream_enabled,
        transport.jetstream_stream_ready,
        transport.jetstream_publish_failures,
    ) {
        (Some(en), Some(rd), Some(fail)) => format!("js on={en} ready={rd} fail={fail}"),
        (Some(en), Some(rd), None) => format!("js on={en} ready={rd}"),
        (Some(en), None, _) => format!("js on={en}"),
        _ => "js —".to_string(),
    };
    let kv = match (
        transport.kv_reachable,
        transport.kv_bucket.as_deref(),
        transport.kv_last_check_at.as_deref(),
    ) {
        (Some(ok), Some(b), Some(at)) => {
            format!("kv ok={ok} {} @ {}", truncate(b, 12), truncate(at, 19))
        }
        (Some(ok), Some(b), None) => format!("kv ok={ok} {}", truncate(b, 16)),
        (Some(ok), None, _) => format!("kv ok={ok}"),
        _ => "kv —".to_string(),
    };
    format!("link: {conn}  snap: {snap}  {js}  {kv}")
}

fn settings_health_flow_lines(
    app: &App,
    transport: &NatsTransportHealthState,
    backends: &HashMap<String, BackendHealthState>,
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
    content_width: usize,
) -> Vec<Line<'static>> {
    let yield_age = yield_age
        .map(|a| a.to_string())
        .unwrap_or_else(|| "—".to_string());
    let transport_detail = transport
        .hint
        .clone()
        .or_else(|| transport.error.clone())
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| {
            if app.nats_status.detail.trim().is_empty() {
                "waiting for transport observations".to_string()
            } else {
                app.nats_status.detail.clone()
            }
        });
    let transport_age = transport
        .age_secs_at(Utc::now())
        .map(|value| format!("{value}s"))
        .unwrap_or_else(|| "—".to_string());
    let transport_role = transport.role().unwrap_or("subscriber").to_string();
    let transport_subject = transport.subject().unwrap_or("system.health").to_string();
    let (transport_sym, transport_color) = transport_summary_label(transport);
    let transport_rtt = transport
        .last_rtt_ms
        .map(|v| format!("{v}ms"))
        .unwrap_or_else(|| "—".to_string());
    let transport_stat_lines = transport_nats_counter_lines(transport, content_width);
    let yw_entry = backends
        .get("tws_yield_curve_daemon")
        .or_else(|| backends.get("yield_curve_writer"));
    let (yw_sym, yw_color, yw_lbl) = match yw_entry {
        Some(h) if h.status == "ok" => ("●", Color::Green, "ok"),
        Some(h) => ("⚠", Color::Yellow, h.status.as_str()),
        None => ("?", Color::DarkGray, "not reporting"),
    };

    let (strategy_nats_status_word, strategy_nats_status_color) = if !app
        .config
        .strategy_nats_subscribe
    {
        ("off", Color::DarkGray)
    } else {
        match app.nats_status.state {
            ConnectionState::Connected => ("ok", Color::Green),
            ConnectionState::Starting | ConnectionState::Retrying => ("degraded", Color::Yellow),
        }
    };
    let strategy_nats_subj_summary = if app.config.strategy_nats_subscribe {
        format!(
            "{} | {}",
            truncate(&app.config.strategy_nats_signal_subject, 20),
            truncate(&app.config.strategy_nats_decision_subject, 20),
        )
    } else {
        "—".to_string()
    };

    let w = content_width.max(48);
    let mut lines: Vec<Line<'static>> = vec![Line::from(vec![
        Span::styled(
            " Transport ",
            Style::default()
                .fg(
                    if section_active(app, SettingsSection::Health)
                        && app.settings_health_focus == SettingsHealthFocus::Transport
                    {
                        Color::Yellow
                    } else {
                        Color::DarkGray
                    },
                )
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" "),
        Span::styled(transport_sym, Style::default().fg(transport_color)),
        Span::raw(" "),
        Span::styled(
            transport.status.clone(),
            Style::default().fg(transport_color),
        ),
        Span::raw("  role: "),
        Span::styled(
            truncate(&transport_role, 16),
            Style::default().fg(Color::Cyan),
        ),
        Span::raw("  subj: "),
        Span::styled(
            truncate(&transport_subject, 20),
            Style::default().fg(Color::Cyan),
        ),
        Span::raw("  age: "),
        Span::styled(
            truncate(&transport_age, 8),
            Style::default().fg(Color::Cyan),
        ),
        Span::raw("  rtt: "),
        Span::styled(transport_rtt, Style::default().fg(Color::Cyan)),
        Span::raw("  recon: "),
        Span::styled(
            transport.reconnect_count.to_string(),
            Style::default().fg(Color::Cyan),
        ),
    ])];
    for stat_line in transport_stat_lines {
        lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled(
                truncate(&stat_line, w),
                Style::default().fg(Color::DarkGray),
            ),
        ]));
    }
    lines.push(Line::from(vec![
        Span::raw("  "),
        Span::styled(
            truncate(&transport_snapshot_jetstream_kv_summary(transport), w),
            Style::default().fg(Color::DarkGray),
        ),
    ]));
    lines.extend(vec![
        Line::from(vec![
            Span::raw(" TUI -> "),
            Span::styled(nats_sym.clone(), Style::default().fg(nats_color)),
            Span::raw(" connection "),
            Span::styled(nats_label, Style::default().fg(nats_color)),
            Span::raw("  detail: "),
            Span::styled(
                truncate(&transport_detail, 42),
                Style::default().fg(Color::DarkGray),
            ),
        ]),
        Line::from(vec![
            Span::raw(" Strategy NATS: "),
            Span::styled(
                strategy_nats_status_word,
                Style::default().fg(strategy_nats_status_color),
            ),
            Span::raw("  "),
            Span::styled(strategy_nats_subj_summary, Style::default().fg(Color::Cyan)),
            Span::raw("  counts "),
            Span::styled(
                format!(
                    "{}/{}",
                    app.strategy_nats_signal_count, app.strategy_nats_decision_count
                ),
                Style::default().fg(Color::Cyan),
            ),
            Span::raw("  last: "),
            Span::styled(
                truncate(&app.strategy_nats_last, 32),
                Style::default().fg(Color::DarkGray),
            ),
        ]),
        Line::from(vec![
            Span::raw(" Services -> "),
            Span::styled(
                format!(
                    "{} total / ok {} / degraded {} / error {} / disabled {} / stale {}",
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
    ]);
    lines
}

fn settings_component_lines(
    transport: &NatsTransportHealthState,
    backends: &HashMap<String, BackendHealthState>,
    backend_counts: &BackendHealthCounts,
    content_width: usize,
) -> Vec<Line<'static>> {
    let w = content_width.max(40);
    let (transport_sym, transport_color) = transport_summary_label(transport);
    let transport_detail = transport
        .hint
        .as_deref()
        .or(transport.error.as_deref())
        .unwrap_or("observing system.health")
        .to_string();
    let mut component_names: Vec<_> = backends.keys().cloned().collect();
    component_names.sort();
    let mut lines = vec![Line::from(vec![
        Span::styled(
            format!("{transport_sym} "),
            Style::default().fg(transport_color),
        ),
        Span::styled("NATS transport", Style::default().fg(Color::Cyan)),
        Span::raw(" "),
        Span::styled(
            transport.status.clone(),
            Style::default().fg(transport_color),
        ),
        Span::raw("  "),
        Span::styled(
            truncate(&transport_detail, w.saturating_sub(24)),
            Style::default().fg(Color::DarkGray),
        ),
    ])];
    for stat_line in transport_nats_counter_lines(transport, content_width) {
        lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled(
                truncate(&stat_line, w),
                Style::default().fg(Color::DarkGray),
            ),
        ]));
    }
    let rtt_rec = format!(
        "last_rtt {}  reconnects {}",
        transport
            .last_rtt_ms
            .map(|v| format!("{v}ms"))
            .unwrap_or_else(|| "—".to_string()),
        transport.reconnect_count,
    );
    lines.push(Line::from(vec![
        Span::raw("  "),
        Span::styled(truncate(&rtt_rec, w), Style::default().fg(Color::DarkGray)),
    ]));
    lines.push(Line::from(vec![
        Span::raw("  "),
        Span::styled(
            truncate(&transport_snapshot_jetstream_kv_summary(transport), w),
            Style::default().fg(Color::DarkGray),
        ),
    ]));
    lines.push(Line::from(vec![
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
    ]));
    if component_names.is_empty() {
        lines.push(Line::from(Span::styled(
            "  No system.health service entries yet.",
            Style::default().fg(Color::DarkGray),
        )));
        return lines;
    }
    lines.extend(component_names.into_iter().filter_map(|name| {
        let state = backends.get(&name)?;
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
    }));
    lines
}
