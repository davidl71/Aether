use std::{collections::HashMap, sync::Arc};

use chrono::{TimeZone, Utc};
use nats_adapter::proto::v1::{BackendHealth, NatsEnvelope};
use prost::Message;
use serde::Serialize;
use tokio::sync::RwLock;

pub type SharedHealthAggregate = Arc<RwLock<HealthAggregateState>>;

#[derive(Debug, Clone, Default)]
pub struct HealthAggregateState {
    pub backends: HashMap<String, BackendHealthState>,
    pub nats_connected: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct HealthAggregateResponse {
    pub status: String,
    pub source: String,
    pub nats_connected: bool,
    pub backends: HashMap<String, serde_json::Value>,
    pub backends_list: Vec<String>,
    pub all_ok: bool,
    pub any_error: bool,
    pub generated_at: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kv_bucket_ok: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kv_bucket: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct BackendHealthState {
    pub backend: String,
    pub status: String,
    pub updated_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hint: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, String>,
}

impl HealthAggregateState {
    pub fn new_shared() -> SharedHealthAggregate {
        Arc::new(RwLock::new(Self::default()))
    }

    pub fn response(&self) -> HealthAggregateResponse {
        let mut backends = self
            .backends
            .iter()
            .map(|(name, value)| {
                (
                    name.clone(),
                    serde_json::to_value(value).unwrap_or_else(|_| serde_json::json!({})),
                )
            })
            .collect::<HashMap<_, _>>();
        let mut backends_list = backends.keys().cloned().collect::<Vec<_>>();
        backends_list.sort();
        let statuses = self
            .backends
            .values()
            .map(|value| value.status.as_str())
            .collect::<Vec<_>>();
        let all_ok = !statuses.is_empty() && statuses.iter().all(|status| *status == "ok");
        let any_error = statuses
            .iter()
            .any(|status| matches!(*status, "error" | "disabled"));

        if backends.is_empty() {
            backends = HashMap::new();
        }

        HealthAggregateResponse {
            status: if all_ok {
                "ok".to_string()
            } else if any_error {
                "error".to_string()
            } else {
                "degraded".to_string()
            },
            source: if self.nats_connected {
                "nats".to_string()
            } else {
                "none".to_string()
            },
            nats_connected: self.nats_connected,
            backends,
            backends_list,
            all_ok,
            any_error,
            generated_at: Utc::now().to_rfc3339(),
            kv_bucket_ok: None,
            kv_bucket: None,
        }
    }
}

impl BackendHealthState {
    pub fn from_proto(proto: BackendHealth) -> Self {
        Self {
            backend: proto.backend.clone(),
            status: proto.status,
            updated_at: proto
                .updated_at
                .and_then(timestamp_to_rfc3339)
                .unwrap_or_else(|| Utc::now().to_rfc3339()),
            error: if proto.error.is_empty() {
                None
            } else {
                Some(proto.error)
            },
            hint: if proto.hint.is_empty() {
                None
            } else {
                Some(proto.hint)
            },
            extra: proto.extra,
        }
    }
}

pub fn backend_health_from_message(data: &[u8]) -> Option<BackendHealth> {
    if let Ok(envelope) = NatsEnvelope::decode(data) {
        if let Ok(health) = BackendHealth::decode(envelope.payload.as_slice()) {
            return Some(health);
        }
    }
    BackendHealth::decode(data).ok()
}

fn timestamp_to_rfc3339(ts: prost_types::Timestamp) -> Option<String> {
    Utc.timestamp_opt(ts.seconds, ts.nanos as u32)
        .single()
        .map(|dt| dt.to_rfc3339())
}
