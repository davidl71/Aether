//! Financial Modeling Prep (FMP) client.
//!
//! Provides financial statements and quote data for fundamental analysis
//! and cross-validation against TWS market data.
//!
//! FMP is a Tier 2 data source — not used for live trading decisions, but for
//! fundamental research, financial statement retrieval, and quote sanity checks.
//!
//! API reference: <https://site.financialmodelingprep.com/developer/docs>

use reqwest::{Client, Url};
use serde::Deserialize;
use tracing::debug;

const DEFAULT_BASE_URL: &str = "https://financialmodelingprep.com/api";

// ---------------------------------------------------------------------------
// Response types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IncomeStatement {
    pub symbol: String,
    pub date: String,
    pub revenue: Option<f64>,
    pub gross_profit: Option<f64>,
    pub operating_income: Option<f64>,
    pub net_income: Option<f64>,
    pub eps: Option<f64>,
    pub eps_diluted: Option<f64>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BalanceSheet {
    pub symbol: String,
    pub date: String,
    pub total_assets: Option<f64>,
    pub total_liabilities: Option<f64>,
    pub total_stockholders_equity: Option<f64>,
    pub cash_and_cash_equivalents: Option<f64>,
    pub total_debt: Option<f64>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CashFlowStatement {
    pub symbol: String,
    pub date: String,
    pub operating_cash_flow: Option<f64>,
    pub capital_expenditure: Option<f64>,
    pub free_cash_flow: Option<f64>,
    pub dividends_paid: Option<f64>,
}

/// Real-time quote used for cross-validation against TWS data.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FmpQuote {
    pub symbol: String,
    pub price: Option<f64>,
    pub open: Option<f64>,
    pub day_high: Option<f64>,
    pub day_low: Option<f64>,
    pub volume: Option<i64>,
    pub previous_close: Option<f64>,
}

// ---------------------------------------------------------------------------
// Client
// ---------------------------------------------------------------------------

pub struct FmpClient {
    client: Client,
    api_key: String,
    base_url: Url,
}

impl FmpClient {
    pub fn new(api_key: impl Into<String>, base_url: Option<&str>) -> anyhow::Result<Self> {
        let base = base_url.unwrap_or(DEFAULT_BASE_URL);
        let base_url = Url::parse(base)
            .map_err(|e| anyhow::anyhow!("invalid FMP base url {base}: {e}"))?;

        let client = Client::builder()
            .user_agent("ib-box-spread-backend/0.1")
            .build()
            .map_err(|e| anyhow::anyhow!("failed to build HTTP client: {e}"))?;

        Ok(Self { client, api_key: api_key.into(), base_url })
    }

    /// Fetch the most recent income statements for `symbol`.
    ///
    /// `limit` controls how many periods are returned (1 = latest only).
    pub async fn income_statement(
        &self,
        symbol: &str,
        limit: u32,
    ) -> anyhow::Result<Vec<IncomeStatement>> {
        let url = self.url(&format!("/v3/income-statement/{symbol}"));
        debug!("FMP income-statement: {url}");

        let items: Vec<IncomeStatement> = self
            .client
            .get(url)
            .query(&[("apikey", self.api_key.as_str()), ("limit", &limit.to_string())])
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("FMP income-statement request failed for {symbol}: {e}"))?
            .error_for_status()
            .map_err(|e| anyhow::anyhow!("FMP income-statement error for {symbol}: {e}"))?
            .json()
            .await
            .map_err(|e| {
                anyhow::anyhow!("FMP income-statement decode failed for {symbol}: {e}")
            })?;

        Ok(items)
    }

    /// Fetch the most recent balance sheets for `symbol`.
    pub async fn balance_sheet(
        &self,
        symbol: &str,
        limit: u32,
    ) -> anyhow::Result<Vec<BalanceSheet>> {
        let url = self.url(&format!("/v3/balance-sheet-statement/{symbol}"));
        debug!("FMP balance-sheet: {url}");

        let items: Vec<BalanceSheet> = self
            .client
            .get(url)
            .query(&[("apikey", self.api_key.as_str()), ("limit", &limit.to_string())])
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("FMP balance-sheet request failed for {symbol}: {e}"))?
            .error_for_status()
            .map_err(|e| anyhow::anyhow!("FMP balance-sheet error for {symbol}: {e}"))?
            .json()
            .await
            .map_err(|e| {
                anyhow::anyhow!("FMP balance-sheet decode failed for {symbol}: {e}")
            })?;

        Ok(items)
    }

    /// Fetch the most recent cash flow statements for `symbol`.
    pub async fn cash_flow(
        &self,
        symbol: &str,
        limit: u32,
    ) -> anyhow::Result<Vec<CashFlowStatement>> {
        let url = self.url(&format!("/v3/cash-flow-statement/{symbol}"));
        debug!("FMP cash-flow: {url}");

        let items: Vec<CashFlowStatement> = self
            .client
            .get(url)
            .query(&[("apikey", self.api_key.as_str()), ("limit", &limit.to_string())])
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("FMP cash-flow request failed for {symbol}: {e}"))?
            .error_for_status()
            .map_err(|e| anyhow::anyhow!("FMP cash-flow error for {symbol}: {e}"))?
            .json()
            .await
            .map_err(|e| anyhow::anyhow!("FMP cash-flow decode failed for {symbol}: {e}"))?;

        Ok(items)
    }

    /// Fetch a real-time quote for `symbol` for cross-validation against TWS data.
    pub async fn quote(&self, symbol: &str) -> anyhow::Result<FmpQuote> {
        let url = self.url(&format!("/v3/quote/{symbol}"));
        debug!("FMP quote: {url}");

        let mut items: Vec<FmpQuote> = self
            .client
            .get(url)
            .query(&[("apikey", self.api_key.as_str())])
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("FMP quote request failed for {symbol}: {e}"))?
            .error_for_status()
            .map_err(|e| anyhow::anyhow!("FMP quote error for {symbol}: {e}"))?
            .json()
            .await
            .map_err(|e| anyhow::anyhow!("FMP quote decode failed for {symbol}: {e}"))?;

        items
            .pop()
            .ok_or_else(|| anyhow::anyhow!("FMP returned empty quote list for {symbol}"))
    }

    fn url(&self, path: &str) -> Url {
        let mut u = self.base_url.clone();
        u.set_path(path);
        u
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{method, path, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn client(base: &str) -> FmpClient {
        FmpClient::new("test-key", Some(base)).unwrap()
    }

    #[tokio::test]
    async fn fetches_income_statement() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/v3/income-statement/AAPL"))
            .and(query_param("apikey", "test-key"))
            .respond_with(ResponseTemplate::new(200).set_body_raw(
                r#"[{
                    "symbol": "AAPL",
                    "date": "2024-09-30",
                    "revenue": 391035000000.0,
                    "grossProfit": 180683000000.0,
                    "operatingIncome": 123216000000.0,
                    "netIncome": 93736000000.0,
                    "eps": 6.11,
                    "epsDiluted": 6.08
                }]"#,
                "application/json",
            ))
            .mount(&server)
            .await;

        let stmts = client(&server.uri()).income_statement("AAPL", 1).await.unwrap();
        assert_eq!(stmts.len(), 1);
        assert_eq!(stmts[0].symbol, "AAPL");
        assert!(stmts[0].net_income.unwrap() > 0.0);
    }

    #[tokio::test]
    async fn fetches_balance_sheet() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/v3/balance-sheet-statement/AAPL"))
            .and(query_param("apikey", "test-key"))
            .respond_with(ResponseTemplate::new(200).set_body_raw(
                r#"[{
                    "symbol": "AAPL",
                    "date": "2024-09-30",
                    "totalAssets": 364980000000.0,
                    "totalLiabilities": 308030000000.0,
                    "totalStockholdersEquity": 56950000000.0,
                    "cashAndCashEquivalents": 29943000000.0,
                    "totalDebt": 101304000000.0
                }]"#,
                "application/json",
            ))
            .mount(&server)
            .await;

        let sheets = client(&server.uri()).balance_sheet("AAPL", 1).await.unwrap();
        assert_eq!(sheets.len(), 1);
        assert!(sheets[0].total_assets.unwrap() > sheets[0].total_liabilities.unwrap());
    }

    #[tokio::test]
    async fn fetches_cash_flow() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/v3/cash-flow-statement/AAPL"))
            .and(query_param("apikey", "test-key"))
            .respond_with(ResponseTemplate::new(200).set_body_raw(
                r#"[{
                    "symbol": "AAPL",
                    "date": "2024-09-30",
                    "operatingCashFlow": 118254000000.0,
                    "capitalExpenditure": -9447000000.0,
                    "freeCashFlow": 108807000000.0,
                    "dividendsPaid": -15234000000.0
                }]"#,
                "application/json",
            ))
            .mount(&server)
            .await;

        let flows = client(&server.uri()).cash_flow("AAPL", 1).await.unwrap();
        assert_eq!(flows.len(), 1);
        assert!(flows[0].free_cash_flow.unwrap() > 0.0);
    }

    #[tokio::test]
    async fn fetches_quote_for_cross_validation() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/v3/quote/SPY"))
            .and(query_param("apikey", "test-key"))
            .respond_with(ResponseTemplate::new(200).set_body_raw(
                r#"[{
                    "symbol": "SPY",
                    "price": 512.34,
                    "open": 510.00,
                    "dayHigh": 513.50,
                    "dayLow": 509.80,
                    "volume": 72000000,
                    "previousClose": 511.20
                }]"#,
                "application/json",
            ))
            .mount(&server)
            .await;

        let q = client(&server.uri()).quote("SPY").await.unwrap();
        assert_eq!(q.symbol, "SPY");
        assert!(q.price.unwrap() > 0.0);
        assert!(q.day_high.unwrap() >= q.day_low.unwrap());
    }

    #[tokio::test]
    async fn empty_quote_list_returns_error() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/v3/quote/FAKE"))
            .respond_with(ResponseTemplate::new(200).set_body_raw("[]", "application/json"))
            .mount(&server)
            .await;

        let err = client(&server.uri()).quote("FAKE").await.unwrap_err();
        assert!(err.to_string().contains("empty quote list"));
    }

    #[tokio::test]
    async fn http_error_propagates() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/v3/quote/SPY"))
            .respond_with(ResponseTemplate::new(401))
            .mount(&server)
            .await;

        let err = client(&server.uri()).quote("SPY").await.unwrap_err();
        assert!(err.to_string().contains("SPY"));
    }
}
