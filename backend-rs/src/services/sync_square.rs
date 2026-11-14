//! Square Sync Adapter — syncs products, orders and inventory levels
//! between a brand's Square account and Communicator's commerce tables via
//! the Square Catalog, Orders and Inventory REST APIs.

use crate::models::SyncConnection;
use crate::services::sync_adapter::{SyncAdapter, SyncResult};
use serde_json::Value;
use sqlx::PgPool;

const SQUARE_API_BASE: &str = "https://connect.squareup.com/v2";
const SQUARE_VERSION: &str = "2024-01-18";

pub struct SquareSyncAdapter {
    pub connection: Option<SyncConnection>,
    pub http_client: reqwest::Client,
}

impl SquareSyncAdapter {
    /// Pull the OAuth access token and location id out of the connection's
    /// `credentials` JSON blob.
    fn creds(&self) -> Result<(String, String), String> {
        let conn = self
            .connection
            .as_ref()
            .ok_or_else(|| "No Square connection configured".to_string())?;
        let raw = conn.credentials.as_deref().unwrap_or("{}");
        let json: Value = serde_json::from_str(raw).unwrap_or(Value::Null);

        let token = json
            .get("access_token")
            .or_else(|| json.get("api_key"))
            .or_else(|| json.get("token"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        let location_id = json
            .get("location_id")
            .or_else(|| json.get("location"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        match (token, location_id) {
            (Some(t), Some(l)) if !t.is_empty() && !l.is_empty() => Ok((t, l)),
            _ => Err(
                "Square connection is missing access_token/location_id in its credentials"
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

    async fn square_get(&self, token: &str, path: &str) -> Result<Value, String> {
        let url = format!("{SQUARE_API_BASE}{path}");
        let resp = self
            .http_client
            .get(&url)
            .header("Authorization", format!("Bearer {token}"))
            .header("Square-Version", SQUARE_VERSION)
            .send()
            .await
            .map_err(|e| format!("Square GET {path} failed: {e}"))?;
        if !resp.status().is_success() {
            let text = resp.text().await.unwrap_or_default();
            return Err(format!("Square GET {path} returned {text}"));
        }
        resp.json().await.map_err(|e| format!("Failed to parse Square response: {e}"))
    }

    async fn square_post(&self, token: &str, path: &str, body: &Value) -> Result<Value, String> {
        let url = format!("{SQUARE_API_BASE}{path}");
        let resp = self
            .http_client
            .post(&url)
            .header("Authorization", format!("Bearer {token}"))
            .header("Square-Version", SQUARE_VERSION)
            .header("Content-Type", "application/json")
            .json(body)
            .send()
            .await
            .map_err(|e| format!("Square POST {path} failed: {e}"))?;
        if !resp.status().is_success() {
            let text = resp.text().await.unwrap_or_default();
            return Err(format!("Square POST {path} returned {text}"));
        }
        resp.json().await.map_err(|e| format!("Failed to parse Square response: {e}"))
    }

    /// Page through `/catalog/list?types=ITEM`, following the `cursor` field
    /// Square returns while more results remain.
    async fn fetch_all_catalog_items(&self, token: &str) -> Result<Vec<Value>, String> {
        let mut all = Vec::new();
        let mut cursor: Option<String> = None;
        loop {
            let path = match &cursor {
                Some(c) => format!("/catalog/list?types=ITEM&cursor={c}"),
                None => "/catalog/list?types=ITEM".to_string(),
            };
            let body = self.square_get(token, &path).await?;
            let page = body.get("objects").and_then(|v| v.as_array()).cloned().unwrap_or_default();
            all.extend(page);
            cursor = body.get("cursor").and_then(|v| v.as_str()).map(|s| s.to_string());
            if cursor.is_none() {
                break;
            }
        }
        Ok(all)
    }

    /// Page through `/orders/search`, feeding the `cursor` field Square
    /// returns back into the next request's body while more results remain.
    async fn fetch_all_orders(&self, token: &str, location_id: &str) -> Result<Vec<Value>, String> {
        let mut all = Vec::new();
        let mut cursor: Option<String> = None;
        loop {
            let mut payload = serde_json::json!({
                "location_ids": [location_id],
                "query": {
                    "sort": { "sort_field": "CREATED_AT", "sort_order": "DESC" }
                },
                "limit": 100,
            });
            if let Some(c) = &cursor {
                payload["cursor"] = Value::String(c.clone());
            }
            let body = self.square_post(token, "/orders/search", &payload).await?;
            let page = body.get("orders").and_then(|v| v.as_array()).cloned().unwrap_or_default();
            all.extend(page);
            cursor = body.get("cursor").and_then(|v| v.as_str()).map(|s| s.to_string());
            if cursor.is_none() {
                break;
            }
        }
        Ok(all)
    }

    /// Page through `/inventory/counts/batch-retrieve` the same way; it can
    /// also return a `cursor` when the requested id list's counts span more
    /// than one page.
    async fn fetch_all_inventory_counts(
        &self,
        token: &str,
        location_id: &str,
        catalog_object_ids: &[String],
    ) -> Result<Vec<Value>, String> {
        let mut all = Vec::new();
        let mut cursor: Option<String> = None;
        loop {
            let mut payload = serde_json::json!({
                "catalog_object_ids": catalog_object_ids,
                "location_ids": [location_id],
            });
            if let Some(c) = &cursor {
                payload["cursor"] = Value::String(c.clone());
            }
            let body = self.square_post(token, "/inventory/counts/batch-retrieve", &payload).await?;
            let page = body.get("counts").and_then(|v| v.as_array()).cloned().unwrap_or_default();
            all.extend(page);
            cursor = body.get("cursor").and_then(|v| v.as_str()).map(|s| s.to_string());
            if cursor.is_none() {
                break;
            }
        }
        Ok(all)
    }

    /// Upsert a Square catalog ITEM (with its first ITEM_VARIATION) into our
    /// `products` table. The variation id is used as the external_id since
    /// that's what the Inventory API keys counts on.
    async fn upsert_product(&self, pool: &PgPool, brand_id: &str, item: &Value) -> Result<(), String> {
        let item_data = item.get("item_data").cloned().unwrap_or(Value::Null);
        let name = item_data.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let description = item_data
            .get("description")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let variations = item_data.get("variations").and_then(|v| v.as_array()).cloned().unwrap_or_default();
        let first_variation = variations.first();
        let variation_data = first_variation.and_then(|v| v.get("item_variation_data"));
        let external_id = first_variation
            .and_then(|v| v.get("id"))
            .and_then(|v| v.as_str())
            .unwrap_or_else(|| item.get("id").and_then(|v| v.as_str()).unwrap_or(""))
            .to_string();
        if external_id.is_empty() {
            return Err("Square catalog item has no id".to_string());
        }
        let sku = variation_data
            .and_then(|v| v.get("sku"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        let price_cents = variation_data
            .and_then(|v| v.get("price_money"))
            .and_then(|m| m.get("amount"))
            .and_then(|a| a.as_i64())
            .unwrap_or(0);
        let price = price_cents as f64 / 100.0;

        let existing_product_id: Option<String> = sqlx::query_scalar(
            "SELECT product_id FROM external_product_mapping \
             WHERE platform = 'square' AND connection_id = $1 AND external_id = $2",
        )
        .bind(self.connection_id())
        .bind(&external_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| format!("Failed to look up product mapping: {e}"))?;

        let product_id = if let Some(pid) = existing_product_id {
            sqlx::query(
                "UPDATE products SET name = $1, description = $2, sku = $3, price = $4, \
                 updated_at = NOW() WHERE id = $5",
            )
            .bind(&name)
            .bind(&description)
            .bind(&sku)
            .bind(price)
            .bind(&pid)
            .execute(pool)
            .await
            .map_err(|e| format!("Failed to update product: {e}"))?;
            pid
        } else {
            let new_id = format!("prod_square_{external_id}");
            sqlx::query(
                "INSERT INTO products (id, brand_id, name, description, sku, price, \
                 track_inventory, is_active, created_at, updated_at) \
                 VALUES ($1, $2, $3, $4, $5, $6, true, true, NOW(), NOW()) \
                 ON CONFLICT (id) DO NOTHING",
            )
            .bind(&new_id)
            .bind(brand_id)
            .bind(&name)
            .bind(&description)
            .bind(&sku)
            .bind(price)
            .execute(pool)
            .await
            .map_err(|e| format!("Failed to insert product: {e}"))?;

            sqlx::query(
                "INSERT INTO external_product_mapping (product_id, platform, connection_id, \
                 external_id, external_sku, last_synced_at) \
                 VALUES ($1, 'square', $2, $3, $4, NOW()) \
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
             WHERE product_id = $1 AND platform = 'square' AND connection_id = $2",
        )
        .bind(&product_id)
        .bind(self.connection_id())
        .execute(pool)
        .await;

        Ok(())
    }

    /// Find or create a local customer for a Square order. Square's Orders
    /// Search response doesn't inline full buyer contact details, so we fall
    /// back to the shipment fulfillment's recipient when present, same as a
    /// guest checkout.
    async fn find_or_create_customer(&self, pool: &PgPool, order: &Value) -> Result<String, String> {
        let square_customer_id = order.get("customer_id").and_then(|v| v.as_str()).map(|s| s.to_string());
        let recipient = order
            .get("fulfillments")
            .and_then(|f| f.as_array())
            .and_then(|arr| arr.first())
            .and_then(|f| f.get("shipment_details"))
            .and_then(|sd| sd.get("recipient"))
            .cloned()
            .unwrap_or(Value::Null);
        let name = recipient
            .get("display_name")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let customer_id = match &square_customer_id {
            Some(cid) => format!("cust_square_{cid}"),
            None => format!("cust_square_guest_{}", uuid::Uuid::new_v4().as_hyphenated()),
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
            return Err("Square order has no id".to_string());
        }
        let order_number = format!("SQ-{external_id}");
        let square_state = o.get("state").and_then(|v| v.as_str()).unwrap_or("OPEN");
        let status = match square_state {
            "COMPLETED" => "delivered",
            "OPEN" => "confirmed",
            "CANCELED" => "cancelled",
            _ => "pending",
        }
        .to_string();
        let total_amount: f64 = o
            .get("total_money")
            .and_then(|m| m.get("amount"))
            .and_then(|a| a.as_i64())
            .map(|cents| cents as f64 / 100.0)
            .unwrap_or(0.0);
        let subtotal: f64 = o
            .get("net_amounts")
            .and_then(|n| n.get("total_money"))
            .and_then(|m| m.get("amount"))
            .and_then(|a| a.as_i64())
            .map(|cents| cents as f64 / 100.0)
            .unwrap_or(total_amount);
        let currency = o
            .get("total_money")
            .and_then(|m| m.get("currency"))
            .and_then(|v| v.as_str())
            .unwrap_or("USD")
            .to_string();

        let customer_id = self.find_or_create_customer(pool, o).await?;

        let existing_order_id: Option<String> = sqlx::query_scalar(
            "SELECT order_id FROM external_order_mapping \
             WHERE platform = 'square' AND connection_id = $1 AND external_id = $2",
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
            let new_id = format!("order_square_{external_id}");
            sqlx::query(
                "INSERT INTO commerce_orders (id, brand_id, customer_id, order_number, \
                 channel_type, status, total_amount, subtotal, currency, payment_status, \
                 source, external_order_id, created_at, updated_at) \
                 VALUES ($1, $2, $3, $4, 'square', $5, $6, $7, $8, $5, 'square', $9, NOW(), NOW()) \
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
                 VALUES ($1, 'square', $2, $3, $4, NOW()) \
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
impl SyncAdapter for SquareSyncAdapter {
    fn platform_name(&self) -> &str {
        "square"
    }

    async fn sync_products(&self, pool: &PgPool, brand_id: &str) -> SyncResult {
        let (token, _location_id) = match self.creds() {
            Ok(c) => c,
            Err(e) => {
                return SyncResult {
                    success: false,
                    platform: "square".to_string(),
                    entity_type: "products".to_string(),
                    entity_id: String::new(),
                    message: "Square products sync failed".to_string(),
                    errors: vec![e],
                }
            }
        };

        // Square Catalog API: list all ITEM objects (each embeds its
        // variations, which carry sku + price), following `cursor` across
        // every page.
        let items = match self.fetch_all_catalog_items(&token).await {
            Ok(i) => i,
            Err(e) => {
                return SyncResult {
                    success: false,
                    platform: "square".to_string(),
                    entity_type: "products".to_string(),
                    entity_id: String::new(),
                    message: "Failed to reach Square Catalog API".to_string(),
                    errors: vec![e],
                }
            }
        };

        let mut errors = Vec::new();
        for item in &items {
            if item.get("type").and_then(|v| v.as_str()) != Some("ITEM") {
                continue;
            }
            if let Err(e) = self.upsert_product(pool, brand_id, item).await {
                errors.push(e);
            }
        }

        SyncResult {
            success: errors.is_empty(),
            platform: "square".to_string(),
            entity_type: "products".to_string(),
            entity_id: String::new(),
            message: format!("Synced {} Square catalog items", items.len()),
            errors,
        }
    }

    async fn sync_orders(&self, pool: &PgPool, brand_id: &str) -> SyncResult {
        let (token, location_id) = match self.creds() {
            Ok(c) => c,
            Err(e) => {
                return SyncResult {
                    success: false,
                    platform: "square".to_string(),
                    entity_type: "orders".to_string(),
                    entity_id: String::new(),
                    message: "Square orders sync failed".to_string(),
                    errors: vec![e],
                }
            }
        };

        // Square Orders API requires a POST to /orders/search (no plain
        // list-all GET endpoint exists); follow `cursor` across every page.
        let orders = match self.fetch_all_orders(&token, &location_id).await {
            Ok(o) => o,
            Err(e) => {
                return SyncResult {
                    success: false,
                    platform: "square".to_string(),
                    entity_type: "orders".to_string(),
                    entity_id: String::new(),
                    message: "Failed to reach Square Orders API".to_string(),
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
            platform: "square".to_string(),
            entity_type: "orders".to_string(),
            entity_id: String::new(),
            message: format!("Synced {} Square orders", orders.len()),
            errors,
        }
    }

    async fn sync_inventory(&self, pool: &PgPool, brand_id: &str) -> SyncResult {
        let (token, location_id) = match self.creds() {
            Ok(c) => c,
            Err(e) => {
                return SyncResult {
                    success: false,
                    platform: "square".to_string(),
                    entity_type: "inventory".to_string(),
                    entity_id: String::new(),
                    message: "Square inventory sync failed".to_string(),
                    errors: vec![e],
                }
            }
        };

        // Pull every catalog_object_id we've already mapped for this
        // connection, then batch-retrieve their live counts from the
        // Inventory API.
        let mappings: Vec<(String, String)> = match sqlx::query_as(
            "SELECT product_id, external_id FROM external_product_mapping \
             WHERE platform = 'square' AND connection_id = $1",
        )
        .bind(self.connection_id())
        .fetch_all(pool)
        .await
        {
            Ok(rows) => rows,
            Err(e) => {
                return SyncResult {
                    success: false,
                    platform: "square".to_string(),
                    entity_type: "inventory".to_string(),
                    entity_id: String::new(),
                    message: "Failed to load Square product mappings".to_string(),
                    errors: vec![e.to_string()],
                }
            }
        };

        if mappings.is_empty() {
            return SyncResult {
                success: true,
                platform: "square".to_string(),
                entity_type: "inventory".to_string(),
                entity_id: String::new(),
                message: "No mapped Square products to refresh inventory for".to_string(),
                errors: vec![],
            };
        }

        let catalog_object_ids: Vec<String> = mappings.iter().map(|(_, ext)| ext.clone()).collect();
        let counts = match self
            .fetch_all_inventory_counts(&token, &location_id, &catalog_object_ids)
            .await
        {
            Ok(c) => c,
            Err(e) => {
                return SyncResult {
                    success: false,
                    platform: "square".to_string(),
                    entity_type: "inventory".to_string(),
                    entity_id: String::new(),
                    message: "Failed to reach Square Inventory API".to_string(),
                    errors: vec![e],
                }
            }
        };

        let mut by_object: std::collections::HashMap<String, i32> = std::collections::HashMap::new();
        for c in &counts {
            let obj_id = c.get("catalog_object_id").and_then(|v| v.as_str()).unwrap_or("");
            let qty: i32 = c
                .get("quantity")
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse().ok())
                .unwrap_or(0);
            by_object.insert(obj_id.to_string(), qty);
        }

        let mut synced = 0;
        let mut errors = Vec::new();
        for (product_id, external_id) in &mappings {
            let qty = by_object.get(external_id).copied().unwrap_or(0);
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
            platform: "square".to_string(),
            entity_type: "inventory".to_string(),
            entity_id: String::new(),
            message: format!("Refreshed inventory counts for {synced} Square products"),
            errors,
        }
    }
}
