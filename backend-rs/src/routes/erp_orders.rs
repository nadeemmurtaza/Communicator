use axum::{Json, extract::{Path, Query, State}, http::StatusCode};
use std::collections::HashMap;
use crate::AppState;
use serde_json::Value;

type ErpOrdersClient = newax_proto::communicator::erp_orders::v1::erp_orders_service_client::ErpOrdersServiceClient<tonic::transport::Channel>;

async fn client(state: &AppState) -> Result<ErpOrdersClient, (StatusCode, String)> {
    let endpoint = std::env::var("ERP_ORDERS_GRPC_URL").unwrap_or("http://127.0.0.1:50099".to_string());
    ErpOrdersClient::connect(endpoint)
        .await
        .map_err(|e| (StatusCode::BAD_GATEWAY, format!("erp_orders grpc: {}", e)))
}

// ============================================================
// Orders
// ============================================================

pub async fn list_orders(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let mut c = client(&state).await?;
    let status = params.get("status").cloned();
    let channel = params.get("channel").cloned();
    let limit: i64 = params.get("limit").and_then(|v| v.parse().ok()).unwrap_or(50);
    let offset: i64 = params.get("offset").and_then(|v| v.parse().ok()).unwrap_or(0);
    let req = newax_proto::communicator::erp_orders::v1::ListOrdersRequest {
        status, channel, limit, offset,
    };
    let resp = c.list_orders(tonic::Request::new(req)).await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    let parsed: Value = serde_json::from_str(&resp.into_inner().json)
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    Ok(Json(parsed))
}

pub async fn get_order(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let mut c = client(&state).await?;
    let req = newax_proto::communicator::erp_orders::v1::IdRequest { id };
    let resp = c.get_order(tonic::Request::new(req)).await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    let parsed: Value = serde_json::from_str(&resp.into_inner().json)
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    Ok(Json(parsed))
}

pub async fn get_order_by_number(
    State(state): State<AppState>,
    Path(number): Path<String>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let mut c = client(&state).await?;
    let req = newax_proto::communicator::erp_orders::v1::IdRequest { id: number };
    let resp = c.get_order_by_number(tonic::Request::new(req)).await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    let parsed: Value = serde_json::from_str(&resp.into_inner().json)
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    Ok(Json(parsed))
}

pub async fn create_order(
    State(state): State<AppState>,
    Json(req_body): Json<Value>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let mut c = client(&state).await?;
    let json_str = serde_json::to_string(&req_body)
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    let req = newax_proto::communicator::erp_orders::v1::CreateOrderRequest { json: json_str };
    let resp = c.create_order(tonic::Request::new(req)).await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    let parsed: Value = serde_json::from_str(&resp.into_inner().json)
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    Ok(Json(parsed))
}

pub async fn update_order_status(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req_body): Json<Value>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let mut c = client(&state).await?;
    let json_str = serde_json::to_string(&req_body)
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    let req = newax_proto::communicator::erp_orders::v1::UpdateOrderStatusRequest {
        order_id: id, json: json_str,
    };
    let resp = c.update_order_status(tonic::Request::new(req)).await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    let parsed: Value = serde_json::from_str(&resp.into_inner().json)
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    Ok(Json(parsed))
}

pub async fn cancel_order(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req_body): Json<Value>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let mut c = client(&state).await?;
    let reason = req_body.get("reason").and_then(|v| v.as_str()).unwrap_or("Cancelled by user").to_string();
    let actor_id = req_body.get("actorId").and_then(|v| v.as_str()).map(|s| s.to_string());
    let req = newax_proto::communicator::erp_orders::v1::CancelOrderRequest {
        order_id: id, reason, actor_id,
    };
    let resp = c.cancel_order(tonic::Request::new(req)).await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    let parsed: Value = serde_json::from_str(&resp.into_inner().json)
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    Ok(Json(parsed))
}

pub async fn get_order_items(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let mut c = client(&state).await?;
    let req = newax_proto::communicator::erp_orders::v1::IdRequest { id };
    let resp = c.get_order_items(tonic::Request::new(req)).await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    let parsed: Value = serde_json::from_str(&resp.into_inner().json)
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    Ok(Json(parsed))
}

pub async fn get_order_status_events(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let mut c = client(&state).await?;
    let req = newax_proto::communicator::erp_orders::v1::IdRequest { id };
    let resp = c.get_order_status_events(tonic::Request::new(req)).await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    let parsed: Value = serde_json::from_str(&resp.into_inner().json)
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    Ok(Json(parsed))
}

pub async fn get_order_stats(
    State(state): State<AppState>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let mut c = client(&state).await?;
    let req = newax_proto::communicator::erp_orders::v1::Empty {};
    let resp = c.get_order_stats(tonic::Request::new(req)).await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    let parsed: Value = serde_json::from_str(&resp.into_inner().json)
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    Ok(Json(parsed))
}

// ============================================================
// Carts
// ============================================================

pub async fn list_carts(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let mut c = client(&state).await?;
    let user_id = params.get("userId").cloned();
    let status = params.get("status").cloned();
    let req = newax_proto::communicator::erp_orders::v1::ListCartsRequest { user_id, status };
    let resp = c.list_carts(tonic::Request::new(req)).await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    let parsed: Value = serde_json::from_str(&resp.into_inner().json)
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    Ok(Json(parsed))
}

pub async fn get_cart(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let mut c = client(&state).await?;
    let req = newax_proto::communicator::erp_orders::v1::IdRequest { id };
    let resp = c.get_cart(tonic::Request::new(req)).await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    let parsed: Value = serde_json::from_str(&resp.into_inner().json)
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    Ok(Json(parsed))
}

pub async fn create_cart(
    State(state): State<AppState>,
    Json(req_body): Json<Value>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let mut c = client(&state).await?;
    let json_str = serde_json::to_string(&req_body)
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    let req = newax_proto::communicator::erp_orders::v1::CreateCartRequest { json: json_str };
    let resp = c.create_cart(tonic::Request::new(req)).await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    let parsed: Value = serde_json::from_str(&resp.into_inner().json)
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    Ok(Json(parsed))
}

pub async fn delete_cart(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    let mut c = client(&state).await?;
    let req = newax_proto::communicator::erp_orders::v1::IdRequest { id };
    c.delete_cart(tonic::Request::new(req)).await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn get_cart_items(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let mut c = client(&state).await?;
    let req = newax_proto::communicator::erp_orders::v1::IdRequest { id };
    let resp = c.get_cart_items(tonic::Request::new(req)).await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    let parsed: Value = serde_json::from_str(&resp.into_inner().json)
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    Ok(Json(parsed))
}

pub async fn add_cart_item(
    State(state): State<AppState>,
    Json(req_body): Json<Value>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let mut c = client(&state).await?;
    let json_str = serde_json::to_string(&req_body)
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    let req = newax_proto::communicator::erp_orders::v1::AddCartItemRequest { json: json_str };
    let resp = c.add_cart_item(tonic::Request::new(req)).await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    let parsed: Value = serde_json::from_str(&resp.into_inner().json)
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    Ok(Json(parsed))
}

pub async fn update_cart_item(
    State(state): State<AppState>,
    Path(item_id): Path<String>,
    Json(req_body): Json<Value>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let mut c = client(&state).await?;
    let json_str = serde_json::to_string(&req_body)
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    let req = newax_proto::communicator::erp_orders::v1::UpdateCartItemRequest {
        item_id, json: json_str,
    };
    let resp = c.update_cart_item(tonic::Request::new(req)).await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    let parsed: Value = serde_json::from_str(&resp.into_inner().json)
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    Ok(Json(parsed))
}

pub async fn remove_cart_item(
    State(state): State<AppState>,
    Path(item_id): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    let mut c = client(&state).await?;
    let req = newax_proto::communicator::erp_orders::v1::IdRequest { id: item_id };
    c.remove_cart_item(tonic::Request::new(req)).await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn checkout_cart(
    State(state): State<AppState>,
    Json(req_body): Json<Value>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let mut c = client(&state).await?;
    let json_str = serde_json::to_string(&req_body)
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    let req = newax_proto::communicator::erp_orders::v1::CheckoutCartRequest { json: json_str };
    let resp = c.checkout_cart(tonic::Request::new(req)).await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    let parsed: Value = serde_json::from_str(&resp.into_inner().json)
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    Ok(Json(parsed))
}

// ============================================================
// Shipments
// ============================================================

pub async fn list_shipments(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let mut c = client(&state).await?;
    let order_id = params.get("orderId").cloned();
    let status = params.get("status").cloned();
    let req = newax_proto::communicator::erp_orders::v1::ListShipmentsRequest { order_id, status };
    let resp = c.list_shipments(tonic::Request::new(req)).await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    let parsed: Value = serde_json::from_str(&resp.into_inner().json)
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    Ok(Json(parsed))
}

pub async fn get_shipment(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let mut c = client(&state).await?;
    let req = newax_proto::communicator::erp_orders::v1::IdRequest { id };
    let resp = c.get_shipment(tonic::Request::new(req)).await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    let parsed: Value = serde_json::from_str(&resp.into_inner().json)
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    Ok(Json(parsed))
}

pub async fn create_shipment(
    State(state): State<AppState>,
    Json(req_body): Json<Value>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let mut c = client(&state).await?;
    let json_str = serde_json::to_string(&req_body)
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    let req = newax_proto::communicator::erp_orders::v1::CreateShipmentRequest { json: json_str };
    let resp = c.create_shipment(tonic::Request::new(req)).await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    let parsed: Value = serde_json::from_str(&resp.into_inner().json)
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    Ok(Json(parsed))
}

pub async fn update_shipment_status(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req_body): Json<Value>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let mut c = client(&state).await?;
    let json_str = serde_json::to_string(&req_body)
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    let req = newax_proto::communicator::erp_orders::v1::UpdateShipmentStatusRequest {
        shipment_id: id, json: json_str,
    };
    let resp = c.update_shipment_status(tonic::Request::new(req)).await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    let parsed: Value = serde_json::from_str(&resp.into_inner().json)
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    Ok(Json(parsed))
}

pub async fn get_shipment_events(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let mut c = client(&state).await?;
    let req = newax_proto::communicator::erp_orders::v1::IdRequest { id };
    let resp = c.get_shipment_events(tonic::Request::new(req)).await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    let parsed: Value = serde_json::from_str(&resp.into_inner().json)
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    Ok(Json(parsed))
}

pub async fn create_shipment_event(
    State(state): State<AppState>,
    Json(req_body): Json<Value>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let mut c = client(&state).await?;
    let json_str = serde_json::to_string(&req_body)
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    let req = newax_proto::communicator::erp_orders::v1::CreateShipmentEventRequest { json: json_str };
    let resp = c.create_shipment_event(tonic::Request::new(req)).await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    let parsed: Value = serde_json::from_str(&resp.into_inner().json)
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    Ok(Json(parsed))
}

// ============================================================
// Back in Stock Requests
// ============================================================

pub async fn list_back_in_stock_requests(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let mut c = client(&state).await?;
    let product_id = params.get("productId").cloned();
    let status = params.get("status").cloned();
    let req = newax_proto::communicator::erp_orders::v1::ListBackInStockRequestsRequest {
        product_id, status,
    };
    let resp = c.list_back_in_stock_requests(tonic::Request::new(req)).await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    let parsed: Value = serde_json::from_str(&resp.into_inner().json)
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    Ok(Json(parsed))
}

pub async fn create_back_in_stock_request(
    State(state): State<AppState>,
    Json(req_body): Json<Value>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let mut c = client(&state).await?;
    let json_str = serde_json::to_string(&req_body)
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    let req = newax_proto::communicator::erp_orders::v1::CreateBackInStockRequest { json: json_str };
    let resp = c.create_back_in_stock_request(tonic::Request::new(req)).await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    let parsed: Value = serde_json::from_str(&resp.into_inner().json)
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    Ok(Json(parsed))
}

pub async fn notify_back_in_stock(
    State(state): State<AppState>,
    Path(product_id): Path<String>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let mut c = client(&state).await?;
    let req = newax_proto::communicator::erp_orders::v1::IdRequest { id: product_id };
    let resp = c.notify_back_in_stock(tonic::Request::new(req)).await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    let parsed: Value = serde_json::from_str(&resp.into_inner().json)
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    Ok(Json(parsed))
}
