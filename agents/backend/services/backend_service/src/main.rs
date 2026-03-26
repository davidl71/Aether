use std::{collections::HashMap, sync::Arc, time::Duration};

use anyhow::Context;
use api::{
    credentials::{get_credential, CredentialKey},
    shared_config::load_shared_config,
    Alert, HealthAggregateState, StrategyController, StrategyDecisionSnapshot, SystemSnapshot,
};
use async_trait::async_trait;
use broker_engine::{BrokerConfig, BrokerEngine, MarketDataSubscriptionError};
use chrono::Utc;
use ib_adapter::IbAdapter;
use market_data::{
    create_provider, FmpClient, MarketDataEvent, MarketDataIngestor, MarketDataSource,
};
use rand::{rngs::StdRng, Rng, SeedableRng};
use risk::{RiskCheck, RiskDecision, RiskEngine, RiskLimit, RiskViolation};
use runtime_state::{
    apply_market_event, apply_risk_status, apply_strategy_execution, RuntimeExecutionState,
    RuntimeMarketState, RuntimeProducerDecision,
};
use serde::Deserialize;
use strategy::model::TradeSide;
use strategy::{Decision as StrategyDecisionModel, StrategySignal};
use tokio::{
    sync::{
        mpsc::{UnboundedReceiver, UnboundedSender},
        watch, RwLock,
    },
    time::sleep,
};
use tracing::{debug, info, warn};

mod api_handlers;
mod collection_aggregation;
mod demo_seed;
mod dlq_consumer;
mod handlers;
mod health_aggregation;
mod ib_positions;
mod nats_integration;
mod rest_snapshot;
mod runtime_state;
mod shared_state;
mod snapshot_publisher;
mod swiftness;
mod yield_curve_writer;

use crate::demo_seed::{seed_demo_market_data, seed_demo_positions};
use crate::shared_state::SharedSnapshot;

#[derive(Debug, Default, Deserialize, Clone)]
struct BackendConfig {
    #[serde(default)]
    market_data: MarketDataSettings,
    #[serde(default)]
    broker: BrokerSettings,
    #[serde(default)]
    yield_curve: YieldCurveSettings,
    /// Seed demo positions/orders/historic trades on startup and project
    /// synthetic decision snapshots into the read-only runtime model.
    /// Off by default; enable with `mock_positions = true` in config or `MOCK_POSITIONS=true` env.
    /// Intended for CI/QA and local smoke-testing only.
    #[serde(default)]
    mock_positions: bool,
    /// Seed demo symbol quotes (market data) on startup.
    /// Off by default; enable with `mock_market_data = true` in config or `MOCK_MARKET_DATA=true` env.
    /// Intended for CI/QA and local smoke-testing only.
    #[serde(default)]
    mock_market_data: bool,
}

#[derive(Debug, Deserialize, Clone)]
struct BrokerSettings {
    #[serde(default = "default_broker_host")]
    host: String,
    #[serde(default = "default_broker_port")]
    port: u16,
    #[serde(default = "default_broker_client_id")]
    client_id: u32,
    #[serde(default = "default_broker_paper")]
    paper_trading: bool,
}

impl Default for BrokerSettings {
    fn default() -> Self {
        Self {
            host: default_broker_host(),
            port: default_broker_port(),
            client_id: default_broker_client_id(),
            paper_trading: default_broker_paper(),
        }
    }
}

fn default_broker_host() -> String {
    "127.0.0.1".into()
}
fn default_broker_port() -> u16 {
    7497
}
fn default_broker_client_id() -> u32 {
    2
}
fn default_broker_paper() -> bool {
    true
}

#[derive(Debug, Deserialize, Clone)]
struct MarketDataSettings {
    #[serde(default = "default_market_provider")]
    provider: String,
    #[serde(default = "default_market_symbols")]
    symbols: Vec<String>,
    #[serde(default = "default_poll_interval_ms")]
    poll_interval_ms: u64,
    #[serde(default)]
    polygon: Option<PolygonSettings>,
}

impl Default for MarketDataSettings {
    fn default() -> Self {
        Self {
            provider: default_market_provider(),
            symbols: default_market_symbols(),
            poll_interval_ms: default_poll_interval_ms(),
            polygon: None,
        }
    }
}

#[derive(Debug, Deserialize, Clone, Default)]
struct PolygonSettings {
    api_key: Option<String>,
    api_key_env: Option<String>,
    base_url: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
struct YieldCurveSettings {
    #[serde(default = "default_yield_curve_enabled")]
    enabled: bool,
    #[serde(default = "default_yield_curve_symbols")]
    symbols: Vec<String>,
    #[serde(default = "default_yield_curve_interval_secs")]
    interval_secs: u64,
    #[serde(default)]
    source: Option<String>,
}

impl Default for YieldCurveSettings {
    fn default() -> Self {
        Self {
            enabled: default_yield_curve_enabled(),
            symbols: default_yield_curve_symbols(),
            interval_secs: default_yield_curve_interval_secs(),
            source: None,
        }
    }
}

fn default_yield_curve_enabled() -> bool {
    true
}

fn default_yield_curve_symbols() -> Vec<String> {
    vec!["SPX".into(), "XSP".into(), "NDX".into()]
}

fn default_yield_curve_interval_secs() -> u64 {
    3600
}

fn default_market_provider() -> String {
    "yahoo".into()
}

fn default_market_symbols() -> Vec<String> {
    // Default to European-style symbols: SPX, XSP, NDX (European exercise style)
    // American-style symbols (SPY, QQQ, IWM) are hidden by default
    vec!["SPX".into(), "XSP".into(), "NDX".into()]
}

fn default_poll_interval_ms() -> u64 {
    800
}

/// Load API credentials from the keyring / credential files into env vars so that
/// downstream factories in the `market_data` crate (which only read `std::env::var`)
/// can find them. Explicit env vars always win — this only fills gaps.
///
/// Must be called before any threads are spawned.
fn bootstrap_credentials() {
    let pairs: &[(CredentialKey, &str)] = &[
        (CredentialKey::FmpApiKey, "FMP_API_KEY"),
        (CredentialKey::PolygonApiKey, "POLYGON_API_KEY"),
        (CredentialKey::FredApiKey, "FRED_API_KEY"),
        (CredentialKey::TaseApiKey, "TASE_API_KEY"),
    ];
    for &(key, env_name) in pairs {
        if std::env::var(env_name).is_ok() {
            continue; // already set — respect explicit env var
        }
        if let Some(val) = get_credential(key) {
            // Safety: single-threaded at this point in main(), before any spawns.
            #[allow(unused_unsafe)]
            unsafe {
                std::env::set_var(env_name, &val);
            }
            info!(credential = env_name, "loaded credential from keyring/file");
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    if std::env::args().any(|a| a == "--validate") {
        run_validate();
    }
    init_tracing();
    bootstrap_credentials();
    let config = load_config().context("failed to load backend config")?;
    let backend_id = resolve_backend_id();

    let state: SharedSnapshot = Arc::new(RwLock::new(SystemSnapshot::default()));
    let (strategy_ctrl_tx, strategy_ctrl_rx) = watch::channel(false);
    let controller = StrategyController::new(strategy_ctrl_tx);
    let health_state = HealthAggregateState::new_shared();

    // Initialize NATS integration (graceful degradation if unavailable)
    let nats_url = std::env::var("NATS_URL").ok();
    let nats_integration = Arc::new(nats_integration::NatsIntegration::new(nats_url.clone()).await);

    if nats_integration
        .as_ref()
        .as_ref()
        .is_some_and(|n| n.is_active())
    {
        info!("NATS integration active");
    } else {
        warn!("NATS integration unavailable, continuing without NATS");
    }

    let mock_positions_enabled = config.mock_positions
        || std::env::var("MOCK_POSITIONS")
            .map(|v| matches!(v.to_lowercase().as_str(), "1" | "true" | "yes"))
            .unwrap_or(false);
    let mock_market_data_enabled = config.mock_market_data
        || std::env::var("MOCK_MARKET_DATA")
            .map(|v| matches!(v.to_lowercase().as_str(), "1" | "true" | "yes"))
            .unwrap_or(false);

    {
        let mut snapshot = state.write().await;
        if mock_positions_enabled {
            seed_demo_positions(&mut snapshot);
        }
        if mock_market_data_enabled {
            seed_demo_market_data(&mut snapshot, &config.market_data.symbols);
        }
        snapshot.set_strategy_status("RUNNING");
        snapshot.risk.allowed = true;
        snapshot.risk.reason = None;
        snapshot.risk.updated_at = Utc::now();
        snapshot.market_data_source = Some(config.market_data.provider.clone());
    }
    let _ = controller.start();

    let risk_engine = Arc::new(RiskEngine::new(vec![Box::new(PositionLimitCheck::new(
        8, 250_000.0,
    ))]));

    let (strategy_signal_tx, strategy_signal_rx) =
        tokio::sync::mpsc::unbounded_channel::<StrategySignal>();
    let (strategy_decision_tx, strategy_decision_rx) =
        tokio::sync::mpsc::unbounded_channel::<StrategyDecisionModel>();

    if mock_positions_enabled {
        spawn_demo_strategy(strategy_signal_rx, strategy_decision_tx);
    }
    let fanout_ctrl_rx = strategy_ctrl_rx.clone();
    spawn_strategy_fanout(
        strategy_decision_rx,
        state.clone(),
        risk_engine.clone(),
        fanout_ctrl_rx,
        nats_integration.clone(),
    );

    let market_ctrl_rx = strategy_ctrl_rx;

    // Create shared aggregator for source priority resolution
    let market_aggregator = Arc::new(market_data::MarketDataAggregator::new());

    // Spawn broker market data loop if broker is enabled
    // This runs at highest priority (100) so it takes precedence over polling sources
    let broker_engine_for_market_data: Option<Arc<dyn BrokerEngine>> = if config
        .broker
        .paper_trading
        || std::env::var("IB_BROKER_ENABLED").is_ok()
    {
        let broker_config = BrokerConfig {
            host: config.broker.host.clone(),
            port: config.broker.port,
            client_id: config.broker.client_id,
            paper_trading: config.broker.paper_trading,
        };
        let adapter = IbAdapter::new(broker_config.clone());
        match adapter.connect().await {
            Ok(()) => {
                info!(host = %broker_config.host, port = %broker_config.port, "Connected to TWS for market data + order placement");
                Some(Arc::new(adapter) as Arc<dyn BrokerEngine>)
            }
            Err(e) => {
                warn!(error = %e, "Failed to connect to TWS for market data; using polling sources only");
                None
            }
        }
    } else {
        None
    };

    if let Some(ref engine) = broker_engine_for_market_data {
        let symbols = if config.market_data.symbols.is_empty() {
            default_market_symbols()
        } else {
            config.market_data.symbols.clone()
        };
        let agg = market_aggregator.clone();
        let state_clone = state.clone();
        let signal_clone = strategy_signal_tx.clone();
        let toggle_clone = market_ctrl_rx.clone();
        let nats_clone = nats_integration.clone();

        info!(provider = "tws", symbol_count = symbols.len(), "spawning TWS market data provider (priority 100, takes precedence over polling sources)");
        spawn_broker_market_data_loop(
            engine.clone(),
            symbols,
            state_clone,
            signal_clone,
            toggle_clone,
            nats_clone,
            agg,
        );
    }

    spawn_market_data_provider(
        &config.market_data,
        state.clone(),
        strategy_signal_tx,
        market_ctrl_rx,
        nats_integration.clone(),
        None, // Broker market data already spawned above
        market_aggregator.clone(),
    )?;

    health_aggregation::spawn_health_aggregator(health_state.clone(), nats_url.clone());
    collection_aggregation::spawn_collection_aggregator(state.clone(), nats_url.clone());
    dlq_consumer::spawn_dlq_consumer(nats_url.clone());

    // Publish full snapshots to NATS for TUI and other subscribers
    let loan_repo = api::LoanRepository::load_default().await.ok();
    let fmp_client = get_credential(CredentialKey::FmpApiKey).and_then(|api_key| {
        FmpClient::new(api_key, std::env::var("FMP_BASE_URL").ok().as_deref())
            .ok()
            .map(Arc::new)
    });
    if let Some(ref nats) = *nats_integration {
        if let Some(client) = nats.client() {
            let broker_engine: Option<Arc<dyn BrokerEngine>> = if let Some(ref engine) =
                broker_engine_for_market_data
            {
                info!("Reusing TWS connection for order placement");
                Some(engine.clone())
            } else if config.broker.paper_trading || std::env::var("IB_BROKER_ENABLED").is_ok() {
                info!("Broker order placement disabled (TWS connection failed earlier)");
                None
            } else {
                info!("Broker order placement disabled (enable with IB_BROKER_ENABLED or set paper_trading=true)");
                None
            };
            let interval_ms: u64 = std::env::var("SNAPSHOT_PUBLISH_INTERVAL_MS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(1000);
            let health_interval_secs: u64 = std::env::var("HEALTH_PUBLISH_INTERVAL_SECS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(15);
            let use_jetstream = matches!(
                std::env::var("NATS_USE_JETSTREAM")
                    .unwrap_or_default()
                    .trim()
                    .to_lowercase()
                    .as_str(),
                "1" | "true" | "yes"
            );
            let mut health_extra = HashMap::new();
            health_extra.insert("pid".to_string(), std::process::id().to_string());
            health_extra.insert("service".to_string(), "backend_service".to_string());
            health_extra.insert("snapshot_backend_id".to_string(), backend_id.clone());
            nats_adapter::spawn_health_publisher(
                client.clone(),
                "backend_service".to_string(),
                health_interval_secs,
                health_extra,
            );
            snapshot_publisher::spawn(
                state.clone(),
                client.clone(),
                backend_id.clone(),
                interval_ms,
                use_jetstream,
            );
            api_handlers::spawn(
                client.clone(),
                loan_repo.map(Arc::new),
                fmp_client,
                controller.clone(),
                state.clone(),
                None,
                broker_engine.clone(),
                backend_id.clone(),
            );

            if config.yield_curve.enabled {
                if let Some(nats_client) = nats.client() {
                    let _refresh_tx = yield_curve_writer::spawn(
                        nats_client,
                        config.yield_curve.symbols.clone(),
                        config.yield_curve.interval_secs,
                        config.yield_curve.source.clone(),
                    );
                    info!(
                        symbols = ?config.yield_curve.symbols,
                        interval_secs = config.yield_curve.interval_secs,
                        source = ?config.yield_curve.source,
                        "yield curve writer started"
                    );
                } else {
                    info!("yield curve writer skipped (NATS unavailable)");
                }
            }
        }
    }

    // Swiftness is temporarily disabled by default; enable explicitly for manual use.
    if swiftness::swiftness_enabled() {
        swiftness::spawn_swiftness_position_fetcher(state.clone());
    } else {
        info!("Swiftness integration disabled (set ENABLE_SWIFTNESS=1 to enable)");
    }

    // IB Client Portal positions: when IB_PORTAL_URL is set, merge real positions into snapshot.
    if ib_positions::ib_positions_enabled() {
        ib_positions::spawn_ib_position_fetcher(state.clone());
        info!("IB Client Portal position fetcher enabled (IB_PORTAL_URL set)");
    } else {
        info!("IB positions disabled (set IB_PORTAL_URL to enable, e.g. https://localhost:5001/v1/portal)");
    }

    rest_snapshot::spawn_if_enabled(
        state.clone(),
        health_state.clone(),
        nats_integration.clone(),
    );

    info!("backend service online (NATS primary; REST snapshot if REST_SNAPSHOT_PORT set)");

    tokio::signal::ctrl_c()
        .await
        .context("failed to listen for shutdown signal")?;

    Ok(())
}

fn run_validate() -> ! {
    match load_config() {
        Ok(_) => {
            println!("Config valid");
            std::process::exit(0);
        }
        Err(e) => {
            eprintln!("Config validation failed: {}", e);
            std::process::exit(1);
        }
    }
}

fn init_tracing() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_target(false)
        .init();
}

fn load_config() -> anyhow::Result<BackendConfig> {
    let path = std::env::var("BACKEND_CONFIG").unwrap_or_else(|_| "config/default.toml".into());
    let path = std::path::Path::new(&path);

    let mut base_value: toml::Value = if path.exists() {
        let data = std::fs::read_to_string(path)
            .with_context(|| format!("unable to read config file {}", path.display()))?;
        toml::from_str(&data).with_context(|| format!("invalid config file {}", path.display()))?
    } else {
        toml::Value::Table(toml::map::Map::new())
    };

    // Optional gitignored local override (e.g. config/config.local.toml)
    let local_path = path
        .parent()
        .map(|p| p.join("config.local.toml"))
        .unwrap_or_else(|| std::path::PathBuf::from("config/config.local.toml"));
    if local_path.exists() {
        let data = std::fs::read_to_string(&local_path)
            .with_context(|| format!("unable to read local config {}", local_path.display()))?;
        let local: toml::Value = toml::from_str(&data)
            .with_context(|| format!("invalid local config {}", local_path.display()))?;
        merge_toml_over(&local, &mut base_value);
        info!(path = %local_path.display(), "Loaded backend config local override");
    }

    let cfg: BackendConfig =
        toml::from_str(&toml::to_string(&base_value).context("serialize merged config")?)
            .context("invalid merged backend config")?;
    Ok(cfg)
}

fn resolve_backend_id() -> String {
    if let Ok(value) = std::env::var("BACKEND_ID") {
        let normalized = value.trim().to_lowercase();
        if !normalized.is_empty() {
            return normalized;
        }
    }

    if let Ok(Some(loaded)) = load_shared_config() {
        let shared_backend_id = loaded
            .value
            .get("dataSources")
            .and_then(|d| d.get("primary"))
            .and_then(|v| v.as_str())
            .or_else(|| {
                loaded
                    .value
                    .get("broker")
                    .and_then(|b| b.get("primary"))
                    .and_then(|v| v.as_str())
            })
            .unwrap_or("")
            .trim()
            .to_lowercase();
        if !shared_backend_id.is_empty() {
            return shared_backend_id;
        }
    }

    "ib".to_string()
}

/// Merge `overlay` into `base` in place (overlay keys take precedence).
fn merge_toml_over(overlay: &toml::Value, base: &mut toml::Value) {
    use toml::Value;
    match (overlay, base) {
        (Value::Table(over_t), Value::Table(base_t)) => {
            for (k, v) in over_t {
                if let Some(base_v) = base_t.get_mut(k) {
                    merge_toml_over(v, base_v);
                } else {
                    base_t.insert(k.clone(), v.clone());
                }
            }
        }
        (over, base) => *base = over.clone(),
    }
}

fn spawn_market_data_provider(
    settings: &MarketDataSettings,
    state: SharedSnapshot,
    strategy_signal: UnboundedSender<StrategySignal>,
    strategy_toggle: watch::Receiver<bool>,
    nats: Arc<Option<nats_integration::NatsIntegration>>,
    broker_engine: Option<Arc<dyn BrokerEngine>>,
    aggregator: Arc<market_data::MarketDataAggregator>,
) -> anyhow::Result<()> {
    let symbols = if settings.symbols.is_empty() {
        default_market_symbols()
    } else {
        settings.symbols.clone()
    };
    let interval = Duration::from_millis(settings.poll_interval_ms.max(10));

    // If broker engine is available and connected, spawn market data loop first (highest priority: 100)
    // This ensures TWS market data takes precedence over polling sources
    if let Some(ref engine) = broker_engine {
        let state_clone = state.clone();
        let signal_clone = strategy_signal.clone();
        let toggle_clone = strategy_toggle.clone();
        let nats_clone = nats.clone();
        let agg = aggregator.clone();
        let symbols_clone = symbols.clone();

        info!(
            provider = "tws",
            symbol_count = symbols.len(),
            "spawning TWS market data provider (priority 100)"
        );
        spawn_broker_market_data_loop(
            engine.clone(),
            symbols_clone,
            state_clone,
            signal_clone,
            toggle_clone,
            nats_clone,
            agg,
        );
    }

    let providers = if settings.provider == "all" {
        vec!["yahoo", "fmp", "polygon"] // Yahoo (free) + FMP (paid) + Polygon (paid)
    } else if settings.provider == "yahoo"
        || settings.provider == "fmp"
        || settings.provider == "polygon"
        || settings.provider == "mock"
    {
        vec![settings.provider.as_str()]
    } else {
        vec!["yahoo"] // Default to Yahoo for real data
    };

    for provider_name in providers {
        let source = create_provider(provider_name, &symbols, interval)
            .with_context(|| format!("failed to create market data provider: {}", provider_name))?;

        let agg = aggregator.clone();
        let state_clone = state.clone();
        let signal_clone = strategy_signal.clone();
        let toggle_clone = strategy_toggle.clone();
        let nats_clone = nats.clone();

        info!(provider = %provider_name, symbol_count = symbols.len(), "spawning market data provider");
        spawn_market_data_loop(
            source,
            state_clone,
            signal_clone,
            toggle_clone,
            nats_clone,
            agg,
        );
    }

    Ok(())
}

fn spawn_market_data_loop(
    source: Box<dyn MarketDataSource + Send + Sync>,
    state: SharedSnapshot,
    strategy_signal: UnboundedSender<StrategySignal>,
    strategy_toggle: watch::Receiver<bool>,
    nats: Arc<Option<nats_integration::NatsIntegration>>,
    aggregator: Arc<market_data::MarketDataAggregator>,
) {
    tokio::spawn(async move {
        let ingestor = MarketDataIngestor::new(source);

        loop {
            match ingestor.poll().await {
                Ok(event) => {
                    let running = *strategy_toggle.borrow();

                    let _updated = aggregator.process_event(&event).await;

                    let best_quote = aggregator.get_quote(&event.symbol).await;

                    handle_market_event(
                        &state,
                        &strategy_signal,
                        &event,
                        running,
                        nats.as_ref().as_ref(),
                        best_quote.as_ref(),
                    )
                    .await;
                }
                Err(err) => {
                    warn!(%err, "market data poll failed, retrying");
                    sleep(Duration::from_secs(1)).await;
                }
            }
        }
    });
}

/// Spawn a market data loop that subscribes the active broker engine to
/// push-based TWS market data.
///
/// The remaining gap is service-side consumption and forwarding into the shared
/// aggregator/NATS path, not quote production inside `IbAdapter`.
fn spawn_broker_market_data_loop(
    engine: Arc<dyn BrokerEngine>,
    symbols: Vec<String>,
    state: SharedSnapshot,
    strategy_signal: UnboundedSender<StrategySignal>,
    strategy_toggle: watch::Receiver<bool>,
    nats: Arc<Option<nats_integration::NatsIntegration>>,
    aggregator: Arc<market_data::MarketDataAggregator>,
) {
    tokio::spawn(async move {
        let mut market_data_rx = engine.subscribe_market_data();

        // Subscribe to market data for each symbol
        for symbol in &symbols {
            match engine.request_market_data(symbol, 0).await {
                Ok(()) => {
                    debug!(symbol = %symbol, "subscribed to market data via broker");
                }
                Err(e) => {
                    warn!(symbol = %symbol, error = %e, "failed to subscribe to market data");
                }
            }
        }

        info!("broker market data loop started (subscriptions active; aggregator bridge enabled)");

        loop {
            match market_data_rx.recv().await {
                Ok(event) => {
                    let running = *strategy_toggle.borrow();
                    let _updated = aggregator.process_event(&event).await;
                    let best_quote = aggregator.get_quote(&event.symbol).await;

                    handle_market_event(
                        &state,
                        &strategy_signal,
                        &event,
                        running,
                        nats.as_ref().as_ref(),
                        best_quote.as_ref(),
                    )
                    .await;
                }
                Err(MarketDataSubscriptionError::Lagged(skipped)) => {
                    warn!(
                        skipped,
                        "broker market data bridge lagged; skipping stale events"
                    );
                }
                Err(MarketDataSubscriptionError::Closed) => {
                    warn!("broker market data bridge closed");
                    break;
                }
            }
        }
    });
}

async fn handle_market_event(
    state: &SharedSnapshot,
    strategy_signal: &UnboundedSender<StrategySignal>,
    event: &MarketDataEvent,
    running: bool,
    nats: Option<&nats_integration::NatsIntegration>,
    best_quote: Option<&market_data::Quote>,
) {
    let spread = event.ask - event.bid;
    let mid = (event.bid + event.ask) * 0.5;
    let mut emitted_alert = None;

    if let Some(q) = best_quote {
        debug!(
            symbol = %event.symbol,
            source = %q.source,
            priority = q.source_priority,
            "best quote from aggregator"
        );
    }

    let current_candle = {
        let mut snapshot = state.write().await;
        apply_market_event(&mut snapshot, event);
        let current_candle = snapshot
            .symbols
            .iter()
            .find(|symbol| symbol.symbol == event.symbol)
            .map(|symbol| symbol.candle.clone());

        if spread > 0.4 {
            let alert = Alert::warning(format!(
                "Wide spread detected on {}: {:.2}",
                event.symbol, spread
            ));
            snapshot.alerts.push(alert.clone());
            emitted_alert = Some(alert);
        }
        while snapshot.alerts.len() > 32 {
            snapshot.alerts.remove(0);
        }
        current_candle
    };

    // Publish market data and candle updates to NATS after snapshot mutation so
    // downstream consumers observe the same candle state the snapshot now holds.
    if let Some(nats_integration) = nats {
        nats_integration.publish_market_data(event).await;
        if let Some(ref candle) = current_candle {
            nats_integration.publish_candle(&event.symbol, candle).await;
        }
        if let Some(ref alert) = emitted_alert {
            nats_integration.publish_alert(alert).await;
        }
    }

    if !running {
        return;
    }

    let signal = StrategySignal {
        symbol: event.symbol.clone(),
        price: mid,
        timestamp: event.timestamp,
    };

    // Publish strategy signal to NATS (parallel to existing channel)
    if let Some(nats_integration) = nats {
        nats_integration.publish_strategy_signal(&signal).await;
    }

    // Existing channel send (unchanged)
    if let Err(err) = strategy_signal.send(signal) {
        warn!(%err, "failed to queue strategy signal");
    }
}

fn spawn_demo_strategy(
    mut signal_rx: UnboundedReceiver<StrategySignal>,
    decision_tx: UnboundedSender<StrategyDecisionModel>,
) {
    tokio::spawn(async move {
        let mut rng = StdRng::from_entropy();

        while let Some(signal) = signal_rx.recv().await {
            if rng.gen_bool(0.35) {
                let side = if rng.gen_bool(0.5) {
                    TradeSide::Buy
                } else {
                    TradeSide::Sell
                };

                let decision = StrategyDecisionModel {
                    symbol: signal.symbol.clone(),
                    quantity: 1,
                    side,
                };

                if let Err(err) = decision_tx.send(decision) {
                    warn!(%err, "failed to push demo strategy decision");
                    break;
                }
            }
        }
    });
}

fn spawn_strategy_fanout(
    mut decisions_rx: UnboundedReceiver<StrategyDecisionModel>,
    state: SharedSnapshot,
    risk_engine: Arc<RiskEngine>,
    strategy_toggle: watch::Receiver<bool>,
    nats: Arc<Option<nats_integration::NatsIntegration>>,
) {
    tokio::spawn(async move {
        while let Some(decision) = decisions_rx.recv().await {
            if !*strategy_toggle.borrow() {
                continue;
            }

            // Publish strategy decision to NATS (parallel to existing processing)
            if let Some(ref nats_integration) = *nats {
                nats_integration.publish_strategy_decision(&decision).await;
            }

            let StrategyDecisionModel {
                symbol,
                quantity,
                side,
            } = decision;

            let (producer_decision, request) = {
                let snapshot = state.read().await;
                let market_state = RuntimeMarketState::from_snapshot(&snapshot);
                let execution_state = RuntimeExecutionState::from_snapshot(&snapshot);
                let mark = market_state.mark_for_symbol(&symbol).unwrap_or(0.0);
                let mark = if mark <= 0.0 { 1.0 } else { mark };
                let strategy_decision = StrategyDecisionModel {
                    symbol: symbol.clone(),
                    quantity,
                    side: side.clone(),
                };
                let producer_decision = RuntimeProducerDecision::from_strategy_decision(
                    &strategy_decision,
                    mark,
                    Utc::now(),
                );
                let request = execution_state.risk_limit_for_decision(&producer_decision);
                (producer_decision, request)
            };

            let outcome = risk_engine.verify(&request).await;
            if !outcome.allowed {
                risk_engine.record_violation(RiskViolation {
                    symbol: symbol.clone(),
                    details: outcome
                        .reason
                        .clone()
                        .unwrap_or_else(|| "risk engine rejected decision".to_string()),
                });
            }

            let decision_snapshot: StrategyDecisionSnapshot = producer_decision.to_snapshot();

            {
                let mut snapshot = state.write().await;
                apply_risk_status(&mut snapshot, &outcome);
                if outcome.allowed {
                    let _ = apply_strategy_execution(&mut snapshot, decision_snapshot.clone());
                } else {
                    let alert = Alert::error(
                        outcome
                            .reason
                            .clone()
                            .unwrap_or_else(|| format!("Risk rejected {} order", symbol)),
                    );
                    snapshot.alerts.push(alert.clone());
                    while snapshot.alerts.len() > 32 {
                        snapshot.alerts.remove(0);
                    }
                    snapshot.set_strategy_status("BLOCKED");
                    if let Some(ref nats_integration) = *nats {
                        nats_integration.publish_alert(&alert).await;
                    }
                }
            }
        }
    });
}

struct PositionLimitCheck {
    max_position: i32,
    max_notional: f64,
}

impl PositionLimitCheck {
    fn new(max_position: i32, max_notional: f64) -> Self {
        Self {
            max_position,
            max_notional,
        }
    }
}

#[async_trait]
impl RiskCheck for PositionLimitCheck {
    async fn evaluate(&self, request: &RiskLimit) -> RiskDecision {
        if request.max_position > self.max_position {
            return RiskDecision {
                allowed: false,
                reason: Some(format!(
                    "position limit exceeded: {} > {}",
                    request.max_position, self.max_position
                )),
            };
        }

        if request.max_notional > self.max_notional {
            return RiskDecision {
                allowed: false,
                reason: Some(format!(
                    "notional limit exceeded: {:.2} > {:.2}",
                    request.max_notional, self.max_notional
                )),
            };
        }

        RiskDecision {
            allowed: true,
            reason: None,
        }
    }
}
