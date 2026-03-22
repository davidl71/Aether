//! Unified domain ↔ protobuf conversion for `SystemSnapshot`.
//!
//! Single module owning both directions: `api::SystemSnapshot` ↔ `pb::SystemSnapshot`.
//! Used by snapshot_publisher (domain→proto), REST snapshot/API handlers (domain→proto),
//! and TUI (proto→RuntimeSnapshotDto). See docs/platform/PROTOBUF_CONVERSION_AND_KV.md §4.1.

use chrono::{DateTime, TimeZone, Utc};
use nats_adapter::proto::v1 as pb;
use prost_types::Timestamp;

use crate::runtime_state::{
    RuntimeDecisionDto, RuntimeHistoricPositionDto, RuntimeOrderDto, RuntimePositionDto,
    RuntimeSnapshotDto,
};
use crate::state::{
    Alert, AlertLevel, CandleSnapshot, HistoricPosition, Metrics, OrderSnapshot, PositionSnapshot,
    RiskStatus, StrategyDecisionSnapshot, SymbolSnapshot, SystemSnapshot,
};

// ---------------------------------------------------------------------------
// Timestamp helpers
// ---------------------------------------------------------------------------

fn dt_to_ts(dt: DateTime<Utc>) -> Timestamp {
    Timestamp {
        seconds: dt.timestamp(),
        nanos: dt.timestamp_subsec_nanos() as i32,
    }
}

fn ts_to_dt(ts: Option<Timestamp>) -> DateTime<Utc> {
    ts.map(|t| {
        Utc.timestamp_opt(t.seconds, t.nanos as u32)
            .single()
            .unwrap_or_else(Utc::now)
    })
    .unwrap_or_else(Utc::now)
}

// ---------------------------------------------------------------------------
// Domain → Proto (snapshot_to_proto and helpers)
// ---------------------------------------------------------------------------

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

/// Converts in-memory `SystemSnapshot` to protobuf for NATS and REST.
pub fn snapshot_to_proto(snap: &SystemSnapshot) -> pb::SystemSnapshot {
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
        market_data_source: snap.market_data_source.clone().unwrap_or_default(),
    }
}

// ---------------------------------------------------------------------------
// Proto → Domain (system_snapshot_from_proto and runtime_snapshot_dto_from_proto)
// ---------------------------------------------------------------------------

fn position_from_proto(pos: pb::Position, fallback_account_id: &str) -> PositionSnapshot {
    PositionSnapshot {
        id: pos.id,
        symbol: pos.symbol,
        quantity: pos.quantity,
        cost_basis: pos.cost_basis,
        mark: pos.mark,
        unrealized_pnl: pos.unrealized_pnl,
        account_id: Some(fallback_account_id.to_string()),
        source: None,
    }
}

fn metrics_from_proto(m: pb::Metrics) -> Metrics {
    Metrics {
        net_liq: m.net_liq,
        buying_power: m.buying_power,
        excess_liquidity: m.excess_liquidity,
        margin_requirement: m.margin_requirement,
        commissions: m.commissions,
        portal_ok: m.portal_ok,
        tws_ok: m.tws_ok,
        tws_address: None,
        questdb_ok: m.questdb_ok,
        nats_ok: m.nats_ok,
    }
}

fn symbol_from_proto(s: pb::SymbolSnapshot) -> SymbolSnapshot {
    let candle = s.candle.map(|c| CandleSnapshot {
        open: c.open,
        high: c.high,
        low: c.low,
        close: c.close,
        volume: c.volume,
        entry: c.entry,
        updated: ts_to_dt(c.updated),
    });
    SymbolSnapshot {
        symbol: s.symbol,
        last: s.last,
        bid: s.bid,
        ask: s.ask,
        spread: s.spread,
        roi: s.roi,
        maker_count: s.maker_count,
        taker_count: s.taker_count,
        volume: s.volume,
        candle: candle.unwrap_or_else(|| CandleSnapshot {
            open: s.last,
            high: s.last,
            low: s.last,
            close: s.last,
            volume: 0,
            entry: s.last,
            updated: Utc::now(),
        }),
    }
}

fn alert_from_proto(a: pb::Alert) -> Alert {
    let level = match pb::AlertLevel::try_from(a.level).unwrap_or(pb::AlertLevel::Unspecified) {
        pb::AlertLevel::Warning => AlertLevel::Warning,
        pb::AlertLevel::Error => AlertLevel::Error,
        _ => AlertLevel::Info,
    };
    Alert {
        level,
        message: a.message,
        timestamp: ts_to_dt(a.timestamp),
    }
}

fn risk_from_proto(r: pb::RiskStatus) -> RiskStatus {
    RiskStatus {
        allowed: r.allowed,
        reason: if r.reason.is_empty() {
            None
        } else {
            Some(r.reason)
        },
        updated_at: ts_to_dt(r.updated_at),
    }
}

/// Converts protobuf `SystemSnapshot` to domain. Use when consumers need `SystemSnapshot`.
/// Note: `SystemSnapshot` does not store scenarios; they are derived on publish. This returns
/// a snapshot without scenarios (caller can derive if needed).
pub fn system_snapshot_from_proto(p: pb::SystemSnapshot) -> SystemSnapshot {
    let account_id = p.account_id.clone();
    SystemSnapshot {
        generated_at: ts_to_dt(p.generated_at),
        started_at: ts_to_dt(p.started_at),
        mode: p.mode,
        strategy: p.strategy,
        account_id: account_id.clone(),
        metrics: p.metrics.map(metrics_from_proto).unwrap_or_default(),
        symbols: p.symbols.into_iter().map(symbol_from_proto).collect(),
        positions: p
            .positions
            .into_iter()
            .map(|pos| position_from_proto(pos, &account_id))
            .collect(),
        historic: p
            .historic
            .into_iter()
            .map(|h| HistoricPosition {
                id: h.id,
                symbol: h.symbol,
                quantity: h.quantity,
                realized_pnl: h.realized_pnl,
                closed_at: ts_to_dt(h.closed_at),
            })
            .collect(),
        orders: p
            .orders
            .into_iter()
            .map(|o| OrderSnapshot {
                id: o.id,
                symbol: o.symbol,
                side: o.side,
                quantity: o.quantity,
                status: o.status,
                submitted_at: ts_to_dt(o.submitted_at),
            })
            .collect(),
        decisions: p
            .decisions
            .into_iter()
            .map(|d| {
                StrategyDecisionSnapshot::new(
                    d.symbol,
                    d.quantity,
                    d.side,
                    d.mark,
                    ts_to_dt(d.created_at),
                )
            })
            .collect(),
        alerts: p.alerts.into_iter().map(alert_from_proto).collect(),
        risk: p.risk.map(risk_from_proto).unwrap_or_default(),
        ledger: None,
        market_data_source: if p.market_data_source.is_empty() {
            None
        } else {
            Some(p.market_data_source)
        },
    }
}

/// Converts protobuf `SystemSnapshot` to `RuntimeSnapshotDto` for TUI/JSON consumers.
pub fn runtime_snapshot_dto_from_proto(p: pb::SystemSnapshot) -> RuntimeSnapshotDto {
    let account_id = p.account_id.clone();
    let positions: Vec<PositionSnapshot> = p
        .positions
        .into_iter()
        .map(|pos| position_from_proto(pos, &account_id))
        .collect();
    let historic: Vec<HistoricPosition> = p
        .historic
        .into_iter()
        .map(|h| HistoricPosition {
            id: h.id,
            symbol: h.symbol,
            quantity: h.quantity,
            realized_pnl: h.realized_pnl,
            closed_at: ts_to_dt(h.closed_at),
        })
        .collect();
    let orders: Vec<OrderSnapshot> = p
        .orders
        .into_iter()
        .map(|o| OrderSnapshot {
            id: o.id,
            symbol: o.symbol,
            side: o.side,
            quantity: o.quantity,
            status: o.status,
            submitted_at: ts_to_dt(o.submitted_at),
        })
        .collect();
    let decisions: Vec<StrategyDecisionSnapshot> = p
        .decisions
        .into_iter()
        .map(|d| {
            StrategyDecisionSnapshot::new(
                d.symbol,
                d.quantity,
                d.side,
                d.mark,
                ts_to_dt(d.created_at),
            )
        })
        .collect();
    let mut position_dtos: Vec<RuntimePositionDto> =
        positions.iter().map(RuntimePositionDto::from).collect();
    crate::combo_strategy::apply_derived_strategy_types(&mut position_dtos);

    RuntimeSnapshotDto {
        generated_at: ts_to_dt(p.generated_at),
        started_at: ts_to_dt(p.started_at),
        mode: p.mode,
        strategy: p.strategy,
        account_id: p.account_id,
        metrics: p.metrics.map(metrics_from_proto).unwrap_or_default(),
        symbols: p.symbols.into_iter().map(symbol_from_proto).collect(),
        positions: position_dtos,
        historic: historic
            .iter()
            .map(RuntimeHistoricPositionDto::from)
            .collect(),
        orders: orders.iter().map(RuntimeOrderDto::from).collect(),
        decisions: decisions.iter().map(RuntimeDecisionDto::from).collect(),
        alerts: p.alerts.into_iter().map(alert_from_proto).collect(),
        risk: p.risk.map(risk_from_proto).unwrap_or_default(),
        scenarios: Vec::new(),
        yield_benchmarks: None,
    }
}
