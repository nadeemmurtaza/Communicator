use std::env;
use tonic::transport::Server;

pub mod shopify_store {
    pub mod v1 {
        tonic::include_proto!("newax.shopify_store.v1");
    }
}

use shopify_store::v1::shopify_store_service_server::ShopifyStoreService;
use shopify_store::v1::{
    JsonResponse, CreateConnectionRequest, ListConnectionsRequest,
    GetChannelPreferencesRequest, UpdateChannelPreferencesRequest,
    CreateAccountRequest, VerifyAccountRequest, ResendOtpRequest,
    ReceiveWebhookRequest, GetConnectionRequest,
};
use newax_shopify_store_integration::*;

#[derive(Clone)]
pub struct ShopifyStoreServerImpl {
    db: sqlx::PgPool,
}

#[tonic::async_trait]
impl ShopifyStoreService for ShopifyStoreServerImpl {
    async fn create_connection(
        &self,
        request: tonic::Request<CreateConnectionRequest>,
    ) -> Result<tonic::Response<JsonResponse>, tonic::Status> {
        let req = request.into_inner();
        let resp = create_connection(&self.db, &CreateShopifyConnectionRequest {
            brand_id: req.brand_id,
            store_domain: req.store_domain,
            store_name: req.store_name,
            shopify_api_key: if req.shopify_api_key.is_empty() { None } else { Some(req.shopify_api_key) },
            shopify_api_secret: if req.shopify_api_secret.is_empty() { None } else { Some(req.shopify_api_secret) },
            shopify_access_token: if req.shopify_access_token.is_empty() { None } else { Some(req.shopify_access_token) },
            shopify_webhook_secret: if req.shopify_webhook_secret.is_empty() { None } else { Some(req.shopify_webhook_secret) },
            auto_create_customer: if req.auto_create_customer.is_empty() { None } else { Some(req.auto_create_customer == "true") },
            customer_tag: if req.customer_tag.is_empty() { None } else { Some(req.customer_tag) },
            default_purpose: if req.default_purpose.is_empty() { None } else { Some(req.default_purpose) },
        }).await;
        match resp {
            Ok(r) => Ok(tonic::Response::new(JsonResponse { json: serde_json::to_string(&serde_json::json!({ "success": true, "id": r.id, "store_domain": r.store_domain, "public_key": r.public_key, "webhook_url": r.webhook_url, "message": r.message })).unwrap_or_default() })),
            Err(e) => Ok(tonic::Response::new(JsonResponse { json: serde_json::to_string(&serde_json::json!({ "success": false, "error": e })).unwrap_or_default() })),
        }
    }

    async fn list_connections(
        &self,
        request: tonic::Request<ListConnectionsRequest>,
    ) -> Result<tonic::Response<JsonResponse>, tonic::Status> {
        let req = request.into_inner();
        match list_by_brand(&self.db, &req.brand_id).await {
            Ok(conns) => {
                let connections: Vec<serde_json::Value> = conns.iter().map(|c| {
                    serde_json::json!({
                        "id": c.id,
                        "store_domain": c.store_domain,
                        "store_name": c.store_name,
                        "auto_create_customer": c.auto_create_customer,
                        "customer_tag": c.customer_tag,
                        "is_active": c.is_active,
                        "created_at": c.created_at
                    })
                }).collect();
                let json = serde_json::json!({ "success": true, "connections": connections });
                Ok(tonic::Response::new(JsonResponse { json: serde_json::to_string(&json).unwrap_or_default() }))
            },
            Err(e) => Ok(tonic::Response::new(JsonResponse { json: serde_json::to_string(&serde_json::json!({ "success": false, "error": e })).unwrap_or_default() })),
        }
    }

    async fn get_channel_preferences(
        &self,
        request: tonic::Request<GetChannelPreferencesRequest>,
    ) -> Result<tonic::Response<JsonResponse>, tonic::Status> {
        let req = request.into_inner();
        match get_channel_preferences(&self.db, &req.connection_id, &req.purpose).await {
            Some(pref) => Ok(tonic::Response::new(JsonResponse { json: serde_json::to_string(&serde_json::json!({ "success": true, "channel_preferences": { "purpose": pref.purpose, "whatsapp_enabled": pref.whatsapp_enabled, "email_enabled": pref.email_enabled, "sms_enabled": pref.sms_enabled, "in_app_enabled": pref.in_app_enabled, "default_channel": pref.default_channel, "priority_order": pref.priority_order }})).unwrap_or_default() })),
            None => Ok(tonic::Response::new(JsonResponse { json: serde_json::to_string(&serde_json::json!({ "success": false, "error": "Channel preferences not found" })).unwrap_or_default() })),
        }
    }

    async fn update_channel_preferences(
        &self,
        request: tonic::Request<UpdateChannelPreferencesRequest>,
    ) -> Result<tonic::Response<JsonResponse>, tonic::Status> {
        let req = request.into_inner();
        let purpose = if req.purpose.is_empty() { None } else { Some(req.purpose) };
        let whatsapp_enabled = if req.whatsapp_enabled.is_empty() { None } else { Some(req.whatsapp_enabled == "true") };
        let email_enabled = if req.email_enabled.is_empty() { None } else { Some(req.email_enabled == "true") };
        let sms_enabled = if req.sms_enabled.is_empty() { None } else { Some(req.sms_enabled == "true") };
        let in_app_enabled = if req.in_app_enabled.is_empty() { None } else { Some(req.in_app_enabled == "true") };
        let default_channel = if req.default_channel.is_empty() { None } else { Some(req.default_channel) };
        let priority_order = if req.priority_order.is_empty() { None } else { serde_json::from_str(&req.priority_order).ok() };
        match update_channel_preferences(&self.db, &req.connection_id, &UpdateChannelPreferenceRequest { purpose, whatsapp_enabled, email_enabled, sms_enabled, in_app_enabled, default_channel, priority_order }).await {
            Ok(pref) => Ok(tonic::Response::new(JsonResponse { json: serde_json::to_string(&serde_json::json!({ "success": true, "channel_preferences": { "purpose": pref.purpose, "whatsapp_enabled": pref.whatsapp_enabled, "email_enabled": pref.email_enabled, "sms_enabled": pref.sms_enabled, "in_app_enabled": pref.in_app_enabled, "default_channel": pref.default_channel, "priority_order": pref.priority_order }})).unwrap_or_default() })),
            Err(e) => Ok(tonic::Response::new(JsonResponse { json: serde_json::to_string(&serde_json::json!({ "success": false, "error": e })).unwrap_or_default() })),
        }
    }

    async fn create_account(
        &self,
        request: tonic::Request<CreateAccountRequest>,
    ) -> Result<tonic::Response<JsonResponse>, tonic::Status> {
        let req = request.into_inner();
        let resp = initiate_account_creation(&self.db, &req.store_domain, &ShopifyAccountCreationRequest {
            email: req.email,
            phone: if req.phone.is_empty() { None } else { Some(req.phone) },
            first_name: if req.first_name.is_empty() { None } else { Some(req.first_name) },
            last_name: if req.last_name.is_empty() { None } else { Some(req.last_name) },
            channel: if req.channel.is_empty() { None } else { Some(req.channel) },
            metadata: if req.metadata.is_empty() { None } else { serde_json::from_str(&req.metadata).ok() },
        }, &req.ip_address, if req.user_agent.is_empty() { None } else { Some(req.user_agent) }).await;
        match resp {
            Ok(r) => Ok(tonic::Response::new(JsonResponse { json: serde_json::to_string(&serde_json::json!({ "success": r.success, "pending_id": r.pending_id, "otp_id": r.otp_id, "channel": r.channel, "message": r.message, "expires_in_seconds": r.expires_in_seconds, "error": r.error })).unwrap_or_default() })),
            Err(e) => Ok(tonic::Response::new(JsonResponse { json: serde_json::to_string(&serde_json::json!({ "success": false, "error": e })).unwrap_or_default() })),
        }
    }

    async fn verify_account(
        &self,
        request: tonic::Request<VerifyAccountRequest>,
    ) -> Result<tonic::Response<JsonResponse>, tonic::Status> {
        let req = request.into_inner();
        let resp = verify_account_creation(&self.db, &ShopifyOtpVerificationRequest { pending_id: req.pending_id, code: req.code }, &req.ip_address, if req.user_agent.is_empty() { None } else { Some(req.user_agent) }).await;
        match resp {
            Ok(r) => Ok(tonic::Response::new(JsonResponse { json: serde_json::to_string(&serde_json::json!({ "success": r.success, "verified": r.verified, "message": r.message, "customer_created": r.customer_created, "shopify_customer_id": r.shopify_customer_id, "session_token": r.session_token, "error": r.error })).unwrap_or_default() })),
            Err(e) => Ok(tonic::Response::new(JsonResponse { json: serde_json::to_string(&serde_json::json!({ "success": false, "error": e })).unwrap_or_default() })),
        }
    }

    async fn resend_otp(
        &self,
        request: tonic::Request<ResendOtpRequest>,
    ) -> Result<tonic::Response<JsonResponse>, tonic::Status> {
        let req = request.into_inner();
        let resp = resend_otp(&self.db, &req.pending_id, &req.ip_address, if req.user_agent.is_empty() { None } else { Some(req.user_agent) }).await;
        match resp {
            Ok(r) => Ok(tonic::Response::new(JsonResponse { json: serde_json::to_string(&serde_json::json!({ "success": r.success, "pending_id": r.pending_id, "otp_id": r.otp_id, "channel": r.channel, "message": r.message, "expires_in_seconds": r.expires_in_seconds, "error": r.error })).unwrap_or_default() })),
            Err(e) => Ok(tonic::Response::new(JsonResponse { json: serde_json::to_string(&serde_json::json!({ "success": false, "error": e })).unwrap_or_default() })),
        }
    }

    async fn receive_webhook(
        &self,
        request: tonic::Request<ReceiveWebhookRequest>,
    ) -> Result<tonic::Response<JsonResponse>, tonic::Status> {
        let req = request.into_inner();
        let payload: serde_json::Value = serde_json::from_str(&req.payload).unwrap_or_else(|_| serde_json::json!({}));
        let connection = get_by_id(&self.db, &req.connection_id).await;
        let verified = if let Ok(conn) = &connection {
            if !req.signature.is_empty() {
                verify_shopify_signature(req.payload.as_bytes(), &req.signature, &conn.shopify_webhook_secret)
            } else {
                false
            }
        } else {
            false
        };
        let sig_str = req.signature.clone();
        match log_webhook_event(&self.db, Some(&req.connection_id), &req.topic, &req.shop_domain, &req.webhook_id, &payload, &sig_str, verified).await {
            Ok(_) => Ok(tonic::Response::new(JsonResponse { json: serde_json::to_string(&serde_json::json!({ "success": true, "message": "Webhook received" })).unwrap_or_default() })),
            Err(e) => Ok(tonic::Response::new(JsonResponse { json: serde_json::to_string(&serde_json::json!({ "success": false, "error": e })).unwrap_or_default() })),
        }
    }

    async fn get_connection(
        &self,
        request: tonic::Request<GetConnectionRequest>,
    ) -> Result<tonic::Response<JsonResponse>, tonic::Status> {
        let req = request.into_inner();
        match get_by_id(&self.db, &req.connection_id).await {
            Ok(conn) => Ok(tonic::Response::new(JsonResponse { json: serde_json::to_string(&serde_json::json!({ "success": true, "connection": { "id": conn.id, "store_domain": conn.store_domain, "store_name": conn.store_name, "is_active": conn.is_active }})).unwrap_or_default() })),
            Err(e) => Ok(tonic::Response::new(JsonResponse { json: serde_json::to_string(&serde_json::json!({ "success": false, "error": e })).unwrap_or_default() })),
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let port = env::var("PORT").unwrap_or_else(|_| "50108".to_string());
    let addr = format!("0.0.0.0:{}", port).parse().expect("invalid address");

    println!("svc-shopify-store listening on {}", addr);

    let pool = sqlx::PgPool::connect(&database_url).await
        .expect("failed to connect to database");

    let server = ShopifyStoreServerImpl { db: pool };

    Server::builder()
        .add_service(shopify_store::v1::shopify_store_service_server::ShopifyStoreServiceServer::new(server))
        .serve(addr)
        .await
        .expect("gRPC server failed");

    Ok(())
}
