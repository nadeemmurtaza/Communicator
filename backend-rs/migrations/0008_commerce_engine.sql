-- Commerce Engine: Product Catalog, Shopping Cart, Orders, Payments, and Sync

-- ---------------------------------------------------------------------------
-- Product Categories
-- ---------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS categories (
    id TEXT PRIMARY KEY,
    brand_id TEXT NOT NULL,
    name TEXT NOT NULL,
    parent_id TEXT,
    sort_order INTEGER DEFAULT 0,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    FOREIGN KEY (brand_id) REFERENCES brands(id) ON DELETE CASCADE,
    FOREIGN KEY (parent_id) REFERENCES categories(id) ON DELETE SET NULL
);

CREATE INDEX IF NOT EXISTS idx_categories_brand ON categories(brand_id);
CREATE INDEX IF NOT EXISTS idx_categories_parent ON categories(parent_id);
CREATE INDEX IF NOT EXISTS idx_categories_active ON categories(is_active);

-- ---------------------------------------------------------------------------
-- Product Catalog
-- ---------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS products (
    id TEXT PRIMARY KEY,
    brand_id TEXT NOT NULL,
    category_id TEXT,
    name TEXT NOT NULL,
    description TEXT,
    sku TEXT,
    price REAL NOT NULL DEFAULT 0,
    compare_at_price REAL,
    currency TEXT DEFAULT 'USD',
    images TEXT,
    variants TEXT,
    inventory_count INTEGER DEFAULT 0,
    track_inventory BOOLEAN DEFAULT false,
    visibility TEXT,
    metadata TEXT,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    FOREIGN KEY (brand_id) REFERENCES brands(id) ON DELETE CASCADE,
    FOREIGN KEY (category_id) REFERENCES categories(id) ON DELETE SET NULL
);

CREATE INDEX IF NOT EXISTS idx_products_brand ON products(brand_id);
CREATE INDEX IF NOT EXISTS idx_products_category ON products(category_id);
CREATE INDEX IF NOT EXISTS idx_products_sku ON products(sku);
CREATE INDEX IF NOT EXISTS idx_products_active ON products(is_active);

-- ---------------------------------------------------------------------------
-- Product Channel Visibility
-- ---------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS product_channel_visibility (
    id TEXT PRIMARY KEY,
    product_id TEXT NOT NULL,
    channel_type TEXT NOT NULL,
    is_visible BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    FOREIGN KEY (product_id) REFERENCES products(id) ON DELETE CASCADE,
    UNIQUE(product_id, channel_type)
);

CREATE INDEX IF NOT EXISTS idx_product_channel_product ON product_channel_visibility(product_id);
CREATE INDEX IF NOT EXISTS idx_product_channel_type ON product_channel_visibility(channel_type);

-- ---------------------------------------------------------------------------
-- Shopping Carts
-- ---------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS carts (
    id TEXT PRIMARY KEY,
    customer_id TEXT NOT NULL,
    brand_id TEXT NOT NULL,
    channel_type TEXT,
    status TEXT DEFAULT 'active',
    metadata TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    FOREIGN KEY (customer_id) REFERENCES customers(id) ON DELETE CASCADE,
    FOREIGN KEY (brand_id) REFERENCES brands(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_carts_customer ON carts(customer_id);
CREATE INDEX IF NOT EXISTS idx_carts_brand ON carts(brand_id);
CREATE INDEX IF NOT EXISTS idx_carts_status ON carts(status);

-- ---------------------------------------------------------------------------
-- Cart Items
-- ---------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS cart_items (
    id TEXT PRIMARY KEY,
    cart_id TEXT NOT NULL,
    product_id TEXT NOT NULL,
    variant_id TEXT,
    quantity INTEGER NOT NULL DEFAULT 1,
    unit_price REAL NOT NULL,
    metadata TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    FOREIGN KEY (cart_id) REFERENCES carts(id) ON DELETE CASCADE,
    FOREIGN KEY (product_id) REFERENCES products(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_cart_items_cart ON cart_items(cart_id);
CREATE INDEX IF NOT EXISTS idx_cart_items_product ON cart_items(product_id);

-- ---------------------------------------------------------------------------
-- Commerce Orders
-- ---------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS commerce_orders (
    id TEXT PRIMARY KEY,
    brand_id TEXT NOT NULL,
    customer_id TEXT NOT NULL,
    order_number TEXT NOT NULL UNIQUE,
    channel_type TEXT,
    status TEXT DEFAULT 'draft',
    total_amount REAL NOT NULL DEFAULT 0,
    subtotal REAL DEFAULT 0,
    tax REAL DEFAULT 0,
    shipping REAL DEFAULT 0,
    currency TEXT DEFAULT 'USD',
    items_snapshot TEXT,
    shipping_address TEXT,
    payment_status TEXT DEFAULT 'pending',
    payment_method TEXT,
    metadata TEXT,
    source TEXT DEFAULT 'internal',
    external_order_id TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    FOREIGN KEY (brand_id) REFERENCES brands(id) ON DELETE CASCADE,
    FOREIGN KEY (customer_id) REFERENCES customers(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_commerce_orders_brand ON commerce_orders(brand_id);
CREATE INDEX IF NOT EXISTS idx_commerce_orders_customer ON commerce_orders(customer_id);
CREATE INDEX IF NOT EXISTS idx_commerce_orders_status ON commerce_orders(status);
CREATE INDEX IF NOT EXISTS idx_commerce_orders_channel ON commerce_orders(channel_type);
CREATE INDEX IF NOT EXISTS idx_commerce_orders_number ON commerce_orders(order_number);

-- ---------------------------------------------------------------------------
-- Commerce Order Items
-- ---------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS commerce_order_items (
    id TEXT PRIMARY KEY,
    order_id TEXT NOT NULL,
    product_id TEXT NOT NULL,
    variant_id TEXT,
    product_name TEXT NOT NULL,
    sku TEXT,
    quantity INTEGER NOT NULL,
    unit_price REAL NOT NULL,
    total_price REAL NOT NULL,
    metadata TEXT,
    FOREIGN KEY (order_id) REFERENCES commerce_orders(id) ON DELETE CASCADE,
    FOREIGN KEY (product_id) REFERENCES products(id) ON DELETE SET NULL
);

CREATE INDEX IF NOT EXISTS idx_order_items_order ON commerce_order_items(order_id);
CREATE INDEX IF NOT EXISTS idx_order_items_product ON commerce_order_items(product_id);

-- ---------------------------------------------------------------------------
-- Commerce Payments
-- ---------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS commerce_payments (
    id TEXT PRIMARY KEY,
    order_id TEXT NOT NULL,
    amount REAL NOT NULL,
    currency TEXT DEFAULT 'USD',
    method TEXT NOT NULL,
    status TEXT DEFAULT 'pending',
    transaction_id TEXT,
    metadata TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    FOREIGN KEY (order_id) REFERENCES commerce_orders(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_commerce_payments_order ON commerce_payments(order_id);
CREATE INDEX IF NOT EXISTS idx_commerce_payments_status ON commerce_payments(status);
CREATE INDEX IF NOT EXISTS idx_commerce_payments_method ON commerce_payments(method);

-- ---------------------------------------------------------------------------
-- Sync Connections (external platform connections)
-- ---------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS sync_connections (
    id TEXT PRIMARY KEY,
    brand_id TEXT NOT NULL,
    platform TEXT NOT NULL,
    credentials TEXT,
    sync_status TEXT DEFAULT 'disconnected',
    last_sync_at TEXT,
    metadata TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    FOREIGN KEY (brand_id) REFERENCES brands(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_sync_connections_brand ON sync_connections(brand_id);
CREATE INDEX IF NOT EXISTS idx_sync_connections_platform ON sync_connections(platform);
CREATE INDEX IF NOT EXISTS idx_sync_connections_status ON sync_connections(sync_status);

-- ---------------------------------------------------------------------------
-- Sync Logs (audit trail)
-- ---------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS sync_logs (
    id TEXT PRIMARY KEY,
    connection_id TEXT NOT NULL,
    direction TEXT NOT NULL,
    entity_type TEXT NOT NULL,
    entity_id TEXT,
    status TEXT NOT NULL,
    error TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    FOREIGN KEY (connection_id) REFERENCES sync_connections(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_sync_logs_connection ON sync_logs(connection_id);
CREATE INDEX IF NOT EXISTS idx_sync_logs_direction ON sync_logs(direction);
CREATE INDEX IF NOT EXISTS idx_sync_logs_status ON sync_logs(status);
CREATE INDEX IF NOT EXISTS idx_sync_logs_created ON sync_logs(created_at);

-- ---------------------------------------------------------------------------
-- Seed default categories
-- ---------------------------------------------------------------------------
INSERT INTO categories (id, brand_id, name, sort_order) VALUES
('cat_default_all', '', 'All Products', 0),
('cat_default_featured', '', 'Featured', 1),
('cat_default_new', '', 'New Arrivals', 2);
