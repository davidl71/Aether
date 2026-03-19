//! Publish this backend's health to `system.health` so aggregators (backend_service
//! health_aggregation, TUI Settings tab) can show "backends" from system.health.
//!
//! One message per backend; each backend instance publishes its own BackendHealth
//! periodically. Subscribers merge by backend id.

use std::sync::Arc;
use std::time::Duration;

use nats_adapter::{encode_envelope, proto::v1 as pb, topics, NatsClient};
use tokio::time;
use tracing::warn;

/// Spawn a task that publishes this backend's BackendHealth to `system.health` every `interval_secs`.
/// No-op if `interval_secs` is 0. Uses `backend_id` (e.g. from BACKEND_ID env) and status "ok".
pub fn spawn(client: Arc<NatsClient>, backend_id: String, interval_secs: u64) {
    if interval_secs == 0 {
        return;
    }

    let subject = topics::system::health().to_string();
    tokio::spawn(async move {
        let mut ticker = time::interval(Duration::from_secs(interval_secs));
        ticker.tick().await; // first tick fires immediately

        loop {
            ticker.tick().await;

            let now = std::time::SystemTime::now();
            let ts = nats_adapter::proto::well_known::Timestamp::from(now);
            let health = pb::BackendHealth {
                backend: backend_id.clone(),
                status: "ok".to_string(),
                updated_at: Some(ts),
                error: String::new(),
                hint: String::new(),
                extra: std::collections::HashMap::new(),
            };

            match encode_envelope("backend_service", "BackendHealth", &health) {
                Ok(bytes) => {
                    if let Err(e) = client.client().publish(subject.clone(), bytes).await {
                        warn!(error = %e, subject = %subject, "Failed to publish health to system.health");
                    }
                }
                Err(e) => warn!(error = %e, "Failed to encode BackendHealth for NATS"),
            }
        }
    });
}
