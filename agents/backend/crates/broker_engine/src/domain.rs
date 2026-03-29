//! Domain types for broker abstraction.
//!
//! These types are broker-agnostic and can be used by any implementation
//! of the [`BrokerEngine`](super::traits::BrokerEngine) trait.
//!
//! Under the current read-only exploration mode, execution-only order and BAG
//! domain types live in `broker_execution_legacy` rather than this active crate.

use rust_decimal::prelude::ToPrimitive;
use serde::{Deserialize, Serialize};

pub use crate::error::BrokerError;

pub use common::{MarketDataEvent, MarketDataEventBuilder};

// -----------------------------------------------------------------------------
// Option contract
// -----------------------------------------------------------------------------

/// Option contract
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptionContract {
    pub symbol: String,
    pub expiry: String,
    pub strike: f64,
    pub is_call: bool,
    /// IBKR contract ID (conId), resolved via `reqContractDetails`.
    pub con_id: Option<i32>,
}

impl OptionContract {
    pub fn new(symbol: &str, expiry: &str, strike: f64, is_call: bool) -> Self {
        Self {
            symbol: symbol.to_string(),
            expiry: expiry.to_string(),
            strike,
            is_call,
            con_id: None,
        }
    }

    /// Parse an OSI option symbol (e.g. `"SPXW231127C03850000"`) into an `OptionContract`.
    ///
    /// Expiry is formatted as `"YYYYMMDD"` to match IBKR BAG order conventions.
    /// Strike is converted from `Decimal` to `f64` (sufficient precision for equity options).
    pub fn from_osi(osi: &str) -> Result<Self, BrokerError> {
        use financial_symbols::OptionType;
        let parsed = financial_symbols::OptionContract::from_osi(osi)
            .map_err(|e| BrokerError::ContractError(e.to_string()))?;
        Ok(Self {
            symbol: parsed.ticker().to_string(),
            expiry: parsed.expiry().format("%Y%m%d").to_string(),
            strike: parsed.strike().to_f64().unwrap_or(0.0),
            is_call: matches!(parsed.option_type(), OptionType::Call),
            con_id: None,
        })
    }
}

// -----------------------------------------------------------------------------
// Position & account
// -----------------------------------------------------------------------------

/// Position
#[derive(Debug, Clone)]
pub struct Position {
    pub contract: OptionContract,
    pub quantity: i32,
    pub avg_price: f64,
    pub market_value: f64,
    pub unrealized_pnl: f64,
}

/// Account info
#[derive(Debug, Clone, Default)]
pub struct AccountInfo {
    pub account_id: String,
    pub net_liquidation: f64,
    pub cash_balance: f64,
    pub buying_power: f64,
    pub maintenance_margin: f64,
    pub initial_margin: f64,
}

impl From<&AccountInfo> for common::Metrics {
    fn from(info: &AccountInfo) -> Self {
        common::Metrics {
            net_liq: info.net_liquidation,
            buying_power: info.buying_power,
            excess_liquidity: info.cash_balance,
            margin_requirement: info.maintenance_margin,
            commissions: 0.0,
            portal_ok: false,
            tws_ok: false,
            tws_address: None,
            questdb_ok: false,
            nats_ok: false,
        }
    }
}

// -----------------------------------------------------------------------------
// Market data
// -----------------------------------------------------------------------------

/// Market data snapshot
#[derive(Debug, Clone)]
pub struct MarketData {
    pub bid: f64,
    pub ask: f64,
    pub last: f64,
    pub volume: i64,
    pub timestamp: i64,
}

// -----------------------------------------------------------------------------
// Event types
// -----------------------------------------------------------------------------

/// Quote quality flags derived from IBKR TickAttrib.
/// Bits: 0=stale, 1=pre_open, 2=non_auto_exec, 3=unreported, 4=bid_past_low, 5=ask_past_high
#[derive(Debug, Clone, Copy, Default)]
pub struct QuoteQuality(u8);

impl From<u8> for QuoteQuality {
    fn from(bits: u8) -> Self {
        Self(bits)
    }
}

impl QuoteQuality {
    pub const STALE: u8 = 1 << 0;
    pub const PRE_OPEN: u8 = 1 << 1;
    pub const NON_AUTO_EXEC: u8 = 1 << 2;
    pub const UNREPORTED: u8 = 1 << 3;
    pub const BID_PAST_LOW: u8 = 1 << 4;
    pub const ASK_PAST_HIGH: u8 = 1 << 5;

    #[inline]
    pub fn from_tick_attrib(can_auto_exec: bool, past_limit: bool, pre_open: bool) -> Self {
        let mut q = Self(0);
        if past_limit {
            q.0 |= Self::STALE;
        }
        if pre_open {
            q.0 |= Self::PRE_OPEN;
        }
        if !can_auto_exec {
            q.0 |= Self::NON_AUTO_EXEC;
        }
        q
    }

    #[inline]
    pub fn test(self, flag: u8) -> bool {
        (self.0 & flag) != 0
    }

    pub fn bits(self) -> u8 {
        self.0
    }
}

/// Position event
#[derive(Debug, Clone, derive_builder::Builder)]
pub struct PositionEvent {
    pub account: String,
    pub symbol: String,
    pub position: i32,
    pub avg_cost: f64,
}

/// Order status event
#[derive(Debug, Clone, derive_builder::Builder)]
pub struct OrderStatusEvent {
    pub order_id: i32,
    pub status: String,
    pub filled: i32,
    pub remaining: i32,
    pub avg_fill_price: f64,
}

// -----------------------------------------------------------------------------
// Connection state & config
// -----------------------------------------------------------------------------

/// Connection state
#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionState {
    Disconnected,
    Connecting,
    Connected,
    Error(String),
}

/// Broker adapter configuration
#[derive(Debug, Clone)]
pub struct BrokerConfig {
    pub host: String,
    pub port: u16,
    pub client_id: u32,
    pub paper_trading: bool,
}

impl Default for BrokerConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 7497,
            client_id: 0,
            paper_trading: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_option_contract_from_osi() {
        // SPXW 2023-11-27 Call 3850.00
        let c = OptionContract::from_osi("SPXW231127C03850000").unwrap();
        assert_eq!(c.symbol, "SPXW");
        assert_eq!(c.expiry, "20231127");
        assert!((c.strike - 3850.0).abs() < 0.01);
        assert!(c.is_call);
        assert!(c.con_id.is_none());

        // XSP 2025-03-21 Put 420.00
        let p = OptionContract::from_osi("XSP250321P00420000").unwrap();
        assert_eq!(p.symbol, "XSP");
        assert_eq!(p.expiry, "20250321");
        assert!((p.strike - 420.0).abs() < 0.01);
        assert!(!p.is_call);
    }
}
