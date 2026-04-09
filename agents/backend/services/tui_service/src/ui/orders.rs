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

use super::numeric_format::{max_display_width, pad_left};

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

/// Pilot: buy vs sell counts for the current filtered list (read-only grouping summary).
fn order_side_rollup_line(app: &App) -> Option<String> {
    let snap = app.snapshot().as_ref()?;
    let filtered = app.filtered_orders(snap);
    if filtered.is_empty() {
        return None;
    }
    let mut buy = 0usize;
    let mut sell = 0usize;
    let mut other = 0usize;
    for o in &filtered {
        match o.side.as_str() {
            "BUY" => buy += 1,
            "SELL" => sell += 1,
            _ => other += 1,
        }
    }
    if other == 0 {
        Some(format!("By side: {buy} buy / {sell} sell"))
    } else {
        Some(format!("By side: {buy} buy / {sell} sell / {other} other"))
    }
}

pub fn render_orders_panel(f: &mut Frame, app: &App, area: Rect) {
    let pal = app.ui_palette();
    let mut filter_text = orders_filter_caption(app);
    for rollup in [order_status_rollup_line(app), order_side_rollup_line(app)]
        .into_iter()
        .flatten()
    {
        filter_text.push('\n');
        filter_text.push_str(&rollup);
    }
    let filter_lines = filter_text.lines().count() as u16;
    // Title row + borders + text lines (compact; avoids clipping the rollup).
    let filter_h = filter_lines.saturating_add(2).max(3).min(10);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(filter_h), Constraint::Min(0)])
        .split(area);

    render_orders_filter(f, app, &pal, filter_text, chunks[0]);
    render_orders_table(f, app, &pal, chunks[1]);
}

fn orders_filter_caption(app: &App) -> String {
    if app.order_filter.is_empty() {
        if app.order_filter_active {
            "Filter (typing) — match symbol, status, or side · Esc clears and exits · / exits when empty"
                .to_string()
        } else {
            "Filter — press / or i to type (substring on symbol, status, side)".to_string()
        }
    } else {
        format!(
            "Filter [{}]: «{}» — Esc clears filter · / exits typing when buffer empty",
            if app.order_filter_active {
                "typing"
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

    let (all_rows, qty_col_w) = if let Some(ref snap) = app.snapshot() {
        let filtered = app.filtered_orders(snap);
        let qty_samples: Vec<String> = filtered.iter().map(|o| o.quantity.to_string()).collect();
        let qty_w = max_display_width(qty_samples.iter().map(|s| s.as_str())).clamp(4, 14);
        let rows = filtered
            .iter()
            .zip(qty_samples.iter())
            .map(|(o, qty_s)| {
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
                    Cell::from(Line::from(pad_left(qty_w, qty_s.as_str())).right_aligned()),
                    Cell::from(o.status.clone()),
                    Cell::from(submitted),
                ])
            })
            .collect();
        let col = qty_w.saturating_add(1) as u16;
        (rows, col)
    } else {
        (vec![Row::new(["No data", "", "", "", "", ""])], 6u16)
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
            Constraint::Length(qty_col_w.max(6)),
            Constraint::Length(10),
            Constraint::Length(12),
        ],
    )
    .header(header);

    f.render_widget(table, inner);
}
