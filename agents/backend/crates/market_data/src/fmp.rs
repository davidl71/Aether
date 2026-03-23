//! Financial Modeling Prep (FMP) client.
//!
//! Provides financial statements and quote data for fundamental analysis
//! and cross-validation against TWS market data.
//!
//! FMP is a Tier 2 data source — not used for live trading decisions, but for
//! fundamental research, financial statement retrieval, and quote sanity checks.
//!
//! **NATS API:** FMP/chart/Swiftness/frontend are not exposed via NATS (deferred).
//! See `docs/platform/NATS_API.md` §3.
//!
//! **Rate limiting:** Free tier has 250 calls/day. Uses 100ms delay between calls
//! and tracks daily usage to avoid hitting limits.
//!
//! API reference: <https://site.financialmodelingprep.com/developer/docs>

use reqwest::{Client, Url};
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use tracing::{debug, warn};

const DEFAULT_BASE_URL: &str = "https://financialmodelingprep.com/api";
const CALL_DELAY_MS: u64 = 50;
const DAILY_LIMIT_FREE: u32 = 250;
const DAILY_LIMIT_PROFESSIONAL: u32 = 5000;
const BANDWIDTH_LIMIT_PROFESSIONAL_MB: u32 = 10240;

#[derive(Debug, Clone)]
pub struct HistoricalCandle {
    pub date: String,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: i64,
    pub adj_close: Option<f64>,
}

// ---------------------------------------------------------------------------
// Response types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Serialize, Deserialize)]
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
    rate_limiter: Arc<RateLimiter>,
}

struct RateLimiter {
    calls_today: AtomicU32,
    window_start: Mutex<Instant>,
    last_call: Mutex<Instant>,
}

impl RateLimiter {
    fn new() -> Self {
        Self {
            calls_today: AtomicU32::new(0),
            window_start: Mutex::new(Instant::now()),
            last_call: Mutex::new(Instant::now()),
        }
    }

    async fn acquire(&self) {
        let mut last = self.last_call.lock().await;
        let elapsed = last.elapsed();
        if elapsed < Duration::from_millis(CALL_DELAY_MS) {
            tokio::time::sleep(Duration::from_millis(CALL_DELAY_MS) - elapsed).await;
        }
        *last = Instant::now();
        drop(last);

        let calls = self.calls_today.fetch_add(1, Ordering::Relaxed) + 1;
        if calls >= DAILY_LIMIT_FREE {
            warn!(
                "FMP daily limit approaching: {}/{} calls",
                calls, DAILY_LIMIT_FREE
            );
        }
    }

    async fn reset_if_new_day(&self) {
        let now = Instant::now();
        let start = self.window_start.lock().await;
        if now.duration_since(*start) > Duration::from_secs(86400) {
            self.calls_today.store(0, Ordering::Relaxed);
            drop(start);
            let mut start = self.window_start.lock().await;
            *start = now;
        }
    }

    fn calls_used(&self) -> u32 {
        self.calls_today.load(Ordering::Relaxed)
    }
}

impl FmpClient {
    pub fn new(api_key: impl Into<String>, base_url: Option<&str>) -> anyhow::Result<Self> {
        let base = base_url.unwrap_or(DEFAULT_BASE_URL);
        let base_url =
            Url::parse(base).map_err(|e| anyhow::anyhow!("invalid FMP base url {base}: {e}"))?;

        let client = Client::builder()
            .user_agent("ib-box-spread-backend/0.1")
            .build()
            .map_err(|e| anyhow::anyhow!("failed to build HTTP client: {e}"))?;

        Ok(Self {
            client,
            api_key: api_key.into(),
            base_url,
            rate_limiter: Arc::new(RateLimiter::new()),
        })
    }

    pub fn calls_remaining(&self) -> u32 {
        DAILY_LIMIT_FREE.saturating_sub(self.rate_limiter.calls_used())
    }

    pub fn calls_used(&self) -> u32 {
        self.rate_limiter.calls_used()
    }

    /// Fetch the most recent income statements for `symbol`.
    ///
    /// `limit` controls how many periods are returned (1 = latest only).
    pub async fn income_statement(
        &self,
        symbol: &str,
        limit: u32,
    ) -> anyhow::Result<Vec<IncomeStatement>> {
        self.rate_limiter.reset_if_new_day().await;
        if self.calls_remaining() == 0 {
            anyhow::bail!("FMP daily limit reached (250 calls/day on free tier)");
        }
        self.rate_limiter.acquire().await;
        let url = self.url(&format!("/v3/income-statement/{symbol}"));
        debug!(
            "FMP income-statement: {url} ({}/{} calls used)",
            self.calls_used(),
            DAILY_LIMIT_FREE
        );

        let items: Vec<IncomeStatement> = self
            .client
            .get(url)
            .query(&[
                ("apikey", self.api_key.as_str()),
                ("limit", &limit.to_string()),
            ])
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("FMP income-statement request failed for {symbol}: {e}"))?
            .error_for_status()
            .map_err(|e| anyhow::anyhow!("FMP income-statement error for {symbol}: {e}"))?
            .json()
            .await
            .map_err(|e| anyhow::anyhow!("FMP income-statement decode failed for {symbol}: {e}"))?;

        Ok(items)
    }

    /// Fetch the most recent balance sheets for `symbol`.
    pub async fn balance_sheet(
        &self,
        symbol: &str,
        limit: u32,
    ) -> anyhow::Result<Vec<BalanceSheet>> {
        self.rate_limiter.reset_if_new_day().await;
        if self.calls_remaining() == 0 {
            anyhow::bail!("FMP daily limit reached (250 calls/day on free tier)");
        }
        self.rate_limiter.acquire().await;
        let url = self.url(&format!("/v3/balance-sheet-statement/{symbol}"));
        debug!(
            "FMP balance-sheet: {url} ({}/{} calls used)",
            self.calls_used(),
            DAILY_LIMIT_FREE
        );

        let items: Vec<BalanceSheet> = self
            .client
            .get(url)
            .query(&[
                ("apikey", self.api_key.as_str()),
                ("limit", &limit.to_string()),
            ])
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("FMP balance-sheet request failed for {symbol}: {e}"))?
            .error_for_status()
            .map_err(|e| anyhow::anyhow!("FMP balance-sheet error for {symbol}: {e}"))?
            .json()
            .await
            .map_err(|e| anyhow::anyhow!("FMP balance-sheet decode failed for {symbol}: {e}"))?;

        Ok(items)
    }

    /// Fetch the most recent cash flow statements for `symbol`.
    pub async fn cash_flow(
        &self,
        symbol: &str,
        limit: u32,
    ) -> anyhow::Result<Vec<CashFlowStatement>> {
        self.rate_limiter.reset_if_new_day().await;
        if self.calls_remaining() == 0 {
            anyhow::bail!("FMP daily limit reached (250 calls/day on free tier)");
        }
        self.rate_limiter.acquire().await;
        let url = self.url(&format!("/v3/cash-flow-statement/{symbol}"));
        debug!(
            "FMP cash-flow: {url} ({}/{} calls used)",
            self.calls_used(),
            DAILY_LIMIT_FREE
        );

        let items: Vec<CashFlowStatement> = self
            .client
            .get(url)
            .query(&[
                ("apikey", self.api_key.as_str()),
                ("limit", &limit.to_string()),
            ])
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
        self.rate_limiter.reset_if_new_day().await;
        if self.calls_remaining() == 0 {
            anyhow::bail!("FMP daily limit reached (250 calls/day on free tier)");
        }
        self.rate_limiter.acquire().await;
        let url = self.url(&format!("/v3/quote/{symbol}"));
        debug!(
            "FMP quote: {url} ({}/{} calls used)",
            self.calls_used(),
            DAILY_LIMIT_FREE
        );

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

    /// Historical OHLCV price data for `symbol`.
    ///
    /// `from` and `to` are ISO 8601 dates (e.g., "2024-01-01").
    /// `interval` is "1min", "5min", "15min", "30min", "1hour", "4hour", "daily", "weekly", "monthly".
    pub async fn historical_price(
        &self,
        symbol: &str,
        from: &str,
        to: &str,
        interval: &str,
    ) -> anyhow::Result<Vec<HistoricalCandle>> {
        self.rate_limiter.reset_if_new_day().await;
        if self.calls_remaining() == 0 {
            anyhow::bail!("FMP daily limit reached");
        }
        self.rate_limiter.acquire().await;

        let url = self.url(&format!("/v3/historical-price-full/{symbol}"));
        debug!(
            "FMP historical: {url} ({}/{} calls)",
            self.calls_used(),
            DAILY_LIMIT_FREE
        );

        #[derive(Deserialize)]
        struct Response {
            #[serde(rename = "historical")]
            candles: Vec<RawCandle>,
        }
        #[derive(Deserialize)]
        struct RawCandle {
            date: String,
            open: Option<f64>,
            high: Option<f64>,
            low: Option<f64>,
            close: Option<f64>,
            volume: Option<i64>,
            adj_close: Option<f64>,
        }

        let resp: Response = self
            .client
            .get(url)
            .query(&[
                ("apikey", self.api_key.as_str()),
                ("from", from),
                ("to", to),
                ("interval", interval),
            ])
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("FMP historical request failed: {e}"))?
            .error_for_status()
            .map_err(|e| anyhow::anyhow!("FMP historical error: {e}"))?
            .json()
            .await
            .map_err(|e| anyhow::anyhow!("FMP historical decode failed: {e}"))?;

        Ok(resp
            .candles
            .into_iter()
            .map(|c| HistoricalCandle {
                date: c.date,
                open: c.open.unwrap_or(0.0),
                high: c.high.unwrap_or(0.0),
                low: c.low.unwrap_or(0.0),
                close: c.close.unwrap_or(0.0),
                volume: c.volume.unwrap_or(0),
                adj_close: c.adj_close,
            })
            .collect())
    }

    /// Daily historical prices for the last N years.
    pub async fn historical_daily(
        &self,
        symbol: &str,
        years: u32,
    ) -> anyhow::Result<Vec<HistoricalCandle>> {
        let to = chrono::Utc::now().format("%Y-%m-%d").to_string();
        let from = (chrono::Utc::now() - chrono::Duration::days(365 * years as i64))
            .format("%Y-%m-%d")
            .to_string();
        self.historical_price(symbol, &from, &to, "daily").await
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

        let stmts = client(&server.uri())
            .income_statement("AAPL", 1)
            .await
            .unwrap();
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

        let sheets = client(&server.uri())
            .balance_sheet("AAPL", 1)
            .await
            .unwrap();
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
