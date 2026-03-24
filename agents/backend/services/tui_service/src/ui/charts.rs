//! Charts tab: displays candlestick chart for selected symbol with search.

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use crate::app::App;
use crate::ui::candlestick::{
    generate_synthetic_candles, render_candlestick, render_volume, Candle,
};

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

const DEFAULT_STRIKE_WIDTHS: &[u32] = &[25, 50, 100, 200];

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

fn is_option_symbol(symbol: &str) -> bool {
    symbol.starts_with("SPX")
        || symbol.starts_with("XSP")
        || symbol.starts_with("NDX")
        || symbol.starts_with("SPXW")
        || symbol.starts_with("XSPW")
}

fn get_expiry_labels(curve: &api::finance_rates::CurveResponse) -> Vec<(i32, String)> {
    let mut labels: Vec<(i32, String)> = curve
        .points
        .iter()
        .map(|p| {
            let label = if p.days_to_expiry <= 30 {
                "30D".to_string()
            } else if p.days_to_expiry <= 60 {
                "60D".to_string()
            } else if p.days_to_expiry <= 90 {
                "90D".to_string()
            } else if p.days_to_expiry <= 180 {
                "6M".to_string()
            } else {
                format!("{}D", p.days_to_expiry)
            };
            (p.days_to_expiry, label)
        })
        .collect();
    labels.sort_by_key(|(dte, _)| *dte);
    labels.dedup_by_key(|(dte, _)| *dte);
    labels
}

fn get_candles_for_symbol(app: &App, symbol: &str) -> Vec<Candle> {
    if is_option_symbol(symbol) {
        if let Some(ref curve) = app.yield_curve {
            return generate_synthetic_candles(curve, 20, 50.0);
        }
    }

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
    for i in 0..20 {
        let open = price;
        let change = (i32::from(i) % 5 - 2) as f64 * 2.0;
        let close = (open + change).max(1.0);
        let high = open.max(close) + (i32::from(i) % 3) as f64;
        let low = open.min(close) - (i32::from(i) % 2) as f64;
        let volume = Some(10000.0 + (i as f64 * 1000.0));
        candles.push(Candle {
            open,
            high,
            low,
            close,
            volume,
        });
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
                Span::raw("  Press "),
                Span::styled("/", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to search symbols, or type directly."),
            ]),
            Line::from(""),
        ]);
        f.render_widget(hint, inner);
        return;
    }

    let symbol = &app.symbol_for_chart;
    let is_option = is_option_symbol(symbol);

    let (pill_area, chart_area, volume_area, info_area) = if is_option && app.yield_curve.is_some()
    {
        let constraints = if app
            .yield_curve
            .as_ref()
            .map(|c| c.points.len())
            .unwrap_or(0)
            > 3
        {
            [
                Constraint::Length(3),
                Constraint::Min(8),
                Constraint::Length(3),
                Constraint::Length(4),
            ]
        } else {
            [
                Constraint::Length(2),
                Constraint::Min(8),
                Constraint::Length(3),
                Constraint::Length(4),
            ]
        };
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(constraints)
            .split(inner);
        (Some(chunks[0]), chunks[1], Some(chunks[2]), Some(chunks[3]))
    } else {
        let chart_height = inner.height.saturating_sub(8);
        let vol_height = inner.height.saturating_sub(chart_height);
        (
            None,
            Rect::new(inner.x, inner.y, inner.width, chart_height),
            Some(Rect::new(
                inner.x,
                inner.y + chart_height,
                inner.width,
                vol_height,
            )),
            None,
        )
    };

    if let Some(pa) = pill_area {
        render_pill_boxes(f, app, pa);
    }

    let candles = get_candles_for_symbol(app, symbol);
    render_candlestick(f, &candles, symbol, chart_area);

    if let Some(va) = volume_area {
        render_volume(f, &candles, va);
    }

    if let Some(ia) = info_area {
        render_chart_info(f, app, symbol, ia);
    } else {
        render_hint_bar(f, symbol, false);
    }
}

fn render_pill_boxes(f: &mut Frame, app: &App, area: Rect) {
    let curve = match &app.yield_curve {
        Some(c) => c,
        None => return,
    };

    let expiry_labels = get_expiry_labels(curve);
    let strike_widths = DEFAULT_STRIKE_WIDTHS;

    let row_height = 1u16;
    let expiry_row_height = if expiry_labels.len() > 3 { 2 } else { 1 };

    let expiry_area = Rect::new(area.x, area.y, area.width, expiry_row_height * row_height);
    let strike_area = Rect::new(area.x, area.y + expiry_row_height, area.width, row_height);

    render_expiry_pills(f, app, curve, &expiry_labels, expiry_area);
    render_strike_width_pills(f, app, strike_widths, strike_area);
}

fn render_expiry_pills(
    f: &mut Frame,
    app: &App,
    curve: &api::finance_rates::CurveResponse,
    labels: &[(i32, String)],
    area: Rect,
) {
    let is_active = app.chart_pill_row == 0;

    let pill_style = |selected: bool, active: bool| {
        if selected {
            if active {
                Style::default()
                    .bg(Color::Yellow)
                    .fg(Color::Black)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
                    .bg(Color::Blue)
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD)
            }
        } else if active {
            Style::default().bg(Color::DarkGray).fg(Color::Yellow)
        } else {
            Style::default().bg(Color::DarkGray).fg(Color::White)
        }
    };

    let mut x = area.x + 1;
    let mut y = area.y;

    let title = Span::styled("Expiry: ", Style::default().fg(Color::DarkGray));
    f.render_widget(Paragraph::new(Line::from(title)), Rect::new(x, y, 8, 1));
    x += 8;

    let max_idx = labels.len().saturating_sub(1);
    let selected_idx = app.chart_expiry_index.min(max_idx);

    for (idx, (dte, label)) in labels.iter().enumerate() {
        let width = (label.len() as u16) + 2;
        if x + width > area.x + area.width - 1 {
            y += 1;
            x = area.x + 1;
        }

        let is_selected = idx == selected_idx;
        let pill = Block::default()
            .borders(Borders::NONE)
            .style(pill_style(is_selected, is_active));

        let pill_area = Rect::new(x, y, width, 1);
        f.render_widget(pill.title(format!(" {} ", label)), pill_area);

        x += width + 1;
    }
}

fn render_strike_width_pills(f: &mut Frame, app: &App, widths: &[u32], area: Rect) {
    let is_active = app.chart_pill_row == 1;

    let pill_style = |selected: bool, active: bool| {
        if selected {
            if active {
                Style::default()
                    .bg(Color::Green)
                    .fg(Color::Black)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
                    .bg(Color::Green)
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD)
            }
        } else if active {
            Style::default().bg(Color::DarkGray).fg(Color::Yellow)
        } else {
            Style::default().bg(Color::DarkGray).fg(Color::White)
        }
    };

    let mut x = area.x + 1;

    let title = Span::styled("Width: ", Style::default().fg(Color::DarkGray));
    f.render_widget(
        Paragraph::new(Line::from(title)),
        Rect::new(x, area.y, 8, 1),
    );
    x += 8;

    for (idx, &width) in widths.iter().enumerate() {
        let label = format!("{}", width);
        let pill_width = (label.len() as u16) + 2;
        let is_selected = idx == app.chart_strike_index.min(widths.len() - 1);

        let pill = Block::default()
            .borders(Borders::NONE)
            .style(pill_style(is_selected, is_active));

        let pill_area = Rect::new(x, area.y, pill_width, 1);
        f.render_widget(pill.title(format!(" {} ", label)), pill_area);

        x += pill_width + 1;
    }

    let custom_label = " custom ";
    let custom_width = (custom_label.len() as u16) + 2;
    if x + custom_width < area.x + area.width - 1 {
        let custom_style = Style::default().bg(Color::DarkGray).fg(Color::Yellow);
        let custom_pill = Block::default().borders(Borders::NONE).style(custom_style);
        f.render_widget(
            custom_pill.title(custom_label),
            Rect::new(x, area.y, custom_width, 1),
        );
    }
}

fn render_chart_info(f: &mut Frame, app: &App, symbol: &str, area: Rect) {
    let curve = match &app.yield_curve {
        Some(c) => c,
        None => {
            render_hint_bar(f, symbol, true);
            return;
        }
    };

    let point = curve
        .points
        .iter()
        .find(|p| p.days_to_expiry == 30)
        .or_else(|| curve.points.first());

    let info_lines = if let Some(p) = point {
        let rate_str = format!("{:.2}%", p.mid_rate * 100.0);
        let buy_str = format!("{:.2}%", p.buy_implied_rate * 100.0);
        let sell_str = format!("{:.2}%", p.sell_implied_rate * 100.0);
        let width_str = format!("{:.0}", p.strike_width);

        let sym_style = Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD);
        let rate_style = Style::default()
            .fg(Color::Green)
            .add_modifier(Modifier::BOLD);
        let muted_style = Style::default().fg(Color::DarkGray);

        vec![
            Line::from(vec![
                Span::raw("Symbol: "),
                Span::styled(format!(" {}", symbol), sym_style),
            ]),
            Line::from(vec![
                Span::raw("Rate: "),
                Span::styled(rate_str, rate_style),
                Span::styled(
                    format!(" (buy: {} sell: {})", buy_str, sell_str),
                    muted_style,
                ),
            ]),
            Line::from(vec![
                Span::raw("Width: "),
                Span::styled(width_str, rate_style),
                Span::styled(" pts  ", muted_style),
            ]),
        ]
    } else {
        render_hint_bar(f, symbol, true);
        return;
    };

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .split(area);

    for (i, line) in info_lines.iter().enumerate() {
        if i < layout.len() {
            f.render_widget(Paragraph::new(line.clone()), layout[i]);
        }
    }

    let hint_style = Style::default().fg(Color::DarkGray);
    let hint = Line::from(vec![
        Span::styled("/", hint_style),
        Span::raw(" search  "),
        Span::styled("←→", hint_style),
        Span::raw(" expiry  "),
        Span::styled("↑↓", hint_style),
        Span::raw(" width  "),
        Span::styled("e", hint_style),
        Span::raw(" edit"),
    ]);
    f.render_widget(Paragraph::new(hint), layout[2]);
}

fn render_hint_bar(f: &mut Frame, symbol: &str, is_option: bool) {
    let hint_style = if is_option {
        Style::default().fg(Color::DarkGray)
    } else {
        Style::default().fg(Color::White)
    };

    let hint = Line::from(vec![
        Span::raw("Symbol: "),
        Span::styled(
            format!(" {} ", symbol),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("/: search  ", hint_style),
        Span::styled("←→: scroll", hint_style),
    ]);
    let para = Paragraph::new(hint);
    f.render_widget(para, Rect::new(0, 0, 80, 1));
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
