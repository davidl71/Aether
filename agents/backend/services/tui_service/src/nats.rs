//! NATS subscriber task.
//!
//! Subscribes to `snapshot.{backend_id}`, decodes protobuf envelopes,
//! converts to `RuntimeSnapshotDto`, and sends updates on a
//! `tokio::sync::watch` channel for the main event loop to consume.
//!
//! Uses a circuit breaker to avoid hammering a downed NATS server:
//! - After 3 consecutive failures the circuit opens for 30s
//! - Reconnect delays grow exponentially: 2s, 4s, 8s … 60s max

use api::{
    Alert, AlertLevel, CandleSnapshot, HistoricPosition, Metrics, OrderSnapshot, PositionSnapshot,
    RiskStatus, RuntimeDecisionDto, RuntimeHistoricPositionDto, RuntimeOrderDto,
    RuntimePositionDto, RuntimeSnapshotDto, StrategyDecisionSnapshot, SymbolSnapshot,
};
use chrono::{DateTime, TimeZone, Utc};
use futures::StreamExt;
use nats_adapter::{extract_proto_payload, proto::v1 as pb, topics, NatsClient};
use prost_types::Timestamp;
use tokio::sync::{mpsc, watch};
use tracing::{debug, info, warn};

use crate::circuit_breaker::CircuitBreaker;
use crate::config::TuiConfig;
use crate::events::{
    AppEvent, ConnectionState, ConnectionStatus, ConnectionTarget, LogEntry, LogLevel,
};
use crate::models::{SnapshotSource, TuiSnapshot};

fn ts_to_dt(ts: Option<Timestamp>) -> DateTime<Utc> {
    ts.map(|t| {
        Utc.timestamp_opt(t.seconds, t.nanos as u32)
            .single()
            .unwrap_or_else(Utc::now)
    })
    .unwrap_or_else(Utc::now)
}

fn proto_to_snapshot(p: pb::SystemSnapshot) -> RuntimeSnapshotDto {
    let positions: Vec<PositionSnapshot> = p
        .positions
        .into_iter()
        .map(|pos| PositionSnapshot {
            id: pos.id,
            symbol: pos.symbol,
            quantity: pos.quantity,
            cost_basis: pos.cost_basis,
            mark: pos.mark,
            unrealized_pnl: pos.unrealized_pnl,
        })
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

    RuntimeSnapshotDto {
        generated_at: ts_to_dt(p.generated_at),
        started_at: ts_to_dt(p.started_at),
        mode: p.mode,
        strategy: p.strategy,
        account_id: p.account_id,
        metrics: p.metrics.map(proto_metrics).unwrap_or_default(),
        symbols: p.symbols.into_iter().map(proto_symbol).collect(),
        positions: positions.iter().map(RuntimePositionDto::from).collect(),
        historic: historic
            .iter()
            .map(RuntimeHistoricPositionDto::from)
            .collect(),
        orders: orders.iter().map(RuntimeOrderDto::from).collect(),
        decisions: decisions.iter().map(RuntimeDecisionDto::from).collect(),
        alerts: p.alerts.into_iter().map(proto_alert).collect(),
        risk: p.risk.map(proto_risk).unwrap_or_default(),
    }
}

fn proto_metrics(m: pb::Metrics) -> Metrics {
    Metrics {
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

fn proto_symbol(s: pb::SymbolSnapshot) -> SymbolSnapshot {
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
        symbol: s.symbol.clone(),
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

fn proto_alert(a: pb::Alert) -> Alert {
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

fn proto_risk(r: pb::RiskStatus) -> RiskStatus {
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

/// Run the NATS subscriber loop. Sends `TuiSnapshot` updates on `tx`.
///
/// Reconnects automatically with exponential backoff (2s → 60s max).
/// A circuit breaker opens after 3 consecutive failures and pauses
/// all attempts for 30s before entering half-open test mode.
pub async fn run(
    config: TuiConfig,
    tx: watch::Sender<Option<TuiSnapshot>>,
    event_tx: mpsc::UnboundedSender<AppEvent>,
) {
    let subject = topics::snapshot::backend(&config.backend_id);
    info!(subject = %subject, nats_url = %config.nats_url, "NATS subscriber starting");
    emit_log(
        &event_tx,
        LogLevel::Info,
        format!("NATS subscriber starting for {subject}"),
    );
    emit_status(
        &event_tx,
        ConnectionState::Starting,
        format!("Connecting to {}", config.nats_url),
    );

    let mut cb = CircuitBreaker::new();

    loop {
        if !cb.can_attempt() {
            // Circuit is open — wait 1s and re-check (avoids busy-spinning)
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            continue;
        }

        match NatsClient::connect(&config.nats_url).await {
            Ok(client) => {
                cb.record_success();
                info!("NATS connected");
                emit_status(
                    &event_tx,
                    ConnectionState::Connected,
                    format!("Connected to {}", config.nats_url),
                );
                emit_log(&event_tx, LogLevel::Info, "NATS connected");
                if let Err(e) = subscribe_loop(&client, &subject, &tx, &event_tx).await {
                    cb.record_failure();
                    let delay = cb.backoff();
                    warn!(
                        error = %e,
                        delay_secs = delay.as_secs(),
                        "NATS subscriber loop exited, reconnecting"
                    );
                    emit_status(&event_tx, ConnectionState::Retrying, e.to_string());
                    emit_log(
                        &event_tx,
                        LogLevel::Warn,
                        format!("NATS subscription lost: {e}"),
                    );
                    tokio::time::sleep(delay).await;
                }
            }
            Err(e) => {
                cb.record_failure();
                let delay = cb.backoff();
                let open_msg = if cb.is_open() {
                    " (circuit open, pausing 30s)".to_string()
                } else {
                    format!(", retrying in {}s", delay.as_secs())
                };
                warn!(error = %e, "NATS connect failed{}", open_msg);
                emit_status(&event_tx, ConnectionState::Retrying, e.to_string());
                emit_log(
                    &event_tx,
                    LogLevel::Warn,
                    format!("NATS connect failed: {e}{open_msg}"),
                );
                if !cb.is_open() {
                    tokio::time::sleep(delay).await;
                }
            }
        }
    }
}

async fn subscribe_loop(
    client: &NatsClient,
    subject: &str,
    tx: &watch::Sender<Option<TuiSnapshot>>,
    event_tx: &mpsc::UnboundedSender<AppEvent>,
) -> anyhow::Result<()> {
    let mut sub = client.client().subscribe(subject.to_string()).await?;
    info!(subject = %subject, "Subscribed to snapshot subject");
    emit_log(event_tx, LogLevel::Info, format!("Subscribed to {subject}"));

    while let Some(msg) = sub.next().await {
        match extract_proto_payload::<pb::SystemSnapshot>(&msg.payload) {
            Ok(proto) => {
                let dto = proto_to_snapshot(proto);
                let snap = TuiSnapshot::new(dto, SnapshotSource::Nats);
                debug!(subject = %subject, "Snapshot received");
                let _ = tx.send(Some(snap));
            }
            Err(e) => {
                warn!(error = %e, "Failed to decode snapshot payload");
                emit_log(
                    event_tx,
                    LogLevel::Warn,
                    format!("Failed to decode NATS snapshot: {e}"),
                );
            }
        }
    }

    anyhow::bail!("NATS subscription ended");
}

fn emit_status(
    event_tx: &mpsc::UnboundedSender<AppEvent>,
    state: ConnectionState,
    detail: impl Into<String>,
) {
    let _ = event_tx.send(AppEvent::Connection {
        target: ConnectionTarget::Nats,
        status: ConnectionStatus::new(state, detail),
    });
}

fn emit_log(
    event_tx: &mpsc::UnboundedSender<AppEvent>,
    level: LogLevel,
    message: impl Into<String>,
) {
    let _ = event_tx.send(AppEvent::Log(LogEntry::new(
        level,
        Some(ConnectionTarget::Nats),
        message,
    )));
}
