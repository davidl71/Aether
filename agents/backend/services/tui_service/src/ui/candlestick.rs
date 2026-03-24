use ratatui::{
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Borders, Widget},
    Frame,
};

use api::finance_rates::CurveResponse;

#[derive(Debug, Clone)]
pub struct Candle {
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: Option<f64>,
}

impl Candle {
    pub fn new(open: f64, high: f64, low: f64, close: f64) -> Self {
        Self {
            open,
            high,
            low,
            close,
            volume: None,
        }
    }
    pub fn bullish(&self) -> bool {
        self.close >= self.open
    }
}

pub struct CandlestickChart {
    candles: Vec<Candle>,
    symbol: String,
    scroll_offset: usize,
}

impl CandlestickChart {
    pub fn new(symbol: String, candles: Vec<Candle>) -> Self {
        Self {
            candles,
            symbol,
            scroll_offset: 0,
        }
    }

    pub fn scroll_left(&mut self) {
        if self.scroll_offset > 0 {
            self.scroll_offset -= 1;
        }
    }

    pub fn scroll_right(&mut self) {
        if self.scroll_offset < self.candles.len().saturating_sub(10) {
            self.scroll_offset += 1;
        }
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
        let chart_height = inner.height as usize;
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

        let start = self.scroll_offset.min(self.candles.len().saturating_sub(1));
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
    let chart_height = inner.height as usize;
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

pub fn generate_synthetic_candles(
    curve: &CurveResponse,
    num_candles: usize,
    volatility_bps: f64,
) -> Vec<Candle> {
    if curve.points.is_empty() {
        return Vec::new();
    }

    let base_rate = curve
        .points
        .iter()
        .find(|p| p.days_to_expiry == 30)
        .or_else(|| curve.points.first())
        .map(|p| p.mid_rate)
        .unwrap_or(0.05);

    let vol = volatility_bps / 10000.0;
    let mut candles = Vec::with_capacity(num_candles);
    let mut current_rate = base_rate;

    for i in 0..num_candles {
        let trend = ((i as f64 / num_candles as f64) - 0.5) * 0.01;
        let noise = (rand_simple(i as u64) - 0.5) * vol;

        let open = current_rate;
        let high_in_day = (rand_simple(i as u64 + 1000) - 0.5) * vol * 0.5;
        let low_in_day = (rand_simple(i as u64 + 2000) - 0.5) * vol * 0.5;

        let close = open * (1.0 + trend + noise);
        let high = open.max(close) * (1.0 + high_in_day.abs());
        let low = open.min(close) * (1.0 - low_in_day.abs());

        let liquidity = curve
            .points
            .iter()
            .find(|p| p.days_to_expiry == 30)
            .map(|p| p.liquidity_score)
            .unwrap_or(70.0);
        let volume = Some((liquidity * 1000.0 * (0.5 + rand_simple(i as u64 + 3000))).max(1000.0));

        candles.push(Candle {
            open,
            high,
            low,
            close,
            volume,
        });

        current_rate = close;
    }

    candles
}

fn rand_simple(seed: u64) -> f64 {
    let x = seed.wrapping_mul(1103515245).wrapping_add(12345);
    ((x >> 16) as f64 / 65535.0) * 2.0 - 1.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use api::finance_rates::{CurveResponse, RatePointResponse};

    #[test]
    fn synthetic_candles_from_curve() {
        let curve = CurveResponse {
            symbol: "SPX".to_string(),
            points: vec![
                RatePointResponse {
                    symbol: "SPX".to_string(),
                    expiry: "2026-03-20".to_string(),
                    days_to_expiry: 30,
                    strike_width: 5.0,
                    buy_implied_rate: 0.048,
                    sell_implied_rate: 0.052,
                    mid_rate: 0.050,
                    net_debit: 4.8,
                    net_credit: 5.2,
                    liquidity_score: 70.0,
                    timestamp: String::new(),
                    spread_id: None,
                    data_source: None,
                    strike_low: Some(5998.0),
                    strike_high: Some(6002.0),
                    convenience_yield: None,
                },
                RatePointResponse {
                    symbol: "SPX".to_string(),
                    expiry: "2026-04-17".to_string(),
                    days_to_expiry: 60,
                    strike_width: 5.0,
                    buy_implied_rate: 0.049,
                    sell_implied_rate: 0.053,
                    mid_rate: 0.051,
                    net_debit: 9.5,
                    net_credit: 10.5,
                    liquidity_score: 75.0,
                    timestamp: String::new(),
                    spread_id: None,
                    data_source: None,
                    strike_low: None,
                    strike_high: None,
                    convenience_yield: None,
                },
            ],
            timestamp: String::new(),
            strike_width: Some(5.0),
            point_count: 2,
            underlying_price: Some(6000.0),
        };

        let candles = generate_synthetic_candles(&curve, 10, 50.0);

        assert_eq!(candles.len(), 10);
        for candle in &candles {
            assert!(candle.high >= candle.open);
            assert!(candle.high >= candle.close);
            assert!(candle.low <= candle.open);
            assert!(candle.low <= candle.close);
            assert!(candle.volume.is_some());
        }
    }

    #[test]
    fn synthetic_candles_empty_curve() {
        let curve = CurveResponse {
            symbol: "SPX".to_string(),
            points: vec![],
            timestamp: String::new(),
            strike_width: None,
            point_count: 0,
            underlying_price: None,
        };

        let candles = generate_synthetic_candles(&curve, 10, 50.0);
        assert!(candles.is_empty());
    }
}
