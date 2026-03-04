//! Bidirectional conversions between hand-written domain types (in the `api`
//! crate's `state` module) and prost-generated protobuf types.
//!
//! These `From` impls let callers do `proto_pos.into()` / `state_pos.into()`
//! without touching internals.

use prost_types::Timestamp;

use crate::proto::v1 as pb;

// ---------------------------------------------------------------------------
// Timestamp helpers
// ---------------------------------------------------------------------------

fn to_timestamp(dt: &chrono::DateTime<chrono::Utc>) -> Timestamp {
    Timestamp {
        seconds: dt.timestamp(),
        nanos: dt.timestamp_subsec_nanos() as i32,
    }
}

fn from_timestamp(ts: &Timestamp) -> chrono::DateTime<chrono::Utc> {
    chrono::DateTime::from_timestamp(ts.seconds, ts.nanos.max(0) as u32).unwrap_or_default()
}

fn opt_ts(dt: &chrono::DateTime<chrono::Utc>) -> Option<Timestamp> {
    Some(to_timestamp(dt))
}

fn unwrap_ts(ts: Option<&Timestamp>) -> chrono::DateTime<chrono::Utc> {
    ts.map(from_timestamp).unwrap_or_default()
}

// ---------------------------------------------------------------------------
// Position ↔ PositionSnapshot
// ---------------------------------------------------------------------------

/// Domain representation used by the API / state layer.
/// Must mirror the fields in `api::state::PositionSnapshot`.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct PositionSnapshot {
    pub id: String,
    pub symbol: String,
    pub quantity: i32,
    pub cost_basis: f64,
    pub mark: f64,
    pub unrealized_pnl: f64,
}

impl From<PositionSnapshot> for pb::Position {
    fn from(p: PositionSnapshot) -> Self {
        pb::Position {
            id: p.id,
            symbol: p.symbol,
            quantity: p.quantity,
            cost_basis: p.cost_basis,
            mark: p.mark,
            unrealized_pnl: p.unrealized_pnl,
        }
    }
}

impl From<pb::Position> for PositionSnapshot {
    fn from(p: pb::Position) -> Self {
        Self {
            id: p.id,
            symbol: p.symbol,
            quantity: p.quantity,
            cost_basis: p.cost_basis,
            mark: p.mark,
            unrealized_pnl: p.unrealized_pnl,
        }
    }
}

// ---------------------------------------------------------------------------
// HistoricPosition
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct HistoricPosition {
    pub id: String,
    pub symbol: String,
    pub quantity: i32,
    pub realized_pnl: f64,
    pub closed_at: chrono::DateTime<chrono::Utc>,
}

impl From<HistoricPosition> for pb::HistoricPosition {
    fn from(h: HistoricPosition) -> Self {
        pb::HistoricPosition {
            id: h.id,
            symbol: h.symbol,
            quantity: h.quantity,
            realized_pnl: h.realized_pnl,
            closed_at: opt_ts(&h.closed_at),
        }
    }
}

impl From<pb::HistoricPosition> for HistoricPosition {
    fn from(h: pb::HistoricPosition) -> Self {
        Self {
            id: h.id,
            symbol: h.symbol,
            quantity: h.quantity,
            realized_pnl: h.realized_pnl,
            closed_at: unwrap_ts(h.closed_at.as_ref()),
        }
    }
}

// ---------------------------------------------------------------------------
// OrderSnapshot ↔ Order
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct OrderSnapshot {
    pub id: String,
    pub symbol: String,
    pub side: String,
    pub quantity: i32,
    pub status: String,
    pub submitted_at: chrono::DateTime<chrono::Utc>,
}

impl From<OrderSnapshot> for pb::Order {
    fn from(o: OrderSnapshot) -> Self {
        pb::Order {
            id: o.id,
            symbol: o.symbol,
            side: o.side,
            quantity: o.quantity,
            status: o.status,
            submitted_at: opt_ts(&o.submitted_at),
        }
    }
}

impl From<pb::Order> for OrderSnapshot {
    fn from(o: pb::Order) -> Self {
        Self {
            id: o.id,
            symbol: o.symbol,
            side: o.side,
            quantity: o.quantity,
            status: o.status,
            submitted_at: unwrap_ts(o.submitted_at.as_ref()),
        }
    }
}

// ---------------------------------------------------------------------------
// StrategyDecisionSnapshot ↔ StrategyDecision
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct StrategyDecisionSnapshot {
    pub symbol: String,
    pub quantity: i32,
    pub side: String,
    pub mark: f64,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl From<StrategyDecisionSnapshot> for pb::StrategyDecision {
    fn from(d: StrategyDecisionSnapshot) -> Self {
        pb::StrategyDecision {
            symbol: d.symbol,
            quantity: d.quantity,
            side: d.side,
            mark: d.mark,
            created_at: opt_ts(&d.created_at),
        }
    }
}

impl From<pb::StrategyDecision> for StrategyDecisionSnapshot {
    fn from(d: pb::StrategyDecision) -> Self {
        Self {
            symbol: d.symbol,
            quantity: d.quantity,
            side: d.side,
            mark: d.mark,
            created_at: unwrap_ts(d.created_at.as_ref()),
        }
    }
}

// ---------------------------------------------------------------------------
// Alert ↔ proto Alert
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Alert {
    pub level: String,
    pub message: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl From<Alert> for pb::Alert {
    fn from(a: Alert) -> Self {
        let level = match a.level.to_uppercase().as_str() {
            "WARNING" => pb::AlertLevel::Warning as i32,
            "ERROR" => pb::AlertLevel::Error as i32,
            _ => pb::AlertLevel::Info as i32,
        };
        pb::Alert {
            level,
            message: a.message,
            timestamp: opt_ts(&a.timestamp),
        }
    }
}

impl From<pb::Alert> for Alert {
    fn from(a: pb::Alert) -> Self {
        let level = match a.level {
            x if x == pb::AlertLevel::Warning as i32 => "WARNING",
            x if x == pb::AlertLevel::Error as i32 => "ERROR",
            _ => "INFO",
        };
        Self {
            level: level.into(),
            message: a.message,
            timestamp: unwrap_ts(a.timestamp.as_ref()),
        }
    }
}

// ---------------------------------------------------------------------------
// RiskStatus
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct RiskStatusSnapshot {
    pub allowed: bool,
    pub reason: Option<String>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl From<RiskStatusSnapshot> for pb::RiskStatus {
    fn from(r: RiskStatusSnapshot) -> Self {
        pb::RiskStatus {
            allowed: r.allowed,
            reason: r.reason.unwrap_or_default(),
            updated_at: opt_ts(&r.updated_at),
        }
    }
}

impl From<pb::RiskStatus> for RiskStatusSnapshot {
    fn from(r: pb::RiskStatus) -> Self {
        Self {
            allowed: r.allowed,
            reason: if r.reason.is_empty() {
                None
            } else {
                Some(r.reason)
            },
            updated_at: unwrap_ts(r.updated_at.as_ref()),
        }
    }
}

// ---------------------------------------------------------------------------
// Metrics
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct MetricsSnapshot {
    pub net_liq: f64,
    pub buying_power: f64,
    pub excess_liquidity: f64,
    pub margin_requirement: f64,
    pub commissions: f64,
    pub portal_ok: bool,
    pub tws_ok: bool,
    pub orats_ok: bool,
    pub questdb_ok: bool,
    pub nats_ok: bool,
}

impl From<MetricsSnapshot> for pb::Metrics {
    fn from(m: MetricsSnapshot) -> Self {
        pb::Metrics {
            net_liq: m.net_liq,
            buying_power: m.buying_power,
            excess_liquidity: m.excess_liquidity,
            margin_requirement: m.margin_requirement,
            commissions: m.commissions,
            portal_ok: m.portal_ok,
            tws_ok: m.tws_ok,
            orats_ok: m.orats_ok,
            questdb_ok: m.questdb_ok,
            nats_ok: m.nats_ok,
        }
    }
}

impl From<pb::Metrics> for MetricsSnapshot {
    fn from(m: pb::Metrics) -> Self {
        Self {
            net_liq: m.net_liq,
            buying_power: m.buying_power,
            excess_liquidity: m.excess_liquidity,
            margin_requirement: m.margin_requirement,
            commissions: m.commissions,
            portal_ok: m.portal_ok,
            tws_ok: m.tws_ok,
            orats_ok: m.orats_ok,
            questdb_ok: m.questdb_ok,
            nats_ok: m.nats_ok,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn position_round_trip() {
        let snap = PositionSnapshot {
            id: "P-1".into(),
            symbol: "SPX".into(),
            quantity: 10,
            cost_basis: 4500.0,
            mark: 4550.0,
            unrealized_pnl: 500.0,
        };
        let proto: pb::Position = snap.clone().into();
        let back: PositionSnapshot = proto.into();
        assert_eq!(snap.id, back.id);
        assert_eq!(snap.symbol, back.symbol);
        assert_eq!(snap.quantity, back.quantity);
        assert!((snap.cost_basis - back.cost_basis).abs() < f64::EPSILON);
    }

    #[test]
    fn historic_position_round_trip() {
        let hist = HistoricPosition {
            id: "H-1".into(),
            symbol: "NDX".into(),
            quantity: 5,
            realized_pnl: 250.0,
            closed_at: chrono::Utc::now(),
        };
        let proto: pb::HistoricPosition = hist.clone().into();
        let back: HistoricPosition = proto.into();
        assert_eq!(hist.id, back.id);
        assert_eq!(hist.quantity, back.quantity);
    }

    #[test]
    fn order_round_trip() {
        let order = OrderSnapshot {
            id: "ORD-1".into(),
            symbol: "SPX".into(),
            side: "BUY".into(),
            quantity: 3,
            status: "FILLED".into(),
            submitted_at: chrono::Utc::now(),
        };
        let proto: pb::Order = order.clone().into();
        let back: OrderSnapshot = proto.into();
        assert_eq!(order.id, back.id);
        assert_eq!(order.side, back.side);
    }

    #[test]
    fn alert_level_round_trip() {
        let alert = Alert {
            level: "WARNING".into(),
            message: "test alert".into(),
            timestamp: chrono::Utc::now(),
        };
        let proto: pb::Alert = alert.clone().into();
        assert_eq!(proto.level, pb::AlertLevel::Warning as i32);
        let back: Alert = proto.into();
        assert_eq!(back.level, "WARNING");
    }
}
