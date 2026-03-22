pub use common::MarketDataEvent;
pub use common::MarketDataEventBuilder;

#[async_trait::async_trait]
pub trait MarketDataSource: Send + Sync {
    async fn next(&self) -> anyhow::Result<common::MarketDataEvent>;
}
