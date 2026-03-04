use axum::{
  extract::{Path, Query, State},
  Json,
};
use chrono::Utc;

use super::models::ChartQuery;
use crate::rest::RestState;

pub async fn get_chart(
  State(state): State<RestState>,
  Path(symbol): Path<String>,
  Query(params): Query<ChartQuery>,
) -> Json<serde_json::Value> {
  let timeframe = params.timeframe.unwrap_or_else(|| "1D".to_string());
  let num_candles: usize = match timeframe.as_str() {
    "1D" => 48,
    "1W" => 7 * 48,
    "1M" => 30,
    "3M" => 90,
    "1Y" => 252,
    _ => 48,
  };

  let snapshot = state.snapshot.read().await;
  let sym_snap = snapshot.symbols.iter().find(|s| s.symbol == symbol);

  let base_price = sym_snap.map(|s| s.candle.close).unwrap_or(450.0);
  let interval_secs: i64 = match timeframe.as_str() {
    "1M" | "3M" | "1Y" => 86400,
    _ => 1800,
  };

  let now = Utc::now();
  let mut bars = Vec::with_capacity(num_candles);
  let mut price = base_price * 0.97;
  let step = (base_price * 0.03) / num_candles as f64;

  for i in 0..num_candles {
    let ts = now - chrono::Duration::seconds(interval_secs * (num_candles - i) as i64);
    let noise = ((i as f64 * 0.7).sin() * 0.005 + (i as f64 * 1.3).cos() * 0.003) * base_price;
    let open = price + noise;
    let close = price + step + noise * 0.5;
    let high = open.max(close) + (base_price * 0.002);
    let low = open.min(close) - (base_price * 0.002);
    let volume = 1000 + ((i * 137 + 42) % 5000) as u64;

    bars.push(serde_json::json!({
      "time": ts.to_rfc3339(),
      "open": (open * 100.0).round() / 100.0,
      "high": (high * 100.0).round() / 100.0,
      "low": (low * 100.0).round() / 100.0,
      "close": (close * 100.0).round() / 100.0,
      "volume": volume,
    }));
    price += step;
  }

  Json(serde_json::json!({
    "candles": bars,
    "symbol": symbol,
    "timeframe": timeframe,
    "count": bars.len(),
    "as_of": now,
  }))
}
