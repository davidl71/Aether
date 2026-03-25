use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use chrono::{DateTime, TimeDelta, Utc};
use serde::Serialize;
use tokio::time;
use tracing::warn;

use crate::{encode_envelope, proto::v1 as pb, topics, NatsClient};

/// Transport-health DTO for a NATS connection or subscription.
#[derive(Debug, Clone, Default, Serialize)]
pub struct NatsTransportHealthState {
    pub connected: bool,
    pub status: String,
    pub updated_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hint: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, String>,
}

impl NatsTransportHealthState {
    pub fn connected(url: Option<String>, updated_at: DateTime<Utc>) -> Self {
        Self {
            connected: true,
            status: "ok".to_string(),
            updated_at: updated_at.to_rfc3339(),
            url,
            error: None,
            hint: None,
            extra: HashMap::new(),
        }
    }

    pub fn disconnected(
        url: Option<String>,
        updated_at: DateTime<Utc>,
        error: Option<String>,
        hint: Option<String>,
    ) -> Self {
        let status = if error.is_some() {
            "error".to_string()
        } else {
            "degraded".to_string()
        };
        Self {
            connected: false,
            status,
            updated_at: updated_at.to_rfc3339(),
            url,
            error,
            hint,
            extra: HashMap::new(),
        }
    }

    pub fn observed(&self, updated_at: DateTime<Utc>) -> Self {
        let mut observed = self.clone();
        observed.connected = true;
        observed.status = "ok".to_string();
        observed.updated_at = updated_at.to_rfc3339();
        observed.error = None;
        observed.hint = None;
        observed
    }

    pub fn effective_at(&self, now: DateTime<Utc>, stale_after: TimeDelta) -> Self {
        let mut effective = self.clone();
        if self.connected {
            if let Some(age_secs) = self.age_secs_at(now) {
                if age_secs > stale_after.num_seconds() {
                    effective.status = "degraded".to_string();
                    if effective.hint.is_none() {
                        effective.hint = Some(format!("stale NATS transport ({}s old)", age_secs));
                    }
                    effective
                        .extra
                        .insert("stale".to_string(), "true".to_string());
                    effective
                        .extra
                        .insert("age_secs".to_string(), age_secs.to_string());
                }
            }
        }
        effective
    }

    pub fn age_secs_at(&self, now: DateTime<Utc>) -> Option<i64> {
        let updated_at = DateTime::parse_from_rfc3339(&self.updated_at).ok()?;
        Some((now - updated_at.with_timezone(&Utc)).num_seconds())
    }
}

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
