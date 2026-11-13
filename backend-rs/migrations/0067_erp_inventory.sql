-- ERP Inventory Module: Port of inventory.prisma (535 lines)
-- Maps the ERP inventory system to PostgreSQL tables for the Rust backend
-- Includes: Stock movements, locations, variant-location stock, variances,
-- spot checks, forecasts, markdown suggestions, lots/batches, serial numbers, stock counts

-- Stock Movement Reasons enum values stored as text
-- PURCHASE_RECEIPT, SALE, CANCELLATION, RETURN, ADJUSTMENT, DAMAGE, COUNT, WRITE_OFF,
-- PRODUCTION_CONSUMPTION, PRODUCTION_RECEIPT, POS_SALE, POS_OFFLINE_VARIANCE, TRANSFER_OUT, TRANSFER_IN

-- Locations (warehouses/stores)
CREATE TABLE IF NOT EXISTS erp_locations (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    code TEXT NOT NULL UNIQUE,
    type TEXT NOT NULL DEFAULT 'STORE',
    retail_brand_id TEXT,
    origin_node_id TEXT,
    is_default BOOLEAN NOT NULL DEFAULT false,
    is_active BOOLEAN NOT NULL DEFAULT true,
    address JSONB,
    created_at TEXT NOT NULL DEFAULT (NOW())
);
CREATE INDEX IF NOT EXISTS idx_erp_locations_active ON erp_locations(is_active);
CREATE INDEX IF NOT EXISTS idx_erp_locations_brand ON erp_locations(retail_brand_id);

-- Variant Location Stock (per variant, per location)
CREATE TABLE IF NOT EXISTS erp_variant_location_stock (
    variant_id TEXT NOT NULL,
    location_id TEXT NOT NULL,
    quantity INTEGER NOT NULL DEFAULT 0,
    updated_at TEXT NOT NULL DEFAULT (NOW()),
    PRIMARY KEY (variant_id, location_id)
);
CREATE INDEX IF NOT EXISTS idx_erp_vls_location ON erp_variant_location_stock(location_id);

-- Stock Movements (the ledger)
CREATE TABLE IF NOT EXISTS erp_stock_movements (
    id TEXT PRIMARY KEY,
    variant_id TEXT NOT NULL,
    delta INTEGER NOT NULL,
    resulting_stock INTEGER NOT NULL,
    reason TEXT NOT NULL,
    location_id TEXT,
    ref_type TEXT,
    ref_id TEXT,
    unit_cost INTEGER,
    note TEXT,
    actor_id TEXT,
    created_at TEXT NOT NULL DEFAULT (NOW())
);
CREATE INDEX IF NOT EXISTS idx_erp_sm_variant_created ON erp_stock_movements(variant_id, created_at);
CREATE INDEX IF NOT EXISTS idx_erp_sm_reason_created ON erp_stock_movements(reason, created_at);
CREATE INDEX IF NOT EXISTS idx_erp_sm_ref ON erp_stock_movements(ref_type, ref_id);
CREATE INDEX IF NOT EXISTS idx_erp_sm_location ON erp_stock_movements(location_id);

-- Stock Variances (oversell tracking)
CREATE TABLE IF NOT EXISTS erp_stock_variances (
    id TEXT PRIMARY KEY,
    variant_id TEXT NOT NULL,
    location_id TEXT NOT NULL,
    overshoot_qty INTEGER NOT NULL,
    stock_movement_id TEXT NOT NULL UNIQUE,
    ref_type TEXT,
    ref_id TEXT,
    status TEXT NOT NULL DEFAULT 'OPEN',
    resolved_by_user_id TEXT,
    resolved_at TEXT,
    resolution_note TEXT,
    pos_shift_id TEXT,
    serialized_unit_id TEXT,
    created_at TEXT NOT NULL DEFAULT (NOW())
);
CREATE INDEX IF NOT EXISTS idx_erp_sv_status ON erp_stock_variances(status);
CREATE INDEX IF NOT EXISTS idx_erp_sv_variant ON erp_stock_variances(variant_id);
CREATE INDEX IF NOT EXISTS idx_erp_sv_location ON erp_stock_variances(location_id);

-- Spot Check Policies
CREATE TABLE IF NOT EXISTS erp_spot_check_policies (
    id TEXT PRIMARY KEY,
    location_id TEXT,
    min_skus_per_check INTEGER NOT NULL DEFAULT 5,
    frequency_days INTEGER NOT NULL DEFAULT 3,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TEXT NOT NULL DEFAULT (NOW()),
    updated_at TEXT NOT NULL DEFAULT (NOW())
);
CREATE INDEX IF NOT EXISTS idx_erp_scp_location_active ON erp_spot_check_policies(location_id, is_active);

-- Spot Checks
CREATE TABLE IF NOT EXISTS erp_spot_checks (
    id TEXT PRIMARY KEY,
    policy_id TEXT NOT NULL,
    location_id TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'PENDING',
    scheduled_for TEXT NOT NULL,
    started_at TEXT,
    completed_at TEXT,
    created_at TEXT NOT NULL DEFAULT (NOW())
);
CREATE INDEX IF NOT EXISTS idx_erp_sc_location_status ON erp_spot_checks(location_id, status);
CREATE INDEX IF NOT EXISTS idx_erp_sc_policy_scheduled ON erp_spot_checks(policy_id, scheduled_for);

-- Spot Check Lines
CREATE TABLE IF NOT EXISTS erp_spot_check_lines (
    id TEXT PRIMARY KEY,
    spot_check_id TEXT NOT NULL,
    variant_id TEXT NOT NULL,
    expected INTEGER NOT NULL,
    counted INTEGER,
    variance INTEGER,
    UNIQUE (spot_check_id, variant_id)
);

-- Variant Sales Stats (cache)
CREATE TABLE IF NOT EXISTS erp_variant_sales_stats (
    variant_id TEXT PRIMARY KEY,
    units_30 INTEGER NOT NULL DEFAULT 0,
    units_90 INTEGER NOT NULL DEFAULT 0,
    last_sold_at TEXT,
    days_of_cover REAL,
    age_days INTEGER,
    is_slow_mover BOOLEAN NOT NULL DEFAULT false,
    is_dead_stock BOOLEAN NOT NULL DEFAULT false,
    last_low_stock_alert_at TEXT,
    computed_at TEXT NOT NULL DEFAULT (NOW())
);

-- Variant Forecasts
CREATE TABLE IF NOT EXISTS erp_variant_forecasts (
    variant_id TEXT PRIMARY KEY,
    forecasted_daily_demand REAL NOT NULL DEFAULT 0,
    seasonal_factor REAL,
    projected_stockout_at TEXT,
    reason_en TEXT,
    computed_at TEXT NOT NULL DEFAULT (NOW()),
    safety_stock_units INTEGER,
    suggested_reorder_point INTEGER,
    service_level_percent INTEGER
);

-- Markdown Suggestions
CREATE TABLE IF NOT EXISTS erp_markdown_suggestions (
    id TEXT PRIMARY KEY,
    variant_id TEXT NOT NULL,
    discount_id TEXT,
    suggested_bps INTEGER NOT NULL,
    estimated_clearance_pct REAL NOT NULL,
    estimated_clearance_days INTEGER NOT NULL,
    margin_before_paisa INTEGER NOT NULL,
    margin_after_paisa INTEGER NOT NULL,
    status TEXT NOT NULL DEFAULT 'PENDING',
    created_at TEXT NOT NULL DEFAULT (NOW()),
    resolved_at TEXT
);
CREATE INDEX IF NOT EXISTS idx_erp_ms_variant_status ON erp_markdown_suggestions(variant_id, status);
CREATE INDEX IF NOT EXISTS idx_erp_ms_status_created ON erp_markdown_suggestions(status, created_at);

-- Lots (batch tracking)
CREATE TABLE IF NOT EXISTS erp_lots (
    id TEXT PRIMARY KEY,
    product_variant_id TEXT NOT NULL,
    lot_code TEXT NOT NULL,
    expires_on TEXT,
    received_on TEXT NOT NULL DEFAULT (NOW()),
    source_receipt_id TEXT,
    created_at TEXT NOT NULL DEFAULT (NOW()),
    UNIQUE (product_variant_id, lot_code)
);
CREATE INDEX IF NOT EXISTS idx_erp_lots_expires ON erp_lots(expires_on);

-- Lot Stock (per lot, per location)
CREATE TABLE IF NOT EXISTS erp_lot_stock (
    id TEXT PRIMARY KEY,
    lot_id TEXT NOT NULL,
    location_id TEXT NOT NULL,
    quantity INTEGER NOT NULL DEFAULT 0,
    UNIQUE (lot_id, location_id)
);
CREATE INDEX IF NOT EXISTS idx_erp_ls_location ON erp_lot_stock(location_id);

-- Lot Allocations (which lot was consumed by which movement)
CREATE TABLE IF NOT EXISTS erp_lot_allocations (
    id TEXT PRIMARY KEY,
    lot_id TEXT NOT NULL,
    stock_movement_id TEXT NOT NULL,
    quantity INTEGER NOT NULL,
    allocated_at TEXT NOT NULL DEFAULT (NOW()),
    UNIQUE (lot_id, stock_movement_id)
);
CREATE INDEX IF NOT EXISTS idx_erp_la_lot ON erp_lot_allocations(lot_id);
CREATE INDEX IF NOT EXISTS idx_erp_la_movement ON erp_lot_allocations(stock_movement_id);

-- Serialized Units (serial number tracking)
CREATE TABLE IF NOT EXISTS erp_serialized_units (
    id TEXT PRIMARY KEY,
    product_variant_id TEXT NOT NULL,
    serial_number TEXT NOT NULL UNIQUE,
    status TEXT NOT NULL DEFAULT 'IN_STOCK',
    lot_id TEXT,
    location_id TEXT,
    order_id TEXT,
    stock_movement_id TEXT,
    created_at TEXT NOT NULL DEFAULT (NOW()),
    updated_at TEXT NOT NULL DEFAULT (NOW())
);
CREATE INDEX IF NOT EXISTS idx_erp_su_variant_status ON erp_serialized_units(product_variant_id, status);
CREATE INDEX IF NOT EXISTS idx_erp_su_order ON erp_serialized_units(order_id);
CREATE INDEX IF NOT EXISTS idx_erp_su_movement ON erp_serialized_units(stock_movement_id);

-- Stock Counts
CREATE TABLE IF NOT EXISTS erp_stock_counts (
    id TEXT PRIMARY KEY,
    count_number TEXT NOT NULL UNIQUE,
    status TEXT NOT NULL DEFAULT 'DRAFT',
    note TEXT,
    started_at TEXT,
    completed_at TEXT,
    cancelled_at TEXT,
    created_by TEXT,
    assigned_to TEXT,
    created_at TEXT NOT NULL DEFAULT (NOW()),
    updated_at TEXT NOT NULL DEFAULT (NOW())
);
CREATE INDEX IF NOT EXISTS idx_erp_sc_status ON erp_stock_counts(status);

-- Stock Count Lines
CREATE TABLE IF NOT EXISTS erp_stock_count_lines (
    id TEXT PRIMARY KEY,
    stock_count_id TEXT NOT NULL,
    variant_id TEXT NOT NULL,
    expected INTEGER,
    counted INTEGER,
    note TEXT,
    created_at TEXT NOT NULL DEFAULT (NOW()),
    updated_at TEXT NOT NULL DEFAULT (NOW()),
    UNIQUE (stock_count_id, variant_id)
);
