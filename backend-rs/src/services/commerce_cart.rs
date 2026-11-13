#![allow(dead_code)]

//! Shopping cart service: cart creation, item management, quantity validation.

use serde::{Deserialize, Serialize};
use sqlx::{Row, PgPool};
use uuid::Uuid;

use crate::models::{Cart, CartItem};

// ---------------------------------------------------------------------------
// Request/Response DTOs
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct CreateCartRequest {
    pub customer_id: String,
    pub brand_id: String,
    pub channel_type: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AddCartItemRequest {
    pub product_id: String,
    pub variant_id: Option<String>,
    pub quantity: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateCartItemRequest {
    pub quantity: i32,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CartWithItems {
    pub cart: Cart,
    pub items: Vec<CartItemWithProduct>,
    pub total: f64,
    pub item_count: i32,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct CartItemWithProduct {
    pub id: String,
    pub cart_id: String,
    pub product_id: String,
    pub variant_id: Option<String>,
    pub quantity: i32,
    pub unit_price: f64,
    pub metadata: Option<String>,
    pub created_at: String,
    pub product_name: Option<String>,
    pub product_sku: Option<String>,
    pub product_images: Option<String>,
}

// ---------------------------------------------------------------------------
// Service
// ---------------------------------------------------------------------------

pub struct CommerceCartService {
    pool: PgPool,
}

impl CommerceCartService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    // ---------------------------------------------------------------------------
    // Cart CRUD
    // ---------------------------------------------------------------------------

    pub async fn get_or_create_cart(
        &self,
        customer_id: &str,
        brand_id: &str,
        channel_type: Option<&str>,
    ) -> Result<CartWithItems, String> {
        // Find active cart for this customer and brand
        let cart = sqlx::query(
            "SELECT id, customer_id, brand_id, channel_type, status, metadata, created_at, updated_at \
             FROM carts WHERE customer_id = ? AND brand_id = ? AND status = 'active' \
             ORDER BY updated_at DESC LIMIT 1",
        )
        .bind(customer_id)
        .bind(brand_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("Failed to find cart: {e}"))?
        .map(|row: sqlx::postgres::PgRow| -> Result<Cart, String> {
            Ok(Cart {
                id: row.get("id"),
                customer_id: row.get("customer_id"),
                brand_id: row.get("brand_id"),
                channel_type: row.get("channel_type"),
                status: row.get("status"),
                metadata: row.get("metadata"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            })
        })
        .transpose()?;

        let cart = match cart {
            Some(c) => c,
            None => {
                let id = Uuid::new_v4().to_string();
                let now = chrono::Utc::now().to_rfc3339();
                let channel = channel_type.map(|s| s.to_string());

                sqlx::query(
                    "INSERT INTO carts (id, customer_id, brand_id, channel_type, status, created_at, updated_at) \
                     VALUES (?, ?, ?, ?, 'active', ?, ?)",
                )
                .bind(&id)
                .bind(customer_id)
                .bind(brand_id)
                .bind(&channel)
                .bind(&now)
                .bind(&now)
                .execute(&self.pool)
                .await
                .map_err(|e| format!("Failed to create cart: {e}"))?;

                Cart {
                    id,
                    customer_id: customer_id.to_string(),
                    brand_id: brand_id.to_string(),
                    channel_type: channel,
                    status: "active".to_string(),
                    metadata: None,
                    created_at: now.clone(),
                    updated_at: now,
                }
            }
        };

        self.get_cart_with_items(&cart.id).await
    }

    pub async fn list_carts(
        &self,
        customer_id: &str,
        brand_id: &str,
        status: Option<&str>,
    ) -> Result<Vec<Cart>, String> {
        let query = if status.is_some() {
            "SELECT id, customer_id, brand_id, channel_type, status, metadata, created_at, updated_at \
             FROM carts WHERE customer_id = ? AND brand_id = ? AND status = ? ORDER BY updated_at DESC"
        } else {
            "SELECT id, customer_id, brand_id, channel_type, status, metadata, created_at, updated_at \
             FROM carts WHERE customer_id = ? AND brand_id = ? ORDER BY updated_at DESC"
        };

        let rows = if let Some(s) = status {
            sqlx::query(query).bind(customer_id).bind(brand_id).bind(s)
        } else {
            sqlx::query(query).bind(customer_id).bind(brand_id)
        }
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to list carts: {e}"))?;

        rows.iter()
            .map(|row| Ok(Cart {
                id: row.get("id"),
                customer_id: row.get("customer_id"),
                brand_id: row.get("brand_id"),
                channel_type: row.get("channel_type"),
                status: row.get("status"),
                metadata: row.get("metadata"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            }))
            .collect()
    }

    pub async fn get_cart(&self, cart_id: &str) -> Result<Cart, String> {
        let row = sqlx::query(
            "SELECT id, customer_id, brand_id, channel_type, status, metadata, created_at, updated_at \
             FROM carts WHERE id = ?",
        )
        .bind(cart_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Cart not found: {e}"))?;
        Ok(Cart {
            id: row.get("id"),
            customer_id: row.get("customer_id"),
            brand_id: row.get("brand_id"),
            channel_type: row.get("channel_type"),
            status: row.get("status"),
            metadata: row.get("metadata"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        })
    }

    pub async fn get_cart_with_items(&self, cart_id: &str) -> Result<CartWithItems, String> {
        let cart = self.get_cart(cart_id).await?;

        let rows = sqlx::query(
            "SELECT ci.id, ci.cart_id, ci.product_id, ci.variant_id, ci.quantity, ci.unit_price, \
             ci.metadata, ci.created_at, p.name AS product_name, p.sku AS product_sku, \
             p.images AS product_images \
             FROM cart_items ci \
             LEFT JOIN products p ON ci.product_id = p.id \
             WHERE ci.cart_id = ?",
        )
        .bind(cart_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to load cart items: {e}"))?;

        let items: Vec<CartItemWithProduct> = rows
            .iter()
            .map(|row| CartItemWithProduct {
                id: row.get("id"),
                cart_id: row.get("cart_id"),
                product_id: row.get("product_id"),
                variant_id: row.get("variant_id"),
                quantity: row.get("quantity"),
                unit_price: row.get("unit_price"),
                metadata: row.get("metadata"),
                created_at: row.get("created_at"),
                product_name: row.get("product_name"),
                product_sku: row.get("product_sku"),
                product_images: row.get("product_images"),
            })
            .collect();

        let total: f64 = items.iter().map(|i| i.quantity as f64 * i.unit_price).sum();
        let item_count: i32 = items.iter().map(|i| i.quantity).sum();

        Ok(CartWithItems {
            cart,
            items,
            total,
            item_count,
        })
    }

    pub async fn update_cart_status(&self, cart_id: &str, status: &str) -> Result<Cart, String> {
        let now = chrono::Utc::now().to_rfc3339();
        sqlx::query("UPDATE carts SET status = ?, updated_at = ? WHERE id = ?")
            .bind(status)
            .bind(&now)
            .bind(cart_id)
            .execute(&self.pool)
            .await
            .map_err(|e| format!("Failed to update cart status: {e}"))?;
        self.get_cart(cart_id).await
    }

    // ---------------------------------------------------------------------------
    // Cart Items
    // ---------------------------------------------------------------------------

    pub async fn add_item(
        &self,
        cart_id: &str,
        req: &AddCartItemRequest,
    ) -> Result<CartItemWithProduct, String> {
        // Validate cart exists and is active
        let cart = self.get_cart(cart_id).await?;
        if cart.status != "active" {
            return Err(format!("Cannot add items to a {} cart", cart.status));
        }

        // Get product price
        let product_row = sqlx::query(
            "SELECT price, inventory_count, track_inventory, name, sku, images \
             FROM products WHERE id = ?",
        )
        .bind(&req.product_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Product not found: {e}"))?;

        let unit_price: f64 = product_row.get("price");
        let inventory_count: i32 = product_row.get("inventory_count");
        let track_inventory: bool = product_row.get("track_inventory");
        let product_name: Option<String> = product_row.get("name");
        let product_sku: Option<String> = product_row.get("sku");
        let product_images: Option<String> = product_row.get("images");

        let quantity = req.quantity.unwrap_or(1);
        if quantity <= 0 {
            return Err("Quantity must be positive".to_string());
        }

        // Check inventory
        if track_inventory {
            // Check existing quantity in cart
            let existing = sqlx::query_scalar::<_, i32>(
                "SELECT COALESCE(SUM(quantity), 0) FROM cart_items WHERE cart_id = ? AND product_id = ? AND variant_id = ?",
            )
            .bind(cart_id)
            .bind(&req.product_id)
            .bind(&req.variant_id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| format!("Failed to check inventory: {e}"))?;

            if existing + quantity > inventory_count {
                return Err(format!(
                    "Insufficient inventory. Available: {}, Requested: {}",
                    inventory_count,
                    existing + quantity
                ));
            }
        }

        let now = chrono::Utc::now().to_rfc3339();

        // Check if item already exists in cart
        let existing = sqlx::query(
            "SELECT id, quantity FROM cart_items WHERE cart_id = ? AND product_id = ? AND variant_id = ?",
        )
        .bind(cart_id)
        .bind(&req.product_id)
        .bind(&req.variant_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("Failed to check cart items: {e}"))?;

        let item_id = if let Some(row) = existing {
            let item_id: String = row.get("id");
            let old_qty: i32 = row.get("quantity");
            let new_qty = old_qty + quantity;

            sqlx::query(
                "UPDATE cart_items SET quantity = ?, unit_price = ? WHERE id = ?",
            )
            .bind(new_qty)
            .bind(unit_price)
            .bind(&item_id)
            .execute(&self.pool)
            .await
            .map_err(|e| format!("Failed to update cart item: {e}"))?;

            item_id
        } else {
            let item_id = Uuid::new_v4().to_string();
            sqlx::query(
                "INSERT INTO cart_items (id, cart_id, product_id, variant_id, quantity, unit_price, created_at) \
                 VALUES (?, ?, ?, ?, ?, ?, ?)",
            )
            .bind(&item_id)
            .bind(cart_id)
            .bind(&req.product_id)
            .bind(&req.variant_id)
            .bind(quantity)
            .bind(unit_price)
            .bind(&now)
            .execute(&self.pool)
            .await
            .map_err(|e| format!("Failed to add cart item: {e}"))?;

            item_id
        };

        // Update cart timestamp
        sqlx::query("UPDATE carts SET updated_at = ? WHERE id = ?")
            .bind(&now)
            .bind(cart_id)
            .execute(&self.pool)
            .await
            .ok();

        Ok(CartItemWithProduct {
            id: item_id,
            cart_id: cart_id.to_string(),
            product_id: req.product_id.clone(),
            variant_id: req.variant_id.clone(),
            quantity,
            unit_price,
            metadata: None,
            created_at: now,
            product_name,
            product_sku,
            product_images,
        })
    }

    pub async fn update_item_quantity(
        &self,
        cart_id: &str,
        item_id: &str,
        req: &UpdateCartItemRequest,
    ) -> Result<CartItemWithProduct, String> {
        if req.quantity <= 0 {
            return Err("Quantity must be positive".to_string());
        }

        let now = chrono::Utc::now().to_rfc3339();
        let row = sqlx::query(
            "SELECT ci.product_id, ci.variant_id, ci.unit_price, p.inventory_count, p.track_inventory \
             FROM cart_items ci \
             JOIN products p ON ci.product_id = p.id \
             WHERE ci.id = ? AND ci.cart_id = ?",
        )
        .bind(item_id)
        .bind(cart_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Cart item not found: {e}"))?;

        let inventory_count: i32 = row.get("inventory_count");
        let track_inventory: bool = row.get("track_inventory");

        if track_inventory && req.quantity > inventory_count {
            return Err(format!(
                "Insufficient inventory. Available: {}",
                inventory_count
            ));
        }

        sqlx::query("UPDATE cart_items SET quantity = ?, unit_price = ? WHERE id = ?")
            .bind(req.quantity)
            .bind(row.get::<f64, _>("unit_price"))
            .bind(item_id)
            .execute(&self.pool)
            .await
            .map_err(|e| format!("Failed to update quantity: {e}"))?;

        // Update cart timestamp
        sqlx::query("UPDATE carts SET updated_at = ? WHERE id = ?")
            .bind(&now)
            .bind(cart_id)
            .execute(&self.pool)
            .await
            .ok();

        // Return updated item
        let row = sqlx::query(
            "SELECT ci.id, ci.cart_id, ci.product_id, ci.variant_id, ci.quantity, ci.unit_price, \
             ci.metadata, ci.created_at, p.name AS product_name, p.sku AS product_sku, \
             p.images AS product_images \
             FROM cart_items ci \
             LEFT JOIN products p ON ci.product_id = p.id \
             WHERE ci.id = ?",
        )
        .bind(item_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Failed to load updated item: {e}"))?;

        Ok(CartItemWithProduct {
            id: row.get("id"),
            cart_id: row.get("cart_id"),
            product_id: row.get("product_id"),
            variant_id: row.get("variant_id"),
            quantity: row.get("quantity"),
            unit_price: row.get("unit_price"),
            metadata: row.get("metadata"),
            created_at: row.get("created_at"),
            product_name: row.get("product_name"),
            product_sku: row.get("product_sku"),
            product_images: row.get("product_images"),
        })
    }

    pub async fn remove_item(&self, cart_id: &str, item_id: &str) -> Result<(), String> {
        let now = chrono::Utc::now().to_rfc3339();
        sqlx::query("DELETE FROM cart_items WHERE id = ? AND cart_id = ?")
            .bind(item_id)
            .bind(cart_id)
            .execute(&self.pool)
            .await
            .map_err(|e| format!("Failed to remove cart item: {e}"))?;

        // Update cart timestamp
        sqlx::query("UPDATE carts SET updated_at = ? WHERE id = ?")
            .bind(&now)
            .bind(cart_id)
            .execute(&self.pool)
            .await
            .ok();

        Ok(())
    }

    pub async fn clear_cart(&self, cart_id: &str) -> Result<(), String> {
        let now = chrono::Utc::now().to_rfc3339();
        sqlx::query("DELETE FROM cart_items WHERE cart_id = ?")
            .bind(cart_id)
            .execute(&self.pool)
            .await
            .map_err(|e| format!("Failed to clear cart: {e}"))?;

        sqlx::query("UPDATE carts SET updated_at = ? WHERE id = ?")
            .bind(&now)
            .bind(cart_id)
            .execute(&self.pool)
            .await
            .ok();

        Ok(())
    }

    pub async fn delete_cart(&self, cart_id: &str) -> Result<(), String> {
        sqlx::query("DELETE FROM carts WHERE id = ?")
            .bind(cart_id)
            .execute(&self.pool)
            .await
            .map_err(|e| format!("Failed to delete cart: {e}"))?;
        Ok(())
    }
}
