-- Enhanced WhatsApp Template Management: Quality scores, variables, multi-language

-- Add quality score and rating columns to whatsapp_templates
ALTER TABLE whatsapp_templates ADD COLUMN quality_score INTEGER;
ALTER TABLE whatsapp_templates ADD COLUMN quality_rating TEXT;
ALTER TABLE whatsapp_templates ADD COLUMN variables TEXT;
ALTER TABLE whatsapp_templates ADD COLUMN header_text TEXT;
ALTER TABLE whatsapp_templates ADD COLUMN footer_text TEXT;

-- Create template language variants table for multi-language support
CREATE TABLE IF NOT EXISTS whatsapp_template_variants (
    id TEXT PRIMARY KEY,
    template_id TEXT NOT NULL,
    brand_id TEXT NOT NULL,
    language TEXT NOT NULL,
    status TEXT DEFAULT 'approved',
    body TEXT NOT NULL,
    components TEXT,
    quality_score INTEGER,
    quality_rating TEXT,
    variables TEXT,
    header_text TEXT,
    footer_text TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    FOREIGN KEY (template_id) REFERENCES whatsapp_templates(id) ON DELETE CASCADE,
    FOREIGN KEY (brand_id) REFERENCES brands(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_template_variants_template ON whatsapp_template_variants(template_id);
CREATE INDEX IF NOT EXISTS idx_template_variants_brand ON whatsapp_template_variants(brand_id);
CREATE INDEX IF NOT EXISTS idx_template_variants_language ON whatsapp_template_variants(language);

-- Add template usage tracking
CREATE TABLE IF NOT EXISTS whatsapp_template_usage (
    id TEXT PRIMARY KEY,
    brand_id TEXT NOT NULL,
    template_name TEXT NOT NULL,
    template_id TEXT,
    sent_count BIGINT DEFAULT 0,
    last_sent_at TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    FOREIGN KEY (brand_id) REFERENCES brands(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_template_usage_brand ON whatsapp_template_usage(brand_id);
CREATE INDEX IF NOT EXISTS idx_template_usage_name ON whatsapp_template_usage(template_name);
