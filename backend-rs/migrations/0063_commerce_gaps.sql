-- Gap #1: Product Options/Add-ons (e.g., "Lemon butter +$2.00")
CREATE TABLE IF NOT EXISTS product_options (
    id TEXT PRIMARY KEY,
    product_id TEXT NOT NULL,
    name TEXT NOT NULL,
    description TEXT,
    price_adjustment REAL NOT NULL DEFAULT 0,
    required BOOLEAN DEFAULT false,
    max_selections BOOLEAN DEFAULT true,
    sort_order INTEGER DEFAULT 0,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    FOREIGN KEY (product_id) REFERENCES products(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS product_option_values (
    id TEXT PRIMARY KEY,
    option_id TEXT NOT NULL,
    name TEXT NOT NULL,
    price_adjustment REAL NOT NULL DEFAULT 0,
    sku TEXT,
    sort_order INTEGER DEFAULT 0,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    FOREIGN KEY (option_id) REFERENCES product_options(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_product_options_product ON product_options(product_id);
CREATE INDEX IF NOT EXISTS idx_product_option_values_option ON product_option_values(option_id);

-- Gap #2: Delivery Fee Calculator
CREATE TABLE IF NOT EXISTS delivery_fee_rules (
    id TEXT PRIMARY KEY,
    brand_id TEXT NOT NULL,
    name TEXT NOT NULL,
    type TEXT NOT NULL DEFAULT 'flat',
    base_fee REAL NOT NULL DEFAULT 0,
    per_km_fee REAL DEFAULT 0,
    per_gram_fee REAL DEFAULT 0,
    max_distance REAL,
    free_delivery_threshold REAL,
    min_order_value REAL,
    postal_codes TEXT,
    regions TEXT,
    is_active BOOLEAN DEFAULT true,
    sort_order INTEGER DEFAULT 0,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    FOREIGN KEY (brand_id) REFERENCES brands(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_delivery_fee_rules_brand ON delivery_fee_rules(brand_id);

-- Gap #3: Payment Proof Upload
CREATE TABLE IF NOT EXISTS payment_proofs (
    id TEXT PRIMARY KEY,
    order_id TEXT NOT NULL,
    customer_id TEXT NOT NULL,
    file_url TEXT NOT NULL,
    file_name TEXT,
    file_size INTEGER,
    status TEXT DEFAULT 'pending',
    merchant_notes TEXT,
    verified_at TIMESTAMPTZ,
    verified_by TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    FOREIGN KEY (order_id) REFERENCES commerce_orders(id) ON DELETE CASCADE,
    FOREIGN KEY (customer_id) REFERENCES customers(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_payment_proofs_order ON payment_proofs(order_id);
CREATE INDEX IF NOT EXISTS idx_payment_proofs_status ON payment_proofs(status);

-- Gap #4: Order Notifications (WhatsApp automation)
CREATE TABLE IF NOT EXISTS order_notification_templates (
    id TEXT PRIMARY KEY,
    brand_id TEXT NOT NULL,
    channel_type TEXT NOT NULL,
    trigger_event TEXT NOT NULL,
    template_name TEXT NOT NULL,
    template_body TEXT NOT NULL,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    FOREIGN KEY (brand_id) REFERENCES brands(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS order_notification_log (
    id TEXT PRIMARY KEY,
    order_id TEXT NOT NULL,
    customer_id TEXT NOT NULL,
    channel_type TEXT NOT NULL,
    trigger_event TEXT NOT NULL,
    message_id TEXT,
    status TEXT DEFAULT 'sent',
    sent_at TIMESTAMPTZ DEFAULT NOW(),
    FOREIGN KEY (order_id) REFERENCES commerce_orders(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_order_notif_templates_brand ON order_notification_templates(brand_id);
CREATE INDEX IF NOT EXISTS idx_order_notif_log_order ON order_notification_log(order_id);

-- Gap #5: Public Storefront (frontend only, backend exists)
-- No new tables needed - storefronts table already exists in 0013_storefront.sql

-- Gap #6: Local Payment Methods
CREATE TABLE IF NOT EXISTS payment_gateways (
    id TEXT PRIMARY KEY,
    brand_id TEXT NOT NULL,
    name TEXT NOT NULL,
    provider TEXT NOT NULL,
    credentials TEXT,
    currency TEXT DEFAULT 'USD',
    is_active BOOLEAN DEFAULT true,
    sort_order INTEGER DEFAULT 0,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    FOREIGN KEY (brand_id) REFERENCES brands(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_payment_gateways_brand ON payment_gateways(brand_id);

-- Gap #7: Delivery/Pickup Selection
CREATE TABLE IF NOT EXISTS service_types (
    id TEXT PRIMARY KEY,
    brand_id TEXT NOT NULL,
    name TEXT NOT NULL,
    type TEXT NOT NULL,
    description TEXT,
    is_active BOOLEAN DEFAULT true,
    sort_order INTEGER DEFAULT 0,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    FOREIGN KEY (brand_id) REFERENCES brands(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_service_types_brand ON service_types(brand_id);

-- Gap #8: Delivery Date & Time Slot
CREATE TABLE IF NOT EXISTS delivery_schedules (
    id TEXT PRIMARY KEY,
    brand_id TEXT NOT NULL,
    day_of_week INTEGER NOT NULL,
    start_time TEXT NOT NULL,
    end_time TEXT NOT NULL,
    cutoff_time TEXT,
    max_capacity INTEGER,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    FOREIGN KEY (brand_id) REFERENCES brands(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS delivery_time_slots (
    id TEXT PRIMARY KEY,
    brand_id TEXT NOT NULL,
    day_of_week INTEGER NOT NULL,
    slot_start TEXT NOT NULL,
    slot_end TEXT NOT NULL,
    label TEXT,
    max_orders INTEGER DEFAULT 10,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    FOREIGN KEY (brand_id) REFERENCES brands(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_delivery_schedules_brand ON delivery_schedules(brand_id);
CREATE INDEX IF NOT EXISTS idx_delivery_time_slots_brand ON delivery_time_slots(brand_id);

-- Gap #9: Order Invoice Generation
CREATE TABLE IF NOT EXISTS invoices (
    id TEXT PRIMARY KEY,
    order_id TEXT NOT NULL,
    invoice_number TEXT NOT NULL,
    pdf_url TEXT,
    issued_at TIMESTAMPTZ DEFAULT (NOW()),
    due_date TEXT,
    status TEXT DEFAULT 'issued',
    created_at TIMESTAMPTZ DEFAULT NOW(),
    FOREIGN KEY (order_id) REFERENCES commerce_orders(id) ON DELETE CASCADE,
    UNIQUE(order_id)
);

CREATE INDEX IF NOT EXISTS idx_invoices_order ON invoices(order_id);

-- Gap #10: CSV Import/Export
CREATE TABLE IF NOT EXISTS csv_import_jobs (
    id TEXT PRIMARY KEY,
    brand_id TEXT NOT NULL,
    entity_type TEXT NOT NULL,
    file_url TEXT NOT NULL,
    status TEXT DEFAULT 'processing',
    total_rows BOOLEAN DEFAULT false,
    processed_rows BOOLEAN DEFAULT false,
    error_count INTEGER DEFAULT 0,
    errors TEXT,
    completed_at TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    FOREIGN KEY (brand_id) REFERENCES brands(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_csv_import_jobs_brand ON csv_import_jobs(brand_id);

-- Gap #11: Customer Login
CREATE TABLE IF NOT EXISTS customer_accounts (
    id TEXT PRIMARY KEY,
    customer_id TEXT NOT NULL UNIQUE,
    email TEXT,
    phone TEXT,
    password_hash TEXT,
    is_verified BOOLEAN DEFAULT false,
    last_login_at TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    FOREIGN KEY (customer_id) REFERENCES customers(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_customer_accounts_email ON customer_accounts(email);
CREATE INDEX IF NOT EXISTS idx_customer_accounts_phone ON customer_accounts(phone);

-- Gap #12: Customer Reviews
CREATE TABLE IF NOT EXISTS customer_reviews (
    id TEXT PRIMARY KEY,
    order_id TEXT NOT NULL,
    customer_id TEXT NOT NULL,
    product_id TEXT,
    rating INTEGER NOT NULL,
    comment TEXT,
    is_public BOOLEAN DEFAULT true,
    status TEXT DEFAULT 'pending',
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    FOREIGN KEY (order_id) REFERENCES commerce_orders(id) ON DELETE CASCADE,
    FOREIGN KEY (customer_id) REFERENCES customers(id) ON DELETE CASCADE,
    FOREIGN KEY (product_id) REFERENCES products(id) ON DELETE SET NULL
);

CREATE INDEX IF NOT EXISTS idx_customer_reviews_product ON customer_reviews(product_id);
CREATE INDEX IF NOT EXISTS idx_customer_reviews_order ON customer_reviews(order_id);

-- Gap #13: Loyalty/Rewards
CREATE TABLE IF NOT EXISTS loyalty_programs (
    id TEXT PRIMARY KEY,
    brand_id TEXT NOT NULL,
    name TEXT NOT NULL,
    points_per_currency REAL NOT NULL DEFAULT 1,
    points_expiry_days INTEGER DEFAULT 365,
    redemption_rate REAL DEFAULT 100,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    FOREIGN KEY (brand_id) REFERENCES brands(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS customer_points (
    id TEXT PRIMARY KEY,
    customer_id TEXT NOT NULL,
    brand_id TEXT NOT NULL,
    points_balance BOOLEAN NOT NULL DEFAULT false,
    points_earned BOOLEAN NOT NULL DEFAULT false,
    points_redeemed BOOLEAN NOT NULL DEFAULT false,
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    FOREIGN KEY (customer_id) REFERENCES customers(id) ON DELETE CASCADE,
    FOREIGN KEY (brand_id) REFERENCES brands(id) ON DELETE CASCADE,
    UNIQUE(customer_id, brand_id)
);

CREATE TABLE IF NOT EXISTS points_transactions (
    id TEXT PRIMARY KEY,
    customer_id TEXT NOT NULL,
    brand_id TEXT NOT NULL,
    points INTEGER NOT NULL,
    transaction_type TEXT NOT NULL,
    reference_type TEXT,
    reference_id TEXT,
    description TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    FOREIGN KEY (customer_id) REFERENCES customers(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_loyalty_programs_brand ON loyalty_programs(brand_id);
CREATE INDEX IF NOT EXISTS idx_customer_points_customer ON customer_points(customer_id);
CREATE INDEX IF NOT EXISTS idx_points_transactions_customer ON points_transactions(customer_id);

-- Gap #14: Payment Reminders
CREATE TABLE IF NOT EXISTS payment_reminders (
    id TEXT PRIMARY KEY,
    order_id TEXT NOT NULL,
    customer_id TEXT NOT NULL,
    channel_type TEXT NOT NULL,
    reminder_type TEXT NOT NULL,
    scheduled_at TEXT,
    sent_at TEXT,
    status TEXT DEFAULT 'pending',
    created_at TIMESTAMPTZ DEFAULT NOW(),
    FOREIGN KEY (order_id) REFERENCES commerce_orders(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_payment_reminders_order ON payment_reminders(order_id);

-- Gap #15: Delivery Reminders
CREATE TABLE IF NOT EXISTS delivery_reminders (
    id TEXT PRIMARY KEY,
    order_id TEXT NOT NULL,
    customer_id TEXT NOT NULL,
    channel_type TEXT NOT NULL,
    reminder_type TEXT NOT NULL,
    scheduled_at TEXT,
    sent_at TEXT,
    status TEXT DEFAULT 'pending',
    created_at TIMESTAMPTZ DEFAULT NOW(),
    FOREIGN KEY (order_id) REFERENCES commerce_orders(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_delivery_reminders_order ON delivery_reminders(order_id);

-- Gap #16: Staff Management
CREATE TABLE IF NOT EXISTS staff_members (
    id TEXT PRIMARY KEY,
    brand_id TEXT NOT NULL,
    name TEXT NOT NULL,
    email TEXT NOT NULL,
    role TEXT NOT NULL DEFAULT 'viewer',
    permissions TEXT,
    is_active BOOLEAN DEFAULT true,
    last_login_at TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    FOREIGN KEY (brand_id) REFERENCES brands(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_staff_members_brand ON staff_members(brand_id);

-- Gap #17: POS System
CREATE TABLE IF NOT EXISTS pos_sessions (
    id TEXT PRIMARY KEY,
    brand_id TEXT NOT NULL,
    staff_id TEXT,
    session_token TEXT NOT NULL,
    status TEXT DEFAULT 'active',
    started_at TIMESTAMPTZ DEFAULT (NOW()),
    ended_at TEXT,
    FOREIGN KEY (brand_id) REFERENCES brands(id) ON DELETE CASCADE,
    FOREIGN KEY (staff_id) REFERENCES staff_members(id) ON DELETE SET NULL
);

CREATE INDEX IF NOT EXISTS idx_pos_sessions_brand ON pos_sessions(brand_id);

-- Gap #18: Receipt Generation
CREATE TABLE IF NOT EXISTS receipts (
    id TEXT PRIMARY KEY,
    order_id TEXT NOT NULL,
    receipt_number TEXT NOT NULL,
    pdf_url TEXT,
    sent_via TEXT,
    sent_at TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    FOREIGN KEY (order_id) REFERENCES commerce_orders(id) ON DELETE CASCADE,
    UNIQUE(order_id)
);

CREATE INDEX IF NOT EXISTS idx_receipts_order ON receipts(order_id);

-- Gap #19: WhatsApp/Instagram Inbox
-- Already partially covered by unified_inbox in 0027_unified_inbox.sql
-- Adding commerce-specific inbox features
CREATE TABLE IF NOT EXISTS commerce_inbox_threads (
    id TEXT PRIMARY KEY,
    brand_id TEXT NOT NULL,
    customer_id TEXT NOT NULL,
    channel_type TEXT NOT NULL,
    last_message_at TEXT,
    unread_count INTEGER DEFAULT 0,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    FOREIGN KEY (brand_id) REFERENCES brands(id) ON DELETE CASCADE,
    FOREIGN KEY (customer_id) REFERENCES customers(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_commerce_inbox_brand ON commerce_inbox_threads(brand_id);
CREATE INDEX IF NOT EXISTS idx_commerce_inbox_customer ON commerce_inbox_threads(customer_id);

-- Gap #20: Wholesale Pricing
CREATE TABLE IF NOT EXISTS wholesale_pricing (
    id TEXT PRIMARY KEY,
    product_id TEXT NOT NULL,
    customer_group TEXT NOT NULL,
    price REAL NOT NULL,
    min_quantity INTEGER DEFAULT 1,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    FOREIGN KEY (product_id) REFERENCES products(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_wholesale_pricing_product ON wholesale_pricing(product_id);

-- Gap #21: Processing Fee Pass-through
CREATE TABLE IF NOT EXISTS payment_fees (
    id TEXT PRIMARY KEY,
    order_id TEXT NOT NULL,
    gateway_id TEXT,
    fee_amount REAL NOT NULL DEFAULT 0,
    fee_percentage REAL DEFAULT 0,
    passed_to_customer BOOLEAN DEFAULT false,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    FOREIGN KEY (order_id) REFERENCES commerce_orders(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_payment_fees_order ON payment_fees(order_id);

-- Gap #22: Payment Recovery
CREATE TABLE IF NOT EXISTS payment_recovery (
    id TEXT PRIMARY KEY,
    order_id TEXT NOT NULL,
    customer_id TEXT NOT NULL,
    channel_type TEXT NOT NULL,
    attempt_count INTEGER DEFAULT 0,
    last_attempt_at TEXT,
    status TEXT DEFAULT 'pending',
    created_at TIMESTAMPTZ DEFAULT NOW(),
    FOREIGN KEY (order_id) REFERENCES commerce_orders(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_payment_recovery_order ON payment_recovery(order_id);

-- Gap #23: Automated Follow-ups (birthday, win-back)
CREATE TABLE IF NOT EXISTS automated_followups (
    id TEXT PRIMARY KEY,
    brand_id TEXT NOT NULL,
    name TEXT NOT NULL,
    trigger_type TEXT NOT NULL,
    trigger_days INTEGER DEFAULT 0,
    channel_type TEXT NOT NULL,
    template_body TEXT NOT NULL,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    FOREIGN KEY (brand_id) REFERENCES brands(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS followup_log (
    id TEXT PRIMARY KEY,
    followup_id TEXT NOT NULL,
    customer_id TEXT NOT NULL,
    channel_type TEXT NOT NULL,
    sent_at TEXT,
    status TEXT DEFAULT 'pending',
    created_at TIMESTAMPTZ DEFAULT NOW(),
    FOREIGN KEY (followup_id) REFERENCES automated_followups(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_automated_followups_brand ON automated_followups(brand_id);
CREATE INDEX IF NOT EXISTS idx_followup_log_customer ON followup_log(customer_id);

-- Gap #24: Custom Domain Support
CREATE TABLE IF NOT EXISTS custom_domains (
    id TEXT PRIMARY KEY,
    brand_id TEXT NOT NULL,
    domain TEXT NOT NULL,
    storefront_id TEXT,
    ssl_enabled BOOLEAN DEFAULT false,
    verified BOOLEAN DEFAULT false,
    dns_records TEXT,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    FOREIGN KEY (brand_id) REFERENCES brands(id) ON DELETE CASCADE,
    FOREIGN KEY (storefront_id) REFERENCES storefronts(id) ON DELETE SET NULL
);

CREATE INDEX IF NOT EXISTS idx_custom_domains_brand ON custom_domains(brand_id);

-- Gap #25: Custom Pages on Storefront
CREATE TABLE IF NOT EXISTS storefront_pages (
    id TEXT PRIMARY KEY,
    storefront_id TEXT NOT NULL,
    slug TEXT NOT NULL,
    title TEXT NOT NULL,
    content TEXT,
    sort_order INTEGER DEFAULT 0,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    FOREIGN KEY (storefront_id) REFERENCES storefronts(id) ON DELETE CASCADE,
    UNIQUE(storefront_id, slug)
);

CREATE INDEX IF NOT EXISTS idx_storefront_pages_storefront ON storefront_pages(storefront_id);

-- Gap #26: Remove Platform Logo
-- Stored in storefront theme JSON, no new table needed

-- Gap #27: Meta Pixel Tracking
CREATE TABLE IF NOT EXISTS meta_pixels (
    id TEXT PRIMARY KEY,
    brand_id TEXT NOT NULL,
    pixel_id TEXT NOT NULL,
    access_token TEXT,
    events TEXT,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    FOREIGN KEY (brand_id) REFERENCES brands(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_meta_pixels_brand ON meta_pixels(brand_id);

-- Gap #28: Service Type (dine-in/QR ordering)
-- Already covered by service_types table (Gap #7)

-- Gap #29: Extra Customer Questions at Checkout
CREATE TABLE IF NOT EXISTS checkout_questions (
    id TEXT PRIMARY KEY,
    brand_id TEXT NOT NULL,
    question TEXT NOT NULL,
    question_type TEXT NOT NULL DEFAULT 'text',
    options TEXT,
    is_required BOOLEAN DEFAULT false,
    sort_order INTEGER DEFAULT 0,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    FOREIGN KEY (brand_id) REFERENCES brands(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS checkout_question_answers (
    id TEXT PRIMARY KEY,
    order_id TEXT NOT NULL,
    question_id TEXT NOT NULL,
    answer TEXT NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    FOREIGN KEY (order_id) REFERENCES commerce_orders(id) ON DELETE CASCADE,
    FOREIGN KEY (question_id) REFERENCES checkout_questions(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_checkout_questions_brand ON checkout_questions(brand_id);
CREATE INDEX IF NOT EXISTS idx_checkout_question_answers_order ON checkout_question_answers(order_id);

-- Gap #30: Proof of Delivery Upload
CREATE TABLE IF NOT EXISTS delivery_proofs (
    id TEXT PRIMARY KEY,
    shipment_id TEXT NOT NULL,
    order_id TEXT NOT NULL,
    file_url TEXT NOT NULL,
    file_name TEXT,
    uploaded_by TEXT,
    
    uploaded_at TEXT DEFAULT (NOW()),
    FOREIGN KEY (shipment_id) REFERENCES shipments(id) ON DELETE CASCADE,
    FOREIGN KEY (order_id) REFERENCES commerce_orders(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_delivery_proofs_shipment ON delivery_proofs(shipment_id);
CREATE INDEX IF NOT EXISTS idx_delivery_proofs_order ON delivery_proofs(order_id);

-- Gap #31: Map Location/Pin Location
-- Adding location fields to commerce_orders metadata
-- No new table needed - stored in shipping_address JSON with lat/lng

-- Extend commerce_orders with delivery-related fields
ALTER TABLE commerce_orders ADD COLUMN service_type TEXT DEFAULT 'delivery';
ALTER TABLE commerce_orders ADD COLUMN delivery_date TEXT;
ALTER TABLE commerce_orders ADD COLUMN delivery_time_slot TEXT;
ALTER TABLE commerce_orders ADD COLUMN delivery_instructions TEXT;
ALTER TABLE commerce_orders ADD COLUMN customer_notes TEXT;

-- Extend carts with service type
ALTER TABLE carts ADD COLUMN service_type TEXT;
ALTER TABLE carts ADD COLUMN delivery_date TEXT;
ALTER TABLE carts ADD COLUMN delivery_time_slot TEXT;
