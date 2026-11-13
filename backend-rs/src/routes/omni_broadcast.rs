//! Omni-Channel Broadcast routes — Phase 46

use serde_json::{json, Value};
use sqlx::Row;
use uuid::Uuid;

use crate::db::AppState;
use crate::services::omni_broadcast::{
    Broadcast, BroadcastMedia, BroadcastChannel, BroadcastRecipient, BroadcastLog,
    BroadcastSettings, BroadcastSummary, ChannelFormat, FormatPreview, SendBroadcastResult,
    get_broadcast_summary, get_format_preview, get_available_channels, send_broadcast,
    preview_html_for_all_channels,
};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};

// ---------------------------------------------------------------------------
// Summary
// ---------------------------------------------------------------------------

pub async fn get_summary(
    State(state): State<AppState>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match get_broadcast_summary(&state).await {
        Ok(summary) => Ok(Json(json!({
            "total_broadcasts": summary.total_broadcasts,
            "draft_broadcasts": summary.draft_broadcasts,
            "sent_broadcasts": summary.sent_broadcasts,
            "failed_broadcasts": summary.failed_broadcasts,
            "scheduled_broadcasts": summary.scheduled_broadcasts,
            "total_recipients_sent": summary.total_recipients_sent,
            "total_delivered": summary.total_delivered,
            "total_failed": summary.total_failed,
            "total_read": summary.total_read,
            "total_clicked": summary.total_clicked,
            "total_replied": summary.total_replied,
            "by_channel": summary.by_channel,
            "recent_broadcasts": summary.recent_broadcasts,
        }))),
        Err(e) => {
            tracing::error!("Failed to get broadcast summary: {}", e);
            Ok(Json(json!({
                "total_broadcasts": 0i32,
                "draft_broadcasts": 0i32,
                "sent_broadcasts": 0i32,
                "failed_broadcasts": 0i32,
                "scheduled_broadcasts": 0i32,
                "total_recipients_sent": 0i32,
                "total_delivered": 0i32,
                "total_failed": 0i32,
                "total_read": 0i32,
                "total_clicked": 0i32,
                "total_replied": 0i32,
                "by_channel": [],
                "recent_broadcasts": [],
            })))
        }
    }
}

// ---------------------------------------------------------------------------
// Broadcast CRUD
// ---------------------------------------------------------------------------

pub async fn list_broadcasts(
    State(state): State<AppState>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let status = params.get("status").map(|s| s.as_str());
    let limit: i32 = params.get("limit").and_then(|s| s.parse().ok()).unwrap_or(50);

    match Broadcast::list(&state.db, status, limit).await {
        Ok(broadcasts) => Ok(Json(json!({ "broadcasts": broadcasts }))),
        Err(e) => {
            tracing::error!("Failed to list broadcasts: {}", e);
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e }))))
        }
    }
}

pub async fn get_broadcast(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match Broadcast::get(&state.db, &id).await {
        Ok(broadcast) => Ok(Json(json!(broadcast))),
        Err(e) => {
            tracing::error!("Failed to get broadcast {}: {}", id, e);
            Err((StatusCode::NOT_FOUND, Json(json!({ "error": e }))))
        }
    }
}

pub async fn create_broadcast(
    State(state): State<AppState>,
    Json(body): Json<Value>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let organization_id = body.get("organization_id").and_then(|v| v.as_str()).unwrap_or("default").to_string();
    let brand_id = body.get("brand_id").and_then(|v| v.as_str()).map(String::from);
    let title = body.get("title").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let created_by = body.get("created_by").and_then(|v| v.as_str()).unwrap_or("system").to_string();
    let body_text = body.get("body_text").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let body_html = body.get("body_html").and_then(|v| v.as_str()).map(String::from);
    let subject_line = body.get("subject_line").and_then(|v| v.as_str()).map(String::from);
    let preview_text = body.get("preview_text").and_then(|v| v.as_str()).map(String::from);
    let media_type = body.get("media_type").and_then(|v| v.as_str()).unwrap_or("text").to_string();
    let primary_media_url = body.get("primary_media_url").and_then(|v| v.as_str()).map(String::from);
    let media_caption = body.get("media_caption").and_then(|v| v.as_str()).map(String::from);
    let link_url = body.get("link_url").and_then(|v| v.as_str()).map(String::from);
    let link_title = body.get("link_title").and_then(|v| v.as_str()).map(String::from);
    let link_description = body.get("link_description").and_then(|v| v.as_str()).map(String::from);
    let link_image_url = body.get("link_image_url").and_then(|v| v.as_str()).map(String::from);
    let target_type = body.get("target_type").and_then(|v| v.as_str()).unwrap_or("contacts").to_string();
    let target_ids = body.get("target_ids").map(|v| v.to_string()).unwrap_or_else(|| "[]".to_string());
    let channels = body.get("channels").map(|v| v.to_string()).unwrap_or_else(|| "[]".to_string());
    let scheduled_at = body.get("scheduled_at").and_then(|v| v.as_str()).map(String::from);

    if title.is_empty() {
        return Err((StatusCode::BAD_REQUEST, Json(json!({ "error": "Title is required" }))));
    }
    if body_text.is_empty() {
        return Err((StatusCode::BAD_REQUEST, Json(json!({ "error": "Body text is required" }))));
    }

    match Broadcast::create(
        &state.db, &organization_id, brand_id.as_deref(), &title, &created_by,
        &body_text, body_html.as_deref(), subject_line.as_deref(), preview_text.as_deref(),
        &media_type, primary_media_url.as_deref(), media_caption.as_deref(),
        link_url.as_deref(), link_title.as_deref(), link_description.as_deref(),
        link_image_url.as_deref(), &target_type, &target_ids, &channels,
        scheduled_at.as_deref(),
    ).await {
        Ok(broadcast) => Ok(Json(json!(broadcast))),
        Err(e) => {
            tracing::error!("Failed to create broadcast: {}", e);
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e }))))
        }
    }
}

pub async fn update_broadcast(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<Value>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let updates = match body.as_object().cloned() {
        Some(map) => map,
        None => return Err((StatusCode::BAD_REQUEST, Json(json!({ "error": "Invalid body" })))),
    };

    match Broadcast::update(&state.db, &id, updates).await {
        Ok(broadcast) => Ok(Json(json!(broadcast))),
        Err(e) => {
            tracing::error!("Failed to update broadcast {}: {}", id, e);
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e }))))
        }
    }
}

pub async fn delete_broadcast(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match Broadcast::delete(&state.db, &id).await {
        Ok(()) => Ok(Json(json!({ "success": true }))),
        Err(e) => {
            tracing::error!("Failed to delete broadcast {}: {}", id, e);
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e }))))
        }
    }
}

// ---------------------------------------------------------------------------
// Broadcast status
// ---------------------------------------------------------------------------

pub async fn update_broadcast_status(
    State(state): State<AppState>,
    Path((id, status)): Path<(String, String)>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match Broadcast::update_status(&state.db, &id, &status).await {
        Ok(broadcast) => Ok(Json(json!(broadcast))),
        Err(e) => {
            tracing::error!("Failed to update status for broadcast {}: {}", id, e);
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e }))))
        }
    }
}

// ---------------------------------------------------------------------------
// Send broadcast
// ---------------------------------------------------------------------------

pub async fn send_broadcast_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match send_broadcast(&state.db, &id).await {
        Ok(result) => Ok(Json(json!(result))),
        Err(e) => {
            tracing::error!("Failed to send broadcast {}: {}", id, e);
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e }))))
        }
    }
}

// ---------------------------------------------------------------------------
// Format preview
// ---------------------------------------------------------------------------

pub async fn get_format_preview_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match get_format_preview(&state.db, &id).await {
        Ok(preview) => Ok(Json(json!(preview))),
        Err(e) => {
            tracing::error!("Failed to get format preview for broadcast {}: {}", id, e);
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e }))))
        }
    }
}

// ---------------------------------------------------------------------------
// Available channels
// ---------------------------------------------------------------------------

pub async fn list_available_channels() -> Json<Value> {
    let channels = get_available_channels();
    Json(json!({ "channels": channels }))
}

// ---------------------------------------------------------------------------
// Media management
// ---------------------------------------------------------------------------

pub async fn upload_media(
    State(state): State<AppState>,
    Json(body): Json<Value>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let broadcast_id = body.get("broadcast_id").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let media_type = body.get("media_type").and_then(|v| v.as_str()).unwrap_or("image").to_string();
    let file_name = body.get("file_name").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let file_size_bytes: i64 = body.get("file_size_bytes").and_then(|v| v.as_i64()).unwrap_or(0);
    let mime_type = body.get("mime_type").and_then(|v| v.as_str()).unwrap_or("application/octet-stream").to_string();
    let file_url = body.get("file_url").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let thumbnail_url = body.get("thumbnail_url").and_then(|v| v.as_str()).map(String::from);

    if broadcast_id.is_empty() || file_url.is_empty() {
        return Err((StatusCode::BAD_REQUEST, Json(json!({ "error": "broadcast_id and file_url are required" }))));
    }

    match BroadcastMedia::upload(
        &state.db, &broadcast_id, &media_type, &file_name, file_size_bytes,
        &mime_type, &file_url, thumbnail_url.as_deref(),
    ).await {
        Ok(media) => Ok(Json(json!(media))),
        Err(e) => {
            tracing::error!("Failed to upload media: {}", e);
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e }))))
        }
    }
}

pub async fn list_media(
    State(state): State<AppState>,
    Path(broadcast_id): Path<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match BroadcastMedia::list_by_broadcast(&state.db, &broadcast_id).await {
        Ok(media) => Ok(Json(json!({ "media": media }))),
        Err(e) => {
            tracing::error!("Failed to list media for broadcast {}: {}", broadcast_id, e);
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e }))))
        }
    }
}

pub async fn delete_media(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match BroadcastMedia::delete(&state.db, &id).await {
        Ok(()) => Ok(Json(json!({ "success": true }))),
        Err(e) => {
            tracing::error!("Failed to delete media {}: {}", id, e);
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e }))))
        }
    }
}

// ---------------------------------------------------------------------------
// Broadcast channels
// ---------------------------------------------------------------------------

pub async fn list_broadcast_channels(
    State(state): State<AppState>,
    Path(broadcast_id): Path<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match BroadcastChannel::list_by_broadcast(&state.db, &broadcast_id).await {
        Ok(channels) => Ok(Json(json!({ "channels": channels }))),
        Err(e) => {
            tracing::error!("Failed to list channels for broadcast {}: {}", broadcast_id, e);
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e }))))
        }
    }
}

// ---------------------------------------------------------------------------
// Broadcast recipients
// ---------------------------------------------------------------------------

pub async fn list_broadcast_recipients(
    State(state): State<AppState>,
    Path(broadcast_id): Path<String>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let status = params.get("status").map(|s| s.as_str());
    let limit: i32 = params.get("limit").and_then(|s| s.parse().ok()).unwrap_or(100);

    match BroadcastRecipient::list_by_broadcast(&state.db, &broadcast_id, status, limit).await {
        Ok(recipients) => Ok(Json(json!({ "recipients": recipients }))),
        Err(e) => {
            tracing::error!("Failed to list recipients for broadcast {}: {}", broadcast_id, e);
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e }))))
        }
    }
}

// ---------------------------------------------------------------------------
// Broadcast logs
// ---------------------------------------------------------------------------

pub async fn list_broadcast_logs(
    State(state): State<AppState>,
    Path(broadcast_id): Path<String>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let limit: i32 = params.get("limit").and_then(|s| s.parse().ok()).unwrap_or(50);

    match BroadcastLog::list_by_broadcast(&state.db, &broadcast_id, limit).await {
        Ok(logs) => Ok(Json(json!({ "logs": logs }))),
        Err(e) => {
            tracing::error!("Failed to list logs for broadcast {}: {}", broadcast_id, e);
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e }))))
        }
    }
}

// ---------------------------------------------------------------------------
// Settings
// ---------------------------------------------------------------------------

pub async fn get_settings(
    State(state): State<AppState>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let org_id = params.get("organization_id").unwrap_or(&"default".to_string()).clone();

    match BroadcastSettings::get(&state.db, &org_id).await {
        Ok(settings) => Ok(Json(json!(settings))),
        Err(e) => {
            tracing::error!("Failed to get settings: {}", e);
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e }))))
        }
    }
}

pub async fn update_settings(
    State(state): State<AppState>,
    Query(params): Query<std::collections::HashMap<String, String>>,
    Json(body): Json<Value>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let org_id = params.get("organization_id").unwrap_or(&"default".to_string()).clone();
    let updates = match body.as_object().cloned() {
        Some(map) => map,
        None => return Err((StatusCode::BAD_REQUEST, Json(json!({ "error": "Invalid body" })))),
    };

    match BroadcastSettings::update(&state.db, &org_id, updates).await {
        Ok(settings) => Ok(Json(json!(settings))),
        Err(e) => {
            tracing::error!("Failed to update settings: {}", e);
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e }))))
        }
    }
}

// ---------------------------------------------------------------------------
// HTML-to-all-channels preview (no broadcast needed)
// ---------------------------------------------------------------------------

pub async fn preview_html_for_channels(
    Json(body): Json<Value>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let html = body.get("html").and_then(|v| v.as_str()).unwrap_or("");
    let body_text = body.get("bodyText").and_then(|v| v.as_str()).unwrap_or("");
    let channels: Vec<String> = body.get("channels")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
        .unwrap_or_else(|| {
            // Default: all channels
            vec!["whatsapp".into(), "email".into(), "sms".into(), "instagram".into(),
                 "facebook".into(), "telegram".into(), "discord".into(), "slack".into()]
        });

    if html.is_empty() && body_text.is_empty() {
        return Err((StatusCode::BAD_REQUEST, Json(json!({ "error": "Provide html or bodyText" }))));
    }

    let formats = preview_html_for_all_channels(html, body_text, &channels);
    Ok(Json(json!({ "formats": formats })))
}
