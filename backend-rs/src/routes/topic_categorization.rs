#![allow(dead_code)]

//! Topic Categorization API routes — Phase 11.
//!
//! Endpoints:
//! - GET    /topics/summary              — Topic distribution summary
//! - GET    /topics/{conversation_id}    — Get topic for a conversation
//! - POST   /topics/{conversation_id}    — Categorize a conversation
//! - PUT    /topics/{conversation_id}    — Override topic manually
//! - POST   /topics/batch-categorize     — Categorize all uncategorized conversations

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde::Deserialize;
use serde_json::json;
use sqlx::Row;

use crate::db::AppState;
use crate::services::topic_categorization;

// ---------------------------------------------------------------------------
// Query parameters
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct OverridePayload {
    pub topic: String,
    pub subtopic: Option<String>,
    pub overridden_by: Option<String>,
}

// ---------------------------------------------------------------------------
// GET /topics/summary — Topic distribution summary
// ---------------------------------------------------------------------------

pub async fn get_topic_summary(State(state): State<AppState>) -> impl axum::response::IntoResponse {
    match topic_categorization::get_topic_summary(&state).await {
        Ok(summary) => Json(summary).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e })),
        )
            .into_response(),
    }
}

// ---------------------------------------------------------------------------
// GET /topics/{conversation_id} — Get topic for a conversation
// ---------------------------------------------------------------------------

pub async fn get_conversation_topic(
    State(state): State<AppState>,
    Path(conversation_id): Path<String>,
) -> impl axum::response::IntoResponse {
    // Fetch existing topic from DB
    let topic = sqlx::query(
        "SELECT id, conversation_id, topic, subtopic, confidence, keywords,
         is_overridden, overridden_by, categorized_at, created_at, updated_at
         FROM conversation_topics WHERE conversation_id = ? ORDER BY created_at DESC LIMIT 1",
    )
    .bind(&conversation_id)
    .fetch_optional(&state.db)
    .await;

    match topic {
        Ok(Some(row)) => {
            let keywords: Vec<String> = row.try_get("keywords").ok()
                .and_then(|s: String| serde_json::from_str(&s).ok())
                .unwrap_or_default();

            let result = topic_categorization::ConversationTopic {
                id: row.get("id"),
                conversation_id: row.get("conversation_id"),
                topic: row.get("topic"),
                subtopic: row.try_get("subtopic").ok(),
                confidence: row.get("confidence"),
                keywords,
                is_overridden: row.get("is_overridden"),
                overridden_by: row.try_get("overridden_by").ok(),
                categorized_at: row.get("categorized_at"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            };
            Json(result).into_response()
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "No topic categorization found for this conversation" })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": format!("Database error: {}", e) })),
        )
            .into_response(),
    }
}

// ---------------------------------------------------------------------------
// POST /topics/{conversation_id} — Categorize a conversation
// ---------------------------------------------------------------------------

pub async fn categorize_conversation(
    State(state): State<AppState>,
    Path(conversation_id): Path<String>,
) -> impl axum::response::IntoResponse {
    match topic_categorization::categorize_conversation(&state, &conversation_id).await {
        Ok(topic) => Json(topic).into_response(),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": e })),
        )
            .into_response(),
    }
}

// ---------------------------------------------------------------------------
// PUT /topics/{conversation_id} — Override topic manually
// ---------------------------------------------------------------------------

pub async fn override_conversation_topic(
    State(state): State<AppState>,
    Path(conversation_id): Path<String>,
    Json(payload): Json<OverridePayload>,
) -> impl axum::response::IntoResponse {
    let overridden_by = payload.overridden_by.unwrap_or_else(|| "api_user".to_string());

    match topic_categorization::override_topic(
        &state,
        &conversation_id,
        &payload.topic,
        payload.subtopic.as_deref(),
        &overridden_by,
    )
    .await
    {
        Ok(topic) => Json(topic).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e })),
        )
            .into_response(),
    }
}

// ---------------------------------------------------------------------------
// POST /topics/batch-categorize — Categorize all uncategorized conversations
// ---------------------------------------------------------------------------

pub async fn batch_categorize(State(state): State<AppState>) -> impl axum::response::IntoResponse {
    match topic_categorization::categorize_uncategorized(&state).await {
        Ok(result) => Json(result).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e })),
        )
            .into_response(),
    }
}
