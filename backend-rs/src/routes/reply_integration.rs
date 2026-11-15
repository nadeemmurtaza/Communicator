//! Thin gRPC client forwarding to svc-reply-integration; REST shape unchanged.

use axum::extract::{Path, Query, State};
use axum::Json;
use crate::db::AppState;

use newax_proto::reply_integration::v1::{
    reply_integration_service_client::ReplyIntegrationServiceClient, BuildContextRequest as GrpcBuildContextRequest,
    Empty, GetSourcesRequest as GrpcGetSourcesRequest, UpdateSettingsRequest as GrpcUpdateSettingsRequest,
};

async fn client() -> Result<
    ReplyIntegrationServiceClient<tonic::transport::Channel>,
    (axum::http::StatusCode, Json<serde_json::Value>),
> {
    let url = std::env::var("REPLY_INTEGRATION_SERVICE_URL").unwrap_or_else(|_| "http://127.0.0.1:50068".to_string());
    ReplyIntegrationServiceClient::connect(url).await.map_err(|e| {
        (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": format!("reply-integration service unavailable: {e}") })),
        )
    })
}

fn grpc_err(e: tonic::Status) -> (axum::http::StatusCode, Json<serde_json::Value>) {
    (
        axum::http::StatusCode::INTERNAL_SERVER_ERROR,
        Json(serde_json::json!({ "error": e.message().to_string() })),
    )
}

fn parse_json(s: &str) -> serde_json::Value {
    serde_json::from_str(s).unwrap_or_default()
}

/// GET /reply-integration/summary
pub async fn get_summary(
    State(_state): State<AppState>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    let mut c = client().await?;
    let resp = c.get_summary(Empty {}).await.map_err(grpc_err)?;
    Ok(Json(parse_json(&resp.into_inner().json)))
}

/// GET /reply-integration/settings
pub async fn get_settings(
    State(_state): State<AppState>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    let mut c = client().await?;
    let resp = c.get_settings(Empty {}).await.map_err(grpc_err)?;
    Ok(Json(parse_json(&resp.into_inner().json)))
}

/// PUT /reply-integration/settings
pub async fn update_settings(
    State(_state): State<AppState>,
    Json(settings): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    let mut c = client().await?;
    let resp = c
        .update_settings(GrpcUpdateSettingsRequest { json: settings.to_string() })
        .await
        .map_err(grpc_err)?;
    Ok(Json(parse_json(&resp.into_inner().json)))
}

/// POST /reply-integration/build-context
pub async fn build_context(
    State(_state): State<AppState>,
    Json(body): Json<BuildContextRequest>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    let mut c = client().await?;
    let resp = c
        .build_context(GrpcBuildContextRequest {
            contact_id: body.contact_id,
            last_inbound: body.last_inbound,
            tenant_key: body.tenant_key,
        })
        .await
        .map_err(grpc_err)?;
    Ok(Json(parse_json(&resp.into_inner().json)))
}

/// GET /reply-integration/sources
pub async fn get_knowledge_sources(
    State(_state): State<AppState>,
    Query(params): Query<GetSourcesQuery>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    let mut c = client().await?;
    let resp = c
        .get_knowledge_sources(GrpcGetSourcesRequest {
            draft_id: params.draft_id,
            conversation_id: params.conversation_id,
            source_type: params.source_type,
        })
        .await
        .map_err(grpc_err)?;
    Ok(Json(parse_json(&resp.into_inner().json)))
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
