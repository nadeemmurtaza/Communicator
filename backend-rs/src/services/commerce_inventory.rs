#![allow(dead_code)]

//! Advanced Inventory service: warehouses, inventory tracking, low-stock alerts, back-in-stock

use serde::{Deserialize, Serialize};
use sqlx::{Row, PgPool};
use uuid::Uuid;

use crate::models::{Warehouse, InventoryByWarehouse, InventoryTransaction, LowStockAlert, BackInStockRequest};

// ---------------------------------------------------------------------------
// DTOs
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct CreateWarehouseRequest {
    pub brand_id: String,
    pub name: String,
    pub address: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateWarehouseRequest {
    pub name: Option<String>,
    pub address: Option<serde_json::Value>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct SetInventoryRequest {
    pub product_id: String,
    pub warehouse_id: String,
    pub quantity: i32,
    pub reorder_point: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct TransferInventoryRequest {
    pub product_id: String,
    pub from_warehouse_id: String,
    pub to_warehouse_id: String,
    pub quantity: i32,
}

#[derive(Debug, Deserialize)]
pub struct BackInStockRequestCreate {
    pub product_id: String,
    pub customer_id: String,
    pub channel_type: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct InventorySummary {
    pub product_id: String,
    pub product_name: String,
    pub total_quantity: i32,
    pub total_reserved: i32,
    pub available_quantity: i32,
    pub warehouses: Vec<WarehouseInventoryItem>,
}

#[derive(Debug, Serialize)]
pub struct WarehouseInventoryItem {
    pub warehouse_id: String,
    pub warehouse_name: String,
    pub quantity: i32,
    pub reserved_quantity: i32,
    pub available: i32,
}

// ---------------------------------------------------------------------------
// Service
// ---------------------------------------------------------------------------

pub struct CommerceInventoryService {
    pool: PgPool,
}

impl CommerceInventoryService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    // ---------------------------------------------------------------------------
    // Warehouses
    // ---------------------------------------------------------------------------

    pub async fn list_warehouses(&self, brand_id: &str) -> Result<Vec<Warehouse>, String> {
        let rows = sqlx::query("SELECT * FROM warehouses WHERE brand_id = $1 ORDER BY name")
            .bind(brand_id)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| e.to_string())?;
        rows.iter()
            .map(|r| self.row_to_warehouse(r))
            .collect::<Result<Vec<_>, _>>()
    }

    pub async fn get_warehouse(&self, id: &str) -> Result<Warehouse, String> {
        let row = sqlx::query("SELECT * FROM warehouses WHERE id = $1")
            .bind(id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| e.to_string())?;
        self.row_to_warehouse(&row)
    }

    pub async fn create_warehouse(&self, req: &CreateWarehouseRequest) -> Result<Warehouse, String> {
        let id = Uuid::new_v4().to_string();
        let address = req.address.as_ref().map(|v| serde_json::to_string(v).unwrap_or_default());
        sqlx::query(
            "INSERT INTO warehouses (id, brand_id, name, address) VALUES ($1, $2, $3, $4)"
        )
        .bind(&id)
        .bind(&req.brand_id)
        .bind(&req.name)
        .bind(address.as_deref())
        .execute(&self.pool)
        .await
        .map_err(|e| e.to_string())?;
        self.get_warehouse(&id).await
    }

    pub async fn update_warehouse(&self, id: &str, req: &UpdateWarehouseRequest) -> Result<Warehouse, String> {
        let mut sets: Vec<String> = Vec::new();
        let mut n = 1;
        if req.name.is_some() { sets.push(format!("name = ${}", n)); n += 1; }
        if req.is_active.is_some() { sets.push(format!("is_active = ${}", n)); n += 1; }
        sets.push("updated_at = NOW()::TEXT".to_string());

        if sets.len() <= 1 {
            return self.get_warehouse(id).await;
        }

        let query = format!("UPDATE warehouses SET {} WHERE id = ${}", sets.join(", "), n);
        let mut q = sqlx::query(&query);
        if let Some(ref name) = req.name { q = q.bind(name); }
        if let Some(is_active) = req.is_active { q = q.bind(is_active); }
        q = q.bind(id);
        q.execute(&self.pool)
            .await
            .map_err(|e| e.to_string())?;
        self.get_warehouse(id).await
    }

    pub async fn delete_warehouse(&self, id: &str) -> Result<(), String> {
        sqlx::query("DELETE FROM warehouses WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    // ---------------------------------------------------------------------------
    // Inventory
    // ---------------------------------------------------------------------------

    pub async fn get_inventory(&self, product_id: &str) -> Result<Vec<InventoryByWarehouse>, String> {
        let rows = sqlx::query("SELECT * FROM inventory_by_warehouse WHERE product_id = $1")
            .bind(product_id)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| e.to_string())?;
        rows.iter()
            .map(|r| self.row_to_inventory(r))
            .collect::<Result<Vec<_>, _>>()
    }

    pub async fn get_inventory_summary(&self, product_id: &str) -> Result<InventorySummary, String> {
        let inventory = self.get_inventory(product_id).await?;

        // Get product name
        let product_name: String = sqlx::query_scalar("SELECT name FROM products WHERE id = $1")
            .bind(product_id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| e.to_string())?;

        let mut total_quantity = 0;
        let mut total_reserved = 0;
        let mut warehouses = Vec::new();

        for inv in &inventory {
            let warehouse = self.get_warehouse(&inv.warehouse_id).await.ok();
            let available = inv.quantity.saturating_sub(inv.reserved_quantity);
            total_quantity += inv.quantity;
            total_reserved += inv.reserved_quantity;

            if let Some(w) = warehouse {
                warehouses.push(WarehouseInventoryItem {
                    warehouse_id: inv.warehouse_id.clone(),
                    warehouse_name: w.name,
                    quantity: inv.quantity,
                    reserved_quantity: inv.reserved_quantity,
                    available,
                });
            }
        }

        Ok(InventorySummary {
            product_id: product_id.to_string(),
            product_name,
            total_quantity,
            total_reserved,
            available_quantity: total_quantity.saturating_sub(total_reserved),
            warehouses,
        })
    }

    pub async fn set_inventory(&self, req: &SetInventoryRequest) -> Result<InventoryByWarehouse, String> {
        let existing = sqlx::query("SELECT id FROM inventory_by_warehouse WHERE product_id = $1 AND warehouse_id = $2")
            .bind(&req.product_id)
            .bind(&req.warehouse_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| e.to_string())?;

        let reorder_point = req.reorder_point.unwrap_or(0);

        if let Some(row) = existing {
            let id: String = row.try_get("id").map_err(|e| e.to_string())?;
            sqlx::query(
                "UPDATE inventory_by_warehouse SET quantity = $1, reorder_point = $2, \
                 updated_at = NOW()::TEXT \
                 WHERE id = $3"
            )
            .bind(req.quantity)
            .bind(reorder_point)
            .bind(&id)
            .execute(&self.pool)
            .await
            .map_err(|e| e.to_string())?;

            // Record transaction
            self.record_transaction(&req.product_id, &req.warehouse_id, "adjustment", req.quantity, None, None).await.ok();

            // Check low stock
            if req.quantity > 0 && req.quantity <= reorder_point {
                self.create_low_stock_alert(&req.product_id, &req.warehouse_id, req.quantity, reorder_point).await.ok();
            }
        } else {
            let id = Uuid::new_v4().to_string();
            sqlx::query(
                "INSERT INTO inventory_by_warehouse (id, product_id, warehouse_id, quantity, reorder_point) \
                 VALUES ($1, $2, $3, $4, $5)"
            )
            .bind(&id)
            .bind(&req.product_id)
            .bind(&req.warehouse_id)
            .bind(req.quantity)
            .bind(reorder_point)
            .execute(&self.pool)
            .await
            .map_err(|e| e.to_string())?;

            self.record_transaction(&req.product_id, &req.warehouse_id, "initial_stock", req.quantity, None, None).await.ok();
        }

        self.get_inventory_by_product_warehouse(&req.product_id, &req.warehouse_id).await
    }

    pub async fn get_inventory_by_product_warehouse(&self, product_id: &str, warehouse_id: &str) -> Result<InventoryByWarehouse, String> {
        let row = sqlx::query("SELECT * FROM inventory_by_warehouse WHERE product_id = $1 AND warehouse_id = $2")
            .bind(product_id)
            .bind(warehouse_id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| e.to_string())?;
        self.row_to_inventory(&row)
    }

    pub async fn transfer_inventory(&self, req: &TransferInventoryRequest) -> Result<(), String> {
        // Check source has enough stock
        let source = self.get_inventory_by_product_warehouse(&req.product_id, &req.from_warehouse_id).await?;
        let available = source.quantity.saturating_sub(source.reserved_quantity);
        if available < req.quantity {
            return Err(format!("Insufficient stock: available {}, requested {}", available, req.quantity));
        }

        // Deduct from source
        sqlx::query(
            "UPDATE inventory_by_warehouse SET quantity = quantity - $1, \
             updated_at = NOW()::TEXT \
             WHERE product_id = $2 AND warehouse_id = $3"
        )
        .bind(req.quantity)
        .bind(&req.product_id)
        .bind(&req.from_warehouse_id)
        .execute(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        // Add to destination
        let dest_existing = sqlx::query("SELECT id FROM inventory_by_warehouse WHERE product_id = $1 AND warehouse_id = $2")
            .bind(&req.product_id)
            .bind(&req.to_warehouse_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| e.to_string())?;

        if let Some(_row) = dest_existing {
            sqlx::query(
                "UPDATE inventory_by_warehouse SET quantity = quantity + $1, \
                 updated_at = NOW()::TEXT \
                 WHERE product_id = $2 AND warehouse_id = $3"
            )
            .bind(req.quantity)
            .bind(&req.product_id)
            .bind(&req.to_warehouse_id)
            .execute(&self.pool)
            .await
            .map_err(|e| e.to_string())?;
        } else {
            let id = Uuid::new_v4().to_string();
            sqlx::query(
                "INSERT INTO inventory_by_warehouse (id, product_id, warehouse_id, quantity) \
                 VALUES ($1, $2, $3, $4)"
            )
            .bind(&id)
            .bind(&req.product_id)
            .bind(&req.to_warehouse_id)
            .bind(req.quantity)
            .execute(&self.pool)
            .await
            .map_err(|e| e.to_string())?;
        }

        // Record transactions
        let to_wh = req.to_warehouse_id.clone();
        let from_wh = req.from_warehouse_id.clone();
        self.record_transaction(&req.product_id, &req.from_warehouse_id, "transfer_out", -(req.quantity as i32), Some("transfer"), Some(&to_wh)).await?;
        self.record_transaction(&req.product_id, &req.to_warehouse_id, "transfer_in", req.quantity, Some("transfer"), Some(&from_wh)).await?;

        Ok(())
    }

    pub async fn reserve_inventory(&self, product_id: &str, warehouse_id: &str, quantity: i32) -> Result<bool, String> {
        let inv = self.get_inventory_by_product_warehouse(product_id, warehouse_id).await?;
        let available = inv.quantity.saturating_sub(inv.reserved_quantity);
        if available < quantity {
            return Ok(false);
        }

        sqlx::query(
            "UPDATE inventory_by_warehouse SET reserved_quantity = reserved_quantity + $1, \
             updated_at = NOW()::TEXT \
             WHERE product_id = $2 AND warehouse_id = $3"
        )
        .bind(quantity)
        .bind(product_id)
        .bind(warehouse_id)
        .execute(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        self.record_transaction(product_id, warehouse_id, "reserve", quantity, None, None).await.ok();
        Ok(true)
    }

    pub async fn release_inventory(&self, product_id: &str, warehouse_id: &str, quantity: i32) -> Result<(), String> {
        sqlx::query(
            "UPDATE inventory_by_warehouse SET reserved_quantity = MAX(0, reserved_quantity - $1), \
             updated_at = NOW()::TEXT \
             WHERE product_id = $2 AND warehouse_id = $3"
        )
        .bind(quantity)
        .bind(product_id)
        .bind(warehouse_id)
        .execute(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        self.record_transaction(product_id, warehouse_id, "release_reserve", -(quantity), None, None).await.ok();
        Ok(())
    }

    pub async fn consume_inventory(&self, product_id: &str, warehouse_id: &str, quantity: i32) -> Result<(), String> {
        sqlx::query(
            "UPDATE inventory_by_warehouse SET quantity = quantity - $1, reserved_quantity = MAX(0, reserved_quantity - $2), \
             updated_at = NOW()::TEXT \
             WHERE product_id = $3 AND warehouse_id = $4"
        )
        .bind(quantity)
        .bind(quantity)
        .bind(product_id)
        .bind(warehouse_id)
        .execute(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        self.record_transaction(product_id, warehouse_id, "sale", -(quantity), None, None).await.ok();

        // Check low stock after consumption
        let inv = self.get_inventory_by_product_warehouse(product_id, warehouse_id).await.ok();
        let has_stock = if let Some(ref inv) = inv {
            if inv.reorder_point > 0 && inv.quantity <= inv.reorder_point {
                self.create_low_stock_alert(product_id, warehouse_id, inv.quantity, inv.reorder_point).await.ok();
            }
            inv.quantity > 0
        } else {
            false
        };

        // Check back-in-stock requests if inventory was depleted and now has stock
        if has_stock {
            self.notify_back_in_stock(product_id).await.ok();
        }

        Ok(())
    }

    // ---------------------------------------------------------------------------
    // Low Stock Alerts
    // ---------------------------------------------------------------------------

    pub async fn list_low_stock_alerts(&self, acknowledged: Option<bool>) -> Result<Vec<LowStockAlert>, String> {
        let query = match acknowledged {
            Some(true) => "SELECT * FROM low_stock_alerts WHERE acknowledged = 1 ORDER BY alerted_at DESC",
            Some(false) => "SELECT * FROM low_stock_alerts WHERE acknowledged = 0 ORDER BY alerted_at DESC",
            None => "SELECT * FROM low_stock_alerts ORDER BY alerted_at DESC",
        };
        let rows = sqlx::query(query)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| e.to_string())?;
        rows.iter()
            .map(|r| self.row_to_low_stock_alert(r))
            .collect::<Result<Vec<_>, _>>()
    }

    pub async fn acknowledge_alert(&self, id: &str) -> Result<(), String> {
        sqlx::query("UPDATE low_stock_alerts SET acknowledged = 1 WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub async fn create_low_stock_alert(&self, product_id: &str, warehouse_id: &str, current_quantity: i32, reorder_point: i32) -> Result<LowStockAlert, String> {
        let id = Uuid::new_v4().to_string();
        sqlx::query(
            "INSERT INTO low_stock_alerts (id, product_id, warehouse_id, current_quantity, reorder_point) \
             VALUES ($1, $2, $3, $4, $5)"
        )
        .bind(&id)
        .bind(product_id)
        .bind(warehouse_id)
        .bind(current_quantity)
        .bind(reorder_point)
        .execute(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        Ok(LowStockAlert {
            id,
            product_id: product_id.to_string(),
            warehouse_id: warehouse_id.to_string(),
            current_quantity,
            reorder_point,
            alerted_at: chrono::Utc::now().to_rfc3339(),
            acknowledged: false,
        })
    }

    // ---------------------------------------------------------------------------
    // Back in Stock Requests
    // ---------------------------------------------------------------------------

    pub async fn list_back_in_stock_requests(&self, product_id: Option<&str>) -> Result<Vec<BackInStockRequest>, String> {
        let (query, rows) = if let Some(pid) = product_id {
            let q = "SELECT * FROM back_in_stock_requests WHERE product_id = $1 ORDER BY created_at DESC";
            (q, sqlx::query(q).bind(pid).fetch_all(&self.pool).await.map_err(|e| e.to_string())?)
        } else {
            let q = "SELECT * FROM back_in_stock_requests ORDER BY created_at DESC LIMIT 100";
            (q, sqlx::query(q).fetch_all(&self.pool).await.map_err(|e| e.to_string())?)
        };
        rows.iter()
            .map(|r| self.row_to_back_in_stock(r))
            .collect::<Result<Vec<_>, _>>()
    }

    pub async fn create_back_in_stock_request(&self, req: &BackInStockRequestCreate) -> Result<BackInStockRequest, String> {
        let id = Uuid::new_v4().to_string();
        sqlx::query(
            "INSERT INTO back_in_stock_requests (id, product_id, customer_id, channel_type) \
             VALUES ($1, $2, $3, $4)"
        )
        .bind(&id)
        .bind(&req.product_id)
        .bind(&req.customer_id)
        .bind(req.channel_type.as_deref())
        .execute(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        Ok(BackInStockRequest {
            id,
            product_id: req.product_id.clone(),
            customer_id: req.customer_id.clone(),
            channel_type: req.channel_type.clone(),
            notified_at: None,
            status: "pending".to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
        })
    }

    pub async fn notify_back_in_stock(&self, product_id: &str) -> Result<usize, String> {
        let result = sqlx::query(
            "UPDATE back_in_stock_requests SET status = 'notified', notified_at = NOW()::TEXT \
             WHERE product_id = $1 AND status = 'pending'"
        )
        .bind(product_id)
        .execute(&self.pool)
        .await
        .map_err(|e| e.to_string())?;
        Ok(result.rows_affected() as usize)
    }

    // ---------------------------------------------------------------------------
    // Transactions
    // ---------------------------------------------------------------------------

    pub async fn record_transaction(&self, product_id: &str, warehouse_id: &str, transaction_type: &str, quantity: i32, reference_type: Option<&str>, reference_id: Option<&str>) -> Result<InventoryTransaction, String> {
        let id = Uuid::new_v4().to_string();
        sqlx::query(
            "INSERT INTO inventory_transactions (id, product_id, warehouse_id, transaction_type, quantity, reference_type, reference_id) \
             VALUES ($1, $2, $3, $4, $5, $6, $7)"
        )
        .bind(&id)
        .bind(product_id)
        .bind(warehouse_id)
        .bind(transaction_type)
        .bind(quantity)
        .bind(reference_type)
        .bind(reference_id)
        .execute(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        Ok(InventoryTransaction {
            id,
            product_id: product_id.to_string(),
            warehouse_id: warehouse_id.to_string(),
            transaction_type: transaction_type.to_string(),
            quantity,
            reference_type: reference_type.map(|s| s.to_string()),
            reference_id: reference_id.map(|s| s.to_string()),
            created_at: chrono::Utc::now().to_rfc3339(),
        })
    }

    pub async fn transaction_history(&self, product_id: &str, warehouse_id: Option<&str>) -> Result<Vec<InventoryTransaction>, String> {
        let (query, rows) = if let Some(wh) = warehouse_id {
            let q = "SELECT * FROM inventory_transactions WHERE product_id = $1 AND warehouse_id = $2 ORDER BY created_at DESC LIMIT 100";
            (q, sqlx::query(q).bind(product_id).bind(wh).fetch_all(&self.pool).await.map_err(|e| e.to_string())?)
        } else {
            let q = "SELECT * FROM inventory_transactions WHERE product_id = $1 ORDER BY created_at DESC LIMIT 100";
            (q, sqlx::query(q).bind(product_id).fetch_all(&self.pool).await.map_err(|e| e.to_string())?)
        };
        rows.iter()
            .map(|r| self.row_to_transaction(r))
            .collect::<Result<Vec<_>, _>>()
    }

    // ---------------------------------------------------------------------------
    // Aggregate inventory for product (across all warehouses)
    // ---------------------------------------------------------------------------

    pub async fn aggregate_inventory(&self, product_id: &str) -> Result<i32, String> {
        let row = sqlx::query(
            "SELECT COALESCE(SUM(quantity - reserved_quantity), 0) as total FROM inventory_by_warehouse WHERE product_id = $1"
        )
        .bind(product_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| e.to_string())?;
        let total: i32 = row.get("total");
        Ok(total)
    }

    // ---------------------------------------------------------------------------
    // Row helpers
    // ---------------------------------------------------------------------------

    fn row_to_warehouse(&self, r: &sqlx::postgres::PgRow) -> Result<Warehouse, String> {
        Ok(Warehouse {
            id: r.try_get("id").map_err(|e| e.to_string())?,
            brand_id: r.try_get("brand_id").map_err(|e| e.to_string())?,
            name: r.try_get("name").map_err(|e| e.to_string())?,
            address: r.try_get("address").map_err(|e| e.to_string())?,
            is_active: r.try_get("is_active").map_err(|e| e.to_string())?,
            created_at: r.try_get("created_at").map_err(|e| e.to_string())?,
            updated_at: r.try_get("updated_at").map_err(|e| e.to_string())?,
        })
    }

    fn row_to_inventory(&self, r: &sqlx::postgres::PgRow) -> Result<InventoryByWarehouse, String> {
        Ok(InventoryByWarehouse {
            id: r.try_get("id").map_err(|e| e.to_string())?,
            product_id: r.try_get("product_id").map_err(|e| e.to_string())?,
            warehouse_id: r.try_get("warehouse_id").map_err(|e| e.to_string())?,
            quantity: r.try_get("quantity").map_err(|e| e.to_string())?,
            reserved_quantity: r.try_get("reserved_quantity").map_err(|e| e.to_string())?,
            reorder_point: r.try_get("reorder_point").map_err(|e| e.to_string())?,
            created_at: r.try_get("created_at").map_err(|e| e.to_string())?,
            updated_at: r.try_get("updated_at").map_err(|e| e.to_string())?,
        })
    }

    fn row_to_transaction(&self, r: &sqlx::postgres::PgRow) -> Result<InventoryTransaction, String> {
        Ok(InventoryTransaction {
            id: r.try_get("id").map_err(|e| e.to_string())?,
            product_id: r.try_get("product_id").map_err(|e| e.to_string())?,
            warehouse_id: r.try_get("warehouse_id").map_err(|e| e.to_string())?,
            transaction_type: r.try_get("transaction_type").map_err(|e| e.to_string())?,
            quantity: r.try_get("quantity").map_err(|e| e.to_string())?,
            reference_type: r.try_get("reference_type").map_err(|e| e.to_string())?,
            reference_id: r.try_get("reference_id").map_err(|e| e.to_string())?,
            created_at: r.try_get("created_at").map_err(|e| e.to_string())?,
        })
    }

    fn row_to_low_stock_alert(&self, r: &sqlx::postgres::PgRow) -> Result<LowStockAlert, String> {
        Ok(LowStockAlert {
            id: r.try_get("id").map_err(|e| e.to_string())?,
            product_id: r.try_get("product_id").map_err(|e| e.to_string())?,
            warehouse_id: r.try_get("warehouse_id").map_err(|e| e.to_string())?,
            current_quantity: r.try_get("current_quantity").map_err(|e| e.to_string())?,
            reorder_point: r.try_get("reorder_point").map_err(|e| e.to_string())?,
            alerted_at: r.try_get("alerted_at").map_err(|e| e.to_string())?,
            acknowledged: r.try_get("acknowledged").map_err(|e| e.to_string())?,
        })
    }

    fn row_to_back_in_stock(&self, r: &sqlx::postgres::PgRow) -> Result<BackInStockRequest, String> {
        Ok(BackInStockRequest {
            id: r.try_get("id").map_err(|e| e.to_string())?,
            product_id: r.try_get("product_id").map_err(|e| e.to_string())?,
            customer_id: r.try_get("customer_id").map_err(|e| e.to_string())?,
            channel_type: r.try_get("channel_type").map_err(|e| e.to_string())?,
            notified_at: r.try_get("notified_at").map_err(|e| e.to_string())?,
            status: r.try_get("status").map_err(|e| e.to_string())?,
            created_at: r.try_get("created_at").map_err(|e| e.to_string())?,
        })
    }
}
