pub mod aggregator;
pub mod cache;
pub mod fmp;
pub mod mock;
pub mod model;
pub mod pipeline;
pub mod polygon;
pub mod polygon_ws;
pub mod yahoo;
pub mod yield_curve;

pub use aggregator::{DataSource, MarketDataAggregator, Quote, QuoteWithStaleness, ResolvedQuote};
pub use cache::{
    CacheError, CachedCandle, CachedQuote, CachedYieldCurve, CachedYieldPoint, Staleness, Ttl,
};
pub use fmp::{
    BalanceSheet, CashFlowStatement, FmpClient, FmpMarketDataSource, FmpMarketDataSourceFactory,
    FmpQuote, FmpSearchResult, FmpStockListEntry, HistoricalCandle, IncomeStatement, SofrRate,
    TreasuryRate,
};
pub use mock::{MockMarketDataSource, MockMarketDataSourceFactory};
pub use model::{
    MarketDataEvent, MarketDataEventBuilder, MarketDataSource, MarketDataSourceFactory,
    SimpleMarketDataSourceFactory,
};
pub use pipeline::{MarketDataIngestor, MarketDataPipeline};
pub use polygon::{PolygonMarketDataSource, PolygonMarketDataSourceFactory};
pub use polygon_ws::PolygonWsMarketDataSource;
pub use yahoo::{
    OptionContractData, OptionsDataSource, OptionsExpiration, YahooFinanceSource,
    YahooFinanceSourceFactory, YahooHistorySource, YahooOptionsSource,
};
pub use yield_curve::{BoxSpreadResult, YahooYieldCurveSource, YieldCurve, YieldCurvePoint};

use std::collections::HashMap;
use std::sync::OnceLock;

type DynFactory = Box<dyn MarketDataSourceFactory + Send + Sync>;

fn register(
    registry: &mut HashMap<&'static str, DynFactory>,
    name: &'static str,
    factory: impl MarketDataSourceFactory + 'static,
) {
    registry.insert(name, Box::new(factory));
}

/// Returns the global provider registry, populated on first access.
pub fn provider_registry() -> &'static HashMap<&'static str, DynFactory> {
    static REGISTRY: OnceLock<HashMap<&'static str, DynFactory>> = OnceLock::new();
    REGISTRY.get_or_init(|| {
        let mut m = HashMap::new();
        register(&mut m, "yahoo", YahooFinanceSourceFactory);
        register(&mut m, "fmp", FmpMarketDataSourceFactory);
        register(&mut m, "mock", MockMarketDataSourceFactory);
        register(&mut m, "polygon", PolygonMarketDataSourceFactory);
        m
    })
}

/// Create a market data source by provider name.
pub fn create_provider(
    name: &str,
    symbols: &[String],
    interval: std::time::Duration,
) -> anyhow::Result<Box<dyn MarketDataSource>> {
    let registry = provider_registry();
    let factory = registry
        .get(name.to_lowercase().trim())
        .ok_or_else(|| anyhow::anyhow!("unknown market data provider: {name}"))?;

    if factory.requires_config() {
        anyhow::ensure!(
            !symbols.is_empty(),
            "at least one symbol must be configured for {name}"
        );
    }

    factory.create(symbols, interval)
}
