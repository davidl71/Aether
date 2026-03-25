//! Legacy execution-only broker interfaces and order-domain types.
//!
//! This crate is intentionally excluded from the default workspace build. The
//! active read-only product direction uses `broker_engine` for market data,
//! positions, account state, and analytics without compiling execution code.

pub mod domain;
pub mod traits;

pub use domain::{
    construct_box_spread_order, BagOrderLeg, Order, OrderAction, OrderStatus, PlaceBagOrderRequest,
    ResolvedOptionContract, TimeInForce,
};
pub use traits::{BrokerExecution, OptionChainProvider};
