//! Snapshot types shared across backend crates.
//!
//! Types used by **multiple** crates (`api`, `nats_adapter`, etc.) live here.
//! Api-only types (`SystemSnapshot`, `SymbolSnapshot`) remain in `api/src/state.rs`.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Position & Order snapshots
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PositionSnapshot {
    pub id: String,
    pub symbol: String,
    pub quantity: i32,
    pub cost_basis: f64,
    pub mark: f64,
    pub unrealized_pnl: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub account_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HistoricPosition {
    pub id: String,
    pub symbol: String,
    pub quantity: i32,
    pub realized_pnl: f64,
    pub closed_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OrderSnapshot {
    pub id: String,
    pub symbol: String,
    pub side: String,
    pub quantity: i32,
    pub status: String,
    pub submitted_at: DateTime<Utc>,
}

// ---------------------------------------------------------------------------
// Candle
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CandleSnapshot {
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: u64,
    pub entry: f64,
    pub updated: DateTime<Utc>,
}

// ---------------------------------------------------------------------------
// Risk
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RiskStatus {
    pub allowed: bool,
    pub reason: Option<String>,
    pub updated_at: DateTime<Utc>,
}

impl Default for RiskStatus {
    fn default() -> Self {
        Self {
            allowed: true,
            reason: None,
            updated_at: Utc::now(),
        }
    }
}

// ---------------------------------------------------------------------------
// Metrics
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Metrics {
    pub net_liq: f64,
    pub buying_power: f64,
    pub excess_liquidity: f64,
    pub margin_requirement: f64,
    pub commissions: f64,
    pub portal_ok: bool,
    pub tws_ok: bool,
    pub tws_address: Option<String>,
    pub questdb_ok: bool,
    pub nats_ok: bool,
}

impl Default for Metrics {
    fn default() -> Self {
        Self {
            net_liq: 100_000.0,
            buying_power: 80_000.0,
            excess_liquidity: 25_000.0,
            margin_requirement: 15_000.0,
            commissions: 0.0,
            portal_ok: false,
            tws_ok: false,
            tws_address: None,
            questdb_ok: false,
            nats_ok: false,
        }
    }
}

// ---------------------------------------------------------------------------
// Alert
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Alert {
    pub level: String,
    pub message: String,
    pub timestamp: DateTime<Utc>,
}

impl Alert {
    pub fn info(message: impl Into<String>) -> Self {
        Self {
            level: "INFO".into(),
            message: message.into(),
            timestamp: Utc::now(),
        }
    }
}

// ---------------------------------------------------------------------------
// Strategy
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StrategyDecisionSnapshot {
    pub symbol: String,
    pub quantity: i32,
    pub side: String,
    pub mark: f64,
    pub created_at: DateTime<Utc>,
}

impl StrategyDecisionSnapshot {
    pub fn new(
        symbol: String,
        quantity: i32,
        side: impl Into<String>,
        mark: f64,
        created_at: DateTime<Utc>,
    ) -> Self {
        Self {
            symbol,
            quantity,
            side: side.into(),
            mark,
            created_at,
        }
    }
}

// ---------------------------------------------------------------------------
// Market Data
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, Default, Serialize, Deserialize, derive_builder::Builder)]
#[builder(setter(into, strip_option), default)]
pub struct MarketDataEvent {
    #[builder(default = "0")]
    pub contract_id: i64,
    #[builder(setter(into))]
    pub symbol: String,
    #[builder(default = "0.0")]
    pub bid: f64,
    #[builder(default = "0.0")]
    pub ask: f64,
    #[builder(default = "0.0")]
    pub last: f64,
    #[builder(default = "0")]
    pub volume: u64,
    #[builder(default = "Utc::now()")]
    pub timestamp: DateTime<Utc>,
    #[builder(default = "0")]
    pub quote_quality: u32,
}
