//! Market data providers (quotes, history, options chains, yield helpers) and ingestion glue.
//!
//! Quote sources register in [`provider_registry`] and are built with [`create_provider`]. Options
//! chains use [`create_options_provider`]. Concrete backends include Yahoo, FMP, Polygon, Alpaca,
//! TASE, SHIR helpers, cache layers, and mocks.
//!
//! # Design constraint
//!
//! This crate must **not** depend on `api` (workspace rule). Use [`credential_store`] only for shared credential paths.
//!
//! # See also
//!
//! - Provider pattern: `docs/MARKET_DATA_PROVIDER_ARCHITECTURE.md`
//! - Workspace map: `AGENTS.md`
//! - NATS subjects for published market data (when wired through services): `docs/NATS_TOPICS_REGISTRY.md` (`nats_adapter::topics::market_data`)

pub mod aggregator;
pub mod alpaca;
pub mod cache;
pub mod fmp;
pub mod mock;
pub mod model;
pub mod pipeline;
pub mod polygon;
pub mod polygon_ws;
pub mod shir;
pub mod tase;
pub mod yahoo;
pub mod yield_curve;

pub use aggregator::{DataSource, MarketDataAggregator, Quote, QuoteWithStaleness, ResolvedQuote};
pub use alpaca::{AlpacaSource, AlpacaSourceFactory};
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
pub use polygon::{
    PolygonMarketDataSource, PolygonMarketDataSourceFactory, PolygonOptionsSource,
    PolygonOptionsSourceFactory,
};
pub use polygon_ws::PolygonWsMarketDataSource;
pub use shir::{default_shir_rate, fetch_shir_rate, ShirRate};
pub use tase::{TaseClient, TaseIndex, TaseQuote, TaseSearchResult};
pub use yahoo::{
    OptionContractData, OptionsDataSource, OptionsExpiration, YahooFinanceSource,
    YahooFinanceSourceFactory, YahooHistorySource, YahooOptionsSource, YahooOptionsSourceFactory,
};
pub use yield_curve::{
    BoxSpreadResult, PolygonYieldCurveSource, YahooYieldCurveSource, YieldCurve, YieldCurvePoint,
};

use std::collections::HashMap;
use std::sync::OnceLock;

type DynFactory = Box<dyn MarketDataSourceFactory + Send + Sync>;
type DynOptionsFactory = Box<dyn Fn() -> anyhow::Result<Box<dyn OptionsDataSource>> + Send + Sync>;

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
        register(&mut m, "alpaca", AlpacaSourceFactory);
        m
    })
}

fn options_registry() -> &'static HashMap<&'static str, DynOptionsFactory> {
    static REGISTRY: OnceLock<HashMap<&'static str, DynOptionsFactory>> = OnceLock::new();
    REGISTRY.get_or_init(|| {
        let mut m = HashMap::new();
        m.insert(
            "yahoo",
            Box::new(|| Ok(Box::new(YahooOptionsSource::new()) as Box<dyn OptionsDataSource>))
                as DynOptionsFactory,
        );
        m.insert(
            "polygon",
            Box::new(|| {
                let source = PolygonOptionsSource::from_env()?;
                Ok(Box::new(source) as Box<dyn OptionsDataSource>)
            }) as DynOptionsFactory,
        );
        m
    })
}

/// Create an options source by provider name.
pub fn create_options_provider(name: &str) -> anyhow::Result<Box<dyn OptionsDataSource>> {
    let registry = options_registry();
    let factory = registry
        .get(name.to_lowercase().trim())
        .ok_or_else(|| anyhow::anyhow!("unknown options provider: {name}"))?;
    factory()
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
