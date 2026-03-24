//! Ratatui rendering: frame layout, tab bar, and per-tab view delegation.

mod alerts;
mod candlestick;
pub mod charts;
mod dashboard;
mod loans;
pub mod logs;
mod orders;
mod positions;
pub use positions::positions_display_info;
mod scenarios;
pub use scenarios::filtered_scenarios;
mod settings;
mod yield_curve;
pub use candlestick::Candle;
pub(crate) use yield_curve::render_yield_curve as render_yield_curve_tab;

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Tabs},
    Frame,
};

use api::CommandStatus;

use crate::app::{App, DetailPopupContent, InputMode, Tab};
use crate::events::{ConnectionState, ConnectionStatus, ConnectionTarget};

pub fn render(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(f.area());

    render_status_bar(f, app, chunks[0]);
    render_tab_bar(f, app, chunks[1]);
    render_main(f, app, chunks[2]);
    render_hint_bar(f, app, chunks[3]);

    if app.show_help {
        render_help_overlay(f, f.area());
    }
    if app.show_log_panel {
        render_log_panel_overlay(f, app, f.area());
    }
    if let Some(ref content) = app.detail_popup {
        render_detail_overlay(f, f.area(), content);
    }
}

fn render_status_bar(f: &mut Frame, app: &App, area: Rect) {
    let (mode, strategy, market_data_source, source_color, is_stale) =
        if let Some(ref snap) = app.snapshot() {
            let stale = snap.is_stale(app.config.snapshot_ttl_secs as i64);
            let color = if stale { Color::Yellow } else { Color::Green };
            let mds = snap
                .inner
                .market_data_source
                .clone()
                .unwrap_or_else(|| "NATS".into());
            (
                snap.dto().mode.as_str().to_owned(),
                snap.dto().strategy.as_str().to_owned(),
                mds,
                color,
                stale,
            )
        } else {
            (
                "---".into(),
                "---".into(),
                "NO DATA".into(),
                Color::DarkGray,
                false,
            )
        };

    let mut spans = vec![
        Span::raw(format!(" {} | ", app.config.backend_id.to_uppercase())),
        Span::styled(mode, Style::default().fg(Color::Cyan)),
        Span::raw(" | "),
        Span::styled(strategy, Style::default().fg(Color::Yellow)),
        Span::raw(" | "),
        Span::styled(
            "READ-ONLY",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
    ];

    if let Some(ref snap) = app.snapshot() {
        let aid = snap.dto().account_id.trim();
        if !aid.is_empty() {
            spans.push(Span::raw(" | "));
            spans.push(Span::styled(
                format!("Account: {}", aid),
                Style::default().fg(Color::Magenta),
            ));
        }
        if let Some(ref addr) = snap.dto().metrics.tws_address {
            spans.push(Span::raw(" | "));
            spans.push(Span::styled(
                format!("TWS: {}", addr),
                Style::default().fg(Color::Green),
            ));
        }
    }

    if app.split_pane {
        spans.push(Span::raw(" | "));
        spans.push(Span::styled(
            "PANE:DASH+POS",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ));
    }

    if let Some(meta) = app.live_market_data_source.as_ref() {
        let age_secs = meta.age_secs();
        let age_label = if age_secs <= 1 {
            "now".to_string()
        } else {
            format!("{}s ago", age_secs)
        };
        let color = if age_secs <= 2 {
            Color::Green
        } else if age_secs <= 6 {
            Color::Yellow
        } else {
            Color::Red
        };
        spans.push(Span::raw(" | "));
        spans.push(Span::styled(
            format!(
                "{}@{} ({})",
                meta.source.to_uppercase(),
                meta.priority,
                age_label
            ),
            Style::default().fg(color),
        ));
    }

    spans.push(Span::raw("  "));
    let pill_color = match market_data_source.to_lowercase().as_str() {
        "yahoo" => Color::Magenta,
        "polygon" => Color::Blue,
        "ib" | "ibkr" => Color::Green,
        _ => source_color,
    };
    spans.push(Span::styled(
        format!("[{}]", market_data_source.to_uppercase()),
        Style::default().fg(pill_color).add_modifier(Modifier::BOLD),
    ));

    if is_stale {
        spans.push(Span::styled(
            " [STALE]",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ));
    }

    if let Some(ref snap) = app.snapshot() {
        let age = snap.age_secs();
        let age_str = if age < 60 {
            format!("{}s ago", age)
        } else {
            format!("{}m ago", age / 60)
        };
        spans.push(Span::raw("  "));
        spans.push(Span::styled(
            format!("Updated {}", age_str),
            Style::default().fg(Color::DarkGray),
        ));
    }

    if let Some(ref w) = app.config_warning {
        spans.push(Span::raw("  "));
        spans.push(Span::styled(
            format!("Config: {}", truncate_detail(w, 24)),
            Style::default().fg(Color::Yellow),
        ));
    }

    if let Some((label, color)) = settings_mode_indicator(app) {
        spans.push(Span::raw("  "));
        spans.push(Span::styled(
            label,
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        ));
    }

    spans.push(Span::raw("  "));
    match app.market_open {
        Some(true) => spans.push(Span::styled(
            "MKT:OPEN",
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        )),
        Some(false) => spans.push(Span::styled(
            "MKT:CLOSED",
            Style::default().fg(Color::DarkGray),
        )),
        None => {}
    }

    spans.push(Span::raw("  "));
    spans.push(render_connection_badge(
        ConnectionTarget::Nats,
        &app.nats_status,
    ));
    if app.nats_status.state != ConnectionState::Connected && !app.nats_status.detail.is_empty() {
        spans.push(Span::raw(" "));
        spans.push(Span::styled(
            truncate_detail(&app.nats_status.detail, 36),
            Style::default().fg(Color::DarkGray),
        ));
    }

    f.render_widget(Paragraph::new(Line::from(spans)), area);
}

fn render_tab_bar(f: &mut Frame, app: &App, area: Rect) {
    let titles: Vec<Line> = Tab::ALL
        .iter()
        .map(|t| Line::from(format!(" {} ", t.label())))
        .collect();

    let active_idx = Tab::ALL
        .iter()
        .position(|t| t == &app.active_tab)
        .unwrap_or(0);

    let tabs = Tabs::new(titles)
        .select(active_idx)
        .block(Block::default().borders(Borders::BOTTOM))
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        );

    f.render_widget(tabs, area);
}

fn render_main(f: &mut Frame, app: &App, area: Rect) {
    if app.split_pane {
        let outer = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(1), Constraint::Min(0)])
            .split(area);
        let split_label = Paragraph::new(Line::from(vec![
            Span::styled(
                " Split pane ",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("Dashboard + Positions"),
        ]));
        f.render_widget(split_label, outer[0]);

        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(outer[1]);
        dashboard::render_dashboard(f, app, chunks[0]);
        positions::render_positions(f, app, chunks[1]);
    } else {
        match app.active_tab {
            Tab::Dashboard => dashboard::render_dashboard(f, app, area),
            Tab::Positions => positions::render_positions(f, app, area),
            Tab::Charts => charts::render_charts(f, app, area),
            Tab::Orders => orders::render_orders(f, app, area),
            Tab::Alerts => alerts::render_alerts(f, app, area),
            Tab::Yield => render_yield_curve_tab(f, app, area),
            Tab::Loans => loans::render_loans(f, app, area),
            Tab::Scenarios => scenarios::render_scenarios(f, app, area),
            Tab::Logs => logs::render_logs(f, app, area),
            Tab::Settings => settings::render_settings(f, app, area),
        }
    }
}

fn render_detail_overlay(f: &mut Frame, area: Rect, content: &DetailPopupContent) {
    let (title, lines) = match content {
        DetailPopupContent::Order(o) => {
            let side_style = if o.side.to_uppercase() == "BUY" {
                Style::default().fg(Color::Green)
            } else {
                Style::default().fg(Color::Red)
            };
            (
                " Order details ",
                vec![
                    Line::from(format!("ID:        {}", o.id)),
                    Line::from(format!("Symbol:    {}", o.symbol)),
                    Line::from(vec![
                        Span::raw("Side:      "),
                        Span::styled(&o.side, side_style),
                    ]),
                    Line::from(format!("Qty:       {}", o.quantity)),
                    Line::from(format!("Status:    {}", o.status)),
                    Line::from(format!(
                        "Submitted: {}",
                        o.submitted_at.format("%Y-%m-%d %H:%M:%S UTC")
                    )),
                ],
            )
        }
        DetailPopupContent::Position(p) => (
            " Position detail ",
            vec![
                Line::from(format!("ID:       {}", p.id)),
                Line::from(format!("Symbol:   {}", p.symbol)),
                Line::from(format!(
                    "Type:     {}",
                    positions::position_type_label(p.position_type.as_deref())
                )),
                Line::from(format!(
                    "Strategy: {}",
                    p.strategy.as_deref().unwrap_or("—")
                )),
                Line::from(format!(
                    "APR %:    {}",
                    p.apr_pct
                        .map(|a| format!("{:.2}%", a))
                        .unwrap_or_else(|| "—".into())
                )),
                Line::from(format!("Qty:      {}", p.quantity)),
                Line::from(format!("Cost:     {:.2}", p.cost_basis)),
                Line::from(format!("Mark:     {:.2}", p.mark)),
                Line::from(format!("P&L:      {:+.2}", p.unrealized_pnl)),
                Line::from(format!("Mkt val:  {:.2}", p.market_value)),
                Line::from(format!(
                    "Account:  {}",
                    p.account_id.as_deref().unwrap_or("—")
                )),
                Line::from(format!("Source:   {}", p.source.as_deref().unwrap_or("—"))),
            ],
        ),
        DetailPopupContent::Scenario(s) => (
            " Scenario detail ",
            vec![
                Line::from(format!("Symbol:   {}", s.symbol)),
                Line::from(format!("Expiration: {}", s.expiration)),
                Line::from(format!("Strike width: {}", s.strike_width)),
                Line::from(format!("Net debit:  {:.2}", s.net_debit)),
                Line::from(format!("Profit:     {:.2}", s.profit)),
                Line::from(format!("ROI %:      {:.2}", s.roi_pct)),
                Line::from(format!("APR %:      {:.2}", s.apr_pct)),
                Line::from(format!("Fill prob:  {:.2}", s.fill_probability)),
            ],
        ),
    };
    let mut all_lines = lines;
    all_lines.push(Line::from(""));
    all_lines.push(Line::from(Span::styled(
        " Esc to close ",
        Style::default().fg(Color::DarkGray),
    )));
    let inner = Paragraph::new(all_lines);
    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));
    let popup_area = centered_rect(50, 28, area);
    f.render_widget(ratatui::widgets::Clear, popup_area);
    f.render_widget(inner.block(block), popup_area);
}

fn render_help_overlay(f: &mut Frame, area: Rect) {
    let lines = vec![
        Line::from(""),
        Line::from(Span::styled(
            " Key bindings ",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled(" q ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw("quit  "),
            Span::styled(" ? ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw("this help  "),
            Span::styled(" Tab / ← → ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw("switch tab"),
        ]),
        Line::from(vec![
            Span::styled(" 0–9 ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(
                "jump to Dash / Pos / Charts / Orders / Alerts / Yield / Loans / Scen / Logs / Set",
            ),
        ]),
        Line::from(vec![
            Span::styled(" M ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw("read-only badge  "),
            Span::styled(" ` ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw("log panel  "),
            Span::styled(" p ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw("split pane  "),
            Span::styled(" Esc ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw("close current mode"),
        ]),
        Line::from(vec![
            Span::styled(
                " Dashboard / Positions / Orders / Alerts / Scenarios ",
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::raw("↑↓ PgUp/PgDn scroll  "),
            Span::styled(" Enter ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw("detail  "),
            Span::styled(" c ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw("Positions combo"),
        ]),
        Line::from(vec![
            Span::styled(" Orders ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw("/ filter mode  type symbol/status/side  Esc clear  "),
            Span::styled(" Loans ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw("n new loan  Tab/Shift-Tab field nav"),
        ]),
        Line::from(vec![
            Span::styled(" Charts ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw("/ search mode  Enter confirm  Esc cancel  ←→ expiry  ↑↓ width  "),
            Span::styled(" Yield ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw("←→ symbol"),
        ]),
        Line::from(vec![
            Span::styled(" Scenarios ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw("[ ] DTE  w strike width  Enter detail  "),
            Span::styled(" Logs ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw("+/- level  e/w/i/d jump  "),
        ]),
        Line::from(vec![
            Span::styled(" Settings ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw("↑↓ section/key  e/Enter edit  a add  Del remove  r reset  "),
            Span::styled(" Status ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw("provider, split-pane, loading, and latest result in bars"),
        ]),
        Line::from(vec![
            Span::raw(" Execution: "),
            Span::styled("S/T/K/F/O/X", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(" disabled in exploration mode"),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            " Press any key to close ",
            Style::default().fg(Color::DarkGray),
        )),
    ];
    let inner = Paragraph::new(lines).alignment(ratatui::layout::Alignment::Center);
    let block = Block::default()
        .title(" Help ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));
    let area = centered_rect(78, 22, area);
    f.render_widget(ratatui::widgets::Clear, area);
    f.render_widget(inner.block(block), area);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let w = (r.width * percent_x) / 100;
    let h = (r.height * percent_y) / 100;
    let x = r.x + (r.width.saturating_sub(w)) / 2;
    let y = r.y + (r.height.saturating_sub(h)) / 2;
    Rect::new(x, y, w, h)
}

pub(crate) fn truncate_detail(detail: &str, max_len: usize) -> String {
    if detail.len() <= max_len {
        detail.to_string()
    } else {
        format!("{}…", &detail[..max_len.saturating_sub(1)])
    }
}

fn render_connection_badge(target: ConnectionTarget, status: &ConnectionStatus) -> Span<'static> {
    let color = match status.state {
        ConnectionState::Connected => Color::Green,
        ConnectionState::Starting => Color::Blue,
        ConnectionState::Retrying => Color::Red,
    };

    Span::styled(
        format!("{}:{}", target.label(), status.state.label()),
        Style::default().fg(color).add_modifier(Modifier::BOLD),
    )
}

fn render_hint_bar(f: &mut Frame, app: &App, area: Rect) {
    let mut spans = vec![
        Span::raw(" "),
        Span::styled("q", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(":quit  "),
        Span::styled("?", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(":help  "),
        Span::styled(
            "←/→ Tab/BackTab",
            Style::default().add_modifier(Modifier::BOLD),
        ),
        Span::raw(":switch tab  "),
        Span::styled("0-9", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(":jump to tab  "),
        Span::styled("M", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(":read-only  "),
        Span::styled("S", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(":disabled  "),
        Span::styled("T", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(":disabled  "),
        Span::styled("K", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(":disabled  "),
        Span::styled("F", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(":disabled  "),
        Span::styled(
            "↑/↓ PgUp/PgDn",
            Style::default().add_modifier(Modifier::BOLD),
        ),
        Span::raw(":scroll  "),
        Span::styled("c", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(":combo  "),
        Span::styled("p", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(if app.split_pane {
            ":single pane"
        } else {
            ":split pane"
        }),
    ];

    if app.active_tab == Tab::Yield {
        spans.push(Span::raw("  "));
        spans.push(Span::styled(
            "←/→",
            Style::default().add_modifier(Modifier::BOLD),
        ));
        spans.push(Span::raw(":Yield symbol"));
    }
    if app.active_tab == Tab::Charts {
        spans.push(Span::raw("  "));
        spans.push(Span::styled(
            "/",
            Style::default().add_modifier(Modifier::BOLD),
        ));
        spans.push(Span::raw(
            if matches!(app.input_mode(), InputMode::ChartSearch) {
                ":search active"
            } else {
                ":chart search"
            },
        ));
    }
    if app.active_tab == Tab::Orders {
        spans.push(Span::raw("  "));
        spans.push(Span::styled(
            "/",
            Style::default().add_modifier(Modifier::BOLD),
        ));
        spans.push(Span::raw(
            if matches!(app.input_mode(), InputMode::OrdersFilter) {
                ":filter active"
            } else {
                ":filter"
            },
        ));
        spans.push(Span::raw("  "));
        spans.push(Span::styled(
            "Enter",
            Style::default().add_modifier(Modifier::BOLD),
        ));
        spans.push(Span::raw(":detail"));
    }
    if app.active_tab == Tab::Scenarios {
        spans.push(Span::raw("  "));
        spans.push(Span::styled(
            "Enter",
            Style::default().add_modifier(Modifier::BOLD),
        ));
        spans.push(Span::raw(":detail"));
        spans.push(Span::raw("  "));
        spans.push(Span::styled(
            "[ ]",
            Style::default().add_modifier(Modifier::BOLD),
        ));
        spans.push(Span::raw(":DTE  "));
        spans.push(Span::styled(
            "w",
            Style::default().add_modifier(Modifier::BOLD),
        ));
        spans.push(Span::raw(":width"));
        if scenarios::filtered_scenarios(app).is_empty() {
            spans.push(Span::raw("  | "));
            spans.push(Span::styled(
                "No results — ] widen DTE or w clear width filter",
                Style::default().fg(Color::Yellow),
            ));
        }
    }
    if app.active_tab == Tab::Orders || app.active_tab == Tab::Scenarios {
        spans.push(Span::raw("  | "));
        spans.push(Span::styled(
            "Exploration mode",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ));
    }
    if app.active_tab == Tab::Settings {
        spans.push(Span::raw("  "));
        spans.push(Span::styled(
            "a",
            Style::default().add_modifier(Modifier::BOLD),
        ));
        spans.push(Span::raw(":add  "));
        spans.push(Span::styled(
            "Del",
            Style::default().add_modifier(Modifier::BOLD),
        ));
        spans.push(Span::raw(":remove  "));
        spans.push(Span::styled(
            "r",
            Style::default().add_modifier(Modifier::BOLD),
        ));
        spans.push(Span::raw(":reset symbols"));
    }

    if let Some((label, color)) = settings_mode_indicator(app) {
        spans.push(Span::raw("  | "));
        spans.push(Span::styled(
            label,
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        ));
    }

    append_async_status_spans(&mut spans, app);

    if let Some(ref snap) = app.snapshot() {
        spans.push(Span::raw("  | "));
        spans.push(Span::raw("Strategy: "));
        let strategy = if snap.inner.strategy.trim().is_empty() {
            "UNKNOWN"
        } else {
            snap.inner.strategy.as_str()
        };
        let strategy_color = match strategy {
            "RUNNING" => Color::Green,
            "BLOCKED" => Color::Red,
            "IDLE" | "STOPPED" => Color::Yellow,
            _ => Color::Cyan,
        };
        spans.push(Span::styled(
            strategy,
            Style::default()
                .fg(strategy_color)
                .add_modifier(Modifier::BOLD),
        ));

        let n = snap.dto().alerts.len();
        spans.push(Span::raw("  | "));
        spans.push(Span::raw("Alerts: "));
        let alert_style = if n > 0 {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::DarkGray)
        };
        spans.push(Span::styled(n.to_string(), alert_style));
    }

    if let Some((msg, level, _)) = app.toast_queue.front() {
        use crate::app::ToastLevel;
        let color = match level {
            ToastLevel::Info => Color::Cyan,
            ToastLevel::Warning => Color::Yellow,
            ToastLevel::Error => Color::Red,
        };
        spans.push(Span::raw("  | "));
        spans.push(Span::styled(
            truncate_detail(msg, 40),
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        ));
    }
    if let Some(ref cmd) = app.last_command_status {
        spans.push(Span::raw("  | "));
        spans.push(Span::styled(
            format!("{}:{}", cmd.action, command_status_label(&cmd.status)),
            Style::default()
                .fg(command_status_color(&cmd.status))
                .add_modifier(Modifier::BOLD),
        ));
        if let Some(ref command_id) = cmd.command_id {
            spans.push(Span::raw(" "));
            spans.push(Span::styled(
                truncate_command_id(command_id),
                Style::default().fg(Color::DarkGray),
            ));
        }
        if let Some(ref msg) = cmd.message {
            spans.push(Span::raw(" "));
            spans.push(Span::styled(
                truncate_detail(msg, 28),
                Style::default().fg(Color::White),
            ));
        } else if let Some(ref err) = cmd.error {
            spans.push(Span::raw(" "));
            spans.push(Span::styled(
                truncate_detail(err, 28),
                Style::default().fg(Color::Red),
            ));
        }
    }

    let line = Line::from(spans);
    f.render_widget(Paragraph::new(line), area);
}

fn settings_mode_indicator(app: &App) -> Option<(String, Color)> {
    match app.input_mode() {
        InputMode::SettingsEditConfig => Some((
            format!(
                "SETTINGS:EDIT {}",
                truncate_detail(
                    app.settings_edit_config_key.as_deref().unwrap_or("CONFIG"),
                    16
                )
            ),
            Color::Yellow,
        )),
        InputMode::SettingsAddSymbol => Some(("SETTINGS:ADD SYMBOL".into(), Color::Yellow)),
        InputMode::ChartSearch => Some(("CHARTS:SEARCH".into(), Color::Cyan)),
        InputMode::OrdersFilter => Some(("ORDERS:FILTER".into(), Color::Yellow)),
        InputMode::LoanForm => Some(("LOANS:FORM".into(), Color::Yellow)),
        InputMode::LogPanel => Some(("LOGS:PANEL".into(), Color::Yellow)),
        _ => None,
    }
}

fn append_async_status_spans<'a>(spans: &mut Vec<Span<'a>>, app: &App) {
    if app.yield_fetch_pending {
        spans.push(Span::raw("  | "));
        spans.push(Span::styled(
            "~ Yield:loading",
            Style::default()
                .fg(Color::Blue)
                .add_modifier(Modifier::BOLD),
        ));
    } else if let Some(ref err) = app.yield_error {
        spans.push(Span::raw("  | "));
        spans.push(Span::styled(
            format!("Yield:{}", truncate_detail(err, 24)),
            Style::default().fg(Color::Red),
        ));
    }

    if app.loans_fetch_pending {
        spans.push(Span::raw("  | "));
        spans.push(Span::styled(
            "~ Loans:loading",
            Style::default()
                .fg(Color::Blue)
                .add_modifier(Modifier::BOLD),
        ));
    } else if let Some(Err(ref err)) = app.loans_list {
        spans.push(Span::raw("  | "));
        spans.push(Span::styled(
            format!("Loans:{}", truncate_detail(err, 24)),
            Style::default().fg(Color::Red),
        ));
    }
}

fn truncate_command_id(command_id: &str) -> String {
    if command_id.len() <= 18 {
        command_id.to_string()
    } else {
        format!("{}…", &command_id[..17])
    }
}

fn command_status_label(status: &CommandStatus) -> &'static str {
    match status {
        CommandStatus::Accepted => "accepted",
        CommandStatus::Completed => "completed",
        CommandStatus::Failed => "failed",
    }
}

fn command_status_color(status: &CommandStatus) -> Color {
    match status {
        CommandStatus::Accepted => Color::Blue,
        CommandStatus::Completed => Color::Green,
        CommandStatus::Failed => Color::Red,
    }
}

fn render_log_panel_overlay(f: &mut Frame, app: &App, area: Rect) {
    let panel_area = centered_rect(80, 70, area);
    f.render_widget(ratatui::widgets::Clear, panel_area);

    let title = " Debug Log [`/Esc]:close  [+/-]:level  [↑↓]:scroll ";
    let widget = tui_logger::TuiLoggerWidget::default()
        .block(
            Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Yellow)),
        )
        .style_error(Style::default().fg(Color::Red))
        .style_warn(Style::default().fg(Color::Yellow))
        .style_info(Style::default().fg(Color::Green))
        .style_debug(Style::default().fg(Color::Cyan))
        .style_trace(Style::default().fg(Color::DarkGray))
        .output_separator(' ')
        .output_timestamp(Some("%H:%M:%S".to_string()))
        .output_level(Some(tui_logger::TuiLoggerLevelOutput::Abbreviated))
        .output_target(false)
        .output_file(false)
        .output_line(false)
        .state(&app.log_state);
    f.render_widget(widget, panel_area);
}
