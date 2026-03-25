//! NATS subscriber task.
//!
//! Subscribes to `snapshot.{backend_id}`, decodes protobuf envelopes to
//! `api::SystemSnapshot` via `api::snapshot_proto::system_snapshot_from_proto`,
//! and passes the unified snapshot to the TUI (TuiSnapshot stores SystemSnapshot
//! and derives RuntimeSnapshotDto for display). See
//! docs/platform/PROTOBUF_CONVERSION_AND_KV.md §4.2.
//!
//! Uses a circuit breaker to avoid hammering a downed NATS server:
//! - After 3 consecutive failures the circuit opens for 30s
//! - Reconnect delays grow exponentially: 2s, 4s, 8s … 60s max
//!
//! **Why the TUI can show "NATS: DOWN" when NATS is up:**
//! 1. **Circuit open** — After 3 failures we stop trying for 30s; status shows
//!    "Circuit open, retrying in 30s". Restarting NATS during this window still shows DOWN.
//! 2. **Connect failed** — Wrong NATS_URL, port, or network (e.g. TUI in another network).
//!    The status detail shows the error (e.g. "Connection refused").
//! 3. **Subscription ended** — We connected but the subscription stream closed (server
//!    disconnect, idle timeout). Detail shows "NATS subscription ended".

use api::{
    backend_health_from_message, BackendHealthState, CommandReply, NatsTransportHealthState,
};
use chrono::Utc;
use futures::StreamExt;
use nats_adapter::{extract_proto_payload, proto::v1 as pb, topics, NatsClient};
use std::collections::HashMap;
use tokio::sync::{mpsc, watch};
use tracing::{debug, info, warn};

use crate::circuit_breaker::CircuitBreaker;
use crate::config::TuiConfig;
use crate::events::{AppEvent, ConnectionState, ConnectionStatus, ConnectionTarget};
use crate::models::{SnapshotSource, TuiSnapshot};

/// Run the NATS subscriber loop. Sends `TuiSnapshot` updates on `tx` and
/// backend health updates on `health_tx` (from `system.health`).
///
/// Reconnects automatically with exponential backoff (2s → 60s max).
/// A circuit breaker opens after 3 consecutive failures and pauses
/// all attempts for 30s before entering half-open test mode.
pub async fn run(
    config: TuiConfig,
    tx: watch::Sender<Option<TuiSnapshot>>,
    event_tx: mpsc::UnboundedSender<AppEvent>,
    health_tx: watch::Sender<HashMap<String, BackendHealthState>>,
) {
    let subject = topics::snapshot::backend(&config.backend_id);
    info!(subject = %subject, nats_url = %config.nats_url, "NATS subscriber starting");
    emit_status(
        &event_tx,
        ConnectionState::Starting,
        format!("Connecting to {}", config.nats_url),
    );

    let mut cb = CircuitBreaker::new();

    loop {
        if !cb.can_attempt() {
            // Circuit is open — don't hammer NATS; emit status so UI shows we're in cooldown
            emit_status(
                &event_tx,
                ConnectionState::Retrying,
                "Circuit open, retrying in 30s",
            );
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
                emit_transport_health(
                    &event_tx,
                    NatsTransportHealthState::connected(Some(config.nats_url.clone()), Utc::now())
                        .with_subject(&subject)
                        .with_role("snapshot-subscriber"),
                );
                // Spawn health subscriber on same connection (drops when connection drops)
                let client_health = client.clone();
                let health_tx = health_tx.clone();
                let health_event_tx = event_tx.clone();
                tokio::spawn(async move {
                    if let Err(e) =
                        run_health_subscriber(client_health, health_tx, health_event_tx).await
                    {
                        warn!(error = %e, "Health subscriber ended");
                    }
                });
                if let Err(e) = subscribe_loop(&client, &subject, &tx).await {
                    cb.record_failure();
                    let delay = cb.backoff();
                    warn!(
                        error = %e,
                        delay_secs = delay.as_secs(),
                        "NATS subscription lost, reconnecting"
                    );
                    emit_status(&event_tx, ConnectionState::Retrying, e.to_string());
                    emit_transport_health(
                        &event_tx,
                        NatsTransportHealthState::disconnected(
                            Some(config.nats_url.clone()),
                            Utc::now(),
                            Some(e.to_string()),
                            Some("snapshot subscription lost".to_string()),
                        )
                        .with_subject(&subject)
                        .with_role("snapshot-subscriber"),
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
                emit_transport_health(
                    &event_tx,
                    NatsTransportHealthState::disconnected(
                        Some(config.nats_url.clone()),
                        Utc::now(),
                        Some(e.to_string()),
                        Some("failed to connect snapshot subscriber to NATS".to_string()),
                    )
                    .with_subject(&subject)
                    .with_role("snapshot-subscriber"),
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
) -> anyhow::Result<()> {
    let mut sub = client.client().subscribe(subject.to_string()).await?;
    info!(subject = %subject, "Subscribed to snapshot subject");

    while let Some(msg) = sub.next().await {
        match extract_proto_payload::<pb::SystemSnapshot>(&msg.payload) {
            Ok(proto) => {
                let snap = api::snapshot_proto::system_snapshot_from_proto(proto);
                let tui_snap = TuiSnapshot::new(snap, SnapshotSource::Nats);
                debug!(subject = %subject, "Snapshot received");
                let _ = tx.send(Some(tui_snap));
            }
            Err(e) => {
                warn!(error = %e, "Failed to decode NATS snapshot payload");
            }
        }
    }

    anyhow::bail!("NATS subscription ended");
}

/// Subscribes to `system.health`, decodes BackendHealth messages, and sends
/// the current map of backend id → health state on each update.
async fn run_health_subscriber(
    client: NatsClient,
    tx: watch::Sender<HashMap<String, BackendHealthState>>,
    event_tx: mpsc::UnboundedSender<AppEvent>,
) -> anyhow::Result<()> {
    let subject = topics::system::health();
    let mut sub = client.client().subscribe(subject.to_string()).await?;
    let mut backends: HashMap<String, BackendHealthState> = HashMap::new();

    while let Some(msg) = sub.next().await {
        emit_transport_health(
            &event_tx,
            NatsTransportHealthState::connected(None, Utc::now())
                .with_subject(subject)
                .with_role("health-subscriber"),
        );
        if let Some(health) = backend_health_from_message(&msg.payload) {
            let state = BackendHealthState::from_proto(health);
            let id = state.backend.clone();
            backends.insert(id, state);
            let _ = tx.send(backends.clone());
        }
    }

    emit_transport_health(
        &event_tx,
        NatsTransportHealthState::disconnected(
            None,
            Utc::now(),
            None,
            Some("system.health subscription ended".to_string()),
        )
        .with_subject(subject)
        .with_role("health-subscriber"),
    );

    anyhow::bail!("Health subscription ended");
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

fn emit_transport_health(
    event_tx: &mpsc::UnboundedSender<AppEvent>,
    state: NatsTransportHealthState,
) {
    let _ = event_tx.send(AppEvent::TransportHealth(state));
}

/// Run a tick subscriber on `market-data.tick.>` using an existing NATS client connection.
/// Decodes `pb::MarketDataEvent` and emits `AppEvent::MarketTick` for each tick.
pub async fn run_tick_subscriber(
    client: NatsClient,
    event_tx: mpsc::UnboundedSender<AppEvent>,
) -> anyhow::Result<()> {
    let subject = topics::market_data::tick(">");
    let mut sub = client.client().subscribe(subject.clone()).await?;
    info!(subject = %subject, "Subscribed to market-data tick wildcard");

    while let Some(msg) = sub.next().await {
        match extract_proto_payload::<pb::MarketDataEvent>(&msg.payload) {
            Ok(proto) => {
                let tick = AppEvent::MarketTick {
                    symbol: proto.symbol,
                    bid: proto.bid,
                    ask: proto.ask,
                    last: proto.last,
                    source: proto.source,
                    source_priority: proto.source_priority,
                };
                let _ = event_tx.send(tick);
            }
            Err(e) => {
                debug!(error = %e, "Failed to decode market-data tick payload");
            }
        }
    }

    anyhow::bail!("Tick subscription ended");
}

/// Run a candle subscriber on `market-data.candle.>` and emit `AppEvent::MarketCandle`.
pub async fn run_candle_subscriber(
    client: NatsClient,
    event_tx: mpsc::UnboundedSender<AppEvent>,
) -> anyhow::Result<()> {
    let subject = topics::market_data::candle(">");
    let mut sub = client.client().subscribe(subject.clone()).await?;
    info!(subject = %subject, "Subscribed to market-data candle wildcard");

    while let Some(msg) = sub.next().await {
        let symbol = msg
            .subject
            .split('.')
            .next_back()
            .unwrap_or_default()
            .to_string();

        match extract_proto_payload::<pb::CandleSnapshot>(&msg.payload) {
            Ok(proto) => {
                let candle = AppEvent::MarketCandle {
                    symbol,
                    open: proto.open,
                    high: proto.high,
                    low: proto.low,
                    close: proto.close,
                    volume: proto.volume,
                };
                let _ = event_tx.send(candle);
            }
            Err(e) => {
                debug!(error = %e, "Failed to decode market-data candle payload");
            }
        }
    }

    anyhow::bail!("Candle subscription ended");
}

/// Run a command-event subscriber on `system.commands.>` and emit `AppEvent::CommandStatus`.
///
/// This is best-effort: if backend-side command events are not published yet,
/// the subscription simply stays idle and the existing request/reply path
/// continues to drive the status area.
pub async fn run_command_subscriber(
    client: NatsClient,
    event_tx: mpsc::UnboundedSender<AppEvent>,
) -> anyhow::Result<()> {
    let subject = "system.commands.>";
    let mut sub = client.client().subscribe(subject.to_string()).await?;
    info!(subject = %subject, "Subscribed to system command events");

    while let Some(msg) = sub.next().await {
        match serde_json::from_slice::<CommandReply>(&msg.payload) {
            Ok(reply) => {
                let _ = event_tx.send(AppEvent::CommandStatus(reply));
            }
            Err(e) => {
                debug!(error = %e, "Failed to decode system command event payload");
            }
        }
    }

    anyhow::bail!("Command-event subscription ended");
}

/// Watch the NATS KV bucket for yield curve updates.
///
/// Watches `yield_curve.*` keys in `bucket`. On each `Put`, decodes the
/// protobuf payload and emits `AppEvent::YieldCurveKvUpdate`. Errors are
/// logged and the watcher is restarted after a short delay.
pub async fn run_yield_kv_watcher(
    client: NatsClient,
    bucket: String,
    event_tx: mpsc::UnboundedSender<AppEvent>,
) -> anyhow::Result<()> {
    use futures::StreamExt;
    use nats_adapter::async_nats::jetstream;
    use nats_adapter::async_nats::jetstream::kv::Operation;

    let js = jetstream::new(client.client().clone());
    let kv: jetstream::kv::Store = match js.get_key_value(&bucket).await {
        Ok(k) => k,
        Err(e) => {
            anyhow::bail!("yield KV bucket {bucket} not found: {e}");
        }
    };

    let mut watcher = kv.watch("yield_curve.*").await?;
    info!(%bucket, "yield KV watcher started (yield_curve.*)");

    while let Some(entry_res) = watcher.next().await {
        let entry = match entry_res {
            Ok(e) => e,
            Err(e) => {
                warn!(error = %e, "yield KV watcher entry error");
                continue;
            }
        };

        // Only process Put (new value); ignore Delete/Purge
        if entry.operation != Operation::Put {
            continue;
        }

        // Key is "yield_curve.{symbol}"
        let symbol = match entry.key.strip_prefix("yield_curve.") {
            Some(s) if !s.is_empty() => s.to_string(),
            _ => continue,
        };

        let bytes = entry.value.as_ref().to_vec();
        let curve = match api::yield_curve_proto::curve_response_from_proto_bytes(&bytes, &symbol) {
            Some(c) if !c.points.is_empty() => c,
            _ => {
                debug!(key = %entry.key, "yield KV entry empty or not decodable, skipping");
                continue;
            }
        };

        let fetched_at = curve.timestamp.clone();
        debug!(symbol = %symbol, points = curve.point_count, "yield KV update received");
        let _ = event_tx.send(AppEvent::YieldCurveKvUpdate {
            symbol,
            curve,
            fetched_at,
        });
    }

    anyhow::bail!("yield KV watcher stream ended");
}

/// Publish a one-shot refresh request to `api.yield_curve.refresh`.
/// The yield_curve_writer backend picks this up and runs a write cycle immediately,
/// which will trigger another KV update that the watcher will receive.
pub async fn send_yield_refresh(client: &NatsClient) -> anyhow::Result<()> {
    client
        .client()
        .publish("api.yield_curve.refresh", "{}".into())
        .await
        .map_err(|e| anyhow::anyhow!("yield refresh publish failed: {e}"))
}

pub async fn run_alert_subscriber(
    client: NatsClient,
    event_tx: mpsc::UnboundedSender<AppEvent>,
) -> anyhow::Result<()> {
    let subject = topics::system::alerts();
    let mut sub = client.client().subscribe(subject.to_string()).await?;
    info!(subject = %subject, "Subscribed to system alerts");

    while let Some(msg) = sub.next().await {
        match extract_proto_payload::<pb::Alert>(&msg.payload) {
            Ok(proto) => {
                let level = match pb::AlertLevel::try_from(proto.level)
                    .unwrap_or(pb::AlertLevel::Unspecified)
                {
                    pb::AlertLevel::Warning => api::AlertLevel::Warning,
                    pb::AlertLevel::Error => api::AlertLevel::Error,
                    _ => api::AlertLevel::Info,
                };
                let timestamp = proto
                    .timestamp
                    .and_then(|ts| {
                        chrono::DateTime::<Utc>::from_timestamp(ts.seconds, ts.nanos as u32)
                    })
                    .unwrap_or_else(Utc::now);
                let _ = event_tx.send(AppEvent::AlertReceived {
                    level,
                    message: proto.message,
                    timestamp,
                });
            }
            Err(e) => {
                debug!(error = %e, "Failed to decode system alert payload");
            }
        }
    }

    anyhow::bail!("Alert subscription ended");
}
