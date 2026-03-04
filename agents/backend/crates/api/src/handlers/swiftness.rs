use std::time::Duration;

use axum::{extract::Query, http::StatusCode, Json};
use reqwest::Client;
use tracing::warn;

use super::models::{ApiResponse, ExchangeRateUpdate, SwiftnessPositionsQuery};

const SWIFTNESS_API_URL: &str = "http://127.0.0.1:8081";

type SwiftnessResult<T> = Result<T, (StatusCode, Json<ApiResponse>)>;

fn error(code: StatusCode, msg: String) -> (StatusCode, Json<ApiResponse>) {
  (code, Json(ApiResponse::error(msg)))
}

fn client() -> SwiftnessResult<Client> {
  Client::builder()
    .timeout(Duration::from_secs(5))
    .build()
    .map_err(|e| error(StatusCode::INTERNAL_SERVER_ERROR, format!("HTTP client error: {e}")))
}

async fn proxy_get(path: &str) -> SwiftnessResult<Json<serde_json::Value>> {
  let c = client()?;
  let url = format!("{SWIFTNESS_API_URL}{path}");
  match c.get(&url).send().await {
    Ok(resp) if resp.status().is_success() => resp
      .json::<serde_json::Value>()
      .await
      .map(Json)
      .map_err(|e| error(StatusCode::INTERNAL_SERVER_ERROR, format!("Parse error: {e}"))),
    Ok(resp) => Err(error(StatusCode::BAD_GATEWAY, format!("Swiftness API error: {}", resp.status()))),
    Err(e) => {
      warn!(%e, "Swiftness API call failed");
      Err(error(StatusCode::BAD_GATEWAY, format!("Swiftness API unavailable: {e}")))
    }
  }
}

pub async fn positions(Query(params): Query<SwiftnessPositionsQuery>) -> SwiftnessResult<Json<serde_json::Value>> {
  let cv = params.check_validity.unwrap_or(true);
  let max_age = params.max_age_days.unwrap_or(90);
  proxy_get(&format!("/positions?check_validity={cv}&max_age_days={max_age}")).await
}

pub async fn portfolio_value() -> SwiftnessResult<Json<serde_json::Value>> {
  proxy_get("/portfolio-value").await
}

pub async fn validate() -> SwiftnessResult<Json<serde_json::Value>> {
  proxy_get("/validate").await
}

pub async fn exchange_rate() -> SwiftnessResult<Json<serde_json::Value>> {
  proxy_get("/exchange-rate").await
}

pub async fn update_exchange_rate(
  Json(update): Json<ExchangeRateUpdate>,
) -> SwiftnessResult<Json<ApiResponse>> {
  if update.rate <= 0.0 {
    return Err(error(StatusCode::BAD_REQUEST, "Exchange rate must be positive".into()));
  }
  let c = client()?;
  let url = format!("{SWIFTNESS_API_URL}/exchange-rate");
  match c.put(&url).json(&update).send().await {
    Ok(resp) if resp.status().is_success() => Ok(Json(ApiResponse::ok_with_data(
      "Exchange rate updated",
      serde_json::json!({ "rate": update.rate }),
    ))),
    Ok(resp) => Err(error(StatusCode::BAD_GATEWAY, format!("Swiftness API error: {}", resp.status()))),
    Err(e) => {
      warn!(%e, "Swiftness API call failed");
      Err(error(StatusCode::BAD_GATEWAY, format!("Swiftness API unavailable: {e}")))
    }
  }
}
