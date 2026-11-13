//! Knowledge Base Document Upload & URL Crawling Routes (Phase 23).

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::db::AppState;
use crate::services::kb_documents;

fn err(status: StatusCode, detail: impl Into<String>) -> Response {
    (status, Json(json!({ "detail": detail.into() }))).into_response()
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

#[derive(Deserialize)]
struct AddURLSourceQuery {
    url: String,
    crawl_interval_hours: Option<i32>,
}

// ——— Handlers ———

/// GET /kb-import/summary
pub async fn get_summary(State(state): State<AppState>) -> Response {
    match kb_documents::get_kb_import_summary(&state).await {
        Ok(summary) => (StatusCode::OK, Json(summary)).into_response(),
        Err(e) => err(StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to get summary: {}", e)),
    }
}

/// POST /kb-import/upload
pub async fn upload_document_handler(
    State(state): State<AppState>,
    Json(body): Json<serde_json::Value>,
) -> Response {
    let organization_id = body.get("organization_id")
        .and_then(|v| v.as_str())
        .unwrap_or("org-default");
    let file_name = body.get("file_name")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown.txt");
    let file_type = body.get("file_type")
        .and_then(|v| v.as_str())
        .unwrap_or("txt");
    let file_data_b64 = body.get("file_data")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    // Decode base64 file data
    let file_data = match base64::decode(file_data_b64) {
        Ok(data) => data,
        Err(e) => return err(StatusCode::BAD_REQUEST, format!("Invalid base64 data: {}", e)),
    };

    match kb_documents::upload_document(&state, organization_id, file_name, file_type, &file_data).await {
        Ok(result) => (StatusCode::OK, Json(result)).into_response(),
        Err(e) => err(StatusCode::BAD_REQUEST, e),
    }
}

/// GET /kb-import/documents
pub async fn list_documents_handler(
    State(state): State<AppState>,
    Query(params): Query<ListDocumentsQuery>,
) -> Response {
    let org = params.organization_id.as_deref();
    let status = params.status.as_deref();

    match kb_documents::list_documents(&state, org, status).await {
        Ok(docs) => (StatusCode::OK, Json(docs)).into_response(),
        Err(e) => err(StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to list documents: {}", e)),
    }
}

/// GET /kb-import/documents/{id}/chunks
pub async fn get_document_chunks_handler(
    State(state): State<AppState>,
    Path(doc_id): Path<String>,
) -> Response {
    match kb_documents::get_document_chunks(&state, &doc_id).await {
        Ok(chunks) => (StatusCode::OK, Json(chunks)).into_response(),
        Err(e) => err(StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to get chunks: {}", e)),
    }
}

/// DELETE /kb-import/documents/{id}
pub async fn delete_document_handler(
    State(state): State<AppState>,
    Path(doc_id): Path<String>,
) -> Response {
    match kb_documents::delete_document(&state, &doc_id).await {
        Ok(()) => (StatusCode::OK, Json(json!({ "deleted": true }))).into_response(),
        Err(e) => err(StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to delete document: {}", e)),
    }
}

/// POST /kb-import/urls
pub async fn add_url_source_handler(
    State(state): State<AppState>,
    Json(body): Json<serde_json::Value>,
) -> Response {
    let organization_id = body.get("organization_id")
        .and_then(|v| v.as_str())
        .unwrap_or("org-default");
    let url = body.get("url")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let crawl_interval_hours = body.get("crawl_interval_hours")
        .and_then(|v| v.as_i64())
        .map(|v| v as i32);

    match kb_documents::add_url_source(&state, organization_id, url, crawl_interval_hours).await {
        Ok(source) => (StatusCode::OK, Json(source)).into_response(),
        Err(e) => err(StatusCode::BAD_REQUEST, e),
    }
}

/// POST /kb-import/urls/{id}/crawl
pub async fn crawl_url_handler(
    State(state): State<AppState>,
    Path(source_id): Path<String>,
) -> Response {
    match kb_documents::crawl_url(&state, &source_id).await {
        Ok(result) => (StatusCode::OK, Json(result)).into_response(),
        Err(e) => err(StatusCode::BAD_REQUEST, e),
    }
}

/// GET /kb-import/urls
pub async fn list_url_sources_handler(
    State(state): State<AppState>,
    Query(params): Query<ListURLSourcesQuery>,
) -> Response {
    let org = params.organization_id.as_deref();
    let status = params.status.as_deref();

    match kb_documents::list_url_sources(&state, org, status).await {
        Ok(sources) => (StatusCode::OK, Json(sources)).into_response(),
        Err(e) => err(StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to list URL sources: {}", e)),
    }
}

/// DELETE /kb-import/urls/{id}
pub async fn delete_url_source_handler(
    State(state): State<AppState>,
    Path(source_id): Path<String>,
) -> Response {
    match kb_documents::delete_url_source(&state, &source_id).await {
        Ok(()) => (StatusCode::OK, Json(json!({ "deleted": true }))).into_response(),
        Err(e) => err(StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to delete URL source: {}", e)),
    }
}

/// GET /kb-import/settings
pub async fn get_settings_handler(
    State(state): State<AppState>,
    Query(params): Query<ListDocumentsQuery>,
) -> Response {
    let org = params.organization_id.as_deref().unwrap_or("org-default");

    match kb_documents::get_settings(&state, org).await {
        Ok(settings) => (StatusCode::OK, Json(settings)).into_response(),
        Err(e) => err(StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to get settings: {}", e)),
    }
}

/// PUT /kb-import/settings
pub async fn update_settings_handler(
    State(state): State<AppState>,
    Json(body): Json<serde_json::Value>,
) -> Response {
    let organization_id = body.get("organization_id")
        .and_then(|v| v.as_str())
        .unwrap_or("org-default");
    let empty_settings = serde_json::json!({});
    let settings = body.get("settings").unwrap_or(&empty_settings);

    match kb_documents::update_settings(&state, organization_id, settings).await {
        Ok(settings) => (StatusCode::OK, Json(settings)).into_response(),
        Err(e) => err(StatusCode::BAD_REQUEST, e),
    }
}
