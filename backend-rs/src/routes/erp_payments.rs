//! ERP Payments Routes
//!
//! Thin gRPC client forwarding to svc-erp-payments; REST shape unchanged.

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;
use std::collections::HashMap;

use crate::db::AppState;

use newax_proto::erp_payments::v1::{
    erp_payments_service_client::ErpPaymentsServiceClient,
    Empty, IdRequest, ListPaymentsRequest, CreatePaymentRequest,
    UpdatePaymentStatusRequest, MarkAsPaidRequest, MarkAsFailedRequest,
    RefundPaymentRequest, UploadReceiptRequest, ApprovePaymentRequest,
    RejectPaymentRequest, PendingApprovalsRequest, CreatePaymentMethodRequest,
    UpdatePaymentMethodRequest,
};

async fn client() -> Result<ErpPaymentsServiceClient<tonic::transport::Channel>, (StatusCode, String)> {
    let url = std::env::var("ERP_PAYMENTS_SERVICE_URL").unwrap_or_else(|_| "http://127.0.0.1:50096".to_string());
    ErpPaymentsServiceClient::connect(url)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("erp-payments service unavailable: {e}")))
}

fn grpc_err(e: tonic::Status) -> (StatusCode, String) {
    (StatusCode::INTERNAL_SERVER_ERROR, e.message().to_string())
}

// ——— Query Params ———

#[derive(Deserialize)]
pub struct ListPaymentsQuery {
    pub orderId: Option<String>,
    pub status: Option<String>,
    pub provider: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Deserialize)]
pub struct PendingApprovalsQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

// ——— Handlers ———

pub async fn list_payments(
    State(_state): State<AppState>,
    Query(params): Query<ListPaymentsQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut cli = client().await.map_err(|e| (e.0, e.1))?;
    let resp = cli.list_payments(ListPaymentsRequest {
        order_id: params.orderId,
        status: params.status,
        provider: params.provider,
        limit: params.limit.unwrap_or(50),
        offset: params.offset.unwrap_or(0),
    }).await.map_err(grpc_err)?;
    let val: serde_json::Value = serde_json::from_str(&resp.into_inner().json).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(val))
}

pub async fn get_payment(
    State(_state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut cli = client().await.map_err(|e| (e.0, e.1))?;
    let resp = cli.get_payment(IdRequest { id }).await.map_err(grpc_err)?;
    let val: serde_json::Value = serde_json::from_str(&resp.into_inner().json).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(val))
}

pub async fn get_payments_by_order(
    State(_state): State<AppState>,
    Path(order_id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut cli = client().await.map_err(|e| (e.0, e.1))?;
    let resp = cli.get_payments_by_order(IdRequest { id: order_id }).await.map_err(grpc_err)?;
    let val: serde_json::Value = serde_json::from_str(&resp.into_inner().json).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(val))
}

pub async fn get_payment_by_reference(
    State(_state): State<AppState>,
    Path(reference): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut cli = client().await.map_err(|e| (e.0, e.1))?;
    let resp = cli.get_payment_by_reference(IdRequest { id: reference }).await.map_err(grpc_err)?;
    let val: serde_json::Value = serde_json::from_str(&resp.into_inner().json).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(val))
}

pub async fn create_payment(
    State(_state): State<AppState>,
    Json(req): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let order_id = req.get("orderId").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let provider = req.get("provider").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let amount = req.get("amount").and_then(|v| v.as_i64()).unwrap_or(0);
    let currency = req.get("currency").and_then(|v| v.as_str()).map(|s| s.to_string());
    let reference = req.get("reference").and_then(|v| v.as_str()).map(|s| s.to_string());
    let meta_json = req.get("meta").map(|v| v.to_string());

    let mut cli = client().await.map_err(|e| (e.0, e.1))?;
    let resp = cli.create_payment(CreatePaymentRequest {
        order_id, provider, amount, currency, reference,
        meta_json,
    }).await.map_err(grpc_err)?;
    let val: serde_json::Value = serde_json::from_str(&resp.into_inner().json).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(val))
}

pub async fn update_payment_status(
    State(_state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let status = req.get("status").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let provider_ref = req.get("providerRef").and_then(|v| v.as_str()).map(|s| s.to_string());
    let receipt_url = req.get("receiptUrl").and_then(|v| v.as_str()).map(|s| s.to_string());
    let failure_reason = req.get("failureReason").and_then(|v| v.as_str()).map(|s| s.to_string());
    let approved_by_id = req.get("approvedById").and_then(|v| v.as_str()).map(|s| s.to_string());
    let meta_json = req.get("meta").map(|v| v.to_string());

    let mut cli = client().await.map_err(|e| (e.0, e.1))?;
    let resp = cli.update_payment_status(UpdatePaymentStatusRequest {
        id, status, provider_ref, receipt_url, failure_reason, approved_by_id,
        meta_json,
    }).await.map_err(grpc_err)?;
    let val: serde_json::Value = serde_json::from_str(&resp.into_inner().json).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(val))
}

pub async fn mark_as_paid(
    State(_state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let provider_ref = req.get("providerRef").and_then(|v| v.as_str()).map(|s| s.to_string());
    let approved_by_id = req.get("approvedById").and_then(|v| v.as_str()).map(|s| s.to_string());

    let mut cli = client().await.map_err(|e| (e.0, e.1))?;
    let resp = cli.mark_as_paid(MarkAsPaidRequest { id, provider_ref, approved_by_id }).await.map_err(grpc_err)?;
    let val: serde_json::Value = serde_json::from_str(&resp.into_inner().json).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(val))
}

pub async fn mark_as_failed(
    State(_state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let reason = req.get("reason").and_then(|v| v.as_str()).unwrap_or("Payment failed").to_string();

    let mut cli = client().await.map_err(|e| (e.0, e.1))?;
    let resp = cli.mark_as_failed(MarkAsFailedRequest { id, reason }).await.map_err(grpc_err)?;
    let val: serde_json::Value = serde_json::from_str(&resp.into_inner().json).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(val))
}

pub async fn refund_payment(
    State(_state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let refund_amount = req.get("refundAmount").and_then(|v| v.as_i64()).unwrap_or(0);
    let reason = req.get("reason").and_then(|v| v.as_str()).map(|s| s.to_string());

    let mut cli = client().await.map_err(|e| (e.0, e.1))?;
    let resp = cli.refund_payment(RefundPaymentRequest { id, refund_amount, reason }).await.map_err(grpc_err)?;
    let val: serde_json::Value = serde_json::from_str(&resp.into_inner().json).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(val))
}

pub async fn upload_receipt(
    State(_state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let receipt_url = req.get("receiptUrl").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let meta_json = req.get("meta").map(|v| v.to_string());

    let mut cli = client().await.map_err(|e| (e.0, e.1))?;
    let resp = cli.upload_receipt(UploadReceiptRequest {
        id, receipt_url, meta_json,
    }).await.map_err(grpc_err)?;
    let val: serde_json::Value = serde_json::from_str(&resp.into_inner().json).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(val))
}

pub async fn approve_payment(
    State(_state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let approved_by_id = req.get("approvedById").and_then(|v| v.as_str()).unwrap_or("system").to_string();

    let mut cli = client().await.map_err(|e| (e.0, e.1))?;
    let resp = cli.approve_payment(ApprovePaymentRequest { id, approved_by_id }).await.map_err(grpc_err)?;
    let val: serde_json::Value = serde_json::from_str(&resp.into_inner().json).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(val))
}

pub async fn reject_payment(
    State(_state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let reason = req.get("reason").and_then(|v| v.as_str()).unwrap_or("Receipt rejected").to_string();

    let mut cli = client().await.map_err(|e| (e.0, e.1))?;
    let resp = cli.reject_payment(RejectPaymentRequest { id, reason }).await.map_err(grpc_err)?;
    let val: serde_json::Value = serde_json::from_str(&resp.into_inner().json).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(val))
}

pub async fn get_pending_approvals(
    State(_state): State<AppState>,
    Query(params): Query<PendingApprovalsQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut cli = client().await.map_err(|e| (e.0, e.1))?;
    let resp = cli.get_pending_approvals(PendingApprovalsRequest {
        limit: params.limit.unwrap_or(50),
        offset: params.offset.unwrap_or(0),
    }).await.map_err(grpc_err)?;
    let val: serde_json::Value = serde_json::from_str(&resp.into_inner().json).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(val))
}

pub async fn get_payment_stats(
    State(_state): State<AppState>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut cli = client().await.map_err(|e| (e.0, e.1))?;
    let resp = cli.get_payment_stats(Empty {}).await.map_err(grpc_err)?;
    let val: serde_json::Value = serde_json::from_str(&resp.into_inner().json).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(val))
}

// ——— Payment Methods ———

pub async fn list_payment_methods(
    State(_state): State<AppState>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut cli = client().await.map_err(|e| (e.0, e.1))?;
    let resp = cli.list_payment_methods(Empty {}).await.map_err(grpc_err)?;
    let val: serde_json::Value = serde_json::from_str(&resp.into_inner().json).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(val))
}

pub async fn get_payment_method(
    State(_state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut cli = client().await.map_err(|e| (e.0, e.1))?;
    let resp = cli.get_payment_method(IdRequest { id }).await.map_err(grpc_err)?;
    let val: serde_json::Value = serde_json::from_str(&resp.into_inner().json).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(val))
}

pub async fn get_payment_method_by_code(
    State(_state): State<AppState>,
    Path(code): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut cli = client().await.map_err(|e| (e.0, e.1))?;
    let resp = cli.get_payment_method_by_code(IdRequest { id: code }).await.map_err(grpc_err)?;
    let val: serde_json::Value = serde_json::from_str(&resp.into_inner().json).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(val))
}

pub async fn create_payment_method(
    State(_state): State<AppState>,
    Json(req): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let name = req.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let code = req.get("code").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let config_json = req.get("config").map(|v| v.to_string());

    let mut cli = client().await.map_err(|e| (e.0, e.1))?;
    let resp = cli.create_payment_method(CreatePaymentMethodRequest {
        name, code, config_json,
    }).await.map_err(grpc_err)?;
    let val: serde_json::Value = serde_json::from_str(&resp.into_inner().json).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(val))
}

pub async fn update_payment_method(
    State(_state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let name = req.get("name").and_then(|v| v.as_str()).map(|s| s.to_string());
    let config_json = req.get("config").map(|v| v.to_string());
    let is_active = req.get("isActive").and_then(|v| v.as_bool());

    let mut cli = client().await.map_err(|e| (e.0, e.1))?;
    let resp = cli.update_payment_method(UpdatePaymentMethodRequest {
        id, name, config_json, is_active,
    }).await.map_err(grpc_err)?;
    let val: serde_json::Value = serde_json::from_str(&resp.into_inner().json).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(val))
}

pub async fn delete_payment_method(
    State(_state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    let mut cli = client().await.map_err(|e| (e.0, e.1))?;
    cli.delete_payment_method(IdRequest { id }).await.map_err(grpc_err)?;
    Ok(StatusCode::NO_CONTENT)
}
