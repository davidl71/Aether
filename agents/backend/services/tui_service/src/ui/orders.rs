//! Orders tab: filter bar and orders table with scroll.

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
    Frame,
};

use crate::app::App;

pub fn render_orders(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(area);

    let filter_text = if app.order_filter.is_empty() {
        "Filter: / to activate".to_string()
    } else {
        format!("Filter: {} (ESC to clear)", app.order_filter)
    };
    let filter_widget = Paragraph::new(filter_text)
        .block(Block::default().title("Orders").borders(Borders::ALL))
        .style(if app.order_filter.is_empty() {
            Style::default().fg(Color::DarkGray)
        } else {
            Style::default().fg(Color::Cyan)
        });
    f.render_widget(filter_widget, chunks[0]);

    let header = Row::new(["ID", "Symbol", "Side", "Qty", "Status", "Submitted"])
        .style(Style::default().add_modifier(Modifier::BOLD | Modifier::UNDERLINED));

    let filter_lower = app.order_filter.to_lowercase();
    let all_rows: Vec<Row> = if let Some(ref snap) = app.snapshot {
        snap.dto()
            .orders
            .iter()
            .filter(|o| {
                if app.order_filter.is_empty() {
                    true
                } else {
                    o.symbol.to_lowercase().contains(&filter_lower)
                        || o.status.to_lowercase().contains(&filter_lower)
                        || o.side.to_lowercase().contains(&filter_lower)
                }
            })
            .map(|o| {
                let side_color = if o.side == "BUY" {
                    Color::Green
                } else {
                    Color::Red
                };
                let submitted = o.submitted_at.format("%m-%d %H:%M").to_string();
                Row::new([
                    Cell::from(o.id.clone()),
                    Cell::from(o.symbol.clone()),
                    Cell::from(o.side.clone()).style(Style::default().fg(side_color)),
                    Cell::from(o.quantity.to_string()),
                    Cell::from(o.status.clone()),
                    Cell::from(submitted),
                ])
            })
            .collect()
    } else {
        vec![Row::new(["No data", "", "", "", "", ""])]
    };

    let len = all_rows.len();
    let visible_height = (chunks[1].height as usize).saturating_sub(2).max(1);
    let scroll = if len <= 1 {
        0
    } else {
        app.orders_scroll.min(len.saturating_sub(1))
    };
    let window: Vec<Row> = all_rows
        .into_iter()
        .skip(scroll)
        .take(visible_height)
        .collect();

    let table = Table::new(
        window,
        [
            Constraint::Length(10),
            Constraint::Length(8),
            Constraint::Length(6),
            Constraint::Length(6),
            Constraint::Length(10),
            Constraint::Length(12),
        ],
    )
    .header(header)
    .block(
        Block::default()
            .title("Orders  [↑↓ PgUp/PgDn]:scroll  [Enter]:detail")
            .borders(Borders::ALL),
    );

    f.render_widget(table, chunks[1]);
}
