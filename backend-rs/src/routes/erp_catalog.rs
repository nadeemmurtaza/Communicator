use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use crate::db::AppState;
use crate::models::*;
use crate::services::erp_catalog::ErpCatalogService;

// ─── Categories ───

pub async fn list_categories(
    State(state): State<AppState>,
    Query(params): Query<serde_json::Value>,
) -> Result<Json<Vec<ErpCategory>>, (StatusCode, String)> {
    let active_only = params.get("activeOnly").and_then(|v| v.as_bool()).unwrap_or(true);
    match ErpCatalogService::list_categories(&state.db, active_only).await {
        Ok(cats) => Ok(Json(cats)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

pub async fn get_category(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ErpCategory>, (StatusCode, String)> {
    match ErpCatalogService::get_category(&state.db, &id).await {
        Ok(cat) => Ok(Json(cat)),
        Err(e) => Err((StatusCode::NOT_FOUND, e)),
    }
}

pub async fn create_category(
    State(state): State<AppState>,
    Json(req): Json<CreateCategoryRequest>,
) -> Result<(StatusCode, Json<ErpCategory>), (StatusCode, String)> {
    match ErpCatalogService::create_category(&state.db, &req).await {
        Ok(cat) => Ok((StatusCode::CREATED, Json(cat))),
        Err(e) => Err((StatusCode::BAD_REQUEST, e)),
    }
}

pub async fn update_category(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<UpdateCategoryRequest>,
) -> Result<Json<ErpCategory>, (StatusCode, String)> {
    match ErpCatalogService::update_category(&state.db, &id, &req).await {
        Ok(cat) => Ok(Json(cat)),
        Err(e) => Err((StatusCode::NOT_FOUND, e)),
    }
}

pub async fn delete_category(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    match ErpCatalogService::delete_category(&state.db, &id).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

// ─── Products ───

pub async fn list_products(
    State(state): State<AppState>,
    Query(params): Query<serde_json::Value>,
) -> Result<Json<Vec<ErpProduct>>, (StatusCode, String)> {
    let status = params.get("status").and_then(|v| v.as_str()).map(|s| s.to_string());
    let kind = params.get("kind").and_then(|v| v.as_str()).map(|s| s.to_string());
    let limit = params.get("limit").and_then(|v| v.as_i64()).unwrap_or(50);
    let offset = params.get("offset").and_then(|v| v.as_i64()).unwrap_or(0);
    match ErpCatalogService::list_products(&state.db, status.as_deref(), kind.as_deref(), limit, offset).await {
        Ok(prods) => Ok(Json(prods)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

pub async fn get_product(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ErpProduct>, (StatusCode, String)> {
    match ErpCatalogService::get_product(&state.db, &id).await {
        Ok(prod) => Ok(Json(prod)),
        Err(e) => Err((StatusCode::NOT_FOUND, e)),
    }
}

pub async fn get_product_by_slug(
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> Result<Json<ErpProduct>, (StatusCode, String)> {
    match ErpCatalogService::get_product_by_slug(&state.db, &slug).await {
        Ok(prod) => Ok(Json(prod)),
        Err(e) => Err((StatusCode::NOT_FOUND, e)),
    }
}

pub async fn create_product(
    State(state): State<AppState>,
    Json(req): Json<CreateProductRequest>,
) -> Result<(StatusCode, Json<ErpProduct>), (StatusCode, String)> {
    match ErpCatalogService::create_product(&state.db, &req).await {
        Ok(prod) => Ok((StatusCode::CREATED, Json(prod))),
        Err(e) => Err((StatusCode::BAD_REQUEST, e)),
    }
}

pub async fn update_product(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<UpdateProductRequest>,
) -> Result<Json<ErpProduct>, (StatusCode, String)> {
    match ErpCatalogService::update_product(&state.db, &id, &req).await {
        Ok(prod) => Ok(Json(prod)),
        Err(e) => Err((StatusCode::NOT_FOUND, e)),
    }
}

pub async fn delete_product(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    match ErpCatalogService::delete_product(&state.db, &id).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

// ─── Product Translations ───

pub async fn list_product_translations(
    State(state): State<AppState>,
    Path(product_id): Path<String>,
) -> Result<Json<Vec<ErpProductTranslation>>, (StatusCode, String)> {
    match ErpCatalogService::list_product_translations(&state.db, &product_id).await {
        Ok(trans) => Ok(Json(trans)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

pub async fn upsert_product_translation(
    State(state): State<AppState>,
    Path((product_id, locale)): Path<(String, String)>,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<ErpProductTranslation>, (StatusCode, String)> {
    let name = body.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let description = body.get("description").and_then(|v| v.as_str());
    let care_instructions = body.get("careInstructions").and_then(|v| v.as_str());
    if name.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "Name is required".to_string()));
    }
    match ErpCatalogService::upsert_product_translation(&state.db, &product_id, &locale, &name, description, care_instructions).await {
        Ok(trans) => Ok(Json(trans)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

// ─── Variants ───

pub async fn list_variants(
    State(state): State<AppState>,
    Path(product_id): Path<String>,
) -> Result<Json<Vec<ErpProductVariant>>, (StatusCode, String)> {
    match ErpCatalogService::list_variants(&state.db, &product_id).await {
        Ok(vars) => Ok(Json(vars)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

pub async fn get_variant(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ErpProductVariant>, (StatusCode, String)> {
    match ErpCatalogService::get_variant(&state.db, &id).await {
        Ok(var) => Ok(Json(var)),
        Err(e) => Err((StatusCode::NOT_FOUND, e)),
    }
}

pub async fn create_variant(
    State(state): State<AppState>,
    Json(req): Json<CreateVariantRequest>,
) -> Result<(StatusCode, Json<ErpProductVariant>), (StatusCode, String)> {
    match ErpCatalogService::create_variant(&state.db, &req).await {
        Ok(var) => Ok((StatusCode::CREATED, Json(var))),
        Err(e) => Err((StatusCode::BAD_REQUEST, e)),
    }
}

pub async fn update_variant(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<UpdateVariantRequest>,
) -> Result<Json<ErpProductVariant>, (StatusCode, String)> {
    match ErpCatalogService::update_variant(&state.db, &id, &req).await {
        Ok(var) => Ok(Json(var)),
        Err(e) => Err((StatusCode::NOT_FOUND, e)),
    }
}

pub async fn delete_variant(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    match ErpCatalogService::delete_variant(&state.db, &id).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

// ─── Images ───

pub async fn list_images(
    State(state): State<AppState>,
    Path(product_id): Path<String>,
) -> Result<Json<Vec<ErpProductImage>>, (StatusCode, String)> {
    match ErpCatalogService::list_images(&state.db, &product_id).await {
        Ok(imgs) => Ok(Json(imgs)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

pub async fn create_image(
    State(state): State<AppState>,
    Json(req): Json<CreateProductImageRequest>,
) -> Result<(StatusCode, Json<ErpProductImage>), (StatusCode, String)> {
    match ErpCatalogService::create_image(&state.db, &req).await {
        Ok(img) => Ok((StatusCode::CREATED, Json(img))),
        Err(e) => Err((StatusCode::BAD_REQUEST, e)),
    }
}

pub async fn delete_image(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    match ErpCatalogService::delete_image(&state.db, &id).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

// ─── Options ───

pub async fn list_product_options(
    State(state): State<AppState>,
    Path(product_id): Path<String>,
) -> Result<Json<Vec<ErpProductOption>>, (StatusCode, String)> {
    match ErpCatalogService::list_product_options(&state.db, &product_id).await {
        Ok(opts) => Ok(Json(opts)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

pub async fn create_product_option(
    State(state): State<AppState>,
    Json(req): Json<CreateProductOptionRequest>,
) -> Result<(StatusCode, Json<ErpProductOption>), (StatusCode, String)> {
    match ErpCatalogService::create_product_option(&state.db, &req).await {
        Ok(opt) => Ok((StatusCode::CREATED, Json(opt))),
        Err(e) => Err((StatusCode::BAD_REQUEST, e)),
    }
}

pub async fn delete_product_option(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    match ErpCatalogService::delete_product_option(&state.db, &id).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

pub async fn list_option_values(
    State(state): State<AppState>,
    Path(option_id): Path<String>,
) -> Result<Json<Vec<ErpProductOptionValue>>, (StatusCode, String)> {
    match ErpCatalogService::list_option_values(&state.db, &option_id).await {
        Ok(vals) => Ok(Json(vals)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

pub async fn create_option_value(
    State(state): State<AppState>,
    Json(req): Json<CreateOptionValueRequest>,
) -> Result<(StatusCode, Json<ErpProductOptionValue>), (StatusCode, String)> {
    match ErpCatalogService::create_option_value(&state.db, &req).await {
        Ok(val) => Ok((StatusCode::CREATED, Json(val))),
        Err(e) => Err((StatusCode::BAD_REQUEST, e)),
    }
}

pub async fn delete_option_value(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    match ErpCatalogService::delete_option_value(&state.db, &id).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

// ─── Collections ───

pub async fn list_collections(
    State(state): State<AppState>,
) -> Result<Json<Vec<ErpCollection>>, (StatusCode, String)> {
    match ErpCatalogService::list_collections(&state.db, true).await {
        Ok(colls) => Ok(Json(colls)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

pub async fn get_collection(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ErpCollection>, (StatusCode, String)> {
    match ErpCatalogService::get_collection(&state.db, &id).await {
        Ok(coll) => Ok(Json(coll)),
        Err(e) => Err((StatusCode::NOT_FOUND, e)),
    }
}

pub async fn create_collection(
    State(state): State<AppState>,
    Json(req): Json<CreateCollectionRequest>,
) -> Result<(StatusCode, Json<ErpCollection>), (StatusCode, String)> {
    match ErpCatalogService::create_collection(&state.db, &req).await {
        Ok(coll) => Ok((StatusCode::CREATED, Json(coll))),
        Err(e) => Err((StatusCode::BAD_REQUEST, e)),
    }
}

pub async fn add_product_to_collection(
    State(state): State<AppState>,
    Path(collection_id): Path<String>,
    Json(body): Json<serde_json::Value>,
) -> Result<StatusCode, (StatusCode, String)> {
    let product_id = body.get("productId").and_then(|v| v.as_str())
        .ok_or_else(|| (StatusCode::BAD_REQUEST, "productId required".to_string()))?;
    let sort_order = body.get("sortOrder").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
    match ErpCatalogService::add_product_to_collection(&state.db, &collection_id, product_id, sort_order).await {
        Ok(()) => Ok(StatusCode::OK),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

pub async fn remove_product_from_collection(
    State(state): State<AppState>,
    Path((collection_id, product_id)): Path<(String, String)>,
) -> Result<StatusCode, (StatusCode, String)> {
    match ErpCatalogService::remove_product_from_collection(&state.db, &collection_id, &product_id).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

pub async fn list_collection_products(
    State(state): State<AppState>,
    Path(collection_id): Path<String>,
) -> Result<Json<Vec<ErpProduct>>, (StatusCode, String)> {
    match ErpCatalogService::list_collection_products(&state.db, &collection_id).await {
        Ok(prods) => Ok(Json(prods)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

// ─── Stock Adjustments ───

pub async fn adjust_stock(
    State(state): State<AppState>,
    Json(req): Json<StockAdjustmentRequest>,
) -> Result<(StatusCode, Json<ErpStockAdjustment>), (StatusCode, String)> {
    match ErpCatalogService::adjust_stock(&state.db, &req).await {
        Ok(adj) => Ok((StatusCode::OK, Json(adj))),
        Err(e) => Err((StatusCode::BAD_REQUEST, e)),
    }
}

pub async fn list_stock_adjustments(
    State(state): State<AppState>,
    Path(variant_id): Path<String>,
    Query(params): Query<serde_json::Value>,
) -> Result<Json<Vec<ErpStockAdjustment>>, (StatusCode, String)> {
    let limit = params.get("limit").and_then(|v| v.as_i64()).unwrap_or(50);
    match ErpCatalogService::list_stock_adjustments(&state.db, &variant_id, limit).await {
        Ok(adjs) => Ok(Json(adjs)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

// ─── Cross-sells ───

pub async fn add_cross_sell(
    State(state): State<AppState>,
    Path(source_id): Path<String>,
    Json(body): Json<serde_json::Value>,
) -> Result<StatusCode, (StatusCode, String)> {
    let target_id = body.get("targetProductId").and_then(|v| v.as_str())
        .ok_or_else(|| (StatusCode::BAD_REQUEST, "targetProductId required".to_string()))?;
    let label = body.get("label").and_then(|v| v.as_str());
    match ErpCatalogService::add_cross_sell(&state.db, &source_id, target_id, label).await {
        Ok(()) => Ok(StatusCode::OK),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

pub async fn list_cross_sells(
    State(state): State<AppState>,
    Path(product_id): Path<String>,
) -> Result<Json<Vec<ErpProductCrossSell>>, (StatusCode, String)> {
    match ErpCatalogService::list_cross_sells(&state.db, &product_id).await {
        Ok(sells) => Ok(Json(sells)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

// ─── AI Suggestions ───

pub async fn list_ai_suggestions(
    State(state): State<AppState>,
    Path(product_id): Path<String>,
) -> Result<Json<Vec<ErpProductAiSuggestion>>, (StatusCode, String)> {
    match ErpCatalogService::list_ai_suggestions(&state.db, &product_id).await {
        Ok(suggs) => Ok(Json(suggs)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

pub async fn approve_ai_suggestion(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    let reviewed_by = "system".to_string();
    match ErpCatalogService::approve_ai_suggestion(&state.db, &id, &reviewed_by).await {
        Ok(()) => Ok(StatusCode::OK),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

pub async fn reject_ai_suggestion(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    let reviewed_by = "system".to_string();
    match ErpCatalogService::reject_ai_suggestion(&state.db, &id, &reviewed_by).await {
        Ok(()) => Ok(StatusCode::OK),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

// ─── Redirects ───

pub async fn list_redirects(
    State(state): State<AppState>,
) -> Result<Json<Vec<ErpRedirect>>, (StatusCode, String)> {
    match ErpCatalogService::list_redirects(&state.db, 100).await {
        Ok(reds) => Ok(Json(reds)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

pub async fn create_redirect(
    State(state): State<AppState>,
    Json(body): Json<serde_json::Value>,
) -> Result<(StatusCode, Json<ErpRedirect>), (StatusCode, String)> {
    let from_path = body.get("fromPath").and_then(|v| v.as_str())
        .ok_or_else(|| (StatusCode::BAD_REQUEST, "fromPath required".to_string()))?;
    let to_path = body.get("toPath").and_then(|v| v.as_str())
        .ok_or_else(|| (StatusCode::BAD_REQUEST, "toPath required".to_string()))?;
    let status_code = body.get("statusCode").and_then(|v| v.as_i64()).unwrap_or(301) as i32;
    match ErpCatalogService::create_redirect(&state.db, from_path, to_path, status_code).await {
        Ok(red) => Ok((StatusCode::CREATED, Json(red))),
        Err(e) => Err((StatusCode::BAD_REQUEST, e)),
    }
}
