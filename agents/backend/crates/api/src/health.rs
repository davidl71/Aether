use std::{collections::HashMap, sync::Arc};

use chrono::{DateTime, TimeDelta, TimeZone, Utc};
pub use nats_adapter::NatsTransportHealthState;
use nats_adapter::proto::v1::{BackendHealth, NatsEnvelope};
use prost::Message;
use serde::Serialize;
use tokio::sync::RwLock;

pub type SharedHealthAggregate = Arc<RwLock<HealthAggregateState>>;

const DEFAULT_HEALTH_STALE_AFTER_SECS: i64 = 45;

#[derive(Debug, Clone, Default)]
pub struct HealthAggregateState {
    pub backends: HashMap<String, BackendHealthState>,
    pub nats_connected: bool,
    pub transport: NatsTransportHealthState,
}

#[derive(Debug, Clone, Serialize)]
pub struct HealthAggregateResponse {
    pub status: String,
    pub source: String,
    pub nats_connected: bool,
    pub transport: NatsTransportHealthState,
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
        self.response_with_stale_after(TimeDelta::seconds(DEFAULT_HEALTH_STALE_AFTER_SECS))
    }

    pub fn response_with_stale_after(&self, stale_after: TimeDelta) -> HealthAggregateResponse {
        let now = Utc::now();
        let effective_backends = self
            .backends
            .iter()
            .map(|(name, value)| (name.clone(), value.effective_at(now, stale_after)))
            .collect::<HashMap<_, _>>();
        let effective_transport = self.transport.effective_at(now, stale_after);
        let mut backends = effective_backends
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
        let backend_statuses = effective_backends
            .values()
            .map(|value| value.status.as_str())
            .collect::<Vec<_>>();
        let all_backends_ok =
            !backend_statuses.is_empty() && backend_statuses.iter().all(|status| *status == "ok");
        let any_backend_error = backend_statuses
            .iter()
            .any(|status| matches!(*status, "error" | "disabled"));
        let transport_status = effective_transport.status.as_str();
        let transport_ok = transport_status == "ok";
        let transport_error = matches!(transport_status, "error" | "disabled");
        let all_ok = all_backends_ok && transport_ok;
        let any_error = transport_error || any_backend_error;
        let nats_connected = self.nats_connected || effective_transport.connected;

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
            source: if nats_connected {
                "nats".to_string()
            } else {
                "none".to_string()
            },
            nats_connected,
            transport: effective_transport,
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

    pub fn effective_at(&self, now: DateTime<Utc>, stale_after: TimeDelta) -> Self {
        let mut effective = self.clone();
        if self.status == "ok" {
            if let Some(age_secs) = self.age_secs_at(now) {
                if age_secs > stale_after.num_seconds() {
                    effective.status = "degraded".to_string();
                    if effective.hint.is_none() {
                        effective.hint = Some(format!("stale heartbeat ({}s old)", age_secs));
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn backend_health_goes_degraded_when_stale() {
        let now = Utc.with_ymd_and_hms(2026, 3, 25, 12, 0, 0).unwrap();
        let stale_at = (now - TimeDelta::seconds(90)).to_rfc3339();
        let state = BackendHealthState {
            backend: "tui_service".to_string(),
            status: "ok".to_string(),
            updated_at: stale_at,
            error: None,
            hint: None,
            extra: HashMap::new(),
        };

        let effective = state.effective_at(now, TimeDelta::seconds(45));

        assert_eq!(effective.status, "degraded");
        assert_eq!(effective.hint.as_deref(), Some("stale heartbeat (90s old)"));
        assert_eq!(
            effective.extra.get("stale").map(String::as_str),
            Some("true")
        );
    }

    #[test]
    fn nats_transport_goes_degraded_when_stale() {
        let now = Utc.with_ymd_and_hms(2026, 3, 25, 12, 0, 0).unwrap();
        let transport = NatsTransportHealthState {
            connected: true,
            status: "ok".to_string(),
            updated_at: (now - TimeDelta::seconds(120)).to_rfc3339(),
            url: Some("nats://127.0.0.1:4222".to_string()),
            error: None,
            hint: None,
            extra: HashMap::new(),
        };

        let effective = transport.effective_at(now, TimeDelta::seconds(45));

        assert_eq!(effective.status, "degraded");
        assert_eq!(
            effective.hint.as_deref(),
            Some("stale NATS transport (120s old)")
        );
        assert_eq!(
            effective.extra.get("stale").map(String::as_str),
            Some("true")
        );
    }
}
