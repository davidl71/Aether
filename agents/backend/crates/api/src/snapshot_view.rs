use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::finance_rates::BenchmarksResponse;
use crate::state::{
    Alert, HistoricPosition, Metrics, OrderSnapshot, PositionSnapshot, RiskStatus,
    StrategyDecisionSnapshot, SymbolSnapshot, SystemSnapshot,
};
use crate::NatsTransportHealthState;

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct RuntimePositionDto {
    pub id: String,
    pub symbol: String,
    pub position_type: Option<String>,
    pub strategy: Option<String>,
    pub quantity: i32,
    pub cost_basis: f64,
    pub mark: f64,
    pub unrealized_pnl: f64,
    pub market_value: f64,
    pub account_id: Option<String>,
    pub apr_pct: Option<f64>,
    pub source: Option<String>,
}

impl From<&PositionSnapshot> for RuntimePositionDto {
    fn from(value: &PositionSnapshot) -> Self {
        let market_value = value.mark * value.quantity as f64;
        Self {
            id: value.id.clone(),
            symbol: value.symbol.clone(),
            position_type: None,
            strategy: None,
            quantity: value.quantity,
            cost_basis: value.cost_basis,
            mark: value.mark,
            unrealized_pnl: value.unrealized_pnl,
            market_value,
            account_id: value.account_id.clone(),
            apr_pct: None,
            source: None,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct RuntimeHistoricPositionDto {
    pub id: String,
    pub symbol: String,
    pub quantity: i32,
    pub realized_pnl: f64,
    pub closed_at: DateTime<Utc>,
}

impl From<&HistoricPosition> for RuntimeHistoricPositionDto {
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

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
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

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct RuntimeDecisionDto {
    pub symbol: String,
    pub quantity: i32,
    pub side: String,
    pub mark: f64,
    pub created_at: DateTime<Utc>,
}

impl From<&StrategyDecisionSnapshot> for RuntimeDecisionDto {
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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RuntimeSnapshotDto {
    pub generated_at: DateTime<Utc>,
    pub started_at: DateTime<Utc>,
    pub mode: String,
    pub strategy: String,
    pub account_id: String,
    pub metrics: Metrics,
    /// Backend NATS transport telemetry (present when publisher populates it).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nats_transport: Option<NatsTransportHealthState>,
    pub symbols: Vec<SymbolSnapshot>,
    pub positions: Vec<RuntimePositionDto>,
    pub historic: Vec<RuntimeHistoricPositionDto>,
    pub orders: Vec<RuntimeOrderDto>,
    pub decisions: Vec<RuntimeDecisionDto>,
    pub alerts: Vec<Alert>,
    pub risk: RiskStatus,
    pub scenarios: Vec<ScenarioDto>,
    #[serde(skip)]
    pub yield_benchmarks: Option<BenchmarksResponse>,
}

impl From<&SystemSnapshot> for RuntimeSnapshotDto {
    fn from(value: &SystemSnapshot) -> Self {
        Self {
            generated_at: value.generated_at,
            started_at: value.started_at,
            mode: value.mode.clone(),
            strategy: value.strategy.clone(),
            account_id: value.account_id.clone(),
            metrics: value.metrics.clone(),
            nats_transport: value.nats_transport.clone(),
            symbols: value.symbols.clone(),
            positions: value
                .positions
                .iter()
                .map(RuntimePositionDto::from)
                .collect(),
            historic: value
                .historic
                .iter()
                .map(RuntimeHistoricPositionDto::from)
                .collect(),
            orders: value.orders.iter().map(RuntimeOrderDto::from).collect(),
            decisions: value
                .decisions
                .iter()
                .map(RuntimeDecisionDto::from)
                .collect(),
            alerts: value.alerts.clone(),
            risk: value.risk.clone(),
            scenarios: Vec::new(),
            yield_benchmarks: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScenarioDto {
    pub symbol: String,
    pub expiration: String,
    pub strike_width: f64,
    pub strike_center: Option<f64>,
    pub days_to_expiry: Option<i32>,
    pub theoretical_value: f64,
    pub estimated_net_debit: f64,
    pub net_debit: f64,
    pub profit: f64,
    pub roi_pct: f64,
    pub apr_pct: f64,
    pub fill_probability: f64,
    pub scenario_type: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::SystemSnapshot;
    use chrono::Utc;

    #[test]
    fn runtime_snapshot_dto_includes_positions() {
        let mut snapshot = SystemSnapshot::default();
        snapshot.positions.push(PositionSnapshot {
            id: "pos-1".to_string(),
            symbol: "SPX".to_string(),
            quantity: 1,
            cost_basis: 100.0,
            mark: 101.0,
            unrealized_pnl: 1.0,
            account_id: Some("DU123".to_string()),
            source: Some("test".to_string()),
        });
        let dto = RuntimeSnapshotDto::from(&snapshot);
        assert_eq!(dto.positions.len(), 1);
        assert_eq!(dto.historic.len(), 0);
    }

    #[test]
    fn runtime_snapshot_dto_includes_nats_transport_when_present() {
        let mut snapshot = SystemSnapshot::default();
        snapshot.nats_transport = Some(
            NatsTransportHealthState::connected(Some("nats://127.0.0.1:4222".into()), Utc::now())
                .with_subject("system.health")
                .with_role("snapshot-publisher"),
        );
        let dto = RuntimeSnapshotDto::from(&snapshot);
        assert!(dto.nats_transport.is_some());
        let t = dto.nats_transport.unwrap();
        assert_eq!(t.role(), Some("snapshot-publisher"));
        assert_eq!(t.subject(), Some("system.health"));
    }
}
