-- Storefront / Link-in-Bio Builder

-- ---------------------------------------------------------------------------
-- Storefronts
-- ---------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS storefronts (
    id TEXT PRIMARY KEY,
    brand_id TEXT NOT NULL,
    slug TEXT NOT NULL UNIQUE,
    title TEXT NOT NULL,
    description TEXT,
    theme TEXT,
    hero_image TEXT,
    seo_title TEXT,
    seo_description TEXT,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    FOREIGN KEY (brand_id) REFERENCES brands(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_storefronts_brand ON storefronts(brand_id);
CREATE INDEX IF NOT EXISTS idx_storefronts_slug ON storefronts(slug);

-- ---------------------------------------------------------------------------
-- Storefront Sections
-- ---------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS storefront_sections (
    id TEXT PRIMARY KEY,
    storefront_id TEXT NOT NULL,
    section_type TEXT NOT NULL,
    sort_order INTEGER DEFAULT 0,
    content TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    FOREIGN KEY (storefront_id) REFERENCES storefronts(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_storefront_sections_storefront ON storefront_sections(storefront_id);
CREATE INDEX IF NOT EXISTS idx_storefront_sections_order ON storefront_sections(sort_order);

-- ---------------------------------------------------------------------------
-- Storefront Visits
-- ---------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS storefront_visits (
    id TEXT PRIMARY KEY,
    storefront_id TEXT NOT NULL,
    visitor_ip TEXT,
    user_agent TEXT,
    path TEXT,
    
    visited_at TEXT DEFAULT (NOW()),
    FOREIGN KEY (storefront_id) REFERENCES storefronts(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_storefront_visits_storefront ON storefront_visits(storefront_id);
CREATE INDEX IF NOT EXISTS idx_storefront_visits_visited ON storefront_visits(visited_at);
