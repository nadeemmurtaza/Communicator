use sqlx::{Pool, Postgres};
use uuid::Uuid;
use chrono::Utc;
use crate::models::*;

pub struct ErpOrdersService;

impl ErpOrdersService {
    // ============================================================
    // Order Number Generation
    // ============================================================

    pub async fn generate_order_number(pool: &Pool<Postgres>) -> Result<String, String> {
        let year = Utc::now().format("%Y").to_string();
        let year_int: i32 = year.parse::<i32>().map_err(|e| e.to_string())?;

        // Atomic increment using INSERT ... ON CONFLICT DO UPDATE
        let row: (i32,) = sqlx::query_as(
            "INSERT INTO erp_order_number_counters (year, counter) VALUES ($1, 0) ON CONFLICT (year) DO UPDATE SET counter = erp_order_number_counters.counter + 1 RETURNING counter"
        )
        .bind(year_int)
        .fetch_one(pool)
        .await
        .map_err(|e| e.to_string())?;

        let counter = row.0;
        Ok(format!("YSN-{}-{:04}", year, counter))
    }

    // ============================================================
    // Orders
    // ============================================================

    pub async fn list_orders(
        pool: &Pool<Postgres>,
        status: Option<&str>,
        channel: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<ErpOrder>, String> {
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

        let orders = sqlx::query_as::<_, ErpOrder>(&query)
            .bind(limit)
            .bind(offset)
            .fetch_all(pool)
            .await
            .map_err(|e| e.to_string())?;

        Ok(orders)
    }

    pub async fn get_order(pool: &Pool<Postgres>, id: &str) -> Result<ErpOrder, String> {
        let order = sqlx::query_as::<_, ErpOrder>(
            "SELECT * FROM erp_orders WHERE id = $1"
        )
        .fetch_one(pool)
        .await
        .map_err(|e| e.to_string())?;
        Ok(order)
    }

    pub async fn get_order_by_number(pool: &Pool<Postgres>, number: &str) -> Result<ErpOrder, String> {
        let order = sqlx::query_as::<_, ErpOrder>(
            "SELECT * FROM erp_orders WHERE number = $1"
        )
        .fetch_one(pool)
        .await
        .map_err(|e| e.to_string())?;
        Ok(order)
    }

    pub async fn create_order(
        pool: &Pool<Postgres>,
        req: &CreateOrderRequest,
    ) -> Result<ErpOrder, String> {
        let id = Uuid::new_v4().to_string();
        let number = Self::generate_order_number(pool).await?;
        let now = Utc::now().to_rfc3339();

        // Calculate totals from items
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

        // Insert order
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
        .execute(pool)
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
            .execute(pool)
            .await
            .map_err(|e| e.to_string())?;
        }

        // If source cart was provided, mark it as converted
        if let Some(ref cart_id) = req.source_cart_id {
            let _ = sqlx::query(
                "UPDATE erp_carts SET status = 'converted', updated_at = $1 WHERE id = $2"
            )
            .bind(&now)
            .bind(cart_id)
            .execute(pool)
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
        .execute(pool)
        .await;

        Self::get_order(pool, &id).await
    }

    pub async fn update_order_status(
        pool: &Pool<Postgres>,
        order_id: &str,
        req: &UpdateOrderStatusRequest,
    ) -> Result<ErpOrder, String> {
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
            .execute(pool)
            .await
            .map_err(|e| e.to_string())?;

        // Create status event
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
        .execute(pool)
        .await;

        Self::get_order(pool, order_id).await
    }

    pub async fn get_order_items(pool: &Pool<Postgres>, order_id: &str) -> Result<Vec<ErpOrderItem>, String> {
        let items = sqlx::query_as::<_, ErpOrderItem>(
            "SELECT * FROM erp_order_items WHERE order_id = $1 ORDER BY id"
        )
        .fetch_all(pool)
        .await
        .map_err(|e| e.to_string())?;
        Ok(items)
    }

    pub async fn get_order_status_events(pool: &Pool<Postgres>, order_id: &str) -> Result<Vec<ErpOrderStatusEvent>, String> {
        let events = sqlx::query_as::<_, ErpOrderStatusEvent>(
            "SELECT * FROM erp_order_status_events WHERE order_id = $1 ORDER BY created_at DESC"
        )
        .fetch_all(pool)
        .await
        .map_err(|e| e.to_string())?;
        Ok(events)
    }

    pub async fn cancel_order(
        pool: &Pool<Postgres>,
        order_id: &str,
        reason: &str,
        actor_id: Option<&str>,
    ) -> Result<ErpOrder, String> {
        let now = Utc::now().to_rfc3339();

        sqlx::query(
            "UPDATE erp_orders SET status = 'CANCELLED', cancel_reason = $1, cancelled_at = $2, updated_at = $2 WHERE id = $3"
        )
        .bind(reason)
        .bind(&now)
        .bind(order_id)
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;

        // Create status event
        let event_id = Uuid::new_v4().to_string();
        let _ = sqlx::query(
            "INSERT INTO erp_order_status_events (id, order_id, to_status, note, actor_id, created_at) VALUES ($1, $2, 'CANCELLED', $3, $4, $5)"
        )
        .bind(&event_id)
        .bind(order_id)
        .bind(reason)
        .bind(actor_id.map(|s| s.to_string()))
        .bind(&now)
        .execute(pool)
        .await;

        Self::get_order(pool, order_id).await
    }

    // ============================================================
    // Carts
    // ============================================================

    pub async fn list_carts(
        pool: &Pool<Postgres>,
        user_id: Option<&str>,
        status: Option<&str>,
    ) -> Result<Vec<ErpCart>, String> {
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

        let carts = sqlx::query_as::<_, ErpCart>(&query)
            .fetch_all(pool)
            .await
            .map_err(|e| e.to_string())?;
        Ok(carts)
    }

    pub async fn get_cart(pool: &Pool<Postgres>, id: &str) -> Result<ErpCart, String> {
        let cart = sqlx::query_as::<_, ErpCart>(
            "SELECT * FROM erp_carts WHERE id = $1"
        )
        .fetch_one(pool)
        .await
        .map_err(|e| e.to_string())?;
        Ok(cart)
    }

    pub async fn create_cart(
        pool: &Pool<Postgres>,
        req: &CreateCartRequest,
    ) -> Result<ErpCart, String> {
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
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;

        Self::get_cart(pool, &id).await
    }

    pub async fn delete_cart(pool: &Pool<Postgres>, id: &str) -> Result<(), String> {
        sqlx::query("DELETE FROM erp_carts WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub async fn get_cart_items(pool: &Pool<Postgres>, cart_id: &str) -> Result<Vec<ErpCartItem>, String> {
        let items = sqlx::query_as::<_, ErpCartItem>(
            "SELECT * FROM erp_cart_items WHERE cart_id = $1 ORDER BY created_at"
        )
        .fetch_all(pool)
        .await
        .map_err(|e| e.to_string())?;
        Ok(items)
    }

    pub async fn add_cart_item(
        pool: &Pool<Postgres>,
        req: &CreateCartItemRequest,
    ) -> Result<ErpCartItem, String> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();

        // Check if item already exists in cart
        let existing: Option<ErpCartItem> = sqlx::query_as::<_, ErpCartItem>(
            "SELECT * FROM erp_cart_items WHERE cart_id = $1 AND product_id = $2 AND variant_id = $3"
        )
        .bind(&req.cart_id)
        .bind(&req.product_id)
        .bind(&req.variant_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| e.to_string())?;

        if let Some(item) = existing {
            // Update quantity
            let new_qty = item.quantity + req.quantity;
            sqlx::query(
                "UPDATE erp_cart_items SET quantity = $1 WHERE id = $2"
            )
            .bind(new_qty)
            .bind(&item.id)
            .execute(pool)
            .await
            .map_err(|e| e.to_string())?;

            sqlx::query_as::<_, ErpCartItem>("SELECT * FROM erp_cart_items WHERE id = $1")
                .bind(&item.id)
                .fetch_one(pool)
                .await
                .map_err(|e| e.to_string())
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
            .execute(pool)
            .await
            .map_err(|e| e.to_string())?;

            sqlx::query_as::<_, ErpCartItem>("SELECT * FROM erp_cart_items WHERE id = $1")
                .bind(&id)
                .fetch_one(pool)
                .await
                .map_err(|e| e.to_string())
        }
    }

    pub async fn update_cart_item(
        pool: &Pool<Postgres>,
        item_id: &str,
        req: &UpdateCartItemRequest,
    ) -> Result<ErpCartItem, String> {
        if req.quantity <= 0 {
            return Err("Quantity must be greater than 0".to_string());
        }

        sqlx::query(
            "UPDATE erp_cart_items SET quantity = $1 WHERE id = $2"
        )
        .bind(req.quantity)
        .bind(item_id)
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;

        sqlx::query_as::<_, ErpCartItem>("SELECT * FROM erp_cart_items WHERE id = $1")
            .bind(item_id)
            .fetch_one(pool)
            .await
            .map_err(|e| e.to_string())
    }

    pub async fn remove_cart_item(pool: &Pool<Postgres>, item_id: &str) -> Result<(), String> {
        sqlx::query("DELETE FROM erp_cart_items WHERE id = $1")
            .bind(item_id)
            .execute(pool)
            .await
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub async fn checkout_cart(
        pool: &Pool<Postgres>,
        req: &CheckoutCartRequest,
    ) -> Result<ErpOrder, String> {
        // Get cart and items
        let cart = Self::get_cart(pool, &req.cart_id).await?;
        let cart_items = Self::get_cart_items(pool, &req.cart_id).await?;

        if cart_items.is_empty() {
            return Err("Cart is empty".to_string());
        }

        // Build order items from cart items
        let mut order_items = Vec::new();
        for ci in &cart_items {
            // Fetch product info for snapshot
            let product: Option<(String, String, i64, String)> = sqlx::query_as(
                "SELECT id, slug, base_price, kind FROM erp_products WHERE id = $1"
            )
            .bind(&ci.product_id)
            .fetch_optional(pool)
            .await
            .map_err(|e| e.to_string())?;

            let (product_name, product_sku, unit_price, kind) = match product {
                Some((name, _slug, price, k)) => (name, None, price, k),
                None => (ci.product_id.clone(), None, 0, "goods".to_string()),
            };

            order_items.push(CreateOrderItemRequest {
                product_id: ci.product_id.clone(),
                variant_id: ci.variant_id.clone(),
                kind: Some(kind),
                name_en: product_name,
                name_ur: None,
                sku: product_sku,
                image_url: None,
                variant_label: None,
                unit_price,
                unit_cost: 0,
                quantity: ci.quantity,
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

        // Create order
        let order_req = CreateOrderRequest {
            user_id: cart.user_id.clone(),
            contact_id: cart.contact_id.clone(),
            guest_name: cart.guest_name.clone(),
            guest_email: cart.guest_email.clone(),
            guest_phone: cart.guest_phone.clone(),
            source_cart_id: Some(req.cart_id.clone()),
            shipping_address: req.shipping_address.clone(),
            billing_address: req.billing_address.clone(),
            currency: req.currency.clone(),
            shipping_zone: None,
            customer_note: req.customer_note.clone(),
            items: order_items,
        };

        Self::create_order(pool, &order_req).await
    }

    // ============================================================
    // Shipments
    // ============================================================

    pub async fn list_shipments(
        pool: &Pool<Postgres>,
        order_id: Option<&str>,
        status: Option<&str>,
    ) -> Result<Vec<ErpShipment>, String> {
        let mut query = String::from("SELECT * FROM erp_shipments WHERE 1=1");
        let mut binds: Vec<String> = Vec::new();
        let mut param = 1;

        if let Some(oid) = order_id {
            query.push_str(&format!(" AND order_id = ${}", param));
            binds.push(oid.to_string());
            param += 1;
        }

        if let Some(s) = status {
            query.push_str(&format!(" AND status = ${}", param));
            binds.push(s.to_string());
            param += 1;
        }

        query.push_str(" ORDER BY created_at DESC");

        let shipments = sqlx::query_as::<_, ErpShipment>(&query)
            .fetch_all(pool)
            .await
            .map_err(|e| e.to_string())?;
        Ok(shipments)
    }

    pub async fn get_shipment(pool: &Pool<Postgres>, id: &str) -> Result<ErpShipment, String> {
        let shipment = sqlx::query_as::<_, ErpShipment>(
            "SELECT * FROM erp_shipments WHERE id = $1"
        )
        .fetch_one(pool)
        .await
        .map_err(|e| e.to_string())?;
        Ok(shipment)
    }

    pub async fn create_shipment(
        pool: &Pool<Postgres>,
        req: &CreateShipmentRequest,
    ) -> Result<ErpShipment, String> {
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
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;

        Self::get_shipment(pool, &id).await
    }

    pub async fn update_shipment_status(
        pool: &Pool<Postgres>,
        shipment_id: &str,
        req: &UpdateShipmentStatusRequest,
    ) -> Result<ErpShipment, String> {
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
            .execute(pool)
            .await
            .map_err(|e| e.to_string())?;

        Self::get_shipment(pool, shipment_id).await
    }

    pub async fn get_shipment_events(pool: &Pool<Postgres>, shipment_id: &str) -> Result<Vec<ErpShipmentEvent>, String> {
        let events = sqlx::query_as::<_, ErpShipmentEvent>(
            "SELECT * FROM erp_shipment_events WHERE shipment_id = $1 ORDER BY timestamp DESC"
        )
        .fetch_all(pool)
        .await
        .map_err(|e| e.to_string())?;
        Ok(events)
    }

    pub async fn create_shipment_event(
        pool: &Pool<Postgres>,
        req: &CreateShipmentEventRequest,
    ) -> Result<ErpShipmentEvent, String> {
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
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;

        sqlx::query_as::<_, ErpShipmentEvent>("SELECT * FROM erp_shipment_events WHERE id = $1")
            .bind(&id)
            .fetch_one(pool)
            .await
            .map_err(|e| e.to_string())
    }

    // ============================================================
    // Back in Stock Requests
    // ============================================================

    pub async fn list_back_in_stock_requests(
        pool: &Pool<Postgres>,
        product_id: Option<&str>,
        status: Option<&str>,
    ) -> Result<Vec<ErpBackInStockRequest>, String> {
        let mut query = String::from("SELECT * FROM erp_back_in_stock_requests WHERE 1=1");
        let mut binds: Vec<String> = Vec::new();
        let mut param = 1;

        if let Some(pid) = product_id {
            query.push_str(&format!(" AND product_id = ${}", param));
            binds.push(pid.to_string());
            param += 1;
        }

        if let Some(s) = status {
            query.push_str(&format!(" AND status = ${}", param));
            binds.push(s.to_string());
            param += 1;
        }

        query.push_str(" ORDER BY created_at DESC");

        let requests = sqlx::query_as::<_, ErpBackInStockRequest>(&query)
            .fetch_all(pool)
            .await
            .map_err(|e| e.to_string())?;
        Ok(requests)
    }

    pub async fn create_back_in_stock_request(
        pool: &Pool<Postgres>,
        req: &CreateBackInStockRequestRequest,
    ) -> Result<ErpBackInStockRequest, String> {
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
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;

        sqlx::query_as::<_, ErpBackInStockRequest>(
            "SELECT * FROM erp_back_in_stock_requests WHERE id = $1"
        )
        .bind(&id)
        .fetch_one(pool)
        .await
        .map_err(|e| e.to_string())
    }

    pub async fn notify_back_in_stock(
        pool: &Pool<Postgres>,
        product_id: &str,
    ) -> Result<u64, String> {
        let now = Utc::now().to_rfc3339();

        let result = sqlx::query(
            "UPDATE erp_back_in_stock_requests SET status = 'notified', notified_at = $1 WHERE product_id = $2 AND status = 'pending'"
        )
        .bind(&now)
        .bind(product_id)
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;

        Ok(result.rows_affected())
    }

    // ============================================================
    // Order Statistics
    // ============================================================

    pub async fn get_order_stats(pool: &Pool<Postgres>) -> Result<serde_json::Value, String> {
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
        .fetch_one(pool)
        .await
        .map_err(|e| e.to_string())?;

        Ok(serde_json::json!({
            "total": stats.0,
            "pending": stats.1,
            "confirmed": stats.2,
            "delivered": stats.3
        }))
    }
}
