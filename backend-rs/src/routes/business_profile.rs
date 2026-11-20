//! Business Profile routes — thin gRPC client forwarding to svc-business-profile.

use crate::db::AppState;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::Value;
use newax_proto::business_profile::v1::business_profile_service_client::BusinessProfileServiceClient;
use newax_proto::business_profile::v1::{EmptyRequest, CreateRequest};

async fn client(
    state: &AppState,
) -> Result<BusinessProfileServiceClient<tonic::transport::Channel>, (StatusCode, Json<Value>)> {
    let url = if state.config.business_profile_grpc_url.is_empty() {
        "http://127.0.0.1:50115".to_string()
    } else {
        state.config.business_profile_grpc_url.clone()
    };
    BusinessProfileServiceClient::connect(url)
        .await
        .map_err(|e| {
            tracing::error!("business_profile gRPC connect failed: {}", e);
            (StatusCode::SERVICE_UNAVAILABLE, Json(serde_json::json!({ "error": "business_profile service unavailable" })))
        })
}

pub async fn get_business_profile(
    State(state): State<AppState>,
) -> Response {
    let mut cli = match client(&state).await {
        Ok(c) => c,
        Err((status, json)) => return (status, json).into_response(),
    };
    let resp = cli.get_business_profile(EmptyRequest {}).await;
    match resp {
        Ok(r) => {
            let val: serde_json::Value = serde_json::from_str(&r.into_inner().json).unwrap_or(serde_json::json!({}));
            (StatusCode::OK, Json(val)).into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    }
}

pub async fn update_business_profile(
    State(state): State<AppState>,
    Json(payload): Json<Value>,
) -> Response {
    let mut cli = match client(&state).await {
        Ok(c) => c,
        Err((status, json)) => return (status, json).into_response(),
    };
    let resp = cli.update_business_profile(CreateRequest { json: serde_json::to_string(&payload).unwrap_or_default() }).await;
    match resp {
        Ok(r) => {
            let val: serde_json::Value = serde_json::from_str(&r.into_inner().json).unwrap_or(serde_json::json!({}));
            (StatusCode::OK, Json(val)).into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    }
}

pub async fn sync_meta(
    State(state): State<AppState>,
) -> Response {
    let mut cli = match client(&state).await {
        Ok(c) => c,
        Err((status, json)) => return (status, json).into_response(),
    };
    let resp = cli.sync_meta(EmptyRequest {}).await;
    match resp {
        Ok(r) => {
            let val: serde_json::Value = serde_json::from_str(&r.into_inner().json).unwrap_or(serde_json::json!({}));
            (StatusCode::OK, Json(val)).into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    }
}
    // COALESCE keeps the stored value for a NULL parameter, so an omitted field