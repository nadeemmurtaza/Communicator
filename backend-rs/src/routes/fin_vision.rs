//! Fin Vision API Routes
//!
//! Endpoints for image upload, AI analysis, and vision settings management.

use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use serde_json::{json, Value};

use crate::db::AppState;
use crate::services::fin_vision;

/// GET /vision/summary
/// Returns overview of vision usage: total images, analyses, confidence, recent analyses.
pub async fn get_summary(State(state): State<AppState>) -> Json<Value> {
    match fin_vision::get_vision_summary(&state).await {
        Ok(summary) => Json(serde_json::to_value(summary).unwrap_or(json!({"error": "serialization failed"}))),
        Err(e) => Json(json!({"error": e})),
    }
}

/// POST /vision/upload
/// Upload an image for AI analysis.
/// Body: { imageData: string (base64), mimeType: string, fileSize: number,
///         conversationId?: string, contactId?: string, channel?: string }
pub async fn upload_image(
    State(state): State<AppState>,
    Json(payload): Json<Value>,
) -> Json<Value> {
    let image_data = payload.get("imageData").and_then(|v| v.as_str()).unwrap_or("");
    let mime_type = payload.get("mimeType").and_then(|v| v.as_str()).unwrap_or("image/jpeg");
    let file_size = payload.get("fileSize").and_then(|v| v.as_i64()).unwrap_or(0);
    let conversation_id = payload.get("conversationId").and_then(|v| v.as_str());
    let contact_id = payload.get("contactId").and_then(|v| v.as_str());
    let channel = payload.get("channel").and_then(|v| v.as_str());
    let org_id = payload.get("organizationId").and_then(|v| v.as_str()).unwrap_or("org-default");

    if image_data.is_empty() {
        return Json(json!({"error": "imageData is required (base64 encoded)"}));
    }

    match fin_vision::upload_image(
        &state, org_id,
        conversation_id, contact_id, channel,
        image_data, mime_type, file_size,
    )
    .await
    {
        Ok(result) => Json(serde_json::to_value(result).unwrap_or(json!({"error": "serialization failed"}))),
        Err(e) => Json(json!({"error": e})),
    }
}

/// POST /vision/analyze/{image_id}
/// Analyze an uploaded image with AI.
/// Body: { analysisType?: string, prompt?: string, claudeModel?: string }
pub async fn analyze_image(
    State(state): State<AppState>,
    axum::extract::Path(image_id): axum::extract::Path<String>,
    Json(payload): Json<Value>,
) -> Json<Value> {
    let analysis_type = payload.get("analysisType").and_then(|v| v.as_str()).unwrap_or("general");
    let prompt = payload.get("prompt").and_then(|v| v.as_str()).map(|s| s.to_string());
    let claude_model = payload.get("claudeModel").and_then(|v| v.as_str()).map(|s| s.to_string());

    match fin_vision::analyze_image(&state, &image_id, analysis_type, prompt, claude_model).await {
        Ok(result) => Json(serde_json::to_value(result).unwrap_or(json!({"error": "serialization failed"}))),
        Err(e) => Json(json!({"error": e})),
    }
}

/// GET /vision/images
/// List uploaded images with optional status filter.
/// Query: ?status=uploaded&limit=50
pub async fn list_images(
    State(state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Json<Value> {
    let status_filter = params.get("status").map(|s| s.as_str());
    let limit: i64 = params.get("limit").and_then(|s| s.parse().ok()).unwrap_or(50);
    let org_id = params.get("organizationId").unwrap_or(&"org-default".to_string());

    match fin_vision::list_images(&state, org_id, status_filter, limit).await {
        Ok(images) => Json(serde_json::to_value(images).unwrap_or(json!({"error": "serialization failed"}))),
        Err(e) => Json(json!({"error": e})),
    }
}

/// GET /vision/images/{image_id}
/// Get a specific image and its analyses.
pub async fn get_image(
    State(state): State<AppState>,
    axum::extract::Path(image_id): axum::extract::Path<String>,
) -> Json<Value> {
    match fin_vision::get_image_with_analysis(&state, &image_id).await {
        Ok((image, analyses)) => Json(json!({
            "image": image,
            "analyses": analyses,
        })),
        Err(e) => Json(json!({"error": e})),
    }
}

/// DELETE /vision/images/{image_id}
/// Delete an image and its analyses.
pub async fn delete_image(
    State(state): State<AppState>,
    axum::extract::Path(image_id): axum::extract::Path<String>,
) -> impl axum::response::IntoResponse {
    match fin_vision::delete_image(&state, &image_id).await {
        Ok(()) => {
            let body = Json(json!({"success": true, "message": "Image deleted"}));
            (StatusCode::OK, body)
        }
        Err(e) => {
            let body = Json(json!({"error": e}));
            (StatusCode::BAD_REQUEST, body)
        }
    }
}

/// GET /vision/settings
/// Get vision settings for an organization.
pub async fn get_settings(
    State(state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Json<Value> {
    let org_id = params.get("organizationId").unwrap_or(&"org-default".to_string());

    match fin_vision::get_vision_settings(&state, org_id).await {
        Some(settings) => Json(serde_json::to_value(settings).unwrap_or(json!({"error": "serialization failed"}))),
        None => Json(json!({"error": "Settings not found"})),
    }
}

/// PUT /vision/settings
/// Update vision settings for an organization.
pub async fn update_settings(
    State(state): State<AppState>,
    Json(payload): Json<Value>,
) -> Json<Value> {
    let org_id = payload.get("organizationId").and_then(|v| v.as_str()).unwrap_or("org-default");

    match fin_vision::update_vision_settings(&state, org_id, &payload).await {
        Ok(settings) => Json(serde_json::to_value(settings).unwrap_or(json!({"error": "serialization failed"}))),
        Err(e) => Json(json!({"error": e})),
    }
}
