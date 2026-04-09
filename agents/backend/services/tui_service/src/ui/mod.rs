//! Ratatui rendering: frame layout, tab bar, and per-tab view delegation.
//!
//! Rendering should treat [`crate::app::App`] as the source of truth and avoid
//! mutating state here except via explicit callbacks (e.g. the command palette).

pub(crate) mod alerts;
mod candlestick;
pub mod charts;
mod chrome_layout;
mod dashboard;
mod discount_bank;
pub mod feedback;
mod loans;
pub mod logs;
mod orders;
mod positions;
mod text_trunc;
mod numeric_format;
pub(crate) mod tree_panel;
pub use positions::positions_display_info;
pub(crate) use positions::sort_positions_for_operator;
mod scenarios;
pub use scenarios::filtered_scenarios;
pub(crate) mod settings;
mod yield_curve;
pub use candlestick::Candle;
pub use feedback::{ToastLevel, ToastManager};

use feedback::render_toast_area;
#[cfg(test)]
pub(crate) use yield_curve::render_yield_curve as render_yield_curve_tab;

use chrome_layout::split_vertical_chrome;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Tabs},
    Frame,
};

use api::CommandStatus;
use chrono::{TimeDelta, Utc};

use crate::app::{App, DetailPopupContent, InputMode, Tab};
use crate::events::{ConnectionState, ConnectionStatus, ConnectionTarget};
use crate::mode::AppMode;
use crate::pane::{pane_spec, PaneHintMode};
use crate::workspace::{VisibleWorkspace, WorkspaceSpec};

/// True when a layered widget may sit above the main layout; partial base redraws must repaint these.
fn layered_chrome_active(app: &App) -> bool {
    app.show_help
        || app.show_log_panel
        || app.show_tree_panel
        || app.detail_popup.is_some()
        || app.command_palette.visible
        || app.toast_manager.has_active()
}

pub fn render(f: &mut Frame, app: &mut App) {
    let flags = app.dirty_flags;
    let paint_all = !flags.is_dirty();
    let paint_layered = paint_all || flags.overlay || layered_chrome_active(app);

    let chunks = split_vertical_chrome(f.area());

    if paint_all || flags.tabs {
        render_tab_bar(f, app, chunks[0]);
    }
    if paint_all || flags.content {
        render_main(f, app, chunks[1]);
    }
    if paint_all || flags.hint_bar {
        render_hint_bar(f, app, chunks[2]);
    }
    if paint_all || flags.status_bar {
        render_status_bar(f, app, chunks[3]);
    }

    if paint_layered {
        render_toast_area(f, &app.toast_manager, f.area());
        if app.show_help {
            render_help_overlay(f, app, f.area());
        }
        if app.show_log_panel {
            render_log_panel_overlay(f, app, f.area());
        }
        if app.show_tree_panel {
            tree_panel::render_tree_panel_overlay(f, app, f.area());
        }
        if let Some(ref content) = app.detail_popup {
            render_detail_overlay(f, f.area(), content);
        }
        crate::discoverability::render_command_palette(
            f,
            &app.command_palette,
            app.config.theme,
            f.area(),
        );
    }

    app.dirty_flags.clear_all();
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
        Span::styled(
            format!("{} {}", app.app_mode.icon(), app.app_mode.label()),
            app.app_mode.style().add_modifier(Modifier::BOLD),
        ),
        Span::raw(" | "),
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
    spans.push(Span::raw(" "));
    spans.push(render_transport_badge(app));
    if app.nats_status.state != ConnectionState::Connected && !app.nats_status.detail.is_empty() {
        spans.push(Span::raw(" "));
        spans.push(Span::styled(
            truncate_detail(&app.nats_status.detail, 36),
            Style::default().fg(Color::DarkGray),
        ));
    }

    f.render_widget(Paragraph::new(Line::from(spans)), area);
}

fn render_transport_badge(app: &App) -> Span<'static> {
    // Keep compact: full details are in Settings → Health.
    const DEFAULT_STALE_AFTER_SECS: i64 = 45;
    let now = Utc::now();
    let transport = app
        .nats_transport
        .effective_at(now, TimeDelta::seconds(DEFAULT_STALE_AFTER_SECS));

    let (label, color) = match transport.status.as_str() {
        "ok" => ("T:OK", Color::Green),
        "degraded" => ("T:WARN", Color::Yellow),
        "error" | "disabled" => ("T:DOWN", Color::Red),
        _ => ("T:—", Color::DarkGray),
    };
    let stale = transport
        .extra
        .get("stale")
        .map(String::as_str)
        .is_some_and(|v| v == "true");
    let suffix = if stale { "*" } else { "" };

    Span::styled(
        format!("{label}{suffix}"),
        Style::default().fg(color).add_modifier(Modifier::BOLD),
    )
}

fn render_tab_bar(f: &mut Frame, app: &App, area: Rect) {
    let titles: Vec<Line> = Tab::ALL
        .iter()
        .map(|t| Line::from(format!(" {} ", t.label())))
        .collect();

    // Record per-tab clickable regions for mouse hit-testing.
    //
    // We intentionally avoid hard-coded x offsets: the tab labels (and terminal width)
    // should be the only inputs to hit-testing.
    let mut regions = Vec::with_capacity(Tab::ALL.len());
    let mut cursor_x = area.x;
    let click_height = area.height.min(2).max(1);
    for (tab, title) in Tab::ALL.iter().zip(titles.iter()) {
        let w = title.width() as u16;
        if w == 0 {
            continue;
        }
        // Clamp to the available width to avoid overflow on narrow terminals.
        let remaining = area.x.saturating_add(area.width).saturating_sub(cursor_x);
        if remaining == 0 {
            break;
        }
        let width = w.min(remaining);
        regions.push((*tab, Rect::new(cursor_x, area.y, width, click_height)));
        cursor_x = cursor_x.saturating_add(w);
    }
    app.set_tab_bar_regions(regions);

    let active_idx = Tab::ALL
        .iter()
        .position(|t| {
            *t == match app.active_tab {
                Tab::Logs => Tab::Alerts,
                other => other,
            }
        })
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
        render_tab_panel(f, app, area, app.active_tab);
    }
}

fn render_tab_panel(f: &mut Frame, app: &App, area: Rect, tab: Tab) {
    match tab {
        Tab::Dashboard => dashboard::render_dashboard_panel(f, app, area),
        Tab::Positions => positions::render_positions_panel(f, app, area),
        Tab::Charts => charts::render_charts(f, app, area),
        Tab::Orders => orders::render_orders_panel(f, app, area),
        Tab::Alerts | Tab::Logs => render_operations_tab(f, app, area),
        Tab::Yield => yield_curve::render_yield_curve_panel(f, app, area),
        Tab::Loans => loans::render_loans(f, app, area),
        Tab::DiscountBank => discount_bank::render_discount_bank(f, app, area),
        Tab::Ledger => {
            let widget = Paragraph::new(vec![
                Line::from(Span::styled(
                    "Ledger journal",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )),
                Line::from(""),
                Line::from(Span::styled(
                    "Not implemented yet.",
                    Style::default().fg(Color::DarkGray),
                )),
            ])
            .block(Block::default().title(" Ledger ").borders(Borders::ALL));
            f.render_widget(widget, area);
        }
        Tab::Scenarios => scenarios::render_scenarios(f, app, area),
        Tab::Settings => settings::render_settings(f, app, area),
    }
}

fn render_operations_tab(f: &mut Frame, app: &App, area: Rect) {
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(42), Constraint::Percentage(58)])
        .split(area);

    let alerts_view = alerts::build_alerts_view(app, rows[0]);
    alerts::render_alerts_panel(f, rows[0], alerts_view);

    let logs_widget = logs::build_logs_widget(app, logs::logs_title(app));
    f.render_widget(logs_widget, rows[1]);
}

/// Single-line workspace banner row plus remaining content (shared by split + tile workspaces).
fn workspace_outer_rows(area: Rect) -> (Rect, Rect) {
    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(0)])
        .split(area);
    (outer[0], outer[1])
}

fn render_split_workspace(f: &mut Frame, app: &App, area: Rect, spec: WorkspaceSpec) {
    let (banner_row, body) = workspace_outer_rows(area);
    let split_label = Paragraph::new(Line::from(vec![
        Span::styled(
            format!(" {} ", spec.title),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(format!("{}  |  Tab/Shift-Tab: focus panes", spec.summary)),
    ]));
    f.render_widget(split_label, banner_row);

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(body);
    dashboard::render_dashboard_panel(f, app, chunks[0]);
    positions::render_positions_panel(f, app, chunks[1]);
}

fn workspace_banner(spec: WorkspaceSpec, focus_label: &str, max_width: u16) -> Paragraph<'static> {
    let extra_hint: &'static str = match spec.kind {
        VisibleWorkspace::Market if max_width >= 140 => "  |  Wheel: pane under cursor",
        VisibleWorkspace::Market if max_width >= 100 => "  |  Wheel→pane",
        VisibleWorkspace::Market => " |^v",
        _ => "",
    };

    let focus_cap = (max_width as usize)
        .saturating_sub(38)
        .saturating_sub(extra_hint.chars().count())
        .clamp(6, 80);
    let focus_display = text_trunc::truncate_chars(focus_label, focus_cap);

    let summary_cap = (max_width as usize)
        .saturating_sub(26)
        .saturating_sub(focus_display.chars().count())
        .saturating_sub(extra_hint.chars().count())
        .max(10);
    let summary_display = text_trunc::truncate_chars(spec.summary, summary_cap);

    let line = Line::from(vec![
        Span::styled(
            format!(" {} ", spec.title),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(format!("{}  |  Focus: ", summary_display)),
        Span::styled(
            focus_display,
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(format!("  |  Tab/Shift-Tab cycles panes{}", extra_hint)),
    ]);

    if line.width() <= max_width as usize || max_width < 24 {
        return Paragraph::new(line);
    }

    let tight_focus = text_trunc::truncate_chars(focus_label, 10);
    let tight_summary = text_trunc::truncate_chars(spec.summary, 18);
    Paragraph::new(Line::from(vec![
        Span::styled(
            format!(" {} ", spec.title),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(format!("{} | ", tight_summary)),
        Span::styled(
            tight_focus,
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" | Tab panes"),
    ]))
}

fn render_market_workspace(f: &mut Frame, app: &App, area: Rect, spec: WorkspaceSpec) {
    let (banner_row, body) = workspace_outer_rows(area);

    f.render_widget(
        workspace_banner(spec, &app.focus_label(), banner_row.width),
        banner_row,
    );

    // Keep splits in sync with `mouse::market_workspace_tab_at_point` (asymmetric grid).
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(body);
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

/// Horizontal split for Operations workspace body (alerts/logs | Settings).
/// Biases width to the Settings column when the frame is narrow so embedded
/// Settings can keep its internal wide (2-column) layout; see `settings_layout_embedded`.
fn operations_workspace_column_constraints(body_width: u16) -> [Constraint; 2] {
    let (left_pct, right_pct) = if body_width < 118 {
        (22, 78)
    } else if body_width < 150 {
        (32, 68)
    } else {
        (40, 60)
    };
    [
        Constraint::Percentage(left_pct),
        Constraint::Percentage(right_pct),
    ]
}

fn render_operations_workspace(f: &mut Frame, app: &App, area: Rect, spec: WorkspaceSpec) {
    let (banner_row, body) = workspace_outer_rows(area);

    f.render_widget(
        workspace_banner(spec, &app.focus_label(), banner_row.width),
        banner_row,
    );

    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(operations_workspace_column_constraints(body.width))
        .split(body);
    let left = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(42), Constraint::Percentage(58)])
        .split(columns[0]);

    let alerts_view = alerts::build_alerts_view(app, left[0]);
    alerts::render_alerts_panel(f, left[0], alerts_view);

    let logs_widget = logs::build_logs_widget(app, logs::logs_title(app));
    f.render_widget(logs_widget, left[1]);

    let layout = settings::settings_layout_embedded(columns[1]);
    settings::render_settings_sections(f, app, layout);
}

fn render_credit_workspace(f: &mut Frame, app: &App, area: Rect, spec: WorkspaceSpec) {
    let (banner_row, body) = workspace_outer_rows(area);

    let (loans_width, bank_width) = if app.active_tab == Tab::Loans {
        (52, 48)
    } else {
        (48, 52)
    };
    f.render_widget(
        workspace_banner(spec, &app.focus_label(), banner_row.width),
        banner_row,
    );

    let panes = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(loans_width),
            Constraint::Percentage(bank_width),
        ])
        .split(body);
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

/// Centered modal keybinding reference (`?` / ⌘/ to open; any key closes). Keep in sync with
/// `docs/TUI_ARCHITECTURE.md` § Help overlay; update hint bar / `discoverability::context_hints_for` when bindings change.
fn render_help_overlay(f: &mut Frame, app: &App, area: Rect) {
    let p = app.ui_palette();
    let lines = vec![
        Line::from(""),
        Line::from(Span::styled(
            " Key bindings ",
            Style::default()
                .fg(p.help_title)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled(" q ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw("quit  "),
            Span::styled(" ? ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw("help  "),
            Span::styled(" : ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw("command palette  "),
            Span::styled(
                " Tab / Shift-Tab ",
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::raw("next / prev tab or workspace pane"),
        ]),
        Line::from(vec![
            Span::styled(
                " Status bar ",
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::raw("NAV / EDIT / VIEW follow focus (browse vs text vs overlays). "),
            Span::styled(
                " Toasts ",
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::raw("stack bottom-right (errors and notices)."),
        ]),
        Line::from(vec![
            Span::styled(" macOS ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(
                "⌘, Settings  ⌘/ help  ⌘⇧P palette  ⌘⇧T theme  ⌘0 Settings  ⌘1–⌘9 = digit jumps  ⌘p split  ⌘r refresh  ⌘w close",
            ),
        ]),
        Line::from(vec![
            Span::styled(" 1–9,0 ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(
                "1 Dash  2 Pos  3 Charts  4 Orders  5 Alerts  6 Yield  7 Loans  8 Disc  9 Ledger  0 Settings",
            ),
        ]),
        Line::from(vec![
            Span::styled(" M ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw("mode / discoverability tips  "),
            Span::styled(" ` ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw("log panel  "),
            Span::styled(" g ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw("tree panel  "),
            Span::styled(" p ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw("split pane  "),
            Span::styled(" f ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw("FMP (Dash/Pos) or snapshot refresh (other tabs)  "),
            Span::styled(" Ctrl+T ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw("theme (TUI_THEME)  "),
            Span::styled(" Esc ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw("close overlay / filter / etc."),
        ]),
        Line::from(vec![
            Span::styled(
                " Workspaces ",
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::raw(
                "Market · Operations · Credit: when the yellow workspace tag shows in the hint bar, ",
            ),
            Span::styled("Tab", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(" / "),
            Span::styled("Shift-Tab", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(" cycle inner panes (not global tab order)."),
        ]),
        Line::from(vec![
            Span::styled(
                " Dashboard / Positions / Alerts ",
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::raw("↑↓ PgUp/PgDn scroll  "),
            Span::styled(" Enter ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw("Dash→Charts / Pos detail  "),
            Span::styled(" c ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw("Positions combo / space  "),
            Span::styled(" r ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw("Positions sort cycle"),
        ]),
        Line::from(vec![
            Span::styled(" Orders ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw("/ filter  / again exits empty filter  Esc clear  "),
            Span::styled(" x ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw("cancel  "),
            Span::styled(" Disc ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw("r refresh  ↑↓ PgUp/Dn"),
        ]),
        Line::from(vec![
            Span::styled(" Loans ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(
                "n new  b/i bulk JSON path  ↑↓ PgUp/Dn scroll  ",
            ),
            Span::styled("form ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw("Tab/Shift-Tab fields · Enter submit · Esc  "),
            Span::styled("path ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw("Enter import · Esc cancel"),
        ]),
        Line::from(vec![
            Span::styled(" Charts ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw("/ search · Enter pick · Esc close  ←→ expiry pills  ↑↓ row (width)  "),
            Span::styled(" Yield ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw("←→ symbol  ↑↓ curve  Enter point  r refresh"),
        ]),
        Line::from(vec![
            Span::styled(" Scenarios ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw("[ ] DTE  w width  Enter detail  "),
            Span::styled(" o ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw("execute (disabled)  "),
            Span::styled(" Logs ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw("+/- level  e/w/i/d  h hide"),
        ]),
        Line::from(vec![
            Span::styled(" Settings ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(
                "0 tab jump (also ⌘0)  ←/→ columns  ↑↓ row  e/Enter edit  a add  d/Del remove  r reset  ",
            ),
            Span::raw("Alpaca & Sources rows: e edit cred  d clear"),
        ]),
        Line::from(vec![
            Span::raw(" Exploration: "),
            Span::styled("S T K O", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(" strategy / exec show disabled status; "),
            Span::styled("X", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(" is Orders cancel only"),
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
        .border_style(Style::default().fg(p.accent));
    let area = centered_rect(86, 34, area);
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
    text_trunc::truncate_chars(detail, max_len)
}

fn render_connection_badge(target: ConnectionTarget, status: &ConnectionStatus) -> Span<'static> {
    let color = connection_status_color(status);

    Span::styled(
        format!("{}:{}", target.label(), status.state.label()),
        Style::default().fg(color).add_modifier(Modifier::BOLD),
    )
}

fn connection_status_color(status: &ConnectionStatus) -> Color {
    match status.state {
        ConnectionState::Connected => Color::Green,
        ConnectionState::Starting => Color::Blue,
        ConnectionState::Retrying => Color::Red,
    }
}

fn render_hint_bar(f: &mut Frame, app: &App, area: Rect) {
    let mut spans = vec![
        Span::raw(" "),
        Span::styled("q", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(":quit  "),
        Span::styled("?", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(":help  "),
        Span::styled("Tab/BackTab", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(":next/prev tab  "),
        Span::styled("0-9", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(":jump to tab  "),
        Span::styled("M", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(":read-only  "),
    ];

    match pane_spec(app.active_tab).hint_mode {
        PaneHintMode::Yield => {
            spans.push(Span::raw("  "));
            spans.push(Span::styled(
                "←/→",
                Style::default().add_modifier(Modifier::BOLD),
            ));
            spans.push(Span::raw(":Yield symbol"));
        }
        PaneHintMode::Settings => {
            spans.push(Span::raw("  "));
            spans.push(Span::styled(
                "←/→",
                Style::default().add_modifier(Modifier::BOLD),
            ));
            spans.push(Span::raw(":section"));
            if let Some(section) = app.secondary_focus().label() {
                spans.push(Span::raw("  "));
                spans.push(Span::styled(
                    "focus",
                    Style::default().add_modifier(Modifier::BOLD),
                ));
                spans.push(Span::raw(format!(":{section}")));
            }
        }
        PaneHintMode::Charts => {
            spans.push(Span::raw("  "));
            spans.push(Span::styled(
                "/",
                Style::default().add_modifier(Modifier::BOLD),
            ));
            spans.push(Span::raw(":search"));
            if matches!(app.input_mode(), InputMode::ChartSearch) {
                spans.push(Span::raw("  "));
                spans.push(Span::styled(
                    "↑↓ Home End",
                    Style::default().add_modifier(Modifier::BOLD),
                ));
                spans.push(Span::raw(":pick  "));
                spans.push(Span::styled(
                    "Enter Esc",
                    Style::default().add_modifier(Modifier::BOLD),
                ));
                spans.push(Span::raw(":apply/close"));
            } else {
                spans.push(Span::raw("  "));
                spans.push(Span::styled(
                    "←→·hl",
                    Style::default().add_modifier(Modifier::BOLD),
                ));
                spans.push(Span::raw(":expiry  "));
                spans.push(Span::styled(
                    "↑↓·jk",
                    Style::default().add_modifier(Modifier::BOLD),
                ));
                spans.push(Span::raw(":row  "));
                spans.push(Span::styled(
                    "Enter",
                    Style::default().add_modifier(Modifier::BOLD),
                ));
                spans.push(Span::raw(":width"));
            }
        }
        PaneHintMode::Orders => {
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
        PaneHintMode::Scenarios => {
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
            spans.push(Span::raw(":width  "));
            spans.push(Span::styled(
                "o",
                Style::default().add_modifier(Modifier::BOLD),
            ));
            spans.push(Span::raw(":execute (off)"));
            if scenarios::filtered_scenarios(app).is_empty() {
                spans.push(Span::raw("  | "));
                spans.push(Span::styled(
                    "No results — ] widen DTE or w clear width filter",
                    Style::default().fg(Color::Yellow),
                ));
            }
        }
        PaneHintMode::None => {}
    }
    if app.split_pane && matches!(app.active_tab, Tab::Dashboard | Tab::Positions) {
        spans.push(Span::raw("  "));
        spans.push(Span::styled(
            "Tab/BackTab",
            Style::default().add_modifier(Modifier::BOLD),
        ));
        spans.push(Span::raw(":focus pane"));
    }
    spans.push(Span::raw("  "));
    spans.push(Span::styled(
        "↑/↓ PgUp/PgDn",
        Style::default().add_modifier(Modifier::BOLD),
    ));
    spans.push(Span::raw(":scroll  "));
    spans.push(Span::styled(
        "c",
        Style::default().add_modifier(Modifier::BOLD),
    ));
    spans.push(Span::raw(":combo  "));
    spans.push(Span::styled(
        "p",
        Style::default().add_modifier(Modifier::BOLD),
    ));
    spans.push(Span::raw(if app.split_pane {
        ":single pane"
    } else {
        ":split pane"
    }));
    spans.push(Span::raw("  "));
    spans.push(Span::styled(
        "S",
        Style::default().add_modifier(Modifier::BOLD),
    ));
    spans.push(Span::raw(":disabled  "));
    spans.push(Span::styled(
        "T",
        Style::default().add_modifier(Modifier::BOLD),
    ));
    spans.push(Span::raw(":disabled  "));
    spans.push(Span::styled(
        "K",
        Style::default().add_modifier(Modifier::BOLD),
    ));
    spans.push(Span::raw(":disabled  "));
    spans.push(Span::styled(
        "F",
        Style::default().add_modifier(Modifier::BOLD),
    ));
    spans.push(Span::raw(":disabled"));
    if app.active_tab == Tab::Orders || app.active_tab == Tab::Scenarios {
        spans.push(Span::raw("  | "));
        spans.push(Span::styled(
            "Exploration mode",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ));
    }
    if app.active_tab == Tab::Loans {
        spans.push(Span::raw("  "));
        match app.input_mode() {
            InputMode::LoanImportPath => {
                spans.push(Span::styled(
                    "Enter",
                    Style::default().add_modifier(Modifier::BOLD),
                ));
                spans.push(Span::raw(":import  "));
                spans.push(Span::styled(
                    "Esc",
                    Style::default().add_modifier(Modifier::BOLD),
                ));
                spans.push(Span::raw(":cancel path"));
            }
            InputMode::LoanForm => {
                spans.push(Span::styled(
                    "Tab",
                    Style::default().add_modifier(Modifier::BOLD),
                ));
                spans.push(Span::raw(":next field  "));
                spans.push(Span::styled(
                    "Enter",
                    Style::default().add_modifier(Modifier::BOLD),
                ));
                spans.push(Span::raw(":submit  "));
                spans.push(Span::styled(
                    "Esc",
                    Style::default().add_modifier(Modifier::BOLD),
                ));
                spans.push(Span::raw(":cancel"));
            }
            _ => {
                spans.push(Span::styled(
                    "n",
                    Style::default().add_modifier(Modifier::BOLD),
                ));
                spans.push(Span::raw(":new  "));
                spans.push(Span::styled(
                    "b/i",
                    Style::default().add_modifier(Modifier::BOLD),
                ));
                spans.push(Span::raw(":bulk JSON"));
            }
        }
    }
    if app.active_tab == Tab::Settings {
        spans.push(Span::raw("  "));
        spans.push(Span::styled(
            "e",
            Style::default().add_modifier(Modifier::BOLD),
        ));
        spans.push(Span::raw(":edit  "));
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

    let fc = app.focus_context();
    let hint_mode = App::app_mode_for_input_mode(fc.input_mode);
    let ctx = crate::discoverability::context_hints_for(&fc);
    let skip_global = if matches!(hint_mode, AppMode::Navigation) {
        3
    } else {
        0
    };
    for (key, desc) in ctx.into_iter().skip(skip_global).take(4) {
        spans.push(Span::raw("  "));
        spans.push(Span::styled(
            key,
            Style::default().add_modifier(Modifier::BOLD),
        ));
        spans.push(Span::raw(":"));
        spans.push(Span::styled(desc, Style::default().fg(Color::DarkGray)));
    }

    // Status cues (toast + command status) belong at the tail so narrow terminals keep them visible.
    if let Some(toast) = app.toast_manager.latest_active_toast() {
        let color = toast.level.color();
        spans.push(Span::raw("  | "));
        spans.push(Span::styled(
            format!(
                "{} {}",
                toast.level.icon(),
                truncate_detail(&toast.message, 36)
            ),
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
        InputMode::SettingsCredentialEntry => Some(("SETTINGS:API KEY".into(), Color::Yellow)),
        InputMode::ChartSearch => Some(("CHARTS:SEARCH".into(), Color::Cyan)),
        InputMode::OrdersFilter => Some(("ORDERS:FILTER".into(), Color::Yellow)),
        InputMode::LoanForm => Some(("LOANS:FORM".into(), Color::Yellow)),
        InputMode::LoanImportPath => Some(("LOANS:IMPORT".into(), Color::Yellow)),
        InputMode::LogPanel => Some(("LOGS:PANEL".into(), Color::Yellow)),
        InputMode::TreePanel => Some(("TREE:PANEL".into(), Color::Yellow)),
        _ => None,
    }
}

fn append_async_status_spans<'a>(spans: &mut Vec<Span<'a>>, app: &App) {
    let spinner = crate::ui::feedback::StatusIndicator::spinner(app.spinner_frame);
    if app.yield_refresh_pending {
        spans.push(Span::raw("  | "));
        spans.push(Span::styled(
            format!("{spinner} Yield:loading"),
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
            format!("{spinner} Loans:loading"),
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

    if app.fmp_fetch_pending {
        spans.push(Span::raw("  | "));
        spans.push(Span::styled(
            "~ FMP:loading",
            Style::default()
                .fg(Color::Blue)
                .add_modifier(Modifier::BOLD),
        ));
    }

    let alpaca_paper_color = connection_status_color(&app.alpaca_paper_status);
    let alpaca_live_color = connection_status_color(&app.alpaca_live_status);
    spans.push(Span::raw("  | "));
    spans.push(Span::styled("A", Style::default().fg(alpaca_paper_color)));
    spans.push(Span::styled("P", Style::default().fg(alpaca_live_color)));
}

fn truncate_command_id(command_id: &str) -> String {
    text_trunc::truncate_chars(command_id, 18)
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
