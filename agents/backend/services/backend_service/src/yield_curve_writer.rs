//! Writes box spread yield curve opportunities to NATS KV for the Yield tab.
//! Key: `yield_curve.{symbol}` in the LIVE_STATE bucket. Value: JSON array of
//! `{ "spread": BoxSpreadInput }` per `api.finance_rates.build_curve` / NATS_KV_USAGE_AND_RECOMMENDATIONS.md.
//!
//! **Pre-population:** Writes once immediately on spawn, then on an interval. Data source:
//! - If `YIELD_CURVE_SOURCE_URL` is set: HTTP GET that URL; expect JSON array of curve points (see docs).
//!   Public reference: boxtrades.com (no API; use for comparison or host your own JSON).
//! - Otherwise: synthetic points (no live option chain).
//! See docs/platform/BOX_SPREAD_YIELD_CURVE_TWS.md and BOXTRADES_REFERENCE.md.

use bytes::Bytes;
use chrono::Utc;
use nats_adapter::async_nats::Client;
use nats_adapter::NatsClient;
use serde::Deserialize;
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, info, warn};

const KV_BUCKET_ENV: &str = "NATS_KV_BUCKET";
const DEFAULT_KV_BUCKET: &str = "LIVE_STATE";
const KEY_PREFIX: &str = "yield_curve";
const STRIKE_WIDTH: f64 = 5.0;
const BASE_RATE: f64 = 0.048;
const RATE_SPREAD: f64 = 0.008;
const LIQUIDITY_SCORE: f64 = 70.0;
const SOURCE_URL_ENV: &str = "YIELD_CURVE_SOURCE_URL";
const HTTP_TIMEOUT_SECS: u64 = 10;

/// One curve point from an external JSON source (e.g. public API or static file).
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
struct ExternalCurvePoint {
    symbol: String,
    expiry: String,
    days_to_expiry: i32,
    #[serde(default = "default_strike_width")]
    strike_width: f64,
    buy_implied_rate: f64,
    sell_implied_rate: f64,
    #[serde(default)]
    net_debit: f64,
    #[serde(default)]
    net_credit: f64,
    #[serde(default = "default_liquidity_score")]
    liquidity_score: f64,
    spread_id: Option<String>,
}

fn default_strike_width() -> f64 {
    STRIKE_WIDTH
}
fn default_liquidity_score() -> f64 {
    LIQUIDITY_SCORE
}

/// DTE values (days to expiry) for synthetic curve points.
const DTE_DAYS: &[i32] = &[30, 60, 90, 120, 180, 365];

/// Build one synthetic BoxSpreadInput for a given symbol, expiry string, and DTE.
fn synthetic_spread(symbol: &str, expiry: &str, dte: i32) -> serde_json::Value {
    let t = dte as f64 / 365.0;
    let mid = BASE_RATE + (dte as f64 / 3650.0); // slight term premium
    let buy_rate = (mid - RATE_SPREAD / 2.0).max(0.001);
    let sell_rate = (mid + RATE_SPREAD / 2.0).min(0.15);
    let net_debit = STRIKE_WIDTH * (1.0 - buy_rate * t);
    let net_credit = STRIKE_WIDTH * (1.0 - sell_rate * t);
    json!({
        "spread": {
            "symbol": symbol,
            "expiry": expiry,
            "days_to_expiry": dte,
            "strike_width": STRIKE_WIDTH,
            "buy_implied_rate": buy_rate,
            "sell_implied_rate": sell_rate,
            "net_debit": (net_debit * 100.0).round() / 100.0,
            "net_credit": (net_credit * 100.0).round() / 100.0,
            "liquidity_score": LIQUIDITY_SCORE,
            "spread_id": null
        }
    })
}

/// Generate synthetic opportunities for a symbol (multiple expiries).
fn synthetic_opportunities(symbol: &str) -> Vec<serde_json::Value> {
    let now = Utc::now();
    DTE_DAYS
        .iter()
        .map(|&dte| {
            let expiry_date = now + chrono::Duration::days(dte as i64);
            let expiry = expiry_date.format("%Y-%m-%d").to_string();
            synthetic_spread(symbol, &expiry, dte)
        })
        .collect()
}

/// Fetch curve points from a public URL. Returns map symbol -> opportunities (each Value is { "spread": BoxSpreadInput }).
/// URL must return a JSON array of objects with: symbol, expiry, days_to_expiry, buy_implied_rate, sell_implied_rate;
/// optional: strike_width (default 5), net_debit, net_credit, liquidity_score (default 50), spread_id.
async fn fetch_curve_from_url(url: &str) -> Option<HashMap<String, Vec<serde_json::Value>>> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(HTTP_TIMEOUT_SECS))
        .build()
        .ok()?;
    let body = client.get(url).send().await.ok()?.text().await.ok()?;
    let points: Vec<ExternalCurvePoint> = serde_json::from_str(&body).ok()?;
    if points.is_empty() {
        return None;
    }
    let mut by_symbol: HashMap<String, Vec<serde_json::Value>> = HashMap::new();
    for p in points {
        let spread = json!({
            "spread": {
                "symbol": p.symbol,
                "expiry": p.expiry,
                "days_to_expiry": p.days_to_expiry,
                "strike_width": p.strike_width,
                "buy_implied_rate": p.buy_implied_rate,
                "sell_implied_rate": p.sell_implied_rate,
                "net_debit": p.net_debit,
                "net_credit": p.net_credit,
                "liquidity_score": p.liquidity_score,
                "spread_id": p.spread_id
            }
        });
    by_symbol
        .entry(p.symbol.clone())
        .or_default()
        .push(spread);
    }
    Some(by_symbol)
}

/// Run the yield curve writer: pre-populate once immediately, then every `interval_secs` write `yield_curve.{symbol}`.
pub fn spawn(nats_client: Arc<NatsClient>, symbols: Vec<String>, interval_secs: u64) {
    if symbols.is_empty() {
        debug!("yield curve writer: no symbols, not spawning");
        return;
    }
    let interval = Duration::from_secs(interval_secs);
    let source_url = std::env::var(SOURCE_URL_ENV).ok();
    tokio::spawn(async move {
        let nc: Client = nats_client.client().clone();
        let bucket = std::env::var(KV_BUCKET_ENV).unwrap_or_else(|_| DEFAULT_KV_BUCKET.to_string());
        let js = nats_adapter::async_nats::jetstream::new(nc.clone());
        let kv = match js.get_key_value(bucket.as_str()).await {
            Ok(k) => k,
            Err(e) => {
                warn!(%bucket, error = %e, "yield curve writer: KV bucket not available, skipping");
                return;
            }
        };
        let source_desc = source_url
            .as_deref()
            .unwrap_or("synthetic");
        info!(
            symbols = ?symbols,
            interval_secs = interval_secs,
            %bucket,
            source = source_desc,
            "yield curve writer started (pre-populate then interval)"
        );

        /// One full write cycle: fetch (if URL set) or synthetic, then write each symbol to KV.
        async fn write_cycle(
            kv: &nats_adapter::async_nats::jetstream::kv::Store,
            symbols: &[String],
            source_url: Option<&str>,
        ) {
            let by_symbol: HashMap<String, Vec<serde_json::Value>> = if let Some(url) = source_url {
                match fetch_curve_from_url(url).await {
                    Some(map) if !map.is_empty() => map,
                    _ => {
                        debug!("yield curve writer: fetch from URL failed or empty, using synthetic");
                        symbols
                            .iter()
                            .map(|s| (s.clone(), synthetic_opportunities(s)))
                            .collect()
                    }
                }
            } else {
                symbols
                    .iter()
                    .map(|s| (s.clone(), synthetic_opportunities(s)))
                    .collect()
            };
            for (symbol, opportunities) in by_symbol {
                if opportunities.is_empty() {
                    continue;
                }
                let key = format!("{}.{}", KEY_PREFIX, symbol);
                let payload = serde_json::to_vec(&opportunities).unwrap_or_default();
                if let Err(e) = kv.put(key.as_str(), Bytes::from(payload)).await {
                    warn!(%key, error = %e, "yield curve writer: put failed");
                } else {
                    debug!(%key, points = opportunities.len(), "yield curve writer: wrote");
                }
            }
        }

        // Pre-populate immediately so the Yield tab has data without waiting for first interval.
        write_cycle(&kv, &symbols, source_url.as_deref()).await;

        loop {
            tokio::time::sleep(interval).await;
            write_cycle(&kv, &symbols, source_url.as_deref()).await;
        }
    });
}
