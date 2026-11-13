-- WhatsApp Campaigns, Templates, Flows, Industries, and Message Limits

-- Industry definitions for template/flow grouping
CREATE TABLE IF NOT EXISTS industries (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    description TEXT,
    icon TEXT,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_industries_active ON industries(is_active);

-- Seed common industries
INSERT INTO industries (id, name, description, icon) VALUES
('ind_healthcare', 'Healthcare', 'Hospitals, clinics, pharmacies, telemedicine', 'ðŸ¥'),
('ind_ecommerce', 'E-commerce', 'Online stores, marketplaces, dropshipping', 'ðŸ›’'),
('ind_education', 'Education', 'Schools, universities, online courses, tutoring', 'ðŸŽ“'),
('ind_finance', 'Finance', 'Banks, insurance, fintech, crypto', 'ðŸ’°'),
('ind_realestate', 'Real Estate', 'Property sales, rentals, property management', 'ðŸ '),
('ind_restaurant', 'Restaurant', 'Restaurants, cafes, food delivery, catering', 'ðŸ½ï¸'),
('ind_retail', 'Retail', 'Brick-and-mortar stores, fashion, electronics', 'ðŸª'),
('ind_technology', 'Technology', 'SaaS, IT services, software companies', 'ðŸ’»'),
('ind_travel', 'Travel', 'Airlines, hotels, tour operators, travel agencies', 'âœˆï¸'),
('ind_fitness', 'Fitness', 'Gyms, personal training, wellness, yoga', 'ðŸ’ª'),
('ind_legal', 'Legal', 'Law firms, legal services, compliance', 'âš–ï¸'),
('ind_automotive', 'Automotive', 'Dealerships, repair shops, car rentals', 'ðŸš—'),
('ind_general', 'General', 'General purpose templates for any industry', 'ðŸ“‹');

-- Template groups by industry for organizing flows
CREATE TABLE IF NOT EXISTS template_groups (
    id TEXT PRIMARY KEY,
    brand_id TEXT NOT NULL,
    industry_id TEXT,
    name TEXT NOT NULL,
    description TEXT,
    sort_order BIGINT DEFAULT 0,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    FOREIGN KEY (brand_id) REFERENCES brands(id) ON DELETE CASCADE,
    FOREIGN KEY (industry_id) REFERENCES industries(id) ON DELETE SET NULL
);

CREATE INDEX IF NOT EXISTS idx_template_groups_brand ON template_groups(brand_id);
CREATE INDEX IF NOT EXISTS idx_template_groups_industry ON template_groups(industry_id);

-- Advanced customizable template flows
CREATE TABLE IF NOT EXISTS whatsapp_template_flows (
    id TEXT PRIMARY KEY,
    brand_id TEXT NOT NULL,
    group_id TEXT,
    industry_id TEXT,
    name TEXT NOT NULL,
    type TEXT NOT NULL DEFAULT 'template',
    category TEXT,
    body TEXT NOT NULL,
    header TEXT,
    footer TEXT,
    buttons TEXT,
    sections TEXT,
    language TEXT DEFAULT 'en_US',
    status TEXT DEFAULT 'draft',
    is_active BOOLEAN DEFAULT true,
    metadata TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    FOREIGN KEY (brand_id) REFERENCES brands(id) ON DELETE CASCADE,
    FOREIGN KEY (group_id) REFERENCES template_groups(id) ON DELETE SET NULL,
    FOREIGN KEY (industry_id) REFERENCES industries(id) ON DELETE SET NULL
);

CREATE INDEX IF NOT EXISTS idx_template_flows_brand ON whatsapp_template_flows(brand_id);
CREATE INDEX IF NOT EXISTS idx_template_flows_industry ON whatsapp_template_flows(industry_id);
CREATE INDEX IF NOT EXISTS idx_template_flows_group ON whatsapp_template_flows(group_id);
CREATE INDEX IF NOT EXISTS idx_template_flows_status ON whatsapp_template_flows(status);

-- Multi-step flow definitions
CREATE TABLE IF NOT EXISTS whatsapp_flow_steps (
    id TEXT PRIMARY KEY,
    flow_id TEXT NOT NULL,
    step_number INTEGER NOT NULL,
    condition TEXT,
    action_type TEXT NOT NULL,
    action_config TEXT,
    next_step_id TEXT,
    delay_seconds BOOLEAN DEFAULT false,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    FOREIGN KEY (flow_id) REFERENCES whatsapp_template_flows(id) ON DELETE CASCADE,
    FOREIGN KEY (next_step_id) REFERENCES whatsapp_flow_steps(id) ON DELETE SET NULL
);

CREATE INDEX IF NOT EXISTS idx_flow_steps_flow ON whatsapp_flow_steps(flow_id);
CREATE INDEX IF NOT EXISTS idx_flow_steps_number ON whatsapp_flow_steps(flow_id, step_number);

-- Campaign execution tracking
CREATE TABLE IF NOT EXISTS campaigns (
    id TEXT PRIMARY KEY,
    brand_id TEXT NOT NULL,
    name TEXT NOT NULL,
    type TEXT NOT NULL DEFAULT 'whatsapp',
    template_flow_id TEXT,
    status TEXT DEFAULT 'draft',
    scheduled_at TEXT,
    started_at TIMESTAMPTZ,
    completed_at TEXT,
    total_contacts BIGINT DEFAULT 0,
    sent_count BIGINT DEFAULT 0,
    delivered_count BIGINT DEFAULT 0,
    read_count BIGINT DEFAULT 0,
    failed_count BIGINT DEFAULT 0,
    retry_count BIGINT DEFAULT 0,
    progress TEXT,
    metadata TEXT,
    created_by TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    FOREIGN KEY (brand_id) REFERENCES brands(id) ON DELETE CASCADE,
    FOREIGN KEY (template_flow_id) REFERENCES whatsapp_template_flows(id) ON DELETE SET NULL
);

CREATE INDEX IF NOT EXISTS idx_campaigns_brand ON campaigns(brand_id);
CREATE INDEX IF NOT EXISTS idx_campaigns_status ON campaigns(status);

-- Contact selection for campaigns
CREATE TABLE IF NOT EXISTS campaign_contacts (
    id TEXT PRIMARY KEY,
    campaign_id TEXT NOT NULL,
    contact_id TEXT NOT NULL,
    status TEXT DEFAULT 'pending',
    message_id TEXT,
    error TEXT,
    sent_at TEXT,
    delivered_at TEXT,
    read_at TEXT,
    replied_at TEXT,
    retry_count BIGINT DEFAULT 0,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    FOREIGN KEY (campaign_id) REFERENCES campaigns(id) ON DELETE CASCADE,
    FOREIGN KEY (contact_id) REFERENCES customers(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_campaign_contacts_campaign ON campaign_contacts(campaign_id);
CREATE INDEX IF NOT EXISTS idx_campaign_contacts_status ON campaign_contacts(status);
CREATE INDEX IF NOT EXISTS idx_campaign_contacts_contact ON campaign_contacts(contact_id);

-- Daily/weekly message limit tracking
CREATE TABLE IF NOT EXISTS message_limits (
    id TEXT PRIMARY KEY,
    brand_id TEXT NOT NULL,
    period_start TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    period_end TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    utility_count BIGINT DEFAULT 0,
    marketing_count BIGINT DEFAULT 0,
    authentication_count BIGINT DEFAULT 0,
    customer_service_count BIGINT DEFAULT 0,
    total_limit INTEGER DEFAULT 1000,
    remaining INTEGER DEFAULT 1000,
    quality_rating TEXT,
    messaging_tier TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    FOREIGN KEY (brand_id) REFERENCES brands(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_message_limits_brand ON message_limits(brand_id);
CREATE INDEX IF NOT EXISTS idx_message_limits_period ON message_limits(brand_id, period_start);

-- Retry queue for failed campaign messages
CREATE TABLE IF NOT EXISTS campaign_retry_queue (
    id TEXT PRIMARY KEY,
    campaign_id TEXT NOT NULL,
    contact_id TEXT NOT NULL,
    template_flow_id TEXT,
    retry_count BIGINT DEFAULT 0,
    max_retries INTEGER DEFAULT 3,
    next_retry_at TEXT NOT NULL,
    error TEXT,
    status TEXT DEFAULT 'pending',
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    FOREIGN KEY (campaign_id) REFERENCES campaigns(id) ON DELETE CASCADE,
    FOREIGN KEY (contact_id) REFERENCES customers(id) ON DELETE CASCADE,
    FOREIGN KEY (template_flow_id) REFERENCES whatsapp_template_flows(id) ON DELETE SET NULL
);

CREATE INDEX IF NOT EXISTS idx_retry_queue_campaign ON campaign_retry_queue(campaign_id);
CREATE INDEX IF NOT EXISTS idx_retry_queue_next_retry ON campaign_retry_queue(next_retry_at, status);
CREATE INDEX IF NOT EXISTS idx_retry_queue_status ON campaign_retry_queue(status);

-- Add industry_id to brands table for brand-level industry selection
ALTER TABLE brands ADD COLUMN industry_id TEXT REFERENCES industries(id) ON DELETE SET NULL;
