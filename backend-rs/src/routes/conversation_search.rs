//! Phase 42: Conversation Search & Discovery Routes

use crate::db::AppState;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::{Deserialize, Serialize};
use serde_json::json;

fn err(status: StatusCode, detail: impl Into<String>) -> Response {
    (status, Json(json!({ "detail": detail.into() }))).into_response()
}

fn db_err(e: String) -> Response {
    err(StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {e}"))
}

// ---------------------------------------------------------------------------
// Query params
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    pub q: String,
    #[serde(default)]
    pub search_type: String,
    #[serde(default)]
    pub limit: i32,
}

#[derive(Debug, Deserialize)]
pub struct SavedSearchQuery {
    #[serde(default)]
    pub search_type: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct HistoryQuery {
    #[serde(default = "default_limit")]
    pub limit: i32,
}

fn default_limit() -> i32 { 50 }

#[derive(Debug, Deserialize)]
pub struct CreateSavedSearchPayload {
    pub name: String,
    pub query: String,
    pub search_type: Option<String>,
    pub filters: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct PinPayload {
    pub pinned: bool,
}

#[derive(Debug, Deserialize)]
pub struct UpdateSettingsPayload {
    pub max_results: Option<i32>,
    pub search_message_content: Option<bool>,
    pub search_contact_details: Option<bool>,
    pub search_conversation_metadata: Option<bool>,
    pub track_search_history: Option<bool>,
    pub auto_suggest: Option<bool>,
    pub min_query_length: Option<i32>,
    pub highlight_matches: Option<bool>,
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

pub async fn get_summary(State(state): State<AppState>) -> impl IntoResponse {
    let service = crate::services::ConversationSearchService::new(state.db.clone());
    match service.get_summary().await {
        Ok(summary) => (StatusCode::OK, Json(summary)).into_response(),
        Err(e) => db_err(e.to_string()),
    }
}

pub async fn search(State(state): State<AppState>, Query(params): Query<SearchQuery>) -> impl IntoResponse {
    let service = crate::services::ConversationSearchService::new(state.db.clone());
    let search_type = if params.search_type.is_empty() { "all".to_string() } else { params.search_type };
    let limit = if params.limit <= 0 { 50 } else { params.limit };
    let filters = serde_json::json!({});

    match service.search(&params.q, &search_type, &filters, limit).await {
        Ok(result) => (StatusCode::OK, Json(result)).into_response(),
        Err(e) => db_err(e),
    }
}

pub async fn list_saved_searches(
    State(state): State<AppState>,
    Query(params): Query<SavedSearchQuery>,
) -> impl IntoResponse {
    let service = crate::services::ConversationSearchService::new(state.db.clone());
    match service.list_saved_searches(params.search_type.as_deref()).await {
        Ok(searches) => (StatusCode::OK, Json(searches)).into_response(),
        Err(e) => db_err(e),
    }
}

pub async fn create_saved_search(
    State(state): State<AppState>,
    Json(payload): Json<CreateSavedSearchPayload>,
) -> impl IntoResponse {
    let service = crate::services::ConversationSearchService::new(state.db.clone());
    let req = crate::services::conversation_search::CreateSavedSearchRequest {
        name: payload.name,
        query: payload.query,
        search_type: payload.search_type,
        filters: payload.filters,
    };
    match service.create_saved_search(&req, "system").await {
        Ok(search) => (StatusCode::CREATED, Json(search)).into_response(),
        Err(e) => db_err(e),
    }
}

pub async fn get_saved_search(State(state): State<AppState>, Path(id): Path<String>) -> impl IntoResponse {
    let service = crate::services::ConversationSearchService::new(state.db.clone());
    match service.get_saved_search(&id).await {
        Ok(search) => (StatusCode::OK, Json(search)).into_response(),
        Err(e) => err(StatusCode::NOT_FOUND, e),
    }
}

pub async fn delete_saved_search(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let service = crate::services::ConversationSearchService::new(state.db.clone());
    match service.delete_saved_search(&id).await {
        Ok(()) => (StatusCode::NO_CONTENT, Json(json!({}))).into_response(),
        Err(e) => db_err(e),
    }
}

pub async fn pin_saved_search(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<PinPayload>,
) -> impl IntoResponse {
    let service = crate::services::ConversationSearchService::new(state.db.clone());
    match service.pin_saved_search(&id, payload.pinned).await {
        Ok(()) => (StatusCode::OK, Json(json!({ "success": true }))).into_response(),
        Err(e) => db_err(e),
    }
}

pub async fn use_saved_search(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let service = crate::services::ConversationSearchService::new(state.db.clone());
    match service.use_saved_search(&id).await {
        Ok(result) => (StatusCode::OK, Json(result)).into_response(),
        Err(e) => db_err(e),
    }
}

pub async fn get_search_history(
    State(state): State<AppState>,
    Query(params): Query<HistoryQuery>,
) -> impl IntoResponse {
    let service = crate::services::ConversationSearchService::new(state.db.clone());
    match service.get_search_history(params.limit).await {
        Ok(history) => (StatusCode::OK, Json(history)).into_response(),
        Err(e) => db_err(e),
    }
}

pub async fn clear_search_history(State(state): State<AppState>) -> impl IntoResponse {
    let service = crate::services::ConversationSearchService::new(state.db.clone());
    match service.clear_search_history().await {
        Ok(()) => (StatusCode::OK, Json(json!({ "success": true }))).into_response(),
        Err(e) => db_err(e),
    }
}

pub async fn get_analytics(State(state): State<AppState>) -> impl IntoResponse {
    let service = crate::services::ConversationSearchService::new(state.db.clone());
    match service.get_analytics().await {
        Ok(analytics) => (StatusCode::OK, Json(analytics)).into_response(),
        Err(e) => db_err(e),
    }
}

pub async fn generate_analytics_snapshot(State(state): State<AppState>) -> impl IntoResponse {
    let service = crate::services::ConversationSearchService::new(state.db.clone());
    match service.generate_analytics_snapshot().await {
        Ok(snapshot) => (StatusCode::OK, Json(snapshot)).into_response(),
        Err(e) => db_err(e),
    }
}

pub async fn get_settings(State(state): State<AppState>) -> impl IntoResponse {
    let service = crate::services::ConversationSearchService::new(state.db.clone());
    match service.get_settings().await {
        Ok(settings) => (StatusCode::OK, Json(settings)).into_response(),
        Err(e) => db_err(e),
    }
}

pub async fn update_settings(
    State(state): State<AppState>,
    Json(payload): Json<UpdateSettingsPayload>,
) -> impl IntoResponse {
    let service = crate::services::ConversationSearchService::new(state.db.clone());
    
    // Get current settings and merge with payload
    let current = match service.get_settings().await {
        Ok(s) => s,
        Err(e) => return db_err(e),
    };

    let updated = crate::services::conversation_search::SearchSettings {
        id: current.id.clone(),
        org_id: current.org_id.clone(),
        max_results: payload.max_results.unwrap_or(current.max_results),
        search_message_content: payload.search_message_content.unwrap_or(current.search_message_content),
        search_contact_details: payload.search_contact_details.unwrap_or(current.search_contact_details),
        search_conversation_metadata: payload.search_conversation_metadata.unwrap_or(current.search_conversation_metadata),
        track_search_history: payload.track_search_history.unwrap_or(current.track_search_history),
        auto_suggest: payload.auto_suggest.unwrap_or(current.auto_suggest),
        min_query_length: payload.min_query_length.unwrap_or(current.min_query_length),
        highlight_matches: payload.highlight_matches.unwrap_or(current.highlight_matches),
        created_at: current.created_at,
        updated_at: current.updated_at,
    };

    match service.update_settings(&updated).await {
        Ok(settings) => (StatusCode::OK, Json(settings)).into_response(),
        Err(e) => db_err(e),
    }
}
