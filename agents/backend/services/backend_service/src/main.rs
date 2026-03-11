use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    sync::Arc,
    time::Duration,
};

use anyhow::Context;
use api::{
    Alert, HealthAggregateState, HistoricPosition, LoanRepository, OrderSnapshot,
    PositionSnapshot, RestServer, RestState, SharedSnapshot, StrategyController,
    StrategyDecisionSnapshot, SystemSnapshot,
};
use async_trait::async_trait;
use chrono::{Duration as ChronoDuration, Utc};
use market_data::{
    MarketDataEvent, MarketDataIngestor, MarketDataSource, MockMarketDataSource,
    PolygonMarketDataSource,
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
use tracing::{info, warn};

mod nats_integration;
mod health_aggregation;
mod swiftness;

#[derive(Debug, Deserialize, Clone)]
struct BackendConfig {
    #[serde(default = "default_rest_addr")]
    rest_addr: SocketAddr,
    #[serde(default)]
    market_data: MarketDataSettings,
}

impl Default for BackendConfig {
    fn default() -> Self {
        Self {
            rest_addr: default_rest_addr(),
            market_data: MarketDataSettings::default(),
        }
    }
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

fn default_rest_addr() -> SocketAddr {
    SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 8080)
}

fn default_market_provider() -> String {
    "mock".into()
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
    init_tracing();
    let config = load_config().context("failed to load backend config")?;

    let rest_addr = config.rest_addr;

    let state: SharedSnapshot = Arc::new(RwLock::new(SystemSnapshot::default()));
    let (strategy_ctrl_tx, strategy_ctrl_rx) = watch::channel(false);
    let controller = StrategyController::new(strategy_ctrl_tx);
    let health_state = HealthAggregateState::new_shared();

    // Initialize NATS integration (graceful degradation if unavailable)
    let nats_integration =
        Arc::new(nats_integration::NatsIntegration::new(std::env::var("NATS_URL").ok()).await);

    if nats_integration
        .as_ref()
        .as_ref()
        .is_some_and(|n| n.is_active())
    {
        info!("NATS integration active");
    } else {
        warn!("NATS integration unavailable, continuing without NATS");
    }

    let loan_repository = LoanRepository::load_default()
        .await
        .context("failed to initialize loan repository")?;
    let rest_state = RestState::new(
        state.clone(),
        controller.clone(),
        loan_repository,
        health_state.clone(),
    );

    {
        let mut snapshot = state.write().await;
        seed_static_data(&mut snapshot);
        snapshot.set_strategy_status("RUNNING");
        snapshot.risk.allowed = true;
        snapshot.risk.reason = None;
        snapshot.risk.updated_at = Utc::now();
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
        nats_integration,
    )?;

    health_aggregation::spawn_health_aggregator(health_state, std::env::var("NATS_URL").ok());

    // Swiftness is temporarily disabled by default; enable explicitly for manual use.
    if swiftness::swiftness_enabled() {
        swiftness::spawn_swiftness_position_fetcher(state.clone());
    } else {
        info!("Swiftness integration disabled (set ENABLE_SWIFTNESS=1 to enable)");
    }

    info!(%rest_addr, "backend service online");

    let rest_handle = RestServer::serve(rest_addr, rest_state).await?;

    tokio::signal::ctrl_c()
        .await
        .context("failed to listen for shutdown signal")?;

    rest_handle.abort();

    Ok(())
}

fn init_tracing() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_target(false)
        .init();
}

fn load_config() -> anyhow::Result<BackendConfig> {
    let path = std::env::var("BACKEND_CONFIG").unwrap_or_else(|_| "config/default.toml".into());

    if std::path::Path::new(&path).exists() {
        let data = std::fs::read_to_string(&path)
            .with_context(|| format!("unable to read config file {path}"))?;
        let cfg: BackendConfig =
            toml::from_str(&data).with_context(|| format!("invalid config file {path}"))?;
        Ok(cfg)
    } else {
        Ok(BackendConfig::default())
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

    match settings.provider.as_str() {
        "polygon" => {
            let polygon_cfg = settings
                .polygon
                .as_ref()
                .context("polygon provider selected but polygon settings missing")?;
            let api_key = resolve_polygon_api_key(polygon_cfg)?;
            let base_url = polygon_cfg.base_url.as_deref();
            let source = PolygonMarketDataSource::new(symbols, interval, api_key, base_url)
                .context("failed to create polygon market data source")?;
            spawn_market_data_loop(source, state, strategy_signal, strategy_toggle, nats);
        }
        provider => {
            if provider != "mock" {
                warn!(%provider, "unknown market data provider requested, falling back to mock");
            }
            let source = MockMarketDataSource::new(symbols, interval);
            spawn_market_data_loop(source, state, strategy_signal, strategy_toggle, nats);
        }
    }

    Ok(())
}

fn resolve_polygon_api_key(settings: &PolygonSettings) -> anyhow::Result<String> {
    if let Some(key) = settings.api_key.clone() {
        return Ok(key);
    }

    if let Some(env) = &settings.api_key_env {
        if let Ok(val) = std::env::var(env) {
            anyhow::ensure!(
                !val.trim().is_empty(),
                "environment variable {env} is set but empty"
            );
            return Ok(val);
        }
        anyhow::bail!("environment variable {env} not found for polygon API key");
    }

    anyhow::bail!(
        "polygon API key not configured (set market_data.polygon.api_key or api_key_env)"
    );
}

fn spawn_market_data_loop<S>(
    source: S,
    state: SharedSnapshot,
    strategy_signal: UnboundedSender<StrategySignal>,
    strategy_toggle: watch::Receiver<bool>,
    nats: Arc<Option<nats_integration::NatsIntegration>>,
) where
    S: MarketDataSource + Send + Sync + 'static,
{
    tokio::spawn(async move {
        let ingestor = MarketDataIngestor::new(source);

        loop {
            match ingestor.poll().await {
                Ok(event) => {
                    let running = *strategy_toggle.borrow();
                    handle_market_event(
                        &state,
                        &strategy_signal,
                        &event,
                        running,
                        nats.as_ref().as_ref(),
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
) {
    let spread = event.ask - event.bid;
    let mid = (event.bid + event.ask) * 0.5;

    // Publish market data to NATS (parallel to existing state update)
    if let Some(nats_integration) = nats {
        nats_integration
            .publish_market_data(&event.symbol, event.bid, event.ask)
            .await;
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

            let (mark_price, current_position) = {
                let snapshot = state.read().await;
                let mark = snapshot
                    .symbols
                    .iter()
                    .find(|s| s.symbol == symbol)
                    .map(|s| s.last)
                    .unwrap_or(0.0);
                let qty = snapshot
                    .positions
                    .iter()
                    .find(|p| p.symbol == symbol)
                    .map(|p| p.quantity)
                    .unwrap_or(0);
                (mark, qty)
            };

            let mark = if mark_price <= 0.0 { 1.0 } else { mark_price };
            let target_qty = current_position + quantity;
            let request = RiskLimit {
                symbol: symbol.clone(),
                max_position: target_qty.abs(),
                max_notional: mark * target_qty.abs() as f64,
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

            let side_str = match side {
                TradeSide::Buy => "BUY",
                TradeSide::Sell => "SELL",
            };

            let decision_snapshot =
                StrategyDecisionSnapshot::new(symbol.clone(), quantity, side_str, mark, Utc::now());

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

fn seed_static_data(snapshot: &mut SystemSnapshot) {
    if snapshot.positions.is_empty() {
        snapshot.positions.push(PositionSnapshot {
            id: "POS-1".into(),
            symbol: "XSP".into(),
            quantity: 2,
            cost_basis: 98.75,
            mark: 101.10,
            unrealized_pnl: 4.7,
        });
    }

    if snapshot.orders.is_empty() {
        snapshot.orders.push(OrderSnapshot {
            id: "ORD-1".into(),
            symbol: "XSP".into(),
            side: "BUY".into(),
            quantity: 2,
            status: "FILLED".into(),
            submitted_at: Utc::now() - ChronoDuration::minutes(30),
        });
    }

    if snapshot.historic.is_empty() {
        snapshot.historic.push(HistoricPosition {
            id: "POS-0".into(),
            symbol: "SPY".into(),
            quantity: 2,
            realized_pnl: 6.2,
            closed_at: Utc::now() - ChronoDuration::hours(5),
        });
    }

    snapshot
        .alerts
        .push(Alert::info("Mock runtime initialised"));
    snapshot
        .alerts
        .push(Alert::info("Waiting for market data updates"));
    while snapshot.alerts.len() > 32 {
        snapshot.alerts.remove(0);
    }
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
