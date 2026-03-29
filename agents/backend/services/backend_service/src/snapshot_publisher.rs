//! Periodic NATS snapshot publisher.
//!
//! Reads the shared `SystemSnapshot` every `interval_ms` milliseconds and
//! publishes it as a protobuf `NatsEnvelope(SystemSnapshot)` on
//! `snapshot.{backend_id}` so subscribers (e.g. tui_service) receive live
//! full-state updates without polling REST.
//!
//! When `use_jetstream` is true (NATS_USE_JETSTREAM=1), snapshots are also
//! published to a JetStream stream "SNAPSHOTS" for replay (late-joining
//! clients can fetch the latest snapshot from the stream).

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use api::snapshot_proto::snapshot_to_proto;
use api::{NatsTransportHealthState, SystemSnapshot};
use chrono::Utc;
use nats_adapter::{async_nats, encode_envelope, topics, NatsClient};
use tokio::{sync::RwLock, time};
use tracing::{info, warn};

const SNAPSHOT_STREAM_NAME: &str = "SNAPSHOTS";
const SNAPSHOT_STREAM_SUBJECTS: &[&str] = &["snapshot.>"];
const SNAPSHOT_STREAM_MAX_AGE_SECS: u64 = 3600; // 1 hour

/// Spawn the periodic snapshot publisher as a background tokio task.
/// When `use_jetstream` is true, ensures JetStream stream "SNAPSHOTS" exists and publishes via JetStream for replay.
pub fn spawn(
    state: Arc<RwLock<SystemSnapshot>>,
    client: Arc<NatsClient>,
    backend_id: String,
    interval_ms: u64,
    use_jetstream: bool,
) {
    tokio::spawn(run(state, client, backend_id, interval_ms, use_jetstream));
}

async fn run(
    state: Arc<RwLock<SystemSnapshot>>,
    client: Arc<NatsClient>,
    backend_id: String,
    interval_ms: u64,
    use_jetstream: bool,
) {
    let subject = topics::snapshot::backend(&backend_id);
    let mut ticker = time::interval(time::Duration::from_millis(interval_ms));
    let publish_errors = Arc::new(AtomicU64::new(0));

    let js_ctx = if use_jetstream {
        match ensure_snapshot_stream(client.clone()).await {
            Ok(()) => {
                info!(subject = %subject, interval_ms, stream = SNAPSHOT_STREAM_NAME, "Snapshot publisher started (JetStream replay enabled)");
                Some(async_nats::jetstream::new(client.client().clone()))
            }
            Err(e) => {
                warn!(error = %e, "JetStream stream not available, snapshot will use core publish only");
                None
            }
        }
    } else {
        info!(subject = %subject, interval_ms, "Snapshot publisher started");
        None
    };

    loop {
        ticker.tick().await;

        let (flush_rtt_ms, flush_err) = {
            let start = Instant::now();
            match client.flush().await {
                Ok(()) => (Some(start.elapsed().as_millis() as u64), None),
                Err(e) => {
                    publish_errors.fetch_add(1, Ordering::Relaxed);
                    warn!(error = %e, "NATS flush failed (snapshot publisher)");
                    (None, Some(e.to_string()))
                }
            }
        };

        {
            let mut snap = state.write().await;
            snap.touch();
            let url = Some(client.url().to_string());
            let mut transport = if let Some(rtt) = flush_rtt_ms {
                let mut t = NatsTransportHealthState::connected(url, Utc::now())
                    .with_subject(subject.clone())
                    .with_role("snapshot-publisher");
                t.last_rtt_ms = Some(rtt);
                t
            } else {
                NatsTransportHealthState::disconnected(
                    url,
                    Utc::now(),
                    flush_err.clone(),
                    Some("NATS flush failed".to_string()),
                )
                .with_subject(subject.clone())
                .with_role("snapshot-publisher")
            };
            transport.extra.insert(
                "publish_errors".to_string(),
                publish_errors.load(Ordering::Relaxed).to_string(),
            );
            snap.nats_transport = Some(transport);
        }

        let proto = {
            let snap = state.read().await;
            snapshot_to_proto(&snap)
        };

        match encode_envelope("backend_service", "SystemSnapshot", &proto) {
            Ok(bytes) => {
                if let Some(ref js) = js_ctx {
                    if let Err(e) = js.publish(subject.clone(), bytes.clone()).await {
                        publish_errors.fetch_add(1, Ordering::Relaxed);
                        warn!(error = %e, subject = %subject, "Failed to publish snapshot to JetStream");
                    }
                } else if let Err(e) = client.client().publish(subject.clone(), bytes).await {
                    publish_errors.fetch_add(1, Ordering::Relaxed);
                    warn!(error = %e, subject = %subject, "Failed to publish snapshot to NATS");
                }
            }
            Err(e) => warn!(error = %e, "Failed to encode snapshot for NATS"),
        }
    }
}

async fn ensure_snapshot_stream(
    client: Arc<NatsClient>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let js = async_nats::jetstream::new(client.client().clone());
    let stream_config = async_nats::jetstream::stream::Config {
        name: SNAPSHOT_STREAM_NAME.to_string(),
        subjects: SNAPSHOT_STREAM_SUBJECTS
            .iter()
            .map(|s| (*s).to_string())
            .collect(),
        retention: async_nats::jetstream::stream::RetentionPolicy::Limits,
        max_messages_per_subject: 1,
        max_age: Duration::from_secs(SNAPSHOT_STREAM_MAX_AGE_SECS),
        ..Default::default()
    };
    js.get_or_create_stream(stream_config).await?;
    Ok(())
}
