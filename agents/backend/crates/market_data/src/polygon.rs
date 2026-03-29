use std::{sync::Arc, time::Duration};

use async_trait::async_trait;
use chrono::{DateTime, NaiveDate, TimeZone, Utc};
use reqwest::{Client, Response, StatusCode, Url};
use tokio::sync::Mutex;
use tracing::debug;

use crate::yahoo::{OptionContractData, OptionsDataSource, OptionsExpiration};
use crate::{MarketDataEvent, MarketDataEventBuilder, MarketDataSource, MarketDataSourceFactory};
use anyhow::Context;

const DEFAULT_BASE_URL: &str = "https://api.polygon.io";

#[derive(Debug)]
struct RoundRobinState {
    idx: usize,
}

pub struct PolygonMarketDataSource {
    client: Client,
    api_key: String,
    base_url: Url,
    symbols: Arc<Vec<String>>,
    poll_interval: Duration,
    state: Mutex<RoundRobinState>,
}

impl PolygonMarketDataSource {
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

        let event = MarketDataEventBuilder::default()
            .symbol(symbol)
            .bid(bid)
            .ask(ask)
            .timestamp(Self::convert_timestamp(timestamp))
            .source("polygon")
            .source_priority(70u32)
            .build()?;

        Ok(event)
    }
}

#[derive(Debug, serde::Deserialize)]
struct NbboResponse {
    #[serde(default)]
    pub status: Option<String>,
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

/// Factory for creating PolygonMarketDataSource instances.
/// Requires `POLYGON_API_KEY` environment variable.
#[derive(Debug)]
pub struct PolygonMarketDataSourceFactory;

impl PolygonMarketDataSourceFactory {
    pub fn new() -> Self {
        Self
    }
}

impl Default for PolygonMarketDataSourceFactory {
    fn default() -> Self {
        Self::new()
    }
}

impl MarketDataSourceFactory for PolygonMarketDataSourceFactory {
    fn name(&self) -> &'static str {
        "polygon"
    }

    fn create(
        &self,
        symbols: &[String],
        interval: Duration,
    ) -> anyhow::Result<Box<dyn MarketDataSource>> {
        let api_key = resolve_polygon_api_key()
            .context("Polygon API key not found (set POLYGON_API_KEY or store via credstore)")?;
        Ok(Box::new(PolygonMarketDataSource::new(
            symbols.to_vec(),
            interval,
            api_key,
            None,
        )?))
    }

    fn requires_config(&self) -> bool {
        true
    }
}

// ---------------------------------------------------------------------------
// Credential helper (mirrors api::credentials without a hard dep)
// ---------------------------------------------------------------------------

/// Resolve the Polygon API key from (in order):
/// 1. `POLYGON_API_KEY` environment variable
/// 2. `~/.config/aether/polygon_api_key.cred` file (written by the credstore)
fn resolve_polygon_api_key() -> Option<String> {
    if let Ok(v) = std::env::var("POLYGON_API_KEY") {
        if !v.trim().is_empty() {
            return Some(v);
        }
    }

    #[cfg(feature = "keyring")]
    {
        if let Ok(entry) = keyring::Entry::new("aether", "polygon_api_key") {
            if let Ok(v) = entry.get_password() {
                if !v.trim().is_empty() {
                    return Some(v.trim().to_string());
                }
            }
        }
    }

    // Fallback: cred file written by the credstore.
    // Mirror the path used by api::credentials (dirs::config_dir() + "aether").
    // On macOS: ~/Library/Application Support/aether/
    // On Linux: ~/.config/aether/
    let config_base = if cfg!(target_os = "macos") {
        std::env::var("HOME").ok().map(|h| {
            std::path::PathBuf::from(h)
                .join("Library")
                .join("Application Support")
        })
    } else {
        std::env::var("XDG_CONFIG_HOME")
            .ok()
            .map(std::path::PathBuf::from)
            .or_else(|| {
                std::env::var("HOME")
                    .ok()
                    .map(|h| std::path::PathBuf::from(h).join(".config"))
            })
    };
    let path = config_base?.join("aether").join("polygon_api_key.cred");
    std::fs::read_to_string(path)
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

// ---------------------------------------------------------------------------
// Options chain source
// ---------------------------------------------------------------------------

/// Polygon.io options source implementing [`OptionsDataSource`].
///
/// Uses the `/v3/snapshot/options/{underlying}` endpoint, which returns live
/// market data (bid/ask/greeks) for all listed contracts without needing to
/// enumerate them first.  Requires a Polygon Starter plan or higher.
///
/// Set `POLYGON_API_KEY` in the environment.
pub struct PolygonOptionsSource {
    client: Client,
    api_key: String,
    base_url: Url,
}

impl PolygonOptionsSource {
    pub fn new(api_key: impl Into<String>, base_url: Option<&str>) -> anyhow::Result<Self> {
        let base = base_url.unwrap_or(DEFAULT_BASE_URL);
        let base_url = Url::parse(base)
            .map_err(|err| anyhow::anyhow!("invalid polygon base url {base}: {err}"))?;
        let client = Client::builder()
            .user_agent("aether-backend/0.1")
            .build()
            .map_err(|err| anyhow::anyhow!("failed to build http client: {err}"))?;
        Ok(Self {
            client,
            api_key: api_key.into(),
            base_url,
        })
    }

    pub fn from_env() -> anyhow::Result<Self> {
        let api_key = resolve_polygon_api_key()
            .context("Polygon API key not found (set POLYGON_API_KEY or store via credstore)")?;
        Self::new(api_key, None)
    }

    /// Fetch one page of option snapshots for `underlying`.
    /// Follows `next_url` pagination automatically.
    async fn fetch_snapshots(
        &self,
        underlying: &str,
        expiration_date: Option<&str>,
    ) -> anyhow::Result<Vec<OptionSnapshotResult>> {
        let mut url = self.base_url.clone();
        url.set_path(&format!(
            "/v3/snapshot/options/{}",
            underlying.to_uppercase()
        ));

        let mut query: Vec<(&str, String)> = vec![
            ("apiKey", self.api_key.clone()),
            ("limit", "250".to_string()),
        ];
        if let Some(exp) = expiration_date {
            query.push(("expiration_date", exp.to_string()));
        }

        let mut all: Vec<OptionSnapshotResult> = Vec::new();
        let mut next_url: Option<String> = None;

        loop {
            let resp = if let Some(ref next) = next_url {
                // next_url already contains apiKey
                self.client.get(next).send().await
            } else {
                self.client.get(url.clone()).query(&query).send().await
            }
            .map_err(|e| anyhow::anyhow!("polygon options request failed: {e}"))?;

            if resp.status() == StatusCode::UNAUTHORIZED {
                anyhow::bail!("polygon options: invalid API key");
            }
            if !resp.status().is_success() {
                let status = resp.status();
                let body = resp.text().await.unwrap_or_default();
                anyhow::bail!("polygon options request failed {status}: {body}");
            }

            let page: OptionSnapshotPage = resp
                .json()
                .await
                .map_err(|e| anyhow::anyhow!("failed to decode polygon options response: {e}"))?;

            all.extend(page.results.unwrap_or_default());

            match page.next_url {
                Some(ref n) if !n.is_empty() => next_url = Some(n.clone()),
                _ => break,
            }
        }

        Ok(all)
    }
}

#[async_trait]
impl OptionsDataSource for PolygonOptionsSource {
    /// Returns unique expiration timestamps (Unix seconds, midnight UTC) for `symbol`.
    async fn get_expirations(&self, symbol: &str) -> anyhow::Result<Vec<i64>> {
        let snapshots = self.fetch_snapshots(symbol, None).await?;

        let mut seen = std::collections::HashSet::new();
        let mut timestamps: Vec<i64> = snapshots
            .iter()
            .filter_map(|s| {
                let exp_str = s.details.as_ref()?.expiration_date.as_deref()?;
                let date = NaiveDate::parse_from_str(exp_str, "%Y-%m-%d").ok()?;
                let ts = Utc
                    .from_utc_datetime(&date.and_hms_opt(0, 0, 0)?)
                    .timestamp();
                if seen.insert(ts) {
                    Some(ts)
                } else {
                    None
                }
            })
            .collect();

        timestamps.sort_unstable();
        Ok(timestamps)
    }

    /// Returns all contracts for a specific expiration timestamp.
    async fn get_chain(
        &self,
        symbol: &str,
        expiration_ts: i64,
    ) -> anyhow::Result<OptionsExpiration> {
        let expiry_date = Utc
            .timestamp_opt(expiration_ts, 0)
            .single()
            .map(|dt| dt.date_naive())
            .ok_or_else(|| anyhow::anyhow!("invalid expiration timestamp {expiration_ts}"))?;

        let exp_str = expiry_date.format("%Y-%m-%d").to_string();
        let snapshots = self.fetch_snapshots(symbol, Some(&exp_str)).await?;

        let mut calls: Vec<OptionContractData> = Vec::new();
        let mut puts: Vec<OptionContractData> = Vec::new();

        for s in snapshots {
            let details = match s.details {
                Some(ref d) => d,
                None => continue,
            };
            let contract_type = details.contract_type.as_deref().unwrap_or("");
            let strike = details.strike_price.unwrap_or(0.0);
            if strike <= 0.0 {
                continue;
            }

            let bid = s.last_quote.as_ref().and_then(|q| q.bid).unwrap_or(0.0);
            let ask = s.last_quote.as_ref().and_then(|q| q.ask).unwrap_or(0.0);
            let volume = s.day.as_ref().and_then(|d| d.volume).unwrap_or(0.0) as u64;
            let open_interest = s.open_interest.unwrap_or(0.0) as u64;
            let implied_volatility = s.implied_volatility.unwrap_or(0.0);
            let in_the_money = s.in_the_money.unwrap_or(false);

            let delta = s.greeks.as_ref().and_then(|g| g.delta);
            let gamma = s.greeks.as_ref().and_then(|g| g.gamma);
            let theta = s.greeks.as_ref().and_then(|g| g.theta);
            let vega = s.greeks.as_ref().and_then(|g| g.vega);

            let contract = OptionContractData {
                contract_symbol: details.ticker.clone().unwrap_or_default(),
                strike,
                bid,
                ask,
                volume,
                open_interest,
                implied_volatility,
                in_the_money,
                delta,
                gamma,
                theta,
                rho: None, // Polygon greeks don't include rho in snapshots
                vega,
            };

            match contract_type {
                "call" => calls.push(contract),
                "put" => puts.push(contract),
                _ => {
                    debug!("polygon options: unknown contract type {:?}", contract_type);
                }
            }
        }

        Ok(OptionsExpiration {
            expiration_date: expiry_date,
            calls,
            puts,
        })
    }
}

/// Factory for Polygon options source (requires API key)
pub struct PolygonOptionsSourceFactory;

impl PolygonOptionsSourceFactory {
    pub fn new() -> Self {
        Self
    }

    pub fn create(&self) -> anyhow::Result<Box<dyn OptionsDataSource>> {
        let source = PolygonOptionsSource::from_env()?;
        Ok(Box::new(source))
    }
}

impl Default for PolygonOptionsSourceFactory {
    fn default() -> Self {
        Self::new()
    }
}

// Polygon /v3/snapshot/options response shapes
#[derive(Debug, serde::Deserialize)]
struct OptionSnapshotPage {
    #[serde(default)]
    results: Option<Vec<OptionSnapshotResult>>,
    next_url: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
struct OptionSnapshotResult {
    details: Option<OptionDetails>,
    last_quote: Option<OptionQuote>,
    day: Option<OptionDay>,
    greeks: Option<OptionGreeks>,
    open_interest: Option<f64>,
    implied_volatility: Option<f64>,
    in_the_money: Option<bool>,
}

#[derive(Debug, serde::Deserialize)]
struct OptionDetails {
    ticker: Option<String>,
    contract_type: Option<String>,
    strike_price: Option<f64>,
    expiration_date: Option<String>, // "2026-04-17"
}

#[derive(Debug, serde::Deserialize)]
struct OptionQuote {
    bid: Option<f64>,
    ask: Option<f64>,
}

#[derive(Debug, serde::Deserialize)]
struct OptionDay {
    volume: Option<f64>,
}

#[derive(Debug, serde::Deserialize)]
struct OptionGreeks {
    delta: Option<f64>,
    gamma: Option<f64>,
    theta: Option<f64>,
    vega: Option<f64>,
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

    /// Live integration test — requires `POLYGON_API_KEY` env var and network access.
    /// Run with: cargo test -p market_data inspects_spy_option_chain_polygon -- --nocapture --ignored
    #[tokio::test]
    #[ignore]
    async fn inspects_spy_option_chain_polygon() {
        use crate::yahoo::OptionsDataSource;

        let source = match PolygonOptionsSource::from_env() {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Skipping: {e}");
                return;
            }
        };

        let timestamps = source.get_expirations("SPY").await.unwrap_or_default();
        eprintln!("SPY has {} expiration dates via Polygon", timestamps.len());

        let today = Utc::now().date_naive();
        let valid: Vec<i64> = timestamps
            .into_iter()
            .filter(|&ts| {
                if let Some(date) = Utc.timestamp_opt(ts, 0).single().map(|dt| dt.date_naive()) {
                    let dte = (date - today).num_days();
                    dte >= 7 && dte <= 60
                } else {
                    false
                }
            })
            .collect();
        eprintln!("Valid expirations (DTE 7-60): {}", valid.len());

        if let Some(&ts) = valid.first() {
            let date = Utc.timestamp_opt(ts, 0).single().map(|dt| dt.date_naive());
            eprintln!("First valid expiry: {:?} (ts={})", date, ts);

            let chain = source.get_chain("SPY", ts).await;
            match chain {
                Ok(c) => {
                    eprintln!("Chain: {} calls, {} puts", c.calls.len(), c.puts.len());
                    let spot = 680.0_f64;
                    let nearby: Vec<_> = c
                        .calls
                        .iter()
                        .filter(|o| (o.strike - spot).abs() <= 3.0 && o.bid > 0.0)
                        .collect();
                    for o in nearby.iter().take(5) {
                        eprintln!(
                            "  K={:.0} bid={:.2} ask={:.2} delta={:?}",
                            o.strike, o.bid, o.ask, o.delta
                        );
                    }
                }
                Err(e) => eprintln!("Chain fetch failed: {e}"),
            }
        }
    }
}
