//! Knowledge Base Document Upload & URL Crawling Routes (Phase 23).
//!
//! This handler no longer runs the logic in-process -- it's a thin gRPC
//! client calling `svc-kb-documents` (see services/kb-documents and
//! crates/kb-documents at the repo root). This module is the first one
//! split out of the monolith per CONVENTIONS.md; the REST surface here
//! is unchanged on purpose so `apps/web` needs no changes.

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use newax_proto::kb_documents::v1::{
    kb_documents_service_client::KbDocumentsServiceClient, AddUrlSourceRequest, DocumentIdRequest,
    GetSummaryRequest, ListDocumentsRequest, ListUrlSourcesRequest, OrgRequest, SourceIdRequest,
    UpdateSettingsRequest, UploadDocumentRequest,
};
use serde::Deserialize;
use serde_json::json;
use tonic::transport::Channel;

use crate::db::AppState;

fn err(status: StatusCode, detail: impl Into<String>) -> Response {
    (status, Json(json!({ "detail": detail.into() }))).into_response()
}

fn grpc_err(status: tonic::Status) -> Response {
    let code = match status.code() {
        tonic::Code::InvalidArgument => StatusCode::BAD_REQUEST,
        tonic::Code::NotFound => StatusCode::NOT_FOUND,
        _ => StatusCode::INTERNAL_SERVER_ERROR,
    };
    err(code, status.message().to_string())
}

/// One lazily-connected channel per request is wasteful but simple, and
/// correct for a pilot -- a shared, pooled channel belongs in AppState
/// once more than one route depends on this pattern.
async fn client() -> Result<KbDocumentsServiceClient<Channel>, Response> {
    let url = std::env::var("KB_DOCUMENTS_SERVICE_URL")
        .unwrap_or_else(|_| "http://127.0.0.1:50061".to_string());
    KbDocumentsServiceClient::connect(url)
        .await
        .map_err(|e| err(StatusCode::SERVICE_UNAVAILABLE, format!("kb-documents service unreachable: {e}")))
}

fn json_value(json_str: String) -> serde_json::Value {
    serde_json::from_str(&json_str).unwrap_or(serde_json::Value::Null)
}

// ——— Query Params ———

#[derive(Deserialize)]
pub struct ListDocumentsQuery {
    organization_id: Option<String>,
    status: Option<String>,
}

#[derive(Deserialize)]
pub struct ListURLSourcesQuery {
    organization_id: Option<String>,
    status: Option<String>,
}

// ——— Handlers ———

/// GET /kb-import/summary
pub async fn get_summary(State(_state): State<AppState>) -> Response {
    let mut c = match client().await {
        Ok(c) => c,
        Err(r) => return r,
    };
    match c.get_summary(GetSummaryRequest {}).await {
        Ok(r) => (StatusCode::OK, Json(json_value(r.into_inner().json))).into_response(),
        Err(e) => grpc_err(e),
    }
}

/// POST /kb-import/upload
pub async fn upload_document_handler(
    State(_state): State<AppState>,
    Json(body): Json<serde_json::Value>,
) -> Response {
    let organization_id = body.get("organization_id").and_then(|v| v.as_str()).unwrap_or("org-default");
    let file_name = body.get("file_name").and_then(|v| v.as_str()).unwrap_or("unknown.txt");
    let file_type = body.get("file_type").and_then(|v| v.as_str()).unwrap_or("txt");
    let file_data_b64 = body.get("file_data").and_then(|v| v.as_str()).unwrap_or("");

    let file_data = match base64::decode(file_data_b64) {
        Ok(data) => data,
        Err(e) => return err(StatusCode::BAD_REQUEST, format!("Invalid base64 data: {}", e)),
    };

    let mut c = match client().await {
        Ok(c) => c,
        Err(r) => return r,
    };
    let req = UploadDocumentRequest {
        organization_id: organization_id.to_string(),
        file_name: file_name.to_string(),
        file_type: file_type.to_string(),
        file_data,
    };
    match c.upload_document(req).await {
        Ok(r) => (StatusCode::OK, Json(json_value(r.into_inner().json))).into_response(),
        Err(e) => grpc_err(e),
    }
}

/// GET /kb-import/documents
pub async fn list_documents_handler(
    State(_state): State<AppState>,
    Query(params): Query<ListDocumentsQuery>,
) -> Response {
    let mut c = match client().await {
        Ok(c) => c,
        Err(r) => return r,
    };
    let req = ListDocumentsRequest { organization_id: params.organization_id, status: params.status };
    match c.list_documents(req).await {
        Ok(r) => (StatusCode::OK, Json(json_value(r.into_inner().json))).into_response(),
        Err(e) => grpc_err(e),
    }
}

/// GET /kb-import/documents/{id}/chunks
pub async fn get_document_chunks_handler(State(_state): State<AppState>, Path(doc_id): Path<String>) -> Response {
    let mut c = match client().await {
        Ok(c) => c,
        Err(r) => return r,
    };
    match c.get_document_chunks(DocumentIdRequest { doc_id }).await {
        Ok(r) => (StatusCode::OK, Json(json_value(r.into_inner().json))).into_response(),
        Err(e) => grpc_err(e),
    }
}

/// DELETE /kb-import/documents/{id}
pub async fn delete_document_handler(State(_state): State<AppState>, Path(doc_id): Path<String>) -> Response {
    let mut c = match client().await {
        Ok(c) => c,
        Err(r) => return r,
    };
    match c.delete_document(DocumentIdRequest { doc_id }).await {
        Ok(r) => (StatusCode::OK, Json(json!({ "deleted": r.into_inner().deleted }))).into_response(),
        Err(e) => grpc_err(e),
    }
}

/// POST /kb-import/urls
pub async fn add_url_source_handler(
    State(_state): State<AppState>,
    Json(body): Json<serde_json::Value>,
) -> Response {
    let organization_id = body.get("organization_id").and_then(|v| v.as_str()).unwrap_or("org-default");
    let url = body.get("url").and_then(|v| v.as_str()).unwrap_or("");
    let crawl_interval_hours = body.get("crawl_interval_hours").and_then(|v| v.as_i64()).map(|v| v as i32);

    let mut c = match client().await {
        Ok(c) => c,
        Err(r) => return r,
    };
    let req = AddUrlSourceRequest {
        organization_id: organization_id.to_string(),
        url: url.to_string(),
        crawl_interval_hours,
    };
    match c.add_url_source(req).await {
        Ok(r) => (StatusCode::OK, Json(json_value(r.into_inner().json))).into_response(),
        Err(e) => grpc_err(e),
    }
}

/// POST /kb-import/urls/{id}/crawl
pub async fn crawl_url_handler(State(_state): State<AppState>, Path(source_id): Path<String>) -> Response {
    let mut c = match client().await {
        Ok(c) => c,
        Err(r) => return r,
    };
    match c.crawl_url(SourceIdRequest { source_id }).await {
        Ok(r) => (StatusCode::OK, Json(json_value(r.into_inner().json))).into_response(),
        Err(e) => grpc_err(e),
    }
}

/// GET /kb-import/urls
pub async fn list_url_sources_handler(
    State(_state): State<AppState>,
    Query(params): Query<ListURLSourcesQuery>,
) -> Response {
    let mut c = match client().await {
        Ok(c) => c,
        Err(r) => return r,
    };
    let req = ListUrlSourcesRequest { organization_id: params.organization_id, status: params.status };
    match c.list_url_sources(req).await {
        Ok(r) => (StatusCode::OK, Json(json_value(r.into_inner().json))).into_response(),
        Err(e) => grpc_err(e),
    }
}

/// DELETE /kb-import/urls/{id}
pub async fn delete_url_source_handler(State(_state): State<AppState>, Path(source_id): Path<String>) -> Response {
    let mut c = match client().await {
        Ok(c) => c,
        Err(r) => return r,
    };
    match c.delete_url_source(SourceIdRequest { source_id }).await {
        Ok(r) => (StatusCode::OK, Json(json!({ "deleted": r.into_inner().deleted }))).into_response(),
        Err(e) => grpc_err(e),
    }
}

/// GET /kb-import/settings
pub async fn get_settings_handler(
    State(_state): State<AppState>,
    Query(params): Query<ListDocumentsQuery>,
) -> Response {
    let organization_id = params.organization_id.unwrap_or_else(|| "org-default".to_string());
    let mut c = match client().await {
        Ok(c) => c,
        Err(r) => return r,
    };
    match c.get_settings(OrgRequest { organization_id }).await {
        Ok(r) => (StatusCode::OK, Json(json_value(r.into_inner().json))).into_response(),
        Err(e) => grpc_err(e),
    }
}

/// PUT /kb-import/settings
pub async fn update_settings_handler(
    State(_state): State<AppState>,
    Json(body): Json<serde_json::Value>,
) -> Response {
    let organization_id = body.get("organization_id").and_then(|v| v.as_str()).unwrap_or("org-default");
    let empty_settings = serde_json::json!({});
    let settings = body.get("settings").unwrap_or(&empty_settings);

    let mut c = match client().await {
        Ok(c) => c,
        Err(r) => return r,
    };
    let req = UpdateSettingsRequest {
        organization_id: organization_id.to_string(),
        settings_json: settings.to_string(),
    };
    match c.update_settings(req).await {
        Ok(r) => (StatusCode::OK, Json(json_value(r.into_inner().json))).into_response(),
        Err(e) => grpc_err(e),
    }
}
