//! Sync Adapter - Abstract interface for e-commerce platform synchronization
//!
//! Provides a unified interface for syncing products, orders, and inventory
//! across multiple e-commerce platforms (Shopify, WooCommerce, Wix, Square).

use crate::models::{SyncConnection, SyncLog};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
use uuid::Uuid;

/// Result of a sync operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncResult {
    pub success: bool,
    pub platform: String,
    pub entity_type: String,
    pub entity_id: String,
    pub message: String,
    pub errors: Vec<String>,
}

/// Request to create a sync connection. `credentials` is a free-form JSON
/// object whose shape depends on the platform, e.g. for Shopify:
/// `{ "shop_domain": "foo.myshopify.com", "access_token": "shpat_..." }`,
/// or for WooCommerce: `{ "store_url": "https://...", "consumer_key": "...",
/// "consumer_secret": "..." }`.
#[derive(Debug, Clone, Deserialize)]
pub struct CreateSyncConnectionRequest {
    pub brand_id: String,
    pub platform: String,
    pub credentials: serde_json::Value,
}

/// Request to update a sync connection
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateSyncConnectionRequest {
    pub credentials: Option<serde_json::Value>,
    pub sync_status: Option<String>,
}

/// Trait for platform-specific sync adapters
#[async_trait::async_trait]
pub trait SyncAdapter: Send + Sync {
    fn platform_name(&self) -> &str;
    async fn sync_products(&self, pool: &PgPool, brand_id: &str) -> SyncResult;
    async fn sync_orders(&self, pool: &PgPool, brand_id: &str) -> SyncResult;
    async fn sync_inventory(&self, pool: &PgPool, brand_id: &str) -> SyncResult;
}

const SELECT_CONNECTION_COLUMNS: &str = "id, brand_id, platform, credentials, sync_status, \
     last_sync_at, metadata, created_at::TEXT AS created_at, updated_at::TEXT AS updated_at";

/// Main sync service that coordinates all platform syncs
pub struct SyncService {
    pool: PgPool,
    http_client: reqwest::Client,
}

impl SyncService {
    pub fn new(pool: PgPool, http_client: reqwest::Client) -> Self {
        Self { pool, http_client }
    }

    /// Resolve the platform-specific adapter for a connection.
    fn adapter_for(&self, conn: &SyncConnection) -> Result<Box<dyn SyncAdapter>, String> {
        match conn.platform.as_str() {
            "shopify" => Ok(Box::new(crate::services::sync_shopify::ShopifySyncAdapter {
                connection: Some(conn.clone()),
                http_client: self.http_client.clone(),
            })),
            "woocommerce" => Ok(Box::new(
                crate::services::sync_woocommerce::WooCommerceSyncAdapter {
                    connection: Some(conn.clone()),
                    http_client: self.http_client.clone(),
                },
            )),
            "square" => Ok(Box::new(crate::services::sync_square::SquareSyncAdapter {
                connection: Some(conn.clone()),
                http_client: self.http_client.clone(),
            })),
            "wix" => Ok(Box::new(crate::services::sync_wix::WixSyncAdapter {
                connection: Some(conn.clone()),
                http_client: self.http_client.clone(),
            })),
            other => Err(format!("Unsupported sync platform: {other}")),
        }
    }

    // -----------------------------------------------------------------------
    // Connections CRUD
    // -----------------------------------------------------------------------

    pub async fn list_connections(&self, brand_id: &str) -> Result<Vec<SyncConnection>, String> {
        sqlx::query_as::<_, SyncConnection>(&format!(
            "SELECT {SELECT_CONNECTION_COLUMNS} FROM sync_connections \
             WHERE brand_id = $1 ORDER BY created_at DESC"
        ))
        .bind(brand_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to list sync connections: {e}"))
    }

    pub async fn get_connection(&self, id: &str) -> Result<SyncConnection, String> {
        sqlx::query_as::<_, SyncConnection>(&format!(
            "SELECT {SELECT_CONNECTION_COLUMNS} FROM sync_connections WHERE id = $1"
        ))
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("Failed to load sync connection: {e}"))?
        .ok_or_else(|| format!("Sync connection {id} not found"))
    }

    pub async fn create_connection(
        &self,
        req: &CreateSyncConnectionRequest,
    ) -> Result<SyncConnection, String> {
        let id = format!("sync_{}", Uuid::new_v4().as_hyphenated());
        let credentials = req.credentials.to_string();
        sqlx::query(
            "INSERT INTO sync_connections (id, brand_id, platform, credentials, sync_status, created_at, updated_at) \
             VALUES ($1, $2, $3, $4, 'disconnected', NOW(), NOW())",
        )
        .bind(&id)
        .bind(&req.brand_id)
        .bind(&req.platform)
        .bind(&credentials)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to create sync connection: {e}"))?;
        self.get_connection(&id).await
    }

    pub async fn update_connection(
        &self,
        id: &str,
        req: &UpdateSyncConnectionRequest,
    ) -> Result<SyncConnection, String> {
        let existing = self.get_connection(id).await?;
        let credentials = match &req.credentials {
            Some(c) => c.to_string(),
            None => existing.credentials.clone().unwrap_or_else(|| "{}".to_string()),
        };
        let status = req.sync_status.clone().unwrap_or(existing.sync_status.clone());
        sqlx::query(
            "UPDATE sync_connections SET credentials = $1, sync_status = $2, updated_at = NOW() WHERE id = $3",
        )
        .bind(&credentials)
        .bind(&status)
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to update sync connection: {e}"))?;
        self.get_connection(id).await
    }

    pub async fn delete_connection(&self, id: &str) -> Result<(), String> {
        sqlx::query("DELETE FROM sync_connections WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map(|_| ())
            .map_err(|e| format!("Failed to delete sync connection: {e}"))
    }

    // -----------------------------------------------------------------------
    // Sync execution
    // -----------------------------------------------------------------------

    async fn log_result(&self, connection_id: &str, result: &SyncResult) {
        let id = format!("synclog_{}", Uuid::new_v4().as_hyphenated());
        let status = if result.success { "success" } else { "error" };
        let error = result.errors.first().cloned();
        let entity_id = if result.entity_id.is_empty() {
            None
        } else {
            Some(result.entity_id.clone())
        };
        let _ = sqlx::query(
            "INSERT INTO sync_logs (id, connection_id, direction, entity_type, entity_id, status, error) \
             VALUES ($1, $2, 'outbound', $3, $4, $5, $6)",
        )
        .bind(&id)
        .bind(connection_id)
        .bind(&result.entity_type)
        .bind(entity_id)
        .bind(status)
        .bind(error)
        .execute(&self.pool)
        .await;
    }

    pub async fn trigger_sync(&self, connection_id: &str) -> Result<SyncResult, String> {
        let conn = self.get_connection(connection_id).await?;
        let adapter = self.adapter_for(&conn)?;

        let results = vec![
            adapter.sync_products(&self.pool, &conn.brand_id).await,
            adapter.sync_orders(&self.pool, &conn.brand_id).await,
            adapter.sync_inventory(&self.pool, &conn.brand_id).await,
        ];
        for r in &results {
            self.log_result(connection_id, r).await;
        }

        let success = results.iter().all(|r| r.success);
        let errors: Vec<String> = results.iter().flat_map(|r| r.errors.clone()).collect();

        let _ = sqlx::query(
            "UPDATE sync_connections SET sync_status = $1, last_sync_at = NOW()::TEXT, updated_at = NOW() WHERE id = $2",
        )
        .bind(if success { "connected" } else { "error" })
        .bind(connection_id)
        .execute(&self.pool)
        .await;

        Ok(SyncResult {
            success,
            platform: conn.platform.clone(),
            entity_type: "all".to_string(),
            entity_id: connection_id.to_string(),
            message: format!(
                "Synced products, orders, and inventory for {}",
                conn.platform
            ),
            errors,
        })
    }

    pub async fn sync_all_platforms(&self, brand_id: &str) -> Result<Vec<SyncResult>, String> {
        let conns = self.list_connections(brand_id).await?;
        let mut out = Vec::with_capacity(conns.len());
        for c in conns {
            match self.trigger_sync(&c.id).await {
                Ok(r) => out.push(r),
                Err(e) => out.push(SyncResult {
                    success: false,
                    platform: c.platform.clone(),
                    entity_type: "all".to_string(),
                    entity_id: c.id.clone(),
                    message: "Sync failed".to_string(),
                    errors: vec![e],
                }),
            }
        }
        Ok(out)
    }

    // -----------------------------------------------------------------------
    // Logs, platform info, mappings, queue, webhooks
    // -----------------------------------------------------------------------

    pub async fn list_sync_logs(
        &self,
        connection_id: Option<&str>,
        limit: i64,
    ) -> Result<Vec<SyncLog>, String> {
        let result = if let Some(cid) = connection_id {
            sqlx::query_as::<_, SyncLog>(
                "SELECT id, connection_id, direction, entity_type, entity_id, status, error, \
                 created_at::TEXT AS created_at FROM sync_logs \
                 WHERE connection_id = $1 ORDER BY created_at DESC LIMIT $2",
            )
            .bind(cid)
            .bind(limit)
            .fetch_all(&self.pool)
            .await
        } else {
            sqlx::query_as::<_, SyncLog>(
                "SELECT id, connection_id, direction, entity_type, entity_id, status, error, \
                 created_at::TEXT AS created_at FROM sync_logs \
                 ORDER BY created_at DESC LIMIT $1",
            )
            .bind(limit)
            .fetch_all(&self.pool)
            .await
        };
        result.map_err(|e| format!("Failed to list sync logs: {e}"))
    }

    pub async fn get_platform_info(&self, connection_id: &str) -> Result<serde_json::Value, String> {
        let conn = self.get_connection(connection_id).await?;
        let adapter = self.adapter_for(&conn)?;
        Ok(serde_json::json!({
            "platform": adapter.platform_name(),
            "connectionId": conn.id,
            "brandId": conn.brand_id,
            "syncStatus": conn.sync_status,
            "lastSyncAt": conn.last_sync_at,
        }))
    }

    pub async fn list_product_mappings(
        &self,
        connection_id: &str,
    ) -> Result<Vec<serde_json::Value>, String> {
        let rows = sqlx::query(
            "SELECT product_id, external_id, external_sku, last_synced_at::TEXT AS last_synced_at \
             FROM external_product_mapping WHERE connection_id = $1 \
             ORDER BY last_synced_at DESC NULLS LAST",
        )
        .bind(connection_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to list product mappings: {e}"))?;

        Ok(rows
            .into_iter()
            .map(|r| {
                serde_json::json!({
                    "productId": r.get::<String, _>("product_id"),
                    "externalId": r.get::<String, _>("external_id"),
                    "externalSku": r.get::<Option<String>, _>("external_sku"),
                    "lastSyncedAt": r.get::<Option<String>, _>("last_synced_at"),
                })
            })
            .collect())
    }

    /// Process pending sync_queue entries, retrying their connection's sync.
    pub async fn process_sync_queue(&self, connection_id: Option<&str>) -> Result<i64, String> {
        let rows = if let Some(cid) = connection_id {
            sqlx::query(
                "SELECT id, connection_id FROM sync_queue \
                 WHERE connection_id = $1 AND status = 'pending' \
                 ORDER BY created_at ASC LIMIT 50",
            )
            .bind(cid)
            .fetch_all(&self.pool)
            .await
        } else {
            sqlx::query(
                "SELECT id, connection_id FROM sync_queue \
                 WHERE status = 'pending' ORDER BY created_at ASC LIMIT 50",
            )
            .fetch_all(&self.pool)
            .await
        }
        .map_err(|e| format!("Failed to load sync queue: {e}"))?;

        let mut processed = 0i64;
        for row in rows {
            let qid: String = row.get("id");
            let conn_id: String = row.get("connection_id");
            let _ = sqlx::query(
                "UPDATE sync_queue SET status = 'processing', started_at = NOW() WHERE id = $1",
            )
            .bind(&qid)
            .execute(&self.pool)
            .await;

            match self.trigger_sync(&conn_id).await {
                Ok(_) => {
                    let _ = sqlx::query(
                        "UPDATE sync_queue SET status = 'completed', completed_at = NOW() WHERE id = $1",
                    )
                    .bind(&qid)
                    .execute(&self.pool)
                    .await;
                    processed += 1;
                }
                Err(e) => {
                    let _ = sqlx::query(
                        "UPDATE sync_queue SET status = 'failed', error = $1, retries = retries + 1 WHERE id = $2",
                    )
                    .bind(&e)
                    .bind(&qid)
                    .execute(&self.pool)
                    .await;
                }
            }
        }
        Ok(processed)
    }

    /// Handle an inbound webhook from a platform by re-syncing the relevant
    /// entity type for whichever connection is configured for that platform.
    pub async fn process_webhook(
        &self,
        platform: &str,
        event_type: &str,
        _payload: &str,
    ) -> Result<(), String> {
        let conn: Option<SyncConnection> = sqlx::query_as::<_, SyncConnection>(&format!(
            "SELECT {SELECT_CONNECTION_COLUMNS} FROM sync_connections \
             WHERE platform = $1 ORDER BY updated_at DESC LIMIT 1"
        ))
        .bind(platform)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("Failed to resolve connection for webhook: {e}"))?;

        let Some(conn) = conn else {
            return Err(format!(
                "No {platform} sync connection configured to handle webhook"
            ));
        };

        let adapter = self.adapter_for(&conn)?;
        let result = if event_type.contains("order") {
            adapter.sync_orders(&self.pool, &conn.brand_id).await
        } else if event_type.contains("inventory") {
            adapter.sync_inventory(&self.pool, &conn.brand_id).await
        } else {
            adapter.sync_products(&self.pool, &conn.brand_id).await
        };

        let _ = sqlx::query(
            "UPDATE webhook_events SET processed = true, error = $1 \
             WHERE platform = $2 AND event_type = $3 AND processed = false",
        )
        .bind(if result.success {
            None
        } else {
            result.errors.first().cloned()
        })
        .bind(platform)
        .bind(event_type)
        .execute(&self.pool)
        .await;

        self.log_result(&conn.id, &result).await;

        if result.success {
            Ok(())
        } else {
            Err(result.errors.join("; "))
        }
    }
}
