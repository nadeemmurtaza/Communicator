-- Analytics: Daily Stats and Events Tracking

-- ---------------------------------------------------------------------------
-- Daily Analytics Stats
-- ---------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS analytics_daily_stats (
    id TEXT PRIMARY KEY,
    brand_id TEXT NOT NULL,
    date TEXT NOT NULL,
    orders_count BOOLEAN DEFAULT false,
    revenue REAL DEFAULT 0,
    avg_order_value REAL DEFAULT 0,
    unique_customers BOOLEAN DEFAULT false,
    products_sold BOOLEAN DEFAULT false,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    FOREIGN KEY (brand_id) REFERENCES brands(id) ON DELETE CASCADE,
    UNIQUE(brand_id, date)
);

CREATE INDEX IF NOT EXISTS idx_analytics_daily_brand ON analytics_daily_stats(brand_id);
CREATE INDEX IF NOT EXISTS idx_analytics_daily_date ON analytics_daily_stats(date);

-- ---------------------------------------------------------------------------
-- Analytics Events
-- ---------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS analytics_events (
    id TEXT PRIMARY KEY,
    brand_id TEXT NOT NULL,
    event_type TEXT NOT NULL,
    entity_type TEXT,
    entity_id TEXT,
    properties TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    FOREIGN KEY (brand_id) REFERENCES brands(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_analytics_events_brand ON analytics_events(brand_id);
CREATE INDEX IF NOT EXISTS idx_analytics_events_type ON analytics_events(event_type);
CREATE INDEX IF NOT EXISTS idx_analytics_events_created ON analytics_events(created_at);
