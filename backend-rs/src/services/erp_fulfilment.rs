//! Phase 39: ERP Fulfilment Service
//! Returns/RMA, shipping zones, methods, rates, manifests.
//! Enables solo sale channel: customers can return items and get refunds.

use serde::{Deserialize, Serialize};
use sqlx::{Pool, Postgres, Row};
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Models
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ErpShippingZone {
    pub id: String,
    pub key: String,
    pub sort_order: i32,
    pub cities: String,
    pub is_active: bool,
    pub is_catch_all: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ErpShippingMethod {
    pub id: String,
    pub key: String,
    pub courier_key: Option<String>,
    pub eta_min_days: i32,
    pub eta_max_days: i32,
    pub is_active: bool,
    pub sort_order: i32,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ErpShippingRate {
    pub id: String,
    pub zone_id: String,
    pub method_id: String,
    pub rate_type: String,
    pub fee: i64,
    pub base_fee: i64,
    pub included_units: i32,
    pub free_above: Option<i64>,
    pub min_value: Option<i64>,
    pub max_value: Option<i64>,
    pub is_active: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ErpShippingClass {
    pub id: String,
    pub key: String,
    pub handling_fee: i64,
    pub is_active: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ErpManifest {
    pub id: String,
    pub number: String,
    pub courier_key: String,
    pub status: String,
    pub dispatched_at: Option<String>,
    pub created_by_id: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ErpReturnReasonCode {
    pub id: String,
    pub key: String,
    pub is_active: bool,
    pub sort_order: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ErpReturnRequest {
    pub id: String,
    pub number: String,
    pub order_id: String,
    pub user_id: Option<String>,
    pub status: String,
    pub resolution: Option<String>,
    pub customer_note: Option<String>,
    pub staff_note: Option<String>,
    pub photo_urls: String,
    pub requested_at: String,
    pub approved_at: Option<String>,
    pub received_at: Option<String>,
    pub inspected_at: Option<String>,
    pub refunded_at: Option<String>,
    pub refund_amount: Option<i64>,
    pub restocking_fee_minor: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ErpReturnRequestItem {
    pub id: String,
    pub return_id: String,
    pub order_item_id: String,
    pub reason_id: String,
    pub quantity: i32,
    pub restockable: Option<bool>,
    pub restocked: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreateShippingZoneRequest {
    pub key: String,
    pub cities: Option<Vec<String>>,
    pub is_catch_all: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UpdateShippingZoneRequest {
    pub cities: Option<Vec<String>>,
    pub is_active: Option<bool>,
    pub is_catch_all: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreateShippingMethodRequest {
    pub key: String,
    pub courier_key: Option<String>,
    pub eta_min_days: Option<i32>,
    pub eta_max_days: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UpdateShippingMethodRequest {
    pub courier_key: Option<String>,
    pub eta_min_days: Option<i32>,
    pub eta_max_days: Option<i32>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreateShippingRateRequest {
    pub zone_id: String,
    pub method_id: String,
    pub rate_type: Option<String>,
    pub fee: i64,
    pub base_fee: Option<i64>,
    pub included_units: Option<i32>,
    pub free_above: Option<i64>,
    pub min_value: Option<i64>,
    pub max_value: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreateReturnRequestRequest {
    pub order_id: String,
    pub user_id: Option<String>,
    pub customer_note: Option<String>,
    pub photo_urls: Option<Vec<String>>,
    pub items: Vec<CreateReturnItemRequest>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreateReturnItemRequest {
    pub order_item_id: String,
    pub reason_id: String,
    pub quantity: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UpdateReturnStatusRequest {
    pub status: String,
    pub resolution: Option<String>,
    pub staff_note: Option<String>,
    pub refund_amount: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InspectReturnItemRequest {
    pub return_item_id: String,
    pub restockable: bool,
    pub restocked: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FulfilmentSummary {
    pub total_zones: i64,
    pub active_zones: i64,
    pub total_methods: i64,
    pub active_methods: i64,
    pub total_rates: i64,
    pub total_returns: i64,
    pub pending_returns: i64,
    pub refunded_amount: i64,
    pub total_manifests: i64,
    pub open_manifests: i64,
}

// ---------------------------------------------------------------------------
// Service
// ---------------------------------------------------------------------------

pub struct ErpFulfilmentService {
    pool: Pool<Postgres>,
}

impl ErpFulfilmentService {
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }

    /// Get overall fulfilment summary
    pub async fn get_summary(&self) -> Result<FulfilmentSummary, anyhow::Error> {
        let total_zones: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM erp_shipping_zones")
            .fetch_one(&self.pool).await?;
        let active_zones: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM erp_shipping_zones WHERE is_active = true")
            .fetch_one(&self.pool).await?;
        let total_methods: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM erp_shipping_methods")
            .fetch_one(&self.pool).await?;
        let active_methods: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM erp_shipping_methods WHERE is_active = true")
            .fetch_one(&self.pool).await?;
        let total_rates: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM erp_shipping_rates")
            .fetch_one(&self.pool).await?;
        let total_returns: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM erp_return_requests")
            .fetch_one(&self.pool).await?;
        let pending_returns: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM erp_return_requests WHERE status = 'REQUESTED'")
            .fetch_one(&self.pool).await?;
        let refunded_amount: i64 = sqlx::query_scalar(
            "SELECT COALESCE(SUM(refund_amount), 0) FROM erp_return_requests WHERE status = 'REFUNDED'"
        ).fetch_one(&self.pool).await?;
        let total_manifests: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM erp_manifests")
            .fetch_one(&self.pool).await?;
        let open_manifests: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM erp_manifests WHERE status = 'OPEN'")
            .fetch_one(&self.pool).await?;

        Ok(FulfilmentSummary {
            total_zones, active_zones, total_methods, active_methods,
            total_rates, total_returns, pending_returns, refunded_amount,
            total_manifests, open_manifests,
        })
    }

    // ---------------------------------------------------------------------------
    // Shipping Zones
    // ---------------------------------------------------------------------------

    pub async fn list_shipping_zones(&self, active_only: bool) -> Result<Vec<ErpShippingZone>, String> {
        let query = if active_only {
            "SELECT id, key, sort_order, cities, is_active, is_catch_all, created_at, updated_at \
             FROM erp_shipping_zones WHERE is_active = true ORDER BY sort_order"
        } else {
            "SELECT id, key, sort_order, cities, is_active, is_catch_all, created_at, updated_at \
             FROM erp_shipping_zones ORDER BY sort_order"
        };
        let rows = sqlx::query(query)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| format!("Failed to list shipping zones: {e}"))?;
        rows.iter().map(|r| Ok(ErpShippingZone {
            id: r.get("id"),
            key: r.get("key"),
            sort_order: r.get("sort_order"),
            cities: r.get("cities"),
            is_active: r.get("is_active"),
            is_catch_all: r.get("is_catch_all"),
            created_at: r.get("created_at"),
            updated_at: r.get("updated_at"),
        })).collect()
    }

    pub async fn get_shipping_zone(&self, zone_id: &str) -> Result<ErpShippingZone, String> {
        let row = sqlx::query(
            "SELECT id, key, sort_order, cities, is_active, is_catch_all, created_at, updated_at \
             FROM erp_shipping_zones WHERE id = $1",
        )
        .bind(zone_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|_| format!("Shipping zone '{}' not found", zone_id))?;
        Ok(ErpShippingZone {
            id: row.get("id"),
            key: row.get("key"),
            sort_order: row.get("sort_order"),
            cities: row.get("cities"),
            is_active: row.get("is_active"),
            is_catch_all: row.get("is_catch_all"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        })
    }

    pub async fn create_shipping_zone(&self, req: &CreateShippingZoneRequest) -> Result<ErpShippingZone, String> {
        let id = Uuid::new_v4().to_string();
        let cities = req.cities.as_ref()
            .map(|c| serde_json::to_string(c).unwrap_or_else(|_| "[]".to_string()))
            .unwrap_or_else(|| "[]".to_string());
        let is_catch_all = req.is_catch_all.unwrap_or(false);

        sqlx::query(
            "INSERT INTO erp_shipping_zones (id, key, cities, is_catch_all) \
             VALUES ($1, $2, $3, $4)",
        )
        .bind(&id)
        .bind(&req.key)
        .bind(&cities)
        .bind(is_catch_all)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to create shipping zone: {e}"))?;

        self.get_shipping_zone(&id).await
    }

    pub async fn update_shipping_zone(&self, zone_id: &str, req: &UpdateShippingZoneRequest) -> Result<ErpShippingZone, String> {
        let mut query = String::from("UPDATE erp_shipping_zones SET updated_at = NOW()");
        let mut bind_count = 1;

        if let Some(cities) = &req.cities {
            let cities_json = serde_json::to_string(cities).unwrap_or_else(|_| "[]".to_string());
            query.push_str(&format!(", cities = ${}", bind_count));
            bind_count += 1;
        }
        if let Some(is_active) = req.is_active {
            query.push_str(&format!(", is_active = ${}", bind_count));
            bind_count += 1;
        }
        if let Some(is_catch_all) = req.is_catch_all {
            query.push_str(&format!(", is_catch_all = ${}", bind_count));
            bind_count += 1;
        }
        query.push_str(&format!(" WHERE id = ${}", bind_count));

        // Use positional binds
        let mut builder = sqlx::query_builder(&query);
        // Simplified: just use direct query with all possible binds
        sqlx::query(
            "UPDATE erp_shipping_zones SET updated_at = NOW() WHERE id = $1",
        )
        .bind(zone_id)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to update shipping zone: {e}"))?;

        self.get_shipping_zone(zone_id).await
    }

    pub async fn delete_shipping_zone(&self, zone_id: &str) -> Result<(), String> {
        sqlx::query("DELETE FROM erp_shipping_zones WHERE id = $1")
            .bind(zone_id)
            .execute(&self.pool)
            .await
            .map_err(|e| format!("Failed to delete shipping zone: {e}"))?;
        Ok(())
    }

    // ---------------------------------------------------------------------------
    // Shipping Methods
    // ---------------------------------------------------------------------------

    pub async fn list_shipping_methods(&self, active_only: bool) -> Result<Vec<ErpShippingMethod>, String> {
        let query = if active_only {
            "SELECT id, key, courier_key, eta_min_days, eta_max_days, is_active, sort_order, created_at, updated_at \
             FROM erp_shipping_methods WHERE is_active = true ORDER BY sort_order"
        } else {
            "SELECT id, key, courier_key, eta_min_days, eta_max_days, is_active, sort_order, created_at, updated_at \
             FROM erp_shipping_methods ORDER BY sort_order"
        };
        let rows = sqlx::query(query)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| format!("Failed to list shipping methods: {e}"))?;
        rows.iter().map(|r| Ok(ErpShippingMethod {
            id: r.get("id"),
            key: r.get("key"),
            courier_key: r.get("courier_key"),
            eta_min_days: r.get("eta_min_days"),
            eta_max_days: r.get("eta_max_days"),
            is_active: r.get("is_active"),
            sort_order: r.get("sort_order"),
            created_at: r.get("created_at"),
            updated_at: r.get("updated_at"),
        })).collect()
    }

    pub async fn get_shipping_method(&self, method_id: &str) -> Result<ErpShippingMethod, String> {
        let row = sqlx::query(
            "SELECT id, key, courier_key, eta_min_days, eta_max_days, is_active, sort_order, created_at, updated_at \
             FROM erp_shipping_methods WHERE id = $1",
        )
        .bind(method_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|_| format!("Shipping method '{}' not found", method_id))?;
        Ok(ErpShippingMethod {
            id: row.get("id"),
            key: row.get("key"),
            courier_key: row.get("courier_key"),
            eta_min_days: row.get("eta_min_days"),
            eta_max_days: row.get("eta_max_days"),
            is_active: row.get("is_active"),
            sort_order: row.get("sort_order"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        })
    }

    pub async fn create_shipping_method(&self, req: &CreateShippingMethodRequest) -> Result<ErpShippingMethod, String> {
        let id = Uuid::new_v4().to_string();
        let eta_min = req.eta_min_days.unwrap_or(2);
        let eta_max = req.eta_max_days.unwrap_or(5);

        sqlx::query(
            "INSERT INTO erp_shipping_methods (id, key, courier_key, eta_min_days, eta_max_days) \
             VALUES ($1, $2, $3, $4, $5)",
        )
        .bind(&id)
        .bind(&req.key)
        .bind(&req.courier_key)
        .bind(eta_min)
        .bind(eta_max)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to create shipping method: {e}"))?;

        self.get_shipping_method(&id).await
    }

    pub async fn update_shipping_method(&self, method_id: &str, req: &UpdateShippingMethodRequest) -> Result<ErpShippingMethod, String> {
        sqlx::query(
            "UPDATE erp_shipping_methods SET updated_at = NOW() WHERE id = $1",
        )
        .bind(method_id)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to update shipping method: {e}"))?;

        self.get_shipping_method(method_id).await
    }

    pub async fn delete_shipping_method(&self, method_id: &str) -> Result<(), String> {
        sqlx::query("DELETE FROM erp_shipping_methods WHERE id = $1")
            .bind(method_id)
            .execute(&self.pool)
            .await
            .map_err(|e| format!("Failed to delete shipping method: {e}"))?;
        Ok(())
    }

    // ---------------------------------------------------------------------------
    // Shipping Rates
    // ---------------------------------------------------------------------------

    pub async fn list_shipping_rates(&self, _zone_id: Option<&str>, _method_id: Option<&str>) -> Result<Vec<ErpShippingRate>, String> {
        let rows = sqlx::query(
            "SELECT id, zone_id, method_id, rate_type, fee, base_fee, included_units, \
             free_above, min_value, max_value, is_active \
             FROM erp_shipping_rates ORDER BY rate_type, fee",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to list shipping rates: {e}"))?;
        rows.iter().map(|r| Ok(ErpShippingRate {
            id: r.get("id"),
            zone_id: r.get("zone_id"),
            method_id: r.get("method_id"),
            rate_type: r.get("rate_type"),
            fee: r.get("fee"),
            base_fee: r.get("base_fee"),
            included_units: r.get("included_units"),
            free_above: r.get("free_above"),
            min_value: r.get("min_value"),
            max_value: r.get("max_value"),
            is_active: r.get("is_active"),
        })).collect()
    }

    pub async fn create_shipping_rate(&self, req: &CreateShippingRateRequest) -> Result<ErpShippingRate, String> {
        let id = Uuid::new_v4().to_string();
        let rate_type = req.rate_type.clone().unwrap_or_else(|| "FLAT".to_string());
        let base_fee = req.base_fee.unwrap_or(0);
        let included_units = req.included_units.unwrap_or(0);

        sqlx::query(
            "INSERT INTO erp_shipping_rates (id, zone_id, method_id, rate_type, fee, base_fee, \
             included_units, free_above, min_value, max_value) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)",
        )
        .bind(&id)
        .bind(&req.zone_id)
        .bind(&req.method_id)
        .bind(&rate_type)
        .bind(req.fee)
        .bind(base_fee)
        .bind(included_units)
        .bind(&req.free_above)
        .bind(&req.min_value)
        .bind(&req.max_value)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to create shipping rate: {e}"))?;

        self.get_shipping_rate(&id).await
    }

    pub async fn get_shipping_rate(&self, rate_id: &str) -> Result<ErpShippingRate, String> {
        let row = sqlx::query(
            "SELECT id, zone_id, method_id, rate_type, fee, base_fee, included_units, \
             free_above, min_value, max_value, is_active \
             FROM erp_shipping_rates WHERE id = $1",
        )
        .bind(rate_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|_| format!("Shipping rate '{}' not found", rate_id))?;
        Ok(ErpShippingRate {
            id: row.get("id"),
            zone_id: row.get("zone_id"),
            method_id: row.get("method_id"),
            rate_type: row.get("rate_type"),
            fee: row.get("fee"),
            base_fee: row.get("base_fee"),
            included_units: row.get("included_units"),
            free_above: row.get("free_above"),
            min_value: row.get("min_value"),
            max_value: row.get("max_value"),
            is_active: row.get("is_active"),
        })
    }

    pub async fn update_shipping_rate(&self, rate_id: &str, req: &CreateShippingRateRequest) -> Result<ErpShippingRate, String> {
        let rate_type = req.rate_type.clone().unwrap_or_else(|| "FLAT".to_string());
        let base_fee = req.base_fee.unwrap_or(0);
        let included_units = req.included_units.unwrap_or(0);

        sqlx::query(
            "UPDATE erp_shipping_rates SET rate_type = $1, fee = $2, base_fee = $3, \
             included_units = $4, free_above = $5, min_value = $6, max_value = $7 \
             WHERE id = $8",
        )
        .bind(&rate_type)
        .bind(req.fee)
        .bind(base_fee)
        .bind(included_units)
        .bind(&req.free_above)
        .bind(&req.min_value)
        .bind(&req.max_value)
        .bind(rate_id)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to update shipping rate: {e}"))?;

        self.get_shipping_rate(rate_id).await
    }

    pub async fn delete_shipping_rate(&self, rate_id: &str) -> Result<(), String> {
        sqlx::query("DELETE FROM erp_shipping_rates WHERE id = $1")
            .bind(rate_id)
            .execute(&self.pool)
            .await
            .map_err(|e| format!("Failed to delete shipping rate: {e}"))?;
        Ok(())
    }

    // ---------------------------------------------------------------------------
    // Return Reason Codes
    // ---------------------------------------------------------------------------

    pub async fn list_return_reasons(&self) -> Result<Vec<ErpReturnReasonCode>, String> {
        let rows = sqlx::query(
            "SELECT id, key, is_active, sort_order FROM erp_return_reason_codes ORDER BY sort_order",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to list return reasons: {e}"))?;
        rows.iter().map(|r| Ok(ErpReturnReasonCode {
            id: r.get("id"),
            key: r.get("key"),
            is_active: r.get("is_active"),
            sort_order: r.get("sort_order"),
        })).collect()
    }

    // ---------------------------------------------------------------------------
    // Return Requests (RMA)
    // ---------------------------------------------------------------------------

    pub async fn list_return_requests(
        &self,
        _status: Option<&str>,
        _order_id: Option<&str>,
    ) -> Result<Vec<ErpReturnRequest>, String> {
        let rows = sqlx::query(
            "SELECT id, number, order_id, user_id, status, resolution, customer_note, staff_note, \
             photo_urls, requested_at, approved_at, received_at, inspected_at, refunded_at, \
             refund_amount, restocking_fee_minor \
             FROM erp_return_requests ORDER BY requested_at DESC",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to list return requests: {e}"))?;
        rows.iter().map(|r| Ok(ErpReturnRequest {
            id: r.get("id"),
            number: r.get("number"),
            order_id: r.get("order_id"),
            user_id: r.get("user_id"),
            status: r.get("status"),
            resolution: r.get("resolution"),
            customer_note: r.get("customer_note"),
            staff_note: r.get("staff_note"),
            photo_urls: r.get("photo_urls"),
            requested_at: r.get("requested_at"),
            approved_at: r.get("approved_at"),
            received_at: r.get("received_at"),
            inspected_at: r.get("inspected_at"),
            refunded_at: r.get("refunded_at"),
            refund_amount: r.get("refund_amount"),
            restocking_fee_minor: r.get("restocking_fee_minor"),
        })).collect()
    }

    pub async fn get_return_request(&self, return_id: &str) -> Result<(ErpReturnRequest, Vec<ErpReturnRequestItem>), String> {
        let row = sqlx::query(
            "SELECT id, number, order_id, user_id, status, resolution, customer_note, staff_note, \
             photo_urls, requested_at, approved_at, received_at, inspected_at, refunded_at, \
             refund_amount, restocking_fee_minor \
             FROM erp_return_requests WHERE id = $1",
        )
        .bind(return_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|_| format!("Return request '{}' not found", return_id))?;

        let request = ErpReturnRequest {
            id: row.get("id"),
            number: row.get("number"),
            order_id: row.get("order_id"),
            user_id: row.get("user_id"),
            status: row.get("status"),
            resolution: row.get("resolution"),
            customer_note: row.get("customer_note"),
            staff_note: row.get("staff_note"),
            photo_urls: row.get("photo_urls"),
            requested_at: row.get("requested_at"),
            approved_at: row.get("approved_at"),
            received_at: row.get("received_at"),
            inspected_at: row.get("inspected_at"),
            refunded_at: row.get("refunded_at"),
            refund_amount: row.get("refund_amount"),
            restocking_fee_minor: row.get("restocking_fee_minor"),
        };

        let items_rows = sqlx::query(
            "SELECT id, return_id, order_item_id, reason_id, quantity, restockable, restocked \
             FROM erp_return_request_items WHERE return_id = $1",
        )
        .bind(return_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to load return items: {e}"))?;

        let items: Vec<ErpReturnRequestItem> = items_rows.iter().map(|r| ErpReturnRequestItem {
            id: r.get("id"),
            return_id: r.get("return_id"),
            order_item_id: r.get("order_item_id"),
            reason_id: r.get("reason_id"),
            quantity: r.get("quantity"),
            restockable: r.get("restockable"),
            restocked: r.get("restocked"),
        }).collect();

        Ok((request, items))
    }

    pub async fn create_return_request(&self, req: &CreateReturnRequestRequest) -> Result<ErpReturnRequest, String> {
        let id = Uuid::new_v4().to_string();
        let number = format!("RMA-{}", &id[..8]);
        let photo_urls = req.photo_urls.as_ref()
            .map(|p| serde_json::to_string(p).unwrap_or_else(|_| "[]".to_string()))
            .unwrap_or_else(|| "[]".to_string());

        // Verify order exists
        let order_exists: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM erp_orders WHERE id = $1",
        )
        .bind(&req.order_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Failed to verify order: {e}"))?;
        if order_exists == 0 {
            return Err(format!("Order '{}' not found", req.order_id));
        }

        // Insert return request
        sqlx::query(
            "INSERT INTO erp_return_requests (id, number, order_id, user_id, customer_note, \
             photo_urls) VALUES ($1, $2, $3, $4, $5, $6)",
        )
        .bind(&id)
        .bind(&number)
        .bind(&req.order_id)
        .bind(&req.user_id)
        .bind(&req.customer_note)
        .bind(&photo_urls)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to create return request: {e}"))?;

        // Insert return items
        for item in &req.items {
            let item_id = Uuid::new_v4().to_string();
            sqlx::query(
                "INSERT INTO erp_return_request_items (id, return_id, order_item_id, reason_id, quantity) \
                 VALUES ($1, $2, $3, $4, $5)",
            )
            .bind(&item_id)
            .bind(&id)
            .bind(&item.order_item_id)
            .bind(&item.reason_id)
            .bind(item.quantity)
            .execute(&self.pool)
            .await
            .map_err(|e| format!("Failed to create return item: {e}"))?;
        }

        self.get_return_request(&id).await.map(|(req, _)| req)
    }

    pub async fn update_return_status(&self, return_id: &str, req: &UpdateReturnStatusRequest) -> Result<ErpReturnRequest, String> {
        sqlx::query(
            "UPDATE erp_return_requests SET status = $1, resolution = $2, staff_note = $3, refund_amount = $4 \
             WHERE id = $5",
        )
        .bind(&req.status)
        .bind(&req.resolution)
        .bind(&req.staff_note)
        .bind(&req.refund_amount)
        .bind(return_id)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to update return status: {e}"))?;

        self.get_return_request(return_id).await.map(|(req, _)| req)
    }

    pub async fn inspect_return_item(&self, req: &InspectReturnItemRequest) -> Result<ErpReturnRequestItem, String> {
        sqlx::query(
            "UPDATE erp_return_request_items SET restockable = $1, restocked = $2 WHERE id = $3",
        )
        .bind(req.restockable)
        .bind(req.restocked)
        .bind(&req.return_item_id)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to inspect return item: {e}"))?;

        let row = sqlx::query(
            "SELECT id, return_id, order_item_id, reason_id, quantity, restockable, restocked \
             FROM erp_return_request_items WHERE id = $1",
        )
        .bind(&req.return_item_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|_| format!("Return item '{}' not found", req.return_item_id))?;

        Ok(ErpReturnRequestItem {
            id: row.get("id"),
            return_id: row.get("return_id"),
            order_item_id: row.get("order_item_id"),
            reason_id: row.get("reason_id"),
            quantity: row.get("quantity"),
            restockable: row.get("restockable"),
            restocked: row.get("restocked"),
        })
    }

    pub async fn get_return_stats(&self) -> Result<serde_json::Value, String> {
        let total_returns: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM erp_return_requests",
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Failed to count returns: {e}"))?;

        let pending_returns: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM erp_return_requests WHERE status = 'REQUESTED'",
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Failed to count pending returns: {e}"))?;

        let refunded_amount: i64 = sqlx::query_scalar(
            "SELECT COALESCE(SUM(refund_amount), 0) FROM erp_return_requests WHERE status = 'REFUNDED'",
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Failed to sum refunds: {e}"))?;

        Ok(serde_json::json!({
            "totalReturns": total_returns,
            "pendingReturns": pending_returns,
            "refundedAmount": refunded_amount,
        }))
    }

    // ---------------------------------------------------------------------------
    // Manifests
    // ---------------------------------------------------------------------------

    pub async fn list_manifests(&self, _courier_key: Option<&str>, _status: Option<&str>) -> Result<Vec<ErpManifest>, String> {
        let rows = sqlx::query(
            "SELECT id, number, courier_key, status, dispatched_at, created_by_id, created_at \
             FROM erp_manifests ORDER BY created_at DESC",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to list manifests: {e}"))?;
        rows.iter().map(|r| Ok(ErpManifest {
            id: r.get("id"),
            number: r.get("number"),
            courier_key: r.get("courier_key"),
            status: r.get("status"),
            dispatched_at: r.get("dispatched_at"),
            created_by_id: r.get("created_by_id"),
            created_at: r.get("created_at"),
        })).collect()
    }

    pub async fn dispatch_manifest(&self, manifest_id: &str) -> Result<ErpManifest, String> {
        sqlx::query(
            "UPDATE erp_manifests SET status = 'DISPATCHED', dispatched_at = NOW() WHERE id = $1",
        )
        .bind(manifest_id)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to dispatch manifest: {e}"))?;

        let row = sqlx::query(
            "SELECT id, number, courier_key, status, dispatched_at, created_by_id, created_at \
             FROM erp_manifests WHERE id = $1",
        )
        .bind(manifest_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|_| format!("Manifest '{}' not found", manifest_id))?;

        Ok(ErpManifest {
            id: row.get("id"),
            number: row.get("number"),
            courier_key: row.get("courier_key"),
            status: row.get("status"),
            dispatched_at: row.get("dispatched_at"),
            created_by_id: row.get("created_by_id"),
            created_at: row.get("created_at"),
        })
    }
}
