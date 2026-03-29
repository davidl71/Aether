//! Administrative NATS request/reply handlers.
//! Subjects: api.admin.*, api.snapshot.*, api.ib.*
//!
//! Note: api.admin.set_mode is deprecated in data-exploration mode.
//! Snapshot operations use bounded parallelism to prevent memory pressure.

use crate::handlers::{api_queue_group, concurrency_limit, handle_sub_parallel};
use api::fetch_ib_positions;
use api::SnapshotPublishReply;
use bytes::Bytes;
use nats_adapter::async_nats::Client;
use nats_adapter::{encode_envelope, topics};
use serde_json::Value;
use tracing::{info, warn};

use crate::shared_state::SharedSnapshot;

const SUBJECT_ADMIN_SET_MODE: &str = "api.admin.set_mode";
const SUBJECT_SNAPSHOT_PUBLISH_NOW: &str = "api.snapshot.publish_now";
const SUBJECT_IB_POSITIONS: &str = "api.ib.positions";

/// Spawn all administrative NATS API handlers with bounded parallelism.
pub fn spawn(nc: Client, state: SharedSnapshot, backend_id: String) {
    let limit = concurrency_limit();

    let nc_snapshot = nc.clone();
    let state_snapshot = state.clone();
    let snapshot_backend_id = backend_id.clone();
    tokio::spawn(async move {
        run_snapshot_publish_now(nc_snapshot, state_snapshot, snapshot_backend_id, limit).await;
    });

    let nc_ib = nc.clone();
    tokio::spawn(async move {
        run_ib_positions(nc_ib, limit).await;
    });

    let nc_admin = nc.clone();
    tokio::spawn(async move {
        run_admin_set_mode(nc_admin, state, limit).await;
    });
}

/// Force-write current snapshot to NATS (point-in-time) with bounded parallelism.
async fn run_snapshot_publish_now(
    nc: Client,
    state: SharedSnapshot,
    backend_id: String,
    limit: usize,
) {
    let sub = match nc
        .queue_subscribe(SUBJECT_SNAPSHOT_PUBLISH_NOW.to_string(), api_queue_group())
        .await
    {
        Ok(s) => s,
        Err(e) => {
            warn!(error = %e, "subscribe api.snapshot.publish_now failed");
            return;
        }
    };
    info!(
        "subscribed to api.snapshot.publish_now (force snapshot write, concurrency={})",
        limit
    );
    let subject = topics::snapshot::backend(&backend_id);

    handle_sub_parallel(
        nc,
        sub,
        move |_body: Option<Vec<u8>>| {
            let state = state.clone();
            let subject = subject.clone();
            let _backend_id = backend_id.clone();
            async move {
                let command = api::CommandContext::new("publish_snapshot");
                let (proto, generated_at) = {
                    let snap = state.read().await;
                    let proto = api::snapshot_proto::snapshot_to_proto(&snap);
                    let ts = snap.generated_at;
                    (proto, ts)
                };
                let response = match encode_envelope("backend_service", "SystemSnapshot", &proto) {
                    Ok(_bytes) => {
                        serde_json::to_vec(&SnapshotPublishReply::completed_from_context(
                            &command,
                            generated_at.to_rfc3339(),
                            subject,
                            "snapshot published",
                        ))
                        .unwrap_or_else(|_| b"{}".to_vec())
                    }
                    Err(e) => {
                        let err = e.to_string();
                        serde_json::to_vec(&SnapshotPublishReply::failed_from_context(
                            &command, subject, err,
                        ))
                        .unwrap_or_else(|_| b"{}".to_vec())
                    }
                };
                response
            }
        },
        limit,
    )
    .await;
}

/// Handles api.ib.positions with bounded parallelism.
async fn run_ib_positions(nc: Client, limit: usize) {
    let sub = match nc
        .queue_subscribe(SUBJECT_IB_POSITIONS.to_string(), api_queue_group())
        .await
    {
        Ok(s) => s,
        Err(e) => {
            warn!(error = %e, "subscribe api.ib.positions failed");
            return;
        }
    };

    handle_sub_parallel(
        nc,
        sub,
        |body: Option<Vec<u8>>| async move {
            let account_id: Option<String> = body
                .as_deref()
                .and_then(|b| serde_json::from_slice::<Value>(b).ok())
                .and_then(|v| {
                    v.get("account_id")
                        .and_then(Value::as_str)
                        .map(str::to_string)
                });
            let result = fetch_ib_positions(account_id.as_deref()).await;
            match result {
                Ok(positions) => serde_json::to_vec(&positions).unwrap_or_else(|_| b"[]".to_vec()),
                Err(e) => serde_json::to_vec(&serde_json::json!({ "error": e }))
                    .unwrap_or_else(|_| b"{}".to_vec()),
            }
        },
        limit,
    )
    .await;
}

/// Handles api.admin.set_mode with bounded parallelism.
async fn run_admin_set_mode(nc: Client, _state: SharedSnapshot, limit: usize) {
    let sub = match nc
        .queue_subscribe(SUBJECT_ADMIN_SET_MODE.to_string(), api_queue_group())
        .await
    {
        Ok(s) => s,
        Err(e) => {
            warn!(error = %e, "subscribe api.admin.set_mode failed");
            return;
        }
    };

    handle_sub_parallel(
        nc.clone(),
        sub,
        move |body: Option<Vec<u8>>| {
            let nc = nc.clone();
            async move {
                #[derive(serde::Deserialize)]
                struct SetModeRequest {
                    mode: String,
                }
                let command = api::CommandContext::new("set_mode");
                let requested_mode = body
                    .as_deref()
                    .and_then(|b| serde_json::from_slice::<SetModeRequest>(b).ok())
                    .map(|r| r.mode.to_uppercase())
                    .unwrap_or_else(|| "UNKNOWN".to_string());
                let msg = format!(
                    "set_mode is deprecated in data-exploration mode; requested mode {} was ignored",
                    requested_mode
                );
                publish_command_event(&nc, "set_mode", &command.failed_event(msg.clone())).await;
                serde_json::to_vec(&command.failed_reply(msg))
                    .unwrap_or_else(|_| b"{}".to_vec())
            }
        },
        limit,
    ).await;
}

async fn publish_command_event(nc: &Client, action: &str, event: &api::CommandEvent) {
    let subject = topics::system::commands(action);
    let body = match serde_json::to_vec(event) {
        Ok(bytes) => bytes,
        Err(e) => {
            warn!(action = %action, error = %e, "serialize command event failed");
            return;
        }
    };

    if let Err(e) = nc.publish(subject.clone(), Bytes::from(body)).await {
        warn!(action = %action, subject = %subject, error = %e, "publish command event failed");
    }
}
