//! Omni-Channel Broadcast routes — thin gRPC client forwarding to svc-omni-broadcast.

use serde_json::{json, Value};
use axum::{
    extract::{Path, Query, State, Json},
    http::StatusCode,
};
use crate::db::AppState;
use newax_proto::omni_broadcast::v1::omni_broadcast_service_client::OmniBroadcastServiceClient;

async fn client(_state: &AppState) -> Result<OmniBroadcastServiceClient<tonic::transport::Channel>, (StatusCode, Json<Value>)> {
    let url = std::env::var("OMNI_BROADCAST_GRPC_URL").unwrap_or_else(|_| "http://127.0.0.1:50096".to_string());
    OmniBroadcastServiceClient::connect(url)
        .await
        .map_err(|e| {
            tracing::error!("omni_broadcast gRPC connect failed: {}", e);
            (StatusCode::SERVICE_UNAVAILABLE, Json(json!({ "error": "omni_broadcast service unavailable" })))
        })
}

// ---------------------------------------------------------------------------
// Summary
// ---------------------------------------------------------------------------

pub async fn get_summary(
    State(state): State<AppState>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let mut cli = client(&state).await?;
    let resp = cli.get_summary(newax_proto::omni_broadcast::v1::Empty {}).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() }))))?;
    let val: Value = serde_json::from_str(&resp.into_inner().json)
        .unwrap_or(json!({}));
    Ok(Json(val))
}

// ---------------------------------------------------------------------------
// Broadcast CRUD
// ---------------------------------------------------------------------------

pub async fn list_broadcasts(
    State(state): State<AppState>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let status = params.get("status").map(|s| s.as_str().to_string()).unwrap_or_default();
    let limit: i32 = params.get("limit").and_then(|s| s.parse().ok()).unwrap_or(50);
    let mut cli = client(&state).await?;
    let resp = cli.list_broadcasts(newax_proto::omni_broadcast::v1::ListBroadcastsRequest { status, limit }).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() }))))?;
    let val: Value = serde_json::from_str(&resp.into_inner().json).unwrap_or(json!({}));
    Ok(Json(val))
}

pub async fn get_broadcast(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let mut cli = client(&state).await?;
    let resp = cli.get_broadcast(newax_proto::omni_broadcast::v1::GetBroadcastRequest { id }).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() }))))?;
    let val: Value = serde_json::from_str(&resp.into_inner().json).unwrap_or(json!({}));
    Ok(Json(val))
}

pub async fn create_broadcast(
    State(state): State<AppState>,
    Json(body): Json<Value>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let organization_id = body.get("organization_id").and_then(|v| v.as_str()).unwrap_or("default").to_string();
    let brand_id = body.get("brand_id").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let title = body.get("title").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let created_by = body.get("created_by").and_then(|v| v.as_str()).unwrap_or("system").to_string();
    let body_text = body.get("body_text").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let body_html = body.get("body_html").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let subject_line = body.get("subject_line").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let preview_text = body.get("preview_text").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let media_type = body.get("media_type").and_then(|v| v.as_str()).unwrap_or("text").to_string();
    let primary_media_url = body.get("primary_media_url").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let media_caption = body.get("media_caption").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let link_url = body.get("link_url").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let link_title = body.get("link_title").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let link_description = body.get("link_description").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let link_image_url = body.get("link_image_url").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let target_type = body.get("target_type").and_then(|v| v.as_str()).unwrap_or("contacts").to_string();
    let target_ids = body.get("target_ids").map(|v| v.to_string()).unwrap_or_else(|| "[]".to_string());
    let channels = body.get("channels").map(|v| v.to_string()).unwrap_or_else(|| "[]".to_string());
    let scheduled_at = body.get("scheduled_at").and_then(|v| v.as_str()).unwrap_or("").to_string();

    if title.is_empty() {
        return Err((StatusCode::BAD_REQUEST, Json(json!({ "error": "Title is required" }))));
    }
    if body_text.is_empty() {
        return Err((StatusCode::BAD_REQUEST, Json(json!({ "error": "Body text is required" }))));
    }

    let mut cli = client(&state).await?;
    let resp = cli.create_broadcast(newax_proto::omni_broadcast::v1::CreateBroadcastRequest {
        organization_id, brand_id, title, created_by, body_text, body_html, subject_line, preview_text,
        media_type, primary_media_url, media_caption, link_url, link_title, link_description,
        link_image_url, target_type, target_ids, channels, scheduled_at,
    }).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() }))))?;
    let val: Value = serde_json::from_str(&resp.into_inner().json).unwrap_or(json!({}));
    Ok(Json(val))
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
    let updates_json = serde_json::to_string(&updates).unwrap_or_default();
    let mut cli = client(&state).await?;
    let resp = cli.update_broadcast(newax_proto::omni_broadcast::v1::UpdateBroadcastRequest { id, updates_json }).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() }))))?;
    let val: Value = serde_json::from_str(&resp.into_inner().json).unwrap_or(json!({}));
    Ok(Json(val))
}

pub async fn delete_broadcast(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let mut cli = client(&state).await?;
    let resp = cli.delete_broadcast(newax_proto::omni_broadcast::v1::DeleteBroadcastRequest { id }).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() }))))?;
    let val: Value = serde_json::from_str(&resp.into_inner().json).unwrap_or(json!({}));
    Ok(Json(val))
}

// ---------------------------------------------------------------------------
// Broadcast status
// ---------------------------------------------------------------------------

pub async fn update_broadcast_status(
    State(state): State<AppState>,
    Path((id, status)): Path<(String, String)>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let mut cli = client(&state).await?;
    let resp = cli.update_broadcast_status(newax_proto::omni_broadcast::v1::UpdateBroadcastStatusRequest { id, status }).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() }))))?;
    let val: Value = serde_json::from_str(&resp.into_inner().json).unwrap_or(json!({}));
    Ok(Json(val))
}

// ---------------------------------------------------------------------------
// Send broadcast
// ---------------------------------------------------------------------------

pub async fn send_broadcast_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let mut cli = client(&state).await?;
    let resp = cli.send_broadcast(newax_proto::omni_broadcast::v1::SendBroadcastRequest { id }).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() }))))?;
    let val: Value = serde_json::from_str(&resp.into_inner().json).unwrap_or(json!({}));
    Ok(Json(val))
}

// ---------------------------------------------------------------------------
// Format preview
// ---------------------------------------------------------------------------

pub async fn get_format_preview_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let mut cli = client(&state).await?;
    let resp = cli.get_format_preview(newax_proto::omni_broadcast::v1::GetFormatPreviewRequest { id }).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() }))))?;
    let val: Value = serde_json::from_str(&resp.into_inner().json).unwrap_or(json!({}));
    Ok(Json(val))
}

// ---------------------------------------------------------------------------
// Available channels
// ---------------------------------------------------------------------------

pub async fn list_available_channels() -> Json<Value> {
    Json(json!({ "channels": [] }))
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
    let thumbnail_url = body.get("thumbnail_url").and_then(|v| v.as_str()).unwrap_or("").to_string();

    if broadcast_id.is_empty() || file_url.is_empty() {
        return Err((StatusCode::BAD_REQUEST, Json(json!({ "error": "broadcast_id and file_url are required" }))));
    }

    let mut cli = client(&state).await?;
    let resp = cli.upload_media(newax_proto::omni_broadcast::v1::UploadMediaRequest {
        broadcast_id, media_type, file_name, file_size_bytes, mime_type, file_url, thumbnail_url,
    }).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() }))))?;
    let val: Value = serde_json::from_str(&resp.into_inner().json).unwrap_or(json!({}));
    Ok(Json(val))
}

pub async fn list_media(
    State(state): State<AppState>,
    Path(broadcast_id): Path<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let mut cli = client(&state).await?;
    let resp = cli.list_media(newax_proto::omni_broadcast::v1::ListMediaRequest { broadcast_id }).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() }))))?;
    let val: Value = serde_json::from_str(&resp.into_inner().json).unwrap_or(json!({}));
    Ok(Json(val))
}

pub async fn delete_media(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let mut cli = client(&state).await?;
    let resp = cli.delete_media(newax_proto::omni_broadcast::v1::DeleteMediaRequest { id }).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() }))))?;
    let val: Value = serde_json::from_str(&resp.into_inner().json).unwrap_or(json!({}));
    Ok(Json(val))
}

// ---------------------------------------------------------------------------
// Broadcast channels
// ---------------------------------------------------------------------------

pub async fn list_broadcast_channels(
    State(state): State<AppState>,
    Path(broadcast_id): Path<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let mut cli = client(&state).await?;
    let resp = cli.list_broadcast_channels(newax_proto::omni_broadcast::v1::ListBroadcastChannelsRequest { broadcast_id }).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() }))))?;
    let val: Value = serde_json::from_str(&resp.into_inner().json).unwrap_or(json!({}));
    Ok(Json(val))
}

// ---------------------------------------------------------------------------
// Broadcast recipients
// ---------------------------------------------------------------------------

pub async fn list_broadcast_recipients(
    State(state): State<AppState>,
    Path(broadcast_id): Path<String>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let status = params.get("status").map(|s| s.as_str().to_string()).unwrap_or_default();
    let limit: i32 = params.get("limit").and_then(|s| s.parse().ok()).unwrap_or(100);
    let mut cli = client(&state).await?;
    let resp = cli.list_broadcast_recipients(newax_proto::omni_broadcast::v1::ListBroadcastRecipientsRequest {
        broadcast_id, status, limit,
    }).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() }))))?;
    let val: Value = serde_json::from_str(&resp.into_inner().json).unwrap_or(json!({}));
    Ok(Json(val))
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
    let mut cli = client(&state).await?;
    let resp = cli.list_broadcast_logs(newax_proto::omni_broadcast::v1::ListBroadcastLogsRequest {
        broadcast_id, limit,
    }).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() }))))?;
    let val: Value = serde_json::from_str(&resp.into_inner().json).unwrap_or(json!({}));
    Ok(Json(val))
}

// ---------------------------------------------------------------------------
// Settings
// ---------------------------------------------------------------------------

pub async fn get_settings(
    State(state): State<AppState>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let org_id = params.get("organization_id").unwrap_or(&"default".to_string()).clone();
    let mut cli = client(&state).await?;
    let resp = cli.get_settings(newax_proto::omni_broadcast::v1::GetSettingsRequest { organization_id: org_id }).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() }))))?;
    let val: Value = serde_json::from_str(&resp.into_inner().json).unwrap_or(json!({}));
    Ok(Json(val))
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
    let updates_json = serde_json::to_string(&updates).unwrap_or_default();
    let mut cli = client(&state).await?;
    let resp = cli.update_settings(newax_proto::omni_broadcast::v1::UpdateSettingsRequest {
        organization_id: org_id, updates_json,
    }).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() }))))?;
    let val: Value = serde_json::from_str(&resp.into_inner().json).unwrap_or(json!({}));
    Ok(Json(val))
}

// ---------------------------------------------------------------------------
// HTML-to-all-channels preview
// ---------------------------------------------------------------------------

pub async fn preview_html_for_channels(
    State(state): State<AppState>,
    Json(body): Json<Value>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let html = body.get("html").and_then(|v| v.as_str()).unwrap_or("");
    let body_text = body.get("bodyText").and_then(|v| v.as_str()).unwrap_or("");
    let channels: Vec<String> = body.get("channels")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
        .unwrap_or_else(|| {
            vec!["whatsapp".into(), "email".into(), "sms".into(), "instagram".into(),
                 "facebook".into(), "telegram".into(), "discord".into(), "slack".into()]
        });

    if html.is_empty() && body_text.is_empty() {
        return Err((StatusCode::BAD_REQUEST, Json(json!({ "error": "Provide html or bodyText" }))));
    }

    let mut cli = client(&state).await?;
    let resp = cli.preview_html_for_channels(newax_proto::omni_broadcast::v1::PreviewHtmlForChannelsRequest {
        html: html.to_string(), body_text: body_text.to_string(), channels,
    }).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() }))))?;
    let val: Value = serde_json::from_str(&resp.into_inner().json).unwrap_or(json!({}));
    Ok(Json(val))
}
