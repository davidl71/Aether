//! Domain types for broker abstraction.
//!
//! These types are broker-agnostic and can be used by any implementation
//! of the [`BrokerEngine`](super::traits::BrokerEngine) trait.

use serde::{Deserialize, Serialize};

pub use crate::error::BrokerError;

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
}

// -----------------------------------------------------------------------------
// Order types
// -----------------------------------------------------------------------------

/// Order action
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default, PartialEq)]
pub enum OrderAction {
    #[default]
    Buy,
    Sell,
}

/// Time in force
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum TimeInForce {
    Day,
    GTC,
    IOC,
    FOK,
}

/// Order status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OrderStatus {
    Submitted,
    Filled,
    PartiallyFilled,
    Cancelled,
    Rejected,
    Pending,
}

/// Order
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    pub order_id: i32,
    pub contract: OptionContract,
    pub action: OrderAction,
    pub quantity: i32,
    pub limit_price: f64,
    pub tif: TimeInForce,
    pub status: OrderStatus,
}

// -----------------------------------------------------------------------------
// Position & account
// -----------------------------------------------------------------------------

/// Position
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub contract: OptionContract,
    pub quantity: i32,
    pub avg_price: f64,
    pub market_value: f64,
    pub unrealized_pnl: f64,
}

/// Account info
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AccountInfo {
    pub account_id: String,
    pub net_liquidation: f64,
    pub cash_balance: f64,
    pub buying_power: f64,
    pub maintenance_margin: f64,
    pub initial_margin: f64,
}

// -----------------------------------------------------------------------------
// Market data
// -----------------------------------------------------------------------------

/// Market data snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketData {
    pub bid: f64,
    pub ask: f64,
    pub last: f64,
    pub volume: i64,
    pub timestamp: i64,
}

// -----------------------------------------------------------------------------
// BAG (combo) order types
// -----------------------------------------------------------------------------

/// One leg of a BAG order. When wired to ibapi, contract will be resolved to
/// conId and sent as ComboLeg(conId, ratio, action, exchange).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BagOrderLeg {
    pub contract: OptionContract,
    pub ratio: i32,
    pub action: OrderAction,
}

/// Request to place a BAG (combo) order via the broker execution client.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaceBagOrderRequest {
    pub underlying_symbol: String,
    pub currency: String,
    pub exchange: String,
    pub legs: Vec<BagOrderLeg>,
    pub quantity: i32,
    pub limit_price: Option<f64>,
    pub tif: TimeInForce,
    pub order_action: OrderAction,
}

/// Construct a BAG order request for a box spread.
///
/// A box spread is a 4-leg synthetic combination:
/// - Leg 1: Long Call at k_low  (lower strike)
/// - Leg 2: Short Put at k_low  (same strike, same "synthetic long" leg)
/// - Leg 3: Short Call at k_high (upper strike)
/// - Leg 4: Long Put at k_high  (same strike, same "synthetic short" leg)
///
/// For a **long box** (bull call spread + bear put spread, net debit): side = BUY
/// For a **short box** (bear call spread + bull put spread, net credit): side = SELL
///
/// Each leg ratio = 1. The net cost = (k_high - k_low) minus intrinsic value at k_low.
///
/// **Note:** `con_id` must be resolved for each leg via `reqContractDetails` before
/// calling `place_bag_order`. See T-1773941020422614000.
#[allow(clippy::too_many_arguments)]
pub fn construct_box_spread_order(
    symbol: &str,
    expiry: &str,
    k_low: f64,
    k_high: f64,
    side: OrderAction,
    quantity: i32,
    limit_price: f64,
    exchange: &str,
    currency: &str,
    tif: TimeInForce,
) -> PlaceBagOrderRequest {
    let (call_low_action, put_low_action, call_high_action, put_high_action) = match side {
        OrderAction::Buy => (
            OrderAction::Buy,
            OrderAction::Sell,
            OrderAction::Sell,
            OrderAction::Buy,
        ),
        OrderAction::Sell => (
            OrderAction::Sell,
            OrderAction::Buy,
            OrderAction::Buy,
            OrderAction::Sell,
        ),
    };

    let legs = vec![
        BagOrderLeg {
            contract: OptionContract::new(symbol, expiry, k_low, true),
            ratio: 1,
            action: call_low_action,
        },
        BagOrderLeg {
            contract: OptionContract::new(symbol, expiry, k_low, false),
            ratio: 1,
            action: put_low_action,
        },
        BagOrderLeg {
            contract: OptionContract::new(symbol, expiry, k_high, true),
            ratio: 1,
            action: call_high_action,
        },
        BagOrderLeg {
            contract: OptionContract::new(symbol, expiry, k_high, false),
            ratio: 1,
            action: put_high_action,
        },
    ];

    PlaceBagOrderRequest {
        underlying_symbol: symbol.to_string(),
        currency: currency.to_string(),
        exchange: exchange.to_string(),
        legs,
        quantity,
        limit_price: Some(limit_price),
        tif,
        order_action: side,
    }
}

// -----------------------------------------------------------------------------
// Event types
// -----------------------------------------------------------------------------

/// Market data event
#[derive(Debug, Clone)]
pub struct MarketDataEvent {
    pub contract_id: i64,
    pub symbol: String,
    pub bid: f64,
    pub ask: f64,
    pub last: f64,
    pub volume: i64,
}

/// Position event
#[derive(Debug, Clone)]
pub struct PositionEvent {
    pub account: String,
    pub symbol: String,
    pub position: i32,
    pub avg_cost: f64,
}

/// Order status event
#[derive(Debug, Clone)]
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
    fn test_construct_long_box_spread() {
        let req = construct_box_spread_order(
            "SPX",
            "20250321",
            5000.0,
            5010.0,
            OrderAction::Buy,
            1,
            8.50,
            "BOX",
            "USD",
            TimeInForce::GTC,
        );

        assert_eq!(req.underlying_symbol, "SPX");
        assert_eq!(req.currency, "USD");
        assert_eq!(req.exchange, "BOX");
        assert_eq!(req.quantity, 1);
        assert_eq!(req.limit_price, Some(8.50));
        assert_eq!(req.order_action, OrderAction::Buy);
        assert_eq!(req.tif, TimeInForce::GTC);
        assert_eq!(req.legs.len(), 4);

        let [call_low, put_low, call_high, put_high] = &req.legs[..4] else {
            panic!("expected 4 legs")
        };

        assert_eq!(call_low.contract.symbol, "SPX");
        assert_eq!(call_low.contract.strike, 5000.0);
        assert!(call_low.contract.is_call);
        assert_eq!(call_low.action, OrderAction::Buy);
        assert_eq!(call_low.ratio, 1);

        assert_eq!(put_low.contract.symbol, "SPX");
        assert_eq!(put_low.contract.strike, 5000.0);
        assert!(!put_low.contract.is_call);
        assert_eq!(put_low.action, OrderAction::Sell);
        assert_eq!(put_low.ratio, 1);

        assert_eq!(call_high.contract.symbol, "SPX");
        assert_eq!(call_high.contract.strike, 5010.0);
        assert!(call_high.contract.is_call);
        assert_eq!(call_high.action, OrderAction::Sell);
        assert_eq!(call_high.ratio, 1);

        assert_eq!(put_high.contract.symbol, "SPX");
        assert_eq!(put_high.contract.strike, 5010.0);
        assert!(!put_high.contract.is_call);
        assert_eq!(put_high.action, OrderAction::Buy);
        assert_eq!(put_high.ratio, 1);
    }

    #[test]
    fn test_construct_short_box_spread() {
        let req = construct_box_spread_order(
            "XSP",
            "20250321",
            400.0,
            405.0,
            OrderAction::Sell,
            2,
            3.20,
            "SMART",
            "USD",
            TimeInForce::Day,
        );

        assert_eq!(req.quantity, 2);
        assert_eq!(req.order_action, OrderAction::Sell);

        let [call_low, put_low, call_high, put_high] = &req.legs[..4] else {
            panic!("expected 4 legs")
        };
        assert_eq!(call_low.action, OrderAction::Sell);
        assert_eq!(put_low.action, OrderAction::Buy);
        assert_eq!(call_high.action, OrderAction::Buy);
        assert_eq!(put_high.action, OrderAction::Sell);
    }

    #[test]
    fn test_box_spread_con_id_not_set() {
        let req = construct_box_spread_order(
            "SPX",
            "20250620",
            4900.0,
            4950.0,
            OrderAction::Buy,
            1,
            48.00,
            "BOX",
            "USD",
            TimeInForce::GTC,
        );

        for leg in &req.legs {
            assert!(leg.contract.con_id.is_none());
        }
    }
}
