//! [`BrokerEngine`] trait — async interface for broker operations.
//!
//! ## Option Chain Resolution
//!
//! There are two interfaces for option chains:
//!
//! - **[`BrokerEngine::request_option_chain`](BrokerEngine::request_option_chain)** — returns
//!   `Vec<OptionContract>` with `con_id = None`. Lightweight, single API call. Suitable for
//!   market data, yield curve, and strike analysis.
//!
//! - **[`OptionChainProvider`]** — returns `Vec<ResolvedOptionContract>` with `con_id`,
//!   `exchange`, `multiplier`, and `trading_class` fully resolved. More expensive (may require
//!   multiple API calls per contract) but necessary for order placement.
//!
//! Callers should use `request_option_chain` when only market data is needed, and
//! `OptionChainProvider::resolve_option_chain` when placing orders or needing contract IDs.

use async_trait::async_trait;
use tokio::sync::mpsc;

use crate::domain::{
    AccountInfo, BrokerError, ConnectionState, MarketDataEvent, OptionContract, OrderAction,
    PlaceBagOrderRequest, PositionEvent, ResolvedOptionContract,
};

#[async_trait]
pub trait BrokerEngine: Send + Sync {
    // -------------------------------------------------------------------------
    // Lifecycle
    // -------------------------------------------------------------------------

    async fn connect(&self) -> Result<(), BrokerError>;
    async fn disconnect(&self) -> Result<(), BrokerError>;
    async fn state(&self) -> ConnectionState;

    // -------------------------------------------------------------------------
    // Market data
    // -------------------------------------------------------------------------

    async fn request_market_data(&self, symbol: &str, contract_id: i64) -> Result<(), BrokerError>;
    async fn request_option_chain(&self, symbol: &str) -> Result<Vec<OptionContract>, BrokerError>;

    // -------------------------------------------------------------------------
    // Orders
    // -------------------------------------------------------------------------

    async fn place_order(
        &self,
        contract: OptionContract,
        action: OrderAction,
        quantity: i32,
        limit_price: f64,
    ) -> Result<i32, BrokerError>;

    async fn place_bag_order(&self, request: PlaceBagOrderRequest) -> Result<i32, BrokerError>;
    async fn cancel_order(&self, order_id: i32) -> Result<(), BrokerError>;
    async fn cancel_all_orders(&self) -> Result<(), BrokerError>;

    // -------------------------------------------------------------------------
    // Positions & account
    // -------------------------------------------------------------------------

    async fn request_positions(&self) -> Result<Vec<PositionEvent>, BrokerError>;
    async fn request_account(&self) -> Result<AccountInfo, BrokerError>;

    // -------------------------------------------------------------------------
    // Sync fallback
    // -------------------------------------------------------------------------

    fn request_positions_sync(&self, timeout_ms: u64) -> Result<Vec<PositionEvent>, BrokerError>;

    // -------------------------------------------------------------------------
    // Capability discovery
    // -------------------------------------------------------------------------

    fn supports_options(&self) -> bool;
    fn supports_box_spreads(&self) -> bool;
    fn supports_combo_orders(&self) -> bool;

    // -------------------------------------------------------------------------
    // Event channels
    // -------------------------------------------------------------------------

    fn market_data_tx(&self) -> mpsc::Sender<MarketDataEvent>;
    fn position_tx(&self) -> mpsc::Sender<PositionEvent>;
    fn order_tx(&self) -> mpsc::Sender<crate::domain::OrderStatusEvent>;
}

// -----------------------------------------------------------------------------
// Option chain resolution
// -----------------------------------------------------------------------------

/// Unified interface for option chain resolution with full contract metadata.
///
/// Unlike [`BrokerEngine::request_option_chain`] which returns lightweight
/// [`OptionContract`] (con_id always `None`), this trait resolves all metadata
/// needed for order placement: conId, exchange, multiplier, trading_class.
///
/// Implementors:
/// - [`IbAdapter`](crate::ib_adapter::IbAdapter) — TWS socket via ibapi
/// - [`YatWSEngine`](crate::yatws_adapter::YatWSEngine) — TWS socket via yatws
/// - [`client_portal_options`](crate::api::client_portal_options) — REST (IB Gateway)
///
/// Each implementation uses a different API path but presents a single unified interface.
#[async_trait]
pub trait OptionChainProvider: Send + Sync {
    async fn resolve_option_chain(
        &self,
        symbol: &str,
    ) -> Result<Vec<ResolvedOptionContract>, BrokerError>;
}
