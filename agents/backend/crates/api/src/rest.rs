use std::{net::SocketAddr, sync::Arc};

use axum::{
  extract::{Path, Query, State},
  http::StatusCode,
  routing::{get, post, put},
  Json, Router,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use tokio::{sync::watch, task::JoinHandle};
use tracing::info;

use crate::state::{Alert, OrderSnapshot, SharedSnapshot, SystemSnapshot};

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
      .route("/api/v1/strategy/status", get(strategy_status))
      .route("/api/v1/orders", get(orders_list))
      .route("/api/v1/orders/cancel", post(cancel_order))
      .route("/api/v1/orders/:order_id", get(order_details))
      .route("/api/mode", post(toggle_mode))
      .route("/api/account", post(change_account))
      .route("/api/v1/config", get(get_config))
      .route("/api/v1/config", put(update_config))
      .route("/api/v1/scenarios", get(get_scenarios))
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

// Request/Response models
#[derive(Debug, Deserialize)]
struct CancelOrderRequest {
  order_id: String,
}

#[derive(Debug, Deserialize)]
struct ModeRequest {
  mode: String,
}

#[derive(Debug, Deserialize)]
struct AccountRequest {
  account_id: String,
}

#[derive(Debug, Serialize)]
struct ApiResponse {
  status: String,
  message: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  data: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
struct StrategyStatusResponse {
  status: String,
  started_at: Option<chrono::DateTime<Utc>>,
  last_update: chrono::DateTime<Utc>,
}

#[derive(Debug, Serialize)]
struct OrdersListResponse {
  orders: Vec<OrderSnapshot>,
}

#[derive(Debug, Deserialize)]
struct OrdersListQuery {
  status: Option<String>,
  limit: Option<usize>,
}

#[derive(Debug, Deserialize)]
struct ConfigUpdateRequest {
  #[serde(default)]
  mode: Option<String>,
  #[serde(default)]
  strategy: Option<serde_json::Value>,
  #[serde(default)]
  risk: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct ScenariosQuery {
  symbol: Option<String>,
  min_apr: Option<f64>,
}

// Endpoint implementations

async fn strategy_status(State(state): State<RestState>) -> Json<StrategyStatusResponse> {
  let snapshot = state.snapshot.read().await;
  Json(StrategyStatusResponse {
    status: snapshot.strategy.clone(),
    started_at: None, // TODO: Track start time
    last_update: snapshot.generated_at,
  })
}

async fn orders_list(
  State(state): State<RestState>,
  Query(params): Query<OrdersListQuery>,
) -> Json<OrdersListResponse> {
  let snapshot = state.snapshot.read().await;
  let mut orders = snapshot.orders.clone();

  // Filter by status if provided
  if let Some(status_filter) = &params.status {
    orders.retain(|o| o.status.eq_ignore_ascii_case(status_filter));
  }

  // Limit results
  if let Some(limit) = params.limit {
    orders.truncate(limit);
  }

  Json(OrdersListResponse { orders })
}

async fn order_details(
  State(state): State<RestState>,
  Path(order_id): Path<String>,
) -> Result<Json<OrderSnapshot>, (StatusCode, Json<ApiResponse>)> {
  let snapshot = state.snapshot.read().await;
  let order = snapshot
    .orders
    .iter()
    .find(|o| o.id == order_id)
    .cloned();

  match order {
    Some(order) => Ok(Json(order)),
    None => Err((
      StatusCode::NOT_FOUND,
      Json(ApiResponse {
        status: "error".into(),
        message: format!("Order {} not found", order_id),
        data: None,
      }),
    )),
  }
}

async fn cancel_order(
  State(state): State<RestState>,
  Json(request): Json<CancelOrderRequest>,
) -> Result<Json<ApiResponse>, (StatusCode, Json<ApiResponse>)> {
  let mut snapshot = state.snapshot.write().await;

  // Find and update order status
  if let Some(order) = snapshot.orders.iter_mut().find(|o| o.id == request.order_id) {
    order.status = "CANCELLED".into();
    snapshot.touch();
    snapshot.alerts.push(Alert::info(&format!(
      "Order {} cancelled via REST",
      request.order_id
    )));
    trim_alerts(&mut snapshot.alerts);

    Ok(Json(ApiResponse {
      status: "ok".into(),
      message: format!("Order {} cancelled", request.order_id),
      data: Some(serde_json::json!({ "order_id": request.order_id })),
    }))
  } else {
    Err((
      StatusCode::NOT_FOUND,
      Json(ApiResponse {
        status: "error".into(),
        message: format!("Order {} not found", request.order_id),
        data: None,
      }),
    ))
  }
}

async fn toggle_mode(
  State(state): State<RestState>,
  Json(request): Json<ModeRequest>,
) -> Result<Json<ApiResponse>, (StatusCode, Json<ApiResponse>)> {
  let valid_modes = ["DRY-RUN", "LIVE"];
  if !valid_modes.contains(&request.mode.as_str()) {
    return Err((
      StatusCode::BAD_REQUEST,
      Json(ApiResponse {
        status: "error".into(),
        message: format!("Invalid mode: {}. Must be one of: {:?}", request.mode, valid_modes),
        data: None,
      }),
    ));
  }

  let mut snapshot = state.snapshot.write().await;
  snapshot.mode = request.mode.clone();
  snapshot.touch();
  snapshot.alerts.push(Alert::info(&format!("Mode changed to {} via REST", request.mode)));
  trim_alerts(&mut snapshot.alerts);

  Ok(Json(ApiResponse {
    status: "ok".into(),
    message: format!("Mode changed to {}", request.mode),
    data: Some(serde_json::json!({ "mode": request.mode })),
  }))
}

async fn change_account(
  State(state): State<RestState>,
  Json(request): Json<AccountRequest>,
) -> Json<ApiResponse> {
  let mut snapshot = state.snapshot.write().await;
  snapshot.account_id = request.account_id.clone();
  snapshot.touch();
  snapshot.alerts.push(Alert::info(&format!(
    "Account changed to {} via REST",
    request.account_id
  )));
  trim_alerts(&mut snapshot.alerts);

  Json(ApiResponse {
    status: "ok".into(),
    message: "Account changed".into(),
    data: Some(serde_json::json!({ "account_id": request.account_id })),
  })
}

async fn get_config(State(state): State<RestState>) -> Json<serde_json::Value> {
  let snapshot = state.snapshot.read().await;
  Json(serde_json::json!({
    "mode": snapshot.mode,
    "account_id": snapshot.account_id,
    "strategy": {
      "status": snapshot.strategy,
      "symbols": snapshot.symbols.iter().map(|s| &s.symbol).collect::<Vec<_>>(),
    },
    "risk": {
      "allowed": snapshot.risk.allowed,
      "reason": snapshot.risk.reason,
    },
  }))
}

async fn update_config(
  State(state): State<RestState>,
  Json(request): Json<ConfigUpdateRequest>,
) -> Result<Json<ApiResponse>, (StatusCode, Json<ApiResponse>)> {
  let mut snapshot = state.snapshot.write().await;

  if let Some(mode) = request.mode {
    let valid_modes = ["DRY-RUN", "LIVE"];
    if !valid_modes.contains(&mode.as_str()) {
      return Err((
        StatusCode::BAD_REQUEST,
        Json(ApiResponse {
          status: "error".into(),
          message: format!("Invalid mode: {}. Must be one of: {:?}", mode, valid_modes),
          data: None,
        }),
      ));
    }
    snapshot.mode = mode;
  }

  // TODO: Apply strategy and risk config updates when those fields are implemented
  if request.strategy.is_some() || request.risk.is_some() {
    snapshot.alerts.push(Alert::info("Configuration update (strategy/risk) not yet fully implemented"));
  }

  snapshot.touch();
  snapshot.alerts.push(Alert::info("Configuration updated via REST"));
  trim_alerts(&mut snapshot.alerts);

  Ok(Json(ApiResponse {
    status: "ok".into(),
    message: "Configuration updated".into(),
    data: None,
  }))
}

async fn get_scenarios(
  State(state): State<RestState>,
  Query(params): Query<ScenariosQuery>,
) -> Json<serde_json::Value> {
  // TODO: Implement actual scenario calculation
  // For now, return empty scenarios
  Json(serde_json::json!({
    "scenarios": [],
    "as_of": Utc::now(),
    "underlying": params.symbol.unwrap_or_else(|| "SPX".to_string()),
  }))
}
