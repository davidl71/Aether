//! Charts tab: displays candlestick chart for selected symbol.

use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::app::App;
use crate::ui::candlestick::{render_candlestick, Candle};

fn demo_candles() -> Vec<Candle> {
    vec![
        Candle::new(100.0, 102.0, 99.0, 101.5),
        Candle::new(101.5, 103.5, 100.5, 102.0),
        Candle::new(102.0, 104.0, 101.0, 103.5),
        Candle::new(103.5, 105.0, 103.0, 104.5),
        Candle::new(104.5, 106.0, 104.0, 105.5),
        Candle::new(105.5, 106.5, 104.5, 105.0),
        Candle::new(105.0, 107.0, 104.5, 106.5),
        Candle::new(106.5, 108.0, 106.0, 107.5),
        Candle::new(107.5, 109.0, 107.0, 108.5),
        Candle::new(108.5, 110.0, 108.0, 109.0),
    ]
}

pub fn render_charts(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title(" Charts ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));
    let inner = block.inner(area);
    f.render_widget(block, area);

    if app.symbol_for_chart.is_empty() {
        let hint = Paragraph::new(vec![
            Line::from(""),
            Line::from("  No symbol selected for charting."),
            Line::from(""),
            Line::from(vec![
                ratatui::text::Span::raw("  Type a symbol and press "),
                ratatui::text::Span::styled("Enter", Style::default().add_modifier(Modifier::BOLD)),
                ratatui::text::Span::raw(" to chart."),
            ]),
            Line::from(""),
        ]);
        f.render_widget(hint, inner);
        return;
    }

    let symbol = &app.symbol_for_chart;
    let candles = demo_candles();

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
        ratatui::text::Span::styled("(demo data)", Style::default().fg(Color::DarkGray)),
    ]));
    f.render_widget(hint, hint_area);
}
