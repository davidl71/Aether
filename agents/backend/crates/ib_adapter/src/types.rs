//! IB Adapter types — re-exported from [`broker_engine::domain`].
//!
//! Types that are broker-agnostic live in `broker_engine::domain`.
//! This module re-exports them for backwards compatibility and adds
//! IBKR-specific types that are not part of the broker abstraction.

pub use broker_engine::domain::{
    construct_box_spread_order, BagOrderLeg, BrokerConfig, MarketDataEvent, OptionContract,
    OrderAction, OrderStatus, OrderStatusEvent, PlaceBagOrderRequest, Position, PositionEvent,
    TimeInForce,
};

pub use broker_engine::domain::AccountInfo;
pub use broker_engine::domain::MarketData;

// Backwards-compatible alias — IbAdapter and its callers use IbConfig
pub type IbConfig = broker_engine::domain::BrokerConfig;
