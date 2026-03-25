//! Positions tab: table with combo/flat view and scroll.

use std::collections::{HashMap, HashSet};

use api::RuntimePositionDto;
use ratatui::{
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Row, Table},
    Frame,
};

use crate::app::App;

/// Return type for `positions_display_info`: (row_count, position_index_per_row, combo_key_per_row)
type DisplayInfo = (
    usize,
    Vec<Option<usize>>,
    Vec<Option<(String, String, String)>>,
);

fn symbol_stem(symbol: &str) -> &str {
    symbol.split_whitespace().next().unwrap_or(symbol)
}

fn make_combo_key(pos: &RuntimePositionDto) -> (String, String, String) {
    let stem = symbol_stem(&pos.symbol).to_string();
    let account = pos.account_id.clone().unwrap_or_default();
    let strategy = pos.strategy.clone().unwrap_or_default();
    (account, strategy, stem)
}

#[derive(Debug)]
pub struct ComboGroup {
    pub key: (String, String, String),
    pub position_indices: Vec<usize>,
    pub total_quantity: i32,
    pub total_pnl: f64,
}

fn build_combo_groups(positions: &[RuntimePositionDto]) -> Vec<ComboGroup> {
    let mut groups: HashMap<(String, String, String), Vec<usize>> = HashMap::new();
    for (i, pos) in positions.iter().enumerate() {
        let key = make_combo_key(pos);
        groups.entry(key).or_default().push(i);
    }
    groups
        .into_iter()
        .map(|((account, strategy, stem), position_indices)| {
            let total_quantity: i32 = position_indices
                .iter()
                .map(|&i| positions[i].quantity)
                .sum();
            let total_pnl: f64 = position_indices
                .iter()
                .map(|&i| positions[i].unrealized_pnl)
                .sum();
            let key = (account, strategy, stem);
            ComboGroup {
                key,
                position_indices,
                total_quantity,
                total_pnl,
            }
        })
        .collect()
}

pub fn positions_display_info(
    positions: &[RuntimePositionDto],
    combo_view: bool,
    expanded: &HashSet<(String, String, String)>,
) -> DisplayInfo {
    if !combo_view || positions.is_empty() {
        let len = positions.len();
        let index_map: Vec<Option<usize>> = (0..len).map(Some).collect();
        let combo_key_per_row = vec![None; len];
        return (len, index_map, combo_key_per_row);
    }

    let groups = build_combo_groups(positions);
    let mut display_len = 0;
    let mut index_map: Vec<Option<usize>> = Vec::new();
    let mut combo_key_per_row: Vec<Option<(String, String, String)>> = Vec::new();

    for group in &groups {
        let is_expanded = expanded.contains(&group.key);
        display_len += 1;
        combo_key_per_row.push(Some(group.key.clone()));
        index_map.push(None);

        if is_expanded {
            for &pos_idx in &group.position_indices {
                display_len += 1;
                index_map.push(Some(pos_idx));
                combo_key_per_row.push(None);
            }
        }
    }

    (display_len, index_map, combo_key_per_row)
}

/// Label for position type (e.g. option, equity).
pub fn position_type_label(typ: Option<&str>) -> &'static str {
    match typ {
        Some("OPT") | Some("OPTION") => "Option",
        Some("STK") | Some("STOCK") => "Equity",
        Some("FUT") => "Futures",
        Some(_) => "—",
        None => "—",
    }
}

#[allow(unused_imports)]
pub use render_positions_panel as render_positions;

pub fn render_positions_panel(f: &mut Frame, app: &App, area: Rect) {
    render_positions_table(f, app, area);
}

pub fn render_positions_table(f: &mut Frame, app: &App, area: Rect) {
    let (header, rows, selected_label) = if let Some(ref snap) = app.snapshot() {
        let positions = &snap.dto().positions;
        let (_, index_map, combo_key_per_row) = positions_display_info(
            positions,
            app.positions_combo_view,
            &app.positions_expanded_combos,
        );
        let groups = if app.positions_combo_view {
            build_combo_groups(positions)
        } else {
            Vec::new()
        };

        let header = Row::new([
            Cell::from("Symbol").style(Style::default().add_modifier(Modifier::BOLD)),
            Cell::from("Qty").style(Style::default().add_modifier(Modifier::BOLD)),
            Cell::from("Cost").style(Style::default().add_modifier(Modifier::BOLD)),
            Cell::from("Mark").style(Style::default().add_modifier(Modifier::BOLD)),
            Cell::from("P&L").style(Style::default().add_modifier(Modifier::BOLD)),
            Cell::from("Strat").style(Style::default().add_modifier(Modifier::BOLD)),
        ]);

        let mut selected_label = String::new();
        let scroll = if index_map.is_empty() {
            0
        } else {
            app.positions_scroll.min(index_map.len().saturating_sub(1))
        };

        let table_rows: Vec<Row> = index_map
            .iter()
            .enumerate()
            .map(|(row_idx, pos_idx)| {
                let is_selected = row_idx == scroll;
                let base_style = if is_selected {
                    Style::default().add_modifier(Modifier::REVERSED)
                } else {
                    Style::default()
                };

                if let Some(Some(combo_key)) = combo_key_per_row.get(row_idx) {
                    let group = groups.iter().find(|g| &g.key == combo_key);
                    let (sym, qty, cost, mark, pnl, strat) = if let Some(g) = group {
                        let stem = &combo_key.2;
                        let total_cost: f64 = g
                            .position_indices
                            .iter()
                            .map(|&i| positions[i].cost_basis)
                            .sum();
                        let avg_cost = if g.total_quantity != 0 {
                            total_cost / g.total_quantity.abs() as f64
                        } else {
                            0.0
                        };
                        let total_mark: f64 = g
                            .position_indices
                            .iter()
                            .map(|&i| positions[i].mark * positions[i].quantity as f64)
                            .sum();
                        (
                            format!(
                                "{} [{}]",
                                stem,
                                if app.positions_expanded_combos.contains(combo_key) {
                                    '-'
                                } else {
                                    '+'
                                }
                            ),
                            g.total_quantity.to_string(),
                            format!("{:.2}", avg_cost),
                            format!("{:.2}", total_mark / g.total_quantity.abs() as f64),
                            format!("{:+.2}", g.total_pnl),
                            combo_key.1.clone(),
                        )
                    } else {
                        (
                            "—".into(),
                            "0".into(),
                            "—".into(),
                            "—".into(),
                            "—".into(),
                            "—".into(),
                        )
                    };
                    if is_selected {
                        selected_label = sym.clone();
                    }
                    Row::new([
                        Cell::from(sym).style(Style::default().add_modifier(Modifier::BOLD)),
                        Cell::from(qty).style(Style::default().add_modifier(Modifier::BOLD)),
                        Cell::from(cost).style(Style::default().add_modifier(Modifier::BOLD)),
                        Cell::from(mark).style(Style::default().add_modifier(Modifier::BOLD)),
                        Cell::from(pnl).style(Style::default().add_modifier(Modifier::BOLD)),
                        Cell::from(strat).style(Style::default().add_modifier(Modifier::BOLD)),
                    ])
                    .style(base_style)
                } else if let Some(idx) = *pos_idx {
                    let pos = &positions[idx];
                    let pnl_color = if pos.unrealized_pnl >= 0.0 {
                        Color::Green
                    } else {
                        Color::Red
                    };
                    if is_selected {
                        selected_label = pos.symbol.clone();
                    }
                    Row::new([
                        Cell::from(format!("  {}", pos.symbol)),
                        Cell::from(pos.quantity.to_string()),
                        Cell::from(format!("{:.2}", pos.cost_basis)),
                        Cell::from(format!("{:.2}", pos.mark)),
                        Cell::from(format!("{:+.2}", pos.unrealized_pnl)).style(if is_selected {
                            base_style
                        } else {
                            Style::default().fg(pnl_color)
                        }),
                        Cell::from(pos.strategy.clone().unwrap_or_else(|| "—".into())),
                    ])
                    .style(base_style)
                } else {
                    Row::new(vec![Cell::from(""); 6])
                }
            })
            .collect();
        (header, table_rows, selected_label)
    } else {
        let header = Row::new(["Symbol", "Qty", "Cost", "Mark", "P&L", "Strat"])
            .style(Style::default().add_modifier(Modifier::BOLD));
        let rows = vec![Row::new(["Waiting for snapshot…", "", "", "", "", ""])];
        (header, rows, String::new())
    };

    let block = Block::default()
        .title({
            if selected_label.is_empty() {
                if app.positions_combo_view {
                    " Positions  [↑↓] navigate  [c] combo view  [Enter] detail ".to_string()
                } else {
                    " Positions  [↑↓] navigate  [Enter] detail ".to_string()
                }
            } else if app.positions_combo_view {
                format!(
                    " Positions  Sel: {}  [↑↓] navigate  [c] combo  [Enter] detail ",
                    selected_label
                )
            } else {
                format!(
                    " Positions  Sel: {}  [↑↓] navigate  [Enter] detail ",
                    selected_label
                )
            }
        })
        .borders(Borders::ALL);

    let inner = block.inner(area);
    f.render_widget(block, area);

    let len = rows.len();
    let visible_height = inner.height.saturating_sub(1).max(1) as usize;
    let cursor = if len == 0 {
        0
    } else {
        app.positions_scroll.min(len - 1)
    };
    let viewport = if len <= visible_height {
        0
    } else {
        cursor
            .saturating_sub(visible_height / 2)
            .min(len - visible_height)
    };
    let window: Vec<Row> = rows
        .into_iter()
        .skip(viewport)
        .take(visible_height.max(1))
        .collect();

    let table = Table::new(
        window,
        [
            Constraint::Length(18),
            Constraint::Length(5),
            Constraint::Length(7),
            Constraint::Length(7),
            Constraint::Length(8),
            Constraint::Length(8),
        ],
    )
    .header(header);
    f.render_widget(table, inner);
}
