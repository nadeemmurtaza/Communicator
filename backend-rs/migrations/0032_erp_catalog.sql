-- ERP Catalog Module: Port of catalog.prisma (790 lines)
-- Maps the ERP product catalog to PostgreSQL tables for the Rust backend

-- Product kinds and statuses
CREATE TABLE IF NOT EXISTS erp_product_kinds (
    id TEXT PRIMARY KEY DEFAULT 'goods',
    name TEXT NOT NULL UNIQUE
);
INSERT INTO erp_product_kinds (id, name) VALUES
    ('goods', 'Physical Goods'),
    ('service', 'Service'),
    ('digital', 'Digital'),
    ('stitched', 'Stitched (Ready to Wear)'),
    ('unstitched', 'Unstitched (Fabric)'),
    ('stitching_service', 'Stitching Service')
ON CONFLICT (id) DO NOTHING;

-- Categories
CREATE TABLE IF NOT EXISTS erp_categories (
    id TEXT PRIMARY KEY,
    slug TEXT NOT NULL UNIQUE,
    code TEXT,
    parent_id TEXT REFERENCES erp_categories(id) ON DELETE SET NULL,
    image_url TEXT,
    sort_order INTEGER NOT NULL DEFAULT 0,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX IF NOT EXISTS idx_erp_categories_parent ON erp_categories(parent_id);
CREATE INDEX IF NOT EXISTS idx_erp_categories_active ON erp_categories(is_active);

-- Category translations
CREATE TABLE IF NOT EXISTS erp_category_translations (
    id TEXT PRIMARY KEY,
    category_id TEXT NOT NULL REFERENCES erp_categories(id) ON DELETE CASCADE,
    locale TEXT NOT NULL,
    name TEXT NOT NULL,
    description TEXT,
    UNIQUE(category_id, locale)
);

-- Brands
CREATE TABLE IF NOT EXISTS erp_brands (
    id TEXT PRIMARY KEY,
    slug TEXT NOT NULL UNIQUE,
    logo_url TEXT,
    sort_order INTEGER NOT NULL DEFAULT 0,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX IF NOT EXISTS idx_erp_brands_active ON erp_brands(is_active);

-- Brand translations
CREATE TABLE IF NOT EXISTS erp_brand_translations (
    id TEXT PRIMARY KEY,
    brand_id TEXT NOT NULL REFERENCES erp_brands(id) ON DELETE CASCADE,
    locale TEXT NOT NULL,
    name TEXT NOT NULL,
    description TEXT,
    UNIQUE(brand_id, locale)
);

-- Products
CREATE TABLE IF NOT EXISTS erp_products (
    id TEXT PRIMARY KEY,
    slug TEXT NOT NULL UNIQUE,
    kind TEXT NOT NULL DEFAULT 'goods',
    status TEXT NOT NULL DEFAULT 'draft',
    category_id TEXT REFERENCES erp_categories(id) ON DELETE SET NULL,
    base_price INTEGER NOT NULL,
    compare_at_price INTEGER,
    brand_id TEXT REFERENCES erp_brands(id) ON DELETE SET NULL,
    fabric TEXT,
    season TEXT,
    tags TEXT NOT NULL DEFAULT '[]',
    code TEXT,
    noindex BOOLEAN NOT NULL DEFAULT false,
    canonical_override TEXT,
    piece_count INTEGER,
    fabric_meters REAL,
    stitching_available BOOLEAN NOT NULL DEFAULT false,
    is_consumable BOOLEAN NOT NULL DEFAULT false,
    garment_type_id TEXT,
    shipping_class_id TEXT,
    featured_rank INTEGER NOT NULL DEFAULT 0,
    published_at TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX IF NOT EXISTS idx_erp_products_status_kind ON erp_products(status, kind);
CREATE INDEX IF NOT EXISTS idx_erp_products_category ON erp_products(category_id);
CREATE INDEX IF NOT EXISTS idx_erp_products_featured ON erp_products(featured_rank);
CREATE INDEX IF NOT EXISTS idx_erp_products_created ON erp_products(created_at);

-- Product translations
CREATE TABLE IF NOT EXISTS erp_product_translations (
    id TEXT PRIMARY KEY,
    product_id TEXT NOT NULL REFERENCES erp_products(id) ON DELETE CASCADE,
    locale TEXT NOT NULL,
    name TEXT NOT NULL,
    description TEXT,
    care_instructions TEXT,
    meta_title TEXT,
    meta_description TEXT,
    focus_keyword TEXT,
    UNIQUE(product_id, locale)
);
CREATE INDEX IF NOT EXISTS idx_erp_product_translations_name ON erp_product_translations(name);

-- Product variants
CREATE TABLE IF NOT EXISTS erp_product_variants (
    id TEXT PRIMARY KEY,
    product_id TEXT NOT NULL REFERENCES erp_products(id) ON DELETE CASCADE,
    sku TEXT NOT NULL UNIQUE,
    size TEXT,
    color TEXT,
    color_hex TEXT,
    label TEXT,
    price_override INTEGER,
    stock INTEGER NOT NULL DEFAULT 0,
    low_stock_threshold INTEGER NOT NULL DEFAULT 3,
    is_active BOOLEAN NOT NULL DEFAULT true,
    cost_price INTEGER NOT NULL DEFAULT 0,
    reorder_point INTEGER,
    reorder_quantity INTEGER,
    preferred_supplier_id TEXT,
    barcode TEXT UNIQUE,
    sku_is_manual BOOLEAN NOT NULL DEFAULT false,
    produced_in_house BOOLEAN NOT NULL DEFAULT false,
    lot_tracked BOOLEAN NOT NULL DEFAULT false,
    tracks_serials BOOLEAN NOT NULL DEFAULT false,
    warranty_months INTEGER,
    tax_class_id TEXT,
    sort_order INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX IF NOT EXISTS idx_erp_variants_product ON erp_product_variants(product_id);
CREATE INDEX IF NOT EXISTS idx_erp_variants_stock ON erp_product_variants(stock);

-- Product images
CREATE TABLE IF NOT EXISTS erp_product_images (
    id TEXT PRIMARY KEY,
    product_id TEXT NOT NULL REFERENCES erp_products(id) ON DELETE CASCADE,
    variant_id TEXT REFERENCES erp_product_variants(id) ON DELETE SET NULL,
    url TEXT NOT NULL,
    alt TEXT,
    sort_order INTEGER NOT NULL DEFAULT 0,
    is_primary BOOLEAN NOT NULL DEFAULT false,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX IF NOT EXISTS idx_erp_images_product ON erp_product_images(product_id, sort_order);

-- Product image translations
CREATE TABLE IF NOT EXISTS erp_product_image_translations (
    id TEXT PRIMARY KEY,
    image_id TEXT NOT NULL REFERENCES erp_product_images(id) ON DELETE CASCADE,
    locale TEXT NOT NULL,
    alt TEXT NOT NULL,
    UNIQUE(image_id, locale)
);

-- Product options (dynamic axes: Size, Color, Storage, etc.)
CREATE TABLE IF NOT EXISTS erp_product_options (
    id TEXT PRIMARY KEY,
    product_id TEXT NOT NULL REFERENCES erp_products(id) ON DELETE CASCADE,
    key TEXT NOT NULL,
    display TEXT NOT NULL DEFAULT 'button',
    sort_order INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(product_id, key)
);
CREATE INDEX IF NOT EXISTS idx_erp_options_product ON erp_product_options(product_id, sort_order);

-- Product option translations
CREATE TABLE IF NOT EXISTS erp_product_option_translations (
    id TEXT PRIMARY KEY,
    option_id TEXT NOT NULL REFERENCES erp_product_options(id) ON DELETE CASCADE,
    locale TEXT NOT NULL,
    label TEXT NOT NULL,
    UNIQUE(option_id, locale)
);

-- Product option values (specific values: "M", "Ferozi", "128GB")
CREATE TABLE IF NOT EXISTS erp_product_option_values (
    id TEXT PRIMARY KEY,
    option_id TEXT NOT NULL REFERENCES erp_product_options(id) ON DELETE CASCADE,
    key TEXT NOT NULL,
    swatch_hex TEXT,
    swatch_url TEXT,
    sort_order INTEGER NOT NULL DEFAULT 0,
    UNIQUE(option_id, key)
);
CREATE INDEX IF NOT EXISTS idx_erp_option_values_option ON erp_product_option_values(option_id, sort_order);

-- Product option value translations
CREATE TABLE IF NOT EXISTS erp_product_option_value_translations (
    id TEXT PRIMARY KEY,
    value_id TEXT NOT NULL REFERENCES erp_product_option_values(id) ON DELETE CASCADE,
    locale TEXT NOT NULL,
    label TEXT NOT NULL,
    UNIQUE(value_id, locale)
);

-- Variant-option-value join (defines a variant by its option values)
CREATE TABLE IF NOT EXISTS erp_variant_option_values (
    variant_id TEXT NOT NULL REFERENCES erp_product_variants(id) ON DELETE CASCADE,
    option_id TEXT NOT NULL,
    value_id TEXT NOT NULL REFERENCES erp_product_option_values(id) ON DELETE RESTRICT,
    PRIMARY KEY (variant_id, option_id)
);
CREATE INDEX IF NOT EXISTS idx_erp_variant_option_value ON erp_variant_option_values(value_id);

-- Option templates (store-level reusable axes)
CREATE TABLE IF NOT EXISTS erp_option_templates (
    id TEXT PRIMARY KEY,
    key TEXT NOT NULL UNIQUE,
    display TEXT NOT NULL DEFAULT 'button',
    sort_order INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX IF NOT EXISTS idx_erp_option_templates_order ON erp_option_templates(sort_order);

-- Option template translations
CREATE TABLE IF NOT EXISTS erp_option_template_translations (
    id TEXT PRIMARY KEY,
    template_id TEXT NOT NULL REFERENCES erp_option_templates(id) ON DELETE CASCADE,
    locale TEXT NOT NULL,
    label TEXT NOT NULL,
    UNIQUE(template_id, locale)
);

-- Option template values
CREATE TABLE IF NOT EXISTS erp_option_template_values (
    id TEXT PRIMARY KEY,
    template_id TEXT NOT NULL REFERENCES erp_option_templates(id) ON DELETE CASCADE,
    key TEXT NOT NULL,
    swatch_hex TEXT,
    swatch_url TEXT,
    sort_order INTEGER NOT NULL DEFAULT 0,
    UNIQUE(template_id, key)
);
CREATE INDEX IF NOT EXISTS idx_erp_template_values_template ON erp_option_template_values(template_id, sort_order);

-- Option template value translations
CREATE TABLE IF NOT EXISTS erp_option_template_value_translations (
    id TEXT PRIMARY KEY,
    value_id TEXT NOT NULL REFERENCES erp_option_template_values(id) ON DELETE CASCADE,
    locale TEXT NOT NULL,
    label TEXT NOT NULL,
    UNIQUE(value_id, locale)
);

-- Collections (merchandising surfaces)
CREATE TABLE IF NOT EXISTS erp_collections (
    id TEXT PRIMARY KEY,
    slug TEXT NOT NULL UNIQUE,
    rule TEXT,
    sort_strategy TEXT NOT NULL DEFAULT 'manual',
    image_url TEXT,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX IF NOT EXISTS idx_erp_collections_active ON erp_collections(is_active);

-- Collection translations
CREATE TABLE IF NOT EXISTS erp_collection_translations (
    id TEXT PRIMARY KEY,
    collection_id TEXT NOT NULL REFERENCES erp_collections(id) ON DELETE CASCADE,
    locale TEXT NOT NULL,
    name TEXT NOT NULL,
    description TEXT,
    meta_title TEXT,
    meta_description TEXT,
    UNIQUE(collection_id, locale)
);

-- Collection products (manual membership)
CREATE TABLE IF NOT EXISTS erp_collection_products (
    collection_id TEXT NOT NULL REFERENCES erp_collections(id) ON DELETE CASCADE,
    product_id TEXT NOT NULL REFERENCES erp_products(id) ON DELETE CASCADE,
    sort_order INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (collection_id, product_id)
);
CREATE INDEX IF NOT EXISTS idx_erp_collection_products_product ON erp_collection_products(product_id);

-- Redirects (slug change tracking)
CREATE TABLE IF NOT EXISTS erp_redirects (
    id TEXT PRIMARY KEY,
    from_path TEXT NOT NULL UNIQUE,
    to_path TEXT NOT NULL,
    status_code INTEGER NOT NULL DEFAULT 301,
    is_automatic BOOLEAN NOT NULL DEFAULT false,
    hit_count INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX IF NOT EXISTS idx_erp_redirects_automatic ON erp_redirects(is_automatic);

-- Stock adjustments (audit trail)
CREATE TABLE IF NOT EXISTS erp_stock_adjustments (
    id TEXT PRIMARY KEY,
    variant_id TEXT NOT NULL REFERENCES erp_product_variants(id) ON DELETE CASCADE,
    delta INTEGER NOT NULL,
    resulting_stock INTEGER NOT NULL,
    reason TEXT NOT NULL,
    note TEXT,
    actor_id TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX IF NOT EXISTS idx_erp_stock_adjustments_variant ON erp_stock_adjustments(variant_id, created_at);

-- Product size bias (AI-computed from reviews)
CREATE TABLE IF NOT EXISTS erp_product_size_bias (
    product_id TEXT PRIMARY KEY REFERENCES erp_products(id) ON DELETE CASCADE,
    too_small_pct REAL NOT NULL DEFAULT 0,
    true_to_size_pct REAL NOT NULL DEFAULT 0,
    too_large_pct REAL NOT NULL DEFAULT 0,
    sample_size INTEGER NOT NULL DEFAULT 0,
    shift INTEGER NOT NULL DEFAULT 0,
    computed_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Product cross-sells (also-bought / complete-the-look)
CREATE TABLE IF NOT EXISTS erp_product_cross_sells (
    source_product_id TEXT NOT NULL REFERENCES erp_products(id) ON DELETE CASCADE,
    target_product_id TEXT NOT NULL REFERENCES erp_products(id) ON DELETE CASCADE,
    label TEXT,
    PRIMARY KEY (source_product_id, target_product_id)
);

-- AI suggestions (draft AI-generated copy)
CREATE TABLE IF NOT EXISTS erp_product_ai_suggestions (
    id TEXT PRIMARY KEY,
    product_id TEXT NOT NULL REFERENCES erp_products(id) ON DELETE CASCADE,
    image_id TEXT REFERENCES erp_product_images(id) ON DELETE CASCADE,
    kind TEXT NOT NULL,
    locale TEXT NOT NULL,
    suggested_text TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending',
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    reviewed_at TEXT,
    reviewed_by_id TEXT
);
CREATE INDEX IF NOT EXISTS idx_erp_ai_suggestions_product ON erp_product_ai_suggestions(product_id, status);
CREATE INDEX IF NOT EXISTS idx_erp_ai_suggestions_status ON erp_product_ai_suggestions(status, created_at);

-- Category presets (preconfigured sector taxonomies)
CREATE TABLE IF NOT EXISTS erp_category_presets (
    id TEXT PRIMARY KEY,
    key TEXT NOT NULL UNIQUE,
    label TEXT NOT NULL,
    tree_json TEXT NOT NULL,
    description TEXT
);
