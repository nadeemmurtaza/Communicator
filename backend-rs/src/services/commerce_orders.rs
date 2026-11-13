#![allow(dead_code)]

//! Order management service: create from cart, status transitions, order history.

use serde::{Deserialize, Serialize};
use sqlx::{Row, PgPool};
use uuid::Uuid;

use crate::models::{CommerceOrder, CommerceOrderItem, OrderStatus, PaymentStatus};

// ---------------------------------------------------------------------------
// Request/Response DTOs
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct CheckoutRequest {
    pub shipping_address: Option<serde_json::Value>,
    pub payment_method: Option<String>,
    pub tax: Option<f64>,
    pub shipping_cost: Option<f64>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateOrderStatusRequest {
    pub status: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderWithItems {
    pub order: CommerceOrder,
    pub items: Vec<CommerceOrderItem>,
}

// ---------------------------------------------------------------------------
// Service
// ---------------------------------------------------------------------------

pub struct CommerceOrdersService {
    pool: PgPool,
}

impl CommerceOrdersService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Generate a unique order number like ORD-2024-00001
    pub async fn generate_order_number(&self) -> Result<String, String> {
        let now = chrono::Utc::now();
        let prefix = format!("ORD-{}", now.format("%Y"));

        // Count orders this year
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM commerce_orders WHERE order_number LIKE $1",
        )
        .bind(format!("{}-%", prefix))
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Failed to generate order number: {e}"))?;

        Ok(format!("{}-{:05}", prefix, count + 1))
    }

    /// Create an order from a cart (checkout)
    pub async fn checkout(
        &self,
        cart_id: &str,
        req: &CheckoutRequest,
    ) -> Result<OrderWithItems, String> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| format!("Failed to begin transaction: {e}"))?;

        // Load cart
        let cart_row = sqlx::query(
            "SELECT id, customer_id, brand_id, channel_type, status FROM carts WHERE id = $1",
        )
        .bind(cart_id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| format!("Failed to load cart: {e}"))?
        .ok_or("Cart not found".to_string())?;

        let cart_status: String = cart_row.get("status");
        if cart_status != "active" {
            return Err(format!("Cannot checkout a {} cart", cart_status));
        }

        let customer_id: String = cart_row.get("customer_id");
        let brand_id: String = cart_row.get("brand_id");
        let channel_type: Option<String> = cart_row.get("channel_type");

        // Load cart items
        let items_rows = sqlx::query(
            "SELECT ci.id, ci.product_id, ci.variant_id, ci.quantity, ci.unit_price, \
             p.name AS product_name, p.sku, p.track_inventory, p.inventory_count \
             FROM cart_items ci \
             JOIN products p ON ci.product_id = p.id \
             WHERE ci.cart_id = $1",
        )
        .bind(cart_id)
        .fetch_all(&mut *tx)
        .await
        .map_err(|e| format!("Failed to load cart items: {e}"))?;

        if items_rows.is_empty() {
            return Err("Cannot checkout an empty cart".to_string());
        }

        // Check inventory
        for row in &items_rows {
            let track_inventory: bool = row.get("track_inventory");
            if track_inventory {
                let inventory: i32 = row.get("inventory_count");
                let qty: i32 = row.get("quantity");
                if qty > inventory {
                    let name: String = row.get("product_name");
                    return Err(format!(
                        "Insufficient inventory for '{}'. Available: {}, Requested: {}",
                        name, inventory, qty
                    ));
                }
            }
        }

        // Calculate totals
        let subtotal: f64 = items_rows
            .iter()
            .map(|row| {
                let qty: i32 = row.get("quantity");
                let price: f64 = row.get("unit_price");
                qty as f64 * price
            })
            .sum();
        let tax = req.tax.unwrap_or(0.0);
        let shipping_cost = req.shipping_cost.unwrap_or(0.0);
        let total_amount = subtotal + tax + shipping_cost;

        // Generate order number
        let order_number = self.generate_order_number().await?;
        let order_id = Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();

        // Create order
        let shipping_address = req
            .shipping_address
            .as_ref()
            .map(|v| serde_json::to_string(v).unwrap_or_default());
        let metadata = req
            .metadata
            .as_ref()
            .map(|v| serde_json::to_string(v).unwrap_or_default());
        // Convert rows to serializable format for snapshot
        let items_snapshot_vec: Vec<serde_json::Value> = items_rows
            .iter()
            .map(|row| {
                let mut map = serde_json::Map::new();
                map.insert("product_id".to_string(), serde_json::json!(row.get::<String, _>("product_id")));
                map.insert("variant_id".to_string(), serde_json::json!(row.get::<Option<String>, _>("variant_id")));
                map.insert("product_name".to_string(), serde_json::json!(row.get::<String, _>("product_name")));
                map.insert("sku".to_string(), serde_json::json!(row.get::<Option<String>, _>("sku")));
                map.insert("quantity".to_string(), serde_json::json!(row.get::<i32, _>("quantity")));
                map.insert("unit_price".to_string(), serde_json::json!(row.get::<f64, _>("unit_price")));
                serde_json::Value::Object(map)
            })
            .collect();
        let items_snapshot = serde_json::to_string(&items_snapshot_vec)
            .ok()
            .or(Some("[]".to_string()));

        sqlx::query(
            "INSERT INTO commerce_orders (id, brand_id, customer_id, order_number, channel_type, \
             status, total_amount, subtotal, tax, shipping, currency, items_snapshot, \
             shipping_address, payment_status, payment_method, metadata, source, created_at, updated_at) \
             VALUES ($1, $2, $3, $4, $5, 'pending', $6, $7, $8, $9, 'USD', $10, $11, 'pending', $12, $13, 'internal', $14, $15)",
        )
        .bind(&order_id)
        .bind(&brand_id)
        .bind(&customer_id)
        .bind(&order_number)
        .bind(&channel_type)
        .bind(total_amount)
        .bind(subtotal)
        .bind(tax)
        .bind(shipping_cost)
        .bind(&items_snapshot)
        .bind(&shipping_address)
        .bind(&req.payment_method)
        .bind(&metadata)
        .bind(&now)
        .bind(&now)
        .execute(&mut *tx)
        .await
        .map_err(|e| format!("Failed to create order: {e}"))?;

        // Create order items and reduce inventory
        let mut order_items = Vec::new();
        for row in &items_rows {
            let item_id = Uuid::new_v4().to_string();
            let product_id: String = row.get("product_id");
            let variant_id: Option<String> = row.get("variant_id");
            let product_name: String = row.get("product_name");
            let sku: Option<String> = row.get("sku");
            let quantity: i32 = row.get("quantity");
            let unit_price: f64 = row.get("unit_price");
            let total_price = quantity as f64 * unit_price;
            let track_inventory: bool = row.get("track_inventory");

            sqlx::query(
                "INSERT INTO commerce_order_items (id, order_id, product_id, variant_id, \
                 product_name, sku, quantity, unit_price, total_price) \
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
            )
            .bind(&item_id)
            .bind(&order_id)
            .bind(&product_id)
            .bind(&variant_id)
            .bind(&product_name)
            .bind(&sku)
            .bind(quantity)
            .bind(unit_price)
            .bind(total_price)
            .execute(&mut *tx)
            .await
            .map_err(|e| format!("Failed to create order item: {e}"))?;

            // Reduce inventory
            if track_inventory {
                let _ = sqlx::query(
                    "UPDATE products SET inventory_count = inventory_count - $1 WHERE id = $2",
                )
                .bind(quantity)
                .bind(&product_id)
                .execute(&mut *tx)
                .await;
            }

            order_items.push(CommerceOrderItem {
                id: item_id,
                order_id: order_id.clone(),
                product_id,
                variant_id,
                product_name,
                sku,
                quantity,
                unit_price,
                total_price,
                metadata: None,
            });
        }

        // Mark cart as converted
        sqlx::query("UPDATE carts SET status = 'converted', updated_at = $1 WHERE id = $2")
            .bind(&now)
            .bind(cart_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| format!("Failed to update cart status: {e}"))?;

        tx.commit()
            .await
            .map_err(|e| format!("Failed to commit transaction: {e}"))?;

        // Load the created order
        let order = self.get_order(&order_id).await?;
        Ok(OrderWithItems {
            order,
            items: order_items,
        })
    }

    /// List orders with filtering
    pub async fn list_orders(
        &self,
        brand_id: &str,
        _customer_id: Option<&str>,
        _status: Option<&str>,
        _channel_type: Option<&str>,
    ) -> Result<Vec<CommerceOrder>, String> {
        let rows = sqlx::query(
            "SELECT id, brand_id, customer_id, order_number, channel_type, status, \
             total_amount, subtotal, tax, shipping, currency, items_snapshot, \
             shipping_address, payment_status, payment_method, metadata, source, \
             external_order_id, created_at, updated_at \
             FROM commerce_orders WHERE brand_id = $1 ORDER BY created_at DESC",
        )
        .bind(brand_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to list orders: {e}"))?;

        rows.iter()
            .map(|row| Ok(CommerceOrder {
                id: row.get("id"),
                brand_id: row.get("brand_id"),
                customer_id: row.get("customer_id"),
                order_number: row.get("order_number"),
                channel_type: row.get("channel_type"),
                status: row.get("status"),
                total_amount: row.get("total_amount"),
                subtotal: row.get("subtotal"),
                tax: row.get("tax"),
                shipping: row.get("shipping"),
                currency: row.get("currency"),
                items_snapshot: row.get("items_snapshot"),
                shipping_address: row.get("shipping_address"),
                payment_status: row.get("payment_status"),
                payment_method: row.get("payment_method"),
                metadata: row.get("metadata"),
                source: row.get("source"),
                external_order_id: row.get("external_order_id"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            }))
            .collect()
    }

    /// Get a single order with items
    pub async fn get_order_with_items(&self, order_id: &str) -> Result<OrderWithItems, String> {
        let order = self.get_order(order_id).await?;
        let items = self.get_order_items(order_id).await?;
        Ok(OrderWithItems { order, items })
    }

    /// Get a single order
    pub async fn get_order(&self, order_id: &str) -> Result<CommerceOrder, String> {
        let row = sqlx::query(
            "SELECT id, brand_id, customer_id, order_number, channel_type, status, \
             total_amount, subtotal, tax, shipping, currency, items_snapshot, \
             shipping_address, payment_status, payment_method, metadata, source, \
             external_order_id, created_at, updated_at \
             FROM commerce_orders WHERE id = $1",
        )
        .bind(order_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Order not found: {e}"))?;

        Ok(CommerceOrder {
            id: row.get("id"),
            brand_id: row.get("brand_id"),
            customer_id: row.get("customer_id"),
            order_number: row.get("order_number"),
            channel_type: row.get("channel_type"),
            status: row.get("status"),
            total_amount: row.get("total_amount"),
            subtotal: row.get("subtotal"),
            tax: row.get("tax"),
            shipping: row.get("shipping"),
            currency: row.get("currency"),
            items_snapshot: row.get("items_snapshot"),
            shipping_address: row.get("shipping_address"),
            payment_status: row.get("payment_status"),
            payment_method: row.get("payment_method"),
            metadata: row.get("metadata"),
            source: row.get("source"),
            external_order_id: row.get("external_order_id"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        })
    }

    /// Get order items
    pub async fn get_order_items(&self, order_id: &str) -> Result<Vec<CommerceOrderItem>, String> {
        let rows = sqlx::query(
            "SELECT id, order_id, product_id, variant_id, product_name, sku, \
             quantity, unit_price, total_price, metadata \
             FROM commerce_order_items WHERE order_id = $1",
        )
        .bind(order_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to load order items: {e}"))?;

        rows.iter()
            .map(|row| Ok(CommerceOrderItem {
                id: row.get("id"),
                order_id: row.get("order_id"),
                product_id: row.get("product_id"),
                variant_id: row.get("variant_id"),
                product_name: row.get("product_name"),
                sku: row.get("sku"),
                quantity: row.get("quantity"),
                unit_price: row.get("unit_price"),
                total_price: row.get("total_price"),
                metadata: row.get("metadata"),
            }))
            .collect()
    }

    /// Update order status with validation
    pub async fn update_order_status(
        &self,
        order_id: &str,
        req: &UpdateOrderStatusRequest,
    ) -> Result<CommerceOrder, String> {
        let new_status = OrderStatus::from_str(&req.status);

        // Validate transition
        let current = self.get_order(order_id).await?;
        let current_status = OrderStatus::from_str(&current.status);

        if !is_valid_transition(&current_status, &new_status) {
            return Err(format!(
                "Cannot transition order from '{}' to '{}'",
                current.status, new_status.as_str()
            ));
        }

        let now = chrono::Utc::now().to_rfc3339();
        sqlx::query("UPDATE commerce_orders SET status = $1, updated_at = $2 WHERE id = $3")
            .bind(new_status.as_str())
            .bind(&now)
            .bind(order_id)
            .execute(&self.pool)
            .await
            .map_err(|e| format!("Failed to update order status: {e}"))?;

        self.get_order(order_id).await
    }

    /// Get order statistics for a brand
    pub async fn get_order_stats(&self, brand_id: &str) -> Result<serde_json::Value, String> {
        let total_orders: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM commerce_orders WHERE brand_id = $1",
        )
        .bind(brand_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Failed to count orders: {e}"))?;

        let total_revenue: f64 = sqlx::query_scalar(
            "SELECT COALESCE(SUM(total_amount), 0) FROM commerce_orders WHERE brand_id = $1",
        )
        .bind(brand_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Failed to sum revenue: {e}"))?;

        let pending_orders: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM commerce_orders WHERE brand_id = $1 AND status = 'pending'",
        )
        .bind(brand_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Failed to count pending: {e}"))?;

        let today_start = chrono::Utc::now().format("%Y-%m-%dT00:00:00Z").to_string();
        let today_orders: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM commerce_orders WHERE brand_id = $1 AND created_at >= $2",
        )
        .bind(brand_id)
        .bind(&today_start)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Failed to count today's orders: {e}"))?;

        Ok(serde_json::json!({
            "totalOrders": total_orders,
            "totalRevenue": total_revenue,
            "pendingOrders": pending_orders,
            "todayOrders": today_orders,
        }))
    }
}

/// Validate order status transitions
fn is_valid_transition(from: &OrderStatus, to: &OrderStatus) -> bool {
    match (from, to) {
        (OrderStatus::Draft, OrderStatus::Pending | OrderStatus::Cancelled) => true,
        (OrderStatus::Pending, OrderStatus::Confirmed | OrderStatus::Cancelled) => true,
        (OrderStatus::Confirmed, OrderStatus::Processing | OrderStatus::Cancelled) => true,
        (OrderStatus::Processing, OrderStatus::Shipped | OrderStatus::Cancelled) => true,
        (OrderStatus::Shipped, OrderStatus::Delivered) => true,
        (OrderStatus::Delivered, OrderStatus::Cancelled) => true,
        _ => false,
    }
}
