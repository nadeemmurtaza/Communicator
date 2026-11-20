//! Organization routes — thin gRPC client forwarding to svc-organization.

use axum::{
    extract::{Path, State, Json},
    http::StatusCode,
    routing::{get, put},
    Router,
};
use crate::db::AppState;
use serde_json::Value;

type OrgClient =
    newax_proto::organization::v1::organization_service_client::OrganizationServiceClient<tonic::transport::Channel>;

async fn client(state: &AppState) -> Result<OrgClient, (StatusCode, String)> {
    let endpoint =
        std::env::var("ORGANIZATION_GRPC_URL").unwrap_or("http://127.0.0.1:50121".to_string());
    OrgClient::connect(endpoint)
        .await
        .map_err(|e| (StatusCode::BAD_GATEWAY, format!("organization grpc: {}", e)))
}

pub async fn get_profile(
    State(state): State<AppState>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let mut c = match client(&state).await {
        Ok(c) => c,
        Err(e) => return Err((e.0, Json(serde_json::json!({ "error": e.1 })))),
    };
    let resp = c
        .get_profile(newax_proto::organization::v1::EmptyRequest {})
        .await
        .map_err(|e| (StatusCode::BAD_GATEWAY, Json(serde_json::json!({ "error": e.to_string() }))))?;
    let parsed: Value = serde_json::from_str(&resp.into_inner().json)
        .map_err(|e| (StatusCode::BAD_GATEWAY, Json(serde_json::json!({ "error": e.to_string() }))))?;
    Ok(Json(parsed))
}

pub async fn update_profile(
    State(state): State<AppState>,
    Json(body): Json<Value>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let mut c = match client(&state).await {
        Ok(c) => c,
        Err(e) => return Err((e.0, Json(serde_json::json!({ "error": e.1 })))),
    };
    let json_str = serde_json::to_string(&body)
        .map_err(|e| (StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": e.to_string() }))))?;
    let resp = c
        .update_profile(newax_proto::organization::v1::UpdateProfileRequest { json: json_str })
        .await
        .map_err(|e| (StatusCode::BAD_GATEWAY, Json(serde_json::json!({ "error": e.to_string() }))))?;
    let parsed: Value = serde_json::from_str(&resp.into_inner().json)
        .map_err(|e| (StatusCode::BAD_GATEWAY, Json(serde_json::json!({ "error": e.to_string() }))))?;
    Ok(Json(parsed))
}

pub async fn get_settings(
    State(state): State<AppState>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let mut c = match client(&state).await {
        Ok(c) => c,
        Err(e) => return Err((e.0, Json(serde_json::json!({ "error": e.1 })))),
    };
    let resp = c
        .get_settings(newax_proto::organization::v1::EmptyRequest {})
        .await
        .map_err(|e| (StatusCode::BAD_GATEWAY, Json(serde_json::json!({ "error": e.to_string() }))))?;
    let parsed: Value = serde_json::from_str(&resp.into_inner().json)
        .map_err(|e| (StatusCode::BAD_GATEWAY, Json(serde_json::json!({ "error": e.to_string() }))))?;
    Ok(Json(parsed))
}

pub async fn update_settings(
    State(state): State<AppState>,
    Json(body): Json<Value>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let mut c = match client(&state).await {
        Ok(c) => c,
        Err(e) => return Err((e.0, Json(serde_json::json!({ "error": e.1 })))),
    };
    let json_str = serde_json::to_string(&body)
        .map_err(|e| (StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": e.to_string() }))))?;
    let resp = c
        .update_settings(newax_proto::organization::v1::UpdateSettingsRequest { json: json_str })
        .await
        .map_err(|e| (StatusCode::BAD_GATEWAY, Json(serde_json::json!({ "error": e.to_string() }))))?;
    let parsed: Value = serde_json::from_str(&resp.into_inner().json)
        .map_err(|e| (StatusCode::BAD_GATEWAY, Json(serde_json::json!({ "error": e.to_string() }))))?;
    Ok(Json(parsed))
}

pub async fn get_stats(
    State(state): State<AppState>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let mut c = match client(&state).await {
        Ok(c) => c,
        Err(e) => return Err((e.0, Json(serde_json::json!({ "error": e.1 })))),
    };
    let resp = c
        .get_stats(newax_proto::organization::v1::EmptyRequest {})
        .await
        .map_err(|e| (StatusCode::BAD_GATEWAY, Json(serde_json::json!({ "error": e.to_string() }))))?;
    let parsed: Value = serde_json::from_str(&resp.into_inner().json)
        .map_err(|e| (StatusCode::BAD_GATEWAY, Json(serde_json::json!({ "error": e.to_string() }))))?;
    Ok(Json(parsed))
}

pub async fn verify_channel(
    State(state): State<AppState>,
    Path(channel): Path<String>,
) -> Json<Value> {
    let mut c = match client(&state).await {
        Ok(c) => c,
        Err(e) => return Json(serde_json::json!({ "error": e.1 })),
    };
    let resp = match c
        .verify_channel(newax_proto::organization::v1::VerifyChannelRequest { channel })
        .await
    {
        Ok(r) => r,
        Err(e) => return Json(serde_json::json!({ "error": e.to_string() })),
    };
    let parsed: Value = serde_json::from_str(&resp.into_inner().json).unwrap_or_default();
    Json(parsed)
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/organization/profile", get(get_profile).put(update_profile))
        .route("/organization/settings", get(get_settings).put(update_settings))
        .route("/organization/stats", get(get_stats))
        .route("/organization/channels/:channel/verify", get(verify_channel))
}
