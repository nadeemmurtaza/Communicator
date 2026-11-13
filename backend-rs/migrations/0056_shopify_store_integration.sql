-- Phase 56: Shopify Store OTP Integration
--
-- Connects a brand's Shopify store (the actual storefront built on Shopify)
-- to the Communicator OTP engine. When a customer creates an account on the
-- Shopify store, the OTP is generated and sent via the brand admin's preferred
-- channels (WhatsApp, SMS, Email). The customer account remains pending until
-- OTP is verified.
--
-- Flow:
-- 1. Customer fills account creation form on Shopify store
-- 2. Store calls Communicator to generate OTP with customer details
-- 3. Communicator sends OTP via brand-preferred channel (WhatsApp/SMS/Email)
-- 4. Customer enters OTP on the store's verification page
-- 5. Store calls Communicator to verify OTP
-- 6. On success, store completes the Shopify customer account creation
--    via Shopify Admin API or storefront custom logic

-- Shopify store connection: one row per Shopify store connected to a brand
CREATE TABLE IF NOT EXISTS shopify_store_connections (
    id TEXT PRIMARY KEY DEFAULT (gen_random_uuid()::TEXT),
    brand_id TEXT NOT NULL,
    store_domain TEXT NOT NULL,                  -- e.g., 'mystore.myshopify.com'
    store_name TEXT NOT NULL DEFAULT '',          -- human-readable store name
    shopify_api_key TEXT NOT NULL DEFAULT '',     -- Shopify custom app API key
    shopify_api_secret_hash TEXT NOT NULL DEFAULT '', -- HMAC-SHA256 hash of the API secret
    shopify_api_secret_salt TEXT NOT NULL DEFAULT '',
    shopify_access_token_hash TEXT DEFAULT '',    -- hashed access token for Admin API
    shopify_access_token_salt TEXT DEFAULT '',
    shopify_webhook_secret TEXT DEFAULT '',       -- shared secret for Shopify webhook verification
    integration_id TEXT,                          -- links to otp_external_integrations if using that layer
    default_purpose TEXT NOT NULL DEFAULT 'sign_up',
    auto_create_customer BOOLEAN NOT NULL DEFAULT true,  -- auto-create Shopify customer after OTP verify
    customer_tag TEXT DEFAULT 'otp_verified',     -- tag applied to verified customers in Shopify
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_shopify_conn_domain ON shopify_store_connections(store_domain);
CREATE INDEX IF NOT EXISTS idx_shopify_conn_brand ON shopify_store_connections(brand_id);

COMMENT ON TABLE shopify_store_connections IS 'Stores the connection between a Shopify storefront and the Communicator OTP engine.';
COMMENT ON COLUMN shopify_store_connections.auto_create_customer IS 'If true, automatically creates the customer in Shopify Admin API after OTP verification.';

-- Pending OTP sessions for Shopify store account creation
-- Tracks the customer data until OTP is verified, then the account is finalized
CREATE TABLE IF NOT EXISTS shopify_pending_customers (
    id TEXT PRIMARY KEY DEFAULT (gen_random_uuid()::TEXT),
    store_connection_id TEXT NOT NULL REFERENCES shopify_store_connections(id) ON DELETE CASCADE,
    brand_id TEXT NOT NULL,
    otp_id TEXT NOT NULL,                        -- links to otp_records.id
    email TEXT NOT NULL,
    phone TEXT DEFAULT '',
    first_name TEXT DEFAULT '',
    last_name TEXT DEFAULT '',
    account_status TEXT NOT NULL DEFAULT 'otp_pending',  -- 'otp_pending', 'verified', 'failed', 'expired'
    shopify_customer_id TEXT,                    -- set after customer is created in Shopify
    verification_attempts INTEGER NOT NULL DEFAULT 0,
    expires_at TIMESTAMPTZ NOT NULL,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    verified_at TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_shopify_pending_store ON shopify_pending_customers(store_connection_id);
CREATE INDEX IF NOT EXISTS idx_shopify_pending_otp ON shopify_pending_customers(otp_id);
CREATE INDEX IF NOT EXISTS idx_shopify_pending_email ON shopify_pending_customers(email);
CREATE INDEX IF NOT EXISTS idx_shopify_pending_status ON shopify_pending_customers(account_status);

COMMENT ON TABLE shopify_pending_customers IS 'Holds customer data during OTP verification. After verification, the account is created in Shopify and this row is marked verified.';

-- Shopify webhook events received from Shopify stores
CREATE TABLE IF NOT EXISTS shopify_webhook_events (
    id TEXT PRIMARY KEY DEFAULT (gen_random_uuid()::TEXT),
    store_connection_id TEXT REFERENCES shopify_store_connections(id) ON DELETE SET NULL,
    topic TEXT NOT NULL,                         -- e.g., 'customers/create', 'orders/create'
    shop_domain TEXT NOT NULL,
    webhook_id TEXT NOT NULL,                    -- Shopify's webhook ID
    payload JSONB NOT NULL DEFAULT '{}',
    signature TEXT DEFAULT '',                   -- Shopify's HMAC signature
    verified BOOLEAN NOT NULL DEFAULT false,
    processed BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_shopify_webhook_store ON shopify_webhook_events(store_connection_id);
CREATE INDEX IF NOT EXISTS idx_shopify_webhook_topic ON shopify_webhook_events(topic);
CREATE INDEX IF NOT EXISTS idx_shopify_webhook_processed ON shopify_webhook_events(processed);

COMMENT ON TABLE shopify_webhook_events IS 'Logs incoming webhook events from Shopify stores for audit and processing.';

-- Channel preference override per store connection
-- Allows per-store channel preferences that override the brand default
CREATE TABLE IF NOT EXISTS shopify_channel_preferences (
    id TEXT PRIMARY KEY DEFAULT (gen_random_uuid()::TEXT),
    store_connection_id TEXT NOT NULL REFERENCES shopify_store_connections(id) ON DELETE CASCADE,
    purpose TEXT NOT NULL DEFAULT 'sign_up',     -- 'sign_up', 'sign_in', 'password_reset', etc.
    whatsapp_enabled BOOLEAN NOT NULL DEFAULT true,
    email_enabled BOOLEAN NOT NULL DEFAULT true,
    sms_enabled BOOLEAN NOT NULL DEFAULT false,
    in_app_enabled BOOLEAN NOT NULL DEFAULT false,
    default_channel TEXT NOT NULL DEFAULT 'whatsapp',  -- 'whatsapp', 'email', 'sms', 'in_app'
    priority_order TEXT[] NOT NULL DEFAULT ARRAY['whatsapp', 'email', 'sms'],
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (store_connection_id, purpose)
);

CREATE INDEX IF NOT EXISTS idx_shopify_channel_pref_store ON shopify_channel_preferences(store_connection_id);

COMMENT ON TABLE shopify_channel_preferences IS 'Per-store channel preferences for OTP delivery. Overrides brand-level defaults for this specific store connection.';
