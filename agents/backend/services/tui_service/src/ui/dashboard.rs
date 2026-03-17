//! Dashboard tab: symbols table, trend sparklines, metrics bar.

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Paragraph, RenderDirection, Row, Sparkline, Table},
    Frame,
};

use crate::app::App;

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

pub fn render_dashboard(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(4)])
        .split(area);

    const TREND_COLUMN_WIDTH: u16 = 14;

    let (table_area, trend_area) = {
        let horz = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Min(10), Constraint::Length(TREND_COLUMN_WIDTH)])
            .split(chunks[0]);
        (horz[0], horz[1])
    };

    let header = Row::new(["Symbol", "Last", "Bid", "Ask", "Spread", "ROI%"])
        .style(Style::default().add_modifier(Modifier::BOLD | Modifier::UNDERLINED));

    let rows: Vec<Row> = if let Some(ref snap) = app.snapshot {
        snap.inner
            .symbols
            .iter()
            .map(|s| {
                let in_watchlist = app.watchlist().iter().any(|w| w == &s.symbol.to_uppercase());
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

    f.render_widget(table, table_area);

    const ROW_HEIGHT: u16 = 1;
    let table_inner_y = table_area.y + 1;
    let trend_header_rect = Rect {
        x: trend_area.x,
        y: table_inner_y,
        width: trend_area.width,
        height: ROW_HEIGHT,
    };
    f.render_widget(
        Paragraph::new("Trend").style(Style::default().add_modifier(Modifier::BOLD | Modifier::UNDERLINED)),
        trend_header_rect,
    );
    if let Some(ref snap) = app.snapshot {
        for (i, s) in snap.inner.symbols.iter().enumerate() {
            let row_rect = Rect {
                x: trend_area.x,
                y: table_inner_y + ROW_HEIGHT + i as u16 * ROW_HEIGHT,
                width: trend_area.width,
                height: ROW_HEIGHT,
            };
            let data = app.roi_history.get(&s.symbol).map(roi_history_to_sparkline_data);
            let sparkline = match &data {
                Some(d) if !d.is_empty() => Sparkline::default()
                    .data(d.clone())
                    .direction(RenderDirection::RightToLeft)
                    .style(Style::default().fg(Color::Cyan)),
                _ => Sparkline::default().data(&[0u64]),
            };
            f.render_widget(sparkline, row_rect);
        }
    }

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
