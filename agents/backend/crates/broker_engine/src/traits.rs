//! [`BrokerEngine`] trait — async interface for broker operations.
//!
//! Under the current read-only exploration mode, this trait is intentionally
//! limited to market data, account, and position access. Legacy execution-only
//! order and resolved-contract interfaces live in `broker_execution_legacy`.

use async_trait::async_trait;

use crate::domain::{
    AccountInfo, BrokerError, ConnectionState, MarketDataEvent, OptionContract, PositionEvent,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MarketDataSubscriptionError {
    Lagged(u64),
    Closed,
}

#[async_trait]
pub trait MarketDataSubscription: Send {
    async fn recv(&mut self) -> Result<MarketDataEvent, MarketDataSubscriptionError>;
}

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
    fn subscribe_market_data(&self) -> Box<dyn MarketDataSubscription>;

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
}
