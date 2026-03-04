use std::time::Duration;

use axum::{extract::State, http::StatusCode, Json};
use serde::Serialize;
use tokio::time::timeout;
use tracing::warn;

use crate::rest::RestState;

#[derive(Serialize)]
pub struct HealthResponse {
  status: String,
  components: ComponentHealth,
}

#[derive(Serialize)]
struct ComponentHealth {
  backend: String,
  nats: String,
}

pub async fn health(State(state): State<RestState>) -> Result<Json<HealthResponse>, StatusCode> {
  let nats_status = check_nats_health().await;

  {
    let mut snapshot = state.snapshot.write().await;
    snapshot.metrics.nats_ok = nats_status == "ok";
  }

  Ok(Json(HealthResponse {
    status: "ok".into(),
    components: ComponentHealth {
      backend: "ok".into(),
      nats: nats_status,
    },
  }))
}

async fn check_nats_health() -> String {
  const NATS_HEALTH_URL: &str = "http://localhost:8222/healthz";

  match timeout(Duration::from_secs(1), reqwest::get(NATS_HEALTH_URL)).await {
    Ok(Ok(resp)) if resp.status().is_success() => "ok".into(),
    Ok(Ok(resp)) => {
      warn!("NATS health check returned: {}", resp.status());
      "degraded".into()
    }
    Ok(Err(e)) => {
      warn!(error = %e, "NATS health check failed");
      "unavailable".into()
    }
    Err(_) => {
      warn!("NATS health check timed out");
      "timeout".into()
    }
  }
}
