//! Wix Sync Adapter — syncs products, orders and inventory levels between a
//! brand's Wix site and Communicator's commerce tables via the Wix REST
//! APIs (Stores Catalog V1, eCommerce Orders V1, Stores Inventory V2).

use crate::models::SyncConnection;
use crate::services::sync_adapter::{SyncAdapter, SyncResult};
use serde_json::Value;
use sqlx::PgPool;

const WIX_API_BASE: &str = "https://www.wixapis.com";

pub struct WixSyncAdapter {
    pub connection: Option<SyncConnection>,
    pub http_client: reqwest::Client,
}

impl WixSyncAdapter {
    /// Pull the API key and site id out of the connection's `credentials`
    /// JSON blob.
    fn creds(&self) -> Result<(String, String), String> {
        let conn = self
            .connection
            .as_ref()
            .ok_or_else(|| "No Wix connection configured".to_string())?;
        let raw = conn.credentials.as_deref().unwrap_or("{}");
        let json: Value = serde_json::from_str(raw).unwrap_or(Value::Null);

        let api_key = json
            .get("api_key")
            .or_else(|| json.get("access_token"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        let site_id = json
            .get("site_id")
            .or_else(|| json.get("siteId"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        match (api_key, site_id) {
            (Some(k), Some(s)) if !k.is_empty() && !s.is_empty() => Ok((k, s)),
            _ => Err("Wix connection is missing api_key/site_id in its credentials".to_string()),
        }
    }

    fn connection_id(&self) -> String {
        self.connection
            .as_ref()
            .map(|c| c.id.clone())
            .unwrap_or_default()
    }

    async fn wix_post(&self, api_key: &str, site_id: &str, path: &str, body: &Value) -> Result<Value, String> {
        let url = format!("{WIX_API_BASE}{path}");
        let resp = self
            .http_client
            .post(&url)
            .header("Authorization", api_key)
            .header("wix-site-id", site_id)
            .header("Content-Type", "application/json")
            .json(body)
            .send()
            .await
            .map_err(|e| format!("Wix POST {path} failed: {e}"))?;
        if !resp.status().is_success() {
            let text = resp.text().await.unwrap_or_default();
            return Err(format!("Wix POST {path} returned {text}"));
        }
        resp.json().await.map_err(|e| format!("Failed to parse Wix response: {e}"))
    }

    /// Page through a Wix query endpoint using `query.paging.limit/offset`,
    /// collecting the named top-level array across every page. Stops once a
    /// page comes back with fewer than `limit` items (and also refetches one
    /// more page past an exact-multiple-of-`limit` result set, which simply
    /// comes back empty and stops the loop) -- the same "short/empty page
    /// means last page" rule Wix's V1/V2 list endpoints all honor regardless
    /// of whether a given endpoint additionally echoes a cursor/offset count.
    async fn fetch_all_wix_paged(
        &self,
        api_key: &str,
        site_id: &str,
        path: &str,
        array_key: &str,
        build_payload: impl Fn(i64, i64) -> Value,
    ) -> Result<Vec<Value>, String> {
        const LIMIT: i64 = 100;
        let mut all = Vec::new();
        let mut offset: i64 = 0;
        loop {
            let payload = build_payload(LIMIT, offset);
            let body = self.wix_post(api_key, site_id, path, &payload).await?;
            let page = body.get(array_key).and_then(|v| v.as_array()).cloned().unwrap_or_default();
            let page_len = page.len() as i64;
            all.extend(page);
            if page_len < LIMIT {
                break;
            }
            offset += LIMIT;
        }
        Ok(all)
    }

    async fn upsert_product(&self, pool: &PgPool, brand_id: &str, p: &Value) -> Result<(), String> {
        let external_id = p.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();
        if external_id.is_empty() {
            return Err("Wix product has no id".to_string());
        }
        let name = p.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let description = p
            .get("description")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let price: f64 = p
            .get("price")
            .and_then(|pr| pr.get("price"))
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);
        let sku = p.get("sku").and_then(|v| v.as_str()).map(|s| s.to_string());
        let inventory_count: i32 = p
            .get("stock")
            .and_then(|s| s.get("quantity"))
            .and_then(|v| v.as_i64())
            .unwrap_or(0) as i32;

        let existing_product_id: Option<String> = sqlx::query_scalar(
            "SELECT product_id FROM external_product_mapping \
             WHERE platform = 'wix' AND connection_id = $1 AND external_id = $2",
        )
        .bind(self.connection_id())
        .bind(&external_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| format!("Failed to look up product mapping: {e}"))?;

        let product_id = if let Some(pid) = existing_product_id {
            sqlx::query(
                "UPDATE products SET name = $1, description = $2, sku = $3, price = $4, \
                 inventory_count = $5, updated_at = NOW() WHERE id = $6",
            )
            .bind(&name)
            .bind(&description)
            .bind(&sku)
            .bind(price)
            .bind(inventory_count)
            .bind(&pid)
            .execute(pool)
            .await
            .map_err(|e| format!("Failed to update product: {e}"))?;
            pid
        } else {
            let new_id = format!("prod_wix_{external_id}");
            sqlx::query(
                "INSERT INTO products (id, brand_id, name, description, sku, price, \
                 inventory_count, track_inventory, is_active, created_at, updated_at) \
                 VALUES ($1, $2, $3, $4, $5, $6, $7, true, true, NOW(), NOW()) \
                 ON CONFLICT (id) DO NOTHING",
            )
            .bind(&new_id)
            .bind(brand_id)
            .bind(&name)
            .bind(&description)
            .bind(&sku)
            .bind(price)
            .bind(inventory_count)
            .execute(pool)
            .await
            .map_err(|e| format!("Failed to insert product: {e}"))?;

            sqlx::query(
                "INSERT INTO external_product_mapping (product_id, platform, connection_id, \
                 external_id, external_sku, last_synced_at) \
                 VALUES ($1, 'wix', $2, $3, $4, NOW()) \
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
             WHERE product_id = $1 AND platform = 'wix' AND connection_id = $2",
        )
        .bind(&product_id)
        .bind(self.connection_id())
        .execute(pool)
        .await;

        Ok(())
    }

    async fn find_or_create_customer(&self, pool: &PgPool, order: &Value) -> Result<String, String> {
        let wix_buyer_id = order
            .get("buyerInfo")
            .and_then(|b| b.get("id"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        let email = order
            .get("buyerInfo")
            .and_then(|b| b.get("email"))
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let contact = order.get("billingInfo").and_then(|b| b.get("contactDetails")).cloned().unwrap_or(Value::Null);
        let first = contact.get("firstName").and_then(|v| v.as_str()).unwrap_or("");
        let last = contact.get("lastName").and_then(|v| v.as_str()).unwrap_or("");
        let name = format!("{first} {last}").trim().to_string();
        let name = if name.is_empty() { email.to_string() } else { name };

        let customer_id = match wix_buyer_id {
            Some(bid) => format!("cust_wix_{bid}"),
            None => format!("cust_wix_guest_{}", uuid::Uuid::new_v4().as_hyphenated()),
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
        let external_id = o.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();
        if external_id.is_empty() {
            return Err("Wix order has no id".to_string());
        }
        let order_number = o
            .get("number")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| external_id.clone());
        let wix_status = o.get("status").and_then(|v| v.as_str()).unwrap_or("APPROVED");
        let status = match wix_status {
            "FULFILLED" => "delivered",
            "PARTIALLY_FULFILLED" => "shipped",
            "CANCELED" => "cancelled",
            "APPROVED" => "confirmed",
            _ => "pending",
        }
        .to_string();
        let payment_status = match o.get("paymentStatus").and_then(|v| v.as_str()).unwrap_or("") {
            "PAID" => "paid",
            "PARTIALLY_PAID" => "partially_paid",
            "FULLY_REFUNDED" => "refunded",
            _ => "pending",
        }
        .to_string();
        let total_amount: f64 = o
            .get("priceSummary")
            .and_then(|p| p.get("total"))
            .and_then(|t| t.get("amount"))
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse().ok())
            .unwrap_or(0.0);
        let subtotal: f64 = o
            .get("priceSummary")
            .and_then(|p| p.get("subtotal"))
            .and_then(|t| t.get("amount"))
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse().ok())
            .unwrap_or(total_amount);
        let currency = o.get("currency").and_then(|v| v.as_str()).unwrap_or("USD").to_string();

        let customer_id = self.find_or_create_customer(pool, o).await?;

        let existing_order_id: Option<String> = sqlx::query_scalar(
            "SELECT order_id FROM external_order_mapping \
             WHERE platform = 'wix' AND connection_id = $1 AND external_id = $2",
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
            .bind(&payment_status)
            .bind(total_amount)
            .bind(subtotal)
            .bind(&oid)
            .execute(pool)
            .await
            .map_err(|e| format!("Failed to update order: {e}"))?;
        } else {
            let new_id = format!("order_wix_{external_id}");
            sqlx::query(
                "INSERT INTO commerce_orders (id, brand_id, customer_id, order_number, \
                 channel_type, status, total_amount, subtotal, currency, payment_status, \
                 source, external_order_id, created_at, updated_at) \
                 VALUES ($1, $2, $3, $4, 'wix', $5, $6, $7, $8, $9, 'wix', $10, NOW(), NOW()) \
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
            .bind(&payment_status)
            .bind(&external_id)
            .execute(pool)
            .await
            .map_err(|e| format!("Failed to insert order: {e}"))?;

            sqlx::query(
                "INSERT INTO external_order_mapping (order_id, platform, connection_id, \
                 external_id, external_order_number, last_synced_at) \
                 VALUES ($1, 'wix', $2, $3, $4, NOW()) \
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
impl SyncAdapter for WixSyncAdapter {
    fn platform_name(&self) -> &str {
        "wix"
    }

    async fn sync_products(&self, pool: &PgPool, brand_id: &str) -> SyncResult {
        let (api_key, site_id) = match self.creds() {
            Ok(c) => c,
            Err(e) => {
                return SyncResult {
                    success: false,
                    platform: "wix".to_string(),
                    entity_type: "products".to_string(),
                    entity_id: String::new(),
                    message: "Wix products sync failed".to_string(),
                    errors: vec![e],
                }
            }
        };

        // Wix Stores Catalog V1: query products, paging through every page.
        let products = match self
            .fetch_all_wix_paged(
                &api_key,
                &site_id,
                "/stores/v1/products/query",
                "products",
                |limit, offset| serde_json::json!({ "query": { "paging": { "limit": limit, "offset": offset } } }),
            )
            .await
        {
            Ok(p) => p,
            Err(e) => {
                return SyncResult {
                    success: false,
                    platform: "wix".to_string(),
                    entity_type: "products".to_string(),
                    entity_id: String::new(),
                    message: "Failed to reach Wix Stores Catalog API".to_string(),
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
            platform: "wix".to_string(),
            entity_type: "products".to_string(),
            entity_id: String::new(),
            message: format!("Synced {} Wix products", products.len()),
            errors,
        }
    }

    async fn sync_orders(&self, pool: &PgPool, brand_id: &str) -> SyncResult {
        let (api_key, site_id) = match self.creds() {
            Ok(c) => c,
            Err(e) => {
                return SyncResult {
                    success: false,
                    platform: "wix".to_string(),
                    entity_type: "orders".to_string(),
                    entity_id: String::new(),
                    message: "Wix orders sync failed".to_string(),
                    errors: vec![e],
                }
            }
        };

        // Wix eCommerce Orders V1: query orders, paging through every page.
        let orders = match self
            .fetch_all_wix_paged(
                &api_key,
                &site_id,
                "/ecom/v1/orders/query",
                "orders",
                |limit, offset| serde_json::json!({ "query": { "paging": { "limit": limit, "offset": offset } } }),
            )
            .await
        {
            Ok(o) => o,
            Err(e) => {
                return SyncResult {
                    success: false,
                    platform: "wix".to_string(),
                    entity_type: "orders".to_string(),
                    entity_id: String::new(),
                    message: "Failed to reach Wix Orders API".to_string(),
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
            platform: "wix".to_string(),
            entity_type: "orders".to_string(),
            entity_id: String::new(),
            message: format!("Synced {} Wix orders", orders.len()),
            errors,
        }
    }

    async fn sync_inventory(&self, pool: &PgPool, brand_id: &str) -> SyncResult {
        let (api_key, site_id) = match self.creds() {
            Ok(c) => c,
            Err(e) => {
                return SyncResult {
                    success: false,
                    platform: "wix".to_string(),
                    entity_type: "inventory".to_string(),
                    entity_id: String::new(),
                    message: "Wix inventory sync failed".to_string(),
                    errors: vec![e],
                }
            }
        };

        let mappings: Vec<(String, String)> = match sqlx::query_as(
            "SELECT product_id, external_id FROM external_product_mapping \
             WHERE platform = 'wix' AND connection_id = $1",
        )
        .bind(self.connection_id())
        .fetch_all(pool)
        .await
        {
            Ok(rows) => rows,
            Err(e) => {
                return SyncResult {
                    success: false,
                    platform: "wix".to_string(),
                    entity_type: "inventory".to_string(),
                    entity_id: String::new(),
                    message: "Failed to load Wix product mappings".to_string(),
                    errors: vec![e.to_string()],
                }
            }
        };

        if mappings.is_empty() {
            return SyncResult {
                success: true,
                platform: "wix".to_string(),
                entity_type: "inventory".to_string(),
                entity_id: String::new(),
                message: "No mapped Wix products to refresh inventory for".to_string(),
                errors: vec![],
            };
        }

        // Wix Stores Inventory V2: query inventory items by product id,
        // paging through every page.
        let product_ids: Vec<String> = mappings.iter().map(|(_, ext)| ext.clone()).collect();
        let items = match self
            .fetch_all_wix_paged(
                &api_key,
                &site_id,
                "/stores/v2/inventoryItems/query",
                "inventoryItems",
                move |limit, offset| {
                    serde_json::json!({
                        "query": {
                            "filter": { "productIds": product_ids },
                            "paging": { "limit": limit, "offset": offset }
                        }
                    })
                },
            )
            .await
        {
            Ok(i) => i,
            Err(e) => {
                return SyncResult {
                    success: false,
                    platform: "wix".to_string(),
                    entity_type: "inventory".to_string(),
                    entity_id: String::new(),
                    message: "Failed to reach Wix Inventory API".to_string(),
                    errors: vec![e],
                }
            }
        };

        let mut by_product: std::collections::HashMap<String, i32> = std::collections::HashMap::new();
        for item in &items {
            let product_id = item.get("productId").and_then(|v| v.as_str()).unwrap_or("");
            let qty = item
                .get("variants")
                .and_then(|v| v.as_array())
                .and_then(|arr| arr.first())
                .and_then(|v| v.get("quantity"))
                .and_then(|q| q.as_i64())
                .unwrap_or(0) as i32;
            by_product.insert(product_id.to_string(), qty);
        }

        let mut synced = 0;
        let mut errors = Vec::new();
        for (product_id, external_id) in &mappings {
            let qty = by_product.get(external_id).copied().unwrap_or(0);
            match sqlx::query("UPDATE products SET inventory_count = $1, updated_at = NOW() WHERE id = $2")
                .bind(qty)
                .bind(product_id)
                .execute(pool)
                .await
            {
                Ok(_) => synced += 1,
                Err(e) => errors.push(format!("Product {product_id}: {e}")),
            }
        }
        let _ = brand_id;

        SyncResult {
            success: errors.is_empty(),
            platform: "wix".to_string(),
            entity_type: "inventory".to_string(),
            entity_id: String::new(),
            message: format!("Refreshed inventory counts for {synced} Wix products"),
            errors,
        }
    }
}
