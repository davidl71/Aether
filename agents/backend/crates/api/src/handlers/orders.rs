use axum::{
  extract::{Path, Query, State},
  http::StatusCode,
  Json,
};

use super::models::{trim_alerts, ApiResponse, CancelOrderRequest, OrdersListQuery, OrdersListResponse};
use crate::rest::RestState;
use crate::state::Alert;

pub async fn list(
  State(state): State<RestState>,
  Query(params): Query<OrdersListQuery>,
) -> Json<OrdersListResponse> {
  let snapshot = state.snapshot.read().await;
  let mut orders = snapshot.orders.clone();

  if let Some(ref status_filter) = params.status {
    orders.retain(|o| o.status.eq_ignore_ascii_case(status_filter));
  }
  if let Some(limit) = params.limit {
    orders.truncate(limit);
  }

  Json(OrdersListResponse { orders })
}

pub async fn details(
  State(state): State<RestState>,
  Path(order_id): Path<String>,
) -> Result<Json<crate::state::OrderSnapshot>, (StatusCode, Json<ApiResponse>)> {
  let snapshot = state.snapshot.read().await;
  snapshot
    .orders
    .iter()
    .find(|o| o.id == order_id)
    .cloned()
    .map(Json)
    .ok_or_else(|| (StatusCode::NOT_FOUND, Json(ApiResponse::error(format!("Order {} not found", order_id)))))
}

pub async fn cancel(
  State(state): State<RestState>,
  Json(request): Json<CancelOrderRequest>,
) -> Result<Json<ApiResponse>, (StatusCode, Json<ApiResponse>)> {
  let mut snapshot = state.snapshot.write().await;

  if let Some(order) = snapshot.orders.iter_mut().find(|o| o.id == request.order_id) {
    order.status = "CANCELLED".into();
    snapshot.touch();
    snapshot.alerts.push(Alert::info(&format!("Order {} cancelled via REST", request.order_id)));
    trim_alerts(&mut snapshot.alerts);

    Ok(Json(ApiResponse::ok_with_data(
      format!("Order {} cancelled", request.order_id),
      serde_json::json!({ "order_id": request.order_id }),
    )))
  } else {
    Err((StatusCode::NOT_FOUND, Json(ApiResponse::error(format!("Order {} not found", request.order_id)))))
  }
}
