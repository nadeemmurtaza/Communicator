//! External OTP Integration — thin gRPC client forwarding to svc-external-otp (port 50108).

use axum::extract::{Path, Query, State, ConnectInfo, Json};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use serde_json::{json, Value};
use std::net::SocketAddr;

use crate::db::AppState;

// ---------------------------------------------------------------------------
// Client
// ---------------------------------------------------------------------------

type Client = newax_proto::external_otp::v1::external_otp_service_client::ExternalOtpServiceClient<tonic::transport::Channel>;

async fn client(state: &AppState) -> Result<Client, (StatusCode, String)> {
    let url = if state.config.external_otp_grpc_url.is_empty() {
        "http://localhost:50108".to_string()
    } else {
        state.config.external_otp_grpc_url.clone()
    };
    Client::connect(url)
        .await
        .map_err(|e| (StatusCode::SERVICE_UNAVAILABLE, format!("svc-external-otp unavailable: {e}")))
}

fn resp(body: Value) -> Response {
    (StatusCode::OK, Json(body)).into_response()
}

fn err(status: StatusCode, message: String) -> Response {
    (status, Json(json!({ "success": false, "error": message }))).into_response()
}

// ---------------------------------------------------------------------------
// Authentication middleware helper
// ---------------------------------------------------------------------------

async fn authenticate_integration(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<String, Response> {
    let auth_header = headers
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| err(StatusCode::UNAUTHORIZED, "Missing Authorization header".to_string()))?;

    let secret_key = if auth_header.starts_with("Bearer ") {
        &auth_header[7..]
    } else if auth_header.starts_with("bearer ") {
        &auth_header[7..]
    } else {
        return Err(err(StatusCode::UNAUTHORIZED, "Invalid Authorization format. Use: Bearer {secret_key}".to_string()));
    };

    let mut client = client(state).await.map_err(|(_, e)| err(StatusCode::SERVICE_UNAVAILABLE, e))?;
    let result = client
        .verify_secret_key(newax_proto::external_otp::v1::SecretKeyRequest {
            secret_key: secret_key.to_string(),
        })
        .await;

    match result {
        Ok(resp) => {
            let body: Value = serde_json::from_str(&resp.into_inner().json)
                .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
            if body.get("success").and_then(|v| v.as_bool()).unwrap_or(false) {
                Ok(secret_key.to_string())
            } else {
                Err(err(StatusCode::UNAUTHORIZED, "Invalid secret key".to_string()))
            }
        }
        Err(e) => Err(err(StatusCode::UNAUTHORIZED, e.to_string())),
    }
}

// ---------------------------------------------------------------------------
// Routes
// ---------------------------------------------------------------------------

/// POST /api/v1/external-otp/integrations
pub async fn create_integration(
    State(state): State<AppState>,
    Json(body): Json<Value>,
) -> Response {
    let mut client = match client(&state).await {
        Ok(c) => c,
        Err(e) => return err(e.0, e.1),
    };

    let result = client
        .create_integration(newax_proto::external_otp::v1::CreateIntegrationRequest {
            body: Some(newax_proto::external_otp::v1::JsonBody {
                json: body.to_string(),
            }),
        })
        .await;

    match result {
        Ok(resp) => {
            let body: Value = serde_json::from_str(&resp.into_inner().json)
                .unwrap_or_else(|_| json!({ "success": false }));
            if body.get("success").and_then(|v| v.as_bool()).unwrap_or(false) {
                (StatusCode::CREATED, Json(body)).into_response()
            } else {
                err(StatusCode::INTERNAL_SERVER_ERROR, body.get("error").and_then(|v| v.as_str()).unwrap_or("").to_string())
            }
        }
        Err(e) => err(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
    }
}

/// GET /api/v1/external-otp/integrations/{brand_id}
pub async fn list_integrations(
    State(state): State<AppState>,
    Path(brand_id): Path<String>,
) -> Response {
    let mut client = match client(&state).await {
        Ok(c) => c,
        Err(e) => return err(e.0, e.1),
    };

    let result = client
        .list_integrations(newax_proto::external_otp::v1::IdRequest { id: brand_id })
        .await;

    match result {
        Ok(resp_result) => {
            let body: Value = serde_json::from_str(&resp_result.into_inner().json)
                .unwrap_or_else(|_| json!({ "success": false }));
            resp(body)
        }
        Err(e) => err(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
    }
}

/// DELETE /api/v1/external-otp/integrations/{integration_id}
pub async fn deactivate_integration(
    State(state): State<AppState>,
    Path(integration_id): Path<String>,
) -> Response {
    let mut client = match client(&state).await {
        Ok(c) => c,
        Err(e) => return err(e.0, e.1),
    };

    let result = client
        .deactivate_integration(newax_proto::external_otp::v1::IdRequest { id: integration_id })
        .await;

    match result {
        Ok(resp_result) => {
            let body: Value = serde_json::from_str(&resp_result.into_inner().json)
                .unwrap_or_else(|_| json!({ "success": false }));
            resp(body)
        }
        Err(e) => err(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
    }
}

/// POST /api/v1/external-otp/generate
pub async fn external_generate_otp(
    State(state): State<AppState>,
    headers: HeaderMap,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Json(body): Json<Value>,
) -> Response {
    let secret_key = match authenticate_integration(&state, &headers).await {
        Ok(sk) => sk,
        Err(e) => return e,
    };

    let ip_address = addr.ip().to_string();
    let user_agent = headers
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
        .unwrap_or_default();

    let mut client = match client(&state).await {
        Ok(c) => c,
        Err(e) => return err(e.0, e.1),
    };

    let result = client
        .external_generate(newax_proto::external_otp::v1::ExternalGenerateRequest {
            body: Some(newax_proto::external_otp::v1::JsonBody {
                json: body.to_string(),
            }),
            secret_key,
            ip_address,
            user_agent,
        })
        .await;

    match result {
        Ok(resp) => {
            let body: Value = serde_json::from_str(&resp.into_inner().json)
                .unwrap_or_else(|_| json!({ "success": false }));
            let status = if body.get("success").and_then(|v| v.as_bool()).unwrap_or(false) {
                StatusCode::OK
            } else {
                StatusCode::BAD_REQUEST
            };
            (status, Json(body)).into_response()
        }
        Err(e) => err(StatusCode::BAD_REQUEST, e.to_string()),
    }
}

/// POST /api/v1/external-otp/verify
pub async fn external_verify_otp(
    State(state): State<AppState>,
    headers: HeaderMap,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Json(body): Json<Value>,
) -> Response {
    let secret_key = match authenticate_integration(&state, &headers).await {
        Ok(sk) => sk,
        Err(e) => return e,
    };

    let ip_address = addr.ip().to_string();
    let user_agent = headers
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
        .unwrap_or_default();

    let mut client = match client(&state).await {
        Ok(c) => c,
        Err(e) => return err(e.0, e.1),
    };

    let result = client
        .external_verify(newax_proto::external_otp::v1::ExternalVerifyRequest {
            body: Some(newax_proto::external_otp::v1::JsonBody {
                json: body.to_string(),
            }),
            secret_key,
            ip_address,
            user_agent,
        })
        .await;

    match result {
        Ok(resp) => {
            let body: Value = serde_json::from_str(&resp.into_inner().json)
                .unwrap_or_else(|_| json!({ "success": false }));
            let status = if body.get("verified").and_then(|v| v.as_bool()).unwrap_or(false) {
                StatusCode::OK
            } else {
                StatusCode::BAD_REQUEST
            };
            (status, Json(body)).into_response()
        }
        Err(e) => err(StatusCode::BAD_REQUEST, e.to_string()),
    }
}

// ---------------------------------------------------------------------------
// Integration info endpoint
// ---------------------------------------------------------------------------

#[derive(Debug, serde::Deserialize)]
pub struct IntegrationInfoQuery {
    pub public_key: String,
}

/// GET /api/v1/external-otp/info?public_key=pk_...
pub async fn get_integration_info(
    State(state): State<AppState>,
    Query(query): Query<IntegrationInfoQuery>,
) -> Response {
    let mut client = match client(&state).await {
        Ok(c) => c,
        Err(e) => return err(e.0, e.1),
    };

    let result = client
        .get_integration_info(newax_proto::external_otp::v1::PublicKeyRequest {
            public_key: query.public_key,
        })
        .await;

    match result {
        Ok(resp_result) => {
            let body: Value = serde_json::from_str(&resp_result.into_inner().json)
                .unwrap_or_else(|_| json!({ "success": false }));
            if body.get("success").and_then(|v| v.as_bool()).unwrap_or(false) {
                resp(body)
            } else {
                err(StatusCode::NOT_FOUND, body.get("error").and_then(|v| v.as_str()).unwrap_or("").to_string())
            }
        }
        Err(e) => err(StatusCode::NOT_FOUND, e.to_string()),
    }
}
