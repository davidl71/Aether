//! Yield curve tab: symbol selector, box spread curve table, benchmark table.

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Row, Table},
    Frame,
};

use crate::app::App;
use crate::expiry_buckets::bucket_label;

pub fn render_yield_curve(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(5),
            Constraint::Length(2),
            Constraint::Min(4),
        ])
        .split(area);

    let watchlist = app.watchlist();
    let symbol_line = if watchlist.is_empty() {
        Line::from(Span::styled(
            "No symbols (configure watchlist / strategy.symbols)",
            Style::default().fg(Color::DarkGray),
        ))
    } else {
        let idx = app
            .yield_symbol_index
            .min(watchlist.len().saturating_sub(1));
        let mut spans = vec![Span::raw("Symbol: ")];
        for (i, sym) in watchlist.iter().enumerate() {
            if i > 0 {
                spans.push(Span::raw(" "));
            }
            if i == idx {
                spans.push(Span::styled(
                    format!("[{}]", sym),
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ));
            } else {
                spans.push(Span::styled(
                    sym.as_str(),
                    Style::default().fg(Color::DarkGray),
                ));
            }
        }
        spans.push(Span::raw("  [← →]: change. Ref: "));
        spans.push(Span::styled(
            "boxtrades.com",
            Style::default().fg(Color::Cyan),
        ));
        Line::from(spans)
    };
    f.render_widget(
        Paragraph::new(symbol_line).block(Block::default().title("Yield").borders(Borders::ALL)),
        chunks[0],
    );

    let header = Row::new(["Symbol", "Expiry", "DTE", "Bucket", "Rate %"])
        .style(Style::default().add_modifier(Modifier::BOLD | Modifier::UNDERLINED));
    let symbol = app
        .config
        .watchlist
        .get(
            app.yield_symbol_index
                .min(app.config.watchlist.len().saturating_sub(1)),
        )
        .cloned()
        .unwrap_or_else(|| "—".to_string());
    let (rows, empty_reason): (Vec<Row>, Option<&'static str>) = match &app.yield_curve {
        Some(curve) if curve.point_count > 0 => (
            curve
                .points
                .iter()
                .map(|p| {
                    Row::new([
                        p.symbol.clone(),
                        p.expiry.clone(),
                        p.days_to_expiry.to_string(),
                        bucket_label(p.days_to_expiry).to_string(),
                        format!("{:.2}", p.mid_rate * 100.0),
                    ])
                })
                .collect(),
            None,
        ),
        Some(_) => (
            vec![Row::new([
                symbol.clone(),
                "—".into(),
                "—".into(),
                "—".into(),
                "No data".into(),
            ])],
            Some("Backend returned 0 points (need yield_curve.{symbol} in NATS KV or run yield curve writer)"),
        ),
        None if app.yield_error.is_some() => (
            vec![Row::new([
                symbol.clone(),
                "—".into(),
                "—".into(),
                "—".into(),
                app.yield_error
                    .as_deref()
                    .map(|e| super::truncate_detail(e, 20))
                    .unwrap_or_else(|| "error".to_string())
                    .into(),
            ])],
            None,
        ),
        None => (
            vec![Row::new([
                symbol.clone(),
                "—".into(),
                "—".into(),
                "—".into(),
                "—".into(),
            ])],
            Some("Request curve to load from api.finance_rates.build_curve"),
        ),
    };
    let title = if app.yield_error.is_some() {
        "Box spread curve (request failed)"
    } else if empty_reason.is_some() {
        "Box spread curve (empty)"
    } else {
        "Box spread curve (api.finance_rates.build_curve)"
    };
    let table = Table::new(
        rows,
        [
            Constraint::Length(10),
            Constraint::Length(12),
            Constraint::Length(6),
            Constraint::Length(16),
            Constraint::Length(8),
        ],
    )
    .header(header)
    .block(Block::default().title(title).borders(Borders::ALL));
    f.render_widget(table, chunks[1]);

    let mut bench_note_spans = vec![
        Span::raw("Benchmark: "),
        Span::styled(
            "api.finance_rates.benchmarks",
            Style::default().fg(Color::Yellow),
        ),
        Span::raw(" (FRED). "),
    ];
    if let Some(ref e) = app.yield_error {
        bench_note_spans.push(Span::styled(
            format!("Curve fetch failed: {}", super::truncate_detail(e, 96)),
            Style::default().fg(Color::Red),
        ));
    } else if let Some(reason) = empty_reason {
        bench_note_spans.push(Span::styled(
            reason.replace("{symbol}", &symbol),
            Style::default().fg(Color::Yellow),
        ));
    } else {
        bench_note_spans.push(Span::raw("See "));
        bench_note_spans.push(Span::styled(
            "NATS_API.md",
            Style::default().fg(Color::Cyan),
        ));
        bench_note_spans.push(Span::raw(", TUI_LEGACY_DESIGN_LEARNINGS."));
    }
    f.render_widget(Paragraph::new(Line::from(bench_note_spans)), chunks[2]);

    let bench_header = Row::new(["Tenor", "Rate %", "Source"])
        .style(Style::default().add_modifier(Modifier::BOLD | Modifier::UNDERLINED));
    let bench_rows: Vec<Row> = match &app.yield_benchmarks {
        Some(b) => {
            let mut rows = Vec::new();
            // FRED returns rates already in percent (e.g. 4.5 = 4.5%); display as-is for "Rate %"
            if let Some(rate) = b.sofr.overnight.rate {
                rows.push(Row::new([
                    "SOFR overnight".into(),
                    format!("{:.2}", rate),
                    b.sofr
                        .overnight
                        .timestamp
                        .as_deref()
                        .unwrap_or("SOFR")
                        .to_string(),
                ]));
            }
            for r in &b.sofr.term_rates {
                rows.push(Row::new([
                    r.tenor.clone(),
                    format!("{:.2}", r.rate),
                    r.source.clone(),
                ]));
            }
            for r in &b.treasury.rates {
                rows.push(Row::new([
                    r.tenor.clone(),
                    format!("{:.2}", r.rate),
                    r.source.clone(),
                ]));
            }
            if rows.is_empty() {
                vec![Row::new(["—", "—", "—"])]
            } else {
                rows
            }
        }
        None => vec![Row::new(["—", "—", "—"])],
    };
    let bench_table = Table::new(
        bench_rows,
        [
            Constraint::Length(16),
            Constraint::Length(8),
            Constraint::Length(24),
        ],
    )
    .header(bench_header)
    .block(
        Block::default()
            .title("Benchmark yield curve (Treasury / SOFR)")
            .borders(Borders::ALL),
    );
    f.render_widget(bench_table, chunks[3]);
}
