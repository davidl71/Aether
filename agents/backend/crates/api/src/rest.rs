use std::{net::SocketAddr, sync::Arc, time::Duration};

use async_stream::stream;
use axum::{
    extract::{Extension, Path, Query},
    http::StatusCode,
    response::sse::{Event, KeepAlive, Sse},
    routing::{get, post, put},
    Json, Router,
};
use base64::{engine::general_purpose::STANDARD as BASE64_STANDARD, Engine as _};
use chrono::{Datelike, Utc};
use futures::{Stream, StreamExt, TryStreamExt};
use nats_adapter::{async_nats, decode_proto, proto::v1::NatsEnvelope};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tokio::{sync::watch, task::JoinHandle, time::timeout};
use tracing::{info, warn};

use crate::discount_bank::{
    get_balance as get_discount_bank_balance, get_bank_accounts as get_discount_bank_accounts,
    get_transactions as get_discount_bank_transactions,
    import_positions as import_discount_bank_positions, ImportPositionsQuery,
};
use crate::finance_rates::{
    build_curve as build_risk_free_curve, compare_rates as compare_risk_free_rates,
    extract_rate as extract_risk_free_rate, get_sofr_rates as fetch_sofr_benchmarks,
    get_treasury_rates as fetch_treasury_benchmarks,
    yield_curve_comparison as compare_yield_curves, BoxSpreadInput, CompareRequest, CurveQuery,
    CurveRequest, YieldCurveComparisonRequest,
};
use crate::health::SharedHealthAggregate;
use crate::ib_positions::fetch_ib_positions;
use crate::loans::{LoanAggregationInput, LoanRecord, LoanRepository};
use crate::project_paths::shared_config_candidate_paths;
use crate::quant::{
    calculate_greeks as calc_greeks, calculate_iv as calc_iv, GreeksRequest, IvRequest,
};
use crate::runtime_state::{
    RuntimeExecutionState, RuntimeOrderDto, RuntimePositionDto, RuntimeSnapshotDto,
};
use crate::state::{Alert, SharedSnapshot};
use crate::websocket::WebSocketServer;

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
    pub loans: LoanRepository,
    pub health: SharedHealthAggregate,
}

// Ensure RestState is Send + Sync for axum Router requirements
// SharedSnapshot is Arc<RwLock<...>> which is Send + Sync
// StrategyController contains Arc<...> which is Send + Sync
// So RestState is automatically Send + Sync

impl RestState {
    pub fn new(
        snapshot: SharedSnapshot,
        controller: StrategyController,
        loans: LoanRepository,
        health: SharedHealthAggregate,
    ) -> Self {
        Self {
            snapshot,
            controller,
            loans,
            health,
        }
    }
}

pub struct RestServer;

impl RestServer {
    pub fn router(state: RestState) -> Router<()> {
        Router::new()
            .route("/health", get(health))
            .route("/gateway/health", get(gateway_health))
            .route("/api/health-aggregated", get(health_aggregated))
            .route("/api/heartbeat", get(health_aggregated))
            .route("/api/heartbeat/*path", get(health_heartbeat_path))
            .route("/api/config", get(shared_config))
            .route("/api/balance", get(discount_bank_balance))
            .route("/api/transactions", get(discount_bank_transactions))
            .route("/api/bank-accounts", get(discount_bank_bank_accounts))
            .route("/api/import-positions", get(discount_bank_import_positions))
            .route("/api/extract-rate", post(extract_rate))
            .route("/api/build-curve", post(build_curve))
            .route("/api/compare", post(compare_rates))
            .route("/api/yield-curve/comparison", post(yield_curve_comparison))
            .route("/api/calculate/greeks", post(calculate_greeks))
            .route("/api/calculate/iv", post(calculate_iv))
            .route("/api/benchmarks/sofr", get(benchmarks_sofr))
            .route("/api/benchmarks/treasury", get(benchmarks_treasury))
            .route("/api/live/state", get(live_state))
            .route("/api/live/state/watch", get(live_state_watch))
            .route("/api/v1/ib/health", get(ib_health))
            .route("/api/v1/ib/snapshot", get(ib_snapshot))
            .route("/api/v1/ib/positions", get(ib_positions))
            .route("/api/v1/positions", get(positions_list))
            .route("/api/v1/positions/:position_id", get(position_details))
            .route("/api/v1/snapshot", get(snapshot))
            .route(
                "/api/v1/cash-flow/timeline",
                post(frontend_cash_flow_timeline),
            )
            .route(
                "/api/v1/opportunity-simulation/scenarios",
                post(frontend_opportunity_scenarios),
            )
            .route(
                "/api/v1/opportunity-simulation/calculate",
                post(frontend_opportunity_calculate),
            )
            .route(
                "/api/v1/frontend/unified-positions",
                post(frontend_unified_positions),
            )
            .route(
                "/api/v1/frontend/relationships",
                post(frontend_relationships),
            )
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
            .route("/api/v1/loans", get(loans_list).post(loans_create))
            .route(
                "/api/v1/loans/:loan_id",
                get(loans_get).put(loans_update).delete(loans_delete),
            )
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
        let app = Self::router(state.clone()).merge(WebSocketServer::route(state));
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

#[derive(Deserialize)]
struct LiveStateQuery {
    key: Option<String>,
}

#[derive(Deserialize)]
struct IbPositionsQuery {
    account_id: Option<String>,
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

#[derive(Serialize)]
struct LiveStateKeyListResponse {
    keys: Vec<String>,
}

#[derive(Serialize)]
struct LiveStateEntryResponse {
    key: String,
    revision: u64,
    value: String,
    envelope: serde_json::Value,
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

#[derive(Serialize)]
struct GatewayHealthResponse {
    status: String,
    as_of: String,
}

async fn live_state(
    Query(query): Query<LiveStateQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorResponse>)> {
    let store = live_state_store().await?;

    if let Some(key) = query.key {
        let entry = store
            .entry(key.clone())
            .await
            .map_err(live_state_internal_error)?;

        if let Some(entry) = entry {
            return Ok(Json(serde_json::json!(LiveStateEntryResponse {
                key: entry.key,
                revision: entry.revision,
                value: BASE64_STANDARD.encode(entry.value.as_ref()),
                envelope: decode_live_state_envelope_metadata(entry.value.as_ref()),
            })));
        }

        return Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "key not found".into(),
            }),
        ));
    }

    let keys = store.keys().await.map_err(live_state_internal_error)?;
    let keys = keys
        .try_collect::<Vec<String>>()
        .await
        .map_err(live_state_internal_error)?;

    Ok(Json(serde_json::json!(LiveStateKeyListResponse { keys })))
}

async fn live_state_watch() -> Result<
    Sse<impl Stream<Item = Result<Event, std::convert::Infallible>>>,
    (StatusCode, Json<ErrorResponse>),
> {
    let client = live_state_client().await?;
    let stream = stream! {
        let jetstream = async_nats::jetstream::new(client);
        let store = match jetstream.get_key_value("LIVE_STATE").await {
            Ok(store) => store,
            Err(err) => {
                yield Ok(Event::default()
                    .event("error")
                    .data(serde_json::json!({
                        "error": format!("LIVE_STATE unavailable: {err}")
                    }).to_string()));
                return;
            }
        };

        yield Ok(Event::default().event("synced").data("{}"));

        let mut watch = match store.watch_all().await {
            Ok(watch) => watch,
            Err(err) => {
                yield Ok(Event::default()
                    .event("error")
                    .data(serde_json::json!({
                        "error": format!("watch failed: {err}")
                    }).to_string()));
                return;
            }
        };

        while let Some(update) = watch.next().await {
            match update {
                Ok(entry) => {
                    let payload = serde_json::json!({
                        "key": entry.key,
                        "revision": entry.revision,
                        "value": BASE64_STANDARD.encode(entry.value.as_ref()),
                        "envelope": decode_live_state_envelope_metadata(entry.value.as_ref()),
                    });
                    yield Ok(Event::default().data(payload.to_string()));
                }
                Err(err) => {
                    yield Ok(Event::default()
                        .event("error")
                        .data(serde_json::json!({
                            "error": format!("watch failed: {err}")
                        }).to_string()));
                    return;
                }
            }
        }
    };

    Ok(Sse::new(stream).keep_alive(KeepAlive::default()))
}

async fn gateway_health() -> Json<GatewayHealthResponse> {
    Json(GatewayHealthResponse {
        status: "ok".into(),
        as_of: Utc::now().to_rfc3339(),
    })
}

async fn health_aggregated(
    Extension(state): Extension<RestState>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorResponse>)> {
    let health = state.health.read().await;
    let response = health.response();
    Ok(Json(
        serde_json::to_value(response).map_err(live_state_internal_error)?,
    ))
}

async fn health_heartbeat_path(
    Path(path): Path<String>,
    Extension(state): Extension<RestState>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorResponse>)> {
    let normalized = path.trim_matches('/');
    let health = state.health.read().await;

    if normalized.is_empty()
        || normalized == "health"
        || normalized == "dashboard"
        || normalized == "health/dashboard"
    {
        let response = health.response();
        return Ok(Json(
            serde_json::to_value(response).map_err(live_state_internal_error)?,
        ));
    }

    let backend_key = normalized.strip_prefix("health/").unwrap_or(normalized);

    if let Some(backend) = health.backends.get(backend_key) {
        return Ok(Json(
            serde_json::to_value(backend).map_err(live_state_internal_error)?,
        ));
    }

    Err((
        StatusCode::NOT_FOUND,
        Json(ErrorResponse {
            error: format!("Backend {backend_key} not in health map"),
        }),
    ))
}

async fn ib_health(
    Extension(state): Extension<RestState>,
) -> Result<Json<HealthResponse>, StatusCode> {
    health(Extension(state)).await
}

async fn ib_snapshot(Extension(state): Extension<RestState>) -> Json<RuntimeSnapshotDto> {
    snapshot(Extension(state)).await
}

async fn ib_positions(
    Query(query): Query<IbPositionsQuery>,
) -> Result<Json<Vec<crate::IbPositionDto>>, (StatusCode, Json<ErrorResponse>)> {
    let positions = fetch_ib_positions(query.account_id.as_deref())
        .await
        .map_err(|error| (StatusCode::BAD_GATEWAY, Json(ErrorResponse { error })))?;
    Ok(Json(positions))
}

async fn extract_rate(
    Json(payload): Json<BoxSpreadInput>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorResponse>)> {
    let response = extract_risk_free_rate(payload)
        .map_err(|error| (StatusCode::BAD_REQUEST, Json(ErrorResponse { error })))?;
    Ok(Json(
        serde_json::to_value(response).map_err(live_state_internal_error)?,
    ))
}

async fn build_curve(
    query: Option<Query<CurveQuery>>,
    Json(payload): Json<CurveRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorResponse>)> {
    let response = build_risk_free_curve(payload, query.map(|item| item.0))
        .map_err(|error| (StatusCode::BAD_REQUEST, Json(ErrorResponse { error })))?;
    Ok(Json(
        serde_json::to_value(response).map_err(live_state_internal_error)?,
    ))
}

async fn compare_rates(
    query: Option<Query<CurveQuery>>,
    Json(payload): Json<CompareRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorResponse>)> {
    let client = Client::new();
    let response = compare_risk_free_rates(payload, query.map(|item| item.0), &client)
        .await
        .map_err(|error| (StatusCode::BAD_REQUEST, Json(ErrorResponse { error })))?;
    Ok(Json(
        serde_json::to_value(response).map_err(live_state_internal_error)?,
    ))
}

async fn yield_curve_comparison(
    Json(payload): Json<YieldCurveComparisonRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorResponse>)> {
    let client = Client::new();
    let response = compare_yield_curves(payload, &client).await;
    Ok(Json(
        serde_json::to_value(response).map_err(live_state_internal_error)?,
    ))
}

async fn calculate_greeks(
    Json(payload): Json<GreeksRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorResponse>)> {
    let response = calc_greeks(&payload).map_err(|error| (StatusCode::BAD_REQUEST, Json(ErrorResponse { error })))?;
    Ok(Json(serde_json::to_value(response).map_err(live_state_internal_error)?))
}

async fn calculate_iv(
    Json(payload): Json<IvRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorResponse>)> {
    let response = calc_iv(&payload).map_err(|error| (StatusCode::BAD_REQUEST, Json(ErrorResponse { error })))?;
    Ok(Json(serde_json::to_value(response).map_err(live_state_internal_error)?))
}

async fn benchmarks_sofr() -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorResponse>)> {
    let client = Client::new();
    let response = fetch_sofr_benchmarks(&client).await;
    Ok(Json(
        serde_json::to_value(response).map_err(live_state_internal_error)?,
    ))
}

async fn benchmarks_treasury() -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorResponse>)>
{
    let client = Client::new();
    let response = fetch_treasury_benchmarks(&client).await;
    Ok(Json(
        serde_json::to_value(response).map_err(live_state_internal_error)?,
    ))
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

async fn live_state_client() -> Result<async_nats::Client, (StatusCode, Json<ErrorResponse>)> {
    let nats_url =
        std::env::var("NATS_URL").unwrap_or_else(|_| "nats://localhost:4222".to_string());
    async_nats::connect(&nats_url).await.map_err(|err| {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ErrorResponse {
                error: format!("LIVE_STATE unavailable: {err}"),
            }),
        )
    })
}

async fn live_state_store(
) -> Result<async_nats::jetstream::kv::Store, (StatusCode, Json<ErrorResponse>)> {
    let client = live_state_client().await?;
    let jetstream = async_nats::jetstream::new(client);
    jetstream.get_key_value("LIVE_STATE").await.map_err(|err| {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ErrorResponse {
                error: format!("LIVE_STATE unavailable: {err}"),
            }),
        )
    })
}

fn live_state_internal_error<E: std::fmt::Display>(err: E) -> (StatusCode, Json<ErrorResponse>) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ErrorResponse {
            error: err.to_string(),
        }),
    )
}

fn decode_live_state_envelope_metadata(bytes: &[u8]) -> serde_json::Value {
    match decode_proto::<NatsEnvelope>(bytes) {
        Ok(envelope) => {
            let timestamp = envelope.timestamp.and_then(|ts| {
                chrono::DateTime::<Utc>::from_timestamp(ts.seconds, ts.nanos as u32)
                    .map(|dt| dt.to_rfc3339_opts(chrono::SecondsFormat::Nanos, true))
            });

            serde_json::json!({
                "id": envelope.id,
                "source": envelope.source,
                "message_type": envelope.message_type,
                "timestamp": timestamp.unwrap_or_default(),
                "payload_b64": BASE64_STANDARD.encode(envelope.payload),
            })
        }
        Err(err) => serde_json::json!({
            "decode_error": err.to_string(),
        }),
    }
}

fn strip_json_comments(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;
    let mut in_double = false;
    let mut in_line = false;
    let mut in_block = false;
    let mut block_depth = 0_i32;

    while i < chars.len() {
        let c = chars[i];
        if in_line {
            if c == '\n' {
                in_line = false;
                out.push(c);
            }
            i += 1;
            continue;
        }
        if in_block {
            if c == '*' && i + 1 < chars.len() && chars[i + 1] == '/' {
                block_depth -= 1;
                if block_depth == 0 {
                    in_block = false;
                }
                i += 2;
            } else if c == '/' && i + 1 < chars.len() && chars[i + 1] == '*' {
                block_depth += 1;
                i += 2;
            } else {
                i += 1;
            }
            continue;
        }
        if in_double {
            if c == '\\' && i + 1 < chars.len() {
                out.push(c);
                out.push(chars[i + 1]);
                i += 2;
            } else if c == '"' {
                in_double = false;
                out.push(c);
                i += 1;
            } else {
                out.push(c);
                i += 1;
            }
            continue;
        }
        if c == '"' {
            in_double = true;
            out.push(c);
            i += 1;
        } else if c == '/' && i + 1 < chars.len() {
            match chars[i + 1] {
                '/' => {
                    in_line = true;
                    i += 2;
                }
                '*' => {
                    in_block = true;
                    block_depth = 1;
                    i += 2;
                }
                _ => {
                    out.push(c);
                    i += 1;
                }
            }
        } else {
            out.push(c);
            i += 1;
        }
    }

    out
}

fn resolve_env_placeholders(value: serde_json::Value) -> serde_json::Value {
    match value {
        serde_json::Value::String(text) => {
            if text.starts_with("${") && text.ends_with('}') {
                let key = &text[2..text.len() - 1];
                if let Ok(resolved) = std::env::var(key) {
                    return serde_json::Value::String(resolved);
                }
            }
            serde_json::Value::String(text)
        }
        serde_json::Value::Array(items) => serde_json::Value::Array(
            items
                .into_iter()
                .map(resolve_env_placeholders)
                .collect::<Vec<_>>(),
        ),
        serde_json::Value::Object(map) => serde_json::Value::Object(
            map.into_iter()
                .map(|(k, v)| (k, resolve_env_placeholders(v)))
                .collect(),
        ),
        other => other,
    }
}

fn load_shared_config_json() -> Result<serde_json::Value, String> {
    let candidates = shared_config_candidate_paths();
    let mut last_error = None;

    for candidate in &candidates {
        if !candidate.exists() || !candidate.is_file() {
            continue;
        }

        match std::fs::read_to_string(candidate) {
            Ok(raw) => {
                let stripped = strip_json_comments(&raw);
                match serde_json::from_str::<serde_json::Value>(&stripped) {
                    Ok(parsed) => return Ok(resolve_env_placeholders(parsed)),
                    Err(err) => {
                        return Err(format!(
                            "Failed to parse shared config at {}: {err}",
                            candidate.display()
                        ));
                    }
                }
            }
            Err(err) => {
                last_error = Some(format!("{}: {err}", candidate.display()));
            }
        }
    }

    let searched = candidates
        .iter()
        .map(|path| path.display().to_string())
        .collect::<Vec<_>>()
        .join(", ");
    Err(last_error.unwrap_or_else(|| format!("Shared config not found. Searched: {searched}")))
}

fn shared_config_response(config: &serde_json::Value) -> serde_json::Value {
    let version = config
        .get("version")
        .cloned()
        .unwrap_or_else(|| serde_json::Value::String("1.0.0".into()));
    let services = config
        .get("services")
        .cloned()
        .unwrap_or_else(|| serde_json::json!({}));

    let broker = config.get("broker");
    let primary = broker
        .and_then(|value| value.get("primary"))
        .cloned()
        .unwrap_or_else(|| serde_json::Value::String("ALPACA".into()));
    let priorities = broker
        .and_then(|value| value.get("priorities"))
        .cloned()
        .unwrap_or_else(|| serde_json::json!(["alpaca", "ib", "mock"]));

    let pwa = config
        .get("pwa")
        .cloned()
        .unwrap_or_else(|| serde_json::json!({}));
    let pwa_service_ports = pwa
        .get("servicePorts")
        .cloned()
        .unwrap_or_else(|| serde_json::json!({}));
    let pwa_default_service = pwa
        .get("defaultService")
        .cloned()
        .unwrap_or_else(|| serde_json::Value::String("ib".into()));
    let pwa_service_urls = pwa
        .get("serviceUrls")
        .cloned()
        .unwrap_or_else(|| serde_json::json!({}));

    serde_json::json!({
        "version": version,
        "services": services,
        "broker": {
            "primary": primary,
            "priorities": priorities,
        },
        "pwa": {
            "servicePorts": pwa_service_ports,
            "defaultService": pwa_default_service,
            "serviceUrls": pwa_service_urls,
        },
    })
}

async fn shared_config() -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorResponse>)> {
    let config = load_shared_config_json().map_err(live_state_internal_error)?;
    Ok(Json(shared_config_response(&config)))
}

#[derive(Deserialize)]
struct DiscountBankTransactionsQuery {
    limit: Option<usize>,
}

async fn discount_bank_balance(
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorResponse>)> {
    let response = get_discount_bank_balance()
        .await
        .map_err(|error| (StatusCode::NOT_FOUND, Json(ErrorResponse { error })))?;
    Ok(Json(
        serde_json::to_value(response).map_err(live_state_internal_error)?,
    ))
}

async fn discount_bank_transactions(
    Query(query): Query<DiscountBankTransactionsQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorResponse>)> {
    let response = get_discount_bank_transactions(query.limit.unwrap_or(20))
        .await
        .map_err(|error| (StatusCode::NOT_FOUND, Json(ErrorResponse { error })))?;
    Ok(Json(
        serde_json::to_value(response).map_err(live_state_internal_error)?,
    ))
}

async fn discount_bank_bank_accounts(
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorResponse>)> {
    let response = get_discount_bank_accounts()
        .await
        .map_err(live_state_internal_error)?;
    Ok(Json(
        serde_json::to_value(response).map_err(live_state_internal_error)?,
    ))
}

async fn discount_bank_import_positions(
    Query(query): Query<ImportPositionsQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorResponse>)> {
    let client = Client::new();
    let response = import_discount_bank_positions(query, &client)
        .await
        .map_err(|error| (StatusCode::BAD_REQUEST, Json(ErrorResponse { error })))?;
    Ok(Json(
        serde_json::to_value(response).map_err(live_state_internal_error)?,
    ))
}

async fn snapshot(Extension(state): Extension<RestState>) -> Json<RuntimeSnapshotDto> {
    let snapshot = state.snapshot.read().await.clone();
    Json(RuntimeSnapshotDto::from(&snapshot))
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
struct LoansListResponse {
    loans: Vec<LoanRecord>,
}

#[derive(Debug, Serialize)]
struct StrategyStatusResponse {
    status: String,
    started_at: Option<chrono::DateTime<Utc>>,
    last_update: chrono::DateTime<Utc>,
}

#[derive(Debug, Serialize)]
struct OrdersListResponse {
    orders: Vec<RuntimeOrderDto>,
}

#[derive(Debug, Serialize)]
struct PositionsListResponse {
    positions: Vec<RuntimePositionDto>,
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

#[derive(Debug, Clone, Deserialize)]
struct FrontendCandleInput {
    #[serde(default)]
    open: Option<f64>,
    #[serde(default)]
    high: Option<f64>,
    #[serde(default)]
    low: Option<f64>,
    #[serde(default)]
    close: Option<f64>,
    #[serde(default)]
    volume: Option<f64>,
    #[serde(default)]
    entry: Option<f64>,
    #[serde(default)]
    updated: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct FrontendPositionInput {
    name: String,
    #[serde(default)]
    quantity: Option<i64>,
    #[serde(default)]
    roi: Option<f64>,
    #[serde(default)]
    maker_count: Option<i64>,
    #[serde(default)]
    taker_count: Option<i64>,
    #[serde(default)]
    rebate_estimate: Option<f64>,
    #[serde(default)]
    vega: Option<f64>,
    #[serde(default)]
    theta: Option<f64>,
    #[serde(default)]
    fair_diff: Option<f64>,
    #[serde(default)]
    maturity_date: Option<String>,
    #[serde(default)]
    cash_flow: Option<f64>,
    #[serde(default)]
    candle: Option<FrontendCandleInput>,
    #[serde(default)]
    instrument_type: Option<String>,
    #[serde(default)]
    rate: Option<f64>,
    #[serde(default)]
    collateral_value: Option<f64>,
    #[serde(default)]
    currency: Option<String>,
    #[serde(default)]
    market_value: Option<f64>,
    #[serde(default)]
    bid: Option<f64>,
    #[serde(default)]
    ask: Option<f64>,
    #[serde(default)]
    last: Option<f64>,
    #[serde(default)]
    spread: Option<f64>,
    #[serde(default)]
    price: Option<f64>,
    #[serde(default)]
    side: Option<String>,
    #[serde(default)]
    expected_cash_at_expiry: Option<f64>,
    #[serde(default)]
    dividend: Option<f64>,
    #[serde(default)]
    conid: Option<i64>,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
struct FrontendBankAccountInput {
    account_name: String,
    balance: f64,
    #[serde(default)]
    account_path: Option<String>,
    #[serde(default)]
    bank_name: Option<String>,
    #[serde(default)]
    account_number: Option<String>,
    #[serde(default)]
    debit_rate: Option<f64>,
    #[serde(default)]
    credit_rate: Option<f64>,
    #[serde(default)]
    currency: Option<String>,
    #[serde(default)]
    balances_by_currency: Option<std::collections::BTreeMap<String, f64>>,
    #[serde(default)]
    is_mixed_currency: bool,
}

#[derive(Debug, Clone, Deserialize)]
struct FrontendViewRequest {
    #[serde(default)]
    positions: Vec<FrontendPositionInput>,
    #[serde(default)]
    bank_accounts: Vec<FrontendBankAccountInput>,
}

#[derive(Debug, Serialize)]
struct FrontendUnifiedPositionsResponse {
    positions: Vec<serde_json::Value>,
}

#[derive(Debug, Serialize)]
struct FrontendRelationship {
    from: String,
    to: String,
    #[serde(rename = "type")]
    relationship_type: String,
    description: String,
    value: f64,
}

#[derive(Debug, Serialize)]
struct FrontendRelationshipResponse {
    relationships: Vec<FrontendRelationship>,
    nodes: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct FrontendCashFlowRequest {
    #[serde(default)]
    positions: Vec<FrontendPositionInput>,
    #[serde(default)]
    bank_accounts: Vec<FrontendBankAccountInput>,
    #[serde(default = "default_projection_months")]
    projection_months: i64,
}

#[derive(Debug, Serialize, Clone)]
struct FrontendCashFlowEvent {
    date: String,
    amount: f64,
    description: String,
    position_name: String,
    #[serde(rename = "type")]
    event_type: String,
}

#[derive(Debug, Serialize, Clone)]
struct FrontendMonthlyCashFlow {
    month: String,
    inflows: f64,
    outflows: f64,
    net: f64,
    events: Vec<FrontendCashFlowEvent>,
}

#[derive(Debug, Serialize)]
struct FrontendCashFlowResponse {
    events: Vec<FrontendCashFlowEvent>,
    monthly_flows: std::collections::BTreeMap<String, FrontendMonthlyCashFlow>,
    total_inflows: f64,
    total_outflows: f64,
    net_cash_flow: f64,
}

#[derive(Debug, Serialize)]
struct FrontendScenario {
    id: String,
    name: String,
    #[serde(rename = "type")]
    scenario_type: String,
    description: String,
    parameters: std::collections::BTreeMap<String, f64>,
    net_benefit: f64,
}

#[derive(Debug, Deserialize)]
struct FrontendScenarioCalculateRequest {
    scenario: FrontendScenarioInput,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct FrontendScenarioInput {
    id: String,
    name: String,
    #[serde(rename = "type")]
    scenario_type: String,
    description: String,
    parameters: std::collections::BTreeMap<String, f64>,
}

#[derive(Debug, Serialize)]
struct FrontendScenarioCalculationResponse {
    net_benefit: f64,
    cash_flow_impact: f64,
    risk_reduction: f64,
    capital_efficiency: Option<f64>,
}

fn default_projection_months() -> i64 {
    12
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
    let runtime_state = RuntimeExecutionState::from_snapshot(&snapshot);
    let mut orders: Vec<RuntimeOrderDto> = runtime_state.order_dtos();

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

async fn positions_list(Extension(state): Extension<RestState>) -> Json<PositionsListResponse> {
    let snapshot = state.snapshot.read().await;
    let runtime_state = RuntimeExecutionState::from_snapshot(&snapshot);
    let positions = runtime_state.position_dtos();
    Json(PositionsListResponse { positions })
}

async fn position_details(
    Extension(state): Extension<RestState>,
    Path(position_id): Path<String>,
) -> Result<Json<RuntimePositionDto>, (StatusCode, Json<ApiResponse>)> {
    let snapshot = state.snapshot.read().await;
    let runtime_state = RuntimeExecutionState::from_snapshot(&snapshot);
    let position = runtime_state.find_position_dto(&position_id);

    match position {
        Some(position) => Ok(Json(position)),
        None => Err((
            StatusCode::NOT_FOUND,
            Json(ApiResponse {
                status: "error".into(),
                message: format!("Position {} not found", position_id),
                data: None,
            }),
        )),
    }
}

async fn order_details(
    Extension(state): Extension<RestState>,
    Path(order_id): Path<String>,
) -> Result<Json<RuntimeOrderDto>, (StatusCode, Json<ApiResponse>)> {
    let snapshot = state.snapshot.read().await;
    let runtime_state = RuntimeExecutionState::from_snapshot(&snapshot);
    let order = runtime_state.find_order_dto(&order_id);

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
        snapshot.alerts.push(Alert::info(format!(
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
    snapshot.alerts.push(Alert::info(format!(
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
    snapshot.alerts.push(Alert::info(format!(
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

async fn loans_list(Extension(state): Extension<RestState>) -> Json<LoansListResponse> {
    Json(LoansListResponse {
        loans: state.loans.list().await,
    })
}

async fn loans_get(
    Path(loan_id): Path<String>,
    Extension(state): Extension<RestState>,
) -> Result<Json<LoanRecord>, (StatusCode, String)> {
    state.loans.get(&loan_id).await.map(Json).ok_or((
        StatusCode::NOT_FOUND,
        format!("Loan with ID {loan_id} not found"),
    ))
}

async fn loans_create(
    Extension(state): Extension<RestState>,
    Json(loan): Json<LoanRecord>,
) -> Result<(StatusCode, Json<LoanRecord>), (StatusCode, String)> {
    state
        .loans
        .create(loan.clone())
        .await
        .map_err(|err| (StatusCode::BAD_REQUEST, err))?;
    Ok((StatusCode::CREATED, Json(loan)))
}

async fn loans_update(
    Path(loan_id): Path<String>,
    Extension(state): Extension<RestState>,
    Json(loan): Json<LoanRecord>,
) -> Result<Json<LoanRecord>, (StatusCode, String)> {
    state
        .loans
        .update(&loan_id, loan.clone())
        .await
        .map_err(|err| {
            let status = if err.contains("not found") {
                StatusCode::NOT_FOUND
            } else {
                StatusCode::BAD_REQUEST
            };
            (status, err)
        })?;
    Ok(Json(loan))
}

async fn loans_delete(
    Path(loan_id): Path<String>,
    Extension(state): Extension<RestState>,
) -> Result<StatusCode, (StatusCode, String)> {
    match state.loans.delete(&loan_id).await {
        Ok(true) => Ok(StatusCode::NO_CONTENT),
        Ok(false) => Err((
            StatusCode::NOT_FOUND,
            format!("Loan with ID {loan_id} not found"),
        )),
        Err(err) => Err((StatusCode::INTERNAL_SERVER_ERROR, err)),
    }
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
    let runtime_state = RuntimeExecutionState::from_snapshot(&snapshot);

    // Build scenario list from current positions and symbol snapshots.
    // Each position that is an options spread contributes a scenario
    // showing its implied APR vs the risk-free benchmark.
    let mut scenarios = Vec::<serde_json::Value>::new();

    for position in &runtime_state.positions {
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

fn default_candle_json() -> serde_json::Value {
    serde_json::json!({
        "open": 0.0,
        "high": 0.0,
        "low": 0.0,
        "close": 0.0,
        "volume": 0.0,
        "entry": 0.0,
        "updated": "",
    })
}

fn candle_to_json(candle: &Option<FrontendCandleInput>) -> serde_json::Value {
    match candle {
        Some(c) => serde_json::json!({
            "open": c.open.unwrap_or(0.0),
            "high": c.high.unwrap_or(0.0),
            "low": c.low.unwrap_or(0.0),
            "close": c.close.unwrap_or(0.0),
            "volume": c.volume.unwrap_or(0.0),
            "entry": c.entry.unwrap_or(0.0),
            "updated": c.updated.clone().unwrap_or_default(),
        }),
        None => default_candle_json(),
    }
}

fn bank_rate(account: &FrontendBankAccountInput) -> Option<f64> {
    match account.credit_rate {
        Some(rate) if rate != 0.0 => Some(rate),
        _ => account.debit_rate,
    }
}

fn make_bank_position(
    account_name: &str,
    amount: f64,
    currency: &str,
    rate: Option<f64>,
    candle: &serde_json::Value,
    name_suffix: Option<&str>,
) -> serde_json::Value {
    let suffix = name_suffix.unwrap_or("");
    serde_json::json!({
        "name": format!("{account_name}{suffix}"),
        "quantity": 1,
        "roi": rate.unwrap_or(0.0) * 100.0,
        "maker_count": 0,
        "taker_count": 0,
        "rebate_estimate": 0.0,
        "vega": 0.0,
        "theta": 0.0,
        "fair_diff": 0.0,
        "candle": candle.clone(),
        "instrument_type": "bank_loan",
        "rate": rate,
        "currency": currency,
        "cash_flow": amount,
    })
}

fn normalize_bank_accounts_to_positions(
    bank_accounts: &[FrontendBankAccountInput],
    reference_candle: &serde_json::Value,
) -> Vec<serde_json::Value> {
    let mut positions = Vec::new();
    for account in bank_accounts {
        let rate = bank_rate(account);
        let account_name = account.account_name.as_str();
        if account.is_mixed_currency {
            if let Some(balances) = &account.balances_by_currency {
                for (currency, amount) in balances {
                    positions.push(make_bank_position(
                        account_name,
                        *amount,
                        if currency.is_empty() {
                            "USD"
                        } else {
                            currency.as_str()
                        },
                        rate,
                        reference_candle,
                        Some(&format!(" ({currency})")),
                    ));
                }
                continue;
            }
        }
        positions.push(make_bank_position(
            account_name,
            account.balance,
            account.currency.as_deref().unwrap_or("USD"),
            rate,
            reference_candle,
            None,
        ));
    }
    positions
}

fn position_value(position: &serde_json::Value) -> f64 {
    position
        .get("cash_flow")
        .and_then(|value| value.as_f64())
        .or_else(|| {
            position
                .get("candle")
                .and_then(|value| value.get("close"))
                .and_then(|value| value.as_f64())
        })
        .unwrap_or(0.0)
}

fn loan_inputs_from_positions(positions: &[FrontendPositionInput]) -> Vec<LoanAggregationInput> {
    positions
        .iter()
        .filter_map(|position| {
            let instrument_type = position.instrument_type.as_deref()?;
            if !matches!(instrument_type, "bank_loan" | "pension_loan") {
                return None;
            }
            Some(LoanAggregationInput {
                loan_id: None,
                name: position.name.clone(),
                instrument_type: instrument_type.to_string(),
                principal: position_cash_value(position),
                annual_rate: position.rate.unwrap_or(0.0),
                monthly_payment: None,
                maturity_date: position.maturity_date.clone(),
            })
        })
        .collect()
}

fn loan_inputs_from_bank_accounts(
    bank_accounts: &[FrontendBankAccountInput],
) -> Vec<LoanAggregationInput> {
    bank_accounts
        .iter()
        .filter_map(|account| {
            let annual_rate = account.debit_rate?;
            if annual_rate <= 0.0 {
                return None;
            }
            Some(LoanAggregationInput {
                loan_id: None,
                name: account.account_name.clone(),
                instrument_type: "bank_loan".into(),
                principal: account.balance,
                annual_rate,
                monthly_payment: None,
                maturity_date: None,
            })
        })
        .collect()
}

fn loan_inputs_from_repository(loans: &[LoanRecord]) -> Vec<LoanAggregationInput> {
    loans.iter().map(LoanRecord::to_aggregation_input).collect()
}

fn is_loan_instrument(instrument_type: Option<&str>) -> bool {
    matches!(instrument_type, Some("bank_loan" | "pension_loan"))
}

fn effective_frontend_positions<'a>(
    positions: &'a [FrontendPositionInput],
    repository_loans: &[LoanRecord],
) -> Vec<&'a FrontendPositionInput> {
    positions
        .iter()
        .filter(|position| {
            !(!repository_loans.is_empty()
                && is_loan_instrument(position.instrument_type.as_deref()))
        })
        .collect()
}

fn aggregated_loan_inputs(
    positions: &[FrontendPositionInput],
    bank_accounts: &[FrontendBankAccountInput],
    repository_loans: &[LoanRecord],
) -> Vec<LoanAggregationInput> {
    let mut loans = if repository_loans.is_empty() {
        loan_inputs_from_positions(positions)
    } else {
        loan_inputs_from_repository(repository_loans)
    };
    loans.extend(loan_inputs_from_bank_accounts(bank_accounts));
    loans
}

fn position_cash_value(position: &FrontendPositionInput) -> f64 {
    position
        .cash_flow
        .or_else(|| position.candle.as_ref().and_then(|candle| candle.close))
        .unwrap_or(0.0)
}

fn parse_frontend_date(date: &str) -> Option<chrono::DateTime<chrono::Utc>> {
    chrono::DateTime::parse_from_rfc3339(date)
        .map(|dt| dt.with_timezone(&chrono::Utc))
        .ok()
        .or_else(|| {
            chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d")
                .ok()
                .and_then(|d| d.and_hms_opt(0, 0, 0))
                .map(|dt| {
                    chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(dt, chrono::Utc)
                })
        })
}

fn date_only(date: &chrono::DateTime<chrono::Utc>) -> String {
    date.format("%Y-%m-%d").to_string()
}

#[cfg(test)]
fn build_cash_flow_response(request: &FrontendCashFlowRequest) -> FrontendCashFlowResponse {
    build_cash_flow_response_with_loans(request, &[])
}

fn build_cash_flow_response_with_loans(
    request: &FrontendCashFlowRequest,
    repository_loans: &[LoanRecord],
) -> FrontendCashFlowResponse {
    let now = chrono::Utc::now();
    let projection_months = request.projection_months.max(0);
    let mut events = Vec::new();
    let loan_inputs =
        aggregated_loan_inputs(&request.positions, &request.bank_accounts, repository_loans);
    let positions = effective_frontend_positions(&request.positions, repository_loans);

    for position in positions {
        if let Some(maturity_date_str) = &position.maturity_date {
            if let Some(maturity_date) = parse_frontend_date(maturity_date_str) {
                let months_ahead = i64::from(maturity_date.year() - now.year()) * 12
                    + i64::from(maturity_date.month() as i32 - now.month() as i32);

                if (0..=projection_months).contains(&months_ahead) {
                    events.push(FrontendCashFlowEvent {
                        date: date_only(&maturity_date),
                        amount: position_cash_value(position),
                        description: format!(
                            "{} maturity",
                            position.instrument_type.as_deref().unwrap_or("Position")
                        ),
                        position_name: position.name.clone(),
                        event_type: "maturity".into(),
                    });
                }
            }
        }

        if let Some(cash_flow) = position.cash_flow.filter(|value| *value != 0.0) {
            events.push(FrontendCashFlowEvent {
                date: date_only(&now),
                amount: cash_flow,
                description: format!(
                    "Current {} cash flow",
                    position.instrument_type.as_deref().unwrap_or("position")
                ),
                position_name: position.name.clone(),
                event_type: "other".into(),
            });
        }
    }

    for loan in &loan_inputs {
        if let Some(maturity_date_str) = &loan.maturity_date {
            if let Some(maturity_date) = parse_frontend_date(maturity_date_str) {
                let months_ahead = i64::from(maturity_date.year() - now.year()) * 12
                    + i64::from(maturity_date.month() as i32 - now.month() as i32);
                if (0..=projection_months).contains(&months_ahead) {
                    events.push(FrontendCashFlowEvent {
                        date: date_only(&maturity_date),
                        amount: loan.principal,
                        description: format!("{} maturity", loan.instrument_type),
                        position_name: loan.name.clone(),
                        event_type: "maturity".into(),
                    });
                }
            }
        }
        for month in 1..=projection_months {
            let payment_date = now + chrono::Duration::days(30 * month);
            events.push(FrontendCashFlowEvent {
                date: date_only(&payment_date),
                amount: -loan.monthly_interest_payment(),
                description: "Monthly interest payment".into(),
                position_name: loan.name.clone(),
                event_type: "loan_payment".into(),
            });
        }
    }

    let mut monthly_flows = std::collections::BTreeMap::<String, FrontendMonthlyCashFlow>::new();
    for event in &events {
        let month = event.date.chars().take(7).collect::<String>();
        let entry = monthly_flows
            .entry(month.clone())
            .or_insert_with(|| FrontendMonthlyCashFlow {
                month,
                inflows: 0.0,
                outflows: 0.0,
                net: 0.0,
                events: Vec::new(),
            });
        entry.events.push(event.clone());
        if event.amount > 0.0 {
            entry.inflows += event.amount;
        } else {
            entry.outflows += event.amount.abs();
        }
        entry.net = entry.inflows - entry.outflows;
    }

    let total_inflows = monthly_flows.values().map(|m| m.inflows).sum::<f64>();
    let total_outflows = monthly_flows.values().map(|m| m.outflows).sum::<f64>();

    FrontendCashFlowResponse {
        events,
        monthly_flows,
        total_inflows,
        total_outflows,
        net_cash_flow: total_inflows - total_outflows,
    }
}

fn scenario_net_benefit(
    scenario_type: &str,
    parameters: &std::collections::BTreeMap<String, f64>,
) -> f64 {
    match scenario_type {
        "loan_consolidation" => {
            let loan_amount = parameters.get("loan_amount").copied().unwrap_or(0.0);
            let loan_rate = parameters.get("loan_rate").copied().unwrap_or(0.0);
            let target_rate = parameters.get("target_rate").copied().unwrap_or(0.0);
            (loan_amount * loan_rate) - (loan_amount * target_rate)
        }
        "margin_for_box_spread" => {
            let loan_amount = parameters.get("loan_amount").copied().unwrap_or(0.0);
            let loan_rate = parameters.get("loan_rate").copied().unwrap_or(0.0);
            let box_spread_rate = parameters.get("box_spread_rate").copied().unwrap_or(0.0);
            (loan_amount * box_spread_rate) - (loan_amount * loan_rate)
        }
        "investment_fund" => {
            let loan_amount = parameters.get("loan_amount").copied().unwrap_or(0.0);
            let loan_rate = parameters.get("loan_rate").copied().unwrap_or(0.0);
            let fund_return = parameters.get("fund_return").copied().unwrap_or(0.0);
            (loan_amount * fund_return) - (loan_amount * loan_rate)
        }
        _ => 0.0,
    }
}

#[cfg(test)]
fn build_opportunity_scenarios_response(request: &FrontendViewRequest) -> Vec<FrontendScenario> {
    build_opportunity_scenarios_response_with_loans(request, &[])
}

fn build_opportunity_scenarios_response_with_loans(
    request: &FrontendViewRequest,
    repository_loans: &[LoanRecord],
) -> Vec<FrontendScenario> {
    let loans =
        aggregated_loan_inputs(&request.positions, &request.bank_accounts, repository_loans);
    let positions = effective_frontend_positions(&request.positions, repository_loans);
    let box_spreads: Vec<&FrontendPositionInput> = request
        .positions
        .iter()
        .filter(|position| position.instrument_type.as_deref() == Some("box_spread"))
        .collect();
    let box_spreads: Vec<&FrontendPositionInput> = if repository_loans.is_empty() {
        box_spreads
    } else {
        positions
            .into_iter()
            .filter(|position| position.instrument_type.as_deref() == Some("box_spread"))
            .collect()
    };

    let mut scenarios = Vec::new();

    if loans.len() > 1 {
        let highest_rate_loan = loans
            .iter()
            .map(|loan| (loan.annual_rate, loan.principal))
            .max_by(|a, b| a.0.total_cmp(&b.0));

        if let Some((loan_rate, loan_amount)) = highest_rate_loan.filter(|(rate, _)| *rate > 0.03) {
            let parameters = std::collections::BTreeMap::from([
                ("loan_amount".into(), loan_amount),
                ("loan_rate".into(), loan_rate),
                ("target_rate".into(), 0.04),
            ]);
            scenarios.push(FrontendScenario {
                id: "loan_consolidation".into(),
                name: "Loan Consolidation".into(),
                scenario_type: "loan_consolidation".into(),
                description: "Consolidate high-rate loans using lower-rate financing".into(),
                net_benefit: scenario_net_benefit("loan_consolidation", &parameters),
                parameters,
            });
        }
    }

    if let (Some(loan), Some(box_spread)) = (loans.first(), box_spreads.first()) {
        let parameters = std::collections::BTreeMap::from([
            ("loan_amount".into(), loan.principal),
            ("loan_rate".into(), loan.annual_rate),
            ("box_spread_rate".into(), box_spread.rate.unwrap_or(0.05)),
        ]);
        scenarios.push(FrontendScenario {
            id: "margin_for_box_spread".into(),
            name: "Use Loan as Margin for Box Spreads".into(),
            scenario_type: "margin_for_box_spread".into(),
            description: "Use loan proceeds as margin collateral for box spread positions".into(),
            net_benefit: scenario_net_benefit("margin_for_box_spread", &parameters),
            parameters,
        });
    }

    if let Some(loan) = loans.first() {
        let parameters = std::collections::BTreeMap::from([
            ("loan_amount".into(), loan.principal),
            ("loan_rate".into(), loan.annual_rate),
            ("fund_return".into(), 0.06),
        ]);
        scenarios.push(FrontendScenario {
            id: "investment_fund".into(),
            name: "Investment Fund Strategy".into(),
            scenario_type: "investment_fund".into(),
            description: "Use loan to invest in fund, use fund as collateral for cheaper loan"
                .into(),
            net_benefit: scenario_net_benefit("investment_fund", &parameters),
            parameters,
        });
    }

    scenarios
}

fn build_scenario_calculation_response(
    request: &FrontendScenarioCalculateRequest,
) -> FrontendScenarioCalculationResponse {
    let net_benefit = scenario_net_benefit(
        &request.scenario.scenario_type,
        &request.scenario.parameters,
    );
    let capital_efficiency = match request.scenario.scenario_type.as_str() {
        "margin_for_box_spread" => Some(1.2),
        "investment_fund" => Some(1.5),
        "loan_consolidation" => Some(1.0),
        _ => None,
    };

    FrontendScenarioCalculationResponse {
        net_benefit,
        cash_flow_impact: net_benefit / 12.0,
        risk_reduction: if request.scenario.scenario_type == "loan_consolidation" {
            0.15
        } else {
            0.05
        },
        capital_efficiency,
    }
}

#[cfg(test)]
fn build_unified_positions_response(
    request: &FrontendViewRequest,
) -> FrontendUnifiedPositionsResponse {
    build_unified_positions_response_with_loans(request, &[])
}

fn build_unified_positions_response_with_loans(
    request: &FrontendViewRequest,
    repository_loans: &[LoanRecord],
) -> FrontendUnifiedPositionsResponse {
    let positions = effective_frontend_positions(&request.positions, repository_loans);
    let positions_json: Vec<serde_json::Value> = request
        .positions
        .iter()
        .map(|position| {
            serde_json::json!({
                "name": position.name,
                "quantity": position.quantity.unwrap_or(0),
                "roi": position.roi.unwrap_or(0.0),
                "maker_count": position.maker_count.unwrap_or(0),
                "taker_count": position.taker_count.unwrap_or(0),
                "rebate_estimate": position.rebate_estimate.unwrap_or(0.0),
                "vega": position.vega.unwrap_or(0.0),
                "theta": position.theta.unwrap_or(0.0),
                "fair_diff": position.fair_diff.unwrap_or(0.0),
                "maturity_date": position.maturity_date,
                "cash_flow": position.cash_flow,
                "candle": candle_to_json(&position.candle),
                "instrument_type": position.instrument_type,
                "rate": position.rate,
                "collateral_value": position.collateral_value,
                "currency": position.currency,
                "market_value": position.market_value,
                "bid": position.bid,
                "ask": position.ask,
                "last": position.last,
                "spread": position.spread,
                "price": position.price,
                "side": position.side,
                "expected_cash_at_expiry": position.expected_cash_at_expiry,
                "dividend": position.dividend,
                "conid": position.conid,
            })
        })
        .collect();
    let positions_json: Vec<serde_json::Value> = if repository_loans.is_empty() {
        positions_json
    } else {
        positions
            .into_iter()
            .map(|position| {
                serde_json::json!({
                    "name": position.name,
                    "quantity": position.quantity.unwrap_or(0),
                    "roi": position.roi.unwrap_or(0.0),
                    "maker_count": position.maker_count.unwrap_or(0),
                    "taker_count": position.taker_count.unwrap_or(0),
                    "rebate_estimate": position.rebate_estimate.unwrap_or(0.0),
                    "vega": position.vega.unwrap_or(0.0),
                    "theta": position.theta.unwrap_or(0.0),
                    "fair_diff": position.fair_diff.unwrap_or(0.0),
                    "maturity_date": position.maturity_date,
                    "cash_flow": position.cash_flow,
                    "candle": candle_to_json(&position.candle),
                    "instrument_type": position.instrument_type,
                    "rate": position.rate,
                    "collateral_value": position.collateral_value,
                    "currency": position.currency,
                    "market_value": position.market_value,
                    "bid": position.bid,
                    "ask": position.ask,
                    "last": position.last,
                    "spread": position.spread,
                    "price": position.price,
                    "side": position.side,
                    "expected_cash_at_expiry": position.expected_cash_at_expiry,
                    "dividend": position.dividend,
                    "conid": position.conid,
                })
            })
            .collect()
    };

    let reference_candle = positions_json
        .first()
        .and_then(|position| position.get("candle").cloned())
        .unwrap_or_else(default_candle_json);
    let bank_positions =
        normalize_bank_accounts_to_positions(&request.bank_accounts, &reference_candle);

    FrontendUnifiedPositionsResponse {
        positions: positions_json
            .into_iter()
            .chain(repository_loans.iter().map(|loan| {
                serde_json::json!({
                    "name": loan.bank_name,
                    "quantity": 1,
                    "roi": 0.0,
                    "maker_count": 0,
                    "taker_count": 0,
                    "rebate_estimate": 0.0,
                    "vega": 0.0,
                    "theta": 0.0,
                    "fair_diff": 0.0,
                    "maturity_date": loan.maturity_date,
                    "cash_flow": loan.principal,
                    "candle": { "close": loan.principal },
                    "instrument_type": "bank_loan",
                    "rate": loan.interest_rate,
                    "collateral_value": serde_json::Value::Null,
                    "currency": "ILS",
                    "market_value": loan.principal,
                    "bid": serde_json::Value::Null,
                    "ask": serde_json::Value::Null,
                    "last": serde_json::Value::Null,
                    "spread": serde_json::Value::Null,
                    "price": serde_json::Value::Null,
                    "side": serde_json::Value::Null,
                    "expected_cash_at_expiry": serde_json::Value::Null,
                    "dividend": serde_json::Value::Null,
                    "conid": serde_json::Value::Null,
                    "loan_id": loan.loan_id,
                })
            }))
            .chain(bank_positions.into_iter())
            .collect(),
    }
}

async fn frontend_cash_flow_timeline(
    Extension(state): Extension<RestState>,
    Json(request): Json<FrontendCashFlowRequest>,
) -> Json<FrontendCashFlowResponse> {
    let loans = state.loans.list().await;
    Json(build_cash_flow_response_with_loans(&request, &loans))
}

async fn frontend_opportunity_scenarios(
    Extension(state): Extension<RestState>,
    Json(request): Json<FrontendViewRequest>,
) -> Json<Vec<FrontendScenario>> {
    let loans = state.loans.list().await;
    Json(build_opportunity_scenarios_response_with_loans(
        &request, &loans,
    ))
}

async fn frontend_opportunity_calculate(
    Json(request): Json<FrontendScenarioCalculateRequest>,
) -> Json<FrontendScenarioCalculationResponse> {
    Json(build_scenario_calculation_response(&request))
}

async fn frontend_unified_positions(
    Extension(state): Extension<RestState>,
    Json(request): Json<FrontendViewRequest>,
) -> Json<FrontendUnifiedPositionsResponse> {
    let loans = state.loans.list().await;
    Json(build_unified_positions_response_with_loans(
        &request, &loans,
    ))
}

#[cfg(test)]
fn build_relationship_response(request: &FrontendViewRequest) -> FrontendRelationshipResponse {
    build_relationship_response_with_loans(request, &[])
}

fn build_relationship_response_with_loans(
    request: &FrontendViewRequest,
    repository_loans: &[LoanRecord],
) -> FrontendRelationshipResponse {
    let positions = effective_frontend_positions(&request.positions, repository_loans);
    let positions_json: Vec<serde_json::Value> = request
        .positions
        .iter()
        .map(|position| {
            serde_json::json!({
                "name": position.name,
                "instrument_type": position.instrument_type,
                "cash_flow": position.cash_flow,
                "rate": position.rate,
                "collateral_value": position.collateral_value,
                "candle": candle_to_json(&position.candle),
            })
        })
        .collect();
    let positions_json: Vec<serde_json::Value> = if repository_loans.is_empty() {
        positions_json
    } else {
        positions
            .into_iter()
            .map(|position| {
                serde_json::json!({
                    "name": position.name,
                    "instrument_type": position.instrument_type,
                    "cash_flow": position.cash_flow,
                    "rate": position.rate,
                    "collateral_value": position.collateral_value,
                    "candle": candle_to_json(&position.candle),
                })
            })
            .collect()
    };
    let loans =
        aggregated_loan_inputs(&request.positions, &request.bank_accounts, repository_loans);
    let box_spreads: Vec<&serde_json::Value> = positions_json
        .iter()
        .filter(|position| {
            position
                .get("instrument_type")
                .and_then(|value| value.as_str())
                == Some("box_spread")
        })
        .collect();
    let bonds: Vec<&serde_json::Value> = positions_json
        .iter()
        .filter(|position| {
            matches!(
                position
                    .get("instrument_type")
                    .and_then(|value| value.as_str()),
                Some("bond" | "t_bill")
            )
        })
        .collect();

    let mut relationships = Vec::new();

    for loan in &loans {
        let loan_value = loan.principal;
        for box_spread in &box_spreads {
            relationships.push(FrontendRelationship {
                from: loan.name.clone(),
                to: box_spread
                    .get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Unknown")
                    .to_string(),
                relationship_type: "margin".into(),
                description: "Loan used as margin for box spread".into(),
                value: loan_value,
            });
        }
    }

    for loan in &loans {
        let loan_value = loan.principal;
        for bond in &bonds {
            relationships.push(FrontendRelationship {
                from: loan.name.clone(),
                to: bond
                    .get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Unknown")
                    .to_string(),
                relationship_type: "investment".into(),
                description: "Loan proceeds invested in bond".into(),
                value: loan_value,
            });
        }
    }

    for bond in &bonds {
        let bond_value = bond
            .get("collateral_value")
            .and_then(|value| value.as_f64())
            .filter(|value| *value != 0.0)
            .unwrap_or_else(|| position_value(bond));
        let bond_rate = bond
            .get("rate")
            .and_then(|value| value.as_f64())
            .unwrap_or(0.0);
        for loan in &loans {
            let loan_rate = loan.annual_rate;
            if bond_rate > loan_rate {
                relationships.push(FrontendRelationship {
                    from: bond
                        .get("name")
                        .and_then(|v| v.as_str())
                        .unwrap_or("Unknown")
                        .to_string(),
                    to: loan.name.clone(),
                    relationship_type: "collateral".into(),
                    description: "Bond used as collateral for loan".into(),
                    value: bond_value,
                });
            }
        }
    }

    for box_spread in &box_spreads {
        relationships.push(FrontendRelationship {
            from: box_spread
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown")
                .to_string(),
            to: "Synthetic Financing".into(),
            relationship_type: "financing".into(),
            description: "Box spread provides synthetic financing".into(),
            value: position_value(box_spread),
        });
    }

    let mut nodes = std::collections::BTreeSet::new();
    for relationship in &relationships {
        if !relationship.from.is_empty() {
            nodes.insert(relationship.from.clone());
        }
        if !relationship.to.is_empty() {
            nodes.insert(relationship.to.clone());
        }
    }
    for position in &request.positions {
        if !position.name.is_empty()
            && !(!repository_loans.is_empty()
                && is_loan_instrument(position.instrument_type.as_deref()))
        {
            nodes.insert(position.name.clone());
        }
    }
    for loan in repository_loans {
        if !loan.bank_name.is_empty() {
            nodes.insert(loan.bank_name.clone());
        }
    }
    for account in &request.bank_accounts {
        if !account.account_name.is_empty() {
            nodes.insert(account.account_name.clone());
        }
    }

    FrontendRelationshipResponse {
        relationships,
        nodes: nodes.into_iter().collect(),
    }
}

async fn frontend_relationships(
    Extension(state): Extension<RestState>,
    Json(request): Json<FrontendViewRequest>,
) -> Json<FrontendRelationshipResponse> {
    let loans = state.loans.list().await;
    Json(build_relationship_response_with_loans(&request, &loans))
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

fn swiftness_enabled() -> bool {
    matches!(
        std::env::var("ENABLE_SWIFTNESS")
            .ok()
            .as_deref()
            .map(str::trim)
            .map(str::to_ascii_lowercase)
            .as_deref(),
        Some("1" | "true" | "yes" | "on")
    )
}

fn ensure_swiftness_enabled() -> SwiftnessResult<()> {
    if swiftness_enabled() {
        Ok(())
    } else {
        Err(swiftness_error(
            StatusCode::SERVICE_UNAVAILABLE,
            "Swiftness integration is temporarily disabled".into(),
        ))
    }
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
    ensure_swiftness_enabled()?;
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
    ensure_swiftness_enabled()?;
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::loans::{LoanRepository, LoanStatus, LoanType};
    use crate::state::{OrderSnapshot, PositionSnapshot, SystemSnapshot};
    use crate::HealthAggregateState;
    use axum::Json;
    use nats_adapter::encode_proto;
    use nats_adapter::proto::v1::NatsEnvelope;
    use std::{env, path::PathBuf, sync::Arc};
    use tokio::sync::{watch, RwLock};
    use uuid::Uuid;

    async fn test_rest_state() -> RestState {
        let path: PathBuf = env::temp_dir().join(format!("loan-test-{}.db", Uuid::new_v4()));
        let repo = LoanRepository::load(path).await.expect("temp loan repo");
        let (tx, _rx) = watch::channel(false);
        RestState::new(
            Arc::new(RwLock::new(SystemSnapshot::default())),
            StrategyController::new(tx),
            repo,
            HealthAggregateState::new_shared(),
        )
    }

    async fn seeded_rest_state() -> RestState {
        let state = test_rest_state().await;
        {
            let mut snapshot = state.snapshot.write().await;
            snapshot.positions.push(PositionSnapshot {
                id: "POS-1".into(),
                symbol: "SPY".into(),
                quantity: 2,
                cost_basis: 100.0,
                mark: 101.5,
                unrealized_pnl: 3.0,
            });
            snapshot.orders.push(OrderSnapshot {
                id: "ORD-1".into(),
                symbol: "SPY".into(),
                side: "BUY".into(),
                quantity: 2,
                status: "FILLED".into(),
                submitted_at: Utc::now(),
            });
        }
        state
    }

    fn sample_loan(loan_id: &str) -> LoanRecord {
        LoanRecord {
            loan_id: loan_id.into(),
            bank_name: "Discount".into(),
            account_number: "123".into(),
            loan_type: LoanType::ShirBased,
            principal: 1000.0,
            original_principal: 1200.0,
            interest_rate: 4.0,
            spread: 0.5,
            base_cpi: 100.0,
            current_cpi: 100.0,
            origination_date: "2025-01-01T00:00:00Z".into(),
            maturity_date: "2030-01-01T00:00:00Z".into(),
            next_payment_date: "2025-02-01T00:00:00Z".into(),
            monthly_payment: 100.0,
            payment_frequency_months: 1,
            status: LoanStatus::Active,
            last_update: "2025-01-01T00:00:00Z".into(),
        }
    }

    fn sample_request() -> FrontendViewRequest {
        FrontendViewRequest {
            positions: vec![
                FrontendPositionInput {
                    name: "Box A".into(),
                    quantity: Some(1),
                    roi: Some(5.0),
                    maker_count: Some(0),
                    taker_count: Some(0),
                    rebate_estimate: Some(0.0),
                    vega: Some(0.0),
                    theta: Some(0.0),
                    fair_diff: Some(0.0),
                    maturity_date: None,
                    cash_flow: Some(1000.0),
                    candle: Some(FrontendCandleInput {
                        open: Some(10.0),
                        high: Some(11.0),
                        low: Some(9.0),
                        close: Some(10.5),
                        volume: Some(100.0),
                        entry: Some(10.0),
                        updated: Some("2026-03-10T00:00:00Z".into()),
                    }),
                    instrument_type: Some("box_spread".into()),
                    rate: Some(0.08),
                    collateral_value: None,
                    currency: Some("USD".into()),
                    market_value: None,
                    bid: None,
                    ask: None,
                    last: None,
                    spread: None,
                    price: None,
                    side: None,
                    expected_cash_at_expiry: None,
                    dividend: None,
                    conid: None,
                },
                FrontendPositionInput {
                    name: "Bond A".into(),
                    quantity: Some(1),
                    roi: Some(3.0),
                    maker_count: Some(0),
                    taker_count: Some(0),
                    rebate_estimate: Some(0.0),
                    vega: Some(0.0),
                    theta: Some(0.0),
                    fair_diff: Some(0.0),
                    maturity_date: None,
                    cash_flow: Some(800.0),
                    candle: Some(FrontendCandleInput {
                        open: Some(8.0),
                        high: Some(8.5),
                        low: Some(7.5),
                        close: Some(8.2),
                        volume: Some(50.0),
                        entry: Some(8.0),
                        updated: Some("2026-03-10T00:00:00Z".into()),
                    }),
                    instrument_type: Some("bond".into()),
                    rate: Some(0.06),
                    collateral_value: Some(850.0),
                    currency: Some("USD".into()),
                    market_value: None,
                    bid: None,
                    ask: None,
                    last: None,
                    spread: None,
                    price: None,
                    side: None,
                    expected_cash_at_expiry: None,
                    dividend: None,
                    conid: None,
                },
            ],
            bank_accounts: vec![FrontendBankAccountInput {
                account_name: "Discount".into(),
                balance: 1200.0,
                account_path: None,
                bank_name: None,
                account_number: None,
                debit_rate: Some(0.04),
                credit_rate: Some(0.02),
                currency: Some("USD".into()),
                balances_by_currency: Some(std::collections::BTreeMap::from([
                    ("EUR".into(), 50.0),
                    ("USD".into(), 1200.0),
                ])),
                is_mixed_currency: true,
            }],
        }
    }

    #[test]
    fn unified_positions_expand_mixed_currency_accounts() {
        let response = build_unified_positions_response(&sample_request());
        assert_eq!(response.positions.len(), 4);
        let names: Vec<String> = response
            .positions
            .iter()
            .filter_map(|value| {
                value
                    .get("name")
                    .and_then(|value| value.as_str())
                    .map(str::to_string)
            })
            .collect();
        assert!(names.contains(&"Discount (EUR)".to_string()));
        assert!(names.contains(&"Discount (USD)".to_string()));
    }

    #[test]
    fn relationships_match_python_shape() {
        let response = build_relationship_response(&sample_request());
        assert_eq!(
            response.nodes,
            vec!["Bond A", "Box A", "Discount", "Synthetic Financing"]
        );
        assert!(response.relationships.iter().any(|rel| {
            rel.from == "Discount"
                && rel.to == "Box A"
                && rel.relationship_type == "margin"
                && (rel.value - 1200.0).abs() < f64::EPSILON
        }));
        assert!(response.relationships.iter().any(|rel| {
            rel.from == "Bond A"
                && rel.to == "Discount"
                && rel.relationship_type == "collateral"
                && (rel.value - 850.0).abs() < f64::EPSILON
        }));
    }

    #[test]
    fn cash_flow_matches_expected_shape() {
        let request = FrontendCashFlowRequest {
            positions: sample_request().positions,
            bank_accounts: sample_request().bank_accounts,
            projection_months: 2,
        };
        let response = build_cash_flow_response(&request);
        assert!(response.total_outflows > 0.0);
        assert!(response.total_inflows > 0.0);
        assert!(!response.monthly_flows.is_empty());
        assert!(response
            .events
            .iter()
            .any(|event| event.event_type == "loan_payment"));
    }

    #[test]
    fn opportunity_scenarios_match_expected_shape() {
        let mut request = sample_request();
        request.positions.push(FrontendPositionInput {
            name: "Loan A".into(),
            quantity: Some(1),
            roi: None,
            maker_count: None,
            taker_count: None,
            rebate_estimate: None,
            vega: None,
            theta: None,
            fair_diff: None,
            maturity_date: Some("2026-09-10".into()),
            cash_flow: Some(1000.0),
            candle: None,
            instrument_type: Some("bank_loan".into()),
            rate: Some(0.07),
            collateral_value: None,
            currency: Some("USD".into()),
            market_value: None,
            bid: None,
            ask: None,
            last: None,
            spread: None,
            price: None,
            side: None,
            expected_cash_at_expiry: None,
            dividend: None,
            conid: None,
        });
        let scenarios = build_opportunity_scenarios_response(&request);
        assert!(scenarios
            .iter()
            .any(|scenario| scenario.id == "margin_for_box_spread"));
        assert!(scenarios
            .iter()
            .any(|scenario| scenario.id == "investment_fund"));
    }

    #[test]
    fn opportunity_calculation_matches_python_shape() {
        let request = FrontendScenarioCalculateRequest {
            scenario: FrontendScenarioInput {
                id: "investment_fund".into(),
                name: "Investment Fund Strategy".into(),
                scenario_type: "investment_fund".into(),
                description: "Use loan to invest in fund".into(),
                parameters: std::collections::BTreeMap::from([
                    ("loan_amount".into(), 1000.0),
                    ("loan_rate".into(), 0.04),
                    ("fund_return".into(), 0.06),
                ]),
            },
        };
        let response = build_scenario_calculation_response(&request);
        assert!((response.net_benefit - 20.0).abs() < f64::EPSILON);
        assert_eq!(response.capital_efficiency, Some(1.5));
    }

    #[tokio::test]
    async fn loans_crud_handlers_work() {
        let state = test_rest_state().await;
        let loan = sample_loan("loan-1");

        let (status, Json(created)) = loans_create(Extension(state.clone()), Json(loan.clone()))
            .await
            .expect("create loan");
        assert_eq!(status, StatusCode::CREATED);
        assert_eq!(created.loan_id, "loan-1");

        let Json(listed) = loans_list(Extension(state.clone())).await;
        assert_eq!(listed.loans.len(), 1);
        assert_eq!(listed.loans[0].loan_id, "loan-1");

        let Json(fetched) = loans_get(Path("loan-1".into()), Extension(state.clone()))
            .await
            .expect("get loan");
        assert_eq!(fetched.bank_name, "Discount");

        let mut updated_loan = loan.clone();
        updated_loan.monthly_payment = 150.0;
        let Json(updated) = loans_update(
            Path("loan-1".into()),
            Extension(state.clone()),
            Json(updated_loan),
        )
        .await
        .expect("update loan");
        assert_eq!(updated.monthly_payment, 150.0);

        let status = loans_delete(Path("loan-1".into()), Extension(state.clone()))
            .await
            .expect("delete loan");
        assert_eq!(status, StatusCode::NO_CONTENT);

        let err = loans_get(Path("loan-1".into()), Extension(state))
            .await
            .expect_err("loan should be gone");
        assert_eq!(err.0, StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn loans_create_rejects_invalid_record() {
        let state = test_rest_state().await;
        let mut loan = sample_loan("bad-loan");
        loan.principal = 0.0;

        let err = loans_create(Extension(state), Json(loan))
            .await
            .expect_err("invalid loan should fail");
        assert_eq!(err.0, StatusCode::BAD_REQUEST);
        assert!(err.1.contains("Principal must be > 0"));
    }

    #[test]
    fn strip_json_comments_preserves_strings() {
        let raw = r#"
        {
          // top-level comment
          "name": "http://example.com//keep",
          "nested": /* block */ { "value": "/* keep */" }
        }
        "#;

        let parsed: serde_json::Value =
            serde_json::from_str(&strip_json_comments(raw)).expect("valid json");

        assert_eq!(parsed["name"], "http://example.com//keep");
        assert_eq!(parsed["nested"]["value"], "/* keep */");
    }

    #[test]
    fn load_shared_config_json_prefers_env_override_and_resolves_placeholders() {
        let config_path = env::temp_dir().join(format!("shared-config-{}.json", Uuid::new_v4()));
        let config_text = r#"
        {
          // comment should be ignored
          "version": "2.0.0",
          "services": { "health_dashboard": { "port": 8011 } },
          "broker": {
            "primary": "${TEST_SHARED_PRIMARY}",
            "priorities": ["ib", "mock"]
          },
          "pwa": {
            "servicePorts": { "ib": 8002 },
            "defaultService": "ib",
            "serviceUrls": { "ib": "${TEST_SHARED_URL}" }
          }
        }
        "#;

        std::fs::write(&config_path, config_text).expect("write shared config");
        env::set_var("IB_BOX_SPREAD_CONFIG", &config_path);
        env::set_var("TEST_SHARED_PRIMARY", "IB");
        env::set_var("TEST_SHARED_URL", "http://127.0.0.1:8002");

        let parsed = load_shared_config_json().expect("load shared config");
        let response = shared_config_response(&parsed);

        assert_eq!(response["version"], "2.0.0");
        assert_eq!(response["broker"]["primary"], "IB");
        assert_eq!(
            response["broker"]["priorities"],
            serde_json::json!(["ib", "mock"])
        );
        assert_eq!(
            response["pwa"]["serviceUrls"]["ib"],
            "http://127.0.0.1:8002"
        );

        env::remove_var("IB_BOX_SPREAD_CONFIG");
        env::remove_var("TEST_SHARED_PRIMARY");
        env::remove_var("TEST_SHARED_URL");
        let _ = std::fs::remove_file(config_path);
    }

    #[test]
    fn live_state_envelope_metadata_decodes_protobuf_envelope() {
        let envelope = NatsEnvelope {
            id: "msg-1".into(),
            timestamp: None,
            source: "collector".into(),
            message_type: "MarketDataEvent".into(),
            payload: vec![1, 2, 3],
        };
        let bytes = encode_proto(&envelope).expect("encode envelope");

        let metadata = decode_live_state_envelope_metadata(bytes.as_ref());

        assert_eq!(metadata["id"], "msg-1");
        assert_eq!(metadata["source"], "collector");
        assert_eq!(metadata["message_type"], "MarketDataEvent");
        assert_eq!(
            metadata["payload_b64"],
            BASE64_STANDARD.encode([1_u8, 2, 3])
        );
    }

    #[test]
    fn live_state_envelope_metadata_reports_decode_error_for_invalid_bytes() {
        let metadata = decode_live_state_envelope_metadata(br#"{"legacy":"json"}"#);
        assert!(metadata.get("decode_error").is_some());
    }

    #[tokio::test]
    async fn runtime_positions_endpoint_uses_runtime_contract() {
        let state = seeded_rest_state().await;
        let Json(response) = positions_list(Extension(state.clone())).await;
        assert_eq!(response.positions.len(), 1);
        assert_eq!(response.positions[0].id, "POS-1");
        assert_eq!(response.positions[0].symbol, "SPY");
        assert!((response.positions[0].market_value - 203.0).abs() < f64::EPSILON);

        let Json(position) = position_details(Extension(state), Path("POS-1".into()))
            .await
            .expect("position should exist");
        assert_eq!(position.quantity, 2);
        assert_eq!(position.symbol, "SPY");
    }

    #[tokio::test]
    async fn runtime_orders_endpoint_uses_runtime_contract() {
        let state = seeded_rest_state().await;
        let Json(response) = orders_list(
            Extension(state.clone()),
            Query(OrdersListQuery {
                status: None,
                limit: None,
            }),
        )
        .await;
        assert_eq!(response.orders.len(), 1);
        assert_eq!(response.orders[0].id, "ORD-1");
        assert_eq!(response.orders[0].symbol, "SPY");

        let Json(order) = order_details(Extension(state), Path("ORD-1".into()))
            .await
            .expect("order should exist");
        assert_eq!(order.side, "BUY");
        assert_eq!(order.status, "FILLED");
    }

    #[tokio::test]
    async fn snapshot_endpoint_uses_runtime_snapshot_contract() {
        let state = seeded_rest_state().await;
        let Json(snapshot) = snapshot(Extension(state)).await;
        assert_eq!(snapshot.positions.len(), 1);
        assert_eq!(snapshot.orders.len(), 1);
        assert_eq!(snapshot.positions[0].id, "POS-1");
        assert_eq!(snapshot.orders[0].id, "ORD-1");
        assert!((snapshot.positions[0].market_value - 203.0).abs() < f64::EPSILON);
    }
}
