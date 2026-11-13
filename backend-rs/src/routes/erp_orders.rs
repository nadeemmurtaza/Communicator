use axum::{Json, extract::{Path, Query, State}, http::StatusCode};
use std::collections::HashMap;
use crate::db::DbPool;
use crate::AppState;
use crate::models::*;
use crate::services::erp_orders::ErpOrdersService;

// ============================================================
// Orders
// ============================================================

pub async fn list_orders(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Vec<ErpOrder>>, (StatusCode, String)> {
    let pool: &DbPool = &state.db;
    let status = params.get("status").map(|s| s.as_str());
    let channel = params.get("channel").map(|s| s.as_str());
    let limit: i64 = params.get("limit").and_then(|v| v.parse().ok()).unwrap_or(50);
    let offset: i64 = params.get("offset").and_then(|v| v.parse().ok()).unwrap_or(0);

    match ErpOrdersService::list_orders(pool, status, channel, limit, offset).await {
        Ok(orders) => Ok(Json(orders)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

pub async fn get_order(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ErpOrder>, (StatusCode, String)> {
    let pool: &DbPool = &state.db;
    match ErpOrdersService::get_order(pool, &id).await {
        Ok(order) => Ok(Json(order)),
        Err(e) => Err((StatusCode::NOT_FOUND, e)),
    }
}

pub async fn get_order_by_number(
    State(state): State<AppState>,
    Path(number): Path<String>,
) -> Result<Json<ErpOrder>, (StatusCode, String)> {
    let pool: &DbPool = &state.db;
    match ErpOrdersService::get_order_by_number(pool, &number).await {
        Ok(order) => Ok(Json(order)),
        Err(e) => Err((StatusCode::NOT_FOUND, e)),
    }
}

pub async fn create_order(
    State(state): State<AppState>,
    Json(req): Json<CreateOrderRequest>,
) -> Result<Json<ErpOrder>, (StatusCode, String)> {
    let pool: &DbPool = &state.db;
    match ErpOrdersService::create_order(pool, &req).await {
        Ok(order) => Ok(Json(order)),
        Err(e) => Err((StatusCode::BAD_REQUEST, e)),
    }
}

pub async fn update_order_status(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<UpdateOrderStatusRequest>,
) -> Result<Json<ErpOrder>, (StatusCode, String)> {
    let pool: &DbPool = &state.db;
    match ErpOrdersService::update_order_status(pool, &id, &req).await {
        Ok(order) => Ok(Json(order)),
        Err(e) => Err((StatusCode::BAD_REQUEST, e)),
    }
}

pub async fn cancel_order(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<serde_json::Value>,
) -> Result<Json<ErpOrder>, (StatusCode, String)> {
    let pool: &DbPool = &state.db;
    let reason = req.get("reason").and_then(|v| v.as_str()).unwrap_or("Cancelled by user");
    let actor_id = req.get("actorId").and_then(|v| v.as_str());

    match ErpOrdersService::cancel_order(pool, &id, reason, actor_id).await {
        Ok(order) => Ok(Json(order)),
        Err(e) => Err((StatusCode::BAD_REQUEST, e)),
    }
}

pub async fn get_order_items(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Vec<ErpOrderItem>>, (StatusCode, String)> {
    let pool: &DbPool = &state.db;
    match ErpOrdersService::get_order_items(pool, &id).await {
        Ok(items) => Ok(Json(items)),
        Err(e) => Err((StatusCode::NOT_FOUND, e)),
    }
}

pub async fn get_order_status_events(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Vec<ErpOrderStatusEvent>>, (StatusCode, String)> {
    let pool: &DbPool = &state.db;
    match ErpOrdersService::get_order_status_events(pool, &id).await {
        Ok(events) => Ok(Json(events)),
        Err(e) => Err((StatusCode::NOT_FOUND, e)),
    }
}

pub async fn get_order_stats(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let pool: &DbPool = &state.db;
    match ErpOrdersService::get_order_stats(pool).await {
        Ok(stats) => Ok(Json(stats)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

// ============================================================
// Carts
// ============================================================

pub async fn list_carts(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Vec<ErpCart>>, (StatusCode, String)> {
    let pool: &DbPool = &state.db;
    let user_id = params.get("userId").map(|s| s.as_str());
    let status = params.get("status").map(|s| s.as_str());

    match ErpOrdersService::list_carts(pool, user_id, status).await {
        Ok(carts) => Ok(Json(carts)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

pub async fn get_cart(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ErpCart>, (StatusCode, String)> {
    let pool: &DbPool = &state.db;
    match ErpOrdersService::get_cart(pool, &id).await {
        Ok(cart) => Ok(Json(cart)),
        Err(e) => Err((StatusCode::NOT_FOUND, e)),
    }
}

pub async fn create_cart(
    State(state): State<AppState>,
    Json(req): Json<CreateCartRequest>,
) -> Result<Json<ErpCart>, (StatusCode, String)> {
    let pool: &DbPool = &state.db;
    match ErpOrdersService::create_cart(pool, &req).await {
        Ok(cart) => Ok(Json(cart)),
        Err(e) => Err((StatusCode::BAD_REQUEST, e)),
    }
}

pub async fn delete_cart(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    let pool: &DbPool = &state.db;
    match ErpOrdersService::delete_cart(pool, &id).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

pub async fn get_cart_items(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Vec<ErpCartItem>>, (StatusCode, String)> {
    let pool: &DbPool = &state.db;
    match ErpOrdersService::get_cart_items(pool, &id).await {
        Ok(items) => Ok(Json(items)),
        Err(e) => Err((StatusCode::NOT_FOUND, e)),
    }
}

pub async fn add_cart_item(
    State(state): State<AppState>,
    Json(req): Json<CreateCartItemRequest>,
) -> Result<Json<ErpCartItem>, (StatusCode, String)> {
    let pool: &DbPool = &state.db;
    match ErpOrdersService::add_cart_item(pool, &req).await {
        Ok(item) => Ok(Json(item)),
        Err(e) => Err((StatusCode::BAD_REQUEST, e)),
    }
}

pub async fn update_cart_item(
    State(state): State<AppState>,
    Path(item_id): Path<String>,
    Json(req): Json<UpdateCartItemRequest>,
) -> Result<Json<ErpCartItem>, (StatusCode, String)> {
    let pool: &DbPool = &state.db;
    match ErpOrdersService::update_cart_item(pool, &item_id, &req).await {
        Ok(item) => Ok(Json(item)),
        Err(e) => Err((StatusCode::BAD_REQUEST, e)),
    }
}

pub async fn remove_cart_item(
    State(state): State<AppState>,
    Path(item_id): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    let pool: &DbPool = &state.db;
    match ErpOrdersService::remove_cart_item(pool, &item_id).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

pub async fn checkout_cart(
    State(state): State<AppState>,
    Json(req): Json<CheckoutCartRequest>,
) -> Result<Json<ErpOrder>, (StatusCode, String)> {
    let pool: &DbPool = &state.db;
    match ErpOrdersService::checkout_cart(pool, &req).await {
        Ok(order) => Ok(Json(order)),
        Err(e) => Err((StatusCode::BAD_REQUEST, e)),
    }
}

// ============================================================
// Shipments
// ============================================================

pub async fn list_shipments(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Vec<ErpShipment>>, (StatusCode, String)> {
    let pool: &DbPool = &state.db;
    let order_id = params.get("orderId").map(|s| s.as_str());
    let status = params.get("status").map(|s| s.as_str());

    match ErpOrdersService::list_shipments(pool, order_id, status).await {
        Ok(shipments) => Ok(Json(shipments)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

pub async fn get_shipment(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ErpShipment>, (StatusCode, String)> {
    let pool: &DbPool = &state.db;
    match ErpOrdersService::get_shipment(pool, &id).await {
        Ok(shipment) => Ok(Json(shipment)),
        Err(e) => Err((StatusCode::NOT_FOUND, e)),
    }
}

pub async fn create_shipment(
    State(state): State<AppState>,
    Json(req): Json<CreateShipmentRequest>,
) -> Result<Json<ErpShipment>, (StatusCode, String)> {
    let pool: &DbPool = &state.db;
    match ErpOrdersService::create_shipment(pool, &req).await {
        Ok(shipment) => Ok(Json(shipment)),
        Err(e) => Err((StatusCode::BAD_REQUEST, e)),
    }
}

pub async fn update_shipment_status(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<UpdateShipmentStatusRequest>,
) -> Result<Json<ErpShipment>, (StatusCode, String)> {
    let pool: &DbPool = &state.db;
    match ErpOrdersService::update_shipment_status(pool, &id, &req).await {
        Ok(shipment) => Ok(Json(shipment)),
        Err(e) => Err((StatusCode::BAD_REQUEST, e)),
    }
}

pub async fn get_shipment_events(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Vec<ErpShipmentEvent>>, (StatusCode, String)> {
    let pool: &DbPool = &state.db;
    match ErpOrdersService::get_shipment_events(pool, &id).await {
        Ok(events) => Ok(Json(events)),
        Err(e) => Err((StatusCode::NOT_FOUND, e)),
    }
}

pub async fn create_shipment_event(
    State(state): State<AppState>,
    Json(req): Json<CreateShipmentEventRequest>,
) -> Result<Json<ErpShipmentEvent>, (StatusCode, String)> {
    let pool: &DbPool = &state.db;
    match ErpOrdersService::create_shipment_event(pool, &req).await {
        Ok(event) => Ok(Json(event)),
        Err(e) => Err((StatusCode::BAD_REQUEST, e)),
    }
}

// ============================================================
// Back in Stock Requests
// ============================================================

pub async fn list_back_in_stock_requests(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Vec<ErpBackInStockRequest>>, (StatusCode, String)> {
    let pool: &DbPool = &state.db;
    let product_id = params.get("productId").map(|s| s.as_str());
    let status = params.get("status").map(|s| s.as_str());

    match ErpOrdersService::list_back_in_stock_requests(pool, product_id, status).await {
        Ok(requests) => Ok(Json(requests)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

pub async fn create_back_in_stock_request(
    State(state): State<AppState>,
    Json(req): Json<CreateBackInStockRequestRequest>,
) -> Result<Json<ErpBackInStockRequest>, (StatusCode, String)> {
    let pool: &DbPool = &state.db;
    match ErpOrdersService::create_back_in_stock_request(pool, &req).await {
        Ok(request) => Ok(Json(request)),
        Err(e) => Err((StatusCode::BAD_REQUEST, e)),
    }
}

pub async fn notify_back_in_stock(
    State(state): State<AppState>,
    Path(product_id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let pool: &DbPool = &state.db;
    match ErpOrdersService::notify_back_in_stock(pool, &product_id).await {
        Ok(count) => Ok(Json(serde_json::json!({ "notified": count }))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}
