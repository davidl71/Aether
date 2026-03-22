pub use common::MarketDataEvent;
pub use common::MarketDataEventBuilder;

use std::time::Duration;

#[async_trait::async_trait]
pub trait MarketDataSource: Send + Sync {
    async fn next(&self) -> anyhow::Result<common::MarketDataEvent>;
}

#[async_trait::async_trait]
impl<T: MarketDataSource + ?Sized> MarketDataSource for Box<T> {
    async fn next(&self) -> anyhow::Result<common::MarketDataEvent> {
        (**self).next().await
    }
}

/// Factory trait for creating market data sources.
pub trait MarketDataSourceFactory: Send + Sync {
    /// Provider name (e.g., "yahoo", "polygon", "mock").
    fn name(&self) -> &'static str;
    /// Create a new instance of the provider.
    fn create(
        &self,
        symbols: &[String],
        interval: Duration,
    ) -> anyhow::Result<Box<dyn MarketDataSource>>;
    /// Whether this provider requires external config (e.g., API key).
    fn requires_config(&self) -> bool {
        false
    }
}

/// Simple factory for providers that don't need config.
pub trait SimpleMarketDataSourceFactory: Send + Sync {
    fn name(&self) -> &'static str;
    fn create(
        &self,
        symbols: &[String],
        interval: Duration,
    ) -> anyhow::Result<Box<dyn MarketDataSource>>;
}

impl<T: SimpleMarketDataSourceFactory + Send + Sync + 'static> MarketDataSourceFactory for T {
    fn name(&self) -> &'static str {
        SimpleMarketDataSourceFactory::name(self)
    }

    fn create(
        &self,
        symbols: &[String],
        interval: Duration,
    ) -> anyhow::Result<Box<dyn MarketDataSource>> {
        SimpleMarketDataSourceFactory::create(self, symbols, interval)
    }

    fn requires_config(&self) -> bool {
        false
    }
}
