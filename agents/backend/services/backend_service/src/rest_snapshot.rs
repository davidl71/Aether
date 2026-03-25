//! Optional REST endpoint for system snapshot and health with content negotiation.
//!
//! When `REST_SNAPSHOT_PORT` is set, serves:
//! - `GET /api/v1/snapshot`: snapshot (Accept: application/x-protobuf → protobuf; else JSON).
//! - `GET /health`: health aggregate with explicit backend counts, transport role/subject/staleness,
//!   plus optional LIVE_STATE KV bucket reachability check (see NATS_KV_USAGE_AND_RECOMMENDATIONS.md).
//!   KV check is optional so health still reports OK when KV is down if that is acceptable.
//! See docs/platform/PROTO_OPPORTUNITIES_AND_BUF_CONFIG.md.

use std::sync::Arc;

use api::{RuntimeSnapshotDto, SharedHealthAggregate};
use axum::{
    extract::State,
    http::{header, HeaderMap, StatusCode},
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use nats_adapter::async_nats;
use prost::Message;

use api::snapshot_proto;
use nats_adapter::proto::v1;

use crate::nats_integration;
use crate::shared_state::SharedSnapshot;

/// Content type for protobuf snapshot response.
const APPLICATION_X_PROTOBUF: &str = "application/x-protobuf";

/// Spawns the REST server if `REST_SNAPSHOT_PORT` is set.
/// Binds to `0.0.0.0:{port}` and serves `GET /api/v1/snapshot` and `GET /health`.
pub fn spawn_if_enabled(
    state: SharedSnapshot,
    health_state: SharedHealthAggregate,
    nats_integration: Arc<Option<nats_integration::NatsIntegration>>,
) {
    let port = match std::env::var("REST_SNAPSHOT_PORT")
        .ok()
        .and_then(|s| s.parse().ok())
    {
        Some(p) if p > 0 => p,
        _ => return,
    };

    let app = Router::new()
        .route("/api/v1/snapshot", get(get_snapshot))
        .route("/health", get(get_health))
        .with_state(AppState {
            state,
            health_state,
            nats_integration,
        });

    tokio::spawn(async move {
        let addr = (std::net::IpAddr::from([0, 0, 0, 0]), port);
        let listener = match tokio::net::TcpListener::bind(addr).await {
            Ok(l) => l,
            Err(e) => {
                tracing::warn!(error = %e, port = port, "REST snapshot server bind failed");
                return;
            }
        };
        tracing::info!(
            port = port,
            "REST server listening on GET /api/v1/snapshot, GET /health"
        );
        let _ = axum::serve(listener, app).await;
    });
}

#[derive(Clone)]
struct AppState {
    state: SharedSnapshot,
    health_state: SharedHealthAggregate,
    nats_integration: Arc<Option<nats_integration::NatsIntegration>>,
}

/// Optional LIVE_STATE (or NATS_KV_BUCKET) bucket reachability check. Returns (ok, bucket name) or
/// None if check skipped (no client or HEALTH_CHECK_KV=0).
async fn check_kv_bucket_reachable(
    nats_integration: &Option<nats_integration::NatsIntegration>,
) -> Option<(bool, String)> {
    let env_val = std::env::var("HEALTH_CHECK_KV").unwrap_or_else(|_| "1".into());
    if matches!(env_val.trim().to_lowercase().as_str(), "0" | "false" | "no") {
        return None;
    }
    let client = nats_integration.as_ref()?.client()?;
    let bucket = std::env::var("NATS_KV_BUCKET").unwrap_or_else(|_| "LIVE_STATE".to_string());
    let js = async_nats::jetstream::new(client.client().clone());
    match js.get_key_value(bucket.as_str()).await {
        Ok(store) => match store.status().await {
            Ok(_) => Some((true, bucket)),
            Err(_) => Some((false, bucket)),
        },
        Err(_) => Some((false, bucket)),
    }
}

async fn get_health(
    State(AppState {
        health_state,
        nats_integration,
        ..
    }): State<AppState>,
) -> impl IntoResponse {
    let mut resp = health_state
        .read()
        .await
        .response_with_stale_after(health_stale_after());
    if let Some((ok, bucket)) = check_kv_bucket_reachable(nats_integration.as_ref()).await {
        resp.kv_bucket_ok = Some(ok);
        resp.kv_bucket = Some(bucket);
    }
    Json(resp)
}

async fn get_snapshot(
    State(AppState { state, .. }): State<AppState>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let snap = state.read().await;

    let want_proto = headers
        .get(header::ACCEPT)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.contains(APPLICATION_X_PROTOBUF))
        .unwrap_or(false);

    if want_proto {
        let proto: v1::SystemSnapshot = snapshot_proto::snapshot_to_proto(&snap);
        let body = proto.encode_to_vec();
        ([(header::CONTENT_TYPE, APPLICATION_X_PROTOBUF)], body).into_response()
    } else {
        let dto = RuntimeSnapshotDto::from(&*snap);
        match serde_json::to_vec(&dto) {
            Ok(body) => ([(header::CONTENT_TYPE, "application/json")], body).into_response(),
            Err(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("JSON serialization error: {}", e),
            )
                .into_response(),
        }
    }
}

fn health_stale_after() -> chrono::TimeDelta {
    let secs = std::env::var("HEALTH_STALE_AFTER_SECS")
        .ok()
        .and_then(|value| value.parse::<i64>().ok())
        .filter(|value| *value > 0)
        .unwrap_or(45);
    chrono::TimeDelta::seconds(secs)
}
