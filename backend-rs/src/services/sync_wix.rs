#![allow(dead_code)]

//! Wix sync adapter — full bidirectional sync implementation.
//!
//! Handles products, orders, and inventory sync between our system and Wix.
//! Uses Wix Headless Commerce API (Coral API).

use serde::{Deserialize, Serialize};
use sqlx::{Row, PgPool};
use uuid::Uuid;

use crate::models::SyncConnection;
use crate::services::sync_adapter::{SyncAdapter, SyncResult};

pub struct WixSyncAdapter {
    pool: PgPool,
    http_client: reqwest::Client,
    connection: SyncConnection,
}

impl WixSyncAdapter {
    pub fn new(pool: PgPool, http_client: reqwest::Client, connection: SyncConnection) -> Self {
        Self { pool, http_client, connection }
    }

    fn api_key(&self) -> Option<String> {
        let creds: serde_json::Value =
            serde_json::from_str(self.connection.credentials.as_ref()?).ok()?;
        creds.get("api_key").and_then(|v| v.as_str()).map(|s| s.to_string())
    }

    fn connection_id(&self) -> &str {
        &self.connection.id
    }

    // -----------------------------------------------------------------------
    // Helper: Wix GraphQL API
    // -----------------------------------------------------------------------

    async fn wix_graphql(&self, query: &str, variables: &serde_json::Value) -> Result<serde_json::Value, String> {
        let api_key = self.api_key()?;
        let resp = self
            .http_client
            .post("https://www.wix.com/_api/_graphql/ecommerce")
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", api_key))
            .json(&serde_json::json!({
                "query": query,
                "variables": variables
            }))
            .send()
            .await
            .map_err(|e| format!("Wix GraphQL failed: {}", e))?;
        if !resp.status().is_success() {
            return Err(format!("Wix GraphQL returned {}", resp.status()));
        }
        let body: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
        if let Some(errors) = body.get("errors") {
            return Err(format!("Wix GraphQL error: {}", errors));
        }
        Ok(body)
    }

    // -----------------------------------------------------------------------
    // Helper: manage external product mapping
    // -----------------------------------------------------------------------

    async fn get_product_mapping(&self, wix_product_id: &str) -> Result<Option<String>, String> {
        let row = sqlx::query(
            "SELECT product_id FROM external_product_mapping \
             WHERE external_id = $1 AND platform = 'wix' AND connection_id = $2",
        )
        .bind(wix_product_id)
        .bind(self.connection_id())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        Ok(row.and_then(|r| r.try_get::<Option<String>, _>("product_id").ok()).flatten())
    }

    async fn upsert_product_mapping(
        &self,
        local_product_id: &str,
        wix_product_id: &str,
        sku: Option<&str>,
    ) -> Result<(), String> {
        let now = chrono::Utc::now().to_rfc3339();
        let id = Uuid::new_v4().to_string();
        let exists = sqlx::query(
            "SELECT COUNT(*) as cnt FROM external_product_mapping \
             WHERE product_id = $1 AND platform = 'wix' AND connection_id = $2",
        )
        .bind(local_product_id)
        .bind(self.connection_id())
        .fetch_one(&self.pool)
        .await
        .map_err(|e| e.to_string())?
        .get::<i64, _>("cnt");

        if exists > 0 {
            let _ = sqlx::query(
                "UPDATE external_product_mapping SET external_id = $1, external_sku = $2, \
                 last_synced_at = $3, updated_at = $4 \
                 WHERE product_id = $5 AND platform = 'wix' AND connection_id = $6",
            )
            .bind(wix_product_id)
            .bind(sku.map(String::from))
            .bind(&now)
            .bind(&now)
            .bind(local_product_id)
            .bind(self.connection_id())
            .execute(&self.pool)
            .await;
        } else {
            let _ = sqlx::query(
                "INSERT INTO external_product_mapping \
                 (id, product_id, platform, connection_id, external_id, external_sku, \
                  last_synced_at, created_at, updated_at) \
                 VALUES ($1, $2, 'wix', $3, $4, $5, $6, $7, $8)",
            )
            .bind(&id)
            .bind(local_product_id)
            .bind(self.connection_id())
            .bind(wix_product_id)
            .bind(sku.map(String::from))
            .bind(&now)
            .bind(&now)
            .bind(&now)
            .execute(&self.pool)
            .await;
        }
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Helper: manage external order mapping
    // -----------------------------------------------------------------------

    async fn get_order_mapping(&self, wix_order_id: &str) -> Result<Option<String>, String> {
        let row = sqlx::query(
            "SELECT order_id FROM external_order_mapping \
             WHERE external_id = $1 AND platform = 'wix' AND connection_id = $2",
        )
        .bind(wix_order_id)
        .bind(self.connection_id())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        Ok(row.and_then(|r| r.try_get::<Option<String>, _>("order_id").ok()).flatten())
    }

    async fn upsert_order_mapping(
        &self,
        local_order_id: &str,
        wix_order_id: &str,
        order_number: Option<&str>,
    ) -> Result<(), String> {
        let now = chrono::Utc::now().to_rfc3339();
        let id = Uuid::new_v4().to_string();
        let exists = sqlx::query(
            "SELECT COUNT(*) as cnt FROM external_order_mapping \
             WHERE order_id = $1 AND platform = 'wix' AND connection_id = $2",
        )
        .bind(local_order_id)
        .bind(self.connection_id())
        .fetch_one(&self.pool)
        .await
        .map_err(|e| e.to_string())?
        .get::<i64, _>("cnt");

        if exists > 0 {
            let _ = sqlx::query(
                "UPDATE external_order_mapping SET external_id = $1, external_order_number = $2, \
                 last_synced_at = $3, updated_at = $4 \
                 WHERE order_id = $5 AND platform = 'wix' AND connection_id = $6",
            )
            .bind(wix_order_id)
            .bind(order_number.map(String::from))
            .bind(&now)
            .bind(&now)
            .bind(local_order_id)
            .bind(self.connection_id())
            .execute(&self.pool)
            .await;
        } else {
            let _ = sqlx::query(
                "INSERT INTO external_order_mapping \
                 (id, order_id, platform, connection_id, external_id, external_order_number, \
                  last_synced_at, created_at, updated_at) \
                 VALUES ($1, $2, 'wix', $3, $4, $5, $6, $7, $8)",
            )
            .bind(&id)
            .bind(local_order_id)
            .bind(self.connection_id())
            .bind(wix_order_id)
            .bind(order_number.map(String::from))
            .bind(&now)
            .bind(&now)
            .bind(&now)
            .execute(&self.pool)
            .await;
        }
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Products OUTBOUND: Our system → Wix
    // -----------------------------------------------------------------------

    async fn sync_products_outbound_impl(&self) -> Result<SyncResult, String> {
        let mut synced = 0;
        let mut errors = Vec::new();

        let products = sqlx::query(
            "SELECT id, name, description, sku, price, compare_at_price, currency, \
             images, variants, inventory_count, is_active \
             FROM products WHERE is_active = true",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        for row in products {
            let product_id: String = row.get("id");
            let name: String = row.get("name");
            let description: Option<String> = row.get("description");
            let sku: Option<String> = row.get("sku");
            let price: f64 = row.get("price");
            let _compare_at_price: Option<f64> = row.get("compare_at_price");
            let _currency: String = row.get("currency");
            let _images_json: Option<String> = row.get("images");
            let _variants_json: Option<String> = row.get("variants");
            let inventory: i32 = row.get("inventory_count");

            let wix_id = self.get_product_mapping(&product_id).await.unwrap_or(None);

            let product_input = serde_json::json!({
                "name": name,
                "description": description.unwrap_or_default(),
                "productType": "PHYSICAL",
                "price": price.to_string(),
                "sku": sku.clone().unwrap_or_else(|| product_id.clone()),
                "inventory": {
                    "quantity": inventory,
                    "trackInventory": true
                },
                "status": "ACTIVE"
            });

            let query_str = if let Some(ref wid) = wix_id {
                "mutation updateProduct($input: ProductInput, $id: ID!) { products.updateProduct(productInput: $input, id: $id) { _id } }".to_string()
            } else {
                "mutation createProduct($input: ProductInput) { products.createProduct(productInput: $input) { _id } }".to_string()
            };

            let variables = if let Some(ref wid) = wix_id {
                serde_json::json!({ "input": product_input, "id": wid })
            } else {
                serde_json::json!({ "input": product_input })
            };

            match self.wix_graphql(&query_str, &variables).await {
                Ok(resp) => {
                    let result = if wix_id.is_some() {
                        resp.get("data").and_then(|d| d.get("products")).and_then(|p| p.get("updateProduct")).and_then(|r| r.get("_id").as_str())
                    } else {
                        resp.get("data").and_then(|d| d.get("products")).and_then(|p| p.get("createProduct")).and_then(|r| r.get("_id").as_str())
                    };

                    if let Some(new_id) = result {
                        synced += 1;
                        self.upsert_product_mapping(&product_id, new_id, sku.as_deref()).await.ok();
                    } else {
                        errors.push(format!("Product {}: no ID in response", product_id));
                    }
                }
                Err(e) => errors.push(format!("Product {}: {}", product_id, e)),
            }
        }

        Ok(SyncResult {
            success: errors.is_empty(),
            direction: "outbound".to_string(),
            entity_type: "products".to_string(),
            synced_count: synced,
            errors,
        })
    }

    // -----------------------------------------------------------------------
    // Products INBOUND: Wix → Our system
    // -----------------------------------------------------------------------

    async fn sync_products_inbound_impl(&self) -> Result<SyncResult, String> {
        let mut synced = 0;
        let mut errors = Vec::new();

        let query_str = "query listProducts($limit: Int) { products.listProducts(limit: $limit) { products { _id name description productOptions { name choices { name } } variants { name sku price { amount } inventory { quantity } } price { amount } inventory { quantity trackInventory } status } } }";

        let variables = serde_json::json!({ "limit": 100 });

        match self.wix_graphql(query_str, &variables).await {
            Ok(resp) => {
                let wix_products = resp.get("data")
                    .and_then(|d| d.get("products"))
                    .and_then(|p| p.get("listProducts"))
                    .and_then(|lp| lp.get("products"))
                    .and_then(|arr| arr.as_array())
                    .unwrap_or(&vec![]);

                for wp in wix_products {
                    let wix_id = match wp.get("_id").and_then(|v| v.as_str()) {
                        Some(id) => id.to_string(),
                        None => continue,
                    };
                    let name = wp.get("name").and_then(|v| v.as_str()).unwrap_or("");
                    let description = wp.get("description")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string());
                    let price = wp.get("price")
                        .and_then(|p| p.get("amount"))
                        .and_then(|a| a.as_str())
                        .and_then(|s| s.parse::<f64>().ok())
                        .unwrap_or(0.0);
                    let inventory = wp.get("inventory")
                        .and_then(|inv| inv.get("quantity"))
                        .and_then(|q| q.as_i64())
                        .unwrap_or(0) as i32;

                    let existing_local_id = self.get_product_mapping(&wix_id).await.ok().flatten();

                    if let Some(local_id) = existing_local_id {
                        let _ = sqlx::query(
                            "UPDATE products SET inventory_count = $1, updated_at = NOW()::TEXT WHERE id = $2",
                        )
                        .bind(inventory)
                        .bind(&local_id)
                        .execute(&self.pool)
                        .await;
                        synced += 1;
                    } else {
                        let product_id = Uuid::new_v4().to_string();
                        let now = chrono::Utc::now().to_rfc3339();
                        let brand_id = &self.connection.brand_id;

                        let _ = sqlx::query(
                            "INSERT INTO products (id, brand_id, name, description, sku, price, \
                             currency, inventory_count, is_active, created_at, updated_at) \
                             VALUES ($1, $2, $3, $4, $5, $6, 'USD', $7, true, $8, $9)",
                        )
                        .bind(&product_id)
                        .bind(brand_id)
                        .bind(name)
                        .bind(&description)
                        .bind::<Option<&str>>(None)
                        .bind(price)
                        .bind(inventory)
                        .bind(&now)
                        .bind(&now)
                        .execute(&self.pool)
                        .await;

                        self.upsert_product_mapping(&product_id, &wix_id, None).await.ok();
                        synced += 1;
                    }
                }
            }
            Err(e) => errors.push(e),
        }

        Ok(SyncResult {
            success: errors.is_empty(),
            direction: "inbound".to_string(),
            entity_type: "products".to_string(),
            synced_count: synced,
            errors,
        })
    }

    // -----------------------------------------------------------------------
    // Orders OUTBOUND: Our system → Wix
    // -----------------------------------------------------------------------

    async fn sync_orders_outbound_impl(&self) -> Result<SyncResult, String> {
        let mut synced = 0;
        let mut errors = Vec::new();

        let orders = sqlx::query(
            "SELECT o.id, o.order_number, o.customer_id, o.total_amount, o.subtotal, \
             o.tax, o.shipping, o.currency, o.shipping_address, o.status, o.payment_status, \
             o.created_at \
             FROM commerce_orders o \
             LEFT JOIN external_order_mapping eom \
             ON o.id = eom.order_id AND eom.platform = 'wix' AND eom.connection_id = $1 \
             WHERE eom.id IS NULL \
             AND o.source != 'wix' \
             ORDER BY o.created_at DESC LIMIT 100",
        )
        .bind(self.connection_id())
        .fetch_all(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        for row in orders {
            let order_id: String = row.get("id");
            let order_number: String = row.get("order_number");
            let _customer_id: Option<String> = row.get("customer_id");
            let _total_amount: f64 = row.get("total_amount");
            let _subtotal: f64 = row.get("subtotal");
            let _tax: f64 = row.get("tax");
            let _shipping: f64 = row.get("shipping");
            let _currency: String = row.get("currency");
            let _shipping_address_json: Option<String> = row.get("shipping_address");
            let _status: String = row.get("status");
            let _payment_status: String = row.get("payment_status");

            // Wix orders are created via checkout, not directly via API
            errors.push(format!("Order {}: Wix does not support direct order creation via API — use checkout flow", order_number));
        }

        Ok(SyncResult {
            success: errors.is_empty(),
            direction: "outbound".to_string(),
            entity_type: "orders".to_string(),
            synced_count: synced,
            errors,
        })
    }

    // -----------------------------------------------------------------------
    // Orders INBOUND: Wix → Our system
    // -----------------------------------------------------------------------

    async fn sync_orders_inbound_impl(&self) -> Result<SyncResult, String> {
        let mut synced = 0;
        let mut errors = Vec::new();

        let since = self.connection.last_sync_at.as_deref().unwrap_or("");

        let query_str = if !since.is_empty() {
            format!(
                "query listOrders($limit: Int, $filter: OrdersFilterInput) {{ orders.listOrders(limit: ${{limit}}, filter: $filter) {{ orders {{ _id orderNumber status paymentStatus total {{ grossAmount }} subtotal {{ amount }} shipping {{ total {{ amount }} }} tax {{ total {{ amount }} }} currency buyerInfo {{ contactEmail firstName lastName }} lineItems {{ catalogReference {{ appId catalogId }} productName quantity unitPrice {{ amount }} totalPrice {{ amount }} }} shippingAddress {{ lines city country state postalCode }} }} }} }}",
            )
        } else {
            "query listOrders($limit: Int) { orders.listOrders(limit: $limit) { orders { _id orderNumber status paymentStatus total { grossAmount } subtotal { amount } shipping { total { amount } } tax { total { amount } } currency buyerInfo { contactEmail firstName lastName } lineItems { catalogReference { appId catalogId } productName quantity unitPrice { amount } totalPrice { amount } } shippingAddress { lines city country state postalCode } } } } }".to_string()
        };

        let variables = if !since.is_empty() {
            serde_json::json!({ "limit": 50, "filter": { "dateRange": { "from": since } } })
        } else {
            serde_json::json!({ "limit": 50 })
        };

        match self.wix_graphql(&query_str, &variables).await {
            Ok(resp) => {
                let wix_orders = resp.get("data")
                    .and_then(|d| d.get("orders"))
                    .and_then(|o| o.get("listOrders"))
                    .and_then(|lo| lo.get("orders"))
                    .and_then(|arr| arr.as_array())
                    .unwrap_or(&vec![]);

                for wo in wix_orders {
                    let wix_order_id = match wo.get("_id").and_then(|v| v.as_str()) {
                        Some(id) => id.to_string(),
                        None => continue,
                    };
                    let order_number = wo.get("orderNumber")
                        .and_then(|v| v.as_i64())
                        .map(|n| n.to_string());

                    let existing = self.get_order_mapping(&wix_order_id).await.ok().flatten();
                    if existing.is_some() {
                        continue;
                    }

                    let total = wo.get("total")
                        .and_then(|t| t.get("grossAmount"))
                        .and_then(|a| a.as_str())
                        .and_then(|s| s.parse::<f64>().ok())
                        .unwrap_or(0.0);
                    let subtotal = wo.get("subtotal")
                        .and_then(|s| s.get("amount"))
                        .and_then(|a| a.as_str())
                        .and_then(|s| s.parse::<f64>().ok())
                        .unwrap_or(0.0);
                    let tax = wo.get("tax")
                        .and_then(|t| t.get("total"))
                        .and_then(|t| t.get("amount"))
                        .and_then(|a| a.as_str())
                        .and_then(|s| s.parse::<f64>().ok())
                        .unwrap_or(0.0);
                    let shipping = wo.get("shipping")
                        .and_then(|s| s.get("total"))
                        .and_then(|t| t.get("amount"))
                        .and_then(|a| a.as_str())
                        .and_then(|s| s.parse::<f64>().ok())
                        .unwrap_or(0.0);
                    let currency = wo.get("currency").and_then(|v| v.as_str()).unwrap_or("USD");

                    let wix_status = wo.get("status").and_then(|v| v.as_str()).unwrap_or("PENDING_PAYMENT");
                    let status = match wix_status {
                        "FULFILLED" => "delivered",
                        "PARTIALLY_FULFILLED" => "shipped",
                        "PAID" => "confirmed",
                        "PENDING_FULFILLMENT" => "confirmed",
                        _ => "pending",
                    };

                    let payment_status = match wo.get("paymentStatus").and_then(|v| v.as_str()).unwrap_or("") {
                        "PAID" => "paid",
                        "REFUNDED" => "refunded",
                        _ => "pending",
                    };

                    let buyer = wo.get("buyerInfo").unwrap_or(&serde_json::json!({}));
                    let customer_email = buyer.get("contactEmail").and_then(|v| v.as_str()).unwrap_or("");

                    let customer_id = if !customer_email.is_empty() {
                        let existing_customer = sqlx::query(
                            "SELECT id FROM customers WHERE email = $1 OR phone = $2 LIMIT 1",
                        )
                        .bind(customer_email)
                        .bind(customer_email)
                        .fetch_optional(&self.pool)
                        .await
                        .ok()
                        .and_then(|r| r.try_get::<Option<String>, _>("id").ok()).flatten();

                        if let Some(cid) = existing_customer {
                            Some(cid)
                        } else {
                            let cid = Uuid::new_v4().to_string();
                            let now = chrono::Utc::now().to_rfc3339();
                            let first_name = buyer.get("firstName").and_then(|n| n.as_str()).unwrap_or("");
                            let last_name = buyer.get("lastName").and_then(|n| n.as_str()).unwrap_or("");
                            let name = format!("{} {}", first_name, last_name).trim().to_string();

                            let _ = sqlx::query(
                                "INSERT INTO customers (id, brand_id, name, email, phone, created_at) \
                                 VALUES ($1, $2, $3, $4, $5, $6)",
                            )
                            .bind(&cid)
                            .bind(&self.connection.brand_id)
                            .bind(&name)
                            .bind(customer_email)
                            .bind(customer_email)
                            .bind(&now)
                            .execute(&self.pool)
                            .await;
                            Some(cid)
                        }
                    } else {
                        None
                    };

                    let cid = customer_id.as_deref().unwrap_or("");

                    let shipping_addr = wo.get("shippingAddress").map(|addr| {
                        serde_json::json!({
                            "firstName": buyer.get("firstName").and_then(|v| v.as_str()).unwrap_or(""),
                            "lastName": buyer.get("lastName").and_then(|v| v.as_str()).unwrap_or(""),
                            "address1": addr.get("lines").and_then(|l| l.as_array()).and_then(|l| l.first()).and_then(|l| l.as_str()).unwrap_or(""),
                            "city": addr.get("city").and_then(|v| v.as_str()).unwrap_or(""),
                            "state": addr.get("state").and_then(|v| v.as_str()),
                            "country": addr.get("country").and_then(|v| v.as_str()).unwrap_or("US"),
                            "zip": addr.get("postalCode").and_then(|v| v.as_str()).unwrap_or(""),
                        })
                    });

                    let order_id = Uuid::new_v4().to_string();
                    let now = chrono::Utc::now().to_rfc3339();
                    let on = order_number.as_deref().unwrap_or(&format!("WIX-{}", wix_order_id));

                    let _ = sqlx::query(
                        "INSERT INTO commerce_orders \
                         (id, brand_id, customer_id, order_number, channel_type, status, \
                          total_amount, subtotal, tax, shipping, currency, items_snapshot, \
                          shipping_address, payment_status, payment_method, metadata, source, \
                          external_order_id, created_at, updated_at) \
                         VALUES ($1, $2, $3, $4, 'wix', $5, $6, $7, $8, $9, $10, '[]', $11, $12, $13, NULL, 'wix', $14, $15, $16)",
                    )
                    .bind(&order_id)
                    .bind(&self.connection.brand_id)
                    .bind(cid)
                    .bind(on)
                    .bind(status)
                    .bind(total)
                    .bind(subtotal)
                    .bind(tax)
                    .bind(shipping)
                    .bind(currency)
                    .bind(shipping_addr.as_ref().map(|v| v.to_string()).unwrap_or_default())
                    .bind(payment_status)
                    .bind("wix_payments")
                    .bind(&wix_order_id)
                    .bind(&now)
                    .bind(&now)
                    .execute(&self.pool)
                    .await;

                    let line_items = wo.get("lineItems").and_then(|v| v.as_array()).unwrap_or(&vec![]);
                    for item in line_items {
                        let item_id = Uuid::new_v4().to_string();
                        let product_name = item.get("productName").and_then(|v| v.as_str()).unwrap_or("");
                        let quantity = item.get("quantity").and_then(|v| v.as_i64()).unwrap_or(1) as i32;
                        let unit_price = item.get("unitPrice")
                            .and_then(|p| p.get("amount"))
                            .and_then(|a| a.as_str())
                            .and_then(|s| s.parse::<f64>().ok())
                            .unwrap_or(0.0);
                        let total_price = unit_price * quantity as f64;

                        let _ = sqlx::query(
                            "INSERT INTO commerce_order_items \
                             (id, order_id, product_id, variant_id, product_name, sku, \
                              quantity, unit_price, total_price) \
                             VALUES ($1, $2, NULL, NULL, $3, '', $4, $5, $6)",
                        )
                        .bind(&item_id)
                        .bind(&order_id)
                        .bind(product_name)
                        .bind(quantity)
                        .bind(unit_price)
                        .bind(total_price)
                        .execute(&self.pool)
                        .await;
                    }

                    let payment_id = Uuid::new_v4().to_string();
                    let _ = sqlx::query(
                        "INSERT INTO commerce_payments \
                         (id, order_id, amount, currency, method, status, created_at) \
                         VALUES ($1, $2, $3, $4, 'wix_payments', $5, $6)",
                    )
                    .bind(&payment_id)
                    .bind(&order_id)
                    .bind(total)
                    .bind(currency)
                    .bind(payment_status)
                    .bind(&now)
                    .execute(&self.pool)
                    .await;

                    self.upsert_order_mapping(&order_id, &wix_order_id, on.as_str()).await.ok();
                    synced += 1;
                }
            }
            Err(e) => errors.push(e),
        }

        Ok(SyncResult {
            success: errors.is_empty(),
            direction: "inbound".to_string(),
            entity_type: "orders".to_string(),
            synced_count: synced,
            errors,
        })
    }

    // -----------------------------------------------------------------------
    // Inventory OUTBOUND: Our system → Wix
    // -----------------------------------------------------------------------

    async fn sync_inventory_outbound_impl(&self) -> Result<SyncResult, String> {
        let mut synced = 0;
        let mut errors = Vec::new();

        let mappings = sqlx::query(
            "SELECT epm.product_id, epm.external_id, p.inventory_count, p.price \
             FROM external_product_mapping epm \
             JOIN products p ON epm.product_id = p.id \
             WHERE epm.platform = 'wix' AND epm.connection_id = $1",
        )
        .bind(self.connection_id())
        .fetch_all(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        for row in mappings {
            let product_id: String = row.get("product_id");
            let external_id: String = row.get("external_id");
            let inventory: i32 = row.get("inventory_count");
            let price: f64 = row.get("price");

            let query_str = "mutation updateProduct($input: ProductInput, $id: ID!) { products.updateProduct(productInput: $input, id: $id) { _id } }";

            let input = serde_json::json!({
                "price": price.to_string(),
                "inventory": {
                    "quantity": inventory,
                    "trackInventory": true
                }
            });

            let variables = serde_json::json!({ "input": input, "id": external_id });

            match self.wix_graphql(query_str, &variables).await {
                Ok(_) => { synced += 1; }
                Err(e) => errors.push(format!("Inventory {}: {}", product_id, e)),
            }
        }

        Ok(SyncResult {
            success: errors.is_empty(),
            direction: "outbound".to_string(),
            entity_type: "inventory".to_string(),
            synced_count: synced,
            errors,
        })
    }
}

#[async_trait::async_trait]
impl SyncAdapter for WixSyncAdapter {
    fn platform_name(&self) -> &str {
        "wix"
    }

    async fn test_connection(&self) -> Result<(), String> {
        let query_str = "query { products.listProducts(limit: 1) { products { _id } } }";
        let variables = serde_json::json!({});
        let resp = self.wix_graphql(query_str, &variables).await?;
        if resp.get("data").is_none() {
            return Err("Wix API returned no data".to_string());
        }
        Ok(())
    }

    async fn sync_products_outbound(&self) -> Result<SyncResult, String> {
        self.sync_products_outbound_impl().await
    }

    async fn sync_products_inbound(&self) -> Result<SyncResult, String> {
        self.sync_products_inbound_impl().await
    }

    async fn sync_orders_outbound(&self) -> Result<SyncResult, String> {
        self.sync_orders_outbound_impl().await
    }

    async fn sync_orders_inbound(&self) -> Result<SyncResult, String> {
        self.sync_orders_inbound_impl().await
    }

    async fn sync_inventory_outbound(&self) -> Result<SyncResult, String> {
        self.sync_inventory_outbound_impl().await
    }
}
