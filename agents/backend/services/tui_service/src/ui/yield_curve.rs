//! Yield curve tab: side-by-side comparison table (box spreads vs benchmarks) + detail view.

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
    Frame,
};

use api::finance_rates::{BenchmarkRateResponse, CurveResponse, RatePointResponse};

use crate::app::App;
use crate::expiry_buckets::bucket_label;

/// Fixed DTE buckets for the comparison table rows.
const COMPARISON_DTES: &[(i32, &str)] = &[
    (30, "1M"),
    (91, "3M"),
    (182, "6M"),
    (365, "1Y"),
    (730, "2Y"),
];

/// Find the curve point with DTE closest to `target`.
fn closest_point(curve: &CurveResponse, target: i32) -> Option<&RatePointResponse> {
    curve
        .points
        .iter()
        .min_by_key(|p| (p.days_to_expiry - target).abs())
}

/// Look up a rate by exact tenor label.
fn rate_by_tenor<'a>(rates: &'a [BenchmarkRateResponse], tenor: &str) -> Option<f64> {
    rates.iter().find(|r| r.tenor == tenor).map(|r| r.rate)
}

/// T-bill tenor closest to a DTE (short end of Treasury curve).
fn tbill_tenor(dte: i32) -> &'static str {
    if dte <= 45 {
        "1M"
    } else if dte <= 120 {
        "3M"
    } else if dte <= 270 {
        "6M"
    } else {
        "1Y"
    }
}

/// Format rate as "4.82" (already in percent from FRED/Yahoo).
fn fmt_rate(rate: f64) -> String {
    format!("{:.2}", rate)
}

fn current_yield_symbol(app: &App) -> String {
    let watchlist = app.watchlist();
    watchlist
        .get(
            app.yield_symbol_index
                .min(watchlist.len().saturating_sub(1)),
        )
        .cloned()
        .unwrap_or_else(|| "—".to_string())
}

pub fn render_yield_curve(f: &mut Frame, app: &App, area: Rect) {
    render_yield_curve_panel(f, app, area);
}

pub fn render_yield_curve_panel(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(8),    // comparison table
            Constraint::Length(1), // hint + task status
            Constraint::Min(5),    // detail table (selected column/symbol)
        ])
        .split(area);

    render_yield_curve_comparison_table(f, app, chunks[0]);
    render_yield_curve_hint(f, app, chunks[1]);
    let symbol = current_yield_symbol(app);
    render_yield_curve_detail_table(f, app, &symbol, chunks[2]);
}

pub fn render_yield_curve_comparison_table(f: &mut Frame, app: &App, area: Rect) {
    let watchlist = app.watchlist();
    // Cap at 5 watchlist symbols.
    let box_symbols: Vec<&str> = watchlist.iter().take(5).map(String::as_str).collect();
    let n = box_symbols.len();

    // Build header: DTE + (rate%, price$) per symbol + T-Bill, 10Y, 30Y, SOFR
    let mut header_cells: Vec<Cell> = vec![Cell::from("DTE")
        .style(Style::default().add_modifier(Modifier::BOLD | Modifier::UNDERLINED))];
    for sym in &box_symbols {
        header_cells.push(
            Cell::from(format!("{} %", sym))
                .style(Style::default().add_modifier(Modifier::BOLD | Modifier::UNDERLINED)),
        );
        header_cells.push(
            Cell::from(format!("{} bid/ask$", sym))
                .style(Style::default().add_modifier(Modifier::BOLD | Modifier::UNDERLINED)),
        );
    }
    for label in ["T-Bill", "10Y", "30Y", "SOFR", "SHIR"] {
        header_cells.push(
            Cell::from(label)
                .style(Style::default().add_modifier(Modifier::BOLD | Modifier::UNDERLINED)),
        );
    }
    let header = Row::new(header_cells);

    // Build one row per DTE bucket
    let mut rows: Vec<Row> = Vec::new();
    for &(dte, label) in COMPARISON_DTES {
        let mut cells: Vec<Cell> = vec![Cell::from(label)];

        // Box spread columns
        for sym in &box_symbols {
            if let Some(curve) = app.yield_curves_all.get(*sym) {
                if let Some(p) = closest_point(curve, dte) {
                    let rate_pct = p.mid_rate * 100.0;
                    // Color rate by comparison to T-bill at same tenor
                    let tbill_rate = app
                        .yield_benchmarks
                        .as_ref()
                        .and_then(|b| rate_by_tenor(&b.treasury.rates, tbill_tenor(dte)));
                    let rate_color = match tbill_rate {
                        Some(tb) if rate_pct >= tb => Color::Green,
                        Some(_) => Color::Yellow,
                        None => Color::White,
                    };
                    cells.push(
                        Cell::from(format!("{:.2}", rate_pct))
                            .style(Style::default().fg(rate_color)),
                    );
                    // Price column: for TWS (live) show bid/mid/ask; others show ask (net_debit).
                    let is_live = p
                        .data_source
                        .as_deref()
                        .map(|s| s.to_lowercase() == "tws")
                        .unwrap_or(false);
                    let price_str = if is_live {
                        let bid = p.net_credit; // sell-box proceeds = bid
                        let ask = p.net_debit; // buy-box cost = ask
                        let mid = (bid + ask) / 2.0;
                        format!("{:.1}/{:.1}/{:.1}", bid, mid, ask)
                    } else {
                        // For model-derived sources: show ask (entry price to buy box).
                        // Fall back to deriving from rate if net_debit is 0.
                        let ask = if p.net_debit > 0.01 {
                            p.net_debit
                        } else {
                            p.strike_width * (1.0 - p.mid_rate * (p.days_to_expiry as f64 / 365.0))
                        };
                        format!("{:.1}", ask)
                    };
                    cells.push(Cell::from(price_str));
                } else {
                    cells.push(Cell::from("—"));
                    cells.push(Cell::from("—"));
                }
            } else {
                cells.push(Cell::from("…"));
                cells.push(Cell::from("…"));
            }
        }

        // Benchmark columns
        if let Some(ref b) = app.yield_benchmarks {
            // T-Bill: closest short-tenor Treasury
            let tbill = rate_by_tenor(&b.treasury.rates, tbill_tenor(dte));
            cells.push(Cell::from(
                tbill.map(fmt_rate).unwrap_or_else(|| "—".into()),
            ));
            // 10Y T-Note
            let note10 = rate_by_tenor(&b.treasury.rates, "10Y");
            cells.push(Cell::from(
                note10.map(fmt_rate).unwrap_or_else(|| "—".into()),
            ));
            // 30Y Bond
            let bond30 = rate_by_tenor(&b.treasury.rates, "30Y");
            cells.push(Cell::from(
                bond30.map(fmt_rate).unwrap_or_else(|| "—".into()),
            ));
            // SOFR overnight
            let sofr = b.sofr.overnight.rate;
            cells.push(Cell::from(sofr.map(fmt_rate).unwrap_or_else(|| "—".into())));
            // SHIR (Israeli overnight rate, from Bank of Israel)
            cells.push(Cell::from(
                b.shir.map(fmt_rate).unwrap_or_else(|| "—".into()),
            ));
        } else {
            for _ in 0..5 {
                cells.push(Cell::from("…"));
            }
        }

        rows.push(Row::new(cells));
    }

    // Column constraints: DTE(5) + per-symbol rate(7)+price(15 for bid/mid/ask) + 4×benchmark(7)
    let mut widths: Vec<Constraint> = vec![Constraint::Length(5)];
    for _ in 0..n {
        widths.push(Constraint::Length(7)); // rate%
        widths.push(Constraint::Length(15)); // price$ (bid/mid/ask for TWS, ask otherwise)
    }
    for _ in 0..5 {
        widths.push(Constraint::Length(7)); // benchmarks (T-Bill, 10Y, 30Y, SOFR, SHIR)
    }

    let status = if app.yield_refresh_pending {
        " (fetching…)"
    } else if app.yield_error.is_some() {
        " (error)"
    } else if app.yield_curves_all.is_empty() {
        " (no data yet)"
    } else {
        ""
    };
    let title = format!(
        "Yield comparison  rate% / price$ (TWS: bid/mid/ask){}",
        status
    );

    let table = Table::new(rows, widths)
        .header(header)
        .block(Block::default().title(title).borders(Borders::ALL));
    f.render_widget(table, area);
}

pub fn render_yield_curve_hint(f: &mut Frame, app: &App, area: Rect) {
    let symbol = current_yield_symbol(app);

    // Build hint line: controls + task status + last refresh age
    let task_status = if app.yield_refresh_pending {
        " ⟳ refreshing…".to_string()
    } else if let Some(task) = app.yield_tasks.front() {
        use crate::app::RefreshTaskStatus;
        match &task.status {
            RefreshTaskStatus::Done => {
                let secs = (chrono::Utc::now() - task.triggered_at).num_seconds();
                format!(" ✓ done {}s ago", secs)
            }
            RefreshTaskStatus::Error(e) => format!(" ✗ {}", super::truncate_detail(e, 30)),
            RefreshTaskStatus::Pending => " ⟳ pending…".to_string(),
        }
    } else if let Some(dt) = app.yield_last_refreshed.get(&symbol) {
        let secs = (chrono::Utc::now() - *dt).num_seconds();
        format!(" last: {}s ago", secs)
    } else {
        " (waiting for writer)".to_string()
    };

    let hint = format!(
        " [← →] symbol  [↑↓] detail  [r] refresh  [Enter] legs  ▶ {}{}",
        symbol, task_status
    );
    f.render_widget(
        Paragraph::new(hint).style(Style::default().fg(Color::DarkGray)),
        area,
    );
}

pub fn render_yield_curve_detail_table(f: &mut Frame, app: &App, symbol: &str, area: Rect) {
    let header = Row::new([
        "Expiry", "DTE", "Bucket", "Rate%", "Buy%", "Sell%", "Debit", "Credit",
    ])
    .style(Style::default().add_modifier(Modifier::BOLD | Modifier::UNDERLINED));
    let visible_height = area.height.saturating_sub(3).max(1) as usize;

    let (rows, empty_reason): (Vec<Row>, Option<&'static str>) = match &app.yield_curve {
        Some(curve) if curve.point_count > 0 => {
            let scroll = app
                .yield_curve_scroll
                .min(curve.point_count.saturating_sub(1));
            let viewport = if curve.point_count <= visible_height {
                0
            } else {
                scroll
                    .saturating_sub(visible_height / 2)
                    .min(curve.point_count - visible_height)
            };
            let rows = curve
                .points
                .iter()
                .enumerate()
                .skip(viewport)
                .take(visible_height)
                .map(|(i, p)| {
                    let row = Row::new([
                        p.expiry.clone(),
                        p.days_to_expiry.to_string(),
                        bucket_label(p.days_to_expiry).to_string(),
                        format!("{:.2}", p.mid_rate * 100.0),
                        format!("{:.2}", p.buy_implied_rate * 100.0),
                        format!("{:.2}", p.sell_implied_rate * 100.0),
                        format!("{:.1}", p.net_debit),
                        format!("{:.1}", p.net_credit),
                    ]);
                    if i == scroll {
                        row.style(Style::default().add_modifier(Modifier::REVERSED))
                    } else {
                        row
                    }
                })
                .collect();
            (rows, None)
        }
        Some(_) => (
            vec![Row::new(["—", "—", "—", "—", "—", "—", "no data", ""])],
            Some("0 points returned — check yield curve writer / NATS KV"),
        ),
        None if app.yield_error.is_some() => {
            let err = super::truncate_detail(app.yield_error.as_deref().unwrap_or("error"), 30);
            (
                vec![Row::new(vec![
                    "—".to_string(),
                    "—".to_string(),
                    "—".to_string(),
                    "—".to_string(),
                    "—".to_string(),
                    "—".to_string(),
                    err,
                    "".to_string(),
                ])],
                None,
            )
        }
        None => (
            vec![Row::new(["—", "—", "—", "—", "—", "—", "—", ""])],
            Some("Select symbol and wait for fetch"),
        ),
    };

    let data_source = app
        .yield_curve
        .as_ref()
        .and_then(|c| c.points.first())
        .and_then(|p| p.data_source.as_deref())
        .unwrap_or("—");

    let title = if let Some(reason) = empty_reason {
        format!("{} detail — {}", symbol, reason)
    } else if app.yield_error.is_some() {
        format!("{} detail (fetch failed)", symbol)
    } else {
        format!(
            "{} detail  [↑↓] navigate  [Enter] legs popup  source:{}",
            symbol, data_source
        )
    };

    let table_block = Block::default().title(title).borders(Borders::ALL);
    let inner = table_block.inner(area);
    f.render_widget(table_block, area);

    let table = Table::new(
        rows,
        [
            Constraint::Length(12), // Expiry
            Constraint::Length(5),  // DTE
            Constraint::Length(12), // Bucket
            Constraint::Length(7),  // Rate%
            Constraint::Length(7),  // Buy%
            Constraint::Length(7),  // Sell%
            Constraint::Length(8),  // Debit
            Constraint::Length(8),  // Credit
        ],
    )
    .header(header);
    f.render_widget(table, inner);
}
