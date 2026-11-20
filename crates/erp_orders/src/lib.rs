//! ERP Orders Library
//!
//! Order management, carts, shipments, and back-in-stock requests.

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::PgPool;
use uuid::Uuid;
use chrono::Utc;

// ——— Types ———

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ErpOrder {
    pub id: String,
    pub number: String,
    pub user_id: Option<String>,
    pub contact_id: Option<String>,
    pub status: String,
    pub currency: String,
    pub channel: String,
    pub location_id: Option<String>,
    pub terminal_id: Option<String>,
    pub shift_id: Option<String>,
    pub origin_node_id: Option<String>,
    pub guest_name: Option<String>,
    pub guest_email: Option<String>,
    pub guest_phone: Option<String>,
    pub source_cart_id: Option<String>,
    pub shipping_address: Value,
    pub billing_address: Option<Value>,
    pub items_subtotal: i64,
    pub stitching_subtotal: i64,
    pub item_discount_total: i64,
    pub order_discount: i64,
    pub shipping_fee: i64,
    pub cod_fee: i64,
    pub tax_amount: i64,
    pub grand_total: i64,
    pub shipping_zone: Option<String>,
    pub customer_note: Option<String>,
    pub staff_note: Option<String>,
    pub cancel_reason: Option<String>,
    pub placed_at: String,
    pub confirmed_at: Option<String>,
    pub delivered_at: Option<String>,
    pub cancelled_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub review_request_sent_at: Option<String>,
    pub cod_risk_score: Option<i32>,
    pub cod_risk_action: Option<String>,
    pub cod_risk_computed_at: Option<String>,
    pub cod_risk_reason_en: Option<String>,
    pub source_type: Option<String>,
    pub source_ref: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ErpOrderItem {
    pub id: String,
    pub order_id: String,
    pub product_id: Option<String>,
    pub variant_id: Option<String>,
    pub kind: String,
    pub name_en: String,
    pub name_ur: Option<String>,
    pub sku: Option<String>,
    pub image_url: Option<String>,
    pub variant_label: Option<String>,
    pub unit_price: i64,
    pub unit_cost: i64,
    pub quantity: i32,
    pub stitching_fee_unit: i64,
    pub discount_unit: i64,
    pub line_total: i64,
    pub tax_amount: i64,
    pub tax_rate_bps: i64,
    pub stitching_service_id: Option<String>,
    pub stitching_service_name: Option<String>,
    pub garment_type_key: Option<String>,
    pub measurement_snapshot: Option<Value>,
    pub options_snapshot: Option<Value>,
    pub stitching_notes: Option<String>,
    pub measurement_values: Option<Value>,
    pub custom_design_text: Option<Value>,
    pub size_token: Option<String>,
    pub replenishment_reminder_sent_at: Option<String>,
    pub fulfillment_status: String,
    pub awaiting_since: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ErpOrderStatusEvent {
    pub id: String,
    pub order_id: String,
    pub from_status: Option<String>,
    pub to_status: String,
    pub note: Option<String>,
    pub actor_id: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ErpCart {
    pub id: String,
    pub user_id: Option<String>,
    pub contact_id: Option<String>,
    pub guest_name: Option<String>,
    pub guest_email: Option<String>,
    pub guest_phone: Option<String>,
    pub status: String,
    pub metadata: Value,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ErpCartItem {
    pub id: String,
    pub cart_id: String,
    pub product_id: String,
    pub variant_id: Option<String>,
    pub quantity: i32,
    pub metadata: Value,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ErpShipment {
    pub id: String,
    pub order_id: String,
    pub courier: String,
    pub tracking_number: Option<String>,
    pub tracking_url: Option<String>,
    pub status: String,
    pub cost: Option<i64>,
    pub dispatched_at: Option<String>,
    pub delivered_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub method_id: Option<String>,
    pub weight_grams: Option<i32>,
    pub package_count: i32,
    pub label_url: Option<String>,
    pub manifest_id: Option<String>,
    pub meta: Option<Value>,
    pub is_rto: bool,
    pub rto_at: Option<String>,
    pub failed_attempts: i32,
    pub last_polled_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ErpShipmentEvent {
    pub id: String,
    pub shipment_id: String,
    pub event_type: String,
    pub location: Option<String>,
    pub description: Option<String>,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ErpBackInStockRequest {
    pub id: String,
    pub product_id: String,
    pub variant_id: Option<String>,
    pub customer_email: String,
    pub status: String,
    pub notified_at: Option<String>,
    pub created_at: String,
}

// ——— Request DTOs ———

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateOrderItemRequest {
    pub product_id: String,
    pub variant_id: Option<String>,
    pub kind: Option<String>,
    pub name_en: String,
    pub name_ur: Option<String>,
    pub sku: Option<String>,
    pub image_url: Option<String>,
    pub variant_label: Option<String>,
    pub unit_price: i64,
    pub unit_cost: i64,
    pub quantity: i32,
    pub stitching_fee_unit: Option<i64>,
    pub discount_unit: Option<i64>,
    pub tax_rate_bps: Option<i64>,
    pub stitching_service_id: Option<String>,
    pub stitching_service_name: Option<String>,
    pub garment_type_key: Option<String>,
    pub measurement_snapshot: Option<Value>,
    pub options_snapshot: Option<Value>,
    pub stitching_notes: Option<String>,
    pub measurement_values: Option<Value>,
    pub custom_design_text: Option<Value>,
    pub size_token: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateOrderRequest {
    pub user_id: Option<String>,
    pub contact_id: Option<String>,
    pub guest_name: Option<String>,
    pub guest_email: Option<String>,
    pub guest_phone: Option<String>,
    pub source_cart_id: Option<String>,
    pub shipping_address: Value,
    pub billing_address: Option<Value>,
    pub currency: Option<String>,
    pub shipping_zone: Option<String>,
    pub customer_note: Option<String>,
    pub items: Vec<CreateOrderItemRequest>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateOrderStatusRequest {
    pub status: String,
    pub note: Option<String>,
    pub actor_id: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateCartRequest {
    pub user_id: Option<String>,
    pub contact_id: Option<String>,
    pub guest_name: Option<String>,
    pub guest_email: Option<String>,
    pub guest_phone: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateCartItemRequest {
    pub cart_id: String,
    pub product_id: String,
    pub variant_id: Option<String>,
    pub quantity: i32,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateCartItemRequest {
    pub quantity: i32,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CheckoutCartRequest {
    pub cart_id: String,
    pub shipping_address: Value,
    pub billing_address: Option<Value>,
    pub customer_note: Option<String>,
    pub currency: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateShipmentRequest {
    pub order_id: String,
    pub courier: Option<String>,
    pub tracking_number: Option<String>,
    pub tracking_url: Option<String>,
    pub weight_grams: Option<i32>,
    pub package_count: Option<i32>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateShipmentStatusRequest {
    pub status: String,
    pub tracking_number: Option<String>,
    pub tracking_url: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateShipmentEventRequest {
    pub shipment_id: String,
    pub event_type: String,
    pub location: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateBackInStockRequest {
    pub product_id: String,
    pub variant_id: Option<String>,
    pub customer_email: String,
}

// ——— Service ———

pub struct ErpOrdersService {
    pool: PgPool,
}

impl ErpOrdersService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    // ============================================================
    // Order Number Generation
    // ============================================================

    async fn generate_order_number(&self) -> Result<String, String> {
        let year = Utc::now().format("%Y").to_string();
        let year_int: i32 = year.parse::<i32>().map_err(|e| e.to_string())?;

        let row: (i32,) = sqlx::query_as(
            "INSERT INTO erp_order_number_counters (year, counter) VALUES ($1, 0) ON CONFLICT (year) DO UPDATE SET counter = erp_order_number_counters.counter + 1 RETURNING counter"
        )
        .bind(year_int)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        let counter = row.0;
        Ok(format!("YSN-{}-{:04}", year, counter))
    }

    // ============================================================
    // Orders
    // ============================================================

    pub async fn list_orders(
        &self,
        status: Option<&str>,
        channel: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Value>, String> {
        let mut query = String::from("SELECT * FROM erp_orders WHERE 1=1");
        let mut param_idx = 1;

        if let Some(s) = status {
            query.push_str(&format!(" AND status = ${}", param_idx));
            param_idx += 1;
        }

        if let Some(c) = channel {
            query.push_str(&format!(" AND channel = ${}", param_idx));
            param_idx += 1;
        }

        let limit_p = param_idx;
        param_idx += 1;
        let offset_p = param_idx;
        query.push_str(&format!(" ORDER BY placed_at DESC LIMIT ${} OFFSET ${}", limit_p, offset_p));

        let rows = sqlx::query(&query)
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| e.to_string())?;

        let orders: Vec<Value> = rows.iter().map(|row| {
            json!({
                "id": row.try_get::<String, _>(0).unwrap_or_default(),
                "number": row.try_get::<String, _>(1).unwrap_or_default(),
                "userId": row.try_get::<Option<String>, _>(2).ok(),
                "contactId": row.try_get::<Option<String>, _>(3).ok(),
                "status": row.try_get::<String, _>(4).unwrap_or_default(),
                "currency": row.try_get::<String, _>(5).unwrap_or_default(),
                "channel": row.try_get::<String, _>(6).unwrap_or_default(),
                "guestName": row.try_get::<Option<String>, _>(9).ok(),
                "guestEmail": row.try_get::<Option<String>, _>(10).ok(),
                "guestPhone": row.try_get::<Option<String>, _>(11).ok(),
                "itemsSubtotal": row.try_get::<i64, _>(15).unwrap_or(0i64),
                "stitchingSubtotal": row.try_get::<i64, _>(16).unwrap_or(0i64),
                "itemDiscountTotal": row.try_get::<i64, _>(17).unwrap_or(0i64),
                "orderDiscount": row.try_get::<i64, _>(18).unwrap_or(0i64),
                "shippingFee": row.try_get::<i64, _>(19).unwrap_or(0i64),
                "codFee": row.try_get::<i64, _>(20).unwrap_or(0i64),
                "taxAmount": row.try_get::<i64, _>(21).unwrap_or(0i64),
                "grandTotal": row.try_get::<i64, _>(22).unwrap_or(0i64),
                "placedAt": row.try_get::<String, _>(25).unwrap_or_default(),
                "createdAt": row.try_get::<String, _>(27).unwrap_or_default(),
                "updatedAt": row.try_get::<String, _>(28).unwrap_or_default(),
            })
        }).collect();

        Ok(orders)
    }

    pub async fn get_order(&self, id: &str) -> Result<Value, String> {
        let row = sqlx::query(
            "SELECT * FROM erp_orders WHERE id = $1"
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        Ok(Self::row_to_order(&row))
    }

    pub async fn get_order_by_number(&self, number: &str) -> Result<Value, String> {
        let row = sqlx::query(
            "SELECT * FROM erp_orders WHERE number = $1"
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        Ok(Self::row_to_order(&row))
    }

    fn row_to_order(row: &sqlx::postgres::PgRow) -> Value {
        json!({
            "id": row.try_get::<String, _>(0).unwrap_or_default(),
            "number": row.try_get::<String, _>(1).unwrap_or_default(),
            "userId": row.try_get::<Option<String>, _>(2).ok(),
            "contactId": row.try_get::<Option<String>, _>(3).ok(),
            "status": row.try_get::<String, _>(4).unwrap_or_default(),
            "currency": row.try_get::<String, _>(5).unwrap_or_default(),
            "channel": row.try_get::<String, _>(6).unwrap_or_default(),
            "guestName": row.try_get::<Option<String>, _>(9).ok(),
            "guestEmail": row.try_get::<Option<String>, _>(10).ok(),
            "guestPhone": row.try_get::<Option<String>, _>(11).ok(),
            "sourceCartId": row.try_get::<Option<String>, _>(12).ok(),
            "shippingAddress": row.try_get::<Value, _>(13).unwrap_or(Value::Null),
            "billingAddress": row.try_get::<Option<Value>, _>(14).ok(),
            "itemsSubtotal": row.try_get::<i64, _>(15).unwrap_or(0i64),
            "stitchingSubtotal": row.try_get::<i64, _>(16).unwrap_or(0i64),
            "itemDiscountTotal": row.try_get::<i64, _>(17).unwrap_or(0i64),
            "orderDiscount": row.try_get::<i64, _>(18).unwrap_or(0i64),
            "shippingFee": row.try_get::<i64, _>(19).unwrap_or(0i64),
            "codFee": row.try_get::<i64, _>(20).unwrap_or(0i64),
            "taxAmount": row.try_get::<i64, _>(21).unwrap_or(0i64),
            "grandTotal": row.try_get::<i64, _>(22).unwrap_or(0i64),
            "shippingZone": row.try_get::<Option<String>, _>(23).ok(),
            "customerNote": row.try_get::<Option<String>, _>(24).ok(),
            "staffNote": row.try_get::<Option<String>, _>(26).ok(),
            "cancelReason": row.try_get::<Option<String>, _>(27).ok(),
            "placedAt": row.try_get::<String, _>(28).unwrap_or_default(),
            "confirmedAt": row.try_get::<Option<String>, _>(29).ok(),
            "deliveredAt": row.try_get::<Option<String>, _>(30).ok(),
            "cancelledAt": row.try_get::<Option<String>, _>(31).ok(),
            "createdAt": row.try_get::<String, _>(32).unwrap_or_default(),
            "updatedAt": row.try_get::<String, _>(33).unwrap_or_default(),
        })
    }

    pub async fn create_order(&self, req: &CreateOrderRequest) -> Result<Value, String> {
        let id = Uuid::new_v4().to_string();
        let number = self.generate_order_number().await?;
        let now = Utc::now().to_rfc3339();

        let mut items_subtotal: i64 = 0;
        let mut stitching_subtotal: i64 = 0;
        let mut item_discount_total: i64 = 0;
        let mut tax_amount: i64 = 0;

        for item in &req.items {
            let line_total = item.unit_price * item.quantity as i64;
            let stitching = item.stitching_fee_unit.unwrap_or(0) * item.quantity as i64;
            let discount = item.discount_unit.unwrap_or(0) * item.quantity as i64;
            let tax_rate = item.tax_rate_bps.unwrap_or(0);
            let tax = ((line_total + stitching - discount) * tax_rate) / 10000;

            items_subtotal += line_total;
            stitching_subtotal += stitching;
            item_discount_total += discount;
            tax_amount += tax;
        }

        let grand_total = items_subtotal + stitching_subtotal - item_discount_total + tax_amount;

        sqlx::query(
            r#"
            INSERT INTO erp_orders (
                id, number, user_id, contact_id, status, currency, channel,
                guest_name, guest_email, guest_phone, source_cart_id,
                shipping_address, billing_address,
                items_subtotal, stitching_subtotal, item_discount_total,
                order_discount, shipping_fee, cod_fee, tax_amount, grand_total,
                shipping_zone, customer_note, placed_at, created_at, updated_at
            ) VALUES (
                $1, $2, $3, $4, 'PENDING', $5, 'WEB',
                $6, $7, $8, $9,
                $10, $11,
                $12, $13, $14,
                0, 0, 0, $15, $16,
                $17, $18, $19, $20, $20
            )
            "#
        )
        .bind(&id)
        .bind(&number)
        .bind(&req.user_id)
        .bind(&req.contact_id)
        .bind(&req.currency.clone().unwrap_or("PKR".to_string()))
        .bind(&req.guest_name)
        .bind(&req.guest_email)
        .bind(&req.guest_phone)
        .bind(&req.source_cart_id)
        .bind(&req.shipping_address.clone())
        .bind(&req.billing_address.clone())
        .bind(items_subtotal)
        .bind(stitching_subtotal)
        .bind(item_discount_total)
        .bind(tax_amount)
        .bind(grand_total)
        .bind(&req.shipping_zone)
        .bind(&req.customer_note)
        .bind(&now)
        .bind(&now)
        .bind(&now)
        .execute(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        // Insert order items
        for item in &req.items {
            let item_id = Uuid::new_v4().to_string();
            let line_total = item.unit_price * item.quantity as i64
                + item.stitching_fee_unit.unwrap_or(0) * item.quantity as i64
                - item.discount_unit.unwrap_or(0) * item.quantity as i64;
            let tax_rate = item.tax_rate_bps.unwrap_or(0);
            let tax = (line_total * tax_rate) / 10000;

            sqlx::query(
                r#"
                INSERT INTO erp_order_items (
                    id, order_id, product_id, variant_id, kind, name_en, name_ur,
                    sku, image_url, variant_label,
                    unit_price, unit_cost, quantity, stitching_fee_unit, discount_unit,
                    line_total, tax_amount, tax_rate_bps,
                    stitching_service_id, stitching_service_name, garment_type_key,
                    measurement_snapshot, options_snapshot, stitching_notes,
                    measurement_values, custom_design_text, size_token,
                    fulfillment_status, created_at
                ) VALUES (
                    $1, $2, $3, $4, $5, $6, $7,
                    $8, $9, $10,
                    $11, $12, $13, $14, $15,
                    $16, $17, $18,
                    $19, $20, $21,
                    $22, $23, $24,
                    $25, $26, $27,
                    'READY', $28
                )
                "#
            )
            .bind(&item_id)
            .bind(&id)
            .bind(&item.product_id)
            .bind(&item.variant_id)
            .bind(&item.kind.clone().unwrap_or("goods".to_string()))
            .bind(&item.name_en)
            .bind(&item.name_ur)
            .bind(&item.sku)
            .bind(&item.image_url)
            .bind(&item.variant_label)
            .bind(item.unit_price)
            .bind(item.unit_cost)
            .bind(item.quantity)
            .bind(item.stitching_fee_unit.unwrap_or(0))
            .bind(item.discount_unit.unwrap_or(0))
            .bind(line_total)
            .bind(tax)
            .bind(tax_rate)
            .bind(&item.stitching_service_id)
            .bind(&item.stitching_service_name)
            .bind(&item.garment_type_key)
            .bind(&item.measurement_snapshot.clone())
            .bind(&item.options_snapshot.clone())
            .bind(&item.stitching_notes)
            .bind(&item.measurement_values.clone())
            .bind(&item.custom_design_text.clone())
            .bind(&item.size_token)
            .bind(&now)
            .execute(&self.pool)
            .await
            .map_err(|e| e.to_string())?;
        }

        // Mark source cart as converted
        if let Some(ref cart_id) = req.source_cart_id {
            let _ = sqlx::query(
                "UPDATE erp_carts SET status = 'converted', updated_at = $1 WHERE id = $2"
            )
            .bind(&now)
            .bind(cart_id)
            .execute(&self.pool)
            .await;
        }

        // Create status event
        let event_id = Uuid::new_v4().to_string();
        let _ = sqlx::query(
            "INSERT INTO erp_order_status_events (id, order_id, from_status, to_status, created_at) VALUES ($1, $2, NULL, 'PENDING', $3)"
        )
        .bind(&event_id)
        .bind(&id)
        .bind(&now)
        .execute(&self.pool)
        .await;

        self.get_order(&id).await
    }

    pub async fn update_order_status(
        &self,
        order_id: &str,
        req: &UpdateOrderStatusRequest,
    ) -> Result<Value, String> {
        let now = Utc::now().to_rfc3339();

        let mut query = format!("UPDATE erp_orders SET status = '${}', updated_at = '{}' WHERE id = '{}'", req.status, now, order_id);
        if req.status == "CONFIRMED" {
            query.push_str(&format!(", confirmed_at = '{}'", now));
        } else if req.status == "DELIVERED" {
            query.push_str(&format!(", delivered_at = '{}'", now));
        } else if req.status == "CANCELLED" {
            query.push_str(&format!(", cancelled_at = '{}'", now));
        }

        let _ = sqlx::query(&query)
            .execute(&self.pool)
            .await
            .map_err(|e| e.to_string())?;

        let event_id = Uuid::new_v4().to_string();
        let _ = sqlx::query(
            "INSERT INTO erp_order_status_events (id, order_id, to_status, note, actor_id, created_at) VALUES ($1, $2, $3, $4, $5, $6)"
        )
        .bind(&event_id)
        .bind(order_id)
        .bind(&req.status)
        .bind(&req.note)
        .bind(&req.actor_id)
        .bind(&now)
        .execute(&self.pool)
        .await;

        self.get_order(order_id).await
    }

    pub async fn get_order_items(&self, order_id: &str) -> Result<Vec<Value>, String> {
        let rows = sqlx::query(
            "SELECT * FROM erp_order_items WHERE order_id = $1 ORDER BY id"
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        let items: Vec<Value> = rows.iter().map(|row| {
            json!({
                "id": row.try_get::<String, _>(0).unwrap_or_default(),
                "orderId": row.try_get::<String, _>(1).unwrap_or_default(),
                "productId": row.try_get::<Option<String>, _>(2).ok(),
                "variantId": row.try_get::<Option<String>, _>(3).ok(),
                "kind": row.try_get::<String, _>(4).unwrap_or_default(),
                "nameEn": row.try_get::<String, _>(5).unwrap_or_default(),
                "nameUr": row.try_get::<Option<String>, _>(6).ok(),
                "sku": row.try_get::<Option<String>, _>(7).ok(),
                "imageUrl": row.try_get::<Option<String>, _>(8).ok(),
                "variantLabel": row.try_get::<Option<String>, _>(9).ok(),
                "unitPrice": row.try_get::<i64, _>(10).unwrap_or(0i64),
                "unitCost": row.try_get::<i64, _>(11).unwrap_or(0i64),
                "quantity": row.try_get::<i32, _>(12).unwrap_or(0i32),
                "stitchingFeeUnit": row.try_get::<i64, _>(13).unwrap_or(0i64),
                "discountUnit": row.try_get::<i64, _>(14).unwrap_or(0i64),
                "lineTotal": row.try_get::<i64, _>(15).unwrap_or(0i64),
                "taxAmount": row.try_get::<i64, _>(16).unwrap_or(0i64),
                "taxRateBps": row.try_get::<i64, _>(17).unwrap_or(0i64),
                "fulfillmentStatus": row.try_get::<String, _>(28).unwrap_or_default(),
                "createdAt": row.try_get::<String, _>(29).unwrap_or_default(),
            })
        }).collect();

        Ok(items)
    }

    pub async fn get_order_status_events(&self, order_id: &str) -> Result<Vec<Value>, String> {
        let rows = sqlx::query(
            "SELECT * FROM erp_order_status_events WHERE order_id = $1 ORDER BY created_at DESC"
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        let events: Vec<Value> = rows.iter().map(|row| {
            json!({
                "id": row.try_get::<String, _>(0).unwrap_or_default(),
                "orderId": row.try_get::<String, _>(1).unwrap_or_default(),
                "fromStatus": row.try_get::<Option<String>, _>(2).ok(),
                "toStatus": row.try_get::<String, _>(3).unwrap_or_default(),
                "note": row.try_get::<Option<String>, _>(4).ok(),
                "actorId": row.try_get::<Option<String>, _>(5).ok(),
                "createdAt": row.try_get::<String, _>(6).unwrap_or_default(),
            })
        }).collect();

        Ok(events)
    }

    pub async fn cancel_order(
        &self,
        order_id: &str,
        reason: &str,
        actor_id: Option<&str>,
    ) -> Result<Value, String> {
        let now = Utc::now().to_rfc3339();

        sqlx::query(
            "UPDATE erp_orders SET status = 'CANCELLED', cancel_reason = $1, cancelled_at = $2, updated_at = $2 WHERE id = $3"
        )
        .bind(reason)
        .bind(&now)
        .bind(order_id)
        .execute(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        let event_id = Uuid::new_v4().to_string();
        let _ = sqlx::query(
            "INSERT INTO erp_order_status_events (id, order_id, to_status, note, actor_id, created_at) VALUES ($1, $2, 'CANCELLED', $3, $4, $5)"
        )
        .bind(&event_id)
        .bind(order_id)
        .bind(reason)
        .bind(actor_id.map(|s| s.to_string()))
        .bind(&now)
        .execute(&self.pool)
        .await;

        self.get_order(order_id).await
    }

    pub async fn get_order_stats(&self) -> Result<Value, String> {
        let stats: (i64, i64, i64, i64) = sqlx::query_as(
            r#"
            SELECT
                COUNT(*)::bigint,
                COUNT(CASE WHEN status = 'PENDING' THEN 1 END)::bigint,
                COUNT(CASE WHEN status = 'CONFIRMED' THEN 1 END)::bigint,
                COUNT(CASE WHEN status = 'DELIVERED' THEN 1 END)::bigint
            FROM erp_orders
            "#
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        Ok(json!({
            "total": stats.0,
            "pending": stats.1,
            "confirmed": stats.2,
            "delivered": stats.3
        }))
    }

    // ============================================================
    // Carts
    // ============================================================

    pub async fn list_carts(
        &self,
        user_id: Option<&str>,
        status: Option<&str>,
    ) -> Result<Vec<Value>, String> {
        let mut query = String::from("SELECT * FROM erp_carts WHERE 1=1");
        let mut param_idx = 1;

        if let Some(_uid) = user_id {
            query.push_str(&format!(" AND user_id = ${}", param_idx));
            param_idx += 1;
        }

        if let Some(_s) = status {
            query.push_str(&format!(" AND status = ${}", param_idx));
            param_idx += 1;
        }

        query.push_str(" ORDER BY updated_at DESC");

        let rows = sqlx::query(&query)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| e.to_string())?;

        let carts: Vec<Value> = rows.iter().map(|row| {
            json!({
                "id": row.try_get::<String, _>(0).unwrap_or_default(),
                "userId": row.try_get::<Option<String>, _>(1).ok(),
                "contactId": row.try_get::<Option<String>, _>(2).ok(),
                "guestName": row.try_get::<Option<String>, _>(3).ok(),
                "guestEmail": row.try_get::<Option<String>, _>(4).ok(),
                "guestPhone": row.try_get::<Option<String>, _>(5).ok(),
                "status": row.try_get::<String, _>(6).unwrap_or_default(),
                "createdAt": row.try_get::<String, _>(8).unwrap_or_default(),
                "updatedAt": row.try_get::<String, _>(9).unwrap_or_default(),
            })
        }).collect();

        Ok(carts)
    }

    pub async fn get_cart(&self, id: &str) -> Result<Value, String> {
        let row = sqlx::query(
            "SELECT * FROM erp_carts WHERE id = $1"
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        Ok(json!({
            "id": row.try_get::<String, _>(0).unwrap_or_default(),
            "userId": row.try_get::<Option<String>, _>(1).ok(),
            "contactId": row.try_get::<Option<String>, _>(2).ok(),
            "guestName": row.try_get::<Option<String>, _>(3).ok(),
            "guestEmail": row.try_get::<Option<String>, _>(4).ok(),
            "guestPhone": row.try_get::<Option<String>, _>(5).ok(),
            "status": row.try_get::<String, _>(6).unwrap_or_default(),
            "metadata": row.try_get::<Value, _>(7).unwrap_or(Value::Null),
            "createdAt": row.try_get::<String, _>(8).unwrap_or_default(),
            "updatedAt": row.try_get::<String, _>(9).unwrap_or_default(),
        }))
    }

    pub async fn create_cart(&self, req: &CreateCartRequest) -> Result<Value, String> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();

        sqlx::query(
            r#"
            INSERT INTO erp_carts (id, user_id, contact_id, guest_name, guest_email, guest_phone, status, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, 'active', $7, $7)
            "#
        )
        .bind(&id)
        .bind(&req.user_id)
        .bind(&req.contact_id)
        .bind(&req.guest_name)
        .bind(&req.guest_email)
        .bind(&req.guest_phone)
        .bind(&now)
        .execute(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        self.get_cart(&id).await
    }

    pub async fn delete_cart(&self, id: &str) -> Result<(), String> {
        sqlx::query("DELETE FROM erp_carts WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub async fn get_cart_items(&self, cart_id: &str) -> Result<Vec<Value>, String> {
        let rows = sqlx::query(
            "SELECT * FROM erp_cart_items WHERE cart_id = $1 ORDER BY created_at"
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        let items: Vec<Value> = rows.iter().map(|row| {
            json!({
                "id": row.try_get::<String, _>(0).unwrap_or_default(),
                "cartId": row.try_get::<String, _>(1).unwrap_or_default(),
                "productId": row.try_get::<String, _>(2).unwrap_or_default(),
                "variantId": row.try_get::<Option<String>, _>(3).ok(),
                "quantity": row.try_get::<i32, _>(4).unwrap_or(0i32),
                "metadata": row.try_get::<Value, _>(5).unwrap_or(Value::Null),
                "createdAt": row.try_get::<String, _>(6).unwrap_or_default(),
            })
        }).collect();

        Ok(items)
    }

    pub async fn add_cart_item(&self, req: &CreateCartItemRequest) -> Result<Value, String> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();

        let existing: Option<sqlx::postgres::PgRow> = sqlx::query(
            "SELECT * FROM erp_cart_items WHERE cart_id = $1 AND product_id = $2 AND variant_id = $3"
        )
        .bind(&req.cart_id)
        .bind(&req.product_id)
        .bind(&req.variant_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        if let Some(row) = existing {
            let item_id = row.try_get::<String, _>(0).unwrap_or_default();
            let current_qty: i32 = row.try_get(4).unwrap_or(0);
            let new_qty = current_qty + req.quantity;
            sqlx::query(
                "UPDATE erp_cart_items SET quantity = $1 WHERE id = $2"
            )
            .bind(new_qty)
            .bind(&item_id)
            .execute(&self.pool)
            .await
            .map_err(|e| e.to_string())?;

            let updated = sqlx::query("SELECT * FROM erp_cart_items WHERE id = $1")
                .bind(&item_id)
                .fetch_one(&self.pool)
                .await
                .map_err(|e| e.to_string())?;

            Ok(json!({
                "id": updated.try_get::<String, _>(0).unwrap_or_default(),
                "cartId": updated.try_get::<String, _>(1).unwrap_or_default(),
                "productId": updated.try_get::<String, _>(2).unwrap_or_default(),
                "variantId": updated.try_get::<Option<String>, _>(3).ok(),
                "quantity": updated.try_get::<i32, _>(4).unwrap_or(0i32),
                "metadata": updated.try_get::<Value, _>(5).unwrap_or(Value::Null),
                "createdAt": updated.try_get::<String, _>(6).unwrap_or_default(),
            }))
        } else {
            sqlx::query(
                r#"
                INSERT INTO erp_cart_items (id, cart_id, product_id, variant_id, quantity, metadata, created_at)
                VALUES ($1, $2, $3, $4, $5, '{}', $6)
                "#
            )
            .bind(&id)
            .bind(&req.cart_id)
            .bind(&req.product_id)
            .bind(&req.variant_id)
            .bind(req.quantity)
            .bind(&now)
            .execute(&self.pool)
            .await
            .map_err(|e| e.to_string())?;

            let row = sqlx::query("SELECT * FROM erp_cart_items WHERE id = $1")
                .bind(&id)
                .fetch_one(&self.pool)
                .await
                .map_err(|e| e.to_string())?;

            Ok(json!({
                "id": row.try_get::<String, _>(0).unwrap_or_default(),
                "cartId": row.try_get::<String, _>(1).unwrap_or_default(),
                "productId": row.try_get::<String, _>(2).unwrap_or_default(),
                "variantId": row.try_get::<Option<String>, _>(3).ok(),
                "quantity": row.try_get::<i32, _>(4).unwrap_or(0i32),
                "metadata": row.try_get::<Value, _>(5).unwrap_or(Value::Null),
                "createdAt": row.try_get::<String, _>(6).unwrap_or_default(),
            }))
        }
    }

    pub async fn update_cart_item(&self, item_id: &str, quantity: i32) -> Result<Value, String> {
        if quantity <= 0 {
            return Err("Quantity must be greater than 0".to_string());
        }

        sqlx::query(
            "UPDATE erp_cart_items SET quantity = $1 WHERE id = $2"
        )
        .bind(quantity)
        .bind(item_id)
        .execute(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        let row = sqlx::query("SELECT * FROM erp_cart_items WHERE id = $1")
            .bind(item_id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| e.to_string())?;

        Ok(json!({
            "id": row.try_get::<String, _>(0).unwrap_or_default(),
            "cartId": row.try_get::<String, _>(1).unwrap_or_default(),
            "productId": row.try_get::<String, _>(2).unwrap_or_default(),
            "variantId": row.try_get::<Option<String>, _>(3).ok(),
            "quantity": row.try_get::<i32, _>(4).unwrap_or(0i32),
            "metadata": row.try_get::<Value, _>(5).unwrap_or(Value::Null),
            "createdAt": row.try_get::<String, _>(6).unwrap_or_default(),
        }))
    }

    pub async fn remove_cart_item(&self, item_id: &str) -> Result<(), String> {
        sqlx::query("DELETE FROM erp_cart_items WHERE id = $1")
            .bind(item_id)
            .execute(&self.pool)
            .await
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub async fn checkout_cart(&self, req: &CheckoutCartRequest) -> Result<Value, String> {
        // Get cart
        let cart_row = sqlx::query(
            "SELECT * FROM erp_carts WHERE id = $1"
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        let cart_id = cart_row.try_get::<String, _>(0).unwrap_or_default();
        let user_id: Option<String> = cart_row.try_get(1).ok();
        let contact_id: Option<String> = cart_row.try_get(2).ok();
        let guest_name: Option<String> = cart_row.try_get(3).ok();
        let guest_email: Option<String> = cart_row.try_get(4).ok();
        let guest_phone: Option<String> = cart_row.try_get(5).ok();

        // Get cart items
        let cart_items_rows = sqlx::query(
            "SELECT * FROM erp_cart_items WHERE cart_id = $1 ORDER BY created_at"
        )
        .bind(&cart_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        if cart_items_rows.is_empty() {
            return Err("Cart is empty".to_string());
        }

        // Build order items from cart items
        let mut order_items = Vec::new();
        for ci in &cart_items_rows {
            let product_id = ci.try_get::<String, _>(2).unwrap_or_default();
            let variant_id: Option<String> = ci.try_get(3).ok();
            let quantity: i32 = ci.try_get(4).unwrap_or(0);

            let product: Option<(String, i64, String)> = sqlx::query_as(
                "SELECT id, base_price, kind FROM erp_products WHERE id = $1"
            )
            .bind(&product_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| e.to_string())?;

            let (product_name, unit_price, kind) = match product {
                Some((name, price, k)) => (name, price, k),
                None => (product_id.clone(), 0, "goods".to_string()),
            };

            order_items.push(CreateOrderItemRequest {
                product_id,
                variant_id,
                kind: Some(kind),
                name_en: product_name,
                name_ur: None,
                sku: None,
                image_url: None,
                variant_label: None,
                unit_price,
                unit_cost: 0,
                quantity,
                stitching_fee_unit: None,
                discount_unit: None,
                tax_rate_bps: None,
                stitching_service_id: None,
                stitching_service_name: None,
                garment_type_key: None,
                measurement_snapshot: None,
                options_snapshot: None,
                stitching_notes: None,
                measurement_values: None,
                custom_design_text: None,
                size_token: None,
            });
        }

        let order_req = CreateOrderRequest {
            user_id,
            contact_id,
            guest_name,
            guest_email,
            guest_phone,
            source_cart_id: Some(req.cart_id.clone()),
            shipping_address: req.shipping_address.clone(),
            billing_address: req.billing_address.clone(),
            currency: req.currency.clone(),
            shipping_zone: None,
            customer_note: req.customer_note.clone(),
            items: order_items,
        };

        self.create_order(&order_req).await
    }

    // ============================================================
    // Shipments
    // ============================================================

    pub async fn list_shipments(
        &self,
        order_id: Option<&str>,
        status: Option<&str>,
    ) -> Result<Vec<Value>, String> {
        let mut query = String::from("SELECT * FROM erp_shipments WHERE 1=1");
        let mut param = 1;

        if let Some(oid) = order_id {
            query.push_str(&format!(" AND order_id = ${}", param));
            param += 1;
        }

        if let Some(s) = status {
            query.push_str(&format!(" AND status = ${}", param));
            param += 1;
        }

        query.push_str(" ORDER BY created_at DESC");

        let rows = sqlx::query(&query)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| e.to_string())?;

        let shipments: Vec<Value> = rows.iter().map(|row| {
            json!({
                "id": row.try_get::<String, _>(0).unwrap_or_default(),
                "orderId": row.try_get::<String, _>(1).unwrap_or_default(),
                "courier": row.try_get::<String, _>(2).unwrap_or_default(),
                "trackingNumber": row.try_get::<Option<String>, _>(3).ok(),
                "trackingUrl": row.try_get::<Option<String>, _>(4).ok(),
                "status": row.try_get::<String, _>(5).unwrap_or_default(),
                "createdAt": row.try_get::<String, _>(8).unwrap_or_default(),
                "updatedAt": row.try_get::<String, _>(9).unwrap_or_default(),
            })
        }).collect();

        Ok(shipments)
    }

    pub async fn get_shipment(&self, id: &str) -> Result<Value, String> {
        let row = sqlx::query(
            "SELECT * FROM erp_shipments WHERE id = $1"
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        Ok(json!({
            "id": row.try_get::<String, _>(0).unwrap_or_default(),
            "orderId": row.try_get::<String, _>(1).unwrap_or_default(),
            "courier": row.try_get::<String, _>(2).unwrap_or_default(),
            "trackingNumber": row.try_get::<Option<String>, _>(3).ok(),
            "trackingUrl": row.try_get::<Option<String>, _>(4).ok(),
            "status": row.try_get::<String, _>(5).unwrap_or_default(),
            "weightGrams": row.try_get::<Option<i32>, _>(13).ok(),
            "packageCount": row.try_get::<i32, _>(14).unwrap_or(0i32),
            "createdAt": row.try_get::<String, _>(8).unwrap_or_default(),
            "updatedAt": row.try_get::<String, _>(9).unwrap_or_default(),
        }))
    }

    pub async fn create_shipment(&self, req: &CreateShipmentRequest) -> Result<Value, String> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();

        sqlx::query(
            r#"
            INSERT INTO erp_shipments (
                id, order_id, courier, tracking_number, tracking_url, status,
                package_count, weight_grams, created_at, updated_at
            ) VALUES ($1, $2, $3, $4, $5, 'PENDING', $6, $7, $8, $8)
            "#
        )
        .bind(&id)
        .bind(&req.order_id)
        .bind(&req.courier.clone().unwrap_or("manual".to_string()))
        .bind(&req.tracking_number)
        .bind(&req.tracking_url)
        .bind(&req.package_count.unwrap_or(1))
        .bind(&req.weight_grams)
        .bind(&now)
        .execute(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        self.get_shipment(&id).await
    }

    pub async fn update_shipment_status(
        &self,
        shipment_id: &str,
        req: &UpdateShipmentStatusRequest,
    ) -> Result<Value, String> {
        let now = Utc::now().to_rfc3339();
        let mut query = format!("UPDATE erp_shipments SET status = '{}', updated_at = '{}' WHERE id = '{}'", req.status, now, shipment_id);

        if let Some(ref tracking) = req.tracking_number {
            query.push_str(&format!(", tracking_number = '{}'", tracking));
        }
        if let Some(ref url) = req.tracking_url {
            query.push_str(&format!(", tracking_url = '{}'", url));
        }

        if req.status == "DISPATCHED" {
            query.push_str(&format!(", dispatched_at = '{}'", now));
        } else if req.status == "DELIVERED" {
            query.push_str(&format!(", delivered_at = '{}'", now));
        }

        let _ = sqlx::query(&query)
            .execute(&self.pool)
            .await
            .map_err(|e| e.to_string())?;

        self.get_shipment(shipment_id).await
    }

    pub async fn get_shipment_events(&self, shipment_id: &str) -> Result<Vec<Value>, String> {
        let rows = sqlx::query(
            "SELECT * FROM erp_shipment_events WHERE shipment_id = $1 ORDER BY timestamp DESC"
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        let events: Vec<Value> = rows.iter().map(|row| {
            json!({
                "id": row.try_get::<String, _>(0).unwrap_or_default(),
                "shipmentId": row.try_get::<String, _>(1).unwrap_or_default(),
                "eventType": row.try_get::<String, _>(2).unwrap_or_default(),
                "location": row.try_get::<Option<String>, _>(3).ok(),
                "description": row.try_get::<Option<String>, _>(4).ok(),
                "timestamp": row.try_get::<String, _>(5).unwrap_or_default(),
            })
        }).collect();

        Ok(events)
    }

    pub async fn create_shipment_event(&self, req: &CreateShipmentEventRequest) -> Result<Value, String> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();

        sqlx::query(
            "INSERT INTO erp_shipment_events (id, shipment_id, event_type, location, description, timestamp) VALUES ($1, $2, $3, $4, $5, $6)"
        )
        .bind(&id)
        .bind(&req.shipment_id)
        .bind(&req.event_type)
        .bind(&req.location)
        .bind(&req.description)
        .bind(&now)
        .execute(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        let row = sqlx::query("SELECT * FROM erp_shipment_events WHERE id = $1")
            .bind(&id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| e.to_string())?;

        Ok(json!({
            "id": row.try_get::<String, _>(0).unwrap_or_default(),
            "shipmentId": row.try_get::<String, _>(1).unwrap_or_default(),
            "eventType": row.try_get::<String, _>(2).unwrap_or_default(),
            "location": row.try_get::<Option<String>, _>(3).ok(),
            "description": row.try_get::<Option<String>, _>(4).ok(),
            "timestamp": row.try_get::<String, _>(5).unwrap_or_default(),
        }))
    }

    // ============================================================
    // Back in Stock Requests
    // ============================================================

    pub async fn list_back_in_stock_requests(
        &self,
        product_id: Option<&str>,
        status: Option<&str>,
    ) -> Result<Vec<Value>, String> {
        let mut query = String::from("SELECT * FROM erp_back_in_stock_requests WHERE 1=1");
        let mut param = 1;

        if let Some(pid) = product_id {
            query.push_str(&format!(" AND product_id = ${}", param));
            param += 1;
        }

        if let Some(s) = status {
            query.push_str(&format!(" AND status = ${}", param));
            param += 1;
        }

        query.push_str(" ORDER BY created_at DESC");

        let rows = sqlx::query(&query)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| e.to_string())?;

        let requests: Vec<Value> = rows.iter().map(|row| {
            json!({
                "id": row.try_get::<String, _>(0).unwrap_or_default(),
                "productId": row.try_get::<String, _>(1).unwrap_or_default(),
                "variantId": row.try_get::<Option<String>, _>(2).ok(),
                "customerEmail": row.try_get::<String, _>(3).unwrap_or_default(),
                "status": row.try_get::<String, _>(4).unwrap_or_default(),
                "notifiedAt": row.try_get::<Option<String>, _>(5).ok(),
                "createdAt": row.try_get::<String, _>(6).unwrap_or_default(),
            })
        }).collect();

        Ok(requests)
    }

    pub async fn create_back_in_stock_request(&self, req: &CreateBackInStockRequest) -> Result<Value, String> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();

        sqlx::query(
            "INSERT INTO erp_back_in_stock_requests (id, product_id, variant_id, customer_email, status, created_at) VALUES ($1, $2, $3, $4, 'pending', $5)"
        )
        .bind(&id)
        .bind(&req.product_id)
        .bind(&req.variant_id)
        .bind(&req.customer_email)
        .bind(&now)
        .execute(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        let row = sqlx::query(
            "SELECT * FROM erp_back_in_stock_requests WHERE id = $1"
        )
        .bind(&id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        Ok(json!({
            "id": row.try_get::<String, _>(0).unwrap_or_default(),
            "productId": row.try_get::<String, _>(1).unwrap_or_default(),
            "variantId": row.try_get::<Option<String>, _>(2).ok(),
            "customerEmail": row.try_get::<String, _>(3).unwrap_or_default(),
            "status": row.try_get::<String, _>(4).unwrap_or_default(),
            "notifiedAt": row.try_get::<Option<String>, _>(5).ok(),
            "createdAt": row.try_get::<String, _>(6).unwrap_or_default(),
        }))
    }

    pub async fn notify_back_in_stock(&self, product_id: &str) -> Result<u64, String> {
        let now = Utc::now().to_rfc3339();

        let result = sqlx::query(
            "UPDATE erp_back_in_stock_requests SET status = 'notified', notified_at = $1 WHERE product_id = $2 AND status = 'pending'"
        )
        .bind(&now)
        .bind(product_id)
        .execute(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        Ok(result.rows_affected())
    }
}
