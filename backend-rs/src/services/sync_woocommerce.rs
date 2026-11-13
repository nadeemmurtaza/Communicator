//! WooCommerce Sync Adapter — syncs products, orders and inventory levels
//! between a brand's WooCommerce store and Communicator's commerce tables
//! via the WooCommerce REST API (v3).

use crate::models::SyncConnection;
use crate::services::sync_adapter::{SyncAdapter, SyncResult};
use serde_json::Value;
use sqlx::PgPool;

pub struct WooCommerceSyncAdapter {
    pub connection: Option<SyncConnection>,
    pub http_client: reqwest::Client,
}

impl WooCommerceSyncAdapter {
    /// Pull the store URL and REST API key pair out of the connection's
    /// `credentials` JSON blob.
    fn creds(&self) -> Result<(String, String, String), String> {
        let conn = self
            .connection
            .as_ref()
            .ok_or_else(|| "No WooCommerce connection configured".to_string())?;
        let raw = conn.credentials.as_deref().unwrap_or("{}");
        let json: Value = serde_json::from_str(raw).unwrap_or(Value::Null);

        let store_url = json
            .get("store_url")
            .or_else(|| json.get("site_url"))
            .or_else(|| json.get("domain"))
            .and_then(|v| v.as_str())
            .map(|s| s.trim_end_matches('/').to_string());
        let key = json
            .get("consumer_key")
            .or_else(|| json.get("api_key"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        let secret = json
            .get("consumer_secret")
            .or_else(|| json.get("api_secret"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        match (store_url, key, secret) {
            (Some(u), Some(k), Some(s)) if !u.is_empty() && !k.is_empty() => Ok((u, k, s)),
            _ => Err(
                "WooCommerce connection is missing store_url/consumer_key/consumer_secret in its credentials"
                    .to_string(),
            ),
        }
    }

    fn connection_id(&self) -> String {
        self.connection
            .as_ref()
            .map(|c| c.id.clone())
            .unwrap_or_default()
    }

    /// Page through a WooCommerce list endpoint with `page=N`, stopping once
    /// a page comes back with fewer than `per_page` items (WooCommerce has
    /// no items left once a short page is seen; the `X-WP-TotalPages`
    /// response header is an alternative signal but isn't guaranteed to be
    /// present on every store/proxy setup, so this is the more robust check).
    async fn fetch_all_paginated(
        &self,
        store_url: &str,
        key: &str,
        secret: &str,
        path_with_query: &str,
        per_page: usize,
    ) -> Result<Vec<Value>, String> {
        let mut all = Vec::new();
        let mut page = 1u32;
        loop {
            let url = format!("{store_url}{path_with_query}&page={page}");
            let resp = self
                .http_client
                .get(&url)
                .basic_auth(key, Some(secret))
                .send()
                .await
                .map_err(|e| format!("Failed to reach WooCommerce: {e}"))?;
            let items: Vec<Value> = resp
                .json()
                .await
                .map_err(|e| format!("Failed to parse WooCommerce response: {e}"))?;

            let got = items.len();
            all.extend(items);
            if got < per_page {
                break;
            }
            page += 1;
        }
        Ok(all)
    }

    async fn upsert_product(&self, pool: &PgPool, brand_id: &str, p: &Value) -> Result<(), String> {
        let external_id = p.get("id").map(|v| v.to_string()).unwrap_or_default();
        let name = p.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let description = p
            .get("description")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let price: f64 = p
            .get("price")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse().ok())
            .unwrap_or(0.0);
        let sku = p.get("sku").and_then(|v| v.as_str()).map(|s| s.to_string());
        let inventory_count: i32 = p
            .get("stock_quantity")
            .and_then(|v| v.as_i64())
            .unwrap_or(0) as i32;
        let images: Vec<String> = p
            .get("images")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|i| i.get("src").and_then(|s| s.as_str()).map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();
        let images_json = serde_json::to_string(&images).unwrap_or_else(|_| "[]".to_string());

        let existing_product_id: Option<String> = sqlx::query_scalar(
            "SELECT product_id FROM external_product_mapping \
             WHERE platform = 'woocommerce' AND connection_id = $1 AND external_id = $2",
        )
        .bind(self.connection_id())
        .bind(&external_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| format!("Failed to look up product mapping: {e}"))?;

        let product_id = if let Some(pid) = existing_product_id {
            sqlx::query(
                "UPDATE products SET name = $1, description = $2, sku = $3, price = $4, \
                 images = $5, inventory_count = $6, updated_at = NOW() WHERE id = $7",
            )
            .bind(&name)
            .bind(&description)
            .bind(&sku)
            .bind(price)
            .bind(&images_json)
            .bind(inventory_count)
            .bind(&pid)
            .execute(pool)
            .await
            .map_err(|e| format!("Failed to update product: {e}"))?;
            pid
        } else {
            let new_id = format!("prod_woocommerce_{external_id}");
            sqlx::query(
                "INSERT INTO products (id, brand_id, name, description, sku, price, images, \
                 inventory_count, track_inventory, is_active, created_at, updated_at) \
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8, true, true, NOW(), NOW()) \
                 ON CONFLICT (id) DO NOTHING",
            )
            .bind(&new_id)
            .bind(brand_id)
            .bind(&name)
            .bind(&description)
            .bind(&sku)
            .bind(price)
            .bind(&images_json)
            .bind(inventory_count)
            .execute(pool)
            .await
            .map_err(|e| format!("Failed to insert product: {e}"))?;

            sqlx::query(
                "INSERT INTO external_product_mapping (product_id, platform, connection_id, \
                 external_id, external_sku, last_synced_at) \
                 VALUES ($1, 'woocommerce', $2, $3, $4, NOW()) \
                 ON CONFLICT (product_id, platform, connection_id) \
                 DO UPDATE SET last_synced_at = NOW()",
            )
            .bind(&new_id)
            .bind(self.connection_id())
            .bind(&external_id)
            .bind(&sku)
            .execute(pool)
            .await
            .map_err(|e| format!("Failed to record product mapping: {e}"))?;
            new_id
        };

        let _ = sqlx::query(
            "UPDATE external_product_mapping SET last_synced_at = NOW() \
             WHERE product_id = $1 AND platform = 'woocommerce' AND connection_id = $2",
        )
        .bind(&product_id)
        .bind(self.connection_id())
        .execute(pool)
        .await;

        Ok(())
    }

    async fn find_or_create_customer(&self, pool: &PgPool, order: &Value) -> Result<String, String> {
        let woo_customer_id = order.get("customer_id").and_then(|v| v.as_i64()).filter(|id| *id != 0);
        let billing = order.get("billing").cloned().unwrap_or(Value::Null);
        let first = billing.get("first_name").and_then(|v| v.as_str()).unwrap_or("");
        let last = billing.get("last_name").and_then(|v| v.as_str()).unwrap_or("");
        let email = billing.get("email").and_then(|v| v.as_str()).unwrap_or("");
        let name = format!("{first} {last}").trim().to_string();
        let name = if name.is_empty() { email.to_string() } else { name };

        let customer_id = match woo_customer_id {
            Some(cid) => format!("cust_woocommerce_{cid}"),
            None => format!("cust_woocommerce_guest_{}", uuid::Uuid::new_v4().as_hyphenated()),
        };

        sqlx::query(
            "INSERT INTO customers (id, tenant_key, display_name, created_at) \
             VALUES ($1, 'newax', $2, NOW()) ON CONFLICT (id) DO NOTHING",
        )
        .bind(&customer_id)
        .bind(&name)
        .execute(pool)
        .await
        .map_err(|e| format!("Failed to upsert customer: {e}"))?;

        Ok(customer_id)
    }

    async fn upsert_order(&self, pool: &PgPool, brand_id: &str, o: &Value) -> Result<(), String> {
        let external_id = o.get("id").map(|v| v.to_string()).unwrap_or_default();
        let order_number = o
            .get("number")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| external_id.clone());
        let status = o.get("status").and_then(|v| v.as_str()).unwrap_or("pending").to_string();
        let total_amount: f64 = o
            .get("total")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse().ok())
            .unwrap_or(0.0);
        let currency = o.get("currency").and_then(|v| v.as_str()).unwrap_or("USD").to_string();

        let customer_id = self.find_or_create_customer(pool, o).await?;

        let existing_order_id: Option<String> = sqlx::query_scalar(
            "SELECT order_id FROM external_order_mapping \
             WHERE platform = 'woocommerce' AND connection_id = $1 AND external_id = $2",
        )
        .bind(self.connection_id())
        .bind(&external_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| format!("Failed to look up order mapping: {e}"))?;

        if let Some(oid) = existing_order_id {
            sqlx::query(
                "UPDATE commerce_orders SET status = $1, payment_status = $1, \
                 total_amount = $2, updated_at = NOW() WHERE id = $3",
            )
            .bind(&status)
            .bind(total_amount)
            .bind(&oid)
            .execute(pool)
            .await
            .map_err(|e| format!("Failed to update order: {e}"))?;
        } else {
            let new_id = format!("order_woocommerce_{external_id}");
            sqlx::query(
                "INSERT INTO commerce_orders (id, brand_id, customer_id, order_number, \
                 channel_type, status, total_amount, currency, payment_status, \
                 source, external_order_id, created_at, updated_at) \
                 VALUES ($1, $2, $3, $4, 'woocommerce', $5, $6, $7, $5, 'woocommerce', $8, NOW(), NOW()) \
                 ON CONFLICT (id) DO NOTHING",
            )
            .bind(&new_id)
            .bind(brand_id)
            .bind(&customer_id)
            .bind(&order_number)
            .bind(&status)
            .bind(total_amount)
            .bind(&currency)
            .bind(&external_id)
            .execute(pool)
            .await
            .map_err(|e| format!("Failed to insert order: {e}"))?;

            sqlx::query(
                "INSERT INTO external_order_mapping (order_id, platform, connection_id, \
                 external_id, external_order_number, last_synced_at) \
                 VALUES ($1, 'woocommerce', $2, $3, $4, NOW()) \
                 ON CONFLICT (order_id, platform, connection_id) \
                 DO UPDATE SET last_synced_at = NOW()",
            )
            .bind(&new_id)
            .bind(self.connection_id())
            .bind(&external_id)
            .bind(&order_number)
            .execute(pool)
            .await
            .map_err(|e| format!("Failed to record order mapping: {e}"))?;
        }

        Ok(())
    }
}

#[async_trait::async_trait]
impl SyncAdapter for WooCommerceSyncAdapter {
    fn platform_name(&self) -> &str {
        "woocommerce"
    }

    async fn sync_products(&self, pool: &PgPool, brand_id: &str) -> SyncResult {
        let (store_url, key, secret) = match self.creds() {
            Ok(c) => c,
            Err(e) => {
                return SyncResult {
                    success: false,
                    platform: "woocommerce".to_string(),
                    entity_type: "products".to_string(),
                    entity_id: String::new(),
                    message: "WooCommerce products sync failed".to_string(),
                    errors: vec![e],
                }
            }
        };

        let products = match self
            .fetch_all_paginated(&store_url, &key, &secret, "/wp-json/wc/v3/products?per_page=100", 100)
            .await
        {
            Ok(p) => p,
            Err(e) => {
                return SyncResult {
                    success: false,
                    platform: "woocommerce".to_string(),
                    entity_type: "products".to_string(),
                    entity_id: String::new(),
                    message: "Failed to fetch WooCommerce products".to_string(),
                    errors: vec![e],
                }
            }
        };

        let mut errors = Vec::new();
        for p in &products {
            if let Err(e) = self.upsert_product(pool, brand_id, p).await {
                errors.push(e);
            }
        }

        SyncResult {
            success: errors.is_empty(),
            platform: "woocommerce".to_string(),
            entity_type: "products".to_string(),
            entity_id: String::new(),
            message: format!("Synced {} WooCommerce products", products.len()),
            errors,
        }
    }

    async fn sync_orders(&self, pool: &PgPool, brand_id: &str) -> SyncResult {
        let (store_url, key, secret) = match self.creds() {
            Ok(c) => c,
            Err(e) => {
                return SyncResult {
                    success: false,
                    platform: "woocommerce".to_string(),
                    entity_type: "orders".to_string(),
                    entity_id: String::new(),
                    message: "WooCommerce orders sync failed".to_string(),
                    errors: vec![e],
                }
            }
        };

        let orders = match self
            .fetch_all_paginated(&store_url, &key, &secret, "/wp-json/wc/v3/orders?per_page=100", 100)
            .await
        {
            Ok(o) => o,
            Err(e) => {
                return SyncResult {
                    success: false,
                    platform: "woocommerce".to_string(),
                    entity_type: "orders".to_string(),
                    entity_id: String::new(),
                    message: "Failed to fetch WooCommerce orders".to_string(),
                    errors: vec![e],
                }
            }
        };

        let mut errors = Vec::new();
        for o in &orders {
            if let Err(e) = self.upsert_order(pool, brand_id, o).await {
                errors.push(e);
            }
        }

        SyncResult {
            success: errors.is_empty(),
            platform: "woocommerce".to_string(),
            entity_type: "orders".to_string(),
            entity_id: String::new(),
            message: format!("Synced {} WooCommerce orders", orders.len()),
            errors,
        }
    }

    async fn sync_inventory(&self, pool: &PgPool, brand_id: &str) -> SyncResult {
        // WooCommerce reports stock on the product payload itself, so
        // inventory sync re-fetches products and refreshes stock counts.
        let result = self.sync_products(pool, brand_id).await;
        SyncResult {
            entity_type: "inventory".to_string(),
            message: format!(
                "Refreshed inventory counts from {} WooCommerce products",
                result.message
            ),
            ..result
        }
    }
}
