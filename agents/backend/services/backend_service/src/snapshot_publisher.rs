//! Periodic NATS snapshot publisher.
//!
//! Reads the shared `SystemSnapshot` every `interval_ms` milliseconds and
//! publishes it as a protobuf `NatsEnvelope(SystemSnapshot)` on
//! `snapshot.{backend_id}` so subscribers (e.g. tui_service) receive live
//! full-state updates without polling REST.

use std::sync::Arc;

use api::{
    Alert, AlertLevel, CandleSnapshot, HistoricPosition, Metrics, OrderSnapshot, PositionSnapshot,
    RiskStatus, StrategyDecisionSnapshot, SymbolSnapshot, SystemSnapshot,
};
use nats_adapter::{encode_envelope, proto::v1 as pb, topics, NatsClient};
use tokio::{sync::RwLock, time};
use tracing::{info, warn};

fn dt_to_ts(dt: chrono::DateTime<chrono::Utc>) -> prost_types::Timestamp {
    prost_types::Timestamp {
        seconds: dt.timestamp(),
        nanos: dt.timestamp_subsec_nanos() as i32,
    }
}

fn candle_to_proto(c: &CandleSnapshot) -> pb::CandleSnapshot {
    pb::CandleSnapshot {
        open: c.open,
        high: c.high,
        low: c.low,
        close: c.close,
        volume: c.volume,
        entry: c.entry,
        updated: Some(dt_to_ts(c.updated)),
    }
}

fn symbol_to_proto(s: &SymbolSnapshot) -> pb::SymbolSnapshot {
    pb::SymbolSnapshot {
        symbol: s.symbol.clone(),
        last: s.last,
        bid: s.bid,
        ask: s.ask,
        spread: s.spread,
        roi: s.roi,
        maker_count: s.maker_count,
        taker_count: s.taker_count,
        volume: s.volume,
        candle: Some(candle_to_proto(&s.candle)),
    }
}

fn position_to_proto(p: &PositionSnapshot) -> pb::Position {
    pb::Position {
        id: p.id.clone(),
        symbol: p.symbol.clone(),
        quantity: p.quantity,
        cost_basis: p.cost_basis,
        mark: p.mark,
        unrealized_pnl: p.unrealized_pnl,
    }
}

fn historic_to_proto(h: &HistoricPosition) -> pb::HistoricPosition {
    pb::HistoricPosition {
        id: h.id.clone(),
        symbol: h.symbol.clone(),
        quantity: h.quantity,
        realized_pnl: h.realized_pnl,
        closed_at: Some(dt_to_ts(h.closed_at)),
    }
}

fn order_to_proto(o: &OrderSnapshot) -> pb::Order {
    pb::Order {
        id: o.id.clone(),
        symbol: o.symbol.clone(),
        side: o.side.clone(),
        quantity: o.quantity,
        status: o.status.clone(),
        submitted_at: Some(dt_to_ts(o.submitted_at)),
    }
}

fn decision_to_proto(d: &StrategyDecisionSnapshot) -> pb::StrategyDecision {
    pb::StrategyDecision {
        symbol: d.symbol.clone(),
        quantity: d.quantity,
        side: d.side.clone(),
        mark: d.mark,
        created_at: Some(dt_to_ts(d.created_at)),
    }
}

fn alert_level_to_proto(level: &AlertLevel) -> i32 {
    match level {
        AlertLevel::Info => pb::AlertLevel::Info as i32,
        AlertLevel::Warning => pb::AlertLevel::Warning as i32,
        AlertLevel::Error => pb::AlertLevel::Error as i32,
    }
}

fn alert_to_proto(a: &Alert) -> pb::Alert {
    pb::Alert {
        level: alert_level_to_proto(&a.level),
        message: a.message.clone(),
        timestamp: Some(dt_to_ts(a.timestamp)),
    }
}

fn metrics_to_proto(m: &Metrics) -> pb::Metrics {
    pb::Metrics {
        net_liq: m.net_liq,
        buying_power: m.buying_power,
        excess_liquidity: m.excess_liquidity,
        margin_requirement: m.margin_requirement,
        commissions: m.commissions,
        portal_ok: m.portal_ok,
        tws_ok: m.tws_ok,
        questdb_ok: m.questdb_ok,
        nats_ok: m.nats_ok,
    }
}

fn risk_to_proto(r: &RiskStatus) -> pb::RiskStatus {
    pb::RiskStatus {
        allowed: r.allowed,
        reason: r.reason.clone().unwrap_or_default(),
        updated_at: Some(dt_to_ts(r.updated_at)),
    }
}

fn snapshot_to_proto(snap: &SystemSnapshot) -> pb::SystemSnapshot {
    pb::SystemSnapshot {
        generated_at: Some(dt_to_ts(snap.generated_at)),
        started_at: Some(dt_to_ts(snap.started_at)),
        mode: snap.mode.clone(),
        strategy: snap.strategy.clone(),
        account_id: snap.account_id.clone(),
        metrics: Some(metrics_to_proto(&snap.metrics)),
        symbols: snap.symbols.iter().map(symbol_to_proto).collect(),
        positions: snap.positions.iter().map(position_to_proto).collect(),
        historic: snap.historic.iter().map(historic_to_proto).collect(),
        orders: snap.orders.iter().map(order_to_proto).collect(),
        decisions: snap.decisions.iter().map(decision_to_proto).collect(),
        alerts: snap.alerts.iter().map(alert_to_proto).collect(),
        risk: Some(risk_to_proto(&snap.risk)),
    }
}

/// Spawn the periodic snapshot publisher as a background tokio task.
pub fn spawn(
    state: Arc<RwLock<SystemSnapshot>>,
    client: Arc<NatsClient>,
    backend_id: String,
    interval_ms: u64,
) {
    tokio::spawn(run(state, client, backend_id, interval_ms));
}

async fn run(
    state: Arc<RwLock<SystemSnapshot>>,
    client: Arc<NatsClient>,
    backend_id: String,
    interval_ms: u64,
) {
    let subject = topics::snapshot::backend(&backend_id);
    let mut ticker = time::interval(time::Duration::from_millis(interval_ms));
    info!(subject = %subject, interval_ms, "Snapshot publisher started");

    loop {
        ticker.tick().await;

        let proto = {
            let snap = state.read().await;
            snapshot_to_proto(&snap)
        };

        match encode_envelope("backend_service", "SystemSnapshot", &proto) {
            Ok(bytes) => {
                if let Err(e) = client.client().publish(subject.clone(), bytes).await {
                    warn!(error = %e, subject = %subject, "Failed to publish snapshot to NATS");
                }
            }
            Err(e) => warn!(error = %e, "Failed to encode snapshot for NATS"),
        }
    }
}
