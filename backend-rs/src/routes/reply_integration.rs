use axum::extract::{Path, Query, State};
use axum::Json;
use sqlx::Row;
use crate::db::AppState;
use crate::services::reply_integration::{ReplyIntegrationService, ReplyIntegrationSettings};

/// GET /reply-integration/summary
pub async fn get_summary(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    let service = ReplyIntegrationService::new(&state.db);
    match service.get_summary().await {
        Ok(summary) => Ok(Json(serde_json::to_value(summary).unwrap_or_default())),
        Err(e) => Err((
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )),
    }
}

/// GET /reply-integration/settings
pub async fn get_settings(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    let service = ReplyIntegrationService::new(&state.db);
    match service.get_settings().await {
        Ok(settings) => Ok(Json(serde_json::to_value(settings).unwrap_or_default())),
        Err(e) => Err((
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )),
    }
}

/// PUT /reply-integration/settings
pub async fn update_settings(
    State(state): State<AppState>,
    Json(settings): Json<ReplyIntegrationSettings>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    let service = ReplyIntegrationService::new(&state.db);
    match service.update_settings(&settings).await {
        Ok(updated) => Ok(Json(serde_json::to_value(updated).unwrap_or_default())),
        Err(e) => Err((
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )),
    }
}

/// POST /reply-integration/build-context
pub async fn build_context(
    State(state): State<AppState>,
    Json(body): Json<BuildContextRequest>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    let service = ReplyIntegrationService::new(&state.db);
    match service.build_context_package(body.contact_id.as_deref(), &body.last_inbound, &body.tenant_key).await {
        Ok(pkg) => Ok(Json(serde_json::to_value(pkg).unwrap_or_default())),
        Err(e) => Err((
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )),
    }
}

/// GET /reply-integration/sources
pub async fn get_knowledge_sources(
    State(state): State<AppState>,
    Query(params): Query<GetSourcesQuery>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    let _service = ReplyIntegrationService::new(&state.db);

    let mut query = String::from(
        "SELECT id, draft_id, conversation_id, source_type, source_id, source_title, relevance_score, used_in_reply, created_at \
         FROM reply_knowledge_sources WHERE 1=1"
    );
    let mut idx = 1;
    if params.draft_id.is_some() {
        query.push_str(&format!(" AND draft_id = ${}", idx));
        idx += 1;
    }
    if params.conversation_id.is_some() {
        query.push_str(&format!(" AND conversation_id = ${}", idx));
        idx += 1;
    }
    if params.source_type.is_some() {
        query.push_str(&format!(" AND source_type = ${}", idx));
        idx += 1;
    }
    query.push_str(" ORDER BY created_at DESC LIMIT 100");

    let mut q = sqlx::query(&query);
    if let Some(ref draft_id) = params.draft_id {
        q = q.bind(draft_id);
    }
    if let Some(ref conversation_id) = params.conversation_id {
        q = q.bind(conversation_id);
    }
    if let Some(ref source_type) = params.source_type {
        q = q.bind(source_type);
    }

    let rows = q.fetch_all(&state.db).await.map_err(|e| (
        axum::http::StatusCode::INTERNAL_SERVER_ERROR,
        Json(serde_json::json!({ "error": e.to_string() })),
    ))?;
    
    let mut sources = Vec::new();
    for row in rows {
        let source: serde_json::Value = serde_json::json!({
            "id": row.try_get::<String, _>("id").unwrap_or_default(),
            "draft_id": row.try_get::<String, _>("draft_id").unwrap_or_default(),
            "conversation_id": row.try_get::<String, _>("conversation_id").unwrap_or_default(),
            "source_type": row.try_get::<String, _>("source_type").unwrap_or_default(),
            "source_id": row.try_get::<Option<String>, _>("source_id").ok(),
            "source_title": row.try_get::<String, _>("source_title").unwrap_or_default(),
            "relevance_score": row.try_get::<f64, _>("relevance_score").unwrap_or(0.5),
            "used_in_reply": row.try_get::<bool, _>("used_in_reply").unwrap_or(true),
            "created_at": row.try_get::<chrono::DateTime<chrono::Utc>, _>("created_at").ok(),
        });
        sources.push(source);
    }
    
    Ok(Json(serde_json::json!({ "sources": sources, "total": sources.len() })))
}

#[derive(serde::Deserialize)]
pub struct BuildContextRequest {
    contact_id: Option<String>,
    last_inbound: String,
    tenant_key: String,
}

#[derive(serde::Deserialize)]
pub struct GetSourcesQuery {
    draft_id: Option<String>,
    conversation_id: Option<String>,
    source_type: Option<String>,
}
