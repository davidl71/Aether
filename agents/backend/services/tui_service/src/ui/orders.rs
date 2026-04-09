//! Orders tab: read-only snapshot orders (filter + scroll). No placement or cancel.

use std::collections::BTreeMap;

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
    Frame,
};

use crate::app::App;
use crate::scrollable_table::{centered_viewport_start, clamp_index};

#[allow(unused_imports)]
pub use render_orders_panel as render_orders;

/// Pilot: status counts for the current filtered list (read-only grouping summary).
fn order_status_rollup_line(app: &App) -> Option<String> {
    let snap = app.snapshot().as_ref()?;
    let filtered = app.filtered_orders(snap);
    if filtered.is_empty() {
        return None;
    }
    let mut counts: BTreeMap<String, usize> = BTreeMap::new();
    for o in &filtered {
        *counts.entry(o.status.clone()).or_insert(0) += 1;
    }
    let parts: Vec<String> = counts
        .into_iter()
        .map(|(k, n)| format!("{k}:{n}"))
        .collect();
    Some(format!("By status: {}", parts.join("  ")))
}

pub fn render_orders_panel(f: &mut Frame, app: &App, area: Rect) {
    let pal = app.ui_palette();
    let rollup = order_status_rollup_line(app);
    let filter_text = orders_filter_caption(app);
    let mut filter_block = filter_text;
    if let Some(r) = &rollup {
        filter_block.push('\n');
        filter_block.push_str(r);
    }
    let filter_lines = filter_block.lines().count() as u16;
    // Title row + borders + text lines (compact; avoids clipping the rollup).
    let filter_h = filter_lines.saturating_add(2).max(3).min(8);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(filter_h), Constraint::Min(0)])
        .split(area);

    render_orders_filter(f, app, &pal, filter_block, chunks[0]);
    render_orders_table(f, app, &pal, chunks[1]);
}

fn orders_filter_caption(app: &App) -> String {
    if app.order_filter.is_empty() {
        if app.order_filter_active {
            "Filter mode: type symbol, status, or side; Esc clear; / exits when empty".to_string()
        } else {
            "Filter: / to activate".to_string()
        }
    } else {
        format!(
            "Filter [{}]: {} (symbol/status/side substring; Esc clear; / exits when empty)",
            if app.order_filter_active {
                "ACTIVE"
            } else {
                "saved"
            },
            app.order_filter
        )
    }
}

fn render_orders_filter(
    f: &mut Frame,
    app: &App,
    pal: &crate::theme_palette::UiPalette,
    text: String,
    area: Rect,
) {
    let filter_widget = Paragraph::new(text)
        .block(
            Block::default()
                .title(if app.order_filter_active {
                    "Orders [FILTER]"
                } else {
                    "Orders"
                })
                .borders(Borders::ALL)
                .border_style(if app.order_filter_active {
                    Style::default()
                        .fg(pal.filter_border_active)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                }),
        )
        .style(if app.order_filter.is_empty() {
            if app.order_filter_active {
                Style::default().fg(pal.filter_text_active)
            } else {
                Style::default().fg(pal.muted)
            }
        } else {
            Style::default().fg(pal.filter_text_active)
        });
    f.render_widget(filter_widget, area);
}

pub fn render_orders_table(
    f: &mut Frame,
    app: &App,
    pal: &crate::theme_palette::UiPalette,
    area: Rect,
) {
    let header = Row::new([
        Cell::from("ID"),
        Cell::from("Symbol"),
        Cell::from("Side"),
        Cell::from(Line::from("Qty").right_aligned()),
        Cell::from("Status"),
        Cell::from("Submitted"),
    ])
    .style(Style::default().add_modifier(Modifier::BOLD | Modifier::UNDERLINED));

    let filter_lower = app.order_filter.to_lowercase();
    let all_rows: Vec<Row> = if let Some(ref snap) = app.snapshot() {
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
                    Cell::from(Line::from(o.quantity.to_string()).right_aligned()),
                    Cell::from(o.status.clone()),
                    Cell::from(submitted),
                ])
            })
            .collect()
    } else {
        vec![Row::new(["No data", "", "", "", "", ""])]
    };

    let selected_order_summary = if let Some(ref snap) = app.snapshot() {
        let filtered = app.filtered_orders(snap);
        filtered
            .get(
                app.orders_table
                    .selected()
                    .min(filtered.len().saturating_sub(1)),
            )
            .map(|order| format!("{} {} {}", order.symbol, order.side, order.status))
    } else {
        None
    };

    let table_block = Block::default()
        .title(
            selected_order_summary
                .as_ref()
                .map(|summary| {
                    format!(
                        "Orders  [↑↓ PgUp/PgDn]:select  [Enter]:detail  [read-only]  Sel: {}",
                        summary
                    )
                })
                .unwrap_or_else(|| {
                    "Orders  [↑↓ PgUp/PgDn]:select  [Enter]:detail  [read-only]".to_string()
                }),
        )
        .borders(Borders::ALL);
    let inner = table_block.inner(area);
    f.render_widget(table_block, area);

    let len = all_rows.len();
    let visible_height = inner.height.saturating_sub(1).max(1) as usize;
    let cursor = clamp_index(app.orders_table.selected(), len);
    let viewport = centered_viewport_start(cursor, len, visible_height);
    let window: Vec<Row> = all_rows
        .into_iter()
        .skip(viewport)
        .take(visible_height)
        .enumerate()
        .map(|(i, row)| {
            let is_selected = i + viewport == cursor;
            if is_selected {
                row.style(
                    Style::default()
                        .fg(pal.selection_fg)
                        .bg(pal.selection_bg)
                        .add_modifier(Modifier::BOLD),
                )
            } else {
                row
            }
        })
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
    .header(header);

    f.render_widget(table, inner);
}
