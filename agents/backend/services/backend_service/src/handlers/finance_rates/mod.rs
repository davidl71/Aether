//! Finance rates NATS request/reply handlers.
//! Subjects: `api.finance_rates.*`, `api.yield_curve.*`
//!
//! - [`helpers`] — KV load, JSON body parsing, strike fill (aggregation).
//! - This module — subscription wiring and JSON response shaping.

mod helpers;

use crate::handlers::{api_queue_group, concurrency_limit, handle_sub_parallel};
use api::finance_rates::{
    build_curve, compare_rates, extract_rate, get_sofr_rates, get_treasury_rates,
    yield_curve_comparison, BoxSpreadInput, CurveRequest, CurveResponse,
    YieldCurveComparisonRequest,
};
use bytes::Bytes;
use futures::StreamExt;
use helpers::{
    fill_missing_strikes, load_yield_curve_from_kv, parse_compare_body, parse_curve_body,
    reference_spot_for_report, DEFAULT_REFERENCE_SPOT,
};
use nats_adapter::async_nats::Client;
use tracing::{debug, warn};
use tws_yield_curve;

/// Default concurrency limit for finance rates HTTP calls.
const DEFAULT_FINANCE_CONCURRENCY: usize = 50;

fn finance_concurrency_limit() -> usize {
    std::env::var("NATS_FINANCE_CONCURRENCY_LIMIT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(DEFAULT_FINANCE_CONCURRENCY)
        .min(concurrency_limit())
}

const SUBJECT_FINANCE_RATES_EXTRACT: &str = "api.finance_rates.extract";
const SUBJECT_FINANCE_RATES_BUILD_CURVE: &str = "api.finance_rates.build_curve";
const SUBJECT_FINANCE_RATES_COMPARE: &str = "api.finance_rates.compare";
const SUBJECT_FINANCE_RATES_YIELD_CURVE: &str = "api.finance_rates.yield_curve";
const SUBJECT_FINANCE_RATES_BENCHMARKS: &str = "api.finance_rates.benchmarks";
const SUBJECT_FINANCE_RATES_SOFR: &str = "api.finance_rates.sofr";
const SUBJECT_FINANCE_RATES_TREASURY: &str = "api.finance_rates.treasury";
const SUBJECT_YIELD_CURVE_REFRESH: &str = "api.yield_curve.refresh";

/// Spawn Finance Rates NATS API handlers with bounded parallelism.
pub async fn spawn(nc: Client, yield_curve_refresh_tx: Option<tokio::sync::mpsc::Sender<()>>) {
    let limit = finance_concurrency_limit();

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .unwrap_or_else(|_| reqwest::Client::new());

    if let Some(tx) = yield_curve_refresh_tx {
        let nc_refresh = nc.clone();
        tokio::spawn(async move {
            let mut sub = match nc_refresh
                .subscribe(SUBJECT_YIELD_CURVE_REFRESH.to_string())
                .await
            {
                Ok(s) => s,
                Err(e) => {
                    tracing::warn!(error = %e, "subscribe api.yield_curve.refresh failed");
                    return;
                }
            };
            while let Some(msg) = sub.next().await {
                let _ = tx.send(()).await;
                if let Some(reply) = msg.reply {
                    let _ = nc_refresh
                        .publish(reply, Bytes::from_static(b"{\"ok\":true}"))
                        .await;
                }
            }
        });
    }

    let sub_extract = match nc
        .queue_subscribe(SUBJECT_FINANCE_RATES_EXTRACT.to_string(), api_queue_group())
        .await
    {
        Ok(s) => s,
        Err(e) => {
            warn!(error = %e, "subscribe api.finance_rates.extract failed");
            return;
        }
    };
    let sub_build = match nc
        .queue_subscribe(
            SUBJECT_FINANCE_RATES_BUILD_CURVE.to_string(),
            api_queue_group(),
        )
        .await
    {
        Ok(s) => s,
        Err(e) => {
            warn!(error = %e, "subscribe api.finance_rates.build_curve failed");
            return;
        }
    };
    let sub_compare = match nc
        .queue_subscribe(SUBJECT_FINANCE_RATES_COMPARE.to_string(), api_queue_group())
        .await
    {
        Ok(s) => s,
        Err(e) => {
            warn!(error = %e, "subscribe api.finance_rates.compare failed");
            return;
        }
    };
    let sub_yield = match nc
        .queue_subscribe(
            SUBJECT_FINANCE_RATES_YIELD_CURVE.to_string(),
            api_queue_group(),
        )
        .await
    {
        Ok(s) => s,
        Err(e) => {
            warn!(error = %e, "subscribe api.finance_rates.yield_curve failed");
            return;
        }
    };
    let sub_benchmarks = match nc
        .queue_subscribe(
            SUBJECT_FINANCE_RATES_BENCHMARKS.to_string(),
            api_queue_group(),
        )
        .await
    {
        Ok(s) => s,
        Err(e) => {
            warn!(error = %e, "subscribe api.finance_rates.benchmarks failed");
            return;
        }
    };
    let sub_sofr = match nc
        .queue_subscribe(SUBJECT_FINANCE_RATES_SOFR.to_string(), api_queue_group())
        .await
    {
        Ok(s) => s,
        Err(e) => {
            warn!(error = %e, "subscribe api.finance_rates.sofr failed");
            return;
        }
    };
    let sub_treasury = match nc
        .queue_subscribe(
            SUBJECT_FINANCE_RATES_TREASURY.to_string(),
            api_queue_group(),
        )
        .await
    {
        Ok(s) => s,
        Err(e) => {
            warn!(error = %e, "subscribe api.finance_rates.treasury failed");
            return;
        }
    };

    let nc_extract = nc.clone();
    tokio::spawn(handle_sub_parallel(
        nc_extract,
        sub_extract,
        move |body: Option<Vec<u8>>| async move {
            let input: BoxSpreadInput =
                match body.as_deref().and_then(|b| serde_json::from_slice(b).ok()) {
                    Some(i) => i,
                    None => {
                        return finance_rates_result::<api::finance_rates::RatePointResponse>(Err(
                            "request body must be BoxSpreadInput JSON".to_string(),
                        ))
                    }
                };
            finance_rates_result(extract_rate(input))
        },
        limit,
    ));

    let nc_build = nc.clone();
    let limit_build = limit;
    tokio::spawn(handle_sub_parallel(
        nc_build.clone(),
        sub_build,
        move |body: Option<Vec<u8>>| {
            let nc_build = nc_build.clone();
            async move {
                let (mut request, query) = parse_curve_body(body.as_deref());
                let symbol: Option<String> = match &request {
                    CurveRequest::Named { symbol: s, .. } => s
                        .clone()
                        .or_else(|| query.as_ref().and_then(|q| q.symbol.clone())),
                    CurveRequest::Opportunities(_) => query.as_ref().and_then(|q| q.symbol.clone()),
                };
                let _limit = limit_build;

                let is_empty = symbol.as_ref().map_or(false, |_sym| match &request {
                    CurveRequest::Opportunities(opps) => opps.is_empty(),
                    CurveRequest::Named { opportunities, .. } => opportunities.is_empty(),
                });

                let mut used_tws = false;

                if is_empty {
                    if let Some(ref sym) = symbol {
                        if let Some(curve_from_kv) =
                            load_yield_curve_from_kv(&nc_build, sym, query.as_ref()).await
                        {
                            let spot = reference_spot_for_report(sym);
                            let mut curve = curve_from_kv;
                            curve.underlying_price = Some(spot);
                            fill_missing_strikes(&mut curve, spot);
                            for p in curve.points.iter_mut() {
                                p.data_source = Some("KV".to_string());
                            }
                            debug!(symbol = %sym, "Using KV yield curve");
                            return finance_rates_result(Ok(curve));
                        }

                        if let Ok(opportunities) =
                            tws_yield_curve::fetch_yield_curve_from_tws(sym).await
                        {
                            if !opportunities.is_empty() {
                                request = CurveRequest::Named {
                                    opportunities,
                                    symbol: Some(sym.clone()),
                                };
                                used_tws = true;
                                debug!(symbol = %sym, "Using TWS yield curve");
                            }
                        }

                        if !used_tws {
                            return finance_rates_result::<CurveResponse>(Err(format!(
                                "no yield curve data for {sym} — yield_curve_writer has not populated KV yet"
                            )));
                        }
                    }
                }

                let mut curve = match build_curve(request, query) {
                    Ok(c) => c,
                    Err(e) => return finance_rates_result::<CurveResponse>(Err(e)),
                };
                let spot = symbol
                    .as_ref()
                    .map(|s| reference_spot_for_report(s))
                    .unwrap_or(DEFAULT_REFERENCE_SPOT);
                if symbol.is_some() {
                    curve.underlying_price = Some(spot);
                }
                fill_missing_strikes(&mut curve, spot);
                let source_label = if used_tws { "TWS" } else { "request" };
                for p in curve.points.iter_mut() {
                    p.data_source = Some(source_label.to_string());
                }
                finance_rates_result(Ok(curve))
            }
        },
        limit_build,
    ));

    let client_compare = client.clone();
    let nc_compare = nc.clone();
    tokio::spawn(handle_sub_parallel(
        nc_compare,
        sub_compare,
        move |body: Option<Vec<u8>>| {
            let client = client_compare.clone();
            async move {
                let (request, query) = parse_compare_body(body.as_deref());
                let r = compare_rates(request, query, &client).await;
                finance_rates_result(r)
            }
        },
        limit,
    ));

    let nc_yield = nc.clone();
    let client_yield = client.clone();
    tokio::spawn(handle_sub_parallel(
        nc_yield,
        sub_yield,
        move |body: Option<Vec<u8>>| {
            let client = client_yield.clone();
            async move {
                let request: YieldCurveComparisonRequest = match body
                    .as_deref()
                    .and_then(|b| serde_json::from_slice(b).ok())
                {
                    Some(r) => r,
                    None => {
                        return serde_json::to_vec(
                            &serde_json::json!({ "error": "request body must be YieldCurveComparisonRequest JSON" }),
                        )
                        .unwrap_or_else(|_| b"{}".to_vec());
                    }
                };
                let response = yield_curve_comparison(request, &client).await;
                serde_json::to_vec(&response).unwrap_or_else(|_| b"{}".to_vec())
            }
        },
        limit,
    ));

    let nc_bench = nc.clone();
    let client_bench = client.clone();
    tokio::spawn(handle_sub_parallel(
        nc_bench,
        sub_benchmarks,
        move |_body: Option<Vec<u8>>| {
            let client = client_bench.clone();
            async move {
                let sofr = get_sofr_rates(&client).await;
                let treasury = get_treasury_rates(&client).await;
                let response = serde_json::json!({
                    "sofr": sofr,
                    "treasury": treasury,
                    "timestamp": chrono::Utc::now().to_rfc3339()
                });
                serde_json::to_vec(&response).unwrap_or_else(|_| b"{}".to_vec())
            }
        },
        limit,
    ));

    let nc_sofr = nc.clone();
    let client_sofr = client.clone();
    tokio::spawn(handle_sub_parallel(
        nc_sofr,
        sub_sofr,
        move |_body: Option<Vec<u8>>| {
            let client = client_sofr.clone();
            async move {
                let response = get_sofr_rates(&client).await;
                serde_json::to_vec(&response).unwrap_or_else(|_| b"{}".to_vec())
            }
        },
        limit,
    ));

    let nc_treasury = nc.clone();
    tokio::spawn(handle_sub_parallel(
        nc_treasury,
        sub_treasury,
        move |_body: Option<Vec<u8>>| {
            let client = client.clone();
            async move {
                let response = get_treasury_rates(&client).await;
                serde_json::to_vec(&response).unwrap_or_else(|_| b"{}".to_vec())
            }
        },
        limit,
    ));
}

fn finance_rates_result<T: serde::Serialize>(r: Result<T, String>) -> Vec<u8> {
    match r {
        Ok(data) => serde_json::to_vec(&data).unwrap_or_else(|_| b"{}".to_vec()),
        Err(e) => serde_json::to_vec(&serde_json::json!({ "error": e }))
            .unwrap_or_else(|_| b"{}".to_vec()),
    }
}

#[cfg(test)]
mod tests {
    use super::helpers::fill_missing_strikes;
    use api::finance_rates::{CurveResponse, RatePointResponse};

    fn point(
        strike_width: f64,
        strike_low: Option<f64>,
        strike_high: Option<f64>,
    ) -> RatePointResponse {
        RatePointResponse {
            symbol: "SPX".to_string(),
            expiry: "2026-04-17".to_string(),
            days_to_expiry: 30,
            strike_width,
            strike_low,
            strike_high,
            buy_implied_rate: 4.4,
            sell_implied_rate: 5.2,
            mid_rate: 4.8,
            net_debit: 80.0,
            net_credit: 80.0,
            liquidity_score: 70.0,
            timestamp: "2026-03-18T00:00:00Z".to_string(),
            spread_id: None,
            convenience_yield: None,
            data_source: None,
        }
    }

    #[test]
    fn fill_missing_strikes_fills_symmetric_strikes_around_spot() {
        let mut curve = CurveResponse {
            symbol: "SPX".to_string(),
            points: vec![point(4.0, None, None), point(4.0, None, None)],
            timestamp: "2026-03-18T00:00:00Z".to_string(),
            strike_width: Some(4.0),
            point_count: 2,
            underlying_price: Some(6000.0),
        };
        fill_missing_strikes(&mut curve, 6000.0);
        for p in &curve.points {
            assert_eq!(
                p.strike_low,
                Some(5998.0),
                "strike_low should be spot - width/2"
            );
            assert_eq!(
                p.strike_high,
                Some(6002.0),
                "strike_high should be spot + width/2"
            );
        }
    }

    #[test]
    fn fill_missing_strikes_leaves_existing_strikes_unchanged() {
        let mut curve = CurveResponse {
            symbol: "SPX".to_string(),
            points: vec![point(4.0, Some(5990.0), Some(5994.0))],
            timestamp: "2026-03-18T00:00:00Z".to_string(),
            strike_width: Some(4.0),
            point_count: 1,
            underlying_price: None,
        };
        fill_missing_strikes(&mut curve, 6000.0);
        assert_eq!(curve.points[0].strike_low, Some(5990.0));
        assert_eq!(curve.points[0].strike_high, Some(5994.0));
    }

    #[test]
    fn fill_missing_strikes_skips_zero_width() {
        let mut curve = CurveResponse {
            symbol: "SPX".to_string(),
            points: vec![point(0.0, None, None)],
            timestamp: "2026-03-18T00:00:00Z".to_string(),
            strike_width: Some(0.0),
            point_count: 1,
            underlying_price: None,
        };
        fill_missing_strikes(&mut curve, 6000.0);
        assert_eq!(curve.points[0].strike_low, None);
        assert_eq!(curve.points[0].strike_high, None);
    }
}
