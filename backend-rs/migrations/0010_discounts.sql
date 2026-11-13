-- Discounts, Coupons, and Memberships

-- ---------------------------------------------------------------------------
-- Coupons
-- ---------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS coupons (
    id TEXT PRIMARY KEY,
    brand_id TEXT NOT NULL,
    code TEXT NOT NULL UNIQUE COLLATE NOCASE,
    type TEXT NOT NULL DEFAULT 'percentage',
    value REAL NOT NULL DEFAULT 0,
    min_order_amount REAL,
    max_discount REAL,
    usage_limit INTEGER,
    used_count BOOLEAN DEFAULT false,
    starts_at TEXT,
    ends_at TEXT,
    applicable_products TEXT,
    applicable_categories TEXT,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    FOREIGN KEY (brand_id) REFERENCES brands(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_coupons_brand ON coupons(brand_id);
CREATE INDEX IF NOT EXISTS idx_coupons_code ON coupons(code);
CREATE INDEX IF NOT EXISTS idx_coupons_active ON coupons(is_active);

-- ---------------------------------------------------------------------------
-- Coupon Usage Log
-- ---------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS coupon_usage (
    id TEXT PRIMARY KEY,
    coupon_id TEXT NOT NULL,
    order_id TEXT NOT NULL,
    customer_id TEXT NOT NULL,
    discount_amount REAL NOT NULL,
    
    used_at TEXT DEFAULT (NOW()),
    FOREIGN KEY (coupon_id) REFERENCES coupons(id) ON DELETE CASCADE,
    FOREIGN KEY (order_id) REFERENCES commerce_orders(id) ON DELETE CASCADE,
    FOREIGN KEY (customer_id) REFERENCES customers(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_coupon_usage_coupon ON coupon_usage(coupon_id);
CREATE INDEX IF NOT EXISTS idx_coupon_usage_order ON coupon_usage(order_id);
CREATE INDEX IF NOT EXISTS idx_coupon_usage_customer ON coupon_usage(customer_id);

-- ---------------------------------------------------------------------------
-- Membership Tiers
-- ---------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS membership_tiers (
    id TEXT PRIMARY KEY,
    brand_id TEXT NOT NULL,
    name TEXT NOT NULL,
    discount_percent REAL DEFAULT 0,
    monthly_price REAL,
    benefits TEXT,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    FOREIGN KEY (brand_id) REFERENCES brands(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_membership_tiers_brand ON membership_tiers(brand_id);

-- ---------------------------------------------------------------------------
-- Customer Memberships
-- ---------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS customer_memberships (
    id TEXT PRIMARY KEY,
    customer_id TEXT NOT NULL,
    membership_tier_id TEXT NOT NULL,
    started_at TIMESTAMPTZ DEFAULT (NOW()),
    ends_at TEXT,
    status TEXT DEFAULT 'active',
    created_at TIMESTAMPTZ DEFAULT NOW(),
    FOREIGN KEY (customer_id) REFERENCES customers(id) ON DELETE CASCADE,
    FOREIGN KEY (membership_tier_id) REFERENCES membership_tiers(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_customer_memberships_customer ON customer_memberships(customer_id);
CREATE INDEX IF NOT EXISTS idx_customer_memberships_tier ON customer_memberships(membership_tier_id);
CREATE INDEX IF NOT EXISTS idx_customer_memberships_status ON customer_memberships(status);
