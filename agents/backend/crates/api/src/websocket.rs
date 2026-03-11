// websocket.rs - WebSocket server for real-time updates
use std::collections::HashMap;

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Extension,
    },
    response::Response,
};
use futures::{SinkExt, StreamExt};
use serde_json::json;
use tokio::time::{interval, Duration};
use tracing::{error, info, warn};

use crate::rest::RestState;
use crate::runtime_state::RuntimeExecutionState;
use crate::{
    RuntimeSnapshotDto,
};
use crate::state::SystemSnapshot;

/// WebSocket server for real-time snapshot updates
pub struct WebSocketServer;

impl WebSocketServer {
    /// Create WebSocket route handler
    pub fn route(state: RestState) -> axum::Router<()> {
        use axum::routing::get;
        axum::Router::new()
            .route("/ws", get(websocket_handler))
            .route("/ws/snapshot", get(websocket_handler))
            .layer(axum::Extension(state))
    }

    /// Start WebSocket server
    pub async fn serve(
        addr: std::net::SocketAddr,
        state: RestState,
    ) -> anyhow::Result<tokio::task::JoinHandle<()>> {
        let app = Self::route(state);
        info!(%addr, "starting WebSocket server");
        let handle = tokio::spawn(async move {
            let listener = tokio::net::TcpListener::bind(&addr)
                .await
                .expect("Failed to bind WebSocket server");
            axum::serve(listener, app)
                .await
                .expect("WebSocket server crashed");
        });
        Ok(handle)
    }
}

/// WebSocket connection handler
async fn websocket_handler(
    ws: WebSocketUpgrade,
    Extension(state): axum::Extension<RestState>,
) -> Response {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

/// Serialise each logical section of a snapshot to its JSON string.
/// Returns a fixed-order vec of (section_name, json_string) pairs.
fn serialise_sections(
    snap: &SystemSnapshot,
) -> Result<Vec<(&'static str, String)>, serde_json::Error> {
    let runtime_state = RuntimeExecutionState::from_snapshot(snap);

    Ok(vec![
        (
            "meta",
            serde_json::to_string(&json!({
                "mode": snap.mode,
                "strategy": snap.strategy,
                "account_id": snap.account_id,
            }))?,
        ),
        ("metrics", serde_json::to_string(&snap.metrics)?),
        ("symbols", serde_json::to_string(&snap.symbols)?),
        (
            "positions",
            serde_json::to_string(&runtime_state.position_dtos())?,
        ),
        ("historic", serde_json::to_string(&runtime_state.historic_dtos())?),
        (
            "orders",
            serde_json::to_string(&runtime_state.order_dtos())?,
        ),
        ("alerts", serde_json::to_string(&snap.alerts)?),
        ("decisions", serde_json::to_string(&runtime_state.decision_dtos())?),
        ("risk", serde_json::to_string(&snap.risk)?),
    ])
}

/// Handle individual WebSocket connection
async fn handle_socket(socket: WebSocket, state: RestState) {
    let (mut sender, mut receiver) = socket.split();
    let mut tick = interval(Duration::from_secs(2));
    let mut prev_sections: HashMap<&'static str, String> = HashMap::new();

    info!("WebSocket client connected");

    // Send initial full snapshot (type:"snapshot")
    if let Ok(snapshot_json) = get_snapshot_json(&state).await {
        if let Err(e) = sender.send(Message::Text(snapshot_json)).await {
            error!("Failed to send initial snapshot: {}", e);
            return;
        }
    }

    loop {
        tokio::select! {
            // Send periodic delta updates
            _ = tick.tick() => {
                // Serialise all sections; release the read lock before doing I/O
                let sections = {
                    let snap = state.snapshot.read().await;
                    match serialise_sections(&snap) {
                        Ok(s) => s,
                        Err(e) => {
                            error!("Failed to serialise snapshot sections: {}", e);
                            continue;
                        }
                    }
                };

                // Collect only sections whose JSON string changed
                let mut changed = serde_json::Map::new();
                for (name, json_str) in &sections {
                    let prev = prev_sections.get(*name).map(String::as_str).unwrap_or("");
                    if json_str.as_str() != prev {
                        match serde_json::from_str::<serde_json::Value>(json_str) {
                            Ok(val) => {
                                changed.insert((*name).to_string(), val);
                            }
                            Err(e) => {
                                error!("Failed to re-parse section JSON for {}: {}", name, e);
                            }
                        }
                    }
                }

                // Nothing changed — skip send entirely
                if changed.is_empty() {
                    continue;
                }

                // Persist new strings for next comparison
                for (name, json_str) in sections {
                    prev_sections.insert(name, json_str);
                }

                let msg = match serde_json::to_string(&json!({ "type": "delta", "sections": changed })) {
                    Ok(s) => s,
                    Err(e) => {
                        error!("Failed to serialise delta message: {}", e);
                        continue;
                    }
                };

                if sender.send(Message::Text(msg)).await.is_err() {
                    warn!("WebSocket send error, client disconnected");
                    break;
                }
            }
            // Handle incoming messages (ping/pong, close, etc.)
            msg = receiver.next() => {
                match msg {
                    Some(Ok(Message::Ping(ping))) => {
                        if let Err(e) = sender.send(Message::Pong(ping)).await {
                            error!("Failed to send pong: {}", e);
                            break;
                        }
                    }
                    Some(Ok(Message::Close(_))) => {
                        info!("WebSocket client closed connection");
                        break;
                    }
                    Some(Err(e)) => {
                        error!("WebSocket error: {}", e);
                        break;
                    }
                    _ => {}
                }
            }
        }
    }

    info!("WebSocket connection closed");
}

/// Get full snapshot as JSON string (used only for the initial connect message)
async fn get_snapshot_json(
    state: &RestState,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let snapshot = state.snapshot.read().await;
    let data = serde_json::to_value(RuntimeSnapshotDto::from(&*snapshot))?;
    Ok(serde_json::to_string(&json!({
      "type": "snapshot",
      "data": data
    }))?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serialise_sections_detects_change() {
        let base = crate::state::SystemSnapshot::default();
        let mut modified = base.clone();
        modified.metrics.net_liq += 1_000.0;

        let base_sections: HashMap<&str, String> =
            serialise_sections(&base).unwrap().into_iter().collect();
        let modified_sections: HashMap<&str, String> =
            serialise_sections(&modified).unwrap().into_iter().collect();

        let changed: Vec<&&str> = modified_sections
            .iter()
            .filter(|(k, v)| base_sections.get(*k).map(|s| s != *v).unwrap_or(true))
            .map(|(k, _)| k)
            .collect();

        assert!(
            changed.contains(&&"metrics"),
            "metrics should be in changed set"
        );
        assert!(
            !changed.contains(&&"positions"),
            "positions should not change"
        );
        assert!(!changed.contains(&&"meta"), "meta should not change");
    }
}
