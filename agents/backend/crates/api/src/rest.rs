use std::{net::SocketAddr, sync::Arc, time::Duration};
use std::process::Command;
use std::path::PathBuf;

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
use tracing::{info, warn, error};

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
  pub service_control_enabled: bool,
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
      service_control_enabled: std::env::var("ENABLE_SERVICE_CONTROL")
        .unwrap_or_else(|_| "false".to_string())
        .parse()
        .unwrap_or(false),
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
      .route("/api/v1/services/:service_name/start", post(service_start))
      .route("/api/v1/services/:service_name/stop", post(service_stop))
      .route("/api/v1/services/:service_name/restart", post(service_restart))
      .route("/api/v1/services/:service_name/enable", post(service_enable))
      .route("/api/v1/services/:service_name/disable", post(service_disable))
      .route("/api/v1/services/:service_name/status", get(service_status))
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
  let nats_server_status = check_nats_server_health().await;
  let nats_client_status = if let Some(ref health_check) = state.nats_health_check {
    health_check().await
  } else {
    "unavailable".to_string()
  };
  let nats_overall_ok = nats_server_status == "ok" && nats_client_status == "ok";
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
  match timeout(Duration::from_secs(2), Client::new().get("http://127.0.0.1:8222/healthz").send()).await {
    Ok(Ok(resp)) if resp.status().is_success() => "ok".to_string(),
    _ => "unavailable".to_string(),
  }
}

// Service control request/response types
#[derive(Deserialize)]
struct ServiceControlRequest {
  #[serde(default)]
  force: bool,
}

#[derive(Serialize)]
struct ServiceControlResponse {
  status: String,
  message: String,
  service: ServiceStatusResponse,
}

#[derive(Serialize)]
struct ServiceStatusResponse {
  name: String,
  running: bool,
  enabled: bool,
  port: u16,
  pid: Option<u32>,
}

// Check if systemctl is available for service control
async fn is_systemctl_available() -> bool {
  Command::new("systemctl")
    .arg("--user")
    .arg("--version")
    .output()
    .map(|o| o.status.success())
    .unwrap_or(false)
}

// Execute systemctl command via helper script
async fn execute_systemctl_command(
  action: &str,
  service_name: &str,
) -> anyhow::Result<String> {
  let project_root = std::env::current_dir()?;
  let helper_script = project_root
    .join("web")
    .join("scripts")
    .join("systemd")
    .join("systemctl-helper.sh");

  if !helper_script.exists() {
    return Err(anyhow::anyhow!("systemctl helper script not found"));
  }

  let output = Command::new("bash")
    .arg(&helper_script)
    .arg(action)
    .arg(service_name)
    .current_dir(&project_root)
    .output()?;

  if !output.status.success() {
    let error = String::from_utf8_lossy(&output.stderr);
    return Err(anyhow::anyhow!("systemctl command failed: {}", error));
  }

  Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

// Service name mapping
fn get_service_script_name(service_name: &str) -> Option<(&'static str, &'static str)> {
  match service_name {
    "alpaca" => Some(("start_alpaca_service.sh", "stop_alpaca_service.sh")),
    "tradestation" => Some(("start_tradestation_service.sh", "stop_tradestation_service.sh")),
    "ib" => Some(("start_ib_service.sh", "stop_ib_service.sh")),
    "discount_bank" => Some(("start_discount_bank_service.sh", "stop_discount_bank_service.sh")),
    "risk_free_rate" => Some(("start_risk_free_rate_service.sh", "stop_risk_free_rate_service.sh")),
    "tastytrade" => Some(("start_tastytrade_service.sh", "stop_tastytrade_service.sh")),
    "web" => Some(("", "")), // Web service handled separately
    "nats" => Some(("", "")), // NATS handled separately
    "rust_backend" => Some(("", "")), // Rust backend handled separately
    _ => None,
  }
}

fn get_service_port(service_name: &str) -> Option<u16> {
  match service_name {
    "alpaca" => Some(8000),
    "tradestation" => Some(8001),
    "ib" => Some(8002),
    "discount_bank" => Some(8003),
    "risk_free_rate" => Some(8004),
    "tastytrade" => Some(8005),
    _ => None,
  }
}

async fn get_service_status_internal(service_name: &str) -> ServiceStatusResponse {
  let port = get_service_port(service_name).unwrap_or(0);

  // Try to get status from systemctl first if available
  let (running, enabled) = if is_systemctl_available().await {
    // Map service name to systemd service name
    let systemd_name = match service_name {
      "web" => "web",
      "alpaca" => "alpaca",
      "tradestation" => "tradestation",
      "ib" => "ib",
      "ib_gateway" | "ib-gateway" | "gateway" => "ib-gateway",
      "discount_bank" | "discount-bank" => "discount-bank",
      "risk_free_rate" | "risk-free-rate" => "risk-free-rate",
      "jupyterlab" => "jupyterlab",
      "nats" => "nats",
      "rust_backend" | "rust-backend" => "rust-backend",
      _ => service_name,
    };

    let is_running = execute_systemctl_command("is-active", systemd_name)
      .await
      .map(|s| s.trim() == "active")
      .unwrap_or(false);

    let is_enabled = execute_systemctl_command("is-enabled", systemd_name)
      .await
      .map(|s| s.trim() == "enabled")
      .unwrap_or(false);

    (is_running, is_enabled)
  } else {
    // Fallback to port-based detection
    let pid = get_service_pid(port).await;
    let running = pid.is_some();
    let enabled = is_service_enabled(service_name).await;
    (running, enabled)
  };

  let pid = if running {
    get_service_pid(port).await
  } else {
    None
  };

  ServiceStatusResponse {
    name: service_name.to_string(),
    running,
    enabled,
    port,
    pid,
  }
}

async fn get_service_pid(port: u16) -> Option<u32> {
  let output = Command::new("lsof")
    .arg("-ti")
    .arg(format!(":{}", port))
    .output()
    .ok()?;

  if output.status.success() {
    let pid_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
    pid_str.parse().ok()
  } else {
    None
  }
}

async fn is_service_enabled(service_name: &str) -> bool {
  // Read from config file: config/services.toml
  let config_path = PathBuf::from("config/services.toml");
  if let Ok(content) = tokio::fs::read_to_string(&config_path).await {
    if let Ok(config) = toml::from_str::<toml::Value>(&content) {
      if let Some(services) = config.get("services") {
        if let Some(service) = services.get(service_name) {
          if let Some(enabled) = service.get("enabled") {
            return enabled.as_bool().unwrap_or(true);
          }
        }
      }
    }
  }
  // Default to enabled if config doesn't exist
  true
}

async fn set_service_enabled(service_name: &str, enabled: bool) -> anyhow::Result<()> {
  let config_path = PathBuf::from("config/services.toml");
  let mut config: toml::Value = if config_path.exists() {
    let content = tokio::fs::read_to_string(&config_path).await?;
        toml::from_str(&content).unwrap_or_else(|_| {
      let mut root = toml::map::Map::new();
      root.insert("services".to_string(), toml::Value::Table(toml::map::Map::new()));
      toml::Value::Table(root)
    })
  } else {
    let mut root = toml::map::Map::new();
    root.insert("services".to_string(), toml::Value::Table(toml::map::Map::new()));
    toml::Value::Table(root)
  };

  if let Some(services) = config.get_mut("services").and_then(|s| s.as_table_mut()) {
    if let Some(service) = services.get_mut(service_name).and_then(|s| s.as_table_mut()) {
      service.insert("enabled".to_string(), toml::Value::Boolean(enabled));
    } else {
      let mut service_table = toml::map::Map::new();
      service_table.insert("enabled".to_string(), toml::Value::Boolean(enabled));
      services.insert(service_name.to_string(), toml::Value::Table(service_table));
    }
  }

  // Ensure config directory exists
  if let Some(parent) = config_path.parent() {
    tokio::fs::create_dir_all(parent).await?;
  }

  let toml_content = toml::to_string_pretty(&config)?;
  tokio::fs::write(&config_path, toml_content).await?;
  Ok(())
}

async fn execute_service_script(script_name: &str) -> anyhow::Result<String> {
  let project_root = std::env::current_dir()?;
  let script_path = project_root.join("scripts").join(script_name);

  if !script_path.exists() {
    return Err(anyhow::anyhow!("Script not found: {}", script_path.display()));
  }

  let output = Command::new("bash")
    .arg(&script_path)
    .current_dir(&project_root)
    .output()?;

  if !output.status.success() {
    let error = String::from_utf8_lossy(&output.stderr);
    return Err(anyhow::anyhow!("Script failed: {}", error));
  }

  Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

async fn service_start(
  Extension(state): Extension<RestState>,
  Path(service_name): Path<String>,
  Json(req): Json<ServiceControlRequest>,
) -> Result<Json<ServiceControlResponse>, StatusCode> {
  if !state.service_control_enabled {
    return Err(StatusCode::FORBIDDEN);
  }

  info!(service = %service_name, "Service start requested");

  // Map service name to systemd service name
  let systemd_name = match service_name.as_str() {
    "web" => "web",
    "alpaca" => "alpaca",
    "tradestation" => "tradestation",
    "ib" => "ib",
    "ib_gateway" | "ib-gateway" | "gateway" => "ib-gateway",
    "discount_bank" | "discount-bank" => "discount-bank",
    "risk_free_rate" | "risk-free-rate" => "risk-free-rate",
    "jupyterlab" => "jupyterlab",
    "nats" => "nats",
    "rust_backend" | "rust-backend" => "rust-backend",
    _ => &service_name,
  };

  // Try systemctl first if available
  let result = if is_systemctl_available().await {
    execute_systemctl_command("start", systemd_name).await
  } else {
    // Fallback to script-based approach
    let (start_script, _) = match get_service_script_name(&service_name) {
      Some(scripts) => scripts,
      None => {
        return Err(StatusCode::NOT_FOUND);
      }
    };

    // Check if service is enabled
    let enabled = is_service_enabled(&service_name).await;
    if !enabled {
      return Err(StatusCode::BAD_REQUEST);
    }

    execute_service_script(start_script).await
  };

  match result {
    Ok(_) => {
      // Wait a bit for service to start
      tokio::time::sleep(Duration::from_secs(2)).await;
      let status = get_service_status_internal(&service_name).await;
      Ok(Json(ServiceControlResponse {
        status: "ok".to_string(),
        message: format!("Service {} started", service_name),
        service: status,
      }))
    }
    Err(e) => {
      error!(service = %service_name, error = %e, "Failed to start service");
      Err(StatusCode::INTERNAL_SERVER_ERROR)
    }
  }
}

async fn service_stop(
  Extension(state): Extension<RestState>,
  Path(service_name): Path<String>,
  Json(_req): Json<ServiceControlRequest>,
) -> Result<Json<ServiceControlResponse>, StatusCode> {
  if !state.service_control_enabled {
    return Err(StatusCode::FORBIDDEN);
  }

  info!(service = %service_name, "Service stop requested");

  // Map service name to systemd service name
  let systemd_name = match service_name.as_str() {
    "web" => "web",
    "alpaca" => "alpaca",
    "tradestation" => "tradestation",
    "ib" => "ib",
    "ib_gateway" | "ib-gateway" | "gateway" => "ib-gateway",
    "discount_bank" | "discount-bank" => "discount-bank",
    "risk_free_rate" | "risk-free-rate" => "risk-free-rate",
    "jupyterlab" => "jupyterlab",
    "nats" => "nats",
    "rust_backend" | "rust-backend" => "rust-backend",
    _ => &service_name,
  };

  // Try systemctl first if available
  let result = if is_systemctl_available().await {
    execute_systemctl_command("stop", systemd_name).await
  } else {
    // Fallback to script-based approach
    let (_, stop_script) = match get_service_script_name(&service_name) {
      Some(scripts) => scripts,
      None => {
        return Err(StatusCode::NOT_FOUND);
      }
    };
    execute_service_script(stop_script).await
  };

  match result {
    Ok(_) => {
      tokio::time::sleep(Duration::from_millis(500)).await;
      let status = get_service_status_internal(&service_name).await;
      Ok(Json(ServiceControlResponse {
        status: "ok".to_string(),
        message: format!("Service {} stopped", service_name),
        service: status,
      }))
    }
    Err(e) => {
      error!(service = %service_name, error = %e, "Failed to stop service");
      Err(StatusCode::INTERNAL_SERVER_ERROR)
    }
  }
}

async fn service_restart(
  Extension(state): Extension<RestState>,
  Path(service_name): Path<String>,
  Json(req): Json<ServiceControlRequest>,
) -> Result<Json<ServiceControlResponse>, StatusCode> {
  if !state.service_control_enabled {
    return Err(StatusCode::FORBIDDEN);
  }

  info!(service = %service_name, "Service restart requested");

  // Map service name to systemd service name
  let systemd_name = match service_name.as_str() {
    "web" => "web",
    "alpaca" => "alpaca",
    "tradestation" => "tradestation",
    "ib" => "ib",
    "ib_gateway" | "ib-gateway" | "gateway" => "ib-gateway",
    "discount_bank" | "discount-bank" => "discount-bank",
    "risk_free_rate" | "risk-free-rate" => "risk-free-rate",
    "jupyterlab" => "jupyterlab",
    "nats" => "nats",
    "rust_backend" | "rust-backend" => "rust-backend",
    _ => &service_name,
  };

  // Try systemctl first if available
  let result = if is_systemctl_available().await {
    execute_systemctl_command("restart", systemd_name).await
  } else {
    // Fallback to script-based approach
    let (_, stop_script) = match get_service_script_name(&service_name) {
      Some(scripts) => scripts,
      None => {
        return Err(StatusCode::NOT_FOUND);
      }
    };
    let _ = execute_service_script(stop_script).await;
    tokio::time::sleep(Duration::from_secs(1)).await;

    let (start_script, _) = match get_service_script_name(&service_name) {
      Some(scripts) => scripts,
      None => {
        return Err(StatusCode::NOT_FOUND);
      }
    };
    execute_service_script(start_script).await
  };

  match result {
    Ok(_) => {
      tokio::time::sleep(Duration::from_secs(2)).await;
      let status = get_service_status_internal(&service_name).await;
      Ok(Json(ServiceControlResponse {
        status: "ok".to_string(),
        message: format!("Service {} restarted", service_name),
        service: status,
      }))
    }
    Err(e) => {
      error!(service = %service_name, error = %e, "Failed to restart service");
      Err(StatusCode::INTERNAL_SERVER_ERROR)
    }
  }
}

async fn service_enable(
  Extension(state): Extension<RestState>,
  Path(service_name): Path<String>,
) -> Result<Json<ServiceControlResponse>, StatusCode> {
  if !state.service_control_enabled {
    return Err(StatusCode::FORBIDDEN);
  }

  info!(service = %service_name, "Service enable requested");

  // Map service name to systemd service name
  let systemd_name = match service_name.as_str() {
    "web" => "web",
    "alpaca" => "alpaca",
    "tradestation" => "tradestation",
    "ib" => "ib",
    "ib_gateway" | "ib-gateway" | "gateway" => "ib-gateway",
    "discount_bank" | "discount-bank" => "discount-bank",
    "risk_free_rate" | "risk-free-rate" => "risk-free-rate",
    "jupyterlab" => "jupyterlab",
    "nats" => "nats",
    "rust_backend" | "rust-backend" => "rust-backend",
    _ => &service_name,
  };

  // Try systemctl first if available
  if is_systemctl_available().await {
    if let Err(e) = execute_systemctl_command("enable", systemd_name).await {
      error!(service = %service_name, error = %e, "Failed to enable service via systemctl");
      return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }
  }

  // Also update config file
  match set_service_enabled(&service_name, true).await {
    Ok(_) => {
      let status = get_service_status_internal(&service_name).await;
      Ok(Json(ServiceControlResponse {
        status: "ok".to_string(),
        message: format!("Service {} enabled", service_name),
        service: status,
      }))
    }
    Err(e) => {
      error!(service = %service_name, error = %e, "Failed to enable service");
      Err(StatusCode::INTERNAL_SERVER_ERROR)
    }
  }
}

async fn service_disable(
  Extension(state): Extension<RestState>,
  Path(service_name): Path<String>,
) -> Result<Json<ServiceControlResponse>, StatusCode> {
  if !state.service_control_enabled {
    return Err(StatusCode::FORBIDDEN);
  }

  info!(service = %service_name, "Service disable requested");

  // Map service name to systemd service name
  let systemd_name = match service_name.as_str() {
    "web" => "web",
    "alpaca" => "alpaca",
    "tradestation" => "tradestation",
    "ib" => "ib",
    "ib_gateway" | "ib-gateway" | "gateway" => "ib-gateway",
    "discount_bank" | "discount-bank" => "discount-bank",
    "risk_free_rate" | "risk-free-rate" => "risk-free-rate",
    "jupyterlab" => "jupyterlab",
    "nats" => "nats",
    "rust_backend" | "rust-backend" => "rust-backend",
    _ => &service_name,
  };

  // Stop the service first if it's running
  if is_systemctl_available().await {
    let _ = execute_systemctl_command("stop", systemd_name).await;
    let _ = execute_systemctl_command("disable", systemd_name).await;
  } else {
    // Fallback to script-based approach
    let (_, stop_script) = match get_service_script_name(&service_name) {
      Some(scripts) => scripts,
      None => {
        return Err(StatusCode::NOT_FOUND);
      }
    };
    let _ = execute_service_script(stop_script).await;
  }

  // Also update config file
  match set_service_enabled(&service_name, false).await {
    Ok(_) => {
      let status = get_service_status_internal(&service_name).await;
      Ok(Json(ServiceControlResponse {
        status: "ok".to_string(),
        message: format!("Service {} disabled", service_name),
        service: status,
      }))
    }
    Err(e) => {
      error!(service = %service_name, error = %e, "Failed to disable service");
      Err(StatusCode::INTERNAL_SERVER_ERROR)
    }
  }
}

async fn service_status(
  Extension(_state): Extension<RestState>,
  Path(service_name): Path<String>,
) -> Result<Json<ServiceStatusResponse>, StatusCode> {
  let status = get_service_status_internal(&service_name).await;
  Ok(Json(status))
}

// ... existing code for other endpoints (snapshot, strategy_start, etc.) ...
// [Keep all existing endpoint implementations below]
