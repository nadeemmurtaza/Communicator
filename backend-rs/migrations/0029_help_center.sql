-- Phase 21: Public Help Center â€” Customer-facing self-service portal
-- Provides a searchable, categorized help center with articles that customers
-- can browse before contacting support, reducing AI load and improving
-- self-service resolution rates. Mirrors Intercom's Help Center feature.

-- Help Center categories (top-level organization)
CREATE TABLE IF NOT EXISTS help_center_categories (
    id TEXT PRIMARY KEY,
    organization_id TEXT NOT NULL,
    name TEXT NOT NULL,
    slug TEXT NOT NULL UNIQUE,
    description TEXT,
    icon TEXT, -- emoji or icon name
    display_order INTEGER NOT NULL DEFAULT 0,
    is_visible BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_help_center_categories_org ON help_center_categories(organization_id);
CREATE INDEX IF NOT EXISTS idx_help_center_categories_visible ON help_center_categories(is_visible);

-- Help Center articles (individual help articles)
CREATE TABLE IF NOT EXISTS help_center_articles (
    id TEXT PRIMARY KEY,
    organization_id TEXT NOT NULL,
    category_id TEXT NOT NULL,
    title TEXT NOT NULL,
    slug TEXT NOT NULL UNIQUE,
    content TEXT NOT NULL, -- HTML or markdown content
    author_id TEXT, -- user who created the article
    status TEXT NOT NULL DEFAULT 'draft', -- 'draft' | 'published' | 'archived'
    is_pinned BOOLEAN NOT NULL DEFAULT false,
    view_count INTEGER NOT NULL DEFAULT 0,
    helpful_count INTEGER NOT NULL DEFAULT 0,
    not_helpful_count INTEGER NOT NULL DEFAULT 0,
    tags TEXT, -- JSON array of tags
    seo_title TEXT, -- SEO-optimized title
    seo_description TEXT, -- SEO meta description
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    published_at TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_help_center_articles_org ON help_center_articles(organization_id);
CREATE INDEX IF NOT EXISTS idx_help_center_articles_category ON help_center_articles(category_id);
CREATE INDEX IF NOT EXISTS idx_help_center_articles_status ON help_center_articles(status);
CREATE INDEX IF NOT EXISTS idx_help_center_articles_slug ON help_center_articles(slug);

-- Help Center search analytics (track what customers search for)
CREATE TABLE IF NOT EXISTS help_center_searches (
    id TEXT PRIMARY KEY,
    organization_id TEXT NOT NULL,
    query TEXT NOT NULL,
    result_count BOOLEAN NOT NULL DEFAULT false,
    clicked_article_id TEXT,
    session_id TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_help_center_searches_org ON help_center_searches(organization_id);
CREATE INDEX IF NOT EXISTS idx_help_center_searches_query ON help_center_searches(query);
CREATE INDEX IF NOT EXISTS idx_help_center_searches_created ON help_center_searches(created_at);

-- Help Center settings (per-organization configuration)
CREATE TABLE IF NOT EXISTS help_center_settings (
    id TEXT PRIMARY KEY,
    organization_id TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL, -- public-facing name of the help center
    tagline TEXT,
    logo_url TEXT,
    primary_color TEXT DEFAULT '#6366f1',
    enable_search BOOLEAN NOT NULL DEFAULT true,
    enable_article_feedback BOOLEAN NOT NULL DEFAULT true,
    enable_email_subscriptions BOOLEAN NOT NULL DEFAULT false,
    default_language TEXT DEFAULT 'en',
    custom_css TEXT,
    footer_text TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Default categories and settings are seeded by the application on first use