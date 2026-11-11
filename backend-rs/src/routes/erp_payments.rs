use axum::{Json, extract::{Path, Query, State}, http::StatusCode};
use std::collections::HashMap;
use crate::db::DbPool;
use crate::AppState;
use crate::models::*;
use crate::services::erp_payments::ErpPaymentsService;

// ============================================================
// Payments
// ============================================================

pub async fn list_payments(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Vec<ErpPayment>>, (StatusCode, String)> {
    let pool: &DbPool = &state.db;
    let order_id = params.get("orderId").map(|s| s.as_str());
    let status = params.get("status").map(|s| s.as_str());
    let provider = params.get("provider").map(|s| s.as_str());
    let limit: i64 = params.get("limit").and_then(|v| v.parse().ok()).unwrap_or(50);
    let offset: i64 = params.get("offset").and_then(|v| v.parse().ok()).unwrap_or(0);

    match ErpPaymentsService::list_payments(pool, order_id, status, provider, limit, offset).await {
        Ok(payments) => Ok(Json(payments)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

pub async fn get_payment(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ErpPayment>, (StatusCode, String)> {
    let pool: &DbPool = &state.db;
    match ErpPaymentsService::get_payment(pool, &id).await {
        Ok(payment) => Ok(Json(payment)),
        Err(e) => Err((StatusCode::NOT_FOUND, e)),
    }
}

pub async fn get_payments_by_order(
    State(state): State<AppState>,
    Path(order_id): Path<String>,
) -> Result<Json<Vec<ErpPayment>>, (StatusCode, String)> {
    let pool: &DbPool = &state.db;
    match ErpPaymentsService::get_payments_by_order(pool, &order_id).await {
        Ok(payments) => Ok(Json(payments)),
        Err(e) => Err((StatusCode::NOT_FOUND, e)),
    }
}

pub async fn get_payment_by_reference(
    State(state): State<AppState>,
    Path(reference): Path<String>,
) -> Result<Json<ErpPayment>, (StatusCode, String)> {
    let pool: &DbPool = &state.db;
    match ErpPaymentsService::get_payment_by_reference(pool, &reference).await {
        Ok(payment) => Ok(Json(payment)),
        Err(e) => Err((StatusCode::NOT_FOUND, e)),
    }
}

pub async fn create_payment(
    State(state): State<AppState>,
    Json(req): Json<CreatePaymentRequest>,
) -> Result<Json<ErpPayment>, (StatusCode, String)> {
    let pool: &DbPool = &state.db;
    match ErpPaymentsService::create_payment(pool, &req).await {
        Ok(payment) => Ok(Json(payment)),
        Err(e) => Err((StatusCode::BAD_REQUEST, e)),
    }
}

pub async fn update_payment_status(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<UpdatePaymentStatusRequest>,
) -> Result<Json<ErpPayment>, (StatusCode, String)> {
    let pool: &DbPool = &state.db;
    match ErpPaymentsService::update_payment_status(pool, &id, &req).await {
        Ok(payment) => Ok(Json(payment)),
        Err(e) => Err((StatusCode::BAD_REQUEST, e)),
    }
}

pub async fn mark_as_paid(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<serde_json::Value>,
) -> Result<Json<ErpPayment>, (StatusCode, String)> {
    let pool: &DbPool = &state.db;
    let provider_ref = req.get("providerRef").and_then(|v| v.as_str());
    let approved_by_id = req.get("approvedById").and_then(|v| v.as_str());

    match ErpPaymentsService::mark_as_paid(pool, &id, provider_ref, approved_by_id).await {
        Ok(payment) => Ok(Json(payment)),
        Err(e) => Err((StatusCode::BAD_REQUEST, e)),
    }
}

pub async fn mark_as_failed(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<serde_json::Value>,
) -> Result<Json<ErpPayment>, (StatusCode, String)> {
    let pool: &DbPool = &state.db;
    let reason = req.get("reason").and_then(|v| v.as_str()).unwrap_or("Payment failed");

    match ErpPaymentsService::mark_as_failed(pool, &id, reason).await {
        Ok(payment) => Ok(Json(payment)),
        Err(e) => Err((StatusCode::BAD_REQUEST, e)),
    }
}

pub async fn refund_payment(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<RefundPaymentRequest>,
) -> Result<Json<ErpPayment>, (StatusCode, String)> {
    let pool: &DbPool = &state.db;
    match ErpPaymentsService::refund_payment(pool, &id, &req).await {
        Ok(payment) => Ok(Json(payment)),
        Err(e) => Err((StatusCode::BAD_REQUEST, e)),
    }
}

pub async fn upload_receipt(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<UploadReceiptRequest>,
) -> Result<Json<ErpPayment>, (StatusCode, String)> {
    let pool: &DbPool = &state.db;
    match ErpPaymentsService::upload_receipt(pool, &id, &req).await {
        Ok(payment) => Ok(Json(payment)),
        Err(e) => Err((StatusCode::BAD_REQUEST, e)),
    }
}

pub async fn approve_payment(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<serde_json::Value>,
) -> Result<Json<ErpPayment>, (StatusCode, String)> {
    let pool: &DbPool = &state.db;
    let approved_by_id = req.get("approvedById").and_then(|v| v.as_str()).unwrap_or("system");

    match ErpPaymentsService::approve_payment(pool, &id, approved_by_id).await {
        Ok(payment) => Ok(Json(payment)),
        Err(e) => Err((StatusCode::BAD_REQUEST, e)),
    }
}

pub async fn reject_payment(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<serde_json::Value>,
) -> Result<Json<ErpPayment>, (StatusCode, String)> {
    let pool: &DbPool = &state.db;
    let reason = req.get("reason").and_then(|v| v.as_str()).unwrap_or("Receipt rejected");

    match ErpPaymentsService::reject_payment(pool, &id, reason).await {
        Ok(payment) => Ok(Json(payment)),
        Err(e) => Err((StatusCode::BAD_REQUEST, e)),
    }
}

pub async fn get_pending_approvals(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Vec<ErpPayment>>, (StatusCode, String)> {
    let pool: &DbPool = &state.db;
    let limit: i64 = params.get("limit").and_then(|v| v.parse().ok()).unwrap_or(50);
    let offset: i64 = params.get("offset").and_then(|v| v.parse().ok()).unwrap_or(0);

    match ErpPaymentsService::get_pending_approvals(pool, limit, offset).await {
        Ok(payments) => Ok(Json(payments)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

pub async fn get_payment_stats(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let pool: &DbPool = &state.db;
    match ErpPaymentsService::get_payment_stats(pool).await {
        Ok(stats) => Ok(Json(stats)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

// ============================================================
// Payment Methods
// ============================================================

pub async fn list_payment_methods(
    State(state): State<AppState>,
) -> Result<Json<Vec<ErpPaymentMethod>>, (StatusCode, String)> {
    let pool: &DbPool = &state.db;
    match ErpPaymentsService::list_payment_methods(pool).await {
        Ok(methods) => Ok(Json(methods)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

pub async fn get_payment_method(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ErpPaymentMethod>, (StatusCode, String)> {
    let pool: &DbPool = &state.db;
    match ErpPaymentsService::get_payment_method(pool, &id).await {
        Ok(method) => Ok(Json(method)),
        Err(e) => Err((StatusCode::NOT_FOUND, e)),
    }
}

pub async fn get_payment_method_by_code(
    State(state): State<AppState>,
    Path(code): Path<String>,
) -> Result<Json<ErpPaymentMethod>, (StatusCode, String)> {
    let pool: &DbPool = &state.db;
    match ErpPaymentsService::get_payment_method_by_code(pool, &code).await {
        Ok(method) => Ok(Json(method)),
        Err(e) => Err((StatusCode::NOT_FOUND, e)),
    }
}

pub async fn create_payment_method(
    State(state): State<AppState>,
    Json(req): Json<CreatePaymentMethodRequest>,
) -> Result<Json<ErpPaymentMethod>, (StatusCode, String)> {
    let pool: &DbPool = &state.db;
    match ErpPaymentsService::create_payment_method(pool, &req).await {
        Ok(method) => Ok(Json(method)),
        Err(e) => Err((StatusCode::BAD_REQUEST, e)),
    }
}

pub async fn update_payment_method(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<UpdatePaymentMethodRequest>,
) -> Result<Json<ErpPaymentMethod>, (StatusCode, String)> {
    let pool: &DbPool = &state.db;
    match ErpPaymentsService::update_payment_method(pool, &id, &req).await {
        Ok(method) => Ok(Json(method)),
        Err(e) => Err((StatusCode::BAD_REQUEST, e)),
    }
}

pub async fn delete_payment_method(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    let pool: &DbPool = &state.db;
    match ErpPaymentsService::delete_payment_method(pool, &id).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}
