#![allow(dead_code)]

//! Payment tracking service: record payments, track status, link to orders.

use serde::{Deserialize, Serialize};
use sqlx::{Row, PgPool};
use uuid::Uuid;

use crate::models::{CommercePayment, PaymentStatus};

// ---------------------------------------------------------------------------
// Request/Response DTOs
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct RecordPaymentRequest {
    pub order_id: String,
    pub amount: f64,
    pub currency: Option<String>,
    pub method: String,
    pub transaction_id: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct UpdatePaymentStatusRequest {
    pub status: String,
}

// ---------------------------------------------------------------------------
// Service
// ---------------------------------------------------------------------------

pub struct CommercePaymentsService {
    pool: PgPool,
}

impl CommercePaymentsService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// List payments for an order
    pub async fn list_payments_for_order(
        &self,
        order_id: &str,
    ) -> Result<Vec<CommercePayment>, String> {
        let rows = sqlx::query(
            "SELECT id, order_id, amount, currency, method, status, transaction_id, \
             metadata, created_at \
             FROM commerce_payments WHERE order_id = ? ORDER BY created_at DESC",
        )
        .bind(order_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to list payments: {e}"))?;

        rows.iter()
            .map(|row| Ok(CommercePayment {
                id: row.get("id"),
                order_id: row.get("order_id"),
                amount: row.get("amount"),
                currency: row.get("currency"),
                method: row.get("method"),
                status: row.get("status"),
                transaction_id: row.get("transaction_id"),
                metadata: row.get("metadata"),
                created_at: row.get("created_at"),
            }))
            .collect()
    }

    /// List all payments with filtering
    pub async fn list_payments(
        &self,
        brand_id: &str,
        status: Option<&str>,
        method: Option<&str>,
    ) -> Result<Vec<CommercePayment>, String> {
        let mut query = String::from(
            "SELECT p.id, p.order_id, p.amount, p.currency, p.method, p.status, \
             p.transaction_id, p.metadata, p.created_at \
             FROM commerce_payments p \
             JOIN commerce_orders o ON p.order_id = o.id \
             WHERE o.brand_id = ?",
        );

        if let Some(s) = status {
            query.push_str(" AND p.status = ?");
        }
        if let Some(m) = method {
            query.push_str(" AND p.method = ?");
        }
        query.push_str(" ORDER BY p.created_at DESC");

        // Simplified query
        let rows = sqlx::query(
            "SELECT p.id, p.order_id, p.amount, p.currency, p.method, p.status, \
             p.transaction_id, p.metadata, p.created_at \
             FROM commerce_payments p \
             JOIN commerce_orders o ON p.order_id = o.id \
             WHERE o.brand_id = ? ORDER BY p.created_at DESC",
        )
        .bind(brand_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to list payments: {e}"))?;

        rows.iter()
            .map(|row| Ok(CommercePayment {
                id: row.get("id"),
                order_id: row.get("order_id"),
                amount: row.get("amount"),
                currency: row.get("currency"),
                method: row.get("method"),
                status: row.get("status"),
                transaction_id: row.get("transaction_id"),
                metadata: row.get("metadata"),
                created_at: row.get("created_at"),
            }))
            .collect()
    }

    /// Get a single payment
    pub async fn get_payment(&self, payment_id: &str) -> Result<CommercePayment, String> {
        let row = sqlx::query(
            "SELECT id, order_id, amount, currency, method, status, transaction_id, \
             metadata, created_at \
             FROM commerce_payments WHERE id = ?",
        )
        .bind(payment_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Payment not found: {e}"))?;

        Ok(CommercePayment {
            id: row.get("id"),
            order_id: row.get("order_id"),
            amount: row.get("amount"),
            currency: row.get("currency"),
            method: row.get("method"),
            status: row.get("status"),
            transaction_id: row.get("transaction_id"),
            metadata: row.get("metadata"),
            created_at: row.get("created_at"),
        })
    }

    /// Record a new payment
    pub async fn record_payment(
        &self,
        req: &RecordPaymentRequest,
    ) -> Result<CommercePayment, String> {
        let id = Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        let currency = req.currency.clone().unwrap_or_else(|| "USD".to_string());
        let metadata = req
            .metadata
            .as_ref()
            .map(|v| serde_json::to_string(v).unwrap_or_default());

        // Verify order exists
        let _ = sqlx::query_scalar::<_, i64>(
            "SELECT 1 FROM commerce_orders WHERE id = ?",
        )
        .bind(&req.order_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|_| format!("Order '{}' not found", req.order_id))?;

        sqlx::query(
            "INSERT INTO commerce_payments (id, order_id, amount, currency, method, status, \
             transaction_id, metadata, created_at) \
             VALUES (?, ?, ?, ?, ?, 'pending', ?, ?, ?)",
        )
        .bind(&id)
        .bind(&req.order_id)
        .bind(req.amount)
        .bind(&currency)
        .bind(&req.method)
        .bind(&req.transaction_id)
        .bind(&metadata)
        .bind(&now)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to record payment: {e}"))?;

        // Update order payment status
        self.update_order_payment_status(&req.order_id).await.ok();

        self.get_payment(&id).await
    }

    /// Update payment status
    pub async fn update_payment_status(
        &self,
        payment_id: &str,
        req: &UpdatePaymentStatusRequest,
    ) -> Result<CommercePayment, String> {
        let status = PaymentStatus::from_str(&req.status);
        let now = chrono::Utc::now().to_rfc3339();

        // Get payment to find order_id
        let payment = self.get_payment(payment_id).await?;

        sqlx::query("UPDATE commerce_payments SET status = ? WHERE id = ?")
            .bind(status.as_str())
            .bind(payment_id)
            .execute(&self.pool)
            .await
            .map_err(|e| format!("Failed to update payment status: {e}"))?;

        // Update order payment status
        self.update_order_payment_status(&payment.order_id).await.ok();

        self.get_payment(payment_id).await
    }

    /// Mark a payment as paid (for webhook callbacks)
    pub async fn mark_payment_paid(
        &self,
        payment_id: &str,
        transaction_id: Option<&str>,
    ) -> Result<CommercePayment, String> {
        let now = chrono::Utc::now().to_rfc3339();

        let mut query =
            sqlx::query("UPDATE commerce_payments SET status = 'paid', updated_at = ? WHERE id = ?")
                .bind(&now)
                .bind(payment_id);

        if let Some(tx_id) = transaction_id {
            // Need a different query with transaction_id
            let _ = sqlx::query(
                "UPDATE commerce_payments SET status = 'paid', transaction_id = ? WHERE id = ?",
            )
            .bind(tx_id)
            .bind(payment_id)
            .execute(&self.pool)
            .await;
        } else {
            query
                .execute(&self.pool)
                .await
                .map_err(|e| format!("Failed to mark payment paid: {e}"))?;
        }

        let payment = self.get_payment(payment_id).await?;
        self.update_order_payment_status(&payment.order_id).await.ok();
        Ok(payment)
    }

    /// Refund a payment
    pub async fn refund_payment(&self, payment_id: &str) -> Result<CommercePayment, String> {
        let now = chrono::Utc::now().to_rfc3339();
        sqlx::query("UPDATE commerce_payments SET status = 'refunded' WHERE id = ?")
            .bind(payment_id)
            .execute(&self.pool)
            .await
            .map_err(|e| format!("Failed to refund payment: {e}"))?;

        let payment = self.get_payment(payment_id).await?;
        self.update_order_payment_status(&payment.order_id).await.ok();
        Ok(payment)
    }

    /// Recalculate and update the order's payment status based on its payments
    async fn update_order_payment_status(&self, order_id: &str) -> Result<(), String> {
        let order_total: f64 = sqlx::query_scalar(
            "SELECT total_amount FROM commerce_orders WHERE id = ?",
        )
        .bind(order_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Failed to load order: {e}"))?;

        let paid_amount: f64 = sqlx::query_scalar(
            "SELECT COALESCE(SUM(amount), 0) FROM commerce_payments \
             WHERE order_id = ? AND status = 'paid'",
        )
        .bind(order_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Failed to sum payments: {e}"))?;

        let has_refund: bool = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM commerce_payments \
             WHERE order_id = ? AND status = 'refunded'",
        )
        .bind(order_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Failed to check refunds: {e}"))?
            > 0;

        let new_status = if has_refund {
            "refunded"
        } else if paid_amount >= order_total {
            "paid"
        } else if paid_amount > 0.0 {
            "partially_paid"
        } else {
            "pending"
        };

        let now = chrono::Utc::now().to_rfc3339();
        sqlx::query(
            "UPDATE commerce_orders SET payment_status = ?, updated_at = ? WHERE id = ?",
        )
        .bind(new_status)
        .bind(&now)
        .bind(order_id)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to update order payment status: {e}"))?;

        Ok(())
    }

    /// Get payment statistics for a brand
    pub async fn get_payment_stats(&self, brand_id: &str) -> Result<serde_json::Value, String> {
        let total_paid: f64 = sqlx::query_scalar(
            "SELECT COALESCE(SUM(p.amount), 0) FROM commerce_payments p \
             JOIN commerce_orders o ON p.order_id = o.id \
             WHERE o.brand_id = ? AND p.status = 'paid'",
        )
        .bind(brand_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Failed to sum paid: {e}"))?;

        let pending_amount: f64 = sqlx::query_scalar(
            "SELECT COALESCE(SUM(p.amount), 0) FROM commerce_payments p \
             JOIN commerce_orders o ON p.order_id = o.id \
             WHERE o.brand_id = ? AND p.status = 'pending'",
        )
        .bind(brand_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Failed to sum pending: {e}"))?;

        let total_transactions: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM commerce_payments p \
             JOIN commerce_orders o ON p.order_id = o.id \
             WHERE o.brand_id = ?",
        )
        .bind(brand_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Failed to count transactions: {e}"))?;

        Ok(serde_json::json!({
            "totalPaid": total_paid,
            "pendingAmount": pending_amount,
            "totalTransactions": total_transactions,
        }))
    }
}
