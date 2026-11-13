#![allow(dead_code)]

//! Product Options/Add-ons service (Gap #1)
//! Allows products to have optional extras with individual prices.

use serde::{Deserialize, Serialize};
use sqlx::{Row, PgPool};
use uuid::Uuid;

use crate::models::{ProductOption, ProductOptionValue};

// ---------------------------------------------------------------------------
// DTOs
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct CreateProductOptionRequest {
    pub product_id: String,
    pub name: String,
    pub description: Option<String>,
    pub price_adjustment: Option<f64>,
    pub required: Option<bool>,
    pub max_selections: Option<i32>,
    pub sort_order: Option<i32>,
    pub values: Option<Vec<CreateProductOptionValueRequest>>,
}

#[derive(Debug, Deserialize)]
pub struct CreateProductOptionValueRequest {
    pub name: String,
    pub price_adjustment: Option<f64>,
    pub sku: Option<String>,
    pub sort_order: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateProductOptionRequest {
    pub name: Option<String>,
    pub description: Option<Option<String>>,
    pub price_adjustment: Option<f64>,
    pub required: Option<bool>,
    pub max_selections: Option<i32>,
    pub sort_order: Option<i32>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct ProductOptionWithValues {
    pub option: ProductOption,
    pub values: Vec<ProductOptionValue>,
}

// ---------------------------------------------------------------------------
// Service
// ---------------------------------------------------------------------------

pub struct ProductOptionsService {
    pool: PgPool,
}

impl ProductOptionsService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn list_options(&self, product_id: &str) -> Result<Vec<ProductOptionWithValues>, String> {
        let options = sqlx::query(
            "SELECT id, product_id, name, description, price_adjustment, required, \
             max_selections, sort_order, is_active, created_at, updated_at \
             FROM product_options WHERE product_id = ? AND is_active = 1 ORDER BY sort_order, name",
        )
        .bind(product_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to list options: {e}"))?;

        let mut result = Vec::new();
        for opt in options {
            let option = ProductOption {
                id: opt.get("id"),
                product_id: opt.get("product_id"),
                name: opt.get("name"),
                description: opt.get("description"),
                price_adjustment: opt.get("price_adjustment"),
                required: opt.get("required"),
                max_selections: opt.get("max_selections"),
                sort_order: opt.get("sort_order"),
                is_active: opt.get("is_active"),
                created_at: opt.get("created_at"),
                updated_at: opt.get("updated_at"),
            };

            let values = self.list_option_values(&option.id).await?;
            result.push(ProductOptionWithValues { option, values });
        }
        Ok(result)
    }

    pub async fn get_option(&self, id: &str) -> Result<ProductOptionWithValues, String> {
        let row = sqlx::query(
            "SELECT id, product_id, name, description, price_adjustment, required, \
             max_selections, sort_order, is_active, created_at, updated_at \
             FROM product_options WHERE id = ?",
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Option not found: {e}"))?;

        let option = ProductOption {
            id: row.get("id"),
            product_id: row.get("product_id"),
            name: row.get("name"),
            description: row.get("description"),
            price_adjustment: row.get("price_adjustment"),
            required: row.get("required"),
            max_selections: row.get("max_selections"),
            sort_order: row.get("sort_order"),
            is_active: row.get("is_active"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        };

        let values = self.list_option_values(&option.id).await?;
        Ok(ProductOptionWithValues { option, values })
    }

    pub async fn create_option(&self, req: &CreateProductOptionRequest) -> Result<ProductOptionWithValues, String> {
        let id = Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        let price_adjustment = req.price_adjustment.unwrap_or(0.0);
        let required = req.required.unwrap_or(false);
        let max_selections = req.max_selections.unwrap_or(1);
        let sort_order = req.sort_order.unwrap_or(0);

        let mut tx = self.pool.begin().await.map_err(|e| format!("Failed to begin transaction: {e}"))?;

        sqlx::query(
            "INSERT INTO product_options (id, product_id, name, description, price_adjustment, \
             required, max_selections, sort_order, is_active, created_at, updated_at) \
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, 1, ?, ?)",
        )
        .bind(&id)
        .bind(&req.product_id)
        .bind(&req.name)
        .bind(&req.description)
        .bind(price_adjustment)
        .bind(required as bool)
        .bind(max_selections as i32)
        .bind(sort_order as i32)
        .bind(&now)
        .bind(&now)
        .execute(&mut *tx)
        .await
        .map_err(|e| format!("Failed to create option: {e}"))?;

        // Create option values
        if let Some(values) = &req.values {
            for val in values {
                let val_id = Uuid::new_v4().to_string();
                let val_price = val.price_adjustment.unwrap_or(0.0);
                let val_sort = val.sort_order.unwrap_or(0);
                let _ = sqlx::query(
                    "INSERT INTO product_option_values (id, option_id, name, price_adjustment, sku, sort_order, is_active, created_at) \
                     VALUES (?, ?, ?, ?, ?, ?, 1, ?)",
                )
                .bind(&val_id)
                .bind(&id)
                .bind(&val.name)
                .bind(val_price)
                .bind(&val.sku)
                .bind(val_sort as i32)
                .bind(&now)
                .execute(&mut *tx)
                .await;
            }
        }

        tx.commit().await.map_err(|e| format!("Failed to commit: {e}"))?;
        self.get_option(&id).await
    }

    pub async fn update_option(
        &self,
        id: &str,
        req: &UpdateProductOptionRequest,
    ) -> Result<ProductOptionWithValues, String> {
        let mut sets = Vec::new();
        let now = chrono::Utc::now().to_rfc3339();

        if let Some(ref name) = req.name {
            sets.push(format!("name = '{}'", name.replace('\'', "''")));
        }
        if let Some(ref desc) = req.description {
            if let Some(d) = desc {
                sets.push(format!("description = '{}'", d.replace('\'', "''")));
            } else {
                sets.push("description = NULL".to_string());
            }
        }
        if let Some(price) = req.price_adjustment {
            sets.push(format!("price_adjustment = {}", price));
        }
        if let Some(required) = req.required {
            sets.push(format!("required = {}", required as i32));
        }
        if let Some(max) = req.max_selections {
            sets.push(format!("max_selections = {}", max));
        }
        if let Some(sort) = req.sort_order {
            sets.push(format!("sort_order = {}", sort));
        }
        if let Some(active) = req.is_active {
            sets.push(format!("is_active = {}", active as i32));
        }
        sets.push(format!("updated_at = '{}'", now));

        if sets.is_empty() {
            return self.get_option(id).await;
        }

        let query = format!("UPDATE product_options SET {} WHERE id = ?", sets.join(", "));
        sqlx::query(&query)
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| format!("Failed to update option: {e}"))?;

        self.get_option(id).await
    }

    pub async fn delete_option(&self, id: &str) -> Result<(), String> {
        sqlx::query("DELETE FROM product_options WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| format!("Failed to delete option: {e}"))?;
        Ok(())
    }

    pub async fn create_option_value(
        &self,
        option_id: &str,
        req: &CreateProductOptionValueRequest,
    ) -> Result<ProductOptionValue, String> {
        let id = Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        let price_adjustment = req.price_adjustment.unwrap_or(0.0);
        let sort_order = req.sort_order.unwrap_or(0);

        sqlx::query(
            "INSERT INTO product_option_values (id, option_id, name, price_adjustment, sku, sort_order, is_active, created_at) \
             VALUES (?, ?, ?, ?, ?, ?, 1, ?)",
        )
        .bind(&id)
        .bind(option_id)
        .bind(&req.name)
        .bind(price_adjustment)
        .bind(&req.sku)
        .bind(sort_order as i32)
        .bind(&now)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to create option value: {e}"))?;

        self.get_option_value(&id).await
    }

    pub async fn delete_option_value(&self, id: &str) -> Result<(), String> {
        sqlx::query("DELETE FROM product_option_values WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| format!("Failed to delete option value: {e}"))?;
        Ok(())
    }

    // ---------------------------------------------------------------------------
    // Helpers
    // ---------------------------------------------------------------------------

    async fn list_option_values(&self, option_id: &str) -> Result<Vec<ProductOptionValue>, String> {
        let rows = sqlx::query(
            "SELECT id, option_id, name, price_adjustment, sku, sort_order, is_active, created_at \
             FROM product_option_values WHERE option_id = ? AND is_active = 1 ORDER BY sort_order, name",
        )
        .bind(option_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to list option values: {e}"))?;

        rows.iter()
            .map(|row| Ok(ProductOptionValue {
                id: row.get("id"),
                option_id: row.get("option_id"),
                name: row.get("name"),
                price_adjustment: row.get("price_adjustment"),
                sku: row.get("sku"),
                sort_order: row.get("sort_order"),
                is_active: row.get("is_active"),
                created_at: row.get("created_at"),
            }))
            .collect()
    }

    async fn get_option_value(&self, id: &str) -> Result<ProductOptionValue, String> {
        let row = sqlx::query(
            "SELECT id, option_id, name, price_adjustment, sku, sort_order, is_active, created_at \
             FROM product_option_values WHERE id = ?",
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Option value not found: {e}"))?;

        Ok(ProductOptionValue {
            id: row.get("id"),
            option_id: row.get("option_id"),
            name: row.get("name"),
            price_adjustment: row.get("price_adjustment"),
            sku: row.get("sku"),
            sort_order: row.get("sort_order"),
            is_active: row.get("is_active"),
            created_at: row.get("created_at"),
        })
    }
}
