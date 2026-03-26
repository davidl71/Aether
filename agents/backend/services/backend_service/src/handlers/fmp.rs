//! Financial Modeling Prep (FMP) NATS request/reply handlers.
//! Subjects: api.fmp.*
//!
//! FMP fundamentals wired when FMP_API_KEY is set.

use std::sync::Arc;

use crate::handlers::{api_queue_group, handle_sub};
use market_data::FmpClient;
use nats_adapter::async_nats::Client;
use serde_json::Value;
use tracing::warn;

const SUBJECT_FMP_INCOME_STATEMENT: &str = "api.fmp.income_statement";
const SUBJECT_FMP_BALANCE_SHEET: &str = "api.fmp.balance_sheet";
const SUBJECT_FMP_CASH_FLOW: &str = "api.fmp.cash_flow";
const SUBJECT_FMP_QUOTE: &str = "api.fmp.quote";

/// Spawn FMP NATS API handlers (configured).
pub async fn spawn(nc: Client, fmp: Arc<FmpClient>) {
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

/// Spawn FMP NATS API handlers (unconfigured - returns error responses).
pub async fn spawn_unconfigured(nc: Client) {
    let sub_income = match nc
        .queue_subscribe(SUBJECT_FMP_INCOME_STATEMENT.to_string(), api_queue_group())
        .await
    {
        Ok(s) => s,
        Err(e) => {
            warn!(error = %e, "subscribe api.fmp.income_statement (unconfigured) failed");
            return;
        }
    };
    let sub_balance = match nc
        .queue_subscribe(SUBJECT_FMP_BALANCE_SHEET.to_string(), api_queue_group())
        .await
    {
        Ok(s) => s,
        Err(e) => {
            warn!(error = %e, "subscribe api.fmp.balance_sheet (unconfigured) failed");
            return;
        }
    };
    let sub_cash = match nc
        .queue_subscribe(SUBJECT_FMP_CASH_FLOW.to_string(), api_queue_group())
        .await
    {
        Ok(s) => s,
        Err(e) => {
            warn!(error = %e, "subscribe api.fmp.cash_flow (unconfigured) failed");
            return;
        }
    };
    let sub_quote = match nc
        .queue_subscribe(SUBJECT_FMP_QUOTE.to_string(), api_queue_group())
        .await
    {
        Ok(s) => s,
        Err(e) => {
            warn!(error = %e, "subscribe api.fmp.quote (unconfigured) failed");
            return;
        }
    };

    let err = b"{\"error\":\"FMP API key not configured\"}".to_vec();
    tokio::spawn(handle_sub(nc.clone(), sub_income, move |_| {
        let e = err.clone();
        async move { e }
    }));
    let err = b"{\"error\":\"FMP API key not configured\"}".to_vec();
    tokio::spawn(handle_sub(nc.clone(), sub_balance, move |_| {
        let e = err.clone();
        async move { e }
    }));
    let err = b"{\"error\":\"FMP API key not configured\"}".to_vec();
    tokio::spawn(handle_sub(nc.clone(), sub_cash, move |_| {
        let e = err.clone();
        async move { e }
    }));
    let err = b"{\"error\":\"FMP API key not configured\"}".to_vec();
    tokio::spawn(handle_sub(nc, sub_quote, move |_| {
        let e = err.clone();
        async move { e }
    }));
}

fn fmp_response<T: serde::Serialize>(r: anyhow::Result<T>) -> Vec<u8> {
    match r {
        Ok(data) => serde_json::to_vec(&data).unwrap_or_else(|_| b"{}".to_vec()),
        Err(e) => serde_json::to_vec(&serde_json::json!({ "error": e.to_string() }))
            .unwrap_or_else(|_| b"{}".to_vec()),
    }
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
