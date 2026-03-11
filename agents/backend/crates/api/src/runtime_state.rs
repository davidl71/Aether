use chrono::{DateTime, Utc};
use market_data::MarketDataEvent;
use risk::{RiskDecision, RiskLimit};
use serde::{Deserialize, Serialize};
use strategy::model::{Decision as StrategyDecisionModel, TradeSide};

use crate::state::{
    Alert, CandleSnapshot, HistoricPosition, Metrics, OrderSnapshot, PositionSnapshot, RiskStatus,
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
}

/// Type of producer that generated a decision or event
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ProducerType {
    Strategy,
    Market,
    Risk,
    Native,
    Broker,
}

/// Classification of financial instruments for type-specific handling
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum InstrumentType {
    Stock,
    Option,
    Future,
    Bond,
    BoxSpread,
    MultiLeg,
}

/// Rich metadata about the producer and context of a decision/event
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ProducerMetadata {
    /// Type of producer (strategy, market, risk, etc.)
    pub producer_type: ProducerType,
    /// Specific instance identifier (e.g., "momentum_v1", "risk_engine_2")
    pub producer_id: Option<String>,
    /// Broker source for broker-originated events (e.g., "ibkr", "alpaca")
    pub broker_source: Option<String>,
    /// Account identifier for multi-account systems
    pub account_id: Option<String>,
    /// Correlation ID for tracing decisions across producer boundaries
    pub correlation_id: Option<String>,
    /// Monotonic sequence number for ordering and conflict resolution
    pub sequence: Option<u64>,
    /// Instrument type for specialized handling
    pub instrument_type: Option<InstrumentType>,
}

impl ProducerMetadata {
    /// Create minimal metadata with just producer type
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

    /// Builder pattern: set producer ID
    pub fn with_producer_id(mut self, id: String) -> Self {
        self.producer_id = Some(id);
        self
    }

    /// Builder pattern: set broker source
    pub fn with_broker_source(mut self, broker: String) -> Self {
        self.broker_source = Some(broker);
        self
    }

    /// Builder pattern: set account ID
    pub fn with_account_id(mut self, account: String) -> Self {
        self.account_id = Some(account);
        self
    }

    /// Builder pattern: set correlation ID
    pub fn with_correlation_id(mut self, correlation: String) -> Self {
        self.correlation_id = Some(correlation);
        self
    }

    /// Builder pattern: set sequence number
    pub fn with_sequence(mut self, seq: u64) -> Self {
        self.sequence = Some(seq);
        self
    }

    /// Builder pattern: set instrument type
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
    /// Optional rich metadata about the producer and context
    pub metadata: Option<ProducerMetadata>,
}

impl RuntimeProducerDecision {
    /// Create from strategy decision with minimal metadata (backward compatible)
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

    /// Create with full producer metadata
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

    /// Check if this decision matches a specific producer type
    pub fn is_from_producer_type(&self, producer_type: &ProducerType) -> bool {
        self.metadata
            .as_ref()
            .map(|m| &m.producer_type == producer_type)
            .unwrap_or(false)
    }

    /// Get the correlation ID if present
    pub fn correlation_id(&self) -> Option<&str> {
        self.metadata
            .as_ref()
            .and_then(|m| m.correlation_id.as_deref())
    }

    /// Get the account ID if present
    pub fn account_id(&self) -> Option<&str> {
        self.metadata.as_ref().and_then(|m| m.account_id.as_deref())
    }

    /// Get the broker source if present
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
        }
    }
}

impl From<RuntimeDecisionState> for StrategyDecisionSnapshot {
    fn from(value: RuntimeDecisionState) -> Self {
        Self {
            symbol: value.symbol,
            quantity: value.quantity,
            side: value.side,
            mark: value.mark,
            created_at: value.created_at,
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
                .map(RuntimePositionState::from)
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
    ) -> RuntimeExecutionUpdate {
        self.decisions.push(RuntimeDecisionState::from(decision));
        if self.decisions.len() > 50 {
            self.decisions.remove(0);
        }

        self.orders.push(RuntimeOrderState {
            id: order_id.clone(),
            symbol: decision.symbol.clone(),
            side: decision.side.clone(),
            quantity: decision.quantity,
            status: "FILLED".into(),
            submitted_at: decision.created_at,
        });
        if self.orders.len() > 32 {
            self.orders.remove(0);
        }

        if let Some(idx) = self
            .positions
            .iter()
            .position(|position| position.symbol == decision.symbol)
        {
            let prev_qty = self.positions[idx].quantity;
            let cost_basis = self.positions[idx].cost_basis;
            let new_qty = prev_qty + decision.quantity;

            if new_qty == 0 {
                let position = self.positions.remove(idx);
                let realized = (decision.mark - cost_basis) * prev_qty as f64;
                self.historic.push(RuntimeHistoricPositionState {
                    id: format!("HIST-{}", self.historic.len() + 1),
                    symbol: position.symbol.clone(),
                    quantity: prev_qty,
                    realized_pnl: realized,
                    closed_at: decision.created_at,
                });

                RuntimeExecutionUpdate::ClosedPosition {
                    symbol: position.symbol,
                    quantity: prev_qty,
                    cost_basis,
                    mark: decision.mark,
                    order_id,
                }
            } else {
                let position = &mut self.positions[idx];
                position.quantity = new_qty;
                position.mark = decision.mark;
                position.unrealized_pnl =
                    (position.mark - position.cost_basis) * position.quantity as f64;

                RuntimeExecutionUpdate::ChangedPosition {
                    symbol: decision.symbol.clone(),
                    quantity: decision.quantity,
                    mark: decision.mark,
                    order_id,
                }
            }
        } else {
            self.positions.push(RuntimePositionState {
                id: format!("POS-{}", self.positions.len() + self.historic.len() + 1),
                symbol: decision.symbol.clone(),
                quantity: decision.quantity,
                cost_basis: decision.mark,
                mark: decision.mark,
                unrealized_pnl: 0.0,
            });

            RuntimeExecutionUpdate::ChangedPosition {
                symbol: decision.symbol.clone(),
                quantity: decision.quantity,
                mark: decision.mark,
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
            .map(StrategyDecisionSnapshot::from)
            .collect();
    }

    pub fn position_dtos(&self) -> Vec<RuntimePositionDto> {
        self.positions
            .iter()
            .map(RuntimePositionDto::from)
            .collect()
    }

    pub fn order_dtos(&self) -> Vec<RuntimeOrderDto> {
        self.orders.iter().map(RuntimeOrderDto::from).collect()
    }

    pub fn decision_dtos(&self) -> Vec<RuntimeDecisionDto> {
        self.decisions
            .iter()
            .map(RuntimeDecisionDto::from)
            .collect()
    }

    pub fn historic_dtos(&self) -> Vec<RuntimeHistoricPositionDto> {
        self.historic
            .iter()
            .map(RuntimeHistoricPositionDto::from)
            .collect()
    }

    pub fn find_position_dto(&self, position_id: &str) -> Option<RuntimePositionDto> {
        self.positions
            .iter()
            .find(|position| position.id == position_id)
            .map(RuntimePositionDto::from)
    }

    pub fn find_order_dto(&self, order_id: &str) -> Option<RuntimeOrderDto> {
        self.orders
            .iter()
            .find(|order| order.id == order_id)
            .map(RuntimeOrderDto::from)
    }

    pub fn position_quantity(&self, symbol: &str) -> i32 {
        self.positions
            .iter()
            .find(|position| position.symbol == symbol)
            .map(|position| position.quantity)
            .unwrap_or(0)
    }

    pub fn risk_limit_for_decision(&self, decision: &RuntimeProducerDecision) -> RiskLimit {
        let target_qty = self.position_quantity(&decision.symbol) + decision.quantity;
        RiskLimit {
            symbol: decision.symbol.clone(),
            max_position: target_qty.abs(),
            max_notional: decision.mark * target_qty.abs() as f64,
        }
    }

    /// Filter decisions by producer type
    pub fn decisions_by_producer_type(
        &self,
        _producer_type: &ProducerType,
    ) -> Vec<&RuntimeDecisionState> {
        // Note: RuntimeDecisionState doesn't have metadata yet, so we can't filter.
        // This is a placeholder for when decisions store producer metadata.
        // For now, return empty vec.
        Vec::new()
    }

    /// Find decisions by correlation ID (for tracing across producers)
    pub fn find_by_correlation_id(&self, _correlation_id: &str) -> Vec<&RuntimeDecisionState> {
        // Note: RuntimeDecisionState doesn't have metadata yet.
        // This is a placeholder for future enhancement.
        Vec::new()
    }

    /// Group positions by account ID (for multi-account systems)
    pub fn positions_by_account(&self, _account_id: &str) -> Vec<&RuntimePositionState> {
        // Note: RuntimePositionState doesn't have account_id yet.
        // This is a placeholder for future enhancement.
        Vec::new()
    }
}

#[derive(Clone, Debug, PartialEq)]
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
}

impl RuntimeMarketState {
    pub fn from_snapshot(snapshot: &SystemSnapshot) -> Self {
        Self {
            symbols: snapshot.symbols.clone(),
            metrics: snapshot.metrics.clone(),
        }
    }

    pub fn apply_market_event(&mut self, event: &MarketDataEvent) {
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
            entry.volume = entry.volume.saturating_add(1);
            entry.roi = (entry.roi * 0.9) + 0.1 * ((mid / entry.candle.entry) - 1.0) * 100.0;
            entry.candle.high = entry.candle.high.max(mid);
            entry.candle.low = entry.candle.low.min(mid);
            entry.candle.close = mid;
            entry.candle.volume = entry.candle.volume.saturating_add(1);
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
                volume: 1,
                candle: CandleSnapshot {
                    open: mid,
                    high: mid,
                    low: mid,
                    close: mid,
                    volume: 1,
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

#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct RuntimePositionDto {
    pub id: String,
    pub symbol: String,
    pub quantity: i32,
    pub cost_basis: f64,
    pub mark: f64,
    pub unrealized_pnl: f64,
    pub market_value: f64,
}

impl From<&PositionSnapshot> for RuntimePositionDto {
    fn from(value: &PositionSnapshot) -> Self {
        Self::from(&RuntimePositionState::from(value))
    }
}

impl From<&RuntimePositionState> for RuntimePositionDto {
    fn from(value: &RuntimePositionState) -> Self {
        Self {
            id: value.id.clone(),
            symbol: value.symbol.clone(),
            quantity: value.quantity,
            cost_basis: value.cost_basis,
            mark: value.mark,
            unrealized_pnl: value.unrealized_pnl,
            market_value: value.mark * value.quantity as f64,
        }
    }
}

#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct RuntimeHistoricPositionDto {
    pub id: String,
    pub symbol: String,
    pub quantity: i32,
    pub realized_pnl: f64,
    pub closed_at: DateTime<Utc>,
}

impl From<&HistoricPosition> for RuntimeHistoricPositionDto {
    fn from(value: &HistoricPosition) -> Self {
        Self::from(&RuntimeHistoricPositionState::from(value))
    }
}

impl From<&RuntimeHistoricPositionState> for RuntimeHistoricPositionDto {
    fn from(value: &RuntimeHistoricPositionState) -> Self {
        Self {
            id: value.id.clone(),
            symbol: value.symbol.clone(),
            quantity: value.quantity,
            realized_pnl: value.realized_pnl,
            closed_at: value.closed_at,
        }
    }
}

#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct RuntimeOrderDto {
    pub id: String,
    pub symbol: String,
    pub side: String,
    pub quantity: i32,
    pub status: String,
    pub submitted_at: DateTime<Utc>,
}

impl From<&OrderSnapshot> for RuntimeOrderDto {
    fn from(value: &OrderSnapshot) -> Self {
        Self::from(&RuntimeOrderState::from(value))
    }
}

impl From<&RuntimeOrderState> for RuntimeOrderDto {
    fn from(value: &RuntimeOrderState) -> Self {
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

#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct RuntimeDecisionDto {
    pub symbol: String,
    pub quantity: i32,
    pub side: String,
    pub mark: f64,
    pub created_at: DateTime<Utc>,
}

impl From<&StrategyDecisionSnapshot> for RuntimeDecisionDto {
    fn from(value: &StrategyDecisionSnapshot) -> Self {
        Self::from(&RuntimeDecisionState::from(value))
    }
}

impl From<&RuntimeDecisionState> for RuntimeDecisionDto {
    fn from(value: &RuntimeDecisionState) -> Self {
        Self {
            symbol: value.symbol.clone(),
            quantity: value.quantity,
            side: value.side.clone(),
            mark: value.mark,
            created_at: value.created_at,
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct RuntimeSnapshotDto {
    pub generated_at: DateTime<Utc>,
    pub started_at: DateTime<Utc>,
    pub mode: String,
    pub strategy: String,
    pub account_id: String,
    pub metrics: Metrics,
    pub symbols: Vec<SymbolSnapshot>,
    pub positions: Vec<RuntimePositionDto>,
    pub historic: Vec<RuntimeHistoricPositionDto>,
    pub orders: Vec<RuntimeOrderDto>,
    pub decisions: Vec<RuntimeDecisionDto>,
    pub alerts: Vec<Alert>,
    pub risk: RiskStatus,
}

impl From<&SystemSnapshot> for RuntimeSnapshotDto {
    fn from(value: &SystemSnapshot) -> Self {
        let runtime_state = RuntimeExecutionState::from_snapshot(value);
        Self {
            generated_at: value.generated_at,
            started_at: value.started_at,
            mode: value.mode.clone(),
            strategy: value.strategy.clone(),
            account_id: value.account_id.clone(),
            metrics: value.metrics.clone(),
            symbols: value.symbols.clone(),
            positions: runtime_state
                .positions
                .iter()
                .map(RuntimePositionDto::from)
                .collect(),
            historic: runtime_state
                .historic
                .iter()
                .map(RuntimeHistoricPositionDto::from)
                .collect(),
            orders: runtime_state
                .orders
                .iter()
                .map(RuntimeOrderDto::from)
                .collect(),
            decisions: runtime_state
                .decisions
                .iter()
                .map(RuntimeDecisionDto::from)
                .collect(),
            alerts: value.alerts.clone(),
            risk: value.risk.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn market_state_applies_new_symbol_event() {
        let mut snapshot = SystemSnapshot::default();
        snapshot.metrics.commissions = 1.0;
        let mut market = RuntimeMarketState::from_snapshot(&snapshot);
        let event = MarketDataEvent {
            symbol: "SPY".into(),
            bid: 500.0,
            ask: 502.0,
            timestamp: Utc::now(),
        };

        market.apply_market_event(&event);

        assert_eq!(market.symbols.len(), 1);
        assert_eq!(market.symbols[0].symbol, "SPY");
        assert_eq!(market.symbols[0].last, 501.0);
        assert!(market.metrics.portal_ok);
        assert!(market.metrics.tws_ok);
        assert!(market.metrics.questdb_ok);
        assert!((market.metrics.net_liq - 100_501.0).abs() < f64::EPSILON);
    }

    #[test]
    fn market_state_updates_existing_symbol_event() {
        let mut snapshot = SystemSnapshot::default();
        snapshot.symbols.push(SymbolSnapshot {
            symbol: "SPY".into(),
            last: 500.0,
            bid: 499.0,
            ask: 501.0,
            spread: 2.0,
            roi: 0.0,
            maker_count: 1,
            taker_count: 0,
            volume: 1,
            candle: CandleSnapshot {
                open: 500.0,
                high: 500.0,
                low: 500.0,
                close: 500.0,
                volume: 1,
                entry: 500.0,
                updated: Utc::now(),
            },
        });
        let mut market = RuntimeMarketState::from_snapshot(&snapshot);
        let event = MarketDataEvent {
            symbol: "SPY".into(),
            bid: 504.0,
            ask: 506.0,
            timestamp: Utc::now(),
        };

        market.apply_market_event(&event);

        assert_eq!(market.symbols.len(), 1);
        assert_eq!(market.symbols[0].last, 505.0);
        assert_eq!(market.symbols[0].volume, 2);
        assert_eq!(market.symbols[0].candle.close, 505.0);
    }

    #[test]
    fn risk_state_applies_decision() {
        let snapshot = SystemSnapshot::default();
        let mut risk = RuntimeRiskState::from_snapshot(&snapshot);
        let outcome = RiskDecision {
            allowed: false,
            reason: Some("position limit".into()),
        };

        risk.apply_risk_decision(&outcome);

        assert!(!risk.allowed);
        assert_eq!(risk.reason.as_deref(), Some("position limit"));
    }

    #[test]
    fn producer_decision_maps_strategy_engine_input() {
        let created_at = Utc::now();
        let decision = StrategyDecisionModel {
            symbol: "SPY".into(),
            quantity: 5,
            side: TradeSide::Buy,
        };

        let producer =
            RuntimeProducerDecision::from_strategy_decision(&decision, 501.0, created_at);

        assert_eq!(producer.symbol, "SPY");
        assert_eq!(producer.quantity, 5);
        assert_eq!(producer.side, "BUY");
        assert_eq!(producer.mark, 501.0);
        assert_eq!(producer.to_snapshot().symbol, "SPY");
    }

    #[test]
    fn execution_state_builds_risk_limit_from_producer_decision() {
        let mut snapshot = SystemSnapshot::default();
        snapshot.positions.push(PositionSnapshot {
            id: "POS-1".into(),
            symbol: "SPY".into(),
            quantity: 2,
            cost_basis: 500.0,
            mark: 501.0,
            unrealized_pnl: 2.0,
        });
        let execution = RuntimeExecutionState::from_snapshot(&snapshot);
        let producer = RuntimeProducerDecision {
            symbol: "SPY".into(),
            quantity: 3,
            side: "BUY".into(),
            mark: 501.0,
            created_at: Utc::now(),
            metadata: Some(ProducerMetadata::new(ProducerType::Strategy)),
        };

        let request = execution.risk_limit_for_decision(&producer);

        assert_eq!(request.symbol, "SPY");
        assert_eq!(request.max_position, 5);
        assert_eq!(request.max_notional, 2505.0);
    }

    #[test]
    fn producer_metadata_builder_pattern() {
        let metadata = ProducerMetadata::new(ProducerType::Strategy)
            .with_producer_id("momentum_v1".into())
            .with_account_id("U12345".into())
            .with_correlation_id("trade-123".into())
            .with_sequence(42)
            .with_instrument_type(InstrumentType::Stock);

        assert_eq!(metadata.producer_type, ProducerType::Strategy);
        assert_eq!(metadata.producer_id.as_deref(), Some("momentum_v1"));
        assert_eq!(metadata.account_id.as_deref(), Some("U12345"));
        assert_eq!(metadata.correlation_id.as_deref(), Some("trade-123"));
        assert_eq!(metadata.sequence, Some(42));
        assert_eq!(metadata.instrument_type, Some(InstrumentType::Stock));
    }

    #[test]
    fn producer_decision_with_rich_metadata() {
        let metadata = ProducerMetadata::new(ProducerType::Broker)
            .with_broker_source("ibkr".into())
            .with_account_id("DU12345".into())
            .with_instrument_type(InstrumentType::Option);

        let decision = RuntimeProducerDecision::new_with_metadata(
            "SPY".into(),
            5,
            "BUY".into(),
            501.0,
            Utc::now(),
            metadata,
        );

        assert_eq!(decision.symbol, "SPY");
        assert!(decision.is_from_producer_type(&ProducerType::Broker));
        assert_eq!(decision.broker_source(), Some("ibkr"));
        assert_eq!(decision.account_id(), Some("DU12345"));
    }

    #[test]
    fn producer_decision_backward_compatible() {
        let decision_model = StrategyDecisionModel {
            symbol: "SPY".into(),
            quantity: 5,
            side: TradeSide::Buy,
        };

        let producer =
            RuntimeProducerDecision::from_strategy_decision(&decision_model, 501.0, Utc::now());

        // Should have minimal metadata set
        assert!(producer.is_from_producer_type(&ProducerType::Strategy));
        assert_eq!(producer.correlation_id(), None);
        assert_eq!(producer.broker_source(), None);
    }

    #[test]
    fn producer_metadata_correlation_tracking() {
        let correlation = "trade-session-789";

        let decision1 = RuntimeProducerDecision::new_with_metadata(
            "SPY".into(),
            5,
            "BUY".into(),
            501.0,
            Utc::now(),
            ProducerMetadata::new(ProducerType::Strategy).with_correlation_id(correlation.into()),
        );

        let decision2 = RuntimeProducerDecision::new_with_metadata(
            "QQQ".into(),
            3,
            "SELL".into(),
            350.0,
            Utc::now(),
            ProducerMetadata::new(ProducerType::Risk).with_correlation_id(correlation.into()),
        );

        assert_eq!(decision1.correlation_id(), Some(correlation));
        assert_eq!(decision2.correlation_id(), Some(correlation));
    }

    #[test]
    fn producer_metadata_multi_account_support() {
        let account1_decision = RuntimeProducerDecision::new_with_metadata(
            "SPY".into(),
            5,
            "BUY".into(),
            501.0,
            Utc::now(),
            ProducerMetadata::new(ProducerType::Strategy).with_account_id("U12345".into()),
        );

        let account2_decision = RuntimeProducerDecision::new_with_metadata(
            "SPY".into(),
            3,
            "BUY".into(),
            501.0,
            Utc::now(),
            ProducerMetadata::new(ProducerType::Strategy).with_account_id("U67890".into()),
        );

        assert_eq!(account1_decision.account_id(), Some("U12345"));
        assert_eq!(account2_decision.account_id(), Some("U67890"));
    }
}
