//! Ratatui rendering: frame layout, tab bar, and per-tab view delegation.

mod alerts;
mod dashboard;
mod loans;
mod logs;
mod orders;
mod positions;
pub use positions::positions_display_info;
mod scenarios;
pub use scenarios::filtered_scenarios;
mod settings;
mod yield_curve;
pub(crate) use yield_curve::render_yield_curve as render_yield_curve_tab;

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Tabs},
    Frame,
};

use crate::app::{App, DetailPopupContent, Tab};
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
    if let Some(ref content) = app.detail_popup {
        render_detail_overlay(f, f.area(), content);
    }
}

fn render_status_bar(f: &mut Frame, app: &App, area: Rect) {
    let (mode, strategy, source_label, source_color, is_stale) =
        if let Some(ref snap) = app.snapshot() {
            let stale = snap.is_stale(app.config.snapshot_ttl_secs as i64);
            let color = if stale { Color::Yellow } else { Color::Green };
            (
                snap.dto().mode.as_str().to_owned(),
                snap.dto().strategy.as_str().to_owned(),
                snap.source.label(),
                color,
                stale,
            )
        } else {
            (
                "---".into(),
                "---".into(),
                "NO DATA",
                Color::DarkGray,
                false,
            )
        };

    let mut spans = vec![
        Span::raw(format!(" {} | ", app.config.backend_id.to_uppercase())),
        Span::styled(mode, Style::default().fg(Color::Cyan)),
        Span::raw(" | "),
        Span::styled(strategy, Style::default().fg(Color::Yellow)),
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

    spans.push(Span::raw("  "));
    spans.push(Span::styled(
        format!("[{}]", source_label),
        Style::default()
            .fg(source_color)
            .add_modifier(Modifier::BOLD),
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
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area);
        dashboard::render_dashboard(f, app, chunks[0]);
        positions::render_positions(f, app, chunks[1]);
    } else {
        match app.active_tab {
            Tab::Dashboard => dashboard::render_dashboard(f, app, area),
            Tab::Positions => positions::render_positions(f, app, area),
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
            Span::styled(" 1–9 ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw("jump to Dash / Pos / Orders / Alerts / Yield / Loans / Scen / Logs / Set"),
        ]),
        Line::from(vec![
            Span::styled(" M ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw("mode Live/Mock/DRY-RUN  "),
            Span::styled(" S ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw("start  "),
            Span::styled(" T ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw("stop  "),
            Span::styled(" K ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw("cancel all  "),
            Span::styled(" F ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw("force snapshot"),
        ]),
        Line::from(vec![
            Span::styled(" / ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw("Orders: focus filter  "),
            Span::styled(" Esc ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw("clear filter  "),
            Span::styled(" x ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw("cancel all"),
        ]),
        Line::from(vec![
            Span::styled(
                " ↑ ↓ PgUp PgDn ",
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::raw("Pos/Orders/Alerts/Scen: scroll  "),
            Span::styled(" c ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw("Pos: combo/flat  "),
            Span::styled(" Enter ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw("detail  "),
            Span::styled(" + − ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw("Logs level  "),
            Span::styled(
                " e / w / i / d ",
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::raw("jump to ERROR/WARN/INFO/DEBUG"),
        ]),
        Line::from(vec![
            Span::styled(" ← → ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw("Yield: symbol  "),
            Span::styled(" [ ] ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw("Scen: DTE expand/contract  "),
            Span::styled(" w ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw("Scen: strike width  "),
            Span::styled(" o ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw("Scen: exec scenario"),
        ]),
        Line::from(vec![
            Span::styled(
                " Settings (8): ",
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::raw("↑↓ section/config key  "),
            Span::styled(" e ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw("edit config  "),
            Span::styled(" a ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw("add symbol  Del remove  "),
            Span::styled(" r ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw("reset watchlist"),
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
    let area = centered_rect(60, 14, area);
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
        Span::styled("1-9", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(":jump to tab  "),
        Span::styled("M", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(":mode  "),
        Span::styled("S", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(":start  "),
        Span::styled("T", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(":stop  "),
        Span::styled("K", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(":cancel all  "),
        Span::styled("F", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(":force snapshot  "),
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
    if app.active_tab == Tab::Scenarios {
        spans.push(Span::raw("  "));
        spans.push(Span::styled(
            "Enter",
            Style::default().add_modifier(Modifier::BOLD),
        ));
        spans.push(Span::raw(":detail"));
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

    if let Some(ref snap) = app.snapshot() {
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

    if let Some(ref res) = app.last_strategy_result {
        let (msg, color) = match res {
            Ok(m) => (m.as_str(), Color::Green),
            Err(e) => (e.as_str(), Color::Red),
        };
        spans.push(Span::raw("  | "));
        spans.push(Span::styled(
            msg,
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        ));
    }

    let line = Line::from(spans);
    f.render_widget(Paragraph::new(line), area);
}
