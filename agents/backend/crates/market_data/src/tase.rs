//! TASE (Tel Aviv Stock Exchange) Data Hub API client.
//!
//! Provides Israeli stock quotes and data via the TASE Data Hub API.
//!
//! **API:** https://datahubapi.tase.co.il
//!
//! **Rate Limits:** 10 requests per 2 seconds (rate + burst)
//!
//! Authentication: API key via `X-API-Key` header.

use anyhow::Context;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

const DEFAULT_BASE_URL: &str = "https://datahubapi.tase.co.il/api/v1";
const RATE_LIMIT_DELAY_MS: u64 = 200;

#[derive(Debug, Clone)]
pub struct TaseClient {
    api_key: String,
    client: Client,
    base_url: String,
}

impl TaseClient {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            client: Client::builder()
                .user_agent("Aether/1.0")
                .timeout(Duration::from_secs(10))
                .build()
                .unwrap_or_default(),
            base_url: DEFAULT_BASE_URL.to_string(),
        }
    }

    pub fn with_base_url(api_key: String, base_url: &str) -> Self {
        Self {
            api_key,
            client: Client::builder()
                .user_agent("Aether/1.0")
                .timeout(Duration::from_secs(10))
                .build()
                .unwrap_or_default(),
            base_url: base_url.to_string(),
        }
    }

    pub async fn quote(&self, symbol: &str) -> anyhow::Result<TaseQuote> {
        tokio::time::sleep(Duration::from_millis(RATE_LIMIT_DELAY_MS)).await;
        let url = format!("{}/quotes/{}", self.base_url, symbol);

        let response = self
            .client
            .get(&url)
            .header("X-API-Key", &self.api_key)
            .send()
            .await
            .context("TASE quote request failed")?;

        if !response.status().is_success() {
            anyhow::bail!("TASE API error: {} for symbol {}", response.status(), symbol);
        }

        let quote: TaseQuote = response.json().await.context("failed to parse TASE quote")?;
        Ok(quote)
    }

    pub async fn quotes(&self, symbols: &[String]) -> anyhow::Result<Vec<TaseQuote>> {
        let symbols_param = symbols.join(",");
        let url = format!("{}/quotes?symbols={}", self.base_url, symbols_param);

        let response = self
            .client
            .get(&url)
            .header("X-API-Key", &self.api_key)
            .send()
            .await
            .context("TASE quotes request failed")?;

        if !response.status().is_success() {
            anyhow::bail!("TASE API error: {}", response.status());
        }

        let quotes: Vec<TaseQuote> = response
            .json()
            .await
            .context("failed to parse TASE quotes")?;
        Ok(quotes)
    }

    pub async fn search(&self, query: &str) -> anyhow::Result<Vec<TaseSearchResult>> {
        let url = format!("{}/search?q={}", self.base_url, query);

        let response = self
            .client
            .get(&url)
            .header("X-API-Key", &self.api_key)
            .send()
            .await
            .context("TASE search request failed")?;

        if !response.status().is_success() {
            anyhow::bail!("TASE API error: {}", response.status());
        }

        let results: Vec<TaseSearchResult> = response
            .json()
            .await
            .context("failed to parse TASE search results")?;
        Ok(results)
    }

    pub async fn indices(&self) -> anyhow::Result<Vec<TaseIndex>> {
        let url = format!("{}/indices", self.base_url);

        let response = self
            .client
            .get(&url)
            .header("X-API-Key", &self.api_key)
            .send()
            .await
            .context("TASE indices request failed")?;

        if !response.status().is_success() {
            anyhow::bail!("TASE API error: {}", response.status());
        }

        let indices: Vec<TaseIndex> = response
            .json()
            .await
            .context("failed to parse TASE indices")?;
        Ok(indices)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaseQuote {
    pub symbol: String,
    pub name: Option<String>,
    pub last: f64,
    pub bid: Option<f64>,
    pub ask: Option<f64>,
    pub open: Option<f64>,
    pub high: Option<f64>,
    pub low: Option<f64>,
    pub volume: Option<i64>,
    pub change: Option<f64>,
    pub change_percent: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaseSearchResult {
    pub symbol: String,
    pub name: String,
    pub name_he: Option<String>,
    pub sector: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaseIndex {
    pub symbol: String,
    pub name: String,
    pub value: f64,
    pub change: Option<f64>,
    pub change_percent: Option<f64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tase_client_creation() {
        let client = TaseClient::new("test-key".to_string());
        assert_eq!(client.base_url, DEFAULT_BASE_URL);
    }

    #[tokio::test]
    async fn tase_quote_response_parse() {
        let json = r#"{
            "symbol": "ESPA",
            "name": "Espial Ltd",
            "last": 125.5,
            "bid": 125.0,
            "ask": 126.0,
            "open": 124.0,
            "high": 127.0,
            "low": 123.0,
            "volume": 50000,
            "change": 1.5,
            "change_percent": 1.21
        }"#;

        let quote: TaseQuote = serde_json::from_str(json).unwrap();
        assert_eq!(quote.symbol, "ESPA");
        assert_eq!(quote.last, 125.5);
        assert_eq!(quote.bid, Some(125.0));
        assert_eq!(quote.ask, Some(126.0));
    }
}
