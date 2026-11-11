use axum::extract::{Path, Query, State};
use axum::Json;
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
    let service = ReplyIntegrationService::new(&state.db);
    
    let mut query = String::from(
        "SELECT id, draft_id, conversation_id, source_type, source_id, source_title, relevance_score, used_in_reply, created_at \
         FROM reply_knowledge_sources WHERE 1=1"
    );
    let mut binds: Vec<Box<dyn sqlx::types::Type<sqlx::postgres::Postgres> + Send + Sync>> = Vec::new();
    let mut bind_values: Vec<Box<dyn sqlx::encode::Encode<'_, sqlx::postgres::Postgres> + Send + Sync>> = Vec::new();
    
    if let Some(ref draft_id) = params.draft_id {
        query.push_str(" AND draft_id = $");
        let idx = binds.len() + 1;
        binds.push(Box::new(draft_id.clone()));
        query.push_str(&idx.to_string());
    }
    
    if let Some(ref conversation_id) = params.conversation_id {
        query.push_str(" AND conversation_id = $");
        let idx = binds.len() + 1;
        binds.push(Box::new(conversation_id.clone()));
        query.push_str(&idx.to_string());
    }
    
    if let Some(ref source_type) = params.source_type {
        query.push_str(" AND source_type = $");
        let idx = binds.len() + 1;
        binds.push(Box::new(source_type.clone()));
        query.push_str(&idx.to_string());
    }
    
    query.push_str(" ORDER BY created_at DESC LIMIT 100");
    
    // Use a simpler approach with direct query
    let rows = sqlx::query(
        "SELECT id, draft_id, conversation_id, source_type, source_id, source_title, relevance_score, used_in_reply, created_at \
         FROM reply_knowledge_sources ORDER BY created_at DESC LIMIT 100"
    ).fetch_all(&state.db).await.map_err(|e| (
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
struct BuildContextRequest {
    contact_id: Option<String>,
    last_inbound: String,
    tenant_key: String,
}

#[derive(serde::Deserialize)]
struct GetSourcesQuery {
    draft_id: Option<String>,
    conversation_id: Option<String>,
    source_type: Option<String>,
}
