#![allow(dead_code)]

//! Delivery and Shipping service: shipping methods, shipments, tracking, delivery events

use serde::{Deserialize, Serialize};
use sqlx::{Row, PgPool};
use uuid::Uuid;

use crate::models::{ShippingMethod, Shipment, DeliveryEvent};

// ---------------------------------------------------------------------------
// DTOs
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct CreateShippingMethodRequest {
    pub brand_id: String,
    pub name: String,
    pub r#type: Option<String>,
    pub rate: Option<f64>,
    pub free_shipping_threshold: Option<f64>,
    pub regions: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateShippingMethodRequest {
    pub name: Option<String>,
    pub r#type: Option<String>,
    pub rate: Option<f64>,
    pub free_shipping_threshold: Option<Option<f64>>,
    pub regions: Option<serde_json::Value>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct CreateShipmentRequest {
    pub order_id: String,
    pub tracking_number: Option<String>,
    pub carrier: Option<String>,
    pub estimated_delivery: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateShipmentRequest {
    pub tracking_number: Option<String>,
    pub carrier: Option<String>,
    pub status: Option<String>,
    pub estimated_delivery: Option<String>,
    pub actual_delivery: Option<String>,
    pub shipping_label_url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AddDeliveryEventRequest {
    pub shipment_id: String,
    pub event_type: String,
    pub location: Option<String>,
    pub description: Option<String>,
}

// ---------------------------------------------------------------------------
// Service
// ---------------------------------------------------------------------------

pub struct CommerceDeliveryService {
    pool: PgPool,
}

impl CommerceDeliveryService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    // ---------------------------------------------------------------------------
    // Shipping Methods
    // ---------------------------------------------------------------------------

    pub async fn list_shipping_methods(&self, brand_id: &str) -> Result<Vec<ShippingMethod>, String> {
        let rows = sqlx::query("SELECT * FROM shipping_methods WHERE brand_id = ? ORDER BY name")
            .bind(brand_id)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| e.to_string())?;
        rows.iter()
            .map(|r| self.row_to_shipping_method(r))
            .collect::<Result<Vec<_>, _>>()
    }

    pub async fn get_shipping_method(&self, id: &str) -> Result<ShippingMethod, String> {
        let row = sqlx::query("SELECT * FROM shipping_methods WHERE id = ?")
            .bind(id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| e.to_string())?;
        self.row_to_shipping_method(&row)
    }

    pub async fn create_shipping_method(&self, req: &CreateShippingMethodRequest) -> Result<ShippingMethod, String> {
        let id = Uuid::new_v4().to_string();
        let r#type = req.r#type.as_deref().unwrap_or("flat_rate");
        let regions = req.regions.as_ref().map(|v| serde_json::to_string(v).unwrap_or_default());
        sqlx::query(
            "INSERT INTO shipping_methods (id, brand_id, name, type, rate, free_shipping_threshold, regions) \
             VALUES (?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(&id)
        .bind(&req.brand_id)
        .bind(&req.name)
        .bind(r#type)
        .bind(req.rate.unwrap_or(0.0))
        .bind(req.free_shipping_threshold)
        .bind(regions.as_deref())
        .execute(&self.pool)
        .await
        .map_err(|e| e.to_string())?;
        self.get_shipping_method(&id).await
    }

    pub async fn update_shipping_method(&self, id: &str, req: &UpdateShippingMethodRequest) -> Result<ShippingMethod, String> {
        let mut sets = Vec::new();
        if let Some(ref name) = req.name { sets.push("name = ?"); }
        if let Some(ref r#type) = req.r#type { sets.push("r#type = ?"); }
        if let Some(ref is_active) = req.is_active { sets.push("is_active = ?"); }
        sets.push("updated_at = NOW()::TEXT");

        if sets.len() <= 1 {
            return self.get_shipping_method(id).await;
        }

        let query = format!("UPDATE shipping_methods SET {} WHERE id = ?", sets.join(", "));
        sqlx::query(&query)
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| e.to_string())?;
        self.get_shipping_method(id).await
    }

    pub async fn delete_shipping_method(&self, id: &str) -> Result<(), String> {
        sqlx::query("DELETE FROM shipping_methods WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    // ---------------------------------------------------------------------------
    // Shipments
    // ---------------------------------------------------------------------------

    pub async fn list_shipments(&self, order_id: Option<&str>) -> Result<Vec<Shipment>, String> {
        let (query, rows) = if let Some(oid) = order_id {
            let q = "SELECT * FROM shipments WHERE order_id = ? ORDER BY created_at DESC";
            (q, sqlx::query(q).bind(oid).fetch_all(&self.pool).await.map_err(|e| e.to_string())?)
        } else {
            let q = "SELECT * FROM shipments ORDER BY created_at DESC LIMIT 100";
            (q, sqlx::query(q).fetch_all(&self.pool).await.map_err(|e| e.to_string())?)
        };
        rows.iter()
            .map(|r| self.row_to_shipment(r))
            .collect::<Result<Vec<_>, _>>()
    }

    pub async fn get_shipment(&self, id: &str) -> Result<Shipment, String> {
        let row = sqlx::query("SELECT * FROM shipments WHERE id = ?")
            .bind(id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| e.to_string())?;
        self.row_to_shipment(&row)
    }

    pub async fn create_shipment(&self, req: &CreateShipmentRequest) -> Result<Shipment, String> {
        let id = Uuid::new_v4().to_string();
        sqlx::query(
            "INSERT INTO shipments (id, order_id, tracking_number, carrier, estimated_delivery) \
             VALUES (?, ?, ?, ?, ?)"
        )
        .bind(&id)
        .bind(&req.order_id)
        .bind(req.tracking_number.as_deref())
        .bind(req.carrier.as_deref())
        .bind(req.estimated_delivery.as_deref())
        .execute(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        // Add initial delivery event
        let event_id = Uuid::new_v4().to_string();
        sqlx::query(
            "INSERT INTO delivery_events (id, shipment_id, event_type, description) VALUES (?, ?, 'created', 'Shipment created')"
        )
        .bind(&event_id)
        .bind(&id)
        .execute(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        // Update order status to 'processing'
        sqlx::query(
            "UPDATE commerce_orders SET status = 'processing', updated_at = NOW()::TEXT \
             WHERE id = ? AND status NOT IN ('shipped', 'delivered', 'cancelled')"
        )
        .bind(&req.order_id)
        .execute(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        self.get_shipment(&id).await
    }

    pub async fn update_shipment(&self, id: &str, req: &UpdateShipmentRequest) -> Result<Shipment, String> {
        let mut sets = Vec::new();
        if let Some(ref tracking) = req.tracking_number { sets.push("tracking_number = ?"); }
        if let Some(ref carrier) = req.carrier { sets.push("carrier = ?"); }
        if let Some(ref status) = req.status {
            sets.push("status = ?");
            // Add delivery event for status change
            let event_id = Uuid::new_v4().to_string();
            sqlx::query(
                "INSERT INTO delivery_events (id, shipment_id, event_type, description) VALUES (?, ?, ?, ?)"
            )
            .bind(&event_id)
            .bind(id)
            .bind(status)
            .bind(format!("Status changed to {}", status))
            .execute(&self.pool)
            .await
            .ok();

            // Update order status if shipment is shipped/delivered
            if status == "shipped" {
                sqlx::query(
                    "UPDATE commerce_orders SET status = 'shipped', updated_at = NOW()::TEXT \
                     WHERE id IN (SELECT order_id FROM shipments WHERE id = ?)"
                )
                .bind(id)
                .execute(&self.pool)
                .await
                .ok();
            } else if status == "delivered" {
                sqlx::query(
                    "UPDATE commerce_orders SET status = 'delivered', updated_at = NOW()::TEXT \
                     WHERE id IN (SELECT order_id FROM shipments WHERE id = ?)"
                )
                .bind(id)
                .execute(&self.pool)
                .await
                .ok();
            }
        }
        if let Some(ref actual) = req.actual_delivery { sets.push("actual_delivery = ?"); }
        sets.push("updated_at = NOW()::TEXT");

        if sets.len() <= 1 {
            return self.get_shipment(id).await;
        }

        let query = format!("UPDATE shipments SET {} WHERE id = ?", sets.join(", "));
        sqlx::query(&query)
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| e.to_string())?;
        self.get_shipment(id).await
    }

    // ---------------------------------------------------------------------------
    // Delivery Events
    // ---------------------------------------------------------------------------

    pub async fn list_delivery_events(&self, shipment_id: &str) -> Result<Vec<DeliveryEvent>, String> {
        let rows = sqlx::query("SELECT * FROM delivery_events WHERE shipment_id = ? ORDER BY timestamp DESC")
            .bind(shipment_id)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| e.to_string())?;
        rows.iter()
            .map(|r| self.row_to_delivery_event(r))
            .collect::<Result<Vec<_>, _>>()
    }

    pub async fn add_delivery_event(&self, req: &AddDeliveryEventRequest) -> Result<DeliveryEvent, String> {
        let id = Uuid::new_v4().to_string();
        sqlx::query(
            "INSERT INTO delivery_events (id, shipment_id, event_type, location, description) \
             VALUES (?, ?, ?, ?, ?)"
        )
        .bind(&id)
        .bind(&req.shipment_id)
        .bind(&req.event_type)
        .bind(req.location.as_deref())
        .bind(req.description.as_deref())
        .execute(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        Ok(DeliveryEvent {
            id,
            shipment_id: req.shipment_id.clone(),
            event_type: req.event_type.clone(),
            location: req.location.clone(),
            description: req.description.clone(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        })
    }

    // ---------------------------------------------------------------------------
    // Row helpers
    // ---------------------------------------------------------------------------

    fn row_to_shipping_method(&self, r: &sqlx::postgres::PgRow) -> Result<ShippingMethod, String> {
        Ok(ShippingMethod {
            id: r.try_get("id").map_err(|e| e.to_string())?,
            brand_id: r.try_get("brand_id").map_err(|e| e.to_string())?,
            name: r.try_get("name").map_err(|e| e.to_string())?,
            r#type: r.try_get("type").map_err(|e| e.to_string())?,
            rate: r.try_get("rate").map_err(|e| e.to_string())?,
            free_shipping_threshold: r.try_get("free_shipping_threshold").map_err(|e| e.to_string())?,
            regions: r.try_get("regions").map_err(|e| e.to_string())?,
            is_active: r.try_get("is_active").map_err(|e| e.to_string())?,
            created_at: r.try_get("created_at").map_err(|e| e.to_string())?,
            updated_at: r.try_get("updated_at").map_err(|e| e.to_string())?,
        })
    }

    fn row_to_shipment(&self, r: &sqlx::postgres::PgRow) -> Result<Shipment, String> {
        Ok(Shipment {
            id: r.try_get("id").map_err(|e| e.to_string())?,
            order_id: r.try_get("order_id").map_err(|e| e.to_string())?,
            tracking_number: r.try_get("tracking_number").map_err(|e| e.to_string())?,
            carrier: r.try_get("carrier").map_err(|e| e.to_string())?,
            status: r.try_get("status").map_err(|e| e.to_string())?,
            estimated_delivery: r.try_get("estimated_delivery").map_err(|e| e.to_string())?,
            actual_delivery: r.try_get("actual_delivery").map_err(|e| e.to_string())?,
            shipping_label_url: r.try_get("shipping_label_url").map_err(|e| e.to_string())?,
            created_at: r.try_get("created_at").map_err(|e| e.to_string())?,
            updated_at: r.try_get("updated_at").map_err(|e| e.to_string())?,
        })
    }

    fn row_to_delivery_event(&self, r: &sqlx::postgres::PgRow) -> Result<DeliveryEvent, String> {
        Ok(DeliveryEvent {
            id: r.try_get("id").map_err(|e| e.to_string())?,
            shipment_id: r.try_get("shipment_id").map_err(|e| e.to_string())?,
            event_type: r.try_get("event_type").map_err(|e| e.to_string())?,
            location: r.try_get("location").map_err(|e| e.to_string())?,
            description: r.try_get("description").map_err(|e| e.to_string())?,
            timestamp: r.try_get("timestamp").map_err(|e| e.to_string())?,
        })
    }
}
