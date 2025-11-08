use std::{net::SocketAddr, sync::Arc};

use axum::{
  extract::State,
  http::StatusCode,
  routing::{get, post},
  Json, Router,
};
use chrono::Utc;
use tokio::{sync::watch, task::JoinHandle};
use tracing::info;

use crate::state::{Alert, SharedSnapshot, SystemSnapshot};

#[derive(Clone)]
pub struct StrategyController {
  tx: Arc<watch::Sender<bool>>,
}

impl StrategyController {
  pub fn new(tx: watch::Sender<bool>) -> Self {
    Self { tx: Arc::new(tx) }
  }

  pub fn start(&self) -> Result<(), watch::error::SendError<bool>> {
    self.tx.send(true)
  }

  pub fn stop(&self) -> Result<(), watch::error::SendError<bool>> {
    self.tx.send(false)
  }
}

#[derive(Clone)]
pub struct RestState {
  pub snapshot: SharedSnapshot,
  pub controller: StrategyController,
}

impl RestState {
  pub fn new(snapshot: SharedSnapshot, controller: StrategyController) -> Self {
    Self { snapshot, controller }
  }
}

pub struct RestServer;

impl RestServer {
  pub fn router(state: RestState) -> Router<RestState> {
    Router::new()
      .route("/health", get(health))
      .route("/api/v1/snapshot", get(snapshot))
      .route("/api/v1/strategy/start", post(strategy_start))
      .route("/api/v1/strategy/stop", post(strategy_stop))
      .with_state(state)
  }

  pub async fn serve(addr: SocketAddr, state: RestState) -> anyhow::Result<JoinHandle<()>> {
    let app = Self::router(state);
    info!(%addr, "starting REST server");
    let handle = tokio::spawn(async move {
      axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .expect("REST server crashed");
    });
    Ok(handle)
  }
}

async fn health() -> &'static str {
  "ok"
}

async fn snapshot(State(state): State<RestState>) -> Json<SystemSnapshot> {
  let snapshot = state.snapshot.read().await.clone();
  Json(snapshot)
}

async fn strategy_start(State(state): State<RestState>) -> Result<StatusCode, (StatusCode, String)> {
  state
    .controller
    .start()
    .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;

  {
    let mut snapshot = state.snapshot.write().await;
    snapshot.set_strategy_status("RUNNING");
    snapshot.risk.allowed = true;
    snapshot.risk.reason = None;
    snapshot.risk.updated_at = Utc::now();
    snapshot.alerts.push(Alert::info("Strategy started via REST"));
    trim_alerts(&mut snapshot.alerts);
  }

  Ok(StatusCode::NO_CONTENT)
}

async fn strategy_stop(State(state): State<RestState>) -> Result<StatusCode, (StatusCode, String)> {
  state
    .controller
    .stop()
    .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;

  {
    let mut snapshot = state.snapshot.write().await;
    snapshot.set_strategy_status("PAUSED");
    snapshot.risk.allowed = false;
    snapshot.risk.reason = Some("Strategy paused via REST".into());
    snapshot.risk.updated_at = Utc::now();
    snapshot.alerts.push(Alert::info("Strategy paused via REST"));
    trim_alerts(&mut snapshot.alerts);
  }

  Ok(StatusCode::NO_CONTENT)
}

fn trim_alerts(alerts: &mut Vec<Alert>) {
  while alerts.len() > 32 {
    alerts.remove(0);
  }
}
