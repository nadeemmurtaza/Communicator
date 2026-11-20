use std::env;

#[derive(Clone, Debug)]
pub struct Config {
    pub database_url: String,
    pub host: String,
    pub port: u16,
    pub static_dir: String,
    pub app_env: String,

    // Meta WhatsApp Cloud API v21.0
    pub meta_whatsapp_access_token: String,
    pub meta_whatsapp_phone_number_id: String,
    pub meta_whatsapp_api_version: String,
    pub meta_webhook_verify_token: String,
    pub meta_app_secret: String,
    pub meta_instagram_app_secret: String,

    // Inbound webhook authentication for the remaining providers. Each one is
    // empty by default: an unconfigured provider refuses its webhook rather than
    // accepting unsigned traffic.
    pub postmark_inbound_webhook_secret: String,
    pub email_inbound_webhook_secret: String,
    pub tiktok_webhook_verify_token: String,
    pub tiktok_app_secret: String,
    pub ecosystem_gateway_key: String,

    // Outbound TikTok token. Distinct from tiktok_app_secret, which only signs
    // inbound webhooks and is not usable as an API credential.
    pub tiktok_access_token: String,

    // Twilio SMS / WhatsApp
    pub twilio_account_sid: String,
    pub twilio_auth_token: String,
    pub twilio_sms_from: String,
    pub twilio_messaging_service_sid: String,

    // AI & Harvester
    pub anthropic_api_key: String,
    pub anthropic_model: String,
    pub harvester_api_base_url: String,
    /// Auto-send gate for AI drafts. Unset means no AI draft is ever sent
    /// without a human: a compiled-in threshold would ship replies the
    /// operator never chose.
    pub reply_auto_send_confidence: Option<f64>,
    /// Slack-compatible incoming webhook for hot-lead alerts. Empty means the
    /// alert is recorded as not-sent rather than posted somewhere guessed.
    pub sales_alert_webhook_url: String,
    /// gRPC address for the standalone access-control service. Empty defaults to 127.0.0.1:50093.
    pub access_control_grpc_url: String,
    /// gRPC address for the standalone mcp-protocol service. Empty defaults to 127.0.0.1:50097.
    pub mcp_protocol_grpc_url: String,
    /// gRPC address for the standalone otp-engine service. Empty defaults to 127.0.0.1:50099.
    pub otp_engine_grpc_url: String,
    /// gRPC address for the standalone nid-tenant-sync service. Empty defaults to 127.0.0.1:50103.
    pub nid_tenant_sync_grpc_url: String,
    /// gRPC address for the standalone fin-operator service. Empty defaults to 127.0.0.1:50104.
    pub fin_operator_grpc_url: String,
    /// gRPC address for the standalone fin-vision service. Empty defaults to 127.0.0.1:50105.
    pub fin_vision_grpc_url: String,
    /// gRPC address for the standalone store-connection service. Empty defaults to 127.0.0.1:50106.
    pub store_connection_grpc_url: String,
    /// gRPC address for the standalone whatsapp-template-sync service. Empty defaults to 127.0.0.1:50104.
    pub whatsapp_template_sync_grpc_url: String,
    /// gRPC address for the standalone whatsapp-campaigns service. Empty defaults to 127.0.0.1:50107.
    pub whatsapp_campaigns_grpc_url: String,
    /// gRPC address for the standalone services-sequences service. Empty defaults to 127.0.0.1:50107.
    pub services_sequences_grpc_url: String,
    /// gRPC address for the standalone external-otp service. Empty defaults to 127.0.0.1:50108.
    pub external_otp_grpc_url: String,
    /// gRPC address for the standalone shopify-store service. Empty defaults to 127.0.0.1:50108.
    pub shopify_store_grpc_url: String,
    /// gRPC address for the standalone sso service. Empty defaults to 127.0.0.1:50109.
    pub sso_grpc_url: String,
    /// gRPC address for the standalone health service. Empty defaults to 127.0.0.1:50110.
    pub health_grpc_url: String,
    /// gRPC address for the standalone translation service. Empty defaults to 127.0.0.1:50111.
    pub translation_grpc_url: String,
    /// gRPC address for the standalone CSAT surveys service. Empty defaults to 127.0.0.1:50112.
    pub csat_surveys_grpc_url: String,
    /// gRPC address for the standalone discovery service. Empty defaults to 127.0.0.1:50113.
    pub discovery_grpc_url: String,
    /// gRPC address for the standalone automation service. Empty defaults to 127.0.0.1:50114.
    pub automation_grpc_url: String,
    /// gRPC address for the standalone business_profile service. Empty defaults to 127.0.0.1:50115.
    pub business_profile_grpc_url: String,
    /// gRPC address for the standalone team_routing service. Empty defaults to 127.0.0.1:50116.
    pub team_routing_grpc_url: String,
    /// gRPC address for the standalone knowledge_base service. Empty defaults to 127.0.0.1:50117.
    pub knowledge_base_grpc_url: String,
    /// gRPC address for the standalone live_proof service. Empty defaults to 127.0.0.1:50118.
    pub live_proof_grpc_url: String,
    /// gRPC address for the standalone commerce-catalog service. Empty defaults to 127.0.0.1:50110.
    pub commerce_catalog_grpc_url: String,
    /// gRPC address for the standalone commerce-orders service. Empty defaults to 127.0.0.1:50111.
    pub commerce_orders_grpc_url: String,
    /// gRPC address for the standalone conversations service. Empty defaults to 127.0.0.1:50120.
    pub conversations_grpc_url: String,
    /// gRPC address for the standalone meta-sync service. Empty defaults to 127.0.0.1:50113.
    pub meta_sync_grpc_url: String,
    // nID identity provider (NEWAX ecosystem SSO + secrets vault)
    pub nid_base_url: String,
    pub nid_client_id: String,
    pub nid_client_secret: String,
    pub nid_redirect_uri: String,
    pub nid_vault_addr: String,
    pub nid_vault_token: String,
    pub nid_vault_role_id: String,
    pub nid_vault_secret_id: String,
    pub nid_allow_mock_auth: bool,
    /// Brand ID used by the OTP engine when nID sign-up generates OTPs.
    pub nid_otp_brand_id: String,

    // Fallback tenant when a request carries no X-Tenant-Key. Intentionally
    // has no compiled-in value: unset means tenant-less requests are refused.
    pub default_tenant_key: String,

    // LiveProof refund verification. `public_url` is only used to build the
    // customer link; when empty the request's own base URL is used instead of
    // a compiled-in domain.
    pub public_url: String,
    pub live_proof_storage_dir: String,

    // External translation engine. Empty means translation is refused rather
    // than answered with the original text dressed up as a translation.
    pub translation_api_url: String,
    pub translation_api_key: String,

    // NeuraCore zero-trust crypto mesh. The three secrets are empty by default:
    // an unconfigured mesh refuses ingress and egress instead of accepting
    // envelopes it has no key to verify or open.
    pub neura_tenant_id: String,
    pub neura_entity_id: String,
    pub neura_key_id: String,
    pub neura_relay_gateway_url: String,
    pub neura_ingress_secret_hex: String,
    pub neura_egress_secret_hex: String,
    pub neura_auth_secret_hex: String,
    pub neura_max_clock_skew_seconds: i64,

    // Widget identity hashing. Empty means any inbox that turns identity
    // validation on is refused rather than validated with a guessed key.
    pub widget_identity_hmac_secret: String,
}

impl Config {
    pub fn from_env() -> Self {
        dotenvy::dotenv().ok();

        let database_url = env::var("RUST_DATABASE_URL")
            .or_else(|_| env::var("DATABASE_URL"))
            .unwrap_or_else(|_| "postgresql://communicator_app:C0mm@nicat0r!2026#Secure@localhost:5432/communicator".to_string());
        let host = env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
        let port = env::var("PORT")
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(8000);
        let static_dir = if let Ok(dir) = env::var("FRONTEND_DIST") {
            dir
        } else if std::path::Path::new("frontend/dist").exists() {
            "frontend/dist".to_string()
        } else if std::path::Path::new("../frontend/dist").exists() {
            "../frontend/dist".to_string()
        } else {
            "frontend/dist".to_string()
        };
        let app_env = env::var("APP_ENV").unwrap_or_else(|_| "development".to_string());

        let meta_whatsapp_access_token = env::var("META_WHATSAPP_ACCESS_TOKEN").unwrap_or_default();
        let meta_whatsapp_phone_number_id = env::var("META_WHATSAPP_PHONE_NUMBER_ID").unwrap_or_default();
        let meta_whatsapp_api_version = env::var("META_WHATSAPP_API_VERSION")
            .unwrap_or_else(|_| "v21.0".to_string());
        let meta_webhook_verify_token = env::var("META_WEBHOOK_VERIFY_TOKEN").unwrap_or_default();
        let meta_app_secret = env::var("META_APP_SECRET").unwrap_or_default();
        let meta_instagram_app_secret =
            env::var("META_INSTAGRAM_APP_SECRET").unwrap_or_default();
        let postmark_inbound_webhook_secret =
            env::var("POSTMARK_INBOUND_WEBHOOK_SECRET").unwrap_or_default();
        let email_inbound_webhook_secret =
            env::var("EMAIL_INBOUND_WEBHOOK_SECRET").unwrap_or_default();
        let tiktok_webhook_verify_token = env::var("TIKTOK_WEBHOOK_VERIFY_TOKEN").unwrap_or_default();
        let tiktok_app_secret = env::var("TIKTOK_APP_SECRET").unwrap_or_default();
        let tiktok_access_token = env::var("TIKTOK_ACCESS_TOKEN").unwrap_or_default();
        let ecosystem_gateway_key = env::var("ECOSYSTEM_GATEWAY_KEY").unwrap_or_default();

        let twilio_account_sid = env::var("TWILIO_ACCOUNT_SID").unwrap_or_default();
        let twilio_auth_token = env::var("TWILIO_AUTH_TOKEN").unwrap_or_default();
        let twilio_sms_from = env::var("TWILIO_SMS_FROM").unwrap_or_default();
        let twilio_messaging_service_sid =
            env::var("TWILIO_MESSAGING_SERVICE_SID").unwrap_or_default();

        let anthropic_api_key = env::var("ANTHROPIC_API_KEY").unwrap_or_default();
        let anthropic_model = env::var("ANTHROPIC_MODEL")
            .unwrap_or_else(|_| "claude-3-5-sonnet".to_string());
        let harvester_api_base_url = env::var("HARVESTER_API_BASE_URL")
            .unwrap_or_else(|_| "http://localhost:8000/api/v1".to_string());
        let reply_auto_send_confidence = env::var("REPLY_AUTO_SEND_CONFIDENCE")
            .ok()
            .and_then(|raw| raw.trim().parse::<f64>().ok())
            .filter(|value| (0.0..=1.0).contains(value));
        let sales_alert_webhook_url = env::var("SALES_ALERT_WEBHOOK_URL").unwrap_or_default();
        let access_control_grpc_url = env::var("ACCESS_CONTROL_GRPC_URL").unwrap_or_default();
        let mcp_protocol_grpc_url = env::var("MCP_PROTOCOL_GRPC_URL").unwrap_or_default();
        let otp_engine_grpc_url = env::var("OTP_ENGINE_GRPC_URL").unwrap_or_default();
        let nid_tenant_sync_grpc_url = env::var("NID_TENANT_SYNC_GRPC_URL").unwrap_or_default();
        let fin_operator_grpc_url = env::var("FIN_OPERATOR_GRPC_URL").unwrap_or_default();
        let fin_vision_grpc_url = env::var("FIN_VISION_GRPC_URL").unwrap_or_default();
        let store_connection_grpc_url = env::var("STORE_CONNECTION_GRPC_URL").unwrap_or_default();
        let external_otp_grpc_url = env::var("EXTERNAL_OTP_GRPC_URL").unwrap_or_default();
        let sso_grpc_url = env::var("SSO_GRPC_URL").unwrap_or_default();
        let health_grpc_url = env::var("HEALTH_GRPC_URL").unwrap_or_default();
        let translation_grpc_url = env::var("TRANSLATION_GRPC_URL").unwrap_or_default();
        let csat_surveys_grpc_url = env::var("CSAT_SURVEYS_GRPC_URL").unwrap_or_default();
        let discovery_grpc_url = env::var("DISCOVERY_GRPC_URL").unwrap_or_default();
        let automation_grpc_url = env::var("AUTOMATION_GRPC_URL").unwrap_or_default();
        let business_profile_grpc_url = env::var("BUSINESS_PROFILE_GRPC_URL").unwrap_or_default();
        let team_routing_grpc_url = env::var("TEAM_ROUTING_GRPC_URL").unwrap_or_default();
        let knowledge_base_grpc_url = env::var("KNOWLEDGE_BASE_GRPC_URL").unwrap_or_default();
        let live_proof_grpc_url = env::var("LIVE_PROOF_GRPC_URL").unwrap_or_default();
        let commerce_catalog_grpc_url = env::var("COMMERCE_CATALOG_GRPC_URL").unwrap_or_default();
        let commerce_orders_grpc_url = env::var("COMMERCE_ORDERS_GRPC_URL").unwrap_or_default();
        let conversations_grpc_url = env::var("CONVERSATIONS_GRPC_URL").unwrap_or_default();
        let meta_sync_grpc_url = env::var("META_SYNC_GRPC_URL").unwrap_or_default();
        let whatsapp_template_sync_grpc_url = env::var("WHATSAPP_TEMPLATE_SYNC_GRPC_URL").unwrap_or_default();
        let whatsapp_campaigns_grpc_url = env::var("WHATSAPP_CAMPAIGNS_GRPC_URL").unwrap_or_default();
        let services_sequences_grpc_url = env::var("SERVICES_SEQUENCES_GRPC_URL").unwrap_or_default();
        let shopify_store_grpc_url = env::var("SHOPIFY_STORE_GRPC_URL").unwrap_or_default();

        let nid_base_url = env::var("NID_BASE_URL")
            .unwrap_or_else(|_| "http://localhost:4000".to_string());
        let nid_client_id = env::var("NID_CLIENT_ID").unwrap_or_default();
        let nid_client_secret = env::var("NID_CLIENT_SECRET").unwrap_or_default();
        let nid_redirect_uri = env::var("NID_REDIRECT_URI")
            .unwrap_or_else(|_| "http://localhost:8000/api/v1/auth/nid/callback".to_string());
        let nid_vault_addr =
            env::var("NID_VAULT_ADDR").unwrap_or_else(|_| nid_base_url.clone());
        let nid_vault_token = env::var("NID_VAULT_TOKEN").unwrap_or_default();
        let nid_vault_role_id = env::var("NID_VAULT_ROLE_ID").unwrap_or_default();
        let nid_vault_secret_id = env::var("NID_VAULT_SECRET_ID").unwrap_or_default();
        let nid_allow_mock_auth = env::var("NID_ALLOW_MOCK_AUTH")
            .map(|v| matches!(v.as_str(), "1" | "true" | "TRUE" | "yes" | "on"))
            .unwrap_or(false);
        let nid_otp_brand_id = env::var("NID_OTP_BRAND_ID").unwrap_or_else(|_| {
            // Default to the client_id so the OTP engine has a brand context
            env::var("NID_CLIENT_ID").unwrap_or_default()
        });
        let default_tenant_key = env::var("DEFAULT_TENANT_KEY").unwrap_or_default();
        let public_url = env::var("PUBLIC_URL").unwrap_or_default();
        let live_proof_storage_dir =
            env::var("LIVE_PROOF_STORAGE_DIR").unwrap_or_else(|_| "live_proof_media".to_string());
        let translation_api_url = env::var("TRANSLATION_API_URL").unwrap_or_default();
        let translation_api_key = env::var("TRANSLATION_API_KEY").unwrap_or_default();
        let neura_tenant_id = env::var("NEURA_CRYPTO_TENANT_ID").unwrap_or_default();
        let neura_entity_id = env::var("NEURA_CRYPTO_ENTITY_ID").unwrap_or_default();
        let neura_key_id = env::var("NEURA_CRYPTO_KEY_ID").unwrap_or_default();
        let neura_relay_gateway_url = env::var("NEURA_RELAY_GATEWAY_URL").unwrap_or_default();
        let neura_ingress_secret_hex = env::var("NEURA_CRYPTO_INGRESS_SECRET_HEX").unwrap_or_default();
        let neura_egress_secret_hex = env::var("NEURA_CRYPTO_EGRESS_SECRET_HEX").unwrap_or_default();
        let neura_auth_secret_hex = env::var("NEURA_CRYPTO_AUTH_SECRET_HEX").unwrap_or_default();
        // A policy window, not a fabricated value: 300s is what the mesh
        // specification ships with and it is overridable per deployment.
        let neura_max_clock_skew_seconds = env::var("NEURA_CRYPTO_MAX_CLOCK_SKEW_SECONDS")
            .ok()
            .and_then(|raw| raw.trim().parse::<i64>().ok())
            .filter(|raw| *raw > 0)
            .unwrap_or(300);
        let widget_identity_hmac_secret = env::var("WIDGET_IDENTITY_HMAC_SECRET").unwrap_or_default();

        Self {
            database_url,
            host,
            port,
            static_dir,
            app_env,
            meta_whatsapp_access_token,
            meta_whatsapp_phone_number_id,
            meta_whatsapp_api_version,
            meta_webhook_verify_token,
            meta_app_secret,
            meta_instagram_app_secret,
            postmark_inbound_webhook_secret,
            email_inbound_webhook_secret,
            tiktok_webhook_verify_token,
            tiktok_app_secret,
            tiktok_access_token,
            ecosystem_gateway_key,
            twilio_account_sid,
            twilio_auth_token,
            twilio_sms_from,
            twilio_messaging_service_sid,
            anthropic_api_key,
            anthropic_model,
            harvester_api_base_url,
            reply_auto_send_confidence,
            sales_alert_webhook_url,
            access_control_grpc_url,
            mcp_protocol_grpc_url,
            otp_engine_grpc_url,
            nid_tenant_sync_grpc_url,
            fin_operator_grpc_url,
            fin_vision_grpc_url,
            store_connection_grpc_url,
            whatsapp_template_sync_grpc_url,
            whatsapp_campaigns_grpc_url,
            services_sequences_grpc_url,
            external_otp_grpc_url,
            shopify_store_grpc_url,
            sso_grpc_url,
            health_grpc_url,
            translation_grpc_url,
            csat_surveys_grpc_url,
            discovery_grpc_url,
            automation_grpc_url,
            business_profile_grpc_url,
            team_routing_grpc_url,
            knowledge_base_grpc_url,
            live_proof_grpc_url,
            commerce_catalog_grpc_url,
            commerce_orders_grpc_url,
            conversations_grpc_url,
            meta_sync_grpc_url,
            nid_base_url,
            nid_client_id,
            nid_client_secret,
            nid_redirect_uri,
            nid_vault_addr,
            nid_vault_token,
            nid_vault_role_id,
            nid_vault_secret_id,
            nid_allow_mock_auth,
            nid_otp_brand_id,
            default_tenant_key,
            public_url,
            live_proof_storage_dir,
            translation_api_url,
            translation_api_key,
            neura_tenant_id,
            neura_entity_id,
            neura_key_id,
            neura_relay_gateway_url,
            neura_ingress_secret_hex,
            neura_egress_secret_hex,
            neura_auth_secret_hex,
            neura_max_clock_skew_seconds,
            widget_identity_hmac_secret,
        }
    }

    fn set_string(field: &mut String, value: &serde_json::Value) {
        match value {
            serde_json::Value::String(text) => *field = text.clone(),
            serde_json::Value::Number(num) => *field = num.to_string(),
            serde_json::Value::Bool(flag) => *field = flag.to_string(),
            _ => {}
        }
    }

    fn set_bool(field: &mut bool, value: &serde_json::Value) {
        *field = match value {
            serde_json::Value::Bool(flag) => *flag,
            serde_json::Value::String(text) => {
                matches!(text.to_ascii_lowercase().as_str(), "1" | "true" | "yes" | "on")
            }
            serde_json::Value::Number(num) => num.as_i64().map(|v| v != 0).unwrap_or(false),
            _ => *field,
        };
    }

    /// Applies one resolved vault secret onto the running configuration.
    /// Returns false for keys this backend does not consume.
    pub fn apply_vault_secret(&mut self, key: &str, value: &serde_json::Value) -> bool {
        match key.to_ascii_lowercase().as_str() {
            "database_url" | "rust_database_url" => {
                Self::set_string(&mut self.database_url, value)
            }
            "host" => Self::set_string(&mut self.host, value),
            "port" => {
                if let Some(text) = value.as_str() {
                    if let Ok(parsed) = text.parse() {
                        self.port = parsed;
                    }
                } else if let Some(parsed) = value.as_u64().and_then(|v| u16::try_from(v).ok()) {
                    self.port = parsed;
                }
            }
            "frontend_dist" | "static_dir" => Self::set_string(&mut self.static_dir, value),
            "app_env" => Self::set_string(&mut self.app_env, value),
            "meta_whatsapp_access_token" => {
                Self::set_string(&mut self.meta_whatsapp_access_token, value)
            }
            "meta_whatsapp_phone_number_id" => {
                Self::set_string(&mut self.meta_whatsapp_phone_number_id, value)
            }
            "meta_whatsapp_api_version" => {
                Self::set_string(&mut self.meta_whatsapp_api_version, value)
            }
            "meta_webhook_verify_token" => Self::set_string(&mut self.meta_webhook_verify_token, value),
            "meta_app_secret" => Self::set_string(&mut self.meta_app_secret, value),
            "meta_instagram_app_secret" => {
                Self::set_string(&mut self.meta_instagram_app_secret, value)
            }
            "postmark_inbound_webhook_secret" => {
                Self::set_string(&mut self.postmark_inbound_webhook_secret, value)
            }
            "email_inbound_webhook_secret" => {
                Self::set_string(&mut self.email_inbound_webhook_secret, value)
            }
            "tiktok_webhook_verify_token" => {
                Self::set_string(&mut self.tiktok_webhook_verify_token, value)
            }
            "tiktok_app_secret" => Self::set_string(&mut self.tiktok_app_secret, value),
            "tiktok_access_token" => Self::set_string(&mut self.tiktok_access_token, value),
            "ecosystem_gateway_key" => Self::set_string(&mut self.ecosystem_gateway_key, value),
            "twilio_account_sid" => Self::set_string(&mut self.twilio_account_sid, value),
            "twilio_auth_token" => Self::set_string(&mut self.twilio_auth_token, value),
            "twilio_sms_from" => Self::set_string(&mut self.twilio_sms_from, value),
            "twilio_messaging_service_sid" => {
                Self::set_string(&mut self.twilio_messaging_service_sid, value)
            }
            "anthropic_api_key" => Self::set_string(&mut self.anthropic_api_key, value),
            "anthropic_model" => Self::set_string(&mut self.anthropic_model, value),
            "harvester_api_base_url" => Self::set_string(&mut self.harvester_api_base_url, value),
            "reply_auto_send_confidence" => {
                let parsed = match value {
                    serde_json::Value::Number(num) => num.as_f64(),
                    serde_json::Value::String(text) => text.trim().parse::<f64>().ok(),
                    _ => None,
                };
                match parsed {
                    Some(v) if (0.0..=1.0).contains(&v) => {
                        self.reply_auto_send_confidence = Some(v)
                    }
                    // Out-of-range or non-numeric: report the key as not applied
                    // instead of silently keeping a stale threshold.
                    _ => return false,
                }
            }
            "sales_alert_webhook_url" => Self::set_string(&mut self.sales_alert_webhook_url, value),
            "access_control_grpc_url" => Self::set_string(&mut self.access_control_grpc_url, value),
            "mcp_protocol_grpc_url" => Self::set_string(&mut self.mcp_protocol_grpc_url, value),
            "otp_engine_grpc_url" => Self::set_string(&mut self.otp_engine_grpc_url, value),
            "nid_tenant_sync_grpc_url" => Self::set_string(&mut self.nid_tenant_sync_grpc_url, value),
            "fin_operator_grpc_url" => Self::set_string(&mut self.fin_operator_grpc_url, value),
            "fin_vision_grpc_url" => Self::set_string(&mut self.fin_vision_grpc_url, value),
            "store_connection_grpc_url" => Self::set_string(&mut self.store_connection_grpc_url, value),
            "whatsapp_template_sync_grpc_url" => Self::set_string(&mut self.whatsapp_template_sync_grpc_url, value),
            "whatsapp_campaigns_grpc_url" => Self::set_string(&mut self.whatsapp_campaigns_grpc_url, value),
            "services_sequences_grpc_url" => Self::set_string(&mut self.services_sequences_grpc_url, value),
            "external_otp_grpc_url" => Self::set_string(&mut self.external_otp_grpc_url, value),
            "sso_grpc_url" => Self::set_string(&mut self.sso_grpc_url, value),
            "health_grpc_url" => Self::set_string(&mut self.health_grpc_url, value),
            "translation_grpc_url" => Self::set_string(&mut self.translation_grpc_url, value),
            "csat_surveys_grpc_url" => Self::set_string(&mut self.csat_surveys_grpc_url, value),
            "discovery_grpc_url" => Self::set_string(&mut self.discovery_grpc_url, value),
            "automation_grpc_url" => Self::set_string(&mut self.automation_grpc_url, value),
            "business_profile_grpc_url" => Self::set_string(&mut self.business_profile_grpc_url, value),
            "team_routing_grpc_url" => Self::set_string(&mut self.team_routing_grpc_url, value),
            "knowledge_base_grpc_url" => Self::set_string(&mut self.knowledge_base_grpc_url, value),
            "live_proof_grpc_url" => Self::set_string(&mut self.live_proof_grpc_url, value),
            "commerce_catalog_grpc_url" => Self::set_string(&mut self.commerce_catalog_grpc_url, value),
            "commerce_orders_grpc_url" => Self::set_string(&mut self.commerce_orders_grpc_url, value),
            "conversations_grpc_url" => Self::set_string(&mut self.conversations_grpc_url, value),
            "meta_sync_grpc_url" => Self::set_string(&mut self.meta_sync_grpc_url, value),
            "nid_base_url" => Self::set_string(&mut self.nid_base_url, value),
            "nid_client_id" => Self::set_string(&mut self.nid_client_id, value),
            "nid_client_secret" => Self::set_string(&mut self.nid_client_secret, value),
            "nid_redirect_uri" => Self::set_string(&mut self.nid_redirect_uri, value),
            "nid_vault_addr" => Self::set_string(&mut self.nid_vault_addr, value),
            "nid_vault_token" => Self::set_string(&mut self.nid_vault_token, value),
            "nid_vault_role_id" => Self::set_string(&mut self.nid_vault_role_id, value),
            "nid_vault_secret_id" => Self::set_string(&mut self.nid_vault_secret_id, value),
            "nid_allow_mock_auth" => Self::set_bool(&mut self.nid_allow_mock_auth, value),
            "nid_otp_brand_id" => Self::set_string(&mut self.nid_otp_brand_id, value),
            "default_tenant_key" => Self::set_string(&mut self.default_tenant_key, value),
            "public_url" => Self::set_string(&mut self.public_url, value),
            "live_proof_storage_dir" => Self::set_string(&mut self.live_proof_storage_dir, value),
            "translation_api_url" => Self::set_string(&mut self.translation_api_url, value),
            "translation_api_key" => Self::set_string(&mut self.translation_api_key, value),
            "neura_crypto_tenant_id" => Self::set_string(&mut self.neura_tenant_id, value),
            "neura_crypto_entity_id" => Self::set_string(&mut self.neura_entity_id, value),
            "neura_crypto_key_id" => Self::set_string(&mut self.neura_key_id, value),
            "neura_relay_gateway_url" => Self::set_string(&mut self.neura_relay_gateway_url, value),
            "neura_crypto_ingress_secret_hex" => {
                Self::set_string(&mut self.neura_ingress_secret_hex, value)
            }
            "neura_crypto_egress_secret_hex" => Self::set_string(&mut self.neura_egress_secret_hex, value),
            "neura_crypto_auth_secret_hex" => Self::set_string(&mut self.neura_auth_secret_hex, value),
            "widget_identity_hmac_secret" => Self::set_string(&mut self.widget_identity_hmac_secret, value),
            "neura_crypto_max_clock_skew_seconds" => {
                self.neura_max_clock_skew_seconds = match value {
                    serde_json::Value::Number(num) => num.as_i64().unwrap_or(self.neura_max_clock_skew_seconds),
                    serde_json::Value::String(text) => {
                        text.trim().parse::<i64>().unwrap_or(self.neura_max_clock_skew_seconds)
                    }
                    _ => self.neura_max_clock_skew_seconds,
                }
            }
            _ => return false,
        }
        true
    }
}
