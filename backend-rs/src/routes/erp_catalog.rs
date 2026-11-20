//! ERP Catalog routes — thin gRPC client forwarding to svc-erp-catalog.

use axum::extract::{Path, Query};
use axum::http::StatusCode;
use axum::Json;
use newax_proto::erp_catalog::v1::erp_catalog_service_client::ErpCatalogServiceClient;
use newax_proto::erp_catalog::v1::{Empty, IdRequest, JsonResponse, SlugRequest, ListCategoriesRequest, ListProductsRequest,
    CreateCategoryRequest, UpdateCategoryRequest, UpsertCategoryTranslationRequest,
    CreateProductRequest, UpdateProductRequest, UpsertProductTranslationRequest,
    CreateVariantRequest, UpdateVariantRequest, CreateImageRequest,
    CreateProductOptionRequest, CreateOptionValueRequest, CreateCollectionRequest,
    AddProductToCollectionRequest, RemoveProductFromCollectionRequest,
    AdjustStockRequest, ListStockAdjustmentsRequest, AddCrossSellRequest, CreateRedirectRequest};

async fn client() -> Result<ErpCatalogServiceClient<tonic::transport::Channel>, (StatusCode, String)> {
    let url = std::env::var("ERP_CATALOG_SERVICE_URL")
        .unwrap_or_else(|_| "http://127.0.0.1:50100".to_string());
    ErpCatalogServiceClient::connect(url)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("erp-catalog service unavailable: {e}")))
}

// ─── Categories ───

pub async fn list_categories(Query(params): Query<serde_json::Value>) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut client = client().await?;
    let active_only = params.get("activeOnly").and_then(|v| v.as_bool()).unwrap_or(true);
    let resp = client.list_categories(ListCategoriesRequest { active_only }).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let json: serde_json::Value = serde_json::from_str(&resp.into_inner().json).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(json))
}

pub async fn get_category(Path(id): Path<String>) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut client = client().await?;
    let resp = client.get_category(IdRequest { id }).await.map_err(|e| (StatusCode::NOT_FOUND, e.to_string()))?;
    let json: serde_json::Value = serde_json::from_str(&resp.into_inner().json).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(json))
}

pub async fn create_category(Json(req): Json<serde_json::Value>) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut client = client().await?;
    let json = serde_json::to_string(&req).map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    let resp = client.create_category(CreateCategoryRequest { json }).await.map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    let json: serde_json::Value = serde_json::from_str(&resp.into_inner().json).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(json))
}

pub async fn update_category(Path(id): Path<String>, Json(req): Json<serde_json::Value>) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut client = client().await?;
    let json = serde_json::to_string(&req).map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    let resp = client.update_category(UpdateCategoryRequest { id, json }).await.map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    let json: serde_json::Value = serde_json::from_str(&resp.into_inner().json).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(json))
}

pub async fn delete_category(Path(id): Path<String>) -> Result<StatusCode, (StatusCode, String)> {
    let mut client = client().await?;
    client.delete_category(IdRequest { id }).await.map(|_| StatusCode::NO_CONTENT).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

// ─── Products ───

pub async fn list_products(Query(params): Query<serde_json::Value>) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut client = client().await?;
    let status = params.get("status").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let kind = params.get("kind").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let limit = params.get("limit").and_then(|v| v.as_i64()).unwrap_or(50);
    let offset = params.get("offset").and_then(|v| v.as_i64()).unwrap_or(0);
    let resp = client.list_products(ListProductsRequest { status, kind, limit, offset }).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let json: serde_json::Value = serde_json::from_str(&resp.into_inner().json).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(json))
}

pub async fn get_product(Path(id): Path<String>) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut client = client().await?;
    let resp = client.get_product(IdRequest { id }).await.map_err(|e| (StatusCode::NOT_FOUND, e.to_string()))?;
    let json: serde_json::Value = serde_json::from_str(&resp.into_inner().json).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(json))
}

pub async fn get_product_by_slug(Path(slug): Path<String>) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut client = client().await?;
    let resp = client.get_product_by_slug(SlugRequest { slug }).await.map_err(|e| (StatusCode::NOT_FOUND, e.to_string()))?;
    let json: serde_json::Value = serde_json::from_str(&resp.into_inner().json).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(json))
}

pub async fn create_product(Json(req): Json<serde_json::Value>) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut client = client().await?;
    let json = serde_json::to_string(&req).map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    let resp = client.create_product(CreateProductRequest { json }).await.map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    let json: serde_json::Value = serde_json::from_str(&resp.into_inner().json).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(json))
}

pub async fn update_product(Path(id): Path<String>, Json(req): Json<serde_json::Value>) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut client = client().await?;
    let json = serde_json::to_string(&req).map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    let resp = client.update_product(UpdateProductRequest { id, json }).await.map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    let json: serde_json::Value = serde_json::from_str(&resp.into_inner().json).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(json))
}

pub async fn delete_product(Path(id): Path<String>) -> Result<StatusCode, (StatusCode, String)> {
    let mut client = client().await?;
    client.delete_product(IdRequest { id }).await.map(|_| StatusCode::NO_CONTENT).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

// ─── Product Translations ───

pub async fn list_product_translations(Path(product_id): Path<String>) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut client = client().await?;
    let resp = client.list_product_translations(IdRequest { id: product_id }).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let json: serde_json::Value = serde_json::from_str(&resp.into_inner().json).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(json))
}

pub async fn upsert_product_translation(Path((product_id, locale)): Path<(String, String)>, Json(body): Json<serde_json::Value>) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut client = client().await?;
    let json = serde_json::to_string(&body).map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    let resp = client.upsert_product_translation(UpsertProductTranslationRequest { product_id, locale, json }).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let json: serde_json::Value = serde_json::from_str(&resp.into_inner().json).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(json))
}

// ─── Variants ───

pub async fn list_variants(Path(product_id): Path<String>) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut client = client().await?;
    let resp = client.list_variants(IdRequest { id: product_id }).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let json: serde_json::Value = serde_json::from_str(&resp.into_inner().json).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(json))
}

pub async fn get_variant(Path(id): Path<String>) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut client = client().await?;
    let resp = client.get_variant(IdRequest { id }).await.map_err(|e| (StatusCode::NOT_FOUND, e.to_string()))?;
    let json: serde_json::Value = serde_json::from_str(&resp.into_inner().json).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(json))
}

pub async fn create_variant(Json(req): Json<serde_json::Value>) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut client = client().await?;
    let json = serde_json::to_string(&req).map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    let resp = client.create_variant(CreateVariantRequest { json }).await.map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    let json: serde_json::Value = serde_json::from_str(&resp.into_inner().json).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(json))
}

pub async fn update_variant(Path(id): Path<String>, Json(req): Json<serde_json::Value>) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut client = client().await?;
    let json = serde_json::to_string(&req).map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    let resp = client.update_variant(UpdateVariantRequest { id, json }).await.map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    let json: serde_json::Value = serde_json::from_str(&resp.into_inner().json).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(json))
}

pub async fn delete_variant(Path(id): Path<String>) -> Result<StatusCode, (StatusCode, String)> {
    let mut client = client().await?;
    client.delete_variant(IdRequest { id }).await.map(|_| StatusCode::NO_CONTENT).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

// ─── Images ───

pub async fn list_images(Path(product_id): Path<String>) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut client = client().await?;
    let resp = client.list_images(IdRequest { id: product_id }).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let json: serde_json::Value = serde_json::from_str(&resp.into_inner().json).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(json))
}

pub async fn create_image(Json(req): Json<serde_json::Value>) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut client = client().await?;
    let json = serde_json::to_string(&req).map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    let resp = client.create_image(CreateImageRequest { json }).await.map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    let json: serde_json::Value = serde_json::from_str(&resp.into_inner().json).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(json))
}

pub async fn delete_image(Path(id): Path<String>) -> Result<StatusCode, (StatusCode, String)> {
    let mut client = client().await?;
    client.delete_image(IdRequest { id }).await.map(|_| StatusCode::NO_CONTENT).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

// ─── Options ───

pub async fn list_product_options(Path(product_id): Path<String>) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut client = client().await?;
    let resp = client.list_product_options(IdRequest { id: product_id }).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let json: serde_json::Value = serde_json::from_str(&resp.into_inner().json).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(json))
}

pub async fn create_product_option(Json(req): Json<serde_json::Value>) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut client = client().await?;
    let json = serde_json::to_string(&req).map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    let resp = client.create_product_option(CreateProductOptionRequest { json }).await.map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    let json: serde_json::Value = serde_json::from_str(&resp.into_inner().json).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(json))
}

pub async fn delete_product_option(Path(id): Path<String>) -> Result<StatusCode, (StatusCode, String)> {
    let mut client = client().await?;
    client.delete_product_option(IdRequest { id }).await.map(|_| StatusCode::NO_CONTENT).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

pub async fn list_option_values(Path(option_id): Path<String>) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut client = client().await?;
    let resp = client.list_option_values(IdRequest { id: option_id }).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let json: serde_json::Value = serde_json::from_str(&resp.into_inner().json).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(json))
}

pub async fn create_option_value(Json(req): Json<serde_json::Value>) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut client = client().await?;
    let json = serde_json::to_string(&req).map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    let resp = client.create_option_value(CreateOptionValueRequest { json }).await.map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    let json: serde_json::Value = serde_json::from_str(&resp.into_inner().json).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(json))
}

pub async fn delete_option_value(Path(id): Path<String>) -> Result<StatusCode, (StatusCode, String)> {
    let mut client = client().await?;
    client.delete_option_value(IdRequest { id }).await.map(|_| StatusCode::NO_CONTENT).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

// ─── Collections ───

pub async fn list_collections() -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut client = client().await?;
    let resp = client.list_collections(Empty {}).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let json: serde_json::Value = serde_json::from_str(&resp.into_inner().json).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(json))
}

pub async fn get_collection(Path(id): Path<String>) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut client = client().await?;
    let resp = client.get_collection(IdRequest { id }).await.map_err(|e| (StatusCode::NOT_FOUND, e.to_string()))?;
    let json: serde_json::Value = serde_json::from_str(&resp.into_inner().json).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(json))
}

pub async fn create_collection(Json(req): Json<serde_json::Value>) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut client = client().await?;
    let json = serde_json::to_string(&req).map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    let resp = client.create_collection(CreateCollectionRequest { json }).await.map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    let json: serde_json::Value = serde_json::from_str(&resp.into_inner().json).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(json))
}

pub async fn add_product_to_collection(Path(collection_id): Path<String>, Json(body): Json<serde_json::Value>) -> Result<StatusCode, (StatusCode, String)> {
    let mut client = client().await?;
    let json = serde_json::to_string(&body).map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    client.add_product_to_collection(AddProductToCollectionRequest { collection_id, json }).await.map(|_| StatusCode::OK).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

pub async fn remove_product_from_collection(Path((collection_id, product_id)): Path<(String, String)>) -> Result<StatusCode, (StatusCode, String)> {
    let mut client = client().await?;
    client.remove_product_from_collection(RemoveProductFromCollectionRequest { collection_id, product_id }).await.map(|_| StatusCode::NO_CONTENT).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

pub async fn list_collection_products(Path(collection_id): Path<String>) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut client = client().await?;
    let resp = client.list_collection_products(IdRequest { id: collection_id }).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let json: serde_json::Value = serde_json::from_str(&resp.into_inner().json).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(json))
}

// ─── Stock Adjustments ───

pub async fn adjust_stock(Json(req): Json<serde_json::Value>) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut client = client().await?;
    let json = serde_json::to_string(&req).map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    let resp = client.adjust_stock(AdjustStockRequest { json }).await.map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    let json: serde_json::Value = serde_json::from_str(&resp.into_inner().json).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(json))
}

pub async fn list_stock_adjustments(Path(variant_id): Path<String>, Query(params): Query<serde_json::Value>) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut client = client().await?;
    let limit = params.get("limit").and_then(|v| v.as_i64()).unwrap_or(50);
    let resp = client.list_stock_adjustments(ListStockAdjustmentsRequest { variant_id, limit }).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let json: serde_json::Value = serde_json::from_str(&resp.into_inner().json).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(json))
}

// ─── Cross-sells ───

pub async fn add_cross_sell(Path(source_id): Path<String>, Json(body): Json<serde_json::Value>) -> Result<StatusCode, (StatusCode, String)> {
    let mut client = client().await?;
    let json = serde_json::to_string(&body).map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    client.add_cross_sell(AddCrossSellRequest { source_id, json }).await.map(|_| StatusCode::OK).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

pub async fn list_cross_sells(Path(product_id): Path<String>) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut client = client().await?;
    let resp = client.list_cross_sells(IdRequest { id: product_id }).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let json: serde_json::Value = serde_json::from_str(&resp.into_inner().json).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(json))
}

// ─── AI Suggestions ───

pub async fn list_ai_suggestions(Path(product_id): Path<String>) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut client = client().await?;
    let resp = client.list_ai_suggestions(IdRequest { id: product_id }).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let json: serde_json::Value = serde_json::from_str(&resp.into_inner().json).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(json))
}

pub async fn approve_ai_suggestion(Path(id): Path<String>) -> Result<StatusCode, (StatusCode, String)> {
    let mut client = client().await?;
    client.approve_ai_suggestion(IdRequest { id }).await.map(|_| StatusCode::OK).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

pub async fn reject_ai_suggestion(Path(id): Path<String>) -> Result<StatusCode, (StatusCode, String)> {
    let mut client = client().await?;
    client.reject_ai_suggestion(IdRequest { id }).await.map(|_| StatusCode::OK).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

// ─── Redirects ───

pub async fn list_redirects() -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut client = client().await?;
    let resp = client.list_redirects(Empty {}).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let json: serde_json::Value = serde_json::from_str(&resp.into_inner().json).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(json))
}

pub async fn create_redirect(Json(body): Json<serde_json::Value>) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut client = client().await?;
    let json = serde_json::to_string(&body).map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    let resp = client.create_redirect(CreateRedirectRequest { json }).await.map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    let json: serde_json::Value = serde_json::from_str(&resp.into_inner().json).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(json))
}
