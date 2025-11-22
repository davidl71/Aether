// websocket.rs - WebSocket server for real-time updates
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

/// WebSocket server for real-time snapshot updates
pub struct WebSocketServer;

impl WebSocketServer {
  /// Create WebSocket route handler
  pub fn route(state: RestState) -> axum::Router<()> {
    use axum::routing::get;
    axum::Router::new().route("/ws/snapshot", get(websocket_handler)).layer(axum::Extension(state))
  }

  /// Start WebSocket server
  pub async fn serve(addr: std::net::SocketAddr, state: RestState) -> anyhow::Result<tokio::task::JoinHandle<()>> {
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

/// Handle individual WebSocket connection
async fn handle_socket(socket: WebSocket, state: RestState) {
  let (mut sender, mut receiver) = socket.split();
  let mut interval = interval(Duration::from_secs(2)); // Send updates every 2 seconds

  info!("WebSocket client connected");

  // Send initial snapshot
  if let Ok(snapshot_json) = get_snapshot_json(&state).await {
    if let Err(e) = sender.send(Message::Text(snapshot_json)).await {
      error!("Failed to send initial snapshot: {}", e);
      return;
    }
  }

  loop {
    tokio::select! {
      // Send periodic snapshot updates
      _ = interval.tick() => {
        if let Ok(snapshot_json) = get_snapshot_json(&state).await {
          if let Err(e) = sender.send(Message::Text(snapshot_json)).await {
            warn!("WebSocket send error, client disconnected");
            break;
          }
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

/// Get snapshot as JSON string
async fn get_snapshot_json(state: &RestState) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
  let snapshot = state.snapshot.read().await;

  // Convert SystemSnapshot to JSON format matching REST API response
  let json_value = json!({
    "generated_at": snapshot.generated_at.to_rfc3339(),
    "mode": snapshot.mode,
    "strategy": snapshot.strategy,
    "account_id": snapshot.account_id,
    "metrics": {
      "net_liq": snapshot.metrics.net_liq,
      "buying_power": snapshot.metrics.buying_power,
      "excess_liquidity": snapshot.metrics.excess_liquidity,
      "margin_requirement": snapshot.metrics.margin_requirement,
      "commissions": snapshot.metrics.commissions,
      "portal_ok": snapshot.metrics.portal_ok,
      "tws_ok": snapshot.metrics.tws_ok,
      "orats_ok": snapshot.metrics.orats_ok,
      "questdb_ok": snapshot.metrics.questdb_ok,
      "nats_ok": snapshot.metrics.nats_ok,
    },
    "symbols": snapshot.symbols.iter().map(|s| json!({
      "symbol": s.symbol,
      "last": s.last,
      "bid": s.bid,
      "ask": s.ask,
      "spread": s.spread,
      "roi": s.roi,
      "volume": s.volume,
    })).collect::<Vec<_>>(),
    "positions": snapshot.positions.iter().map(|p| json!({
      "id": p.id,
      "symbol": p.symbol,
      "quantity": p.quantity,
      "cost_basis": p.cost_basis,
      "mark": p.mark,
      "unrealized_pnl": p.unrealized_pnl,
    })).collect::<Vec<_>>(),
    "orders": snapshot.orders.iter().map(|o| json!({
      "id": o.id,
      "symbol": o.symbol,
      "status": o.status,
      "quantity": o.quantity,
      "side": o.side,
    })).collect::<Vec<_>>(),
    "alerts": snapshot.alerts.iter().map(|a| json!({
      "timestamp": a.timestamp.to_rfc3339(),
      "level": a.level,
      "message": a.message,
    })).collect::<Vec<_>>(),
  });

  Ok(serde_json::to_string(&json!({
    "type": "snapshot",
    "data": json_value
  }))?)
}
