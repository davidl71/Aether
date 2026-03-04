use std::{net::SocketAddr, sync::Arc, time::Duration};

use axum::{
    extract::{Extension, Path, Query},
    http::StatusCode,
    routing::{get, post, put},
    Json, Router,
};
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

#[derive(Clone)]
pub struct RestState {
    pub snapshot: SharedSnapshot,
    pub controller: StrategyController,
}

// Ensure RestState is Send + Sync for axum Router requirements
// SharedSnapshot is Arc<RwLock<...>> which is Send + Sync
// StrategyController contains Arc<...> which is Send + Sync
// So RestState is automatically Send + Sync

impl RestState {
    pub fn new(snapshot: SharedSnapshot, controller: StrategyController) -> Self {
        Self {
            snapshot,
            controller,
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
            .route("/api/v1/chart/:symbol", get(get_chart))
            .route("/api/v1/swiftness/positions", get(swiftness_positions))
            .route(
                "/api/v1/swiftness/portfolio-value",
                get(swiftness_portfolio_value),
            )
            .route("/api/v1/swiftness/validate", get(swiftness_validate))
            .route(
                "/api/v1/swiftness/exchange-rate",
                get(swiftness_exchange_rate),
            )
            .route(
                "/api/v1/swiftness/exchange-rate",
                put(swiftness_update_exchange_rate),
            )
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
    nats: String,
}

async fn health(
    Extension(state): Extension<RestState>,
) -> Result<Json<HealthResponse>, StatusCode> {
    // Check NATS server health (non-blocking, 1 second timeout)
    let nats_status = check_nats_health().await;

    // Update metrics with NATS status
    {
        let mut snapshot = state.snapshot.write().await;
        snapshot.metrics.nats_ok = nats_status == "ok";
    }

    let response = HealthResponse {
        status: "ok".to_string(),
        components: ComponentHealth {
            backend: "ok".to_string(),
            nats: nats_status,
        },
    };

    Ok(Json(response))
}

async fn check_nats_health() -> String {
    const NATS_HEALTH_URL: &str = "http://localhost:8222/healthz";
    const TIMEOUT_SECS: u64 = 1;

    match timeout(
        Duration::from_secs(TIMEOUT_SECS),
        reqwest::get(NATS_HEALTH_URL),
    )
    .await
    {
        Ok(Ok(response)) => {
            if response.status().is_success() {
                "ok".to_string()
            } else {
                warn!(
                    "NATS health check returned non-success status: {}",
                    response.status()
                );
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

async fn strategy_start(
    Extension(state): Extension<RestState>,
) -> Result<StatusCode, (StatusCode, String)> {
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
        snapshot
            .alerts
            .push(Alert::info("Strategy started via REST"));
        trim_alerts(&mut snapshot.alerts);
    }

    Ok(StatusCode::NO_CONTENT)
}

async fn strategy_stop(
    Extension(state): Extension<RestState>,
) -> Result<StatusCode, (StatusCode, String)> {
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
        snapshot
            .alerts
            .push(Alert::info("Strategy paused via REST"));
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
        started_at: Some(snapshot.started_at),
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
    let order = snapshot.orders.iter().find(|o| o.id == order_id).cloned();

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
    if let Some(order) = snapshot
        .orders
        .iter_mut()
        .find(|o| o.id == request.order_id)
    {
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
                message: format!(
                    "Invalid mode: {}. Must be one of: {:?}",
                    request.mode, valid_modes
                ),
                data: None,
            }),
        ));
    }

    let mut snapshot = state.snapshot.write().await;
    snapshot.mode = request.mode.clone();
    snapshot.touch();
    snapshot.alerts.push(Alert::info(&format!(
        "Mode changed to {} via REST",
        request.mode
    )));
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

    if let Some(strategy_cfg) = &request.strategy {
        if let Some(status) = strategy_cfg.get("status").and_then(|v| v.as_str()) {
            snapshot.strategy = status.to_string();
            snapshot.alerts.push(Alert::info(format!(
                "Strategy status updated to '{}'",
                status
            )));
        }
        if let Some(symbols) = strategy_cfg.get("symbols").and_then(|v| v.as_array()) {
            let new_symbols: Vec<String> = symbols
                .iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect();
            if !new_symbols.is_empty() {
                snapshot
                    .alerts
                    .push(Alert::info(format!("Watchlist updated: {:?}", new_symbols)));
            }
        }
    }

    if let Some(risk_cfg) = &request.risk {
        if let Some(allowed) = risk_cfg.get("allowed").and_then(|v| v.as_bool()) {
            snapshot.risk.allowed = allowed;
            snapshot.alerts.push(Alert::info(format!(
                "Risk trading allowed set to {}",
                allowed
            )));
        }
        if let Some(reason) = risk_cfg.get("reason").and_then(|v| v.as_str()) {
            snapshot.risk.reason = Some(reason.to_string());
        }
        snapshot.risk.updated_at = Utc::now();
    }

    snapshot.touch();
    snapshot
        .alerts
        .push(Alert::info("Configuration updated via REST"));
    trim_alerts(&mut snapshot.alerts);

    Ok(Json(ApiResponse {
        status: "ok".into(),
        message: "Configuration updated".into(),
        data: None,
    }))
}

async fn get_scenarios(
    Extension(state): Extension<RestState>,
    Query(params): Query<ScenariosQuery>,
) -> Json<serde_json::Value> {
    let snapshot = state.snapshot.read().await;
    let underlying = params.symbol.unwrap_or_else(|| "SPX".to_string());
    let min_apr = params.min_apr.unwrap_or(0.0);

    // Build scenario list from current positions and symbol snapshots.
    // Each position that is an options spread contributes a scenario
    // showing its implied APR vs the risk-free benchmark.
    let mut scenarios = Vec::<serde_json::Value>::new();

    for position in &snapshot.positions {
        if position.symbol.contains(&underlying) || underlying == "SPX" {
            let cost = position.cost_basis.abs();
            if cost < 1e-6 {
                continue;
            }

            let profit = position.mark - position.cost_basis;
            let roi_pct = (profit / cost) * 100.0;
            let annualized_apr = roi_pct * 4.0; // rough quarterly-to-annual

            if annualized_apr < min_apr {
                continue;
            }

            scenarios.push(serde_json::json!({
              "symbol": position.symbol,
              "cost_basis": position.cost_basis,
              "current_mark": position.mark,
              "unrealized_pnl": position.unrealized_pnl,
              "roi_percent": roi_pct,
              "annualized_apr": annualized_apr,
              "quantity": position.quantity,
            }));
        }
    }

    // If no live positions, generate indicative scenarios from symbol snapshots.
    if scenarios.is_empty() {
        for sym in &snapshot.symbols {
            if sym.symbol.contains(&underlying) || underlying == "SPX" {
                let strike_widths = [5.0, 10.0, 25.0, 50.0];
                for &width in &strike_widths {
                    let theoretical = width * 100.0;
                    let mid = sym.last;
                    if mid <= 0.0 {
                        continue;
                    }
                    let net_debit = theoretical - (mid * 0.001 * width);
                    let implied_apr = if net_debit > 0.0 {
                        ((theoretical - net_debit) / net_debit) * (365.0 / 30.0) * 100.0
                    } else {
                        0.0
                    };

                    if implied_apr < min_apr {
                        continue;
                    }

                    scenarios.push(serde_json::json!({
                      "symbol": sym.symbol,
                      "strike_width": width,
                      "theoretical_value": theoretical,
                      "estimated_net_debit": net_debit,
                      "implied_apr": implied_apr,
                      "type": "indicative",
                    }));
                }
            }
        }
    }

    scenarios.sort_by(|a, b| {
        let apr_a = a
            .get("annualized_apr")
            .or(a.get("implied_apr"))
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);
        let apr_b = b
            .get("annualized_apr")
            .or(b.get("implied_apr"))
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);
        apr_b
            .partial_cmp(&apr_a)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    Json(serde_json::json!({
      "scenarios": scenarios,
      "count": scenarios.len(),
      "as_of": Utc::now(),
      "underlying": underlying,
      "min_apr_filter": min_apr,
    }))
}

// Chart endpoint

#[derive(Debug, Deserialize)]
struct ChartQuery {
    timeframe: Option<String>,
}

async fn get_chart(
    Extension(state): Extension<RestState>,
    Path(symbol): Path<String>,
    Query(params): Query<ChartQuery>,
) -> Json<serde_json::Value> {
    let timeframe = params.timeframe.unwrap_or_else(|| "1D".to_string());
    let num_candles: usize = match timeframe.as_str() {
        "1D" => 48,
        "1W" => 7 * 48,
        "1M" => 30,
        "3M" => 90,
        "1Y" => 252,
        _ => 48,
    };

    let snapshot = state.snapshot.read().await;
    let sym_snap = snapshot.symbols.iter().find(|s| s.symbol == symbol);

    let candles: Vec<serde_json::Value> = match sym_snap {
        Some(sym) => {
            let base_price = sym.candle.close;
            let now = Utc::now();
            let interval_secs: i64 = match timeframe.as_str() {
                "1D" => 1800, // 30-min bars
                "1W" => 1800,
                "1M" => 86400, // daily bars
                "3M" => 86400,
                "1Y" => 86400,
                _ => 1800,
            };

            let mut bars = Vec::with_capacity(num_candles);
            let mut price = base_price * 0.97;
            let step = (base_price - price) / num_candles as f64;

            for i in 0..num_candles {
                let ts = now - chrono::Duration::seconds(interval_secs * (num_candles - i) as i64);
                let noise =
                    ((i as f64 * 0.7).sin() * 0.005 + (i as f64 * 1.3).cos() * 0.003) * base_price;
                let open = price + noise;
                let close = price + step + noise * 0.5;
                let high = open.max(close) + (base_price * 0.002);
                let low = open.min(close) - (base_price * 0.002);
                let volume = 1000 + ((i * 137 + 42) % 5000) as u64;

                bars.push(serde_json::json!({
                  "time": ts.to_rfc3339(),
                  "open": (open * 100.0).round() / 100.0,
                  "high": (high * 100.0).round() / 100.0,
                  "low": (low * 100.0).round() / 100.0,
                  "close": (close * 100.0).round() / 100.0,
                  "volume": volume,
                }));
                price += step;
            }
            bars
        }
        None => {
            let now = Utc::now();
            let base_price = 450.0;
            let interval_secs: i64 = if num_candles <= 48 { 1800 } else { 86400 };
            let mut bars = Vec::with_capacity(num_candles);
            let mut price = base_price * 0.97;
            let step = (base_price * 0.03) / num_candles as f64;

            for i in 0..num_candles {
                let ts = now - chrono::Duration::seconds(interval_secs * (num_candles - i) as i64);
                let noise = ((i as f64 * 0.7).sin() * 0.005) * base_price;
                let open = price + noise;
                let close = price + step;
                let high = open.max(close) + 0.8;
                let low = open.min(close) - 0.8;
                bars.push(serde_json::json!({
                  "time": ts.to_rfc3339(),
                  "open": (open * 100.0).round() / 100.0,
                  "high": (high * 100.0).round() / 100.0,
                  "low": (low * 100.0).round() / 100.0,
                  "close": (close * 100.0).round() / 100.0,
                  "volume": 1000 + (i * 100) as u64,
                }));
                price += step;
            }
            bars
        }
    };

    Json(serde_json::json!({
      "candles": candles,
      "symbol": symbol,
      "timeframe": timeframe,
      "count": candles.len(),
      "as_of": Utc::now(),
    }))
}

// Swiftness API proxy helper

type SwiftnessResult<T> = Result<T, (StatusCode, Json<ApiResponse>)>;

fn swiftness_error(code: StatusCode, msg: String) -> (StatusCode, Json<ApiResponse>) {
    (
        code,
        Json(ApiResponse {
            status: "error".into(),
            message: msg,
            data: None,
        }),
    )
}

fn swiftness_client() -> SwiftnessResult<Client> {
    Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .map_err(|e| {
            swiftness_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("HTTP client error: {e}"),
            )
        })
}

async fn swiftness_proxy_get(path: &str) -> SwiftnessResult<Json<serde_json::Value>> {
    let client = swiftness_client()?;
    let url = format!("{}{}", SWIFTNESS_API_URL, path);
    match client.get(&url).send().await {
        Ok(resp) if resp.status().is_success() => resp
            .json::<serde_json::Value>()
            .await
            .map(Json)
            .map_err(|e| {
                swiftness_error(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Parse error: {e}"),
                )
            }),
        Ok(resp) => Err(swiftness_error(
            StatusCode::BAD_GATEWAY,
            format!("Swiftness API error: {}", resp.status()),
        )),
        Err(e) => {
            warn!(%e, "failed to call Swiftness API");
            Err(swiftness_error(
                StatusCode::BAD_GATEWAY,
                format!("Swiftness API unavailable: {e}"),
            ))
        }
    }
}

#[derive(Debug, Deserialize)]
struct SwiftnessPositionsQuery {
    check_validity: Option<bool>,
    max_age_days: Option<u32>,
}

async fn swiftness_positions(
    Query(params): Query<SwiftnessPositionsQuery>,
) -> SwiftnessResult<Json<serde_json::Value>> {
    let cv = params.check_validity.unwrap_or(true);
    let max_age = params.max_age_days.unwrap_or(90);
    swiftness_proxy_get(&format!(
        "/positions?check_validity={cv}&max_age_days={max_age}"
    ))
    .await
}

async fn swiftness_portfolio_value() -> SwiftnessResult<Json<serde_json::Value>> {
    swiftness_proxy_get("/portfolio-value").await
}

async fn swiftness_validate() -> SwiftnessResult<Json<serde_json::Value>> {
    swiftness_proxy_get("/validate").await
}

async fn swiftness_exchange_rate() -> SwiftnessResult<Json<serde_json::Value>> {
    swiftness_proxy_get("/exchange-rate").await
}

#[derive(Debug, Deserialize, Serialize)]
struct ExchangeRateUpdate {
    rate: f64,
}

async fn swiftness_update_exchange_rate(
    Json(update): Json<ExchangeRateUpdate>,
) -> SwiftnessResult<Json<ApiResponse>> {
    if update.rate <= 0.0 {
        return Err(swiftness_error(
            StatusCode::BAD_REQUEST,
            "Exchange rate must be positive".into(),
        ));
    }
    let client = swiftness_client()?;
    let url = format!("{}/exchange-rate", SWIFTNESS_API_URL);
    match client.put(&url).json(&update).send().await {
        Ok(resp) if resp.status().is_success() => Ok(Json(ApiResponse {
            status: "ok".into(),
            message: "Exchange rate updated".into(),
            data: Some(serde_json::json!({ "rate": update.rate })),
        })),
        Ok(resp) => Err(swiftness_error(
            StatusCode::BAD_GATEWAY,
            format!("Swiftness API error: {}", resp.status()),
        )),
        Err(e) => {
            warn!(%e, "failed to call Swiftness API");
            Err(swiftness_error(
                StatusCode::BAD_GATEWAY,
                format!("Swiftness API unavailable: {e}"),
            ))
        }
    }
}
