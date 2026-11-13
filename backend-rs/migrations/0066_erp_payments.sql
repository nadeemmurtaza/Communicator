-- ERP Payments Module: Port of payments.prisma (55 lines)
-- Maps the ERP payment system to PostgreSQL tables for the Rust backend
-- Includes: Payments with provider support, receipt uploads, staff approval, refunds

-- Payments
CREATE TABLE IF NOT EXISTS erp_payments (
    id TEXT PRIMARY KEY,
    order_id TEXT NOT NULL REFERENCES erp_orders(id) ON DELETE CASCADE,
    provider TEXT NOT NULL DEFAULT 'cod',
    status TEXT NOT NULL DEFAULT 'PENDING',
    amount INTEGER NOT NULL DEFAULT 0,
    currency TEXT NOT NULL DEFAULT 'PKR',
    reference TEXT UNIQUE,
    provider_ref TEXT,
    meta JSONB,
    receipt_url TEXT,
    failure_reason TEXT,
    approved_by_id TEXT,
    paid_at TEXT,
    refunded_at TEXT,
    refund_amount INTEGER,
    created_at TEXT NOT NULL DEFAULT (NOW()),
    updated_at TEXT NOT NULL DEFAULT (NOW())
);
CREATE INDEX IF NOT EXISTS idx_erp_payments_order ON erp_payments(order_id);
CREATE INDEX IF NOT EXISTS idx_erp_payments_status ON erp_payments(status);
CREATE INDEX IF NOT EXISTS idx_erp_payments_provider_status ON erp_payments(provider, status);
CREATE INDEX IF NOT EXISTS idx_erp_payments_reference ON erp_payments(reference);

-- Payment methods (configurable providers)
CREATE TABLE IF NOT EXISTS erp_payment_methods (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    code TEXT NOT NULL UNIQUE,
    config JSONB NOT NULL DEFAULT '{}',
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TEXT NOT NULL DEFAULT (NOW()),
    updated_at TEXT NOT NULL DEFAULT (NOW())
);
-- Seed default payment methods
INSERT INTO erp_payment_methods (id, name, code, config) VALUES
    ('pm_cod', 'Cash on Delivery', 'cod', '{"type": "cod"}'::jsonb),
    ('pm_bank', 'Bank Transfer', 'bank_transfer', '{"type": "bank_transfer"}'::jsonb),
    ('pm_jazzcash', 'JazzCash', 'jazzcash', '{"type": "jazzcash"}'::jsonb),
    ('pm_easypaisa', 'EasyPaisa', 'easypaisa', '{"type": "easypaisa"}'::jsonb)
ON CONFLICT (code) DO NOTHING;
