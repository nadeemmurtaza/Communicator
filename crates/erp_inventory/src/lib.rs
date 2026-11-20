use sqlx::PgPool;
use uuid::Uuid;
use chrono::Utc;
use serde::{Deserialize, Serialize};

// ─── Models ───

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ErpLocation {
    pub id: String,
    pub name: String,
    pub code: String,
    pub r#type: String,
    pub retail_brand_id: Option<String>,
    pub origin_node_id: Option<String>,
    pub is_default: bool,
    pub is_active: bool,
    pub address: Option<serde_json::Value>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ErpVariantLocationStock {
    pub variant_id: String,
    pub location_id: String,
    pub quantity: i32,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ErpStockMovement {
    pub id: String,
    pub variant_id: String,
    pub delta: i32,
    pub resulting_stock: i32,
    pub reason: String,
    pub location_id: Option<String>,
    pub ref_type: Option<String>,
    pub ref_id: Option<String>,
    pub unit_cost: Option<i32>,
    pub note: Option<String>,
    pub actor_id: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ErpStockVariance {
    pub id: String,
    pub variant_id: String,
    pub location_id: String,
    pub overshoot_qty: i32,
    pub stock_movement_id: String,
    pub ref_type: Option<String>,
    pub ref_id: Option<String>,
    pub status: String,
    pub resolved_by_user_id: Option<String>,
    pub resolved_at: Option<String>,
    pub resolution_note: Option<String>,
    pub pos_shift_id: Option<String>,
    pub serialized_unit_id: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ErpSpotCheckPolicy {
    pub id: String,
    pub location_id: Option<String>,
    pub min_skus_per_check: i32,
    pub frequency_days: i32,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ErpSpotCheck {
    pub id: String,
    pub policy_id: String,
    pub location_id: String,
    pub status: String,
    pub scheduled_for: String,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ErpSpotCheckLine {
    pub id: String,
    pub spot_check_id: String,
    pub variant_id: String,
    pub expected: i32,
    pub counted: Option<i32>,
    pub variance: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ErpVariantSalesStat {
    pub variant_id: String,
    pub units_30: i32,
    pub units_90: i32,
    pub last_sold_at: Option<String>,
    pub days_of_cover: Option<f64>,
    pub age_days: Option<i32>,
    pub is_slow_mover: bool,
    pub is_dead_stock: bool,
    pub last_low_stock_alert_at: Option<String>,
    pub computed_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ErpVariantForecast {
    pub variant_id: String,
    pub forecasted_daily_demand: f64,
    pub seasonal_factor: Option<f64>,
    pub projected_stockout_at: Option<String>,
    pub reason_en: Option<String>,
    pub computed_at: String,
    pub safety_stock_units: Option<i32>,
    pub suggested_reorder_point: Option<i32>,
    pub service_level_percent: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ErpMarkdownSuggestion {
    pub id: String,
    pub variant_id: String,
    pub discount_id: Option<String>,
    pub suggested_bps: i32,
    pub estimated_clearance_pct: f64,
    pub estimated_clearance_days: i32,
    pub margin_before_paisa: i32,
    pub margin_after_paisa: i32,
    pub status: String,
    pub created_at: String,
    pub resolved_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ErpLot {
    pub id: String,
    pub product_variant_id: String,
    pub lot_code: String,
    pub expires_on: Option<String>,
    pub received_on: String,
    pub source_receipt_id: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ErpLotStock {
    pub id: String,
    pub lot_id: String,
    pub location_id: String,
    pub quantity: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ErpLotAllocation {
    pub id: String,
    pub lot_id: String,
    pub stock_movement_id: String,
    pub quantity: i32,
    pub allocated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ErpSerializedUnit {
    pub id: String,
    pub product_variant_id: String,
    pub serial_number: String,
    pub status: String,
    pub lot_id: Option<String>,
    pub location_id: Option<String>,
    pub order_id: Option<String>,
    pub stock_movement_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ErpStockCount {
    pub id: String,
    pub count_number: String,
    pub status: String,
    pub note: Option<String>,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
    pub cancelled_at: Option<String>,
    pub created_by: Option<String>,
    pub assigned_to: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ErpStockCountLine {
    pub id: String,
    pub stock_count_id: String,
    pub variant_id: String,
    pub expected: Option<i32>,
    pub counted: Option<i32>,
    pub note: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

// ─── Request DTOs ───

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateLocationRequest {
    pub name: String,
    pub code: String,
    pub r#type: Option<String>,
    pub address: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateLocationRequest {
    pub name: Option<String>,
    pub r#type: Option<String>,
    pub is_active: Option<bool>,
    pub address: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateStockMovementRequest {
    pub variant_id: String,
    pub delta: i32,
    pub reason: String,
    pub location_id: Option<String>,
    pub ref_type: Option<String>,
    pub ref_id: Option<String>,
    pub unit_cost: Option<i32>,
    pub note: Option<String>,
    pub actor_id: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateStockVarianceRequest {
    pub variant_id: String,
    pub location_id: String,
    pub overshoot_qty: i32,
    pub stock_movement_id: String,
    pub ref_type: Option<String>,
    pub ref_id: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolveVarianceRequest {
    pub resolved_by_user_id: String,
    pub resolution_note: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateSpotCheckPolicyRequest {
    pub location_id: Option<String>,
    pub min_skus_per_check: i32,
    pub frequency_days: i32,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateSpotCheckRequest {
    pub policy_id: String,
    pub location_id: String,
    pub scheduled_for: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateSpotCheckLineRequest {
    pub spot_check_id: String,
    pub variant_id: String,
    pub expected: i32,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateSpotCheckLineRequest {
    pub counted: i32,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateLotRequest {
    pub product_variant_id: String,
    pub lot_code: String,
    pub expires_on: Option<String>,
    pub source_receipt_id: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateLotStockRequest {
    pub lot_id: String,
    pub location_id: String,
    pub quantity: i32,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateLotAllocationRequest {
    pub lot_id: String,
    pub stock_movement_id: String,
    pub quantity: i32,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateSerializedUnitRequest {
    pub product_variant_id: String,
    pub serial_number: String,
    pub lot_id: Option<String>,
    pub location_id: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateSerializedUnitStatusRequest {
    pub status: String,
    pub order_id: Option<String>,
    pub location_id: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateStockCountRequest {
    pub count_number: Option<String>,
    pub note: Option<String>,
    pub assigned_to: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateStockCountLineRequest {
    pub stock_count_id: String,
    pub variant_id: String,
    pub expected: Option<i32>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateStockCountLineRequest {
    pub counted: i32,
    pub note: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateVariantForecastRequest {
    pub variant_id: String,
    pub forecasted_daily_demand: f64,
    pub seasonal_factor: Option<f64>,
    pub reason_en: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateMarkdownSuggestionRequest {
    pub variant_id: String,
    pub suggested_bps: i32,
    pub estimated_clearance_pct: f64,
    pub estimated_clearance_days: i32,
    pub margin_before_paisa: i32,
    pub margin_after_paisa: i32,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateMarkdownSuggestionStatusRequest {
    pub status: String,
    pub discount_id: Option<String>,
}

// ─── Service ───

#[derive(Clone)]
pub struct ErpInventoryService {
    pool: PgPool,
}

impl ErpInventoryService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    // ============================================================
    // Locations
    // ============================================================

    pub async fn list_locations(&self, is_active: Option<bool>) -> Result<String, String> {
        let locations = if let Some(active) = is_active {
            sqlx::query_as::<_, ErpLocation>(
                "SELECT * FROM erp_locations WHERE is_active = $1 ORDER BY code"
            )
            .bind(active)
            .fetch_all(&self.pool)
            .await
        } else {
            sqlx::query_as::<_, ErpLocation>(
                "SELECT * FROM erp_locations ORDER BY code"
            )
            .fetch_all(&self.pool)
            .await
        }
        .map_err(|e| e.to_string())?;
        serde_json::to_string(&locations).map_err(|e| e.to_string())
    }

    pub async fn get_location(&self, id: &str) -> Result<String, String> {
        let location = sqlx::query_as::<_, ErpLocation>(
            "SELECT * FROM erp_locations WHERE id = $1"
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| e.to_string())?;
        serde_json::to_string(&location).map_err(|e| e.to_string())
    }

    pub async fn get_location_by_code(&self, code: &str) -> Result<String, String> {
        let location = sqlx::query_as::<_, ErpLocation>(
            "SELECT * FROM erp_locations WHERE code = $1"
        )
        .bind(code)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| e.to_string())?;
        serde_json::to_string(&location).map_err(|e| e.to_string())
    }

    pub async fn create_location(&self, json: &str) -> Result<String, String> {
        let req: CreateLocationRequest = serde_json::from_str(json).map_err(|e| e.to_string())?;
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();
        let r#type = req.r#type.clone().unwrap_or_else(|| "STORE".to_string());
        let address_json = match &req.address {
            Some(a) => a.to_string(),
            None => "{}".to_string(),
        };

        let location = sqlx::query_as::<_, ErpLocation>(
            "INSERT INTO erp_locations (id, name, code, type, address, created_at)
             VALUES ($1, $2, $3, $4, $5::jsonb, $6)
             RETURNING *"
        )
        .bind(&id)
        .bind(&req.name)
        .bind(&req.code)
        .bind(&r#type)
        .bind(&address_json)
        .bind(&now)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        serde_json::to_string(&location).map_err(|e| e.to_string())
    }

    pub async fn update_location(&self, id: &str, json: &str) -> Result<String, String> {
        let req: UpdateLocationRequest = serde_json::from_str(json).map_err(|e| e.to_string())?;
        let now = Utc::now().to_rfc3339();

        let location = sqlx::query_as::<_, ErpLocation>(
            "UPDATE erp_locations SET
                name = COALESCE($1, name),
                type = COALESCE($2, type),
                is_active = COALESCE($3, is_active),
                address = COALESCE($4::jsonb, address),
                updated_at = $5
            WHERE id = $6 RETURNING *"
        )
        .bind(&req.name)
        .bind(&req.r#type)
        .bind(req.is_active)
        .bind(req.address.as_ref().map(|a| a.to_string()))
        .bind(&now)
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        serde_json::to_string(&location).map_err(|e| e.to_string())
    }

    // ============================================================
    // Variant Location Stock
    // ============================================================

    pub async fn get_variant_location_stock(&self, variant_id: &str, location_id: &str) -> Result<String, String> {
        let stock = sqlx::query_as::<_, ErpVariantLocationStock>(
            "SELECT * FROM erp_variant_location_stock WHERE variant_id = $1 AND location_id = $2"
        )
        .bind(variant_id)
        .bind(location_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| e.to_string())?;
        serde_json::to_string(&stock).map_err(|e| e.to_string())
    }

    pub async fn get_stock_by_location(&self, location_id: &str) -> Result<String, String> {
        let stocks = sqlx::query_as::<_, ErpVariantLocationStock>(
            "SELECT * FROM erp_variant_location_stock WHERE location_id = $1 ORDER BY variant_id"
        )
        .bind(location_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| e.to_string())?;
        serde_json::to_string(&stocks).map_err(|e| e.to_string())
    }

    pub async fn get_stock_by_variant(&self, variant_id: &str) -> Result<String, String> {
        let stocks = sqlx::query_as::<_, ErpVariantLocationStock>(
            "SELECT * FROM erp_variant_location_stock WHERE variant_id = $1 ORDER BY location_id"
        )
        .bind(variant_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| e.to_string())?;
        serde_json::to_string(&stocks).map_err(|e| e.to_string())
    }

    pub async fn update_variant_location_stock(&self, variant_id: &str, location_id: &str, quantity: i32) -> Result<String, String> {
        let now = Utc::now().to_rfc3339();

        let stock = sqlx::query_as::<_, ErpVariantLocationStock>(
            "INSERT INTO erp_variant_location_stock (variant_id, location_id, quantity, updated_at)
             VALUES ($1, $2, $3, $4)
             ON CONFLICT (variant_id, location_id)
             DO UPDATE SET quantity = $3, updated_at = $4
             RETURNING *"
        )
        .bind(variant_id)
        .bind(location_id)
        .bind(quantity)
        .bind(&now)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        serde_json::to_string(&stock).map_err(|e| e.to_string())
    }

    pub async fn get_total_stock(&self, variant_id: &str) -> Result<String, String> {
        let row: (i32,) = sqlx::query_as(
            "SELECT COALESCE(SUM(quantity), 0) FROM erp_variant_location_stock WHERE variant_id = $1"
        )
        .bind(variant_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| e.to_string())?;
        serde_json::to_string(&serde_json::json!({"variantId": variant_id, "totalStock": row.0})).map_err(|e| e.to_string())
    }

    // ============================================================
    // Stock Movements (the ledger)
    // ============================================================

    pub async fn list_stock_movements(&self, variant_id: Option<&str>, reason: Option<&str>, location_id: Option<&str>, limit: i64, offset: i64) -> Result<String, String> {
        let mut query = String::from("SELECT * FROM erp_stock_movements WHERE 1=1");
        let mut param_idx = 1;

        if let Some(v) = variant_id {
            query.push_str(&format!(" AND variant_id = ${}", param_idx));
            param_idx += 1;
        }

        if let Some(r) = reason {
            query.push_str(&format!(" AND reason = ${}", param_idx));
            param_idx += 1;
        }

        if let Some(l) = location_id {
            query.push_str(&format!(" AND location_id = ${}", param_idx));
            param_idx += 1;
        }

        let limit_p = param_idx;
        param_idx += 1;
        let offset_p = param_idx;
        query.push_str(&format!(" ORDER BY created_at DESC LIMIT ${} OFFSET ${}", limit_p, offset_p));

        let movements = sqlx::query_as::<_, ErpStockMovement>(&query)
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| e.to_string())?;

        serde_json::to_string(&movements).map_err(|e| e.to_string())
    }

    pub async fn get_stock_movement(&self, id: &str) -> Result<String, String> {
        let movement = sqlx::query_as::<_, ErpStockMovement>(
            "SELECT * FROM erp_stock_movements WHERE id = $1"
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| e.to_string())?;
        serde_json::to_string(&movement).map_err(|e| e.to_string())
    }

    pub async fn get_movements_by_variant(&self, variant_id: &str, limit: i64) -> Result<String, String> {
        let movements = sqlx::query_as::<_, ErpStockMovement>(
            "SELECT * FROM erp_stock_movements WHERE variant_id = $1 ORDER BY created_at DESC LIMIT $2"
        )
        .bind(variant_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| e.to_string())?;
        serde_json::to_string(&movements).map_err(|e| e.to_string())
    }

    pub async fn create_stock_movement(&self, json: &str) -> Result<String, String> {
        let req: CreateStockMovementRequest = serde_json::from_str(json).map_err(|e| e.to_string())?;
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();

        // Calculate resulting stock
        let current_stock = Self::get_total_stock_int(&self.pool, &req.variant_id).await.unwrap_or(0);
        let resulting_stock = current_stock + req.delta;

        // Update variant location stock if location specified
        if let Some(ref loc_id) = req.location_id {
            Self::update_variant_location_stock_int(&self.pool, &req.variant_id, loc_id, resulting_stock).await?;
        }

        let movement = sqlx::query_as::<_, ErpStockMovement>(
            "INSERT INTO erp_stock_movements (id, variant_id, delta, resulting_stock, reason, location_id, ref_type, ref_id, unit_cost, note, actor_id, created_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
             RETURNING *"
        )
        .bind(&id)
        .bind(&req.variant_id)
        .bind(req.delta)
        .bind(resulting_stock)
        .bind(&req.reason)
        .bind(&req.location_id)
        .bind(&req.ref_type)
        .bind(&req.ref_id)
        .bind(req.unit_cost)
        .bind(&req.note)
        .bind(&req.actor_id)
        .bind(&now)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        serde_json::to_string(&movement).map_err(|e| e.to_string())
    }

    // ============================================================
    // Stock Variances
    // ============================================================

    pub async fn list_stock_variances(&self, status: Option<&str>, variant_id: Option<&str>, limit: i64, offset: i64) -> Result<String, String> {
        let mut query = String::from("SELECT * FROM erp_stock_variances WHERE 1=1");
        let mut param_idx = 1;

        if let Some(s) = status {
            query.push_str(&format!(" AND status = ${}", param_idx));
            param_idx += 1;
        }

        if let Some(v) = variant_id {
            query.push_str(&format!(" AND variant_id = ${}", param_idx));
            param_idx += 1;
        }

        let limit_p = param_idx;
        param_idx += 1;
        let offset_p = param_idx;
        query.push_str(&format!(" ORDER BY created_at DESC LIMIT ${} OFFSET ${}", limit_p, offset_p));

        let variances = sqlx::query_as::<_, ErpStockVariance>(&query)
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| e.to_string())?;

        serde_json::to_string(&variances).map_err(|e| e.to_string())
    }

    pub async fn get_stock_variance(&self, id: &str) -> Result<String, String> {
        let variance = sqlx::query_as::<_, ErpStockVariance>(
            "SELECT * FROM erp_stock_variances WHERE id = $1"
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| e.to_string())?;
        serde_json::to_string(&variance).map_err(|e| e.to_string())
    }

    pub async fn create_stock_variance(&self, json: &str) -> Result<String, String> {
        let req: CreateStockVarianceRequest = serde_json::from_str(json).map_err(|e| e.to_string())?;
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();

        let variance = sqlx::query_as::<_, ErpStockVariance>(
            "INSERT INTO erp_stock_variances (id, variant_id, location_id, overshoot_qty, stock_movement_id, ref_type, ref_id, created_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
             RETURNING *"
        )
        .bind(&id)
        .bind(&req.variant_id)
        .bind(&req.location_id)
        .bind(req.overshoot_qty)
        .bind(&req.stock_movement_id)
        .bind(&req.ref_type)
        .bind(&req.ref_id)
        .bind(&now)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        serde_json::to_string(&variance).map_err(|e| e.to_string())
    }

    pub async fn resolve_variance(&self, id: &str, json: &str) -> Result<String, String> {
        let req: ResolveVarianceRequest = serde_json::from_str(json).map_err(|e| e.to_string())?;
        let now = Utc::now().to_rfc3339();

        let variance = sqlx::query_as::<_, ErpStockVariance>(
            "UPDATE erp_stock_variances SET status = 'RESOLVED', resolved_by_user_id = $1, resolved_at = $2, resolution_note = $3 WHERE id = $4 RETURNING *"
        )
        .bind(&req.resolved_by_user_id)
        .bind(&now)
        .bind(&req.resolution_note)
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        serde_json::to_string(&variance).map_err(|e| e.to_string())
    }

    // ============================================================
    // Spot Check Policies
    // ============================================================

    pub async fn list_spot_check_policies(&self, is_active: Option<bool>) -> Result<String, String> {
        let policies = if let Some(active) = is_active {
            sqlx::query_as::<_, ErpSpotCheckPolicy>(
                "SELECT * FROM erp_spot_check_policies WHERE is_active = $1 ORDER BY id"
            )
            .bind(active)
            .fetch_all(&self.pool)
            .await
        } else {
            sqlx::query_as::<_, ErpSpotCheckPolicy>(
                "SELECT * FROM erp_spot_check_policies ORDER BY id"
            )
            .fetch_all(&self.pool)
            .await
        }
        .map_err(|e| e.to_string())?;
        serde_json::to_string(&policies).map_err(|e| e.to_string())
    }

    pub async fn create_spot_check_policy(&self, json: &str) -> Result<String, String> {
        let req: CreateSpotCheckPolicyRequest = serde_json::from_str(json).map_err(|e| e.to_string())?;
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();

        let policy = sqlx::query_as::<_, ErpSpotCheckPolicy>(
            "INSERT INTO erp_spot_check_policies (id, location_id, min_skus_per_check, frequency_days, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, $6)
             RETURNING *"
        )
        .bind(&id)
        .bind(&req.location_id)
        .bind(req.min_skus_per_check)
        .bind(req.frequency_days)
        .bind(&now)
        .bind(&now)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        serde_json::to_string(&policy).map_err(|e| e.to_string())
    }

    // ============================================================
    // Spot Checks
    // ============================================================

    pub async fn list_spot_checks(&self, location_id: Option<&str>, status: Option<&str>, limit: i64, offset: i64) -> Result<String, String> {
        let mut query = String::from("SELECT * FROM erp_spot_checks WHERE 1=1");
        let mut param_idx = 1;

        if let Some(l) = location_id {
            query.push_str(&format!(" AND location_id = ${}", param_idx));
            param_idx += 1;
        }

        if let Some(s) = status {
            query.push_str(&format!(" AND status = ${}", param_idx));
            param_idx += 1;
        }

        let limit_p = param_idx;
        param_idx += 1;
        let offset_p = param_idx;
        query.push_str(&format!(" ORDER BY scheduled_for DESC LIMIT ${} OFFSET ${}", limit_p, offset_p));

        let checks = sqlx::query_as::<_, ErpSpotCheck>(&query)
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| e.to_string())?;

        serde_json::to_string(&checks).map_err(|e| e.to_string())
    }

    pub async fn get_spot_check(&self, id: &str) -> Result<String, String> {
        let check = sqlx::query_as::<_, ErpSpotCheck>(
            "SELECT * FROM erp_spot_checks WHERE id = $1"
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| e.to_string())?;
        serde_json::to_string(&check).map_err(|e| e.to_string())
    }

    pub async fn create_spot_check(&self, json: &str) -> Result<String, String> {
        let req: CreateSpotCheckRequest = serde_json::from_str(json).map_err(|e| e.to_string())?;
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();

        let check = sqlx::query_as::<_, ErpSpotCheck>(
            "INSERT INTO erp_spot_checks (id, policy_id, location_id, scheduled_for, created_at)
             VALUES ($1, $2, $3, $4, $5)
             RETURNING *"
        )
        .bind(&id)
        .bind(&req.policy_id)
        .bind(&req.location_id)
        .bind(&req.scheduled_for)
        .bind(&now)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        serde_json::to_string(&check).map_err(|e| e.to_string())
    }

    pub async fn start_spot_check(&self, id: &str) -> Result<String, String> {
        let now = Utc::now().to_rfc3339();

        let check = sqlx::query_as::<_, ErpSpotCheck>(
            "UPDATE erp_spot_checks SET status = 'IN_PROGRESS', started_at = $1 WHERE id = $2 RETURNING *"
        )
        .bind(&now)
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        serde_json::to_string(&check).map_err(|e| e.to_string())
    }

    pub async fn complete_spot_check(&self, id: &str) -> Result<String, String> {
        let now = Utc::now().to_rfc3339();

        let check = sqlx::query_as::<_, ErpSpotCheck>(
            "UPDATE erp_spot_checks SET status = 'COMPLETED', completed_at = $1 WHERE id = $2 RETURNING *"
        )
        .bind(&now)
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        serde_json::to_string(&check).map_err(|e| e.to_string())
    }

    // ============================================================
    // Spot Check Lines
    // ============================================================

    pub async fn list_spot_check_lines(&self, spot_check_id: &str) -> Result<String, String> {
        let lines = sqlx::query_as::<_, ErpSpotCheckLine>(
            "SELECT * FROM erp_spot_check_lines WHERE spot_check_id = $1 ORDER BY variant_id"
        )
        .bind(spot_check_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| e.to_string())?;
        serde_json::to_string(&lines).map_err(|e| e.to_string())
    }

    pub async fn create_spot_check_line(&self, json: &str) -> Result<String, String> {
        let req: CreateSpotCheckLineRequest = serde_json::from_str(json).map_err(|e| e.to_string())?;
        let id = Uuid::new_v4().to_string();

        let line = sqlx::query_as::<_, ErpSpotCheckLine>(
            "INSERT INTO erp_spot_check_lines (id, spot_check_id, variant_id, expected)
             VALUES ($1, $2, $3, $4)
             RETURNING *"
        )
        .bind(&id)
        .bind(&req.spot_check_id)
        .bind(&req.variant_id)
        .bind(req.expected)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        serde_json::to_string(&line).map_err(|e| e.to_string())
    }

    pub async fn update_spot_check_line(&self, id: &str, json: &str) -> Result<String, String> {
        let req: UpdateSpotCheckLineRequest = serde_json::from_str(json).map_err(|e| e.to_string())?;
        let variance = req.counted as i32;

        let line = sqlx::query_as::<_, ErpSpotCheckLine>(
            "UPDATE erp_spot_check_lines SET counted = $1, variance = $2 WHERE id = $3 RETURNING *"
        )
        .bind(req.counted)
        .bind(&variance)
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        serde_json::to_string(&line).map_err(|e| e.to_string())
    }

    // ============================================================
    // Variant Sales Stats
    // ============================================================

    pub async fn get_variant_sales_stat(&self, variant_id: &str) -> Result<String, String> {
        let stat = sqlx::query_as::<_, ErpVariantSalesStat>(
            "SELECT * FROM erp_variant_sales_stats WHERE variant_id = $1"
        )
        .bind(variant_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| e.to_string())?;
        serde_json::to_string(&stat).map_err(|e| e.to_string())
    }

    pub async fn list_slow_movers(&self, limit: i64, offset: i64) -> Result<String, String> {
        let stats = sqlx::query_as::<_, ErpVariantSalesStat>(
            "SELECT * FROM erp_variant_sales_stats WHERE is_slow_mover = true OR is_dead_stock = true ORDER BY units_30 ASC LIMIT $1 OFFSET $2"
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| e.to_string())?;
        serde_json::to_string(&stats).map_err(|e| e.to_string())
    }

    // ============================================================
    // Variant Forecasts
    // ============================================================

    pub async fn get_variant_forecast(&self, variant_id: &str) -> Result<String, String> {
        let forecast = sqlx::query_as::<_, ErpVariantForecast>(
            "SELECT * FROM erp_variant_forecasts WHERE variant_id = $1"
        )
        .bind(variant_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| e.to_string())?;
        serde_json::to_string(&forecast).map_err(|e| e.to_string())
    }

    pub async fn create_variant_forecast(&self, json: &str) -> Result<String, String> {
        let req: CreateVariantForecastRequest = serde_json::from_str(json).map_err(|e| e.to_string())?;
        let now = Utc::now().to_rfc3339();

        let forecast = sqlx::query_as::<_, ErpVariantForecast>(
            "INSERT INTO erp_variant_forecasts (variant_id, forecasted_daily_demand, seasonal_factor, reason_en, computed_at)
             VALUES ($1, $2, $3, $4, $5)
             ON CONFLICT (variant_id)
             DO UPDATE SET forecasted_daily_demand = $2, seasonal_factor = $3, reason_en = $4, computed_at = $5
             RETURNING *"
        )
        .bind(&req.variant_id)
        .bind(req.forecasted_daily_demand)
        .bind(req.seasonal_factor)
        .bind(&req.reason_en)
        .bind(&now)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        serde_json::to_string(&forecast).map_err(|e| e.to_string())
    }

    pub async fn list_forecasts(&self, limit: i64, offset: i64) -> Result<String, String> {
        let forecasts = sqlx::query_as::<_, ErpVariantForecast>(
            "SELECT * FROM erp_variant_forecasts ORDER BY forecasted_daily_demand DESC LIMIT $1 OFFSET $2"
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| e.to_string())?;
        serde_json::to_string(&forecasts).map_err(|e| e.to_string())
    }

    // ============================================================
    // Markdown Suggestions
    // ============================================================

    pub async fn list_markdown_suggestions(&self, status: Option<&str>, limit: i64, offset: i64) -> Result<String, String> {
        let mut query = String::from("SELECT * FROM erp_markdown_suggestions WHERE 1=1");
        let mut param_idx = 1;

        if let Some(s) = status {
            query.push_str(&format!(" AND status = ${}", param_idx));
            param_idx += 1;
        }

        let limit_p = param_idx;
        param_idx += 1;
        let offset_p = param_idx;
        query.push_str(&format!(" ORDER BY created_at DESC LIMIT ${} OFFSET ${}", limit_p, offset_p));

        let suggestions = sqlx::query_as::<_, ErpMarkdownSuggestion>(&query)
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| e.to_string())?;

        serde_json::to_string(&suggestions).map_err(|e| e.to_string())
    }

    pub async fn create_markdown_suggestion(&self, json: &str) -> Result<String, String> {
        let req: CreateMarkdownSuggestionRequest = serde_json::from_str(json).map_err(|e| e.to_string())?;
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();

        let suggestion = sqlx::query_as::<_, ErpMarkdownSuggestion>(
            "INSERT INTO erp_markdown_suggestions (id, variant_id, suggested_bps, estimated_clearance_pct, estimated_clearance_days, margin_before_paisa, margin_after_paisa, created_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
             RETURNING *"
        )
        .bind(&id)
        .bind(&req.variant_id)
        .bind(req.suggested_bps)
        .bind(req.estimated_clearance_pct)
        .bind(req.estimated_clearance_days)
        .bind(req.margin_before_paisa)
        .bind(req.margin_after_paisa)
        .bind(&now)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        serde_json::to_string(&suggestion).map_err(|e| e.to_string())
    }

    pub async fn update_markdown_suggestion_status(&self, id: &str, json: &str) -> Result<String, String> {
        let req: UpdateMarkdownSuggestionStatusRequest = serde_json::from_str(json).map_err(|e| e.to_string())?;
        let now = Utc::now().to_rfc3339();
        let resolved_at = if req.status == "CREATED" || req.status == "DISMISSED" {
            Some(now.clone())
        } else {
            None
        };

        let suggestion = sqlx::query_as::<_, ErpMarkdownSuggestion>(
            "UPDATE erp_markdown_suggestions SET status = $1, discount_id = $2, resolved_at = $3 WHERE id = $4 RETURNING *"
        )
        .bind(&req.status)
        .bind(&req.discount_id)
        .bind(&resolved_at)
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        serde_json::to_string(&suggestion).map_err(|e| e.to_string())
    }

    // ============================================================
    // Lots (Batch Tracking)
    // ============================================================

    pub async fn list_lots(&self, product_variant_id: Option<&str>, limit: i64, offset: i64) -> Result<String, String> {
        let mut query = String::from("SELECT * FROM erp_lots WHERE 1=1");
        let mut param_idx = 1;

        if let Some(v) = product_variant_id {
            query.push_str(&format!(" AND product_variant_id = ${}", param_idx));
            param_idx += 1;
        }

        let limit_p = param_idx;
        param_idx += 1;
        let offset_p = param_idx;
        query.push_str(&format!(" ORDER BY received_on DESC LIMIT ${} OFFSET ${}", limit_p, offset_p));

        let lots = sqlx::query_as::<_, ErpLot>(&query)
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| e.to_string())?;

        serde_json::to_string(&lots).map_err(|e| e.to_string())
    }

    pub async fn get_lot(&self, id: &str) -> Result<String, String> {
        let lot = sqlx::query_as::<_, ErpLot>(
            "SELECT * FROM erp_lots WHERE id = $1"
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| e.to_string())?;
        serde_json::to_string(&lot).map_err(|e| e.to_string())
    }

    pub async fn get_expiring_lots(&self, days: i32) -> Result<String, String> {
        let cutoff = (Utc::now() + chrono::Duration::days(days as i64)).to_rfc3339();

        let lots = sqlx::query_as::<_, ErpLot>(
            "SELECT * FROM erp_lots WHERE expires_on IS NOT NULL AND expires_on <= $1 ORDER BY expires_on ASC"
        )
        .bind(&cutoff)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        serde_json::to_string(&lots).map_err(|e| e.to_string())
    }

    pub async fn create_lot(&self, json: &str) -> Result<String, String> {
        let req: CreateLotRequest = serde_json::from_str(json).map_err(|e| e.to_string())?;
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();

        let lot = sqlx::query_as::<_, ErpLot>(
            "INSERT INTO erp_lots (id, product_variant_id, lot_code, expires_on, received_on, source_receipt_id, created_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7)
             RETURNING *"
        )
        .bind(&id)
        .bind(&req.product_variant_id)
        .bind(&req.lot_code)
        .bind(&req.expires_on)
        .bind(&now)
        .bind(&req.source_receipt_id)
        .bind(&now)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        serde_json::to_string(&lot).map_err(|e| e.to_string())
    }

    // ============================================================
    // Lot Stock
    // ============================================================

    pub async fn list_lot_stock(&self, lot_id: &str) -> Result<String, String> {
        let stocks = sqlx::query_as::<_, ErpLotStock>(
            "SELECT * FROM erp_lot_stock WHERE lot_id = $1 ORDER BY location_id"
        )
        .bind(lot_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| e.to_string())?;
        serde_json::to_string(&stocks).map_err(|e| e.to_string())
    }

    pub async fn create_lot_stock(&self, json: &str) -> Result<String, String> {
        let req: CreateLotStockRequest = serde_json::from_str(json).map_err(|e| e.to_string())?;
        let id = Uuid::new_v4().to_string();

        let stock = sqlx::query_as::<_, ErpLotStock>(
            "INSERT INTO erp_lot_stock (id, lot_id, location_id, quantity)
             VALUES ($1, $2, $3, $4)
             ON CONFLICT (lot_id, location_id)
             DO UPDATE SET quantity = quantity + $4
             RETURNING *"
        )
        .bind(&id)
        .bind(&req.lot_id)
        .bind(&req.location_id)
        .bind(req.quantity)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        serde_json::to_string(&stock).map_err(|e| e.to_string())
    }

    // ============================================================
    // Lot Allocations
    // ============================================================

    pub async fn list_lot_allocations(&self, lot_id: &str) -> Result<String, String> {
        let allocations = sqlx::query_as::<_, ErpLotAllocation>(
            "SELECT * FROM erp_lot_allocations WHERE lot_id = $1 ORDER BY allocated_at DESC"
        )
        .bind(lot_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| e.to_string())?;
        serde_json::to_string(&allocations).map_err(|e| e.to_string())
    }

    pub async fn create_lot_allocation(&self, json: &str) -> Result<String, String> {
        let req: CreateLotAllocationRequest = serde_json::from_str(json).map_err(|e| e.to_string())?;
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();

        let allocation = sqlx::query_as::<_, ErpLotAllocation>(
            "INSERT INTO erp_lot_allocations (id, lot_id, stock_movement_id, quantity, allocated_at)
             VALUES ($1, $2, $3, $4, $5)
             RETURNING *"
        )
        .bind(&id)
        .bind(&req.lot_id)
        .bind(&req.stock_movement_id)
        .bind(req.quantity)
        .bind(&now)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        serde_json::to_string(&allocation).map_err(|e| e.to_string())
    }

    // ============================================================
    // Serialized Units
    // ============================================================

    pub async fn list_serialized_units(&self, product_variant_id: Option<&str>, status: Option<&str>, limit: i64, offset: i64) -> Result<String, String> {
        let mut query = String::from("SELECT * FROM erp_serialized_units WHERE 1=1");
        let mut param_idx = 1;

        if let Some(v) = product_variant_id {
            query.push_str(&format!(" AND product_variant_id = ${}", param_idx));
            param_idx += 1;
        }

        if let Some(s) = status {
            query.push_str(&format!(" AND status = ${}", param_idx));
            param_idx += 1;
        }

        let limit_p = param_idx;
        param_idx += 1;
        let offset_p = param_idx;
        query.push_str(&format!(" ORDER BY created_at DESC LIMIT ${} OFFSET ${}", limit_p, offset_p));

        let units = sqlx::query_as::<_, ErpSerializedUnit>(&query)
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| e.to_string())?;

        serde_json::to_string(&units).map_err(|e| e.to_string())
    }

    pub async fn get_serialized_unit(&self, id: &str) -> Result<String, String> {
        let unit = sqlx::query_as::<_, ErpSerializedUnit>(
            "SELECT * FROM erp_serialized_units WHERE id = $1"
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| e.to_string())?;
        serde_json::to_string(&unit).map_err(|e| e.to_string())
    }

    pub async fn get_serialized_unit_by_serial(&self, serial_number: &str) -> Result<String, String> {
        let unit = sqlx::query_as::<_, ErpSerializedUnit>(
            "SELECT * FROM erp_serialized_units WHERE serial_number = $1"
        )
        .bind(serial_number)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| e.to_string())?;
        serde_json::to_string(&unit).map_err(|e| e.to_string())
    }

    pub async fn create_serialized_unit(&self, json: &str) -> Result<String, String> {
        let req: CreateSerializedUnitRequest = serde_json::from_str(json).map_err(|e| e.to_string())?;
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();

        let unit = sqlx::query_as::<_, ErpSerializedUnit>(
            "INSERT INTO erp_serialized_units (id, product_variant_id, serial_number, lot_id, location_id, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7)
             RETURNING *"
        )
        .bind(&id)
        .bind(&req.product_variant_id)
        .bind(&req.serial_number)
        .bind(&req.lot_id)
        .bind(&req.location_id)
        .bind(&now)
        .bind(&now)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        serde_json::to_string(&unit).map_err(|e| e.to_string())
    }

    pub async fn update_serialized_unit_status(&self, id: &str, json: &str) -> Result<String, String> {
        let req: UpdateSerializedUnitStatusRequest = serde_json::from_str(json).map_err(|e| e.to_string())?;
        let now = Utc::now().to_rfc3339();

        let unit = sqlx::query_as::<_, ErpSerializedUnit>(
            "UPDATE erp_serialized_units SET status = $1, order_id = COALESCE($2, order_id), location_id = COALESCE($3, location_id), updated_at = $4 WHERE id = $5 RETURNING *"
        )
        .bind(&req.status)
        .bind(&req.order_id)
        .bind(&req.location_id)
        .bind(&now)
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        serde_json::to_string(&unit).map_err(|e| e.to_string())
    }

    // ============================================================
    // Stock Counts
    // ============================================================

    pub async fn list_stock_counts(&self, status: Option<&str>, limit: i64, offset: i64) -> Result<String, String> {
        let mut query = String::from("SELECT * FROM erp_stock_counts WHERE 1=1");
        let mut param_idx = 1;

        if let Some(s) = status {
            query.push_str(&format!(" AND status = ${}", param_idx));
            param_idx += 1;
        }

        let limit_p = param_idx;
        param_idx += 1;
        let offset_p = param_idx;
        query.push_str(&format!(" ORDER BY created_at DESC LIMIT ${} OFFSET ${}", limit_p, offset_p));

        let counts = sqlx::query_as::<_, ErpStockCount>(&query)
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| e.to_string())?;

        serde_json::to_string(&counts).map_err(|e| e.to_string())
    }

    pub async fn get_stock_count(&self, id: &str) -> Result<String, String> {
        let count = sqlx::query_as::<_, ErpStockCount>(
            "SELECT * FROM erp_stock_counts WHERE id = $1"
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| e.to_string())?;
        serde_json::to_string(&count).map_err(|e| e.to_string())
    }

    pub async fn create_stock_count(&self, json: &str) -> Result<String, String> {
        let req: CreateStockCountRequest = serde_json::from_str(json).map_err(|e| e.to_string())?;
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();
        let count_number = req.count_number.clone().unwrap_or_else(|| format!("SC-{}", Uuid::new_v4().hyphenated().to_string().chars().take(8).collect::<String>()));

        let count = sqlx::query_as::<_, ErpStockCount>(
            "INSERT INTO erp_stock_counts (id, count_number, note, assigned_to, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, $6)
             RETURNING *"
        )
        .bind(&id)
        .bind(&count_number)
        .bind(&req.note)
        .bind(&req.assigned_to)
        .bind(&now)
        .bind(&now)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        serde_json::to_string(&count).map_err(|e| e.to_string())
    }

    pub async fn start_stock_count(&self, id: &str) -> Result<String, String> {
        let now = Utc::now().to_rfc3339();

        let count = sqlx::query_as::<_, ErpStockCount>(
            "UPDATE erp_stock_counts SET status = 'IN_PROGRESS', started_at = $1, updated_at = $2 WHERE id = $3 RETURNING *"
        )
        .bind(&now)
        .bind(&now)
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        serde_json::to_string(&count).map_err(|e| e.to_string())
    }

    pub async fn complete_stock_count(&self, id: &str) -> Result<String, String> {
        let now = Utc::now().to_rfc3339();

        let count = sqlx::query_as::<_, ErpStockCount>(
            "UPDATE erp_stock_counts SET status = 'COMPLETED', completed_at = $1, updated_at = $2 WHERE id = $3 RETURNING *"
        )
        .bind(&now)
        .bind(&now)
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        serde_json::to_string(&count).map_err(|e| e.to_string())
    }

    pub async fn cancel_stock_count(&self, id: &str) -> Result<String, String> {
        let now = Utc::now().to_rfc3339();

        let count = sqlx::query_as::<_, ErpStockCount>(
            "UPDATE erp_stock_counts SET status = 'CANCELLED', cancelled_at = $1, updated_at = $2 WHERE id = $3 RETURNING *"
        )
        .bind(&now)
        .bind(&now)
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        serde_json::to_string(&count).map_err(|e| e.to_string())
    }

    // ============================================================
    // Stock Count Lines
    // ============================================================

    pub async fn list_stock_count_lines(&self, stock_count_id: &str) -> Result<String, String> {
        let lines = sqlx::query_as::<_, ErpStockCountLine>(
            "SELECT * FROM erp_stock_count_lines WHERE stock_count_id = $1 ORDER BY variant_id"
        )
        .bind(stock_count_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| e.to_string())?;
        serde_json::to_string(&lines).map_err(|e| e.to_string())
    }

    pub async fn create_stock_count_line(&self, json: &str) -> Result<String, String> {
        let req: CreateStockCountLineRequest = serde_json::from_str(json).map_err(|e| e.to_string())?;
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();

        let line = sqlx::query_as::<_, ErpStockCountLine>(
            "INSERT INTO erp_stock_count_lines (id, stock_count_id, variant_id, expected, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, $6)
             ON CONFLICT (stock_count_id, variant_id)
             DO UPDATE SET expected = COALESCE($4, expected)
             RETURNING *"
        )
        .bind(&id)
        .bind(&req.stock_count_id)
        .bind(&req.variant_id)
        .bind(req.expected)
        .bind(&now)
        .bind(&now)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        serde_json::to_string(&line).map_err(|e| e.to_string())
    }

    pub async fn update_stock_count_line(&self, id: &str, json: &str) -> Result<String, String> {
        let req: UpdateStockCountLineRequest = serde_json::from_str(json).map_err(|e| e.to_string())?;
        let now = Utc::now().to_rfc3339();

        let line = sqlx::query_as::<_, ErpStockCountLine>(
            "UPDATE erp_stock_count_lines SET counted = $1, note = COALESCE($2, note), updated_at = $3 WHERE id = $4 RETURNING *"
        )
        .bind(req.counted)
        .bind(&req.note)
        .bind(&now)
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        serde_json::to_string(&line).map_err(|e| e.to_string())
    }

    // ============================================================
    // Inventory Stats
    // ============================================================

    pub async fn get_inventory_stats(&self) -> Result<String, String> {
        let locations: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM erp_locations WHERE is_active = true")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| e.to_string())?;

        let variants: (i64,) = sqlx::query_as("SELECT COUNT(DISTINCT variant_id) FROM erp_variant_location_stock WHERE quantity > 0")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| e.to_string())?;

        let low_stock: (i64,) = sqlx::query_as("SELECT COUNT(DISTINCT variant_id) FROM erp_variant_location_stock WHERE quantity > 0 AND quantity < 10")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| e.to_string())?;

        let open_variances: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM erp_stock_variances WHERE status = 'OPEN'")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| e.to_string())?;

        let pending_checks: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM erp_spot_checks WHERE status = 'PENDING'")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| e.to_string())?;

        serde_json::to_string(&serde_json::json!({
            "activeLocations": locations.0,
            "variantsWithStock": variants.0,
            "lowStockVariants": low_stock.0,
            "openVariances": open_variances.0,
            "pendingSpotChecks": pending_checks.0
        })).map_err(|e| e.to_string())
    }

    // ─── Private helpers (used by create_stock_movement) ───

    async fn get_total_stock_int(pool: &PgPool, variant_id: &str) -> Result<i32, String> {
        let row: (i32,) = sqlx::query_as(
            "SELECT COALESCE(SUM(quantity), 0) FROM erp_variant_location_stock WHERE variant_id = $1"
        )
        .bind(variant_id)
        .fetch_one(pool)
        .await
        .map_err(|e| e.to_string())?;
        Ok(row.0)
    }

    async fn update_variant_location_stock_int(pool: &PgPool, variant_id: &str, location_id: &str, quantity: i32) -> Result<(), String> {
        let now = Utc::now().to_rfc3339();
        sqlx::query(
            "INSERT INTO erp_variant_location_stock (variant_id, location_id, quantity, updated_at)
             VALUES ($1, $2, $3, $4)
             ON CONFLICT (variant_id, location_id)
             DO UPDATE SET quantity = $3, updated_at = $4"
        )
        .bind(variant_id)
        .bind(location_id)
        .bind(quantity)
        .bind(&now)
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;
        Ok(())
    }
}
