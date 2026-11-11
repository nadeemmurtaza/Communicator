use axum::{Json, extract::{Path, Query, State}, http::StatusCode, routing::{get, post, put}};
use std::collections::HashMap;
use serde::Deserialize;
use crate::db::DbPool;
use crate::AppState;
use crate::models::*;
use crate::services::erp_inventory::ErpInventoryService;

// ============================================================
// Locations
// ============================================================

pub async fn list_locations(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Vec<ErpLocation>>, (StatusCode, String)> {
    let pool: &DbPool = &state.db;
    let is_active = params.get("isActive").and_then(|v| v.parse().ok());
    match ErpInventoryService::list_locations(pool, is_active).await {
        Ok(locations) => Ok(Json(locations)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

pub async fn get_location(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ErpLocation>, (StatusCode, String)> {
    let pool: &DbPool = &state.db;
    match ErpInventoryService::get_location(pool, &id).await {
        Ok(location) => Ok(Json(location)),
        Err(e) => Err((StatusCode::NOT_FOUND, e)),
    }
}

pub async fn create_location(
    State(state): State<AppState>,
    Json(req): Json<CreateLocationRequest>,
) -> Result<Json<ErpLocation>, (StatusCode, String)> {
    let pool: &DbPool = &state.db;
    match ErpInventoryService::create_location(pool, &req).await {
        Ok(location) => Ok(Json(location)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

pub async fn update_location(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<UpdateLocationRequest>,
) -> Result<Json<ErpLocation>, (StatusCode, String)> {
    let pool: &DbPool = &state.db;
    match ErpInventoryService::update_location(pool, &id, &req).await {
        Ok(location) => Ok(Json(location)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

// ============================================================
// Variant Location Stock
// ============================================================

pub async fn get_stock_by_location(
    State(state): State<AppState>,
    Path(location_id): Path<String>,
) -> Result<Json<Vec<ErpVariantLocationStock>>, (StatusCode, String)> {
    let pool: &DbPool = &state.db;
    match ErpInventoryService::get_stock_by_location(pool, &location_id).await {
        Ok(stocks) => Ok(Json(stocks)),
        Err(e) => Err((StatusCode::NOT_FOUND, e)),
    }
}

pub async fn get_stock_by_variant(
    State(state): State<AppState>,
    Path(variant_id): Path<String>,
) -> Result<Json<Vec<ErpVariantLocationStock>>, (StatusCode, String)> {
    let pool: &DbPool = &state.db;
    match ErpInventoryService::get_stock_by_variant(pool, &variant_id).await {
        Ok(stocks) => Ok(Json(stocks)),
        Err(e) => Err((StatusCode::NOT_FOUND, e)),
    }
}

pub async fn update_variant_stock(
    State(state): State<AppState>,
    Path((variant_id, location_id)): Path<(String, String)>,
    Json(req): Json<UpdateVariantStockRequest>,
) -> Result<Json<ErpVariantLocationStock>, (StatusCode, String)> {
    let pool: &DbPool = &state.db;
    match ErpInventoryService::update_variant_location_stock(pool, &variant_id, &location_id, req.quantity).await {
        Ok(stock) => Ok(Json(stock)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

pub async fn get_total_stock(
    State(state): State<AppState>,
    Path(variant_id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let pool: &DbPool = &state.db;
    match ErpInventoryService::get_total_stock(pool, &variant_id).await {
        Ok(total) => Ok(Json(serde_json::json!({"variantId": variant_id, "totalStock": total}))),
        Err(e) => Err((StatusCode::NOT_FOUND, e)),
    }
}

// ============================================================
// Stock Movements
// ============================================================

pub async fn list_stock_movements(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Vec<ErpStockMovement>>, (StatusCode, String)> {
    let pool: &DbPool = &state.db;
    let variant_id = params.get("variantId").map(|s| s.as_str());
    let reason = params.get("reason").map(|s| s.as_str());
    let location_id = params.get("locationId").map(|s| s.as_str());
    let limit: i64 = params.get("limit").and_then(|v| v.parse().ok()).unwrap_or(50);
    let offset: i64 = params.get("offset").and_then(|v| v.parse().ok()).unwrap_or(0);

    match ErpInventoryService::list_stock_movements(pool, variant_id, reason, location_id, limit, offset).await {
        Ok(movements) => Ok(Json(movements)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

pub async fn get_stock_movement(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ErpStockMovement>, (StatusCode, String)> {
    let pool: &DbPool = &state.db;
    match ErpInventoryService::get_stock_movement(pool, &id).await {
        Ok(movement) => Ok(Json(movement)),
        Err(e) => Err((StatusCode::NOT_FOUND, e)),
    }
}

pub async fn create_stock_movement(
    State(state): State<AppState>,
    Json(req): Json<CreateStockMovementRequest>,
) -> Result<Json<ErpStockMovement>, (StatusCode, String)> {
    let pool: &DbPool = &state.db;
    match ErpInventoryService::create_stock_movement(pool, &req).await {
        Ok(movement) => Ok(Json(movement)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

pub async fn get_movements_by_variant(
    State(state): State<AppState>,
    Path(variant_id): Path<String>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Vec<ErpStockMovement>>, (StatusCode, String)> {
    let pool: &DbPool = &state.db;
    let limit: i64 = params.get("limit").and_then(|v| v.parse().ok()).unwrap_or(50);
    match ErpInventoryService::get_movements_by_variant(pool, &variant_id, limit).await {
        Ok(movements) => Ok(Json(movements)),
        Err(e) => Err((StatusCode::NOT_FOUND, e)),
    }
}

// ============================================================
// Stock Variances
// ============================================================

pub async fn list_stock_variances(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Vec<ErpStockVariance>>, (StatusCode, String)> {
    let pool: &DbPool = &state.db;
    let status = params.get("status").map(|s| s.as_str());
    let variant_id = params.get("variantId").map(|s| s.as_str());
    let limit: i64 = params.get("limit").and_then(|v| v.parse().ok()).unwrap_or(50);
    let offset: i64 = params.get("offset").and_then(|v| v.parse().ok()).unwrap_or(0);

    match ErpInventoryService::list_stock_variances(pool, status, variant_id, limit, offset).await {
        Ok(variances) => Ok(Json(variances)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

pub async fn get_stock_variance(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ErpStockVariance>, (StatusCode, String)> {
    let pool: &DbPool = &state.db;
    match ErpInventoryService::get_stock_variance(pool, &id).await {
        Ok(variance) => Ok(Json(variance)),
        Err(e) => Err((StatusCode::NOT_FOUND, e)),
    }
}

pub async fn create_stock_variance(
    State(state): State<AppState>,
    Json(req): Json<CreateStockVarianceRequest>,
) -> Result<Json<ErpStockVariance>, (StatusCode, String)> {
    let pool: &DbPool = &state.db;
    match ErpInventoryService::create_stock_variance(pool, &req).await {
        Ok(variance) => Ok(Json(variance)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

pub async fn resolve_variance(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<ResolveVarianceRequest>,
) -> Result<Json<ErpStockVariance>, (StatusCode, String)> {
    let pool: &DbPool = &state.db;
    match ErpInventoryService::resolve_variance(pool, &id, &req).await {
        Ok(variance) => Ok(Json(variance)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

// ============================================================
// Spot Check Policies
// ============================================================

pub async fn list_spot_check_policies(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Vec<ErpSpotCheckPolicy>>, (StatusCode, String)> {
    let pool: &DbPool = &state.db;
    let is_active = params.get("isActive").and_then(|v| v.parse().ok());
    match ErpInventoryService::list_spot_check_policies(pool, is_active).await {
        Ok(policies) => Ok(Json(policies)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

pub async fn create_spot_check_policy(
    State(state): State<AppState>,
    Json(req): Json<CreateSpotCheckPolicyRequest>,
) -> Result<Json<ErpSpotCheckPolicy>, (StatusCode, String)> {
    let pool: &DbPool = &state.db;
    match ErpInventoryService::create_spot_check_policy(pool, &req).await {
        Ok(policy) => Ok(Json(policy)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

// ============================================================
// Spot Checks
// ============================================================

pub async fn list_spot_checks(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Vec<ErpSpotCheck>>, (StatusCode, String)> {
    let pool: &DbPool = &state.db;
    let location_id = params.get("locationId").map(|s| s.as_str());
    let status = params.get("status").map(|s| s.as_str());
    let limit: i64 = params.get("limit").and_then(|v| v.parse().ok()).unwrap_or(50);
    let offset: i64 = params.get("offset").and_then(|v| v.parse().ok()).unwrap_or(0);

    match ErpInventoryService::list_spot_checks(pool, location_id, status, limit, offset).await {
        Ok(checks) => Ok(Json(checks)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

pub async fn get_spot_check(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ErpSpotCheck>, (StatusCode, String)> {
    let pool: &DbPool = &state.db;
    match ErpInventoryService::get_spot_check(pool, &id).await {
        Ok(check) => Ok(Json(check)),
        Err(e) => Err((StatusCode::NOT_FOUND, e)),
    }
}

pub async fn create_spot_check(
    State(state): State<AppState>,
    Json(req): Json<CreateSpotCheckRequest>,
) -> Result<Json<ErpSpotCheck>, (StatusCode, String)> {
    let pool: &DbPool = &state.db;
    match ErpInventoryService::create_spot_check(pool, &req).await {
        Ok(check) => Ok(Json(check)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

pub async fn start_spot_check(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ErpSpotCheck>, (StatusCode, String)> {
    let pool: &DbPool = &state.db;
    match ErpInventoryService::start_spot_check(pool, &id).await {
        Ok(check) => Ok(Json(check)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

pub async fn complete_spot_check(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ErpSpotCheck>, (StatusCode, String)> {
    let pool: &DbPool = &state.db;
    match ErpInventoryService::complete_spot_check(pool, &id).await {
        Ok(check) => Ok(Json(check)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

// ============================================================
// Spot Check Lines
// ============================================================

pub async fn list_spot_check_lines(
    State(state): State<AppState>,
    Path(spot_check_id): Path<String>,
) -> Result<Json<Vec<ErpSpotCheckLine>>, (StatusCode, String)> {
    let pool: &DbPool = &state.db;
    match ErpInventoryService::list_spot_check_lines(pool, &spot_check_id).await {
        Ok(lines) => Ok(Json(lines)),
        Err(e) => Err((StatusCode::NOT_FOUND, e)),
    }
}

pub async fn create_spot_check_line(
    State(state): State<AppState>,
    Json(req): Json<CreateSpotCheckLineRequest>,
) -> Result<Json<ErpSpotCheckLine>, (StatusCode, String)> {
    let pool: &DbPool = &state.db;
    match ErpInventoryService::create_spot_check_line(pool, &req).await {
        Ok(line) => Ok(Json(line)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

pub async fn update_spot_check_line(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<UpdateSpotCheckLineRequest>,
) -> Result<Json<ErpSpotCheckLine>, (StatusCode, String)> {
    let pool: &DbPool = &state.db;
    match ErpInventoryService::update_spot_check_line(pool, &id, &req).await {
        Ok(line) => Ok(Json(line)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

// ============================================================
// Variant Sales Stats
// ============================================================

pub async fn get_variant_sales_stat(
    State(state): State<AppState>,
    Path(variant_id): Path<String>,
) -> Result<Json<ErpVariantSalesStat>, (StatusCode, String)> {
    let pool: &DbPool = &state.db;
    match ErpInventoryService::get_variant_sales_stat(pool, &variant_id).await {
        Ok(stat) => Ok(Json(stat)),
        Err(e) => Err((StatusCode::NOT_FOUND, e)),
    }
}

pub async fn list_slow_movers(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Vec<ErpVariantSalesStat>>, (StatusCode, String)> {
    let pool: &DbPool = &state.db;
    let limit: i64 = params.get("limit").and_then(|v| v.parse().ok()).unwrap_or(50);
    let offset: i64 = params.get("offset").and_then(|v| v.parse().ok()).unwrap_or(0);

    match ErpInventoryService::list_slow_movers(pool, limit, offset).await {
        Ok(stats) => Ok(Json(stats)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

// ============================================================
// Variant Forecasts
// ============================================================

pub async fn get_variant_forecast(
    State(state): State<AppState>,
    Path(variant_id): Path<String>,
) -> Result<Json<ErpVariantForecast>, (StatusCode, String)> {
    let pool: &DbPool = &state.db;
    match ErpInventoryService::get_variant_forecast(pool, &variant_id).await {
        Ok(forecast) => Ok(Json(forecast)),
        Err(e) => Err((StatusCode::NOT_FOUND, e)),
    }
}

pub async fn create_variant_forecast(
    State(state): State<AppState>,
    Json(req): Json<CreateVariantForecastRequest>,
) -> Result<Json<ErpVariantForecast>, (StatusCode, String)> {
    let pool: &DbPool = &state.db;
    match ErpInventoryService::create_variant_forecast(pool, &req).await {
        Ok(forecast) => Ok(Json(forecast)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

pub async fn list_forecasts(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Vec<ErpVariantForecast>>, (StatusCode, String)> {
    let pool: &DbPool = &state.db;
    let limit: i64 = params.get("limit").and_then(|v| v.parse().ok()).unwrap_or(50);
    let offset: i64 = params.get("offset").and_then(|v| v.parse().ok()).unwrap_or(0);

    match ErpInventoryService::list_forecasts(pool, limit, offset).await {
        Ok(forecasts) => Ok(Json(forecasts)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

// ============================================================
// Markdown Suggestions
// ============================================================

pub async fn list_markdown_suggestions(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Vec<ErpMarkdownSuggestion>>, (StatusCode, String)> {
    let pool: &DbPool = &state.db;
    let status = params.get("status").map(|s| s.as_str());
    let limit: i64 = params.get("limit").and_then(|v| v.parse().ok()).unwrap_or(50);
    let offset: i64 = params.get("offset").and_then(|v| v.parse().ok()).unwrap_or(0);

    match ErpInventoryService::list_markdown_suggestions(pool, status, limit, offset).await {
        Ok(suggestions) => Ok(Json(suggestions)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

pub async fn create_markdown_suggestion(
    State(state): State<AppState>,
    Json(req): Json<CreateMarkdownSuggestionRequest>,
) -> Result<Json<ErpMarkdownSuggestion>, (StatusCode, String)> {
    let pool: &DbPool = &state.db;
    match ErpInventoryService::create_markdown_suggestion(pool, &req).await {
        Ok(suggestion) => Ok(Json(suggestion)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

pub async fn update_markdown_suggestion_status(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<UpdateMarkdownSuggestionStatusRequest>,
) -> Result<Json<ErpMarkdownSuggestion>, (StatusCode, String)> {
    let pool: &DbPool = &state.db;
    match ErpInventoryService::update_markdown_suggestion_status(pool, &id, &req).await {
        Ok(suggestion) => Ok(Json(suggestion)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

// ============================================================
// Lots
// ============================================================

pub async fn list_lots(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Vec<ErpLot>>, (StatusCode, String)> {
    let pool: &DbPool = &state.db;
    let product_variant_id = params.get("productVariantId").map(|s| s.as_str());
    let limit: i64 = params.get("limit").and_then(|v| v.parse().ok()).unwrap_or(50);
    let offset: i64 = params.get("offset").and_then(|v| v.parse().ok()).unwrap_or(0);

    match ErpInventoryService::list_lots(pool, product_variant_id, limit, offset).await {
        Ok(lots) => Ok(Json(lots)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

pub async fn get_lot(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ErpLot>, (StatusCode, String)> {
    let pool: &DbPool = &state.db;
    match ErpInventoryService::get_lot(pool, &id).await {
        Ok(lot) => Ok(Json(lot)),
        Err(e) => Err((StatusCode::NOT_FOUND, e)),
    }
}

pub async fn get_expiring_lots(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Vec<ErpLot>>, (StatusCode, String)> {
    let pool: &DbPool = &state.db;
    let days: i32 = params.get("days").and_then(|v| v.parse().ok()).unwrap_or(30);
    match ErpInventoryService::get_expiring_lots(pool, days).await {
        Ok(lots) => Ok(Json(lots)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

pub async fn create_lot(
    State(state): State<AppState>,
    Json(req): Json<CreateLotRequest>,
) -> Result<Json<ErpLot>, (StatusCode, String)> {
    let pool: &DbPool = &state.db;
    match ErpInventoryService::create_lot(pool, &req).await {
        Ok(lot) => Ok(Json(lot)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

// ============================================================
// Lot Stock
// ============================================================

pub async fn list_lot_stock(
    State(state): State<AppState>,
    Path(lot_id): Path<String>,
) -> Result<Json<Vec<ErpLotStock>>, (StatusCode, String)> {
    let pool: &DbPool = &state.db;
    match ErpInventoryService::list_lot_stock(pool, &lot_id).await {
        Ok(stocks) => Ok(Json(stocks)),
        Err(e) => Err((StatusCode::NOT_FOUND, e)),
    }
}

pub async fn create_lot_stock(
    State(state): State<AppState>,
    Json(req): Json<CreateLotStockRequest>,
) -> Result<Json<ErpLotStock>, (StatusCode, String)> {
    let pool: &DbPool = &state.db;
    match ErpInventoryService::create_lot_stock(pool, &req).await {
        Ok(stock) => Ok(Json(stock)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

// ============================================================
// Lot Allocations
// ============================================================

pub async fn list_lot_allocations(
    State(state): State<AppState>,
    Path(lot_id): Path<String>,
) -> Result<Json<Vec<ErpLotAllocation>>, (StatusCode, String)> {
    let pool: &DbPool = &state.db;
    match ErpInventoryService::list_lot_allocations(pool, &lot_id).await {
        Ok(allocations) => Ok(Json(allocations)),
        Err(e) => Err((StatusCode::NOT_FOUND, e)),
    }
}

pub async fn create_lot_allocation(
    State(state): State<AppState>,
    Json(req): Json<CreateLotAllocationRequest>,
) -> Result<Json<ErpLotAllocation>, (StatusCode, String)> {
    let pool: &DbPool = &state.db;
    match ErpInventoryService::create_lot_allocation(pool, &req).await {
        Ok(allocation) => Ok(Json(allocation)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

// ============================================================
// Serialized Units
// ============================================================

pub async fn list_serialized_units(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Vec<ErpSerializedUnit>>, (StatusCode, String)> {
    let pool: &DbPool = &state.db;
    let product_variant_id = params.get("productVariantId").map(|s| s.as_str());
    let status = params.get("status").map(|s| s.as_str());
    let limit: i64 = params.get("limit").and_then(|v| v.parse().ok()).unwrap_or(50);
    let offset: i64 = params.get("offset").and_then(|v| v.parse().ok()).unwrap_or(0);

    match ErpInventoryService::list_serialized_units(pool, product_variant_id, status, limit, offset).await {
        Ok(units) => Ok(Json(units)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

pub async fn get_serialized_unit(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ErpSerializedUnit>, (StatusCode, String)> {
    let pool: &DbPool = &state.db;
    match ErpInventoryService::get_serialized_unit(pool, &id).await {
        Ok(unit) => Ok(Json(unit)),
        Err(e) => Err((StatusCode::NOT_FOUND, e)),
    }
}

pub async fn get_serialized_unit_by_serial(
    State(state): State<AppState>,
    Path(serial_number): Path<String>,
) -> Result<Json<ErpSerializedUnit>, (StatusCode, String)> {
    let pool: &DbPool = &state.db;
    match ErpInventoryService::get_serialized_unit_by_serial(pool, &serial_number).await {
        Ok(unit) => Ok(Json(unit)),
        Err(e) => Err((StatusCode::NOT_FOUND, e)),
    }
}

pub async fn create_serialized_unit(
    State(state): State<AppState>,
    Json(req): Json<CreateSerializedUnitRequest>,
) -> Result<Json<ErpSerializedUnit>, (StatusCode, String)> {
    let pool: &DbPool = &state.db;
    match ErpInventoryService::create_serialized_unit(pool, &req).await {
        Ok(unit) => Ok(Json(unit)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

pub async fn update_serialized_unit_status(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<UpdateSerializedUnitStatusRequest>,
) -> Result<Json<ErpSerializedUnit>, (StatusCode, String)> {
    let pool: &DbPool = &state.db;
    match ErpInventoryService::update_serialized_unit_status(pool, &id, &req).await {
        Ok(unit) => Ok(Json(unit)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

// ============================================================
// Stock Counts
// ============================================================

pub async fn list_stock_counts(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Vec<ErpStockCount>>, (StatusCode, String)> {
    let pool: &DbPool = &state.db;
    let status = params.get("status").map(|s| s.as_str());
    let limit: i64 = params.get("limit").and_then(|v| v.parse().ok()).unwrap_or(50);
    let offset: i64 = params.get("offset").and_then(|v| v.parse().ok()).unwrap_or(0);

    match ErpInventoryService::list_stock_counts(pool, status, limit, offset).await {
        Ok(counts) => Ok(Json(counts)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

pub async fn get_stock_count(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ErpStockCount>, (StatusCode, String)> {
    let pool: &DbPool = &state.db;
    match ErpInventoryService::get_stock_count(pool, &id).await {
        Ok(count) => Ok(Json(count)),
        Err(e) => Err((StatusCode::NOT_FOUND, e)),
    }
}

pub async fn create_stock_count(
    State(state): State<AppState>,
    Json(req): Json<CreateStockCountRequest>,
) -> Result<Json<ErpStockCount>, (StatusCode, String)> {
    let pool: &DbPool = &state.db;
    match ErpInventoryService::create_stock_count(pool, &req).await {
        Ok(count) => Ok(Json(count)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

pub async fn start_stock_count(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ErpStockCount>, (StatusCode, String)> {
    let pool: &DbPool = &state.db;
    match ErpInventoryService::start_stock_count(pool, &id).await {
        Ok(count) => Ok(Json(count)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

pub async fn complete_stock_count(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ErpStockCount>, (StatusCode, String)> {
    let pool: &DbPool = &state.db;
    match ErpInventoryService::complete_stock_count(pool, &id).await {
        Ok(count) => Ok(Json(count)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

pub async fn cancel_stock_count(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ErpStockCount>, (StatusCode, String)> {
    let pool: &DbPool = &state.db;
    match ErpInventoryService::cancel_stock_count(pool, &id).await {
        Ok(count) => Ok(Json(count)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

// ============================================================
// Stock Count Lines
// ============================================================

pub async fn list_stock_count_lines(
    State(state): State<AppState>,
    Path(stock_count_id): Path<String>,
) -> Result<Json<Vec<ErpStockCountLine>>, (StatusCode, String)> {
    let pool: &DbPool = &state.db;
    match ErpInventoryService::list_stock_count_lines(pool, &stock_count_id).await {
        Ok(lines) => Ok(Json(lines)),
        Err(e) => Err((StatusCode::NOT_FOUND, e)),
    }
}

pub async fn create_stock_count_line(
    State(state): State<AppState>,
    Json(req): Json<CreateStockCountLineRequest>,
) -> Result<Json<ErpStockCountLine>, (StatusCode, String)> {
    let pool: &DbPool = &state.db;
    match ErpInventoryService::create_stock_count_line(pool, &req).await {
        Ok(line) => Ok(Json(line)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

pub async fn update_stock_count_line(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<UpdateStockCountLineRequest>,
) -> Result<Json<ErpStockCountLine>, (StatusCode, String)> {
    let pool: &DbPool = &state.db;
    match ErpInventoryService::update_stock_count_line(pool, &id, &req).await {
        Ok(line) => Ok(Json(line)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

// ============================================================
// Inventory Stats
// ============================================================

pub async fn get_inventory_stats(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let pool: &DbPool = &state.db;
    match ErpInventoryService::get_inventory_stats(pool).await {
        Ok(stats) => Ok(Json(stats)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
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

#[derive(Debug, Deserialize)]
pub struct UpdateVariantStockRequest {
    pub quantity: i32,
}
