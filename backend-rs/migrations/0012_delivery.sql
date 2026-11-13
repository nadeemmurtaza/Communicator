-- Delivery Tracking and Shipping

-- ---------------------------------------------------------------------------
-- Shipping Methods
-- ---------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS shipping_methods (
    id TEXT PRIMARY KEY,
    brand_id TEXT NOT NULL,
    name TEXT NOT NULL,
    type TEXT NOT NULL DEFAULT 'flat_rate',
    rate REAL DEFAULT 0,
    free_shipping_threshold REAL,
    regions TEXT,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    FOREIGN KEY (brand_id) REFERENCES brands(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_shipping_methods_brand ON shipping_methods(brand_id);

-- ---------------------------------------------------------------------------
-- Shipments
-- ---------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS shipments (
    id TEXT PRIMARY KEY,
    order_id TEXT NOT NULL,
    tracking_number TEXT,
    carrier TEXT,
    status TEXT DEFAULT 'pending',
    estimated_delivery TEXT,
    actual_delivery TEXT,
    shipping_label_url TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    FOREIGN KEY (order_id) REFERENCES commerce_orders(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_shipments_order ON shipments(order_id);
CREATE INDEX IF NOT EXISTS idx_shipments_tracking ON shipments(tracking_number);
CREATE INDEX IF NOT EXISTS idx_shipments_status ON shipments(status);

-- ---------------------------------------------------------------------------
-- Delivery Events
-- ---------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS delivery_events (
    id TEXT PRIMARY KEY,
    shipment_id TEXT NOT NULL,
    event_type TEXT NOT NULL,
    location TEXT,
    description TEXT,
    
    timestamp TEXT DEFAULT (NOW()),
    FOREIGN KEY (shipment_id) REFERENCES shipments(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_delivery_events_shipment ON delivery_events(shipment_id);
CREATE INDEX IF NOT EXISTS idx_delivery_events_timestamp ON delivery_events(timestamp);
