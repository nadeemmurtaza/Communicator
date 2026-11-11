use sqlx::{Pool, Postgres};
use uuid::Uuid;
use chrono::Utc;
use crate::models::*;

pub struct ErpPaymentsService;

impl ErpPaymentsService {
    // ============================================================
    // Payments
    // ============================================================

    pub async fn list_payments(
        pool: &Pool<Postgres>,
        order_id: Option<&str>,
        status: Option<&str>,
        provider: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<ErpPayment>, String> {
        let mut query = String::from("SELECT * FROM erp_payments WHERE 1=1");
        let mut param_idx = 1;

        if let Some(o) = order_id {
            query.push_str(&format!(" AND order_id = ${}", param_idx));
            param_idx += 1;
        }

        if let Some(s) = status {
            query.push_str(&format!(" AND status = ${}", param_idx));
            param_idx += 1;
        }

        if let Some(p) = provider {
            query.push_str(&format!(" AND provider = ${}", param_idx));
            param_idx += 1;
        }

        let limit_p = param_idx;
        param_idx += 1;
        let offset_p = param_idx;
        query.push_str(&format!(" ORDER BY created_at DESC LIMIT ${} OFFSET ${}", limit_p, offset_p));

        let payments = sqlx::query_as::<_, ErpPayment>(&query)
            .bind(limit)
            .bind(offset)
            .fetch_all(pool)
            .await
            .map_err(|e| e.to_string())?;

        Ok(payments)
    }

    pub async fn get_payment(pool: &Pool<Postgres>, id: &str) -> Result<ErpPayment, String> {
        let payment = sqlx::query_as::<_, ErpPayment>(
            "SELECT * FROM erp_payments WHERE id = $1"
        )
        .fetch_one(pool)
        .await
        .map_err(|e| e.to_string())?;
        Ok(payment)
    }

    pub async fn get_payments_by_order(pool: &Pool<Postgres>, order_id: &str) -> Result<Vec<ErpPayment>, String> {
        let payments = sqlx::query_as::<_, ErpPayment>(
            "SELECT * FROM erp_payments WHERE order_id = $1 ORDER BY created_at DESC"
        )
        .bind(order_id)
        .fetch_all(pool)
        .await
        .map_err(|e| e.to_string())?;
        Ok(payments)
    }

    pub async fn get_payment_by_reference(pool: &Pool<Postgres>, reference: &str) -> Result<ErpPayment, String> {
        let payment = sqlx::query_as::<_, ErpPayment>(
            "SELECT * FROM erp_payments WHERE reference = $1"
        )
        .fetch_one(pool)
        .await
        .map_err(|e| e.to_string())?;
        Ok(payment)
    }

    pub async fn create_payment(
        pool: &Pool<Postgres>,
        req: &CreatePaymentRequest,
    ) -> Result<ErpPayment, String> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();
        let reference = req.reference.clone().unwrap_or_else(|| format!("PAY-{}", Uuid::new_v4().hyphenated().to_string().chars().take(8).collect::<String>()));
        let currency = req.currency.clone().unwrap_or_else(|| "PKR".to_string());

        // Verify order exists
        let _order_exists: (i32,) = sqlx::query_as(
            "SELECT COUNT(*) FROM erp_orders WHERE id = $1"
        )
        .bind(&req.order_id)
        .fetch_one(pool)
        .await
        .map_err(|e| e.to_string())?;

        if _order_exists.0 == 0 {
            return Err(format!("Order {} not found", req.order_id));
        }

        let meta_json = match &req.meta {
            Some(m) => m.to_string(),
            None => "{}".to_string(),
        };

        let payment = sqlx::query_as::<_, ErpPayment>(
            "INSERT INTO erp_payments (id, order_id, provider, status, amount, currency, reference, meta, created_at, updated_at)
             VALUES ($1, $2, $3, 'PENDING', $4, $5, $6, $7::jsonb, $8, $9)
             RETURNING *"
        )
        .bind(&id)
        .bind(&req.order_id)
        .bind(&req.provider)
        .bind(req.amount)
        .bind(&currency)
        .bind(&reference)
        .bind(&meta_json)
        .bind(&now)
        .bind(&now)
        .fetch_one(pool)
        .await
        .map_err(|e| e.to_string())?;

        Ok(payment)
    }

    pub async fn update_payment_status(
        pool: &Pool<Postgres>,
        id: &str,
        req: &UpdatePaymentStatusRequest,
    ) -> Result<ErpPayment, String> {
        let now = Utc::now().to_rfc3339();

        // Use CASE expressions for conditional timestamp updates
        let payment = sqlx::query_as::<_, ErpPayment>(
            "UPDATE erp_payments SET
                status = $1,
                provider_ref = COALESCE($2, provider_ref),
                receipt_url = COALESCE($3, receipt_url),
                failure_reason = COALESCE($4, failure_reason),
                approved_by_id = COALESCE($5, approved_by_id),
                paid_at = CASE WHEN $1 = 'PAID' THEN $now ELSE paid_at END,
                refunded_at = CASE WHEN $1 = 'REFUNDED' THEN $now ELSE refunded_at END,
                updated_at = $now
            WHERE id = $6 RETURNING *"
        )
        .bind(&req.status)
        .bind(&req.provider_ref)
        .bind(&req.receipt_url)
        .bind(&req.failure_reason)
        .bind(&req.approved_by_id)
        .bind(&now)
        .bind(id)
        .fetch_one(pool)
        .await
        .map_err(|e| e.to_string())?;

        Ok(payment)
    }

    pub async fn mark_as_paid(
        pool: &Pool<Postgres>,
        id: &str,
        provider_ref: Option<&str>,
        approved_by_id: Option<&str>,
    ) -> Result<ErpPayment, String> {
        let now = Utc::now().to_rfc3339();

        let payment = sqlx::query_as::<_, ErpPayment>(
            "UPDATE erp_payments SET status = 'PAID', paid_at = $1, provider_ref = COALESCE($2, provider_ref), approved_by_id = $3, updated_at = $4 WHERE id = $5 RETURNING *"
        )
        .bind(&now)
        .bind(provider_ref)
        .bind(approved_by_id)
        .bind(&now)
        .bind(id)
        .fetch_one(pool)
        .await
        .map_err(|e| e.to_string())?;

        Ok(payment)
    }

    pub async fn mark_as_failed(
        pool: &Pool<Postgres>,
        id: &str,
        failure_reason: &str,
    ) -> Result<ErpPayment, String> {
        let now = Utc::now().to_rfc3339();

        let payment = sqlx::query_as::<_, ErpPayment>(
            "UPDATE erp_payments SET status = 'FAILED', failure_reason = $1, updated_at = $2 WHERE id = $3 RETURNING *"
        )
        .bind(failure_reason)
        .bind(&now)
        .bind(id)
        .fetch_one(pool)
        .await
        .map_err(|e| e.to_string())?;

        Ok(payment)
    }

    pub async fn refund_payment(
        pool: &Pool<Postgres>,
        id: &str,
        req: &RefundPaymentRequest,
    ) -> Result<ErpPayment, String> {
        let now = Utc::now().to_rfc3339();

        // Verify payment exists and is paid
        let payment = Self::get_payment(pool, id).await?;
        if payment.status != "PAID" {
            return Err(format!("Payment is not in PAID status (current: {})", payment.status));
        }

        if req.refund_amount > payment.amount {
            return Err(format!("Refund amount ({}) exceeds payment amount ({})", req.refund_amount, payment.amount));
        }

        let meta_json = match payment.meta {
            Some(ref m) => m.to_string(),
            None => "{}".to_string(),
        };

        let payment = sqlx::query_as::<_, ErpPayment>(
            "UPDATE erp_payments SET status = 'REFUNDED', refunded_at = $1, refund_amount = $2, meta = $3::jsonb, updated_at = $4 WHERE id = $5 RETURNING *"
        )
        .bind(&now)
        .bind(req.refund_amount)
        .bind(&meta_json)
        .bind(&now)
        .bind(id)
        .fetch_one(pool)
        .await
        .map_err(|e| e.to_string())?;

        Ok(payment)
    }

    pub async fn upload_receipt(
        pool: &Pool<Postgres>,
        id: &str,
        req: &UploadReceiptRequest,
    ) -> Result<ErpPayment, String> {
        let now = Utc::now().to_rfc3339();

        let meta_json = match &req.meta {
            Some(m) => m.to_string(),
            None => "{}".to_string(),
        };

        let payment = sqlx::query_as::<_, ErpPayment>(
            "UPDATE erp_payments SET status = 'AWAITING_APPROVAL', receipt_url = $1, meta = COALESCE($2::jsonb, meta), updated_at = $3 WHERE id = $4 RETURNING *"
        )
        .bind(&req.receipt_url)
        .bind(&meta_json)
        .bind(&now)
        .bind(id)
        .fetch_one(pool)
        .await
        .map_err(|e| e.to_string())?;

        Ok(payment)
    }

    pub async fn approve_payment(
        pool: &Pool<Postgres>,
        id: &str,
        approved_by_id: &str,
    ) -> Result<ErpPayment, String> {
        let now = Utc::now().to_rfc3339();

        let payment = sqlx::query_as::<_, ErpPayment>(
            "UPDATE erp_payments SET status = 'PAID', paid_at = $1, approved_by_id = $2, updated_at = $3 WHERE id = $4 RETURNING *"
        )
        .bind(&now)
        .bind(approved_by_id)
        .bind(&now)
        .bind(id)
        .fetch_one(pool)
        .await
        .map_err(|e| e.to_string())?;

        Ok(payment)
    }

    pub async fn reject_payment(
        pool: &Pool<Postgres>,
        id: &str,
        failure_reason: &str,
    ) -> Result<ErpPayment, String> {
        let now = Utc::now().to_rfc3339();

        let payment = sqlx::query_as::<_, ErpPayment>(
            "UPDATE erp_payments SET status = 'FAILED', failure_reason = $1, updated_at = $2 WHERE id = $3 RETURNING *"
        )
        .bind(failure_reason)
        .bind(&now)
        .bind(id)
        .fetch_one(pool)
        .await
        .map_err(|e| e.to_string())?;

        Ok(payment)
    }

    pub async fn get_pending_approvals(
        pool: &Pool<Postgres>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<ErpPayment>, String> {
        let payments = sqlx::query_as::<_, ErpPayment>(
            "SELECT * FROM erp_payments WHERE status = 'AWAITING_APPROVAL' ORDER BY created_at DESC LIMIT $1 OFFSET $2"
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await
        .map_err(|e| e.to_string())?;

        Ok(payments)
    }

    pub async fn get_payment_stats(pool: &Pool<Postgres>) -> Result<serde_json::Value, String> {
        let stats: Vec<(String, i64, i64)> = sqlx::query_as(
            "SELECT status, COUNT(*)::bigint, SUM(amount)::bigint FROM erp_payments GROUP BY status"
        )
        .fetch_all(pool)
        .await
        .map_err(|e| e.to_string())?;

        let mut result = serde_json::Map::new();
        let mut total_amount: i64 = 0;
        let mut total_count: i64 = 0;

        for (status, count, amount) in &stats {
            result.insert(status.clone(), serde_json::json!({"count": count, "amount": amount}));
            total_amount += amount;
            total_count += count;
        }

        result.insert("total".to_string(), serde_json::json!({"count": total_count, "amount": total_amount}));

        Ok(serde_json::Value::Object(result))
    }

    // ============================================================
    // Payment Methods
    // ============================================================

    pub async fn list_payment_methods(pool: &Pool<Postgres>) -> Result<Vec<ErpPaymentMethod>, String> {
        let methods = sqlx::query_as::<_, ErpPaymentMethod>(
            "SELECT * FROM erp_payment_methods ORDER BY name"
        )
        .fetch_all(pool)
        .await
        .map_err(|e| e.to_string())?;
        Ok(methods)
    }

    pub async fn get_payment_method(pool: &Pool<Postgres>, id: &str) -> Result<ErpPaymentMethod, String> {
        let method = sqlx::query_as::<_, ErpPaymentMethod>(
            "SELECT * FROM erp_payment_methods WHERE id = $1"
        )
        .fetch_one(pool)
        .await
        .map_err(|e| e.to_string())?;
        Ok(method)
    }

    pub async fn get_payment_method_by_code(pool: &Pool<Postgres>, code: &str) -> Result<ErpPaymentMethod, String> {
        let method = sqlx::query_as::<_, ErpPaymentMethod>(
            "SELECT * FROM erp_payment_methods WHERE code = $1"
        )
        .fetch_one(pool)
        .await
        .map_err(|e| e.to_string())?;
        Ok(method)
    }

    pub async fn create_payment_method(
        pool: &Pool<Postgres>,
        req: &CreatePaymentMethodRequest,
    ) -> Result<ErpPaymentMethod, String> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();
        let config_json = match &req.config {
            Some(c) => c.to_string(),
            None => "{}".to_string(),
        };

        let method = sqlx::query_as::<_, ErpPaymentMethod>(
            "INSERT INTO erp_payment_methods (id, name, code, config, is_active, created_at, updated_at)
             VALUES ($1, $2, $3, $4::jsonb, true, $5, $6)
             RETURNING *"
        )
        .bind(&id)
        .bind(&req.name)
        .bind(&req.code)
        .bind(&config_json)
        .bind(&now)
        .bind(&now)
        .fetch_one(pool)
        .await
        .map_err(|e| e.to_string())?;

        Ok(method)
    }

    pub async fn update_payment_method(
        pool: &Pool<Postgres>,
        id: &str,
        req: &UpdatePaymentMethodRequest,
    ) -> Result<ErpPaymentMethod, String> {
        let now = Utc::now().to_rfc3339();

        let mut set_clauses = vec![];
        let mut param_idx = 1;

        if let Some(ref name) = req.name {
            set_clauses.push(format!("name = ${}", param_idx));
            param_idx += 1;
        }

        if let Some(ref config) = req.config {
            set_clauses.push(format!("config = ${}::jsonb", param_idx));
            param_idx += 1;
        }

        if let Some(is_active) = req.is_active {
            set_clauses.push(format!("is_active = ${}", param_idx));
            param_idx += 1;
        }

        if set_clauses.is_empty() {
            return Self::get_payment_method(pool, id).await;
        }

        set_clauses.push(format!("updated_at = ${}", param_idx));
        let id_param = param_idx + 1;

        let method = sqlx::query_as::<_, ErpPaymentMethod>(
            &format!("UPDATE erp_payment_methods SET {} WHERE id = ${} RETURNING *", set_clauses.join(", "), id_param)
        )
        .fetch_one(pool)
        .await
        .map_err(|e| e.to_string())?;

        Ok(method)
    }

    pub async fn delete_payment_method(pool: &Pool<Postgres>, id: &str) -> Result<(), String> {
        sqlx::query(
            "DELETE FROM erp_payment_methods WHERE id = $1"
        )
        .bind(id)
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;
        Ok(())
    }
}
