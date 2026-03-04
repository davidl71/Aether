use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::state::OrderSnapshot;

#[derive(Debug, Serialize)]
pub struct ApiResponse {
  pub status: String,
  pub message: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub data: Option<serde_json::Value>,
}

impl ApiResponse {
  pub fn ok(message: impl Into<String>) -> Self {
    Self { status: "ok".into(), message: message.into(), data: None }
  }

  pub fn ok_with_data(message: impl Into<String>, data: serde_json::Value) -> Self {
    Self { status: "ok".into(), message: message.into(), data: Some(data) }
  }

  pub fn error(message: impl Into<String>) -> Self {
    Self { status: "error".into(), message: message.into(), data: None }
  }
}

#[derive(Debug, Deserialize)]
pub struct CancelOrderRequest {
  pub order_id: String,
}

#[derive(Debug, Deserialize)]
pub struct ModeRequest {
  pub mode: String,
}

#[derive(Debug, Deserialize)]
pub struct AccountRequest {
  pub account_id: String,
}

#[derive(Debug, Serialize)]
pub struct StrategyStatusResponse {
  pub status: String,
  pub started_at: Option<DateTime<Utc>>,
  pub last_update: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct OrdersListResponse {
  pub orders: Vec<OrderSnapshot>,
}

#[derive(Debug, Deserialize)]
pub struct OrdersListQuery {
  pub status: Option<String>,
  pub limit: Option<usize>,
}

#[derive(Debug, Deserialize)]
pub struct ConfigUpdateRequest {
  #[serde(default)]
  pub mode: Option<String>,
  #[serde(default)]
  pub strategy: Option<serde_json::Value>,
  #[serde(default)]
  pub risk: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct ScenariosQuery {
  pub symbol: Option<String>,
  pub min_apr: Option<f64>,
}

#[derive(Debug, Deserialize)]
pub struct ChartQuery {
  pub timeframe: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SwiftnessPositionsQuery {
  pub check_validity: Option<bool>,
  pub max_age_days: Option<u32>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ExchangeRateUpdate {
  pub rate: f64,
}

pub const VALID_MODES: &[&str] = &["DRY-RUN", "LIVE"];

pub fn trim_alerts(alerts: &mut Vec<crate::state::Alert>) {
  while alerts.len() > 32 {
    alerts.remove(0);
  }
}
