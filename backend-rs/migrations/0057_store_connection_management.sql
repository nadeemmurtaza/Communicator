-- Phase 57: Platform-Agnostic Store Connection Management
--
-- Replaces the Shopify-specific integration with a universal system that works
-- with any e-commerce platform: Shopify, Wix, WordPress/WooCommerce, Square,
-- Magento, BigCommerce, custom stores, or any other website/store platform.
--
-- Key features:
-- 1. Platform-agnostic store connections — one row per store/website connected to a brand
-- 2. Connection status monitoring — heartbeat, health checks, error tracking
-- 3. UI compatibility checks — verifies the store frontend has correct OTP UI
--    (input boxes, paste support, accessibility)
-- 4. Brand OTP isolation — each brand has its own isolated OTP engine
--    with no mixing between brands, even within the same organization

-- ---------------------------------------------------------------------------
-- 1. Store connections: platform-agnostic link between a brand and its store
-- ---------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS store_connections (
    id TEXT PRIMARY KEY DEFAULT (gen_random_uuid()::TEXT),
    brand_id TEXT NOT NULL,
    organization_id TEXT,                          -- brand's organization for isolation checks
    platform_name TEXT NOT NULL,                   -- 'shopify', 'wix', 'wordpress', 'woocommerce', 'square', 'magento', 'bigcommerce', 'custom', 'other'
    platform_version TEXT DEFAULT '',              -- e.g., 'Shopify 2.0', 'WooCommerce 8.5'
    store_url TEXT NOT NULL,                       -- full URL of the store, e.g., 'https://mystore.myshopify.com'
    store_domain TEXT NOT NULL,                    -- domain only, e.g., 'mystore.myshopify.com'
    store_name TEXT NOT NULL DEFAULT '',           -- human-readable name
    display_name TEXT DEFAULT '',                  -- name shown in admin UI
    
    -- Platform-specific credentials (hashed where applicable)
    api_key TEXT DEFAULT '',                       -- platform API key
    api_secret_hash TEXT DEFAULT '',               -- HMAC-SHA256 hash of API secret
    api_secret_salt TEXT DEFAULT '',
    access_token_hash TEXT DEFAULT '',             -- hashed access token for Admin API
    access_token_salt TEXT DEFAULT '',
    webhook_secret TEXT DEFAULT '',                -- shared secret for webhook signature verification
    
    -- Integration linkage
    external_integration_id TEXT,                  -- links to otp_external_integrations if using that layer
    public_key TEXT NOT NULL,                      -- safe to expose in frontend JS
    webhook_url TEXT,                              -- URL for platform to send webhooks to
    
    -- OTP configuration for this store
    default_purpose TEXT NOT NULL DEFAULT 'sign_up', -- 'sign_up', 'sign_in', 'password_reset', 'order_confirmation'
    auto_create_customer BOOLEAN NOT NULL DEFAULT true,
    customer_tag TEXT DEFAULT 'otp_verified',
    
    -- Channel preferences for this store (override brand defaults)
    whatsapp_enabled BOOLEAN NOT NULL DEFAULT true,
    email_enabled BOOLEAN NOT NULL DEFAULT true,
    sms_enabled BOOLEAN NOT NULL DEFAULT false,
    in_app_enabled BOOLEAN NOT NULL DEFAULT false,
    default_channel TEXT NOT NULL DEFAULT 'whatsapp',
    priority_order TEXT[] NOT NULL DEFAULT ARRAY['whatsapp', 'email', 'sms'],
    
    -- Status
    is_active BOOLEAN NOT NULL DEFAULT true,
    connection_status TEXT NOT NULL DEFAULT 'pending',  -- 'pending', 'connected', 'disconnected', 'error', 'ui_incompatible'
    last_status_check TIMESTAMPTZ,
    last_error TEXT DEFAULT '',
    
    -- Metadata
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_store_conn_domain ON store_connections(store_domain);
CREATE INDEX IF NOT EXISTS idx_store_conn_brand ON store_connections(brand_id);
CREATE INDEX IF NOT EXISTS idx_store_conn_platform ON store_connections(platform_name);
CREATE INDEX IF NOT EXISTS idx_store_conn_status ON store_connections(connection_status);
CREATE INDEX IF NOT EXISTS idx_store_conn_org ON store_connections(organization_id);

COMMENT ON TABLE store_connections IS 'Platform-agnostic connection between a brand and its e-commerce store/website. Supports Shopify, Wix, WordPress, WooCommerce, Square, Magento, BigCommerce, custom stores, and any other platform.';

-- ---------------------------------------------------------------------------
-- 2. Connection status: heartbeat and health monitoring
-- ---------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS store_connection_status (
    id TEXT PRIMARY KEY DEFAULT (gen_random_uuid()::TEXT),
    store_connection_id TEXT NOT NULL REFERENCES store_connections(id) ON DELETE CASCADE,
    brand_id TEXT NOT NULL,
    
    -- Health metrics
    heartbeat_status TEXT NOT NULL DEFAULT 'unknown',  -- 'healthy', 'degraded', 'unhealthy', 'unknown'
    last_heartbeat_at TIMESTAMPTZ,
    response_time_ms INTEGER DEFAULT 0,
    uptime_percentage DOUBLE PRECISION DEFAULT 0,         -- e.g., 99.95
    
    -- API connectivity
    api_reachable BOOLEAN NOT NULL DEFAULT false,
    last_api_check_at TIMESTAMPTZ,
    api_version TEXT DEFAULT '',
    api_rate_limit_remaining INTEGER DEFAULT 0,
    api_rate_limit_reset_at TIMESTAMPTZ,
    
    -- Webhook connectivity
    webhook_reachable BOOLEAN NOT NULL DEFAULT false,
    last_webhook_check_at TIMESTAMPTZ,
    webhook_events_received INTEGER NOT NULL DEFAULT 0,
    last_webhook_event_at TIMESTAMPTZ,
    
    -- Error tracking
    error_count_24h INTEGER NOT NULL DEFAULT 0,
    last_error_at TIMESTAMPTZ,
    last_error_message TEXT DEFAULT '',
    consecutive_errors INTEGER NOT NULL DEFAULT 0,
    
    -- Checks
    last_full_check_at TIMESTAMPTZ,
    next_check_at TIMESTAMPTZ,
    check_interval_minutes INTEGER NOT NULL DEFAULT 30,
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_store_conn_status_conn ON store_connection_status(store_connection_id);
CREATE INDEX IF NOT EXISTS idx_store_conn_status_brand ON store_connection_status(brand_id);
CREATE INDEX IF NOT EXISTS idx_store_conn_status_health ON store_connection_status(heartbeat_status);

COMMENT ON TABLE store_connection_status IS 'Tracks the health and connectivity status of store connections. Monitors API reachability, webhook delivery, and error rates.';

-- ---------------------------------------------------------------------------
-- 3. UI compatibility: checks if the store frontend has correct OTP UI
-- ---------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS store_ui_compatibility (
    id TEXT PRIMARY KEY DEFAULT (gen_random_uuid()::TEXT),
    store_connection_id TEXT NOT NULL REFERENCES store_connections(id) ON DELETE CASCADE,
    brand_id TEXT NOT NULL,
    
    -- Check results
    check_status TEXT NOT NULL DEFAULT 'pending',  -- 'pending', 'passed', 'failed', 'partial'
    check_score INTEGER NOT NULL DEFAULT 0,         -- 0-100 score
    checked_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- Input field checks
    input_boxes_count INTEGER DEFAULT 0,            -- number of OTP input boxes found
    expected_boxes_count INTEGER NOT NULL DEFAULT 12, -- matches OTP code length
    input_boxes_match BOOLEAN NOT NULL DEFAULT false,
    input_boxes_error TEXT DEFAULT '',
    
    -- Paste functionality
    paste_supported BOOLEAN NOT NULL DEFAULT false,
    paste_event_handler_found BOOLEAN NOT NULL DEFAULT false,
    paste_error TEXT DEFAULT '',
    
    -- Individual input paste (for split input boxes)
    individual_input_paste BOOLEAN NOT NULL DEFAULT false,
    individual_input_paste_error TEXT DEFAULT '',
    
    -- Accessibility
    aria_labels_present BOOLEAN NOT NULL DEFAULT false,
    aria_labels_error TEXT DEFAULT '',
    keyboard_navigation BOOLEAN NOT NULL DEFAULT false,
    keyboard_navigation_error TEXT DEFAULT '',
    screen_reader_compatible BOOLEAN NOT NULL DEFAULT false,
    
    -- Form validation
    form_validation_present BOOLEAN NOT NULL DEFAULT false,
    form_validation_error TEXT DEFAULT '',
    error_messages_visible BOOLEAN NOT NULL DEFAULT false,
    
    -- Timer and resend
    countdown_timer_present BOOLEAN NOT NULL DEFAULT false,
    resend_button_present BOOLEAN NOT NULL DEFAULT false,
    resend_cooldown_enforced BOOLEAN NOT NULL DEFAULT false,
    
    -- Security
    https_enforced BOOLEAN NOT NULL DEFAULT false,
    csrf_protection_present BOOLEAN NOT NULL DEFAULT false,
    rate_limit_ui_feedback BOOLEAN NOT NULL DEFAULT false,
    
    -- Mobile compatibility
    mobile_responsive BOOLEAN NOT NULL DEFAULT false,
    mobile_touch_targets_ok BOOLEAN NOT NULL DEFAULT false,
    
    -- Overall
    overall_compatible BOOLEAN NOT NULL DEFAULT false,
    issues_found TEXT[] NOT NULL DEFAULT ARRAY[]::TEXT[],
    recommendations TEXT[] NOT NULL DEFAULT ARRAY[]::TEXT[],
    
    -- Metadata
    check_method TEXT DEFAULT '',                   -- 'auto', 'manual', 'widget_report'
    user_agent TEXT DEFAULT '',                     -- browser user agent used for check
    page_url TEXT DEFAULT '',                       -- URL that was checked
    screenshot_url TEXT DEFAULT '',                 -- URL to screenshot if available
    metadata JSONB DEFAULT '{}',
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_ui_compat_store ON store_ui_compatibility(store_connection_id);
CREATE INDEX IF NOT EXISTS idx_ui_compat_brand ON store_ui_compatibility(brand_id);
CREATE INDEX IF NOT EXISTS idx_ui_compat_status ON store_ui_compatibility(check_status);
CREATE INDEX IF NOT EXISTS idx_ui_compat_checked ON store_ui_compatibility(checked_at);

COMMENT ON TABLE store_ui_compatibility IS 'Stores the results of UI compatibility checks for store frontends. Verifies that the OTP input UI has the correct number of input boxes, paste support, accessibility, and other requirements.';

-- ---------------------------------------------------------------------------
-- 4. Brand OTP isolation: ensures each brand has its own isolated OTP engine
-- ---------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS brand_otp_isolation (
    id TEXT PRIMARY KEY DEFAULT (gen_random_uuid()::TEXT),
    brand_id TEXT NOT NULL UNIQUE,
    organization_id TEXT,
    
    -- Isolation settings
    isolation_enabled BOOLEAN NOT NULL DEFAULT true,
    isolation_mode TEXT NOT NULL DEFAULT 'strict',  -- 'strict', 'relaxed', 'disabled'
    
    -- Separate OTP configuration per brand
    otp_prefix TEXT NOT NULL,                       -- unique prefix for this brand's OTPs
    otp_namespace TEXT NOT NULL,                    -- namespace for rate limiting isolation
    
    -- Channel isolation
    isolated_whatsapp BOOLEAN NOT NULL DEFAULT true,  -- brand uses its own WhatsApp credentials
    isolated_email BOOLEAN NOT NULL DEFAULT true,     -- brand uses its own SMTP settings
    isolated_sms BOOLEAN NOT NULL DEFAULT true,       -- brand uses its own SMS credentials
    
    -- Rate limit isolation
    rate_limit_per_minute INTEGER NOT NULL DEFAULT 3,
    rate_limit_per_hour INTEGER NOT NULL DEFAULT 10,
    cooldown_seconds INTEGER NOT NULL DEFAULT 60,
    lockout_seconds INTEGER NOT NULL DEFAULT 900,
    
    -- Cross-brand protection
    block_cross_brand_otp BOOLEAN NOT NULL DEFAULT true,  -- prevent OTP from one brand verifying on another
    block_shared_recipient BOOLEAN NOT NULL DEFAULT false, -- allow blocking same recipient across brands
    
    -- Monitoring
    total_otp_generated INTEGER NOT NULL DEFAULT 0,
    total_otp_verified INTEGER NOT NULL DEFAULT 0,
    total_otp_failed INTEGER NOT NULL DEFAULT 0,
    last_otp_generated_at TIMESTAMPTZ,
    
    -- Metadata
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_brand_otp_isolation_brand ON brand_otp_isolation(brand_id);
CREATE INDEX IF NOT EXISTS idx_brand_otp_isolation_org ON brand_otp_isolation(organization_id);

COMMENT ON TABLE brand_otp_isolation IS 'Ensures each brand has a completely isolated OTP engine. No mixing of OTPs, rate limits, or credentials between brands, even within the same organization.';

-- ---------------------------------------------------------------------------
-- 5. Store pending customers: platform-agnostic pending account sessions
-- ---------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS store_pending_customers (
    id TEXT PRIMARY KEY DEFAULT (gen_random_uuid()::TEXT),
    store_connection_id TEXT NOT NULL REFERENCES store_connections(id) ON DELETE CASCADE,
    brand_id TEXT NOT NULL,
    otp_id TEXT NOT NULL,                          -- links to otp_records.id
    email TEXT NOT NULL,
    phone TEXT DEFAULT '',
    first_name TEXT DEFAULT '',
    last_name TEXT DEFAULT '',
    account_status TEXT NOT NULL DEFAULT 'otp_pending',  -- 'otp_pending', 'verified', 'failed', 'expired'
    platform_customer_id TEXT,                     -- set after customer is created on the platform
    verification_attempts INTEGER NOT NULL DEFAULT 0,
    expires_at TIMESTAMPTZ NOT NULL,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    verified_at TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_store_pending_store ON store_pending_customers(store_connection_id);
CREATE INDEX IF NOT EXISTS idx_store_pending_otp ON store_pending_customers(otp_id);
CREATE INDEX IF NOT EXISTS idx_store_pending_email ON store_pending_customers(email);
CREATE INDEX IF NOT EXISTS idx_store_pending_status ON store_pending_customers(account_status);
CREATE INDEX IF NOT EXISTS idx_store_pending_brand ON store_pending_customers(brand_id);

COMMENT ON TABLE store_pending_customers IS 'Platform-agnostic pending customer records. Holds customer data during OTP verification until the account is finalized on the connected store platform.';

-- ---------------------------------------------------------------------------
-- 6. Store webhook events: incoming webhooks from any platform
-- ---------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS store_webhook_events (
    id TEXT PRIMARY KEY DEFAULT (gen_random_uuid()::TEXT),
    store_connection_id TEXT REFERENCES store_connections(id) ON DELETE SET NULL,
    brand_id TEXT,
    platform_name TEXT NOT NULL,
    topic TEXT NOT NULL,                           -- e.g., 'customers/create', 'orders/create'
    store_domain TEXT NOT NULL,
    webhook_id TEXT NOT NULL,
    payload JSONB NOT NULL DEFAULT '{}',
    signature TEXT DEFAULT '',
    verified BOOLEAN NOT NULL DEFAULT false,
    processed BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_store_webhook_store ON store_webhook_events(store_connection_id);
CREATE INDEX IF NOT EXISTS idx_store_webhook_topic ON store_webhook_events(topic);
CREATE INDEX IF NOT EXISTS idx_store_webhook_processed ON store_webhook_events(processed);
CREATE INDEX IF NOT EXISTS idx_store_webhook_brand ON store_webhook_events(brand_id);

COMMENT ON TABLE store_webhook_events IS 'Logs incoming webhook events from any connected store platform for audit and processing.';
