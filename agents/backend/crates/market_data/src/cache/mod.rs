//! Market data cache types.
//!
//! ## Historical Data
//!
//! Historical candles and data can be cached via SQLite when needed.
//! The `SqliteCache` provides TTL-based expiration for candles and quotes.

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

pub type Result<T> = std::result::Result<T, CacheError>;

#[derive(Debug, thiserror::Error)]
pub enum CacheError {
    #[error("not found: {0}")]
    NotFound(String),

    #[error("expired: {key} (age: {age}s, ttl: {ttl}s)")]
    Expired { key: String, age: i64, ttl: i64 },

    #[error("other error: {0}")]
    Other(String),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CachedQuote {
    pub symbol: String,
    pub bid: f64,
    pub ask: f64,
    pub last: f64,
    pub volume: i64,
    pub timestamp: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CachedCandle {
    pub symbol: String,
    pub timestamp: DateTime<Utc>,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: i64,
}

impl CachedCandle {
    pub fn into_candle_snapshot(self) -> common::CandleSnapshot {
        common::CandleSnapshot {
            open: self.open,
            high: self.high,
            low: self.low,
            close: self.close,
            volume: self.volume as u64,
            entry: self.open,
            updated: self.timestamp,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CachedYieldCurve {
    pub symbol: String,
    pub points: Vec<CachedYieldPoint>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CachedYieldPoint {
    pub days_to_expiry: i32,
    pub mid_rate: f64,
    pub buy_implied_rate: f64,
    pub sell_implied_rate: f64,
    pub strike_width: f64,
}

pub struct Ttl {
    seconds: i64,
}

impl Ttl {
    pub fn seconds(s: i64) -> Self {
        Self { seconds: s }
    }

    pub fn minutes(m: i64) -> Self {
        Self { seconds: m * 60 }
    }

    pub fn hours(h: i64) -> Self {
        Self { seconds: h * 3600 }
    }

    pub fn days(d: i64) -> Self {
        Self { seconds: d * 86400 }
    }

    pub fn as_duration(&self) -> Duration {
        Duration::seconds(self.seconds)
    }

    pub fn as_seconds(&self) -> i64 {
        self.seconds
    }
}

impl Default for Ttl {
    fn default() -> Self {
        Self::minutes(5)
    }
}

pub struct Staleness {
    pub data_age_seconds: i64,
    pub ttl_seconds: i64,
}

impl Staleness {
    pub fn new(data_age_seconds: i64, ttl_seconds: i64) -> Self {
        Self {
            data_age_seconds,
            ttl_seconds,
        }
    }

    pub fn is_stale(&self) -> bool {
        self.data_age_seconds > self.ttl_seconds
    }

    pub fn staleness_ratio(&self) -> f64 {
        if self.ttl_seconds == 0 {
            return 1.0;
        }
        (self.data_age_seconds as f64) / (self.ttl_seconds as f64)
    }
}
