//! ERP Fulfilment routes — thin gRPC client forwarding to svc-erp-fulfilment.

use axum::extract::{Path, Query};
use axum::http::StatusCode;
use axum::Json;
use newax_proto::erp_fulfilment::v1::erp_fulfilment_service_client::ErpFulfilmentServiceClient;
use newax_proto::erp_fulfilment::v1::{Empty, IdRequest, JsonResponse, ListShippingZonesRequest, CreateShippingZoneRequest, UpdateShippingZoneRequest, ListShippingMethodsRequest, CreateShippingMethodRequest, CreateShippingRateRequest, UpdateShippingRateRequest, CreateReturnRequestRequest, InspectReturnItemRequest};

async fn client() -> Result<ErpFulfilmentServiceClient<tonic::transport::Channel>, (StatusCode, String)> {
    let url = std::env::var("ERP_FULFILMENT_SERVICE_URL")
        .unwrap_or_else(|_| "http://127.0.0.1:50097".to_string());
    ErpFulfilmentServiceClient::connect(url)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("erp-fulfilment service unavailable: {e}")))
}

// ---------------------------------------------------------------------------
// Summary
// ---------------------------------------------------------------------------

pub async fn get_summary() -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut client = client().await?;
    let resp = client.get_summary(Empty {}).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let json: serde_json::Value = serde_json::from_str(&resp.into_inner().json).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(json))
}

// ---------------------------------------------------------------------------
// Shipping Zones
// ---------------------------------------------------------------------------

pub async fn list_shipping_zones(
    Query(params): Query<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut client = client().await?;
    let active_only = params.get("activeOnly").and_then(|v| v.as_bool()).unwrap_or(true);
    let resp = client.list_shipping_zones(ListShippingZonesRequest { active_only }).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let json: serde_json::Value = serde_json::from_str(&resp.into_inner().json).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(json))
}

pub async fn get_shipping_zone(
    Path(zone_id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut client = client().await?;
    let resp = client.get_shipping_zone(IdRequest { id: zone_id }).await.map_err(|e| (StatusCode::NOT_FOUND, e.to_string()))?;
    let json: serde_json::Value = serde_json::from_str(&resp.into_inner().json).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(json))
}

pub async fn create_shipping_zone(
    Json(req): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut client = client().await?;
    let json = serde_json::to_string(&req).map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    let resp = client.create_shipping_zone(CreateShippingZoneRequest { json }).await.map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    let json: serde_json::Value = serde_json::from_str(&resp.into_inner().json).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(json))
}

pub async fn update_shipping_zone(
    Path(zone_id): Path<String>,
    Json(req): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut client = client().await?;
    let json = serde_json::to_string(&req).map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    let resp = client.update_shipping_zone(UpdateShippingZoneRequest { zone_id, json }).await.map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    let json: serde_json::Value = serde_json::from_str(&resp.into_inner().json).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(json))
}

pub async fn delete_shipping_zone(
    Path(zone_id): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    let mut client = client().await?;
    client.delete_shipping_zone(IdRequest { id: zone_id }).await.map(|_| StatusCode::NO_CONTENT).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

// ---------------------------------------------------------------------------
// Shipping Methods
// ---------------------------------------------------------------------------

pub async fn list_shipping_methods(
    Query(params): Query<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut client = client().await?;
    let active_only = params.get("activeOnly").and_then(|v| v.as_bool()).unwrap_or(true);
    let resp = client.list_shipping_methods(ListShippingMethodsRequest { active_only }).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let json: serde_json::Value = serde_json::from_str(&resp.into_inner().json).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(json))
}

pub async fn get_shipping_method(
    Path(method_id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut client = client().await?;
    let resp = client.get_shipping_method(IdRequest { id: method_id }).await.map_err(|e| (StatusCode::NOT_FOUND, e.to_string()))?;
    let json: serde_json::Value = serde_json::from_str(&resp.into_inner().json).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(json))
}

pub async fn create_shipping_method(
    Json(req): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut client = client().await?;
    let json = serde_json::to_string(&req).map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    let resp = client.create_shipping_method(CreateShippingMethodRequest { json }).await.map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    let json: serde_json::Value = serde_json::from_str(&resp.into_inner().json).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(json))
}

pub async fn update_shipping_method(
    Path(method_id): Path<String>,
    Json(req): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut client = client().await?;
    let json = serde_json::to_string(&req).map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    let resp = client.update_shipping_method(newax_proto::erp_fulfilment::v1::UpdateShippingMethodRequest { method_id, json }).await.map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    let json: serde_json::Value = serde_json::from_str(&resp.into_inner().json).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(json))
}

pub async fn delete_shipping_method(
    Path(method_id): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    let mut client = client().await?;
    client.delete_shipping_method(IdRequest { id: method_id }).await.map(|_| StatusCode::NO_CONTENT).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

// ---------------------------------------------------------------------------
// Shipping Rates
// ---------------------------------------------------------------------------

pub async fn list_shipping_rates() -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut client = client().await?;
    let resp = client.list_shipping_rates(Empty {}).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let json: serde_json::Value = serde_json::from_str(&resp.into_inner().json).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(json))
}

pub async fn create_shipping_rate(
    Json(req): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut client = client().await?;
    let json = serde_json::to_string(&req).map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    let resp = client.create_shipping_rate(CreateShippingRateRequest { json }).await.map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    let json: serde_json::Value = serde_json::from_str(&resp.into_inner().json).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(json))
}

pub async fn get_shipping_rate(
    Path(rate_id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut client = client().await?;
    let resp = client.get_shipping_rate(IdRequest { id: rate_id }).await.map_err(|e| (StatusCode::NOT_FOUND, e.to_string()))?;
    let json: serde_json::Value = serde_json::from_str(&resp.into_inner().json).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(json))
}

pub async fn update_shipping_rate(
    Path(rate_id): Path<String>,
    Json(req): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut client = client().await?;
    let json = serde_json::to_string(&req).map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    let resp = client.update_shipping_rate(UpdateShippingRateRequest { rate_id, json }).await.map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    let json: serde_json::Value = serde_json::from_str(&resp.into_inner().json).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(json))
}

pub async fn delete_shipping_rate(
    Path(rate_id): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    let mut client = client().await?;
    client.delete_shipping_rate(IdRequest { id: rate_id }).await.map(|_| StatusCode::NO_CONTENT).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

// ---------------------------------------------------------------------------
// Return Reasons
// ---------------------------------------------------------------------------

pub async fn list_return_reasons() -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut client = client().await?;
    let resp = client.list_return_reasons(Empty {}).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let json: serde_json::Value = serde_json::from_str(&resp.into_inner().json).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(json))
}

// ---------------------------------------------------------------------------
// Return Requests (RMA)
// ---------------------------------------------------------------------------

pub async fn list_return_requests(
    Query(_params): Query<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut client = client().await?;
    let resp = client.list_return_requests(Empty {}).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let json: serde_json::Value = serde_json::from_str(&resp.into_inner().json).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(json))
}

pub async fn get_return_request(
    Path(return_id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut client = client().await?;
    let resp = client.get_return_request(IdRequest { id: return_id }).await.map_err(|e| (StatusCode::NOT_FOUND, e.to_string()))?;
    let json: serde_json::Value = serde_json::from_str(&resp.into_inner().json).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(json))
}

pub async fn create_return_request(
    Json(req): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut client = client().await?;
    let json = serde_json::to_string(&req).map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    let resp = client.create_return_request(CreateReturnRequestRequest { json }).await.map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    let json: serde_json::Value = serde_json::from_str(&resp.into_inner().json).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(json))
}

pub async fn update_return_status(
    Path(return_id): Path<String>,
    Json(req): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut client = client().await?;
    let json = serde_json::to_string(&req).map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    let resp = client.update_return_status(newax_proto::erp_fulfilment::v1::UpdateReturnStatusRequest { return_id, json }).await.map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    let json: serde_json::Value = serde_json::from_str(&resp.into_inner().json).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(json))
}

pub async fn inspect_return_item(
    Json(req): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut client = client().await?;
    let json = serde_json::to_string(&req).map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    let resp = client.inspect_return_item(InspectReturnItemRequest { json }).await.map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    let json: serde_json::Value = serde_json::from_str(&resp.into_inner().json).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(json))
}

pub async fn get_return_stats() -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut client = client().await?;
    let resp = client.get_return_stats(Empty {}).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let json: serde_json::Value = serde_json::from_str(&resp.into_inner().json).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(json))
}

// ---------------------------------------------------------------------------
// Manifests
// ---------------------------------------------------------------------------

pub async fn list_manifests() -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut client = client().await?;
    let resp = client.list_manifests(Empty {}).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let json: serde_json::Value = serde_json::from_str(&resp.into_inner().json).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(json))
}

pub async fn dispatch_manifest(
    Path(manifest_id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut client = client().await?;
    let resp = client.dispatch_manifest(IdRequest { id: manifest_id }).await.map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    let json: serde_json::Value = serde_json::from_str(&resp.into_inner().json).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(json))
}
