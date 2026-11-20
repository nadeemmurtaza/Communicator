use sqlx::{PgPool, Row};
use uuid::Uuid;
use serde::{Deserialize, Serialize};

// ─── Models ───

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ErpCategory {
    pub id: String,
    pub slug: String,
    pub code: Option<String>,
    pub parent_id: Option<String>,
    pub image_url: Option<String>,
    pub sort_order: i32,
    pub is_active: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ErpCategoryTranslation {
    pub id: String,
    pub category_id: String,
    pub locale: String,
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ErpProduct {
    pub id: String,
    pub slug: String,
    pub kind: String,
    pub status: String,
    pub category_id: Option<String>,
    pub base_price: i64,
    pub compare_at_price: Option<i64>,
    pub brand_id: Option<String>,
    pub fabric: Option<String>,
    pub season: Option<String>,
    pub tags: Vec<String>,
    pub code: Option<String>,
    pub noindex: bool,
    pub canonical_override: Option<String>,
    pub piece_count: Option<i32>,
    pub fabric_meters: Option<f64>,
    pub stitching_available: bool,
    pub is_consumable: bool,
    pub garment_type_id: Option<String>,
    pub shipping_class_id: Option<String>,
    pub featured_rank: i32,
    pub published_at: Option<chrono::DateTime<chrono::Utc>>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ErpProductTranslation {
    pub id: String,
    pub product_id: String,
    pub locale: String,
    pub name: String,
    pub description: Option<String>,
    pub care_instructions: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ErpProductVariant {
    pub id: String,
    pub product_id: String,
    pub sku: String,
    pub size: Option<String>,
    pub color: Option<String>,
    pub color_hex: Option<String>,
    pub label: Option<String>,
    pub price_override: Option<i64>,
    pub stock: i32,
    pub low_stock_threshold: i32,
    pub is_active: bool,
    pub cost_price: i64,
    pub reorder_point: Option<i32>,
    pub reorder_quantity: Option<i32>,
    pub preferred_supplier_id: Option<String>,
    pub barcode: Option<String>,
    pub sku_is_manual: bool,
    pub produced_in_house: bool,
    pub lot_tracked: bool,
    pub tracks_serials: bool,
    pub warranty_months: Option<i32>,
    pub tax_class_id: Option<String>,
    pub sort_order: i32,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ErpProductImage {
    pub id: String,
    pub product_id: String,
    pub variant_id: Option<String>,
    pub url: String,
    pub alt: Option<String>,
    pub sort_order: i32,
    pub is_primary: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ErpProductOption {
    pub id: String,
    pub product_id: String,
    pub key: String,
    pub display: String,
    pub sort_order: i32,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ErpProductOptionValue {
    pub id: String,
    pub option_id: String,
    pub key: String,
    pub swatch_hex: Option<String>,
    pub swatch_url: Option<String>,
    pub sort_order: i32,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ErpCollection {
    pub id: String,
    pub slug: String,
    pub rule: Option<String>,
    pub sort_strategy: String,
    pub image_url: Option<String>,
    pub is_active: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ErpStockAdjustment {
    pub id: String,
    pub variant_id: String,
    pub delta: i32,
    pub resulting_stock: i32,
    pub reason: String,
    pub note: Option<String>,
    pub actor_id: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ErpProductCrossSell {
    pub source_product_id: String,
    pub target_product_id: String,
    pub label: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ErpProductAiSuggestion {
    pub id: String,
    pub product_id: String,
    pub field: String,
    pub suggested_value: String,
    pub status: String,
    pub reviewed_by_id: Option<String>,
    pub reviewed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ErpRedirect {
    pub id: String,
    pub from_path: String,
    pub to_path: String,
    pub status_code: i32,
    pub is_automatic: bool,
    pub hit_count: i32,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

// ─── Request DTOs ───

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateCategoryDto {
    pub slug: String,
    pub code: Option<String>,
    pub parent_id: Option<String>,
    pub image_url: Option<String>,
    pub sort_order: Option<i32>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateCategoryDto {
    pub slug: Option<String>,
    pub code: Option<String>,
    pub parent_id: Option<String>,
    pub image_url: Option<String>,
    pub sort_order: Option<i32>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateProductDto {
    pub slug: String,
    pub kind: String,
    pub category_id: String,
    pub base_price: i64,
    pub compare_at_price: Option<i64>,
    pub brand_id: Option<String>,
    pub fabric: Option<String>,
    pub season: Option<String>,
    pub tags: Option<Vec<String>>,
    pub code: Option<String>,
    pub piece_count: Option<i32>,
    pub fabric_meters: Option<f64>,
    pub stitching_available: Option<bool>,
    pub is_consumable: Option<bool>,
    pub featured_rank: Option<i32>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateProductDto {
    pub slug: Option<String>,
    pub kind: Option<String>,
    pub status: Option<String>,
    pub category_id: Option<String>,
    pub base_price: Option<i64>,
    pub compare_at_price: Option<i64>,
    pub brand_id: Option<String>,
    pub fabric: Option<String>,
    pub season: Option<String>,
    pub tags: Option<Vec<String>>,
    pub code: Option<String>,
    pub piece_count: Option<i32>,
    pub fabric_meters: Option<f64>,
    pub stitching_available: Option<bool>,
    pub is_consumable: Option<bool>,
    pub featured_rank: Option<i32>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateVariantDto {
    pub product_id: String,
    pub sku: String,
    pub size: String,
    pub color: String,
    pub color_hex: Option<String>,
    pub label: Option<String>,
    pub price_override: Option<i64>,
    pub stock: Option<i32>,
    pub low_stock_threshold: Option<i32>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateVariantDto {
    pub sku: Option<String>,
    pub size: Option<String>,
    pub color: Option<String>,
    pub color_hex: Option<String>,
    pub label: Option<String>,
    pub price_override: Option<i64>,
    pub stock: Option<i32>,
    pub low_stock_threshold: Option<i32>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateProductImageDto {
    pub product_id: String,
    pub variant_id: Option<String>,
    pub url: String,
    pub sort_order: Option<i32>,
    pub is_primary: Option<bool>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateProductOptionDto {
    pub product_id: String,
    pub key: String,
    pub display: Option<String>,
    pub sort_order: Option<i32>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateOptionValueDto {
    pub option_id: String,
    pub key: String,
    pub swatch_hex: Option<String>,
    pub swatch_url: Option<String>,
    pub sort_order: Option<i32>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateCollectionDto {
    pub slug: String,
    pub rule: Option<String>,
    pub sort_strategy: Option<String>,
    pub image_url: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StockAdjustmentDto {
    pub variant_id: String,
    pub delta: i32,
    pub reason: String,
    pub note: Option<String>,
}

// ─── Service ───

pub struct ErpCatalogService {
    pool: PgPool,
}

impl ErpCatalogService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    // ─── Categories ───

    pub async fn list_categories(&self, active_only: bool) -> Result<String, String> {
        let query = if active_only {
            "SELECT * FROM erp_categories WHERE is_active = true ORDER BY sort_order, id"
        } else {
            "SELECT * FROM erp_categories ORDER BY sort_order, id"
        };
        let rows = sqlx::query_as(query).fetch_all(&self.pool).await.map_err(|e| e.to_string())?;
        serde_json::to_string(&rows).map_err(|e| e.to_string())
    }

    pub async fn get_category(&self, id: &str) -> Result<String, String> {
        let cat = sqlx::query_as("SELECT * FROM erp_categories WHERE id = $1")
            .bind(id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| e.to_string())?;
        serde_json::to_string(&cat).map_err(|e| e.to_string())
    }

    pub async fn create_category(&self, json: &str) -> Result<String, String> {
        let req: CreateCategoryDto = serde_json::from_str(json).map_err(|e| e.to_string())?;
        let id = Uuid::new_v4().to_string();
        let slug_exists = sqlx::query("SELECT COUNT(*) FROM erp_categories WHERE slug = $1")
            .bind(&req.slug)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| e.to_string())?
            .get::<i64, _>(0) > 0;
        if slug_exists {
            return Err(format!("Category slug '{}' already exists", req.slug));
        }
        let row = sqlx::query(
            "INSERT INTO erp_categories (id, slug, code, parent_id, image_url, sort_order) \
             VALUES ($1, $2, $3, $4, $5, $6) RETURNING *"
        )
        .bind(&id)
        .bind(&req.slug)
        .bind(&req.code)
        .bind(&req.parent_id)
        .bind(&req.image_url)
        .bind(req.sort_order.unwrap_or(0))
        .fetch_one(&self.pool)
        .await
        .map_err(|e| e.to_string())?;
        let cat = ErpCategory {
            id,
            slug: row.try_get("slug").unwrap(),
            code: row.try_get("code").ok(),
            parent_id: row.try_get("parent_id").ok(),
            image_url: row.try_get("image_url").ok(),
            sort_order: row.try_get("sort_order").unwrap(),
            is_active: row.try_get("is_active").unwrap_or(true),
            created_at: row.try_get("created_at").unwrap_or_default(),
            updated_at: row.try_get("updated_at").unwrap_or_default(),
        };
        serde_json::to_string(&cat).map_err(|e| e.to_string())
    }

    pub async fn update_category(&self, id: &str, json: &str) -> Result<String, String> {
        let req: UpdateCategoryDto = serde_json::from_str(json).map_err(|e| e.to_string())?;
        let mut sets: Vec<String> = Vec::new();
        let mut idx = 1;
        macro_rules! add_set {
            ($present:expr, $col:expr) => {
                if $present {
                    sets.push(format!("{} = ${}", $col, idx));
                    idx += 1;
                }
            };
        }
        add_set!(req.slug.is_some(), "slug");
        add_set!(req.code.is_some(), "code");
        add_set!(req.parent_id.is_some(), "parent_id");
        add_set!(req.image_url.is_some(), "image_url");
        add_set!(req.sort_order.is_some(), "sort_order");
        add_set!(req.is_active.is_some(), "is_active");

        if sets.is_empty() {
            return self.get_category(id).await;
        }

        sets.push("updated_at = NOW()".to_string());
        let query = format!(
            "UPDATE erp_categories SET {} WHERE id = ${} RETURNING *",
            sets.join(", "),
            idx
        );

        let mut q = sqlx::query_as(&query);
        if let Some(ref s) = req.slug { q = q.bind(s); }
        if let Some(ref c) = req.code { q = q.bind(c); }
        if let Some(ref p) = req.parent_id { q = q.bind(p); }
        if let Some(ref i) = req.image_url { q = q.bind(i); }
        if let Some(s) = req.sort_order { q = q.bind(s); }
        if let Some(a) = req.is_active { q = q.bind(a); }
        q = q.bind(id);

        let cat: ErpCategory = q.fetch_one(&self.pool).await.map_err(|e| e.to_string())?;
        serde_json::to_string(&cat).map_err(|e| e.to_string())
    }

    pub async fn delete_category(&self, id: &str) -> Result<(), String> {
        sqlx::query("DELETE FROM erp_categories WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    // ─── Category Translations ───

    pub async fn list_category_translations(&self, category_id: &str) -> Result<String, String> {
        let rows = sqlx::query_as("SELECT * FROM erp_category_translations WHERE category_id = $1 ORDER BY locale")
            .bind(category_id)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| e.to_string())?;
        serde_json::to_string(&rows).map_err(|e| e.to_string())
    }

    pub async fn upsert_category_translation(&self, category_id: &str, locale: &str, json: &str) -> Result<String, String> {
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct Req { name: String, description: Option<String> }
        let req: Req = serde_json::from_str(json).map_err(|e| e.to_string())?;
        let id = Uuid::new_v4().to_string();
        let trans = sqlx::query_as(
            "INSERT INTO erp_category_translations (id, category_id, locale, name, description) \
             VALUES ($1, $2, $3, $4, $5) \
             ON CONFLICT (category_id, locale) DO UPDATE SET name = $4, description = $5 \
             RETURNING *"
        )
        .bind(&id)
        .bind(category_id)
        .bind(locale)
        .bind(&req.name)
        .bind(req.description.as_deref())
        .fetch_one(&self.pool)
        .await
        .map_err(|e| e.to_string())?;
        serde_json::to_string(&trans).map_err(|e| e.to_string())
    }

    // ─── Products ───

    pub async fn list_products(&self, status: Option<&str>, kind: Option<&str>, limit: i64, offset: i64) -> Result<String, String> {
        let mut query = String::from("SELECT * FROM erp_products WHERE 1=1");
        let mut idx = 1;

        if status.is_some() {
            query.push_str(&format!(" AND status = ${}", idx));
            idx += 1;
        }
        if kind.is_some() {
            query.push_str(&format!(" AND kind = ${}", idx));
            idx += 1;
        }
        query.push_str(&format!(
            " ORDER BY featured_rank DESC, created_at DESC LIMIT ${} OFFSET ${}",
            idx,
            idx + 1
        ));

        let mut q = sqlx::query_as(&query);
        if let Some(s) = status { q = q.bind(s); }
        if let Some(k) = kind { q = q.bind(k); }
        q = q.bind(limit.max(1).min(100)).bind(offset.max(0));

        let rows = q.fetch_all(&self.pool).await.map_err(|e| e.to_string())?;
        serde_json::to_string(&rows).map_err(|e| e.to_string())
    }

    pub async fn get_product(&self, id: &str) -> Result<String, String> {
        let prod = sqlx::query_as("SELECT * FROM erp_products WHERE id = $1")
            .bind(id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| e.to_string())?;
        serde_json::to_string(&prod).map_err(|e| e.to_string())
    }

    pub async fn get_product_by_slug(&self, slug: &str) -> Result<String, String> {
        let prod = sqlx::query_as("SELECT * FROM erp_products WHERE slug = $1 AND status = 'active'")
            .bind(slug)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| e.to_string())?;
        serde_json::to_string(&prod).map_err(|e| e.to_string())
    }

    pub async fn create_product(&self, json: &str) -> Result<String, String> {
        let req: CreateProductDto = serde_json::from_str(json).map_err(|e| e.to_string())?;
        let id = Uuid::new_v4().to_string();
        let slug_exists = sqlx::query("SELECT COUNT(*) FROM erp_products WHERE slug = $1")
            .bind(&req.slug)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| e.to_string())?
            .get::<i64, _>(0) > 0;
        if slug_exists {
            return Err(format!("Product slug '{}' already exists", req.slug));
        }
        let tags_json = serde_json::to_string(&req.tags.as_ref().unwrap_or(&vec![]))
            .unwrap_or_else(|_| "[]".to_string());

        let row = sqlx::query(
            "INSERT INTO erp_products (id, slug, kind, status, category_id, base_price, compare_at_price, \
             brand_id, fabric, season, tags, code, piece_count, fabric_meters, stitching_available, \
             is_consumable, featured_rank) \
             VALUES ($1, $2, $3, 'active', $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16) \
             RETURNING *"
        )
        .bind(&id)
        .bind(&req.slug)
        .bind(&req.kind)
        .bind(&req.category_id)
        .bind(req.base_price)
        .bind(req.compare_at_price)
        .bind(&req.brand_id)
        .bind(&req.fabric)
        .bind(&req.season)
        .bind(&tags_json)
        .bind(&req.code)
        .bind(req.piece_count)
        .bind(req.fabric_meters)
        .bind(req.stitching_available.unwrap_or(false))
        .bind(req.is_consumable.unwrap_or(false))
        .bind(req.featured_rank.unwrap_or(0))
        .fetch_one(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        let prod = ErpProduct {
            id,
            slug: row.try_get("slug").unwrap(),
            kind: row.try_get("kind").unwrap(),
            status: row.try_get("status").unwrap(),
            category_id: row.try_get("category_id").ok(),
            base_price: row.try_get("base_price").unwrap(),
            compare_at_price: row.try_get("compare_at_price").ok(),
            brand_id: row.try_get("brand_id").ok(),
            fabric: row.try_get("fabric").ok(),
            season: row.try_get("season").ok(),
            tags: row.try_get("tags").unwrap_or_default(),
            code: row.try_get("code").ok(),
            noindex: row.try_get("noindex").unwrap_or(false),
            canonical_override: row.try_get("canonical_override").ok(),
            piece_count: row.try_get("piece_count").ok(),
            fabric_meters: row.try_get("fabric_meters").ok(),
            stitching_available: row.try_get("stitching_available").unwrap_or(false),
            is_consumable: row.try_get("is_consumable").unwrap_or(false),
            garment_type_id: row.try_get("garment_type_id").ok(),
            shipping_class_id: row.try_get("shipping_class_id").ok(),
            featured_rank: row.try_get("featured_rank").unwrap_or(0),
            published_at: row.try_get("published_at").ok(),
            created_at: row.try_get("created_at").unwrap_or_default(),
            updated_at: row.try_get("updated_at").unwrap_or_default(),
        };
        serde_json::to_string(&prod).map_err(|e| e.to_string())
    }

    pub async fn update_product(&self, id: &str, json: &str) -> Result<String, String> {
        let req: UpdateProductDto = serde_json::from_str(json).map_err(|e| e.to_string())?;
        let mut updates = Vec::new();

        if let Some(ref slug) = req.slug { updates.push(format!("slug = '{}'", slug.replace('\'', "''"))); }
        if let Some(ref kind) = req.kind { updates.push(format!("kind = '{}'", kind.replace('\'', "''"))); }
        if let Some(ref status) = req.status { updates.push(format!("status = '{}'", status.replace('\'', "''"))); }
        if let Some(ref cat) = req.category_id { updates.push(format!("category_id = '{}'", cat)); }
        if let Some(price) = req.base_price { updates.push(format!("base_price = {}", price)); }
        if let Some(price) = req.compare_at_price { updates.push(format!("compare_at_price = {}", price)); }
        if let Some(ref brand) = req.brand_id { updates.push(format!("brand_id = '{}'", brand)); }
        if let Some(ref fabric) = req.fabric { updates.push(format!("fabric = '{}'", fabric.replace('\'', "''"))); }
        if let Some(ref season) = req.season { updates.push(format!("season = '{}'", season.replace('\'', "''"))); }
        if let Some(ref tags) = req.tags {
            let tags_json = serde_json::to_string(tags).unwrap_or_else(|_| "[]".to_string());
            updates.push(format!("tags = '{}'", tags_json.replace('\'', "''")));
        }
        if let Some(ref code) = req.code { updates.push(format!("code = '{}'", code.replace('\'', "''"))); }
        if let Some(pc) = req.piece_count { updates.push(format!("piece_count = {}", pc)); }
        if let Some(fm) = req.fabric_meters { updates.push(format!("fabric_meters = {}", fm)); }
        if let Some(sa) = req.stitching_available { updates.push(format!("stitching_available = {}", sa)); }
        if let Some(ic) = req.is_consumable { updates.push(format!("is_consumable = {}", ic)); }
        if let Some(fr) = req.featured_rank { updates.push(format!("featured_rank = {}", fr)); }

        if updates.is_empty() {
            return self.get_product(id).await;
        }

        updates.push("updated_at = NOW()".to_string());
        let query = format!(
            "UPDATE erp_products SET {} WHERE id = '{}' RETURNING *",
            updates.join(", "),
            id.replace('\'', "''")
        );
        let prod: ErpProduct = sqlx::query_as(&query)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| e.to_string())?;
        serde_json::to_string(&prod).map_err(|e| e.to_string())
    }

    pub async fn delete_product(&self, id: &str) -> Result<(), String> {
        sqlx::query("DELETE FROM erp_products WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    // ─── Product Translations ───

    pub async fn list_product_translations(&self, product_id: &str) -> Result<String, String> {
        let rows = sqlx::query_as("SELECT * FROM erp_product_translations WHERE product_id = $1 ORDER BY locale")
            .bind(product_id)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| e.to_string())?;
        serde_json::to_string(&rows).map_err(|e| e.to_string())
    }

    pub async fn upsert_product_translation(&self, product_id: &str, locale: &str, json: &str) -> Result<String, String> {
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct Req { name: String, description: Option<String>, care_instructions: Option<String> }
        let req: Req = serde_json::from_str(json).map_err(|e| e.to_string())?;
        if req.name.is_empty() {
            return Err("Name is required".to_string());
        }
        let id = Uuid::new_v4().to_string();
        let trans = sqlx::query_as(
            "INSERT INTO erp_product_translations (id, product_id, locale, name, description, care_instructions) \
             VALUES ($1, $2, $3, $4, $5, $6) \
             ON CONFLICT (product_id, locale) DO UPDATE SET name = $4, description = $5, care_instructions = $6 \
             RETURNING *"
        )
        .bind(&id)
        .bind(product_id)
        .bind(locale)
        .bind(&req.name)
        .bind(req.description.as_deref())
        .bind(req.care_instructions.as_deref())
        .fetch_one(&self.pool)
        .await
        .map_err(|e| e.to_string())?;
        serde_json::to_string(&trans).map_err(|e| e.to_string())
    }

    // ─── Variants ───

    pub async fn list_variants(&self, product_id: &str) -> Result<String, String> {
        let rows = sqlx::query_as("SELECT * FROM erp_product_variants WHERE product_id = $1 ORDER BY sort_order, id")
            .bind(product_id)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| e.to_string())?;
        serde_json::to_string(&rows).map_err(|e| e.to_string())
    }

    pub async fn get_variant(&self, id: &str) -> Result<String, String> {
        let var = sqlx::query_as("SELECT * FROM erp_product_variants WHERE id = $1")
            .bind(id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| e.to_string())?;
        serde_json::to_string(&var).map_err(|e| e.to_string())
    }

    pub async fn create_variant(&self, json: &str) -> Result<String, String> {
        let req: CreateVariantDto = serde_json::from_str(json).map_err(|e| e.to_string())?;
        let id = Uuid::new_v4().to_string();
        let sku_exists = sqlx::query("SELECT COUNT(*) FROM erp_product_variants WHERE sku = $1")
            .bind(&req.sku)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| e.to_string())?
            .get::<i64, _>(0) > 0;
        if sku_exists {
            return Err(format!("Variant SKU '{}' already exists", req.sku));
        }
        let row = sqlx::query(
            "INSERT INTO erp_product_variants (id, product_id, sku, size, color, color_hex, label, \
             price_override, stock, low_stock_threshold) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10) RETURNING *"
        )
        .bind(&id)
        .bind(&req.product_id)
        .bind(&req.sku)
        .bind(&req.size)
        .bind(&req.color)
        .bind(&req.color_hex)
        .bind(&req.label)
        .bind(req.price_override)
        .bind(req.stock.unwrap_or(0))
        .bind(req.low_stock_threshold.unwrap_or(3))
        .fetch_one(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        let var = ErpProductVariant {
            id,
            product_id: row.try_get("product_id").unwrap(),
            sku: row.try_get("sku").unwrap(),
            size: row.try_get("size").ok(),
            color: row.try_get("color").ok(),
            color_hex: row.try_get("color_hex").ok(),
            label: row.try_get("label").ok(),
            price_override: row.try_get("price_override").ok(),
            stock: row.try_get("stock").unwrap(),
            low_stock_threshold: row.try_get("low_stock_threshold").unwrap(),
            is_active: row.try_get("is_active").unwrap_or(true),
            cost_price: row.try_get("cost_price").unwrap_or(0),
            reorder_point: row.try_get("reorder_point").ok(),
            reorder_quantity: row.try_get("reorder_quantity").ok(),
            preferred_supplier_id: row.try_get("preferred_supplier_id").ok(),
            barcode: row.try_get("barcode").ok(),
            sku_is_manual: row.try_get("sku_is_manual").unwrap_or(false),
            produced_in_house: row.try_get("produced_in_house").unwrap_or(false),
            lot_tracked: row.try_get("lot_tracked").unwrap_or(false),
            tracks_serials: row.try_get("tracks_serials").unwrap_or(false),
            warranty_months: row.try_get("warranty_months").ok(),
            tax_class_id: row.try_get("tax_class_id").ok(),
            sort_order: row.try_get("sort_order").unwrap_or(0),
            created_at: row.try_get("created_at").unwrap_or_default(),
            updated_at: row.try_get("updated_at").unwrap_or_default(),
        };
        serde_json::to_string(&var).map_err(|e| e.to_string())
    }

    pub async fn update_variant(&self, id: &str, json: &str) -> Result<String, String> {
        let req: UpdateVariantDto = serde_json::from_str(json).map_err(|e| e.to_string())?;
        let mut updates = Vec::new();
        if let Some(ref sku) = req.sku { updates.push(format!("sku = '{}'", sku.replace('\'', "''"))); }
        if let Some(ref size) = req.size { updates.push(format!("size = '{}'", size.replace('\'', "''"))); }
        if let Some(ref color) = req.color { updates.push(format!("color = '{}'", color.replace('\'', "''"))); }
        if let Some(ref hex) = req.color_hex { updates.push(format!("color_hex = '{}'", hex)); }
        if let Some(ref label) = req.label { updates.push(format!("label = '{}'", label.replace('\'', "''"))); }
        if let Some(po) = req.price_override { updates.push(format!("price_override = {}", po)); }
        if let Some(stock) = req.stock { updates.push(format!("stock = {}", stock)); }
        if let Some(threshold) = req.low_stock_threshold { updates.push(format!("low_stock_threshold = {}", threshold)); }
        if let Some(active) = req.is_active { updates.push(format!("is_active = {}", active)); }

        if updates.is_empty() {
            return self.get_variant(id).await;
        }

        updates.push("updated_at = NOW()".to_string());
        let query = format!(
            "UPDATE erp_product_variants SET {} WHERE id = '{}' RETURNING *",
            updates.join(", "),
            id.replace('\'', "''")
        );
        let var: ErpProductVariant = sqlx::query_as(&query)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| e.to_string())?;
        serde_json::to_string(&var).map_err(|e| e.to_string())
    }

    pub async fn delete_variant(&self, id: &str) -> Result<(), String> {
        sqlx::query("DELETE FROM erp_product_variants WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    // ─── Images ───

    pub async fn list_images(&self, product_id: &str) -> Result<String, String> {
        let rows = sqlx::query_as("SELECT * FROM erp_product_images WHERE product_id = $1 ORDER BY sort_order, id")
            .bind(product_id)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| e.to_string())?;
        serde_json::to_string(&rows).map_err(|e| e.to_string())
    }

    pub async fn create_image(&self, json: &str) -> Result<String, String> {
        let req: CreateProductImageDto = serde_json::from_str(json).map_err(|e| e.to_string())?;
        let id = Uuid::new_v4().to_string();
        let row = sqlx::query(
            "INSERT INTO erp_product_images (id, product_id, variant_id, url, sort_order, is_primary) \
             VALUES ($1, $2, $3, $4, $5, $6) RETURNING *"
        )
        .bind(&id)
        .bind(&req.product_id)
        .bind(&req.variant_id)
        .bind(&req.url)
        .bind(req.sort_order.unwrap_or(0))
        .bind(req.is_primary.unwrap_or(false))
        .fetch_one(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        let img = ErpProductImage {
            id,
            product_id: row.try_get("product_id").unwrap(),
            variant_id: row.try_get("variant_id").ok(),
            url: row.try_get("url").unwrap(),
            alt: row.try_get("alt").ok(),
            sort_order: row.try_get("sort_order").unwrap(),
            is_primary: row.try_get("is_primary").unwrap(),
            created_at: row.try_get("created_at").unwrap_or_default(),
        };
        serde_json::to_string(&img).map_err(|e| e.to_string())
    }

    pub async fn delete_image(&self, id: &str) -> Result<(), String> {
        sqlx::query("DELETE FROM erp_product_images WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    // ─── Options ───

    pub async fn list_product_options(&self, product_id: &str) -> Result<String, String> {
        let rows = sqlx::query_as("SELECT * FROM erp_product_options WHERE product_id = $1 ORDER BY sort_order, id")
            .bind(product_id)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| e.to_string())?;
        serde_json::to_string(&rows).map_err(|e| e.to_string())
    }

    pub async fn create_product_option(&self, json: &str) -> Result<String, String> {
        let req: CreateProductOptionDto = serde_json::from_str(json).map_err(|e| e.to_string())?;
        let id = Uuid::new_v4().to_string();
        let row = sqlx::query(
            "INSERT INTO erp_product_options (id, product_id, key, display, sort_order) \
             VALUES ($1, $2, $3, $4, $5) RETURNING *"
        )
        .bind(&id)
        .bind(&req.product_id)
        .bind(&req.key)
        .bind(req.display.as_deref().unwrap_or("button"))
        .bind(req.sort_order.unwrap_or(0))
        .fetch_one(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        let opt = ErpProductOption {
            id,
            product_id: row.try_get("product_id").unwrap(),
            key: row.try_get("key").unwrap(),
            display: row.try_get("display").unwrap(),
            sort_order: row.try_get("sort_order").unwrap(),
            created_at: row.try_get("created_at").unwrap_or_default(),
        };
        serde_json::to_string(&opt).map_err(|e| e.to_string())
    }

    pub async fn delete_product_option(&self, id: &str) -> Result<(), String> {
        sqlx::query("DELETE FROM erp_product_options WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub async fn list_option_values(&self, option_id: &str) -> Result<String, String> {
        let rows = sqlx::query_as("SELECT * FROM erp_product_option_values WHERE option_id = $1 ORDER BY sort_order, id")
            .bind(option_id)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| e.to_string())?;
        serde_json::to_string(&rows).map_err(|e| e.to_string())
    }

    pub async fn create_option_value(&self, json: &str) -> Result<String, String> {
        let req: CreateOptionValueDto = serde_json::from_str(json).map_err(|e| e.to_string())?;
        let id = Uuid::new_v4().to_string();
        let row = sqlx::query(
            "INSERT INTO erp_product_option_values (id, option_id, key, swatch_hex, swatch_url, sort_order) \
             VALUES ($1, $2, $3, $4, $5, $6) RETURNING *"
        )
        .bind(&id)
        .bind(&req.option_id)
        .bind(&req.key)
        .bind(&req.swatch_hex)
        .bind(&req.swatch_url)
        .bind(req.sort_order.unwrap_or(0))
        .fetch_one(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        let val = ErpProductOptionValue {
            id,
            option_id: row.try_get("option_id").unwrap(),
            key: row.try_get("key").unwrap(),
            swatch_hex: row.try_get("swatch_hex").ok(),
            swatch_url: row.try_get("swatch_url").ok(),
            sort_order: row.try_get("sort_order").unwrap(),
        };
        serde_json::to_string(&val).map_err(|e| e.to_string())
    }

    pub async fn delete_option_value(&self, id: &str) -> Result<(), String> {
        sqlx::query("DELETE FROM erp_product_option_values WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    // ─── Collections ───

    pub async fn list_collections(&self) -> Result<String, String> {
        let rows = sqlx::query_as("SELECT * FROM erp_collections WHERE is_active = true ORDER BY created_at DESC")
            .fetch_all(&self.pool)
            .await
            .map_err(|e| e.to_string())?;
        serde_json::to_string(&rows).map_err(|e| e.to_string())
    }

    pub async fn get_collection(&self, id: &str) -> Result<String, String> {
        let coll = sqlx::query_as("SELECT * FROM erp_collections WHERE id = $1")
            .bind(id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| e.to_string())?;
        serde_json::to_string(&coll).map_err(|e| e.to_string())
    }

    pub async fn create_collection(&self, json: &str) -> Result<String, String> {
        let req: CreateCollectionDto = serde_json::from_str(json).map_err(|e| e.to_string())?;
        let id = Uuid::new_v4().to_string();
        let row = sqlx::query(
            "INSERT INTO erp_collections (id, slug, rule, sort_strategy, image_url) \
             VALUES ($1, $2, $3, $4, $5) RETURNING *"
        )
        .bind(&id)
        .bind(&req.slug)
        .bind(&req.rule)
        .bind(req.sort_strategy.as_deref().unwrap_or("manual"))
        .bind(&req.image_url)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        let coll = ErpCollection {
            id,
            slug: row.try_get("slug").unwrap(),
            rule: row.try_get("rule").ok(),
            sort_strategy: row.try_get("sort_strategy").unwrap(),
            image_url: row.try_get("image_url").ok(),
            is_active: row.try_get("is_active").unwrap(),
            created_at: row.try_get("created_at").unwrap_or_default(),
            updated_at: row.try_get("updated_at").unwrap_or_default(),
        };
        serde_json::to_string(&coll).map_err(|e| e.to_string())
    }

    pub async fn add_product_to_collection(&self, collection_id: &str, json: &str) -> Result<(), String> {
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct Req { product_id: String, sort_order: Option<i32> }
        let req: Req = serde_json::from_str(json).map_err(|e| e.to_string())?;
        sqlx::query(
            "INSERT INTO erp_collection_products (collection_id, product_id, sort_order) \
             VALUES ($1, $2, $3) ON CONFLICT (collection_id, product_id) DO UPDATE SET sort_order = $3"
        )
        .bind(collection_id)
        .bind(&req.product_id)
        .bind(req.sort_order.unwrap_or(0))
        .execute(&self.pool)
        .await
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub async fn remove_product_from_collection(&self, collection_id: &str, product_id: &str) -> Result<(), String> {
        sqlx::query(
            "DELETE FROM erp_collection_products WHERE collection_id = $1 AND product_id = $2"
        )
        .bind(collection_id)
        .bind(product_id)
        .execute(&self.pool)
        .await
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub async fn list_collection_products(&self, collection_id: &str) -> Result<String, String> {
        let rows = sqlx::query_as(
            "SELECT p.* FROM erp_products p \
             INNER JOIN erp_collection_products cp ON p.id = cp.product_id \
             WHERE cp.collection_id = $1 \
             ORDER BY cp.sort_order, p.id"
        )
        .bind(collection_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| e.to_string())?;
        serde_json::to_string(&rows).map_err(|e| e.to_string())
    }

    // ─── Stock Adjustments ───

    pub async fn adjust_stock(&self, json: &str) -> Result<String, String> {
        let req: StockAdjustmentDto = serde_json::from_str(json).map_err(|e| e.to_string())?;
        let id = Uuid::new_v4().to_string();
        let current_stock: i32 = sqlx::query_scalar(
            "SELECT stock FROM erp_product_variants WHERE id = $1"
        )
        .bind(&req.variant_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        let new_stock = current_stock + req.delta;
        if new_stock < 0 {
            return Err("Stock cannot go below zero".to_string());
        }

        sqlx::query(
            "UPDATE erp_product_variants SET stock = $1, updated_at = NOW() WHERE id = $2"
        )
        .bind(new_stock)
        .bind(&req.variant_id)
        .execute(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        let row = sqlx::query(
            "INSERT INTO erp_stock_adjustments (id, variant_id, delta, resulting_stock, reason, note) \
             VALUES ($1, $2, $3, $4, $5, $6) RETURNING *"
        )
        .bind(&id)
        .bind(&req.variant_id)
        .bind(req.delta)
        .bind(new_stock)
        .bind(&req.reason)
        .bind(&req.note)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        let adj = ErpStockAdjustment {
            id,
            variant_id: row.try_get("variant_id").unwrap(),
            delta: row.try_get("delta").unwrap(),
            resulting_stock: row.try_get("resulting_stock").unwrap(),
            reason: row.try_get("reason").unwrap(),
            note: row.try_get("note").ok(),
            actor_id: row.try_get("actor_id").ok(),
            created_at: row.try_get("created_at").unwrap_or_default(),
        };
        serde_json::to_string(&adj).map_err(|e| e.to_string())
    }

    pub async fn list_stock_adjustments(&self, variant_id: &str, limit: i64) -> Result<String, String> {
        let rows = sqlx::query_as(
            "SELECT * FROM erp_stock_adjustments WHERE variant_id = $1 ORDER BY created_at DESC LIMIT $2"
        )
        .bind(variant_id)
        .bind(limit.max(1).min(100))
        .fetch_all(&self.pool)
        .await
        .map_err(|e| e.to_string())?;
        serde_json::to_string(&rows).map_err(|e| e.to_string())
    }

    // ─── Cross-sells ───

    pub async fn add_cross_sell(&self, source_id: &str, json: &str) -> Result<(), String> {
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct Req { target_product_id: String, label: Option<String> }
        let req: Req = serde_json::from_str(json).map_err(|e| e.to_string())?;
        sqlx::query(
            "INSERT INTO erp_product_cross_sells (source_product_id, target_product_id, label) \
             VALUES ($1, $2, $3) ON CONFLICT (source_product_id, target_product_id) DO UPDATE SET label = $3"
        )
        .bind(source_id)
        .bind(&req.target_product_id)
        .bind(req.label.as_deref())
        .execute(&self.pool)
        .await
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub async fn list_cross_sells(&self, product_id: &str) -> Result<String, String> {
        let rows = sqlx::query_as(
            "SELECT * FROM erp_product_cross_sells WHERE source_product_id = $1"
        )
        .bind(product_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| e.to_string())?;
        serde_json::to_string(&rows).map_err(|e| e.to_string())
    }

    // ─── AI Suggestions ───

    pub async fn list_ai_suggestions(&self, product_id: &str) -> Result<String, String> {
        let rows = sqlx::query_as(
            "SELECT * FROM erp_product_ai_suggestions WHERE product_id = $1 ORDER BY created_at DESC"
        )
        .bind(product_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| e.to_string())?;
        serde_json::to_string(&rows).map_err(|e| e.to_string())
    }

    pub async fn approve_ai_suggestion(&self, id: &str) -> Result<(), String> {
        let reviewed_by = "system".to_string();
        sqlx::query(
            "UPDATE erp_product_ai_suggestions SET status = 'approved', reviewed_at = NOW(), reviewed_by_id = $2 WHERE id = $1"
        )
        .bind(id)
        .bind(&reviewed_by)
        .execute(&self.pool)
        .await
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub async fn reject_ai_suggestion(&self, id: &str) -> Result<(), String> {
        let reviewed_by = "system".to_string();
        sqlx::query(
            "UPDATE erp_product_ai_suggestions SET status = 'rejected', reviewed_at = NOW(), reviewed_by_id = $2 WHERE id = $1"
        )
        .bind(id)
        .bind(&reviewed_by)
        .execute(&self.pool)
        .await
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    // ─── Redirects ───

    pub async fn list_redirects(&self) -> Result<String, String> {
        let rows = sqlx::query_as(
            "SELECT * FROM erp_redirects ORDER BY hit_count DESC, created_at DESC LIMIT $1"
        )
        .bind(100)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| e.to_string())?;
        serde_json::to_string(&rows).map_err(|e| e.to_string())
    }

    pub async fn create_redirect(&self, json: &str) -> Result<String, String> {
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct Req { from_path: String, to_path: String, status_code: Option<i32> }
        let req: Req = serde_json::from_str(json).map_err(|e| e.to_string())?;
        let id = Uuid::new_v4().to_string();
        let row = sqlx::query(
            "INSERT INTO erp_redirects (id, from_path, to_path, status_code) \
             VALUES ($1, $2, $3, $4) RETURNING *"
        )
        .bind(&id)
        .bind(&req.from_path)
        .bind(&req.to_path)
        .bind(req.status_code.unwrap_or(301))
        .fetch_one(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        let red = ErpRedirect {
            id,
            from_path: row.try_get("from_path").unwrap(),
            to_path: row.try_get("to_path").unwrap(),
            status_code: row.try_get("status_code").unwrap(),
            is_automatic: row.try_get("is_automatic").unwrap(),
            hit_count: row.try_get("hit_count").unwrap(),
            created_at: row.try_get("created_at").unwrap_or_default(),
        };
        serde_json::to_string(&red).map_err(|e| e.to_string())
    }
}
