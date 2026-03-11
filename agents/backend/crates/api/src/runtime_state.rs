use chrono::{DateTime, Utc};
use serde::Serialize;

use crate::state::{
    Alert, HistoricPosition, Metrics, OrderSnapshot, PositionSnapshot, RiskStatus,
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
            orders: snapshot.orders.iter().map(RuntimeOrderState::from).collect(),
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
        snapshot.positions = self.positions.into_iter().map(PositionSnapshot::from).collect();
        snapshot.historic = self.historic.into_iter().map(HistoricPosition::from).collect();
        snapshot.orders = self.orders.into_iter().map(OrderSnapshot::from).collect();
        snapshot.decisions = self
            .decisions
            .into_iter()
            .map(StrategyDecisionSnapshot::from)
            .collect();
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
