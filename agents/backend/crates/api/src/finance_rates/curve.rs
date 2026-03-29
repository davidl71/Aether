//! Opportunity JSON aggregation, synthetic curves, and per-point extraction.

use crate::finance_rates::types::{
    BoxSpreadInput, CompareRequest, CurveQuery, CurveRequest, CurveResponse, RatePointResponse,
};
use chrono::Utc;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

pub(crate) const DEFAULT_MIN_LIQUIDITY_SCORE: f64 = 50.0;

const SYNTHETIC_DTE_DAYS: &[i32] = &[30, 60, 90, 120, 180, 365];
const SYNTHETIC_BASE_RATE: f64 = 0.045;
const SYNTHETIC_SPOT: f64 = 6000.0;
const SYNTHETIC_STRIKE_WIDTH: f64 = 4.0;
const SYNTHETIC_LIQUIDITY: f64 = 70.0;

pub fn extract_rate(input: BoxSpreadInput) -> Result<RatePointResponse, String> {
    build_rate_point(input, DEFAULT_MIN_LIQUIDITY_SCORE)
        .ok_or_else(|| "Invalid box spread data or below minimum liquidity threshold".to_string())
}

pub fn build_curve(
    request: CurveRequest,
    query: Option<CurveQuery>,
) -> Result<CurveResponse, String> {
    let (opportunities, symbol) = split_curve_request(request, query)?;
    let points = aggregate_opportunities(&opportunities, &symbol, DEFAULT_MIN_LIQUIDITY_SCORE);
    Ok(CurveResponse {
        symbol,
        strike_width: points.first().map(|point| point.strike_width),
        point_count: points.len(),
        points,
        timestamp: Utc::now().to_rfc3339(),
        underlying_price: None,
    })
}

pub fn build_synthetic_curve(symbol: &str, live_base_rate: Option<f64>) -> CurveResponse {
    let base_rate = live_base_rate.unwrap_or(SYNTHETIC_BASE_RATE);
    let now = Utc::now();
    let points: Vec<RatePointResponse> = SYNTHETIC_DTE_DAYS
        .iter()
        .map(|&dte| {
            let t = dte as f64 / 365.0;
            let mid_rate = base_rate.min(0.12);
            let buy_rate = (mid_rate - 0.004).max(0.001);
            let sell_rate = (mid_rate + 0.004).min(0.10);
            let spot = SYNTHETIC_SPOT;
            let width = SYNTHETIC_STRIKE_WIDTH;
            let strike_low = (spot - width / 2.0).round();
            let strike_high = (spot + width / 2.0).round();
            let net_debit = width * (1.0 - buy_rate * t);
            let net_credit = width * (1.0 - sell_rate * t);
            let expiry_date = now + chrono::Duration::days(dte as i64);
            RatePointResponse {
                symbol: symbol.to_string(),
                expiry: expiry_date.format("%Y-%m-%d").to_string(),
                days_to_expiry: dte,
                strike_width: width,
                buy_implied_rate: buy_rate,
                sell_implied_rate: sell_rate,
                mid_rate,
                net_debit: (net_debit * 100.0).round() / 100.0,
                net_credit: (net_credit * 100.0).round() / 100.0,
                liquidity_score: SYNTHETIC_LIQUIDITY,
                timestamp: Utc::now().to_rfc3339(),
                spread_id: None,
                data_source: Some("synthetic".to_string()),
                strike_low: Some(strike_low),
                strike_high: Some(strike_high),
                convenience_yield: None,
            }
        })
        .collect();

    CurveResponse {
        symbol: symbol.to_string(),
        strike_width: Some(SYNTHETIC_STRIKE_WIDTH),
        point_count: points.len(),
        points,
        timestamp: Utc::now().to_rfc3339(),
        underlying_price: Some(SYNTHETIC_SPOT),
    }
}

fn split_curve_request(
    request: CurveRequest,
    query: Option<CurveQuery>,
) -> Result<(Vec<Value>, String), String> {
    match request {
        CurveRequest::Opportunities(opportunities) => Ok((
            opportunities,
            query
                .and_then(|item| item.symbol)
                .ok_or_else(|| "symbol is required".to_string())?,
        )),
        CurveRequest::Named {
            opportunities,
            symbol,
        } => Ok((
            opportunities,
            symbol
                .or_else(|| query.and_then(|item| item.symbol))
                .ok_or_else(|| "symbol is required".to_string())?,
        )),
    }
}

pub(crate) fn split_compare_request(
    request: CompareRequest,
    query: Option<CurveQuery>,
) -> Result<(Vec<Value>, String), String> {
    match request {
        CompareRequest::Opportunities(opportunities) => Ok((
            opportunities,
            query
                .and_then(|item| item.symbol)
                .ok_or_else(|| "symbol is required".to_string())?,
        )),
        CompareRequest::Named {
            opportunities,
            symbol,
        } => Ok((
            opportunities,
            symbol
                .or_else(|| query.and_then(|item| item.symbol))
                .ok_or_else(|| "symbol is required".to_string())?,
        )),
    }
}

pub(crate) fn aggregate_opportunities(
    opportunities: &[Value],
    symbol: &str,
    min_liquidity_score: f64,
) -> Vec<RatePointResponse> {
    let mut grouped: BTreeMap<i32, Vec<(RatePointResponse, Option<String>)>> = BTreeMap::new();

    for opportunity in opportunities {
        let source = opportunity_data_source(opportunity);
        let Some(spread) = opportunity.get("spread") else {
            continue;
        };
        let Ok(input) = serde_json::from_value::<BoxSpreadInput>(spread.clone()) else {
            continue;
        };
        if let Some(point) = build_rate_point(input, min_liquidity_score) {
            grouped
                .entry(point.days_to_expiry)
                .or_default()
                .push((point, source));
        }
    }

    grouped
        .into_iter()
        .filter_map(|(_dte, mut points)| {
            if points.len() == 1 {
                return points.pop().map(|(mut point, source)| {
                    point.data_source = source;
                    point
                });
            }
            let total_weight: f64 = points.iter().map(|(point, _)| point.liquidity_score).sum();
            let divisor = if total_weight > 0.0 {
                total_weight
            } else {
                points.len() as f64
            };
            let weighted = |accessor: fn(&RatePointResponse) -> f64| -> f64 {
                if total_weight > 0.0 {
                    points
                        .iter()
                        .map(|(point, _)| accessor(point) * point.liquidity_score)
                        .sum::<f64>()
                        / divisor
                } else {
                    points.iter().map(|(point, _)| accessor(point)).sum::<f64>() / divisor
                }
            };
            let template = points.first()?;
            let source = source_label(points.iter().filter_map(|(_, source)| source.clone()));
            Some(RatePointResponse {
                symbol: symbol.to_string(),
                expiry: template.0.expiry.clone(),
                days_to_expiry: template.0.days_to_expiry,
                strike_width: template.0.strike_width,
                buy_implied_rate: weighted(|point| point.buy_implied_rate),
                sell_implied_rate: weighted(|point| point.sell_implied_rate),
                mid_rate: weighted(|point| point.mid_rate),
                net_debit: points.iter().map(|(point, _)| point.net_debit).sum::<f64>()
                    / points.len() as f64,
                net_credit: points
                    .iter()
                    .map(|(point, _)| point.net_credit)
                    .sum::<f64>()
                    / points.len() as f64,
                liquidity_score: points
                    .iter()
                    .map(|(point, _)| point.liquidity_score)
                    .fold(0.0_f64, f64::max),
                timestamp: Utc::now().to_rfc3339(),
                spread_id: None,
                data_source: source,
                strike_low: None,
                strike_high: None,
                convenience_yield: None,
            })
        })
        .collect()
}

fn opportunity_data_source(opportunity: &Value) -> Option<String> {
    opportunity
        .get("data_source")
        .and_then(|value| value.as_str())
        .map(|value| value.trim().to_lowercase())
        .filter(|value| !value.is_empty())
}

fn source_label<I>(sources: I) -> Option<String>
where
    I: IntoIterator<Item = String>,
{
    let set = sources.into_iter().collect::<BTreeSet<_>>();
    match set.len() {
        0 => None,
        1 => set.into_iter().next(),
        _ => Some("mixed".to_string()),
    }
}

fn build_rate_point(input: BoxSpreadInput, min_liquidity_score: f64) -> Option<RatePointResponse> {
    let mid_rate = if input.buy_implied_rate > 0.0 && input.sell_implied_rate > 0.0 {
        (input.buy_implied_rate + input.sell_implied_rate) / 2.0
    } else if input.buy_implied_rate > 0.0 {
        input.buy_implied_rate
    } else if input.sell_implied_rate > 0.0 {
        input.sell_implied_rate
    } else {
        return None;
    };

    if input.days_to_expiry <= 0
        || input.strike_width <= 0.0
        || input.liquidity_score < min_liquidity_score
    {
        return None;
    }

    Some(RatePointResponse {
        symbol: input.symbol,
        expiry: input.expiry,
        days_to_expiry: input.days_to_expiry,
        strike_width: input.strike_width,
        buy_implied_rate: input.buy_implied_rate,
        sell_implied_rate: input.sell_implied_rate,
        mid_rate,
        net_debit: input.net_debit,
        net_credit: input.net_credit,
        liquidity_score: input.liquidity_score,
        timestamp: Utc::now().to_rfc3339(),
        spread_id: input.spread_id,
        data_source: None,
        strike_low: None,
        strike_high: None,
        convenience_yield: None,
    })
}
