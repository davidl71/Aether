use std::{
  net::{IpAddr, Ipv4Addr, SocketAddr},
  sync::Arc,
  time::Duration,
};

use api::{
  ib_backend_proto::v1::StrategyDecision as GrpcDecision,
  Alert,
  GrpcServer,
  HistoricPosition,
  OrderSnapshot,
  PositionSnapshot,
  RestServer,
  RestState,
  SharedSnapshot,
  StrategyController,
  StrategyDecisionSnapshot,
  SystemSnapshot,
};
use anyhow::Context;
use async_trait::async_trait;
use chrono::{Duration as ChronoDuration, Utc};
use market_data::{MockMarketDataSource, MarketDataEvent, MarketDataIngestor, MarketDataSource};
use rand::{rngs::StdRng, Rng, SeedableRng};
use risk::{RiskCheck, RiskDecision, RiskEngine, RiskLimit, RiskViolation};
use serde::Deserialize;
use strategy::{Decision as StrategyDecisionModel, StrategySignal, TradeSide};
use tokio::{
  sync::{
    broadcast,
    mpsc::{UnboundedReceiver, UnboundedSender},
    watch,
    RwLock,
  },
  time::sleep,
};
use tracing::{info, warn};

#[derive(Debug, Deserialize)]
struct BackendConfig {
  rest_addr: Option<SocketAddr>,
  grpc_addr: Option<SocketAddr>,
}

impl Default for BackendConfig {
  fn default() -> Self {
    Self {
      rest_addr: Some(SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 8080)),
      grpc_addr: Some(SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 50051)),
    }
  }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  init_tracing();
  let config = load_config().context("failed to load backend config")?;

  let rest_addr = config.rest_addr.unwrap_or_else(|| SocketAddr::from(([127, 0, 0, 1], 8080)));
  let grpc_addr = config.grpc_addr.unwrap_or_else(|| SocketAddr::from(([127, 0, 0, 1], 50051)));

  let state: SharedSnapshot = Arc::new(RwLock::new(SystemSnapshot::default()));
  let (strategy_ctrl_tx, strategy_ctrl_rx) = watch::channel(false);
  let controller = StrategyController::new(strategy_ctrl_tx);
  let rest_state = RestState::new(state.clone(), controller.clone());

  {
    let mut snapshot = state.write().await;
    seed_static_data(&mut snapshot);
    snapshot.set_strategy_status("RUNNING");
    snapshot.risk.allowed = true;
    snapshot.risk.reason = None;
    snapshot.risk.updated_at = Utc::now();
  }
  let _ = controller.start();

  let (grpc_decision_tx, _) = broadcast::channel(256);
  let risk_engine = Arc::new(RiskEngine::new(vec![Box::new(PositionLimitCheck::new(8, 250_000.0))]));

  let (strategy_signal_tx, strategy_signal_rx) = tokio::sync::mpsc::unbounded_channel::<StrategySignal>();
  let (strategy_decision_tx, strategy_decision_rx) = tokio::sync::mpsc::unbounded_channel::<StrategyDecisionModel>();

  spawn_mock_strategy(strategy_signal_rx, strategy_decision_tx);
  let fanout_ctrl_rx = strategy_ctrl_rx.clone();
  spawn_strategy_fanout(
    strategy_decision_rx,
    state.clone(),
    grpc_decision_tx.clone(),
    risk_engine.clone(),
    fanout_ctrl_rx,
  );

  let market_ctrl_rx = strategy_ctrl_rx;
  let mock_source = MockMarketDataSource::new(["XSP", "SPY", "QQQ", "IWM"], Duration::from_millis(800));
  spawn_market_data_loop(mock_source, state.clone(), strategy_signal_tx, market_ctrl_rx);

  info!(%rest_addr, %grpc_addr, "backend service online");

  let rest_handle = RestServer::serve(rest_addr, rest_state).await?;
  let grpc_handle = GrpcServer::serve(grpc_addr, state.clone(), grpc_decision_tx).await?;

  tokio::signal::ctrl_c()
    .await
    .context("failed to listen for shutdown signal")?;

  rest_handle.abort();
  grpc_handle.abort();

  Ok(())
}

fn init_tracing() {
  tracing_subscriber::fmt()
    .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
    .with_target(false)
    .init();
}

fn load_config() -> anyhow::Result<BackendConfig> {
  let default_cfg = BackendConfig::default();
  let path = std::env::var("BACKEND_CONFIG").unwrap_or_else(|_| "config/default.toml".into());

  if std::path::Path::new(&path).exists() {
    let data = std::fs::read_to_string(&path)
      .with_context(|| format!("unable to read config file {path}"))?;
    let cfg: BackendConfig = toml::from_str(&data)
      .with_context(|| format!("invalid config file {path}"))?;
    Ok(BackendConfig {
      rest_addr: cfg.rest_addr.or(default_cfg.rest_addr),
      grpc_addr: cfg.grpc_addr.or(default_cfg.grpc_addr),
    })
  } else {
    Ok(default_cfg)
  }
}

fn spawn_market_data_loop<S>(
  source: S,
  state: SharedSnapshot,
  strategy_signal: UnboundedSender<StrategySignal>,
  mut strategy_toggle: watch::Receiver<bool>,
) where
  S: MarketDataSource + Send + Sync + 'static,
{
  tokio::spawn(async move {
    let ingestor = MarketDataIngestor::new(source);

    loop {
      match ingestor.poll().await {
        Ok(event) => {
          let running = *strategy_toggle.borrow();
          handle_market_event(&state, &strategy_signal, event, running).await;
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
  event: MarketDataEvent,
  running: bool,
) {
  let spread = event.ask - event.bid;
  let mid = (event.bid + event.ask) * 0.5;

  {
    let mut snapshot = state.write().await;
    snapshot.apply_market_event(&event);

    if spread > 0.4 {
      snapshot
        .alerts
        .push(Alert::warning(format!("Wide spread detected on {}: {:.2}", event.symbol, spread)));
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
  broadcaster: broadcast::Sender<GrpcDecision>,
  risk_engine: Arc<RiskEngine>,
  mut strategy_toggle: watch::Receiver<bool>,
) {
  tokio::spawn(async move {
    while let Some(decision) = decisions_rx.recv().await {
      if !*strategy_toggle.borrow() {
        continue;
      }

      let StrategyDecisionModel { symbol, quantity, side } = decision;

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

      let decision_snapshot = StrategyDecisionSnapshot::new(
        symbol.clone(),
        quantity,
        side_str,
        mark,
        Utc::now(),
      );

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

      if outcome.allowed {
        if let Err(err) = broadcaster.send(GrpcDecision {
          symbol,
          quantity,
          side: side_str.into(),
        }) {
          warn!(%err, "failed to broadcast gRPC decision");
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

  snapshot.alerts.push(Alert::info("Mock runtime initialised"));
  snapshot.alerts.push(Alert::info("Waiting for market data updates"));
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
