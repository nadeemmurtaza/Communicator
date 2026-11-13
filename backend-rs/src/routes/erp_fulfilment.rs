//! Phase 39: ERP Fulfilment routes: Returns/RMA, shipping zones, methods, rates, manifests.

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use crate::AppState;
use crate::services::erp_fulfilment::ErpFulfilmentService;
use crate::services::erp_fulfilment::{
    CreateShippingZoneRequest, UpdateShippingZoneRequest,
    CreateShippingMethodRequest, UpdateShippingMethodRequest,
    CreateShippingRateRequest, CreateReturnRequestRequest,
    UpdateReturnStatusRequest, InspectReturnItemRequest,
};

// ---------------------------------------------------------------------------
// Summary
// ---------------------------------------------------------------------------

pub async fn get_summary(
    State(state): State<AppState>,
) -> Result<Json<crate::services::erp_fulfilment::FulfilmentSummary>, (StatusCode, String)> {
    let service = ErpFulfilmentService::new(state.db_pool.clone());
    service.get_summary().await.map(Json).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

// ---------------------------------------------------------------------------
// Shipping Zones
// ---------------------------------------------------------------------------

pub async fn list_shipping_zones(
    State(state): State<AppState>,
    Query(params): Query<serde_json::Value>,
) -> Result<Json<Vec<crate::services::erp_fulfilment::ErpShippingZone>>, (StatusCode, String)> {
    let service = ErpFulfilmentService::new(state.db_pool.clone());
    let active_only = params.get("activeOnly").and_then(|v| v.as_bool()).unwrap_or(true);
    service.list_shipping_zones(active_only).await.map(Json).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))
}

pub async fn get_shipping_zone(
    State(state): State<AppState>,
    Path(zone_id): Path<String>,
) -> Result<Json<crate::services::erp_fulfilment::ErpShippingZone>, (StatusCode, String)> {
    let service = ErpFulfilmentService::new(state.db_pool.clone());
    service.get_shipping_zone(&zone_id).await.map(Json).map_err(|e| (StatusCode::NOT_FOUND, e))
}

pub async fn create_shipping_zone(
    State(state): State<AppState>,
    Json(req): Json<CreateShippingZoneRequest>,
) -> Result<Json<crate::services::erp_fulfilment::ErpShippingZone>, (StatusCode, String)> {
    let service = ErpFulfilmentService::new(state.db_pool.clone());
    service.create_shipping_zone(&req).await.map(Json).map_err(|e| (StatusCode::BAD_REQUEST, e))
}

pub async fn update_shipping_zone(
    State(state): State<AppState>,
    Path(zone_id): Path<String>,
    Json(req): Json<UpdateShippingZoneRequest>,
) -> Result<Json<crate::services::erp_fulfilment::ErpShippingZone>, (StatusCode, String)> {
    let service = ErpFulfilmentService::new(state.db_pool.clone());
    service.update_shipping_zone(&zone_id, &req).await.map(Json).map_err(|e| (StatusCode::BAD_REQUEST, e))
}

pub async fn delete_shipping_zone(
    State(state): State<AppState>,
    Path(zone_id): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    let service = ErpFulfilmentService::new(state.db_pool.clone());
    service.delete_shipping_zone(&zone_id).await.map(|_| StatusCode::NO_CONTENT).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))
}

// ---------------------------------------------------------------------------
// Shipping Methods
// ---------------------------------------------------------------------------

pub async fn list_shipping_methods(
    State(state): State<AppState>,
    Query(params): Query<serde_json::Value>,
) -> Result<Json<Vec<crate::services::erp_fulfilment::ErpShippingMethod>>, (StatusCode, String)> {
    let service = ErpFulfilmentService::new(state.db_pool.clone());
    let active_only = params.get("activeOnly").and_then(|v| v.as_bool()).unwrap_or(true);
    service.list_shipping_methods(active_only).await.map(Json).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))
}

pub async fn get_shipping_method(
    State(state): State<AppState>,
    Path(method_id): Path<String>,
) -> Result<Json<crate::services::erp_fulfilment::ErpShippingMethod>, (StatusCode, String)> {
    let service = ErpFulfilmentService::new(state.db_pool.clone());
    service.get_shipping_method(&method_id).await.map(Json).map_err(|e| (StatusCode::NOT_FOUND, e))
}

pub async fn create_shipping_method(
    State(state): State<AppState>,
    Json(req): Json<CreateShippingMethodRequest>,
) -> Result<Json<crate::services::erp_fulfilment::ErpShippingMethod>, (StatusCode, String)> {
    let service = ErpFulfilmentService::new(state.db_pool.clone());
    service.create_shipping_method(&req).await.map(Json).map_err(|e| (StatusCode::BAD_REQUEST, e))
}

pub async fn update_shipping_method(
    State(state): State<AppState>,
    Path(method_id): Path<String>,
    Json(req): Json<UpdateShippingMethodRequest>,
) -> Result<Json<crate::services::erp_fulfilment::ErpShippingMethod>, (StatusCode, String)> {
    let service = ErpFulfilmentService::new(state.db_pool.clone());
    service.update_shipping_method(&method_id, &req).await.map(Json).map_err(|e| (StatusCode::BAD_REQUEST, e))
}

pub async fn delete_shipping_method(
    State(state): State<AppState>,
    Path(method_id): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    let service = ErpFulfilmentService::new(state.db_pool.clone());
    service.delete_shipping_method(&method_id).await.map(|_| StatusCode::NO_CONTENT).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))
}

// ---------------------------------------------------------------------------
// Shipping Rates
// ---------------------------------------------------------------------------

pub async fn list_shipping_rates(
    State(state): State<AppState>,
) -> Result<Json<Vec<crate::services::erp_fulfilment::ErpShippingRate>>, (StatusCode, String)> {
    let service = ErpFulfilmentService::new(state.db_pool.clone());
    service.list_shipping_rates(None, None).await.map(Json).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))
}

pub async fn create_shipping_rate(
    State(state): State<AppState>,
    Json(req): Json<CreateShippingRateRequest>,
) -> Result<Json<crate::services::erp_fulfilment::ErpShippingRate>, (StatusCode, String)> {
    let service = ErpFulfilmentService::new(state.db_pool.clone());
    service.create_shipping_rate(&req).await.map(Json).map_err(|e| (StatusCode::BAD_REQUEST, e))
}

pub async fn get_shipping_rate(
    State(state): State<AppState>,
    Path(rate_id): Path<String>,
) -> Result<Json<crate::services::erp_fulfilment::ErpShippingRate>, (StatusCode, String)> {
    let service = ErpFulfilmentService::new(state.db_pool.clone());
    service.get_shipping_rate(&rate_id).await.map(Json).map_err(|e| (StatusCode::NOT_FOUND, e))
}

pub async fn update_shipping_rate(
    State(state): State<AppState>,
    Path(rate_id): Path<String>,
    Json(req): Json<CreateShippingRateRequest>,
) -> Result<Json<crate::services::erp_fulfilment::ErpShippingRate>, (StatusCode, String)> {
    let service = ErpFulfilmentService::new(state.db_pool.clone());
    service.update_shipping_rate(&rate_id, &req).await.map(Json).map_err(|e| (StatusCode::BAD_REQUEST, e))
}

pub async fn delete_shipping_rate(
    State(state): State<AppState>,
    Path(rate_id): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    let service = ErpFulfilmentService::new(state.db_pool.clone());
    service.delete_shipping_rate(&rate_id).await.map(|_| StatusCode::NO_CONTENT).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))
}

// ---------------------------------------------------------------------------
// Return Reasons
// ---------------------------------------------------------------------------

pub async fn list_return_reasons(
    State(state): State<AppState>,
) -> Result<Json<Vec<crate::services::erp_fulfilment::ErpReturnReasonCode>>, (StatusCode, String)> {
    let service = ErpFulfilmentService::new(state.db_pool.clone());
    service.list_return_reasons().await.map(Json).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))
}

// ---------------------------------------------------------------------------
// Return Requests (RMA)
// ---------------------------------------------------------------------------

pub async fn list_return_requests(
    State(state): State<AppState>,
    Query(params): Query<serde_json::Value>,
) -> Result<Json<Vec<crate::services::erp_fulfilment::ErpReturnRequest>>, (StatusCode, String)> {
    let service = ErpFulfilmentService::new(state.db_pool.clone());
    let status = params.get("status").and_then(|v| v.as_str());
    let order_id = params.get("orderId").and_then(|v| v.as_str());
    service.list_return_requests(status, order_id).await.map(Json).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))
}

pub async fn get_return_request(
    State(state): State<AppState>,
    Path(return_id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let service = ErpFulfilmentService::new(state.db_pool.clone());
    let (request, items) = service.get_return_request(&return_id).await.map_err(|e| (StatusCode::NOT_FOUND, e))?;
    Ok(Json(serde_json::json!({ "request": request, "items": items })))
}

pub async fn create_return_request(
    State(state): State<AppState>,
    Json(req): Json<CreateReturnRequestRequest>,
) -> Result<Json<crate::services::erp_fulfilment::ErpReturnRequest>, (StatusCode, String)> {
    let service = ErpFulfilmentService::new(state.db_pool.clone());
    service.create_return_request(&req).await.map(Json).map_err(|e| (StatusCode::BAD_REQUEST, e))
}

pub async fn update_return_status(
    State(state): State<AppState>,
    Path(return_id): Path<String>,
    Json(req): Json<UpdateReturnStatusRequest>,
) -> Result<Json<crate::services::erp_fulfilment::ErpReturnRequest>, (StatusCode, String)> {
    let service = ErpFulfilmentService::new(state.db_pool.clone());
    service.update_return_status(&return_id, &req).await.map(Json).map_err(|e| (StatusCode::BAD_REQUEST, e))
}

pub async fn inspect_return_item(
    State(state): State<AppState>,
    Json(req): Json<InspectReturnItemRequest>,
) -> Result<Json<crate::services::erp_fulfilment::ErpReturnRequestItem>, (StatusCode, String)> {
    let service = ErpFulfilmentService::new(state.db_pool.clone());
    service.inspect_return_item(&req).await.map(Json).map_err(|e| (StatusCode::BAD_REQUEST, e))
}

pub async fn get_return_stats(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let service = ErpFulfilmentService::new(state.db_pool.clone());
    service.get_return_stats().await.map(Json).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))
}

// ---------------------------------------------------------------------------
// Manifests
// ---------------------------------------------------------------------------

pub async fn list_manifests(
    State(state): State<AppState>,
) -> Result<Json<Vec<crate::services::erp_fulfilment::ErpManifest>>, (StatusCode, String)> {
    let service = ErpFulfilmentService::new(state.db_pool.clone());
    service.list_manifests(None, None).await.map(Json).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))
}

pub async fn dispatch_manifest(
    State(state): State<AppState>,
    Path(manifest_id): Path<String>,
) -> Result<Json<crate::services::erp_fulfilment::ErpManifest>, (StatusCode, String)> {
    let service = ErpFulfilmentService::new(state.db_pool.clone());
    service.dispatch_manifest(&manifest_id).await.map(Json).map_err(|e| (StatusCode::BAD_REQUEST, e))
}
