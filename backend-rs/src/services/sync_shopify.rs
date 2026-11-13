//! Shopify Sync Adapter — syncs products, orders and inventory levels
//! between a brand's Shopify store and Communicator's commerce tables via
//! the Shopify Admin REST API.

use crate::models::SyncConnection;
use crate::services::sync_adapter::{SyncAdapter, SyncResult};
use serde_json::Value;
use sqlx::PgPool;

const API_VERSION: &str = "2024-01";

pub struct ShopifySyncAdapter {
    pub connection: Option<SyncConnection>,
    pub http_client: reqwest::Client,
}

/// Parse a RFC 5988 `Link` header and return the URL for `rel="next"`, if
/// present. Shopify hands back a ready-to-call URL (with the `page_info`
/// cursor baked in), so following pagination is just re-issuing GETs against
/// this value until it's no longer present.
fn parse_shopify_next_link(value: &str) -> Option<String> {
    for part in value.split(',') {
        let mut segments = part.trim().split(';');
        let url_part = segments.next()?.trim();
        let is_next = segments.any(|seg| {
            let seg = seg.trim();
            seg == "rel=\"next\"" || seg == "rel=next"
        });
        if is_next {
            return Some(url_part.trim_start_matches('<').trim_end_matches('>').to_string());
        }
    }
    None
}

impl ShopifySyncAdapter {
    /// Pull the shop domain and access token out of the connection's
    /// `credentials` JSON blob. Accepts a couple of plausible key spellings
    /// so the frontend's free-form credentials object isn't over-constrained.
    fn creds(&self) -> Result<(String, String), String> {
        let conn = self
            .connection
            .as_ref()
            .ok_or_else(|| "No Shopify connection configured".to_string())?;
        let raw = conn.credentials.as_deref().unwrap_or("{}");
        let json: Value = serde_json::from_str(raw).unwrap_or(Value::Null);

        let domain = json
            .get("shop_domain")
            .or_else(|| json.get("store_domain"))
            .or_else(|| json.get("domain"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        let token = json
            .get("access_token")
            .or_else(|| json.get("api_key"))
            .or_else(|| json.get("token"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        match (domain, token) {
            (Some(d), Some(t)) if !d.is_empty() && !t.is_empty() => Ok((d, t)),
            _ => Err(
                "Shopify connection is missing shop_domain/access_token in its credentials"
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

    /// GET `url` and every subsequent page the `Link: rel="next"` header
    /// points at, collecting the named top-level array across all pages.
    async fn fetch_all_paginated(
        &self,
        token: &str,
        mut url: String,
        array_key: &str,
    ) -> Result<Vec<Value>, String> {
        let mut all = Vec::new();
        loop {
            let resp = self
                .http_client
                .get(&url)
                .header("X-Shopify-Access-Token", token)
                .send()
                .await
                .map_err(|e| format!("Failed to reach Shopify: {e}"))?;

            let next_url = resp
                .headers()
                .get(reqwest::header::LINK)
                .and_then(|v| v.to_str().ok())
                .and_then(parse_shopify_next_link);

            let body: Value = resp
                .json()
                .await
                .map_err(|e| format!("Failed to parse Shopify response: {e}"))?;

            let page = body.get(array_key).and_then(|v| v.as_array()).cloned().unwrap_or_default();
            let page_was_empty = page.is_empty();
            all.extend(page);

            match next_url {
                Some(n) if !page_was_empty => url = n,
                _ => break,
            }
        }
        Ok(all)
    }

    async fn upsert_product(&self, pool: &PgPool, brand_id: &str, p: &Value) -> Result<(), String> {
        let external_id = p.get("id").map(|v| v.to_string()).unwrap_or_default();
        let name = p.get("title").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let description = p
            .get("body_html")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let variants = p.get("variants").and_then(|v| v.as_array()).cloned().unwrap_or_default();
        let first_variant = variants.first();
        let price: f64 = first_variant
            .and_then(|v| v.get("price"))
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse().ok())
            .unwrap_or(0.0);
        let sku = first_variant
            .and_then(|v| v.get("sku"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        let inventory_count: i32 = variants
            .iter()
            .filter_map(|v| v.get("inventory_quantity").and_then(|q| q.as_i64()))
            .sum::<i64>() as i32;
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
             WHERE platform = 'shopify' AND connection_id = $1 AND external_id = $2",
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
            let new_id = format!("prod_shopify_{external_id}");
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
                 VALUES ($1, 'shopify', $2, $3, $4, NOW()) \
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
             WHERE product_id = $1 AND platform = 'shopify' AND connection_id = $2",
        )
        .bind(&product_id)
        .bind(self.connection_id())
        .execute(pool)
        .await;

        Ok(())
    }

    async fn find_or_create_customer(&self, pool: &PgPool, order: &Value) -> Result<String, String> {
        let shopify_customer_id = order
            .get("customer")
            .and_then(|c| c.get("id"))
            .map(|v| v.to_string());
        let email = order.get("email").and_then(|v| v.as_str()).unwrap_or("");
        let name = order
            .get("customer")
            .map(|c| {
                let first = c.get("first_name").and_then(|v| v.as_str()).unwrap_or("");
                let last = c.get("last_name").and_then(|v| v.as_str()).unwrap_or("");
                format!("{first} {last}").trim().to_string()
            })
            .filter(|n| !n.is_empty())
            .unwrap_or_else(|| email.to_string());

        let customer_id = match shopify_customer_id {
            Some(cid) => format!("cust_shopify_{cid}"),
            None => format!("cust_shopify_guest_{}", uuid::Uuid::new_v4().as_hyphenated()),
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
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or(&external_id)
            .to_string();
        let status = o
            .get("financial_status")
            .and_then(|v| v.as_str())
            .unwrap_or("pending")
            .to_string();
        let total_amount: f64 = o
            .get("total_price")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse().ok())
            .unwrap_or(0.0);
        let subtotal: f64 = o
            .get("subtotal_price")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse().ok())
            .unwrap_or(0.0);
        let currency = o
            .get("currency")
            .and_then(|v| v.as_str())
            .unwrap_or("USD")
            .to_string();

        let customer_id = self.find_or_create_customer(pool, o).await?;

        let existing_order_id: Option<String> = sqlx::query_scalar(
            "SELECT order_id FROM external_order_mapping \
             WHERE platform = 'shopify' AND connection_id = $1 AND external_id = $2",
        )
        .bind(self.connection_id())
        .bind(&external_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| format!("Failed to look up order mapping: {e}"))?;

        if let Some(oid) = existing_order_id {
            sqlx::query(
                "UPDATE commerce_orders SET status = $1, payment_status = $2, \
                 total_amount = $3, subtotal = $4, updated_at = NOW() WHERE id = $5",
            )
            .bind(&status)
            .bind(&status)
            .bind(total_amount)
            .bind(subtotal)
            .bind(&oid)
            .execute(pool)
            .await
            .map_err(|e| format!("Failed to update order: {e}"))?;
        } else {
            let new_id = format!("order_shopify_{external_id}");
            sqlx::query(
                "INSERT INTO commerce_orders (id, brand_id, customer_id, order_number, \
                 channel_type, status, total_amount, subtotal, currency, payment_status, \
                 source, external_order_id, created_at, updated_at) \
                 VALUES ($1, $2, $3, $4, 'shopify', $5, $6, $7, $8, $5, 'shopify', $9, NOW(), NOW()) \
                 ON CONFLICT (id) DO NOTHING",
            )
            .bind(&new_id)
            .bind(brand_id)
            .bind(&customer_id)
            .bind(&order_number)
            .bind(&status)
            .bind(total_amount)
            .bind(subtotal)
            .bind(&currency)
            .bind(&external_id)
            .execute(pool)
            .await
            .map_err(|e| format!("Failed to insert order: {e}"))?;

            sqlx::query(
                "INSERT INTO external_order_mapping (order_id, platform, connection_id, \
                 external_id, external_order_number, last_synced_at) \
                 VALUES ($1, 'shopify', $2, $3, $4, NOW()) \
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
impl SyncAdapter for ShopifySyncAdapter {
    fn platform_name(&self) -> &str {
        "shopify"
    }

    async fn sync_products(&self, pool: &PgPool, brand_id: &str) -> SyncResult {
        let (domain, token) = match self.creds() {
            Ok(c) => c,
            Err(e) => {
                return SyncResult {
                    success: false,
                    platform: "shopify".to_string(),
                    entity_type: "products".to_string(),
                    entity_id: String::new(),
                    message: "Shopify products sync failed".to_string(),
                    errors: vec![e],
                }
            }
        };

        let url = format!("https://{domain}/admin/api/{API_VERSION}/products.json?limit=250");
        let products = match self.fetch_all_paginated(&token, url, "products").await {
            Ok(p) => p,
            Err(e) => {
                return SyncResult {
                    success: false,
                    platform: "shopify".to_string(),
                    entity_type: "products".to_string(),
                    entity_id: String::new(),
                    message: "Failed to fetch Shopify products".to_string(),
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
            platform: "shopify".to_string(),
            entity_type: "products".to_string(),
            entity_id: String::new(),
            message: format!("Synced {} Shopify products", products.len()),
            errors,
        }
    }

    async fn sync_orders(&self, pool: &PgPool, brand_id: &str) -> SyncResult {
        let (domain, token) = match self.creds() {
            Ok(c) => c,
            Err(e) => {
                return SyncResult {
                    success: false,
                    platform: "shopify".to_string(),
                    entity_type: "orders".to_string(),
                    entity_id: String::new(),
                    message: "Shopify orders sync failed".to_string(),
                    errors: vec![e],
                }
            }
        };

        let url = format!(
            "https://{domain}/admin/api/{API_VERSION}/orders.json?status=any&limit=250"
        );
        let orders = match self.fetch_all_paginated(&token, url, "orders").await {
            Ok(o) => o,
            Err(e) => {
                return SyncResult {
                    success: false,
                    platform: "shopify".to_string(),
                    entity_type: "orders".to_string(),
                    entity_id: String::new(),
                    message: "Failed to fetch Shopify orders".to_string(),
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
            platform: "shopify".to_string(),
            entity_type: "orders".to_string(),
            entity_id: String::new(),
            message: format!("Synced {} Shopify orders", orders.len()),
            errors,
        }
    }

    async fn sync_inventory(&self, pool: &PgPool, brand_id: &str) -> SyncResult {
        // Shopify reports inventory on the product/variants payload itself,
        // so inventory sync re-fetches products and refreshes stock counts.
        let result = self.sync_products(pool, brand_id).await;
        SyncResult {
            entity_type: "inventory".to_string(),
            message: format!(
                "Refreshed inventory counts from {} Shopify products",
                result.message
            ),
            ..result
        }
    }
}
