//! Administrative NATS request/reply handlers.
//! Subjects: api.admin.*, api.snapshot.*, api.ib.*
//!
//! Note: api.admin.set_mode is deprecated in data-exploration mode.

use crate::handlers::api_queue_group;
use api::fetch_ib_positions;
use api::SnapshotPublishReply;
use bytes::Bytes;
use futures::StreamExt;
use nats_adapter::async_nats::Client;
use nats_adapter::{encode_envelope, topics};
use serde_json::Value;
use tracing::{info, warn};

use crate::shared_state::SharedSnapshot;

const SUBJECT_ADMIN_SET_MODE: &str = "api.admin.set_mode";
const SUBJECT_SNAPSHOT_PUBLISH_NOW: &str = "api.snapshot.publish_now";
const SUBJECT_IB_POSITIONS: &str = "api.ib.positions";

/// Spawn all administrative NATS API handlers.
pub fn spawn(
    nc: Client,
    state: SharedSnapshot,
    backend_id: String,
) {
    let nc_snapshot = nc.clone();
    let state_snapshot = state.clone();
    let snapshot_backend_id = backend_id.clone();
    tokio::spawn(async move {
        run_snapshot_publish_now(nc_snapshot, state_snapshot, snapshot_backend_id).await;
    });

    let nc_ib = nc.clone();
    tokio::spawn(async move {
        run_ib_positions(nc_ib).await;
    });

    let nc_admin = nc.clone();
    tokio::spawn(async move {
        run_admin_set_mode(nc_admin, state).await;
    });
}

/// Force-write current snapshot to NATS (point-in-time). Subscribes to api.snapshot.publish_now;
/// on request, publishes current SystemSnapshot to snapshot.{backend_id} and replies with ok + generated_at.
async fn run_snapshot_publish_now(nc: Client, state: SharedSnapshot, backend_id: String) {
    let mut sub = match nc
        .queue_subscribe(SUBJECT_SNAPSHOT_PUBLISH_NOW.to_string(), api_queue_group())
        .await
    {
        Ok(s) => s,
        Err(e) => {
            warn!(error = %e, "subscribe api.snapshot.publish_now failed");
            return;
        }
    };
    info!("subscribed to api.snapshot.publish_now (force snapshot write)");
    let subject = topics::snapshot::backend(&backend_id);
    let nc_events = nc.clone();
    while let Some(msg) = sub.next().await {
        let reply = match msg.reply {
            Some(r) => r,
            None => continue,
        };
        let command = api::CommandContext::new("publish_snapshot");
        let nc = nc_events.clone();
        publish_command_event(
            &nc,
            "publish_snapshot",
            &command.accepted_event("publish snapshot accepted"),
        )
        .await;
        let (proto, generated_at) = {
            let snap = state.read().await;
            let proto = api::snapshot_proto::snapshot_to_proto(&snap);
            let ts = snap.generated_at;
            (proto, ts)
        };
        let response = match encode_envelope("backend_service", "SystemSnapshot", &proto) {
            Ok(bytes) => {
                if let Err(e) = nc.publish(subject.clone(), bytes.into()).await {
                    warn!(error = %e, subject = %subject, "publish snapshot failed");
                    let err = e.to_string();
                    publish_command_event(
                        &nc,
                        "publish_snapshot",
                        &command.failed_event(err.clone()),
                    )
                    .await;
                    serde_json::to_vec(&SnapshotPublishReply::failed_from_context(
                        &command,
                        subject.clone(),
                        err,
                    ))
                    .unwrap_or_else(|_| b"{}".to_vec())
                } else {
                    publish_command_event(
                        &nc,
                        "publish_snapshot",
                        &command.completed_event("snapshot published"),
                    )
                    .await;
                    serde_json::to_vec(&SnapshotPublishReply::completed_from_context(
                        &command,
                        generated_at.to_rfc3339(),
                        subject.clone(),
                        "snapshot published",
                    ))
                    .unwrap_or_else(|_| b"{}".to_vec())
                }
            }
            Err(e) => {
                let err = e.to_string();
                publish_command_event(&nc, "publish_snapshot", &command.failed_event(err.clone()))
                    .await;
                serde_json::to_vec(&SnapshotPublishReply::failed_from_context(
                    &command,
                    subject.clone(),
                    err,
                ))
                .unwrap_or_else(|_| b"{}".to_vec())
            }
        };
        if let Err(e) = nc.publish(reply, Bytes::from(response)).await {
            warn!(error = %e, "reply to api.snapshot.publish_now failed");
        }
    }
}

/// Handles api.ib.positions - fetches IB positions.
async fn run_ib_positions(nc: Client) {
    let mut sub = match nc
        .queue_subscribe(SUBJECT_IB_POSITIONS.to_string(), api_queue_group())
        .await
    {
        Ok(s) => s,
        Err(e) => {
            warn!(error = %e, "subscribe api.ib.positions failed");
            return;
        }
    };

    tokio::spawn(async move {
        while let Some(msg) = sub.next().await {
            let reply = match msg.reply {
                Some(r) => r,
                None => continue,
            };
            let body = if msg.payload.is_empty() {
                None
            } else {
                Some(msg.payload.to_vec())
            };
            let account_id: Option<String> = body
                .as_deref()
                .and_then(|b| serde_json::from_slice::<Value>(b).ok())
                .and_then(|v| {
                    v.get("account_id")
                        .and_then(Value::as_str)
                        .map(str::to_string)
                });
            let result = fetch_ib_positions(account_id.as_deref()).await;
            let response = match result {
                Ok(positions) => serde_json::to_vec(&positions).unwrap_or_else(|_| b"[]".to_vec()),
                Err(e) => serde_json::to_vec(&serde_json::json!({ "error": e }))
                    .unwrap_or_else(|_| b"{}".to_vec()),
            };
            if let Err(e) = nc.publish(reply, Bytes::from(response)).await {
                warn!(error = %e, "reply to api.ib.positions failed");
            }
        }
    });
}

/// Handles api.admin.set_mode. This is deprecated while the product is in read-only exploration mode.
async fn run_admin_set_mode(nc: Client, _state: SharedSnapshot) {
    let mut sub = match nc
        .queue_subscribe(SUBJECT_ADMIN_SET_MODE.to_string(), api_queue_group())
        .await
    {
        Ok(s) => s,
        Err(e) => {
            warn!(error = %e, "subscribe api.admin.set_mode failed");
            return;
        }
    };
    let nc_events = nc.clone();

    tokio::spawn(async move {
        while let Some(msg) = sub.next().await {
            let reply = match msg.reply {
                Some(r) => r,
                None => continue,
            };
            let body = if msg.payload.is_empty() {
                None
            } else {
                Some(msg.payload.to_vec())
            };

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
            publish_command_event(&nc_events, "set_mode", &command.failed_event(msg.clone())).await;
            let response = serde_json::to_vec(&command.failed_reply(msg))
                .unwrap_or_else(|_| b"{}".to_vec());
            if let Err(e) = nc_events.publish(reply, Bytes::from(response)).await {
                warn!(error = %e, "reply to api.admin.set_mode failed");
            }
        }
    });
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
