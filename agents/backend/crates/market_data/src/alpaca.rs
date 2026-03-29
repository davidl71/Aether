//! Alpaca market data source.
//! Provides market data and position data from Alpaca API.

use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use chrono::Utc;
use tokio::sync::Mutex;
use tracing::{debug, warn};

use crate::model::{MarketDataSource, MarketDataSourceFactory};
use crate::{MarketDataEvent, MarketDataEventBuilder};

/// Alpaca quote data
#[derive(Debug, Clone)]
pub struct AlpacaQuote {
    pub symbol: String,
    pub bid_price: f32,
    pub ask_price: f32,
    pub bid_size: i32,
    pub ask_size: i32,
    pub timestamp: String,
}

/// Alpaca market data source.
pub struct AlpacaSource {
    is_paper: bool,
    symbols: Arc<Vec<String>>,
    poll_interval: Duration,
    state: Mutex<AlpacaState>,
}

struct AlpacaState {
    idx: usize,
}

impl AlpacaSource {
    /// Creates a new AlpacaSource polling the given symbols.
    pub fn new<I, S>(is_paper: bool, symbols: I, poll_interval: Duration) -> anyhow::Result<Self>
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let symbols_vec: Vec<String> = symbols.into_iter().map(Into::into).collect();
        if symbols_vec.is_empty() {
            anyhow::bail!("at least one symbol must be configured");
        }

        Ok(Self {
            is_paper,
            symbols: Arc::new(symbols_vec),
            poll_interval,
            state: Mutex::new(AlpacaState { idx: 0 }),
        })
    }

    /// Create new Alpaca source from environment.
    /// Credentials are resolved by the TUI/backend via credential management
    /// (keyring, env, or file) and passed as APCA_API_KEY_ID/APCA_API_SECRET_KEY.
    pub fn from_env() -> Option<(bool, String, String)> {
        let key_id = std::env::var("APCA_API_KEY_ID").ok()?;
        let secret_key = std::env::var("APCA_API_SECRET_KEY").ok()?;

        let is_paper = std::env::var("APCA_API_BASE_URL")
            .map(|url| url.contains("paper"))
            .unwrap_or(true);

        Some((is_paper, key_id, secret_key))
    }

    async fn next_symbol(&self) -> String {
        let mut state = self.state.lock().await;
        let idx = state.idx % self.symbols.len();
        state.idx = state.idx.wrapping_add(1);
        self.symbols[idx].clone()
    }

    /// Check if this is paper trading.
    pub fn is_paper(&self) -> bool {
        self.is_paper
    }

    /// Source name based on environment.
    pub fn source_name(&self) -> &'static str {
        if self.is_paper {
            "alpaca_paper"
        } else {
            "alpaca_live"
        }
    }

    /// Fetch latest quote for a symbol (synchronous API call).
    pub fn get_quote_sync(&self, symbol: &str) -> Result<AlpacaQuote, Box<dyn std::error::Error>> {
        use alpaca_api_client::market_data::stocks::LatestQuotesQuery;

        let query = LatestQuotesQuery::new(vec![symbol]);
        let quotes = query.send()?;

        let quote = quotes
            .get(symbol)
            .ok_or_else(|| format!("No quote returned for {}", symbol))?;

        Ok(AlpacaQuote {
            symbol: symbol.to_string(),
            bid_price: quote.bp,
            ask_price: quote.ap,
            bid_size: quote.bs,
            ask_size: quote.r#as,
            timestamp: quote.t.clone(),
        })
    }
}

#[async_trait]
impl MarketDataSource for AlpacaSource {
    async fn next(&self) -> anyhow::Result<MarketDataEvent> {
        tokio::time::sleep(self.poll_interval).await;
        let symbol = self.next_symbol().await;

        let _is_paper = self.is_paper;
        let symbol_clone = symbol.clone();
        let quote_result = tokio::task::spawn_blocking(move || {
            use alpaca_api_client::market_data::stocks::LatestQuotesQuery;
            let query = LatestQuotesQuery::new(vec![&symbol_clone]);
            query.send()
        })
        .await?;

        match quote_result {
            Ok(quotes) => {
                if let Some(quote) = quotes.get(&symbol) {
                    let bid = quote.bp as f64;
                    let ask = quote.ap as f64;

                    debug!(
                        "{} quote for {}: bid={:.2}, ask={:.2}",
                        self.source_name(),
                        symbol,
                        bid,
                        ask
                    );

                    let event = MarketDataEventBuilder::default()
                        .symbol(symbol)
                        .bid(bid)
                        .ask(ask)
                        .timestamp(Utc::now())
                        .source(self.source_name())
                        .source_priority(if self.is_paper { 55u32 } else { 75u32 })
                        .build()?;

                    Ok(event)
                } else {
                    warn!("Alpaca returned no quote for {}", symbol);
                    anyhow::bail!("no quote returned from Alpaca for {}", symbol)
                }
            }
            Err(e) => {
                warn!("Alpaca API error for {}: {}", symbol, e);
                anyhow::bail!("Alpaca API error: {}", e)
            }
        }
    }
}

/// Factory for creating Alpaca sources.
pub struct AlpacaSourceFactory;

impl MarketDataSourceFactory for AlpacaSourceFactory {
    fn name(&self) -> &'static str {
        "alpaca"
    }

    fn requires_config(&self) -> bool {
        true
    }

    fn create(
        &self,
        symbols: &[String],
        interval: std::time::Duration,
    ) -> anyhow::Result<Box<dyn MarketDataSource>> {
        let (is_paper, _key_id, _secret_key) = AlpacaSource::from_env()
            .ok_or_else(|| anyhow::anyhow!(
                "Alpaca credentials not configured. Set via TUI Settings > Sources or configure APCA_API_KEY_ID \
                and APCA_API_SECRET_KEY environment variables. Paper: https://paper-api.alpaca.markets \
                Live: https://api.alpaca.markets"
            ))?;

        let source = AlpacaSource::new(is_paper, symbols.iter().cloned(), interval)?;
        Ok(Box::new(source))
    }
}

/// Resolve Alpaca credentials from environment.
pub fn resolve_alpaca_credentials() -> Option<(String, String, bool)> {
    let key_id = std::env::var("APCA_API_KEY_ID").ok()?;
    let secret_key = std::env::var("APCA_API_SECRET_KEY").ok()?;
    let is_paper = std::env::var("APCA_API_BASE_URL")
        .map(|url| url.contains("paper"))
        .unwrap_or(true);

    Some((key_id, secret_key, is_paper))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alpaca_source_name() {
        let source = AlpacaSource::new(true, vec!["AAPL"], Duration::from_secs(1)).unwrap();
        assert_eq!(source.source_name(), "alpaca_paper");

        let source = AlpacaSource::new(false, vec!["AAPL"], Duration::from_secs(1)).unwrap();
        assert_eq!(source.source_name(), "alpaca_live");
    }

    #[test]
    fn test_alpaca_source_new_requires_symbols() {
        let result = AlpacaSource::new(true, Vec::<String>::new(), Duration::from_secs(1));
        assert!(result.is_err());
    }
}
