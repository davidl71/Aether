use broker_engine::OptionContract;
use serde::{Deserialize, Serialize};

/// Legacy order action.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default, PartialEq)]
pub enum OrderAction {
    #[default]
    Buy,
    Sell,
}

/// Legacy time in force for execution-only order paths.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum TimeInForce {
    Day,
    GTC,
    IOC,
    FOK,
}

/// Legacy order status type retained for execution-oriented tests and tools.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OrderStatus {
    Submitted,
    Filled,
    PartiallyFilled,
    Cancelled,
    Rejected,
    Pending,
}

/// Legacy order representation.
#[derive(Debug, Clone, Serialize, Deserialize, derive_builder::Builder)]
pub struct Order {
    pub order_id: i32,
    pub contract: OptionContract,
    pub action: OrderAction,
    pub quantity: i32,
    pub limit_price: f64,
    pub tif: TimeInForce,
    pub status: OrderStatus,
}

/// Option contract with fully resolved IBKR metadata for legacy execution paths.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolvedOptionContract {
    pub symbol: String,
    pub expiry: String,
    pub strike: f64,
    pub is_call: bool,
    pub con_id: i32,
    pub exchange: String,
    pub multiplier: f64,
    pub trading_class: String,
}

impl ResolvedOptionContract {
    pub fn into_option_contract(self) -> OptionContract {
        OptionContract {
            symbol: self.symbol,
            expiry: self.expiry,
            strike: self.strike,
            is_call: self.is_call,
            con_id: Some(self.con_id),
        }
    }
}

/// One leg of a legacy BAG order.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BagOrderLeg {
    pub contract: OptionContract,
    pub ratio: i32,
    pub action: OrderAction,
}

/// Request to place a legacy BAG (combo) order.
#[derive(Debug, Clone, Serialize, Deserialize, derive_builder::Builder)]
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

/// Construct a legacy BAG order request for a box spread.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn construct_long_box_spread_request() {
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

        assert_eq!(req.order_action, OrderAction::Buy);
        assert_eq!(req.tif, TimeInForce::GTC);
        assert_eq!(req.legs.len(), 4);
    }

    #[test]
    fn construct_short_box_spread_request() {
        let req = construct_box_spread_order(
            "SPX",
            "20250321",
            5000.0,
            5010.0,
            OrderAction::Sell,
            1,
            1.50,
            "BOX",
            "USD",
            TimeInForce::Day,
        );

        assert_eq!(req.order_action, OrderAction::Sell);
        assert_eq!(req.tif, TimeInForce::Day);
        assert_eq!(req.legs.len(), 4);
    }
}
