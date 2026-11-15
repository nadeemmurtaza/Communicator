//! Crypto Mesh API — Thin gRPC client forwarding to svc-crypto-mesh; REST shape unchanged.

use axum::extract::{State, Json};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde_json::{json, Value};

use crate::db::AppState;

use newax_proto::crypto_mesh::v1::{
    crypto_mesh_service_client::CryptoMeshServiceClient,
    EgressRequest, Empty, IngressRequest, JsonResponse,
};

async fn client() -> Result<CryptoMeshServiceClient<tonic::transport::Channel>, (StatusCode, String)> {
    let url = std::env::var("CRYPTO_MESH_SERVICE_URL").unwrap_or_else(|_| "http://127.0.0.1:50085".to_string());
    CryptoMeshServiceClient::connect(url)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("crypto-mesh service unavailable: {e}")))
}

fn grpc_err(e: tonic::Status) -> (StatusCode, String) {
    (StatusCode::INTERNAL_SERVER_ERROR, e.message().to_string())
}

fn bad_request_err(e: tonic::Status) -> (StatusCode, String) {
    (StatusCode::BAD_REQUEST, e.message().to_string())
}

fn parse_json(s: &str) -> Result<serde_json::Value, (StatusCode, String)> {
    serde_json::from_str(s).map_err(|e| (StatusCode::BAD_REQUEST, format!("invalid json: {e}")))
}

/// `POST /crypto-mesh/ingress`
pub async fn ingress(
    State(_state): State<AppState>,
    Json(envelope): Json<Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut c = client().await?;

    let version = envelope.get("version").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let tenant_id = envelope.get("tenantId").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let sender_id = envelope.get("senderId").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let recipient_id = envelope.get("recipientId").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let iv = envelope.get("iv").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let ciphertext = envelope.get("ciphertext").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let auth_tag = envelope.get("authTag").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let key_id = envelope.get("keyId").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let timestamp = envelope.get("timestamp").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let signature = envelope.get("signature").and_then(|v| v.as_str()).unwrap_or("").to_string();

    let resp = c
        .ingress(IngressRequest {
            version,
            tenant_id,
            sender_id,
            recipient_id,
            iv,
            ciphertext,
            auth_tag,
            key_id,
            timestamp,
            signature,
        })
        .await
        .map_err(bad_request_err)?;

    let body = parse_json(&resp.into_inner().json)?;
    Ok(Json(body))
}

/// `POST /crypto-mesh/egress`
pub async fn egress(
    State(_state): State<AppState>,
    Json(body): Json<Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut c = client().await?;

    let name = body.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let tenant_id = body.get("tenantId").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let recipient_id = body
        .get("recipientId")
        .or_else(|| body.get("targetModule"))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let organization_id = body.get("organizationId").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let correlation_id = body.get("correlationId").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let payload = body.get("payload").cloned().unwrap_or_else(|| json!({}));

    let resp = c
        .egress(EgressRequest {
            name,
            tenant_id,
            recipient_id,
            organization_id,
            correlation_id,
            payload: payload.to_string(),
        })
        .await
        .map_err(bad_request_err)?;

    let body = parse_json(&resp.into_inner().json)?;
    Ok(Json(body))
}

/// `GET /crypto-mesh/outbox`
pub async fn outbox(
    State(_state): State<AppState>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut c = client().await?;

    let resp = c
        .list_outbox(Empty {})
        .await
        .map_err(grpc_err)?;

    let body = parse_json(&resp.into_inner().json)?;
    Ok(Json(body))
}

/// `POST /crypto-mesh/outbox/flush`
pub async fn flush_outbox(
    State(_state): State<AppState>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut c = client().await?;

    let resp = c
        .flush_outbox(Empty {})
        .await
        .map_err(grpc_err)?;

    let body = parse_json(&resp.into_inner().json)?;
    Ok(Json(body))
}
