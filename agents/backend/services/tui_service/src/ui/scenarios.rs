//! Scenarios tab: box spread scenarios per calendar day, DTE +4 around the money.
//! Supports expanding/contracting DTE window ([ ]) and strike width filter (w). Compares to T-bill benchmark.

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
    Frame,
};

use api::finance_rates::BenchmarksResponse;
use api::ScenarioDto;
use chrono::Utc;

use crate::app::App;

/// Consider ATM when strike is within this fraction of underlying (2%).
const ATM_TOLERANCE: f64 = 0.02;

/// Closest benchmark rate (T-bill / Treasury) for a given DTE; returns rate in percent (e.g. 4.5).
fn benchmark_rate_for_dte(dte: i32, benchmarks: Option<&BenchmarksResponse>) -> Option<f64> {
    let b = benchmarks.as_ref()?;
    let mut best: Option<(i32, f64)> = None;
    for r in &b.treasury.rates {
        let rdte = r.days_to_expiry.unwrap_or(0);
        let dist = (rdte - dte).abs();
        if best.map(|(d, _)| dist < d).unwrap_or(true) {
            best = Some((dist, r.rate));
        }
    }
    best.map(|(_, rate)| rate)
}

pub fn days_to_expiry(s: &ScenarioDto) -> Option<i32> {
    s.days_to_expiry.or_else(|| {
        // Parse YYYY-MM-DD and compute calendar days to expiry.
        let expiry = chrono::NaiveDate::parse_from_str(s.expiration.trim(), "%Y-%m-%d").ok()?;
        let expiry = expiry.and_hms_opt(0, 0, 0)?.and_utc();
        let now = Utc::now();
        Some((expiry - now).num_days() as i32)
    })
}

fn is_atm(s: &ScenarioDto, symbol_last: f64) -> bool {
    match s.strike_center {
        None => true,
        Some(_center) if symbol_last <= 0.0 => true,
        Some(center) => (center - symbol_last).abs() / symbol_last <= ATM_TOLERANCE,
    }
}

/// Filtered and sorted scenarios for the Scenarios tab (DTE window, strike width, ATM).
/// Used by both render and by App for scroll/Enter.
pub fn filtered_scenarios(app: &App) -> Vec<ScenarioDto> {
    let scenarios = app
        .snapshot()
        .as_ref()
        .map(|s| s.dto().scenarios.clone())
        .unwrap_or_default();
    let symbol_last: std::collections::HashMap<String, f64> = app
        .snapshot()
        .as_ref()
        .map(|s| {
            s.inner
                .symbols
                .iter()
                .map(|sym| (sym.symbol.clone(), sym.last))
                .collect()
        })
        .unwrap_or_default();
    let (center, half) = (app.scenarios_dte_center, app.scenarios_dte_half_width);
    let dte_min = center - half;
    let dte_max = center + half;

    let mut out: Vec<ScenarioDto> = scenarios
        .into_iter()
        .filter(|s| {
            if let Some(w) = app.scenarios_strike_width_filter {
                if s.strike_width != w as f64 {
                    return false;
                }
            }
            let dte = match days_to_expiry(s) {
                Some(d) => d,
                None => return true,
            };
            if dte < dte_min || dte > dte_max {
                return false;
            }
            let last = symbol_last.get(&s.symbol).copied().unwrap_or(0.0);
            is_atm(s, last)
        })
        .collect();
    out.sort_by(|a, b| {
        a.expiration.cmp(&b.expiration).then_with(|| {
            b.apr_pct
                .partial_cmp(&a.apr_pct)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    });
    out
}

pub fn render_scenarios(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(5), Constraint::Min(5)])
        .split(area);

    let filtered = filtered_scenarios(app);
    let (center, half) = (app.scenarios_dte_center, app.scenarios_dte_half_width);
    let dte_min = center - half;
    let dte_max = center + half;

    let (total, avg_apr, probable_count, max_apr_str) = if filtered.is_empty() {
        (0, "—".to_string(), 0, "—".to_string())
    } else {
        let total = filtered.len();
        let avg_apr = filtered.iter().map(|x| x.apr_pct).sum::<f64>() / total as f64;
        let probable_count = filtered.iter().filter(|x| x.fill_probability > 0.0).count();
        let max_opt = filtered.iter().max_by(|a, b| {
            a.apr_pct
                .partial_cmp(&b.apr_pct)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        let max_apr_str = max_opt
            .map(|s| format!("{:.1}% ({} {} 100)", s.apr_pct, s.symbol, s.expiration))
            .unwrap_or_else(|| "—".to_string());
        (
            total,
            format!("{:.1}%", avg_apr),
            probable_count,
            max_apr_str,
        )
    };

    let width_hint = match app.scenarios_strike_width_filter {
        None => "all".to_string(),
        Some(w) => format!("{}pt", w),
    };
    let summary_text = format!(
        " Total: {}   Avg APR: {}   Probable: {}   Max APR: {}   DTE: {}–{}   Width: {}   [ ] expand/contract  [w] width  Ref: Yield tab for Tbill] ",
        total, avg_apr, probable_count, max_apr_str, dte_min, dte_max, width_hint
    );
    let summary_block = Block::default()
        .title(" Box spread scenarios (per calendar day) ")
        .borders(Borders::ALL);
    f.render_widget(Paragraph::new(summary_text).block(summary_block), chunks[0]);

    let sorted = &filtered;
    let snapshot_scenarios = app
        .snapshot()
        .as_ref()
        .map(|s| s.dto().scenarios.len())
        .unwrap_or(0);

    let header = Row::new([
        Cell::from("Symbol"),
        Cell::from("Expiry"),
        Cell::from(Line::from("DTE").right_aligned()),
        Cell::from(Line::from("Width").right_aligned()),
        Cell::from(Line::from("Center").right_aligned()),
        Cell::from(Line::from("APR%").right_aligned()),
        Cell::from(Line::from("Tbill%").right_aligned()),
        Cell::from(Line::from("vs bps").right_aligned()),
        Cell::from(Line::from("Net Debit").right_aligned()),
        Cell::from(Line::from("ROI%").right_aligned()),
        Cell::from(Line::from("FillProb").right_aligned()),
    ])
    .style(Style::default().add_modifier(Modifier::BOLD | Modifier::UNDERLINED));

    let visible_height = chunks[1].height.saturating_sub(3) as usize; // borders + header
    let scroll = if sorted.is_empty() {
        0
    } else {
        app.scenarios_scroll.min(sorted.len().saturating_sub(1))
    };

    let rows: Vec<Row> = if sorted.is_empty() {
        vec![Row::new(vec![
            Cell::from("No scenarios match the current DTE/width filters"),
            Cell::from("—"),
            Cell::from("—"),
            Cell::from("—"),
            Cell::from("—"),
            Cell::from("—"),
            Cell::from("—"),
            Cell::from("—"),
            Cell::from("—"),
            Cell::from("—"),
            Cell::from("—"),
        ])]
    } else {
        sorted
            .iter()
            .enumerate()
            .map(|(i, s): (usize, &ScenarioDto)| {
                let is_selected = i == scroll;
                let dte = days_to_expiry(s);
                let dte_str = dte
                    .map(|d| d.to_string())
                    .unwrap_or_else(|| "—".to_string());
                let center_str = s
                    .strike_center
                    .map(|c| format!("{:.0}", c))
                    .unwrap_or_else(|| "—".to_string());
                let (tbill_str, vs_bps_str) = match (dte, app.yield_benchmarks.as_ref()) {
                    (Some(d), Some(bench)) => match benchmark_rate_for_dte(d, Some(bench)) {
                        Some(tbill) => {
                            let bps = (s.apr_pct - tbill) * 100.0; // APR and tbill both in %
                            (format!("{:.2}", tbill), format!("{:+.0}", bps))
                        }
                        _ => ("—".to_string(), "—".to_string()),
                    },
                    _ => ("—".to_string(), "—".to_string()),
                };
                let row = Row::new([
                    Cell::from(s.symbol.clone()),
                    Cell::from(s.expiration.clone()),
                    Cell::from(Line::from(dte_str).right_aligned()),
                    Cell::from(Line::from(s.strike_width.to_string()).right_aligned()),
                    Cell::from(Line::from(center_str).right_aligned()),
                    Cell::from(Line::from(format!("{:.2}", s.apr_pct)).right_aligned()),
                    Cell::from(Line::from(tbill_str).right_aligned()),
                    Cell::from(Line::from(vs_bps_str).right_aligned()),
                    Cell::from(Line::from(format!("{:.2}", s.net_debit)).right_aligned()),
                    Cell::from(Line::from(format!("{:.2}", s.roi_pct)).right_aligned()),
                    Cell::from(Line::from(format!("{:.2}", s.fill_probability)).right_aligned()),
                ]);
                if is_selected {
                    row.style(Style::default().add_modifier(Modifier::REVERSED))
                } else {
                    row
                }
            })
            .collect()
    };

    let window: Vec<Row> = rows
        .into_iter()
        .skip(scroll)
        .take(visible_height.max(1))
        .collect();

    let table = Table::new(
        window,
        [
            Constraint::Length(6),
            Constraint::Length(12),
            Constraint::Length(4),
            Constraint::Length(6),
            Constraint::Length(8),
            Constraint::Length(6),
            Constraint::Length(7),
            Constraint::Length(7),
            Constraint::Length(9),
            Constraint::Length(6),
            Constraint::Length(8),
        ],
    )
    .header(header)
    .block(
        Block::default()
            .title(" Scenarios  [↑↓ scroll] [[]/[]] DTE  [w] width  [read-only] ")
            .borders(Borders::ALL),
    );

    f.render_widget(table, chunks[1]);

    if sorted.is_empty() {
        let hint = if snapshot_scenarios == 0 {
            "No scenarios are loaded yet. Wait for a fresh snapshot or publish scenarios from the backend."
        } else {
            "Try ] to widen DTE, [ to narrow less aggressively, or w to clear/change the strike-width filter."
        };
        let hint_area = Rect::new(
            chunks[1].x + 2,
            chunks[1].y + chunks[1].height.saturating_sub(2),
            chunks[1].width.saturating_sub(4),
            1,
        );
        f.render_widget(
            Paragraph::new(hint).style(Style::default().fg(Color::Yellow)),
            hint_area,
        );
    }
}
