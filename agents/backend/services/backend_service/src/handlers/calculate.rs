//! Quantitative calculation NATS request/reply handlers.
//! Subjects: api.calculate.*

use crate::handlers::{api_queue_group, handle_sub};
use api::quant::{
    calculate_box_spread, calculate_greeks, calculate_historical_volatility, calculate_iv,
    calculate_jelly_roll, calculate_ratio_spread, calculate_risk_metrics, calculate_strategy,
    BoxSpreadRequest, GreeksRequest, HistoricalVolRequest, IvRequest, JellyRollRequest,
    RatioSpreadRequest, RiskMetricsRequest, StrategyRequest,
};
use nats_adapter::async_nats::Client;
use tracing::warn;

const SUBJECT_CALCULATE_GREEKS: &str = "api.calculate.greeks";
const SUBJECT_CALCULATE_IV: &str = "api.calculate.iv";
const SUBJECT_CALCULATE_HISTORICAL_VOLATILITY: &str = "api.calculate.historical_volatility";
const SUBJECT_CALCULATE_RISK_METRICS: &str = "api.calculate.risk_metrics";
const SUBJECT_CALCULATE_STRATEGY: &str = "api.calculate.strategy";
const SUBJECT_CALCULATE_BOX_SPREAD: &str = "api.calculate.box_spread";
const SUBJECT_CALCULATE_JELLY_ROLL: &str = "api.calculate.jelly_roll";
const SUBJECT_CALCULATE_RATIO_SPREAD: &str = "api.calculate.ratio_spread";

/// Spawn Calculate NATS API handlers.
pub async fn spawn(nc: Client) {
    let sub_greeks = match nc
        .queue_subscribe(SUBJECT_CALCULATE_GREEKS.to_string(), api_queue_group())
        .await
    {
        Ok(s) => s,
        Err(e) => {
            warn!(error = %e, "subscribe api.calculate.greeks failed");
            return;
        }
    };
    let sub_iv = match nc
        .queue_subscribe(SUBJECT_CALCULATE_IV.to_string(), api_queue_group())
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
            SUBJECT_CALCULATE_HISTORICAL_VOLATILITY.to_string(),
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
            SUBJECT_CALCULATE_RISK_METRICS.to_string(),
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
        .queue_subscribe(SUBJECT_CALCULATE_STRATEGY.to_string(), api_queue_group())
        .await
    {
        Ok(s) => s,
        Err(e) => {
            warn!(error = %e, "subscribe api.calculate.strategy failed");
            return;
        }
    };
    let sub_box = match nc
        .queue_subscribe(SUBJECT_CALCULATE_BOX_SPREAD.to_string(), api_queue_group())
        .await
    {
        Ok(s) => s,
        Err(e) => {
            warn!(error = %e, "subscribe api.calculate.box_spread failed");
            return;
        }
    };
    let sub_jelly = match nc
        .queue_subscribe(SUBJECT_CALCULATE_JELLY_ROLL.to_string(), api_queue_group())
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
            SUBJECT_CALCULATE_RATIO_SPREAD.to_string(),
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

    tokio::spawn(handle_sub(
        nc.clone(),
        sub_greeks,
        |body: Option<Vec<u8>>| async move {
            let request: GreeksRequest =
                match body.as_deref().and_then(|b| serde_json::from_slice(b).ok()) {
                    Some(r) => r,
                    None => {
                        return calculate_result::<api::quant::GreeksResponse>(Err(
                            "request body must be GreeksRequest JSON".to_string(),
                        ))
                    }
                };
            calculate_result(calculate_greeks(&request))
        },
    ));
    tokio::spawn(handle_sub(
        nc.clone(),
        sub_iv,
        |body: Option<Vec<u8>>| async move {
            let request: IvRequest =
                match body.as_deref().and_then(|b| serde_json::from_slice(b).ok()) {
                    Some(r) => r,
                    None => {
                        return calculate_result::<api::quant::IvResponse>(Err(
                            "request body must be IvRequest JSON".to_string(),
                        ))
                    }
                };
            calculate_result(calculate_iv(&request))
        },
    ));
    tokio::spawn(handle_sub(
        nc.clone(),
        sub_hv,
        |body: Option<Vec<u8>>| async move {
            let request: HistoricalVolRequest =
                match body.as_deref().and_then(|b| serde_json::from_slice(b).ok()) {
                    Some(r) => r,
                    None => {
                        return calculate_result::<api::quant::HistoricalVolResponse>(Err(
                            "request body must be HistoricalVolRequest JSON".to_string(),
                        ))
                    }
                };
            calculate_result(calculate_historical_volatility(&request))
        },
    ));
    tokio::spawn(handle_sub(
        nc.clone(),
        sub_risk,
        |body: Option<Vec<u8>>| async move {
            let request: RiskMetricsRequest =
                match body.as_deref().and_then(|b| serde_json::from_slice(b).ok()) {
                    Some(r) => r,
                    None => {
                        return calculate_result::<api::quant::RiskMetricsResponse>(Err(
                            "request body must be RiskMetricsRequest JSON".to_string(),
                        ))
                    }
                };
            calculate_result(calculate_risk_metrics(&request))
        },
    ));
    tokio::spawn(handle_sub(
        nc.clone(),
        sub_strategy,
        |body: Option<Vec<u8>>| async move {
            let request: StrategyRequest =
                match body.as_deref().and_then(|b| serde_json::from_slice(b).ok()) {
                    Some(r) => r,
                    None => {
                        return calculate_result::<api::quant::StrategyResponse>(Err(
                            "request body must be StrategyRequest JSON".to_string(),
                        ))
                    }
                };
            calculate_result(calculate_strategy(&request))
        },
    ));
    tokio::spawn(handle_sub(
        nc.clone(),
        sub_box,
        |body: Option<Vec<u8>>| async move {
            let request: BoxSpreadRequest =
                match body.as_deref().and_then(|b| serde_json::from_slice(b).ok()) {
                    Some(r) => r,
                    None => {
                        return calculate_result::<api::quant::BoxSpreadResponse>(Err(
                            "request body must be BoxSpreadRequest JSON".to_string(),
                        ))
                    }
                };
            calculate_result(calculate_box_spread(&request))
        },
    ));
    tokio::spawn(handle_sub(
        nc.clone(),
        sub_jelly,
        |body: Option<Vec<u8>>| async move {
            let request: JellyRollRequest =
                match body.as_deref().and_then(|b| serde_json::from_slice(b).ok()) {
                    Some(r) => r,
                    None => {
                        return calculate_result::<api::quant::JellyRollResponse>(Err(
                            "request body must be JellyRollRequest JSON".to_string(),
                        ))
                    }
                };
            calculate_result(calculate_jelly_roll(&request))
        },
    ));
    tokio::spawn(handle_sub(
        nc.clone(),
        sub_ratio,
        |body: Option<Vec<u8>>| async move {
            let request: RatioSpreadRequest =
                match body.as_deref().and_then(|b| serde_json::from_slice(b).ok()) {
                    Some(r) => r,
                    None => {
                        return calculate_result::<api::quant::RatioSpreadResponse>(Err(
                            "request body must be RatioSpreadRequest JSON".to_string(),
                        ))
                    }
                };
            calculate_result(calculate_ratio_spread(&request))
        },
    ));
}

fn calculate_result<T: serde::Serialize>(r: Result<T, String>) -> Vec<u8> {
    match r {
        Ok(data) => serde_json::to_vec(&data).unwrap_or_else(|_| b"{}".to_vec()),
        Err(e) => serde_json::to_vec(&serde_json::json!({ "error": e }))
            .unwrap_or_else(|_| b"{}".to_vec()),
    }
}
