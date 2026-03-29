//! Yield-curve aggregation helpers: KV load, JSON body parsing, strike fill.

use nats_adapter::async_nats::Client;
use serde_json::Value;
use tracing::debug;

use api::finance_rates::{build_curve, CompareRequest, CurveQuery, CurveRequest, CurveResponse};
use api::yield_curve_proto;

pub(super) const KV_KEY_PREFIX_YIELD_CURVE: &str = "yield_curve";
pub(super) const REFERENCE_SPOT_ENV_PREFIX: &str = "YIELD_CURVE_REFERENCE_SPOT_";
pub(super) const DEFAULT_REFERENCE_SPOT: f64 = 6000.0;

/// Reference/underlying price for symbol (env YIELD_CURVE_REFERENCE_SPOT_{SYMBOL} or default).
pub(super) fn reference_spot_for_report(symbol: &str) -> f64 {
    let key = format!("{}{}", REFERENCE_SPOT_ENV_PREFIX, symbol.to_uppercase());
    std::env::var(&key)
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(DEFAULT_REFERENCE_SPOT)
}

/// Fill strike_low/strike_high on curve points when missing.
pub(super) fn fill_missing_strikes(curve: &mut CurveResponse, spot: f64) {
    let mut filled = 0_usize;
    for p in curve.points.iter_mut() {
        if p.strike_low.is_none() && p.strike_high.is_none() && p.strike_width > 0.0 {
            let half = p.strike_width / 2.0;
            let round = |x: f64| (x * 10.0).round() / 10.0;
            p.strike_low = Some(round(spot - half));
            p.strike_high = Some(round(spot + half));
            filled += 1;
        }
    }
    if filled > 0 {
        tracing::debug!(filled, %spot, "fill_missing_strikes: filled strike_low/strike_high for points");
    }
}

/// Load yield curve from NATS KV for a symbol.
pub(super) async fn load_yield_curve_from_kv(
    nc: &Client,
    symbol: &str,
    query: Option<&CurveQuery>,
) -> Option<CurveResponse> {
    let bucket = std::env::var("NATS_KV_BUCKET").unwrap_or_else(|_| "LIVE_STATE".to_string());
    let js = nats_adapter::async_nats::jetstream::new(nc.clone());
    let kv: nats_adapter::async_nats::jetstream::kv::Store =
        match js.get_key_value(bucket.as_str()).await {
            Ok(k) => k,
            Err(e) => {
                debug!(%bucket, error = %e, "KV bucket not available for yield curve");
                return None;
            }
        };
    let key = format!("{}.{}", KV_KEY_PREFIX_YIELD_CURVE, symbol);
    let entry = match kv.entry(key.as_str()).await {
        Ok(Some(e)) => e,
        Ok(None) => {
            debug!(%key, "no yield curve key in KV");
            return None;
        }
        Err(e) => {
            debug!(%key, error = %e, "KV get failed for yield curve");
            return None;
        }
    };
    let bytes = entry.value.as_ref().to_vec();
    if let Some(curve) = yield_curve_proto::curve_response_from_proto_bytes(&bytes, symbol) {
        if !curve.points.is_empty() {
            return Some(curve);
        }
    }
    let arr: Vec<Value> = match serde_json::from_slice(&bytes) {
        Ok(a) => a,
        Err(e) => {
            debug!(%key, error = %e, "yield curve KV value not proto, not valid JSON array");
            return None;
        }
    };
    if arr.is_empty() {
        return None;
    }
    let request = CurveRequest::Named {
        opportunities: arr,
        symbol: Some(symbol.to_string()),
    };
    build_curve(request, query.cloned()).ok()
}

pub(super) fn parse_curve_body(body: Option<&[u8]>) -> (CurveRequest, Option<CurveQuery>) {
    let (request, query) = body
        .and_then(|b| serde_json::from_slice::<Value>(b).ok())
        .map(|v| {
            let request = serde_json::from_value::<CurveRequest>(v.clone()).unwrap_or_else(|_| {
                CurveRequest::Named {
                    opportunities: vec![],
                    symbol: None,
                }
            });
            let query = v.get("symbol").map(|s| CurveQuery {
                symbol: s.as_str().map(String::from),
            });
            (request, query)
        })
        .unwrap_or_else(|| {
            (
                CurveRequest::Named {
                    opportunities: vec![],
                    symbol: None,
                },
                None,
            )
        });
    (request, query)
}

pub(super) fn parse_compare_body(body: Option<&[u8]>) -> (CompareRequest, Option<CurveQuery>) {
    let (request, query) = body
        .and_then(|b| serde_json::from_slice::<Value>(b).ok())
        .map(|v| {
            let request =
                serde_json::from_value::<CompareRequest>(v.clone()).unwrap_or_else(|_| {
                    CompareRequest::Named {
                        opportunities: vec![],
                        symbol: None,
                    }
                });
            let query = v.get("symbol").map(|s| CurveQuery {
                symbol: s.as_str().map(String::from),
            });
            (request, query)
        })
        .unwrap_or_else(|| {
            (
                CompareRequest::Named {
                    opportunities: vec![],
                    symbol: None,
                },
                None,
            )
        });
    (request, query)
}
