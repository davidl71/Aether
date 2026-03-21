//! [`BrokerEngine`] trait — async interface for broker operations.

use async_trait::async_trait;
use tokio::sync::mpsc;

use crate::domain::{
    AccountInfo, BrokerError, ConnectionState, MarketDataEvent, OptionContract, OrderAction,
    PlaceBagOrderRequest, PositionEvent,
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
    // Event channels
    // -------------------------------------------------------------------------

    fn market_data_tx(&self) -> mpsc::Sender<MarketDataEvent>;
    fn position_tx(&self) -> mpsc::Sender<PositionEvent>;
    fn order_tx(&self) -> mpsc::Sender<crate::domain::OrderStatusEvent>;
}
