//! Alpaca market data source.
//! Provides market data and position data from Alpaca API.

use async_trait::async_trait;
use tracing::debug;

use crate::model::{MarketDataSource, MarketDataSourceFactory};
use common::MarketDataEvent;

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
}

impl AlpacaSource {
    /// Create new Alpaca source from environment variables.
    pub fn from_env() -> Option<Self> {
        let has_key = std::env::var("APCA_API_KEY_ID").is_ok();
        let has_secret = std::env::var("APCA_API_SECRET_KEY").is_ok();
        
        if !has_key || !has_secret {
            return None;
        }
        
        let is_paper = std::env::var("APCA_API_BASE_URL")
            .map(|url| url.contains("paper"))
            .unwrap_or(true);
        
        Some(Self { is_paper })
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
        
        let quote = quotes.get(symbol)
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
        debug!("Alpaca market data source - full implementation pending");
        Ok(MarketDataEvent::default())
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
        _symbols: &[String],
        _interval: std::time::Duration,
    ) -> anyhow::Result<Box<dyn MarketDataSource>> {
        let source = AlpacaSource::from_env()
            .ok_or_else(|| anyhow::anyhow!(
                "Alpaca credentials not found. Set APCA_API_KEY_ID and APCA_API_SECRET_KEY environment variables. \
                For paper trading: https://paper-api.alpaca.markets \
                For live trading: https://api.alpaca.markets"
            ))?;
        
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
        let paper = AlpacaSource { is_paper: true };
        assert_eq!(paper.source_name(), "alpaca_paper");
        
        let live = AlpacaSource { is_paper: false };
        assert_eq!(live.source_name(), "alpaca_live");
    }
}
