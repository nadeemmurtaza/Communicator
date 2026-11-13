#![allow(dead_code)]

//! Product catalog service: categories, products, variants, inventory, and channel visibility.

use serde::{Deserialize, Serialize};
use sqlx::{Row, PgPool};
use uuid::Uuid;

use crate::models::{
    Category, Product, ProductChannelVisibility, ProductVariant,
};

// ---------------------------------------------------------------------------
// Request/Response DTOs
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct CreateCategoryRequest {
    pub brand_id: String,
    pub name: String,
    pub parent_id: Option<String>,
    pub sort_order: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateCategoryRequest {
    pub name: Option<String>,
    pub parent_id: Option<String>,
    pub sort_order: Option<i32>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct CreateProductRequest {
    pub brand_id: String,
    pub category_id: Option<String>,
    pub name: String,
    pub description: Option<String>,
    pub sku: Option<String>,
    pub price: f64,
    pub compare_at_price: Option<f64>,
    pub currency: Option<String>,
    pub images: Option<serde_json::Value>,
    pub variants: Option<Vec<ProductVariant>>,
    pub inventory_count: Option<i32>,
    pub track_inventory: Option<bool>,
    pub visibility: Option<serde_json::Value>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateProductRequest {
    pub category_id: Option<String>,
    pub name: Option<String>,
    pub description: Option<Option<String>>,
    pub sku: Option<Option<String>>,
    pub price: Option<f64>,
    pub compare_at_price: Option<Option<f64>>,
    pub currency: Option<String>,
    pub images: Option<serde_json::Value>,
    pub variants: Option<Vec<ProductVariant>>,
    pub inventory_count: Option<i32>,
    pub track_inventory: Option<bool>,
    pub visibility: Option<serde_json::Value>,
    pub metadata: Option<serde_json::Value>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateInventoryRequest {
    pub inventory_count: i32,
}

#[derive(Debug, Deserialize)]
pub struct SetChannelVisibilityRequest {
    pub channel_type: String,
    pub is_visible: bool,
}

// ---------------------------------------------------------------------------
// Service
// ---------------------------------------------------------------------------

pub struct CommerceCatalogService {
    pool: PgPool,
}

impl CommerceCatalogService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    // ---------------------------------------------------------------------------
    // Categories
    // ---------------------------------------------------------------------------

    pub async fn list_categories(
        &self,
        brand_id: &str,
        active_only: bool,
    ) -> Result<Vec<Category>, String> {
        let query = if active_only {
            "SELECT id, brand_id, name, parent_id, sort_order, is_active, created_at, updated_at \
             FROM categories WHERE brand_id = $1 AND is_active = 1 ORDER BY sort_order, name"
        } else {
            "SELECT id, brand_id, name, parent_id, sort_order, is_active, created_at, updated_at \
             FROM categories WHERE brand_id = $1 ORDER BY sort_order, name"
        };
        let rows = sqlx::query(query)
            .bind(brand_id)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| format!("Failed to list categories: {e}"))?;
        rows.iter()
            .map(|row| Ok(Category {
                id: row.get("id"),
                brand_id: row.get("brand_id"),
                name: row.get("name"),
                parent_id: row.get("parent_id"),
                sort_order: row.get("sort_order"),
                is_active: row.get("is_active"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            }))
            .collect()
    }

    pub async fn get_category(&self, id: &str) -> Result<Category, String> {
        let row = sqlx::query(
            "SELECT id, brand_id, name, parent_id, sort_order, is_active, created_at, updated_at \
             FROM categories WHERE id = $1",
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Category not found: {e}"))?;
        Ok(Category {
            id: row.get("id"),
            brand_id: row.get("brand_id"),
            name: row.get("name"),
            parent_id: row.get("parent_id"),
            sort_order: row.get("sort_order"),
            is_active: row.get("is_active"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        })
    }

    pub async fn create_category(&self, req: &CreateCategoryRequest) -> Result<Category, String> {
        let id = Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        let sort_order = req.sort_order.unwrap_or(0);

        sqlx::query(
            "INSERT INTO categories (id, brand_id, name, parent_id, sort_order, is_active, created_at, updated_at) \
             VALUES ($1, $2, $3, $4, $5, 1, $6, $7)",
        )
        .bind(&id)
        .bind(&req.brand_id)
        .bind(&req.name)
        .bind(&req.parent_id)
        .bind(sort_order as i32)
        .bind(&now)
        .bind(&now)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to create category: {e}"))?;

        self.get_category(&id).await
    }

    pub async fn update_category(
        &self,
        id: &str,
        req: &UpdateCategoryRequest,
    ) -> Result<Category, String> {
        let mut sets: Vec<String> = Vec::new();
        let mut n = 1;

        if req.name.is_some() {
            sets.push(format!("name = ${}", n));
            n += 1;
        }
        if req.parent_id.is_some() {
            sets.push(format!("parent_id = ${}", n));
            n += 1;
        }
        if req.sort_order.is_some() {
            sets.push(format!("sort_order = ${}", n));
            n += 1;
        }
        if req.is_active.is_some() {
            sets.push(format!("is_active = ${}", n));
            n += 1;
        }
        sets.push(format!("updated_at = ${}", n));
        n += 1;

        if sets.len() <= 1 {
            return self.get_category(id).await;
        }

        let query = format!(
            "UPDATE categories SET {} WHERE id = ${}",
            sets.join(", "),
            n
        );

        let now = chrono::Utc::now().to_rfc3339();
        let mut q = sqlx::query(&query);
        if let Some(ref name) = req.name {
            q = q.bind(name);
        }
        if let Some(ref parent_id) = req.parent_id {
            q = q.bind(parent_id);
        }
        if let Some(sort_order) = req.sort_order {
            q = q.bind(sort_order as i32);
        }
        if let Some(is_active) = req.is_active {
            q = q.bind(is_active as bool);
        }
        q = q.bind(&now).bind(id);

        q.execute(&self.pool)
            .await
            .map_err(|e| format!("Failed to update category: {e}"))?;

        self.get_category(id).await
    }

    pub async fn delete_category(&self, id: &str) -> Result<(), String> {
        sqlx::query("DELETE FROM categories WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| format!("Failed to delete category: {e}"))?;
        Ok(())
    }

    // ---------------------------------------------------------------------------
    // Products
    // ---------------------------------------------------------------------------

    pub async fn list_products(
        &self,
        brand_id: &str,
        category_id: Option<&str>,
        active_only: bool,
        channel_type: Option<&str>,
    ) -> Result<Vec<Product>, String> {
        let mut query =
            String::from("SELECT p.id, p.brand_id, p.category_id, p.name, p.description, p.sku, \
                          p.price, p.compare_at_price, p.currency, p.images, p.variants, \
                          p.inventory_count, p.track_inventory, p.visibility, p.metadata, \
                          p.is_active, p.created_at, p.updated_at \
                          FROM products p WHERE p.brand_id = $1");
        let mut n = 1;

        if active_only {
            query.push_str(" AND p.is_active = 1");
        }
        if category_id.is_some() {
            n += 1;
            query.push_str(&format!(" AND p.category_id = ${}", n));
        }

        query.push_str(" ORDER BY p.name");

        let mut q = sqlx::query(&query).bind(brand_id);
        if let Some(cat) = category_id {
            q = q.bind(cat);
        }
        let row = q
            .fetch_all(&self.pool)
            .await
            .map_err(|e| format!("Failed to list products: {e}"))?;

        row.iter()
            .map(|row| Ok(Product {
                id: row.get("id"),
                brand_id: row.get("brand_id"),
                category_id: row.get("category_id"),
                name: row.get("name"),
                description: row.get("description"),
                sku: row.get("sku"),
                price: row.get("price"),
                compare_at_price: row.get("compare_at_price"),
                currency: row.get("currency"),
                images: row.get("images"),
                variants: row.get("variants"),
                inventory_count: row.get("inventory_count"),
                track_inventory: row.get("track_inventory"),
                visibility: row.get("visibility"),
                metadata: row.get("metadata"),
                is_active: row.get("is_active"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            }))
            .collect()
    }

    pub async fn get_product(&self, id: &str) -> Result<Product, String> {
        let row = sqlx::query(
            "SELECT id, brand_id, category_id, name, description, sku, price, compare_at_price, \
             currency, images, variants, inventory_count, track_inventory, visibility, metadata, \
             is_active, created_at, updated_at FROM products WHERE id = $1",
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Product not found: {e}"))?;
        Ok(Product {
            id: row.get("id"),
            brand_id: row.get("brand_id"),
            category_id: row.get("category_id"),
            name: row.get("name"),
            description: row.get("description"),
            sku: row.get("sku"),
            price: row.get("price"),
            compare_at_price: row.get("compare_at_price"),
            currency: row.get("currency"),
            images: row.get("images"),
            variants: row.get("variants"),
            inventory_count: row.get("inventory_count"),
            track_inventory: row.get("track_inventory"),
            visibility: row.get("visibility"),
            metadata: row.get("metadata"),
            is_active: row.get("is_active"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        })
    }

    pub async fn create_product(&self, req: &CreateProductRequest) -> Result<Product, String> {
        let id = Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        let currency = req.currency.clone().unwrap_or_else(|| "USD".to_string());
        let images = req
            .images
            .as_ref()
            .map(|v| serde_json::to_string(v).unwrap_or_default());
        let variants = req
            .variants
            .as_ref()
            .map(|v| serde_json::to_string(v).unwrap_or_default());
        let visibility = req
            .visibility
            .as_ref()
            .map(|v| serde_json::to_string(v).unwrap_or_default());
        let metadata = req
            .metadata
            .as_ref()
            .map(|v| serde_json::to_string(v).unwrap_or_default());
        let inventory_count = req.inventory_count.unwrap_or(0);
        let track_inventory = req.track_inventory.unwrap_or(false);

        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| format!("Failed to begin transaction: {e}"))?;

        sqlx::query(
            "INSERT INTO products (id, brand_id, category_id, name, description, sku, price, \
             compare_at_price, currency, images, variants, inventory_count, track_inventory, \
             visibility, metadata, is_active, created_at, updated_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, 1, $16, $17)",
        )
        .bind(&id)
        .bind(&req.brand_id)
        .bind(&req.category_id)
        .bind(&req.name)
        .bind(&req.description)
        .bind(&req.sku)
        .bind(req.price)
        .bind(&req.compare_at_price)
        .bind(&currency)
        .bind(&images)
        .bind(&variants)
        .bind(inventory_count as i32)
        .bind(track_inventory as bool)
        .bind(&visibility)
        .bind(&metadata)
        .bind(&now)
        .bind(&now)
        .execute(&mut *tx)
        .await
        .map_err(|e| format!("Failed to create product: {e}"))?;

        // Set channel visibility if provided
        if let Some(vis) = &req.visibility {
            if let Some(channels) = vis.as_object() {
                for (channel, visible) in channels {
                    let vis_id = Uuid::new_v4().to_string();
                    let is_visible = visible.as_bool().unwrap_or(true) as bool;
                    let _ = sqlx::query(
                        "INSERT INTO product_channel_visibility (id, product_id, channel_type, is_visible, created_at) \
                         VALUES ($1, $2, $3, $4, $5)",
                    )
                    .bind(&vis_id)
                    .bind(&id)
                    .bind(channel)
                    .bind(is_visible as bool)
                    .bind(&now)
                    .execute(&mut *tx)
                    .await;
                }
            }
        }

        tx.commit()
            .await
            .map_err(|e| format!("Failed to commit transaction: {e}"))?;

        self.get_product(&id).await
    }

    pub async fn update_product(
        &self,
        id: &str,
        req: &UpdateProductRequest,
    ) -> Result<Product, String> {
        let mut sets = Vec::new();
        let now = chrono::Utc::now().to_rfc3339();

        if let Some(ref name) = req.name {
            sets.push(format!("name = '{}'", name.replace('\'', "''")));
        }
        if let Some(ref description) = req.description {
            if let Some(desc) = description {
                sets.push(format!("description = '{}'", desc.replace('\'', "''")));
            } else {
                sets.push("description = NULL".to_string());
            }
        }
        if let Some(ref sku) = req.sku {
            if let Some(s) = sku {
                sets.push(format!("sku = '{}'", s.replace('\'', "''")));
            } else {
                sets.push("sku = NULL".to_string());
            }
        }
        if let Some(price) = req.price {
            sets.push(format!("price = {}", price));
        }
        if let Some(ref compare_at_price) = req.compare_at_price {
            if let Some(p) = compare_at_price {
                sets.push(format!("compare_at_price = {}", p));
            } else {
                sets.push("compare_at_price = NULL".to_string());
            }
        }
        if let Some(ref currency) = req.currency {
            sets.push(format!("currency = '{}'", currency.replace('\'', "''")));
        }
        if let Some(ref images) = req.images {
            let s = serde_json::to_string(images).unwrap_or_default();
            sets.push(format!("images = '{}'", s.replace('\'', "''")));
        }
        if let Some(ref variants) = req.variants {
            let s = serde_json::to_string(variants).unwrap_or_default();
            sets.push(format!("variants = '{}'", s.replace('\'', "''")));
        }
        if let Some(inventory_count) = req.inventory_count {
            sets.push(format!("inventory_count = {}", inventory_count));
        }
        if let Some(track_inventory) = req.track_inventory {
            sets.push(format!("track_inventory = {}", track_inventory as i32));
        }
        if let Some(ref visibility) = req.visibility {
            let s = serde_json::to_string(visibility).unwrap_or_default();
            sets.push(format!("visibility = '{}'", s.replace('\'', "''")));
        }
        if let Some(ref metadata) = req.metadata {
            let s = serde_json::to_string(metadata).unwrap_or_default();
            sets.push(format!("metadata = '{}'", s.replace('\'', "''")));
        }
        if let Some(is_active) = req.is_active {
            sets.push(format!("is_active = {}", is_active as i32));
        }
        if let Some(ref category_id) = req.category_id {
            sets.push(format!("category_id = '{}'", category_id.replace('\'', "''")));
        }

        sets.push("updated_at = '{}'".to_string());

        if sets.is_empty() {
            return self.get_product(id).await;
        }

        // Use parameterized query for safety
        let query_str = format!(
            "UPDATE products SET {} WHERE id = $1",
            sets.join(", ")
        );
        sqlx::query(&query_str)
            .bind(&now)
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| format!("Failed to update product: {e}"))?;

        self.get_product(id).await
    }

    pub async fn delete_product(&self, id: &str) -> Result<(), String> {
        sqlx::query("DELETE FROM products WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| format!("Failed to delete product: {e}"))?;
        Ok(())
    }

    // ---------------------------------------------------------------------------
    // Inventory
    // ---------------------------------------------------------------------------

    pub async fn update_inventory(
        &self,
        product_id: &str,
        req: &UpdateInventoryRequest,
    ) -> Result<Product, String> {
        let now = chrono::Utc::now().to_rfc3339();
        sqlx::query("UPDATE products SET inventory_count = $1, updated_at = $2 WHERE id = $3")
            .bind(req.inventory_count as i32)
            .bind(&now)
            .bind(product_id)
            .execute(&self.pool)
            .await
            .map_err(|e| format!("Failed to update inventory: {e}"))?;
        self.get_product(product_id).await
    }

    // ---------------------------------------------------------------------------
    // Channel Visibility
    // ---------------------------------------------------------------------------

    pub async fn get_channel_visibility(
        &self,
        product_id: &str,
    ) -> Result<Vec<ProductChannelVisibility>, String> {
        let rows = sqlx::query(
            "SELECT id, product_id, channel_type, is_visible, created_at \
             FROM product_channel_visibility WHERE product_id = $1",
        )
        .bind(product_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to get channel visibility: {e}"))?;
        rows.iter()
            .map(|row| Ok(ProductChannelVisibility {
                id: row.get("id"),
                product_id: row.get("product_id"),
                channel_type: row.get("channel_type"),
                is_visible: row.get("is_visible"),
                created_at: row.get("created_at"),
            }))
            .collect()
    }

    pub async fn set_channel_visibility(
        &self,
        product_id: &str,
        req: &SetChannelVisibilityRequest,
    ) -> Result<ProductChannelVisibility, String> {
        let now = chrono::Utc::now().to_rfc3339();

        // Check if visibility record exists
        let existing = sqlx::query(
            "SELECT id FROM product_channel_visibility WHERE product_id = $1 AND channel_type = $2",
        )
        .bind(product_id)
        .bind(&req.channel_type)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("Failed to check visibility: {e}"))?;

        if let Some(row) = existing {
            let id: String = row.get("id");
            sqlx::query(
                "UPDATE product_channel_visibility SET is_visible = $1 WHERE id = $2",
            )
            .bind(req.is_visible as bool)
            .bind(&id)
            .execute(&self.pool)
            .await
            .map_err(|e| format!("Failed to update visibility: {e}"))?;

            Ok(ProductChannelVisibility {
                id,
                product_id: product_id.to_string(),
                channel_type: req.channel_type.clone(),
                is_visible: req.is_visible,
                created_at: now,
            })
        } else {
            let id = Uuid::new_v4().to_string();
            sqlx::query(
                "INSERT INTO product_channel_visibility (id, product_id, channel_type, is_visible, created_at) \
                 VALUES ($1, $2, $3, $4, $5)",
            )
            .bind(&id)
            .bind(product_id)
            .bind(&req.channel_type)
            .bind(req.is_visible as bool)
            .bind(&now)
            .execute(&self.pool)
            .await
            .map_err(|e| format!("Failed to set visibility: {e}"))?;

            Ok(ProductChannelVisibility {
                id,
                product_id: product_id.to_string(),
                channel_type: req.channel_type.clone(),
                is_visible: req.is_visible,
                created_at: now,
            })
        }
    }
}
