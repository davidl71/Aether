use std::{sync::Arc, time::Duration};

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use reqwest::{Client, Response, StatusCode, Url};
use tokio::sync::Mutex;
use tracing::debug;

use crate::{MarketDataEvent, MarketDataSource};

const DEFAULT_BASE_URL: &str = "https://api.polygon.io";

#[derive(Debug)]
struct RoundRobinState {
    idx: usize,
}

/// A Polygon.io-backed market data source that polls NBBO quotes for a configured symbol list.
pub struct PolygonMarketDataSource {
    client: Client,
    api_key: String,
    base_url: Url,
    symbols: Arc<Vec<String>>,
    poll_interval: Duration,
    state: Mutex<RoundRobinState>,
}

impl PolygonMarketDataSource {
    /// Create a new Polygon market data source.
    ///
    /// * `symbols` - The list of symbols to poll (e.g., `["SPY", "QQQ"]`).
    /// * `poll_interval` - The delay between successive requests.
    /// * `api_key` - Polygon.io API key.
    /// * `base_url` - Optional override for the Polygon API base URL.
    pub fn new<I, S>(
        symbols: I,
        poll_interval: Duration,
        api_key: impl Into<String>,
        base_url: Option<&str>,
    ) -> anyhow::Result<Self>
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let symbols_vec: Vec<String> = symbols.into_iter().map(Into::into).collect();
        anyhow::ensure!(
            !symbols_vec.is_empty(),
            "at least one symbol must be configured"
        );

        let base = base_url.unwrap_or(DEFAULT_BASE_URL);
        let base_url = Url::parse(base)
            .map_err(|err| anyhow::anyhow!("invalid polygon base url {base}: {err}"))?;

        let client: Client = Client::builder()
            .user_agent("ib-box-spread-backend/0.1")
            .build()
            .map_err(|err| anyhow::anyhow!("failed to initialise http client: {err}"))?;

        Ok(Self {
            client,
            api_key: api_key.into(),
            base_url,
            symbols: Arc::new(symbols_vec),
            poll_interval,
            state: Mutex::new(RoundRobinState { idx: 0 }),
        })
    }

    async fn next_symbol(&self) -> String {
        let mut guard = self.state.lock().await;
        let idx = guard.idx % self.symbols.len();
        guard.idx = guard.idx.wrapping_add(1);
        self.symbols[idx].clone()
    }

    fn build_url(&self, symbol: &str) -> Url {
        let mut url = self.base_url.clone();
        url.set_path(&format!("/v2/last/nbbo/{symbol}"));
        url
    }

    fn convert_timestamp(ts: i64) -> DateTime<Utc> {
        // Polygon uses nanoseconds since epoch for NBBO, but be defensive.
        if ts > 1_000_000_000_000_000_000 {
            let secs = ts / 1_000_000_000;
            let nanos = (ts % 1_000_000_000) as u32;
            DateTime::from_timestamp(secs, nanos).unwrap_or_default()
        } else if ts > 1_000_000_000_000 {
            let secs = ts / 1_000_000;
            let micros = (ts % 1_000_000) as u32;
            DateTime::from_timestamp(secs, micros * 1_000).unwrap_or_default()
        } else if ts > 1_000_000_000 {
            let secs = ts / 1_000;
            let millis = (ts % 1_000) as u32;
            DateTime::from_timestamp(secs, millis * 1_000_000).unwrap_or_default()
        } else {
            DateTime::from_timestamp(ts, 0).unwrap_or_default()
        }
    }
}

#[async_trait]
impl MarketDataSource for PolygonMarketDataSource {
    async fn next(&self) -> anyhow::Result<MarketDataEvent> {
        tokio::time::sleep(self.poll_interval).await;
        let symbol = self.next_symbol().await;

        let url = self.build_url(&symbol);
        let response: Response = self
            .client
            .get(url)
            .query(&[("apiKey", self.api_key.as_str())])
            .send()
            .await
            .map_err(|err| anyhow::anyhow!("polygon request failed for {symbol}: {err}"))?;

        if response.status() == StatusCode::UNAUTHORIZED {
            anyhow::bail!("polygon request for {symbol} rejected: invalid API key");
        }

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("polygon request for {symbol} failed with {status}: {body}");
        }

        let payload: NbboResponse = response.json().await.map_err(|err| {
            anyhow::anyhow!("failed to decode polygon response for {symbol}: {err}")
        })?;

        let last = payload.last.ok_or_else(|| {
            anyhow::anyhow!("polygon response for {symbol} missing last quote data")
        })?;

        if let Some(ref s) = payload.status {
            debug!("polygon nbbo status: {s}");
        }

        let symbol = payload.symbol.unwrap_or(symbol);

        let bid = last.bid_price.unwrap_or(0.0);
        let ask = last.ask_price.unwrap_or(0.0);

        if bid <= 0.0 || ask <= 0.0 {
            debug!("polygon returned non-positive quote for {symbol}: bid={bid}, ask={ask}");
            anyhow::bail!("invalid quote received from polygon for {symbol}");
        }

        let timestamp = last.timestamp.unwrap_or(0);
        let event = MarketDataEvent {
            symbol,
            bid,
            ask,
            timestamp: Self::convert_timestamp(timestamp),
        };

        Ok(event)
    }
}

/// Polygon /v2/last/nbbo API response shape.
#[derive(Debug, serde::Deserialize)]
struct NbboResponse {
    /// API response status (e.g. "OK"). Logged at debug for diagnostics.
    #[serde(default)]
    pub status: Option<String>,
    /// Symbol from the response; used as event symbol when present.
    #[serde(default)]
    pub symbol: Option<String>,
    #[serde(default)]
    pub last: Option<NbboQuote>,
}

#[derive(Debug, serde::Deserialize)]
struct NbboQuote {
    #[serde(rename = "bid_price")]
    pub bid_price: Option<f64>,
    #[serde(rename = "ask_price")]
    pub ask_price: Option<f64>,
    #[serde(rename = "timestamp")]
    pub timestamp: Option<i64>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use tokio::time::timeout;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn polls_round_robin_quotes() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/v2/last/nbbo/SPY"))
            .respond_with(ResponseTemplate::new(200).set_body_raw(
                r#"{
          "status": "success",
          "symbol": "SPY",
          "last": {
            "bid_price": 100.5,
            "ask_price": 100.7,
            "timestamp": 1700000000000000000
          }
        }"#,
                "application/json",
            ))
            .mount(&server)
            .await;

        Mock::given(method("GET"))
            .and(path("/v2/last/nbbo/QQQ"))
            .respond_with(ResponseTemplate::new(200).set_body_raw(
                r#"{
          "status": "success",
          "symbol": "QQQ",
          "last": {
            "bid_price": 200.1,
            "ask_price": 200.3,
            "timestamp": 1700000100000000000
          }
        }"#,
                "application/json",
            ))
            .mount(&server)
            .await;

        let source = PolygonMarketDataSource::new(
            ["SPY", "QQQ"],
            Duration::from_millis(10),
            "test-key",
            Some(&server.uri()),
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
