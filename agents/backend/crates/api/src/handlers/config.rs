use axum::{
  extract::{Query, State},
  http::StatusCode,
  Json,
};
use chrono::Utc;

use super::models::{
  trim_alerts, AccountRequest, ApiResponse, ConfigUpdateRequest, ModeRequest, ScenariosQuery,
  VALID_MODES,
};
use crate::rest::RestState;
use crate::state::Alert;

pub async fn toggle_mode(
  State(state): State<RestState>,
  Json(request): Json<ModeRequest>,
) -> Result<Json<ApiResponse>, (StatusCode, Json<ApiResponse>)> {
  if !VALID_MODES.contains(&request.mode.as_str()) {
    return Err((
      StatusCode::BAD_REQUEST,
      Json(ApiResponse::error(format!("Invalid mode: {}. Must be one of: {:?}", request.mode, VALID_MODES))),
    ));
  }

  let mut snapshot = state.snapshot.write().await;
  snapshot.mode = request.mode.clone();
  snapshot.touch();
  snapshot.alerts.push(Alert::info(&format!("Mode changed to {} via REST", request.mode)));
  trim_alerts(&mut snapshot.alerts);

  Ok(Json(ApiResponse::ok_with_data(
    format!("Mode changed to {}", request.mode),
    serde_json::json!({ "mode": request.mode }),
  )))
}

pub async fn change_account(
  State(state): State<RestState>,
  Json(request): Json<AccountRequest>,
) -> Json<ApiResponse> {
  let mut snapshot = state.snapshot.write().await;
  snapshot.account_id = request.account_id.clone();
  snapshot.touch();
  snapshot.alerts.push(Alert::info(&format!("Account changed to {} via REST", request.account_id)));
  trim_alerts(&mut snapshot.alerts);

  Json(ApiResponse::ok_with_data("Account changed", serde_json::json!({ "account_id": request.account_id })))
}

pub async fn get_config(State(state): State<RestState>) -> Json<serde_json::Value> {
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

pub async fn update_config(
  State(state): State<RestState>,
  Json(request): Json<ConfigUpdateRequest>,
) -> Result<Json<ApiResponse>, (StatusCode, Json<ApiResponse>)> {
  let mut snapshot = state.snapshot.write().await;

  if let Some(mode) = request.mode {
    if !VALID_MODES.contains(&mode.as_str()) {
      return Err((
        StatusCode::BAD_REQUEST,
        Json(ApiResponse::error(format!("Invalid mode: {}. Must be one of: {:?}", mode, VALID_MODES))),
      ));
    }
    snapshot.mode = mode;
  }

  if let Some(strategy_cfg) = &request.strategy {
    if let Some(status) = strategy_cfg.get("status").and_then(|v| v.as_str()) {
      snapshot.strategy = status.to_string();
      snapshot.alerts.push(Alert::info(format!("Strategy status updated to '{}'", status)));
    }
    if let Some(symbols) = strategy_cfg.get("symbols").and_then(|v| v.as_array()) {
      let new_symbols: Vec<String> = symbols.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect();
      if !new_symbols.is_empty() {
        snapshot.alerts.push(Alert::info(format!("Watchlist updated: {:?}", new_symbols)));
      }
    }
  }

  if let Some(risk_cfg) = &request.risk {
    if let Some(allowed) = risk_cfg.get("allowed").and_then(|v| v.as_bool()) {
      snapshot.risk.allowed = allowed;
      snapshot.alerts.push(Alert::info(format!("Risk trading allowed set to {}", allowed)));
    }
    if let Some(reason) = risk_cfg.get("reason").and_then(|v| v.as_str()) {
      snapshot.risk.reason = Some(reason.to_string());
    }
    snapshot.risk.updated_at = Utc::now();
  }

  snapshot.touch();
  snapshot.alerts.push(Alert::info("Configuration updated via REST"));
  trim_alerts(&mut snapshot.alerts);

  Ok(Json(ApiResponse::ok("Configuration updated")))
}

pub async fn get_scenarios(
  State(state): State<RestState>,
  Query(params): Query<ScenariosQuery>,
) -> Json<serde_json::Value> {
  let snapshot = state.snapshot.read().await;
  let underlying = params.symbol.unwrap_or_else(|| "SPX".to_string());
  let min_apr = params.min_apr.unwrap_or(0.0);

  let mut scenarios = Vec::<serde_json::Value>::new();

  for position in &snapshot.positions {
    if position.symbol.contains(&underlying) || underlying == "SPX" {
      let cost = position.cost_basis.abs();
      if cost < 1e-6 { continue; }

      let profit = position.mark - position.cost_basis;
      let roi_pct = (profit / cost) * 100.0;
      let annualized_apr = roi_pct * 4.0;

      if annualized_apr < min_apr { continue; }

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

  if scenarios.is_empty() {
    for sym in &snapshot.symbols {
      if sym.symbol.contains(&underlying) || underlying == "SPX" {
        for &width in &[5.0, 10.0, 25.0, 50.0] {
          let theoretical = width * 100.0;
          let mid = sym.last;
          if mid <= 0.0 { continue; }
          let net_debit = theoretical - (mid * 0.001 * width);
          let implied_apr = if net_debit > 0.0 {
            ((theoretical - net_debit) / net_debit) * (365.0 / 30.0) * 100.0
          } else { 0.0 };

          if implied_apr < min_apr { continue; }

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
    let apr_a = a.get("annualized_apr").or(a.get("implied_apr")).and_then(|v| v.as_f64()).unwrap_or(0.0);
    let apr_b = b.get("annualized_apr").or(b.get("implied_apr")).and_then(|v| v.as_f64()).unwrap_or(0.0);
    apr_b.partial_cmp(&apr_a).unwrap_or(std::cmp::Ordering::Equal)
  });

  Json(serde_json::json!({
    "scenarios": scenarios,
    "count": scenarios.len(),
    "as_of": Utc::now(),
    "underlying": underlying,
    "min_apr_filter": min_apr,
  }))
}
