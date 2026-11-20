//! Shopify Store OTP Integration API Routes (gRPC client)

use crate::db::AppState;
use newax_shopify_store_integration::{
    CreateShopifyConnectionRequest, ShopifyAccountCreationRequest,
    ShopifyOtpVerificationRequest, UpdateChannelPreferenceRequest,
};
use axum::{
    extract::{Path, State, ConnectInfo, Json},
    http::{HeaderMap, StatusCode},
    response::Json as JsonResponse,
};
use std::net::SocketAddr;
use serde::Deserialize;

fn shopify_store_client_url(state: &AppState) -> String {
    state.config.shopify_store_grpc_url.clone()
}

// ---------------------------------------------------------------------------
// Store connection management (admin endpoints)
// ---------------------------------------------------------------------------

pub async fn create_shopify_connection(
    State(state): State<AppState>,
    Json(body): Json<CreateShopifyConnectionRequest>,
) -> (StatusCode, JsonResponse<serde_json::Value>) {
    let url = shopify_store_client_url(&state);
    let mut client = match newax_proto::shopify_store::v1::shopify_store_service_client::ShopifyStoreServiceClient::connect(url).await {
        Ok(c) => c,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, JsonResponse(serde_json::json!({ "error": format!("Failed to connect: {}", e) }))),
    };

    let resp = match client.create_connection(
        newax_proto::shopify_store::v1::CreateConnectionRequest {
            brand_id: body.brand_id,
            store_domain: body.store_domain,
            store_name: body.store_name,
            shopify_api_key: body.shopify_api_key.unwrap_or_default(),
            shopify_api_secret: body.shopify_api_secret.unwrap_or_default(),
            shopify_access_token: body.shopify_access_token.unwrap_or_default(),
            shopify_webhook_secret: body.shopify_webhook_secret.unwrap_or_default(),
            auto_create_customer: body.auto_create_customer.map(|b| b.to_string()).unwrap_or_default(),
            customer_tag: body.customer_tag.unwrap_or_default(),
            default_purpose: body.default_purpose.unwrap_or_default(),
        },
    ).await {
        Ok(r) => r.into_inner(),
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, JsonResponse(serde_json::json!({ "error": e.to_string() }))),
    };

    match serde_json::from_str(&resp.json) {
        Ok(v) => (StatusCode::CREATED, JsonResponse(v)),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, JsonResponse(serde_json::json!({ "error": "Invalid response" }))),
    }
}

pub async fn list_shopify_connections(
    State(state): State<AppState>,
    Path(brand_id): Path<String>,
) -> (StatusCode, JsonResponse<serde_json::Value>) {
    let url = shopify_store_client_url(&state);
    let mut client = match newax_proto::shopify_store::v1::shopify_store_service_client::ShopifyStoreServiceClient::connect(url).await {
        Ok(c) => c,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, JsonResponse(serde_json::json!({ "error": format!("Failed to connect: {}", e) }))),
    };

    let resp = match client.list_connections(
        newax_proto::shopify_store::v1::ListConnectionsRequest { brand_id },
    ).await {
        Ok(r) => r.into_inner(),
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, JsonResponse(serde_json::json!({ "error": e.to_string() }))),
    };

    match serde_json::from_str(&resp.json) {
        Ok(v) => (StatusCode::OK, JsonResponse(v)),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, JsonResponse(serde_json::json!({ "error": "Invalid response" }))),
    }
}

pub async fn get_channel_preferences(
    State(state): State<AppState>,
    Path(connection_id): Path<String>,
) -> (StatusCode, JsonResponse<serde_json::Value>) {
    let url = shopify_store_client_url(&state);
    let mut client = match newax_proto::shopify_store::v1::shopify_store_service_client::ShopifyStoreServiceClient::connect(url).await {
        Ok(c) => c,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, JsonResponse(serde_json::json!({ "error": format!("Failed to connect: {}", e) }))),
    };

    let resp = match client.get_channel_preferences(
        newax_proto::shopify_store::v1::GetChannelPreferencesRequest {
            connection_id,
            purpose: "sign_up".to_string(),
        },
    ).await {
        Ok(r) => r.into_inner(),
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, JsonResponse(serde_json::json!({ "error": e.to_string() }))),
    };

    match serde_json::from_str(&resp.json) {
        Ok(v) => (StatusCode::OK, JsonResponse(v)),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, JsonResponse(serde_json::json!({ "error": "Invalid response" }))),
    }
}

pub async fn update_channel_preferences(
    State(state): State<AppState>,
    Path(connection_id): Path<String>,
    Json(body): Json<UpdateChannelPreferenceRequest>,
) -> (StatusCode, JsonResponse<serde_json::Value>) {
    let url = shopify_store_client_url(&state);
    let mut client = match newax_proto::shopify_store::v1::shopify_store_service_client::ShopifyStoreServiceClient::connect(url).await {
        Ok(c) => c,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, JsonResponse(serde_json::json!({ "error": format!("Failed to connect: {}", e) }))),
    };

    let resp = match client.update_channel_preferences(
        newax_proto::shopify_store::v1::UpdateChannelPreferencesRequest {
            connection_id,
            purpose: body.purpose.unwrap_or_default(),
            whatsapp_enabled: body.whatsapp_enabled.map(|b| b.to_string()).unwrap_or_default(),
            email_enabled: body.email_enabled.map(|b| b.to_string()).unwrap_or_default(),
            sms_enabled: body.sms_enabled.map(|b| b.to_string()).unwrap_or_default(),
            in_app_enabled: body.in_app_enabled.map(|b| b.to_string()).unwrap_or_default(),
            default_channel: body.default_channel.unwrap_or_default(),
            priority_order: body.priority_order.map(|v| serde_json::to_string(&v).unwrap_or_default()).unwrap_or_default(),
        },
    ).await {
        Ok(r) => r.into_inner(),
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, JsonResponse(serde_json::json!({ "error": e.to_string() }))),
    };

    match serde_json::from_str(&resp.json) {
        Ok(v) => (StatusCode::OK, JsonResponse(v)),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, JsonResponse(serde_json::json!({ "error": "Invalid response" }))),
    }
}

// ---------------------------------------------------------------------------
// Account creation flow (store-facing endpoints)
// ---------------------------------------------------------------------------

pub async fn create_shopify_account(
    State(state): State<AppState>,
    Path(store_domain): Path<String>,
    headers: HeaderMap,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Json(body): Json<ShopifyAccountCreationRequest>,
) -> (StatusCode, JsonResponse<serde_json::Value>) {
    let ip_address = addr.ip().to_string();
    let user_agent = headers.get("user-agent").and_then(|v| v.to_str().ok()).map(|s| s.to_string()).unwrap_or_default();

    let url = shopify_store_client_url(&state);
    let mut client = match newax_proto::shopify_store::v1::shopify_store_service_client::ShopifyStoreServiceClient::connect(url).await {
        Ok(c) => c,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, JsonResponse(serde_json::json!({ "error": format!("Failed to connect: {}", e) }))),
    };

    let resp = match client.create_account(
        newax_proto::shopify_store::v1::CreateAccountRequest {
            store_domain,
            email: body.email,
            phone: body.phone.unwrap_or_default(),
            first_name: body.first_name.unwrap_or_default(),
            last_name: body.last_name.unwrap_or_default(),
            channel: body.channel.unwrap_or_default(),
            metadata: body.metadata.map(|v| serde_json::to_string(&v).unwrap_or_default()).unwrap_or_default(),
            ip_address,
            user_agent,
        },
    ).await {
        Ok(r) => r.into_inner(),
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, JsonResponse(serde_json::json!({ "error": e.to_string() }))),
    };

    match serde_json::from_str::<serde_json::Value>(&resp.json) {
        Ok(v) => {
            let status = if v.get("success").and_then(|s| s.as_bool()) == Some(true) {
                StatusCode::OK
            } else {
                StatusCode::BAD_REQUEST
            };
            (status, JsonResponse(v))
        },
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, JsonResponse(serde_json::json!({ "error": "Invalid response" }))),
    }
}

pub async fn verify_shopify_account(
    State(state): State<AppState>,
    headers: HeaderMap,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Json(body): Json<ShopifyOtpVerificationRequest>,
) -> (StatusCode, JsonResponse<serde_json::Value>) {
    let ip_address = addr.ip().to_string();
    let user_agent = headers.get("user-agent").and_then(|v| v.to_str().ok()).map(|s| s.to_string()).unwrap_or_default();

    let url = shopify_store_client_url(&state);
    let mut client = match newax_proto::shopify_store::v1::shopify_store_service_client::ShopifyStoreServiceClient::connect(url).await {
        Ok(c) => c,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, JsonResponse(serde_json::json!({ "error": format!("Failed to connect: {}", e) }))),
    };

    let resp = match client.verify_account(
        newax_proto::shopify_store::v1::VerifyAccountRequest {
            pending_id: body.pending_id,
            code: body.code,
            ip_address,
            user_agent,
        },
    ).await {
        Ok(r) => r.into_inner(),
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, JsonResponse(serde_json::json!({ "error": e.to_string() }))),
    };

    match serde_json::from_str::<serde_json::Value>(&resp.json) {
        Ok(v) => {
            let status = if v.get("verified").and_then(|s| s.as_bool()) == Some(true) {
                StatusCode::OK
            } else {
                StatusCode::BAD_REQUEST
            };
            (status, JsonResponse(v))
        },
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, JsonResponse(serde_json::json!({ "error": "Invalid response" }))),
    }
}

#[derive(Debug, Deserialize)]
pub struct ResendOtpRequest {
    pub pending_id: String,
}

pub async fn resend_shopify_otp(
    State(state): State<AppState>,
    headers: HeaderMap,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Json(body): Json<ResendOtpRequest>,
) -> (StatusCode, JsonResponse<serde_json::Value>) {
    let ip_address = addr.ip().to_string();
    let user_agent = headers.get("user-agent").and_then(|v| v.to_str().ok()).map(|s| s.to_string()).unwrap_or_default();

    let url = shopify_store_client_url(&state);
    let mut client = match newax_proto::shopify_store::v1::shopify_store_service_client::ShopifyStoreServiceClient::connect(url).await {
        Ok(c) => c,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, JsonResponse(serde_json::json!({ "error": format!("Failed to connect: {}", e) }))),
    };

    let resp = match client.resend_otp(
        newax_proto::shopify_store::v1::ResendOtpRequest {
            pending_id: body.pending_id,
            ip_address,
            user_agent,
        },
    ).await {
        Ok(r) => r.into_inner(),
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, JsonResponse(serde_json::json!({ "error": e.to_string() }))),
    };

    match serde_json::from_str::<serde_json::Value>(&resp.json) {
        Ok(v) => {
            let status = if v.get("success").and_then(|s| s.as_bool()) == Some(true) {
                StatusCode::OK
            } else {
                StatusCode::BAD_REQUEST
            };
            (status, JsonResponse(v))
        },
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, JsonResponse(serde_json::json!({ "error": "Invalid response" }))),
    }
}

// ---------------------------------------------------------------------------
// Shopify webhook receiver
// ---------------------------------------------------------------------------

pub async fn receive_shopify_webhook(
    State(state): State<AppState>,
    Path(connection_id): Path<String>,
    headers: HeaderMap,
    body: axum::body::Bytes,
) -> (StatusCode, JsonResponse<serde_json::Value>) {
    let signature = headers.get("X-Shopify-Hmac-Sha256").and_then(|v| v.to_str().ok()).map(|s| s.to_string()).unwrap_or_default();
    let topic = headers.get("X-Shopify-Topic").and_then(|v| v.to_str().ok()).unwrap_or("unknown");
    let shop_domain = headers.get("X-Shopify-Shop-Domain").and_then(|v| v.to_str().ok()).unwrap_or("");
    let webhook_id = headers.get("X-Shopify-Webhook-Id").and_then(|v| v.to_str().ok()).unwrap_or("");
    let payload: serde_json::Value = serde_json::from_slice(&body).unwrap_or_else(|_| serde_json::json!({}));

    let url = shopify_store_client_url(&state);
    let mut client = match newax_proto::shopify_store::v1::shopify_store_service_client::ShopifyStoreServiceClient::connect(url).await {
        Ok(c) => c,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, JsonResponse(serde_json::json!({ "error": format!("Failed to connect: {}", e) }))),
    };

    let resp = match client.receive_webhook(
        newax_proto::shopify_store::v1::ReceiveWebhookRequest {
            connection_id,
            signature,
            topic: topic.to_string(),
            shop_domain: shop_domain.to_string(),
            webhook_id: webhook_id.to_string(),
            payload: serde_json::to_string(&payload).unwrap_or_default(),
        },
    ).await {
        Ok(r) => r.into_inner(),
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, JsonResponse(serde_json::json!({ "error": e.to_string() }))),
    };

    match serde_json::from_str(&resp.json) {
        Ok(v) => (StatusCode::OK, JsonResponse(v)),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, JsonResponse(serde_json::json!({ "error": "Invalid response" }))),
    }
}
