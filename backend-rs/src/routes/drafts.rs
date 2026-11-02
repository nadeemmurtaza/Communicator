use crate::{db::AppState, models::Draft};
use axum::{
    extract::{Path, Query, State},
    response::IntoResponse,
    Json,
};
use chrono::Utc;
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct DraftQuery {
    pub status: Option<String>,
}

pub async fn list_drafts(
    State(state): State<AppState>,
    Query(query): Query<DraftQuery>,
) -> impl IntoResponse {
    let status_filter = query.status.unwrap_or_else(|| "pending_review".to_string());

    let drafts = sqlx::query_as::<_, Draft>(
        "SELECT id, conversation_id, body, confidence, citations, status, created_at FROM drafts WHERE status = ? ORDER BY created_at DESC",
    )
    .bind(&status_filter)
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    Json(drafts)
}

pub async fn approve_draft(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let now = Utc::now().to_rfc3339();
    let draft = sqlx::query_as::<_, Draft>(
        "SELECT id, conversation_id, body, confidence, citations, status, created_at FROM drafts WHERE id = ?",
    )
    .bind(&id)
    .fetch_optional(&state.db)
    .await
    .unwrap_or(None);

    if let Some(d) = draft {
        // Mark draft approved
        let _ = sqlx::query("UPDATE drafts SET status = 'approved' WHERE id = ?")
            .bind(&id)
            .execute(&state.db)
            .await;

        // Auto-send as outbound message
        let msg_id = Uuid::new_v4().to_string();
        let _ = sqlx::query(
            "INSERT INTO messages (id, conversation_id, direction, channel, body, status, sent_at, sender_type, sender_name) VALUES (?, ?, 'outbound', 'whatsapp', ?, 'delivered', ?, 'agent', 'AI Copilot')",
        )
        .bind(&msg_id)
        .bind(&d.conversation_id)
        .bind(&d.body)
        .bind(&now)
        .execute(&state.db)
        .await;

        Json(json!({
            "id": d.id,
            "conversationId": d.conversation_id,
            "body": d.body,
            "confidence": d.confidence,
            "status": "approved",
            "createdAt": d.created_at
        }))
    } else {
        Json(json!({ "error": "Draft not found" }))
    }
}

#[derive(Deserialize)]
pub struct RejectDraftPayload {
    pub reason: Option<String>,
}

pub async fn reject_draft(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(_payload): Json<RejectDraftPayload>,
) -> impl IntoResponse {
    let _ = sqlx::query("UPDATE drafts SET status = 'rejected' WHERE id = ?")
        .bind(&id)
        .execute(&state.db)
        .await;

    Json(json!({ "id": id, "status": "rejected" }))
}

#[derive(Deserialize)]
pub struct EditDraftPayload {
    pub body: String,
}

pub async fn edit_draft(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<EditDraftPayload>,
) -> impl IntoResponse {
    let _ = sqlx::query("UPDATE drafts SET body = ?, status = 'pending_review' WHERE id = ?")
        .bind(&payload.body)
        .bind(&id)
        .execute(&state.db)
        .await;

    Json(json!({ "id": id, "body": payload.body, "status": "pending_review" }))
}

pub async fn generate_draft(
    State(state): State<AppState>,
    Path(conversation_id): Path<String>,
) -> impl IntoResponse {
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();

    // Check last customer message in thread to generate a hyper-relevant AI response
    let last_msg: Option<String> = sqlx::query_scalar::<_, String>(
        "SELECT body FROM messages WHERE conversation_id = ? AND direction = 'inbound' ORDER BY sent_at DESC LIMIT 1",
    )
    .bind(&conversation_id)
    .fetch_optional(&state.db)
    .await
    .unwrap_or(None);

    let draft_text = match last_msg {
        Some(msg) => {
            if msg.to_lowercase().contains("service") || msg.to_lowercase().contains("pricing") {
                "Hello! Thank you for inquiring about Newax Technologies services. We offer high-performance enterprise messaging, cross-platform communications, and automated lead intelligence solutions. Would you like to schedule a quick 15-minute introductory walkthrough?".to_string()
            } else if msg.to_lowercase().contains("architecture") || msg.to_lowercase().contains("model") {
                "Thank you for sharing your architecture specifications. Our team has reviewed the multi-stage pipeline and recommendations. Let's arrange a technical sync with our engineering team to review integration milestones.".to_string()
            } else {
                format!("Hi there! Thank you for contacting us regarding: \"{}\". We have received your inquiry and an agent is ready to help you directly.", msg.chars().take(80).collect::<String>())
            }
        }
        None => "Hi there! Thank you for reaching out to Newax Technologies. How can we assist you today?".to_string(),
    };

    let _ = sqlx::query(
        "INSERT INTO drafts (id, conversation_id, body, confidence, citations, status, created_at) VALUES (?, ?, ?, 0.94, '[]', 'pending_review', ?)",
    )
    .bind(&id)
    .bind(&conversation_id)
    .bind(&draft_text)
    .bind(&now)
    .execute(&state.db)
    .await;

    Json(json!({
        "id": id,
        "conversationId": conversation_id,
        "body": draft_text,
        "confidence": 0.94,
        "citations": [],
        "status": "pending_review",
        "createdAt": now
    }))
}
