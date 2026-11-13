#![allow(dead_code)]

//! Analytics service: daily stats, revenue trends, top products, customer insights

use serde::{Deserialize, Serialize};
use sqlx::{Row, PgPool};
use uuid::Uuid;

use crate::models::{AnalyticsDailyStat, AnalyticsEvent};

// ---------------------------------------------------------------------------
// DTOs
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize)]
pub struct AnalyticsOverview {
    pub total_revenue: f64,
    pub total_orders: i32,
    pub avg_order_value: f64,
    pub total_customers: i32,
    pub total_products: i32,
    pub revenue_change_percent: f64,
    pub orders_change_percent: f64,
}

#[derive(Debug, Serialize)]
pub struct RevenueTrendPoint {
    pub date: String,
    pub revenue: f64,
    pub orders: i32,
}

#[derive(Debug, Serialize)]
pub struct TopProduct {
    pub product_id: String,
    pub product_name: String,
    pub sku: Option<String>,
    pub quantity_sold: i32,
    pub revenue: f64,
}

#[derive(Debug, Serialize)]
pub struct TopCustomer {
    pub customer_id: String,
    pub customer_name: Option<String>,
    pub email: Option<String>,
    pub total_orders: i32,
    pub total_spent: f64,
}

#[derive(Debug, Serialize)]
pub struct ChannelPerformance {
    pub channel_type: String,
    pub orders_count: i32,
    pub revenue: f64,
}

#[derive(Debug, Deserialize)]
pub struct RecordEventRequest {
    pub brand_id: String,
    pub event_type: String,
    pub entity_type: Option<String>,
    pub entity_id: Option<String>,
    pub properties: Option<serde_json::Value>,
}

// ---------------------------------------------------------------------------
// Service
// ---------------------------------------------------------------------------

pub struct CommerceAnalyticsService {
    pool: PgPool,
}

impl CommerceAnalyticsService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn overview(&self, brand_id: &str, days: i32) -> Result<AnalyticsOverview, String> {
        let days = if days > 0 { days } else { 30 };
        let cutoff = (chrono::Utc::now() - chrono::TimeDelta::days(days as i64)).format("%Y-%m-%dT%H:%M:%SZ").to_string();
        let cutoff2 = (chrono::Utc::now() - chrono::TimeDelta::days((days * 2) as i64)).format("%Y-%m-%dT%H:%M:%SZ").to_string();

        // Current period
        let current: (f64, i32) = sqlx::query_as(
            "SELECT COALESCE(SUM(total_amount), 0), COUNT(*) FROM commerce_orders \
             WHERE brand_id = ? AND created_at >= ?"
        )
        .bind(brand_id)
        .bind(&cutoff)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        // Previous period for comparison
        let previous: (f64, i32) = sqlx::query_as(
            "SELECT COALESCE(SUM(total_amount), 0), COUNT(*) FROM commerce_orders \
             WHERE brand_id = ? AND created_at >= ? AND created_at < ?"
        )
        .bind(brand_id)
        .bind(&cutoff2)
        .bind(&cutoff)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        let total_orders = current.1;
        let total_revenue = current.0;
        let avg_order_value = if total_orders > 0 { total_revenue / total_orders as f64 } else { 0.0 };

        let revenue_change = if previous.0 > 0.0 {
            ((current.0 - previous.0) / previous.0) * 100.0
        } else {
            0.0
        };
        let orders_change = if previous.1 > 0 {
            ((current.1 as f64 - previous.1 as f64) / previous.1 as f64) * 100.0
        } else {
            0.0
        };

        let total_customers: i32 = sqlx::query_scalar(
            "SELECT COUNT(DISTINCT customer_id) FROM commerce_orders WHERE brand_id = ?"
        )
        .bind(brand_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        let total_products: i32 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM products WHERE brand_id = ? AND is_active = 1"
        )
        .bind(brand_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        Ok(AnalyticsOverview {
            total_revenue,
            total_orders,
            avg_order_value,
            total_customers,
            total_products,
            revenue_change_percent: (revenue_change * 100.0).round() / 100.0,
            orders_change_percent: (orders_change * 100.0).round() / 100.0,
        })
    }

    pub async fn revenue_trend(&self, brand_id: &str, days: i32) -> Result<Vec<RevenueTrendPoint>, String> {
        let days = if days > 0 { days } else { 30 };
        let rows = sqlx::query(
            "SELECT DATE(created_at) as date, COALESCE(SUM(total_amount), 0) as revenue, COUNT(*) as orders \
             FROM commerce_orders WHERE brand_id = ? AND created_at >= CURRENT_DATE - ?::INTEGER \
             GROUP BY DATE(created_at) ORDER BY date"
        )
        .bind(brand_id)
        .bind(days)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        rows.iter()
            .map(|r| {
                Ok(RevenueTrendPoint {
                    date: r.try_get("date").map_err(|e| e.to_string())?,
                    revenue: r.try_get("revenue").map_err(|e| e.to_string())?,
                    orders: r.try_get("orders").map_err(|e| e.to_string())?,
                })
            })
            .collect::<Result<Vec<_>, _>>()
    }

    pub async fn top_products(&self, brand_id: &str, limit: i32) -> Result<Vec<TopProduct>, String> {
        let limit = if limit > 0 { limit } else { 10 };
        let rows = sqlx::query(
            "SELECT p.id as product_id, p.name as product_name, p.sku, \
             COALESCE(SUM(oi.quantity), 0) as quantity_sold, \
             COALESCE(SUM(oi.total_price), 0) as revenue \
             FROM commerce_order_items oi \
             JOIN commerce_orders o ON oi.order_id = o.id \
             JOIN products p ON oi.product_id = p.id \
             WHERE o.brand_id = ? \
             GROUP BY p.id \
             ORDER BY revenue DESC \
             LIMIT ?"
        )
        .bind(brand_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        rows.iter()
            .map(|r| {
                Ok(TopProduct {
                    product_id: r.try_get("product_id").map_err(|e| e.to_string())?,
                    product_name: r.try_get("product_name").map_err(|e| e.to_string())?,
                    sku: r.try_get("sku").map_err(|e| e.to_string())?,
                    quantity_sold: r.try_get("quantity_sold").map_err(|e| e.to_string())?,
                    revenue: r.try_get("revenue").map_err(|e| e.to_string())?,
                })
            })
            .collect::<Result<Vec<_>, _>>()
    }

    pub async fn top_customers(&self, brand_id: &str, limit: i32) -> Result<Vec<TopCustomer>, String> {
        let limit = if limit > 0 { limit } else { 10 };
        let rows = sqlx::query(
            "SELECT c.id as customer_id, c.name as customer_name, c.email, \
             COUNT(o.id) as total_orders, \
             COALESCE(SUM(o.total_amount), 0) as total_spent \
             FROM customers c \
             JOIN commerce_orders o ON c.id = o.customer_id \
             WHERE o.brand_id = ? \
             GROUP BY c.id \
             ORDER BY total_spent DESC \
             LIMIT ?"
        )
        .bind(brand_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        rows.iter()
            .map(|r| {
                Ok(TopCustomer {
                    customer_id: r.try_get("customer_id").map_err(|e| e.to_string())?,
                    customer_name: r.try_get("customer_name").map_err(|e| e.to_string())?,
                    email: r.try_get("email").map_err(|e| e.to_string())?,
                    total_orders: r.try_get("total_orders").map_err(|e| e.to_string())?,
                    total_spent: r.try_get("total_spent").map_err(|e| e.to_string())?,
                })
            })
            .collect::<Result<Vec<_>, _>>()
    }

    pub async fn channel_performance(&self, brand_id: &str) -> Result<Vec<ChannelPerformance>, String> {
        let rows = sqlx::query(
            "SELECT COALESCE(channel_type, 'direct') as channel_type, \
             COUNT(*) as orders_count, \
             COALESCE(SUM(total_amount), 0) as revenue \
             FROM commerce_orders WHERE brand_id = ? \
             GROUP BY channel_type ORDER BY revenue DESC"
        )
        .bind(brand_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        rows.iter()
            .map(|r| {
                Ok(ChannelPerformance {
                    channel_type: r.try_get("channel_type").map_err(|e| e.to_string())?,
                    orders_count: r.try_get("orders_count").map_err(|e| e.to_string())?,
                    revenue: r.try_get("revenue").map_err(|e| e.to_string())?,
                })
            })
            .collect::<Result<Vec<_>, _>>()
    }

    pub async fn update_daily_stats(&self, brand_id: &str) -> Result<(), String> {
        let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
        let start_of_day = format!("{}T00:00:00Z", today);
        let end_of_day = format!("{}T23:59:59Z", today);

        let stats: (i32, f64, i32, i32) = sqlx::query_as(
            "SELECT COUNT(*), COALESCE(SUM(total_amount), 0), \
             COUNT(DISTINCT customer_id), \
             (SELECT COALESCE(SUM(quantity), 0) FROM commerce_order_items \
              WHERE order_id IN (SELECT id FROM commerce_orders WHERE brand_id = ? AND created_at >= ? AND created_at <= ?)) \
             FROM commerce_orders WHERE brand_id = ? AND created_at >= ? AND created_at <= ?"
        )
        .bind(brand_id)
        .bind(&start_of_day)
        .bind(&end_of_day)
        .bind(brand_id)
        .bind(&start_of_day)
        .bind(&end_of_day)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        let orders_count = stats.0;
        let revenue = stats.1;
        let avg_order_value = if orders_count > 0 { revenue / orders_count as f64 } else { 0.0 };
        let unique_customers = stats.2;
        let products_sold = stats.3;

        let id = Uuid::new_v4().to_string();
        sqlx::query(
            "INSERT INTO analytics_daily_stats (id, brand_id, date, orders_count, revenue, \
             avg_order_value, unique_customers, products_sold) \
             VALUES (?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(&id)
        .bind(brand_id)
        .bind(&today)
        .bind(orders_count)
        .bind(revenue)
        .bind(avg_order_value)
        .bind(unique_customers)
        .bind(products_sold)
        .execute(&self.pool)
        .await
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub async fn record_event(&self, req: &RecordEventRequest) -> Result<AnalyticsEvent, String> {
        let id = Uuid::new_v4().to_string();
        let properties = req.properties.as_ref().map(|v| serde_json::to_string(v).unwrap_or_default());
        sqlx::query(
            "INSERT INTO analytics_events (id, brand_id, event_type, entity_type, entity_id, properties) \
             VALUES (?, ?, ?, ?, ?, ?)"
        )
        .bind(&id)
        .bind(&req.brand_id)
        .bind(&req.event_type)
        .bind(req.entity_type.as_deref())
        .bind(req.entity_id.as_deref())
        .bind(properties.as_deref())
        .execute(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        Ok(AnalyticsEvent {
            id,
            brand_id: req.brand_id.clone(),
            event_type: req.event_type.clone(),
            entity_type: req.entity_type.clone().unwrap_or_default(),
            entity_id: req.entity_id.clone().unwrap_or_default(),
            properties,
            created_at: chrono::Utc::now().to_rfc3339(),
        })
    }

    pub async fn daily_stats(&self, brand_id: &str, days: i32) -> Result<Vec<AnalyticsDailyStat>, String> {
        let days = if days > 0 { days } else { 30 };
        let rows = sqlx::query(
            "SELECT * FROM analytics_daily_stats WHERE brand_id = ? AND date >= CURRENT_DATE - ?::INTEGER \
             ORDER BY date"
        )
        .bind(brand_id)
        .bind(days)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        rows.iter()
            .map(|r| self.row_to_daily_stat(r))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())
    }

    fn row_to_daily_stat(&self, r: &sqlx::postgres::PgRow) -> Result<AnalyticsDailyStat, String> {
        Ok(AnalyticsDailyStat {
            id: r.try_get("id").map_err(|e| e.to_string())?,
            brand_id: r.try_get("brand_id").map_err(|e| e.to_string())?,
            date: r.try_get("date").map_err(|e| e.to_string())?,
            orders_count: r.try_get("orders_count").map_err(|e| e.to_string())?,
            revenue: r.try_get("revenue").map_err(|e| e.to_string())?,
            avg_order_value: r.try_get("avg_order_value").map_err(|e| e.to_string())?,
            unique_customers: r.try_get("unique_customers").map_err(|e| e.to_string())?,
            products_sold: r.try_get("products_sold").map_err(|e| e.to_string())?,
            created_at: r.try_get("created_at").map_err(|e| e.to_string())?,
        })
    }
}
