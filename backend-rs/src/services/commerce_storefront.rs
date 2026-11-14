#![allow(dead_code)]

//! Storefront service: public-facing storefront, sections, and visit tracking

use serde::{Deserialize, Serialize};
use sqlx::{Row, PgPool};
use uuid::Uuid;

use crate::models::{Storefront, StorefrontSection, StorefrontVisit};

// ---------------------------------------------------------------------------
// DTOs
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct CreateStorefrontRequest {
    pub brand_id: String,
    pub slug: String,
    pub title: String,
    pub description: Option<String>,
    pub theme: Option<serde_json::Value>,
    pub hero_image: Option<String>,
    pub seo_title: Option<String>,
    pub seo_description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateStorefrontRequest {
    pub title: Option<String>,
    pub description: Option<Option<String>>,
    pub theme: Option<serde_json::Value>,
    pub hero_image: Option<Option<String>>,
    pub seo_title: Option<Option<String>>,
    pub seo_description: Option<Option<String>>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct CreateStorefrontSectionRequest {
    pub storefront_id: String,
    pub section_type: String,
    pub sort_order: Option<i32>,
    pub content: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateStorefrontSectionRequest {
    pub section_type: Option<String>,
    pub sort_order: Option<i32>,
    pub content: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
pub struct StorefrontPublicResponse {
    pub storefront: Storefront,
    pub sections: Vec<StorefrontSection>,
    pub products: Vec<serde_json::Value>,
}

// ---------------------------------------------------------------------------
// Service
// ---------------------------------------------------------------------------

pub struct CommerceStorefrontService {
    pool: PgPool,
}

impl CommerceStorefrontService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    // ---------------------------------------------------------------------------
    // Storefronts
    // ---------------------------------------------------------------------------

    pub async fn list_storefronts(&self, brand_id: &str) -> Result<Vec<Storefront>, String> {
        let rows = sqlx::query("SELECT * FROM storefronts WHERE brand_id = $1 ORDER BY created_at DESC")
            .bind(brand_id)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| e.to_string())?;
        rows.iter()
            .map(|r| self.row_to_storefront(r))
            .collect::<Result<Vec<_>, _>>()
    }

    pub async fn get_storefront(&self, id: &str) -> Result<Storefront, String> {
        let row = sqlx::query("SELECT * FROM storefronts WHERE id = $1")
            .bind(id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| e.to_string())?;
        self.row_to_storefront(&row)
    }

    pub async fn get_storefront_by_slug(&self, slug: &str) -> Result<Option<Storefront>, String> {
        let row = sqlx::query("SELECT * FROM storefronts WHERE slug = $1 AND is_active = 1")
            .bind(slug)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| e.to_string())?;
        match row {
            Some(r) => Ok(Some(self.row_to_storefront(&r)?)),
            None => Ok(None),
        }
    }

    pub async fn create_storefront(&self, req: &CreateStorefrontRequest) -> Result<Storefront, String> {
        let id = Uuid::new_v4().to_string();
        let theme = req.theme.as_ref().map(|v| serde_json::to_string(v).unwrap_or_default());
        sqlx::query(
            "INSERT INTO storefronts (id, brand_id, slug, title, description, theme, hero_image, \
             seo_title, seo_description) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)"
        )
        .bind(&id)
        .bind(&req.brand_id)
        .bind(&req.slug)
        .bind(&req.title)
        .bind(req.description.as_deref())
        .bind(theme.as_deref())
        .bind(req.hero_image.as_deref())
        .bind(req.seo_title.as_deref())
        .bind(req.seo_description.as_deref())
        .execute(&self.pool)
        .await
        .map_err(|e| e.to_string())?;
        self.get_storefront(&id).await
    }

    pub async fn update_storefront(&self, id: &str, req: &UpdateStorefrontRequest) -> Result<Storefront, String> {
        let mut sets: Vec<String> = Vec::new();
        let mut n = 1;
        if req.title.is_some() { sets.push(format!("title = ${}", n)); n += 1; }
        if req.is_active.is_some() { sets.push(format!("is_active = ${}", n)); n += 1; }
        sets.push("updated_at = NOW()::TEXT".to_string());

        if sets.len() <= 1 {
            return self.get_storefront(id).await;
        }

        let query = format!("UPDATE storefronts SET {} WHERE id = ${}", sets.join(", "), n);
        let mut q = sqlx::query(&query);
        if let Some(ref title) = req.title { q = q.bind(title); }
        if let Some(is_active) = req.is_active { q = q.bind(is_active); }
        q = q.bind(id);
        q.execute(&self.pool)
            .await
            .map_err(|e| e.to_string())?;
        self.get_storefront(id).await
    }

    pub async fn delete_storefront(&self, id: &str) -> Result<(), String> {
        sqlx::query("DELETE FROM storefronts WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    // ---------------------------------------------------------------------------
    // Storefront Sections
    // ---------------------------------------------------------------------------

    pub async fn list_sections(&self, storefront_id: &str) -> Result<Vec<StorefrontSection>, String> {
        let rows = sqlx::query("SELECT * FROM storefront_sections WHERE storefront_id = $1 ORDER BY sort_order")
            .bind(storefront_id)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| e.to_string())?;
        rows.iter()
            .map(|r| self.row_to_section(r))
            .collect::<Result<Vec<_>, _>>()
    }

    pub async fn create_section(&self, req: &CreateStorefrontSectionRequest) -> Result<StorefrontSection, String> {
        let id = Uuid::new_v4().to_string();
        let content = req.content.as_ref().map(|v| serde_json::to_string(v).unwrap_or_default());
        sqlx::query(
            "INSERT INTO storefront_sections (id, storefront_id, section_type, sort_order, content) \
             VALUES ($1, $2, $3, $4, $5)"
        )
        .bind(&id)
        .bind(&req.storefront_id)
        .bind(&req.section_type)
        .bind(req.sort_order.unwrap_or(0))
        .bind(content.as_deref())
        .execute(&self.pool)
        .await
        .map_err(|e| e.to_string())?;
        self.get_section(&id).await
    }

    pub async fn get_section(&self, id: &str) -> Result<StorefrontSection, String> {
        let row = sqlx::query("SELECT * FROM storefront_sections WHERE id = $1")
            .bind(id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| e.to_string())?;
        self.row_to_section(&row)
    }

    pub async fn update_section(&self, id: &str, req: &UpdateStorefrontSectionRequest) -> Result<StorefrontSection, String> {
        let mut sets: Vec<String> = Vec::new();
        let mut n = 1;
        if req.section_type.is_some() { sets.push(format!("section_type = ${}", n)); n += 1; }
        if req.sort_order.is_some() { sets.push(format!("sort_order = ${}", n)); n += 1; }

        if sets.is_empty() {
            return self.get_section(id).await;
        }

        let query = format!("UPDATE storefront_sections SET {} WHERE id = ${}", sets.join(", "), n);
        let mut q = sqlx::query(&query);
        if let Some(ref section_type) = req.section_type { q = q.bind(section_type); }
        if let Some(sort_order) = req.sort_order { q = q.bind(sort_order); }
        q = q.bind(id);
        q.execute(&self.pool)
            .await
            .map_err(|e| e.to_string())?;
        self.get_section(id).await
    }

    pub async fn delete_section(&self, id: &str) -> Result<(), String> {
        sqlx::query("DELETE FROM storefront_sections WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    // ---------------------------------------------------------------------------
    // Public Storefront
    // ---------------------------------------------------------------------------

    pub async fn get_public_storefront(&self, slug: &str) -> Result<Option<StorefrontPublicResponse>, String> {
        let storefront = match self.get_storefront_by_slug(slug).await? {
            Some(sf) => sf,
            None => return Ok(None),
        };

        let sections = self.list_sections(&storefront.id).await?;

        // Get active products for this brand
        let brand_id = &storefront.brand_id;
        let products = sqlx::query(
            "SELECT id, name, description, sku, price, compare_at_price, currency, images, \
             inventory_count, is_active FROM products \
             WHERE brand_id = $1 AND is_active = 1 ORDER BY created_at DESC LIMIT 50"
        )
        .bind(brand_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        let product_values: Vec<serde_json::Value> = products.iter().map(|r| {
            serde_json::json!({
                "id": r.try_get::<String, _>("id").unwrap_or_default(),
                "name": r.try_get::<String, _>("name").unwrap_or_default(),
                "description": r.try_get::<Option<String>, _>("description").unwrap_or(None),
                "sku": r.try_get::<Option<String>, _>("sku").unwrap_or(None),
                "price": r.try_get::<f64, _>("price").unwrap_or(0.0),
                "compareAtPrice": r.try_get::<Option<f64>, _>("compare_at_price").unwrap_or(None),
                "currency": r.try_get::<String, _>("currency").unwrap_or_else(|_| "USD".to_string()),
                "images": r.try_get::<Option<String>, _>("images").unwrap_or(None),
                "inventoryCount": r.try_get::<i32, _>("inventory_count").unwrap_or(0),
            })
        }).collect();

        Ok(Some(StorefrontPublicResponse {
            storefront,
            sections,
            products: product_values,
        }))
    }

    // ---------------------------------------------------------------------------
    // Visit Tracking
    // ---------------------------------------------------------------------------

    pub async fn track_visit(&self, storefront_id: &str, visitor_ip: Option<&str>, user_agent: Option<&str>, path: Option<&str>) -> Result<StorefrontVisit, String> {
        let id = Uuid::new_v4().to_string();
        sqlx::query(
            "INSERT INTO storefront_visits (id, storefront_id, visitor_ip, user_agent, path) \
             VALUES ($1, $2, $3, $4, $5)"
        )
        .bind(&id)
        .bind(storefront_id)
        .bind(visitor_ip)
        .bind(user_agent)
        .bind(path)
        .execute(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        Ok(StorefrontVisit {
            id,
            storefront_id: storefront_id.to_string(),
            visitor_ip: visitor_ip.unwrap_or_default().to_string(),
            user_agent: user_agent.map(|s| s.to_string()),
            path: path.unwrap_or_default().to_string(),
            visited_at: chrono::Utc::now().to_rfc3339(),
        })
    }

    pub async fn visit_stats(&self, storefront_id: &str, days: i32) -> Result<(i32, i32), String> {
        let days = if days > 0 { days } else { 30 };
        let row = sqlx::query(
            "SELECT COUNT(*) as total, COUNT(DISTINCT visitor_ip) as unique_visitors \
             FROM storefront_visits WHERE storefront_id = $1 AND visited_at >= CURRENT_DATE - $2::INTEGER"
        )
        .bind(storefront_id)
        .bind(days)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| e.to_string())?;
        let total: i32 = row.get("total");
        let unique: i32 = row.get("unique_visitors");
        Ok((total, unique))
    }

    // ---------------------------------------------------------------------------
    // Row helpers
    // ---------------------------------------------------------------------------

    fn row_to_storefront(&self, r: &sqlx::postgres::PgRow) -> Result<Storefront, String> {
        Ok(Storefront {
            id: r.try_get("id").map_err(|e| e.to_string())?,
            brand_id: r.try_get("brand_id").map_err(|e| e.to_string())?,
            slug: r.try_get("slug").map_err(|e| e.to_string())?,
            title: r.try_get("title").map_err(|e| e.to_string())?,
            description: r.try_get("description").map_err(|e| e.to_string())?,
            theme: r.try_get("theme").map_err(|e| e.to_string())?,
            hero_image: r.try_get("hero_image").map_err(|e| e.to_string())?,
            seo_title: r.try_get("seo_title").map_err(|e| e.to_string())?,
            seo_description: r.try_get("seo_description").map_err(|e| e.to_string())?,
            is_active: r.try_get("is_active").map_err(|e| e.to_string())?,
            created_at: r.try_get("created_at").map_err(|e| e.to_string())?,
            updated_at: r.try_get("updated_at").map_err(|e| e.to_string())?,
        })
    }

    fn row_to_section(&self, r: &sqlx::postgres::PgRow) -> Result<StorefrontSection, String> {
        Ok(StorefrontSection {
            id: r.try_get("id").map_err(|e| e.to_string())?,
            storefront_id: r.try_get("storefront_id").map_err(|e| e.to_string())?,
            section_type: r.try_get("section_type").map_err(|e| e.to_string())?,
            sort_order: r.try_get("sort_order").map_err(|e| e.to_string())?,
            content: r.try_get("content").map_err(|e| e.to_string())?,
            created_at: r.try_get("created_at").map_err(|e| e.to_string())?,
        })
    }
}
