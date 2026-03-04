use std::sync::Arc;

use api::{Alert, SharedSnapshot, StrategyDecisionSnapshot};
use async_trait::async_trait;
use chrono::Utc;
use rand::{rngs::StdRng, Rng, SeedableRng};
use risk::{RiskCheck, RiskDecision, RiskEngine, RiskLimit, RiskViolation};
use strategy::{Decision as StrategyDecisionModel, StrategySignal};
use strategy::model::TradeSide;
use tokio::sync::{mpsc::UnboundedReceiver, watch};
use tracing::warn;

use crate::nats_integration;

pub fn create_risk_engine() -> Arc<RiskEngine> {
  Arc::new(RiskEngine::new(vec![
    Box::new(PositionLimitCheck::new(8, 250_000.0)),
  ]))
}

pub fn spawn_mock_strategy(
  mut signal_rx: UnboundedReceiver<StrategySignal>,
  decision_tx: tokio::sync::mpsc::UnboundedSender<StrategyDecisionModel>,
) {
  tokio::spawn(async move {
    let mut rng = StdRng::from_entropy();

    while let Some(signal) = signal_rx.recv().await {
      if rng.gen_bool(0.35) {
        let side = if rng.gen_bool(0.5) { TradeSide::Buy } else { TradeSide::Sell };

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

pub fn spawn_strategy_fanout(
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

      if let Some(ref n) = *nats {
        n.publish_strategy_decision(&decision).await;
      }

      let StrategyDecisionModel { symbol, quantity, side } = decision;

      let (mark_price, current_position) = {
        let snapshot = state.read().await;
        let mark = snapshot.symbols.iter().find(|s| s.symbol == symbol).map(|s| s.last).unwrap_or(0.0);
        let qty = snapshot.positions.iter().find(|p| p.symbol == symbol).map(|p| p.quantity).unwrap_or(0);
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
          details: outcome.reason.clone().unwrap_or_else(|| "risk engine rejected decision".into()),
        });
      }

      let side_str = match side {
        TradeSide::Buy => "BUY",
        TradeSide::Sell => "SELL",
      };

      let decision_snapshot = StrategyDecisionSnapshot::new(symbol.clone(), quantity, side_str, mark, Utc::now());

      {
        let mut snapshot = state.write().await;
        snapshot.update_risk_status(&outcome);
        if outcome.allowed {
          snapshot.apply_strategy_execution(decision_snapshot.clone());
        } else {
          snapshot.alerts.push(Alert::error(
            outcome.reason.clone().unwrap_or_else(|| format!("Risk rejected {} order", symbol)),
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
    Self { max_position, max_notional }
  }
}

#[async_trait]
impl RiskCheck for PositionLimitCheck {
  async fn evaluate(&self, request: &RiskLimit) -> RiskDecision {
    if request.max_position > self.max_position {
      return RiskDecision {
        allowed: false,
        reason: Some(format!("position limit exceeded: {} > {}", request.max_position, self.max_position)),
      };
    }

    if request.max_notional > self.max_notional {
      return RiskDecision {
        allowed: false,
        reason: Some(format!("notional limit exceeded: {:.2} > {:.2}", request.max_notional, self.max_notional)),
      };
    }

    RiskDecision { allowed: true, reason: None }
  }
}
