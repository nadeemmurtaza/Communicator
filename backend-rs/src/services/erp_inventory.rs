use sqlx::{Pool, Postgres};
use uuid::Uuid;
use chrono::Utc;
use crate::models::*;

pub struct ErpInventoryService;

impl ErpInventoryService {
    // ============================================================
    // Locations
    // ============================================================

    pub async fn list_locations(
        pool: &Pool<Postgres>,
        is_active: Option<bool>,
    ) -> Result<Vec<ErpLocation>, String> {
        let locations = if let Some(active) = is_active {
            sqlx::query_as::<_, ErpLocation>(
                "SELECT * FROM erp_locations WHERE is_active = $1 ORDER BY code"
            )
            .bind(active)
            .fetch_all(pool)
            .await
        } else {
            sqlx::query_as::<_, ErpLocation>(
                "SELECT * FROM erp_locations ORDER BY code"
            )
            .fetch_all(pool)
            .await
        }
        .map_err(|e| e.to_string())?;
        Ok(locations)
    }

    pub async fn get_location(pool: &Pool<Postgres>, id: &str) -> Result<ErpLocation, String> {
        let location = sqlx::query_as::<_, ErpLocation>(
            "SELECT * FROM erp_locations WHERE id = $1"
        )
        .fetch_one(pool)
        .await
        .map_err(|e| e.to_string())?;
        Ok(location)
    }

    pub async fn get_location_by_code(pool: &Pool<Postgres>, code: &str) -> Result<ErpLocation, String> {
        let location = sqlx::query_as::<_, ErpLocation>(
            "SELECT * FROM erp_locations WHERE code = $1"
        )
        .fetch_one(pool)
        .await
        .map_err(|e| e.to_string())?;
        Ok(location)
    }

    pub async fn create_location(
        pool: &Pool<Postgres>,
        req: &CreateLocationRequest,
    ) -> Result<ErpLocation, String> {
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
        .fetch_one(pool)
        .await
        .map_err(|e| e.to_string())?;

        Ok(location)
    }

    pub async fn update_location(
        pool: &Pool<Postgres>,
        id: &str,
        req: &UpdateLocationRequest,
    ) -> Result<ErpLocation, String> {
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
        .fetch_one(pool)
        .await
        .map_err(|e| e.to_string())?;

        Ok(location)
    }

    // ============================================================
    // Variant Location Stock
    // ============================================================

    pub async fn get_variant_location_stock(
        pool: &Pool<Postgres>,
        variant_id: &str,
        location_id: &str,
    ) -> Result<ErpVariantLocationStock, String> {
        let stock = sqlx::query_as::<_, ErpVariantLocationStock>(
            "SELECT * FROM erp_variant_location_stock WHERE variant_id = $1 AND location_id = $2"
        )
        .bind(variant_id)
        .bind(location_id)
        .fetch_one(pool)
        .await
        .map_err(|e| e.to_string())?;
        Ok(stock)
    }

    pub async fn get_stock_by_location(
        pool: &Pool<Postgres>,
        location_id: &str,
    ) -> Result<Vec<ErpVariantLocationStock>, String> {
        let stocks = sqlx::query_as::<_, ErpVariantLocationStock>(
            "SELECT * FROM erp_variant_location_stock WHERE location_id = $1 ORDER BY variant_id"
        )
        .bind(location_id)
        .fetch_all(pool)
        .await
        .map_err(|e| e.to_string())?;
        Ok(stocks)
    }

    pub async fn get_stock_by_variant(
        pool: &Pool<Postgres>,
        variant_id: &str,
    ) -> Result<Vec<ErpVariantLocationStock>, String> {
        let stocks = sqlx::query_as::<_, ErpVariantLocationStock>(
            "SELECT * FROM erp_variant_location_stock WHERE variant_id = $1 ORDER BY location_id"
        )
        .bind(variant_id)
        .fetch_all(pool)
        .await
        .map_err(|e| e.to_string())?;
        Ok(stocks)
    }

    pub async fn update_variant_location_stock(
        pool: &Pool<Postgres>,
        variant_id: &str,
        location_id: &str,
        quantity: i32,
    ) -> Result<ErpVariantLocationStock, String> {
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
        .fetch_one(pool)
        .await
        .map_err(|e| e.to_string())?;

        Ok(stock)
    }

    pub async fn get_total_stock(
        pool: &Pool<Postgres>,
        variant_id: &str,
    ) -> Result<i32, String> {
        let row: (i32,) = sqlx::query_as(
            "SELECT COALESCE(SUM(quantity), 0) FROM erp_variant_location_stock WHERE variant_id = $1"
        )
        .bind(variant_id)
        .fetch_one(pool)
        .await
        .map_err(|e| e.to_string())?;
        Ok(row.0)
    }

    // ============================================================
    // Stock Movements (the ledger)
    // ============================================================

    pub async fn list_stock_movements(
        pool: &Pool<Postgres>,
        variant_id: Option<&str>,
        reason: Option<&str>,
        location_id: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<ErpStockMovement>, String> {
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
            .fetch_all(pool)
            .await
            .map_err(|e| e.to_string())?;

        Ok(movements)
    }

    pub async fn get_stock_movement(pool: &Pool<Postgres>, id: &str) -> Result<ErpStockMovement, String> {
        let movement = sqlx::query_as::<_, ErpStockMovement>(
            "SELECT * FROM erp_stock_movements WHERE id = $1"
        )
        .fetch_one(pool)
        .await
        .map_err(|e| e.to_string())?;
        Ok(movement)
    }

    pub async fn get_movements_by_variant(
        pool: &Pool<Postgres>,
        variant_id: &str,
        limit: i64,
    ) -> Result<Vec<ErpStockMovement>, String> {
        let movements = sqlx::query_as::<_, ErpStockMovement>(
            "SELECT * FROM erp_stock_movements WHERE variant_id = $1 ORDER BY created_at DESC LIMIT $2"
        )
        .bind(variant_id)
        .bind(limit)
        .fetch_all(pool)
        .await
        .map_err(|e| e.to_string())?;
        Ok(movements)
    }

    pub async fn create_stock_movement(
        pool: &Pool<Postgres>,
        req: &CreateStockMovementRequest,
    ) -> Result<ErpStockMovement, String> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();

        // Calculate resulting stock
        let current_stock = Self::get_total_stock(pool, &req.variant_id).await.unwrap_or(0);
        let resulting_stock = current_stock + req.delta;

        // Update variant location stock if location specified
        if let Some(ref loc_id) = req.location_id {
            Self::update_variant_location_stock(pool, &req.variant_id, loc_id, resulting_stock).await?;
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
        .fetch_one(pool)
        .await
        .map_err(|e| e.to_string())?;

        Ok(movement)
    }

    // ============================================================
    // Stock Variances
    // ============================================================

    pub async fn list_stock_variances(
        pool: &Pool<Postgres>,
        status: Option<&str>,
        variant_id: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<ErpStockVariance>, String> {
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
            .fetch_all(pool)
            .await
            .map_err(|e| e.to_string())?;

        Ok(variances)
    }

    pub async fn get_stock_variance(pool: &Pool<Postgres>, id: &str) -> Result<ErpStockVariance, String> {
        let variance = sqlx::query_as::<_, ErpStockVariance>(
            "SELECT * FROM erp_stock_variances WHERE id = $1"
        )
        .fetch_one(pool)
        .await
        .map_err(|e| e.to_string())?;
        Ok(variance)
    }

    pub async fn create_stock_variance(
        pool: &Pool<Postgres>,
        req: &CreateStockVarianceRequest,
    ) -> Result<ErpStockVariance, String> {
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
        .fetch_one(pool)
        .await
        .map_err(|e| e.to_string())?;

        Ok(variance)
    }

    pub async fn resolve_variance(
        pool: &Pool<Postgres>,
        id: &str,
        req: &ResolveVarianceRequest,
    ) -> Result<ErpStockVariance, String> {
        let now = Utc::now().to_rfc3339();

        let variance = sqlx::query_as::<_, ErpStockVariance>(
            "UPDATE erp_stock_variances SET status = 'RESOLVED', resolved_by_user_id = $1, resolved_at = $2, resolution_note = $3 WHERE id = $4 RETURNING *"
        )
        .bind(&req.resolved_by_user_id)
        .bind(&now)
        .bind(&req.resolution_note)
        .bind(id)
        .fetch_one(pool)
        .await
        .map_err(|e| e.to_string())?;

        Ok(variance)
    }

    // ============================================================
    // Spot Check Policies
    // ============================================================

    pub async fn list_spot_check_policies(
        pool: &Pool<Postgres>,
        is_active: Option<bool>,
    ) -> Result<Vec<ErpSpotCheckPolicy>, String> {
        let policies = if let Some(active) = is_active {
            sqlx::query_as::<_, ErpSpotCheckPolicy>(
                "SELECT * FROM erp_spot_check_policies WHERE is_active = $1 ORDER BY id"
            )
            .bind(active)
            .fetch_all(pool)
            .await
        } else {
            sqlx::query_as::<_, ErpSpotCheckPolicy>(
                "SELECT * FROM erp_spot_check_policies ORDER BY id"
            )
            .fetch_all(pool)
            .await
        }
        .map_err(|e| e.to_string())?;
        Ok(policies)
    }

    pub async fn create_spot_check_policy(
        pool: &Pool<Postgres>,
        req: &CreateSpotCheckPolicyRequest,
    ) -> Result<ErpSpotCheckPolicy, String> {
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
        .fetch_one(pool)
        .await
        .map_err(|e| e.to_string())?;

        Ok(policy)
    }

    // ============================================================
    // Spot Checks
    // ============================================================

    pub async fn list_spot_checks(
        pool: &Pool<Postgres>,
        location_id: Option<&str>,
        status: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<ErpSpotCheck>, String> {
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
            .fetch_all(pool)
            .await
            .map_err(|e| e.to_string())?;

        Ok(checks)
    }

    pub async fn get_spot_check(pool: &Pool<Postgres>, id: &str) -> Result<ErpSpotCheck, String> {
        let check = sqlx::query_as::<_, ErpSpotCheck>(
            "SELECT * FROM erp_spot_checks WHERE id = $1"
        )
        .fetch_one(pool)
        .await
        .map_err(|e| e.to_string())?;
        Ok(check)
    }

    pub async fn create_spot_check(
        pool: &Pool<Postgres>,
        req: &CreateSpotCheckRequest,
    ) -> Result<ErpSpotCheck, String> {
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
        .fetch_one(pool)
        .await
        .map_err(|e| e.to_string())?;

        Ok(check)
    }

    pub async fn start_spot_check(pool: &Pool<Postgres>, id: &str) -> Result<ErpSpotCheck, String> {
        let now = Utc::now().to_rfc3339();

        let check = sqlx::query_as::<_, ErpSpotCheck>(
            "UPDATE erp_spot_checks SET status = 'IN_PROGRESS', started_at = $1 WHERE id = $2 RETURNING *"
        )
        .bind(&now)
        .bind(id)
        .fetch_one(pool)
        .await
        .map_err(|e| e.to_string())?;

        Ok(check)
    }

    pub async fn complete_spot_check(pool: &Pool<Postgres>, id: &str) -> Result<ErpSpotCheck, String> {
        let now = Utc::now().to_rfc3339();

        let check = sqlx::query_as::<_, ErpSpotCheck>(
            "UPDATE erp_spot_checks SET status = 'COMPLETED', completed_at = $1 WHERE id = $2 RETURNING *"
        )
        .bind(&now)
        .bind(id)
        .fetch_one(pool)
        .await
        .map_err(|e| e.to_string())?;

        Ok(check)
    }

    // ============================================================
    // Spot Check Lines
    // ============================================================

    pub async fn list_spot_check_lines(
        pool: &Pool<Postgres>,
        spot_check_id: &str,
    ) -> Result<Vec<ErpSpotCheckLine>, String> {
        let lines = sqlx::query_as::<_, ErpSpotCheckLine>(
            "SELECT * FROM erp_spot_check_lines WHERE spot_check_id = $1 ORDER BY variant_id"
        )
        .bind(spot_check_id)
        .fetch_all(pool)
        .await
        .map_err(|e| e.to_string())?;
        Ok(lines)
    }

    pub async fn create_spot_check_line(
        pool: &Pool<Postgres>,
        req: &CreateSpotCheckLineRequest,
    ) -> Result<ErpSpotCheckLine, String> {
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
        .fetch_one(pool)
        .await
        .map_err(|e| e.to_string())?;

        Ok(line)
    }

    pub async fn update_spot_check_line(
        pool: &Pool<Postgres>,
        id: &str,
        req: &UpdateSpotCheckLineRequest,
    ) -> Result<ErpSpotCheckLine, String> {
        let variance = req.counted as i32; // variance will be computed

        let line = sqlx::query_as::<_, ErpSpotCheckLine>(
            "UPDATE erp_spot_check_lines SET counted = $1, variance = $2 WHERE id = $3 RETURNING *"
        )
        .bind(req.counted)
        .bind(&variance)
        .bind(id)
        .fetch_one(pool)
        .await
        .map_err(|e| e.to_string())?;

        Ok(line)
    }

    // ============================================================
    // Variant Sales Stats
    // ============================================================

    pub async fn get_variant_sales_stat(
        pool: &Pool<Postgres>,
        variant_id: &str,
    ) -> Result<ErpVariantSalesStat, String> {
        let stat = sqlx::query_as::<_, ErpVariantSalesStat>(
            "SELECT * FROM erp_variant_sales_stats WHERE variant_id = $1"
        )
        .fetch_one(pool)
        .await
        .map_err(|e| e.to_string())?;
        Ok(stat)
    }

    pub async fn list_slow_movers(
        pool: &Pool<Postgres>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<ErpVariantSalesStat>, String> {
        let stats = sqlx::query_as::<_, ErpVariantSalesStat>(
            "SELECT * FROM erp_variant_sales_stats WHERE is_slow_mover = true OR is_dead_stock = true ORDER BY units_30 ASC LIMIT $1 OFFSET $2"
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await
        .map_err(|e| e.to_string())?;
        Ok(stats)
    }

    // ============================================================
    // Variant Forecasts
    // ============================================================

    pub async fn get_variant_forecast(
        pool: &Pool<Postgres>,
        variant_id: &str,
    ) -> Result<ErpVariantForecast, String> {
        let forecast = sqlx::query_as::<_, ErpVariantForecast>(
            "SELECT * FROM erp_variant_forecasts WHERE variant_id = $1"
        )
        .fetch_one(pool)
        .await
        .map_err(|e| e.to_string())?;
        Ok(forecast)
    }

    pub async fn create_variant_forecast(
        pool: &Pool<Postgres>,
        req: &CreateVariantForecastRequest,
    ) -> Result<ErpVariantForecast, String> {
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
        .fetch_one(pool)
        .await
        .map_err(|e| e.to_string())?;

        Ok(forecast)
    }

    pub async fn list_forecasts(
        pool: &Pool<Postgres>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<ErpVariantForecast>, String> {
        let forecasts = sqlx::query_as::<_, ErpVariantForecast>(
            "SELECT * FROM erp_variant_forecasts ORDER BY forecasted_daily_demand DESC LIMIT $1 OFFSET $2"
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await
        .map_err(|e| e.to_string())?;
        Ok(forecasts)
    }

    // ============================================================
    // Markdown Suggestions
    // ============================================================

    pub async fn list_markdown_suggestions(
        pool: &Pool<Postgres>,
        status: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<ErpMarkdownSuggestion>, String> {
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
            .fetch_all(pool)
            .await
            .map_err(|e| e.to_string())?;

        Ok(suggestions)
    }

    pub async fn create_markdown_suggestion(
        pool: &Pool<Postgres>,
        req: &CreateMarkdownSuggestionRequest,
    ) -> Result<ErpMarkdownSuggestion, String> {
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
        .fetch_one(pool)
        .await
        .map_err(|e| e.to_string())?;

        Ok(suggestion)
    }

    pub async fn update_markdown_suggestion_status(
        pool: &Pool<Postgres>,
        id: &str,
        req: &UpdateMarkdownSuggestionStatusRequest,
    ) -> Result<ErpMarkdownSuggestion, String> {
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
        .fetch_one(pool)
        .await
        .map_err(|e| e.to_string())?;

        Ok(suggestion)
    }

    // ============================================================
    // Lots (Batch Tracking)
    // ============================================================

    pub async fn list_lots(
        pool: &Pool<Postgres>,
        product_variant_id: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<ErpLot>, String> {
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
            .fetch_all(pool)
            .await
            .map_err(|e| e.to_string())?;

        Ok(lots)
    }

    pub async fn get_lot(pool: &Pool<Postgres>, id: &str) -> Result<ErpLot, String> {
        let lot = sqlx::query_as::<_, ErpLot>(
            "SELECT * FROM erp_lots WHERE id = $1"
        )
        .fetch_one(pool)
        .await
        .map_err(|e| e.to_string())?;
        Ok(lot)
    }

    pub async fn get_expiring_lots(
        pool: &Pool<Postgres>,
        days: i32,
    ) -> Result<Vec<ErpLot>, String> {
        let cutoff = (Utc::now() + chrono::Duration::days(days as i64)).to_rfc3339();

        let lots = sqlx::query_as::<_, ErpLot>(
            "SELECT * FROM erp_lots WHERE expires_on IS NOT NULL AND expires_on <= $1 ORDER BY expires_on ASC"
        )
        .bind(&cutoff)
        .fetch_all(pool)
        .await
        .map_err(|e| e.to_string())?;

        Ok(lots)
    }

    pub async fn create_lot(
        pool: &Pool<Postgres>,
        req: &CreateLotRequest,
    ) -> Result<ErpLot, String> {
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
        .fetch_one(pool)
        .await
        .map_err(|e| e.to_string())?;

        Ok(lot)
    }

    // ============================================================
    // Lot Stock
    // ============================================================

    pub async fn list_lot_stock(
        pool: &Pool<Postgres>,
        lot_id: &str,
    ) -> Result<Vec<ErpLotStock>, String> {
        let stocks = sqlx::query_as::<_, ErpLotStock>(
            "SELECT * FROM erp_lot_stock WHERE lot_id = $1 ORDER BY location_id"
        )
        .bind(lot_id)
        .fetch_all(pool)
        .await
        .map_err(|e| e.to_string())?;
        Ok(stocks)
    }

    pub async fn create_lot_stock(
        pool: &Pool<Postgres>,
        req: &CreateLotStockRequest,
    ) -> Result<ErpLotStock, String> {
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
        .fetch_one(pool)
        .await
        .map_err(|e| e.to_string())?;

        Ok(stock)
    }

    // ============================================================
    // Lot Allocations
    // ============================================================

    pub async fn list_lot_allocations(
        pool: &Pool<Postgres>,
        lot_id: &str,
    ) -> Result<Vec<ErpLotAllocation>, String> {
        let allocations = sqlx::query_as::<_, ErpLotAllocation>(
            "SELECT * FROM erp_lot_allocations WHERE lot_id = $1 ORDER BY allocated_at DESC"
        )
        .bind(lot_id)
        .fetch_all(pool)
        .await
        .map_err(|e| e.to_string())?;
        Ok(allocations)
    }

    pub async fn create_lot_allocation(
        pool: &Pool<Postgres>,
        req: &CreateLotAllocationRequest,
    ) -> Result<ErpLotAllocation, String> {
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
        .fetch_one(pool)
        .await
        .map_err(|e| e.to_string())?;

        Ok(allocation)
    }

    // ============================================================
    // Serialized Units
    // ============================================================

    pub async fn list_serialized_units(
        pool: &Pool<Postgres>,
        product_variant_id: Option<&str>,
        status: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<ErpSerializedUnit>, String> {
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
            .fetch_all(pool)
            .await
            .map_err(|e| e.to_string())?;

        Ok(units)
    }

    pub async fn get_serialized_unit(pool: &Pool<Postgres>, id: &str) -> Result<ErpSerializedUnit, String> {
        let unit = sqlx::query_as::<_, ErpSerializedUnit>(
            "SELECT * FROM erp_serialized_units WHERE id = $1"
        )
        .fetch_one(pool)
        .await
        .map_err(|e| e.to_string())?;
        Ok(unit)
    }

    pub async fn get_serialized_unit_by_serial(
        pool: &Pool<Postgres>,
        serial_number: &str,
    ) -> Result<ErpSerializedUnit, String> {
        let unit = sqlx::query_as::<_, ErpSerializedUnit>(
            "SELECT * FROM erp_serialized_units WHERE serial_number = $1"
        )
        .fetch_one(pool)
        .await
        .map_err(|e| e.to_string())?;
        Ok(unit)
    }

    pub async fn create_serialized_unit(
        pool: &Pool<Postgres>,
        req: &CreateSerializedUnitRequest,
    ) -> Result<ErpSerializedUnit, String> {
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
        .fetch_one(pool)
        .await
        .map_err(|e| e.to_string())?;

        Ok(unit)
    }

    pub async fn update_serialized_unit_status(
        pool: &Pool<Postgres>,
        id: &str,
        req: &UpdateSerializedUnitStatusRequest,
    ) -> Result<ErpSerializedUnit, String> {
        let now = Utc::now().to_rfc3339();

        let unit = sqlx::query_as::<_, ErpSerializedUnit>(
            "UPDATE erp_serialized_units SET status = $1, order_id = COALESCE($2, order_id), location_id = COALESCE($3, location_id), updated_at = $4 WHERE id = $5 RETURNING *"
        )
        .bind(&req.status)
        .bind(&req.order_id)
        .bind(&req.location_id)
        .bind(&now)
        .bind(id)
        .fetch_one(pool)
        .await
        .map_err(|e| e.to_string())?;

        Ok(unit)
    }

    // ============================================================
    // Stock Counts
    // ============================================================

    pub async fn list_stock_counts(
        pool: &Pool<Postgres>,
        status: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<ErpStockCount>, String> {
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
            .fetch_all(pool)
            .await
            .map_err(|e| e.to_string())?;

        Ok(counts)
    }

    pub async fn get_stock_count(pool: &Pool<Postgres>, id: &str) -> Result<ErpStockCount, String> {
        let count = sqlx::query_as::<_, ErpStockCount>(
            "SELECT * FROM erp_stock_counts WHERE id = $1"
        )
        .fetch_one(pool)
        .await
        .map_err(|e| e.to_string())?;
        Ok(count)
    }

    pub async fn create_stock_count(
        pool: &Pool<Postgres>,
        req: &CreateStockCountRequest,
    ) -> Result<ErpStockCount, String> {
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
        .fetch_one(pool)
        .await
        .map_err(|e| e.to_string())?;

        Ok(count)
    }

    pub async fn start_stock_count(pool: &Pool<Postgres>, id: &str) -> Result<ErpStockCount, String> {
        let now = Utc::now().to_rfc3339();

        let count = sqlx::query_as::<_, ErpStockCount>(
            "UPDATE erp_stock_counts SET status = 'IN_PROGRESS', started_at = $1, updated_at = $2 WHERE id = $3 RETURNING *"
        )
        .bind(&now)
        .bind(&now)
        .bind(id)
        .fetch_one(pool)
        .await
        .map_err(|e| e.to_string())?;

        Ok(count)
    }

    pub async fn complete_stock_count(pool: &Pool<Postgres>, id: &str) -> Result<ErpStockCount, String> {
        let now = Utc::now().to_rfc3339();

        let count = sqlx::query_as::<_, ErpStockCount>(
            "UPDATE erp_stock_counts SET status = 'COMPLETED', completed_at = $1, updated_at = $2 WHERE id = $3 RETURNING *"
        )
        .bind(&now)
        .bind(&now)
        .bind(id)
        .fetch_one(pool)
        .await
        .map_err(|e| e.to_string())?;

        Ok(count)
    }

    pub async fn cancel_stock_count(pool: &Pool<Postgres>, id: &str) -> Result<ErpStockCount, String> {
        let now = Utc::now().to_rfc3339();

        let count = sqlx::query_as::<_, ErpStockCount>(
            "UPDATE erp_stock_counts SET status = 'CANCELLED', cancelled_at = $1, updated_at = $2 WHERE id = $3 RETURNING *"
        )
        .bind(&now)
        .bind(&now)
        .bind(id)
        .fetch_one(pool)
        .await
        .map_err(|e| e.to_string())?;

        Ok(count)
    }

    // ============================================================
    // Stock Count Lines
    // ============================================================

    pub async fn list_stock_count_lines(
        pool: &Pool<Postgres>,
        stock_count_id: &str,
    ) -> Result<Vec<ErpStockCountLine>, String> {
        let lines = sqlx::query_as::<_, ErpStockCountLine>(
            "SELECT * FROM erp_stock_count_lines WHERE stock_count_id = $1 ORDER BY variant_id"
        )
        .bind(stock_count_id)
        .fetch_all(pool)
        .await
        .map_err(|e| e.to_string())?;
        Ok(lines)
    }

    pub async fn create_stock_count_line(
        pool: &Pool<Postgres>,
        req: &CreateStockCountLineRequest,
    ) -> Result<ErpStockCountLine, String> {
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
        .fetch_one(pool)
        .await
        .map_err(|e| e.to_string())?;

        Ok(line)
    }

    pub async fn update_stock_count_line(
        pool: &Pool<Postgres>,
        id: &str,
        req: &UpdateStockCountLineRequest,
    ) -> Result<ErpStockCountLine, String> {
        let now = Utc::now().to_rfc3339();

        let line = sqlx::query_as::<_, ErpStockCountLine>(
            "UPDATE erp_stock_count_lines SET counted = $1, note = COALESCE($2, note), updated_at = $3 WHERE id = $4 RETURNING *"
        )
        .bind(req.counted)
        .bind(&req.note)
        .bind(&now)
        .bind(id)
        .fetch_one(pool)
        .await
        .map_err(|e| e.to_string())?;

        Ok(line)
    }

    // ============================================================
    // Inventory Stats
    // ============================================================

    pub async fn get_inventory_stats(pool: &Pool<Postgres>) -> Result<serde_json::Value, String> {
        // Total locations
        let locations: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM erp_locations WHERE is_active = true")
            .fetch_one(pool)
            .await
            .map_err(|e| e.to_string())?;

        // Total variants with stock
        let variants: (i64,) = sqlx::query_as("SELECT COUNT(DISTINCT variant_id) FROM erp_variant_location_stock WHERE quantity > 0")
            .fetch_one(pool)
            .await
            .map_err(|e| e.to_string())?;

        // Low stock variants
        let low_stock: (i64,) = sqlx::query_as("SELECT COUNT(DISTINCT variant_id) FROM erp_variant_location_stock WHERE quantity > 0 AND quantity < 10")
            .fetch_one(pool)
            .await
            .map_err(|e| e.to_string())?;

        // Open variances
        let open_variances: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM erp_stock_variances WHERE status = 'OPEN'")
            .fetch_one(pool)
            .await
            .map_err(|e| e.to_string())?;

        // Pending spot checks
        let pending_checks: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM erp_spot_checks WHERE status = 'PENDING'")
            .fetch_one(pool)
            .await
            .map_err(|e| e.to_string())?;

        Ok(serde_json::json!({
            "activeLocations": locations.0,
            "variantsWithStock": variants.0,
            "lowStockVariants": low_stock.0,
            "openVariances": open_variances.0,
            "pendingSpotChecks": pending_checks.0
        }))
    }
}
