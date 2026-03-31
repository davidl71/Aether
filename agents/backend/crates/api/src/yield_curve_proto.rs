//! Yield curve NATS KV proto conversion (PROTOBUF_CONVERSION_AND_KV §4.4).
//! Converts between `YieldCurve` / `YieldCurvePoint` (proto) and `CurveResponse` / opportunities (JSON).

use chrono::{TimeZone, Utc};
use nats_adapter::proto::v1::{YieldCurve, YieldCurvePoint};
use prost::Message;
use prost_types::Timestamp;
use serde_json::Value;

use crate::finance_rates::{BoxSpreadInput, CurveResponse, RatePointResponse};

/// Decode KV value as `YieldCurve` proto and convert to `CurveResponse`. Returns `None` if bytes
/// are not valid proto.
pub fn curve_response_from_proto_bytes(bytes: &[u8], symbol: &str) -> Option<CurveResponse> {
    let yc = YieldCurve::decode(bytes).ok()?;
    Some(curve_response_from_proto(&yc, symbol))
}

/// Convert proto `YieldCurve` to domain `CurveResponse`.
pub fn curve_response_from_proto(yc: &YieldCurve, symbol_override: &str) -> CurveResponse {
    let symbol = if yc.symbol.is_empty() {
        symbol_override.to_string()
    } else {
        yc.symbol.clone()
    };
    let strike_width = if yc.strike_width != 0.0 {
        Some(yc.strike_width)
    } else {
        None
    };
    let points: Vec<RatePointResponse> = yc
        .points
        .iter()
        .map(|p| proto_point_to_rate_point(p, &symbol, strike_width))
        .collect();
    let timestamp = yc
        .points
        .first()
        .and_then(|p| p.as_of.as_ref())
        .map(ts_to_rfc3339)
        .unwrap_or_else(|| Utc::now().to_rfc3339());
    CurveResponse {
        symbol,
        points: points.clone(),
        timestamp,
        strike_width,
        point_count: points.len(),
        underlying_price: None,
    }
}

/// Build `YieldCurve` proto from KV-style opportunities (array of `{ "spread": BoxSpreadInput }`).
/// Returns `None` if no valid points (e.g. empty or all filtered).
pub fn yield_curve_from_opportunities(
    opportunities: &[Value],
    symbol: &str,
    strike_width: f64,
) -> Option<YieldCurve> {
    let mut points = Vec::new();
    for opp in opportunities {
        let data_source = opp
            .get("data_source")
            .and_then(|value| value.as_str())
            .filter(|s| !s.trim().is_empty())
            .map(|s| s.to_string());
        let spread = opp.get("spread")?;
        let input: BoxSpreadInput = serde_json::from_value(spread.clone()).ok()?;
        points.push(box_spread_input_to_proto_point(&input, data_source));
    }
    if points.is_empty() {
        return None;
    }
    Some(YieldCurve {
        symbol: symbol.to_string(),
        strike_width,
        benchmark_rate: 0.0,
        points,
    })
}

/// Encode `YieldCurve` to bytes for KV put.
pub fn encode_yield_curve_to_bytes(yc: &YieldCurve) -> Vec<u8> {
    let mut buf = Vec::with_capacity(yc.encoded_len());
    let _ = yc.encode(&mut buf);
    buf
}

fn proto_point_to_rate_point(
    p: &YieldCurvePoint,
    symbol: &str,
    strike_width: Option<f64>,
) -> RatePointResponse {
    let implied = p.implied_rate;
    let timestamp = p
        .as_of
        .as_ref()
        .map(ts_to_rfc3339)
        .unwrap_or_else(|| Utc::now().to_rfc3339());
    let expiry = expiry_placeholder(p.days_to_expiry);
    RatePointResponse {
        symbol: symbol.to_string(),
        expiry,
        days_to_expiry: p.days_to_expiry,
        strike_width: strike_width.unwrap_or(0.0),
        buy_implied_rate: implied,
        sell_implied_rate: implied,
        mid_rate: implied,
        net_debit: p.net_debit,
        net_credit: 0.0,
        liquidity_score: 70.0,
        timestamp,
        spread_id: None,
        data_source: (!p.data_source.trim().is_empty()).then(|| p.data_source.clone()),
        strike_low: None,
        strike_high: None,
        convenience_yield: None,
    }
}

fn box_spread_input_to_proto_point(
    input: &BoxSpreadInput,
    data_source: Option<String>,
) -> YieldCurvePoint {
    let mid_rate = if input.buy_implied_rate > 0.0 && input.sell_implied_rate > 0.0 {
        (input.buy_implied_rate + input.sell_implied_rate) / 2.0
    } else if input.buy_implied_rate > 0.0 {
        input.buy_implied_rate
    } else {
        input.sell_implied_rate
    };
    YieldCurvePoint {
        days_to_expiry: input.days_to_expiry,
        implied_rate: mid_rate,
        effective_rate: mid_rate,
        net_debit: input.net_debit,
        spread_bps: 0.0,
        as_of: Some(Timestamp::from(std::time::SystemTime::now())),
        data_source: data_source.unwrap_or_default(),
    }
}

fn ts_to_rfc3339(ts: &Timestamp) -> String {
    Utc.timestamp_opt(ts.seconds, ts.nanos.max(0) as u32)
        .single()
        .map(|dt| dt.to_rfc3339())
        .unwrap_or_else(|| Utc::now().to_rfc3339())
}

fn expiry_placeholder(dte: i32) -> String {
    let d = Utc::now() + chrono::Duration::days(dte as i64);
    d.format("%Y-%m-%d").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn opportunities_round_trip_preserves_data_source() {
        let opportunities = vec![json!({
            "data_source": "tws",
            "spread": {
                "symbol": "SPX",
                "expiry": "2026-01-16",
                "days_to_expiry": 30,
                "strike_width": 4.0,
                "strike_low": 100.0,
                "strike_high": 104.0,
                "buy_implied_rate": 0.05,
                "sell_implied_rate": 0.05,
                "mid_rate": 0.05,
                "net_debit": 3.5,
                "net_credit": 0.0,
                "liquidity_score": 70.0,
                "timestamp": "2026-01-01T00:00:00Z",
                "spread_id": null,
                "data_source": null,
                "convenience_yield": null
            }
        })];

        let yc = yield_curve_from_opportunities(&opportunities, "SPX", 4.0).expect("curve");
        assert_eq!(yc.points.len(), 1);
        assert_eq!(yc.points[0].data_source, "tws");

        let curve = curve_response_from_proto(&yc, "SPX");
        assert_eq!(curve.points.len(), 1);
        assert_eq!(curve.points[0].data_source.as_deref(), Some("tws"));
    }
}
