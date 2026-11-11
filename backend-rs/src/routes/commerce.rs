#![allow(dead_code)]

//! Commerce engine routes: catalog, cart, orders, payments, and sync.

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;

use crate::db::AppState;
use crate::services::commerce_catalog::{
    CommerceCatalogService, CreateCategoryRequest, CreateProductRequest, SetChannelVisibilityRequest,
    UpdateCategoryRequest, UpdateInventoryRequest, UpdateProductRequest,
};
use crate::services::commerce_cart::{
    AddCartItemRequest, CommerceCartService, CreateCartRequest, UpdateCartItemRequest,
};
use crate::services::commerce_orders::{
    CheckoutRequest, CommerceOrdersService, UpdateOrderStatusRequest,
};
use crate::services::commerce_payments::{
    CommercePaymentsService, RecordPaymentRequest, UpdatePaymentStatusRequest,
};
use crate::services::sync_adapter::{
    CreateSyncConnectionRequest, SyncService, UpdateSyncConnectionRequest,
};
use crate::services::commerce_options::{
    ProductOptionsService, CreateProductOptionRequest, CreateProductOptionValueRequest,
    UpdateProductOptionRequest,
};
use crate::services::commerce_gaps::{
    CommerceGapsService, CreateDeliveryFeeRuleRequest, CreatePaymentGatewayRequest,
    CreateServiceTypeRequest, CreateDeliveryScheduleRequest, CreateDeliveryTimeSlotRequest,
    CreateNotificationTemplateRequest, UploadPaymentProofRequest, VerifyPaymentProofRequest,
    CreateReviewRequest, CreateLoyaltyProgramRequest, CreateStaffMemberRequest,
    CreateWholesalePricingRequest, CreateFollowupRequest, CreateCustomDomainRequest,
    CreateStorefrontPageRequest, CreateMetaPixelRequest, CreateCheckoutQuestionRequest,
    SubmitCheckoutAnswersRequest, UploadDeliveryProofRequest, ImportCsvRequest,
    CreateCustomerAccountRequest, LoginRequest,
};

// ---------------------------------------------------------------------------
// Query parameters
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct ListCategoriesQuery {
    pub brand_id: String,
    pub active_only: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct ListProductsQuery {
    pub brand_id: String,
    pub category_id: Option<String>,
    pub active_only: Option<bool>,
    pub channel_type: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ListCartsQuery {
    pub customer_id: String,
    pub brand_id: String,
    pub status: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct GetOrCreateCartQuery {
    pub customer_id: String,
    pub brand_id: String,
    pub channel_type: Option<String>,
}

// ---------------------------------------------------------------------------
// Categories
// ---------------------------------------------------------------------------

pub async fn list_categories(
    State(state): State<AppState>,
    Query(query): Query<ListCategoriesQuery>,
) -> Result<Json<Vec<crate::models::Category>>, (StatusCode, String)> {
    let service = CommerceCatalogService::new(state.db.clone());
    match service
        .list_categories(&query.brand_id, query.active_only.unwrap_or(true))
        .await
    {
        Ok(cats) => Ok(Json(cats)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

pub async fn get_category(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<crate::models::Category>, (StatusCode, String)> {
    let service = CommerceCatalogService::new(state.db.clone());
    match service.get_category(&id).await {
        Ok(cat) => Ok(Json(cat)),
        Err(e) => Err((StatusCode::NOT_FOUND, e)),
    }
}

pub async fn create_category(
    State(state): State<AppState>,
    Json(req): Json<CreateCategoryRequest>,
) -> Result<Json<crate::models::Category>, (StatusCode, String)> {
    let service = CommerceCatalogService::new(state.db.clone());
    match service.create_category(&req).await {
        Ok(cat) => Ok(Json(cat)),
        Err(e) => Err((StatusCode::BAD_REQUEST, e)),
    }
}

pub async fn update_category(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<UpdateCategoryRequest>,
) -> Result<Json<crate::models::Category>, (StatusCode, String)> {
    let service = CommerceCatalogService::new(state.db.clone());
    match service.update_category(&id, &req).await {
        Ok(cat) => Ok(Json(cat)),
        Err(e) => Err((StatusCode::BAD_REQUEST, e)),
    }
}

pub async fn delete_category(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    let service = CommerceCatalogService::new(state.db.clone());
    match service.delete_category(&id).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

// ---------------------------------------------------------------------------
// Products
// ---------------------------------------------------------------------------

pub async fn list_products(
    State(state): State<AppState>,
    Query(query): Query<ListProductsQuery>,
) -> Result<Json<Vec<crate::models::Product>>, (StatusCode, String)> {
    let service = CommerceCatalogService::new(state.db.clone());
    let cat_id = query.category_id.as_deref();
    let ch = query.channel_type.as_deref();
    match service
        .list_products(&query.brand_id, cat_id, query.active_only.unwrap_or(true), ch)
        .await
    {
        Ok(prods) => Ok(Json(prods)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

pub async fn get_product(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<crate::models::Product>, (StatusCode, String)> {
    let service = CommerceCatalogService::new(state.db.clone());
    match service.get_product(&id).await {
        Ok(prod) => Ok(Json(prod)),
        Err(e) => Err((StatusCode::NOT_FOUND, e)),
    }
}

pub async fn create_product(
    State(state): State<AppState>,
    Json(req): Json<CreateProductRequest>,
) -> Result<Json<crate::models::Product>, (StatusCode, String)> {
    let service = CommerceCatalogService::new(state.db.clone());
    match service.create_product(&req).await {
        Ok(prod) => {
            // Auto-sync: queue product for outbound sync to all platforms
            let sync = crate::services::sync_adapter::SyncService::new(
                state.db.clone(),
                state.http_client.clone(),
            );
            let brand_id = prod.brand_id.clone();
            let pid = prod.id.clone();
            let _ = tokio::spawn(async move {
                let _ = sync.queue_product_sync(&pid, &brand_id).await;
            });
            Ok(Json(prod))
        }
        Err(e) => Err((StatusCode::BAD_REQUEST, e)),
    }
}

pub async fn update_product(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<UpdateProductRequest>,
) -> Result<Json<crate::models::Product>, (StatusCode, String)> {
    let service = CommerceCatalogService::new(state.db.clone());
    match service.update_product(&id, &req).await {
        Ok(prod) => {
            // Auto-sync: queue product for outbound sync
            let sync = crate::services::sync_adapter::SyncService::new(
                state.db.clone(),
                state.http_client.clone(),
            );
            let brand_id = prod.brand_id.clone();
            let pid = prod.id.clone();
            let _ = tokio::spawn(async move {
                let _ = sync.queue_product_sync(&pid, &brand_id).await;
            });
            Ok(Json(prod))
        }
        Err(e) => Err((StatusCode::BAD_REQUEST, e)),
    }
}

pub async fn delete_product(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    let service = CommerceCatalogService::new(state.db.clone());
    match service.delete_product(&id).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

// ---------------------------------------------------------------------------
// Inventory
// ---------------------------------------------------------------------------

pub async fn update_inventory(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<UpdateInventoryRequest>,
) -> Result<Json<crate::models::Product>, (StatusCode, String)> {
    let service = CommerceCatalogService::new(state.db.clone());
    match service.update_inventory(&id, &req).await {
        Ok(prod) => {
            // Auto-sync: queue inventory update for outbound sync
            let sync = crate::services::sync_adapter::SyncService::new(
                state.db.clone(),
                state.http_client.clone(),
            );
            let brand_id = prod.brand_id.clone();
            let pid = prod.id.clone();
            let _ = tokio::spawn(async move {
                let _ = sync.queue_inventory_sync(&pid, &brand_id).await;
            });
            Ok(Json(prod))
        }
        Err(e) => Err((StatusCode::BAD_REQUEST, e)),
    }
}

// ---------------------------------------------------------------------------
// Channel Visibility
// ---------------------------------------------------------------------------

pub async fn get_channel_visibility(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<
    Json<Vec<crate::models::ProductChannelVisibility>>,
    (StatusCode, String),
> {
    let service = CommerceCatalogService::new(state.db.clone());
    match service.get_channel_visibility(&id).await {
        Ok(vis) => Ok(Json(vis)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

pub async fn set_channel_visibility(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<SetChannelVisibilityRequest>,
) -> Result<Json<crate::models::ProductChannelVisibility>, (StatusCode, String)> {
    let service = CommerceCatalogService::new(state.db.clone());
    match service.set_channel_visibility(&id, &req).await {
        Ok(vis) => Ok(Json(vis)),
        Err(e) => Err((StatusCode::BAD_REQUEST, e)),
    }
}

// ---------------------------------------------------------------------------
// Carts
// ---------------------------------------------------------------------------

pub async fn get_or_create_cart(
    State(state): State<AppState>,
    Query(query): Query<GetOrCreateCartQuery>,
) -> Result<Json<crate::services::commerce_cart::CartWithItems>, (StatusCode, String)> {
    let service = CommerceCartService::new(state.db.clone());
    let ch = query.channel_type.as_deref();
    match service
        .get_or_create_cart(&query.customer_id, &query.brand_id, ch)
        .await
    {
        Ok(cart) => Ok(Json(cart)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

pub async fn list_carts(
    State(state): State<AppState>,
    Query(query): Query<ListCartsQuery>,
) -> Result<Json<Vec<crate::models::Cart>>, (StatusCode, String)> {
    let service = CommerceCartService::new(state.db.clone());
    let status = query.status.as_deref();
    match service
        .list_carts(&query.customer_id, &query.brand_id, status)
        .await
    {
        Ok(carts) => Ok(Json(carts)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

pub async fn get_cart(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<crate::services::commerce_cart::CartWithItems>, (StatusCode, String)> {
    let service = CommerceCartService::new(state.db.clone());
    match service.get_cart_with_items(&id).await {
        Ok(cart) => Ok(Json(cart)),
        Err(e) => Err((StatusCode::NOT_FOUND, e)),
    }
}

pub async fn create_cart(
    State(state): State<AppState>,
    Json(req): Json<CreateCartRequest>,
) -> Result<Json<crate::models::Cart>, (StatusCode, String)> {
    let service = CommerceCartService::new(state.db.clone());
    let ch = req.channel_type.as_deref();
    match service
        .get_or_create_cart(&req.customer_id, &req.brand_id, ch)
        .await
    {
        Ok(cart_with_items) => Ok(Json(cart_with_items.cart)),
        Err(e) => Err((StatusCode::BAD_REQUEST, e)),
    }
}

pub async fn delete_cart(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    let service = CommerceCartService::new(state.db.clone());
    match service.delete_cart(&id).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

// ---------------------------------------------------------------------------
// Cart Items
// ---------------------------------------------------------------------------

pub async fn add_cart_item(
    State(state): State<AppState>,
    Path(cart_id): Path<String>,
    Json(req): Json<AddCartItemRequest>,
) -> Result<Json<crate::services::commerce_cart::CartItemWithProduct>, (StatusCode, String)> {
    let service = CommerceCartService::new(state.db.clone());
    match service.add_item(&cart_id, &req).await {
        Ok(item) => Ok(Json(item)),
        Err(e) => Err((StatusCode::BAD_REQUEST, e)),
    }
}

pub async fn update_cart_item(
    State(state): State<AppState>,
    Path((cart_id, item_id)): Path<(String, String)>,
    Json(req): Json<UpdateCartItemRequest>,
) -> Result<Json<crate::services::commerce_cart::CartItemWithProduct>, (StatusCode, String)> {
    let service = CommerceCartService::new(state.db.clone());
    match service
        .update_item_quantity(&cart_id, &item_id, &req)
        .await
    {
        Ok(item) => Ok(Json(item)),
        Err(e) => Err((StatusCode::BAD_REQUEST, e)),
    }
}

pub async fn remove_cart_item(
    State(state): State<AppState>,
    Path((cart_id, item_id)): Path<(String, String)>,
) -> Result<StatusCode, (StatusCode, String)> {
    let service = CommerceCartService::new(state.db.clone());
    match service.remove_item(&cart_id, &item_id).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

pub async fn clear_cart(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    let service = CommerceCartService::new(state.db.clone());
    match service.clear_cart(&id).await {
        Ok(()) => Ok(StatusCode::OK),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

// ---------------------------------------------------------------------------
// Checkout
// ---------------------------------------------------------------------------

pub async fn checkout_cart(
    State(state): State<AppState>,
    Path(cart_id): Path<String>,
    Json(req): Json<CheckoutRequest>,
) -> Result<Json<crate::services::commerce_orders::OrderWithItems>, (StatusCode, String)> {
    let service = CommerceOrdersService::new(state.db.clone());
    match service.checkout(&cart_id, &req).await {
        Ok(order) => {
            // Auto-sync: queue order for outbound sync to all platforms
            let sync = crate::services::sync_adapter::SyncService::new(
                state.db.clone(),
                state.http_client.clone(),
            );
            let brand_id = order.order.brand_id.clone();
            let oid = order.order.id.clone();
            let _ = tokio::spawn(async move {
                let _ = sync.queue_order_sync(&oid, &brand_id).await;
            });
            Ok(Json(order))
        }
        Err(e) => Err((StatusCode::BAD_REQUEST, e)),
    }
}

// ---------------------------------------------------------------------------
// Orders
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct ListOrdersQuery {
    pub brand_id: String,
    pub customer_id: Option<String>,
    pub status: Option<String>,
    pub channel_type: Option<String>,
}

pub async fn list_orders(
    State(state): State<AppState>,
    Query(query): Query<ListOrdersQuery>,
) -> Result<Json<Vec<crate::models::CommerceOrder>>, (StatusCode, String)> {
    let service = CommerceOrdersService::new(state.db.clone());
    let cid = query.customer_id.as_deref();
    let st = query.status.as_deref();
    let ch = query.channel_type.as_deref();
    match service
        .list_orders(&query.brand_id, cid, st, ch)
        .await
    {
        Ok(orders) => Ok(Json(orders)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

pub async fn get_order(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<crate::services::commerce_orders::OrderWithItems>, (StatusCode, String)> {
    let service = CommerceOrdersService::new(state.db.clone());
    match service.get_order_with_items(&id).await {
        Ok(order) => Ok(Json(order)),
        Err(e) => Err((StatusCode::NOT_FOUND, e)),
    }
}

pub async fn update_order_status(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<UpdateOrderStatusRequest>,
) -> Result<Json<crate::models::CommerceOrder>, (StatusCode, String)> {
    let service = CommerceOrdersService::new(state.db.clone());
    match service.update_order_status(&id, &req).await {
        Ok(order) => {
            // Auto-sync: queue order status update for outbound sync
            let sync = crate::services::sync_adapter::SyncService::new(
                state.db.clone(),
                state.http_client.clone(),
            );
            let brand_id = order.brand_id.clone();
            let oid = order.id.clone();
            let _ = tokio::spawn(async move {
                let _ = sync.queue_order_sync(&oid, &brand_id).await;
            });
            Ok(Json(order))
        }
        Err(e) => Err((StatusCode::BAD_REQUEST, e)),
    }
}

pub async fn get_order_stats(
    State(state): State<AppState>,
    Query(query): Query<ListCategoriesQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let service = CommerceOrdersService::new(state.db.clone());
    match service.get_order_stats(&query.brand_id).await {
        Ok(stats) => Ok(Json(stats)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

// ---------------------------------------------------------------------------
// Payments
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct ListPaymentsQuery {
    pub brand_id: String,
    pub status: Option<String>,
    pub method: Option<String>,
}

pub async fn list_payments(
    State(state): State<AppState>,
    Query(query): Query<ListPaymentsQuery>,
) -> Result<Json<Vec<crate::models::CommercePayment>>, (StatusCode, String)> {
    let service = CommercePaymentsService::new(state.db.clone());
    let st = query.status.as_deref();
    let m = query.method.as_deref();
    match service
        .list_payments(&query.brand_id, st, m)
        .await
    {
        Ok(payments) => Ok(Json(payments)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

pub async fn list_order_payments(
    State(state): State<AppState>,
    Path(order_id): Path<String>,
) -> Result<Json<Vec<crate::models::CommercePayment>>, (StatusCode, String)> {
    let service = CommercePaymentsService::new(state.db.clone());
    match service.list_payments_for_order(&order_id).await {
        Ok(payments) => Ok(Json(payments)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

pub async fn get_payment(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<crate::models::CommercePayment>, (StatusCode, String)> {
    let service = CommercePaymentsService::new(state.db.clone());
    match service.get_payment(&id).await {
        Ok(payment) => Ok(Json(payment)),
        Err(e) => Err((StatusCode::NOT_FOUND, e)),
    }
}

pub async fn record_payment(
    State(state): State<AppState>,
    Json(req): Json<RecordPaymentRequest>,
) -> Result<Json<crate::models::CommercePayment>, (StatusCode, String)> {
    let service = CommercePaymentsService::new(state.db.clone());
    match service.record_payment(&req).await {
        Ok(payment) => Ok(Json(payment)),
        Err(e) => Err((StatusCode::BAD_REQUEST, e)),
    }
}

pub async fn update_payment_status(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<UpdatePaymentStatusRequest>,
) -> Result<Json<crate::models::CommercePayment>, (StatusCode, String)> {
    let service = CommercePaymentsService::new(state.db.clone());
    match service.update_payment_status(&id, &req).await {
        Ok(payment) => Ok(Json(payment)),
        Err(e) => Err((StatusCode::BAD_REQUEST, e)),
    }
}

pub async fn refund_payment(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<crate::models::CommercePayment>, (StatusCode, String)> {
    let service = CommercePaymentsService::new(state.db.clone());
    match service.refund_payment(&id).await {
        Ok(payment) => Ok(Json(payment)),
        Err(e) => Err((StatusCode::BAD_REQUEST, e)),
    }
}

pub async fn get_payment_stats(
    State(state): State<AppState>,
    Query(query): Query<ListCategoriesQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let service = CommercePaymentsService::new(state.db.clone());
    match service.get_payment_stats(&query.brand_id).await {
        Ok(stats) => Ok(Json(stats)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

// ---------------------------------------------------------------------------
// Sync Connections
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct ListSyncConnectionsQuery {
    pub brand_id: String,
}

pub async fn list_sync_connections(
    State(state): State<AppState>,
    Query(query): Query<ListSyncConnectionsQuery>,
) -> Result<Json<Vec<crate::models::SyncConnection>>, (StatusCode, String)> {
    let service = SyncService::new(state.db.clone(), state.http_client.clone());
    match service.list_connections(&query.brand_id).await {
        Ok(conns) => Ok(Json(conns)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

pub async fn get_sync_connection(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<crate::models::SyncConnection>, (StatusCode, String)> {
    let service = SyncService::new(state.db.clone(), state.http_client.clone());
    match service.get_connection(&id).await {
        Ok(conn) => Ok(Json(conn)),
        Err(e) => Err((StatusCode::NOT_FOUND, e)),
    }
}

pub async fn create_sync_connection(
    State(state): State<AppState>,
    Json(req): Json<CreateSyncConnectionRequest>,
) -> Result<Json<crate::models::SyncConnection>, (StatusCode, String)> {
    let service = SyncService::new(state.db.clone(), state.http_client.clone());
    match service.create_connection(&req).await {
        Ok(conn) => Ok(Json(conn)),
        Err(e) => Err((StatusCode::BAD_REQUEST, e)),
    }
}

pub async fn update_sync_connection(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<UpdateSyncConnectionRequest>,
) -> Result<Json<crate::models::SyncConnection>, (StatusCode, String)> {
    let service = SyncService::new(state.db.clone(), state.http_client.clone());
    match service.update_connection(&id, &req).await {
        Ok(conn) => Ok(Json(conn)),
        Err(e) => Err((StatusCode::BAD_REQUEST, e)),
    }
}

pub async fn delete_sync_connection(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    let service = SyncService::new(state.db.clone(), state.http_client.clone());
    match service.delete_connection(&id).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

pub async fn trigger_sync(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<crate::services::sync_adapter::SyncResult>, (StatusCode, String)> {
    let service = SyncService::new(state.db.clone(), state.http_client.clone());
    match service.trigger_sync(&id).await {
        Ok(result) => Ok(Json(result)),
        Err(e) => Err((StatusCode::BAD_REQUEST, e)),
    }
}

#[derive(Debug, Deserialize)]
pub struct ListSyncLogsQuery {
    pub connection_id: Option<String>,
    pub limit: Option<i64>,
}

pub async fn list_sync_logs(
    State(state): State<AppState>,
    Query(query): Query<ListSyncLogsQuery>,
) -> Result<Json<Vec<crate::models::SyncLog>>, (StatusCode, String)> {
    let service = SyncService::new(state.db.clone(), state.http_client.clone());
    let cid = query.connection_id.as_deref();
    let limit = query.limit.unwrap_or(50);
    match service.list_sync_logs(cid, limit).await {
        Ok(logs) => Ok(Json(logs)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

// ---------------------------------------------------------------------------
// Sync: Multi-Platform, Platform Info, Product Mappings, Queue Processing
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct SyncAllQuery {
    pub brand_id: String,
}

pub async fn sync_all_platforms(
    State(state): State<AppState>,
    Query(query): Query<SyncAllQuery>,
) -> Result<Json<Vec<crate::services::sync_adapter::SyncResult>>, (StatusCode, String)> {
    let service = SyncService::new(state.db.clone(), state.http_client.clone());
    match service.sync_all_platforms(&query.brand_id).await {
        Ok(results) => Ok(Json(results)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

pub async fn get_platform_info(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let service = SyncService::new(state.db.clone(), state.http_client.clone());
    match service.get_platform_info(&id).await {
        Ok(info) => Ok(Json(info)),
        Err(e) => Err((StatusCode::BAD_REQUEST, e)),
    }
}

pub async fn list_product_mappings(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Vec<serde_json::Value>>, (StatusCode, String)> {
    let service = SyncService::new(state.db.clone(), state.http_client.clone());
    match service.list_product_mappings(&id).await {
        Ok(mappings) => Ok(Json(mappings)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

#[derive(Debug, Deserialize)]
pub struct ProcessSyncQueueQuery {
    pub connection_id: Option<String>,
}

pub async fn process_sync_queue(
    State(state): State<AppState>,
    Query(query): Query<ProcessSyncQueueQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let service = SyncService::new(state.db.clone(), state.http_client.clone());
    match service.process_sync_queue(query.connection_id.as_deref()).await {
        Ok(processed) => Ok(Json(serde_json::json!({ "processed": processed }))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

// ---------------------------------------------------------------------------
// Webhook Endpoints: receive real-time events from external platforms
// ---------------------------------------------------------------------------

pub async fn webhook_shopify(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    body: String,
) -> Result<StatusCode, (StatusCode, String)> {
    // Extract event type from headers
    let event_type = headers
        .get("X-Shopify-Topic")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown");
    // Store webhook event for processing
    let now = chrono::Utc::now().to_rfc3339();
    let _ = sqlx::query(
        "INSERT INTO webhook_events (platform, event_type, payload, processed, created_at) \
         VALUES ('shopify', ?, ?, 0, ?)",
    )
    .bind(event_type)
    .bind(&body)
    .bind(&now)
    .execute(&state.db)
    .await;
    // Process the webhook asynchronously
    let db = state.db.clone();
    let http = state.http_client.clone();
    let event = event_type.to_string();
    let payload = body;
    tokio::spawn(async move {
        let svc = SyncService::new(db, http);
        match event.as_str() {
            "products/create" | "products/update" | "products/delete" => {
                let _ = svc.process_webhook("shopify", &event, &payload).await;
            }
            "orders/create" | "orders/updated" => {
                let _ = svc.process_webhook("shopify", &event, &payload).await;
            }
            "inventory_levels/update" => {
                let _ = svc.process_webhook("shopify", &event, &payload).await;
            }
            _ => {}
        }
    });
    Ok(StatusCode::OK)
}

pub async fn webhook_woocommerce(
    State(state): State<AppState>,
    body: String,
) -> Result<StatusCode, (StatusCode, String)> {
    // Parse event type from WooCommerce webhook topic
    let event_type = serde_json::from_str::<serde_json::Value>(&body)
        .ok()
        .and_then(|v| v.get("topic").and_then(|t| t.as_str()))
        .unwrap_or("unknown");
    let now = chrono::Utc::now().to_rfc3339();
    let _ = sqlx::query(
        "INSERT INTO webhook_events (platform, event_type, payload, processed, created_at) \
         VALUES ('woocommerce', ?, ?, 0, ?)",
    )
    .bind(event_type)
    .bind(&body)
    .bind(&now)
    .execute(&state.db)
    .await;
    let db = state.db.clone();
    let http = state.http_client.clone();
    let event = event_type.to_string();
    let payload = body;
    tokio::spawn(async move {
        let svc = SyncService::new(db, http);
        match event.as_str() {
            "created.product" | "updated.product" | "deleted.product" => {
                let _ = svc.process_webhook("woocommerce", &event, &payload).await;
            }
            "created.order" | "updated.order" => {
                let _ = svc.process_webhook("woocommerce", &event, &payload).await;
            }
            _ => {}
        }
    });
    Ok(StatusCode::OK)
}

// ---------------------------------------------------------------------------
// Phase 2: Coupons
// ---------------------------------------------------------------------------

use crate::services::commerce_discounts::{
    CommerceDiscountsService, CreateCouponRequest, UpdateCouponRequest,
    ValidateCouponRequest, CreateMembershipTierRequest, UpdateMembershipTierRequest,
    AssignMembershipRequest,
};
use crate::services::commerce_analytics::{
    CommerceAnalyticsService, RecordEventRequest,
};
use crate::services::commerce_delivery::{
    CommerceDeliveryService, CreateShippingMethodRequest, UpdateShippingMethodRequest,
    CreateShipmentRequest, UpdateShipmentRequest, AddDeliveryEventRequest,
};
use crate::services::commerce_storefront::{
    CommerceStorefrontService, CreateStorefrontRequest, UpdateStorefrontRequest,
    CreateStorefrontSectionRequest, UpdateStorefrontSectionRequest,
};
use crate::services::commerce_inventory::{
    CommerceInventoryService, CreateWarehouseRequest, UpdateWarehouseRequest,
    SetInventoryRequest, TransferInventoryRequest, BackInStockRequestCreate,
};

#[derive(Debug, Deserialize)]
pub struct ListCouponsQuery {
    pub brand_id: String,
    pub active_only: Option<bool>,
}

pub async fn list_coupons(
    State(state): State<AppState>,
    Query(query): Query<ListCouponsQuery>,
) -> Result<Json<Vec<crate::models::Coupon>>, (StatusCode, String)> {
    let service = CommerceDiscountsService::new(state.db.clone());
    match service.list_coupons(&query.brand_id, query.active_only.unwrap_or(true)).await {
        Ok(coupons) => Ok(Json(coupons)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

pub async fn get_coupon(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<crate::models::Coupon>, (StatusCode, String)> {
    let service = CommerceDiscountsService::new(state.db.clone());
    match service.get_coupon(&id).await {
        Ok(coupon) => Ok(Json(coupon)),
        Err(e) => Err((StatusCode::NOT_FOUND, e)),
    }
}

pub async fn create_coupon(
    State(state): State<AppState>,
    Json(req): Json<CreateCouponRequest>,
) -> Result<Json<crate::models::Coupon>, (StatusCode, String)> {
    let service = CommerceDiscountsService::new(state.db.clone());
    match service.create_coupon(&req).await {
        Ok(coupon) => Ok(Json(coupon)),
        Err(e) => Err((StatusCode::BAD_REQUEST, e)),
    }
}

pub async fn update_coupon(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<UpdateCouponRequest>,
) -> Result<Json<crate::models::Coupon>, (StatusCode, String)> {
    let service = CommerceDiscountsService::new(state.db.clone());
    match service.update_coupon(&id, &req).await {
        Ok(coupon) => Ok(Json(coupon)),
        Err(e) => Err((StatusCode::BAD_REQUEST, e)),
    }
}

pub async fn delete_coupon(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    let service = CommerceDiscountsService::new(state.db.clone());
    match service.delete_coupon(&id).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

pub async fn validate_coupon(
    State(state): State<AppState>,
    Json(req): Json<ValidateCouponRequest>,
) -> Result<Json<crate::services::commerce_discounts::ValidateCouponResponse>, (StatusCode, String)> {
    let service = CommerceDiscountsService::new(state.db.clone());
    match service.validate_coupon(&req).await {
        Ok(resp) => Ok(Json(resp)),
        Err(e) => Err((StatusCode::BAD_REQUEST, e)),
    }
}

pub async fn coupon_usage_history(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Vec<crate::models::CouponUsage>>, (StatusCode, String)> {
    let service = CommerceDiscountsService::new(state.db.clone());
    match service.coupon_usage_history(&id).await {
        Ok(usage) => Ok(Json(usage)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

// ---------------------------------------------------------------------------
// Phase 2: Memberships
// ---------------------------------------------------------------------------

pub async fn list_membership_tiers(
    State(state): State<AppState>,
    Query(query): Query<ListCouponsQuery>,
) -> Result<Json<Vec<crate::models::MembershipTier>>, (StatusCode, String)> {
    let service = CommerceDiscountsService::new(state.db.clone());
    match service.list_membership_tiers(&query.brand_id).await {
        Ok(tiers) => Ok(Json(tiers)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

pub async fn get_membership_tier(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<crate::models::MembershipTier>, (StatusCode, String)> {
    let service = CommerceDiscountsService::new(state.db.clone());
    match service.get_membership_tier(&id).await {
        Ok(tier) => Ok(Json(tier)),
        Err(e) => Err((StatusCode::NOT_FOUND, e)),
    }
}

pub async fn create_membership_tier(
    State(state): State<AppState>,
    Json(req): Json<CreateMembershipTierRequest>,
) -> Result<Json<crate::models::MembershipTier>, (StatusCode, String)> {
    let service = CommerceDiscountsService::new(state.db.clone());
    match service.create_membership_tier(&req).await {
        Ok(tier) => Ok(Json(tier)),
        Err(e) => Err((StatusCode::BAD_REQUEST, e)),
    }
}

pub async fn update_membership_tier(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<UpdateMembershipTierRequest>,
) -> Result<Json<crate::models::MembershipTier>, (StatusCode, String)> {
    let service = CommerceDiscountsService::new(state.db.clone());
    match service.update_membership_tier(&id, &req).await {
        Ok(tier) => Ok(Json(tier)),
        Err(e) => Err((StatusCode::BAD_REQUEST, e)),
    }
}

pub async fn delete_membership_tier(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    let service = CommerceDiscountsService::new(state.db.clone());
    match service.delete_membership_tier(&id).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

pub async fn assign_membership(
    State(state): State<AppState>,
    Json(req): Json<AssignMembershipRequest>,
) -> Result<Json<crate::models::CustomerMembership>, (StatusCode, String)> {
    let service = CommerceDiscountsService::new(state.db.clone());
    match service.assign_membership(&req).await {
        Ok(m) => Ok(Json(m)),
        Err(e) => Err((StatusCode::BAD_REQUEST, e)),
    }
}

#[derive(Debug, Deserialize)]
pub struct CustomerMembershipsQuery {
    pub customer_id: String,
}

pub async fn list_customer_memberships(
    State(state): State<AppState>,
    Query(query): Query<CustomerMembershipsQuery>,
) -> Result<Json<Vec<crate::models::CustomerMembership>>, (StatusCode, String)> {
    let service = CommerceDiscountsService::new(state.db.clone());
    match service.list_customer_memberships(&query.customer_id).await {
        Ok(m) => Ok(Json(m)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

// ---------------------------------------------------------------------------
// Phase 2: Analytics
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct AnalyticsQuery {
    pub brand_id: String,
    pub days: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct TopProductsQuery {
    pub brand_id: String,
    pub limit: Option<i32>,
}

pub async fn analytics_overview(
    State(state): State<AppState>,
    Query(query): Query<AnalyticsQuery>,
) -> Result<Json<crate::services::commerce_analytics::AnalyticsOverview>, (StatusCode, String)> {
    let service = CommerceAnalyticsService::new(state.db.clone());
    match service.overview(&query.brand_id, query.days.unwrap_or(30)).await {
        Ok(overview) => Ok(Json(overview)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

pub async fn analytics_revenue_trend(
    State(state): State<AppState>,
    Query(query): Query<AnalyticsQuery>,
) -> Result<Json<Vec<crate::services::commerce_analytics::RevenueTrendPoint>>, (StatusCode, String)> {
    let service = CommerceAnalyticsService::new(state.db.clone());
    match service.revenue_trend(&query.brand_id, query.days.unwrap_or(30)).await {
        Ok(trend) => Ok(Json(trend)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

pub async fn analytics_top_products(
    State(state): State<AppState>,
    Query(query): Query<TopProductsQuery>,
) -> Result<Json<Vec<crate::services::commerce_analytics::TopProduct>>, (StatusCode, String)> {
    let service = CommerceAnalyticsService::new(state.db.clone());
    match service.top_products(&query.brand_id, query.limit.unwrap_or(10)).await {
        Ok(products) => Ok(Json(products)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

pub async fn analytics_top_customers(
    State(state): State<AppState>,
    Query(query): Query<TopProductsQuery>,
) -> Result<Json<Vec<crate::services::commerce_analytics::TopCustomer>>, (StatusCode, String)> {
    let service = CommerceAnalyticsService::new(state.db.clone());
    match service.top_customers(&query.brand_id, query.limit.unwrap_or(10)).await {
        Ok(customers) => Ok(Json(customers)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

pub async fn analytics_channels(
    State(state): State<AppState>,
    Query(query): Query<AnalyticsQuery>,
) -> Result<Json<Vec<crate::services::commerce_analytics::ChannelPerformance>>, (StatusCode, String)> {
    let service = CommerceAnalyticsService::new(state.db.clone());
    match service.channel_performance(&query.brand_id).await {
        Ok(channels) => Ok(Json(channels)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

pub async fn analytics_update_daily_stats(
    State(state): State<AppState>,
    Query(query): Query<AnalyticsQuery>,
) -> Result<StatusCode, (StatusCode, String)> {
    let service = CommerceAnalyticsService::new(state.db.clone());
    match service.update_daily_stats(&query.brand_id).await {
        Ok(()) => Ok(StatusCode::OK),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

pub async fn analytics_daily_stats(
    State(state): State<AppState>,
    Query(query): Query<AnalyticsQuery>,
) -> Result<Json<Vec<crate::models::AnalyticsDailyStat>>, (StatusCode, String)> {
    let service = CommerceAnalyticsService::new(state.db.clone());
    match service.daily_stats(&query.brand_id, query.days.unwrap_or(30)).await {
        Ok(stats) => Ok(Json(stats)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

pub async fn record_analytics_event(
    State(state): State<AppState>,
    Json(req): Json<RecordEventRequest>,
) -> Result<Json<crate::models::AnalyticsEvent>, (StatusCode, String)> {
    let service = CommerceAnalyticsService::new(state.db.clone());
    match service.record_event(&req).await {
        Ok(event) => Ok(Json(event)),
        Err(e) => Err((StatusCode::BAD_REQUEST, e)),
    }
}

// ---------------------------------------------------------------------------
// Phase 2: Shipping Methods
// ---------------------------------------------------------------------------

pub async fn list_shipping_methods(
    State(state): State<AppState>,
    Query(query): Query<ListCouponsQuery>,
) -> Result<Json<Vec<crate::models::ShippingMethod>>, (StatusCode, String)> {
    let service = CommerceDeliveryService::new(state.db.clone());
    match service.list_shipping_methods(&query.brand_id).await {
        Ok(methods) => Ok(Json(methods)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

pub async fn get_shipping_method(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<crate::models::ShippingMethod>, (StatusCode, String)> {
    let service = CommerceDeliveryService::new(state.db.clone());
    match service.get_shipping_method(&id).await {
        Ok(method) => Ok(Json(method)),
        Err(e) => Err((StatusCode::NOT_FOUND, e)),
    }
}

pub async fn create_shipping_method(
    State(state): State<AppState>,
    Json(req): Json<CreateShippingMethodRequest>,
) -> Result<Json<crate::models::ShippingMethod>, (StatusCode, String)> {
    let service = CommerceDeliveryService::new(state.db.clone());
    match service.create_shipping_method(&req).await {
        Ok(method) => Ok(Json(method)),
        Err(e) => Err((StatusCode::BAD_REQUEST, e)),
    }
}

pub async fn update_shipping_method(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<UpdateShippingMethodRequest>,
) -> Result<Json<crate::models::ShippingMethod>, (StatusCode, String)> {
    let service = CommerceDeliveryService::new(state.db.clone());
    match service.update_shipping_method(&id, &req).await {
        Ok(method) => Ok(Json(method)),
        Err(e) => Err((StatusCode::BAD_REQUEST, e)),
    }
}

pub async fn delete_shipping_method(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    let service = CommerceDeliveryService::new(state.db.clone());
    match service.delete_shipping_method(&id).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

// ---------------------------------------------------------------------------
// Phase 2: Shipments
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct ListShipmentsQuery {
    pub order_id: Option<String>,
}

pub async fn list_shipments(
    State(state): State<AppState>,
    Query(query): Query<ListShipmentsQuery>,
) -> Result<Json<Vec<crate::models::Shipment>>, (StatusCode, String)> {
    let service = CommerceDeliveryService::new(state.db.clone());
    match service.list_shipments(query.order_id.as_deref()).await {
        Ok(shipments) => Ok(Json(shipments)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

pub async fn get_shipment(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<crate::models::Shipment>, (StatusCode, String)> {
    let service = CommerceDeliveryService::new(state.db.clone());
    match service.get_shipment(&id).await {
        Ok(shipment) => Ok(Json(shipment)),
        Err(e) => Err((StatusCode::NOT_FOUND, e)),
    }
}

pub async fn create_shipment(
    State(state): State<AppState>,
    Json(req): Json<CreateShipmentRequest>,
) -> Result<Json<crate::models::Shipment>, (StatusCode, String)> {
    let service = CommerceDeliveryService::new(state.db.clone());
    match service.create_shipment(&req).await {
        Ok(shipment) => Ok(Json(shipment)),
        Err(e) => Err((StatusCode::BAD_REQUEST, e)),
    }
}

pub async fn update_shipment(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<UpdateShipmentRequest>,
) -> Result<Json<crate::models::Shipment>, (StatusCode, String)> {
    let service = CommerceDeliveryService::new(state.db.clone());
    match service.update_shipment(&id, &req).await {
        Ok(shipment) => Ok(Json(shipment)),
        Err(e) => Err((StatusCode::BAD_REQUEST, e)),
    }
}

pub async fn list_delivery_events(
    State(state): State<AppState>,
    Path(shipment_id): Path<String>,
) -> Result<Json<Vec<crate::models::DeliveryEvent>>, (StatusCode, String)> {
    let service = CommerceDeliveryService::new(state.db.clone());
    match service.list_delivery_events(&shipment_id).await {
        Ok(events) => Ok(Json(events)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

pub async fn add_delivery_event(
    State(state): State<AppState>,
    Json(req): Json<AddDeliveryEventRequest>,
) -> Result<Json<crate::models::DeliveryEvent>, (StatusCode, String)> {
    let service = CommerceDeliveryService::new(state.db.clone());
    match service.add_delivery_event(&req).await {
        Ok(event) => Ok(Json(event)),
        Err(e) => Err((StatusCode::BAD_REQUEST, e)),
    }
}

// ---------------------------------------------------------------------------
// Phase 2: Storefront
// ---------------------------------------------------------------------------

pub async fn list_storefronts(
    State(state): State<AppState>,
    Query(query): Query<ListCouponsQuery>,
) -> Result<Json<Vec<crate::models::Storefront>>, (StatusCode, String)> {
    let service = CommerceStorefrontService::new(state.db.clone());
    match service.list_storefronts(&query.brand_id).await {
        Ok(storefronts) => Ok(Json(storefronts)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

pub async fn get_storefront(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<crate::models::Storefront>, (StatusCode, String)> {
    let service = CommerceStorefrontService::new(state.db.clone());
    match service.get_storefront(&id).await {
        Ok(sf) => Ok(Json(sf)),
        Err(e) => Err((StatusCode::NOT_FOUND, e)),
    }
}

pub async fn create_storefront(
    State(state): State<AppState>,
    Json(req): Json<CreateStorefrontRequest>,
) -> Result<Json<crate::models::Storefront>, (StatusCode, String)> {
    let service = CommerceStorefrontService::new(state.db.clone());
    match service.create_storefront(&req).await {
        Ok(sf) => Ok(Json(sf)),
        Err(e) => Err((StatusCode::BAD_REQUEST, e)),
    }
}

pub async fn update_storefront(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<UpdateStorefrontRequest>,
) -> Result<Json<crate::models::Storefront>, (StatusCode, String)> {
    let service = CommerceStorefrontService::new(state.db.clone());
    match service.update_storefront(&id, &req).await {
        Ok(sf) => Ok(Json(sf)),
        Err(e) => Err((StatusCode::BAD_REQUEST, e)),
    }
}

pub async fn delete_storefront(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    let service = CommerceStorefrontService::new(state.db.clone());
    match service.delete_storefront(&id).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

pub async fn get_public_storefront(
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> Result<Json<crate::services::commerce_storefront::StorefrontPublicResponse>, (StatusCode, String)> {
    let service = CommerceStorefrontService::new(state.db.clone());
    match service.get_public_storefront(&slug).await {
        Ok(Some(resp)) => Ok(Json(resp)),
        Ok(None) => Err((StatusCode::NOT_FOUND, "Storefront not found".to_string())),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

pub async fn list_storefront_sections(
    State(state): State<AppState>,
    Path(storefront_id): Path<String>,
) -> Result<Json<Vec<crate::models::StorefrontSection>>, (StatusCode, String)> {
    let service = CommerceStorefrontService::new(state.db.clone());
    match service.list_sections(&storefront_id).await {
        Ok(sections) => Ok(Json(sections)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

pub async fn create_storefront_section(
    State(state): State<AppState>,
    Json(req): Json<CreateStorefrontSectionRequest>,
) -> Result<Json<crate::models::StorefrontSection>, (StatusCode, String)> {
    let service = CommerceStorefrontService::new(state.db.clone());
    match service.create_section(&req).await {
        Ok(section) => Ok(Json(section)),
        Err(e) => Err((StatusCode::BAD_REQUEST, e)),
    }
}

pub async fn update_storefront_section(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<UpdateStorefrontSectionRequest>,
) -> Result<Json<crate::models::StorefrontSection>, (StatusCode, String)> {
    let service = CommerceStorefrontService::new(state.db.clone());
    match service.update_section(&id, &req).await {
        Ok(section) => Ok(Json(section)),
        Err(e) => Err((StatusCode::BAD_REQUEST, e)),
    }
}

pub async fn delete_storefront_section(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    let service = CommerceStorefrontService::new(state.db.clone());
    match service.delete_section(&id).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

// ---------------------------------------------------------------------------
// Phase 2: Warehouses
// ---------------------------------------------------------------------------

pub async fn list_warehouses(
    State(state): State<AppState>,
    Query(query): Query<ListCouponsQuery>,
) -> Result<Json<Vec<crate::models::Warehouse>>, (StatusCode, String)> {
    let service = CommerceInventoryService::new(state.db.clone());
    match service.list_warehouses(&query.brand_id).await {
        Ok(warehouses) => Ok(Json(warehouses)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

pub async fn get_warehouse(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<crate::models::Warehouse>, (StatusCode, String)> {
    let service = CommerceInventoryService::new(state.db.clone());
    match service.get_warehouse(&id).await {
        Ok(wh) => Ok(Json(wh)),
        Err(e) => Err((StatusCode::NOT_FOUND, e)),
    }
}

pub async fn create_warehouse(
    State(state): State<AppState>,
    Json(req): Json<CreateWarehouseRequest>,
) -> Result<Json<crate::models::Warehouse>, (StatusCode, String)> {
    let service = CommerceInventoryService::new(state.db.clone());
    match service.create_warehouse(&req).await {
        Ok(wh) => Ok(Json(wh)),
        Err(e) => Err((StatusCode::BAD_REQUEST, e)),
    }
}

pub async fn update_warehouse(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<UpdateWarehouseRequest>,
) -> Result<Json<crate::models::Warehouse>, (StatusCode, String)> {
    let service = CommerceInventoryService::new(state.db.clone());
    match service.update_warehouse(&id, &req).await {
        Ok(wh) => Ok(Json(wh)),
        Err(e) => Err((StatusCode::BAD_REQUEST, e)),
    }
}

pub async fn delete_warehouse(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    let service = CommerceInventoryService::new(state.db.clone());
    match service.delete_warehouse(&id).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

// ---------------------------------------------------------------------------
// Phase 2: Inventory
// ---------------------------------------------------------------------------

pub async fn get_inventory(
    State(state): State<AppState>,
    Path(product_id): Path<String>,
) -> Result<Json<Vec<crate::models::InventoryByWarehouse>>, (StatusCode, String)> {
    let service = CommerceInventoryService::new(state.db.clone());
    match service.get_inventory(&product_id).await {
        Ok(inv) => Ok(Json(inv)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

pub async fn get_inventory_summary(
    State(state): State<AppState>,
    Path(product_id): Path<String>,
) -> Result<Json<crate::services::commerce_inventory::InventorySummary>, (StatusCode, String)> {
    let service = CommerceInventoryService::new(state.db.clone());
    match service.get_inventory_summary(&product_id).await {
        Ok(summary) => Ok(Json(summary)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

pub async fn set_inventory(
    State(state): State<AppState>,
    Json(req): Json<SetInventoryRequest>,
) -> Result<Json<crate::models::InventoryByWarehouse>, (StatusCode, String)> {
    let service = CommerceInventoryService::new(state.db.clone());
    match service.set_inventory(&req).await {
        Ok(inv) => Ok(Json(inv)),
        Err(e) => Err((StatusCode::BAD_REQUEST, e)),
    }
}

pub async fn transfer_inventory(
    State(state): State<AppState>,
    Json(req): Json<TransferInventoryRequest>,
) -> Result<StatusCode, (StatusCode, String)> {
    let service = CommerceInventoryService::new(state.db.clone());
    match service.transfer_inventory(&req).await {
        Ok(()) => Ok(StatusCode::OK),
        Err(e) => Err((StatusCode::BAD_REQUEST, e)),
    }
}

pub async fn list_low_stock_alerts(
    State(state): State<AppState>,
) -> Result<Json<Vec<crate::models::LowStockAlert>>, (StatusCode, String)> {
    let service = CommerceInventoryService::new(state.db.clone());
    match service.list_low_stock_alerts(None).await {
        Ok(alerts) => Ok(Json(alerts)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

pub async fn acknowledge_low_stock_alert(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    let service = CommerceInventoryService::new(state.db.clone());
    match service.acknowledge_alert(&id).await {
        Ok(()) => Ok(StatusCode::OK),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

pub async fn list_back_in_stock_requests(
    State(state): State<AppState>,
) -> Result<Json<Vec<crate::models::BackInStockRequest>>, (StatusCode, String)> {
    let service = CommerceInventoryService::new(state.db.clone());
    match service.list_back_in_stock_requests(None).await {
        Ok(requests) => Ok(Json(requests)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

pub async fn create_back_in_stock_request(
    State(state): State<AppState>,
    Json(req): Json<BackInStockRequestCreate>,
) -> Result<Json<crate::models::BackInStockRequest>, (StatusCode, String)> {
    let service = CommerceInventoryService::new(state.db.clone());
    match service.create_back_in_stock_request(&req).await {
        Ok(req) => Ok(Json(req)),
        Err(e) => Err((StatusCode::BAD_REQUEST, e)),
    }
}

pub async fn transaction_history(
    State(state): State<AppState>,
    Path(product_id): Path<String>,
) -> Result<Json<Vec<crate::models::InventoryTransaction>>, (StatusCode, String)> {
    let service = CommerceInventoryService::new(state.db.clone());
    match service.transaction_history(&product_id, None).await {
        Ok(tx) => Ok(Json(tx)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

// ---------------------------------------------------------------------------
// Gap #1: Product Options
// ---------------------------------------------------------------------------

pub async fn list_product_options(
    State(state): State<AppState>,
    Path(product_id): Path<String>,
) -> Result<Json<Vec<crate::services::commerce_options::ProductOptionWithValues>>, (StatusCode, String)> {
    let service = ProductOptionsService::new(state.db.clone());
    match service.list_options(&product_id).await {
        Ok(opts) => Ok(Json(opts)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

pub async fn get_product_option(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<crate::services::commerce_options::ProductOptionWithValues>, (StatusCode, String)> {
    let service = ProductOptionsService::new(state.db.clone());
    match service.get_option(&id).await {
        Ok(opt) => Ok(Json(opt)),
        Err(e) => Err((StatusCode::NOT_FOUND, e)),
    }
}

pub async fn create_product_option(
    State(state): State<AppState>,
    Json(req): Json<CreateProductOptionRequest>,
) -> Result<Json<crate::services::commerce_options::ProductOptionWithValues>, (StatusCode, String)> {
    let service = ProductOptionsService::new(state.db.clone());
    match service.create_option(&req).await {
        Ok(opt) => Ok(Json(opt)),
        Err(e) => Err((StatusCode::BAD_REQUEST, e)),
    }
}

pub async fn update_product_option(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<UpdateProductOptionRequest>,
) -> Result<Json<crate::services::commerce_options::ProductOptionWithValues>, (StatusCode, String)> {
    let service = ProductOptionsService::new(state.db.clone());
    match service.update_option(&id, &req).await {
        Ok(opt) => Ok(Json(opt)),
        Err(e) => Err((StatusCode::BAD_REQUEST, e)),
    }
}

pub async fn delete_product_option(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    let service = ProductOptionsService::new(state.db.clone());
    match service.delete_option(&id).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

pub async fn create_product_option_value(
    State(state): State<AppState>,
    Path(option_id): Path<String>,
    Json(req): Json<CreateProductOptionValueRequest>,
) -> Result<Json<crate::models::ProductOptionValue>, (StatusCode, String)> {
    let service = ProductOptionsService::new(state.db.clone());
    match service.create_option_value(&option_id, &req).await {
        Ok(val) => Ok(Json(val)),
        Err(e) => Err((StatusCode::BAD_REQUEST, e)),
    }
}

pub async fn delete_product_option_value(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    let service = ProductOptionsService::new(state.db.clone());
    match service.delete_option_value(&id).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

// ---------------------------------------------------------------------------
// Gap #2: Delivery Fee Rules
// ---------------------------------------------------------------------------

pub async fn list_delivery_fee_rules(
    State(state): State<AppState>,
    Query(query): Query<ListCategoriesQuery>,
) -> Result<Json<Vec<crate::models::DeliveryFeeRule>>, (StatusCode, String)> {
    let service = CommerceGapsService::new(state.db.clone());
    match service.list_delivery_fee_rules(&query.brand_id).await {
        Ok(rules) => Ok(Json(rules)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

pub async fn create_delivery_fee_rule(
    State(state): State<AppState>,
    Json(req): Json<CreateDeliveryFeeRuleRequest>,
) -> Result<Json<crate::models::DeliveryFeeRule>, (StatusCode, String)> {
    let service = CommerceGapsService::new(state.db.clone());
    match service.create_delivery_fee_rule(&req).await {
        Ok(rule) => Ok(Json(rule)),
        Err(e) => Err((StatusCode::BAD_REQUEST, e)),
    }
}

pub async fn update_delivery_fee_rule(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<CreateDeliveryFeeRuleRequest>,
) -> Result<Json<crate::models::DeliveryFeeRule>, (StatusCode, String)> {
    let service = CommerceGapsService::new(state.db.clone());
    match service.update_delivery_fee_rule(&id, &req).await {
        Ok(rule) => Ok(Json(rule)),
        Err(e) => Err((StatusCode::BAD_REQUEST, e)),
    }
}

pub async fn delete_delivery_fee_rule(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    let service = CommerceGapsService::new(state.db.clone());
    match service.delete_delivery_fee_rule(&id).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

// ---------------------------------------------------------------------------
// Gap #3: Payment Proofs
// ---------------------------------------------------------------------------

pub async fn upload_payment_proof(
    State(state): State<AppState>,
    Json(req): Json<UploadPaymentProofRequest>,
) -> Result<Json<crate::models::PaymentProof>, (StatusCode, String)> {
    let service = CommerceGapsService::new(state.db.clone());
    match service.upload_payment_proof(&req).await {
        Ok(proof) => Ok(Json(proof)),
        Err(e) => Err((StatusCode::BAD_REQUEST, e)),
    }
}

pub async fn list_payment_proofs(
    State(state): State<AppState>,
) -> Result<Json<Vec<crate::models::PaymentProof>>, (StatusCode, String)> {
    let service = CommerceGapsService::new(state.db.clone());
    match service.list_payment_proofs(None).await {
        Ok(proofs) => Ok(Json(proofs)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

pub async fn verify_payment_proof(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<VerifyPaymentProofRequest>,
) -> Result<Json<crate::models::PaymentProof>, (StatusCode, String)> {
    let service = CommerceGapsService::new(state.db.clone());
    match service.verify_payment_proof(&id, &req).await {
        Ok(proof) => Ok(Json(proof)),
        Err(e) => Err((StatusCode::BAD_REQUEST, e)),
    }
}

// ---------------------------------------------------------------------------
// Gap #4: Order Notification Templates
// ---------------------------------------------------------------------------

pub async fn list_notification_templates(
    State(state): State<AppState>,
    Query(query): Query<ListCategoriesQuery>,
) -> Result<Json<Vec<crate::models::OrderNotificationTemplate>>, (StatusCode, String)> {
    let service = CommerceGapsService::new(state.db.clone());
    match service.list_notification_templates(&query.brand_id).await {
        Ok(tpls) => Ok(Json(tpls)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

pub async fn create_notification_template(
    State(state): State<AppState>,
    Json(req): Json<CreateNotificationTemplateRequest>,
) -> Result<Json<crate::models::OrderNotificationTemplate>, (StatusCode, String)> {
    let service = CommerceGapsService::new(state.db.clone());
    match service.create_notification_template(&req).await {
        Ok(tpl) => Ok(Json(tpl)),
        Err(e) => Err((StatusCode::BAD_REQUEST, e)),
    }
}

pub async fn delete_notification_template(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    let service = CommerceGapsService::new(state.db.clone());
    match service.delete_notification_template(&id).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

// ---------------------------------------------------------------------------
// Gap #6: Payment Gateways
// ---------------------------------------------------------------------------

pub async fn list_payment_gateways(
    State(state): State<AppState>,
    Query(query): Query<ListCategoriesQuery>,
) -> Result<Json<Vec<crate::models::PaymentGateway>>, (StatusCode, String)> {
    let service = CommerceGapsService::new(state.db.clone());
    match service.list_payment_gateways(&query.brand_id).await {
        Ok(gws) => Ok(Json(gws)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

pub async fn create_payment_gateway(
    State(state): State<AppState>,
    Json(req): Json<CreatePaymentGatewayRequest>,
) -> Result<Json<crate::models::PaymentGateway>, (StatusCode, String)> {
    let service = CommerceGapsService::new(state.db.clone());
    match service.create_payment_gateway(&req).await {
        Ok(gw) => Ok(Json(gw)),
        Err(e) => Err((StatusCode::BAD_REQUEST, e)),
    }
}

pub async fn delete_payment_gateway(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    let service = CommerceGapsService::new(state.db.clone());
    match service.delete_payment_gateway(&id).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

// ---------------------------------------------------------------------------
// Gap #7: Service Types
// ---------------------------------------------------------------------------

pub async fn list_service_types(
    State(state): State<AppState>,
    Query(query): Query<ListCategoriesQuery>,
) -> Result<Json<Vec<crate::models::ServiceType>>, (StatusCode, String)> {
    let service = CommerceGapsService::new(state.db.clone());
    match service.list_service_types(&query.brand_id).await {
        Ok(types) => Ok(Json(types)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

pub async fn create_service_type(
    State(state): State<AppState>,
    Json(req): Json<CreateServiceTypeRequest>,
) -> Result<Json<crate::models::ServiceType>, (StatusCode, String)> {
    let service = CommerceGapsService::new(state.db.clone());
    match service.create_service_type(&req).await {
        Ok(st) => Ok(Json(st)),
        Err(e) => Err((StatusCode::BAD_REQUEST, e)),
    }
}

pub async fn delete_service_type(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    let service = CommerceGapsService::new(state.db.clone());
    match service.delete_service_type(&id).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

// ---------------------------------------------------------------------------
// Gap #8: Delivery Schedules & Time Slots
// ---------------------------------------------------------------------------

pub async fn list_delivery_schedules(
    State(state): State<AppState>,
    Query(query): Query<ListCategoriesQuery>,
) -> Result<Json<Vec<crate::models::DeliverySchedule>>, (StatusCode, String)> {
    let service = CommerceGapsService::new(state.db.clone());
    match service.list_delivery_schedules(&query.brand_id).await {
        Ok(schedules) => Ok(Json(schedules)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

pub async fn create_delivery_schedule(
    State(state): State<AppState>,
    Json(req): Json<CreateDeliveryScheduleRequest>,
) -> Result<Json<crate::models::DeliverySchedule>, (StatusCode, String)> {
    let service = CommerceGapsService::new(state.db.clone());
    match service.create_delivery_schedule(&req).await {
        Ok(schedule) => Ok(Json(schedule)),
        Err(e) => Err((StatusCode::BAD_REQUEST, e)),
    }
}

pub async fn delete_delivery_schedule(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    let service = CommerceGapsService::new(state.db.clone());
    match service.delete_delivery_schedule(&id).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

pub async fn list_time_slots(
    State(state): State<AppState>,
    Query(query): Query<ListCategoriesQuery>,
) -> Result<Json<Vec<crate::models::DeliveryTimeSlot>>, (StatusCode, String)> {
    let service = CommerceGapsService::new(state.db.clone());
    match service.list_time_slots(&query.brand_id, None).await {
        Ok(slots) => Ok(Json(slots)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

pub async fn create_time_slot(
    State(state): State<AppState>,
    Json(req): Json<CreateDeliveryTimeSlotRequest>,
) -> Result<Json<crate::models::DeliveryTimeSlot>, (StatusCode, String)> {
    let service = CommerceGapsService::new(state.db.clone());
    match service.create_time_slot(&req).await {
        Ok(slot) => Ok(Json(slot)),
        Err(e) => Err((StatusCode::BAD_REQUEST, e)),
    }
}

pub async fn delete_time_slot(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    let service = CommerceGapsService::new(state.db.clone());
    match service.delete_time_slot(&id).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

// ---------------------------------------------------------------------------
// Gap #9: Invoices
// ---------------------------------------------------------------------------

pub async fn generate_invoice(
    State(state): State<AppState>,
    Path(order_id): Path<String>,
) -> Result<Json<crate::models::Invoice>, (StatusCode, String)> {
    let service = CommerceGapsService::new(state.db.clone());
    match service.generate_invoice(&order_id).await {
        Ok(inv) => Ok(Json(inv)),
        Err(e) => Err((StatusCode::BAD_REQUEST, e)),
    }
}

// ---------------------------------------------------------------------------
// Gap #10: CSV Import
// ---------------------------------------------------------------------------

pub async fn import_csv(
    State(state): State<AppState>,
    Json(req): Json<ImportCsvRequest>,
) -> Result<Json<crate::models::CsvImportJob>, (StatusCode, String)> {
    let service = CommerceGapsService::new(state.db.clone());
    match service.import_csv(&req).await {
        Ok(job) => Ok(Json(job)),
        Err(e) => Err((StatusCode::BAD_REQUEST, e)),
    }
}

pub async fn list_csv_import_jobs(
    State(state): State<AppState>,
    Query(query): Query<ListCategoriesQuery>,
) -> Result<Json<Vec<crate::models::CsvImportJob>>, (StatusCode, String)> {
    let service = CommerceGapsService::new(state.db.clone());
    match service.list_csv_import_jobs(&query.brand_id).await {
        Ok(jobs) => Ok(Json(jobs)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

// ---------------------------------------------------------------------------
// Gap #11: Customer Accounts
// ---------------------------------------------------------------------------

pub async fn create_customer_account(
    State(state): State<AppState>,
    Path(customer_id): Path<String>,
    Json(req): Json<CreateCustomerAccountRequest>,
) -> Result<Json<crate::models::CustomerAccount>, (StatusCode, String)> {
    let service = CommerceGapsService::new(state.db.clone());
    match service.create_customer_account(&customer_id, &req).await {
        Ok(acc) => Ok(Json(acc)),
        Err(e) => Err((StatusCode::BAD_REQUEST, e)),
    }
}

pub async fn customer_login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<crate::models::CustomerAccount>, (StatusCode, String)> {
    let service = CommerceGapsService::new(state.db.clone());
    match service.login(&req).await {
        Ok(acc) => Ok(Json(acc)),
        Err(e) => Err((StatusCode::UNAUTHORIZED, e)),
    }
}

// ---------------------------------------------------------------------------
// Gap #12: Customer Reviews
// ---------------------------------------------------------------------------

pub async fn create_review(
    State(state): State<AppState>,
    Json(req): Json<CreateReviewRequest>,
) -> Result<Json<crate::models::CustomerReview>, (StatusCode, String)> {
    let service = CommerceGapsService::new(state.db.clone());
    match service.create_review(&req).await {
        Ok(review) => Ok(Json(review)),
        Err(e) => Err((StatusCode::BAD_REQUEST, e)),
    }
}

pub async fn list_reviews(
    State(state): State<AppState>,
) -> Result<Json<Vec<crate::models::CustomerReview>>, (StatusCode, String)> {
    let service = CommerceGapsService::new(state.db.clone());
    match service.list_reviews(None, true).await {
        Ok(reviews) => Ok(Json(reviews)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

// ---------------------------------------------------------------------------
// Gap #13: Loyalty Programs
// ---------------------------------------------------------------------------

pub async fn list_loyalty_programs(
    State(state): State<AppState>,
    Query(query): Query<ListCategoriesQuery>,
) -> Result<Json<Vec<crate::models::LoyaltyProgram>>, (StatusCode, String)> {
    let service = CommerceGapsService::new(state.db.clone());
    match service.list_loyalty_programs(&query.brand_id).await {
        Ok(programs) => Ok(Json(programs)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

pub async fn create_loyalty_program(
    State(state): State<AppState>,
    Json(req): Json<CreateLoyaltyProgramRequest>,
) -> Result<Json<crate::models::LoyaltyProgram>, (StatusCode, String)> {
    let service = CommerceGapsService::new(state.db.clone());
    match service.create_loyalty_program(&req).await {
        Ok(program) => Ok(Json(program)),
        Err(e) => Err((StatusCode::BAD_REQUEST, e)),
    }
}

pub async fn get_customer_points(
    State(state): State<AppState>,
    Query(query): Query<GetOrCreateCartQuery>,
) -> Result<Json<crate::models::CustomerPoints>, (StatusCode, String)> {
    let service = CommerceGapsService::new(state.db.clone());
    match service.get_customer_points(&query.customer_id, &query.brand_id).await {
        Ok(points) => Ok(Json(points)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

// ---------------------------------------------------------------------------
// Gap #16: Staff Management
// ---------------------------------------------------------------------------

pub async fn list_staff_members(
    State(state): State<AppState>,
    Query(query): Query<ListCategoriesQuery>,
) -> Result<Json<Vec<crate::models::StaffMember>>, (StatusCode, String)> {
    let service = CommerceGapsService::new(state.db.clone());
    match service.list_staff_members(&query.brand_id).await {
        Ok(staff) => Ok(Json(staff)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

pub async fn create_staff_member(
    State(state): State<AppState>,
    Json(req): Json<CreateStaffMemberRequest>,
) -> Result<Json<crate::models::StaffMember>, (StatusCode, String)> {
    let service = CommerceGapsService::new(state.db.clone());
    match service.create_staff_member(&req).await {
        Ok(member) => Ok(Json(member)),
        Err(e) => Err((StatusCode::BAD_REQUEST, e)),
    }
}

pub async fn delete_staff_member(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    let service = CommerceGapsService::new(state.db.clone());
    match service.delete_staff_member(&id).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

// ---------------------------------------------------------------------------
// Gap #17: POS Sessions
// ---------------------------------------------------------------------------

pub async fn create_pos_session(
    State(state): State<AppState>,
    Query(query): Query<ListCategoriesQuery>,
) -> Result<Json<crate::models::PosSession>, (StatusCode, String)> {
    let service = CommerceGapsService::new(state.db.clone());
    match service.create_pos_session(&query.brand_id, None).await {
        Ok(session) => Ok(Json(session)),
        Err(e) => Err((StatusCode::BAD_REQUEST, e)),
    }
}

pub async fn end_pos_session(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    let service = CommerceGapsService::new(state.db.clone());
    match service.end_pos_session(&id).await {
        Ok(()) => Ok(StatusCode::OK),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

// ---------------------------------------------------------------------------
// Gap #18: Receipts
// ---------------------------------------------------------------------------

pub async fn generate_receipt(
    State(state): State<AppState>,
    Path(order_id): Path<String>,
) -> Result<Json<crate::models::Receipt>, (StatusCode, String)> {
    let service = CommerceGapsService::new(state.db.clone());
    match service.generate_receipt(&order_id).await {
        Ok(receipt) => Ok(Json(receipt)),
        Err(e) => Err((StatusCode::BAD_REQUEST, e)),
    }
}

// ---------------------------------------------------------------------------
// Gap #20: Wholesale Pricing
// ---------------------------------------------------------------------------

pub async fn list_wholesale_pricing(
    State(state): State<AppState>,
    Path(product_id): Path<String>,
) -> Result<Json<Vec<crate::models::WholesalePricing>>, (StatusCode, String)> {
    let service = CommerceGapsService::new(state.db.clone());
    match service.list_wholesale_pricing(&product_id).await {
        Ok(pricing) => Ok(Json(pricing)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

pub async fn create_wholesale_pricing(
    State(state): State<AppState>,
    Json(req): Json<CreateWholesalePricingRequest>,
) -> Result<Json<crate::models::WholesalePricing>, (StatusCode, String)> {
    let service = CommerceGapsService::new(state.db.clone());
    match service.create_wholesale_pricing(&req).await {
        Ok(pricing) => Ok(Json(pricing)),
        Err(e) => Err((StatusCode::BAD_REQUEST, e)),
    }
}

pub async fn delete_wholesale_pricing(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    let service = CommerceGapsService::new(state.db.clone());
    match service.delete_wholesale_pricing(&id).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

// ---------------------------------------------------------------------------
// Gap #23: Automated Follow-ups
// ---------------------------------------------------------------------------

pub async fn list_followups(
    State(state): State<AppState>,
    Query(query): Query<ListCategoriesQuery>,
) -> Result<Json<Vec<crate::models::AutomatedFollowup>>, (StatusCode, String)> {
    let service = CommerceGapsService::new(state.db.clone());
    match service.list_followups(&query.brand_id).await {
        Ok(followups) => Ok(Json(followups)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

pub async fn create_followup(
    State(state): State<AppState>,
    Json(req): Json<CreateFollowupRequest>,
) -> Result<Json<crate::models::AutomatedFollowup>, (StatusCode, String)> {
    let service = CommerceGapsService::new(state.db.clone());
    match service.create_followup(&req).await {
        Ok(followup) => Ok(Json(followup)),
        Err(e) => Err((StatusCode::BAD_REQUEST, e)),
    }
}

pub async fn delete_followup(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    let service = CommerceGapsService::new(state.db.clone());
    match service.delete_followup(&id).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

// ---------------------------------------------------------------------------
// Gap #24: Custom Domains
// ---------------------------------------------------------------------------

pub async fn list_custom_domains(
    State(state): State<AppState>,
    Query(query): Query<ListCategoriesQuery>,
) -> Result<Json<Vec<crate::models::CustomDomain>>, (StatusCode, String)> {
    let service = CommerceGapsService::new(state.db.clone());
    match service.list_custom_domains(&query.brand_id).await {
        Ok(domains) => Ok(Json(domains)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

pub async fn create_custom_domain(
    State(state): State<AppState>,
    Json(req): Json<CreateCustomDomainRequest>,
) -> Result<Json<crate::models::CustomDomain>, (StatusCode, String)> {
    let service = CommerceGapsService::new(state.db.clone());
    match service.create_custom_domain(&req).await {
        Ok(domain) => Ok(Json(domain)),
        Err(e) => Err((StatusCode::BAD_REQUEST, e)),
    }
}

pub async fn verify_custom_domain(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<crate::models::CustomDomain>, (StatusCode, String)> {
    let service = CommerceGapsService::new(state.db.clone());
    match service.verify_custom_domain(&id).await {
        Ok(domain) => Ok(Json(domain)),
        Err(e) => Err((StatusCode::BAD_REQUEST, e)),
    }
}

pub async fn delete_custom_domain(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    let service = CommerceGapsService::new(state.db.clone());
    match service.delete_custom_domain(&id).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

// ---------------------------------------------------------------------------
// Gap #25: Storefront Pages
// ---------------------------------------------------------------------------

pub async fn list_storefront_pages(
    State(state): State<AppState>,
    Path(storefront_id): Path<String>,
) -> Result<Json<Vec<crate::models::StorefrontPage>>, (StatusCode, String)> {
    let service = CommerceGapsService::new(state.db.clone());
    match service.list_storefront_pages(&storefront_id).await {
        Ok(pages) => Ok(Json(pages)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

pub async fn create_storefront_page(
    State(state): State<AppState>,
    Json(req): Json<CreateStorefrontPageRequest>,
) -> Result<Json<crate::models::StorefrontPage>, (StatusCode, String)> {
    let service = CommerceGapsService::new(state.db.clone());
    match service.create_storefront_page(&req).await {
        Ok(page) => Ok(Json(page)),
        Err(e) => Err((StatusCode::BAD_REQUEST, e)),
    }
}

pub async fn delete_storefront_page(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    let service = CommerceGapsService::new(state.db.clone());
    match service.delete_storefront_page(&id).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

// ---------------------------------------------------------------------------
// Gap #27: Meta Pixels
// ---------------------------------------------------------------------------

pub async fn list_meta_pixels(
    State(state): State<AppState>,
    Query(query): Query<ListCategoriesQuery>,
) -> Result<Json<Vec<crate::models::MetaPixel>>, (StatusCode, String)> {
    let service = CommerceGapsService::new(state.db.clone());
    match service.list_meta_pixels(&query.brand_id).await {
        Ok(pixels) => Ok(Json(pixels)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

pub async fn create_meta_pixel(
    State(state): State<AppState>,
    Json(req): Json<CreateMetaPixelRequest>,
) -> Result<Json<crate::models::MetaPixel>, (StatusCode, String)> {
    let service = CommerceGapsService::new(state.db.clone());
    match service.create_meta_pixel(&req).await {
        Ok(pixel) => Ok(Json(pixel)),
        Err(e) => Err((StatusCode::BAD_REQUEST, e)),
    }
}

pub async fn delete_meta_pixel(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    let service = CommerceGapsService::new(state.db.clone());
    match service.delete_meta_pixel(&id).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

// ---------------------------------------------------------------------------
// Gap #29: Checkout Questions
// ---------------------------------------------------------------------------

pub async fn list_checkout_questions(
    State(state): State<AppState>,
    Query(query): Query<ListCategoriesQuery>,
) -> Result<Json<Vec<crate::models::CheckoutQuestion>>, (StatusCode, String)> {
    let service = CommerceGapsService::new(state.db.clone());
    match service.list_checkout_questions(&query.brand_id).await {
        Ok(questions) => Ok(Json(questions)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

pub async fn create_checkout_question(
    State(state): State<AppState>,
    Json(req): Json<CreateCheckoutQuestionRequest>,
) -> Result<Json<crate::models::CheckoutQuestion>, (StatusCode, String)> {
    let service = CommerceGapsService::new(state.db.clone());
    match service.create_checkout_question(&req).await {
        Ok(question) => Ok(Json(question)),
        Err(e) => Err((StatusCode::BAD_REQUEST, e)),
    }
}

pub async fn submit_checkout_answers(
    State(state): State<AppState>,
    Json(req): Json<SubmitCheckoutAnswersRequest>,
) -> Result<Json<Vec<crate::models::CheckoutQuestionAnswer>>, (StatusCode, String)> {
    let service = CommerceGapsService::new(state.db.clone());
    match service.submit_checkout_answers(&req).await {
        Ok(answers) => Ok(Json(answers)),
        Err(e) => Err((StatusCode::BAD_REQUEST, e)),
    }
}

pub async fn delete_checkout_question(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    let service = CommerceGapsService::new(state.db.clone());
    match service.delete_checkout_question(&id).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

// ---------------------------------------------------------------------------
// Gap #30: Delivery Proofs
// ---------------------------------------------------------------------------

pub async fn upload_delivery_proof(
    State(state): State<AppState>,
    Json(req): Json<UploadDeliveryProofRequest>,
) -> Result<Json<crate::models::DeliveryProof>, (StatusCode, String)> {
    let service = CommerceGapsService::new(state.db.clone());
    match service.upload_delivery_proof(&req).await {
        Ok(proof) => Ok(Json(proof)),
        Err(e) => Err((StatusCode::BAD_REQUEST, e)),
    }
}

pub async fn list_delivery_proofs(
    State(state): State<AppState>,
    Path(order_id): Path<String>,
) -> Result<Json<Vec<crate::models::DeliveryProof>>, (StatusCode, String)> {
    let service = CommerceGapsService::new(state.db.clone());
    match service.list_delivery_proofs(&order_id).await {
        Ok(proofs) => Ok(Json(proofs)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}
