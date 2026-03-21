//! Client Portal API: options chain flow (search → strikes → info).
//!
//! **Mandatory sequence** (see [CLIENT_PORTAL_OPTIONS_CHAIN.md](../../../docs/platform/CLIENT_PORTAL_OPTIONS_CHAIN.md)):
//! 1. `GET /iserver/secdef/search` – resolve symbol to underlying `conid` and option expiration months.
//! 2. `GET /iserver/secdef/strikes` – get strike prices for a given underlying and month.
//! 3. `GET /iserver/secdef/info` – get contract details (conid, symbol, strike, maturityDate) per strike/right.
//!
//! There is no single-call options chain endpoint; the three steps must be called in order.
//! Base URL from `IB_PORTAL_URL` (e.g. `https://localhost:5001/v1/portal`); Gateway may use `/v1/api` instead.

use std::time::Duration;

use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::ib_positions::truncate_body;

const DEFAULT_IB_PORTAL_URL: &str = "https://localhost:5001/v1/portal";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub conid: i64,
    pub months: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrikesResult {
    pub put: Vec<f64>,
    pub call: Vec<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractInfo {
    pub conid: i64,
    pub symbol: String,
    pub strike: f64,
    pub maturity_date: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptionsChainResult {
    pub search: SearchResult,
    pub strikes: StrikesResult,
    pub info_contracts: Vec<ContractInfo>,
}

fn portal_base() -> String {
    std::env::var("IB_PORTAL_URL")
        .unwrap_or_else(|_| DEFAULT_IB_PORTAL_URL.to_string())
        .trim_end_matches('/')
        .to_string()
}

fn build_client() -> Result<Client, String> {
    Client::builder()
        .danger_accept_invalid_certs(true)
        .cookie_store(true)
        .timeout(Duration::from_secs(10))
        .build()
        .map_err(|e| format!("Failed to build Client Portal client: {e}"))
}

async fn get_json(client: &Client, url: &str) -> Result<Value, String> {
    let response = client
        .get(url)
        .send()
        .await
        .map_err(|e| format!("Client Portal request failed: {e}"))?;
    let status = response.status();
    let body = response
        .text()
        .await
        .map_err(|e| format!("Client Portal response read failed: {e}"))?;
    if !status.is_success() {
        return Err(format!(
            "Client Portal responded with status {status}: {}",
            truncate_body(&body)
        ));
    }
    serde_json::from_str(&body)
        .map_err(|e| format!("Failed to decode Client Portal response: {e}"))
}

fn value_as_i64(value: Option<&Value>) -> Option<i64> {
    match value? {
        Value::Number(n) => n.as_i64(),
        Value::String(s) => s.trim().parse::<i64>().ok(),
        _ => None,
    }
}

fn value_as_str(value: Option<&Value>) -> Option<String> {
    match value? {
        Value::String(s) => {
            let t = s.trim();
            (!t.is_empty()).then(|| t.to_string())
        }
        Value::Number(n) => Some(n.to_string()),
        _ => None,
    }
}

pub async fn search(symbol: &str, listing_exchange: Option<&str>) -> Result<SearchResult, String> {
    let client = build_client()?;
    let base = portal_base();
    let mut url = format!("{base}/iserver/secdef/search?symbol={symbol}");
    if let Some(ex) = listing_exchange {
        url.push_str(&format!("&listingExchange={ex}"));
    }
    let payload: Value = get_json(&client, &url).await?;
    let items = payload.as_array().ok_or("secdef/search response is not an array")?;
    let contract = items.first().ok_or("secdef/search returned empty array")?;
    let map = contract.as_object().ok_or("secdef/search item is not an object")?;
    let conid = value_as_i64(map.get("conid")).ok_or("secdef/search: missing or invalid conid")?;
    let sections = map
        .get("sections")
        .and_then(Value::as_array)
        .ok_or("secdef/search: missing or invalid sections")?;
    let opt_section = sections
        .iter()
        .find(|s| s.get("secType").and_then(Value::as_str) == Some("OPT"));
    let months_str = opt_section
        .and_then(|s| s.get("months"))
        .and_then(Value::as_str)
        .unwrap_or("");
    let months: Vec<String> = months_str
        .split(';')
        .map(String::from)
        .filter(|s| !s.is_empty())
        .collect();
    if months.is_empty() {
        return Err("secdef/search: no OPT months found".to_string());
    }
    Ok(SearchResult { conid, months })
}

pub async fn strikes(conid: i64, month: &str) -> Result<StrikesResult, String> {
    let client = build_client()?;
    let base = portal_base();
    let url = format!(
        "{base}/iserver/secdef/strikes?conid={conid}&secType=OPT&month={month}"
    );
    let payload: Value = get_json(&client, &url).await?;
    let parse_f64_array = |key: &str| -> Vec<f64> {
        payload
            .get(key)
            .and_then(Value::as_array)
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| match v {
                        Value::Number(n) => n.as_f64(),
                        Value::String(s) => s.parse::<f64>().ok(),
                        _ => None,
                    })
                    .collect()
            })
            .unwrap_or_default()
    };
    let put = parse_f64_array("put");
    let call = parse_f64_array("call");
    Ok(StrikesResult { put, call })
}

pub async fn info(
    conid: i64,
    month: &str,
    strike: f64,
    right: &str,
) -> Result<Vec<ContractInfo>, String> {
    let client = build_client()?;
    let base = portal_base();
    let url = format!(
        "{base}/iserver/secdef/info?conid={conid}&month={month}&strike={strike}&secType=OPT&right={right}"
    );
    let payload: Value = get_json(&client, &url).await?;
    let items = payload.as_array().ok_or("secdef/info response is not an array")?;
    let contracts: Vec<ContractInfo> = items
        .iter()
        .filter_map(|item| {
            let map = item.as_object()?;
            let conid = value_as_i64(map.get("conid"))?;
            let symbol = value_as_str(map.get("symbol"))?;
            let strike = map
                .get("strike")
                .and_then(|v| match v {
                    Value::Number(n) => n.as_f64(),
                    Value::String(s) => s.parse::<f64>().ok(),
                    _ => None,
                })
                .unwrap_or(0.0);
            let maturity_date = value_as_str(map.get("maturityDate")).unwrap_or_default();
            Some(ContractInfo {
                conid,
                symbol,
                strike,
                maturity_date,
            })
        })
        .collect();
    Ok(contracts)
}

pub async fn options_chain_flow(symbol: &str) -> Result<OptionsChainResult, String> {
    let search_result = search(symbol, None).await?;
    let month = search_result
        .months
        .first()
        .map(String::as_str)
        .unwrap_or("202505");
    let strikes_result = strikes(search_result.conid, month).await?;
    let mut info_contracts = Vec::new();
    for &strike in strikes_result.call.iter().take(3) {
        if let Ok(contracts) = info(search_result.conid, month, strike, "C").await {
            info_contracts.extend(contracts);
        }
    }
    Ok(OptionsChainResult {
        search: search_result,
        strikes: strikes_result,
        info_contracts,
    })
}

#[cfg(test)]
mod tests {
    use super::{portal_base, search, strikes, info};

    #[test]
    fn portal_base_returns_valid_url() {
        let base = portal_base();
        assert!(base.starts_with("http"));
    }

    #[tokio::test]
    async fn search_rejects_empty_symbol() {
        let result = search("", None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn strikes_with_invalid_conid_gives_error() {
        let result = strikes(0, "202505").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn info_with_invalid_params_gives_error() {
        let result = info(0, "202505", 0.0, "C").await;
        assert!(result.is_err());
    }
}
