use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use tokio::time;
use tracing::warn;

use crate::{encode_envelope, proto::v1 as pb, topics, NatsClient};

/// Spawn a periodic `system.health` publisher for a single runtime component.
/// `backend_id` is the component name visible to subscribers and health aggregation.
pub fn spawn_health_publisher(
    client: Arc<NatsClient>,
    backend_id: String,
    interval_secs: u64,
    extra: HashMap<String, String>,
) {
    if interval_secs == 0 {
        return;
    }

    let subject = topics::system::health().to_string();
    tokio::spawn(async move {
        let mut ticker = time::interval(Duration::from_secs(interval_secs));
        ticker.tick().await;

        loop {
            ticker.tick().await;

            let health = pb::BackendHealth {
                backend: backend_id.clone(),
                status: "ok".to_string(),
                updated_at: Some(pb_timestamp_now()),
                error: String::new(),
                hint: String::new(),
                extra: extra.clone(),
            };

            match encode_envelope("rust_service", "BackendHealth", &health) {
                Ok(bytes) => {
                    if let Err(err) = client.client().publish(subject.clone(), bytes).await {
                        warn!(error = %err, subject = %subject, backend = %backend_id, "Failed to publish health to system.health");
                    }
                }
                Err(err) => {
                    warn!(error = %err, backend = %backend_id, "Failed to encode BackendHealth for NATS");
                }
            }
        }
    });
}

fn pb_timestamp_now() -> prost_types::Timestamp {
    let now = std::time::SystemTime::now();
    prost_types::Timestamp::from(now)
}
