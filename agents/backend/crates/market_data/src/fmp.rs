//! Financial Modeling Prep (FMP) client.
//!
//! Provides financial statements and quote data for fundamental analysis
//! and cross-validation against TWS market data.
//!
//! FMP is a Tier 2 data source — used for continuous quotes when Yahoo/Polygon unavailable.
//! With paid tier (5000 calls/day), can poll ~1x per symbol per 17 seconds with 5 symbols.
//!
//! **NATS API:** FMP/chart/Swiftness/frontend are not exposed via NATS (deferred).
//! See `docs/platform/NATS_API.md` §3.
//!
//! **Rate limiting:** Free tier has 250 calls/day, Professional has 5000 calls/day.
//! Uses 50ms delay between calls and tracks daily usage to avoid hitting limits.
//!
//! API reference: <https://site.financialmodelingprep.com/developer/docs>

use anyhow::Context;
use async_trait::async_trait;
use chrono::Utc;
use reqwest::{Client, Url};
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use tracing::{debug, warn};

use crate::{
    MarketDataEvent, MarketDataEventBuilder, MarketDataSource, SimpleMarketDataSourceFactory,
};

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

/// Entry in the stock list directory.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FmpStockListEntry {
    pub symbol: String,
    pub name: Option<String>,
    pub exchange: Option<String>,
    #[serde(rename = "exchangeShortName")]
    pub exchange_short_name: Option<String>,
}

/// Search result for symbol search endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FmpSearchResult {
    pub symbol: String,
    pub name: Option<String>,
    pub exchange: Option<String>,
    #[serde(rename = "exchangeShortName")]
    pub exchange_short_name: Option<String>,
    #[serde(rename = "type")]
    pub instrument_type: Option<String>,
}

/// Treasury rate for a specific maturity.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TreasuryRate {
    pub date: String,
    #[serde(rename = "1month")]
    pub one_month: Option<f64>,
    #[serde(rename = "2month")]
    pub two_month: Option<f64>,
    #[serde(rename = "3month")]
    pub three_month: Option<f64>,
    #[serde(rename = "6month")]
    pub six_month: Option<f64>,
    #[serde(rename = "1year")]
    pub one_year: Option<f64>,
    #[serde(rename = "2year")]
    pub two_year: Option<f64>,
    #[serde(rename = "3year")]
    pub three_year: Option<f64>,
    #[serde(rename = "5year")]
    pub five_year: Option<f64>,
    #[serde(rename = "7year")]
    pub seven_year: Option<f64>,
    #[serde(rename = "10year")]
    pub ten_year: Option<f64>,
    #[serde(rename = "20year")]
    pub twenty_year: Option<f64>,
    #[serde(rename = "30year")]
    pub thirty_year: Option<f64>,
}

/// SOFR rate data.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SofrRate {
    pub date: String,
    pub rate: Option<f64>,
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

    /// Fetch real-time quotes for multiple symbols in a single API call.
    ///
    /// This is ~5x more efficient than calling `quote()` for each symbol.
    /// Endpoint: `/stable/batch-quote?symbols=AAPL,MSFT,GOOG`
    pub async fn batch_quote(&self, symbols: &[String]) -> anyhow::Result<Vec<FmpQuote>> {
        if symbols.is_empty() {
            return Ok(vec![]);
        }

        self.rate_limiter.reset_if_new_day().await;
        if self.calls_remaining() == 0 {
            anyhow::bail!("FMP daily limit reached (250 calls/day on free tier)");
        }
        self.rate_limiter.acquire().await;

        let symbols_csv = symbols.join(",");
        let url = self.url("/stable/batch-quote");
        debug!(
            "FMP batch-quote: {} symbols ({}/{} calls used)",
            symbols.len(),
            self.calls_used(),
            DAILY_LIMIT_FREE
        );

        let quotes: Vec<FmpQuote> = self
            .client
            .get(url)
            .query(&[("apikey", self.api_key.as_str()), ("symbols", &symbols_csv)])
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("FMP batch-quote request failed: {e}"))?
            .error_for_status()
            .map_err(|e| anyhow::anyhow!("FMP batch-quote error: {e}"))?
            .json()
            .await
            .map_err(|e| anyhow::anyhow!("FMP batch-quote decode failed: {e}"))?;

        debug!("FMP batch-quote returned {} quotes", quotes.len());
        Ok(quotes)
    }

    /// Fetch the complete list of available stock symbols.
    ///
    /// This endpoint returns all symbols across all exchanges.
    /// Use this for symbol discovery and validation.
    /// Endpoint: `/stable/stock-list`
    pub async fn stock_list(&self) -> anyhow::Result<Vec<FmpStockListEntry>> {
        self.rate_limiter.reset_if_new_day().await;
        if self.calls_remaining() == 0 {
            anyhow::bail!("FMP daily limit reached (250 calls/day on free tier)");
        }
        self.rate_limiter.acquire().await;

        let url = self.url("/stable/stock-list");
        debug!(
            "FMP stock-list: ({}/{} calls used)",
            self.calls_used(),
            DAILY_LIMIT_FREE
        );

        let items: Vec<FmpStockListEntry> = self
            .client
            .get(url)
            .query(&[("apikey", self.api_key.as_str())])
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("FMP stock-list request failed: {e}"))?
            .error_for_status()
            .map_err(|e| anyhow::anyhow!("FMP stock-list error: {e}"))?
            .json()
            .await
            .map_err(|e| anyhow::anyhow!("FMP stock-list decode failed: {e}"))?;

        debug!("FMP stock-list returned {} symbols", items.len());
        Ok(items)
    }

    /// Search for symbols by name or symbol prefix.
    ///
    /// Useful for autocomplete and symbol discovery.
    /// Endpoint: `/stable/search-symbol?query=AA`
    pub async fn search_symbol(&self, query: &str) -> anyhow::Result<Vec<FmpSearchResult>> {
        if query.trim().is_empty() {
            return Ok(vec![]);
        }

        self.rate_limiter.reset_if_new_day().await;
        if self.calls_remaining() == 0 {
            anyhow::bail!("FMP daily limit reached (250 calls/day on free tier)");
        }
        self.rate_limiter.acquire().await;

        let url = self.url("/stable/search-symbol");
        debug!(
            "FMP search-symbol: query='{}' ({}/{} calls used)",
            query,
            self.calls_used(),
            DAILY_LIMIT_FREE
        );

        let items: Vec<FmpSearchResult> = self
            .client
            .get(url)
            .query(&[("apikey", self.api_key.as_str()), ("query", query)])
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("FMP search-symbol request failed: {e}"))?
            .error_for_status()
            .map_err(|e| anyhow::anyhow!("FMP search-symbol error: {e}"))?
            .json()
            .await
            .map_err(|e| anyhow::anyhow!("FMP search-symbol decode failed: {e}"))?;

        debug!("FMP search-symbol '{}' returned {} results", query, items.len());
        Ok(items)
    }

    /// Fetch symbols that have financial statement data available.
    ///
    /// This is a smaller, curated list of symbols with fundamentals.
    /// Endpoint: `/stable/financial-statement-symbol-list`
    pub async fn financial_statement_symbols(&self) -> anyhow::Result<Vec<String>> {
        self.rate_limiter.reset_if_new_day().await;
        if self.calls_remaining() == 0 {
            anyhow::bail!("FMP daily limit reached (250 calls/day on free tier)");
        }
        self.rate_limiter.acquire().await;

        let url = self.url("/stable/financial-statement-symbol-list");
        debug!(
            "FMP financial-statement-symbol-list: ({}/{} calls used)",
            self.calls_used(),
            DAILY_LIMIT_FREE
        );

        let items: Vec<String> = self
            .client
            .get(url)
            .query(&[("apikey", self.api_key.as_str())])
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("FMP financial-statement-symbol-list request failed: {e}"))?
            .error_for_status()
            .map_err(|e| anyhow::anyhow!("FMP financial-statement-symbol-list error: {e}"))?
            .json()
            .await
            .map_err(|e| anyhow::anyhow!("FMP financial-statement-symbol-list decode failed: {e}"))?;

        debug!(
            "FMP financial-statement-symbol-list returned {} symbols",
            items.len()
        );
        Ok(items)
    }

    /// Fetch the latest treasury rates for all maturities.
    ///
    /// Returns rates from 1-month to 30-year treasury yields.
    /// Endpoint: `/stable/treasury-rates`
    pub async fn treasury_rates(&self) -> anyhow::Result<TreasuryRate> {
        self.rate_limiter.reset_if_new_day().await;
        if self.calls_remaining() == 0 {
            anyhow::bail!("FMP daily limit reached (250 calls/day on free tier)");
        }
        self.rate_limiter.acquire().await;

        let url = self.url("/stable/treasury-rates");
        debug!(
            "FMP treasury-rates: ({}/{} calls used)",
            self.calls_used(),
            DAILY_LIMIT_FREE
        );

        let rate: TreasuryRate = self
            .client
            .get(url)
            .query(&[("apikey", self.api_key.as_str())])
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("FMP treasury-rates request failed: {e}"))?
            .error_for_status()
            .map_err(|e| anyhow::anyhow!("FMP treasury-rates error: {e}"))?
            .json()
            .await
            .map_err(|e| anyhow::anyhow!("FMP treasury-rates decode failed: {e}"))?;

        debug!("FMP treasury-rates for date: {}", rate.date);
        Ok(rate)
    }

    /// Fetch historical treasury rates.
    ///
    /// `from` and `to` are ISO 8601 dates.
    /// Endpoint: `/stable/treasury-rates`
    pub async fn treasury_rates_historical(
        &self,
        from: &str,
        to: &str,
    ) -> anyhow::Result<Vec<TreasuryRate>> {
        self.rate_limiter.reset_if_new_day().await;
        if self.calls_remaining() == 0 {
            anyhow::bail!("FMP daily limit reached (250 calls/day on free tier)");
        }
        self.rate_limiter.acquire().await;

        let url = self.url("/stable/treasury-rates");
        debug!(
            "FMP treasury-rates historical: {} to {} ({}/{} calls used)",
            from,
            to,
            self.calls_used(),
            DAILY_LIMIT_FREE
        );

        let rates: Vec<TreasuryRate> = self
            .client
            .get(url)
            .query(&[
                ("apikey", self.api_key.as_str()),
                ("from", from),
                ("to", to),
            ])
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("FMP treasury-rates historical request failed: {e}"))?
            .error_for_status()
            .map_err(|e| anyhow::anyhow!("FMP treasury-rates historical error: {e}"))?
            .json()
            .await
            .map_err(|e| anyhow::anyhow!("FMP treasury-rates historical decode failed: {e}"))?;

        debug!("FMP treasury-rates historical returned {} records", rates.len());
        Ok(rates)
    }

    /// Fetch SOFR (Secured Overnight Financing Rate) data.
    ///
    /// SOFR is the broad repo rate used as the primary reference for USD derivatives.
    /// Endpoint: `/stable/sofr-rates`
    pub async fn sofr_rates(&self) -> anyhow::Result<Vec<SofrRate>> {
        self.rate_limiter.reset_if_new_day().await;
        if self.calls_remaining() == 0 {
            anyhow::bail!("FMP daily limit reached (250 calls/day on free tier)");
        }
        self.rate_limiter.acquire().await;

        let url = self.url("/stable/sofr-rates");
        debug!(
            "FMP sofr-rates: ({}/{} calls used)",
            self.calls_used(),
            DAILY_LIMIT_FREE
        );

        let rates: Vec<SofrRate> = self
            .client
            .get(url)
            .query(&[("apikey", self.api_key.as_str())])
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("FMP sofr-rates request failed: {e}"))?
            .error_for_status()
            .map_err(|e| anyhow::anyhow!("FMP sofr-rates error: {e}"))?
            .json()
            .await
            .map_err(|e| anyhow::anyhow!("FMP sofr-rates decode failed: {e}"))?;

        debug!("FMP sofr-rates returned {} records", rates.len());
        Ok(rates)
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
// Market Data Source (polling)
// ---------------------------------------------------------------------------

/// FMP market data source for continuous quotes.
/// Uses batch quote API for efficient multi-symbol fetching.
/// Fetches all symbols in one API call, then returns events one at a time.
pub struct FmpMarketDataSource {
    client: Arc<FmpClient>,
    symbols: Arc<Vec<String>>,
    poll_interval: Duration,
    state: Mutex<FmpMarketDataState>,
}

struct FmpMarketDataState {
    idx: usize,
    cached_quotes: Vec<FmpQuote>,
}

impl FmpMarketDataSource {
    /// Create a new FMP market data source.
    /// Requires FMP_API_KEY environment variable or pass api_key directly.
    pub fn new<I, S>(
        symbols: I,
        poll_interval: Duration,
        api_key: Option<String>,
    ) -> anyhow::Result<Self>
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let symbols_vec: Vec<String> = symbols.into_iter().map(Into::into).collect();
        if symbols_vec.is_empty() {
            anyhow::bail!("at least one symbol must be configured");
        }

        let api_key = api_key
            .or_else(|| std::env::var("FMP_API_KEY").ok())
            .filter(|k| !k.trim().is_empty())
            .context("FMP_API_KEY not set")?;

        let client = FmpClient::new(api_key, None)?;

        Ok(Self {
            client: Arc::new(client),
            symbols: Arc::new(symbols_vec),
            poll_interval,
            state: Mutex::new(FmpMarketDataState {
                idx: 0,
                cached_quotes: vec![],
            }),
        })
    }

    /// Fetch a fresh batch of quotes from FMP and cache them.
    async fn refresh_batch(&self) -> anyhow::Result<()> {
        let quotes = self.client.batch_quote(&self.symbols).await?;
        let mut state = self.state.lock().await;
        state.cached_quotes = quotes;
        state.idx = 0;
        Ok(())
    }
}

#[async_trait]
impl MarketDataSource for FmpMarketDataSource {
    async fn next(&self) -> anyhow::Result<MarketDataEvent> {
        let quote = {
            let mut state = self.state.lock().await;
            if state.cached_quotes.is_empty() {
                drop(state);
                self.refresh_batch().await?;
                let state = self.state.lock().await;
                state.cached_quotes.first().cloned()
            } else {
                let quote = state.cached_quotes.get(state.idx).cloned();
                state.idx = state.idx.wrapping_add(1);
                if state.idx >= state.cached_quotes.len() {
                    state.cached_quotes.clear();
                    state.idx = 0;
                }
                quote
            }
        };

        tokio::time::sleep(self.poll_interval).await;

        let quote = quote.ok_or_else(|| anyhow::anyhow!("FMP batch cache empty"))?;

        let symbol = quote.symbol.clone();
        let price = quote.price.unwrap_or(0.0);
        if price <= 0.0 {
            anyhow::bail!("FMP returned invalid price for {symbol}");
        }

        let day_high = quote.day_high.unwrap_or(price);
        let day_low = quote.day_low.unwrap_or(price);
        let spread = (day_high - day_low).max(price * 0.0001);
        let bid = price - spread / 2.0;
        let ask = price + spread / 2.0;

        debug!(
            "FMP quote for {}: bid={:.2}, ask={:.2} (price={:.2})",
            symbol, bid, ask, price
        );

        let event = MarketDataEventBuilder::default()
            .symbol(symbol)
            .bid(bid)
            .ask(ask)
            .last(price)
            .volume(quote.volume.unwrap_or(0) as u64)
            .timestamp(Utc::now())
            .source("fmp")
            .source_priority(60u32)
            .build()?;

        Ok(event)
    }
}

/// Factory for creating FMP market data sources.
pub struct FmpMarketDataSourceFactory;

impl SimpleMarketDataSourceFactory for FmpMarketDataSourceFactory {
    fn name(&self) -> &'static str {
        "fmp"
    }

    fn create(
        &self,
        symbols: &[String],
        interval: std::time::Duration,
    ) -> anyhow::Result<Box<dyn MarketDataSource>> {
        let source = FmpMarketDataSource::new(symbols.to_vec(), interval, None)?;
        Ok(Box::new(source))
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

    #[tokio::test]
    async fn stock_list_returns_symbols() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/stable/stock-list"))
            .and(query_param("apikey", "test-key"))
            .respond_with(ResponseTemplate::new(200).set_body_raw(
                r#"[{
                    "symbol": "AAPL",
                    "name": "Apple Inc",
                    "exchange": "NASDAQ",
                    "exchangeShortName": "NASDAQ"
                }, {
                    "symbol": "MSFT",
                    "name": "Microsoft Corporation",
                    "exchange": "NASDAQ",
                    "exchangeShortName": "NASDAQ"
                }]"#,
                "application/json",
            ))
            .mount(&server)
            .await;

        let items = client(&server.uri()).stock_list().await.unwrap();
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].symbol, "AAPL");
        assert_eq!(items[0].exchange_short_name.as_deref(), Some("NASDAQ"));
    }

    #[tokio::test]
    async fn search_symbol_returns_matches() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/stable/search-symbol"))
            .and(query_param("apikey", "test-key"))
            .and(query_param("query", "AA"))
            .respond_with(ResponseTemplate::new(200).set_body_raw(
                r#"[{
                    "symbol": "AA",
                    "name": "Alcoa Corporation",
                    "exchange": "NYSE",
                    "exchangeShortName": "NYSE",
                    "type": "stock"
                }, {
                    "symbol": "AAPL",
                    "name": "Apple Inc",
                    "exchange": "NASDAQ",
                    "exchangeShortName": "NASDAQ",
                    "type": "stock"
                }]"#,
                "application/json",
            ))
            .mount(&server)
            .await;

        let results = client(&server.uri()).search_symbol("AA").await.unwrap();
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].symbol, "AA");
        assert_eq!(results[0].instrument_type.as_deref(), Some("stock"));
    }

    #[tokio::test]
    async fn financial_statement_symbols_returns_list() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/stable/financial-statement-symbol-list"))
            .and(query_param("apikey", "test-key"))
            .respond_with(ResponseTemplate::new(200).set_body_raw(
                r#"["AAPL", "MSFT", "GOOG"]"#,
                "application/json",
            ))
            .mount(&server)
            .await;

        let symbols = client(&server.uri()).financial_statement_symbols().await.unwrap();
        assert_eq!(symbols.len(), 3);
        assert_eq!(symbols[0], "AAPL");
    }

    #[tokio::test]
    async fn treasury_rates_returns_current_rates() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/stable/treasury-rates"))
            .and(query_param("apikey", "test-key"))
            .respond_with(ResponseTemplate::new(200).set_body_raw(
                r#"{
                    "date": "2026-03-24",
                    "1month": 5.52,
                    "2month": 5.51,
                    "3month": 5.53,
                    "6month": 5.48,
                    "1year": 5.22,
                    "2year": 4.89,
                    "3year": 4.75,
                    "5year": 4.62,
                    "7year": 4.68,
                    "10year": 4.71,
                    "20year": 5.02,
                    "30year": 5.18
                }"#,
                "application/json",
            ))
            .mount(&server)
            .await;

        let rate = client(&server.uri()).treasury_rates().await.unwrap();
        assert_eq!(rate.date, "2026-03-24");
        assert!(rate.one_month.unwrap() > 0.0);
        assert!(rate.ten_year.unwrap() > 0.0);
        assert!(rate.thirty_year.unwrap() > rate.ten_year.unwrap());
    }

    #[tokio::test]
    async fn sofr_rates_returns_rate_history() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/stable/sofr-rates"))
            .and(query_param("apikey", "test-key"))
            .respond_with(ResponseTemplate::new(200).set_body_raw(
                r#"[{
                    "date": "2026-03-21",
                    "rate": 5.33
                }, {
                    "date": "2026-03-20",
                    "rate": 5.32
                }]"#,
                "application/json",
            ))
            .mount(&server)
            .await;

        let rates = client(&server.uri()).sofr_rates().await.unwrap();
        assert_eq!(rates.len(), 2);
        assert_eq!(rates[0].date, "2026-03-21");
        assert!(rates[0].rate.unwrap() > 0.0);
    }
}
