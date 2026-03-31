//! Shared types and helpers used across backend crates.
//!
//! # Modules
//!
//! - [`snapshot`] — Snapshot and live-state DTOs shared by `api`, `nats_adapter`, and related crates
//!   (`PositionSnapshot`, `MarketDataEvent`, candles, orders, alerts). Prefer this crate over
//!   duplicating structs per crate (see project `no-new-duplicate-types` guidance).
//! - [`expiry`] — Option/expiry parsing helpers (`YYYY-MM-DD`).
//! - [`backoff`] — Small reusable backoff helpers for retry loops.
//!
//! Canonical project map: repository `AGENTS.md` (Rust workspace under `agents/backend/`).

pub mod expiry;
pub mod backoff;
pub mod snapshot;

pub use snapshot::{
    Alert, CandleSnapshot, HistoricPosition, MarketDataEvent, MarketDataEventBuilder, Metrics,
    OrderSnapshot, PositionSnapshot, RiskStatus, StrategyDecisionSnapshot,
};
