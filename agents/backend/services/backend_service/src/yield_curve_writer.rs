//! Writes box spread yield curve to NATS KV for the Yield tab.
//! Key: `yield_curve.{symbol}` in the LIVE_STATE bucket. Values are source-annotated JSON
//! opportunities for this backend writer path; backend readers still try proto first for older
//! entries and other writers.
//!
//! **Pre-population:** Writes once immediately on spawn, then on an interval. This selects a
//! yield-curve data source, not a trading or execution engine.
//! - If `YIELD_CURVE_SOURCE=tws` (or config): call `tws_yield_curve::fetch_yield_curve_from_tws(symbol)` per symbol;
//!   same env as daemon (TWS_HOST, TWS_PORT, YIELD_CURVE_USE_CLOSING, etc.). This branch writes
//!   source-annotated JSON so the TUI can keep the live source label.
//! - Else if `YIELD_CURVE_SOURCE_URL` is set: HTTP GET that URL; expect JSON array of curve points (see docs).
//!   Public reference: boxtrades.com (no API; use for comparison or host your own JSON).
//! - Otherwise: query Yahoo option chains per symbol, then fall back to synthetic placeholder points when Yahoo is unavailable.
//! See docs/platform/BOX_SPREAD_YIELD_CURVE_TWS.md and TWS_YIELD_CURVE_KV_WRITER.md.

use bytes::Bytes;
use chrono::Utc;
use market_data::yield_curve::{PolygonYieldCurveSource, YahooYieldCurveSource};
use nats_adapter::async_nats::Client;
use nats_adapter::NatsClient;
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, info, warn};

// ---------------------------------------------------------------------------
// External URL fetching (planned future functionality)
// TBD: create task for YIELD_CURVE_SOURCE_URL integration
// ---------------------------------------------------------------------------
#[allow(dead_code)]
const KV_BUCKET_ENV: &str = "NATS_KV_BUCKET";
#[allow(dead_code)]
const DEFAULT_KV_BUCKET: &str = "LIVE_STATE";
#[allow(dead_code)]
const KEY_PREFIX: &str = "yield_curve";
/// Default strike width (points). Symmetric ±width/2 around spot (e.g. 4 → ±2 around the money).
const STRIKE_WIDTH: f64 = 4.0;
/// Default reference spot when env not set (ensures synthetic strikes bracket spot: one leg ITM, one OTM).
const DEFAULT_REFERENCE_SPOT: f64 = 6000.0;
const REFERENCE_SPOT_ENV_PREFIX: &str = "YIELD_CURVE_REFERENCE_SPOT_";
/// Base short-term rate (decimal, e.g. 0.045 = 4.5%). Updated to be closer to current SOFR rates (~4.5-5.3%).
const BASE_RATE: f64 = 0.045;
/// Bid-ask spread in rate terms (decimal, e.g. 0.008 = 80 bps).
const RATE_SPREAD: f64 = 0.008;
/// Term premium per year (decimal). 0 = flat curve (box APR same for all DTE). Non-zero slopes the curve.
const TERM_PREMIUM_PER_YEAR: f64 = 0.0;
/// Convenience yield (decimal, e.g. 0.005 = 0.5%). Benefit of holding the underlying; cost-of-carry ≈ rate − convenience_yield.
const CONVENIENCE_YIELD: f64 = 0.005;
const LIQUIDITY_SCORE: f64 = 70.0;
#[allow(dead_code)]
const SOURCE_ENV: &str = "YIELD_CURVE_SOURCE";
#[allow(dead_code)]
const SOURCE_URL_ENV: &str = "YIELD_CURVE_SOURCE_URL";
#[allow(dead_code)]
const HTTP_TIMEOUT_SECS: u64 = 10;

/// One curve point from an external JSON source (e.g. public API or static file).
#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
struct ExternalCurvePoint {
    symbol: String,
    expiry: String,
    days_to_expiry: i32,
    #[serde(default = "default_strike_width")]
    strike_width: f64,
    #[serde(default)]
    strike_low: Option<f64>,
    #[serde(default)]
    strike_high: Option<f64>,
    buy_implied_rate: f64,
    sell_implied_rate: f64,
    #[serde(default)]
    net_debit: f64,
    #[serde(default)]
    net_credit: f64,
    #[serde(default = "default_liquidity_score")]
    liquidity_score: f64,
    spread_id: Option<String>,
    #[serde(default)]
    convenience_yield: Option<f64>,
}

#[allow(dead_code)]
fn default_strike_width() -> f64 {
    STRIKE_WIDTH
}
#[allow(dead_code)]
fn default_liquidity_score() -> f64 {
    LIQUIDITY_SCORE
}

fn annotate_data_source(mut opportunities: Vec<Value>, source: &str) -> Vec<Value> {
    for opportunity in &mut opportunities {
        if let Some(obj) = opportunity.as_object_mut() {
            obj.entry("data_source".to_string())
                .or_insert_with(|| Value::String(source.to_string()));
        }
    }
    opportunities
}

fn annotate_data_sources(
    opportunities: HashMap<String, Vec<Value>>,
    source: &str,
) -> HashMap<String, Vec<Value>> {
    opportunities
        .into_iter()
        .map(|(symbol, opportunities)| (symbol, annotate_data_source(opportunities, source)))
        .collect()
}

/// DTE values (days to expiry) for synthetic curve points.
const DTE_DAYS: &[i32] = &[30, 60, 90, 120, 180, 365];

/// Reference spot for a symbol (env YIELD_CURVE_REFERENCE_SPOT_{SYMBOL} or default).
fn reference_spot(symbol: &str) -> f64 {
    let key = format!("{}{}", REFERENCE_SPOT_ENV_PREFIX, symbol.to_uppercase());
    std::env::var(&key)
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(DEFAULT_REFERENCE_SPOT)
}

/// Symmetric ±width/2 around spot (e.g. width 4 → ±2, so K_low = spot-2, K_high = spot+2). One leg ITM, one OTM.
fn strikes_symmetric_fixed(spot: f64, width: f64) -> (f64, f64) {
    let half = width / 2.0;
    let round = |x: f64| (x * 10.0).round() / 10.0;
    (round(spot - half), round(spot + half))
}

/// Build one synthetic BoxSpreadInput for a given symbol, expiry string, and DTE.
/// Uses symmetric strike width (default 4 pts → ±2 around reference spot).
/// Box PV: you pay net_debit now, receive width at expiry → implied (simple) annual rate r satisfies
/// net_debit = width * (1 - r*T), so r = (1 - net_debit/width)/T. We set buy_rate/sell_rate and
/// derive net_debit/net_credit from that; displayed APR is the same rate (mid of buy/sell).
fn synthetic_spread(symbol: &str, expiry: &str, dte: i32) -> serde_json::Value {
    let t = dte as f64 / 365.0;
    let term_premium = (dte as f64 / 365.0) * TERM_PREMIUM_PER_YEAR;
    let mid = (BASE_RATE + term_premium).min(0.12); // cap ~12% if term premium is used
    let buy_rate = (mid - RATE_SPREAD / 2.0).max(0.001);
    let sell_rate = (mid + RATE_SPREAD / 2.0).min(0.15);
    let spot = reference_spot(symbol);
    let (strike_low, strike_high) = strikes_symmetric_fixed(spot, STRIKE_WIDTH);
    let width = strike_high - strike_low;
    // PV = width * (1 - r*T) => net_debit (buy side), net_credit (sell side)
    let net_debit = width * (1.0 - buy_rate * t);
    let net_credit = width * (1.0 - sell_rate * t);
    json!({
        "spread": {
            "symbol": symbol,
            "expiry": expiry,
            "days_to_expiry": dte,
            "strike_width": (width * 100.0).round() / 100.0,
            "strike_low": strike_low,
            "strike_high": strike_high,
            "buy_implied_rate": buy_rate,
            "sell_implied_rate": sell_rate,
            "net_debit": (net_debit * 100.0).round() / 100.0,
            "net_credit": (net_credit * 100.0).round() / 100.0,
            "liquidity_score": LIQUIDITY_SCORE,
            "spread_id": null,
            "convenience_yield": CONVENIENCE_YIELD
        }
    })
}

/// Generate synthetic opportunities for a symbol (multiple expiries).
/// Uses live_benchmark_rate if provided (e.g., from SOFR/Treasury), otherwise BASE_RATE (4.8%).
pub(crate) fn synthetic_opportunities(
    symbol: &str,
    live_benchmark_rate: Option<f64>,
) -> Vec<serde_json::Value> {
    let now = Utc::now();
    let base_rate = live_benchmark_rate.unwrap_or(BASE_RATE);
    DTE_DAYS
        .iter()
        .map(|&dte| {
            let expiry_date = now + chrono::Duration::days(dte as i64);
            let expiry = expiry_date.format("%Y-%m-%d").to_string();
            synthetic_spread_with_rate(symbol, &expiry, dte, base_rate)
        })
        .collect()
}

fn synthetic_spread_with_rate(
    symbol: &str,
    expiry: &str,
    dte: i32,
    base_rate: f64,
) -> serde_json::Value {
    let t = dte as f64 / 365.0;
    let term_premium = (dte as f64 / 365.0) * TERM_PREMIUM_PER_YEAR;
    let mid = (base_rate + term_premium).min(0.12);
    let buy_rate = (mid - RATE_SPREAD / 2.0).max(0.001);
    let sell_rate = (mid + RATE_SPREAD / 2.0).min(0.15);
    let spot = reference_spot(symbol);
    let (strike_low, strike_high) = strikes_symmetric_fixed(spot, STRIKE_WIDTH);
    let width = strike_high - strike_low;
    let net_debit = width * (1.0 - buy_rate * t);
    let net_credit = width * (1.0 - sell_rate * t);
    json!({
        "spread": {
            "symbol": symbol,
            "expiry": expiry,
            "days_to_expiry": dte,
            "strike_width": (width * 100.0).round() / 100.0,
            "strike_low": strike_low,
            "strike_high": strike_high,
            "buy_implied_rate": (buy_rate * 100.0).round() / 100.0,
            "sell_implied_rate": (sell_rate * 100.0).round() / 100.0,
            "net_debit": (net_debit * 100.0).round() / 100.0,
            "net_credit": (net_credit * 100.0).round() / 100.0,
            "liquidity_score": LIQUIDITY_SCORE,
        },
        "data_source": "synthetic"
    })
}

/// Fetch curve points from a public URL. Returns map symbol -> opportunities (each Value is { "spread": BoxSpreadInput }).
/// URL must return a JSON array of objects with: symbol, expiry, days_to_expiry, buy_implied_rate, sell_implied_rate;
/// optional: strike_width (default 5), net_debit, net_credit, liquidity_score (default 50), spread_id.
/// TBD: create task for YIELD_CURVE_SOURCE_URL integration
#[allow(dead_code)]
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
        let mut spread_obj = serde_json::json!({
            "symbol": p.symbol,
            "expiry": p.expiry,
            "days_to_expiry": p.days_to_expiry,
            "strike_width": p.strike_width,
            "strike_low": p.strike_low,
            "strike_high": p.strike_high,
            "buy_implied_rate": p.buy_implied_rate,
            "sell_implied_rate": p.sell_implied_rate,
            "net_debit": p.net_debit,
            "net_credit": p.net_credit,
            "liquidity_score": p.liquidity_score,
            "spread_id": p.spread_id
        });
        if let Some(cy) = p.convenience_yield {
            spread_obj["convenience_yield"] = serde_json::json!(cy);
        }
        let spread = json!({ "spread": spread_obj });
        by_symbol.entry(p.symbol.clone()).or_default().push(spread);
    }
    Some(by_symbol)
}

/// Run the yield curve writer: pre-populate once immediately, then every `interval_secs` write `yield_curve.{symbol}`.
/// Returns a sender to trigger one immediate write cycle (e.g. for `api.yield_curve.refresh`).
/// Source: `config_source` (from the `yield_curve` config table) or env `YIELD_CURVE_SOURCE`; "tws" => TWS, else URL/Yahoo/synthetic.
pub fn spawn(
    nats_client: Arc<NatsClient>,
    symbols: Vec<String>,
    interval_secs: u64,
    config_source: Option<String>,
) -> Option<tokio::sync::mpsc::Sender<()>> {
    if symbols.is_empty() {
        debug!("yield curve writer: no symbols, not spawning");
        return None;
    }
    let (tx, mut rx) = tokio::sync::mpsc::channel::<()>(4);
    let interval = Duration::from_secs(interval_secs);
    let source = config_source
        .or_else(|| std::env::var(SOURCE_ENV).ok())
        .unwrap_or_default()
        .trim()
        .to_lowercase();
    let source_tws = source == "tws";
    let source_polygon = source == "polygon";
    let source_url = if source_tws {
        None
    } else {
        std::env::var(SOURCE_URL_ENV).ok()
    };
    tokio::spawn(async move {
        let nc: Client = nats_client.client().clone();
        let bucket = std::env::var(KV_BUCKET_ENV).unwrap_or_else(|_| DEFAULT_KV_BUCKET.to_string());
        let js = nats_adapter::async_nats::jetstream::new(nc.clone());
        let kv = match js.get_key_value(bucket.as_str()).await {
            Ok(k) => k,
            Err(_) => match js
                .create_key_value(nats_adapter::async_nats::jetstream::kv::Config {
                    bucket: bucket.clone(),
                    history: 3,
                    max_age: Duration::from_secs(86400),
                    max_value_size: 65_536,
                    max_bytes: 10 * 1024 * 1024,
                    ..Default::default()
                })
                .await
            {
                Ok(k) => {
                    info!(%bucket, "yield curve writer: created KV bucket");
                    k
                }
                Err(e) => {
                    warn!(%bucket, error = %e, "yield curve writer: KV bucket not available, skipping");
                    return;
                }
            },
        };
        let source_desc = if source_tws {
            "tws"
        } else if source_polygon {
            "polygon"
        } else {
            source_url.as_deref().unwrap_or("synthetic")
        };
        info!(
            symbols = ?symbols,
            interval_secs = interval_secs,
            %bucket,
            source = source_desc,
            "yield curve writer started (pre-populate then interval)"
        );

        /// Best-effort overnight SOFR (decimal, e.g. 0.052). Used to anchor synthetic fallback near live funding.
        async fn live_synthetic_benchmark_rate() -> Option<f64> {
            let client = match reqwest::Client::builder()
                .timeout(Duration::from_secs(HTTP_TIMEOUT_SECS))
                .build()
            {
                Ok(c) => c,
                Err(_) => return None,
            };
            let resp = api::finance_rates::get_sofr_rates(&client).await;
            resp.overnight.rate.filter(|r| *r > 0.0 && *r <= 0.2)
        }

        /// One full write cycle: TWS, URL, Yahoo, or synthetic placeholder; then write each symbol to KV.
        async fn write_cycle(
            kv: &nats_adapter::async_nats::jetstream::kv::Store,
            symbols: &[String],
            source_url: Option<&str>,
            source_tws: bool,
            source_polygon: bool,
        ) {
            if source_tws {
                for symbol in symbols {
                    match tws_yield_curve::fetch_yield_curve_from_tws(symbol).await {
                        Ok(opportunities) if !opportunities.is_empty() => {
                            let key = format!("{}.{}", KEY_PREFIX, symbol);
                            let payload =
                                serde_json::to_vec(&annotate_data_source(opportunities, "tws"))
                                    .unwrap_or_default();
                            if let Err(e) = kv.put(key.as_str(), Bytes::from(payload)).await {
                                warn!(%key, error = %e, "yield curve writer: put failed");
                            } else {
                                debug!(%key, "yield curve writer: wrote (TWS JSON)");
                            }
                        }
                        Ok(_) => debug!(%symbol, "yield curve writer: no TWS points"),
                        Err(e) => {
                            warn!(%symbol, error = %e, "yield curve writer: TWS fetch failed")
                        }
                    }
                }
                return;
            }
            if source_polygon {
                let polygon = match PolygonYieldCurveSource::from_env() {
                    Ok(p) => p,
                    Err(e) => {
                        debug!(error = %e, "yield curve writer: Polygon source unavailable, using synthetic");
                        let live_bench = live_synthetic_benchmark_rate().await;
                        for sym in symbols {
                            let key = format!("{}.{}", KEY_PREFIX, sym);
                            let payload = serde_json::to_vec(&synthetic_opportunities(sym, live_bench))
                                .unwrap_or_default();
                            if let Err(e) = kv.put(key.as_str(), Bytes::from(payload)).await {
                                warn!(%key, error = %e, "yield curve writer: put failed");
                            }
                        }
                        return;
                    }
                };

                for sym in symbols {
                    match polygon.fetch_yield_curve(sym).await {
                        Ok(curve) if !curve.points.is_empty() => {
                            let opps: Vec<Value> = curve
                                .points
                                .iter()
                                .map(|p| {
                                    json!({
                                        "spread": {
                                            "symbol": curve.symbol,
                                            "expiry": p.expiry.format("%Y-%m-%d").to_string(),
                                            "days_to_expiry": p.dte,
                                            "strike_width": p.strike_width,
                                            "strike_low": p.strike_low,
                                            "strike_high": p.strike_high,
                                            "buy_implied_rate": p.buy_implied_rate,
                                            "sell_implied_rate": p.sell_implied_rate,
                                            "net_debit": p.net_debit,
                                            "net_credit": p.net_credit,
                                            "liquidity_score": p.liquidity_score,
                                        },
                                        "data_source": "polygon"
                                    })
                                })
                                .collect();
                            let key = format!("{}.{}", KEY_PREFIX, sym);
                            let payload = serde_json::to_vec(&opps).unwrap_or_default();
                            if let Err(e) = kv.put(key.as_str(), Bytes::from(payload)).await {
                                warn!(%key, error = %e, "yield curve writer: put failed");
                            } else {
                                debug!(%key, points = opps.len(), "yield curve writer: wrote (Polygon JSON)");
                            }
                        }
                        Ok(_) => debug!(symbol = %sym, "yield curve writer: Polygon returned empty, using synthetic"),
                        Err(e) => debug!(symbol = %sym, error = %e, "yield curve writer: Polygon failed, using synthetic"),
                    }
                }
                return;
            }
            let live_bench = live_synthetic_benchmark_rate().await;
            if live_bench.is_some() {
                debug!(
                    rate = ?live_bench,
                    "yield curve writer: using live SOFR overnight for synthetic anchor (fallback paths)"
                );
            } else {
                debug!("yield curve writer: no live SOFR (unavailable or out of range), synthetic uses BASE_RATE");
            }
            let by_symbol: HashMap<String, Vec<serde_json::Value>> = if let Some(url) = source_url {
                match fetch_curve_from_url(url).await {
                    Some(map) if !map.is_empty() => annotate_data_sources(map, "url"),
                    _ => {
                        debug!(
                            "yield curve writer: fetch from URL failed or empty, using synthetic"
                        );
                        symbols
                            .iter()
                            .map(|s| (s.clone(), synthetic_opportunities(s, live_bench)))
                            .collect()
                    }
                }
            } else {
                // Default: try Yahoo option chains for real implied rates; fall back to synthetic
                // placeholder if Yahoo is unavailable (offline, rate-limited, or market closed).
                let yahoo = YahooYieldCurveSource::new();
                let mut result: HashMap<String, Vec<serde_json::Value>> = HashMap::new();
                for sym in symbols {
                    match yahoo.fetch_yield_curve(sym).await {
                        Ok(curve) if !curve.points.is_empty() => {
                            let opps: Vec<serde_json::Value> = curve
                                .points
                                .iter()
                                .map(|p| {
                                    json!({
                                        "spread": {
                                            "symbol": curve.symbol,
                                            "expiry": p.expiry.format("%Y-%m-%d").to_string(),
                                            "days_to_expiry": p.dte,
                                            "strike_width": p.strike_width,
                                            "strike_low": p.strike_low,
                                            "strike_high": p.strike_high,
                                            "buy_implied_rate": p.buy_implied_rate,
                                            "sell_implied_rate": p.sell_implied_rate,
                                            "net_debit": p.net_debit,
                                            "net_credit": p.net_credit,
                                            "liquidity_score": p.liquidity_score,
                                        },
                                        "data_source": "yahoo"
                                    })
                                })
                                .collect();
                            debug!(symbol = %sym, points = opps.len(), "yield curve writer: Yahoo option chain");
                            result.insert(sym.clone(), opps);
                        }
                        Ok(_) => {
                            debug!(symbol = %sym, "yield curve writer: Yahoo returned empty, using synthetic");
                            result.insert(sym.clone(), synthetic_opportunities(sym, live_bench));
                        }
                        Err(e) => {
                            debug!(symbol = %sym, error = %e, "yield curve writer: Yahoo failed, using synthetic");
                            result.insert(sym.clone(), synthetic_opportunities(sym, live_bench));
                        }
                    }
                }
                result
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
                    debug!(%key, points = opportunities.len(), "yield curve writer: wrote (JSON)");
                }
            }
        }

        // Pre-populate immediately so the Yield tab has data without waiting for first interval.
        write_cycle(
            &kv,
            &symbols,
            source_url.as_deref(),
            source_tws,
            source_polygon,
        )
        .await;

        let mut interval_tick = tokio::time::interval(interval);
        interval_tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        loop {
            tokio::select! {
                _ = interval_tick.tick() => {
                    write_cycle(
                        &kv,
                        &symbols,
                        source_url.as_deref(),
                        source_tws,
                        source_polygon,
                    )
                    .await;
                }
                _ = rx.recv() => {
                    write_cycle(
                        &kv,
                        &symbols,
                        source_url.as_deref(),
                        source_tws,
                        source_polygon,
                    )
                    .await;
                }
            }
        }
    });
    Some(tx)
}
