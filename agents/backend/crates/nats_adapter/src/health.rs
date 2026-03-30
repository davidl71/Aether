use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use chrono::{DateTime, TimeDelta, Utc};
use serde::{Deserialize, Serialize};
use tokio::time;
use tracing::warn;

use crate::{encode_envelope, proto::v1 as pb, topics, NatsClient};

/// Transport-health DTO for a NATS connection or subscription.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
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
    /// Reconnect cycles (subscriber) or cumulative publish/flush failures (publisher), depending on role.
    #[serde(default)]
    pub reconnect_count: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_rtt_ms: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active_subscriptions: Option<u32>,
    // async-nats client lifetime counters (Client::statistics()).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub in_bytes: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub out_bytes: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub in_messages: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub out_messages: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub connects: Option<u64>,
    /// `connected` | `disconnected` | `degraded` (empty when unset / legacy payload).
    #[serde(default)]
    pub connection_state: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub snapshot_backend_id: Option<String>,
    /// RFC3339 timestamp of the last `SystemSnapshot` clock embedded alongside this transport.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub snapshot_generated_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jetstream_enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jetstream_stream_ready: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jetstream_publish_failures: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kv_reachable: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kv_bucket: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kv_last_check_at: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, String>,
}

impl NatsTransportHealthState {
    pub fn with_extra(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.extra.insert(key.into(), value.into());
        self
    }

    pub fn with_subject(self, subject: impl Into<String>) -> Self {
        self.with_extra("subject", subject)
    }

    pub fn with_role(self, role: impl Into<String>) -> Self {
        self.with_extra("role", role)
    }

    pub fn connected(url: Option<String>, updated_at: DateTime<Utc>) -> Self {
        Self {
            connected: true,
            status: "ok".to_string(),
            updated_at: updated_at.to_rfc3339(),
            url,
            error: None,
            hint: None,
            reconnect_count: 0,
            last_rtt_ms: None,
            active_subscriptions: None,
            in_bytes: None,
            out_bytes: None,
            in_messages: None,
            out_messages: None,
            connects: None,
            connection_state: "connected".to_string(),
            snapshot_backend_id: None,
            snapshot_generated_at: None,
            jetstream_enabled: None,
            jetstream_stream_ready: None,
            jetstream_publish_failures: None,
            kv_reachable: None,
            kv_bucket: None,
            kv_last_check_at: None,
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
        let connection_state = if error.is_some() {
            "disconnected".to_string()
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
            reconnect_count: 0,
            last_rtt_ms: None,
            active_subscriptions: None,
            in_bytes: None,
            out_bytes: None,
            in_messages: None,
            out_messages: None,
            connects: None,
            connection_state,
            snapshot_backend_id: None,
            snapshot_generated_at: None,
            jetstream_enabled: None,
            jetstream_stream_ready: None,
            jetstream_publish_failures: None,
            kv_reachable: None,
            kv_bucket: None,
            kv_last_check_at: None,
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
        observed.connection_state = "connected".to_string();
        observed
    }

    pub fn subject(&self) -> Option<&str> {
        self.extra.get("subject").map(String::as_str)
    }

    pub fn role(&self) -> Option<&str> {
        self.extra.get("role").map(String::as_str)
    }

    pub fn is_stale_at(&self, now: DateTime<Utc>, stale_after: TimeDelta) -> bool {
        self.connected
            && self
                .age_secs_at(now)
                .is_some_and(|age_secs| age_secs > stale_after.num_seconds())
    }

    pub fn effective_at(&self, now: DateTime<Utc>, stale_after: TimeDelta) -> Self {
        let mut effective = self.clone();
        if self.is_stale_at(now, stale_after) {
            if let Some(age_secs) = self.age_secs_at(now) {
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
    let mut extra = extra;
    extra
        .entry("subject".to_string())
        .or_insert_with(|| subject.clone());
    extra
        .entry("role".to_string())
        .or_insert_with(|| "publisher".to_string());
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
