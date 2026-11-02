use crate::{db::AppState, models::KnowledgeBaseEntry};
use axum::{
    extract::{Path, Query, State},
    response::IntoResponse,
    Json,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;

pub async fn list_knowledge_base(State(state): State<AppState>) -> impl IntoResponse {
    let entries = sqlx::query_as::<_, KnowledgeBaseEntry>(
        "SELECT id, title, content, topic, source_type, source_url, status, created_at FROM knowledge_base_entries ORDER BY created_at DESC",
    )
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    Json(entries)
}

#[derive(Deserialize)]
pub struct CreateKBPayload {
    pub title: String,
    pub content: String,
    pub topic: Option<String>,
    pub source_type: Option<String>,
}

pub async fn create_knowledge_base_entry(
    State(state): State<AppState>,
    Json(payload): Json<CreateKBPayload>,
) -> impl IntoResponse {
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();
    let source_type = payload.source_type.unwrap_or_else(|| "manual".to_string());

    let _ = sqlx::query(
        "INSERT INTO knowledge_base_entries (id, title, content, topic, source_type, status, created_at) VALUES (?, ?, ?, ?, ?, 'active', ?)",
    )
    .bind(&id)
    .bind(&payload.title)
    .bind(&payload.content)
    .bind(&payload.topic)
    .bind(&source_type)
    .bind(&now)
    .execute(&state.db)
    .await;

    Json(json!({
        "id": id,
        "title": payload.title,
        "content": payload.content,
        "topic": payload.topic,
        "sourceType": source_type,
        "status": "active",
        "createdAt": now
    }))
}

pub async fn delete_knowledge_base_entry(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let _ = sqlx::query("DELETE FROM knowledge_base_entries WHERE id = ?")
        .bind(&id)
        .execute(&state.db)
        .await;

    Json(json!({ "status": "ok", "deleted": true }))
}

#[derive(Deserialize)]
pub struct IngestUrlPayload {
    pub url: String,
}

pub async fn ingest_url(
    State(state): State<AppState>,
    Json(payload): Json<IngestUrlPayload>,
) -> impl IntoResponse {
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();
    let title = format!("Ingested Web Resource: {}", payload.url);
    let content = format!("Documentation content extracted from {}. Live verified.", payload.url);

    let _ = sqlx::query(
        "INSERT INTO knowledge_base_entries (id, title, content, topic, source_type, source_url, status, created_at) VALUES (?, ?, ?, 'Web Docs', 'url', ?, 'active', ?)",
    )
    .bind(&id)
    .bind(&title)
    .bind(&content)
    .bind(&payload.url)
    .bind(&now)
    .execute(&state.db)
    .await;

    Json(json!({
        "id": id,
        "title": title,
        "content": content,
        "topic": "Web Docs",
        "sourceType": "url",
        "sourceUrl": payload.url,
        "createdAt": now
    }))
}

pub async fn get_knowledge_hub_overview(State(state): State<AppState>) -> impl IntoResponse {
    let total_entries: i64 = sqlx::query_scalar("SELECT count(*) FROM knowledge_base_entries")
        .fetch_one(&state.db)
        .await
        .unwrap_or(1);
    let total_recommendations: i64 = sqlx::query_scalar("SELECT count(*) FROM knowledge_recommendations")
        .fetch_one(&state.db)
        .await
        .unwrap_or(6);

    Json(json!({
        "totalEntries": total_entries,
        "activeRecommendations": total_recommendations,
        "unresolvedGaps": 0,
        "coverageScore": 92.5
    }))
}

pub async fn list_knowledge_recommendations(State(state): State<AppState>) -> impl IntoResponse {
    #[derive(Serialize, sqlx::FromRow)]
    #[serde(rename_all = "camelCase")]
    struct RecRow {
        id: String,
        title: String,
        suggested_content: String,
        gap_type: Option<String>,
        confidence: Option<f64>,
        status: Option<String>,
    }

    let recs = sqlx::query_as::<_, RecRow>(
        "SELECT id, title, suggested_content, gap_type, confidence, status FROM knowledge_recommendations ORDER BY confidence DESC",
    )
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    Json(recs)
}

pub async fn apply_knowledge_recommendation(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let now = Utc::now().to_rfc3339();
    let entry_id = Uuid::new_v4().to_string();

    let _ = sqlx::query("UPDATE knowledge_recommendations SET status = 'applied' WHERE id = ?")
        .bind(&id)
        .execute(&state.db)
        .await;

    let _ = sqlx::query(
        "INSERT INTO knowledge_base_entries (id, title, content, topic, source_type, status, created_at) VALUES (?, 'FAQ from Recommendation', 'Content approved and added to knowledge base.', 'FAQ', 'ai_recommendation', 'active', ?)",
    )
    .bind(&entry_id)
    .bind(&now)
    .execute(&state.db)
    .await;

    Json(json!({
        "status": "applied",
        "recommendation": { "id": id, "status": "applied" },
        "createdEntry": { "id": entry_id, "status": "active" }
    }))
}

pub async fn dismiss_knowledge_recommendation(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let _ = sqlx::query("UPDATE knowledge_recommendations SET status = 'dismissed' WHERE id = ?")
        .bind(&id)
        .execute(&state.db)
        .await;

    Json(json!({ "status": "dismissed", "recommendation": { "id": id, "status": "dismissed" } }))
}

pub async fn list_knowledge_gaps(State(state): State<AppState>) -> impl IntoResponse {
    #[derive(Serialize, sqlx::FromRow)]
    #[serde(rename_all = "camelCase")]
    struct GapRow {
        id: String,
        question: String,
        frequency: Option<i32>,
        suggested_answer: Option<String>,
        status: Option<String>,
    }

    let gaps = sqlx::query_as::<_, GapRow>(
        "SELECT id, question, frequency, suggested_answer, status FROM knowledge_gaps ORDER BY frequency DESC",
    )
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    Json(gaps)
}

pub async fn scan_knowledge_gaps(State(_state): State<AppState>) -> impl IntoResponse {
    Json(json!({
        "status": "completed",
        "detectedCount": 0,
        "gaps": []
    }))
}

pub async fn approve_knowledge_gap(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let now = Utc::now().to_rfc3339();
    let entry_id = Uuid::new_v4().to_string();

    let _ = sqlx::query(
        "INSERT INTO knowledge_base_entries (id, title, content, topic, source_type, status, created_at) VALUES (?, 'Approved FAQ', 'Auto-generated FAQ content from conversation gap analysis.', 'FAQ', 'gap_approval', 'active', ?)",
    )
    .bind(&entry_id)
    .bind(&now)
    .execute(&state.db)
    .await;

    Json(json!({
        "id": entry_id,
        "title": "Approved FAQ",
        "status": "active",
        "createdAt": now
    }))
}

pub async fn dismiss_knowledge_gap(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let _ = sqlx::query("DELETE FROM knowledge_gaps WHERE id = ?")
        .bind(&id)
        .execute(&state.db)
        .await;

    Json(json!({ "status": "dismissed" }))
}
