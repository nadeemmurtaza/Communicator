-- Migration 0034: Fin Vision (Image Understanding) (PostgreSQL)
-- Enables the AI to analyze images/screenshots sent by customers
-- and incorporate visual context into conversation responses.
-- Mirrors Intercom's Fin Vision capability.

-- ============================================================================
-- 1. VISION IMAGES (uploaded images for AI analysis)
-- ============================================================================
CREATE TABLE IF NOT EXISTS vision_images (
    id TEXT PRIMARY KEY DEFAULT gen_random_uuid()::TEXT,
    organization_id TEXT NOT NULL,
    conversation_id TEXT,
    contact_id TEXT,
    channel TEXT,
    image_url TEXT NOT NULL DEFAULT '',
    image_data TEXT, -- base64 encoded image data (optional, for storage)
    mime_type TEXT NOT NULL DEFAULT 'image/jpeg',
    file_size BOOLEAN NOT NULL DEFAULT false,
    status TEXT NOT NULL DEFAULT 'uploaded', -- 'uploaded', 'analyzing', 'completed', 'failed'
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_vision_images_org ON vision_images(organization_id);
CREATE INDEX IF NOT EXISTS idx_vision_images_conversation ON vision_images(conversation_id);
CREATE INDEX IF NOT EXISTS idx_vision_images_status ON vision_images(status);

-- ============================================================================
-- 2. VISION ANALYSES (AI analysis results for images)
-- ============================================================================
CREATE TABLE IF NOT EXISTS vision_analyses (
    id TEXT PRIMARY KEY DEFAULT gen_random_uuid()::TEXT,
    image_id TEXT NOT NULL,
    analysis_type TEXT NOT NULL DEFAULT 'general', -- 'general', 'screenshot', 'document', 'error_code', 'product'
    prompt TEXT, -- custom prompt for analysis
    result TEXT NOT NULL DEFAULT '', -- JSON analysis result
    confidence REAL NOT NULL DEFAULT 0.0,
    objects_detected TEXT NOT NULL DEFAULT '[]', -- JSON array of detected objects
    text_extracted TEXT, -- OCR text extracted from image
    summary TEXT, -- human-readable summary
    processing_time_ms BOOLEAN NOT NULL DEFAULT false,
    claude_model TEXT NOT NULL DEFAULT 'claude-sonnet-4-20250514',
    error_message TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_vision_analyses_image ON vision_analyses(image_id);
CREATE INDEX IF NOT EXISTS idx_vision_analyses_type ON vision_analyses(analysis_type);

-- ============================================================================
-- 3. VISION SETTINGS (per-organization configuration)
-- ============================================================================
CREATE TABLE IF NOT EXISTS vision_settings (
    id TEXT PRIMARY KEY DEFAULT gen_random_uuid()::TEXT,
    organization_id TEXT NOT NULL UNIQUE,
    enabled BOOLEAN NOT NULL DEFAULT true,
    auto_analyze BOOLEAN NOT NULL DEFAULT true, -- auto-analyze on upload
    default_analysis_type TEXT NOT NULL DEFAULT 'general',
    claude_model TEXT NOT NULL DEFAULT 'claude-sonnet-4-20250514',
    max_file_size_mb REAL NOT NULL DEFAULT 10.0,
    allowed_mime_types TEXT NOT NULL DEFAULT '["image/jpeg", "image/png", "image/webp", "image/gif"]',
    ocr_enabled BOOLEAN NOT NULL DEFAULT true,
    object_detection_enabled BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Default settings
INSERT INTO vision_settings (id, organization_id)
VALUES ('vision-settings-default', 'org-default')
ON CONFLICT (id) DO NOTHING;
