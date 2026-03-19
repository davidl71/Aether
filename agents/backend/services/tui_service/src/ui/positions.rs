//! Positions tab: table with combo/flat view and scroll.

use std::collections::HashSet;

use api::RuntimePositionDto;
use ratatui::{
    layout::{Constraint, Rect},
    style::{Modifier, Style},
    widgets::{Block, Borders, Cell, Row, Table},
    Frame,
};

use crate::app::App;

/// Display row count and index maps for positions (flat or combo grouped).
/// Returns (display_len, row_index -> Option<position_index>, row_index -> Option<combo_key>).
pub fn positions_display_info(
    positions: &[RuntimePositionDto],
    _combo_view: bool,
    _expanded: &HashSet<(String, String, String)>,
) -> (
    usize,
    Vec<Option<usize>>,
    Vec<Option<(String, String, String)>>,
) {
    let len = positions.len();
    let index_map: Vec<Option<usize>> = (0..len).map(Some).collect();
    let combo_key_per_row = vec![None; len];
    (len, index_map, combo_key_per_row)
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

pub fn render_positions(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title(" Positions ")
        .borders(Borders::ALL);
    let inner = block.inner(area);
    f.render_widget(block, area);

    let (header, rows) = if let Some(ref snap) = app.snapshot {
        let positions = &snap.dto().positions;
        let header = Row::new([
            Cell::from("ID").style(Style::default().add_modifier(Modifier::BOLD)),
            Cell::from("Symbol").style(Style::default().add_modifier(Modifier::BOLD)),
            Cell::from("Qty").style(Style::default().add_modifier(Modifier::BOLD)),
            Cell::from("Mark").style(Style::default().add_modifier(Modifier::BOLD)),
            Cell::from("P&L").style(Style::default().add_modifier(Modifier::BOLD)),
        ]);
        let rows: Vec<Row> = positions
            .iter()
            .map(|p| {
                Row::new([
                    Cell::from(p.id.clone()),
                    Cell::from(p.symbol.clone()),
                    Cell::from(p.quantity.to_string()),
                    Cell::from(format!("{:.2}", p.mark)),
                    Cell::from(format!("{:+.2}", p.unrealized_pnl)),
                ])
            })
            .collect();
        (header, rows)
    } else {
        let header = Row::new(["ID", "Symbol", "Qty", "Mark", "P&L"])
            .style(Style::default().add_modifier(Modifier::BOLD));
        let rows = vec![Row::new(["Waiting for snapshot…", "", "", "", ""])];
        (header, rows)
    };

    let len = rows.len();
    let visible_height = (inner.height as usize).saturating_sub(2).max(1);
    let scroll = if len <= 1 {
        0
    } else {
        app.positions_scroll.min(len.saturating_sub(1))
    };
    let window: Vec<Row> = rows.into_iter().skip(scroll).take(visible_height).collect();
    let table = Table::new(window, [
        Constraint::Length(12),
        Constraint::Length(10),
        Constraint::Length(6),
        Constraint::Length(10),
        Constraint::Length(10),
    ])
    .header(header);
    f.render_widget(table, inner);
}
