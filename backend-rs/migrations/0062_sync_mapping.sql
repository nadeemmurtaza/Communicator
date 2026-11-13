-- Migration 0015: Sync mapping tables for external e-commerce platforms
-- These tables enable bidirectional sync by tracking external IDs

-- Maps our products to external platform product IDs
CREATE TABLE IF NOT EXISTS external_product_mapping (
    id TEXT PRIMARY KEY DEFAULT gen_random_uuid()::TEXT,
    product_id TEXT NOT NULL,
    platform TEXT NOT NULL,
    connection_id TEXT NOT NULL,
    external_id TEXT NOT NULL,
    external_sku TEXT,
    last_synced_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    FOREIGN KEY (product_id) REFERENCES products(id) ON DELETE CASCADE,
    FOREIGN KEY (connection_id) REFERENCES sync_connections(id) ON DELETE CASCADE,
    UNIQUE(product_id, platform, connection_id)
);

-- Maps our orders to external platform order IDs
CREATE TABLE IF NOT EXISTS external_order_mapping (
    id TEXT PRIMARY KEY DEFAULT gen_random_uuid()::TEXT,
    order_id TEXT NOT NULL,
    platform TEXT NOT NULL,
    connection_id TEXT NOT NULL,
    external_id TEXT NOT NULL,
    external_order_number TEXT,
    last_synced_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    FOREIGN KEY (order_id) REFERENCES commerce_orders(id) ON DELETE CASCADE,
    FOREIGN KEY (connection_id) REFERENCES sync_connections(id) ON DELETE CASCADE,
    UNIQUE(order_id, platform, connection_id)
);

-- Sync queue for rate limiting and retry logic
CREATE TABLE IF NOT EXISTS sync_queue (
    id TEXT PRIMARY KEY DEFAULT gen_random_uuid()::TEXT,
    connection_id TEXT NOT NULL,
    entity_type TEXT NOT NULL,
    entity_id TEXT,
    direction TEXT NOT NULL,
    payload TEXT,
    status TEXT DEFAULT 'pending',
    priority INTEGER DEFAULT 0,
    retries INTEGER DEFAULT 0,
    max_retries INTEGER DEFAULT 3,
    error TEXT,
    scheduled_at TEXT,
    started_at TIMESTAMPTZ,
    completed_at TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    FOREIGN KEY (connection_id) REFERENCES sync_connections(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_ext_product_product ON external_product_mapping(product_id);
CREATE INDEX IF NOT EXISTS idx_ext_product_external ON external_product_mapping(external_id);
CREATE INDEX IF NOT EXISTS idx_ext_product_platform ON external_product_mapping(platform);
CREATE INDEX IF NOT EXISTS idx_ext_order_order ON external_order_mapping(order_id);
CREATE INDEX IF NOT EXISTS idx_ext_order_external ON external_order_mapping(external_id);
CREATE INDEX IF NOT EXISTS idx_sync_queue_status ON sync_queue(status);
CREATE INDEX IF NOT EXISTS idx_sync_queue_connection ON sync_queue(connection_id);

-- Webhook events log for real-time sync
CREATE TABLE IF NOT EXISTS webhook_events (
    id TEXT PRIMARY KEY DEFAULT gen_random_uuid()::TEXT,
    platform TEXT NOT NULL,
    event_type TEXT NOT NULL,
    payload TEXT NOT NULL,
    processed BOOLEAN DEFAULT false,
    error TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_webhook_processed ON webhook_events(processed);
