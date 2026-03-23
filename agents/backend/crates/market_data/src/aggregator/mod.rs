//! Market data aggregation module.
//!
//! Provides unified market data aggregation with source priority resolution.

mod sqlite_aggregator;

pub use sqlite_aggregator::{MarketDataAggregator, Quote, QuoteWithStaleness};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DataSource {
    Nautilus,
    Tws,
    Polygon,
    Fmp,
    Yahoo,
    Mock,
}

impl DataSource {
    pub fn priority(&self) -> u32 {
        match self {
            Self::Nautilus => 100,
            Self::Tws => 100,
            Self::Polygon => 70,
            Self::Fmp => 60,
            Self::Yahoo => 50,
            Self::Mock => 0,
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "nautilus" => Some(Self::Nautilus),
            "tws" => Some(Self::Tws),
            "polygon" => Some(Self::Polygon),
            "fmp" => Some(Self::Fmp),
            "yahoo" => Some(Self::Yahoo),
            "mock" => Some(Self::Mock),
            _ => None,
        }
    }
}

impl std::fmt::Display for DataSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Nautilus => write!(f, "nautilus"),
            Self::Tws => write!(f, "tws"),
            Self::Polygon => write!(f, "polygon"),
            Self::Fmp => write!(f, "fmp"),
            Self::Yahoo => write!(f, "yahoo"),
            Self::Mock => write!(f, "mock"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolvedQuote {
    pub symbol: String,
    pub bid: f64,
    pub ask: f64,
    pub last: f64,
    pub volume: i64,
    pub source: String,
    pub source_priority: u32,
    pub timestamp: DateTime<Utc>,
    pub stale: bool,
}

impl From<Quote> for ResolvedQuote {
    fn from(q: Quote) -> Self {
        let stale = q.is_stale();
        Self {
            symbol: q.symbol,
            bid: q.bid,
            ask: q.ask,
            last: q.last,
            volume: q.volume,
            source: q.source,
            source_priority: q.source_priority,
            timestamp: q.timestamp,
            stale,
        }
    }
}
