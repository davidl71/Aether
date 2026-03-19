//! IB Adapter types

use serde::{Deserialize, Serialize};

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
    GTC, // Good Till Canceled
    IOC, // Immediate or Cancel
    FOK, // Fill or Kill
}

/// Order status
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OrderStatus {
    Submitted,
    Filled,
    PartiallyFilled,
    Cancelled,
    Rejected,
    Pending,
}

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

/// Market data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketData {
    pub bid: f64,
    pub ask: f64,
    pub last: f64,
    pub volume: i64,
    pub timestamp: i64,
}

// -----------------------------------------------------------------------------
// BAG (combo) order types for multi-leg strategies (e.g. box spread)
// -----------------------------------------------------------------------------

/// One leg of a BAG order. When wired to ibapi, contract will be resolved to
/// conId and sent as ComboLeg(conId, ratio, action, exchange).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BagOrderLeg {
    pub contract: OptionContract,
    /// Ratio for this leg (e.g. 1 for box spread leg).
    pub ratio: i32,
    pub action: OrderAction,
}

/// Request to place a BAG (combo) order via the IB execution client.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaceBagOrderRequest {
    /// Underlying symbol for the combo (e.g. "SPX", "XSP").
    pub underlying_symbol: String,
    pub currency: String,
    /// Exchange for the combo (e.g. "BOX" for box options).
    pub exchange: String,
    pub legs: Vec<BagOrderLeg>,
    /// Total quantity of the combo (number of spreads).
    pub quantity: i32,
    /// Limit price for the whole combo; None for market.
    pub limit_price: Option<f64>,
    pub tif: TimeInForce,
    /// Side for the whole combo order.
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
