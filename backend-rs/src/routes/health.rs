//! Health check route — thin gRPC client forwarding to svc-health.

use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use crate::db::AppState;
use newax_proto::health::v1::health_service_client::HealthServiceClient;
use newax_proto::health::v1::EmptyRequest;
use tonic::transport::Channel;

#[derive(Clone)]
struct HealthClient(HealthServiceClient<Channel>);

async fn client(state: &AppState) -> Result<HealthClient, String> {
    let url = if state.config.health_grpc_url.is_empty() {
        "http://[::1]:50110"
    } else {
        &state.config.health_grpc_url
    };
    let channel = Channel::from_shared(url.to_string())
        .map_err(|e| format!("Invalid gRPC URL: {}", e))?
        .connect()
        .await
        .map_err(|e| format!("Failed to connect to health service: {}", e))?;
    Ok(HealthClient(HealthServiceClient::new(channel)))
}

/// Liveness/readiness probe forwarded to svc-health.
pub async fn health_check(State(state): State<AppState>) -> impl IntoResponse {
    match client(&state).await {
        Ok(mut c) => {
            match c.0.check(EmptyRequest {}).await {
                Ok(resp) => {
                    let json_str = resp.into_inner().json;
                    match serde_json::from_str::<serde_json::Value>(&json_str) {
                        Ok(val) => (StatusCode::OK, Json(val)).into_response(),
                        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": "Invalid JSON from health service" }))).into_response(),
                    }
                }
                Err(e) => {
                    (StatusCode::SERVICE_UNAVAILABLE, Json(serde_json::json!({
                        "status": "degraded",
                        "database": {
                            "reachable": false,
                            "error": format!("gRPC error: {}", e),
                        },
                        "checkedAt": chrono::Utc::now().to_rfc3339(),
                    }))).into_response()
                }
            }
        }
        Err(e) => {
            (StatusCode::SERVICE_UNAVAILABLE, Json(serde_json::json!({
                "status": "degraded",
                "database": {
                    "reachable": false,
                    "error": format!("Failed to connect to health service: {}", e),
                },
                "checkedAt": chrono::Utc::now().to_rfc3339(),
            }))).into_response()
        }
    }
}