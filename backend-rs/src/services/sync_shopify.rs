//! Shopify Sync Adapter

use crate::models::SyncConnection;
use crate::services::sync_adapter::{SyncAdapter, SyncResult};
use sqlx::PgPool;

pub struct ShopifySyncAdapter {
    pub connection: Option<SyncConnection>,
}

#[async_trait::async_trait]
impl SyncAdapter for ShopifySyncAdapter {
    fn platform_name(&self) -> &str {
        "shopify"
    }

    async fn sync_products(&self, _pool: &PgPool, _brand_id: &str) -> SyncResult {
        SyncResult {
            success: true,
            platform: "shopify".to_string(),
            entity_type: "products".to_string(),
            entity_id: "".to_string(),
            message: "Shopify products sync stub".to_string(),
            errors: Vec::new(),
        }
    }

    async fn sync_orders(&self, _pool: &PgPool, _brand_id: &str) -> SyncResult {
        SyncResult {
            success: true,
            platform: "shopify".to_string(),
            entity_type: "orders".to_string(),
            entity_id: "".to_string(),
            message: "Shopify orders sync stub".to_string(),
            errors: Vec::new(),
        }
    }

    async fn sync_inventory(&self, _pool: &PgPool, _brand_id: &str) -> SyncResult {
        SyncResult {
            success: true,
            platform: "shopify".to_string(),
            entity_type: "inventory".to_string(),
            entity_id: "".to_string(),
            message: "Shopify inventory sync stub".to_string(),
            errors: Vec::new(),
        }
    }
}
