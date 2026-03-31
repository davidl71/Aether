//! Quantitative calculation NATS request/reply handlers.
//! Subjects: api.calculate.*
//!
//! Parallelism: Uses spawn_cpu_work for CPU-intensive calculations to avoid
//! blocking the async runtime.

use crate::handlers::{api_queue_group, concurrency_limit, handle_sub_parallel, spawn_cpu_work};
use api::quant::{
    calculate_box_spread, calculate_greeks, calculate_historical_volatility, calculate_iv,
    calculate_jelly_roll, calculate_ratio_spread, calculate_risk_metrics, calculate_strategy,
    BoxSpreadRequest, GreeksRequest, HistoricalVolRequest, IvRequest, JellyRollRequest,
    RatioSpreadRequest, RiskMetricsRequest, StrategyRequest,
};
use nats_adapter::async_nats::Client;
use nats_adapter::topics;
use tracing::warn;

fn parse_request<T>(body: Option<Vec<u8>>, err: &'static str) -> Result<T, String>
where
    T: serde::de::DeserializeOwned,
{
    let bytes = body.ok_or_else(|| err.to_string())?;
    serde_json::from_slice(&bytes).map_err(|_| err.to_string())
}

/// Spawn Calculate NATS API handlers with parallel processing.
/// CPU-intensive calculations run on the blocking thread pool.
pub async fn spawn(nc: Client) {
    let limit = concurrency_limit();

    let sub_greeks = match nc
        .queue_subscribe(
            topics::api::calculate::GREEKS.to_string(),
            api_queue_group(),
        )
        .await
    {
        Ok(s) => s,
        Err(e) => {
            warn!(error = %e, "subscribe api.calculate.greeks failed");
            return;
        }
    };
    let sub_iv = match nc
        .queue_subscribe(topics::api::calculate::IV.to_string(), api_queue_group())
        .await
    {
        Ok(s) => s,
        Err(e) => {
            warn!(error = %e, "subscribe api.calculate.iv failed");
            return;
        }
    };
    let sub_hv = match nc
        .queue_subscribe(
            topics::api::calculate::HISTORICAL_VOLATILITY.to_string(),
            api_queue_group(),
        )
        .await
    {
        Ok(s) => s,
        Err(e) => {
            warn!(error = %e, "subscribe api.calculate.historical_volatility failed");
            return;
        }
    };
    let sub_risk = match nc
        .queue_subscribe(
            topics::api::calculate::RISK_METRICS.to_string(),
            api_queue_group(),
        )
        .await
    {
        Ok(s) => s,
        Err(e) => {
            warn!(error = %e, "subscribe api.calculate.risk_metrics failed");
            return;
        }
    };
    let sub_strategy = match nc
        .queue_subscribe(
            topics::api::calculate::STRATEGY.to_string(),
            api_queue_group(),
        )
        .await
    {
        Ok(s) => s,
        Err(e) => {
            warn!(error = %e, "subscribe api.calculate.strategy failed");
            return;
        }
    };
    let sub_box = match nc
        .queue_subscribe(
            topics::api::calculate::BOX_SPREAD.to_string(),
            api_queue_group(),
        )
        .await
    {
        Ok(s) => s,
        Err(e) => {
            warn!(error = %e, "subscribe api.calculate.box_spread failed");
            return;
        }
    };
    let sub_jelly = match nc
        .queue_subscribe(
            topics::api::calculate::JELLY_ROLL.to_string(),
            api_queue_group(),
        )
        .await
    {
        Ok(s) => s,
        Err(e) => {
            warn!(error = %e, "subscribe api.calculate.jelly_roll failed");
            return;
        }
    };
    let sub_ratio = match nc
        .queue_subscribe(
            topics::api::calculate::RATIO_SPREAD.to_string(),
            api_queue_group(),
        )
        .await
    {
        Ok(s) => s,
        Err(e) => {
            warn!(error = %e, "subscribe api.calculate.ratio_spread failed");
            return;
        }
    };

    // Spawn handlers with parallel processing and CPU-bound work offloading
    let nc_greeks = nc.clone();
    tokio::spawn(handle_sub_parallel(
        nc_greeks,
        sub_greeks,
        move |body: Option<Vec<u8>>| async move {
            let request: GreeksRequest = match parse_request(body, "request body must be GreeksRequest JSON") {
                Ok(r) => r,
                Err(e) => return calculate_result::<api::quant::GreeksResponse>(Err(e)),
            };
            // Offload CPU work to blocking pool
            spawn_cpu_work(move || calculate_result(calculate_greeks(&request))).await
        },
        limit,
    ));

    let nc_iv = nc.clone();
    tokio::spawn(handle_sub_parallel(
        nc_iv,
        sub_iv,
        move |body: Option<Vec<u8>>| async move {
            let request: IvRequest = match parse_request(body, "request body must be IvRequest JSON") {
                Ok(r) => r,
                Err(e) => return calculate_result::<api::quant::IvResponse>(Err(e)),
            };
            spawn_cpu_work(move || calculate_result(calculate_iv(&request))).await
        },
        limit,
    ));

    let nc_hv = nc.clone();
    tokio::spawn(handle_sub_parallel(
        nc_hv,
        sub_hv,
        move |body: Option<Vec<u8>>| async move {
            let request: HistoricalVolRequest =
                match parse_request(body, "request body must be HistoricalVolRequest JSON") {
                    Ok(r) => r,
                    Err(e) => return calculate_result::<api::quant::HistoricalVolResponse>(Err(e)),
                };
            spawn_cpu_work(move || calculate_result(calculate_historical_volatility(&request)))
                .await
        },
        limit,
    ));

    let nc_risk = nc.clone();
    tokio::spawn(handle_sub_parallel(
        nc_risk,
        sub_risk,
        move |body: Option<Vec<u8>>| async move {
            let request: RiskMetricsRequest =
                match parse_request(body, "request body must be RiskMetricsRequest JSON") {
                    Ok(r) => r,
                    Err(e) => return calculate_result::<api::quant::RiskMetricsResponse>(Err(e)),
                };
            spawn_cpu_work(move || calculate_result(calculate_risk_metrics(&request))).await
        },
        limit,
    ));

    let nc_strategy = nc.clone();
    tokio::spawn(handle_sub_parallel(
        nc_strategy,
        sub_strategy,
        move |body: Option<Vec<u8>>| async move {
            let request: StrategyRequest =
                match parse_request(body, "request body must be StrategyRequest JSON") {
                    Ok(r) => r,
                    Err(e) => return calculate_result::<api::quant::StrategyResponse>(Err(e)),
                };
            spawn_cpu_work(move || calculate_result(calculate_strategy(&request))).await
        },
        limit,
    ));

    let nc_box = nc.clone();
    tokio::spawn(handle_sub_parallel(
        nc_box,
        sub_box,
        move |body: Option<Vec<u8>>| async move {
            let request: BoxSpreadRequest =
                match parse_request(body, "request body must be BoxSpreadRequest JSON") {
                    Ok(r) => r,
                    Err(e) => return calculate_result::<api::quant::BoxSpreadResponse>(Err(e)),
                };
            spawn_cpu_work(move || calculate_result(calculate_box_spread(&request))).await
        },
        limit,
    ));

    let nc_jelly = nc.clone();
    tokio::spawn(handle_sub_parallel(
        nc_jelly,
        sub_jelly,
        move |body: Option<Vec<u8>>| async move {
            let request: JellyRollRequest =
                match parse_request(body, "request body must be JellyRollRequest JSON") {
                    Ok(r) => r,
                    Err(e) => return calculate_result::<api::quant::JellyRollResponse>(Err(e)),
                };
            spawn_cpu_work(move || calculate_result(calculate_jelly_roll(&request))).await
        },
        limit,
    ));

    let nc_ratio = nc.clone();
    tokio::spawn(handle_sub_parallel(
        nc_ratio,
        sub_ratio,
        move |body: Option<Vec<u8>>| async move {
            let request: RatioSpreadRequest =
                match parse_request(body, "request body must be RatioSpreadRequest JSON") {
                    Ok(r) => r,
                    Err(e) => return calculate_result::<api::quant::RatioSpreadResponse>(Err(e)),
                };
            spawn_cpu_work(move || calculate_result(calculate_ratio_spread(&request))).await
        },
        limit,
    ));
}

fn calculate_result<T: serde::Serialize>(r: Result<T, String>) -> Vec<u8> {
    match r {
        Ok(data) => serde_json::to_vec(&data).unwrap_or_else(|_| b"{}".to_vec()),
        Err(e) => serde_json::to_vec(&serde_json::json!({ "error": e }))
            .unwrap_or_else(|_| b"{}".to_vec()),
    }
}
