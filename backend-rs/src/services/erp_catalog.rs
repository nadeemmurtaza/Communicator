use sqlx::{PgPool, Row};
use uuid::Uuid;
use crate::models::*;

pub struct ErpCatalogService;

impl ErpCatalogService {
    // ─── Categories ───

    pub async fn list_categories(pool: &PgPool, active_only: bool) -> Result<Vec<ErpCategory>, String> {
        let query = if active_only {
            "SELECT * FROM erp_categories WHERE is_active = true ORDER BY sort_order, id"
        } else {
            "SELECT * FROM erp_categories ORDER BY sort_order, id"
        };
        let rows = sqlx::query_as(query).fetch_all(pool).await.map_err(|e| e.to_string())?;
        Ok(rows)
    }

    pub async fn get_category(pool: &PgPool, id: &str) -> Result<ErpCategory, String> {
        sqlx::query_as("SELECT * FROM erp_categories WHERE id = $1")
            .bind(id)
            .fetch_one(pool)
            .await
            .map_err(|e| e.to_string())
    }

    pub async fn create_category(pool: &PgPool, req: &CreateCategoryRequest) -> Result<ErpCategory, String> {
        let id = Uuid::new_v4().to_string();
        let slug_exists = sqlx::query("SELECT COUNT(*) FROM erp_categories WHERE slug = $1")
            .bind(&req.slug)
            .fetch_one(pool)
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
        .fetch_one(pool)
        .await
        .map_err(|e| e.to_string())?;
        Ok(ErpCategory {
            id,
            slug: row.try_get("slug").unwrap(),
            code: row.try_get("code").ok(),
            parent_id: row.try_get("parent_id").ok(),
            image_url: row.try_get("image_url").ok(),
            sort_order: row.try_get("sort_order").unwrap(),
            is_active: row.try_get("is_active").unwrap_or(true),
            created_at: row.try_get("created_at").unwrap_or_default(),
            updated_at: row.try_get("updated_at").unwrap_or_default(),
        })
    }

    pub async fn update_category(pool: &PgPool, id: &str, req: &UpdateCategoryRequest) -> Result<ErpCategory, String> {
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
            return Self::get_category(pool, id).await;
        }

        sets.push("updated_at = NOW()".to_string());
        let query = format!(
            "UPDATE erp_categories SET {} WHERE id = ${} RETURNING *",
            sets.join(", "),
            idx
        );

        let mut q = sqlx::query_as(&query);
        if let Some(ref s) = req.slug {
            q = q.bind(s);
        }
        if let Some(ref c) = req.code {
            q = q.bind(c);
        }
        if let Some(ref p) = req.parent_id {
            q = q.bind(p);
        }
        if let Some(ref i) = req.image_url {
            q = q.bind(i);
        }
        if let Some(s) = req.sort_order {
            q = q.bind(s);
        }
        if let Some(a) = req.is_active {
            q = q.bind(a);
        }
        q = q.bind(id);

        q.fetch_one(pool)
            .await
            .map_err(|e| e.to_string())
    }

    pub async fn delete_category(pool: &PgPool, id: &str) -> Result<(), String> {
        sqlx::query("DELETE FROM erp_categories WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    // ─── Category Translations ───

    pub async fn list_category_translations(pool: &PgPool, category_id: &str) -> Result<Vec<ErpCategoryTranslation>, String> {
        sqlx::query_as("SELECT * FROM erp_category_translations WHERE category_id = $1 ORDER BY locale")
            .bind(category_id)
            .fetch_all(pool)
            .await
            .map_err(|e| e.to_string())
    }

    pub async fn upsert_category_translation(pool: &PgPool, category_id: &str, locale: &str, name: &str, description: Option<&str>) -> Result<ErpCategoryTranslation, String> {
        let id = Uuid::new_v4().to_string();
        sqlx::query_as(
            "INSERT INTO erp_category_translations (id, category_id, locale, name, description) \
             VALUES ($1, $2, $3, $4, $5) \
             ON CONFLICT (category_id, locale) DO UPDATE SET name = $4, description = $5 \
             RETURNING *"
        )
        .bind(&id)
        .bind(category_id)
        .bind(locale)
        .bind(name)
        .bind(description)
        .fetch_one(pool)
        .await
        .map_err(|e| e.to_string())
    }

    // ─── Products ───

    pub async fn list_products(pool: &PgPool, status: Option<&str>, kind: Option<&str>, limit: i64, offset: i64) -> Result<Vec<ErpProduct>, String> {
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
        if let Some(s) = status {
            q = q.bind(s);
        }
        if let Some(k) = kind {
            q = q.bind(k);
        }
        q = q.bind(limit.max(1).min(100)).bind(offset.max(0));

        q.fetch_all(pool).await.map_err(|e| e.to_string())
    }

    pub async fn get_product(pool: &PgPool, id: &str) -> Result<ErpProduct, String> {
        sqlx::query_as("SELECT * FROM erp_products WHERE id = $1")
            .bind(id)
            .fetch_one(pool)
            .await
            .map_err(|e| e.to_string())
    }

    pub async fn get_product_by_slug(pool: &PgPool, slug: &str) -> Result<ErpProduct, String> {
        sqlx::query_as("SELECT * FROM erp_products WHERE slug = $1 AND status = 'active'")
            .bind(slug)
            .fetch_one(pool)
            .await
            .map_err(|e| e.to_string())
    }

    pub async fn create_product(pool: &PgPool, req: &CreateProductRequest) -> Result<ErpProduct, String> {
        let id = Uuid::new_v4().to_string();
        let slug_exists = sqlx::query("SELECT COUNT(*) FROM erp_products WHERE slug = $1")
            .bind(&req.slug)
            .fetch_one(pool)
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
        .fetch_one(pool)
        .await
        .map_err(|e| e.to_string())?;

        Ok(ErpProduct {
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
        })
    }

    pub async fn update_product(pool: &PgPool, id: &str, req: &UpdateProductRequest) -> Result<ErpProduct, String> {
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
            return Self::get_product(pool, id).await;
        }

        updates.push("updated_at = NOW()".to_string());
        let query = format!(
            "UPDATE erp_products SET {} WHERE id = '{}' RETURNING *",
            updates.join(", "),
            id.replace('\'', "''")
        );
        sqlx::query_as(&query)
            .fetch_one(pool)
            .await
            .map_err(|e| e.to_string())
    }

    pub async fn delete_product(pool: &PgPool, id: &str) -> Result<(), String> {
        sqlx::query("DELETE FROM erp_products WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    // ─── Product Translations ───

    pub async fn list_product_translations(pool: &PgPool, product_id: &str) -> Result<Vec<ErpProductTranslation>, String> {
        sqlx::query_as("SELECT * FROM erp_product_translations WHERE product_id = $1 ORDER BY locale")
            .bind(product_id)
            .fetch_all(pool)
            .await
            .map_err(|e| e.to_string())
    }

    pub async fn upsert_product_translation(pool: &PgPool, product_id: &str, locale: &str, name: &str, description: Option<&str>, care_instructions: Option<&str>) -> Result<ErpProductTranslation, String> {
        let id = Uuid::new_v4().to_string();
        sqlx::query_as(
            "INSERT INTO erp_product_translations (id, product_id, locale, name, description, care_instructions) \
             VALUES ($1, $2, $3, $4, $5, $6) \
             ON CONFLICT (product_id, locale) DO UPDATE SET name = $4, description = $5, care_instructions = $6 \
             RETURNING *"
        )
        .bind(&id)
        .bind(product_id)
        .bind(locale)
        .bind(name)
        .bind(description)
        .bind(care_instructions)
        .fetch_one(pool)
        .await
        .map_err(|e| e.to_string())
    }

    // ─── Variants ───

    pub async fn list_variants(pool: &PgPool, product_id: &str) -> Result<Vec<ErpProductVariant>, String> {
        sqlx::query_as("SELECT * FROM erp_product_variants WHERE product_id = $1 ORDER BY sort_order, id")
            .bind(product_id)
            .fetch_all(pool)
            .await
            .map_err(|e| e.to_string())
    }

    pub async fn get_variant(pool: &PgPool, id: &str) -> Result<ErpProductVariant, String> {
        sqlx::query_as("SELECT * FROM erp_product_variants WHERE id = $1")
            .bind(id)
            .fetch_one(pool)
            .await
            .map_err(|e| e.to_string())
    }

    pub async fn create_variant(pool: &PgPool, req: &CreateVariantRequest) -> Result<ErpProductVariant, String> {
        let id = Uuid::new_v4().to_string();
        let sku_exists = sqlx::query("SELECT COUNT(*) FROM erp_product_variants WHERE sku = $1")
            .bind(&req.sku)
            .fetch_one(pool)
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
        .fetch_one(pool)
        .await
        .map_err(|e| e.to_string())?;

        Ok(ErpProductVariant {
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
        })
    }

    pub async fn update_variant(pool: &PgPool, id: &str, req: &UpdateVariantRequest) -> Result<ErpProductVariant, String> {
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
            return Self::get_variant(pool, id).await;
        }

        updates.push("updated_at = NOW()".to_string());
        let query = format!(
            "UPDATE erp_product_variants SET {} WHERE id = '{}' RETURNING *",
            updates.join(", "),
            id.replace('\'', "''")
        );
        sqlx::query_as(&query)
            .fetch_one(pool)
            .await
            .map_err(|e| e.to_string())
    }

    pub async fn delete_variant(pool: &PgPool, id: &str) -> Result<(), String> {
        sqlx::query("DELETE FROM erp_product_variants WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    // ─── Images ───

    pub async fn list_images(pool: &PgPool, product_id: &str) -> Result<Vec<ErpProductImage>, String> {
        sqlx::query_as("SELECT * FROM erp_product_images WHERE product_id = $1 ORDER BY sort_order, id")
            .bind(product_id)
            .fetch_all(pool)
            .await
            .map_err(|e| e.to_string())
    }

    pub async fn create_image(pool: &PgPool, req: &CreateProductImageRequest) -> Result<ErpProductImage, String> {
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
        .fetch_one(pool)
        .await
        .map_err(|e| e.to_string())?;

        Ok(ErpProductImage {
            id,
            product_id: row.try_get("product_id").unwrap(),
            variant_id: row.try_get("variant_id").ok(),
            url: row.try_get("url").unwrap(),
            alt: row.try_get("alt").ok(),
            sort_order: row.try_get("sort_order").unwrap(),
            is_primary: row.try_get("is_primary").unwrap(),
            created_at: row.try_get("created_at").unwrap_or_default(),
        })
    }

    pub async fn delete_image(pool: &PgPool, id: &str) -> Result<(), String> {
        sqlx::query("DELETE FROM erp_product_images WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    // ─── Options ───

    pub async fn list_product_options(pool: &PgPool, product_id: &str) -> Result<Vec<ErpProductOption>, String> {
        sqlx::query_as("SELECT * FROM erp_product_options WHERE product_id = $1 ORDER BY sort_order, id")
            .bind(product_id)
            .fetch_all(pool)
            .await
            .map_err(|e| e.to_string())
    }

    pub async fn create_product_option(pool: &PgPool, req: &CreateProductOptionRequest) -> Result<ErpProductOption, String> {
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
        .fetch_one(pool)
        .await
        .map_err(|e| e.to_string())?;

        Ok(ErpProductOption {
            id,
            product_id: row.try_get("product_id").unwrap(),
            key: row.try_get("key").unwrap(),
            display: row.try_get("display").unwrap(),
            sort_order: row.try_get("sort_order").unwrap(),
            created_at: row.try_get("created_at").unwrap_or_default(),
        })
    }

    pub async fn delete_product_option(pool: &PgPool, id: &str) -> Result<(), String> {
        sqlx::query("DELETE FROM erp_product_options WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub async fn list_option_values(pool: &PgPool, option_id: &str) -> Result<Vec<ErpProductOptionValue>, String> {
        sqlx::query_as("SELECT * FROM erp_product_option_values WHERE option_id = $1 ORDER BY sort_order, id")
            .bind(option_id)
            .fetch_all(pool)
            .await
            .map_err(|e| e.to_string())
    }

    pub async fn create_option_value(pool: &PgPool, req: &CreateOptionValueRequest) -> Result<ErpProductOptionValue, String> {
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
        .fetch_one(pool)
        .await
        .map_err(|e| e.to_string())?;

        Ok(ErpProductOptionValue {
            id,
            option_id: row.try_get("option_id").unwrap(),
            key: row.try_get("key").unwrap(),
            swatch_hex: row.try_get("swatch_hex").ok(),
            swatch_url: row.try_get("swatch_url").ok(),
            sort_order: row.try_get("sort_order").unwrap(),
        })
    }

    pub async fn delete_option_value(pool: &PgPool, id: &str) -> Result<(), String> {
        sqlx::query("DELETE FROM erp_product_option_values WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    // ─── Collections ───

    pub async fn list_collections(pool: &PgPool, active_only: bool) -> Result<Vec<ErpCollection>, String> {
        let query = if active_only {
            "SELECT * FROM erp_collections WHERE is_active = true ORDER BY created_at DESC"
        } else {
            "SELECT * FROM erp_collections ORDER BY created_at DESC"
        };
        sqlx::query_as(query).fetch_all(pool).await.map_err(|e| e.to_string())
    }

    pub async fn get_collection(pool: &PgPool, id: &str) -> Result<ErpCollection, String> {
        sqlx::query_as("SELECT * FROM erp_collections WHERE id = $1")
            .bind(id)
            .fetch_one(pool)
            .await
            .map_err(|e| e.to_string())
    }

    pub async fn create_collection(pool: &PgPool, req: &CreateCollectionRequest) -> Result<ErpCollection, String> {
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
        .fetch_one(pool)
        .await
        .map_err(|e| e.to_string())?;

        Ok(ErpCollection {
            id,
            slug: row.try_get("slug").unwrap(),
            rule: row.try_get("rule").ok(),
            sort_strategy: row.try_get("sort_strategy").unwrap(),
            image_url: row.try_get("image_url").ok(),
            is_active: row.try_get("is_active").unwrap(),
            created_at: row.try_get("created_at").unwrap_or_default(),
            updated_at: row.try_get("updated_at").unwrap_or_default(),
        })
    }

    pub async fn add_product_to_collection(pool: &PgPool, collection_id: &str, product_id: &str, sort_order: i32) -> Result<(), String> {
        sqlx::query(
            "INSERT INTO erp_collection_products (collection_id, product_id, sort_order) \
             VALUES ($1, $2, $3) ON CONFLICT (collection_id, product_id) DO UPDATE SET sort_order = $3"
        )
        .bind(collection_id)
        .bind(product_id)
        .bind(sort_order)
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub async fn remove_product_from_collection(pool: &PgPool, collection_id: &str, product_id: &str) -> Result<(), String> {
        sqlx::query(
            "DELETE FROM erp_collection_products WHERE collection_id = $1 AND product_id = $2"
        )
        .bind(collection_id)
        .bind(product_id)
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub async fn list_collection_products(pool: &PgPool, collection_id: &str) -> Result<Vec<ErpProduct>, String> {
        sqlx::query_as(
            "SELECT p.* FROM erp_products p \
             INNER JOIN erp_collection_products cp ON p.id = cp.product_id \
             WHERE cp.collection_id = $1 \
             ORDER BY cp.sort_order, p.id"
        )
        .bind(collection_id)
        .fetch_all(pool)
        .await
        .map_err(|e| e.to_string())
    }

    // ─── Stock Adjustments ───

    pub async fn adjust_stock(pool: &PgPool, req: &StockAdjustmentRequest) -> Result<ErpStockAdjustment, String> {
        let id = Uuid::new_v4().to_string();
        // Get current stock
        let current_stock: i32 = sqlx::query_scalar(
            "SELECT stock FROM erp_product_variants WHERE id = $1"
        )
        .bind(&req.variant_id)
        .fetch_one(pool)
        .await
        .map_err(|e| e.to_string())?;

        let new_stock = current_stock + req.delta;
        if new_stock < 0 {
            return Err("Stock cannot go below zero".to_string());
        }

        // Update stock
        sqlx::query(
            "UPDATE erp_product_variants SET stock = $1, updated_at = NOW() WHERE id = $2"
        )
        .bind(new_stock)
        .bind(&req.variant_id)
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;

        // Record adjustment
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
        .fetch_one(pool)
        .await
        .map_err(|e| e.to_string())?;

        Ok(ErpStockAdjustment {
            id,
            variant_id: row.try_get("variant_id").unwrap(),
            delta: row.try_get("delta").unwrap(),
            resulting_stock: row.try_get("resulting_stock").unwrap(),
            reason: row.try_get("reason").unwrap(),
            note: row.try_get("note").ok(),
            actor_id: row.try_get("actor_id").ok(),
            created_at: row.try_get("created_at").unwrap_or_default(),
        })
    }

    pub async fn list_stock_adjustments(pool: &PgPool, variant_id: &str, limit: i64) -> Result<Vec<ErpStockAdjustment>, String> {
        sqlx::query_as(
            "SELECT * FROM erp_stock_adjustments WHERE variant_id = $1 ORDER BY created_at DESC LIMIT $2"
        )
        .bind(variant_id)
        .bind(limit.max(1).min(100))
        .fetch_all(pool)
        .await
        .map_err(|e| e.to_string())
    }

    // ─── Cross-sells ───

    pub async fn add_cross_sell(pool: &PgPool, source_id: &str, target_id: &str, label: Option<&str>) -> Result<(), String> {
        sqlx::query(
            "INSERT INTO erp_product_cross_sells (source_product_id, target_product_id, label) \
             VALUES ($1, $2, $3) ON CONFLICT (source_product_id, target_product_id) DO UPDATE SET label = $3"
        )
        .bind(source_id)
        .bind(target_id)
        .bind(label)
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub async fn remove_cross_sell(pool: &PgPool, source_id: &str, target_id: &str) -> Result<(), String> {
        sqlx::query(
            "DELETE FROM erp_product_cross_sells WHERE source_product_id = $1 AND target_product_id = $2"
        )
        .bind(source_id)
        .bind(target_id)
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub async fn list_cross_sells(pool: &PgPool, product_id: &str) -> Result<Vec<ErpProductCrossSell>, String> {
        sqlx::query_as(
            "SELECT * FROM erp_product_cross_sells WHERE source_product_id = $1"
        )
        .bind(product_id)
        .fetch_all(pool)
        .await
        .map_err(|e| e.to_string())
    }

    // ─── AI Suggestions ───

    pub async fn list_ai_suggestions(pool: &PgPool, product_id: &str) -> Result<Vec<ErpProductAiSuggestion>, String> {
        sqlx::query_as(
            "SELECT * FROM erp_product_ai_suggestions WHERE product_id = $1 ORDER BY created_at DESC"
        )
        .bind(product_id)
        .fetch_all(pool)
        .await
        .map_err(|e| e.to_string())
    }

    pub async fn approve_ai_suggestion(pool: &PgPool, id: &str, reviewed_by: &str) -> Result<(), String> {
        sqlx::query(
            "UPDATE erp_product_ai_suggestions SET status = 'approved', reviewed_at = NOW(), reviewed_by_id = $2 WHERE id = $1"
        )
        .bind(id)
        .bind(reviewed_by)
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub async fn reject_ai_suggestion(pool: &PgPool, id: &str, reviewed_by: &str) -> Result<(), String> {
        sqlx::query(
            "UPDATE erp_product_ai_suggestions SET status = 'rejected', reviewed_at = NOW(), reviewed_by_id = $2 WHERE id = $1"
        )
        .bind(id)
        .bind(reviewed_by)
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    // ─── Redirects ───

    pub async fn create_redirect(pool: &PgPool, from_path: &str, to_path: &str, status_code: i32) -> Result<ErpRedirect, String> {
        let id = Uuid::new_v4().to_string();
        let row = sqlx::query(
            "INSERT INTO erp_redirects (id, from_path, to_path, status_code) \
             VALUES ($1, $2, $3, $4) RETURNING *"
        )
        .bind(&id)
        .bind(from_path)
        .bind(to_path)
        .bind(status_code)
        .fetch_one(pool)
        .await
        .map_err(|e| e.to_string())?;

        Ok(ErpRedirect {
            id,
            from_path: row.try_get("from_path").unwrap(),
            to_path: row.try_get("to_path").unwrap(),
            status_code: row.try_get("status_code").unwrap(),
            is_automatic: row.try_get("is_automatic").unwrap(),
            hit_count: row.try_get("hit_count").unwrap(),
            created_at: row.try_get("created_at").unwrap_or_default(),
        })
    }

    pub async fn resolve_redirect(pool: &PgPool, path: &str) -> Result<Option<ErpRedirect>, String> {
        let redirect = sqlx::query_as("SELECT * FROM erp_redirects WHERE from_path = $1")
            .bind(path)
            .fetch_optional(pool)
            .await
            .map_err(|e| e.to_string())?;

        // Increment hit count
        if redirect.is_some() {
            let _ = sqlx::query("UPDATE erp_redirects SET hit_count = hit_count + 1 WHERE from_path = $1")
                .bind(path)
                .execute(pool)
                .await;
        }

        Ok(redirect)
    }

    pub async fn list_redirects(pool: &PgPool, limit: i64) -> Result<Vec<ErpRedirect>, String> {
        sqlx::query_as(
            "SELECT * FROM erp_redirects ORDER BY hit_count DESC, created_at DESC LIMIT $1"
        )
        .bind(limit.max(1).min(100))
        .fetch_all(pool)
        .await
        .map_err(|e| e.to_string())
    }
}
