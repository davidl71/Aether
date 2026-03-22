//! Yahoo Finance market data source.
//!
//! Fetches daily OHLCV data via `/v8/finance/chart/{symbol}` endpoint.
//! Returns bid/ask derived from `regularMarketPrice` with 0.01% spread.
//!
//! **Free tier:** No API key required. Rate-limited by Yahoo (respects poll_interval).

use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use chrono::Utc;
use reqwest::{Client, Url};
use serde::Deserialize;
use tokio::sync::Mutex;
use tracing::{debug, warn};

use crate::{
    MarketDataEvent, MarketDataEventBuilder, MarketDataSource, SimpleMarketDataSourceFactory,
};

const YAHOO_BASE_URL: &str = "https://query1.finance.yahoo.com";

/// Yahoo Finance polling source. Cycles through configured symbols in round-robin.
pub struct YahooFinanceSource {
    client: Client,
    base_url: Url,
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
        Self::with_base_url(symbols, poll_interval, YAHOO_BASE_URL)
    }

    /// Creates a new YahooFinanceSource with a custom base URL (for testing).
    fn with_base_url<I, S>(
        symbols: I,
        poll_interval: Duration,
        base_url: &str,
    ) -> anyhow::Result<Self>
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let symbols_vec: Vec<String> = symbols.into_iter().map(Into::into).collect();
        if symbols_vec.is_empty() {
            anyhow::bail!("at least one symbol must be configured");
        }

        let client = Client::builder()
            .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36")
            .build()
            .map_err(|err| anyhow::anyhow!("failed to create HTTP client: {err}"))?;

        let base_url =
            Url::parse(base_url).map_err(|err| anyhow::anyhow!("invalid base URL: {err}"))?;

        Ok(Self {
            client,
            base_url,
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

        let today = Utc::now().format("%Y-%m-%d").to_string();
        let week_ago = (Utc::now() - chrono::Duration::days(7))
            .format("%Y-%m-%d")
            .to_string();

        let mut url = self.base_url.clone();
        url.set_path(&format!("v8/finance/chart/{symbol}"));
        let response = self
            .client
            .get(url)
            .query(&[
                ("interval", "1d"),
                ("period1", week_ago.as_str()),
                ("period2", today.as_str()),
            ])
            .send()
            .await
            .map_err(|err| anyhow::anyhow!("yahoo request failed for {symbol}: {err}"))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            warn!("yahoo request for {symbol} failed with {status}: {body}");
            anyhow::bail!("yahoo request failed with status {status}");
        }

        let payload: YahooChartResponse = response.json().await.map_err(|err| {
            anyhow::anyhow!("failed to decode yahoo response for {symbol}: {err}")
        })?;

        let result = payload
            .chart
            .result
            .into_iter()
            .next()
            .ok_or_else(|| anyhow::anyhow!("yahoo response for {symbol} missing chart data"))?;

        let meta = result.meta;
        let regular_price = meta.regular_price.unwrap_or(0.0);

        if regular_price <= 0.0 {
            debug!("yahoo returned zero price for {symbol}");
            anyhow::bail!("invalid quote received from yahoo for {symbol}");
        }

        let spread = regular_price * 0.0001;
        let bid = regular_price - spread / 2.0;
        let ask = regular_price + spread / 2.0;

        let exchange = meta.exchange_name.as_deref().unwrap_or("N/A");
        debug!(
            "yahoo quote for {}: bid={bid:.2}, ask={ask:.2} (from {})",
            meta.symbol, exchange
        );

        let event = MarketDataEventBuilder::default()
            .symbol(meta.symbol)
            .bid(bid)
            .ask(ask)
            .timestamp(Utc::now())
            .build()?;

        Ok(event)
    }
}

#[derive(Debug, Deserialize)]
struct YahooChartResponse {
    chart: YahooChart,
}

#[derive(Debug, Deserialize)]
struct YahooChart {
    result: Vec<YahooChartResult>,
}

#[derive(Debug, Deserialize)]
struct YahooChartResult {
    meta: YahooMeta,
}

#[derive(Debug, Deserialize)]
struct YahooMeta {
    symbol: String,
    #[serde(rename = "regularMarketPrice", default)]
    regular_price: Option<f64>,
    #[serde(default)]
    exchange_name: Option<String>,
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
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn polls_round_robin_quotes() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/v8/finance/chart/SPY"))
            .respond_with(ResponseTemplate::new(200).set_body_raw(
                r#"{
          "chart": {
            "result": [{
              "meta": {
                "symbol": "SPY",
                "regularMarketPrice": 500.30,
                "exchangeName": "NYSE"
              }
            }]
          }
        }"#,
                "application/json",
            ))
            .mount(&server)
            .await;

        Mock::given(method("GET"))
            .and(path("/v8/finance/chart/QQQ"))
            .respond_with(ResponseTemplate::new(200).set_body_raw(
                r#"{
          "chart": {
            "result": [{
              "meta": {
                "symbol": "QQQ",
                "regularMarketPrice": 430.15,
                "exchangeName": "NASDAQ"
              }
            }]
          }
        }"#,
                "application/json",
            ))
            .mount(&server)
            .await;

        let source = YahooFinanceSource::with_base_url(
            ["SPY", "QQQ"],
            Duration::from_millis(10),
            &server.uri(),
        )
        .expect("failed to create source");

        let first = timeout(Duration::from_secs(1), source.next())
            .await
            .expect("first poll timed out")
            .expect("first poll failed");
        assert_eq!(first.symbol, "SPY");
        assert!(first.bid > 0.0);
        assert!(first.ask > first.bid);

        let second = timeout(Duration::from_secs(1), source.next())
            .await
            .expect("second poll timed out")
            .expect("second poll failed");
        assert_eq!(second.symbol, "QQQ");
        assert!(second.bid > 0.0);
        assert!(second.ask > second.bid);
    }
}
