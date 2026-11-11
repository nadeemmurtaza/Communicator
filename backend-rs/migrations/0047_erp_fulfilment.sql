-- Phase 39: ERP Fulfilment Module
-- Returns/RMA, shipping zones, methods, rates, manifests, shipment tracking
-- Enables solo sale channel: customers can return items and get refunds

-- Shipping Zones
CREATE TABLE IF NOT EXISTS erp_shipping_zones (
    id TEXT PRIMARY KEY DEFAULT (gen_random_uuid()::TEXT),
    key TEXT NOT NULL UNIQUE,
    sort_order INTEGER NOT NULL DEFAULT 0,
    cities TEXT NOT NULL DEFAULT '[]', -- JSON array of city names
    is_active BOOLEAN NOT NULL DEFAULT true,
    is_catch_all BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_erp_sz_active ON erp_shipping_zones(is_active);

-- Shipping Zone Translations
CREATE TABLE IF NOT EXISTS erp_shipping_zone_translations (
    id TEXT PRIMARY KEY DEFAULT (gen_random_uuid()::TEXT),
    zone_id TEXT NOT NULL,
    locale TEXT NOT NULL,
    label TEXT NOT NULL,
    FOREIGN KEY (zone_id) REFERENCES erp_shipping_zones(id) ON DELETE CASCADE,
    UNIQUE(zone_id, locale)
);

-- Shipping Methods
CREATE TABLE IF NOT EXISTS erp_shipping_methods (
    id TEXT PRIMARY KEY DEFAULT (gen_random_uuid()::TEXT),
    key TEXT NOT NULL UNIQUE,
    courier_key TEXT,
    eta_min_days INTEGER NOT NULL DEFAULT 2,
    eta_max_days INTEGER NOT NULL DEFAULT 5,
    is_active BOOLEAN NOT NULL DEFAULT true,
    sort_order INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_erp_sm_active ON erp_shipping_methods(is_active);

-- Shipping Method Translations
CREATE TABLE IF NOT EXISTS erp_shipping_method_translations (
    id TEXT PRIMARY KEY DEFAULT (gen_random_uuid()::TEXT),
    method_id TEXT NOT NULL,
    locale TEXT NOT NULL,
    label TEXT NOT NULL,
    FOREIGN KEY (method_id) REFERENCES erp_shipping_methods(id) ON DELETE CASCADE,
    UNIQUE(method_id, locale)
);

-- Shipping Rates
CREATE TABLE IF NOT EXISTS erp_shipping_rates (
    id TEXT PRIMARY KEY DEFAULT (gen_random_uuid()::TEXT),
    zone_id TEXT NOT NULL,
    method_id TEXT NOT NULL,
    rate_type TEXT NOT NULL DEFAULT 'FLAT', -- FLAT, BY_WEIGHT, BY_PRICE, BY_ITEM
    fee INTEGER NOT NULL, -- paisa
    base_fee INTEGER NOT NULL DEFAULT 0,
    included_units INTEGER NOT NULL DEFAULT 0,
    free_above INTEGER, -- paisa, order subtotal above which free
    min_value INTEGER,
    max_value INTEGER,
    is_active BOOLEAN NOT NULL DEFAULT true,
    FOREIGN KEY (zone_id) REFERENCES erp_shipping_zones(id) ON DELETE CASCADE,
    FOREIGN KEY (method_id) REFERENCES erp_shipping_methods(id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_erp_sr_zone_method ON erp_shipping_rates(zone_id, method_id);

-- Shipping Classes (per-product handling fees)
CREATE TABLE IF NOT EXISTS erp_shipping_classes (
    id TEXT PRIMARY KEY DEFAULT (gen_random_uuid()::TEXT),
    key TEXT NOT NULL UNIQUE,
    handling_fee INTEGER NOT NULL DEFAULT 0, -- paisa
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Shipping Class Translations
CREATE TABLE IF NOT EXISTS erp_shipping_class_translations (
    id TEXT PRIMARY KEY DEFAULT (gen_random_uuid()::TEXT),
    class_id TEXT NOT NULL,
    locale TEXT NOT NULL,
    label TEXT NOT NULL,
    FOREIGN KEY (class_id) REFERENCES erp_shipping_classes(id) ON DELETE CASCADE,
    UNIQUE(class_id, locale)
);

-- Manifests (day's pickup grouped by courier)
CREATE TABLE IF NOT EXISTS erp_manifests (
    id TEXT PRIMARY KEY DEFAULT (gen_random_uuid()::TEXT),
    number TEXT NOT NULL UNIQUE, -- "MFT-2026-0001"
    courier_key TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'OPEN', -- OPEN, DISPATCHED
    dispatched_at TIMESTAMPTZ,
    created_by_id TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_erp_mf_courier_status ON erp_manifests(courier_key, status);

-- Manifest Number Counter
CREATE TABLE IF NOT EXISTS erp_manifest_number_counter (
    year INTEGER PRIMARY KEY,
    counter INTEGER NOT NULL DEFAULT 0
);

-- Return Reason Codes
CREATE TABLE IF NOT EXISTS erp_return_reason_codes (
    id TEXT PRIMARY KEY DEFAULT (gen_random_uuid()::TEXT),
    key TEXT NOT NULL UNIQUE, -- "too_small", "damaged", "not_as_described"
    is_active BOOLEAN NOT NULL DEFAULT true,
    sort_order INTEGER NOT NULL DEFAULT 0
);

-- Return Reason Translations
CREATE TABLE IF NOT EXISTS erp_return_reason_translations (
    id TEXT PRIMARY KEY DEFAULT (gen_random_uuid()::TEXT),
    reason_id TEXT NOT NULL,
    locale TEXT NOT NULL,
    label TEXT NOT NULL,
    FOREIGN KEY (reason_id) REFERENCES erp_return_reason_codes(id) ON DELETE CASCADE,
    UNIQUE(reason_id, locale)
);

-- Return Requests (RMA)
CREATE TABLE IF NOT EXISTS erp_return_requests (
    id TEXT PRIMARY KEY DEFAULT (gen_random_uuid()::TEXT),
    number TEXT NOT NULL UNIQUE, -- "RMA-2026-0001"
    order_id TEXT NOT NULL,
    user_id TEXT,
    status TEXT NOT NULL DEFAULT 'REQUESTED', -- REQUESTED, APPROVED, REJECTED, IN_TRANSIT, RECEIVED, INSPECTED, REFUNDED, CLOSED
    resolution TEXT, -- REFUND, EXCHANGE, STORE_CREDIT, REPAIR, REJECTED
    customer_note TEXT,
    staff_note TEXT,
    photo_urls TEXT NOT NULL DEFAULT '[]', -- JSON array of URLs
    requested_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    approved_at TIMESTAMPTZ,
    received_at TIMESTAMPTZ,
    inspected_at TIMESTAMPTZ,
    refunded_at TIMESTAMPTZ,
    refund_amount INTEGER, -- paisa
    restocking_fee_minor INTEGER, -- paisa
    FOREIGN KEY (order_id) REFERENCES erp_orders(id) ON DELETE RESTRICT
);
CREATE INDEX IF NOT EXISTS idx_erp_rr_status ON erp_return_requests(status, requested_at);
CREATE INDEX IF NOT EXISTS idx_erp_rr_order ON erp_return_requests(order_id);

-- Return Request Items
CREATE TABLE IF NOT EXISTS erp_return_request_items (
    id TEXT PRIMARY KEY DEFAULT (gen_random_uuid()::TEXT),
    return_id TEXT NOT NULL,
    order_item_id TEXT NOT NULL,
    reason_id TEXT NOT NULL,
    quantity INTEGER NOT NULL,
    restockable BOOLEAN, -- NULL until inspected
    restocked BOOLEAN NOT NULL DEFAULT false,
    FOREIGN KEY (return_id) REFERENCES erp_return_requests(id) ON DELETE CASCADE,
    FOREIGN KEY (reason_id) REFERENCES erp_return_reason_codes(id)
);
CREATE INDEX IF NOT EXISTS idx_erp_rri_return ON erp_return_request_items(return_id);

-- Return Request Number Counter
CREATE TABLE IF NOT EXISTS erp_return_request_number_counter (
    year INTEGER PRIMARY KEY,
    counter INTEGER NOT NULL DEFAULT 0
);

-- Seed default return reason codes
INSERT INTO erp_return_reason_codes (id, key, sort_order) VALUES
    (gen_random_uuid()::TEXT, 'too_small', 1),
    (gen_random_uuid()::TEXT, 'too_large', 2),
    (gen_random_uuid()::TEXT, 'damaged', 3),
    (gen_random_uuid()::TEXT, 'not_as_described', 4),
    (gen_random_uuid()::TEXT, 'wrong_item', 5),
    (gen_random_uuid()::TEXT, 'changed_mind', 6)
ON CONFLICT (key) DO NOTHING;
