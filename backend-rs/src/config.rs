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

    // AI & Harvester
    pub anthropic_api_key: String,
    pub anthropic_model: String,
    pub harvester_api_base_url: String,

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

    // Fallback tenant when a request carries no X-Tenant-Key. Intentionally
    // has no compiled-in value: unset means tenant-less requests are refused.
    pub default_tenant_key: String,
}

impl Config {
    pub fn from_env() -> Self {
        dotenvy::dotenv().ok();

        let raw_db = env::var("RUST_DATABASE_URL")
            .or_else(|_| env::var("DATABASE_URL"))
            .unwrap_or_else(|_| "sqlite://communicator.db?mode=rwc".to_string());
        
        let database_url = if raw_db.contains("sqlite") {
            raw_db
        } else {
            // For desktop & standalone portable mode, use high-speed embedded SQLite
            "sqlite://communicator.db?mode=rwc".to_string()
        };
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

        let anthropic_api_key = env::var("ANTHROPIC_API_KEY").unwrap_or_default();
        let anthropic_model = env::var("ANTHROPIC_MODEL")
            .unwrap_or_else(|_| "claude-3-5-sonnet".to_string());
        let harvester_api_base_url = env::var("HARVESTER_API_BASE_URL")
            .unwrap_or_else(|_| "http://localhost:8000/api/v1".to_string());

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
        let default_tenant_key = env::var("DEFAULT_TENANT_KEY").unwrap_or_default();

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
            anthropic_api_key,
            anthropic_model,
            harvester_api_base_url,
            nid_base_url,
            nid_client_id,
            nid_client_secret,
            nid_redirect_uri,
            nid_vault_addr,
            nid_vault_token,
            nid_vault_role_id,
            nid_vault_secret_id,
            nid_allow_mock_auth,
            default_tenant_key,
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
            "anthropic_api_key" => Self::set_string(&mut self.anthropic_api_key, value),
            "anthropic_model" => Self::set_string(&mut self.anthropic_model, value),
            "harvester_api_base_url" => Self::set_string(&mut self.harvester_api_base_url, value),
            "nid_base_url" => Self::set_string(&mut self.nid_base_url, value),
            "nid_client_id" => Self::set_string(&mut self.nid_client_id, value),
            "nid_client_secret" => Self::set_string(&mut self.nid_client_secret, value),
            "nid_redirect_uri" => Self::set_string(&mut self.nid_redirect_uri, value),
            "nid_vault_addr" => Self::set_string(&mut self.nid_vault_addr, value),
            "nid_vault_token" => Self::set_string(&mut self.nid_vault_token, value),
            "nid_vault_role_id" => Self::set_string(&mut self.nid_vault_role_id, value),
            "nid_vault_secret_id" => Self::set_string(&mut self.nid_vault_secret_id, value),
            "nid_allow_mock_auth" => Self::set_bool(&mut self.nid_allow_mock_auth, value),
            "default_tenant_key" => Self::set_string(&mut self.default_tenant_key, value),
            _ => return false,
        }
        true
    }
}
