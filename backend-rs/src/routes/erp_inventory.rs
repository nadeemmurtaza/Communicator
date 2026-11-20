use axum::{Json, extract::{Path, Query, State}, http::StatusCode, routing::{get, post, put}};
use std::collections::HashMap;
use crate::AppState;
use serde_json::Value;

type ErpInventoryClient = newax_proto::erp_inventory::v1::erp_inventory_service_client::ErpInventoryServiceClient<tonic::transport::Channel>;

async fn client(state: &AppState) -> Result<ErpInventoryClient, (StatusCode, String)> {
    let endpoint = std::env::var("ERP_INVENTORY_GRPC_URL").unwrap_or("http://127.0.0.1:50101".to_string());
    ErpInventoryClient::connect(endpoint)
        .await
        .map_err(|e| (StatusCode::BAD_GATEWAY, format!("erp_inventory grpc: {}", e)))
}

// ============================================================
// Locations
// ============================================================

pub async fn list_locations(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let mut c = client(&state).await?;
    let is_active = params.get("isActive").and_then(|v| v.parse().ok());
    let req = newax_proto::erp_inventory::v1::ListLocationsRequest {
        has_is_active: is_active.is_some(),
        is_active: is_active.unwrap_or(false),
    };
    let resp = c.list_locations(tonic::Request::new(req)).await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    let parsed: Value = serde_json::from_str(&resp.into_inner().json)
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    Ok(Json(parsed))
}

pub async fn get_location(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let mut c = client(&state).await?;
    let req = newax_proto::erp_inventory::v1::GetLocationRequest { id };
    let resp = c.get_location(tonic::Request::new(req)).await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    let parsed: Value = serde_json::from_str(&resp.into_inner().json)
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    Ok(Json(parsed))
}

pub async fn create_location(
    State(state): State<AppState>,
    Json(req_body): Json<Value>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let mut c = client(&state).await?;
    let json_str = serde_json::to_string(&req_body)
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    let req = newax_proto::erp_inventory::v1::CreateLocationRequest {
        body: Some(newax_proto::erp_inventory::v1::JsonBody { json: json_str }),
    };
    let resp = c.create_location(tonic::Request::new(req)).await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    let parsed: Value = serde_json::from_str(&resp.into_inner().json)
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    Ok(Json(parsed))
}

pub async fn update_location(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req_body): Json<Value>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let mut c = client(&state).await?;
    let json_str = serde_json::to_string(&req_body)
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    let req = newax_proto::erp_inventory::v1::UpdateLocationRequest {
        id,
        body: Some(newax_proto::erp_inventory::v1::JsonBody { json: json_str }),
    };
    let resp = c.update_location(tonic::Request::new(req)).await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    let parsed: Value = serde_json::from_str(&resp.into_inner().json)
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    Ok(Json(parsed))
}

// ============================================================
// Variant Location Stock
// ============================================================

pub async fn get_stock_by_location(
    State(state): State<AppState>,
    Path(location_id): Path<String>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let mut c = client(&state).await?;
    let req = newax_proto::erp_inventory::v1::GetStockByLocationRequest { location_id };
    let resp = c.get_stock_by_location(tonic::Request::new(req)).await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    let parsed: Value = serde_json::from_str(&resp.into_inner().json)
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    Ok(Json(parsed))
}

pub async fn get_stock_by_variant(
    State(state): State<AppState>,
    Path(variant_id): Path<String>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let mut c = client(&state).await?;
    let req = newax_proto::erp_inventory::v1::GetStockByVariantRequest { variant_id };
    let resp = c.get_stock_by_variant(tonic::Request::new(req)).await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    let parsed: Value = serde_json::from_str(&resp.into_inner().json)
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    Ok(Json(parsed))
}

pub async fn update_variant_stock(
    State(state): State<AppState>,
    Path((variant_id, location_id)): Path<(String, String)>,
    Json(req_body): Json<Value>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let mut c = client(&state).await?;
    let quantity = req_body.get("quantity").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
    let req = newax_proto::erp_inventory::v1::UpdateVariantStockRequest {
        variant_id, location_id, quantity,
    };
    let resp = c.update_variant_stock(tonic::Request::new(req)).await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    let parsed: Value = serde_json::from_str(&resp.into_inner().json)
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    Ok(Json(parsed))
}

pub async fn get_total_stock(
    State(state): State<AppState>,
    Path(variant_id): Path<String>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let mut c = client(&state).await?;
    let req = newax_proto::erp_inventory::v1::GetTotalStockRequest { variant_id };
    let resp = c.get_total_stock(tonic::Request::new(req)).await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    let parsed: Value = serde_json::from_str(&resp.into_inner().json)
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    Ok(Json(parsed))
}

// ============================================================
// Stock Movements
// ============================================================

pub async fn list_stock_movements(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let mut c = client(&state).await?;
    let variant_id = params.get("variantId").cloned().unwrap_or_default();
    let reason = params.get("reason").cloned().unwrap_or_default();
    let location_id = params.get("locationId").cloned().unwrap_or_default();
    let limit: i64 = params.get("limit").and_then(|v| v.parse().ok()).unwrap_or(50);
    let offset: i64 = params.get("offset").and_then(|v| v.parse().ok()).unwrap_or(0);
    let req = newax_proto::erp_inventory::v1::ListStockMovementsRequest {
        variant_id, reason, location_id, limit, offset,
    };
    let resp = c.list_stock_movements(tonic::Request::new(req)).await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    let parsed: Value = serde_json::from_str(&resp.into_inner().json)
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    Ok(Json(parsed))
}

pub async fn get_stock_movement(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let mut c = client(&state).await?;
    let req = newax_proto::erp_inventory::v1::GetStockMovementRequest { id };
    let resp = c.get_stock_movement(tonic::Request::new(req)).await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    let parsed: Value = serde_json::from_str(&resp.into_inner().json)
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    Ok(Json(parsed))
}

pub async fn create_stock_movement(
    State(state): State<AppState>,
    Json(req_body): Json<Value>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let mut c = client(&state).await?;
    let json_str = serde_json::to_string(&req_body)
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    let req = newax_proto::erp_inventory::v1::CreateStockMovementRequest {
        body: Some(newax_proto::erp_inventory::v1::JsonBody { json: json_str }),
    };
    let resp = c.create_stock_movement(tonic::Request::new(req)).await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    let parsed: Value = serde_json::from_str(&resp.into_inner().json)
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    Ok(Json(parsed))
}

pub async fn get_movements_by_variant(
    State(state): State<AppState>,
    Path(variant_id): Path<String>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let mut c = client(&state).await?;
    let limit: i64 = params.get("limit").and_then(|v| v.parse().ok()).unwrap_or(50);
    let req = newax_proto::erp_inventory::v1::GetMovementsByVariantRequest {
        variant_id, limit,
    };
    let resp = c.get_movements_by_variant(tonic::Request::new(req)).await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    let parsed: Value = serde_json::from_str(&resp.into_inner().json)
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    Ok(Json(parsed))
}

// ============================================================
// Stock Variances
// ============================================================

pub async fn list_stock_variances(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let mut c = client(&state).await?;
    let status = params.get("status").cloned().unwrap_or_default();
    let variant_id = params.get("variantId").cloned().unwrap_or_default();
    let limit: i64 = params.get("limit").and_then(|v| v.parse().ok()).unwrap_or(50);
    let offset: i64 = params.get("offset").and_then(|v| v.parse().ok()).unwrap_or(0);
    let req = newax_proto::erp_inventory::v1::ListStockVariancesRequest {
        status, variant_id, limit, offset,
    };
    let resp = c.list_stock_variances(tonic::Request::new(req)).await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    let parsed: Value = serde_json::from_str(&resp.into_inner().json)
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    Ok(Json(parsed))
}

pub async fn get_stock_variance(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let mut c = client(&state).await?;
    let req = newax_proto::erp_inventory::v1::GetStockVarianceRequest { id };
    let resp = c.get_stock_variance(tonic::Request::new(req)).await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    let parsed: Value = serde_json::from_str(&resp.into_inner().json)
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    Ok(Json(parsed))
}

pub async fn create_stock_variance(
    State(state): State<AppState>,
    Json(req_body): Json<Value>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let mut c = client(&state).await?;
    let json_str = serde_json::to_string(&req_body)
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    let req = newax_proto::erp_inventory::v1::CreateStockVarianceRequest {
        body: Some(newax_proto::erp_inventory::v1::JsonBody { json: json_str }),
    };
    let resp = c.create_stock_variance(tonic::Request::new(req)).await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    let parsed: Value = serde_json::from_str(&resp.into_inner().json)
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    Ok(Json(parsed))
}

pub async fn resolve_variance(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req_body): Json<Value>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let mut c = client(&state).await?;
    let json_str = serde_json::to_string(&req_body)
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    let req = newax_proto::erp_inventory::v1::ResolveVarianceRequest {
        id,
        body: Some(newax_proto::erp_inventory::v1::JsonBody { json: json_str }),
    };
    let resp = c.resolve_variance(tonic::Request::new(req)).await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    let parsed: Value = serde_json::from_str(&resp.into_inner().json)
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    Ok(Json(parsed))
}

// ============================================================
// Spot Check Policies
// ============================================================

pub async fn list_spot_check_policies(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let mut c = client(&state).await?;
    let is_active = params.get("isActive").and_then(|v| v.parse().ok());
    let req = newax_proto::erp_inventory::v1::ListSpotCheckPoliciesRequest {
        has_is_active: is_active.is_some(),
        is_active: is_active.unwrap_or(false),
    };
    let resp = c.list_spot_check_policies(tonic::Request::new(req)).await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    let parsed: Value = serde_json::from_str(&resp.into_inner().json)
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    Ok(Json(parsed))
}

pub async fn create_spot_check_policy(
    State(state): State<AppState>,
    Json(req_body): Json<Value>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let mut c = client(&state).await?;
    let json_str = serde_json::to_string(&req_body)
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    let req = newax_proto::erp_inventory::v1::CreateSpotCheckPolicyRequest {
        body: Some(newax_proto::erp_inventory::v1::JsonBody { json: json_str }),
    };
    let resp = c.create_spot_check_policy(tonic::Request::new(req)).await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    let parsed: Value = serde_json::from_str(&resp.into_inner().json)
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    Ok(Json(parsed))
}

// ============================================================
// Spot Checks
// ============================================================

pub async fn list_spot_checks(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let mut c = client(&state).await?;
    let location_id = params.get("locationId").cloned().unwrap_or_default();
    let status = params.get("status").cloned().unwrap_or_default();
    let limit: i64 = params.get("limit").and_then(|v| v.parse().ok()).unwrap_or(50);
    let offset: i64 = params.get("offset").and_then(|v| v.parse().ok()).unwrap_or(0);
    let req = newax_proto::erp_inventory::v1::ListSpotChecksRequest {
        location_id, status, limit, offset,
    };
    let resp = c.list_spot_checks(tonic::Request::new(req)).await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    let parsed: Value = serde_json::from_str(&resp.into_inner().json)
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    Ok(Json(parsed))
}

pub async fn get_spot_check(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let mut c = client(&state).await?;
    let req = newax_proto::erp_inventory::v1::GetSpotCheckRequest { id };
    let resp = c.get_spot_check(tonic::Request::new(req)).await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    let parsed: Value = serde_json::from_str(&resp.into_inner().json)
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    Ok(Json(parsed))
}

pub async fn create_spot_check(
    State(state): State<AppState>,
    Json(req_body): Json<Value>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let mut c = client(&state).await?;
    let json_str = serde_json::to_string(&req_body)
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    let req = newax_proto::erp_inventory::v1::CreateSpotCheckRequest {
        body: Some(newax_proto::erp_inventory::v1::JsonBody { json: json_str }),
    };
    let resp = c.create_spot_check(tonic::Request::new(req)).await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    let parsed: Value = serde_json::from_str(&resp.into_inner().json)
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    Ok(Json(parsed))
}

pub async fn start_spot_check(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let mut c = client(&state).await?;
    let req = newax_proto::erp_inventory::v1::StartSpotCheckRequest { id };
    let resp = c.start_spot_check(tonic::Request::new(req)).await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    let parsed: Value = serde_json::from_str(&resp.into_inner().json)
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    Ok(Json(parsed))
}

pub async fn complete_spot_check(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let mut c = client(&state).await?;
    let req = newax_proto::erp_inventory::v1::CompleteSpotCheckRequest { id };
    let resp = c.complete_spot_check(tonic::Request::new(req)).await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    let parsed: Value = serde_json::from_str(&resp.into_inner().json)
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    Ok(Json(parsed))
}

// ============================================================
// Spot Check Lines
// ============================================================

pub async fn list_spot_check_lines(
    State(state): State<AppState>,
    Path(spot_check_id): Path<String>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let mut c = client(&state).await?;
    let req = newax_proto::erp_inventory::v1::ListSpotCheckLinesRequest { spot_check_id };
    let resp = c.list_spot_check_lines(tonic::Request::new(req)).await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    let parsed: Value = serde_json::from_str(&resp.into_inner().json)
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    Ok(Json(parsed))
}

pub async fn create_spot_check_line(
    State(state): State<AppState>,
    Json(req_body): Json<Value>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let mut c = client(&state).await?;
    let json_str = serde_json::to_string(&req_body)
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    let req = newax_proto::erp_inventory::v1::CreateSpotCheckLineRequest {
        body: Some(newax_proto::erp_inventory::v1::JsonBody { json: json_str }),
    };
    let resp = c.create_spot_check_line(tonic::Request::new(req)).await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    let parsed: Value = serde_json::from_str(&resp.into_inner().json)
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    Ok(Json(parsed))
}

pub async fn update_spot_check_line(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req_body): Json<Value>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let mut c = client(&state).await?;
    let json_str = serde_json::to_string(&req_body)
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    let req = newax_proto::erp_inventory::v1::UpdateSpotCheckLineRequest {
        id,
        body: Some(newax_proto::erp_inventory::v1::JsonBody { json: json_str }),
    };
    let resp = c.update_spot_check_line(tonic::Request::new(req)).await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    let parsed: Value = serde_json::from_str(&resp.into_inner().json)
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    Ok(Json(parsed))
}

// ============================================================
// Variant Sales Stats
// ============================================================

pub async fn get_variant_sales_stat(
    State(state): State<AppState>,
    Path(variant_id): Path<String>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let mut c = client(&state).await?;
    let req = newax_proto::erp_inventory::v1::GetVariantSalesStatRequest { variant_id };
    let resp = c.get_variant_sales_stat(tonic::Request::new(req)).await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    let parsed: Value = serde_json::from_str(&resp.into_inner().json)
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    Ok(Json(parsed))
}

pub async fn list_slow_movers(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let mut c = client(&state).await?;
    let limit: i64 = params.get("limit").and_then(|v| v.parse().ok()).unwrap_or(50);
    let offset: i64 = params.get("offset").and_then(|v| v.parse().ok()).unwrap_or(0);
    let req = newax_proto::erp_inventory::v1::ListSlowMoversRequest { limit, offset };
    let resp = c.list_slow_movers(tonic::Request::new(req)).await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    let parsed: Value = serde_json::from_str(&resp.into_inner().json)
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    Ok(Json(parsed))
}

// ============================================================
// Variant Forecasts
// ============================================================

pub async fn get_variant_forecast(
    State(state): State<AppState>,
    Path(variant_id): Path<String>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let mut c = client(&state).await?;
    let req = newax_proto::erp_inventory::v1::GetVariantForecastRequest { variant_id };
    let resp = c.get_variant_forecast(tonic::Request::new(req)).await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    let parsed: Value = serde_json::from_str(&resp.into_inner().json)
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    Ok(Json(parsed))
}

pub async fn create_variant_forecast(
    State(state): State<AppState>,
    Json(req_body): Json<Value>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let mut c = client(&state).await?;
    let json_str = serde_json::to_string(&req_body)
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    let req = newax_proto::erp_inventory::v1::CreateVariantForecastRequest {
        body: Some(newax_proto::erp_inventory::v1::JsonBody { json: json_str }),
    };
    let resp = c.create_variant_forecast(tonic::Request::new(req)).await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    let parsed: Value = serde_json::from_str(&resp.into_inner().json)
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    Ok(Json(parsed))
}

pub async fn list_forecasts(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let mut c = client(&state).await?;
    let limit: i64 = params.get("limit").and_then(|v| v.parse().ok()).unwrap_or(50);
    let offset: i64 = params.get("offset").and_then(|v| v.parse().ok()).unwrap_or(0);
    let req = newax_proto::erp_inventory::v1::ListForecastsRequest { limit, offset };
    let resp = c.list_forecasts(tonic::Request::new(req)).await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    let parsed: Value = serde_json::from_str(&resp.into_inner().json)
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    Ok(Json(parsed))
}

// ============================================================
// Markdown Suggestions
// ============================================================

pub async fn list_markdown_suggestions(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let mut c = client(&state).await?;
    let status = params.get("status").cloned().unwrap_or_default();
    let limit: i64 = params.get("limit").and_then(|v| v.parse().ok()).unwrap_or(50);
    let offset: i64 = params.get("offset").and_then(|v| v.parse().ok()).unwrap_or(0);
    let req = newax_proto::erp_inventory::v1::ListMarkdownSuggestionsRequest {
        status, limit, offset,
    };
    let resp = c.list_markdown_suggestions(tonic::Request::new(req)).await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    let parsed: Value = serde_json::from_str(&resp.into_inner().json)
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    Ok(Json(parsed))
}

pub async fn create_markdown_suggestion(
    State(state): State<AppState>,
    Json(req_body): Json<Value>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let mut c = client(&state).await?;
    let json_str = serde_json::to_string(&req_body)
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    let req = newax_proto::erp_inventory::v1::CreateMarkdownSuggestionRequest {
        body: Some(newax_proto::erp_inventory::v1::JsonBody { json: json_str }),
    };
    let resp = c.create_markdown_suggestion(tonic::Request::new(req)).await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    let parsed: Value = serde_json::from_str(&resp.into_inner().json)
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    Ok(Json(parsed))
}

pub async fn update_markdown_suggestion_status(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req_body): Json<Value>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let mut c = client(&state).await?;
    let json_str = serde_json::to_string(&req_body)
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    let req = newax_proto::erp_inventory::v1::UpdateMarkdownSuggestionStatusRequest {
        id,
        body: Some(newax_proto::erp_inventory::v1::JsonBody { json: json_str }),
    };
    let resp = c.update_markdown_suggestion_status(tonic::Request::new(req)).await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    let parsed: Value = serde_json::from_str(&resp.into_inner().json)
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    Ok(Json(parsed))
}

// ============================================================
// Lots
// ============================================================

pub async fn list_lots(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let mut c = client(&state).await?;
    let product_variant_id = params.get("productVariantId").cloned().unwrap_or_default();
    let limit: i64 = params.get("limit").and_then(|v| v.parse().ok()).unwrap_or(50);
    let offset: i64 = params.get("offset").and_then(|v| v.parse().ok()).unwrap_or(0);
    let req = newax_proto::erp_inventory::v1::ListLotsRequest {
        product_variant_id, limit, offset,
    };
    let resp = c.list_lots(tonic::Request::new(req)).await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    let parsed: Value = serde_json::from_str(&resp.into_inner().json)
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    Ok(Json(parsed))
}

pub async fn get_lot(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let mut c = client(&state).await?;
    let req = newax_proto::erp_inventory::v1::GetLotRequest { id };
    let resp = c.get_lot(tonic::Request::new(req)).await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    let parsed: Value = serde_json::from_str(&resp.into_inner().json)
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    Ok(Json(parsed))
}

pub async fn get_expiring_lots(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let mut c = client(&state).await?;
    let days: i32 = params.get("days").and_then(|v| v.parse().ok()).unwrap_or(30);
    let req = newax_proto::erp_inventory::v1::GetExpiringLotsRequest { days };
    let resp = c.get_expiring_lots(tonic::Request::new(req)).await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    let parsed: Value = serde_json::from_str(&resp.into_inner().json)
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    Ok(Json(parsed))
}

pub async fn create_lot(
    State(state): State<AppState>,
    Json(req_body): Json<Value>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let mut c = client(&state).await?;
    let json_str = serde_json::to_string(&req_body)
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    let req = newax_proto::erp_inventory::v1::CreateLotRequest {
        body: Some(newax_proto::erp_inventory::v1::JsonBody { json: json_str }),
    };
    let resp = c.create_lot(tonic::Request::new(req)).await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    let parsed: Value = serde_json::from_str(&resp.into_inner().json)
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    Ok(Json(parsed))
}

// ============================================================
// Lot Stock
// ============================================================

pub async fn list_lot_stock(
    State(state): State<AppState>,
    Path(lot_id): Path<String>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let mut c = client(&state).await?;
    let req = newax_proto::erp_inventory::v1::ListLotStockRequest { lot_id };
    let resp = c.list_lot_stock(tonic::Request::new(req)).await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    let parsed: Value = serde_json::from_str(&resp.into_inner().json)
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    Ok(Json(parsed))
}

pub async fn create_lot_stock(
    State(state): State<AppState>,
    Json(req_body): Json<Value>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let mut c = client(&state).await?;
    let json_str = serde_json::to_string(&req_body)
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    let req = newax_proto::erp_inventory::v1::CreateLotStockRequest {
        body: Some(newax_proto::erp_inventory::v1::JsonBody { json: json_str }),
    };
    let resp = c.create_lot_stock(tonic::Request::new(req)).await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    let parsed: Value = serde_json::from_str(&resp.into_inner().json)
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    Ok(Json(parsed))
}

// ============================================================
// Lot Allocations
// ============================================================

pub async fn list_lot_allocations(
    State(state): State<AppState>,
    Path(lot_id): Path<String>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let mut c = client(&state).await?;
    let req = newax_proto::erp_inventory::v1::ListLotAllocationsRequest { lot_id };
    let resp = c.list_lot_allocations(tonic::Request::new(req)).await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    let parsed: Value = serde_json::from_str(&resp.into_inner().json)
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    Ok(Json(parsed))
}

pub async fn create_lot_allocation(
    State(state): State<AppState>,
    Json(req_body): Json<Value>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let mut c = client(&state).await?;
    let json_str = serde_json::to_string(&req_body)
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    let req = newax_proto::erp_inventory::v1::CreateLotAllocationRequest {
        body: Some(newax_proto::erp_inventory::v1::JsonBody { json: json_str }),
    };
    let resp = c.create_lot_allocation(tonic::Request::new(req)).await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    let parsed: Value = serde_json::from_str(&resp.into_inner().json)
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    Ok(Json(parsed))
}

// ============================================================
// Serialized Units
// ============================================================

pub async fn list_serialized_units(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let mut c = client(&state).await?;
    let product_variant_id = params.get("productVariantId").cloned().unwrap_or_default();
    let status = params.get("status").cloned().unwrap_or_default();
    let limit: i64 = params.get("limit").and_then(|v| v.parse().ok()).unwrap_or(50);
    let offset: i64 = params.get("offset").and_then(|v| v.parse().ok()).unwrap_or(0);
    let req = newax_proto::erp_inventory::v1::ListSerializedUnitsRequest {
        product_variant_id, status, limit, offset,
    };
    let resp = c.list_serialized_units(tonic::Request::new(req)).await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    let parsed: Value = serde_json::from_str(&resp.into_inner().json)
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    Ok(Json(parsed))
}

pub async fn get_serialized_unit(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let mut c = client(&state).await?;
    let req = newax_proto::erp_inventory::v1::GetSerializedUnitRequest { id };
    let resp = c.get_serialized_unit(tonic::Request::new(req)).await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    let parsed: Value = serde_json::from_str(&resp.into_inner().json)
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    Ok(Json(parsed))
}

pub async fn get_serialized_unit_by_serial(
    State(state): State<AppState>,
    Path(serial_number): Path<String>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let mut c = client(&state).await?;
    let req = newax_proto::erp_inventory::v1::GetSerializedUnitBySerialRequest { serial_number };
    let resp = c.get_serialized_unit_by_serial(tonic::Request::new(req)).await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    let parsed: Value = serde_json::from_str(&resp.into_inner().json)
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    Ok(Json(parsed))
}

pub async fn create_serialized_unit(
    State(state): State<AppState>,
    Json(req_body): Json<Value>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let mut c = client(&state).await?;
    let json_str = serde_json::to_string(&req_body)
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    let req = newax_proto::erp_inventory::v1::CreateSerializedUnitRequest {
        body: Some(newax_proto::erp_inventory::v1::JsonBody { json: json_str }),
    };
    let resp = c.create_serialized_unit(tonic::Request::new(req)).await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    let parsed: Value = serde_json::from_str(&resp.into_inner().json)
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    Ok(Json(parsed))
}

pub async fn update_serialized_unit_status(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req_body): Json<Value>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let mut c = client(&state).await?;
    let json_str = serde_json::to_string(&req_body)
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    let req = newax_proto::erp_inventory::v1::UpdateSerializedUnitStatusRequest {
        id,
        body: Some(newax_proto::erp_inventory::v1::JsonBody { json: json_str }),
    };
    let resp = c.update_serialized_unit_status(tonic::Request::new(req)).await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    let parsed: Value = serde_json::from_str(&resp.into_inner().json)
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    Ok(Json(parsed))
}

// ============================================================
// Stock Counts
// ============================================================

pub async fn list_stock_counts(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let mut c = client(&state).await?;
    let status = params.get("status").cloned().unwrap_or_default();
    let limit: i64 = params.get("limit").and_then(|v| v.parse().ok()).unwrap_or(50);
    let offset: i64 = params.get("offset").and_then(|v| v.parse().ok()).unwrap_or(0);
    let req = newax_proto::erp_inventory::v1::ListStockCountsRequest {
        status, limit, offset,
    };
    let resp = c.list_stock_counts(tonic::Request::new(req)).await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    let parsed: Value = serde_json::from_str(&resp.into_inner().json)
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    Ok(Json(parsed))
}

pub async fn get_stock_count(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let mut c = client(&state).await?;
    let req = newax_proto::erp_inventory::v1::GetStockCountRequest { id };
    let resp = c.get_stock_count(tonic::Request::new(req)).await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    let parsed: Value = serde_json::from_str(&resp.into_inner().json)
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    Ok(Json(parsed))
}

pub async fn create_stock_count(
    State(state): State<AppState>,
    Json(req_body): Json<Value>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let mut c = client(&state).await?;
    let json_str = serde_json::to_string(&req_body)
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    let req = newax_proto::erp_inventory::v1::CreateStockCountRequest {
        body: Some(newax_proto::erp_inventory::v1::JsonBody { json: json_str }),
    };
    let resp = c.create_stock_count(tonic::Request::new(req)).await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    let parsed: Value = serde_json::from_str(&resp.into_inner().json)
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    Ok(Json(parsed))
}

pub async fn start_stock_count(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let mut c = client(&state).await?;
    let req = newax_proto::erp_inventory::v1::StartStockCountRequest { id };
    let resp = c.start_stock_count(tonic::Request::new(req)).await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    let parsed: Value = serde_json::from_str(&resp.into_inner().json)
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    Ok(Json(parsed))
}

pub async fn complete_stock_count(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let mut c = client(&state).await?;
    let req = newax_proto::erp_inventory::v1::CompleteStockCountRequest { id };
    let resp = c.complete_stock_count(tonic::Request::new(req)).await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    let parsed: Value = serde_json::from_str(&resp.into_inner().json)
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    Ok(Json(parsed))
}

pub async fn cancel_stock_count(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let mut c = client(&state).await?;
    let req = newax_proto::erp_inventory::v1::CancelStockCountRequest { id };
    let resp = c.cancel_stock_count(tonic::Request::new(req)).await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    let parsed: Value = serde_json::from_str(&resp.into_inner().json)
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    Ok(Json(parsed))
}

// ============================================================
// Stock Count Lines
// ============================================================

pub async fn list_stock_count_lines(
    State(state): State<AppState>,
    Path(stock_count_id): Path<String>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let mut c = client(&state).await?;
    let req = newax_proto::erp_inventory::v1::ListStockCountLinesRequest { stock_count_id };
    let resp = c.list_stock_count_lines(tonic::Request::new(req)).await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    let parsed: Value = serde_json::from_str(&resp.into_inner().json)
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    Ok(Json(parsed))
}

pub async fn create_stock_count_line(
    State(state): State<AppState>,
    Json(req_body): Json<Value>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let mut c = client(&state).await?;
    let json_str = serde_json::to_string(&req_body)
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    let req = newax_proto::erp_inventory::v1::CreateStockCountLineRequest {
        body: Some(newax_proto::erp_inventory::v1::JsonBody { json: json_str }),
    };
    let resp = c.create_stock_count_line(tonic::Request::new(req)).await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    let parsed: Value = serde_json::from_str(&resp.into_inner().json)
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    Ok(Json(parsed))
}

pub async fn update_stock_count_line(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req_body): Json<Value>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let mut c = client(&state).await?;
    let json_str = serde_json::to_string(&req_body)
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    let req = newax_proto::erp_inventory::v1::UpdateStockCountLineRequest {
        id,
        body: Some(newax_proto::erp_inventory::v1::JsonBody { json: json_str }),
    };
    let resp = c.update_stock_count_line(tonic::Request::new(req)).await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    let parsed: Value = serde_json::from_str(&resp.into_inner().json)
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    Ok(Json(parsed))
}

// ============================================================
// Inventory Stats
// ============================================================

pub async fn get_inventory_stats(
    State(state): State<AppState>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let mut c = client(&state).await?;
    let req = newax_proto::erp_inventory::v1::GetInventoryStatsRequest {};
    let resp = c.get_inventory_stats(tonic::Request::new(req)).await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    let parsed: Value = serde_json::from_str(&resp.into_inner().json)
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    Ok(Json(parsed))
}

// ============================================================
// Router
// ============================================================

pub fn router() -> axum::Router<AppState> {
    axum::Router::new()
        // Locations
        .route("/erp/inventory/locations", get(list_locations).post(create_location))
        .route("/erp/inventory/locations/:id", get(get_location).put(update_location))
        // Variant Location Stock
        .route("/erp/inventory/stock/location/:location_id", get(get_stock_by_location))
        .route("/erp/inventory/stock/variant/:variant_id", get(get_stock_by_variant))
        .route("/erp/inventory/stock/:variant_id/:location_id", put(update_variant_stock))
        .route("/erp/inventory/stock/:variant_id/total", get(get_total_stock))
        // Stock Movements
        .route("/erp/inventory/movements", get(list_stock_movements).post(create_stock_movement))
        .route("/erp/inventory/movements/:id", get(get_stock_movement))
        .route("/erp/inventory/movements/variant/:variant_id", get(get_movements_by_variant))
        // Stock Variances
        .route("/erp/inventory/variances", get(list_stock_variances).post(create_stock_variance))
        .route("/erp/inventory/variances/:id", get(get_stock_variance))
        .route("/erp/inventory/variances/:id/resolve", put(resolve_variance))
        // Spot Check Policies
        .route("/erp/inventory/spot-check-policies", get(list_spot_check_policies).post(create_spot_check_policy))
        // Spot Checks
        .route("/erp/inventory/spot-checks", get(list_spot_checks).post(create_spot_check))
        .route("/erp/inventory/spot-checks/:id", get(get_spot_check))
        .route("/erp/inventory/spot-checks/:id/start", put(start_spot_check))
        .route("/erp/inventory/spot-checks/:id/complete", put(complete_spot_check))
        // Spot Check Lines
        .route("/erp/inventory/spot-check-lines/:spot_check_id", get(list_spot_check_lines))
        .route("/erp/inventory/spot-check-lines", post(create_spot_check_line))
        .route("/erp/inventory/spot-check-lines/:id", put(update_spot_check_line))
        // Variant Sales Stats
        .route("/erp/inventory/sales-stats/:variant_id", get(get_variant_sales_stat))
        .route("/erp/inventory/slow-movers", get(list_slow_movers))
        // Variant Forecasts
        .route("/erp/inventory/forecasts/:variant_id", get(get_variant_forecast))
        .route("/erp/inventory/forecasts", post(create_variant_forecast).get(list_forecasts))
        // Markdown Suggestions
        .route("/erp/inventory/markdown-suggestions", get(list_markdown_suggestions).post(create_markdown_suggestion))
        .route("/erp/inventory/markdown-suggestions/:id/status", put(update_markdown_suggestion_status))
        // Lots
        .route("/erp/inventory/lots", get(list_lots).post(create_lot))
        .route("/erp/inventory/lots/:id", get(get_lot))
        .route("/erp/inventory/lots/expiring", get(get_expiring_lots))
        // Lot Stock
        .route("/erp/inventory/lot-stock/:lot_id", get(list_lot_stock))
        .route("/erp/inventory/lot-stock", post(create_lot_stock))
        // Lot Allocations
        .route("/erp/inventory/lot-allocations/:lot_id", get(list_lot_allocations))
        .route("/erp/inventory/lot-allocations", post(create_lot_allocation))
        // Serialized Units
        .route("/erp/inventory/serialized-units", get(list_serialized_units).post(create_serialized_unit))
        .route("/erp/inventory/serialized-units/:id", get(get_serialized_unit))
        .route("/erp/inventory/serialized-units/serial/:serial_number", get(get_serialized_unit_by_serial))
        .route("/erp/inventory/serialized-units/:id/status", put(update_serialized_unit_status))
        // Stock Counts
        .route("/erp/inventory/stock-counts", get(list_stock_counts).post(create_stock_count))
        .route("/erp/inventory/stock-counts/:id", get(get_stock_count))
        .route("/erp/inventory/stock-counts/:id/start", put(start_stock_count))
        .route("/erp/inventory/stock-counts/:id/complete", put(complete_stock_count))
        .route("/erp/inventory/stock-counts/:id/cancel", put(cancel_stock_count))
        // Stock Count Lines
        .route("/erp/inventory/stock-count-lines/:stock_count_id", get(list_stock_count_lines))
        .route("/erp/inventory/stock-count-lines", post(create_stock_count_line))
        .route("/erp/inventory/stock-count-lines/:id", put(update_stock_count_line))
        // Stats
        .route("/erp/inventory/stats", get(get_inventory_stats))
}
