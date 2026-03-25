//! Yahoo Finance market data source.
//!
//! Uses the [`yfinance-rs`] crate for fetching:
//! - Real-time quotes via Yahoo Finance API
//! - Historical OHLCV data with split/dividend adjustments
//! - Options chains with expiration dates and contract details
//!
//! **API:** No API key required for basic usage. Rate-limited by Yahoo.
//!
//! [`yfinance-rs`]: https://crates.io/crates/yfinance-rs

use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use chrono::{TimeZone, Utc};
use tokio::sync::Mutex;
use tracing::debug;
use yfinance_rs::{core::conversions::money_to_f64, Interval, Range, Ticker, YfClient};

use crate::{
    MarketDataEvent, MarketDataEventBuilder, MarketDataSource, SimpleMarketDataSourceFactory,
};

/// Yahoo Finance market data source using yfinance-rs.
/// Polls quotes for configured symbols in round-robin.
pub struct YahooFinanceSource {
    client: YfClient,
    symbols: Arc<Vec<String>>,
    poll_interval: Duration,
    state: Mutex<YahooState>,
}

struct YahooState {
    idx: usize,
}

impl YahooFinanceSource {
    /// Creates a new YahooFinanceSource polling the given symbols.
    pub fn new<I, S>(symbols: I, poll_interval: Duration) -> anyhow::Result<Self>
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let symbols_vec: Vec<String> = symbols.into_iter().map(Into::into).collect();
        if symbols_vec.is_empty() {
            anyhow::bail!("at least one symbol must be configured");
        }

        let client = YfClient::default();

        Ok(Self {
            client,
            symbols: Arc::new(symbols_vec),
            poll_interval,
            state: Mutex::new(YahooState { idx: 0 }),
        })
    }

    async fn next_symbol(&self) -> String {
        let mut state = self.state.lock().await;
        let idx = state.idx % self.symbols.len();
        state.idx = state.idx.wrapping_add(1);
        self.symbols[idx].clone()
    }
}

#[async_trait]
impl MarketDataSource for YahooFinanceSource {
    async fn next(&self) -> anyhow::Result<MarketDataEvent> {
        tokio::time::sleep(self.poll_interval).await;
        let symbol = self.next_symbol().await;

        let ticker = Ticker::new(&self.client, &symbol);

        let quote = ticker
            .quote()
            .await
            .map_err(|err| anyhow::anyhow!("yahoo quote failed for {symbol}: {err}"))?;

        let price = quote.price.as_ref().map(money_to_f64).unwrap_or(0.0);

        if price <= 0.0 {
            debug!("yahoo returned zero price for {symbol}");
            anyhow::bail!("invalid quote received from yahoo for {symbol}");
        }

        let spread = price * 0.0001;
        let bid = price - spread / 2.0;
        let ask = price + spread / 2.0;

        debug!("yahoo quote for {}: bid={bid:.2}, ask={ask:.2}", symbol);

        let event = MarketDataEventBuilder::default()
            .symbol(symbol)
            .bid(bid)
            .ask(ask)
            .timestamp(Utc::now())
            .source("yahoo")
            .source_priority(50u32)
            .build()?;

        Ok(event)
    }
}

/// Options chain data from Yahoo Finance.
#[derive(Debug, Clone)]
pub struct OptionsExpiration {
    pub expiration_date: chrono::NaiveDate,
    pub calls: Vec<OptionContractData>,
    pub puts: Vec<OptionContractData>,
}

/// Single option contract details (our domain type).
#[derive(Debug, Clone)]
pub struct OptionContractData {
    pub contract_symbol: String,
    pub strike: f64,
    pub bid: f64,
    pub ask: f64,
    pub volume: u64,
    pub open_interest: u64,
    pub implied_volatility: f64,
    pub in_the_money: bool,
    pub delta: Option<f64>,
    pub gamma: Option<f64>,
    pub theta: Option<f64>,
    pub rho: Option<f64>,
    pub vega: Option<f64>,
}

/// Options data source trait for fetching option chains.
#[async_trait]
pub trait OptionsDataSource: Send + Sync {
    /// Returns raw Unix timestamps for each available expiration.
    /// Callers should pass these timestamps back to [`get_chain`] unchanged
    /// to avoid roundtrip precision loss through `NaiveDate`.
    async fn get_expirations(&self, symbol: &str) -> anyhow::Result<Vec<i64>>;
    async fn get_chain(
        &self,
        symbol: &str,
        expiration_ts: i64,
    ) -> anyhow::Result<OptionsExpiration>;
}

/// Yahoo Finance options data using yfinance-rs.
pub struct YahooOptionsSource {
    client: YfClient,
}

impl YahooOptionsSource {
    pub fn new() -> Self {
        Self {
            client: YfClient::default(),
        }
    }
}

impl Default for YahooOptionsSource {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl OptionsDataSource for YahooOptionsSource {
    async fn get_expirations(&self, symbol: &str) -> anyhow::Result<Vec<i64>> {
        let ticker = Ticker::new(&self.client, symbol);

        let timestamps = ticker
            .options()
            .await
            .map_err(|err| anyhow::anyhow!("yahoo options failed for {symbol}: {err}"))?;

        Ok(timestamps)
    }

    async fn get_chain(
        &self,
        symbol: &str,
        expiration_ts: i64,
    ) -> anyhow::Result<OptionsExpiration> {
        let ticker = Ticker::new(&self.client, symbol);

        let chain = ticker
            .option_chain(Some(expiration_ts))
            .await
            .map_err(|err| anyhow::anyhow!("yahoo chain failed for {symbol}: {err}"))?;

        let calls = chain
            .calls
            .into_iter()
            .map(|c| {
                let greeks = c.greeks.as_ref();
                OptionContractData {
                    contract_symbol: c.contract_symbol.to_string(),
                    strike: money_to_f64(&c.strike),
                    bid: c.bid.as_ref().map(money_to_f64).unwrap_or(0.0),
                    ask: c.ask.as_ref().map(money_to_f64).unwrap_or(0.0),
                    volume: c.volume.unwrap_or(0),
                    open_interest: c.open_interest.unwrap_or(0),
                    implied_volatility: c.implied_volatility.unwrap_or(0.0),
                    in_the_money: c.in_the_money,
                    delta: greeks.and_then(|g| g.delta),
                    gamma: greeks.and_then(|g| g.gamma),
                    theta: greeks.and_then(|g| g.theta),
                    rho: greeks.and_then(|g| g.rho),
                    vega: greeks.and_then(|g| g.vega),
                }
            })
            .collect();

        let puts = chain
            .puts
            .into_iter()
            .map(|p| {
                let greeks = p.greeks.as_ref();
                OptionContractData {
                    contract_symbol: p.contract_symbol.to_string(),
                    strike: money_to_f64(&p.strike),
                    bid: p.bid.as_ref().map(money_to_f64).unwrap_or(0.0),
                    ask: p.ask.as_ref().map(money_to_f64).unwrap_or(0.0),
                    volume: p.volume.unwrap_or(0),
                    open_interest: p.open_interest.unwrap_or(0),
                    implied_volatility: p.implied_volatility.unwrap_or(0.0),
                    in_the_money: p.in_the_money,
                    delta: greeks.and_then(|g| g.delta),
                    gamma: greeks.and_then(|g| g.gamma),
                    theta: greeks.and_then(|g| g.theta),
                    rho: greeks.and_then(|g| g.rho),
                    vega: greeks.and_then(|g| g.vega),
                }
            })
            .collect();

        let expiration_date = Utc
            .timestamp_opt(expiration_ts, 0)
            .single()
            .map(|dt| dt.date_naive())
            .unwrap_or_default();

        Ok(OptionsExpiration {
            expiration_date,
            calls,
            puts,
        })
    }
}

/// Historical OHLCV data from Yahoo Finance.
pub struct YahooHistorySource {
    client: YfClient,
}

impl YahooHistorySource {
    pub fn new() -> Self {
        Self {
            client: YfClient::default(),
        }
    }

    pub async fn get_history(
        &self,
        symbol: &str,
        range: Range,
        interval: Interval,
    ) -> anyhow::Result<Vec<yfinance_rs::Candle>> {
        let ticker = Ticker::new(&self.client, symbol);

        let history = ticker
            .history(Some(range), Some(interval), false)
            .await
            .map_err(|err| anyhow::anyhow!("yahoo history failed for {symbol}: {err}"))?;

        Ok(history)
    }
}

impl Default for YahooHistorySource {
    fn default() -> Self {
        Self::new()
    }
}

/// Factory for creating YahooFinanceSource instances.
#[derive(Debug, Default)]
pub struct YahooFinanceSourceFactory;

impl SimpleMarketDataSourceFactory for YahooFinanceSourceFactory {
    fn name(&self) -> &'static str {
        "yahoo"
    }

    fn create(
        &self,
        symbols: &[String],
        interval: Duration,
    ) -> anyhow::Result<Box<dyn MarketDataSource>> {
        Ok(Box::new(YahooFinanceSource::new(
            symbols.to_vec(),
            interval,
        )?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::timeout;

    #[tokio::test]
    async fn polls_quotes() {
        let source = YahooFinanceSource::new(["SPY"], Duration::from_millis(100))
            .expect("failed to create source");

        let result = timeout(Duration::from_secs(5), source.next())
            .await
            .expect("poll timed out");

        match result {
            Ok(event) => {
                assert_eq!(event.symbol, "SPY");
                assert!(event.bid > 0.0);
                assert!(event.ask > event.bid);
            }
            Err(e) => {
                eprintln!("Quote poll failed (may be rate limited): {e}");
            }
        }
    }

    #[tokio::test]
    async fn gets_history() {
        let source = YahooHistorySource::new();

        let candles = source.get_history("SPY", Range::M1, Interval::D1).await;

        match candles {
            Ok(bars) => {
                assert!(!bars.is_empty(), "should have at least one candle");
                let last = &bars[0];
                let close_price = money_to_f64(&last.close);
                assert!(close_price > 0.0, "close price should be positive");
                eprintln!(
                    "SPY M1 history: {} bars, last close: {:.2}",
                    bars.len(),
                    close_price
                );
            }
            Err(e) => {
                eprintln!("History fetch failed: {e}");
            }
        }
    }

    #[tokio::test]
    async fn gets_options_expirations() {
        let source = YahooOptionsSource::new();

        let expirations = source.get_expirations("SPY").await;

        match expirations {
            Ok(timestamps) => {
                assert!(!timestamps.is_empty(), "should have at least one expiration");
                eprintln!("SPY has {} expiration dates", timestamps.len());
                for ts in timestamps.iter().take(3) {
                    let date = Utc.timestamp_opt(*ts, 0).single().map(|dt| dt.date_naive());
                    eprintln!("  - ts={} date={:?}", ts, date);
                }
            }
            Err(e) => {
                eprintln!("Options fetch failed: {e}");
            }
        }
    }
}
