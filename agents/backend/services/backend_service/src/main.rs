use std::{sync::Arc, time::Duration};

use anyhow::Context;
use api::{
    mock_data::seed_snapshot, Alert, HealthAggregateState, RuntimeExecutionState,
    RuntimeMarketState, RuntimeProducerDecision, SharedSnapshot, StrategyController,
    StrategyDecisionSnapshot, SystemSnapshot,
};
use async_trait::async_trait;
use broker_engine::{BrokerConfig, BrokerEngine};
use chrono::Utc;
use ib_adapter::IbAdapter;
use market_data::{
    create_provider, FmpClient, MarketDataEvent, MarketDataIngestor, MarketDataSource,
    MarketDataSourceFactory,
};
use rand::{rngs::StdRng, Rng, SeedableRng};
use risk::{RiskCheck, RiskDecision, RiskEngine, RiskLimit, RiskViolation};
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
mod dlq_consumer;
mod health_aggregation;
mod ib_positions;
mod nats_integration;
mod rest_snapshot;
mod snapshot_publisher;
mod swiftness;
mod yield_curve_writer;

#[derive(Debug, Default, Deserialize, Clone)]
struct BackendConfig {
    #[serde(default)]
    market_data: MarketDataSettings,
    #[serde(default)]
    broker: BrokerSettings,
    #[serde(default)]
    yield_curve: YieldCurveSettings,
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

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    if std::env::args().any(|a| a == "--validate") {
        run_validate();
    }
    init_tracing();
    let config = load_config().context("failed to load backend config")?;

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

    {
        let mut snapshot = state.write().await;
        seed_snapshot(&mut snapshot, &config.market_data.symbols);
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

    spawn_mock_strategy(strategy_signal_rx, strategy_decision_tx);
    let fanout_ctrl_rx = strategy_ctrl_rx.clone();
    spawn_strategy_fanout(
        strategy_decision_rx,
        state.clone(),
        risk_engine.clone(),
        fanout_ctrl_rx,
        nats_integration.clone(),
    );

    let market_ctrl_rx = strategy_ctrl_rx;

    spawn_market_data_provider(
        &config.market_data,
        state.clone(),
        strategy_signal_tx,
        market_ctrl_rx,
        nats_integration.clone(),
    )?;

    health_aggregation::spawn_health_aggregator(health_state.clone(), nats_url.clone());
    collection_aggregation::spawn_collection_aggregator(state.clone(), nats_url.clone());
    dlq_consumer::spawn_dlq_consumer(nats_url.clone());

    // Publish full snapshots to NATS for TUI and other subscribers
    let loan_repo = api::LoanRepository::load_default().await.ok();
    let fmp_client = std::env::var("FMP_API_KEY")
        .ok()
        .filter(|k| !k.trim().is_empty())
        .and_then(|api_key| {
            FmpClient::new(api_key, std::env::var("FMP_BASE_URL").ok().as_deref())
                .ok()
                .map(Arc::new)
        });
    if let Some(ref nats) = *nats_integration {
        if let Some(client) = nats.client() {
            let broker_engine: Option<Arc<dyn BrokerEngine>> = if config.broker.paper_trading
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
                        info!(host = %broker_config.host, port = %broker_config.port, "Connected to TWS for order placement");
                        Some(Arc::new(adapter) as Arc<dyn BrokerEngine>)
                    }
                    Err(e) => {
                        warn!(error = %e, "Failed to connect to TWS; order placement disabled");
                        None
                    }
                }
            } else {
                info!("Broker order placement disabled (enable with IB_BROKER_ENABLED or set paper_trading=true)");
                None
            };
            let backend_id = std::env::var("BACKEND_ID").unwrap_or_else(|_| "ib".into());
            let interval_ms: u64 = std::env::var("SNAPSHOT_PUBLISH_INTERVAL_MS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(1000);
            let use_jetstream = matches!(
                std::env::var("NATS_USE_JETSTREAM")
                    .unwrap_or_default()
                    .trim()
                    .to_lowercase()
                    .as_str(),
                "1" | "true" | "yes"
            );
            snapshot_publisher::spawn(
                state.clone(),
                client.clone(),
                backend_id,
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
) -> anyhow::Result<()> {
    let symbols = if settings.symbols.is_empty() {
        default_market_symbols()
    } else {
        settings.symbols.clone()
    };
    let interval = Duration::from_millis(settings.poll_interval_ms.max(10));

    let aggregator = Arc::new(
        market_data::MarketDataAggregator::new()
    );

    let providers = if settings.provider == "all" {
        vec!["yahoo", "polygon"]  // Yahoo (free) + Polygon (paid) as fallback
    } else if settings.provider == "yahoo" || settings.provider == "polygon" || settings.provider == "mock" {
        vec![settings.provider.as_str()]
    } else {
        vec!["yahoo"]  // Default to Yahoo for real data
    };

    for provider_name in providers {
        let source = create_provider(provider_name, &symbols, interval).with_context(|| {
            format!("failed to create market data provider: {}", provider_name)
        })?;

        let agg = aggregator.clone();
        let state_clone = state.clone();
        let signal_clone = strategy_signal.clone();
        let toggle_clone = strategy_toggle.clone();
        let nats_clone = nats.clone();

        info!(provider = %provider_name, symbol_count = symbols.len(), "spawning market data provider");
        spawn_market_data_loop(source, state_clone, signal_clone, toggle_clone, nats_clone, agg);
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

    if let Some(q) = best_quote {
        debug!(
            symbol = %event.symbol,
            source = %q.source,
            priority = q.source_priority,
            "best quote from aggregator"
        );
    }

    // Publish market data to NATS (parallel to existing state update)
    if let Some(nats_integration) = nats {
        nats_integration.publish_market_data(event).await;
    }

    {
        let mut snapshot = state.write().await;
        snapshot.apply_market_event(event);

        if spread > 0.4 {
            snapshot.alerts.push(Alert::warning(format!(
                "Wide spread detected on {}: {:.2}",
                event.symbol, spread
            )));
        }
        while snapshot.alerts.len() > 32 {
            snapshot.alerts.remove(0);
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

fn spawn_mock_strategy(
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
                    warn!(%err, "failed to push mock strategy decision");
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
                snapshot.update_risk_status(&outcome);
                if outcome.allowed {
                    snapshot.apply_strategy_execution(decision_snapshot.clone());
                } else {
                    snapshot.alerts.push(Alert::error(
                        outcome
                            .reason
                            .clone()
                            .unwrap_or_else(|| format!("Risk rejected {} order", symbol)),
                    ));
                    while snapshot.alerts.len() > 32 {
                        snapshot.alerts.remove(0);
                    }
                    snapshot.set_strategy_status("BLOCKED");
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
