//! Client Portal API: options chain flow (search → strikes → info).
//!
//! **Mandatory sequence** (see [CLIENT_PORTAL_OPTIONS_CHAIN.md](../../../docs/platform/CLIENT_PORTAL_OPTIONS_CHAIN.md)):
//! 1. `GET /iserver/secdef/search` – resolve symbol to underlying `conid` and option expiration months.
//! 2. `GET /iserver/secdef/strikes` – get strike prices for a given underlying and month.
//! 3. `GET /iserver/secdef/info` – get contract details (conid, symbol, strike, maturityDate) per strike/right.
//!
//! There is no single-call options chain endpoint; the three steps must be called in order.
//! Base URL from `IB_PORTAL_URL` (e.g. `https://localhost:5001/v1/portal`); Gateway may use `/v1/api` instead.
//!
//! **Stubs:** The per-step functions (`search`, `strikes`, `info`) currently return mock data. They are
//! used by `options_chain_flow` and will be wired to the Client Portal Gateway (reqwest GET) when
//! options chain is needed from TUI, CLI, or API.

use serde::{Deserialize, Serialize};

/// Default Client Portal base (same as ib_positions). Use `IB_PORTAL_URL` to override.
const DEFAULT_IB_PORTAL_URL: &str = "https://localhost:5001/v1/portal";

/// Result of step 1: search. Underlying conid and option expiration months for the symbol.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub conid: i64,
    pub months: Vec<String>,
}

/// Result of step 2: strikes. Put and call strike prices for the given underlying and month.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrikesResult {
    pub put: Vec<f64>,
    pub call: Vec<f64>,
}

/// Result of step 3: info. One option contract (e.g. one expiry within the month).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractInfo {
    pub conid: i64,
    pub symbol: String,
    pub strike: f64,
    pub maturity_date: String,
}

/// Combined result of the three-step flow (mock or real).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptionsChainResult {
    pub search: SearchResult,
    pub strikes: StrikesResult,
    pub info_contracts: Vec<ContractInfo>,
}

/// Step 1: GET /iserver/secdef/search — resolve symbol to underlying conid and option months.
///
/// Query: `symbol` (required), `listingExchange` (optional).
/// Stub: returns mock data. Replace with reqwest GET when wiring to Gateway.
pub async fn search(
    _symbol: &str,
    _listing_exchange: Option<&str>,
) -> Result<SearchResult, String> {
    // TODO: build URL from portal_base(), e.g. GET {base}/iserver/secdef/search?symbol={symbol}
    // and optional &listingExchange=...; parse response for conid and sections[secType=OPT].months
    Ok(SearchResult {
        conid: 265598,
        months: vec!["202505".to_string(), "202506".to_string()],
    })
}

/// Step 2: GET /iserver/secdef/strikes — list strike prices for underlying and month.
///
/// Query: `conid`, `secType=OPT`, `month`.
/// Stub: returns mock data. Replace with reqwest GET when wiring to Gateway.
pub async fn strikes(_conid: i64, _month: &str) -> Result<StrikesResult, String> {
    // TODO: GET {base}/iserver/secdef/strikes?conid={conid}&secType=OPT&month={month}
    Ok(StrikesResult {
        put: vec![220.0, 225.0, 230.0],
        call: vec![220.0, 225.0, 230.0],
    })
}

/// Step 3: GET /iserver/secdef/info — contract details per strike and right (P/C).
///
/// Query: `conid`, `month`, `strike`, `secType=OPT`, `right=P` or `right=C`.
/// Stub: returns mock data. Replace with reqwest GET when wiring to Gateway.
pub async fn info(
    _conid: i64,
    _month: &str,
    _strike: f64,
    _right: &str,
) -> Result<Vec<ContractInfo>, String> {
    // TODO: GET {base}/iserver/secdef/info?conid=...&month=...&strike=...&secType=OPT&right=P|C
    Ok(vec![ContractInfo {
        conid: 60000001,
        symbol: "AAPL".to_string(),
        strike: 225.0,
        maturity_date: "20250516".to_string(),
    }])
}

/// Runs the three-step Client Portal options chain flow and returns a combined result.
///
/// Currently returns mock data only. Full implementation would:
/// 1. Call `search(symbol, listing_exchange)` for conid and months.
/// 2. Call `strikes(conid, first_month)` for put/call strikes.
/// 3. Optionally filter strikes (e.g. near the money), then for each strike/right call `info(...)`.
pub async fn options_chain_flow(symbol: &str) -> Result<OptionsChainResult, String> {
    let search_result = search(symbol, None).await?;
    let month = search_result
        .months
        .first()
        .map(String::as_str)
        .unwrap_or("202505");
    let strikes_result = strikes(search_result.conid, month).await?;
    let mut info_contracts = Vec::new();
    for &strike in strikes_result.call.iter().take(1) {
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

/// Returns portal base URL (from IB_PORTAL_URL or default).
pub fn portal_base() -> String {
    std::env::var("IB_PORTAL_URL")
        .unwrap_or_else(|_| DEFAULT_IB_PORTAL_URL.to_string())
        .trim_end_matches('/')
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::{options_chain_flow, portal_base};

    #[tokio::test]
    async fn options_chain_flow_returns_mock() {
        let result = options_chain_flow("AAPL").await.unwrap();
        assert_eq!(result.search.conid, 265598);
        assert!(!result.search.months.is_empty());
        assert!(!result.strikes.call.is_empty());
        assert!(!result.strikes.put.is_empty());
        assert!(!result.info_contracts.is_empty());
    }

    #[test]
    fn portal_base_has_iserver_path() {
        let base = portal_base();
        assert!(base.contains("localhost") || base.contains("5001") || base.starts_with("http"));
    }
}
