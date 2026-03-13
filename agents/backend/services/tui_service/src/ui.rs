//! Ratatui rendering.

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, Tabs},
    Frame,
};
use tui_logger::{TuiLoggerLevelOutput, TuiLoggerWidget};

use crate::app::{App, Tab};
use crate::events::{ConnectionState, ConnectionStatus, ConnectionTarget};

pub fn render(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // status bar
            Constraint::Length(3), // tab bar
            Constraint::Min(0),    // main content
            Constraint::Length(1), // hint bar
        ])
        .split(f.area());

    render_status_bar(f, app, chunks[0]);
    render_tab_bar(f, app, chunks[1]);
    render_main(f, app, chunks[2]);
    render_hint_bar(f, chunks[3]);
}

fn render_status_bar(f: &mut Frame, app: &App, area: Rect) {
    let (mode, strategy, source_label, source_color, is_stale) =
        if let Some(ref snap) = app.snapshot {
            let stale = snap.is_stale(app.config.snapshot_ttl_secs as i64);
            let color = if stale { Color::Yellow } else { Color::Green };
            (
                snap.inner.mode.as_str().to_owned(),
                snap.inner.strategy.as_str().to_owned(),
                snap.source.label(),
                color,
                stale,
            )
        } else {
            ("---".into(), "---".into(), "NO DATA", Color::DarkGray, false)
        };

    let mut spans = vec![
        Span::raw(format!(" {} | ", app.config.backend_id.to_uppercase())),
        Span::styled(mode, Style::default().fg(Color::Cyan)),
        Span::raw(" | "),
        Span::styled(strategy, Style::default().fg(Color::Yellow)),
        Span::raw("  "),
        Span::styled(
            format!("[{}]", source_label),
            Style::default()
                .fg(source_color)
                .add_modifier(Modifier::BOLD),
        ),
    ];

    if is_stale {
        spans.push(Span::styled(
            " [STALE]",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ));
    }

    spans.push(Span::raw("  "));
    spans.push(render_connection_badge(ConnectionTarget::Nats, &app.nats_status));

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
    match app.active_tab {
        Tab::Dashboard => render_dashboard(f, app, area),
        Tab::Positions => render_positions(f, app, area),
        Tab::Orders => render_orders(f, app, area),
        Tab::Alerts => render_alerts(f, app, area),
        Tab::Logs => render_logs(f, app, area),
    }
}

fn render_dashboard(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(4)])
        .split(area);

    // Symbols table
    // TODO(exarp): T-1773357423912362000 — add a "Trend" column using ratatui's
    // built-in Sparkline widget.
    // Requires a per-symbol ring buffer of recent roi values in app state
    // (e.g. HashMap<String, VecDeque<u64>> updated each tick; Sparkline takes &[u64]).
    // TODO(exarp): T-1773357423930509000 — on Enter/Space over a row, open a
    // tui-popup (tui-widgets crate) showing full SymbolSnapshot details: candle
    // OHLCV, maker/taker counts, volume.
    let header = Row::new(["Symbol", "Last", "Bid", "Ask", "Spread", "ROI%"])
        .style(Style::default().add_modifier(Modifier::BOLD | Modifier::UNDERLINED));

    let rows: Vec<Row> = if let Some(ref snap) = app.snapshot {
        snap.inner
            .symbols
            .iter()
            .map(|s| {
                let in_watchlist = app.config.watchlist.contains(&s.symbol.to_uppercase());
                let style = if in_watchlist {
                    Style::default().fg(Color::Cyan)
                } else {
                    Style::default()
                };
                Row::new([
                    Cell::from(s.symbol.clone()),
                    Cell::from(format!("{:.2}", s.last)),
                    Cell::from(format!("{:.2}", s.bid)),
                    Cell::from(format!("{:.2}", s.ask)),
                    Cell::from(format!("{:.2}", s.spread)),
                    Cell::from(format!("{:.2}", s.roi)),
                ])
                .style(style)
            })
            .collect()
    } else {
        vec![Row::new(["Waiting for data...", "", "", "", "", ""])]
    };

    let table = Table::new(
        rows,
        [
            Constraint::Length(8),
            Constraint::Length(9),
            Constraint::Length(9),
            Constraint::Length(9),
            Constraint::Length(9),
            Constraint::Length(8),
        ],
    )
    .header(header)
    .block(Block::default().title("Symbols").borders(Borders::ALL));

    f.render_widget(table, chunks[0]);

    // Metrics bar
    let metrics_text = if let Some(ref snap) = app.snapshot {
        let m = &snap.inner.metrics;
        format!(
            " Net Liq: ${:.0}  |  BP: ${:.0}  |  Margin: ${:.0}  |  Comms: ${:.2}  |  TWS: {}  |  Portal: {}",
            m.net_liq,
            m.buying_power,
            m.margin_requirement,
            m.commissions,
            if m.tws_ok { "OK" } else { "--" },
            if m.portal_ok { "OK" } else { "--" },
        )
    } else {
        " No metrics".into()
    };

    let metrics_widget = Paragraph::new(metrics_text)
        .block(Block::default().title("Metrics").borders(Borders::ALL))
        .style(Style::default().fg(Color::White));

    f.render_widget(metrics_widget, chunks[1]);
}

fn render_positions(f: &mut Frame, app: &App, area: Rect) {
    let header = Row::new(["Symbol", "Qty", "Cost", "Mark", "P&L"])
        .style(Style::default().add_modifier(Modifier::BOLD | Modifier::UNDERLINED));

    let rows: Vec<Row> = if let Some(ref snap) = app.snapshot {
        snap.inner
            .positions
            .iter()
            .map(|p| {
                let pnl_color = if p.unrealized_pnl >= 0.0 {
                    Color::Green
                } else {
                    Color::Red
                };
                Row::new([
                    Cell::from(p.symbol.clone()),
                    Cell::from(p.quantity.to_string()),
                    Cell::from(format!("{:.2}", p.cost_basis)),
                    Cell::from(format!("{:.2}", p.mark)),
                    Cell::from(format!("{:+.2}", p.unrealized_pnl))
                        .style(Style::default().fg(pnl_color)),
                ])
            })
            .collect()
    } else {
        vec![Row::new(["No data", "", "", "", ""])]
    };

    let table = Table::new(
        rows,
        [
            Constraint::Length(10),
            Constraint::Length(6),
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Length(10),
        ],
    )
    .header(header)
    .block(Block::default().title("Positions").borders(Borders::ALL));

    f.render_widget(table, area);
}

fn render_orders(f: &mut Frame, app: &App, area: Rect) {
    // TODO(exarp): T-1773357423930509000 — add a cancel-order confirmation
    // modal using tui-popup (tui-widgets).
    // On Enter over a selected row: render tui_popup::Popup over the table asking
    // "Cancel order {id}? [y/n]". Requires TableState for row selection.
    // TODO(exarp): T-1773357423945485000 — add an order filter input bar
    // (ratatui-textarea crate, single-line mode) at the top of this view to
    // filter orders by symbol or status.
    let header = Row::new(["ID", "Symbol", "Side", "Qty", "Status"])
        .style(Style::default().add_modifier(Modifier::BOLD | Modifier::UNDERLINED));

    let rows: Vec<Row> = if let Some(ref snap) = app.snapshot {
        snap.inner
            .orders
            .iter()
            .map(|o| {
                let side_color = if o.side == "BUY" {
                    Color::Green
                } else {
                    Color::Red
                };
                Row::new([
                    Cell::from(o.id.clone()),
                    Cell::from(o.symbol.clone()),
                    Cell::from(o.side.clone()).style(Style::default().fg(side_color)),
                    Cell::from(o.quantity.to_string()),
                    Cell::from(o.status.clone()),
                ])
            })
            .collect()
    } else {
        vec![Row::new(["No data", "", "", "", ""])]
    };

    let table = Table::new(
        rows,
        [
            Constraint::Length(10),
            Constraint::Length(8),
            Constraint::Length(6),
            Constraint::Length(6),
            Constraint::Length(10),
        ],
    )
    .header(header)
    .block(Block::default().title("Orders").borders(Borders::ALL));

    f.render_widget(table, area);
}

fn render_alerts(f: &mut Frame, app: &App, area: Rect) {
    use api::AlertLevel;

    let lines: Vec<Line> = if let Some(ref snap) = app.snapshot {
        snap.inner
            .alerts
            .iter()
            .rev()
            .map(|a| {
                let color = match a.level {
                    AlertLevel::Info => Color::Cyan,
                    AlertLevel::Warning => Color::Yellow,
                    AlertLevel::Error => Color::Red,
                };
                Line::from(Span::styled(
                    format!("[{}] {}", a.timestamp.format("%H:%M:%S"), a.message),
                    Style::default().fg(color),
                ))
            })
            .collect()
    } else {
        vec![Line::from("No alerts")]
    };

    let widget =
        Paragraph::new(lines).block(Block::default().title("Alerts").borders(Borders::ALL));

    f.render_widget(widget, area);
}

fn render_logs(f: &mut Frame, app: &App, area: Rect) {
    let widget = TuiLoggerWidget::default()
        .block(Block::default().title("Logs  [+/-]:level  [↑↓ PgUp/Dn]:scroll  [h]:hide  [Esc]:reset").borders(Borders::ALL))
        .style_error(Style::default().fg(Color::Red))
        .style_warn(Style::default().fg(Color::Yellow))
        .style_info(Style::default().fg(Color::Cyan))
        .style_debug(Style::default().fg(Color::White))
        .style_trace(Style::default().fg(Color::DarkGray))
        .output_separator(' ')
        .output_timestamp(Some("%H:%M:%S".to_string()))
        .output_level(Some(TuiLoggerLevelOutput::Abbreviated))
        .output_target(false)
        .output_file(false)
        .output_line(false)
        .state(&app.log_state);
    f.render_widget(widget, area);
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

fn render_hint_bar(f: &mut Frame, area: Rect) {
    let line = Line::from(vec![
        Span::raw(" "),
        Span::styled("q", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(":quit  "),
        Span::styled(
            "←/→ Tab/BackTab",
            Style::default().add_modifier(Modifier::BOLD),
        ),
        Span::raw(":switch tab  "),
        Span::styled("1-5", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(":jump to tab  "),
        Span::styled(
            "↑/↓ PgUp/PgDn",
            Style::default().add_modifier(Modifier::BOLD),
        ),
        Span::raw(":scroll logs"),
    ]);
    f.render_widget(Paragraph::new(line), area);
}
