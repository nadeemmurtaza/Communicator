#![allow(dead_code)]

//! Commerce Gaps service — covers all 30 Take.app gap features.
//! Gaps #2-#30: Delivery fees, payment proofs, notifications, payment gateways,
//! service types, delivery schedules, invoices, CSV import, customer accounts,
//! reviews, loyalty, reminders, staff, POS, receipts, inbox, wholesale pricing,
//! payment fees, payment recovery, follow-ups, custom domains, storefront pages,
//! meta pixels, checkout questions, delivery proofs.

use serde::{Deserialize, Serialize};
use sqlx::{Row, PgPool};
use uuid::Uuid;

use crate::models::{
    DeliveryFeeRule, PaymentProof, OrderNotificationTemplate, PaymentGateway,
    ServiceType, DeliverySchedule, DeliveryTimeSlot, Invoice, CsvImportJob,
    CustomerAccount, CustomerReview, LoyaltyProgram, CustomerPoints, PointsTransaction,
    PaymentReminder, DeliveryReminder, StaffMember, PosSession, Receipt,
    CommerceInboxThread, WholesalePricing, PaymentFee, PaymentRecovery,
    AutomatedFollowup, FollowupLog, CustomDomain, StorefrontPage, MetaPixel,
    CheckoutQuestion, CheckoutQuestionAnswer, DeliveryProof,
};

// ---------------------------------------------------------------------------
// DTOs
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct CreateDeliveryFeeRuleRequest {
    pub brand_id: String,
    pub name: String,
    pub r#type: Option<String>,
    pub base_fee: Option<f64>,
    pub per_km_fee: Option<f64>,
    pub per_gram_fee: Option<f64>,
    pub max_distance: Option<f64>,
    pub free_delivery_threshold: Option<f64>,
    pub min_order_value: Option<f64>,
    pub postal_codes: Option<String>,
    pub regions: Option<String>,
    pub sort_order: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct CreatePaymentGatewayRequest {
    pub brand_id: String,
    pub name: String,
    pub provider: String,
    pub credentials: Option<serde_json::Value>,
    pub currency: Option<String>,
    pub sort_order: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct CreateServiceTypeRequest {
    pub brand_id: String,
    pub name: String,
    pub r#type: String,
    pub description: Option<String>,
    pub sort_order: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct CreateDeliveryScheduleRequest {
    pub brand_id: String,
    pub day_of_week: i32,
    pub start_time: String,
    pub end_time: String,
    pub cutoff_time: Option<String>,
    pub max_capacity: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct CreateDeliveryTimeSlotRequest {
    pub brand_id: String,
    pub day_of_week: i32,
    pub slot_start: String,
    pub slot_end: String,
    pub label: Option<String>,
    pub max_orders: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct CreateNotificationTemplateRequest {
    pub brand_id: String,
    pub channel_type: String,
    pub trigger_event: String,
    pub template_name: String,
    pub template_body: String,
}

#[derive(Debug, Deserialize)]
pub struct UploadPaymentProofRequest {
    pub order_id: String,
    pub customer_id: String,
    pub file_url: String,
    pub file_name: Option<String>,
    pub file_size: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct VerifyPaymentProofRequest {
    pub verified_by: String,
    pub merchant_notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateReviewRequest {
    pub order_id: String,
    pub customer_id: String,
    pub product_id: Option<String>,
    pub rating: i32,
    pub comment: Option<String>,
    pub is_public: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct CreateLoyaltyProgramRequest {
    pub brand_id: String,
    pub name: String,
    pub points_per_currency: Option<f64>,
    pub points_expiry_days: Option<i32>,
    pub redemption_rate: Option<f64>,
}

#[derive(Debug, Deserialize)]
pub struct CreateStaffMemberRequest {
    pub brand_id: String,
    pub name: String,
    pub email: String,
    pub role: Option<String>,
    pub permissions: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct CreateWholesalePricingRequest {
    pub product_id: String,
    pub customer_group: String,
    pub price: f64,
    pub min_quantity: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct CreateFollowupRequest {
    pub brand_id: String,
    pub name: String,
    pub trigger_type: String,
    pub trigger_days: Option<i32>,
    pub channel_type: String,
    pub template_body: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateCustomDomainRequest {
    pub brand_id: String,
    pub domain: String,
    pub storefront_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateStorefrontPageRequest {
    pub storefront_id: String,
    pub slug: String,
    pub title: String,
    pub content: Option<String>,
    pub sort_order: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct CreateMetaPixelRequest {
    pub brand_id: String,
    pub pixel_id: String,
    pub access_token: Option<String>,
    pub events: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct CreateCheckoutQuestionRequest {
    pub brand_id: String,
    pub question: String,
    pub question_type: Option<String>,
    pub options: Option<serde_json::Value>,
    pub is_required: Option<bool>,
    pub sort_order: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct SubmitCheckoutAnswersRequest {
    pub order_id: String,
    pub answers: Vec<CheckoutAnswerItem>,
}

#[derive(Debug, Deserialize)]
pub struct CheckoutAnswerItem {
    pub question_id: String,
    pub answer: String,
}

#[derive(Debug, Deserialize)]
pub struct UploadDeliveryProofRequest {
    pub shipment_id: String,
    pub order_id: String,
    pub file_url: String,
    pub file_name: Option<String>,
    pub uploaded_by: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ImportCsvRequest {
    pub brand_id: String,
    pub entity_type: String,
    pub file_url: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateCustomerAccountRequest {
    pub email: Option<String>,
    pub phone: Option<String>,
    pub password: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: Option<String>,
    pub phone: Option<String>,
    pub password: String,
}

// ---------------------------------------------------------------------------
// Service
// ---------------------------------------------------------------------------

pub struct CommerceGapsService {
    pool: PgPool,
}

impl CommerceGapsService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    // -----------------------------------------------------------------------
    // Gap #2: Delivery Fee Calculator
    // -----------------------------------------------------------------------

    pub async fn list_delivery_fee_rules(&self, brand_id: &str) -> Result<Vec<DeliveryFeeRule>, String> {
        let rows = sqlx::query(
            "SELECT id, brand_id, name, type, base_fee, per_km_fee, per_gram_fee, \
             max_distance, free_delivery_threshold, min_order_value, postal_codes, regions, \
             is_active, sort_order, created_at, updated_at \
             FROM delivery_fee_rules WHERE brand_id = $1 ORDER BY sort_order, name",
        )
        .bind(brand_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to list delivery fee rules: {e}"))?;

        rows.iter().map(|r| Ok(DeliveryFeeRule {
            id: r.get("id"), brand_id: r.get("brand_id"), name: r.get("name"),
            r#type: r.get("type"), base_fee: r.get("base_fee"),
            per_km_fee: r.get("per_km_fee"), per_gram_fee: r.get("per_gram_fee"),
            max_distance: r.get("max_distance"), free_delivery_threshold: r.get("free_delivery_threshold"),
            min_order_value: r.get("min_order_value"), postal_codes: r.get("postal_codes"),
            regions: r.get("regions"), is_active: r.get("is_active"),
            sort_order: r.get("sort_order"), created_at: r.get("created_at"),
            updated_at: r.get("updated_at"),
        })).collect()
    }

    pub async fn create_delivery_fee_rule(&self, req: &CreateDeliveryFeeRuleRequest) -> Result<DeliveryFeeRule, String> {
        let id = Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        let r#type = req.r#type.clone().unwrap_or_else(|| "flat".to_string());
        let base_fee = req.base_fee.unwrap_or(0.0);
        let sort_order = req.sort_order.unwrap_or(0);

        sqlx::query(
            "INSERT INTO delivery_fee_rules (id, brand_id, name, type, base_fee, per_km_fee, per_gram_fee, \
             max_distance, free_delivery_threshold, min_order_value, postal_codes, regions, \
             is_active, sort_order, created_at, updated_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, 1, $13, $14, $15)",
        )
        .bind(&id).bind(&req.brand_id).bind(&req.name).bind(&r#type)
        .bind(base_fee).bind(req.per_km_fee.unwrap_or(0.0)).bind(req.per_gram_fee.unwrap_or(0.0))
        .bind(&req.max_distance).bind(&req.free_delivery_threshold).bind(&req.min_order_value)
        .bind(&req.postal_codes).bind(&req.regions).bind(sort_order as i32)
        .bind(&now).bind(&now)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to create delivery fee rule: {e}"))?;

        self.get_delivery_fee_rule(&id).await
    }

    pub async fn update_delivery_fee_rule(&self, id: &str, req: &CreateDeliveryFeeRuleRequest) -> Result<DeliveryFeeRule, String> {
        let now = chrono::Utc::now().to_rfc3339();
        let r#type = req.r#type.clone().unwrap_or_else(|| "flat".to_string());
        sqlx::query(
            "UPDATE delivery_fee_rules SET name = $1, type = $2, base_fee = $3, per_km_fee = $4, \
             per_gram_fee = $5, max_distance = $6, free_delivery_threshold = $7, min_order_value = $8, \
             postal_codes = $9, regions = $10, sort_order = $11, updated_at = $12 WHERE id = $13",
        )
        .bind(&req.name).bind(&r#type).bind(req.base_fee.unwrap_or(0.0))
        .bind(req.per_km_fee.unwrap_or(0.0)).bind(req.per_gram_fee.unwrap_or(0.0))
        .bind(&req.max_distance).bind(&req.free_delivery_threshold).bind(&req.min_order_value)
        .bind(&req.postal_codes).bind(&req.regions).bind(req.sort_order.unwrap_or(0) as i32)
        .bind(&now).bind(id)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to update delivery fee rule: {e}"))?;

        self.get_delivery_fee_rule(id).await
    }

    pub async fn delete_delivery_fee_rule(&self, id: &str) -> Result<(), String> {
        sqlx::query("DELETE FROM delivery_fee_rules WHERE id = $1")
            .bind(id).execute(&self.pool).await
            .map_err(|e| format!("Failed to delete delivery fee rule: {e}"))?;
        Ok(())
    }

    async fn get_delivery_fee_rule(&self, id: &str) -> Result<DeliveryFeeRule, String> {
        let r = sqlx::query(
            "SELECT id, brand_id, name, type, base_fee, per_km_fee, per_gram_fee, \
             max_distance, free_delivery_threshold, min_order_value, postal_codes, regions, \
             is_active, sort_order, created_at, updated_at FROM delivery_fee_rules WHERE id = $1",
        )
        .bind(id).fetch_one(&self.pool).await
            .map_err(|e| format!("Delivery fee rule not found: {e}"))?;
        Ok(DeliveryFeeRule {
            id: r.get("id"), brand_id: r.get("brand_id"), name: r.get("name"),
            r#type: r.get("type"), base_fee: r.get("base_fee"),
            per_km_fee: r.get("per_km_fee"), per_gram_fee: r.get("per_gram_fee"),
            max_distance: r.get("max_distance"), free_delivery_threshold: r.get("free_delivery_threshold"),
            min_order_value: r.get("min_order_value"), postal_codes: r.get("postal_codes"),
            regions: r.get("regions"), is_active: r.get("is_active"),
            sort_order: r.get("sort_order"), created_at: r.get("created_at"),
            updated_at: r.get("updated_at"),
        })
    }

    /// Calculate delivery fee based on rules, distance, weight, and order total
    pub async fn calculate_delivery_fee(
        &self, brand_id: &str, distance_km: f64, weight_grams: f64, order_total: f64, postal_code: Option<&str>,
    ) -> Result<f64, String> {
        let rules = self.list_delivery_fee_rules(brand_id).await?;
        for rule in rules {
            if !rule.is_active { continue; }
            if let Some(min_val) = rule.min_order_value {
                if order_total < min_val { continue; }
            }
            if let Some(threshold) = rule.free_delivery_threshold {
                if order_total >= threshold { return Ok(0.0); }
            }
            if let Some(max_dist) = rule.max_distance {
                if distance_km > max_dist { continue; }
            }
            if let Some(ref pc) = rule.postal_codes {
                if let Some(code) = postal_code {
                    if !pc.contains(code) { continue; }
                }
            }
            let mut fee = rule.base_fee;
            if rule.r#type == "distance" {
                fee += distance_km * rule.per_km_fee;
            } else if rule.r#type == "weight" {
                fee += (weight_grams / 1000.0) * rule.per_gram_fee;
            }
            return Ok(fee);
        }
        Ok(0.0)
    }

    // -----------------------------------------------------------------------
    // Gap #3: Payment Proof Upload
    // -----------------------------------------------------------------------

    pub async fn upload_payment_proof(&self, req: &UploadPaymentProofRequest) -> Result<PaymentProof, String> {
        let id = Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        sqlx::query(
            "INSERT INTO payment_proofs (id, order_id, customer_id, file_url, file_name, file_size, status, created_at) \
             VALUES ($1, $2, $3, $4, $5, $6, 'pending', $7)",
        )
        .bind(&id).bind(&req.order_id).bind(&req.customer_id)
        .bind(&req.file_url).bind(&req.file_name).bind(&req.file_size).bind(&now)
        .execute(&self.pool).await
            .map_err(|e| format!("Failed to upload payment proof: {e}"))?;
        self.get_payment_proof(&id).await
    }

    pub async fn list_payment_proofs(&self, order_id: Option<&str>) -> Result<Vec<PaymentProof>, String> {
        let (query, rows) = if let Some(oid) = order_id {
            let q = "SELECT id, order_id, customer_id, file_url, file_name, file_size, status, \
                     merchant_notes, verified_at, verified_by, created_at \
                     FROM payment_proofs WHERE order_id = $1 ORDER BY created_at DESC";
            (q, sqlx::query(q).bind(oid).fetch_all(&self.pool).await
                .map_err(|e| format!("Failed to list payment proofs: {e}"))?)
        } else {
            let q = "SELECT id, order_id, customer_id, file_url, file_name, file_size, status, \
                     merchant_notes, verified_at, verified_by, created_at \
                     FROM payment_proofs ORDER BY created_at DESC";
            (q, sqlx::query(q).fetch_all(&self.pool).await
                .map_err(|e| format!("Failed to list payment proofs: {e}"))?)
        };
        rows.iter().map(|r| Ok(PaymentProof {
            id: r.get("id"), order_id: r.get("order_id"), customer_id: r.get("customer_id"),
            file_url: r.get("file_url"), file_name: r.get("file_name"), file_size: r.get("file_size"),
            status: r.get("status"), merchant_notes: r.get("merchant_notes"),
            verified_at: r.get("verified_at"), verified_by: r.get("verified_by"),
            created_at: r.get("created_at"),
        })).collect()
    }

    pub async fn verify_payment_proof(&self, id: &str, req: &VerifyPaymentProofRequest) -> Result<PaymentProof, String> {
        let now = chrono::Utc::now().to_rfc3339();
        sqlx::query(
            "UPDATE payment_proofs SET status = 'verified', verified_at = $1, verified_by = $2, merchant_notes = $3 WHERE id = $4",
        )
        .bind(&now).bind(&req.verified_by).bind(&req.merchant_notes).bind(id)
        .execute(&self.pool).await
            .map_err(|e| format!("Failed to verify payment proof: {e}"))?;
        self.get_payment_proof(id).await
    }

    pub async fn reject_payment_proof(&self, id: &str, notes: &str) -> Result<PaymentProof, String> {
        sqlx::query(
            "UPDATE payment_proofs SET status = 'rejected', merchant_notes = $1 WHERE id = $2",
        )
        .bind(notes).bind(id).execute(&self.pool).await
            .map_err(|e| format!("Failed to reject payment proof: {e}"))?;
        self.get_payment_proof(id).await
    }

    async fn get_payment_proof(&self, id: &str) -> Result<PaymentProof, String> {
        let r = sqlx::query(
            "SELECT id, order_id, customer_id, file_url, file_name, file_size, status, \
             merchant_notes, verified_at, verified_by, created_at FROM payment_proofs WHERE id = $1",
        )
        .bind(id).fetch_one(&self.pool).await
            .map_err(|e| format!("Payment proof not found: {e}"))?;
        Ok(PaymentProof {
            id: r.get("id"), order_id: r.get("order_id"), customer_id: r.get("customer_id"),
            file_url: r.get("file_url"), file_name: r.get("file_name"), file_size: r.get("file_size"),
            status: r.get("status"), merchant_notes: r.get("merchant_notes"),
            verified_at: r.get("verified_at"), verified_by: r.get("verified_by"),
            created_at: r.get("created_at"),
        })
    }

    // -----------------------------------------------------------------------
    // Gap #4: Order Notification Templates
    // -----------------------------------------------------------------------

    pub async fn list_notification_templates(&self, brand_id: &str) -> Result<Vec<OrderNotificationTemplate>, String> {
        let rows = sqlx::query(
            "SELECT id, brand_id, channel_type, trigger_event, template_name, template_body, \
             is_active, created_at, updated_at \
             FROM order_notification_templates WHERE brand_id = $1 ORDER BY trigger_event",
        )
        .bind(brand_id).fetch_all(&self.pool).await
            .map_err(|e| format!("Failed to list notification templates: {e}"))?;
        rows.iter().map(|r| Ok(OrderNotificationTemplate {
            id: r.get("id"), brand_id: r.get("brand_id"), channel_type: r.get("channel_type"),
            trigger_event: r.get("trigger_event"), template_name: r.get("template_name"),
            template_body: r.get("template_body"), is_active: r.get("is_active"),
            created_at: r.get("created_at"), updated_at: r.get("updated_at"),
        })).collect()
    }

    pub async fn create_notification_template(&self, req: &CreateNotificationTemplateRequest) -> Result<OrderNotificationTemplate, String> {
        let id = Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        sqlx::query(
            "INSERT INTO order_notification_templates (id, brand_id, channel_type, trigger_event, \
             template_name, template_body, is_active, created_at, updated_at) \
             VALUES ($1, $2, $3, $4, $5, $6, 1, $7, $8)",
        )
        .bind(&id).bind(&req.brand_id).bind(&req.channel_type).bind(&req.trigger_event)
        .bind(&req.template_name).bind(&req.template_body).bind(&now).bind(&now)
        .execute(&self.pool).await
            .map_err(|e| format!("Failed to create notification template: {e}"))?;
        self.get_notification_template(&id).await
    }

    pub async fn delete_notification_template(&self, id: &str) -> Result<(), String> {
        sqlx::query("DELETE FROM order_notification_templates WHERE id = $1")
            .bind(id).execute(&self.pool).await
            .map_err(|e| format!("Failed to delete notification template: {e}"))?;
        Ok(())
    }

    async fn get_notification_template(&self, id: &str) -> Result<OrderNotificationTemplate, String> {
        let r = sqlx::query(
            "SELECT id, brand_id, channel_type, trigger_event, template_name, template_body, \
             is_active, created_at, updated_at FROM order_notification_templates WHERE id = $1",
        )
        .bind(id).fetch_one(&self.pool).await
            .map_err(|e| format!("Notification template not found: {e}"))?;
        Ok(OrderNotificationTemplate {
            id: r.get("id"), brand_id: r.get("brand_id"), channel_type: r.get("channel_type"),
            trigger_event: r.get("trigger_event"), template_name: r.get("template_name"),
            template_body: r.get("template_body"), is_active: r.get("is_active"),
            created_at: r.get("created_at"), updated_at: r.get("updated_at"),
        })
    }

    // -----------------------------------------------------------------------
    // Gap #6: Payment Gateways
    // -----------------------------------------------------------------------

    pub async fn list_payment_gateways(&self, brand_id: &str) -> Result<Vec<PaymentGateway>, String> {
        let rows = sqlx::query(
            "SELECT id, brand_id, name, provider, credentials, currency, is_active, sort_order, created_at, updated_at \
             FROM payment_gateways WHERE brand_id = $1 ORDER BY sort_order, name",
        )
        .bind(brand_id).fetch_all(&self.pool).await
            .map_err(|e| format!("Failed to list payment gateways: {e}"))?;
        rows.iter().map(|r| Ok(PaymentGateway {
            id: r.get("id"), brand_id: r.get("brand_id"), name: r.get("name"),
            provider: r.get("provider"), credentials: r.get("credentials"),
            currency: r.get("currency"), is_active: r.get("is_active"),
            sort_order: r.get("sort_order"), created_at: r.get("created_at"),
            updated_at: r.get("updated_at"),
        })).collect()
    }

    pub async fn create_payment_gateway(&self, req: &CreatePaymentGatewayRequest) -> Result<PaymentGateway, String> {
        let id = Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        let credentials = req.credentials.as_ref().map(|v| serde_json::to_string(v).unwrap_or_default());
        let currency = req.currency.clone().unwrap_or_else(|| "USD".to_string());
        let sort_order = req.sort_order.unwrap_or(0);
        sqlx::query(
            "INSERT INTO payment_gateways (id, brand_id, name, provider, credentials, currency, is_active, sort_order, created_at, updated_at) \
             VALUES ($1, $2, $3, $4, $5, $6, 1, $7, $8, $9)",
        )
        .bind(&id).bind(&req.brand_id).bind(&req.name).bind(&req.provider)
        .bind(&credentials).bind(&currency).bind(sort_order as i32).bind(&now).bind(&now)
        .execute(&self.pool).await
            .map_err(|e| format!("Failed to create payment gateway: {e}"))?;
        self.get_payment_gateway(&id).await
    }

    pub async fn delete_payment_gateway(&self, id: &str) -> Result<(), String> {
        sqlx::query("DELETE FROM payment_gateways WHERE id = $1")
            .bind(id).execute(&self.pool).await
            .map_err(|e| format!("Failed to delete payment gateway: {e}"))?;
        Ok(())
    }

    async fn get_payment_gateway(&self, id: &str) -> Result<PaymentGateway, String> {
        let r = sqlx::query(
            "SELECT id, brand_id, name, provider, credentials, currency, is_active, sort_order, created_at, updated_at \
             FROM payment_gateways WHERE id = $1",
        )
        .bind(id).fetch_one(&self.pool).await
            .map_err(|e| format!("Payment gateway not found: {e}"))?;
        Ok(PaymentGateway {
            id: r.get("id"), brand_id: r.get("brand_id"), name: r.get("name"),
            provider: r.get("provider"), credentials: r.get("credentials"),
            currency: r.get("currency"), is_active: r.get("is_active"),
            sort_order: r.get("sort_order"), created_at: r.get("created_at"),
            updated_at: r.get("updated_at"),
        })
    }

    // -----------------------------------------------------------------------
    // Gap #7: Service Types
    // -----------------------------------------------------------------------

    pub async fn list_service_types(&self, brand_id: &str) -> Result<Vec<ServiceType>, String> {
        let rows = sqlx::query(
            "SELECT id, brand_id, name, type, description, is_active, sort_order, created_at, updated_at \
             FROM service_types WHERE brand_id = $1 ORDER BY sort_order, name",
        )
        .bind(brand_id).fetch_all(&self.pool).await
            .map_err(|e| format!("Failed to list service types: {e}"))?;
        rows.iter().map(|r| Ok(ServiceType {
            id: r.get("id"), brand_id: r.get("brand_id"), name: r.get("name"),
            r#type: r.get("type"), description: r.get("description"),
            is_active: r.get("is_active"), sort_order: r.get("sort_order"),
            created_at: r.get("created_at"), updated_at: r.get("updated_at"),
        })).collect()
    }

    pub async fn create_service_type(&self, req: &CreateServiceTypeRequest) -> Result<ServiceType, String> {
        let id = Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        let sort_order = req.sort_order.unwrap_or(0);
        sqlx::query(
            "INSERT INTO service_types (id, brand_id, name, type, description, is_active, sort_order, created_at, updated_at) \
             VALUES ($1, $2, $3, $4, $5, 1, $6, $7, $8)",
        )
        .bind(&id).bind(&req.brand_id).bind(&req.name).bind(&req.r#type)
        .bind(&req.description).bind(sort_order as i32).bind(&now).bind(&now)
        .execute(&self.pool).await
            .map_err(|e| format!("Failed to create service type: {e}"))?;
        self.get_service_type(&id).await
    }

    pub async fn delete_service_type(&self, id: &str) -> Result<(), String> {
        sqlx::query("DELETE FROM service_types WHERE id = $1")
            .bind(id).execute(&self.pool).await
            .map_err(|e| format!("Failed to delete service type: {e}"))?;
        Ok(())
    }

    async fn get_service_type(&self, id: &str) -> Result<ServiceType, String> {
        let r = sqlx::query(
            "SELECT id, brand_id, name, type, description, is_active, sort_order, created_at, updated_at \
             FROM service_types WHERE id = $1",
        )
        .bind(id).fetch_one(&self.pool).await
            .map_err(|e| format!("Service type not found: {e}"))?;
        Ok(ServiceType {
            id: r.get("id"), brand_id: r.get("brand_id"), name: r.get("name"),
            r#type: r.get("type"), description: r.get("description"),
            is_active: r.get("is_active"), sort_order: r.get("sort_order"),
            created_at: r.get("created_at"), updated_at: r.get("updated_at"),
        })
    }

    // -----------------------------------------------------------------------
    // Gap #8: Delivery Schedules & Time Slots
    // -----------------------------------------------------------------------

    pub async fn list_delivery_schedules(&self, brand_id: &str) -> Result<Vec<DeliverySchedule>, String> {
        let rows = sqlx::query(
            "SELECT id, brand_id, day_of_week, start_time, end_time, cutoff_time, max_capacity, is_active, created_at, updated_at \
             FROM delivery_schedules WHERE brand_id = $1 ORDER BY day_of_week, start_time",
        )
        .bind(brand_id).fetch_all(&self.pool).await
            .map_err(|e| format!("Failed to list delivery schedules: {e}"))?;
        rows.iter().map(|r| Ok(DeliverySchedule {
            id: r.get("id"), brand_id: r.get("brand_id"), day_of_week: r.get("day_of_week"),
            start_time: r.get("start_time"), end_time: r.get("end_time"),
            cutoff_time: r.get("cutoff_time"), max_capacity: r.get("max_capacity"),
            is_active: r.get("is_active"), created_at: r.get("created_at"),
            updated_at: r.get("updated_at"),
        })).collect()
    }

    pub async fn create_delivery_schedule(&self, req: &CreateDeliveryScheduleRequest) -> Result<DeliverySchedule, String> {
        let id = Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        sqlx::query(
            "INSERT INTO delivery_schedules (id, brand_id, day_of_week, start_time, end_time, cutoff_time, max_capacity, is_active, created_at, updated_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, 1, $8, $9)",
        )
        .bind(&id).bind(&req.brand_id).bind(req.day_of_week as i32)
        .bind(&req.start_time).bind(&req.end_time).bind(&req.cutoff_time)
        .bind(&req.max_capacity).bind(&now).bind(&now)
        .execute(&self.pool).await
            .map_err(|e| format!("Failed to create delivery schedule: {e}"))?;
        self.get_delivery_schedule(&id).await
    }

    pub async fn delete_delivery_schedule(&self, id: &str) -> Result<(), String> {
        sqlx::query("DELETE FROM delivery_schedules WHERE id = $1")
            .bind(id).execute(&self.pool).await
            .map_err(|e| format!("Failed to delete delivery schedule: {e}"))?;
        Ok(())
    }

    async fn get_delivery_schedule(&self, id: &str) -> Result<DeliverySchedule, String> {
        let r = sqlx::query(
            "SELECT id, brand_id, day_of_week, start_time, end_time, cutoff_time, max_capacity, is_active, created_at, updated_at \
             FROM delivery_schedules WHERE id = $1",
        )
        .bind(id).fetch_one(&self.pool).await
            .map_err(|e| format!("Delivery schedule not found: {e}"))?;
        Ok(DeliverySchedule {
            id: r.get("id"), brand_id: r.get("brand_id"), day_of_week: r.get("day_of_week"),
            start_time: r.get("start_time"), end_time: r.get("end_time"),
            cutoff_time: r.get("cutoff_time"), max_capacity: r.get("max_capacity"),
            is_active: r.get("is_active"), created_at: r.get("created_at"),
            updated_at: r.get("updated_at"),
        })
    }

    pub async fn list_time_slots(&self, brand_id: &str, day_of_week: Option<i32>) -> Result<Vec<DeliveryTimeSlot>, String> {
        let query = if day_of_week.is_some() {
            "SELECT id, brand_id, day_of_week, slot_start, slot_end, label, max_orders, is_active, created_at \
             FROM delivery_time_slots WHERE brand_id = $1 AND day_of_week = $2 ORDER BY slot_start"
        } else {
            "SELECT id, brand_id, day_of_week, slot_start, slot_end, label, max_orders, is_active, created_at \
             FROM delivery_time_slots WHERE brand_id = $1 ORDER BY day_of_week, slot_start"
        };
        let rows = if let Some(day) = day_of_week {
            sqlx::query(query).bind(brand_id).bind(day as i32).fetch_all(&self.pool).await
        } else {
            sqlx::query(query).bind(brand_id).fetch_all(&self.pool).await
        }.map_err(|e| format!("Failed to list time slots: {e}"))?;
        rows.iter().map(|r| Ok(DeliveryTimeSlot {
            id: r.get("id"), brand_id: r.get("brand_id"), day_of_week: r.get("day_of_week"),
            slot_start: r.get("slot_start"), slot_end: r.get("slot_end"),
            label: r.get("label"), max_orders: r.get("max_orders"),
            is_active: r.get("is_active"), created_at: r.get("created_at"),
        })).collect()
    }

    pub async fn create_time_slot(&self, req: &CreateDeliveryTimeSlotRequest) -> Result<DeliveryTimeSlot, String> {
        let id = Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        let max_orders = req.max_orders.unwrap_or(10);
        sqlx::query(
            "INSERT INTO delivery_time_slots (id, brand_id, day_of_week, slot_start, slot_end, label, max_orders, is_active, created_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, 1, $8)",
        )
        .bind(&id).bind(&req.brand_id).bind(req.day_of_week as i32)
        .bind(&req.slot_start).bind(&req.slot_end).bind(&req.label)
        .bind(max_orders as i32).bind(&now)
        .execute(&self.pool).await
            .map_err(|e| format!("Failed to create time slot: {e}"))?;
        self.get_time_slot(&id).await
    }

    pub async fn delete_time_slot(&self, id: &str) -> Result<(), String> {
        sqlx::query("DELETE FROM delivery_time_slots WHERE id = $1")
            .bind(id).execute(&self.pool).await
            .map_err(|e| format!("Failed to delete time slot: {e}"))?;
        Ok(())
    }

    async fn get_time_slot(&self, id: &str) -> Result<DeliveryTimeSlot, String> {
        let r = sqlx::query(
            "SELECT id, brand_id, day_of_week, slot_start, slot_end, label, max_orders, is_active, created_at \
             FROM delivery_time_slots WHERE id = $1",
        )
        .bind(id).fetch_one(&self.pool).await
            .map_err(|e| format!("Time slot not found: {e}"))?;
        Ok(DeliveryTimeSlot {
            id: r.get("id"), brand_id: r.get("brand_id"), day_of_week: r.get("day_of_week"),
            slot_start: r.get("slot_start"), slot_end: r.get("slot_end"),
            label: r.get("label"), max_orders: r.get("max_orders"),
            is_active: r.get("is_active"), created_at: r.get("created_at"),
        })
    }

    // -----------------------------------------------------------------------
    // Gap #9: Invoices
    // -----------------------------------------------------------------------

    pub async fn generate_invoice(&self, order_id: &str) -> Result<Invoice, String> {
        // Check if invoice already exists
        let existing = sqlx::query(
            "SELECT id FROM invoices WHERE order_id = $1",
        )
        .bind(order_id).fetch_optional(&self.pool).await
            .map_err(|e| format!("Failed to check invoice: {e}"))?;
        if let Some(row) = existing {
            let id: String = row.get("id");
            return self.get_invoice(&id).await;
        }

        let id = Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        // Generate invoice number from order
        let order_row = sqlx::query(
            "SELECT order_number FROM commerce_orders WHERE id = $1",
        )
        .bind(order_id).fetch_one(&self.pool).await
            .map_err(|e| format!("Order not found: {e}"))?;
        let order_number: String = order_row.get("order_number");
        let invoice_number = format!("INV-{}", order_number);

        sqlx::query(
            "INSERT INTO invoices (id, order_id, invoice_number, issued_at, status, created_at) \
             VALUES ($1, $2, $3, $4, 'issued', $5)",
        )
        .bind(&id).bind(order_id).bind(&invoice_number).bind(&now).bind(&now)
        .execute(&self.pool).await
            .map_err(|e| format!("Failed to create invoice: {e}"))?;
        self.get_invoice(&id).await
    }

    pub async fn get_invoice(&self, id: &str) -> Result<Invoice, String> {
        let r = sqlx::query(
            "SELECT id, order_id, invoice_number, pdf_url, issued_at, due_date, status, created_at \
             FROM invoices WHERE id = $1",
        )
        .bind(id).fetch_one(&self.pool).await
            .map_err(|e| format!("Invoice not found: {e}"))?;
        Ok(Invoice {
            id: r.get("id"), order_id: r.get("order_id"), invoice_number: r.get("invoice_number"),
            pdf_url: r.get("pdf_url"), issued_at: r.get("issued_at"),
            due_date: r.get("due_date"), status: r.get("status"), created_at: r.get("created_at"),
        })
    }

    // -----------------------------------------------------------------------
    // Gap #10: CSV Import
    // -----------------------------------------------------------------------

    pub async fn import_csv(&self, req: &ImportCsvRequest) -> Result<CsvImportJob, String> {
        let id = Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        sqlx::query(
            "INSERT INTO csv_import_jobs (id, brand_id, entity_type, file_url, status, created_at) \
             VALUES ($1, $2, $3, $4, 'processing', $5)",
        )
        .bind(&id).bind(&req.brand_id).bind(&req.entity_type)
        .bind(&req.file_url).bind(&now)
        .execute(&self.pool).await
            .map_err(|e| format!("Failed to create CSV import job: {e}"))?;
        self.get_csv_import_job(&id).await
    }

    pub async fn list_csv_import_jobs(&self, brand_id: &str) -> Result<Vec<CsvImportJob>, String> {
        let rows = sqlx::query(
            "SELECT id, brand_id, entity_type, file_url, status, total_rows, processed_rows, \
             error_count, errors, completed_at, created_at \
             FROM csv_import_jobs WHERE brand_id = $1 ORDER BY created_at DESC",
        )
        .bind(brand_id).fetch_all(&self.pool).await
            .map_err(|e| format!("Failed to list CSV import jobs: {e}"))?;
        rows.iter().map(|r| Ok(CsvImportJob {
            id: r.get("id"), brand_id: r.get("brand_id"), entity_type: r.get("entity_type"),
            file_url: r.get("file_url"), status: r.get("status"),
            total_rows: r.get("total_rows"), processed_rows: r.get("processed_rows"),
            error_count: r.get("error_count"), errors: r.get("errors"),
            completed_at: r.get("completed_at"), created_at: r.get("created_at"),
        })).collect()
    }

    async fn get_csv_import_job(&self, id: &str) -> Result<CsvImportJob, String> {
        let r = sqlx::query(
            "SELECT id, brand_id, entity_type, file_url, status, total_rows, processed_rows, \
             error_count, errors, completed_at, created_at FROM csv_import_jobs WHERE id = $1",
        )
        .bind(id).fetch_one(&self.pool).await
            .map_err(|e| format!("CSV import job not found: {e}"))?;
        Ok(CsvImportJob {
            id: r.get("id"), brand_id: r.get("brand_id"), entity_type: r.get("entity_type"),
            file_url: r.get("file_url"), status: r.get("status"),
            total_rows: r.get("total_rows"), processed_rows: r.get("processed_rows"),
            error_count: r.get("error_count"), errors: r.get("errors"),
            completed_at: r.get("completed_at"), created_at: r.get("created_at"),
        })
    }

    // -----------------------------------------------------------------------
    // Gap #11: Customer Accounts
    // -----------------------------------------------------------------------

    pub async fn create_customer_account(&self, customer_id: &str, req: &CreateCustomerAccountRequest) -> Result<CustomerAccount, String> {
        let id = Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        // Simple hash for demo — use proper bcrypt in production
        let password_hash = req.password.as_ref().map(|p| format!("${}", p));
        sqlx::query(
            "INSERT INTO customer_accounts (id, customer_id, email, phone, password_hash, created_at, updated_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7)",
        )
        .bind(&id).bind(customer_id).bind(&req.email).bind(&req.phone)
        .bind(&password_hash).bind(&now).bind(&now)
        .execute(&self.pool).await
            .map_err(|e| format!("Failed to create customer account: {e}"))?;
        self.get_customer_account(&id).await
    }

    pub async fn login(&self, req: &LoginRequest) -> Result<CustomerAccount, String> {
        let query = if req.email.is_some() {
            "SELECT ca.id, ca.customer_id, ca.email, ca.phone, ca.password_hash, ca.is_verified, \
             ca.last_login_at, ca.created_at, ca.updated_at \
             FROM customer_accounts ca WHERE ca.email = $1"
        } else {
            "SELECT ca.id, ca.customer_id, ca.email, ca.phone, ca.password_hash, ca.is_verified, \
             ca.last_login_at, ca.created_at, ca.updated_at \
             FROM customer_accounts ca WHERE ca.phone = $1"
        };
        let identifier = req.email.clone().unwrap_or_else(|| req.phone.clone().unwrap_or_default());
        let r = sqlx::query(query).bind(&identifier).fetch_one(&self.pool).await
            .map_err(|_| "Invalid credentials".to_string())?;

        let password_hash: Option<String> = r.get("password_hash");
        if let Some(hash) = &password_hash {
            if hash != &format!("${}", req.password) {
                return Err("Invalid credentials".to_string());
            }
        }

        let id: String = r.get("id");
        let now = chrono::Utc::now().to_rfc3339();
        sqlx::query("UPDATE customer_accounts SET last_login_at = $1 WHERE id = $2")
            .bind(&now).bind(&id).execute(&self.pool).await
            .map_err(|e| format!("Failed to update login: {e}"))?;

        Ok(CustomerAccount {
            id, customer_id: r.get("customer_id"), email: r.get("email"),
            phone: r.get("phone"), password_hash, is_verified: r.get("is_verified"),
            last_login_at: Some(now), created_at: r.get("created_at"),
            updated_at: r.get("updated_at"),
        })
    }

    async fn get_customer_account(&self, id: &str) -> Result<CustomerAccount, String> {
        let r = sqlx::query(
            "SELECT id, customer_id, email, phone, password_hash, is_verified, last_login_at, created_at, updated_at \
             FROM customer_accounts WHERE id = $1",
        )
        .bind(id).fetch_one(&self.pool).await
            .map_err(|e| format!("Customer account not found: {e}"))?;
        Ok(CustomerAccount {
            id: r.get("id"), customer_id: r.get("customer_id"), email: r.get("email"),
            phone: r.get("phone"), password_hash: r.get("password_hash"),
            is_verified: r.get("is_verified"), last_login_at: r.get("last_login_at"),
            created_at: r.get("created_at"), updated_at: r.get("updated_at"),
        })
    }

    // -----------------------------------------------------------------------
    // Gap #12: Customer Reviews
    // -----------------------------------------------------------------------

    pub async fn create_review(&self, req: &CreateReviewRequest) -> Result<CustomerReview, String> {
        if req.rating < 1 || req.rating > 5 {
            return Err("Rating must be between 1 and 5".to_string());
        }
        let id = Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        let is_public = req.is_public.unwrap_or(true);
        sqlx::query(
            "INSERT INTO customer_reviews (id, order_id, customer_id, product_id, rating, comment, is_public, status, created_at, updated_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, 'approved', $8, $9)",
        )
        .bind(&id).bind(&req.order_id).bind(&req.customer_id).bind(&req.product_id)
        .bind(req.rating as i32).bind(&req.comment).bind(is_public as bool)
        .bind(&now).bind(&now)
        .execute(&self.pool).await
            .map_err(|e| format!("Failed to create review: {e}"))?;
        self.get_review(&id).await
    }

    pub async fn list_reviews(&self, product_id: Option<&str>, approved_only: bool) -> Result<Vec<CustomerReview>, String> {
        let (query, rows) = if let Some(pid) = product_id {
            let q = if approved_only {
                "SELECT id, order_id, customer_id, product_id, rating, comment, is_public, status, created_at, updated_at \
                 FROM customer_reviews WHERE product_id = $1 AND status = 'approved' AND is_public = 1 ORDER BY created_at DESC"
            } else {
                "SELECT id, order_id, customer_id, product_id, rating, comment, is_public, status, created_at, updated_at \
                 FROM customer_reviews WHERE product_id = $1 ORDER BY created_at DESC"
            };
            (q, sqlx::query(q).bind(pid).fetch_all(&self.pool).await
                .map_err(|e| format!("Failed to list reviews: {e}"))?)
        } else {
            let q = if approved_only {
                "SELECT id, order_id, customer_id, product_id, rating, comment, is_public, status, created_at, updated_at \
                 FROM customer_reviews WHERE status = 'approved' AND is_public = 1 ORDER BY created_at DESC"
            } else {
                "SELECT id, order_id, customer_id, product_id, rating, comment, is_public, status, created_at, updated_at \
                 FROM customer_reviews ORDER BY created_at DESC"
            };
            (q, sqlx::query(q).fetch_all(&self.pool).await
                .map_err(|e| format!("Failed to list reviews: {e}"))?)
        };
        rows.iter().map(|r| Ok(CustomerReview {
            id: r.get("id"), order_id: r.get("order_id"), customer_id: r.get("customer_id"),
            product_id: r.get("product_id"), rating: r.get("rating"),
            comment: r.get("comment"), is_public: r.get("is_public"),
            status: r.get("status"), created_at: r.get("created_at"),
            updated_at: r.get("updated_at"),
        })).collect()
    }

    async fn get_review(&self, id: &str) -> Result<CustomerReview, String> {
        let r = sqlx::query(
            "SELECT id, order_id, customer_id, product_id, rating, comment, is_public, status, created_at, updated_at \
             FROM customer_reviews WHERE id = $1",
        )
        .bind(id).fetch_one(&self.pool).await
            .map_err(|e| format!("Review not found: {e}"))?;
        Ok(CustomerReview {
            id: r.get("id"), order_id: r.get("order_id"), customer_id: r.get("customer_id"),
            product_id: r.get("product_id"), rating: r.get("rating"),
            comment: r.get("comment"), is_public: r.get("is_public"),
            status: r.get("status"), created_at: r.get("created_at"),
            updated_at: r.get("updated_at"),
        })
    }

    // -----------------------------------------------------------------------
    // Gap #13: Loyalty/Rewards
    // -----------------------------------------------------------------------

    pub async fn list_loyalty_programs(&self, brand_id: &str) -> Result<Vec<LoyaltyProgram>, String> {
        let rows = sqlx::query(
            "SELECT id, brand_id, name, points_per_currency, points_expiry_days, redemption_rate, is_active, created_at, updated_at \
             FROM loyalty_programs WHERE brand_id = $1",
        )
        .bind(brand_id).fetch_all(&self.pool).await
            .map_err(|e| format!("Failed to list loyalty programs: {e}"))?;
        rows.iter().map(|r| Ok(LoyaltyProgram {
            id: r.get("id"), brand_id: r.get("brand_id"), name: r.get("name"),
            points_per_currency: r.get("points_per_currency"),
            points_expiry_days: r.get("points_expiry_days"),
            redemption_rate: r.get("redemption_rate"), is_active: r.get("is_active"),
            created_at: r.get("created_at"), updated_at: r.get("updated_at"),
        })).collect()
    }

    pub async fn create_loyalty_program(&self, req: &CreateLoyaltyProgramRequest) -> Result<LoyaltyProgram, String> {
        let id = Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        let points_per_currency = req.points_per_currency.unwrap_or(1.0);
        let points_expiry_days = req.points_expiry_days.unwrap_or(365);
        let redemption_rate = req.redemption_rate.unwrap_or(100.0);
        sqlx::query(
            "INSERT INTO loyalty_programs (id, brand_id, name, points_per_currency, points_expiry_days, redemption_rate, is_active, created_at, updated_at) \
             VALUES ($1, $2, $3, $4, $5, $6, 1, $7, $8)",
        )
        .bind(&id).bind(&req.brand_id).bind(&req.name)
        .bind(points_per_currency).bind(points_expiry_days as i32).bind(redemption_rate)
        .bind(&now).bind(&now)
        .execute(&self.pool).await
            .map_err(|e| format!("Failed to create loyalty program: {e}"))?;
        self.get_loyalty_program(&id).await
    }

    pub async fn get_customer_points(&self, customer_id: &str, brand_id: &str) -> Result<CustomerPoints, String> {
        // Create if not exists
        let existing = sqlx::query(
            "SELECT id FROM customer_points WHERE customer_id = $1 AND brand_id = $2",
        )
        .bind(customer_id).bind(brand_id).fetch_optional(&self.pool).await
            .map_err(|e| format!("Failed to check customer points: {e}"))?;

        if existing.is_none() {
            let id = Uuid::new_v4().to_string();
            let now = chrono::Utc::now().to_rfc3339();
            sqlx::query(
                "INSERT INTO customer_points (id, customer_id, brand_id, points_balance, points_earned, points_redeemed, updated_at) \
                 VALUES ($1, $2, $3, 0, 0, 0, $4)",
            )
            .bind(&id).bind(customer_id).bind(brand_id).bind(&now)
            .execute(&self.pool).await
                .map_err(|e| format!("Failed to create customer points: {e}"))?;
        }

        let r = sqlx::query(
            "SELECT id, customer_id, brand_id, points_balance, points_earned, points_redeemed, updated_at \
             FROM customer_points WHERE customer_id = $1 AND brand_id = $2",
        )
        .bind(customer_id).bind(brand_id).fetch_one(&self.pool).await
            .map_err(|e| format!("Customer points not found: {e}"))?;
        Ok(CustomerPoints {
            id: r.get("id"), customer_id: r.get("customer_id"), brand_id: r.get("brand_id"),
            points_balance: r.get("points_balance"), points_earned: r.get("points_earned"),
            points_redeemed: r.get("points_redeemed"), updated_at: r.get("updated_at"),
        })
    }

    pub async fn add_points(
        &self, customer_id: &str, brand_id: &str, points: i32, reference_type: &str, reference_id: &str, description: &str,
    ) -> Result<CustomerPoints, String> {
        let id = Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        let mut tx = self.pool.begin().await.map_err(|e| format!("Failed to begin transaction: {e}"))?;

        // Record transaction
        sqlx::query(
            "INSERT INTO points_transactions (id, customer_id, brand_id, points, transaction_type, reference_type, reference_id, description, created_at) \
             VALUES ($1, $2, $3, $4, 'earn', $5, $6, $7, $8)",
        )
        .bind(&id).bind(customer_id).bind(brand_id).bind(points as i32)
        .bind(reference_type).bind(reference_id).bind(description).bind(&now)
        .execute(&mut *tx).await
            .map_err(|e| format!("Failed to record points transaction: {e}"))?;

        // Update balance
        sqlx::query(
            "UPDATE customer_points SET points_balance = points_balance + $1, points_earned = points_earned + $2, updated_at = $3 \
             WHERE customer_id = $4 AND brand_id = $5",
        )
        .bind(points as i32).bind(points as i32).bind(&now).bind(customer_id).bind(brand_id)
        .execute(&mut *tx).await
            .map_err(|e| format!("Failed to update points balance: {e}"))?;

        tx.commit().await.map_err(|e| format!("Failed to commit: {e}"))?;
        self.get_customer_points(customer_id, brand_id).await
    }

    async fn get_loyalty_program(&self, id: &str) -> Result<LoyaltyProgram, String> {
        let r = sqlx::query(
            "SELECT id, brand_id, name, points_per_currency, points_expiry_days, redemption_rate, is_active, created_at, updated_at \
             FROM loyalty_programs WHERE id = $1",
        )
        .bind(id).fetch_one(&self.pool).await
            .map_err(|e| format!("Loyalty program not found: {e}"))?;
        Ok(LoyaltyProgram {
            id: r.get("id"), brand_id: r.get("brand_id"), name: r.get("name"),
            points_per_currency: r.get("points_per_currency"),
            points_expiry_days: r.get("points_expiry_days"),
            redemption_rate: r.get("redemption_rate"), is_active: r.get("is_active"),
            created_at: r.get("created_at"), updated_at: r.get("updated_at"),
        })
    }

    // -----------------------------------------------------------------------
    // Gap #14: Payment Reminders
    // -----------------------------------------------------------------------

    pub async fn create_payment_reminder(
        &self, order_id: &str, customer_id: &str, channel_type: &str, reminder_type: &str,
    ) -> Result<PaymentReminder, String> {
        let id = Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        sqlx::query(
            "INSERT INTO payment_reminders (id, order_id, customer_id, channel_type, reminder_type, status, created_at) \
             VALUES ($1, $2, $3, $4, $5, 'pending', $6)",
        )
        .bind(&id).bind(order_id).bind(customer_id).bind(channel_type)
        .bind(reminder_type).bind(&now)
        .execute(&self.pool).await
            .map_err(|e| format!("Failed to create payment reminder: {e}"))?;
        self.get_payment_reminder(&id).await
    }

    pub async fn list_payment_reminders(&self, order_id: Option<&str>) -> Result<Vec<PaymentReminder>, String> {
        let (query, rows) = if let Some(oid) = order_id {
            let q = "SELECT id, order_id, customer_id, channel_type, reminder_type, scheduled_at, sent_at, status, created_at \
                     FROM payment_reminders WHERE order_id = $1 ORDER BY created_at DESC";
            (q, sqlx::query(q).bind(oid).fetch_all(&self.pool).await
                .map_err(|e| format!("Failed to list payment reminders: {e}"))?)
        } else {
            let q = "SELECT id, order_id, customer_id, channel_type, reminder_type, scheduled_at, sent_at, status, created_at \
                     FROM payment_reminders ORDER BY created_at DESC";
            (q, sqlx::query(q).fetch_all(&self.pool).await
                .map_err(|e| format!("Failed to list payment reminders: {e}"))?)
        };
        rows.iter().map(|r| Ok(PaymentReminder {
            id: r.get("id"), order_id: r.get("order_id"), customer_id: r.get("customer_id"),
            channel_type: r.get("channel_type"), reminder_type: r.get("reminder_type"),
            scheduled_at: r.get("scheduled_at"), sent_at: r.get("sent_at"),
            status: r.get("status"), created_at: r.get("created_at"),
        })).collect()
    }

    async fn get_payment_reminder(&self, id: &str) -> Result<PaymentReminder, String> {
        let r = sqlx::query(
            "SELECT id, order_id, customer_id, channel_type, reminder_type, scheduled_at, sent_at, status, created_at \
             FROM payment_reminders WHERE id = $1",
        )
        .bind(id).fetch_one(&self.pool).await
            .map_err(|e| format!("Payment reminder not found: {e}"))?;
        Ok(PaymentReminder {
            id: r.get("id"), order_id: r.get("order_id"), customer_id: r.get("customer_id"),
            channel_type: r.get("channel_type"), reminder_type: r.get("reminder_type"),
            scheduled_at: r.get("scheduled_at"), sent_at: r.get("sent_at"),
            status: r.get("status"), created_at: r.get("created_at"),
        })
    }

    // -----------------------------------------------------------------------
    // Gap #15: Delivery Reminders
    // -----------------------------------------------------------------------

    pub async fn create_delivery_reminder(
        &self, order_id: &str, customer_id: &str, channel_type: &str, reminder_type: &str,
    ) -> Result<DeliveryReminder, String> {
        let id = Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        sqlx::query(
            "INSERT INTO delivery_reminders (id, order_id, customer_id, channel_type, reminder_type, status, created_at) \
             VALUES ($1, $2, $3, $4, $5, 'pending', $6)",
        )
        .bind(&id).bind(order_id).bind(customer_id).bind(channel_type)
        .bind(reminder_type).bind(&now)
        .execute(&self.pool).await
            .map_err(|e| format!("Failed to create delivery reminder: {e}"))?;
        self.get_delivery_reminder(&id).await
    }

    pub async fn list_delivery_reminders(&self, order_id: Option<&str>) -> Result<Vec<DeliveryReminder>, String> {
        let (query, rows) = if let Some(oid) = order_id {
            let q = "SELECT id, order_id, customer_id, channel_type, reminder_type, scheduled_at, sent_at, status, created_at \
                     FROM delivery_reminders WHERE order_id = $1 ORDER BY created_at DESC";
            (q, sqlx::query(q).bind(oid).fetch_all(&self.pool).await
                .map_err(|e| format!("Failed to list delivery reminders: {e}"))?)
        } else {
            let q = "SELECT id, order_id, customer_id, channel_type, reminder_type, scheduled_at, sent_at, status, created_at \
                     FROM delivery_reminders ORDER BY created_at DESC";
            (q, sqlx::query(q).fetch_all(&self.pool).await
                .map_err(|e| format!("Failed to list delivery reminders: {e}"))?)
        };
        rows.iter().map(|r| Ok(DeliveryReminder {
            id: r.get("id"), order_id: r.get("order_id"), customer_id: r.get("customer_id"),
            channel_type: r.get("channel_type"), reminder_type: r.get("reminder_type"),
            scheduled_at: r.get("scheduled_at"), sent_at: r.get("sent_at"),
            status: r.get("status"), created_at: r.get("created_at"),
        })).collect()
    }

    async fn get_delivery_reminder(&self, id: &str) -> Result<DeliveryReminder, String> {
        let r = sqlx::query(
            "SELECT id, order_id, customer_id, channel_type, reminder_type, scheduled_at, sent_at, status, created_at \
             FROM delivery_reminders WHERE id = $1",
        )
        .bind(id).fetch_one(&self.pool).await
            .map_err(|e| format!("Delivery reminder not found: {e}"))?;
        Ok(DeliveryReminder {
            id: r.get("id"), order_id: r.get("order_id"), customer_id: r.get("customer_id"),
            channel_type: r.get("channel_type"), reminder_type: r.get("reminder_type"),
            scheduled_at: r.get("scheduled_at"), sent_at: r.get("sent_at"),
            status: r.get("status"), created_at: r.get("created_at"),
        })
    }

    // -----------------------------------------------------------------------
    // Gap #16: Staff Management
    // -----------------------------------------------------------------------

    pub async fn list_staff_members(&self, brand_id: &str) -> Result<Vec<StaffMember>, String> {
        let rows = sqlx::query(
            "SELECT id, brand_id, name, email, role, permissions, is_active, last_login_at, created_at, updated_at \
             FROM staff_members WHERE brand_id = $1 ORDER BY name",
        )
        .bind(brand_id).fetch_all(&self.pool).await
            .map_err(|e| format!("Failed to list staff members: {e}"))?;
        rows.iter().map(|r| Ok(StaffMember {
            id: r.get("id"), brand_id: r.get("brand_id"), name: r.get("name"),
            email: r.get("email"), role: r.get("role"), permissions: r.get("permissions"),
            is_active: r.get("is_active"), last_login_at: r.get("last_login_at"),
            created_at: r.get("created_at"), updated_at: r.get("updated_at"),
        })).collect()
    }

    pub async fn create_staff_member(&self, req: &CreateStaffMemberRequest) -> Result<StaffMember, String> {
        let id = Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        let role = req.role.clone().unwrap_or_else(|| "viewer".to_string());
        let permissions = req.permissions.as_ref().map(|v| serde_json::to_string(v).unwrap_or_default());
        sqlx::query(
            "INSERT INTO staff_members (id, brand_id, name, email, role, permissions, is_active, created_at, updated_at) \
             VALUES ($1, $2, $3, $4, $5, $6, 1, $7, $8)",
        )
        .bind(&id).bind(&req.brand_id).bind(&req.name).bind(&req.email)
        .bind(&role).bind(&permissions).bind(&now).bind(&now)
        .execute(&self.pool).await
            .map_err(|e| format!("Failed to create staff member: {e}"))?;
        self.get_staff_member(&id).await
    }

    pub async fn delete_staff_member(&self, id: &str) -> Result<(), String> {
        sqlx::query("DELETE FROM staff_members WHERE id = $1")
            .bind(id).execute(&self.pool).await
            .map_err(|e| format!("Failed to delete staff member: {e}"))?;
        Ok(())
    }

    async fn get_staff_member(&self, id: &str) -> Result<StaffMember, String> {
        let r = sqlx::query(
            "SELECT id, brand_id, name, email, role, permissions, is_active, last_login_at, created_at, updated_at \
             FROM staff_members WHERE id = $1",
        )
        .bind(id).fetch_one(&self.pool).await
            .map_err(|e| format!("Staff member not found: {e}"))?;
        Ok(StaffMember {
            id: r.get("id"), brand_id: r.get("brand_id"), name: r.get("name"),
            email: r.get("email"), role: r.get("role"), permissions: r.get("permissions"),
            is_active: r.get("is_active"), last_login_at: r.get("last_login_at"),
            created_at: r.get("created_at"), updated_at: r.get("updated_at"),
        })
    }

    // -----------------------------------------------------------------------
    // Gap #17: POS Sessions
    // -----------------------------------------------------------------------

    pub async fn create_pos_session(&self, brand_id: &str, staff_id: Option<&str>) -> Result<PosSession, String> {
        let id = Uuid::new_v4().to_string();
        let token = Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        sqlx::query(
            "INSERT INTO pos_sessions (id, brand_id, staff_id, session_token, status, started_at) \
             VALUES ($1, $2, $3, $4, 'active', $5)",
        )
        .bind(&id).bind(brand_id).bind(staff_id).bind(&token).bind(&now)
        .execute(&self.pool).await
            .map_err(|e| format!("Failed to create POS session: {e}"))?;
        self.get_pos_session(&id).await
    }

    pub async fn end_pos_session(&self, id: &str) -> Result<(), String> {
        let now = chrono::Utc::now().to_rfc3339();
        sqlx::query(
            "UPDATE pos_sessions SET status = 'ended', ended_at = $1 WHERE id = $2",
        )
        .bind(&now).bind(id).execute(&self.pool).await
            .map_err(|e| format!("Failed to end POS session: {e}"))?;
        Ok(())
    }

    pub async fn list_pos_sessions(&self, brand_id: &str) -> Result<Vec<PosSession>, String> {
        let rows = sqlx::query(
            "SELECT id, brand_id, staff_id, session_token, status, started_at, ended_at \
             FROM pos_sessions WHERE brand_id = $1 ORDER BY started_at DESC",
        )
        .bind(brand_id).fetch_all(&self.pool).await
            .map_err(|e| format!("Failed to list POS sessions: {e}"))?;
        rows.iter().map(|r| Ok(PosSession {
            id: r.get("id"), brand_id: r.get("brand_id"), staff_id: r.get("staff_id"),
            session_token: r.get("session_token"), status: r.get("status"),
            started_at: r.get("started_at"), ended_at: r.get("ended_at"),
        })).collect()
    }

    async fn get_pos_session(&self, id: &str) -> Result<PosSession, String> {
        let r = sqlx::query(
            "SELECT id, brand_id, staff_id, session_token, status, started_at, ended_at \
             FROM pos_sessions WHERE id = $1",
        )
        .bind(id).fetch_one(&self.pool).await
            .map_err(|e| format!("POS session not found: {e}"))?;
        Ok(PosSession {
            id: r.get("id"), brand_id: r.get("brand_id"), staff_id: r.get("staff_id"),
            session_token: r.get("session_token"), status: r.get("status"),
            started_at: r.get("started_at"), ended_at: r.get("ended_at"),
        })
    }

    // -----------------------------------------------------------------------
    // Gap #18: Receipts
    // -----------------------------------------------------------------------

    pub async fn generate_receipt(&self, order_id: &str) -> Result<Receipt, String> {
        let existing = sqlx::query("SELECT id FROM receipts WHERE order_id = $1")
            .bind(order_id).fetch_optional(&self.pool).await
            .map_err(|e| format!("Failed to check receipt: {e}"))?;
        if let Some(row) = existing {
            let id: String = row.get("id");
            return self.get_receipt(&id).await;
        }

        let id = Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        let order_row = sqlx::query("SELECT order_number FROM commerce_orders WHERE id = $1")
            .bind(order_id).fetch_one(&self.pool).await
            .map_err(|e| format!("Order not found: {e}"))?;
        let order_number: String = order_row.get("order_number");
        let receipt_number = format!("RCP-{}", order_number);

        sqlx::query(
            "INSERT INTO receipts (id, order_id, receipt_number, created_at) VALUES ($1, $2, $3, $4)",
        )
        .bind(&id).bind(order_id).bind(&receipt_number).bind(&now)
        .execute(&self.pool).await
            .map_err(|e| format!("Failed to create receipt: {e}"))?;
        self.get_receipt(&id).await
    }

    async fn get_receipt(&self, id: &str) -> Result<Receipt, String> {
        let r = sqlx::query(
            "SELECT id, order_id, receipt_number, pdf_url, sent_via, sent_at, created_at \
             FROM receipts WHERE id = $1",
        )
        .bind(id).fetch_one(&self.pool).await
            .map_err(|e| format!("Receipt not found: {e}"))?;
        Ok(Receipt {
            id: r.get("id"), order_id: r.get("order_id"), receipt_number: r.get("receipt_number"),
            pdf_url: r.get("pdf_url"), sent_via: r.get("sent_via"),
            sent_at: r.get("sent_at"), created_at: r.get("created_at"),
        })
    }

    // -----------------------------------------------------------------------
    // Gap #19: Commerce Inbox Threads
    // -----------------------------------------------------------------------

    pub async fn list_inbox_threads(&self, brand_id: &str) -> Result<Vec<CommerceInboxThread>, String> {
        let rows = sqlx::query(
            "SELECT id, brand_id, customer_id, channel_type, last_message_at, unread_count, is_active, created_at, updated_at \
             FROM commerce_inbox_threads WHERE brand_id = $1 AND is_active = 1 ORDER BY last_message_at DESC",
        )
        .bind(brand_id).fetch_all(&self.pool).await
            .map_err(|e| format!("Failed to list inbox threads: {e}"))?;
        rows.iter().map(|r| Ok(CommerceInboxThread {
            id: r.get("id"), brand_id: r.get("brand_id"), customer_id: r.get("customer_id"),
            channel_type: r.get("channel_type"), last_message_at: r.get("last_message_at"),
            unread_count: r.get("unread_count"), is_active: r.get("is_active"),
            created_at: r.get("created_at"), updated_at: r.get("updated_at"),
        })).collect()
    }

    // -----------------------------------------------------------------------
    // Gap #20: Wholesale Pricing
    // -----------------------------------------------------------------------

    pub async fn list_wholesale_pricing(&self, product_id: &str) -> Result<Vec<WholesalePricing>, String> {
        let rows = sqlx::query(
            "SELECT id, product_id, customer_group, price, min_quantity, is_active, created_at, updated_at \
             FROM wholesale_pricing WHERE product_id = $1 AND is_active = 1",
        )
        .bind(product_id).fetch_all(&self.pool).await
            .map_err(|e| format!("Failed to list wholesale pricing: {e}"))?;
        rows.iter().map(|r| Ok(WholesalePricing {
            id: r.get("id"), product_id: r.get("product_id"),
            customer_group: r.get("customer_group"), price: r.get("price"),
            min_quantity: r.get("min_quantity"), is_active: r.get("is_active"),
            created_at: r.get("created_at"), updated_at: r.get("updated_at"),
        })).collect()
    }

    pub async fn create_wholesale_pricing(&self, req: &CreateWholesalePricingRequest) -> Result<WholesalePricing, String> {
        let id = Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        let min_quantity = req.min_quantity.unwrap_or(1);
        sqlx::query(
            "INSERT INTO wholesale_pricing (id, product_id, customer_group, price, min_quantity, is_active, created_at, updated_at) \
             VALUES ($1, $2, $3, $4, $5, 1, $6, $7)",
        )
        .bind(&id).bind(&req.product_id).bind(&req.customer_group)
        .bind(req.price).bind(min_quantity as i32).bind(&now).bind(&now)
        .execute(&self.pool).await
            .map_err(|e| format!("Failed to create wholesale pricing: {e}"))?;
        self.get_wholesale_pricing(&id).await
    }

    pub async fn delete_wholesale_pricing(&self, id: &str) -> Result<(), String> {
        sqlx::query("DELETE FROM wholesale_pricing WHERE id = $1")
            .bind(id).execute(&self.pool).await
            .map_err(|e| format!("Failed to delete wholesale pricing: {e}"))?;
        Ok(())
    }

    async fn get_wholesale_pricing(&self, id: &str) -> Result<WholesalePricing, String> {
        let r = sqlx::query(
            "SELECT id, product_id, customer_group, price, min_quantity, is_active, created_at, updated_at \
             FROM wholesale_pricing WHERE id = $1",
        )
        .bind(id).fetch_one(&self.pool).await
            .map_err(|e| format!("Wholesale pricing not found: {e}"))?;
        Ok(WholesalePricing {
            id: r.get("id"), product_id: r.get("product_id"),
            customer_group: r.get("customer_group"), price: r.get("price"),
            min_quantity: r.get("min_quantity"), is_active: r.get("is_active"),
            created_at: r.get("created_at"), updated_at: r.get("updated_at"),
        })
    }

    // -----------------------------------------------------------------------
    // Gap #21: Payment Fees
    // -----------------------------------------------------------------------

    pub async fn record_payment_fee(
        &self, order_id: &str, gateway_id: Option<&str>, fee_amount: f64, fee_percentage: f64, passed_to_customer: bool,
    ) -> Result<PaymentFee, String> {
        let id = Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        sqlx::query(
            "INSERT INTO payment_fees (id, order_id, gateway_id, fee_amount, fee_percentage, passed_to_customer, created_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7)",
        )
        .bind(&id).bind(order_id).bind(gateway_id).bind(fee_amount)
        .bind(fee_percentage).bind(passed_to_customer as bool).bind(&now)
        .execute(&self.pool).await
            .map_err(|e| format!("Failed to record payment fee: {e}"))?;
        self.get_payment_fee(&id).await
    }

    async fn get_payment_fee(&self, id: &str) -> Result<PaymentFee, String> {
        let r = sqlx::query(
            "SELECT id, order_id, gateway_id, fee_amount, fee_percentage, passed_to_customer, created_at \
             FROM payment_fees WHERE id = $1",
        )
        .bind(id).fetch_one(&self.pool).await
            .map_err(|e| format!("Payment fee not found: {e}"))?;
        Ok(PaymentFee {
            id: r.get("id"), order_id: r.get("order_id"), gateway_id: r.get("gateway_id"),
            fee_amount: r.get("fee_amount"), fee_percentage: r.get("fee_percentage"),
            passed_to_customer: r.get("passed_to_customer"), created_at: r.get("created_at"),
        })
    }

    // -----------------------------------------------------------------------
    // Gap #22: Payment Recovery
    // -----------------------------------------------------------------------

    pub async fn create_payment_recovery(
        &self, order_id: &str, customer_id: &str, channel_type: &str,
    ) -> Result<PaymentRecovery, String> {
        let id = Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        sqlx::query(
            "INSERT INTO payment_recovery (id, order_id, customer_id, channel_type, status, created_at) \
             VALUES ($1, $2, $3, $4, 'pending', $5)",
        )
        .bind(&id).bind(order_id).bind(customer_id).bind(channel_type).bind(&now)
        .execute(&self.pool).await
            .map_err(|e| format!("Failed to create payment recovery: {e}"))?;
        self.get_payment_recovery(&id).await
    }

    pub async fn attempt_recovery(&self, id: &str) -> Result<PaymentRecovery, String> {
        let now = chrono::Utc::now().to_rfc3339();
        sqlx::query(
            "UPDATE payment_recovery SET attempt_count = attempt_count + 1, last_attempt_at = $1 WHERE id = $2",
        )
        .bind(&now).bind(id).execute(&self.pool).await
            .map_err(|e| format!("Failed to attempt recovery: {e}"))?;
        self.get_payment_recovery(id).await
    }

    async fn get_payment_recovery(&self, id: &str) -> Result<PaymentRecovery, String> {
        let r = sqlx::query(
            "SELECT id, order_id, customer_id, channel_type, attempt_count, last_attempt_at, status, created_at \
             FROM payment_recovery WHERE id = $1",
        )
        .bind(id).fetch_one(&self.pool).await
            .map_err(|e| format!("Payment recovery not found: {e}"))?;
        Ok(PaymentRecovery {
            id: r.get("id"), order_id: r.get("order_id"), customer_id: r.get("customer_id"),
            channel_type: r.get("channel_type"), attempt_count: r.get("attempt_count"),
            last_attempt_at: r.get("last_attempt_at"), status: r.get("status"),
            created_at: r.get("created_at"),
        })
    }

    // -----------------------------------------------------------------------
    // Gap #23: Automated Follow-ups
    // -----------------------------------------------------------------------

    pub async fn list_followups(&self, brand_id: &str) -> Result<Vec<AutomatedFollowup>, String> {
        let rows = sqlx::query(
            "SELECT id, brand_id, name, trigger_type, trigger_days, channel_type, template_body, is_active, created_at, updated_at \
             FROM automated_followups WHERE brand_id = $1 ORDER BY name",
        )
        .bind(brand_id).fetch_all(&self.pool).await
            .map_err(|e| format!("Failed to list follow-ups: {e}"))?;
        rows.iter().map(|r| Ok(AutomatedFollowup {
            id: r.get("id"), brand_id: r.get("brand_id"), name: r.get("name"),
            trigger_type: r.get("trigger_type"), trigger_days: r.get("trigger_days"),
            channel_type: r.get("channel_type"), template_body: r.get("template_body"),
            is_active: r.get("is_active"), created_at: r.get("created_at"),
            updated_at: r.get("updated_at"),
        })).collect()
    }

    pub async fn create_followup(&self, req: &CreateFollowupRequest) -> Result<AutomatedFollowup, String> {
        let id = Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        let trigger_days = req.trigger_days.unwrap_or(0);
        sqlx::query(
            "INSERT INTO automated_followups (id, brand_id, name, trigger_type, trigger_days, channel_type, template_body, is_active, created_at, updated_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, 1, $8, $9)",
        )
        .bind(&id).bind(&req.brand_id).bind(&req.name).bind(&req.trigger_type)
        .bind(trigger_days as i32).bind(&req.channel_type).bind(&req.template_body)
        .bind(&now).bind(&now)
        .execute(&self.pool).await
            .map_err(|e| format!("Failed to create follow-up: {e}"))?;
        self.get_followup(&id).await
    }

    pub async fn delete_followup(&self, id: &str) -> Result<(), String> {
        sqlx::query("DELETE FROM automated_followups WHERE id = $1")
            .bind(id).execute(&self.pool).await
            .map_err(|e| format!("Failed to delete follow-up: {e}"))?;
        Ok(())
    }

    async fn get_followup(&self, id: &str) -> Result<AutomatedFollowup, String> {
        let r = sqlx::query(
            "SELECT id, brand_id, name, trigger_type, trigger_days, channel_type, template_body, is_active, created_at, updated_at \
             FROM automated_followups WHERE id = $1",
        )
        .bind(id).fetch_one(&self.pool).await
            .map_err(|e| format!("Follow-up not found: {e}"))?;
        Ok(AutomatedFollowup {
            id: r.get("id"), brand_id: r.get("brand_id"), name: r.get("name"),
            trigger_type: r.get("trigger_type"), trigger_days: r.get("trigger_days"),
            channel_type: r.get("channel_type"), template_body: r.get("template_body"),
            is_active: r.get("is_active"), created_at: r.get("created_at"),
            updated_at: r.get("updated_at"),
        })
    }

    // -----------------------------------------------------------------------
    // Gap #24: Custom Domains
    // -----------------------------------------------------------------------

    pub async fn list_custom_domains(&self, brand_id: &str) -> Result<Vec<CustomDomain>, String> {
        let rows = sqlx::query(
            "SELECT id, brand_id, domain, storefront_id, ssl_enabled, verified, dns_records, is_active, created_at, updated_at \
             FROM custom_domains WHERE brand_id = $1 ORDER BY domain",
        )
        .bind(brand_id).fetch_all(&self.pool).await
            .map_err(|e| format!("Failed to list custom domains: {e}"))?;
        rows.iter().map(|r| Ok(CustomDomain {
            id: r.get("id"), brand_id: r.get("brand_id"), domain: r.get("domain"),
            storefront_id: r.get("storefront_id"), ssl_enabled: r.get("ssl_enabled"),
            verified: r.get("verified"), dns_records: r.get("dns_records"),
            is_active: r.get("is_active"), created_at: r.get("created_at"),
            updated_at: r.get("updated_at"),
        })).collect()
    }

    pub async fn create_custom_domain(&self, req: &CreateCustomDomainRequest) -> Result<CustomDomain, String> {
        let id = Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        sqlx::query(
            "INSERT INTO custom_domains (id, brand_id, domain, storefront_id, ssl_enabled, verified, is_active, created_at, updated_at) \
             VALUES ($1, $2, $3, $4, 0, 0, 1, $5, $6)",
        )
        .bind(&id).bind(&req.brand_id).bind(&req.domain).bind(&req.storefront_id)
        .bind(&now).bind(&now)
        .execute(&self.pool).await
            .map_err(|e| format!("Failed to create custom domain: {e}"))?;
        self.get_custom_domain(&id).await
    }

    pub async fn verify_custom_domain(&self, id: &str) -> Result<CustomDomain, String> {
        let now = chrono::Utc::now().to_rfc3339();
        sqlx::query(
            "UPDATE custom_domains SET verified = 1, ssl_enabled = 1, updated_at = $1 WHERE id = $2",
        )
        .bind(&now).bind(id).execute(&self.pool).await
            .map_err(|e| format!("Failed to verify custom domain: {e}"))?;
        self.get_custom_domain(id).await
    }

    pub async fn delete_custom_domain(&self, id: &str) -> Result<(), String> {
        sqlx::query("DELETE FROM custom_domains WHERE id = $1")
            .bind(id).execute(&self.pool).await
            .map_err(|e| format!("Failed to delete custom domain: {e}"))?;
        Ok(())
    }

    async fn get_custom_domain(&self, id: &str) -> Result<CustomDomain, String> {
        let r = sqlx::query(
            "SELECT id, brand_id, domain, storefront_id, ssl_enabled, verified, dns_records, is_active, created_at, updated_at \
             FROM custom_domains WHERE id = $1",
        )
        .bind(id).fetch_one(&self.pool).await
            .map_err(|e| format!("Custom domain not found: {e}"))?;
        Ok(CustomDomain {
            id: r.get("id"), brand_id: r.get("brand_id"), domain: r.get("domain"),
            storefront_id: r.get("storefront_id"), ssl_enabled: r.get("ssl_enabled"),
            verified: r.get("verified"), dns_records: r.get("dns_records"),
            is_active: r.get("is_active"), created_at: r.get("created_at"),
            updated_at: r.get("updated_at"),
        })
    }

    // -----------------------------------------------------------------------
    // Gap #25: Storefront Pages
    // -----------------------------------------------------------------------

    pub async fn list_storefront_pages(&self, storefront_id: &str) -> Result<Vec<StorefrontPage>, String> {
        let rows = sqlx::query(
            "SELECT id, storefront_id, slug, title, content, sort_order, is_active, created_at, updated_at \
             FROM storefront_pages WHERE storefront_id = $1 ORDER BY sort_order, title",
        )
        .bind(storefront_id).fetch_all(&self.pool).await
            .map_err(|e| format!("Failed to list storefront pages: {e}"))?;
        rows.iter().map(|r| Ok(StorefrontPage {
            id: r.get("id"), storefront_id: r.get("storefront_id"), slug: r.get("slug"),
            title: r.get("title"), content: r.get("content"),
            sort_order: r.get("sort_order"), is_active: r.get("is_active"),
            created_at: r.get("created_at"), updated_at: r.get("updated_at"),
        })).collect()
    }

    pub async fn create_storefront_page(&self, req: &CreateStorefrontPageRequest) -> Result<StorefrontPage, String> {
        let id = Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        let sort_order = req.sort_order.unwrap_or(0);
        sqlx::query(
            "INSERT INTO storefront_pages (id, storefront_id, slug, title, content, sort_order, is_active, created_at, updated_at) \
             VALUES ($1, $2, $3, $4, $5, $6, 1, $7, $8)",
        )
        .bind(&id).bind(&req.storefront_id).bind(&req.slug).bind(&req.title)
        .bind(&req.content).bind(sort_order as i32).bind(&now).bind(&now)
        .execute(&self.pool).await
            .map_err(|e| format!("Failed to create storefront page: {e}"))?;
        self.get_storefront_page(&id).await
    }

    pub async fn delete_storefront_page(&self, id: &str) -> Result<(), String> {
        sqlx::query("DELETE FROM storefront_pages WHERE id = $1")
            .bind(id).execute(&self.pool).await
            .map_err(|e| format!("Failed to delete storefront page: {e}"))?;
        Ok(())
    }

    async fn get_storefront_page(&self, id: &str) -> Result<StorefrontPage, String> {
        let r = sqlx::query(
            "SELECT id, storefront_id, slug, title, content, sort_order, is_active, created_at, updated_at \
             FROM storefront_pages WHERE id = $1",
        )
        .bind(id).fetch_one(&self.pool).await
            .map_err(|e| format!("Storefront page not found: {e}"))?;
        Ok(StorefrontPage {
            id: r.get("id"), storefront_id: r.get("storefront_id"), slug: r.get("slug"),
            title: r.get("title"), content: r.get("content"),
            sort_order: r.get("sort_order"), is_active: r.get("is_active"),
            created_at: r.get("created_at"), updated_at: r.get("updated_at"),
        })
    }

    // -----------------------------------------------------------------------
    // Gap #27: Meta Pixels
    // -----------------------------------------------------------------------

    pub async fn list_meta_pixels(&self, brand_id: &str) -> Result<Vec<MetaPixel>, String> {
        let rows = sqlx::query(
            "SELECT id, brand_id, pixel_id, access_token, events, is_active, created_at, updated_at \
             FROM meta_pixels WHERE brand_id = $1",
        )
        .bind(brand_id).fetch_all(&self.pool).await
            .map_err(|e| format!("Failed to list meta pixels: {e}"))?;
        rows.iter().map(|r| Ok(MetaPixel {
            id: r.get("id"), brand_id: r.get("brand_id"), pixel_id: r.get("pixel_id"),
            access_token: r.get("access_token"), events: r.get("events"),
            is_active: r.get("is_active"), created_at: r.get("created_at"),
            updated_at: r.get("updated_at"),
        })).collect()
    }

    pub async fn create_meta_pixel(&self, req: &CreateMetaPixelRequest) -> Result<MetaPixel, String> {
        let id = Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        let events = req.events.as_ref().map(|v| serde_json::to_string(v).unwrap_or_default());
        sqlx::query(
            "INSERT INTO meta_pixels (id, brand_id, pixel_id, access_token, events, is_active, created_at, updated_at) \
             VALUES ($1, $2, $3, $4, $5, 1, $6, $7)",
        )
        .bind(&id).bind(&req.brand_id).bind(&req.pixel_id).bind(&req.access_token)
        .bind(&events).bind(&now).bind(&now)
        .execute(&self.pool).await
            .map_err(|e| format!("Failed to create meta pixel: {e}"))?;
        self.get_meta_pixel(&id).await
    }

    pub async fn delete_meta_pixel(&self, id: &str) -> Result<(), String> {
        sqlx::query("DELETE FROM meta_pixels WHERE id = $1")
            .bind(id).execute(&self.pool).await
            .map_err(|e| format!("Failed to delete meta pixel: {e}"))?;
        Ok(())
    }

    async fn get_meta_pixel(&self, id: &str) -> Result<MetaPixel, String> {
        let r = sqlx::query(
            "SELECT id, brand_id, pixel_id, access_token, events, is_active, created_at, updated_at \
             FROM meta_pixels WHERE id = $1",
        )
        .bind(id).fetch_one(&self.pool).await
            .map_err(|e| format!("Meta pixel not found: {e}"))?;
        Ok(MetaPixel {
            id: r.get("id"), brand_id: r.get("brand_id"), pixel_id: r.get("pixel_id"),
            access_token: r.get("access_token"), events: r.get("events"),
            is_active: r.get("is_active"), created_at: r.get("created_at"),
            updated_at: r.get("updated_at"),
        })
    }

    // -----------------------------------------------------------------------
    // Gap #29: Checkout Questions
    // -----------------------------------------------------------------------

    pub async fn list_checkout_questions(&self, brand_id: &str) -> Result<Vec<CheckoutQuestion>, String> {
        let rows = sqlx::query(
            "SELECT id, brand_id, question, question_type, options, is_required, sort_order, is_active, created_at, updated_at \
             FROM checkout_questions WHERE brand_id = $1 AND is_active = 1 ORDER BY sort_order",
        )
        .bind(brand_id).fetch_all(&self.pool).await
            .map_err(|e| format!("Failed to list checkout questions: {e}"))?;
        rows.iter().map(|r| Ok(CheckoutQuestion {
            id: r.get("id"), brand_id: r.get("brand_id"), question: r.get("question"),
            question_type: r.get("question_type"), options: r.get("options"),
            is_required: r.get("is_required"), sort_order: r.get("sort_order"),
            is_active: r.get("is_active"), created_at: r.get("created_at"),
            updated_at: r.get("updated_at"),
        })).collect()
    }

    pub async fn create_checkout_question(&self, req: &CreateCheckoutQuestionRequest) -> Result<CheckoutQuestion, String> {
        let id = Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        let question_type = req.question_type.clone().unwrap_or_else(|| "text".to_string());
        let is_required = req.is_required.unwrap_or(false);
        let sort_order = req.sort_order.unwrap_or(0);
        let options = req.options.as_ref().map(|v| serde_json::to_string(v).unwrap_or_default());
        sqlx::query(
            "INSERT INTO checkout_questions (id, brand_id, question, question_type, options, is_required, sort_order, is_active, created_at, updated_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, 1, $8, $9)",
        )
        .bind(&id).bind(&req.brand_id).bind(&req.question).bind(&question_type)
        .bind(&options).bind(is_required as bool).bind(sort_order as i32)
        .bind(&now).bind(&now)
        .execute(&self.pool).await
            .map_err(|e| format!("Failed to create checkout question: {e}"))?;
        self.get_checkout_question(&id).await
    }

    pub async fn submit_checkout_answers(&self, req: &SubmitCheckoutAnswersRequest) -> Result<Vec<CheckoutQuestionAnswer>, String> {
        let now = chrono::Utc::now().to_rfc3339();
        let mut results = Vec::new();
        for answer in &req.answers {
            let id = Uuid::new_v4().to_string();
            sqlx::query(
                "INSERT INTO checkout_question_answers (id, order_id, question_id, answer, created_at) \
                 VALUES ($1, $2, $3, $4, $5)",
            )
            .bind(&id).bind(&req.order_id).bind(&answer.question_id)
            .bind(&answer.answer).bind(&now)
            .execute(&self.pool).await
                .map_err(|e| format!("Failed to submit answer: {e}"))?;
            results.push(CheckoutQuestionAnswer {
                id, order_id: req.order_id.clone(),
                question_id: answer.question_id.clone(),
                answer: answer.answer.clone(), created_at: now.clone(),
            });
        }
        Ok(results)
    }

    pub async fn delete_checkout_question(&self, id: &str) -> Result<(), String> {
        sqlx::query("DELETE FROM checkout_questions WHERE id = $1")
            .bind(id).execute(&self.pool).await
            .map_err(|e| format!("Failed to delete checkout question: {e}"))?;
        Ok(())
    }

    async fn get_checkout_question(&self, id: &str) -> Result<CheckoutQuestion, String> {
        let r = sqlx::query(
            "SELECT id, brand_id, question, question_type, options, is_required, sort_order, is_active, created_at, updated_at \
             FROM checkout_questions WHERE id = $1",
        )
        .bind(id).fetch_one(&self.pool).await
            .map_err(|e| format!("Checkout question not found: {e}"))?;
        Ok(CheckoutQuestion {
            id: r.get("id"), brand_id: r.get("brand_id"), question: r.get("question"),
            question_type: r.get("question_type"), options: r.get("options"),
            is_required: r.get("is_required"), sort_order: r.get("sort_order"),
            is_active: r.get("is_active"), created_at: r.get("created_at"),
            updated_at: r.get("updated_at"),
        })
    }

    // -----------------------------------------------------------------------
    // Gap #30: Delivery Proofs
    // -----------------------------------------------------------------------

    pub async fn upload_delivery_proof(&self, req: &UploadDeliveryProofRequest) -> Result<DeliveryProof, String> {
        let id = Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        sqlx::query(
            "INSERT INTO delivery_proofs (id, shipment_id, order_id, file_url, file_name, uploaded_by, uploaded_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7)",
        )
        .bind(&id).bind(&req.shipment_id).bind(&req.order_id)
        .bind(&req.file_url).bind(&req.file_name).bind(&req.uploaded_by).bind(&now)
        .execute(&self.pool).await
            .map_err(|e| format!("Failed to upload delivery proof: {e}"))?;
        self.get_delivery_proof(&id).await
    }

    pub async fn list_delivery_proofs(&self, order_id: &str) -> Result<Vec<DeliveryProof>, String> {
        let rows = sqlx::query(
            "SELECT id, shipment_id, order_id, file_url, file_name, uploaded_by, uploaded_at \
             FROM delivery_proofs WHERE order_id = $1 ORDER BY uploaded_at DESC",
        )
        .bind(order_id).fetch_all(&self.pool).await
            .map_err(|e| format!("Failed to list delivery proofs: {e}"))?;
        rows.iter().map(|r| Ok(DeliveryProof {
            id: r.get("id"), shipment_id: r.get("shipment_id"), order_id: r.get("order_id"),
            file_url: r.get("file_url"), file_name: r.get("file_name"),
            uploaded_by: r.get("uploaded_by"), uploaded_at: r.get("uploaded_at"),
        })).collect()
    }

    async fn get_delivery_proof(&self, id: &str) -> Result<DeliveryProof, String> {
        let r = sqlx::query(
            "SELECT id, shipment_id, order_id, file_url, file_name, uploaded_by, uploaded_at \
             FROM delivery_proofs WHERE id = $1",
        )
        .bind(id).fetch_one(&self.pool).await
            .map_err(|e| format!("Delivery proof not found: {e}"))?;
        Ok(DeliveryProof {
            id: r.get("id"), shipment_id: r.get("shipment_id"), order_id: r.get("order_id"),
            file_url: r.get("file_url"), file_name: r.get("file_name"),
            uploaded_by: r.get("uploaded_by"), uploaded_at: r.get("uploaded_at"),
        })
    }
}
