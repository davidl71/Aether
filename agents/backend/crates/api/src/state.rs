use std::sync::Arc;

use chrono::Utc;
use market_data::MarketDataEvent;
use risk::RiskDecision;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::{debug, warn};

use common::snapshot as cmn;

use crate::runtime_state::{
    RuntimeExecutionState, RuntimeExecutionUpdate, RuntimeMarketState, RuntimeRiskState,
};

pub type SharedSnapshot = Arc<RwLock<SystemSnapshot>>;

// ---------------------------------------------------------------------------
// Api-only types (not shared)
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum AlertLevel {
    Info,
    Warning,
    Error,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Alert {
    pub level: AlertLevel,
    pub message: String,
    pub timestamp: chrono::DateTime<Utc>,
}

impl Alert {
    pub fn info(message: impl Into<String>) -> Self {
        Self {
            level: AlertLevel::Info,
            message: message.into(),
            timestamp: Utc::now(),
        }
    }

    pub fn warning(message: impl Into<String>) -> Self {
        Self {
            level: AlertLevel::Warning,
            message: message.into(),
            timestamp: Utc::now(),
        }
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self {
            level: AlertLevel::Error,
            message: message.into(),
            timestamp: Utc::now(),
        }
    }
}

impl From<cmn::Alert> for Alert {
    fn from(a: cmn::Alert) -> Self {
        let level = match a.level.to_uppercase().as_str() {
            "WARNING" => AlertLevel::Warning,
            "ERROR" => AlertLevel::Error,
            _ => AlertLevel::Info,
        };
        Alert {
            level,
            message: a.message,
            timestamp: a.timestamp,
        }
    }
}

impl From<Alert> for cmn::Alert {
    fn from(a: Alert) -> Self {
        let level = match a.level {
            AlertLevel::Info => "INFO",
            AlertLevel::Warning => "WARNING",
            AlertLevel::Error => "ERROR",
        };
        cmn::Alert {
            level: level.to_string(),
            message: a.message,
            timestamp: a.timestamp,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SymbolSnapshot {
    pub symbol: String,
    pub last: f64,
    pub bid: f64,
    pub ask: f64,
    pub spread: f64,
    pub roi: f64,
    pub maker_count: u32,
    pub taker_count: u32,
    pub volume: u64,
    pub candle: cmn::CandleSnapshot,
}

// ---------------------------------------------------------------------------
// System snapshot (api-only aggregate)
// ---------------------------------------------------------------------------

#[derive(Clone, Serialize, Deserialize)]
pub struct SystemSnapshot {
    pub generated_at: chrono::DateTime<Utc>,
    pub started_at: chrono::DateTime<Utc>,
    pub mode: String,
    pub strategy: String,
    pub account_id: String,
    pub metrics: cmn::Metrics,
    pub symbols: Vec<SymbolSnapshot>,
    pub positions: Vec<cmn::PositionSnapshot>,
    pub historic: Vec<cmn::HistoricPosition>,
    pub orders: Vec<cmn::OrderSnapshot>,
    pub decisions: Vec<cmn::StrategyDecisionSnapshot>,
    pub alerts: Vec<Alert>,
    pub risk: cmn::RiskStatus,
    #[serde(skip_serializing, skip_deserializing)]
    pub ledger: Option<Arc<ledger::LedgerEngine>>,
}

impl std::fmt::Debug for SystemSnapshot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SystemSnapshot")
            .field("generated_at", &self.generated_at)
            .field("started_at", &self.started_at)
            .field("mode", &self.mode)
            .field("strategy", &self.strategy)
            .field("account_id", &self.account_id)
            .field("metrics", &self.metrics)
            .field("symbols", &self.symbols)
            .field("positions", &self.positions)
            .field("historic", &self.historic)
            .field("orders", &self.orders)
            .field("decisions", &self.decisions)
            .field("alerts", &self.alerts)
            .field("risk", &self.risk)
            .field(
                "ledger",
                &self.ledger.as_ref().map(|_| "Some(LedgerEngine)"),
            )
            .finish()
    }
}

impl Default for SystemSnapshot {
    fn default() -> Self {
        Self {
            generated_at: Utc::now(),
            started_at: Utc::now(),
            mode: "DRY-RUN".into(),
            strategy: "IDLE".into(),
            account_id: "DU123456".into(),
            metrics: cmn::Metrics::default(),
            symbols: Vec::new(),
            positions: Vec::new(),
            historic: Vec::new(),
            orders: Vec::new(),
            decisions: Vec::new(),
            alerts: vec![Alert::info("Backend initialising")],
            risk: cmn::RiskStatus::default(),
            ledger: None,
        }
    }
}

impl SystemSnapshot {
    pub fn touch(&mut self) {
        self.generated_at = Utc::now();
    }

    pub fn set_strategy_status(&mut self, status: impl Into<String>) {
        self.strategy = status.into();
    }

    pub fn apply_market_event(&mut self, event: &MarketDataEvent) {
        self.touch();
        let mut runtime_market = RuntimeMarketState::from_snapshot(self);
        runtime_market.apply_market_event(event);
        runtime_market.project_into_snapshot(self);
    }

    pub fn apply_strategy_execution(&mut self, decision: StrategyDecisionSnapshot) {
        self.touch();
        self.strategy = "RUNNING".into();
        let order_id = format!("ORD-{}", Utc::now().timestamp_millis());
        let mut runtime_state = RuntimeExecutionState::from_snapshot(self);
        let update =
            runtime_state.apply_strategy_decision(&decision, order_id.clone(), &self.account_id);
        runtime_state.project_into_snapshot(self);

        match update {
            RuntimeExecutionUpdate::ClosedPosition {
                symbol,
                quantity,
                cost_basis,
                mark,
                order_id,
            } => {
                if let Some(ref ledger) = self.ledger {
                    let ledger_clone = ledger.clone();
                    tokio::spawn(async move {
                        if let Err(err) = ledger::record_position_close(
                            ledger_clone,
                            &symbol,
                            quantity,
                            cost_basis,
                            mark,
                            ledger::Currency::USD,
                            Some(&order_id),
                        )
                        .await
                        {
                            warn!(error = %err, symbol = %symbol, "Failed to record position close in ledger (non-blocking)");
                        }
                    });
                }
            }
            RuntimeExecutionUpdate::ChangedPosition {
                symbol,
                quantity,
                mark,
                order_id,
            } => {
                if let Some(ref ledger) = self.ledger {
                    let ledger_clone = ledger.clone();
                    tokio::spawn(async move {
                        ledger::record_position_change_safe(
                            ledger_clone,
                            &symbol,
                            quantity,
                            mark,
                            ledger::Currency::USD,
                            Some(&order_id),
                        )
                        .await;
                    });
                }
            }
        }
    }

    pub fn set_ledger(&mut self, ledger: Arc<ledger::LedgerEngine>) {
        self.ledger = Some(ledger);
        debug!("Ledger engine attached to SystemSnapshot");
    }

    pub fn record_box_spread_async(
        &self,
        symbol: &str,
        strike1: i32,
        strike2: i32,
        expiry: &str,
        net_debit: f64,
        trade_id: Option<&str>,
    ) {
        if let Some(ref ledger) = self.ledger {
            let symbol_for_log = symbol.to_string();
            let expiry_for_log = expiry.to_string();
            debug!(
              symbol = %symbol_for_log,
              strike1,
              strike2,
              expiry = %expiry_for_log,
              net_debit,
              "Box spread transaction queued for ledger recording"
            );
            let ledger_clone = ledger.clone();
            let symbol = symbol.to_string();
            let expiry = expiry.to_string();
            let trade_id = trade_id.map(|s| s.to_string());
            tokio::spawn(async move {
                ledger::record_box_spread_safe(
                    ledger_clone,
                    &symbol,
                    strike1,
                    strike2,
                    &expiry,
                    net_debit,
                    trade_id.as_deref(),
                    ledger::Currency::USD,
                )
                .await;
            });
        } else {
            debug!("Ledger not configured, skipping box spread transaction recording");
        }
    }

    pub fn record_cash_flow_async(
        &self,
        amount: f64,
        currency: ledger::Currency,
        description: &str,
        is_deposit: bool,
    ) {
        if let Some(ref ledger) = self.ledger {
            let ledger_clone = ledger.clone();
            let description_clone = description.to_string();
            debug!(
              amount,
              currency = ?currency,
              description = %description_clone,
              is_deposit,
              "Cash flow transaction queued for ledger recording"
            );
            let description = description.to_string();
            tokio::spawn(async move {
                if let Err(err) = ledger::record_cash_flow(
                    ledger_clone,
                    amount,
                    currency,
                    &description,
                    is_deposit,
                )
                .await
                {
                    warn!(error = %err, description = %description, "Failed to record cash flow in ledger (non-blocking)");
                }
            });
        } else {
            debug!("Ledger not configured, skipping cash flow recording");
        }
    }

    pub fn update_risk_status(&mut self, outcome: &RiskDecision) {
        self.touch();
        let mut runtime_risk = RuntimeRiskState::from_snapshot(self);
        runtime_risk.apply_risk_decision(outcome);
        runtime_risk.project_into_snapshot(self);
    }
}

// ---------------------------------------------------------------------------
// Re-exports from common
// ---------------------------------------------------------------------------

pub use common::snapshot::CandleSnapshot;
pub use common::snapshot::HistoricPosition;
pub use common::snapshot::Metrics;
pub use common::snapshot::OrderSnapshot;
pub use common::snapshot::PositionSnapshot;
pub use common::snapshot::RiskStatus;
pub use common::snapshot::StrategyDecisionSnapshot;
pub use common::snapshot::Alert as CommonAlert;

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn apply_market_event_delegates_to_runtime_market_state() {
        let mut snapshot = SystemSnapshot::default();
        let event = MarketDataEvent {
            symbol: "SPY".into(),
            bid: 500.0,
            ask: 502.0,
            timestamp: Utc::now(),
            ..Default::default()
        };

        snapshot.apply_market_event(&event);

        assert_eq!(snapshot.symbols.len(), 1);
        assert_eq!(snapshot.symbols[0].symbol, "SPY");
        assert_eq!(snapshot.symbols[0].last, 501.0);
        assert!(snapshot.metrics.portal_ok);
        assert!(snapshot.metrics.tws_ok);
        assert!(snapshot.metrics.questdb_ok);
    }

    #[test]
    fn update_risk_status_delegates_to_runtime_risk_state() {
        let mut snapshot = SystemSnapshot::default();

        snapshot.update_risk_status(&RiskDecision {
            allowed: false,
            reason: Some("limit".into()),
        });

        assert!(!snapshot.risk.allowed);
        assert_eq!(snapshot.risk.reason.as_deref(), Some("limit"));
    }

    #[test]
    fn apply_strategy_execution_creates_runtime_position_and_order() {
        let mut snapshot = SystemSnapshot::default();
        let created_at = Utc::now();

        snapshot.apply_strategy_execution(StrategyDecisionSnapshot::new(
            "AAPL".into(),
            10,
            "BUY",
            150.0,
            created_at,
        ));

        assert_eq!(snapshot.positions.len(), 1);
        assert_eq!(snapshot.orders.len(), 1);
        assert_eq!(snapshot.decisions.len(), 1);
        assert_eq!(snapshot.historic.len(), 0);
        assert_eq!(snapshot.positions[0].symbol, "AAPL");
        assert_eq!(snapshot.positions[0].quantity, 10);
        assert_eq!(snapshot.positions[0].cost_basis, 150.0);
    }

    #[test]
    fn apply_strategy_execution_closing_position_moves_to_history() {
        let mut snapshot = SystemSnapshot::default();
        let opened_at = Utc::now();
        let closed_at = opened_at + chrono::TimeDelta::seconds(1);

        snapshot.apply_strategy_execution(StrategyDecisionSnapshot::new(
            "AAPL".into(),
            10,
            "BUY",
            150.0,
            opened_at,
        ));
        snapshot.apply_strategy_execution(StrategyDecisionSnapshot::new(
            "AAPL".into(),
            -10,
            "SELL",
            155.0,
            closed_at,
        ));

        assert!(snapshot.positions.is_empty());
        assert_eq!(snapshot.orders.len(), 2);
        assert_eq!(snapshot.decisions.len(), 2);
        assert_eq!(snapshot.historic.len(), 1);
        assert_eq!(snapshot.historic[0].symbol, "AAPL");
        assert_eq!(snapshot.historic[0].quantity, 10);
        assert_eq!(snapshot.historic[0].realized_pnl, 50.0);
        assert_eq!(snapshot.historic[0].closed_at, closed_at);
    }

    #[test]
    fn alert_from_common_round_trip() {
        let alert = Alert::info("test");
        let common: cmn::Alert = alert.clone().into();
        let back: Alert = common.into();
        assert!(matches!(back.level, AlertLevel::Info));
        assert_eq!(back.message, "test");
    }
}
