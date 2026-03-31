//! Shared utilities used across backend crates.

pub mod expiry;
pub mod backoff;
pub mod snapshot;

pub use snapshot::{
    Alert, CandleSnapshot, HistoricPosition, MarketDataEvent, MarketDataEventBuilder, Metrics,
    OrderSnapshot, PositionSnapshot, RiskStatus, StrategyDecisionSnapshot,
};
