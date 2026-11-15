//! Phase 42: Conversation Search & Discovery Routes
//! Thin gRPC client forwarding to svc-conversation-search; REST shape unchanged.

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::{Deserialize, Serialize};
use serde_json::json;

use newax_proto::conversation_search::v1::{
    conversation_search_service_client::ConversationSearchServiceClient, CreateSavedSearchRequest as GrpcCreateSavedSearchRequest,
    Empty, HistoryRequest, ListSavedSearchesRequest, PinSavedSearchRequest as GrpcPinSavedSearchRequest,
    SavedSearchIdRequest, SearchRequest as GrpcSearchRequest, UpdateSettingsRequest as GrpcUpdateSettingsRequest,
};

fn err(status: StatusCode, detail: impl Into<String>) -> Response {
    (status, Json(json!({ "detail": detail.into() }))).into_response()
}

fn grpc_err(e: tonic::Status) -> Response {
    let status = if e.code() == tonic::Code::NotFound {
        StatusCode::NOT_FOUND
    } else {
        StatusCode::INTERNAL_SERVER_ERROR
    };
    err(status, format!("conversation-search service error: {e}"))
}

async fn client() -> Result<ConversationSearchServiceClient<tonic::transport::Channel>, Response> {
    let url = std::env::var("CONVERSATION_SEARCH_SERVICE_URL")
        .unwrap_or_else(|_| "http://127.0.0.1:50067".to_string());
    ConversationSearchServiceClient::connect(url)
        .await
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, format!("conversation-search service unavailable: {e}")))
}

// ---------------------------------------------------------------------------
// Query params / payloads (unchanged REST contract)
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

#[derive(Debug, Serialize)]
struct SavedSearchOut {
    id: String,
    org_id: String,
    name: String,
    query: String,
    search_type: String,
    filters: String,
    created_by: String,
    created_at: String,
    updated_at: String,
    is_pinned: bool,
    usage_count: i32,
}

fn saved_search_from_grpc(x: newax_proto::conversation_search::v1::SavedSearch) -> SavedSearchOut {
    SavedSearchOut {
        id: x.id,
        org_id: x.org_id,
        name: x.name,
        query: x.query,
        search_type: x.search_type,
        filters: x.filters,
        created_by: x.created_by,
        created_at: x.created_at,
        updated_at: x.updated_at,
        is_pinned: x.is_pinned,
        usage_count: x.usage_count,
    }
}

#[derive(Debug, Serialize)]
struct SearchHistoryEntryOut {
    id: String,
    org_id: String,
    query: String,
    search_type: String,
    result_count: i64,
    filters: String,
    performed_by: String,
    performed_at: String,
    duration_ms: i32,
}

fn history_entry_from_grpc(x: newax_proto::conversation_search::v1::SearchHistoryEntry) -> SearchHistoryEntryOut {
    SearchHistoryEntryOut {
        id: x.id,
        org_id: x.org_id,
        query: x.query,
        search_type: x.search_type,
        result_count: x.result_count,
        filters: x.filters,
        performed_by: x.performed_by,
        performed_at: x.performed_at,
        duration_ms: x.duration_ms,
    }
}

#[derive(Debug, Serialize)]
struct SearchAnalyticsOut {
    id: String,
    org_id: String,
    window_start: String,
    window_end: String,
    total_searches: i64,
    avg_result_count: f64,
    avg_duration_ms: f64,
    zero_result_searches: i64,
    top_queries: String,
    top_search_types: String,
    created_at: String,
}

fn analytics_from_grpc(x: newax_proto::conversation_search::v1::SearchAnalytics) -> SearchAnalyticsOut {
    SearchAnalyticsOut {
        id: x.id,
        org_id: x.org_id,
        window_start: x.window_start,
        window_end: x.window_end,
        total_searches: x.total_searches,
        avg_result_count: x.avg_result_count,
        avg_duration_ms: x.avg_duration_ms,
        zero_result_searches: x.zero_result_searches,
        top_queries: x.top_queries,
        top_search_types: x.top_search_types,
        created_at: x.created_at,
    }
}

#[derive(Debug, Serialize)]
struct SearchAnalyticsSnapshotOut {
    window_start: String,
    window_end: String,
    total_searches: i64,
    avg_result_count: f64,
    avg_duration_ms: f64,
    zero_result_searches: i64,
}

fn snapshot_from_grpc(x: newax_proto::conversation_search::v1::SearchAnalyticsSnapshot) -> SearchAnalyticsSnapshotOut {
    SearchAnalyticsSnapshotOut {
        window_start: x.window_start,
        window_end: x.window_end,
        total_searches: x.total_searches,
        avg_result_count: x.avg_result_count,
        avg_duration_ms: x.avg_duration_ms,
        zero_result_searches: x.zero_result_searches,
    }
}

#[derive(Debug, Serialize)]
struct SearchSettingsOut {
    id: String,
    org_id: String,
    max_results: i32,
    search_message_content: bool,
    search_contact_details: bool,
    search_conversation_metadata: bool,
    track_search_history: bool,
    auto_suggest: bool,
    min_query_length: i32,
    highlight_matches: bool,
    created_at: String,
    updated_at: String,
}

fn settings_from_grpc(x: newax_proto::conversation_search::v1::SearchSettings) -> SearchSettingsOut {
    SearchSettingsOut {
        id: x.id,
        org_id: x.org_id,
        max_results: x.max_results,
        search_message_content: x.search_message_content,
        search_contact_details: x.search_contact_details,
        search_conversation_metadata: x.search_conversation_metadata,
        track_search_history: x.track_search_history,
        auto_suggest: x.auto_suggest,
        min_query_length: x.min_query_length,
        highlight_matches: x.highlight_matches,
        created_at: x.created_at,
        updated_at: x.updated_at,
    }
}

#[derive(Debug, Serialize)]
struct SearchResultOut {
    search_type: String,
    id: String,
    title: String,
    snippet: String,
    metadata: serde_json::Value,
    matched_fields: Vec<String>,
    score: f64,
}

#[derive(Debug, Serialize)]
struct SearchResponseOut {
    query: String,
    total_results: i64,
    conversations: Vec<SearchResultOut>,
    messages: Vec<SearchResultOut>,
    contacts: Vec<SearchResultOut>,
    duration_ms: i64,
}

fn result_from_grpc(r: newax_proto::conversation_search::v1::SearchResult) -> SearchResultOut {
    SearchResultOut {
        search_type: r.search_type,
        id: r.id,
        title: r.title,
        snippet: r.snippet,
        metadata: serde_json::from_str(&r.metadata_json).unwrap_or(json!({})),
        matched_fields: r.matched_fields,
        score: r.score,
    }
}

fn search_response_from_grpc(r: newax_proto::conversation_search::v1::SearchResponse) -> SearchResponseOut {
    SearchResponseOut {
        query: r.query,
        total_results: r.total_results,
        conversations: r.conversations.into_iter().map(result_from_grpc).collect(),
        messages: r.messages.into_iter().map(result_from_grpc).collect(),
        contacts: r.contacts.into_iter().map(result_from_grpc).collect(),
        duration_ms: r.duration_ms,
    }
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

pub async fn get_summary(State(_state): State<crate::db::AppState>) -> impl IntoResponse {
    let mut c = match client().await { Ok(c) => c, Err(e) => return e };
    match c.get_summary(Empty {}).await {
        Ok(resp) => {
            let json: serde_json::Value = serde_json::from_str(&resp.into_inner().json).unwrap_or(json!({}));
            (StatusCode::OK, Json(json)).into_response()
        }
        Err(e) => grpc_err(e),
    }
}

pub async fn search(
    State(_state): State<crate::db::AppState>,
    Query(params): Query<SearchQuery>,
) -> impl IntoResponse {
    let mut c = match client().await { Ok(c) => c, Err(e) => return e };
    match c
        .search(GrpcSearchRequest { q: params.q, search_type: params.search_type, limit: params.limit })
        .await
    {
        Ok(resp) => (StatusCode::OK, Json(search_response_from_grpc(resp.into_inner()))).into_response(),
        Err(e) => grpc_err(e),
    }
}

pub async fn list_saved_searches(
    State(_state): State<crate::db::AppState>,
    Query(params): Query<SavedSearchQuery>,
) -> impl IntoResponse {
    let mut c = match client().await { Ok(c) => c, Err(e) => return e };
    match c.list_saved_searches(ListSavedSearchesRequest { search_type: params.search_type }).await {
        Ok(resp) => {
            let searches: Vec<SavedSearchOut> = resp.into_inner().searches.into_iter().map(saved_search_from_grpc).collect();
            (StatusCode::OK, Json(searches)).into_response()
        }
        Err(e) => grpc_err(e),
    }
}

pub async fn create_saved_search(
    State(_state): State<crate::db::AppState>,
    Json(payload): Json<CreateSavedSearchPayload>,
) -> impl IntoResponse {
    let mut c = match client().await { Ok(c) => c, Err(e) => return e };
    let req = GrpcCreateSavedSearchRequest {
        name: payload.name,
        query: payload.query,
        search_type: payload.search_type,
        filters_json: payload.filters.map(|v| v.to_string()),
    };
    match c.create_saved_search(req).await {
        Ok(resp) => (StatusCode::CREATED, Json(saved_search_from_grpc(resp.into_inner()))).into_response(),
        Err(e) => grpc_err(e),
    }
}

pub async fn get_saved_search(State(_state): State<crate::db::AppState>, Path(id): Path<String>) -> impl IntoResponse {
    let mut c = match client().await { Ok(c) => c, Err(e) => return e };
    match c.get_saved_search(SavedSearchIdRequest { id }).await {
        Ok(resp) => (StatusCode::OK, Json(saved_search_from_grpc(resp.into_inner()))).into_response(),
        Err(e) => grpc_err(e),
    }
}

pub async fn delete_saved_search(
    State(_state): State<crate::db::AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let mut c = match client().await { Ok(c) => c, Err(e) => return e };
    match c.delete_saved_search(SavedSearchIdRequest { id }).await {
        Ok(_) => (StatusCode::NO_CONTENT, Json(json!({}))).into_response(),
        Err(e) => grpc_err(e),
    }
}

pub async fn pin_saved_search(
    State(_state): State<crate::db::AppState>,
    Path(id): Path<String>,
    Json(payload): Json<PinPayload>,
) -> impl IntoResponse {
    let mut c = match client().await { Ok(c) => c, Err(e) => return e };
    match c.pin_saved_search(GrpcPinSavedSearchRequest { id, pinned: payload.pinned }).await {
        Ok(_) => (StatusCode::OK, Json(json!({ "success": true }))).into_response(),
        Err(e) => grpc_err(e),
    }
}

pub async fn use_saved_search(
    State(_state): State<crate::db::AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let mut c = match client().await { Ok(c) => c, Err(e) => return e };
    match c.use_saved_search(SavedSearchIdRequest { id }).await {
        Ok(resp) => (StatusCode::OK, Json(search_response_from_grpc(resp.into_inner()))).into_response(),
        Err(e) => grpc_err(e),
    }
}

pub async fn get_search_history(
    State(_state): State<crate::db::AppState>,
    Query(params): Query<HistoryQuery>,
) -> impl IntoResponse {
    let mut c = match client().await { Ok(c) => c, Err(e) => return e };
    match c.get_search_history(HistoryRequest { limit: params.limit }).await {
        Ok(resp) => {
            let history: Vec<SearchHistoryEntryOut> = resp.into_inner().history.into_iter().map(history_entry_from_grpc).collect();
            (StatusCode::OK, Json(history)).into_response()
        }
        Err(e) => grpc_err(e),
    }
}

pub async fn clear_search_history(State(_state): State<crate::db::AppState>) -> impl IntoResponse {
    let mut c = match client().await { Ok(c) => c, Err(e) => return e };
    match c.clear_search_history(Empty {}).await {
        Ok(_) => (StatusCode::OK, Json(json!({ "success": true }))).into_response(),
        Err(e) => grpc_err(e),
    }
}

pub async fn get_analytics(State(_state): State<crate::db::AppState>) -> impl IntoResponse {
    let mut c = match client().await { Ok(c) => c, Err(e) => return e };
    match c.get_analytics(Empty {}).await {
        Ok(resp) => {
            let analytics: Vec<SearchAnalyticsOut> = resp.into_inner().analytics.into_iter().map(analytics_from_grpc).collect();
            (StatusCode::OK, Json(analytics)).into_response()
        }
        Err(e) => grpc_err(e),
    }
}

pub async fn generate_analytics_snapshot(State(_state): State<crate::db::AppState>) -> impl IntoResponse {
    let mut c = match client().await { Ok(c) => c, Err(e) => return e };
    match c.generate_analytics_snapshot(Empty {}).await {
        Ok(resp) => (StatusCode::OK, Json(snapshot_from_grpc(resp.into_inner()))).into_response(),
        Err(e) => grpc_err(e),
    }
}

pub async fn get_settings(State(_state): State<crate::db::AppState>) -> impl IntoResponse {
    let mut c = match client().await { Ok(c) => c, Err(e) => return e };
    match c.get_settings(Empty {}).await {
        Ok(resp) => (StatusCode::OK, Json(settings_from_grpc(resp.into_inner()))).into_response(),
        Err(e) => grpc_err(e),
    }
}

pub async fn update_settings(
    State(_state): State<crate::db::AppState>,
    Json(payload): Json<UpdateSettingsPayload>,
) -> impl IntoResponse {
    let mut c = match client().await { Ok(c) => c, Err(e) => return e };
    let req = GrpcUpdateSettingsRequest {
        max_results: payload.max_results,
        search_message_content: payload.search_message_content,
        search_contact_details: payload.search_contact_details,
        search_conversation_metadata: payload.search_conversation_metadata,
        track_search_history: payload.track_search_history,
        auto_suggest: payload.auto_suggest,
        min_query_length: payload.min_query_length,
        highlight_matches: payload.highlight_matches,
    };
    match c.update_settings(req).await {
        Ok(resp) => (StatusCode::OK, Json(settings_from_grpc(resp.into_inner()))).into_response(),
        Err(e) => grpc_err(e),
    }
}
