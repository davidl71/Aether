//! Charts tab: displays candlestick chart for selected symbol with search.

use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use crate::app::App;
use crate::ui::candlestick::{render_candlestick, Candle};

const SEARCH_RESULTS: &[&str] = &[
    "SPX",
    "SPY",
    "SPXW",
    "SPY241220C600",
    "QQQ",
    "QQQ241220C500",
    "NDX",
    "AAPL",
    "MSFT",
    "GOOG",
    "AMZN",
    "TSLA",
    "META",
    "NVDA",
    "AMD",
    "INTC",
    "JPM",
    "GS",
    "BAC",
    "XSP",
    "XSP241220C500",
];

pub fn update_search_results(app: &mut App) {
    let now_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);

    if now_ms.saturating_sub(app.chart_search_last_search_ms) < app.chart_search_debounce_ms {
        return;
    }
    app.chart_search_last_search_ms = now_ms;

    let query = app.chart_search_input.to_uppercase();
    app.chart_search_results.clear();

    if query.is_empty() {
        for symbol in app.chart_search_history.iter().take(5) {
            app.chart_search_results.push(symbol.clone());
        }
    } else {
        for symbol in SEARCH_RESULTS.iter() {
            if symbol.contains(&query) || symbol.to_uppercase().starts_with(&query) {
                app.chart_search_results.push(symbol.to_string());
                if app.chart_search_results.len() >= 10 {
                    break;
                }
            }
        }
    }

    app.chart_search_selected = 0;
}

fn demo_candles(symbol: &str) -> Vec<Candle> {
    let base_price = match symbol.to_uppercase().as_str() {
        "SPX" | "SPY" => 500.0,
        "QQQ" => 430.0,
        "NDX" => 18000.0,
        "AAPL" => 180.0,
        "MSFT" => 420.0,
        "GOOG" => 175.0,
        "TSLA" => 250.0,
        _ => 100.0 + (symbol.len() as f64 * 10.0),
    };

    let mut candles = Vec::new();
    let mut price = base_price;
    for i in 0..10 {
        let open = price;
        let change = (i32::from(i) % 5 - 2) as f64 * 2.0;
        let close = (open + change).max(1.0);
        let high = open.max(close) + (i32::from(i) % 3) as f64;
        let low = open.min(close) - (i32::from(i) % 2) as f64;
        candles.push(Candle::new(open, high, low, close));
        price = close;
    }
    candles
}

pub fn render_charts(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title(" Charts ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));
    let inner = block.inner(area);
    f.render_widget(block, area);

    if app.chart_search_visible {
        render_search_ui(f, app, inner);
        return;
    }

    if app.symbol_for_chart.is_empty() {
        let hint = Paragraph::new(vec![
            Line::from(""),
            Line::from("  No symbol selected for charting."),
            Line::from(""),
            Line::from(vec![
                ratatui::text::Span::raw("  Press "),
                ratatui::text::Span::styled("/", Style::default().add_modifier(Modifier::BOLD)),
                ratatui::text::Span::raw(" to search symbols, or type directly."),
            ]),
            Line::from(""),
        ]);
        f.render_widget(hint, inner);
        return;
    }

    let symbol = &app.symbol_for_chart;
    let candles = demo_candles(symbol);

    let chart_height = inner.height.saturating_sub(4);
    if chart_height > 3 {
        let chart_area = Rect::new(inner.x, inner.y, inner.width, chart_height);
        render_candlestick(f, &candles, symbol, chart_area);
    }

    let hint_area = Rect::new(
        inner.x,
        inner.y + chart_height,
        inner.width,
        area.height.saturating_sub(chart_height + 2),
    );
    let hint = Paragraph::new(Line::from(vec![
        ratatui::text::Span::raw("Symbol: "),
        ratatui::text::Span::styled(
            format!(" {} ", symbol),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        ratatui::text::Span::styled("/: search  ", Style::default().fg(Color::DarkGray)),
        ratatui::text::Span::styled("←→: scroll", Style::default().fg(Color::DarkGray)),
    ]));
    f.render_widget(hint, hint_area);
}

fn render_search_ui(f: &mut Frame, app: &App, area: Rect) {
    let search_height = 4u16.min(area.height);
    let list_height = (area.height - search_height).min(10);

    let input_area = Rect::new(area.x, area.y, area.width, search_height);
    let list_area = Rect::new(area.x, area.y + search_height, area.width, list_height);

    let search_block = Block::default()
        .title(" Search Symbol ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));
    let input_inner = search_block.inner(input_area);
    f.render_widget(search_block, input_area);

    let input_text = if app.chart_search_input.is_empty() {
        if app.chart_search_results.is_empty() {
            String::from("Type to search...")
        } else {
            String::new()
        }
    } else {
        app.chart_search_input.clone()
    };

    let input_para = Paragraph::new(vec![Line::from(vec![
        ratatui::text::Span::raw("> "),
        ratatui::text::Span::styled(&input_text, Style::default().fg(Color::White)),
    ])]);
    f.render_widget(input_para, input_inner);

    if !app.chart_search_results.is_empty() && list_height > 0 {
        let list_items: Vec<ListItem> = app
            .chart_search_results
            .iter()
            .enumerate()
            .map(|(idx, symbol)| {
                let style = if idx == app.chart_search_selected {
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                };
                ListItem::new(Line::from(vec![ratatui::text::Span::styled(
                    format!("  {}", symbol),
                    style,
                )]))
            })
            .collect();

        let list = List::new(list_items)
            .block(
                Block::default()
                    .borders(Borders::LEFT | Borders::RIGHT | Borders::BOTTOM)
                    .border_style(Style::default().fg(Color::DarkGray)),
            )
            .highlight_style(Style::default().bg(Color::Blue).fg(Color::White));

        f.render_widget(list, list_area);
    }

    let hint_para = Paragraph::new(Line::from(vec![
        ratatui::text::Span::styled("↑↓", Style::default().add_modifier(Modifier::BOLD)),
        ratatui::text::Span::raw(" navigate  "),
        ratatui::text::Span::styled("Enter", Style::default().add_modifier(Modifier::BOLD)),
        ratatui::text::Span::raw(" select  "),
        ratatui::text::Span::styled("Esc", Style::default().add_modifier(Modifier::BOLD)),
        ratatui::text::Span::raw(" close"),
    ]));
    let hint_area = Rect::new(
        area.x,
        area.y + search_height + list_height,
        area.width,
        area.height.saturating_sub(search_height + list_height),
    );
    f.render_widget(hint_para, hint_area);
}
