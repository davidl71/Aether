//! Ratatui rendering: frame layout, tab bar, and per-tab view delegation.

mod alerts;
mod candlestick;
pub mod charts;
mod dashboard;
mod discount_bank;
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
#[cfg(test)]
pub(crate) use yield_curve::render_yield_curve as render_yield_curve_tab;

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Tabs},
    Frame,
};

use api::CommandStatus;

use crate::app::{App, DetailPopupContent, InputMode, Tab, VisibleWorkspace, WorkspaceSpec};
use crate::events::{ConnectionState, ConnectionStatus, ConnectionTarget};

pub fn render(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // tab bar
            Constraint::Min(0),    // main content
            Constraint::Length(1), // hint bar
            Constraint::Length(1), // status bar (moved to bottom)
        ])
        .split(f.area());

    render_tab_bar(f, app, chunks[0]);
    render_main(f, app, chunks[1]);
    render_hint_bar(f, app, chunks[2]);
    render_status_bar(f, app, chunks[3]);

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

    if matches!(app.visible_workspace(), VisibleWorkspace::SplitPane) {
        spans.push(Span::raw(" | "));
        spans.push(Span::styled(
            "PANE:DASH+POS",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ));
    }

    // Market data source detail (source@priority, age) lives in the Settings → Data sources tab.
    // We show only a compact [SOURCE] pill in the title bar to indicate which provider is active.

    spans.push(Span::raw("  "));
    let pill_color = match market_data_source.to_lowercase().as_str() {
        "yahoo" => Color::Magenta,
        "mock" => Color::Cyan,
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
    app.set_last_main_area_size(area.width, area.height);
    if let Some(spec) = app.visible_workspace_spec() {
        match spec.kind {
            VisibleWorkspace::SplitPane => render_split_workspace(f, app, area, spec),
            VisibleWorkspace::Market => render_market_workspace(f, app, area, spec),
            VisibleWorkspace::Operations => render_operations_workspace(f, app, area, spec),
            VisibleWorkspace::Credit => render_credit_workspace(f, app, area, spec),
            VisibleWorkspace::None => unreachable!("visible_workspace_spec never returns None"),
        }
    } else {
        match app.active_tab {
            Tab::Dashboard => dashboard::render_dashboard_panel(f, app, area),
            Tab::Positions => positions::render_positions_panel(f, app, area),
            Tab::Charts => charts::render_charts(f, app, area),
            Tab::Orders => orders::render_orders_panel(f, app, area),
            Tab::Alerts => alerts::render_alerts(f, app, area),
            Tab::Yield => yield_curve::render_yield_curve_panel(f, app, area),
            Tab::Loans => loans::render_loans(f, app, area),
            Tab::DiscountBank => discount_bank::render_discount_bank(f, app, area),
            Tab::Scenarios => scenarios::render_scenarios(f, app, area),
            Tab::Logs => logs::render_logs(f, app, area),
            Tab::Settings => settings::render_settings(f, app, area),
        }
    }
}

fn render_split_workspace(f: &mut Frame, app: &App, area: Rect, spec: WorkspaceSpec) {
    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(0)])
        .split(area);
    let split_label = Paragraph::new(Line::from(vec![
        Span::styled(
            format!(" {} ", spec.title),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(format!("{}  |  Tab/Shift-Tab: focus panes", spec.summary)),
    ]));
    f.render_widget(split_label, outer[0]);

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(outer[1]);
    dashboard::render_dashboard_panel(f, app, chunks[0]);
    positions::render_positions_panel(f, app, chunks[1]);
}

fn workspace_banner(spec: WorkspaceSpec, focus_label: &str) -> Paragraph<'static> {
    Paragraph::new(Line::from(vec![
        Span::styled(
            format!(" {} ", spec.title),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(format!("{}  |  Focus: ", spec.summary)),
        Span::styled(
            focus_label.to_string(),
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("  |  Tab/Shift-Tab cycles panes"),
    ]))
}

fn render_market_workspace(f: &mut Frame, app: &App, area: Rect, spec: WorkspaceSpec) {
    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(0)])
        .split(area);

    f.render_widget(workspace_banner(spec, app.active_tab.title()), outer[0]);

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(outer[1]);
    let top = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(52), Constraint::Percentage(48)])
        .split(rows[0]);
    let bottom = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(48), Constraint::Percentage(52)])
        .split(rows[1]);

    dashboard::render_dashboard_panel(f, app, top[0]);
    positions::render_positions_panel(f, app, top[1]);
    orders::render_orders_panel(f, app, bottom[0]);
    yield_curve::render_yield_curve_panel(f, app, bottom[1]);
}

fn render_operations_workspace(f: &mut Frame, app: &App, area: Rect, spec: WorkspaceSpec) {
    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(0)])
        .split(area);

    f.render_widget(workspace_banner(spec, app.active_tab.title()), outer[0]);

    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(38), Constraint::Percentage(62)])
        .split(outer[1]);
    let left = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(42), Constraint::Percentage(58)])
        .split(columns[0]);

    let alerts_view = alerts::build_alerts_view(app, left[0]);
    alerts::render_alerts_panel(f, left[0], alerts_view);

    let logs_widget = logs::build_logs_widget(app, logs::logs_title(app));
    f.render_widget(logs_widget, left[1]);

    let layout = settings::settings_layout(columns[1]);
    settings::render_settings_health_section(f, app, layout.health);
    settings::render_settings_config_section(f, app, layout.config);
    settings::render_settings_symbols_section(f, app, layout.symbols);
    settings::render_settings_sources_section(f, app, layout.sources);
    settings::render_settings_hint_section(f, app, layout.hint);
}

fn render_credit_workspace(f: &mut Frame, app: &App, area: Rect, spec: WorkspaceSpec) {
    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(0)])
        .split(area);

    let (loans_width, bank_width) = if app.active_tab == Tab::Loans {
        (52, 48)
    } else {
        (48, 52)
    };
    f.render_widget(workspace_banner(spec, app.active_tab.title()), outer[0]);

    let panes = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(loans_width),
            Constraint::Percentage(bank_width),
        ])
        .split(outer[1]);
    loans::render_loans(f, app, panes[0]);
    discount_bank::render_discount_bank(f, app, panes[1]);
}

fn render_detail_overlay(f: &mut Frame, area: Rect, content: &DetailPopupContent) {
    let (title, lines): (String, Vec<Line>) = match content {
        DetailPopupContent::Order(o) => {
            let side_style = if o.side.to_uppercase() == "BUY" {
                Style::default().fg(Color::Green)
            } else {
                Style::default().fg(Color::Red)
            };
            (
                " Order details ".to_string(),
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
        DetailPopupContent::Position(p, greeks) => {
            let mut lines = vec![
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
            ];
            if let Some(g) = greeks {
                lines.push(Line::from(""));
                lines.push(Line::from(Span::styled(
                    "— Greeks —",
                    Style::default().fg(Color::Cyan),
                )));
                lines.push(Line::from(format!("IV:    {:.1}%", g.iv * 100.0)));
                lines.push(Line::from(format!("Δ delta:{:+.4}", g.delta)));
                lines.push(Line::from(format!("Γ gamma:{:.6}", g.gamma)));
                lines.push(Line::from(format!("Θ theta:{:+.4}", g.theta)));
                lines.push(Line::from(format!("ν vega: {:.4}", g.vega)));
                lines.push(Line::from(format!("ρ rho:  {:+.4}", g.rho)));
            } else if p
                .position_type
                .as_deref()
                .map(|t| t == "OPT" || t == "OPTION")
                .unwrap_or(false)
            {
                lines.push(Line::from(""));
                lines.push(Line::from(Span::styled(
                    "Greeks: loading…",
                    Style::default().fg(Color::DarkGray),
                )));
            }
            (" Position detail ".to_string(), lines)
        }
        DetailPopupContent::Scenario(s) => (
            " Scenario detail ".to_string(),
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
        DetailPopupContent::YieldPoint(p) => (
            " Box spread legs ".to_string(),
            vec![
                Line::from(format!("Symbol:      {}", p.symbol)),
                Line::from(format!("Expiry:      {}", p.expiry)),
                Line::from(format!("DTE:         {}", p.days_to_expiry)),
                Line::from(format!("Width:       {:.0}", p.strike_width)),
                Line::from(vec![
                    Span::raw("Strikes:     "),
                    Span::styled(
                        format!(
                            "{} / {}",
                            p.strike_low
                                .map(|s| format!("{:.0}", s))
                                .unwrap_or("—".into()),
                            p.strike_high
                                .map(|s| format!("{:.0}", s))
                                .unwrap_or("—".into()),
                        ),
                        Style::default().fg(Color::Cyan),
                    ),
                ]),
                Line::from(vec![
                    Span::raw("Buy rate:    "),
                    Span::styled(
                        format!("{:.3}%", p.buy_implied_rate * 100.0),
                        Style::default().fg(Color::Green),
                    ),
                ]),
                Line::from(vec![
                    Span::raw("Sell rate:   "),
                    Span::styled(
                        format!("{:.3}%", p.sell_implied_rate * 100.0),
                        Style::default().fg(Color::Red),
                    ),
                ]),
                Line::from(vec![
                    Span::raw("Mid rate:    "),
                    Span::styled(
                        format!("{:.3}%", p.mid_rate * 100.0),
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    ),
                ]),
                Line::from(format!("Net debit:   {:.2}", p.net_debit)),
                Line::from(format!("Net credit:  {:.2}", p.net_credit)),
                Line::from(format!("Liquidity:   {:.2}", p.liquidity_score)),
                Line::from(format!(
                    "Source:      {}",
                    p.data_source.as_deref().unwrap_or("—")
                )),
                Line::from(format!(
                    "Spread ID:   {}",
                    p.spread_id.as_deref().unwrap_or("—")
                )),
                Line::from(format!("Timestamp:   {}", p.timestamp)),
            ],
        ),
        DetailPopupContent::FmpSymbol(data) => {
            let mut lines = vec![
                Line::from(format!("Symbol:    {}", data.symbol)),
                Line::from(""),
                Line::from(Span::styled("— Quote —", Style::default().fg(Color::Cyan))),
            ];
            if let Some(p) = data.price {
                lines.push(Line::from(format!("Price:     {:.2}", p)));
            }
            if let Some(v) = data.prev_close {
                lines.push(Line::from(format!("Prev close:{:.2}", v)));
            }
            if let Some(h) = data.day_high {
                lines.push(Line::from(format!("Day high:  {:.2}", h)));
            }
            if let Some(l) = data.day_low {
                lines.push(Line::from(format!("Day low:   {:.2}", l)));
            }
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                "— Fundamentals (latest) —",
                Style::default().fg(Color::Cyan),
            )));
            if let Some(r) = data.revenue {
                lines.push(Line::from(format!("Revenue:   ${:.0}", r)));
            }
            if let Some(n) = data.net_income {
                lines.push(Line::from(format!("Net income:${:.0}", n)));
            }
            if let Some(e) = data.eps {
                lines.push(Line::from(format!("EPS:       {:.2}", e)));
            }
            if data.revenue.is_none() && data.eps.is_none() {
                lines.push(Line::from(Span::styled(
                    "No fundamentals (FMP key required)",
                    Style::default().fg(Color::DarkGray),
                )));
            }
            (format!(" FMP: {} ", data.symbol), lines)
        }
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
    if app.split_pane && matches!(app.active_tab, Tab::Dashboard | Tab::Positions) {
        spans.push(Span::raw("  "));
        spans.push(Span::styled(
            "←/→",
            Style::default().add_modifier(Modifier::BOLD),
        ));
        spans.push(Span::raw(":focus pane"));
    }
    if app.active_tab == Tab::Settings {
        spans.push(Span::raw("  "));
        spans.push(Span::styled(
            "←/→",
            Style::default().add_modifier(Modifier::BOLD),
        ));
        spans.push(Span::raw(":section"));
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
    if let Some(spec) = app.visible_workspace_spec() {
        spans.push(Span::raw("  | "));
        spans.push(Span::styled(
            spec.hint_label,
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ));
        spans.push(Span::raw(": Tab/Shift-Tab cycles panes"));
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
    if app.yield_refresh_pending {
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
