use std::{collections::HashMap, sync::Arc};

use chrono::{DateTime, TimeDelta, TimeZone, Utc};
use nats_adapter::proto::v1::{BackendHealth, NatsEnvelope};
pub use nats_adapter::NatsTransportHealthState;
use prost::Message;
use serde::Serialize;
use tokio::sync::RwLock;

pub type SharedHealthAggregate = Arc<RwLock<HealthAggregateState>>;

const DEFAULT_HEALTH_STALE_AFTER_SECS: i64 = 45;

#[derive(Debug, Clone, Default, Serialize)]
pub struct HealthAggregateCounts {
    pub total: usize,
    pub ok: usize,
    pub degraded: usize,
    pub error: usize,
    pub disabled: usize,
    pub stale: usize,
}

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
    pub transport_role: Option<String>,
    pub transport_subject: Option<String>,
    pub transport_age_secs: Option<i64>,
    pub transport_stale: bool,
    pub backends: HashMap<String, serde_json::Value>,
    pub backends_list: Vec<String>,
    pub backend_counts: HealthAggregateCounts,
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
        let backends = effective_backends
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
        let backend_counts = backend_counts(&effective_backends);
        let transport_status = effective_transport.status.as_str();
        let transport_ok = transport_status == "ok";
        let transport_error = matches!(transport_status, "error" | "disabled");
        let all_ok =
            backend_counts.total > 0 && backend_counts.ok == backend_counts.total && transport_ok;
        let any_error = transport_error || backend_counts.error > 0 || backend_counts.disabled > 0;
        let nats_connected = self.nats_connected || effective_transport.connected;
        let transport_role = effective_transport.role().map(|value| value.to_string());
        let transport_subject = effective_transport.subject().map(|value| value.to_string());
        let transport_age_secs = effective_transport.age_secs_at(now);
        let transport_stale = effective_transport.is_stale_at(now, stale_after);

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
            transport_role,
            transport_subject,
            transport_age_secs,
            transport_stale,
            backends,
            backends_list,
            backend_counts,
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
        if let Some(age_secs) = self.age_secs_at(now) {
            if age_secs > stale_after.num_seconds() {
                if effective.hint.is_none() {
                    effective.hint = Some(format!("stale heartbeat ({}s old)", age_secs));
                }
                if effective.status == "ok" {
                    effective.status = "degraded".to_string();
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

    pub fn is_stale_at(&self, now: DateTime<Utc>, stale_after: TimeDelta) -> bool {
        self.age_secs_at(now)
            .is_some_and(|age_secs| age_secs > stale_after.num_seconds())
    }

    pub fn age_secs_at(&self, now: DateTime<Utc>) -> Option<i64> {
        let updated_at = DateTime::parse_from_rfc3339(&self.updated_at).ok()?;
        Some((now - updated_at.with_timezone(&Utc)).num_seconds())
    }
}

fn backend_counts(backends: &HashMap<String, BackendHealthState>) -> HealthAggregateCounts {
    backends
        .values()
        .fold(HealthAggregateCounts::default(), |mut counts, backend| {
            counts.total += 1;
            match backend.status.as_str() {
                "ok" => counts.ok += 1,
                "degraded" => counts.degraded += 1,
                "error" => counts.error += 1,
                "disabled" => counts.disabled += 1,
                _ => {}
            }
            if backend
                .extra
                .get("stale")
                .map(String::as_str)
                .is_some_and(|value| value == "true")
            {
                counts.stale += 1;
            }
            counts
        })
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
    fn backend_health_retains_error_status_when_stale() {
        let now = Utc.with_ymd_and_hms(2026, 3, 25, 12, 0, 0).unwrap();
        let stale_at = (now - TimeDelta::seconds(90)).to_rfc3339();
        let state = BackendHealthState {
            backend: "backend_service".to_string(),
            status: "error".to_string(),
            updated_at: stale_at,
            error: Some("boom".to_string()),
            hint: None,
            extra: HashMap::new(),
        };

        let effective = state.effective_at(now, TimeDelta::seconds(45));

        assert_eq!(effective.status, "error");
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

    #[test]
    fn health_response_exposes_backend_counts_and_transport_metadata() {
        let now = Utc::now();
        let stale_at = (now - TimeDelta::seconds(90)).to_rfc3339();
        let mut state = HealthAggregateState::default();
        state.backends.insert(
            "backend_a".to_string(),
            BackendHealthState {
                backend: "backend_a".to_string(),
                status: "ok".to_string(),
                updated_at: now.to_rfc3339(),
                error: None,
                hint: None,
                extra: HashMap::new(),
            },
        );
        state.backends.insert(
            "backend_b".to_string(),
            BackendHealthState {
                backend: "backend_b".to_string(),
                status: "degraded".to_string(),
                updated_at: stale_at,
                error: None,
                hint: Some("manual degradation".to_string()),
                extra: HashMap::new(),
            },
        );
        state.transport =
            NatsTransportHealthState::connected(Some("nats://127.0.0.1:4222".to_string()), now)
                .with_subject("system.health")
                .with_role("subscriber");

        let resp = state.response_with_stale_after(TimeDelta::seconds(45));

        assert_eq!(resp.backend_counts.total, 2);
        assert_eq!(resp.backend_counts.ok, 1);
        assert_eq!(resp.backend_counts.degraded, 1);
        assert_eq!(resp.backend_counts.stale, 1);
        assert_eq!(resp.transport_subject.as_deref(), Some("system.health"));
        assert_eq!(resp.transport_role.as_deref(), Some("subscriber"));
        assert_eq!(resp.transport_age_secs, Some(0));
        assert!(!resp.transport_stale);
    }
}
