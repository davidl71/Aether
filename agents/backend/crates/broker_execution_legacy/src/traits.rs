use async_trait::async_trait;

use broker_engine::{BrokerError, OptionContract};

use crate::domain::{OrderAction, PlaceBagOrderRequest, ResolvedOptionContract};

/// Legacy execution-only broker interface retained outside the default build.
#[async_trait]
pub trait BrokerExecution: Send + Sync {
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
}

/// Legacy execution seam for resolving contract metadata needed by order flows.
#[async_trait]
pub trait OptionChainProvider: Send + Sync {
    async fn resolve_option_chain(
        &self,
        symbol: &str,
    ) -> Result<Vec<ResolvedOptionContract>, BrokerError>;
}
