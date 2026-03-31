//! Dashboard tab: symbols table, trend sparklines, metrics bar.

use std::collections::HashSet;

use chrono::{Local, Utc};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, Borders, Cell, Paragraph, RenderDirection, Row, Sparkline, Table},
    Frame,
};

use crate::app::App;

const TREND_COLUMN_WIDTH: u16 = 14;
const METRICS_HEIGHT: u16 = 4;

/// Normalize ROI history (f64) to u64 in 0..=100 for Sparkline. Handles empty and constant data.
fn roi_history_to_sparkline_data(history: &std::collections::VecDeque<f64>) -> Vec<u64> {
    if history.is_empty() {
        return vec![];
    }
    let min = history.iter().cloned().fold(f64::INFINITY, f64::min);
    let max = history.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let range = (max - min).max(1e-9);
    history
        .iter()
        .map(|&v| {
            let n = ((v - min) / range * 99.0 + 0.5).clamp(0.0, 99.0);
            n as u64
        })
        .collect()
}

#[allow(unused_imports)]
pub use render_dashboard_panel as render_dashboard;

pub fn render_dashboard_panel(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(METRICS_HEIGHT)])
        .split(area);

    render_dashboard_market_view(f, app, chunks[0]);
    render_dashboard_metrics(f, app, chunks[1]);
}

pub fn render_dashboard_market_view(f: &mut Frame, app: &App, area: Rect) {
    let horz = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(10), Constraint::Length(TREND_COLUMN_WIDTH)])
        .split(area);
    let table_area = horz[0];
    let trend_area = horz[1];

    let header = Row::new([
        Cell::from("Symbol"),
        Cell::from(Line::from("Last").right_aligned()),
        Cell::from(Line::from("Bid").right_aligned()),
        Cell::from(Line::from("Ask").right_aligned()),
        Cell::from(Line::from("Spread").right_aligned()),
        Cell::from(Line::from("Move%").right_aligned()),
    ])
    .style(Style::default().add_modifier(Modifier::BOLD | Modifier::UNDERLINED));

    let rows: Vec<Row> = if let Some(ref snap) = app.snapshot() {
        let watchlist_upper: HashSet<_> =
            app.watchlist().iter().map(|w| w.to_uppercase()).collect();
        snap.inner
            .symbols
            .iter()
            .enumerate()
            .map(|(i, s)| {
                let in_watchlist = watchlist_upper.contains(&s.symbol.to_uppercase());
                let is_selected = i == app.dashboard_table.selected();
                let style = if is_selected {
                    Style::default()
                        .add_modifier(Modifier::REVERSED)
                        .fg(if in_watchlist {
                            Color::Cyan
                        } else {
                            Color::White
                        })
                } else if in_watchlist {
                    Style::default().fg(Color::Cyan)
                } else {
                    Style::default()
                };
                Row::new([
                    Cell::from(s.symbol.clone()),
                    Cell::from(Line::from(format!("{:.2}", s.last)).right_aligned()),
                    Cell::from(Line::from(format!("{:.2}", s.bid)).right_aligned()),
                    Cell::from(Line::from(format!("{:.2}", s.ask)).right_aligned()),
                    Cell::from(Line::from(format!("{:.2}", s.spread)).right_aligned()),
                    Cell::from(Line::from(format!("{:.2}", s.roi)).right_aligned()),
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

    f.render_widget(table, table_area);

    let trend_header_rect = Rect {
        x: trend_area.x,
        y: trend_area.y + 1,
        width: trend_area.width,
        height: 1,
    };
    f.render_widget(
        Paragraph::new("Trend")
            .style(Style::default().add_modifier(Modifier::BOLD | Modifier::UNDERLINED)),
        trend_header_rect,
    );
    if let Some(ref snap) = app.snapshot() {
        for (i, s) in snap.inner.symbols.iter().enumerate() {
            let row_rect = Rect {
                x: trend_area.x,
                y: trend_area.y + 2 + i as u16,
                width: trend_area.width,
                height: 1,
            };
            let data = app
                .roi_history
                .get(&s.symbol)
                .map(roi_history_to_sparkline_data);
            let sparkline = match &data {
                Some(d) if !d.is_empty() => Sparkline::default()
                    .data(d.clone())
                    .direction(RenderDirection::RightToLeft)
                    .style(Style::default().fg(Color::Cyan)),
                _ => Sparkline::default().data([0u64].as_ref()),
            };
            f.render_widget(sparkline, row_rect);
        }
    }
}

pub fn render_dashboard_metrics(f: &mut Frame, app: &App, area: Rect) {
    let clock_text = format!(
        " Local: {}  |  UTC: {}",
        Local::now().format("%H:%M:%S"),
        Utc::now().format("%H:%M:%S"),
    );

    let metrics_lines = if let Some(ref snap) = app.snapshot() {
        let m = &snap.inner.metrics;
        vec![
            Line::from(format!(
                " Net Liq: ${:.0}  |  BP: ${:.0}  |  Margin: ${:.0}  |  Comms: ${:.2}  |  TWS: {}  |  Portal: {}",
                m.net_liq,
                m.buying_power,
                m.margin_requirement,
                m.commissions,
                if m.tws_ok { "OK" } else { "--" },
                if m.portal_ok { "OK" } else { "--" },
            )),
            Line::from(clock_text),
        ]
    } else {
        vec![Line::from(" No metrics"), Line::from(clock_text)]
    };

    let metrics_widget = Paragraph::new(metrics_lines)
        .block(Block::default().title("Metrics").borders(Borders::ALL))
        .style(Style::default().fg(Color::White));

    f.render_widget(metrics_widget, area);
}
