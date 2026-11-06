-- Migration 0005: Comprehensive Client Profiles with Self-Learning Engine
-- Adds unified customer IDs, rich client profiles, self-learning engine,
-- transaction history, preferences, and RAG integration

-- ============================================================================
-- 1. ENHANCE EXISTING CUSTOMERS TABLE
-- ============================================================================

-- Add unified customer ID and rich profile fields to customers
ALTER TABLE customers ADD COLUMN customer_id TEXT UNIQUE;
ALTER TABLE customers ADD COLUMN brand_id TEXT REFERENCES brands(id);
ALTER TABLE customers ADD COLUMN first_name TEXT DEFAULT '';
ALTER TABLE customers ADD COLUMN last_name TEXT DEFAULT '';
ALTER TABLE customers ADD COLUMN email TEXT DEFAULT '';
ALTER TABLE customers ADD COLUMN phone TEXT DEFAULT '';
ALTER TABLE customers ADD COLUMN company TEXT DEFAULT '';
ALTER TABLE customers ADD COLUMN designation TEXT DEFAULT '';
ALTER TABLE customers ADD COLUMN role TEXT DEFAULT '';
ALTER TABLE customers ADD COLUMN industry TEXT DEFAULT '';
ALTER TABLE customers ADD COLUMN avatar_url TEXT DEFAULT '';
ALTER TABLE customers ADD COLUMN timezone TEXT DEFAULT '';
ALTER TABLE customers ADD COLUMN language TEXT DEFAULT 'en';
ALTER TABLE customers ADD COLUMN customer_type TEXT DEFAULT 'individual';  -- 'individual', 'business'
ALTER TABLE customers ADD COLUMN customer_segment TEXT DEFAULT '';
ALTER TABLE customers ADD COLUMN customer_tier TEXT DEFAULT '';  -- 'bronze', 'silver', 'gold', 'platinum'
ALTER TABLE customers ADD COLUMN lifetime_value REAL DEFAULT 0;
ALTER TABLE customers ADD COLUMN total_orders INTEGER DEFAULT 0;
ALTER TABLE customers ADD COLUMN total_spent REAL DEFAULT 0;
ALTER TABLE customers ADD COLUMN last_order_date TEXT;
ALTER TABLE customers ADD COLUMN first_contact_date TEXT;
ALTER TABLE customers ADD COLUMN engagement_score REAL DEFAULT 0;
ALTER TABLE customers ADD COLUMN sentiment_score REAL DEFAULT 0;
ALTER TABLE customers ADD COLUMN notes TEXT DEFAULT '';
ALTER TABLE customers ADD COLUMN tags TEXT DEFAULT '[]';
ALTER TABLE customers ADD COLUMN metadata TEXT DEFAULT '{}';
ALTER TABLE customers ADD COLUMN created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'));
ALTER TABLE customers ADD COLUMN updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'));

CREATE INDEX IF NOT EXISTS idx_customers_customer_id ON customers(customer_id);
CREATE INDEX IF NOT EXISTS idx_customers_brand ON customers(brand_id);
CREATE INDEX IF NOT EXISTS idx_customers_email ON customers(email);
CREATE INDEX IF NOT EXISTS idx_customers_phone ON customers(phone);
CREATE INDEX IF NOT EXISTS idx_customers_company ON customers(company);

-- ============================================================================
-- 2. UNIFIED CUSTOMER ID MAPPING (Cross-Channel Identity)
-- ============================================================================

-- Maps different channel identifiers to a single unified customer
CREATE TABLE IF NOT EXISTS customer_channel_ids (
    id TEXT PRIMARY KEY,
    customer_id TEXT NOT NULL REFERENCES customers(id) ON DELETE CASCADE,
    channel_type TEXT NOT NULL,  -- 'whatsapp', 'sms', 'email', 'telegram', 'line', 'facebook', 'instagram', 'tiktok'
    channel_identifier TEXT NOT NULL,  -- phone number, email, telegram username, etc.
    channel_profile_name TEXT DEFAULT '',  -- name from the channel's profile
    is_primary INTEGER NOT NULL DEFAULT 0,
    verified_at TEXT,
    last_seen_at TEXT,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE INDEX IF NOT EXISTS idx_customer_channel_ids_customer ON customer_channel_ids(customer_id);
CREATE UNIQUE INDEX IF NOT EXISTS idx_customer_channel_ids_unique ON customer_channel_ids(channel_type, channel_identifier);
CREATE INDEX IF NOT EXISTS idx_customer_channel_ids_channel ON customer_channel_ids(channel_type);

-- ============================================================================
-- 3. CLIENT PROFILE DATA SOURCES
-- ============================================================================

-- Track the source of each data field in client profiles
CREATE TABLE IF NOT EXISTS client_profile_sources (
    id TEXT PRIMARY KEY,
    customer_id TEXT NOT NULL REFERENCES customers(id) ON DELETE CASCADE,
    field_name TEXT NOT NULL,
    source_type TEXT NOT NULL,  -- 'meta', 'telegram', 'line', 'user', 'ai_learning', 'import'
    source_user_id TEXT DEFAULT '',
    source_user_name TEXT DEFAULT '',
    source_record_id TEXT DEFAULT '',
    source_url TEXT DEFAULT '',
    synced_at TEXT,
    confidence_score REAL DEFAULT 1.0,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE INDEX IF NOT EXISTS idx_client_sources_customer ON client_profile_sources(customer_id);
CREATE INDEX IF NOT EXISTS idx_client_sources_field ON client_profile_sources(field_name);

-- ============================================================================
-- 4. SELF-LEARNING ENGINE
-- ============================================================================

-- AI-extracted insights from conversations
CREATE TABLE IF NOT EXISTS client_learning_insights (
    id TEXT PRIMARY KEY,
    customer_id TEXT NOT NULL REFERENCES customers(id) ON DELETE CASCADE,
    insight_type TEXT NOT NULL,  -- 'preference', 'behavior', 'sentiment', 'intent', 'pain_point', 'interest'
    category TEXT NOT NULL,  -- 'communication', 'product', 'service', 'pricing', 'support'
    insight TEXT NOT NULL,
    confidence REAL NOT NULL DEFAULT 0.5,
    evidence_count INTEGER NOT NULL DEFAULT 1,
    source_conversation_id TEXT,
    source_message_id TEXT,
    extracted_at TEXT NOT NULL,
    is_verified INTEGER NOT NULL DEFAULT 0,
    verified_by TEXT DEFAULT '',
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE INDEX IF NOT EXISTS idx_learning_insights_customer ON client_learning_insights(customer_id);
CREATE INDEX IF NOT EXISTS idx_learning_insights_type ON client_learning_insights(insight_type);

-- Learning patterns and trends
CREATE TABLE IF NOT EXISTS client_learning_patterns (
    id TEXT PRIMARY KEY,
    customer_id TEXT NOT NULL REFERENCES customers(id) ON DELETE CASCADE,
    pattern_type TEXT NOT NULL,  -- 'communication_style', 'response_time', 'channel_preference', 'time_preference'
    pattern_data TEXT NOT NULL,  -- JSON with pattern details
    confidence REAL NOT NULL DEFAULT 0.5,
    sample_size INTEGER NOT NULL DEFAULT 0,
    last_updated TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE INDEX IF NOT EXISTS idx_learning_patterns_customer ON client_learning_patterns(customer_id);

-- ============================================================================
-- 5. CLIENT PREFERENCES
-- ============================================================================

-- Communication preferences learned from behavior
CREATE TABLE IF NOT EXISTS client_preferences (
    id TEXT PRIMARY KEY,
    customer_id TEXT NOT NULL REFERENCES customers(id) ON DELETE CASCADE,
    preference_type TEXT NOT NULL,  -- 'communication_style', 'response_time', 'channel', 'time', 'language'
    preference_value TEXT NOT NULL,
    confidence REAL NOT NULL DEFAULT 0.5,
    source TEXT NOT NULL DEFAULT 'ai_learning',  -- 'ai_learning', 'user_set', 'inferred'
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE INDEX IF NOT EXISTS idx_client_preferences_customer ON client_preferences(customer_id);
CREATE INDEX IF NOT EXISTS idx_client_preferences_type ON client_preferences(preference_type);

-- ============================================================================
-- 6. TRANSACTION HISTORY
-- ============================================================================

-- Orders and purchases
CREATE TABLE IF NOT EXISTS client_orders (
    id TEXT PRIMARY KEY,
    customer_id TEXT NOT NULL REFERENCES customers(id) ON DELETE CASCADE,
    order_number TEXT NOT NULL,
    order_date TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending',  -- 'pending', 'confirmed', 'shipped', 'delivered', 'cancelled'
    total_amount REAL NOT NULL DEFAULT 0,
    currency TEXT NOT NULL DEFAULT 'USD',
    items TEXT DEFAULT '[]',  -- JSON array of order items
    payment_status TEXT DEFAULT 'pending',  -- 'pending', 'paid', 'refunded'
    payment_method TEXT DEFAULT '',
    channel TEXT DEFAULT '',  -- channel where order was placed
    metadata TEXT DEFAULT '{}',
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE INDEX IF NOT EXISTS idx_client_orders_customer ON client_orders(customer_id);
CREATE INDEX IF NOT EXISTS idx_client_orders_date ON client_orders(order_date);

-- Invoices
CREATE TABLE IF NOT EXISTS client_invoices (
    id TEXT PRIMARY KEY,
    customer_id TEXT NOT NULL REFERENCES customers(id) ON DELETE CASCADE,
    invoice_number TEXT NOT NULL,
    order_id TEXT REFERENCES client_orders(id),
    invoice_date TEXT NOT NULL,
    due_date TEXT,
    total_amount REAL NOT NULL DEFAULT 0,
    paid_amount REAL NOT NULL DEFAULT 0,
    remaining_amount REAL NOT NULL DEFAULT 0,
    currency TEXT NOT NULL DEFAULT 'USD',
    status TEXT NOT NULL DEFAULT 'pending',  -- 'pending', 'paid', 'overdue', 'cancelled'
    payment_method TEXT DEFAULT '',
    metadata TEXT DEFAULT '{}',
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE INDEX IF NOT EXISTS idx_client_invoices_customer ON client_invoices(customer_id);
CREATE INDEX IF NOT EXISTS idx_client_invoices_order ON client_invoices(order_id);

-- ============================================================================
-- 7. BANK ACCOUNT & PAYMENT DETAILS
-- ============================================================================

-- Stored payment and bank details for clients
CREATE TABLE IF NOT EXISTS client_payment_details (
    id TEXT PRIMARY KEY,
    customer_id TEXT NOT NULL REFERENCES customers(id) ON DELETE CASCADE,
    detail_type TEXT NOT NULL,  -- 'bank_account', 'credit_card', 'paypal', 'crypto'
    account_holder TEXT DEFAULT '',
    account_number TEXT DEFAULT '',
    bank_name TEXT DEFAULT '',
    routing_number TEXT DEFAULT '',
    card_last_four TEXT DEFAULT '',
    is_default INTEGER NOT NULL DEFAULT 0,
    is_verified INTEGER NOT NULL DEFAULT 0,
    metadata TEXT DEFAULT '{}',
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE INDEX IF NOT EXISTS idx_payment_details_customer ON client_payment_details(customer_id);

-- ============================================================================
-- 8. CLIENT CONTACTS (All Channels)
-- ============================================================================

-- All contact points for a client across all channels
CREATE TABLE IF NOT EXISTS client_contact_points (
    id TEXT PRIMARY KEY,
    customer_id TEXT NOT NULL REFERENCES customers(id) ON DELETE CASCADE,
    contact_type TEXT NOT NULL,  -- 'email', 'phone', 'whatsapp', 'telegram', 'line', 'social'
    contact_value TEXT NOT NULL,
    is_primary INTEGER NOT NULL DEFAULT 0,
    is_verified INTEGER NOT NULL DEFAULT 0,
    last_used_at TEXT,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE INDEX IF NOT EXISTS idx_contact_points_customer ON client_contact_points(customer_id);

-- ============================================================================
-- 9. CLIENT ORDER STAGE & PIPELINE
-- ============================================================================

-- Current order/sales stage for each client
CREATE TABLE IF NOT EXISTS client_order_stage (
    id TEXT PRIMARY KEY,
    customer_id TEXT NOT NULL REFERENCES customers(id) ON DELETE CASCADE,
    stage_name TEXT NOT NULL,  -- 'lead', 'qualified', 'proposal', 'negotiation', 'won', 'lost'
    stage_value INTEGER NOT NULL DEFAULT 0,
    probability REAL DEFAULT 0,
    expected_close_date TEXT,
    current_order_id TEXT REFERENCES client_orders(id),
    notes TEXT DEFAULT '',
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE INDEX IF NOT EXISTS idx_order_stage_customer ON client_order_stage(customer_id);

-- ============================================================================
-- 10. TASK CHECKLISTS
-- ============================================================================

-- Relevant task checklists for each client
CREATE TABLE IF NOT EXISTS client_task_checklists (
    id TEXT PRIMARY KEY,
    customer_id TEXT NOT NULL REFERENCES customers(id) ON DELETE CASCADE,
    checklist_name TEXT NOT NULL,
    checklist_type TEXT NOT NULL,  -- 'onboarding', 'order', 'support', 'custom'
    is_active INTEGER NOT NULL DEFAULT 1,
    created_by TEXT DEFAULT '',
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE INDEX IF NOT EXISTS idx_task_checklists_customer ON client_task_checklists(customer_id);

CREATE TABLE IF NOT EXISTS client_task_items (
    id TEXT PRIMARY KEY,
    checklist_id TEXT NOT NULL REFERENCES client_task_checklists(id) ON DELETE CASCADE,
    task_name TEXT NOT NULL,
    task_description TEXT DEFAULT '',
    is_completed INTEGER NOT NULL DEFAULT 0,
    completed_at TEXT,
    completed_by TEXT DEFAULT '',
    due_date TEXT,
    priority TEXT NOT NULL DEFAULT 'medium',  -- 'low', 'medium', 'high', 'urgent'
    display_order INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE INDEX IF NOT EXISTS idx_task_items_checklist ON client_task_items(checklist_id);

-- ============================================================================
-- 11. CLIENT HISTORY & INTERACTIONS
-- ============================================================================

-- Complete interaction history timeline
CREATE TABLE IF NOT EXISTS client_interaction_history (
    id TEXT PRIMARY KEY,
    customer_id TEXT NOT NULL REFERENCES customers(id) ON DELETE CASCADE,
    interaction_type TEXT NOT NULL,  -- 'message', 'call', 'email', 'meeting', 'ticket', 'order'
    channel TEXT DEFAULT '',
    direction TEXT DEFAULT 'inbound',  -- 'inbound', 'outbound'
    summary TEXT DEFAULT '',
    sentiment TEXT DEFAULT '',  -- 'positive', 'neutral', 'negative'
    duration_seconds INTEGER DEFAULT 0,
    interaction_date TEXT NOT NULL,
    metadata TEXT DEFAULT '{}',
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE INDEX IF NOT EXISTS idx_interaction_history_customer ON client_interaction_history(customer_id);
CREATE INDEX IF NOT EXISTS idx_interaction_history_date ON client_interaction_history(interaction_date);

-- ============================================================================
-- 12. CUSTOM CLIENT PROFILE SECTIONS
-- ============================================================================

-- User-defined custom sections for client profiles
CREATE TABLE IF NOT EXISTS client_custom_sections (
    id TEXT PRIMARY KEY,
    customer_id TEXT NOT NULL REFERENCES customers(id) ON DELETE CASCADE,
    section_name TEXT NOT NULL,
    section_type TEXT NOT NULL,  -- 'table', 'list', 'paragraph', 'key_value', 'rich_text', 'timeline'
    section_source TEXT NOT NULL DEFAULT 'user',  -- 'user', 'ai_learning', 'system'
    display_order INTEGER NOT NULL DEFAULT 0,
    is_active INTEGER NOT NULL DEFAULT 1,
    created_by TEXT DEFAULT '',
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE INDEX IF NOT EXISTS idx_client_custom_sections_customer ON client_custom_sections(customer_id);

CREATE TABLE IF NOT EXISTS client_custom_section_data (
    id TEXT PRIMARY KEY,
    section_id TEXT NOT NULL REFERENCES client_custom_sections(id) ON DELETE CASCADE,
    row_key TEXT DEFAULT '',
    field_key TEXT NOT NULL,
    field_value TEXT NOT NULL,
    field_type TEXT NOT NULL DEFAULT 'text',
    display_order INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE INDEX IF NOT EXISTS idx_client_section_data_section ON client_custom_section_data(section_id);

-- ============================================================================
-- 13. RAG INTEGRATION
-- ============================================================================

-- Link client profile sections to RAG for AI agent context
CREATE TABLE IF NOT EXISTS client_rag_mapping (
    id TEXT PRIMARY KEY,
    customer_id TEXT NOT NULL REFERENCES customers(id) ON DELETE CASCADE,
    section_type TEXT NOT NULL,  -- 'preferences', 'insights', 'orders', 'custom_section'
    section_id TEXT DEFAULT '',
    is_indexed INTEGER NOT NULL DEFAULT 1,
    embedding_status TEXT NOT NULL DEFAULT 'pending',  -- 'pending', 'indexed', 'failed'
    last_indexed_at TEXT,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE INDEX IF NOT EXISTS idx_client_rag_mapping_customer ON client_rag_mapping(customer_id);

-- ============================================================================
-- 14. BACKWARD COMPATIBILITY
-- ============================================================================

-- Set customer_id for existing customers if not set
UPDATE customers SET customer_id = id WHERE customer_id IS NULL;

-- Create unified customer ID mapping for existing contacts
INSERT OR IGNORE INTO customer_channel_ids (id, customer_id, channel_type, channel_identifier, is_primary)
SELECT 
    'cc_' || c.id,
    c.customer_id,
    c.channel,
    c.value,
    1
FROM contacts c
WHERE c.customer_id IS NOT NULL;
