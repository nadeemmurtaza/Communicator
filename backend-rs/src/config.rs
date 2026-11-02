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
        }
    }
}
