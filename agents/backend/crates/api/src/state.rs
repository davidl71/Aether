use std::sync::Arc;

use chrono::{DateTime, Utc};
use market_data::MarketDataEvent;
use risk::RiskDecision;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::{debug, warn};

use crate::runtime_state::{
    RuntimeExecutionState, RuntimeExecutionUpdate, RuntimeMarketState, RuntimeRiskState,
};

pub type SharedSnapshot = Arc<RwLock<SystemSnapshot>>;

#[derive(Clone, Serialize, Deserialize)]
pub struct SystemSnapshot {
    pub generated_at: DateTime<Utc>,
    pub started_at: DateTime<Utc>,
    pub mode: String,
    pub strategy: String,
    pub account_id: String,
    pub metrics: Metrics,
    pub symbols: Vec<SymbolSnapshot>,
    pub positions: Vec<PositionSnapshot>,
    pub historic: Vec<HistoricPosition>,
    pub orders: Vec<OrderSnapshot>,
    pub decisions: Vec<StrategyDecisionSnapshot>,
    pub alerts: Vec<Alert>,
    pub risk: RiskStatus,
    /// Optional ledger engine for transaction recording (not serialized)
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
            metrics: Metrics::default(),
            symbols: Vec::new(),
            positions: Vec::new(),
            historic: Vec::new(),
            orders: Vec::new(),
            decisions: Vec::new(),
            alerts: vec![Alert::info("Backend initialising")],
            risk: RiskStatus::default(),
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
        let update = runtime_state.apply_strategy_decision(&decision, order_id.clone());
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

    /// Set ledger engine for transaction recording
    pub fn set_ledger(&mut self, ledger: Arc<ledger::LedgerEngine>) {
        self.ledger = Some(ledger);
        debug!("Ledger engine attached to SystemSnapshot");
    }

    /// Record box spread transaction (async, non-blocking)
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

    /// Record cash flow transaction (async, non-blocking)
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
            debug!("Ledger not configured, skipping cash flow transaction recording");
        }
    }

    pub fn update_risk_status(&mut self, outcome: &RiskDecision) {
        self.touch();
        let mut runtime_risk = RuntimeRiskState::from_snapshot(self);
        runtime_risk.apply_risk_decision(outcome);
        runtime_risk.project_into_snapshot(self);
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Metrics {
    pub net_liq: f64,
    pub buying_power: f64,
    pub excess_liquidity: f64,
    pub margin_requirement: f64,
    pub commissions: f64,
    pub portal_ok: bool,
    pub tws_ok: bool,
    pub questdb_ok: bool,
    pub nats_ok: bool,
}

impl Default for Metrics {
    fn default() -> Self {
        Self {
            net_liq: 100_000.0,
            buying_power: 80_000.0,
            excess_liquidity: 25_000.0,
            margin_requirement: 15_000.0,
            commissions: 0.0,
            portal_ok: false,
            tws_ok: false,
            questdb_ok: false,
            nats_ok: false,
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
    pub candle: CandleSnapshot,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CandleSnapshot {
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: u64,
    pub entry: f64,
    pub updated: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PositionSnapshot {
    pub id: String,
    pub symbol: String,
    pub quantity: i32,
    pub cost_basis: f64,
    pub mark: f64,
    pub unrealized_pnl: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HistoricPosition {
    pub id: String,
    pub symbol: String,
    pub quantity: i32,
    pub realized_pnl: f64,
    pub closed_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OrderSnapshot {
    pub id: String,
    pub symbol: String,
    pub side: String,
    pub quantity: i32,
    pub status: String,
    pub submitted_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StrategyDecisionSnapshot {
    pub symbol: String,
    pub quantity: i32,
    pub side: String,
    pub mark: f64,
    pub created_at: DateTime<Utc>,
}

impl StrategyDecisionSnapshot {
    pub fn new(
        symbol: String,
        quantity: i32,
        side: impl Into<String>,
        mark: f64,
        created_at: DateTime<Utc>,
    ) -> Self {
        Self {
            symbol,
            quantity,
            side: side.into(),
            mark,
            created_at,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RiskStatus {
    pub allowed: bool,
    pub reason: Option<String>,
    pub updated_at: DateTime<Utc>,
}

impl Default for RiskStatus {
    fn default() -> Self {
        Self {
            allowed: true,
            reason: None,
            updated_at: Utc::now(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Alert {
    pub level: AlertLevel,
    pub message: String,
    pub timestamp: DateTime<Utc>,
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

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum AlertLevel {
    Info,
    Warning,
    Error,
}

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
}
