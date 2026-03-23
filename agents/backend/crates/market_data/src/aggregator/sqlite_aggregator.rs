//! In-memory market data aggregator with source priority resolution.
//!
//! Stores the best quote per symbol, resolved by source_priority.
//! Only updates if the new event has higher or equal priority and fresher data.
//!
//! This is purely in-memory - quotes come continuously from data sources.

use super::ResolvedQuote;
use crate::MarketDataEvent;
use chrono::{DateTime, Duration, Utc};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::debug;

const QUOTE_TTL_SECONDS: i64 = 30;

pub struct MarketDataAggregator {
    memory: Arc<RwLock<HashMap<String, Quote>>>,
}

#[derive(Debug, Clone)]
pub struct Quote {
    pub symbol: String,
    pub bid: f64,
    pub ask: f64,
    pub last: f64,
    pub volume: i64,
    pub source: String,
    pub source_priority: u32,
    pub timestamp: DateTime<Utc>,
}

impl Quote {
    pub fn is_stale(&self) -> bool {
        let age = Utc::now() - self.timestamp;
        age.num_seconds() > QUOTE_TTL_SECONDS
    }

    pub fn staleness_ratio(&self) -> f64 {
        let age = Utc::now() - self.timestamp;
        (age.num_seconds() as f64) / (QUOTE_TTL_SECONDS as f64)
    }
}

pub struct QuoteWithStaleness {
    pub quote: Quote,
    pub is_stale: bool,
    pub staleness_ratio: f64,
}

impl MarketDataAggregator {
    pub fn new() -> Self {
        Self {
            memory: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn from_path(_path: &str) -> anyhow::Result<Self> {
        Ok(Self::new())
    }

    pub async fn in_memory() -> anyhow::Result<Self> {
        Ok(Self::new())
    }

    /// Process a market data event, updating the stored quote if priority permits.
    pub async fn process_event(&self, event: &MarketDataEvent) -> bool {
        let incoming_priority = event.source_priority;
        let symbol = event.symbol.clone();

        let should_update = {
            let memory = self.memory.read().await;
            match memory.get(&symbol) {
                Some(existing) => {
                    incoming_priority > existing.source_priority
                        || (incoming_priority == existing.source_priority
                            && event.timestamp > existing.timestamp)
                }
                None => true,
            }
        };

        if !should_update {
            debug!(
                symbol = %symbol,
                incoming_priority,
                "ignored lower priority event"
            );
            return false;
        }

        let quote = Quote {
            symbol: event.symbol.clone(),
            bid: event.bid,
            ask: event.ask,
            last: if event.last != 0.0 {
                event.last
            } else {
                (event.bid + event.ask) / 2.0
            },
            volume: event.volume as i64,
            source: event.source.clone(),
            source_priority: event.source_priority,
            timestamp: event.timestamp,
        };

        {
            let mut memory = self.memory.write().await;
            memory.insert(symbol.clone(), quote.clone());
        }

        debug!(
            symbol = %symbol,
            source = %quote.source,
            priority = quote.source_priority,
            "updated quote"
        );

        true
    }

    /// Get the best quote for a symbol from memory.
    pub async fn get_quote(&self, symbol: &str) -> Option<Quote> {
        let memory = self.memory.read().await;
        memory.get(symbol).cloned()
    }

    /// Get quote with staleness info.
    pub async fn get_quote_with_staleness(&self, symbol: &str) -> Option<QuoteWithStaleness> {
        let quote = self.get_quote(symbol).await?;
        let staleness_ratio = quote.staleness_ratio();
        Some(QuoteWithStaleness {
            quote,
            is_stale: staleness_ratio > 0.5,
            staleness_ratio,
        })
    }

    /// Get all quotes, optionally filtered by minimum priority.
    pub async fn get_all_quotes(&self, min_priority: Option<u32>) -> Vec<ResolvedQuote> {
        let memory = self.memory.read().await;
        memory
            .values()
            .filter(|q| {
                if let Some(min_p) = min_priority {
                    q.source_priority >= min_p
                } else {
                    true
                }
            })
            .cloned()
            .map(ResolvedQuote::from)
            .collect()
    }

    /// Clean up stale entries from memory.
    pub async fn cleanup_stale(&self, max_age_seconds: i64) -> usize {
        let cutoff = Utc::now() - Duration::seconds(max_age_seconds);
        let mut memory = self.memory.write().await;
        let before = memory.len();
        memory.retain(|_, q| q.timestamp > cutoff);
        before - memory.len()
    }
}

impl Default for MarketDataAggregator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::MarketDataEventBuilder;

    fn make_event(
        symbol: &str,
        bid: f64,
        ask: f64,
        source: &str,
        priority: u32,
    ) -> MarketDataEvent {
        MarketDataEventBuilder::default()
            .symbol(symbol)
            .bid(bid)
            .ask(ask)
            .last((bid + ask) / 2.0)
            .timestamp(Utc::now())
            .source(source)
            .source_priority(priority)
            .build()
            .unwrap()
    }

    #[tokio::test]
    async fn higher_priority_wins() {
        let agg = MarketDataAggregator::in_memory().await.unwrap();

        let tws = make_event("AAPL", 180.0, 180.5, "tws", 100);
        agg.process_event(&tws).await;

        let yahoo = make_event("AAPL", 181.0, 181.5, "yahoo", 50);
        agg.process_event(&yahoo).await;

        let quote = agg.get_quote("AAPL").await.unwrap();
        assert_eq!(quote.source, "tws");
        assert_eq!(quote.bid, 180.0);
    }

    #[tokio::test]
    async fn same_priority_updates() {
        let agg = MarketDataAggregator::in_memory().await.unwrap();

        let yahoo1 = make_event("AAPL", 180.0, 180.5, "yahoo", 50);
        agg.process_event(&yahoo1).await;

        let yahoo2 = make_event("AAPL", 181.0, 181.5, "yahoo", 50);
        agg.process_event(&yahoo2).await;

        let quote = agg.get_quote("AAPL").await.unwrap();
        assert_eq!(quote.bid, 181.0);
    }

    #[tokio::test]
    async fn upgrade_priority_replaces() {
        let agg = MarketDataAggregator::in_memory().await.unwrap();

        let yahoo = make_event("AAPL", 180.0, 180.5, "yahoo", 50);
        agg.process_event(&yahoo).await;

        let tws = make_event("AAPL", 179.0, 179.5, "tws", 100);
        agg.process_event(&tws).await;

        let quote = agg.get_quote("AAPL").await.unwrap();
        assert_eq!(quote.source, "tws");
        assert_eq!(quote.bid, 179.0);
    }
}
