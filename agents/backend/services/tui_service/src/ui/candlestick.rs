use ratatui::{
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Borders, Widget},
    Frame,
};

#[derive(Debug, Clone)]
pub struct Candle {
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: Option<f64>,
}

impl Candle {
    pub fn bullish(&self) -> bool {
        self.close >= self.open
    }
}

pub struct CandlestickChart {
    candles: Vec<Candle>,
    symbol: String,
}

impl CandlestickChart {
    pub fn new(symbol: String, candles: Vec<Candle>) -> Self {
        Self { candles, symbol }
    }
}

impl Widget for CandlestickChart {
    fn render(self, area: Rect, buf: &mut ratatui::buffer::Buffer) {
        if area.width < 20 || area.height < 5 {
            return;
        }

        let block = Block::default()
            .title(format!(" {} ", self.symbol))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));
        let inner = block.inner(area);
        block.render(area, buf);
        if inner.is_empty() {
            return;
        }

        let chart_width = inner.width as usize;
        let visible_candles = chart_width / 6;

        if self.candles.is_empty() {
            let msg = "No data";
            let x = inner.x + inner.width.saturating_sub(msg.len() as u16) / 2;
            buf.set_string(
                x,
                inner.y + inner.height / 2,
                msg,
                Style::default().fg(Color::DarkGray),
            );
            return;
        }

        let start = self.candles.len().saturating_sub(visible_candles.max(1));
        let end = (start + visible_candles).min(self.candles.len());
        let visible: Vec<_> = self.candles[start..end].iter().collect();

        if visible.is_empty() {
            return;
        }

        let min_price = visible.iter().map(|c| c.low).fold(f64::INFINITY, f64::min);
        let max_price = visible.iter().map(|c| c.high).fold(0.0, f64::max);
        let price_range = (max_price - min_price).max(0.001);

        let price_to_y = |price: f64| -> u16 {
            let pct = (price - min_price) / price_range;
            let y = inner.y + inner.height - 1;
            let offset = (pct * (inner.height - 2) as f64) as u16;
            (y.saturating_sub(offset)).max(inner.y)
        };

        for (i, candle) in visible.iter().enumerate() {
            let x = inner.x + 1 + (i * 6) as u16;
            if x + 4 >= inner.x + inner.width {
                break;
            }

            let color = if candle.bullish() {
                Color::Green
            } else {
                Color::Red
            };

            let open_y = price_to_y(candle.open);
            let close_y = price_to_y(candle.close);
            let high_y = price_to_y(candle.high);
            let low_y = price_to_y(candle.low);

            let body_top = open_y.min(close_y);
            let body_bottom = open_y.max(close_y);

            for y in high_y..=low_y {
                if y >= inner.y && y < inner.y + inner.height {
                    if let Some(cell) = buf.cell_mut((x + 2, y)) {
                        cell.set_char('│').set_style(Style::default().fg(color));
                    }
                }
            }

            for y in body_top..=body_bottom {
                if y >= inner.y && y < inner.y + inner.height {
                    if let Some(cell) = buf.cell_mut((x + 1, y)) {
                        cell.set_char('█').set_style(Style::default().fg(color));
                    }
                    if let Some(cell) = buf.cell_mut((x + 3, y)) {
                        cell.set_char('█').set_style(Style::default().fg(color));
                    }
                }
            }
        }

        let price_str = format!("{:.2}", max_price);
        buf.set_string(
            inner.x,
            inner.y,
            &price_str,
            Style::default().fg(Color::DarkGray),
        );
        let price_str = format!("{:.2}", min_price);
        buf.set_string(
            inner.x,
            inner.y + inner.height - 1,
            &price_str,
            Style::default().fg(Color::DarkGray),
        );
    }
}

pub fn render_candlestick(f: &mut Frame, candles: &[Candle], symbol: &str, area: Rect) {
    CandlestickChart::new(symbol.to_string(), candles.to_vec()).render(area, f.buffer_mut());
}

pub fn render_volume(f: &mut Frame, candles: &[Candle], area: Rect) {
    if area.width < 20 || area.height < 2 {
        return;
    }

    let block = Block::default()
        .title(" Volume ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));
    let inner = block.inner(area);
    block.render(area, f.buffer_mut());

    if inner.is_empty() || candles.is_empty() {
        return;
    }

    let chart_width = inner.width as usize;
    let candle_width = 6;
    let visible_candles = chart_width / candle_width;

    let start = candles.len().saturating_sub(visible_candles);
    let end = candles.len();
    let visible: Vec<_> = candles[start..end].iter().collect();

    if visible.is_empty() {
        return;
    }

    let max_volume = visible
        .iter()
        .filter_map(|c| c.volume)
        .fold(0.0, f64::max)
        .max(1.0);

    let volume_to_height =
        |vol: f64| -> u16 { ((vol / max_volume) * (inner.height - 1) as f64) as u16 };

    let block_style = Style::default().fg(Color::DarkGray);
    let bull_style = Style::default().fg(Color::Green);
    let bear_style = Style::default().fg(Color::Red);

    for (i, candle) in visible.iter().enumerate() {
        let x = inner.x + 1 + (i * candle_width) as u16;
        if x + 3 >= inner.x + inner.width {
            break;
        }

        let vol = candle.volume.unwrap_or(0.0);
        let vol_height = volume_to_height(vol);
        let base_y = inner.y + inner.height - 1;

        let style = if candle.bullish() {
            &bull_style
        } else {
            &bear_style
        };

        for y in base_y.saturating_sub(vol_height)..=base_y {
            if y >= inner.y && y < inner.y + inner.height {
                if let Some(cell) = f.buffer_mut().cell_mut((x + 1, y)) {
                    cell.set_char('█').set_style(*style);
                }
                if let Some(cell) = f.buffer_mut().cell_mut((x + 2, y)) {
                    cell.set_char('█').set_style(*style);
                }
                if let Some(cell) = f.buffer_mut().cell_mut((x + 3, y)) {
                    cell.set_char('█').set_style(*style);
                }
            }
        }
    }

    let vol_label = format!("{:e}", max_volume);
    if let Some(cell) = f.buffer_mut().cell_mut((inner.x, inner.y)) {
        cell.set_char(' ').set_style(block_style);
        for (i, c) in vol_label.chars().take(5).enumerate() {
            if let Some(cell) = f.buffer_mut().cell_mut((inner.x + 1 + i as u16, inner.y)) {
                cell.set_char(c).set_style(block_style);
            }
        }
    }
}
