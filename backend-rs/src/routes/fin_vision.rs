//! Fin Vision — thin gRPC client forwarding to svc-fin-vision

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;
use serde_json::Value;

use crate::db::AppState;

use newax_proto::fin_vision::v1::fin_vision_service_client::FinVisionServiceClient;
use newax_proto::fin_vision::v1::{
    Empty, UploadImageRequest, AnalyzeImageRequest, ListImagesRequest,
    IdRequest, GetSettingsRequest, UpdateSettingsRequest, JsonBody,
};

#[derive(Deserialize, Default)]
pub struct ImageQueryParams {
    pub organization_id: Option<String>,
    pub status: Option<String>,
    pub limit: Option<i64>,
}

#[derive(Deserialize, Default)]
pub struct SettingsQueryParams {
    pub organization_id: Option<String>,
}

async fn client(state: &AppState) -> Result<FinVisionServiceClient<tonic::transport::Channel>, (StatusCode, String)> {
    let url = if state.config.fin_vision_grpc_url.is_empty() {
        "http://127.0.0.1:50105".to_string()
    } else {
        state.config.fin_vision_grpc_url.clone()
    };
    FinVisionServiceClient::connect(url)
        .await
        .map_err(|e| (StatusCode::SERVICE_UNAVAILABLE, format!("gRPC connection failed: {}", e)))
}

fn parse_json_resp(resp: tonic::Response<newax_proto::fin_vision::v1::JsonResponse>) -> Value {
    let json_str = resp.into_inner().json;
    serde_json::from_str(&json_str).unwrap_or_else(|_| serde_json::json!({"raw": json_str}))
}

/// GET /vision/summary
pub async fn get_summary(
    State(state): State<AppState>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let mut c = client(&state).await?;
    let resp = c.get_summary(tonic::Request::new(Empty {})).await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    Ok(Json(parse_json_resp(resp)))
}

/// POST /vision/upload
pub async fn upload_image(
    State(state): State<AppState>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let image_data = payload.get("imageData").and_then(|v| v.as_str()).unwrap_or("");
    let mime_type = payload.get("mimeType").and_then(|v| v.as_str()).unwrap_or("image/jpeg");
    let file_size = payload.get("fileSize").and_then(|v| v.as_i64()).unwrap_or(0);
    let conversation_id = payload.get("conversationId").and_then(|v| v.as_str()).unwrap_or("");
    let contact_id = payload.get("contactId").and_then(|v| v.as_str()).unwrap_or("");
    let channel = payload.get("channel").and_then(|v| v.as_str()).unwrap_or("");
    let organization_id = payload.get("organizationId").and_then(|v| v.as_str()).unwrap_or("org-default");

    if image_data.is_empty() {
        return Ok(Json(serde_json::json!({"error": "imageData is required (base64 encoded)"})));
    }

    let mut c = client(&state).await?;
    let req = UploadImageRequest {
        image_data: image_data.to_string(),
        mime_type: mime_type.to_string(),
        file_size,
        organization_id: organization_id.to_string(),
        conversation_id: conversation_id.to_string(),
        contact_id: contact_id.to_string(),
        channel: channel.to_string(),
    };
    let resp = c.upload_image(tonic::Request::new(req)).await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    Ok(Json(parse_json_resp(resp)))
}

/// POST /vision/analyze/{image_id}
pub async fn analyze_image(
    State(state): State<AppState>,
    Path(image_id): Path<String>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let analysis_type = payload.get("analysisType").and_then(|v| v.as_str()).unwrap_or("general");
    let prompt = payload.get("prompt").and_then(|v| v.as_str()).unwrap_or("");
    let claude_model = payload.get("claudeModel").and_then(|v| v.as_str()).unwrap_or("");

    let mut c = client(&state).await?;
    let req = AnalyzeImageRequest {
        image_id,
        analysis_type: analysis_type.to_string(),
        prompt: prompt.to_string(),
        claude_model: claude_model.to_string(),
    };
    let resp = c.analyze_image(tonic::Request::new(req)).await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    Ok(Json(parse_json_resp(resp)))
}

/// GET /vision/images
pub async fn list_images(
    State(state): State<AppState>,
    Query(params): Query<ImageQueryParams>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let organization_id = params.organization_id.unwrap_or_else(|| "org-default".to_string());
    let status = params.status.unwrap_or_default();
    let limit = params.limit.unwrap_or(50);

    let mut c = client(&state).await?;
    let req = ListImagesRequest {
        organization_id,
        status,
        limit,
    };
    let resp = c.list_images(tonic::Request::new(req)).await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    Ok(Json(parse_json_resp(resp)))
}

/// GET /vision/images/{image_id}
pub async fn get_image(
    State(state): State<AppState>,
    Path(image_id): Path<String>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let mut c = client(&state).await?;
    let req = IdRequest { id: image_id };
    let resp = c.get_image(tonic::Request::new(req)).await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    Ok(Json(parse_json_resp(resp)))
}

/// DELETE /vision/images/{image_id}
pub async fn delete_image(
    State(state): State<AppState>,
    Path(image_id): Path<String>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let mut c = client(&state).await?;
    let req = IdRequest { id: image_id };
    let resp = c.delete_image(tonic::Request::new(req)).await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    Ok(Json(parse_json_resp(resp)))
}

/// GET /vision/settings
pub async fn get_settings(
    State(state): State<AppState>,
    Query(params): Query<SettingsQueryParams>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let organization_id = params.organization_id.unwrap_or_else(|| "org-default".to_string());
    let mut c = client(&state).await?;
    let req = GetSettingsRequest { organization_id };
    let resp = c.get_settings(tonic::Request::new(req)).await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    Ok(Json(parse_json_resp(resp)))
}

/// PUT /vision/settings
pub async fn update_settings(
    State(state): State<AppState>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let organization_id = payload.get("organizationId").and_then(|v| v.as_str()).unwrap_or("org-default");
    let json_str = serde_json::to_string(&payload).map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;

    let mut c = client(&state).await?;
    let req = UpdateSettingsRequest {
        organization_id: organization_id.to_string(),
        body: Some(JsonBody { json: json_str }),
    };
    let resp = c.update_settings(tonic::Request::new(req)).await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    Ok(Json(parse_json_resp(resp)))
}