use axum::{extract::State, http::StatusCode, Json};
use chrono::Utc;

use super::models::{trim_alerts, StrategyStatusResponse};
use crate::rest::RestState;
use crate::state::Alert;

pub async fn start(State(state): State<RestState>) -> Result<StatusCode, (StatusCode, String)> {
  state.controller.start().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

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

pub async fn stop(State(state): State<RestState>) -> Result<StatusCode, (StatusCode, String)> {
  state.controller.stop().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

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

pub async fn status(State(state): State<RestState>) -> Json<StrategyStatusResponse> {
  let snapshot = state.snapshot.read().await;
  Json(StrategyStatusResponse {
    status: snapshot.strategy.clone(),
    started_at: Some(snapshot.started_at),
    last_update: snapshot.generated_at,
  })
}
