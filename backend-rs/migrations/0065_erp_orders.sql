-- ERP Orders Module: Port of orders.prisma (369 lines)
-- Maps the ERP order system to PostgreSQL tables for the Rust backend
-- Includes: Orders, OrderItems, OrderStatusEvents, Carts, CartItems, Shipments, BackInStockRequests

-- Orders
CREATE TABLE IF NOT EXISTS erp_orders (
    id TEXT PRIMARY KEY,
    number TEXT NOT NULL UNIQUE,
    user_id TEXT,
    contact_id TEXT,
    status TEXT NOT NULL DEFAULT 'PENDING',
    currency TEXT NOT NULL DEFAULT 'PKR',
    channel TEXT NOT NULL DEFAULT 'WEB',
    location_id TEXT,
    terminal_id TEXT,
    shift_id TEXT,
    origin_node_id TEXT,
    guest_name TEXT,
    guest_email TEXT,
    guest_phone TEXT,
    source_cart_id TEXT,
    shipping_address JSONB NOT NULL DEFAULT '{}',
    billing_address JSONB,
    items_subtotal INTEGER NOT NULL DEFAULT 0,
    stitching_subtotal INTEGER NOT NULL DEFAULT 0,
    item_discount_total INTEGER NOT NULL DEFAULT 0,
    order_discount INTEGER NOT NULL DEFAULT 0,
    shipping_fee INTEGER NOT NULL DEFAULT 0,
    cod_fee INTEGER NOT NULL DEFAULT 0,
    tax_amount INTEGER NOT NULL DEFAULT 0,
    grand_total INTEGER NOT NULL DEFAULT 0,
    shipping_zone TEXT,
    customer_note TEXT,
    staff_note TEXT,
    cancel_reason TEXT,
    placed_at TEXT NOT NULL DEFAULT (NOW()),
    confirmed_at TEXT,
    delivered_at TEXT,
    cancelled_at TEXT,
    created_at TEXT NOT NULL DEFAULT (NOW()),
    updated_at TEXT NOT NULL DEFAULT (NOW()),
    review_request_sent_at TEXT,
    cod_risk_score INTEGER,
    cod_risk_action TEXT,
    cod_risk_computed_at TEXT,
    cod_risk_reason_en TEXT,
    source_type TEXT,
    source_ref TEXT,
    UNIQUE(source_type, source_ref)
);
CREATE INDEX IF NOT EXISTS idx_erp_orders_status_placed ON erp_orders(status, placed_at);
CREATE INDEX IF NOT EXISTS idx_erp_orders_user ON erp_orders(user_id);
CREATE INDEX IF NOT EXISTS idx_erp_orders_contact ON erp_orders(contact_id);
CREATE INDEX IF NOT EXISTS idx_erp_orders_guest_phone ON erp_orders(guest_phone);
CREATE INDEX IF NOT EXISTS idx_erp_orders_channel ON erp_orders(channel);
CREATE INDEX IF NOT EXISTS idx_erp_orders_shift ON erp_orders(shift_id);
CREATE INDEX IF NOT EXISTS idx_erp_orders_source_cart ON erp_orders(source_cart_id);
CREATE INDEX IF NOT EXISTS idx_erp_orders_cod_risk ON erp_orders(cod_risk_action, placed_at);

-- Order items (snapshots at purchase time)
CREATE TABLE IF NOT EXISTS erp_order_items (
    id TEXT PRIMARY KEY,
    order_id TEXT NOT NULL REFERENCES erp_orders(id) ON DELETE CASCADE,
    product_id TEXT,
    variant_id TEXT,
    kind TEXT NOT NULL DEFAULT 'goods',
    name_en TEXT NOT NULL,
    name_ur TEXT,
    sku TEXT,
    image_url TEXT,
    variant_label TEXT,
    unit_price INTEGER NOT NULL,
    unit_cost INTEGER NOT NULL DEFAULT 0,
    quantity INTEGER NOT NULL DEFAULT 1,
    stitching_fee_unit INTEGER NOT NULL DEFAULT 0,
    discount_unit INTEGER NOT NULL DEFAULT 0,
    line_total INTEGER NOT NULL,
    tax_amount INTEGER NOT NULL DEFAULT 0,
    tax_rate_bps INTEGER NOT NULL DEFAULT 0,
    stitching_service_id TEXT,
    stitching_service_name TEXT,
    garment_type_key TEXT,
    measurement_snapshot JSONB,
    options_snapshot JSONB,
    stitching_notes TEXT,
    measurement_values JSONB,
    custom_design_text JSONB,
    size_token TEXT,
    replenishment_reminder_sent_at TEXT,
    fulfillment_status TEXT NOT NULL DEFAULT 'READY',
    awaiting_since TEXT,
    created_at TEXT NOT NULL DEFAULT (NOW())
);
CREATE INDEX IF NOT EXISTS idx_erp_order_items_order ON erp_order_items(order_id);
CREATE INDEX IF NOT EXISTS idx_erp_order_items_product ON erp_order_items(product_id);

-- Order status events (audit trail)
CREATE TABLE IF NOT EXISTS erp_order_status_events (
    id TEXT PRIMARY KEY,
    order_id TEXT NOT NULL REFERENCES erp_orders(id) ON DELETE CASCADE,
    from_status TEXT,
    to_status TEXT NOT NULL,
    note TEXT,
    actor_id TEXT,
    created_at TEXT NOT NULL DEFAULT (NOW())
);
CREATE INDEX IF NOT EXISTS idx_erp_order_events_order ON erp_order_status_events(order_id, created_at);

-- Carts
CREATE TABLE IF NOT EXISTS erp_carts (
    id TEXT PRIMARY KEY,
    user_id TEXT,
    contact_id TEXT,
    guest_name TEXT,
    guest_email TEXT,
    guest_phone TEXT,
    status TEXT NOT NULL DEFAULT 'active',
    metadata JSONB NOT NULL DEFAULT '{}',
    created_at TEXT NOT NULL DEFAULT (NOW()),
    updated_at TEXT NOT NULL DEFAULT (NOW())
);
CREATE INDEX IF NOT EXISTS idx_erp_carts_user ON erp_carts(user_id);
CREATE INDEX IF NOT EXISTS idx_erp_carts_status ON erp_carts(status);

-- Cart items
CREATE TABLE IF NOT EXISTS erp_cart_items (
    id TEXT PRIMARY KEY,
    cart_id TEXT NOT NULL REFERENCES erp_carts(id) ON DELETE CASCADE,
    product_id TEXT NOT NULL,
    variant_id TEXT,
    quantity INTEGER NOT NULL DEFAULT 1,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_at TEXT NOT NULL DEFAULT (NOW())
);
CREATE INDEX IF NOT EXISTS idx_erp_cart_items_cart ON erp_cart_items(cart_id);

-- Shipments
CREATE TABLE IF NOT EXISTS erp_shipments (
    id TEXT PRIMARY KEY,
    order_id TEXT NOT NULL REFERENCES erp_orders(id) ON DELETE CASCADE,
    courier TEXT NOT NULL DEFAULT 'manual',
    tracking_number TEXT,
    tracking_url TEXT,
    status TEXT NOT NULL DEFAULT 'PENDING',
    cost INTEGER,
    dispatched_at TEXT,
    delivered_at TEXT,
    created_at TEXT NOT NULL DEFAULT (NOW()),
    updated_at TEXT NOT NULL DEFAULT (NOW()),
    method_id TEXT,
    weight_grams INTEGER,
    package_count INTEGER NOT NULL DEFAULT 1,
    label_url TEXT,
    manifest_id TEXT,
    meta JSONB,
    is_rto BOOLEAN NOT NULL DEFAULT false,
    rto_at TEXT,
    failed_attempts INTEGER NOT NULL DEFAULT 0,
    last_polled_at TEXT
);
CREATE INDEX IF NOT EXISTS idx_erp_shipments_order ON erp_shipments(order_id);
CREATE INDEX IF NOT EXISTS idx_erp_shipments_tracking ON erp_shipments(tracking_number);
CREATE INDEX IF NOT EXISTS idx_erp_shipments_status ON erp_shipments(status);
CREATE INDEX IF NOT EXISTS idx_erp_shipments_manifest ON erp_shipments(manifest_id);

-- Shipment items
CREATE TABLE IF NOT EXISTS erp_shipment_items (
    id TEXT PRIMARY KEY,
    shipment_id TEXT NOT NULL REFERENCES erp_shipments(id) ON DELETE CASCADE,
    order_item_id TEXT NOT NULL REFERENCES erp_order_items(id) ON DELETE CASCADE,
    quantity INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (NOW())
);
CREATE INDEX IF NOT EXISTS idx_erp_shipment_items_shipment ON erp_shipment_items(shipment_id);

-- Shipment events (tracking timeline)
CREATE TABLE IF NOT EXISTS erp_shipment_events (
    id TEXT PRIMARY KEY,
    shipment_id TEXT NOT NULL REFERENCES erp_shipments(id) ON DELETE CASCADE,
    event_type TEXT NOT NULL,
    location TEXT,
    description TEXT,
    timestamp TEXT NOT NULL DEFAULT (NOW())
);
CREATE INDEX IF NOT EXISTS idx_erp_shipment_events_shipment ON erp_shipment_events(shipment_id, timestamp);

-- City COD risk stats (precomputed for COD risk scoring)
CREATE TABLE IF NOT EXISTS erp_city_cod_risk_stats (
    city TEXT PRIMARY KEY,
    total_shipments INTEGER NOT NULL DEFAULT 0,
    rto_shipments INTEGER NOT NULL DEFAULT 0,
    avg_order_value INTEGER NOT NULL DEFAULT 0,
    computed_at TEXT NOT NULL DEFAULT (NOW())
);

-- Order number counter (sequential per-year order numbers)
CREATE TABLE IF NOT EXISTS erp_order_number_counters (
    year INTEGER PRIMARY KEY,
    counter INTEGER NOT NULL DEFAULT 0
);
-- Seed current year
INSERT INTO erp_order_number_counters (year, counter) VALUES (2026, 0)
ON CONFLICT (year) DO NOTHING;

-- Back in stock requests
CREATE TABLE IF NOT EXISTS erp_back_in_stock_requests (
    id TEXT PRIMARY KEY,
    product_id TEXT NOT NULL,
    variant_id TEXT,
    customer_email TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending',
    notified_at TEXT,
    created_at TEXT NOT NULL DEFAULT (NOW())
);
CREATE INDEX IF NOT EXISTS idx_erp_back_in_stock_product ON erp_back_in_stock_requests(product_id);
CREATE INDEX IF NOT EXISTS idx_erp_back_in_stock_status ON erp_back_in_stock_requests(status);
