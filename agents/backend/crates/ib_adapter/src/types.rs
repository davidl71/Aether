//! IB Adapter types

use serde::{Deserialize, Serialize};

/// Option contract
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptionContract {
    pub symbol: String,
    pub expiry: String,
    pub strike: f64,
    pub is_call: bool,
}

impl OptionContract {
    pub fn new(symbol: &str, expiry: &str, strike: f64, is_call: bool) -> Self {
        Self {
            symbol: symbol.to_string(),
            expiry: expiry.to_string(),
            strike,
            is_call,
        }
    }
}

/// Order action
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub enum OrderAction {
    #[default]
    Buy,
    Sell,
}

/// Time in force
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
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
}
