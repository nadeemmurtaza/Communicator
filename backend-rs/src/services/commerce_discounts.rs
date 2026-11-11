#![allow(dead_code)]

//! Discounts, Coupons, and Memberships service

use serde::{Deserialize, Serialize};
use sqlx::{Row, PgPool};
use uuid::Uuid;

use crate::models::{Coupon, CouponUsage, MembershipTier, CustomerMembership};

// ---------------------------------------------------------------------------
// Request/Response DTOs
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct CreateCouponRequest {
    pub brand_id: String,
    pub code: String,
    pub r#type: Option<String>,
    pub value: f64,
    pub min_order_amount: Option<f64>,
    pub max_discount: Option<f64>,
    pub usage_limit: Option<i32>,
    pub starts_at: Option<String>,
    pub ends_at: Option<String>,
    pub applicable_products: Option<serde_json::Value>,
    pub applicable_categories: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateCouponRequest {
    pub code: Option<String>,
    pub r#type: Option<String>,
    pub value: Option<f64>,
    pub min_order_amount: Option<Option<f64>>,
    pub max_discount: Option<Option<f64>>,
    pub usage_limit: Option<Option<i32>>,
    pub starts_at: Option<Option<String>>,
    pub ends_at: Option<Option<String>>,
    pub applicable_products: Option<serde_json::Value>,
    pub applicable_categories: Option<serde_json::Value>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct ValidateCouponRequest {
    pub code: String,
    pub order_amount: f64,
    pub customer_id: Option<String>,
    pub product_ids: Option<Vec<String>>,
}

#[derive(Debug, Serialize)]
pub struct ValidateCouponResponse {
    pub valid: bool,
    pub coupon: Option<Coupon>,
    pub discount_amount: f64,
    pub message: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateMembershipTierRequest {
    pub brand_id: String,
    pub name: String,
    pub discount_percent: Option<f64>,
    pub monthly_price: Option<f64>,
    pub benefits: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateMembershipTierRequest {
    pub name: Option<String>,
    pub discount_percent: Option<f64>,
    pub monthly_price: Option<Option<f64>>,
    pub benefits: Option<serde_json::Value>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct AssignMembershipRequest {
    pub customer_id: String,
    pub membership_tier_id: String,
    pub ends_at: Option<String>,
}

// ---------------------------------------------------------------------------
// Service
// ---------------------------------------------------------------------------

pub struct CommerceDiscountsService {
    pool: PgPool,
}

impl CommerceDiscountsService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    // ---------------------------------------------------------------------------
    // Coupons
    // ---------------------------------------------------------------------------

    pub async fn list_coupons(&self, brand_id: &str, active_only: bool) -> Result<Vec<Coupon>, String> {
        let query = if active_only {
            "SELECT * FROM coupons WHERE brand_id = ? AND is_active = 1 ORDER BY created_at DESC"
        } else {
            "SELECT * FROM coupons WHERE brand_id = ? ORDER BY created_at DESC"
        };
        let rows = sqlx::query(query)
            .bind(brand_id)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| e.to_string())?;
        rows.iter()
            .map(|r| self.row_to_coupon(r))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())
    }

    pub async fn get_coupon(&self, id: &str) -> Result<Coupon, String> {
        let row = sqlx::query("SELECT * FROM coupons WHERE id = ?")
            .bind(id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| e.to_string())?;
        self.row_to_coupon(&row)
    }

    pub async fn create_coupon(&self, req: &CreateCouponRequest) -> Result<Coupon, String> {
        let id = Uuid::new_v4().to_string();
        let coupon_type = req.r#type.as_deref().unwrap_or("percentage");
        let applicable_products = req.applicable_products.as_ref().map(|v| serde_json::to_string(v).unwrap_or_default());
        let applicable_categories = req.applicable_categories.as_ref().map(|v| serde_json::to_string(v).unwrap_or_default());
        sqlx::query(
            "INSERT INTO coupons (id, brand_id, code, type, value, min_order_amount, max_discount, \
             usage_limit, starts_at, ends_at, applicable_products, applicable_categories) \
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(&id)
        .bind(&req.brand_id)
        .bind(&req.code)
        .bind(coupon_type)
        .bind(req.value)
        .bind(req.min_order_amount)
        .bind(req.max_discount)
        .bind(req.usage_limit)
        .bind(req.starts_at.as_deref())
        .bind(req.ends_at.as_deref())
        .bind(applicable_products.as_deref())
        .bind(applicable_categories.as_deref())
        .execute(&self.pool)
        .await
        .map_err(|e| e.to_string())?;
        self.get_coupon(&id).await
    }

    pub async fn update_coupon(&self, id: &str, req: &UpdateCouponRequest) -> Result<Coupon, String> {
        // Use individual UPDATE statements for each field to avoid dynamic binding issues
        if let Some(ref code) = req.code {
            sqlx::query("UPDATE coupons SET code = ?, updated_at = NOW()::TEXT WHERE id = ?")
                .bind(code).bind(id)
                .execute(&self.pool).await.map_err(|e| e.to_string())?;
        }
        if let Some(ref r#type) = req.r#type {
            sqlx::query("UPDATE coupons SET `type` = ?, updated_at = NOW()::TEXT WHERE id = ?")
                .bind(r#type).bind(id)
                .execute(&self.pool).await.map_err(|e| e.to_string())?;
        }
        if let Some(value) = req.value {
            sqlx::query("UPDATE coupons SET value = ?, updated_at = NOW()::TEXT WHERE id = ?")
                .bind(&value).bind(id)
                .execute(&self.pool).await.map_err(|e| e.to_string())?;
        }
        if let Some(ref min_order_amount) = req.min_order_amount {
            sqlx::query("UPDATE coupons SET min_order_amount = ?, updated_at = NOW()::TEXT WHERE id = ?")
                .bind(min_order_amount).bind(id)
                .execute(&self.pool).await.map_err(|e| e.to_string())?;
        }
        if let Some(ref max_discount) = req.max_discount {
            sqlx::query("UPDATE coupons SET max_discount = ?, updated_at = NOW()::TEXT WHERE id = ?")
                .bind(max_discount).bind(id)
                .execute(&self.pool).await.map_err(|e| e.to_string())?;
        }
        if let Some(ref is_active) = req.is_active {
            sqlx::query("UPDATE coupons SET is_active = ?, updated_at = NOW()::TEXT WHERE id = ?")
                .bind(is_active).bind(id)
                .execute(&self.pool).await.map_err(|e| e.to_string())?;
        }

        self.get_coupon(id).await
    }

    pub async fn delete_coupon(&self, id: &str) -> Result<(), String> {
        sqlx::query("DELETE FROM coupons WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub async fn validate_coupon(&self, req: &ValidateCouponRequest) -> Result<ValidateCouponResponse, String> {
        let coupon = sqlx::query("SELECT * FROM coupons WHERE code = ? AND is_active = 1")
            .bind(&req.code)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| e.to_string())?;

        let Some(row) = coupon else {
            return Ok(ValidateCouponResponse {
                valid: false,
                coupon: None,
                discount_amount: 0.0,
                message: Some("Coupon not found or inactive".to_string()),
            });
        };

        let coupon = self.row_to_coupon(&row)?;

        // Check date range
        if let Some(ref starts_at) = coupon.starts_at {
            if starts_at > &chrono::Utc::now().to_rfc3339() {
                return Ok(ValidateCouponResponse {
                    valid: false,
                    coupon: Some(coupon),
                    discount_amount: 0.0,
                    message: Some("Coupon has not started yet".to_string()),
                });
            }
        }
        if let Some(ref ends_at) = coupon.ends_at {
            if ends_at < &chrono::Utc::now().to_rfc3339() {
                return Ok(ValidateCouponResponse {
                    valid: false,
                    coupon: Some(coupon),
                    discount_amount: 0.0,
                    message: Some("Coupon has expired".to_string()),
                });
            }
        }

        // Check minimum order amount
        if let Some(min_amount) = coupon.min_order_amount {
            if req.order_amount < min_amount {
                return Ok(ValidateCouponResponse {
                    valid: false,
                    coupon: Some(coupon),
                    discount_amount: 0.0,
                    message: Some(format!("Minimum order amount is ${:.2}", min_amount)),
                });
            }
        }

        // Check usage limit
        if let Some(limit) = coupon.usage_limit {
            if coupon.used_count >= limit {
                return Ok(ValidateCouponResponse {
                    valid: false,
                    coupon: Some(coupon),
                    discount_amount: 0.0,
                    message: Some("Coupon usage limit reached".to_string()),
                });
            }
        }

        // Calculate discount
        let discount = match coupon.r#type.as_str() {
            "percentage" => {
                let amount = req.order_amount * (coupon.value / 100.0);
                coupon.max_discount.map(|max| amount.min(max)).unwrap_or(amount)
            }
            "fixed" => coupon.value.min(req.order_amount),
            _ => 0.0,
        };

        Ok(ValidateCouponResponse {
            valid: true,
            coupon: Some(coupon),
            discount_amount: discount,
            message: None,
        })
    }

    pub async fn record_coupon_usage(&self, coupon_id: &str, order_id: &str, customer_id: &str, discount_amount: f64) -> Result<(), String> {
        let id = Uuid::new_v4().to_string();
        sqlx::query(
            "INSERT INTO coupon_usage (id, coupon_id, order_id, customer_id, discount_amount) \
             VALUES (?, ?, ?, ?, ?)"
        )
        .bind(&id)
        .bind(coupon_id)
        .bind(order_id)
        .bind(customer_id)
        .bind(discount_amount)
        .execute(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        // Increment used_count
        sqlx::query("UPDATE coupons SET used_count = used_count + 1, updated_at = NOW()::TEXT WHERE id = ?")
            .bind(coupon_id)
            .execute(&self.pool)
            .await
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub async fn coupon_usage_history(&self, coupon_id: &str) -> Result<Vec<CouponUsage>, String> {
        let rows = sqlx::query("SELECT * FROM coupon_usage WHERE coupon_id = ? ORDER BY used_at DESC")
            .bind(coupon_id)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| e.to_string())?;
        rows.iter()
            .map(|r| self.row_to_coupon_usage(r))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())
    }

    // ---------------------------------------------------------------------------
    // Membership Tiers
    // ---------------------------------------------------------------------------

    pub async fn list_membership_tiers(&self, brand_id: &str) -> Result<Vec<MembershipTier>, String> {
        let rows = sqlx::query("SELECT * FROM membership_tiers WHERE brand_id = ? ORDER BY name")
            .bind(brand_id)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| e.to_string())?;
        rows.iter()
            .map(|r| self.row_to_membership_tier(r))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())
    }

    pub async fn create_membership_tier(&self, req: &CreateMembershipTierRequest) -> Result<MembershipTier, String> {
        let id = Uuid::new_v4().to_string();
        let benefits = req.benefits.as_ref().map(|v| serde_json::to_string(v).unwrap_or_default());
        sqlx::query(
            "INSERT INTO membership_tiers (id, brand_id, name, discount_percent, monthly_price, benefits) \
             VALUES (?, ?, ?, ?, ?, ?)"
        )
        .bind(&id)
        .bind(&req.brand_id)
        .bind(&req.name)
        .bind(req.discount_percent.unwrap_or(0.0))
        .bind(req.monthly_price)
        .bind(benefits.as_deref())
        .execute(&self.pool)
        .await
        .map_err(|e| e.to_string())?;
        self.get_membership_tier(&id).await
    }

    pub async fn get_membership_tier(&self, id: &str) -> Result<MembershipTier, String> {
        let row = sqlx::query("SELECT * FROM membership_tiers WHERE id = ?")
            .bind(id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| e.to_string())?;
        self.row_to_membership_tier(&row)
    }

    pub async fn update_membership_tier(&self, id: &str, req: &UpdateMembershipTierRequest) -> Result<MembershipTier, String> {
        let mut sets = Vec::new();
        if let Some(ref name) = req.name { sets.push("name = ?"); }
        if let Some(dp) = req.discount_percent { sets.push("discount_percent = ?"); }
        if let Some(ref is_active) = req.is_active { sets.push("is_active = ?"); }
        sets.push("updated_at = NOW()::TEXT");

        if sets.len() <= 1 {
            return self.get_membership_tier(id).await;
        }

        let query = format!("UPDATE membership_tiers SET {} WHERE id = ?", sets.join(", "));
        sqlx::query(&query)
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| e.to_string())?;
        self.get_membership_tier(id).await
    }

    pub async fn delete_membership_tier(&self, id: &str) -> Result<(), String> {
        sqlx::query("DELETE FROM membership_tiers WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    // ---------------------------------------------------------------------------
    // Customer Memberships
    // ---------------------------------------------------------------------------

    pub async fn list_customer_memberships(&self, customer_id: &str) -> Result<Vec<CustomerMembership>, String> {
        let rows = sqlx::query("SELECT * FROM customer_memberships WHERE customer_id = ? ORDER BY started_at DESC")
            .bind(customer_id)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| e.to_string())?;
        rows.iter()
            .map(|r| self.row_to_customer_membership(r))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())
    }

    pub async fn assign_membership(&self, req: &AssignMembershipRequest) -> Result<CustomerMembership, String> {
        let id = Uuid::new_v4().to_string();
        sqlx::query(
            "INSERT INTO customer_memberships (id, customer_id, membership_tier_id, ends_at) \
             VALUES (?, ?, ?, ?)"
        )
        .bind(&id)
        .bind(&req.customer_id)
        .bind(&req.membership_tier_id)
        .bind(req.ends_at.as_deref())
        .execute(&self.pool)
        .await
        .map_err(|e| e.to_string())?;
        self.get_customer_membership(&id).await
    }

    pub async fn get_customer_membership(&self, id: &str) -> Result<CustomerMembership, String> {
        let row = sqlx::query("SELECT * FROM customer_memberships WHERE id = ?")
            .bind(id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| e.to_string())?;
        self.row_to_customer_membership(&row)
    }

    pub async fn get_active_customer_membership(&self, customer_id: &str) -> Result<Option<CustomerMembership>, String> {
        let row = sqlx::query(
            "SELECT * FROM customer_memberships WHERE customer_id = ? AND status = 'active' \
             AND (ends_at IS NULL OR ends_at > NOW()::TEXT) \
             LIMIT 1"
        )
        .bind(customer_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| e.to_string())?;
        match row {
            Some(r) => Ok(Some(self.row_to_customer_membership(&r)?)),
            None => Ok(None),
        }
    }

    pub async fn customer_discount(&self, customer_id: &str) -> Result<f64, String> {
        let membership = self.get_active_customer_membership(customer_id).await?;
        let Some(m) = membership else { return Ok(0.0); };
        let tier = self.get_membership_tier(&m.membership_tier_id).await?;
        Ok(tier.discount_percent)
    }

    // ---------------------------------------------------------------------------
    // Row helpers
    // ---------------------------------------------------------------------------

    fn row_to_coupon(&self, r: &sqlx::postgres::PgRow) -> Result<Coupon, String> {
        Ok(Coupon {
            id: r.try_get("id").map_err(|e| e.to_string())?,
            brand_id: r.try_get("brand_id").map_err(|e| e.to_string())?,
            code: r.try_get("code").map_err(|e| e.to_string())?,
            r#type: r.try_get("type").map_err(|e| e.to_string())?,
            value: r.try_get("value").map_err(|e| e.to_string())?,
            min_order_amount: r.try_get("min_order_amount").map_err(|e| e.to_string())?,
            max_discount: r.try_get("max_discount").map_err(|e| e.to_string())?,
            usage_limit: r.try_get("usage_limit").map_err(|e| e.to_string())?,
            used_count: r.try_get("used_count").map_err(|e| e.to_string())?,
            starts_at: r.try_get("starts_at").map_err(|e| e.to_string())?,
            ends_at: r.try_get("ends_at").map_err(|e| e.to_string())?,
            applicable_products: r.try_get("applicable_products").map_err(|e| e.to_string())?,
            applicable_categories: r.try_get("applicable_categories").map_err(|e| e.to_string())?,
            is_active: r.try_get("is_active").map_err(|e| e.to_string())?,
            created_at: r.try_get("created_at").map_err(|e| e.to_string())?,
            updated_at: r.try_get("updated_at").map_err(|e| e.to_string())?,
        })
    }

    fn row_to_coupon_usage(&self, r: &sqlx::postgres::PgRow) -> Result<CouponUsage, String> {
        Ok(CouponUsage {
            id: r.try_get("id").map_err(|e| e.to_string())?,
            coupon_id: r.try_get("coupon_id").map_err(|e| e.to_string())?,
            order_id: r.try_get("order_id").map_err(|e| e.to_string())?,
            customer_id: r.try_get("customer_id").map_err(|e| e.to_string())?,
            discount_amount: r.try_get("discount_amount").map_err(|e| e.to_string())?,
            used_at: r.try_get("used_at").map_err(|e| e.to_string())?,
        })
    }

    fn row_to_membership_tier(&self, r: &sqlx::postgres::PgRow) -> Result<MembershipTier, String> {
        Ok(MembershipTier {
            id: r.try_get("id").map_err(|e| e.to_string())?,
            brand_id: r.try_get("brand_id").map_err(|e| e.to_string())?,
            name: r.try_get("name").map_err(|e| e.to_string())?,
            discount_percent: r.try_get("discount_percent").map_err(|e| e.to_string())?,
            monthly_price: r.try_get("monthly_price").map_err(|e| e.to_string())?,
            benefits: r.try_get("benefits").map_err(|e| e.to_string())?,
            is_active: r.try_get("is_active").map_err(|e| e.to_string())?,
            created_at: r.try_get("created_at").map_err(|e| e.to_string())?,
            updated_at: r.try_get("updated_at").map_err(|e| e.to_string())?,
        })
    }

    fn row_to_customer_membership(&self, r: &sqlx::postgres::PgRow) -> Result<CustomerMembership, String> {
        Ok(CustomerMembership {
            id: r.try_get("id").map_err(|e| e.to_string())?,
            customer_id: r.try_get("customer_id").map_err(|e| e.to_string())?,
            membership_tier_id: r.try_get("membership_tier_id").map_err(|e| e.to_string())?,
            started_at: r.try_get("started_at").map_err(|e| e.to_string())?,
            ends_at: r.try_get("ends_at").map_err(|e| e.to_string())?,
            status: r.try_get("status").map_err(|e| e.to_string())?,
            created_at: r.try_get("created_at").map_err(|e| e.to_string())?,
        })
    }
}
