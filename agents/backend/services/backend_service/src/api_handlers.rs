//! NATS request/reply handlers for api.discount_bank.*, api.loans.*, api.fmp.*, and api.strategy.*.
//! Scope per docs/platform/NATS_API.md §3.
//! FMP fundamentals wired when FMP_API_KEY is set (task T-1773509396765766000).
//! Strategy start/stop: api.strategy.start, api.strategy.stop (task T-1773515657237625000).

use std::sync::Arc;

use api::discount_bank::{get_balance, get_bank_accounts, get_transactions, ImportPositionsQuery};
use api::fetch_ib_positions;
use api::finance_rates::{
    build_curve, compare_rates, extract_rate, get_sofr_rates, get_treasury_rates,
    yield_curve_comparison, BoxSpreadInput, CompareRequest, CurveQuery, CurveRequest,
    CurveResponse, YieldCurveComparisonRequest,
};
use api::loans::loans_response_proto;
use api::mock_data::{
    mock_discount_bank_accounts, mock_discount_bank_balance, mock_discount_bank_transactions,
    mock_fmp_balance_sheet, mock_fmp_cash_flow, mock_fmp_income_statement, mock_fmp_quote,
    mock_loans_list, mock_sofr_benchmarks, mock_treasury_benchmarks,
};
use api::quant::{
    calculate_box_spread, calculate_greeks, calculate_historical_volatility, calculate_iv,
    calculate_jelly_roll, calculate_ratio_spread, calculate_risk_metrics, calculate_strategy,
    BoxSpreadRequest, GreeksRequest, HistoricalVolRequest, IvRequest, JellyRollRequest,
    RatioSpreadRequest, RiskMetricsRequest, StrategyRequest,
};
use api::{LoanRecord, LoanRepository, SharedSnapshot, StrategyController};
use bytes::Bytes;
use futures::StreamExt;
use market_data::FmpClient;
use nats_adapter::async_nats::Client;
use nats_adapter::{encode_envelope, topics, NatsClient};
use prost::Message;
use reqwest::Client as ReqwestClient;
use serde_json::Value;
use tracing::{debug, info, warn};
use tws_yield_curve;

const SUBJECT_DISCOUNT_BANK_BALANCE: &str = "api.discount_bank.balance";
const SUBJECT_DISCOUNT_BANK_TRANSACTIONS: &str = "api.discount_bank.transactions";
const SUBJECT_DISCOUNT_BANK_BANK_ACCOUNTS: &str = "api.discount_bank.bank_accounts";
const SUBJECT_DISCOUNT_BANK_IMPORT_POSITIONS: &str = "api.discount_bank.import_positions";
const SUBJECT_LOANS_LIST: &str = "api.loans.list";
const SUBJECT_LOANS_LIST_PROTO: &str = "api.loans.list.proto";
const SUBJECT_LOANS_GET: &str = "api.loans.get";
const SUBJECT_LOANS_CREATE: &str = "api.loans.create";
const SUBJECT_LOANS_UPDATE: &str = "api.loans.update";
const SUBJECT_LOANS_DELETE: &str = "api.loans.delete";

const SUBJECT_FMP_INCOME_STATEMENT: &str = "api.fmp.income_statement";
const SUBJECT_FMP_BALANCE_SHEET: &str = "api.fmp.balance_sheet";
const SUBJECT_FMP_CASH_FLOW: &str = "api.fmp.cash_flow";
const SUBJECT_FMP_QUOTE: &str = "api.fmp.quote";

const SUBJECT_STRATEGY_START: &str = "api.strategy.start";
const SUBJECT_STRATEGY_STOP: &str = "api.strategy.stop";
const SUBJECT_STRATEGY_CANCEL_ALL: &str = "api.strategy.cancel_all";
const SUBJECT_STRATEGY_EXECUTE: &str = "api.strategy.execute";
const SUBJECT_ADMIN_SET_MODE: &str = "api.admin.set_mode";

const SUBJECT_IB_POSITIONS: &str = "api.ib.positions";

const SUBJECT_FINANCE_RATES_EXTRACT: &str = "api.finance_rates.extract";
const SUBJECT_FINANCE_RATES_BUILD_CURVE: &str = "api.finance_rates.build_curve";
const SUBJECT_FINANCE_RATES_COMPARE: &str = "api.finance_rates.compare";
const SUBJECT_FINANCE_RATES_YIELD_CURVE: &str = "api.finance_rates.yield_curve";
const SUBJECT_FINANCE_RATES_BENCHMARKS: &str = "api.finance_rates.benchmarks";
const SUBJECT_FINANCE_RATES_SOFR: &str = "api.finance_rates.sofr";
const SUBJECT_FINANCE_RATES_TREASURY: &str = "api.finance_rates.treasury";
const SUBJECT_YIELD_CURVE_REFRESH: &str = "api.yield_curve.refresh";

const SUBJECT_CALCULATE_GREEKS: &str = "api.calculate.greeks";
const SUBJECT_CALCULATE_IV: &str = "api.calculate.iv";
const SUBJECT_CALCULATE_HISTORICAL_VOLATILITY: &str = "api.calculate.historical_volatility";
const SUBJECT_CALCULATE_RISK_METRICS: &str = "api.calculate.risk_metrics";
const SUBJECT_CALCULATE_STRATEGY: &str = "api.calculate.strategy";
const SUBJECT_CALCULATE_BOX_SPREAD: &str = "api.calculate.box_spread";
const SUBJECT_CALCULATE_JELLY_ROLL: &str = "api.calculate.jelly_roll";
const SUBJECT_CALCULATE_RATIO_SPREAD: &str = "api.calculate.ratio_spread";

const SUBJECT_SNAPSHOT_PUBLISH_NOW: &str = "api.snapshot.publish_now";

/// Default queue group for api.* request/reply when scaling multiple backends. Override with NATS_API_QUEUE_GROUP.
const DEFAULT_API_QUEUE_GROUP: &str = "api";

fn api_queue_group() -> String {
    std::env::var("NATS_API_QUEUE_GROUP").unwrap_or_else(|_| DEFAULT_API_QUEUE_GROUP.into())
}

/// Spawn NATS API handlers for Discount Bank, Loans, optionally FMP fundamentals, finance rates, calculate, and strategy control.
pub fn spawn(
    nats_client: Arc<NatsClient>,
    loan_repo: Option<Arc<LoanRepository>>,
    fmp_client: Option<Arc<FmpClient>>,
    strategy_controller: StrategyController,
    state: SharedSnapshot,
    yield_curve_refresh_tx: Option<tokio::sync::mpsc::Sender<()>>,
) {
    let nc = nats_client.client().clone();
    let nc_loans = nc.clone();
    let nc_strategy = nats_client.client().clone();
    let nc_snapshot = nats_client.client().clone();
    let state_snapshot = state.clone();
    tokio::spawn(async move {
        run_snapshot_publish_now(nc_snapshot, state_snapshot).await;
    });
    tokio::spawn(async move {
        run_discount_bank(nc).await;
    });
    tokio::spawn(async move {
        run_loans(nc_loans, loan_repo).await;
    });
    let state_strategy = state.clone();
    tokio::spawn(async move {
        run_strategy_control(nc_strategy, strategy_controller, state_strategy).await;
    });
    let nc_finance = nats_client.client().clone();
    tokio::spawn(async move {
        run_finance_rates(nc_finance, yield_curve_refresh_tx).await;
    });
    let nc_calculate = nats_client.client().clone();
    tokio::spawn(async move {
        run_calculate(nc_calculate).await;
    });
    if let Some(fmp) = fmp_client {
        let nc_fmp = nats_client.client().clone();
        tokio::spawn(async move {
            run_fmp(nc_fmp, fmp).await;
        });
        info!("NATS API handlers spawned (discount_bank, loans, fmp, finance_rates, calculate, strategy, ib.positions)");
    } else {
        let nc_fmp = nats_client.client().clone();
        tokio::spawn(async move {
            run_fmp_mock(nc_fmp).await;
        });
        info!("NATS API handlers spawned (discount_bank, loans, fmp_mock, finance_rates, calculate, strategy, ib.positions)");
    }
    let nc_ib = nats_client.client().clone();
    tokio::spawn(async move {
        run_ib_positions(nc_ib).await;
    });
    let nc_admin = nats_client.client().clone();
    tokio::spawn(async move {
        run_admin_set_mode(nc_admin, state).await;
    });
}

async fn run_strategy_control(nc: Client, controller: StrategyController, state: SharedSnapshot) {
    let sub_start = match nc
        .queue_subscribe(SUBJECT_STRATEGY_START.to_string(), api_queue_group())
        .await
    {
        Ok(s) => s,
        Err(e) => {
            warn!(error = %e, "subscribe api.strategy.start failed");
            return;
        }
    };
    let sub_stop = match nc
        .queue_subscribe(SUBJECT_STRATEGY_STOP.to_string(), api_queue_group())
        .await
    {
        Ok(s) => s,
        Err(e) => {
            warn!(error = %e, "subscribe api.strategy.stop failed");
            return;
        }
    };
    let sub_cancel_all = match nc
        .queue_subscribe(SUBJECT_STRATEGY_CANCEL_ALL.to_string(), api_queue_group())
        .await
    {
        Ok(s) => s,
        Err(e) => {
            warn!(error = %e, "subscribe api.strategy.cancel_all failed");
            return;
        }
    };
    let sub_execute = match nc
        .queue_subscribe(SUBJECT_STRATEGY_EXECUTE.to_string(), api_queue_group())
        .await
    {
        Ok(s) => s,
        Err(e) => {
            warn!(error = %e, "subscribe api.strategy.execute failed");
            return;
        }
    };

    let c_start = controller.clone();
    tokio::spawn(handle_sub(
        nc.clone(),
        sub_start,
        move |_body: Option<Vec<u8>>| {
            let c = c_start.clone();
            async move {
                let result = c.start();
                strategy_reply(result)
            }
        },
    ));
    let c_stop = controller.clone();
    tokio::spawn(handle_sub(
        nc.clone(),
        sub_stop,
        move |_body: Option<Vec<u8>>| {
            let c = c_stop.clone();
            async move {
                let result = c.stop();
                strategy_reply(result)
            }
        },
    ));
    let state_cancel = state.clone();
    tokio::spawn(handle_sub(
        nc,
        sub_cancel_all,
        move |_body: Option<Vec<u8>>| {
            let state = state_cancel.clone();
            async move { cancel_all_reply(state).await }
        },
    ));
    tokio::spawn(handle_sub(
        nc,
        sub_execute,
        move |_body: Option<Vec<u8>>| async move { execute_scenario_reply().await },
    ));
}

/// Handles api.strategy.execute: places orders for a scenario.
/// Currently a stub — requires broker engine to be wired into backend_service.
async fn execute_scenario_reply() -> Vec<u8> {
    serde_json::to_vec(&serde_json::json!({
        "ok": false,
        "error": "scenario execution not wired: backend_service requires Arc<dyn BrokerEngine> to place bag orders"
    }))
    .unwrap_or_else(|_| b"{}".to_vec())
}

/// Handles api.admin.set_mode: set snapshot.mode to LIVE, MOCK, or DRY-RUN for quick display switch.
async fn run_admin_set_mode(nc: Client, state: SharedSnapshot) {
    let sub = match nc
        .queue_subscribe(SUBJECT_ADMIN_SET_MODE.to_string(), api_queue_group())
        .await
    {
        Ok(s) => s,
        Err(e) => {
            warn!(error = %e, "subscribe api.admin.set_mode failed");
            return;
        }
    };
    let state = state.clone();
    tokio::spawn(handle_sub(nc, sub, move |body: Option<Vec<u8>>| {
        let state = state.clone();
        async move {
            #[derive(serde::Deserialize)]
            struct SetModeRequest {
                mode: String,
            }
            let mode = body
                .as_deref()
                .and_then(|b| serde_json::from_slice::<SetModeRequest>(b).ok())
                .map(|r| r.mode.to_uppercase());
            let mode = match mode.as_deref() {
                Some("LIVE") | Some("MOCK") | Some("DRY-RUN") | Some("TUI") => mode.unwrap(),
                _ => {
                    let err = serde_json::json!({ "ok": false, "error": "mode must be LIVE, MOCK, or DRY-RUN" });
                    return serde_json::to_vec(&err).unwrap_or_else(|_| b"{}".to_vec());
                }
            };
            let mode_label = if mode == "TUI" {
                "DRY-RUN"
            } else {
                mode.as_str()
            };
            {
                let mut snap = state.write().await;
                snap.mode = mode_label.to_string();
                snap.touch();
            }
            let msg = format!("mode set to {}", mode_label);
            info!(mode = %mode_label, "admin set_mode");
            serde_json::to_vec(&serde_json::json!({ "ok": true, "message": msg }))
                .unwrap_or_else(|_| b"{}".to_vec())
        }
    }));
}

/// Force-write current snapshot to NATS (point-in-time). Subscribes to api.snapshot.publish_now;
/// on request, publishes current SystemSnapshot to snapshot.{backend_id} and replies with ok + generated_at.
async fn run_snapshot_publish_now(nc: Client, state: SharedSnapshot) {
    let mut sub = match nc
        .queue_subscribe(SUBJECT_SNAPSHOT_PUBLISH_NOW.to_string(), api_queue_group())
        .await
    {
        Ok(s) => s,
        Err(e) => {
            warn!(error = %e, "subscribe api.snapshot.publish_now failed");
            return;
        }
    };
    info!("subscribed to api.snapshot.publish_now (force snapshot write)");
    let backend_id = std::env::var("BACKEND_ID").unwrap_or_else(|_| "ib".to_string());
    let subject = topics::snapshot::backend(&backend_id);
    while let Some(msg) = sub.next().await {
        let reply = match msg.reply {
            Some(r) => r,
            None => continue,
        };
        let (proto, generated_at) = {
            let snap = state.read().await;
            let proto = api::snapshot_proto::snapshot_to_proto(&snap);
            let ts = snap.generated_at;
            (proto, ts)
        };
        let response = match encode_envelope("backend_service", "SystemSnapshot", &proto) {
            Ok(bytes) => {
                if let Err(e) = nc.publish(subject.clone(), bytes.into()).await {
                    warn!(error = %e, subject = %subject, "publish snapshot failed");
                    serde_json::to_vec(&serde_json::json!({ "ok": false, "error": e.to_string() }))
                        .unwrap_or_else(|_| b"{}".to_vec())
                } else {
                    serde_json::to_vec(&serde_json::json!({
                        "ok": true,
                        "generated_at": generated_at.to_rfc3339(),
                        "subject": subject
                    }))
                    .unwrap_or_else(|_| b"{}".to_vec())
                }
            }
            Err(e) => {
                serde_json::to_vec(&serde_json::json!({ "ok": false, "error": e.to_string() }))
                    .unwrap_or_else(|_| b"{}".to_vec())
            }
        };
        if let Err(e) = nc.publish(reply, Bytes::from(response)).await {
            warn!(error = %e, "reply to api.snapshot.publish_now failed");
        }
    }
}

/// Reply for api.strategy.cancel_all: reads snapshot for open orders; broker cancel not wired yet.
async fn cancel_all_reply(state: SharedSnapshot) -> Vec<u8> {
    let open_count = {
        let snap = state.read().await;
        snap.orders
            .iter()
            .filter(|o| {
                let s = o.status.to_uppercase();
                !matches!(
                    s.as_str(),
                    "FILLED" | "CANCELLED" | "CANCELED" | "REJECTED" | "INACTIVE" | "EXPIRED"
                )
            })
            .count()
    };
    let message = format!(
        "{} open order(s); cancel-all received (broker not wired)",
        open_count
    );
    serde_json::to_vec(&serde_json::json!({ "ok": true, "message": message }))
        .unwrap_or_else(|_| b"{}".to_vec())
}

fn strategy_reply(result: Result<(), tokio::sync::watch::error::SendError<bool>>) -> Vec<u8> {
    match result {
        Ok(()) => serde_json::to_vec(&serde_json::json!({ "ok": true }))
            .unwrap_or_else(|_| b"{}".to_vec()),
        Err(e) => serde_json::to_vec(&serde_json::json!({ "ok": false, "error": e.to_string() }))
            .unwrap_or_else(|_| b"{}".to_vec()),
    }
}

fn finance_rates_result<T: serde::Serialize>(r: Result<T, String>) -> Vec<u8> {
    match r {
        Ok(data) => serde_json::to_vec(&data).unwrap_or_else(|_| b"{}".to_vec()),
        Err(e) => serde_json::to_vec(&serde_json::json!({ "error": e }))
            .unwrap_or_else(|_| b"{}".to_vec()),
    }
}

async fn run_finance_rates(
    nc: Client,
    yield_curve_refresh_tx: Option<tokio::sync::mpsc::Sender<()>>,
) {
    // Timeout so FRED/New York Fed calls don't hang and cause NATS request timeouts
    let client = ReqwestClient::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .unwrap_or_else(|_| ReqwestClient::new());

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

    tokio::spawn(handle_sub(
        nc.clone(),
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
    ));

    let nc_build = nc.clone();
    tokio::spawn(handle_sub(
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
                let mut loaded_from_kv = false;
                let mut used_synthetic = false;
                let mut used_tws = false;
                if let Some(ref sym) = symbol {
                    let is_empty = match &request {
                        CurveRequest::Opportunities(opps) => opps.is_empty(),
                        CurveRequest::Named { opportunities, .. } => opportunities.is_empty(),
                    };
                    if is_empty {
                        // Live TWS: build curve from option chain + option quotes (bypass KV).
                        let use_tws = std::env::var("YIELD_CURVE_USE_TWS")
                            .ok()
                            .map(|v| matches!(v.to_lowercase().as_str(), "1" | "true" | "yes"))
                            .unwrap_or(false);
                        if use_tws {
                            if let Ok(opportunities) =
                                tws_yield_curve::fetch_yield_curve_from_tws(sym).await
                            {
                                if !opportunities.is_empty() {
                                    request = CurveRequest::Named {
                                        opportunities,
                                        symbol: Some(sym.clone()),
                                    };
                                    used_tws = true;
                                }
                            }
                        }
                        if !used_tws {
                            // Testing: bypass KV and use in-process synthetic curve (same algorithm as writer).
                            let bypass_kv = std::env::var("YIELD_CURVE_BYPASS_KV")
                                .ok()
                                .map(|v| matches!(v.to_lowercase().as_str(), "1" | "true" | "yes"))
                                .unwrap_or(false);
                            if bypass_kv {
                                let opportunities =
                                    crate::yield_curve_writer::synthetic_opportunities(sym);
                                if !opportunities.is_empty() {
                                    request = CurveRequest::Named {
                                        opportunities,
                                        symbol: Some(sym.clone()),
                                    };
                                    used_synthetic = true;
                                }
                            }
                        }
                        if !used_tws && !used_synthetic {
                            if let Some(curve_from_kv) =
                                load_yield_curve_from_kv(&nc_build, sym, query.as_ref()).await
                            {
                                loaded_from_kv = true;
                                let spot = symbol
                                    .as_ref()
                                    .map(|s| reference_spot_for_report(s))
                                    .unwrap_or(DEFAULT_REFERENCE_SPOT);
                                let mut curve = curve_from_kv;
                                if symbol.is_some() {
                                    curve.underlying_price = Some(spot);
                                }
                                fill_missing_strikes(&mut curve, spot);
                                for p in curve.points.iter_mut() {
                                    p.data_source = Some("KV".to_string());
                                }
                                return finance_rates_result(Ok(curve));
                            }
                        }
                    }
                }
                let mut curve = match build_curve(request, query) {
                    Ok(c) => c,
                    Err(e) => {
                        return finance_rates_result::<api::finance_rates::CurveResponse>(Err(e))
                    }
                };
                let spot = symbol
                    .as_ref()
                    .map(|s| reference_spot_for_report(s))
                    .unwrap_or(DEFAULT_REFERENCE_SPOT);
                if symbol.is_some() {
                    curve.underlying_price = Some(spot);
                }
                fill_missing_strikes(&mut curve, spot);
                let source_label = if used_tws {
                    "TWS"
                } else if used_synthetic {
                    "synthetic"
                } else if loaded_from_kv {
                    "KV"
                } else {
                    "request"
                };
                for p in curve.points.iter_mut() {
                    p.data_source = Some(source_label.to_string());
                }
                finance_rates_result(Ok(curve))
            }
        },
    ));

    let client_compare = client.clone();
    tokio::spawn(handle_sub(
        nc.clone(),
        sub_compare,
        move |body: Option<Vec<u8>>| {
            let client = client_compare.clone();
            async move {
                let (request, query) = parse_compare_body(body.as_deref());
                let r = compare_rates(request, query, &client).await;
                finance_rates_result(r)
            }
        },
    ));

    let client_yield = client.clone();
    tokio::spawn(handle_sub(
        nc.clone(),
        sub_yield,
        move |body: Option<Vec<u8>>| {
            let client = client_yield.clone();
            async move {
                let request: YieldCurveComparisonRequest = match body
                .as_deref()
                .and_then(|b| serde_json::from_slice(b).ok())
            {
                Some(r) => r,
                None => return serde_json::to_vec(&serde_json::json!({ "error": "request body must be YieldCurveComparisonRequest JSON" }))
                    .unwrap_or_else(|_| b"{}".to_vec()),
            };
                let response = yield_curve_comparison(request, &client).await;
                serde_json::to_vec(&response).unwrap_or_else(|_| b"{}".to_vec())
            }
        },
    ));

    let client_bench = client.clone();
    tokio::spawn(handle_sub(
        nc.clone(),
        sub_benchmarks,
        move |_body: Option<Vec<u8>>| {
            let client = client_bench.clone();
            async move {
                let mut sofr = get_sofr_rates(&client).await;
                let mut treasury = get_treasury_rates(&client).await;
                if sofr.term_rates.is_empty() && sofr.overnight.rate.is_none() {
                    sofr = mock_sofr_benchmarks();
                }
                if treasury.rates.is_empty() {
                    treasury = mock_treasury_benchmarks();
                }
                let response = serde_json::json!({ "sofr": sofr, "treasury": treasury });
                serde_json::to_vec(&response).unwrap_or_else(|_| b"{}".to_vec())
            }
        },
    ));

    let client_sofr = client.clone();
    tokio::spawn(handle_sub(
        nc.clone(),
        sub_sofr,
        move |_body: Option<Vec<u8>>| {
            let client = client_sofr.clone();
            async move {
                let mut response = get_sofr_rates(&client).await;
                if response.term_rates.is_empty() && response.overnight.rate.is_none() {
                    response = mock_sofr_benchmarks();
                }
                serde_json::to_vec(&response).unwrap_or_else(|_| b"{}".to_vec())
            }
        },
    ));

    tokio::spawn(handle_sub(
        nc,
        sub_treasury,
        move |_body: Option<Vec<u8>>| {
            let client = client.clone();
            async move {
                let mut response = get_treasury_rates(&client).await;
                if response.rates.is_empty() {
                    response = mock_treasury_benchmarks();
                }
                serde_json::to_vec(&response).unwrap_or_else(|_| b"{}".to_vec())
            }
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

async fn run_calculate(nc: Client) {
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

/// KV key for box spread opportunities per symbol (real yield curve).
/// Value: JSON array of objects with "spread" key (BoxSpreadInput).
const KV_KEY_PREFIX_YIELD_CURVE: &str = "yield_curve";

const REFERENCE_SPOT_ENV_PREFIX: &str = "YIELD_CURVE_REFERENCE_SPOT_";
const DEFAULT_REFERENCE_SPOT: f64 = 6000.0;

/// Reference/underlying price for symbol (env YIELD_CURVE_REFERENCE_SPOT_{SYMBOL} or default). Used for report display.
fn reference_spot_for_report(symbol: &str) -> f64 {
    let key = format!("{}{}", REFERENCE_SPOT_ENV_PREFIX, symbol.to_uppercase());
    std::env::var(&key)
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(DEFAULT_REFERENCE_SPOT)
}

/// Fill strike_low/strike_high on curve points when missing (e.g. old KV or URL source without strikes), using symmetric ±width/2 around spot.
fn fill_missing_strikes(curve: &mut CurveResponse, spot: f64) {
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

/// Load yield curve from NATS KV for a symbol. Tries proto decode first (YieldCurve), then JSON array fallback.
/// Bucket from NATS_KV_BUCKET (default LIVE_STATE). Key: yield_curve.{symbol}.
async fn load_yield_curve_from_kv(
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
    if let Some(curve) = api::yield_curve_proto::curve_response_from_proto_bytes(&bytes, symbol) {
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

fn parse_curve_body(body: Option<&[u8]>) -> (CurveRequest, Option<CurveQuery>) {
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

fn parse_compare_body(body: Option<&[u8]>) -> (CompareRequest, Option<CurveQuery>) {
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

async fn run_ib_positions(nc: Client) {
    let sub = match nc
        .queue_subscribe(SUBJECT_IB_POSITIONS.to_string(), api_queue_group())
        .await
    {
        Ok(s) => s,
        Err(e) => {
            warn!(error = %e, "subscribe api.ib.positions failed");
            return;
        }
    };
    tokio::spawn(handle_sub(nc, sub, |body: Option<Vec<u8>>| async move {
        let account_id: Option<String> = body
            .as_deref()
            .and_then(|b| serde_json::from_slice::<Value>(b).ok())
            .and_then(|v| {
                v.get("account_id")
                    .and_then(Value::as_str)
                    .map(str::to_string)
            });
        let result = fetch_ib_positions(account_id.as_deref()).await;
        match result {
            Ok(positions) => serde_json::to_vec(&positions).unwrap_or_else(|_| b"[]".to_vec()),
            Err(e) => serde_json::to_vec(&serde_json::json!({ "error": e }))
                .unwrap_or_else(|_| b"{}".to_vec()),
        }
    }));
}

async fn run_discount_bank(nc: Client) {
    let sub_balance = match nc
        .queue_subscribe(SUBJECT_DISCOUNT_BANK_BALANCE.to_string(), api_queue_group())
        .await
    {
        Ok(s) => s,
        Err(e) => {
            warn!(error = %e, "subscribe api.discount_bank.balance failed");
            return;
        }
    };
    let sub_tx = match nc
        .queue_subscribe(
            SUBJECT_DISCOUNT_BANK_TRANSACTIONS.to_string(),
            api_queue_group(),
        )
        .await
    {
        Ok(s) => s,
        Err(e) => {
            warn!(error = %e, "subscribe api.discount_bank.transactions failed");
            return;
        }
    };
    let sub_accounts = match nc
        .queue_subscribe(
            SUBJECT_DISCOUNT_BANK_BANK_ACCOUNTS.to_string(),
            api_queue_group(),
        )
        .await
    {
        Ok(s) => s,
        Err(e) => {
            warn!(error = %e, "subscribe api.discount_bank.bank_accounts failed");
            return;
        }
    };
    let sub_import = match nc
        .queue_subscribe(
            SUBJECT_DISCOUNT_BANK_IMPORT_POSITIONS.to_string(),
            api_queue_group(),
        )
        .await
    {
        Ok(s) => s,
        Err(e) => {
            warn!(error = %e, "subscribe api.discount_bank.import_positions failed");
            return;
        }
    };

    let client = ReqwestClient::new();

    tokio::spawn(handle_sub(
        nc.clone(),
        sub_balance,
        |_body: Option<Vec<u8>>| async {
            let r: Result<api::discount_bank::DiscountBankBalanceDto, String> = get_balance()
                .await
                .or_else(|_| Ok(mock_discount_bank_balance()));
            serde_json::to_vec(&r).unwrap_or_else(|_| b"{}".to_vec())
        },
    ));
    tokio::spawn(handle_sub(
        nc.clone(),
        sub_tx,
        |body: Option<Vec<u8>>| async move {
            let limit = body
                .as_deref()
                .and_then(|b| serde_json::from_slice::<Value>(b).ok())
                .and_then(|v| v.get("limit").and_then(Value::as_u64))
                .unwrap_or(100) as usize;
            let r: Result<api::discount_bank::DiscountBankTransactionsListDto, String> =
                get_transactions(limit)
                    .await
                    .or_else(|_| Ok(mock_discount_bank_transactions(limit)));
            serde_json::to_vec(&r).unwrap_or_else(|_| b"{}".to_vec())
        },
    ));
    tokio::spawn(handle_sub(
        nc.clone(),
        sub_accounts,
        |_body: Option<Vec<u8>>| async {
            let r: Result<api::discount_bank::BankAccountsListDto, String> = get_bank_accounts()
                .await
                .or_else(|_| Ok(mock_discount_bank_accounts()));
            serde_json::to_vec(&r).unwrap_or_else(|_| b"{}".to_vec())
        },
    ));
    tokio::spawn(handle_sub_with_client(nc, sub_import, client));
}

async fn run_fmp(nc: Client, fmp: Arc<FmpClient>) {
    let sub_income = match nc
        .queue_subscribe(SUBJECT_FMP_INCOME_STATEMENT.to_string(), api_queue_group())
        .await
    {
        Ok(s) => s,
        Err(e) => {
            warn!(error = %e, "subscribe api.fmp.income_statement failed");
            return;
        }
    };
    let sub_balance = match nc
        .queue_subscribe(SUBJECT_FMP_BALANCE_SHEET.to_string(), api_queue_group())
        .await
    {
        Ok(s) => s,
        Err(e) => {
            warn!(error = %e, "subscribe api.fmp.balance_sheet failed");
            return;
        }
    };
    let sub_cash = match nc
        .queue_subscribe(SUBJECT_FMP_CASH_FLOW.to_string(), api_queue_group())
        .await
    {
        Ok(s) => s,
        Err(e) => {
            warn!(error = %e, "subscribe api.fmp.cash_flow failed");
            return;
        }
    };
    let sub_quote = match nc
        .queue_subscribe(SUBJECT_FMP_QUOTE.to_string(), api_queue_group())
        .await
    {
        Ok(s) => s,
        Err(e) => {
            warn!(error = %e, "subscribe api.fmp.quote failed");
            return;
        }
    };

    let fmp_income = fmp.clone();
    tokio::spawn(handle_sub(
        nc.clone(),
        sub_income,
        move |body: Option<Vec<u8>>| {
            let fmp = fmp_income.clone();
            async move {
                let (symbol, limit) = parse_fmp_request(body.as_deref());
                let r = fmp.income_statement(&symbol, limit).await;
                fmp_response(r)
            }
        },
    ));
    let fmp_balance = fmp.clone();
    tokio::spawn(handle_sub(
        nc.clone(),
        sub_balance,
        move |body: Option<Vec<u8>>| {
            let fmp = fmp_balance.clone();
            async move {
                let (symbol, limit) = parse_fmp_request(body.as_deref());
                let r = fmp.balance_sheet(&symbol, limit).await;
                fmp_response(r)
            }
        },
    ));
    let fmp_cash = fmp.clone();
    tokio::spawn(handle_sub(
        nc.clone(),
        sub_cash,
        move |body: Option<Vec<u8>>| {
            let fmp = fmp_cash.clone();
            async move {
                let (symbol, limit) = parse_fmp_request(body.as_deref());
                let r = fmp.cash_flow(&symbol, limit).await;
                fmp_response(r)
            }
        },
    ));
    let fmp_quote = fmp.clone();
    tokio::spawn(handle_sub(nc, sub_quote, move |body: Option<Vec<u8>>| {
        let fmp = fmp_quote.clone();
        async move {
            let (symbol, _) = parse_fmp_request(body.as_deref());
            let r = fmp.quote(&symbol).await;
            fmp_response(r)
        }
    }));
}

fn fmp_response<T: serde::Serialize>(r: anyhow::Result<T>) -> Vec<u8> {
    match r {
        Ok(data) => serde_json::to_vec(&data).unwrap_or_else(|_| b"{}".to_vec()),
        Err(e) => serde_json::to_vec(&serde_json::json!({ "error": e.to_string() }))
            .unwrap_or_else(|_| b"{}".to_vec()),
    }
}

async fn run_fmp_mock(nc: Client) {
    let sub_income = match nc
        .queue_subscribe(SUBJECT_FMP_INCOME_STATEMENT.to_string(), api_queue_group())
        .await
    {
        Ok(s) => s,
        Err(e) => {
            warn!(error = %e, "subscribe api.fmp.income_statement (mock) failed");
            return;
        }
    };
    let sub_balance = match nc
        .queue_subscribe(SUBJECT_FMP_BALANCE_SHEET.to_string(), api_queue_group())
        .await
    {
        Ok(s) => s,
        Err(e) => {
            warn!(error = %e, "subscribe api.fmp.balance_sheet (mock) failed");
            return;
        }
    };
    let sub_cash = match nc
        .queue_subscribe(SUBJECT_FMP_CASH_FLOW.to_string(), api_queue_group())
        .await
    {
        Ok(s) => s,
        Err(e) => {
            warn!(error = %e, "subscribe api.fmp.cash_flow (mock) failed");
            return;
        }
    };
    let sub_quote = match nc
        .queue_subscribe(SUBJECT_FMP_QUOTE.to_string(), api_queue_group())
        .await
    {
        Ok(s) => s,
        Err(e) => {
            warn!(error = %e, "subscribe api.fmp.quote (mock) failed");
            return;
        }
    };

    tokio::spawn(handle_sub(
        nc.clone(),
        sub_income,
        |body: Option<Vec<u8>>| async move {
            let (symbol, limit) = parse_fmp_request(body.as_deref());
            let r = mock_fmp_income_statement(&symbol, limit);
            fmp_response(Ok(r))
        },
    ));
    tokio::spawn(handle_sub(
        nc.clone(),
        sub_balance,
        |body: Option<Vec<u8>>| async move {
            let (symbol, limit) = parse_fmp_request(body.as_deref());
            let r = mock_fmp_balance_sheet(&symbol, limit);
            fmp_response(Ok(r))
        },
    ));
    tokio::spawn(handle_sub(
        nc.clone(),
        sub_cash,
        |body: Option<Vec<u8>>| async move {
            let (symbol, limit) = parse_fmp_request(body.as_deref());
            let r = mock_fmp_cash_flow(&symbol, limit);
            fmp_response(Ok(r))
        },
    ));
    tokio::spawn(handle_sub(
        nc,
        sub_quote,
        |body: Option<Vec<u8>>| async move {
            let (symbol, _) = parse_fmp_request(body.as_deref());
            let r = mock_fmp_quote(&symbol);
            fmp_response(Ok(r))
        },
    ));
}

fn parse_fmp_request(body: Option<&[u8]>) -> (String, u32) {
    let (symbol, limit) = body
        .and_then(|b| serde_json::from_slice::<Value>(b).ok())
        .map(|v| {
            let symbol = v
                .get("symbol")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string();
            let limit = v.get("limit").and_then(Value::as_u64).unwrap_or(1) as u32;
            (symbol, limit)
        })
        .unwrap_or_else(|| (String::new(), 1));
    (symbol, limit)
}

async fn handle_sub<F, Fut>(nc: Client, mut sub: nats_adapter::async_nats::Subscriber, handler: F)
where
    F: Fn(Option<Vec<u8>>) -> Fut + Send + Sync + 'static,
    Fut: std::future::Future<Output = Vec<u8>> + Send,
{
    while let Some(msg) = sub.next().await {
        let reply = match msg.reply {
            Some(r) => r,
            None => continue,
        };
        let body = if msg.payload.is_empty() {
            None
        } else {
            Some(msg.payload.to_vec())
        };
        let response = handler(body).await;
        if let Err(e) = nc.publish(reply, Bytes::from(response)).await {
            warn!(error = %e, "reply publish failed");
        }
    }
}

async fn handle_sub_with_client(
    nc: Client,
    mut sub: nats_adapter::async_nats::Subscriber,
    client: ReqwestClient,
) {
    while let Some(msg) = sub.next().await {
        let reply = match msg.reply {
            Some(r) => r,
            None => continue,
        };
        let query: ImportPositionsQuery = msg
            .payload
            .as_ref()
            .len()
            .gt(&0)
            .then(|| serde_json::from_slice(msg.payload.as_ref()))
            .and_then(Result::ok)
            .unwrap_or_else(|| ImportPositionsQuery {
                broker: "ibkr".to_string(),
                account_id: None,
                dry_run: Some(true),
            });
        let r = api::discount_bank::import_positions(query, &client).await;
        let response = serde_json::to_vec(&r).unwrap_or_else(|_| b"{}".to_vec());
        if let Err(e) = nc.publish(reply, Bytes::from(response)).await {
            warn!(error = %e, "reply publish failed");
        }
    }
}

async fn run_loans(nc: Client, loan_repo: Option<Arc<LoanRepository>>) {
    match loan_repo {
        Some(repo) => run_loans_with_repo(nc, repo).await,
        None => {
            info!("LoanRepository not configured, loans API using mock data");
            run_loans_mock(nc).await;
        }
    }
}

async fn run_loans_mock(nc: Client) {
    let sub_list = match nc
        .queue_subscribe(SUBJECT_LOANS_LIST.to_string(), api_queue_group())
        .await
    {
        Ok(s) => s,
        Err(e) => {
            warn!(error = %e, "subscribe api.loans.list failed");
            return;
        }
    };
    let sub_list_proto = match nc
        .queue_subscribe(SUBJECT_LOANS_LIST_PROTO.to_string(), api_queue_group())
        .await
    {
        Ok(s) => s,
        Err(e) => {
            warn!(error = %e, "subscribe api.loans.list.proto failed");
            return;
        }
    };
    let sub_get = match nc
        .queue_subscribe(SUBJECT_LOANS_GET.to_string(), api_queue_group())
        .await
    {
        Ok(s) => s,
        Err(e) => {
            warn!(error = %e, "subscribe api.loans.get failed");
            return;
        }
    };
    tokio::spawn(handle_sub(
        nc.clone(),
        sub_list,
        move |_body: Option<Vec<u8>>| async move {
            let list = mock_loans_list();
            let r: Result<Vec<LoanRecord>, String> = Ok(list);
            serde_json::to_vec(&r).unwrap_or_else(|_| b"{}".to_vec())
        },
    ));
    tokio::spawn(handle_sub(
        nc.clone(),
        sub_list_proto,
        move |_body: Option<Vec<u8>>| async move {
            let list = mock_loans_list();
            let resp = loans_response_proto(&list);
            resp.encode_to_vec()
        },
    ));
    tokio::spawn(handle_sub(
        nc.clone(),
        sub_get,
        move |body: Option<Vec<u8>>| async move {
            let list = mock_loans_list();
            let loan_id = body
                .as_deref()
                .and_then(|b| serde_json::from_slice::<Value>(b).ok())
                .and_then(|v| v.get("loan_id").and_then(Value::as_str).map(String::from));
            let r: Result<Option<LoanRecord>, String> =
                Ok(loan_id.and_then(|id| list.into_iter().find(|l| l.loan_id == id)));
            serde_json::to_vec(&r).unwrap_or_else(|_| b"{}".to_vec())
        },
    ));
}

async fn run_loans_with_repo(nc: Client, repo: Arc<LoanRepository>) {
    let sub_list = match nc
        .queue_subscribe(SUBJECT_LOANS_LIST.to_string(), api_queue_group())
        .await
    {
        Ok(s) => s,
        Err(e) => {
            warn!(error = %e, "subscribe api.loans.list failed");
            return;
        }
    };
    let sub_list_proto = match nc
        .queue_subscribe(SUBJECT_LOANS_LIST_PROTO.to_string(), api_queue_group())
        .await
    {
        Ok(s) => s,
        Err(e) => {
            warn!(error = %e, "subscribe api.loans.list.proto failed");
            return;
        }
    };
    let sub_get = match nc
        .queue_subscribe(SUBJECT_LOANS_GET.to_string(), api_queue_group())
        .await
    {
        Ok(s) => s,
        Err(e) => {
            warn!(error = %e, "subscribe api.loans.get failed");
            return;
        }
    };
    let sub_create = match nc
        .queue_subscribe(SUBJECT_LOANS_CREATE.to_string(), api_queue_group())
        .await
    {
        Ok(s) => s,
        Err(e) => {
            warn!(error = %e, "subscribe api.loans.create failed");
            return;
        }
    };
    let sub_update = match nc
        .queue_subscribe(SUBJECT_LOANS_UPDATE.to_string(), api_queue_group())
        .await
    {
        Ok(s) => s,
        Err(e) => {
            warn!(error = %e, "subscribe api.loans.update failed");
            return;
        }
    };
    let sub_delete = match nc
        .queue_subscribe(SUBJECT_LOANS_DELETE.to_string(), api_queue_group())
        .await
    {
        Ok(s) => s,
        Err(e) => {
            warn!(error = %e, "subscribe api.loans.delete failed");
            return;
        }
    };

    let repo_list = repo.clone();
    tokio::spawn(handle_sub(
        nc.clone(),
        sub_list,
        move |_body: Option<Vec<u8>>| {
            let repo = repo_list.clone();
            async move {
                let list = repo.list().await;
                let r: Result<_, String> = Ok(list);
                serde_json::to_vec(&r).unwrap_or_else(|_| b"{}".to_vec())
            }
        },
    ));
    let repo_list_proto = repo.clone();
    tokio::spawn(handle_sub(
        nc.clone(),
        sub_list_proto,
        move |_body: Option<Vec<u8>>| {
            let repo = repo_list_proto.clone();
            async move {
                let list = repo.list().await;
                let resp = loans_response_proto(&list);
                resp.encode_to_vec()
            }
        },
    ));
    let repo_get = repo.clone();
    tokio::spawn(handle_sub(
        nc.clone(),
        sub_get,
        move |body: Option<Vec<u8>>| {
            let repo = repo_get.clone();
            async move {
                let loan_id = body
                    .as_deref()
                    .and_then(|b| serde_json::from_slice::<Value>(b).ok())
                    .and_then(|v| v.get("loan_id").and_then(Value::as_str).map(String::from));
                let r = match loan_id {
                    Some(id) => Ok(repo.get(&id).await),
                    None => Err("loan_id required".to_string()),
                };
                serde_json::to_vec(&r).unwrap_or_else(|_| b"{}".to_vec())
            }
        },
    ));
    let repo_create = repo.clone();
    tokio::spawn(handle_sub(
        nc.clone(),
        sub_create,
        move |body: Option<Vec<u8>>| {
            let repo = repo_create.clone();
            async move {
                let loan: LoanRecord =
                    match body.as_deref().and_then(|b| serde_json::from_slice(b).ok()) {
                        Some(l) => l,
                        None => {
                            let r: Result<(), String> =
                                Err("request body must be a LoanRecord".to_string());
                            return serde_json::to_vec(&r).unwrap_or_else(|_| b"{}".to_vec());
                        }
                    };
                let r = repo.create(loan).await;
                serde_json::to_vec(&r).unwrap_or_else(|_| b"{}".to_vec())
            }
        },
    ));
    let repo_update = repo.clone();
    tokio::spawn(handle_sub(
        nc.clone(),
        sub_update,
        move |body: Option<Vec<u8>>| {
            let repo = repo_update.clone();
            async move {
                let loan: LoanRecord =
                    match body.as_deref().and_then(|b| serde_json::from_slice(b).ok()) {
                        Some(l) => l,
                        None => {
                            let r: Result<(), String> =
                                Err("request body must be a LoanRecord".to_string());
                            return serde_json::to_vec(&r).unwrap_or_else(|_| b"{}".to_vec());
                        }
                    };
                let loan_id = loan.loan_id.clone();
                let r = repo.update(&loan_id, loan).await;
                serde_json::to_vec(&r).unwrap_or_else(|_| b"{}".to_vec())
            }
        },
    ));
    tokio::spawn(handle_sub(nc, sub_delete, move |body: Option<Vec<u8>>| {
        let repo = repo.clone();
        async move {
            let loan_id = body
                .as_deref()
                .and_then(|b| serde_json::from_slice::<Value>(b).ok())
                .and_then(|v| v.get("loan_id").and_then(Value::as_str).map(String::from));
            let r = match loan_id {
                Some(id) => repo.delete(&id).await,
                None => Err("loan_id required".to_string()),
            };
            serde_json::to_vec(&r).unwrap_or_else(|_| b"{}".to_vec())
        }
    }));
}

#[cfg(test)]
mod tests {
    use super::fill_missing_strikes;
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
