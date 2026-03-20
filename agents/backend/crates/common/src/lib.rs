//! Shared utilities used across backend crates.

pub mod expiry;
pub mod snapshot;

pub use snapshot::{
    Alert, CandleSnapshot, HistoricPosition, Metrics, OrderSnapshot, PositionSnapshot, RiskStatus,
    StrategyDecisionSnapshot,
};
