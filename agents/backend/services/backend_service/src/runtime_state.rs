use std::collections::HashMap;

use chrono::{DateTime, Utc};
use market_data::MarketDataEvent;
use risk::{RiskDecision, RiskLimit};
use serde::{Deserialize, Serialize};
use strategy::model::{Decision as StrategyDecisionModel, TradeSide};

use api::{
    CandleSnapshot, HistoricPosition, Metrics, OrderSnapshot, PositionSnapshot,
    StrategyDecisionSnapshot, SymbolSnapshot, SystemSnapshot,
};

#[derive(Clone, Debug, PartialEq)]
pub struct RuntimePositionState {
    pub id: String,
    pub symbol: String,
    pub quantity: i32,
    pub cost_basis: f64,
    pub mark: f64,
    pub unrealized_pnl: f64,
    pub account_id: Option<String>,
}

impl From<&PositionSnapshot> for RuntimePositionState {
    fn from(value: &PositionSnapshot) -> Self {
        Self {
            id: value.id.clone(),
            symbol: value.symbol.clone(),
            quantity: value.quantity,
            cost_basis: value.cost_basis,
            mark: value.mark,
            unrealized_pnl: value.unrealized_pnl,
            account_id: value.account_id.clone(),
        }
    }
}

impl From<RuntimePositionState> for PositionSnapshot {
    fn from(value: RuntimePositionState) -> Self {
        Self {
            id: value.id,
            symbol: value.symbol,
            quantity: value.quantity,
            cost_basis: value.cost_basis,
            mark: value.mark,
            unrealized_pnl: value.unrealized_pnl,
            account_id: value.account_id,
            source: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct RuntimeHistoricPositionState {
    pub id: String,
    pub symbol: String,
    pub quantity: i32,
    pub realized_pnl: f64,
    pub closed_at: DateTime<Utc>,
}

impl From<&HistoricPosition> for RuntimeHistoricPositionState {
    fn from(value: &HistoricPosition) -> Self {
        Self {
            id: value.id.clone(),
            symbol: value.symbol.clone(),
            quantity: value.quantity,
            realized_pnl: value.realized_pnl,
            closed_at: value.closed_at,
        }
    }
}

impl From<RuntimeHistoricPositionState> for HistoricPosition {
    fn from(value: RuntimeHistoricPositionState) -> Self {
        Self {
            id: value.id,
            symbol: value.symbol,
            quantity: value.quantity,
            realized_pnl: value.realized_pnl,
            closed_at: value.closed_at,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct RuntimeOrderState {
    pub id: String,
    pub symbol: String,
    pub side: String,
    pub quantity: i32,
    pub status: String,
    pub submitted_at: DateTime<Utc>,
}

impl From<&OrderSnapshot> for RuntimeOrderState {
    fn from(value: &OrderSnapshot) -> Self {
        Self {
            id: value.id.clone(),
            symbol: value.symbol.clone(),
            side: value.side.clone(),
            quantity: value.quantity,
            status: value.status.clone(),
            submitted_at: value.submitted_at,
        }
    }
}

impl From<RuntimeOrderState> for OrderSnapshot {
    fn from(value: RuntimeOrderState) -> Self {
        Self {
            id: value.id,
            symbol: value.symbol,
            side: value.side,
            quantity: value.quantity,
            status: value.status,
            submitted_at: value.submitted_at,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct RuntimeDecisionState {
    pub symbol: String,
    pub quantity: i32,
    pub side: String,
    pub mark: f64,
    pub created_at: DateTime<Utc>,
    pub metadata: Option<ProducerMetadata>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ProducerType {
    Strategy,
    Market,
    Risk,
    Native,
    Broker,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum InstrumentType {
    Stock,
    Option,
    Future,
    Bond,
    BoxSpread,
    MultiLeg,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ProducerMetadata {
    pub producer_type: ProducerType,
    pub producer_id: Option<String>,
    pub broker_source: Option<String>,
    pub account_id: Option<String>,
    pub correlation_id: Option<String>,
    pub sequence: Option<u64>,
    pub instrument_type: Option<InstrumentType>,
}

impl ProducerMetadata {
    pub fn new(producer_type: ProducerType) -> Self {
        Self {
            producer_type,
            producer_id: None,
            broker_source: None,
            account_id: None,
            correlation_id: None,
            sequence: None,
            instrument_type: None,
        }
    }

    pub fn with_producer_id(mut self, id: String) -> Self {
        self.producer_id = Some(id);
        self
    }

    pub fn with_broker_source(mut self, broker: String) -> Self {
        self.broker_source = Some(broker);
        self
    }

    pub fn with_account_id(mut self, account: String) -> Self {
        self.account_id = Some(account);
        self
    }

    pub fn with_correlation_id(mut self, correlation: String) -> Self {
        self.correlation_id = Some(correlation);
        self
    }

    pub fn with_sequence(mut self, seq: u64) -> Self {
        self.sequence = Some(seq);
        self
    }

    pub fn with_instrument_type(mut self, instrument: InstrumentType) -> Self {
        self.instrument_type = Some(instrument);
        self
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct RuntimeProducerDecision {
    pub symbol: String,
    pub quantity: i32,
    pub side: String,
    pub mark: f64,
    pub created_at: DateTime<Utc>,
    pub metadata: Option<ProducerMetadata>,
}

impl RuntimeProducerDecision {
    pub fn from_strategy_decision(
        decision: &StrategyDecisionModel,
        mark: f64,
        created_at: DateTime<Utc>,
    ) -> Self {
        let side = match decision.side {
            TradeSide::Buy => "BUY",
            TradeSide::Sell => "SELL",
        };

        Self {
            symbol: decision.symbol.clone(),
            quantity: decision.quantity,
            side: side.into(),
            mark,
            created_at,
            metadata: Some(ProducerMetadata::new(ProducerType::Strategy)),
        }
    }

    pub fn new_with_metadata(
        symbol: String,
        quantity: i32,
        side: String,
        mark: f64,
        created_at: DateTime<Utc>,
        metadata: ProducerMetadata,
    ) -> Self {
        Self {
            symbol,
            quantity,
            side,
            mark,
            created_at,
            metadata: Some(metadata),
        }
    }

    pub fn to_snapshot(&self) -> StrategyDecisionSnapshot {
        StrategyDecisionSnapshot::new(
            self.symbol.clone(),
            self.quantity,
            self.side.clone(),
            self.mark,
            self.created_at,
        )
    }

    pub fn is_from_producer_type(&self, producer_type: &ProducerType) -> bool {
        self.metadata
            .as_ref()
            .map(|m| &m.producer_type == producer_type)
            .unwrap_or(false)
    }

    pub fn correlation_id(&self) -> Option<&str> {
        self.metadata
            .as_ref()
            .and_then(|m| m.correlation_id.as_deref())
    }

    pub fn account_id(&self) -> Option<&str> {
        self.metadata.as_ref().and_then(|m| m.account_id.as_deref())
    }

    pub fn broker_source(&self) -> Option<&str> {
        self.metadata
            .as_ref()
            .and_then(|m| m.broker_source.as_deref())
    }
}

impl From<&StrategyDecisionSnapshot> for RuntimeDecisionState {
    fn from(value: &StrategyDecisionSnapshot) -> Self {
        Self {
            symbol: value.symbol.clone(),
            quantity: value.quantity,
            side: value.side.clone(),
            mark: value.mark,
            created_at: value.created_at,
            metadata: None,
        }
    }
}

impl From<&RuntimeProducerDecision> for RuntimeDecisionState {
    fn from(value: &RuntimeProducerDecision) -> Self {
        Self {
            symbol: value.symbol.clone(),
            quantity: value.quantity,
            side: value.side.clone(),
            mark: value.mark,
            created_at: value.created_at,
            metadata: value.metadata.clone(),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct RuntimeExecutionState {
    pub positions: Vec<RuntimePositionState>,
    pub historic: Vec<RuntimeHistoricPositionState>,
    pub orders: Vec<RuntimeOrderState>,
    pub decisions: Vec<RuntimeDecisionState>,
}

impl RuntimeExecutionState {
    pub fn from_snapshot(snapshot: &SystemSnapshot) -> Self {
        Self {
            positions: snapshot
                .positions
                .iter()
                .map(|p| {
                    let mut r = RuntimePositionState::from(p);
                    if r.account_id.is_none() {
                        r.account_id = Some(snapshot.account_id.clone());
                    }
                    r
                })
                .collect(),
            historic: snapshot
                .historic
                .iter()
                .map(RuntimeHistoricPositionState::from)
                .collect(),
            orders: snapshot
                .orders
                .iter()
                .map(RuntimeOrderState::from)
                .collect(),
            decisions: snapshot
                .decisions
                .iter()
                .map(RuntimeDecisionState::from)
                .collect(),
        }
    }

    pub fn apply_strategy_decision(
        &mut self,
        decision: &StrategyDecisionSnapshot,
        order_id: String,
        account_id: &str,
    ) -> RuntimeExecutionUpdate {
        self.decisions.push(RuntimeDecisionState::from(decision));
        if self.decisions.len() > 50 {
            self.decisions.remove(0);
        }
        self.apply_decision_order_and_position(
            &decision.symbol,
            decision.quantity,
            &decision.side,
            decision.mark,
            decision.created_at,
            order_id,
            account_id,
        )
    }

    pub fn apply_producer_decision(
        &mut self,
        decision: &RuntimeProducerDecision,
        order_id: String,
        account_id: &str,
    ) -> RuntimeExecutionUpdate {
        self.decisions.push(RuntimeDecisionState::from(decision));
        if self.decisions.len() > 50 {
            self.decisions.remove(0);
        }
        let account = decision.account_id().unwrap_or(account_id).to_string();
        self.apply_decision_order_and_position(
            &decision.symbol,
            decision.quantity,
            &decision.side,
            decision.mark,
            decision.created_at,
            order_id,
            &account,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn apply_decision_order_and_position(
        &mut self,
        symbol: &str,
        quantity: i32,
        side: &str,
        mark: f64,
        created_at: DateTime<Utc>,
        order_id: String,
        account_id: &str,
    ) -> RuntimeExecutionUpdate {
        self.orders.push(RuntimeOrderState {
            id: order_id.clone(),
            symbol: symbol.to_string(),
            side: side.to_string(),
            quantity,
            status: "FILLED".into(),
            submitted_at: created_at,
        });
        if self.orders.len() > 32 {
            self.orders.remove(0);
        }

        if let Some(idx) = self
            .positions
            .iter()
            .position(|position| position.symbol == symbol)
        {
            let prev_qty = self.positions[idx].quantity;
            let cost_basis = self.positions[idx].cost_basis;
            let new_qty = prev_qty + quantity;

            if new_qty == 0 {
                let position = self.positions.remove(idx);
                let realized = (mark - cost_basis) * prev_qty as f64;
                self.historic.push(RuntimeHistoricPositionState {
                    id: format!("HIST-{}", self.historic.len() + 1),
                    symbol: position.symbol.clone(),
                    quantity: prev_qty,
                    realized_pnl: realized,
                    closed_at: created_at,
                });

                RuntimeExecutionUpdate::ClosedPosition {
                    symbol: position.symbol,
                    quantity: prev_qty,
                    cost_basis,
                    mark,
                    order_id,
                }
            } else {
                let position = &mut self.positions[idx];
                position.quantity = new_qty;
                position.mark = mark;
                position.unrealized_pnl =
                    (position.mark - position.cost_basis) * position.quantity as f64;

                RuntimeExecutionUpdate::ChangedPosition {
                    symbol: symbol.to_string(),
                    quantity,
                    mark,
                    order_id,
                }
            }
        } else {
            self.positions.push(RuntimePositionState {
                id: format!("POS-{}", self.positions.len() + self.historic.len() + 1),
                symbol: symbol.to_string(),
                quantity,
                cost_basis: mark,
                mark,
                unrealized_pnl: 0.0,
                account_id: Some(account_id.to_string()),
            });

            RuntimeExecutionUpdate::ChangedPosition {
                symbol: symbol.to_string(),
                quantity,
                mark,
                order_id,
            }
        }
    }

    pub fn project_into_snapshot(self, snapshot: &mut SystemSnapshot) {
        snapshot.positions = self
            .positions
            .into_iter()
            .map(PositionSnapshot::from)
            .collect();
        snapshot.historic = self
            .historic
            .into_iter()
            .map(HistoricPosition::from)
            .collect();
        snapshot.orders = self.orders.into_iter().map(OrderSnapshot::from).collect();
        snapshot.decisions = self
            .decisions
            .into_iter()
            .map(|decision| {
                StrategyDecisionSnapshot::new(
                    decision.symbol,
                    decision.quantity,
                    decision.side,
                    decision.mark,
                    decision.created_at,
                )
            })
            .collect();
    }

    pub fn risk_limit_for_decision(&self, decision: &RuntimeProducerDecision) -> RiskLimit {
        let target_qty = self.position_quantity(&decision.symbol) + decision.quantity;
        RiskLimit {
            symbol: decision.symbol.clone(),
            max_position: target_qty.abs(),
            max_notional: decision.mark * target_qty.abs() as f64,
        }
    }

    pub fn position_quantity(&self, symbol: &str) -> i32 {
        self.positions
            .iter()
            .find(|position| position.symbol == symbol)
            .map(|position| position.quantity)
            .unwrap_or(0)
    }

    pub fn decisions_by_producer_type(
        &self,
        producer_type: &ProducerType,
    ) -> Vec<&RuntimeDecisionState> {
        self.decisions
            .iter()
            .filter(|d| {
                d.metadata
                    .as_ref()
                    .map(|m| &m.producer_type == producer_type)
                    .unwrap_or(false)
            })
            .collect()
    }

    pub fn find_by_correlation_id(&self, correlation_id: &str) -> Vec<&RuntimeDecisionState> {
        self.decisions
            .iter()
            .filter(|d| {
                d.metadata
                    .as_ref()
                    .and_then(|m| m.correlation_id.as_deref())
                    .map(|s| s == correlation_id)
                    .unwrap_or(false)
            })
            .collect()
    }

    pub fn positions_by_account(&self, account_id: &str) -> Vec<&RuntimePositionState> {
        self.positions
            .iter()
            .filter(|p| p.account_id.as_deref() == Some(account_id))
            .collect()
    }
}

#[derive(Clone, Debug)]
pub enum RuntimeExecutionUpdate {
    ChangedPosition {
        symbol: String,
        quantity: i32,
        mark: f64,
        order_id: String,
    },
    ClosedPosition {
        symbol: String,
        quantity: i32,
        cost_basis: f64,
        mark: f64,
        order_id: String,
    },
}

#[derive(Clone, Debug)]
pub struct RuntimeMarketState {
    pub symbols: Vec<SymbolSnapshot>,
    pub metrics: Metrics,
    last_cumulative_volume: HashMap<String, u64>,
}

impl RuntimeMarketState {
    pub fn from_snapshot(snapshot: &SystemSnapshot) -> Self {
        Self {
            symbols: snapshot.symbols.clone(),
            metrics: snapshot.metrics.clone(),
            last_cumulative_volume: HashMap::new(),
        }
    }

    pub fn apply_market_event(&mut self, event: &MarketDataEvent) {
        let volume_delta = if event.volume > 0 {
            let last = self
                .last_cumulative_volume
                .get(&event.symbol)
                .copied()
                .unwrap_or(0);
            if event.volume > last {
                let delta = event.volume - last;
                self.last_cumulative_volume
                    .insert(event.symbol.clone(), event.volume);
                delta
            } else {
                1
            }
        } else {
            1
        };

        if let Some(entry) = self
            .symbols
            .iter_mut()
            .find(|symbol| symbol.symbol == event.symbol)
        {
            let mid = (event.bid + event.ask) * 0.5;
            entry.last = mid;
            entry.bid = event.bid;
            entry.ask = event.ask;
            entry.spread = (event.ask - event.bid).max(0.0);
            entry.volume = entry.volume.saturating_add(volume_delta);
            entry.roi = (entry.roi * 0.9) + 0.1 * ((mid / entry.candle.entry) - 1.0) * 100.0;
            entry.candle.high = entry.candle.high.max(mid);
            entry.candle.low = entry.candle.low.min(mid);
            entry.candle.close = mid;
            entry.candle.volume = entry.candle.volume.saturating_add(volume_delta);
            entry.candle.updated = event.timestamp;
        } else {
            let mid = (event.bid + event.ask) * 0.5;
            self.symbols.push(SymbolSnapshot {
                symbol: event.symbol.clone(),
                last: mid,
                bid: event.bid,
                ask: event.ask,
                spread: (event.ask - event.bid).max(0.0),
                roi: 0.0,
                maker_count: 1,
                taker_count: 0,
                volume: volume_delta,
                candle: CandleSnapshot {
                    open: mid,
                    high: mid,
                    low: mid,
                    close: mid,
                    volume: volume_delta,
                    entry: mid,
                    updated: event.timestamp,
                },
            });
        }

        let mark_sum = self.symbols.iter().map(|symbol| symbol.last).sum::<f64>();
        self.metrics.net_liq = 100_000.0 + mark_sum;
        self.metrics.buying_power = self.metrics.net_liq * 0.8;
        self.metrics.excess_liquidity = self.metrics.net_liq * 0.25;
        self.metrics.margin_requirement = self.metrics.net_liq * 0.15;
        self.metrics.commissions = (self.metrics.commissions * 0.98) + 0.02;
        self.metrics.portal_ok = true;
        self.metrics.tws_ok = true;
        self.metrics.questdb_ok = true;
    }

    pub fn project_into_snapshot(self, snapshot: &mut SystemSnapshot) {
        snapshot.symbols = self.symbols;
        snapshot.metrics = self.metrics;
    }

    pub fn mark_for_symbol(&self, symbol: &str) -> Option<f64> {
        self.symbols
            .iter()
            .find(|entry| entry.symbol == symbol)
            .map(|entry| entry.last)
    }
}

#[derive(Clone, Debug)]
pub struct RuntimeRiskState {
    pub allowed: bool,
    pub reason: Option<String>,
    pub updated_at: DateTime<Utc>,
}

impl RuntimeRiskState {
    pub fn from_snapshot(snapshot: &SystemSnapshot) -> Self {
        Self {
            allowed: snapshot.risk.allowed,
            reason: snapshot.risk.reason.clone(),
            updated_at: snapshot.risk.updated_at,
        }
    }

    pub fn apply_risk_decision(&mut self, outcome: &RiskDecision) {
        self.allowed = outcome.allowed;
        self.reason = outcome.reason.clone();
        self.updated_at = Utc::now();
    }

    pub fn project_into_snapshot(self, snapshot: &mut SystemSnapshot) {
        snapshot.risk.allowed = self.allowed;
        snapshot.risk.reason = self.reason;
        snapshot.risk.updated_at = self.updated_at;
    }
}

pub fn apply_market_event(snapshot: &mut SystemSnapshot, event: &MarketDataEvent) {
    snapshot.touch();
    let mut market_state = RuntimeMarketState::from_snapshot(snapshot);
    market_state.apply_market_event(event);
    market_state.project_into_snapshot(snapshot);
}

pub fn apply_strategy_execution(
    snapshot: &mut SystemSnapshot,
    decision: StrategyDecisionSnapshot,
) -> RuntimeExecutionUpdate {
    snapshot.touch();
    snapshot.strategy = "RUNNING".into();
    let order_id = format!("ORD-{}", Utc::now().timestamp_millis());
    let mut execution_state = RuntimeExecutionState::from_snapshot(snapshot);
    let update =
        execution_state.apply_strategy_decision(&decision, order_id.clone(), &snapshot.account_id);
    execution_state.project_into_snapshot(snapshot);
    update
}

pub fn apply_risk_status(snapshot: &mut SystemSnapshot, outcome: &RiskDecision) {
    snapshot.touch();
    let mut risk_state = RuntimeRiskState::from_snapshot(snapshot);
    risk_state.apply_risk_decision(outcome);
    risk_state.project_into_snapshot(snapshot);
}

#[cfg(test)]
mod tests {
    use super::*;
    use api::state::SystemSnapshot;
    use market_data::MarketDataEvent;

    #[test]
    fn market_event_updates_snapshot() {
        let mut snapshot = SystemSnapshot::default();
        let event = MarketDataEvent {
            symbol: "SPY".into(),
            bid: 500.0,
            ask: 502.0,
            timestamp: Utc::now(),
            ..Default::default()
        };

        apply_market_event(&mut snapshot, &event);

        assert_eq!(snapshot.symbols.len(), 1);
        assert_eq!(snapshot.symbols[0].symbol, "SPY");
    }

    #[test]
    fn strategy_execution_updates_positions() {
        let mut snapshot = SystemSnapshot::default();
        let update = apply_strategy_execution(
            &mut snapshot,
            StrategyDecisionSnapshot::new("AAPL".into(), 5, "BUY".into(), 150.0, Utc::now()),
        );

        assert!(matches!(
            update,
            RuntimeExecutionUpdate::ChangedPosition { .. }
        ));
        assert_eq!(snapshot.orders.len(), 1);
    }

    #[test]
    fn risk_status_update_applies_outcome() {
        let mut snapshot = SystemSnapshot::default();
        let outcome = RiskDecision {
            allowed: false,
            reason: Some("limit".into()),
        };

        apply_risk_status(&mut snapshot, &outcome);

        assert!(!snapshot.risk.allowed);
        assert_eq!(snapshot.risk.reason.as_deref(), Some("limit"));
    }
}
