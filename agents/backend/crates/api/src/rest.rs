use std::{net::SocketAddr, sync::Arc, time::Duration};

use axum::{
  extract::{Extension, Path, Query},
  http::StatusCode,
  routing::{get, post, put},
  Json, Router,
};
use crate::websocket;
use tower::make::Shared;
use chrono::Utc;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tokio::{sync::watch, task::JoinHandle, time::timeout};
use tracing::{info, warn};

use crate::state::{Alert, OrderSnapshot, SharedSnapshot, SystemSnapshot};

const SWIFTNESS_API_URL: &str = "http://127.0.0.1:8081";

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

/// Health check function for NATS client connection
pub type NatsHealthCheck = Arc<dyn Fn() -> Box<dyn std::future::Future<Output = String> + Send + Unpin> + Send + Sync>;

#[derive(Clone)]
pub struct RestState {
  pub snapshot: SharedSnapshot,
  pub controller: StrategyController,
  pub nats_health_check: Option<NatsHealthCheck>,
}

// Ensure RestState is Send + Sync for axum Router requirements
// SharedSnapshot is Arc<RwLock<...>> which is Send + Sync
// StrategyController contains Arc<...> which is Send + Sync
// So RestState is automatically Send + Sync

impl RestState {
  pub fn new(
    snapshot: SharedSnapshot,
    controller: StrategyController,
    nats_health_check: Option<NatsHealthCheck>,
  ) -> Self {
    Self {
      snapshot,
      controller,
      nats_health_check,
    }
  }
}

pub struct RestServer;

impl RestServer {
  pub fn router(state: RestState) -> Router<()> {
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
      .route("/api/v1/swiftness/positions", get(swiftness_positions))
      .route("/api/v1/swiftness/portfolio-value", get(swiftness_portfolio_value))
      .route("/api/v1/swiftness/validate", get(swiftness_validate))
      .route("/api/v1/swiftness/exchange-rate", get(swiftness_exchange_rate))
      .route("/api/v1/swiftness/exchange-rate", put(swiftness_update_exchange_rate))
      .merge(websocket::WebSocketServer::route(state.clone()))
      .layer(axum::Extension(state))
  }

  pub async fn serve(addr: SocketAddr, state: RestState) -> anyhow::Result<JoinHandle<()>> {
    let app = Self::router(state);
    info!(%addr, "starting REST server");
    let handle = tokio::spawn(async move {
      let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("Failed to bind REST server");
      // In axum 0.7, Router<()> implements Service<IncomingStream> directly
      // We use Extension layer instead of with_state to make the router stateless
      axum::serve(listener, app)
        .await
        .expect("REST server crashed");
    });
    Ok(handle)
  }
}

#[derive(Serialize)]
struct HealthResponse {
  status: String,
  components: ComponentHealth,
}

#[derive(Serialize)]
struct ComponentHealth {
  backend: String,
  nats: NatsHealth,
}

#[derive(Serialize)]
struct NatsHealth {
  server: String,
  client: String,
}

async fn health(Extension(state): Extension<RestState>) -> Result<Json<HealthResponse>, StatusCode> {
  // Check NATS server HTTP health endpoint (non-blocking, 1 second timeout)
  let nats_server_status = check_nats_server_health().await;

  // Check NATS client connection health
  let nats_client_status = if let Some(ref health_check) = state.nats_health_check {
    health_check().await
  } else {
    "unavailable".to_string()
  };

  // Overall NATS status: ok if both server and client are ok
  let nats_overall_ok = nats_server_status == "ok" && nats_client_status == "ok";

  // Update metrics with NATS status
  {
    let mut snapshot = state.snapshot.write().await;
    snapshot.metrics.nats_ok = nats_overall_ok;
  }

  let response = HealthResponse {
    status: "ok".to_string(),
    components: ComponentHealth {
      backend: "ok".to_string(),
      nats: NatsHealth {
        server: nats_server_status,
        client: nats_client_status,
      },
    },
  };

  Ok(Json(response))
}

async fn check_nats_server_health() -> String {
  const NATS_HEALTH_URL: &str = "http://localhost:8222/healthz";
  const TIMEOUT_SECS: u64 = 1;

  match timeout(Duration::from_secs(TIMEOUT_SECS), reqwest::get(NATS_HEALTH_URL)).await {
    Ok(Ok(response)) => {
      if response.status().is_success() {
        "ok".to_string()
      } else {
        warn!("NATS health check returned non-success status: {}", response.status());
        "degraded".to_string()
      }
    }
    Ok(Err(e)) => {
      warn!(error = %e, "NATS health check request failed");
      "unavailable".to_string()
    }
    Err(_) => {
      warn!("NATS health check timed out after {} seconds", TIMEOUT_SECS);
      "timeout".to_string()
    }
  }
}

async fn snapshot(Extension(state): Extension<RestState>) -> Json<SystemSnapshot> {
  let snapshot = state.snapshot.read().await.clone();
  Json(snapshot)
}

async fn strategy_start(Extension(state): Extension<RestState>) -> Result<StatusCode, (StatusCode, String)> {
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

async fn strategy_stop(Extension(state): Extension<RestState>) -> Result<StatusCode, (StatusCode, String)> {
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

async fn strategy_status(Extension(state): Extension<RestState>) -> Json<StrategyStatusResponse> {
  let snapshot = state.snapshot.read().await;
  Json(StrategyStatusResponse {
    status: snapshot.strategy.clone(),
    started_at: None, // TODO: Track start time
    last_update: snapshot.generated_at,
  })
}

async fn orders_list(
  Extension(state): Extension<RestState>,
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
  Extension(state): Extension<RestState>,
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
  Extension(state): Extension<RestState>,
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
  Extension(state): Extension<RestState>,
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
  Extension(state): Extension<RestState>,
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

async fn get_config(Extension(state): Extension<RestState>) -> Json<serde_json::Value> {
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
  Extension(state): Extension<RestState>,
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
  Extension(_state): Extension<RestState>,
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

// Swiftness API proxy endpoints

#[derive(Debug, Deserialize)]
struct SwiftnessPositionsQuery {
  check_validity: Option<bool>,
  max_age_days: Option<u32>,
}

async fn swiftness_positions(
  Query(params): Query<SwiftnessPositionsQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ApiResponse>)> {
  let client = Client::builder()
    .timeout(Duration::from_secs(5))
    .build()
    .map_err(|e| {
      (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ApiResponse {
          status: "error".into(),
          message: format!("Failed to create HTTP client: {}", e),
          data: None,
        }),
      )
    })?;

  let mut url = format!("{}/positions", SWIFTNESS_API_URL);
  let check_validity = params.check_validity.unwrap_or(true);
  let max_age_days = params.max_age_days.unwrap_or(90);
  url.push_str(&format!("?check_validity={}&max_age_days={}", check_validity, max_age_days));

  match client.get(&url).send().await {
    Ok(response) => {
      if response.status().is_success() {
        match response.json::<serde_json::Value>().await {
          Ok(data) => Ok(Json(data)),
          Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse {
              status: "error".into(),
              message: format!("Failed to parse Swiftness API response: {}", e),
              data: None,
            }),
          )),
        }
      } else {
        Err((
          StatusCode::BAD_GATEWAY,
          Json(ApiResponse {
            status: "error".into(),
            message: format!("Swiftness API returned error: {}", response.status()),
            data: None,
          }),
        ))
      }
    }
    Err(e) => {
      warn!(%e, "failed to call Swiftness API");
      Err((
        StatusCode::BAD_GATEWAY,
        Json(ApiResponse {
          status: "error".into(),
          message: format!("Swiftness API unavailable: {}", e),
          data: None,
        }),
      ))
    }
  }
}

async fn swiftness_portfolio_value() -> Result<Json<serde_json::Value>, (StatusCode, Json<ApiResponse>)> {
  let client = Client::builder()
    .timeout(Duration::from_secs(5))
    .build()
    .map_err(|e| {
      (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ApiResponse {
          status: "error".into(),
          message: format!("Failed to create HTTP client: {}", e),
          data: None,
        }),
      )
    })?;

  let url = format!("{}/portfolio-value", SWIFTNESS_API_URL);

  match client.get(&url).send().await {
    Ok(response) => {
      if response.status().is_success() {
        match response.json::<serde_json::Value>().await {
          Ok(data) => Ok(Json(data)),
          Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse {
              status: "error".into(),
              message: format!("Failed to parse Swiftness API response: {}", e),
              data: None,
            }),
          )),
        }
      } else {
        Err((
          StatusCode::BAD_GATEWAY,
          Json(ApiResponse {
            status: "error".into(),
            message: format!("Swiftness API returned error: {}", response.status()),
            data: None,
          }),
        ))
      }
    }
    Err(e) => {
      warn!(%e, "failed to call Swiftness API");
      Err((
        StatusCode::BAD_GATEWAY,
        Json(ApiResponse {
          status: "error".into(),
          message: format!("Swiftness API unavailable: {}", e),
          data: None,
        }),
      ))
    }
  }
}

async fn swiftness_validate() -> Result<Json<serde_json::Value>, (StatusCode, Json<ApiResponse>)> {
  let client = Client::builder()
    .timeout(Duration::from_secs(5))
    .build()
    .map_err(|e| {
      (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ApiResponse {
          status: "error".into(),
          message: format!("Failed to create HTTP client: {}", e),
          data: None,
        }),
      )
    })?;

  let url = format!("{}/validate", SWIFTNESS_API_URL);

  match client.get(&url).send().await {
    Ok(response) => {
      if response.status().is_success() {
        match response.json::<serde_json::Value>().await {
          Ok(data) => Ok(Json(data)),
          Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse {
              status: "error".into(),
              message: format!("Failed to parse Swiftness API response: {}", e),
              data: None,
            }),
          )),
        }
      } else {
        Err((
          StatusCode::BAD_GATEWAY,
          Json(ApiResponse {
            status: "error".into(),
            message: format!("Swiftness API returned error: {}", response.status()),
            data: None,
          }),
        ))
      }
    }
    Err(e) => {
      warn!(%e, "failed to call Swiftness API");
      Err((
        StatusCode::BAD_GATEWAY,
        Json(ApiResponse {
          status: "error".into(),
          message: format!("Swiftness API unavailable: {}", e),
          data: None,
        }),
      ))
    }
  }
}

async fn swiftness_exchange_rate() -> Result<Json<serde_json::Value>, (StatusCode, Json<ApiResponse>)> {
  let client = Client::builder()
    .timeout(Duration::from_secs(5))
    .build()
    .map_err(|e| {
      (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ApiResponse {
          status: "error".into(),
          message: format!("Failed to create HTTP client: {}", e),
          data: None,
        }),
      )
    })?;

  let url = format!("{}/exchange-rate", SWIFTNESS_API_URL);

  match client.get(&url).send().await {
    Ok(response) => {
      if response.status().is_success() {
        match response.json::<serde_json::Value>().await {
          Ok(data) => Ok(Json(data)),
          Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse {
              status: "error".into(),
              message: format!("Failed to parse Swiftness API response: {}", e),
              data: None,
            }),
          )),
        }
      } else {
        Err((
          StatusCode::BAD_GATEWAY,
          Json(ApiResponse {
            status: "error".into(),
            message: format!("Swiftness API returned error: {}", response.status()),
            data: None,
          }),
        ))
      }
    }
    Err(e) => {
      warn!(%e, "failed to call Swiftness API");
      Err((
        StatusCode::BAD_GATEWAY,
        Json(ApiResponse {
          status: "error".into(),
          message: format!("Swiftness API unavailable: {}", e),
          data: None,
        }),
      ))
    }
  }
}

#[derive(Debug, Deserialize)]
#[derive(Serialize)]
struct ExchangeRateUpdate {
  rate: f64,
}

async fn swiftness_update_exchange_rate(
  Json(update): Json<ExchangeRateUpdate>,
) -> Result<Json<ApiResponse>, (StatusCode, Json<ApiResponse>)> {
  if update.rate <= 0.0 {
    return Err((
      StatusCode::BAD_REQUEST,
      Json(ApiResponse {
        status: "error".into(),
        message: "Exchange rate must be positive".into(),
        data: None,
      }),
    ));
  }

  let client = Client::builder()
    .timeout(Duration::from_secs(5))
    .build()
    .map_err(|e| {
      (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ApiResponse {
          status: "error".into(),
          message: format!("Failed to create HTTP client: {}", e),
          data: None,
        }),
      )
    })?;

  let url = format!("{}/exchange-rate", SWIFTNESS_API_URL);

  match client.put(&url).json(&update).send().await {
    Ok(response) => {
      if response.status().is_success() {
        Ok(Json(ApiResponse {
          status: "ok".into(),
          message: "Exchange rate updated".into(),
          data: Some(serde_json::json!({ "rate": update.rate })),
        }))
      } else {
        Err((
          StatusCode::BAD_GATEWAY,
          Json(ApiResponse {
            status: "error".into(),
            message: format!("Swiftness API returned error: {}", response.status()),
            data: None,
          }),
        ))
      }
    }
    Err(e) => {
      warn!(%e, "failed to call Swiftness API");
      Err((
        StatusCode::BAD_GATEWAY,
        Json(ApiResponse {
          status: "error".into(),
          message: format!("Swiftness API unavailable: {}", e),
          data: None,
        }),
      ))
    }
  }
}
