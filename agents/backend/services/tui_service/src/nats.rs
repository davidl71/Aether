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

use api::{backend_health_from_message, BackendHealthState};
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
                // Spawn health subscriber on same connection (drops when connection drops)
                let client_health = client.clone();
                let health_tx = health_tx.clone();
                tokio::spawn(async move {
                    if let Err(e) = run_health_subscriber(client_health, health_tx).await {
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
) -> anyhow::Result<()> {
    let subject = topics::system::health();
    let mut sub = client.client().subscribe(subject.to_string()).await?;
    let mut backends: HashMap<String, BackendHealthState> = HashMap::new();

    while let Some(msg) = sub.next().await {
        if let Some(health) = backend_health_from_message(&msg.payload) {
            let state = BackendHealthState::from_proto(health);
            let id = state.backend.clone();
            backends.insert(id, state);
            let _ = tx.send(backends.clone());
        }
    }

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

/// Run a tick subscriber on `market-data.>` using an existing NATS client connection.
/// Decodes `pb::MarketDataEvent` and emits `AppEvent::MarketTick` for each tick.
pub async fn run_tick_subscriber(
    client: NatsClient,
    event_tx: mpsc::UnboundedSender<AppEvent>,
) -> anyhow::Result<()> {
    let subject = "market-data.>";
    let mut sub = client.client().subscribe(subject.to_string()).await?;
    info!(subject = %subject, "Subscribed to market-data tick wildcard");

    while let Some(msg) = sub.next().await {
        match extract_proto_payload::<pb::MarketDataEvent>(&msg.payload) {
            Ok(proto) => {
                let tick = AppEvent::MarketTick {
                    symbol: proto.symbol,
                    bid: proto.bid,
                    ask: proto.ask,
                    last: proto.last,
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
