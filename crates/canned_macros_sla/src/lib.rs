//! Canned responses, macros, and SLA policies management.

use serde::{Deserialize, Serialize};
use sqlx::PgPool;

#[derive(Clone)]
pub struct CannedMacrosSlaState {
    pub db: PgPool,
    pub http_client: reqwest::Client,
    pub meta_whatsapp_access_token: String,
    pub meta_whatsapp_phone_number_id: String,
    pub meta_whatsapp_api_version: String,
}

impl CannedMacrosSlaState {
    pub fn new(db: PgPool) -> Self {
        Self {
            db,
            http_client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .unwrap_or_default(),
            meta_whatsapp_access_token: std::env::var("META_WHATSAPP_ACCESS_TOKEN").unwrap_or_default(),
            meta_whatsapp_phone_number_id: std::env::var("META_WHATSAPP_PHONE_NUMBER_ID").unwrap_or_default(),
            meta_whatsapp_api_version: std::env::var("META_WHATSAPP_API_VERSION").unwrap_or_default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct CannedResponseRow {
    pub id: String,
    pub short_code: String,
    pub content: String,
    pub category: Option<String>,
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct MacroItem {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub actions: Option<String>,
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct SlaPolicyRow {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub first_response_time_threshold: Option<i32>,
    pub resolution_time_threshold: Option<i32>,
    pub only_during_business_hours: Option<bool>,
    pub priorities: Option<String>,
    pub inbox_ids: Option<String>,
    pub is_active: Option<bool>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}
