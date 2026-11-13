-- Phase 52: SMTP & Email Templates
-- Adds SMTP configuration and email template management for transactional emails
-- (OTP, password reset, welcome, etc.). Users can configure SMTP directly in
-- the Communicator and manage email templates with variable substitution.

-- SMTP settings (global + per-brand configuration)
CREATE TABLE IF NOT EXISTS smtp_settings (
    id TEXT PRIMARY KEY,
    brand_id TEXT,
    host TEXT NOT NULL,
    port INTEGER NOT NULL DEFAULT 587,
    security TEXT NOT NULL DEFAULT 'starttls',  -- 'starttls', 'tls', 'none'
    auth_method TEXT NOT NULL DEFAULT 'login',   -- 'login', 'plain', 'none'
    username TEXT NOT NULL DEFAULT '',
    password TEXT NOT NULL DEFAULT '',           -- encrypted at rest in production
    from_address TEXT NOT NULL,
    from_name TEXT NOT NULL DEFAULT '',
    reply_to_address TEXT,
    connection_timeout_seconds INTEGER NOT NULL DEFAULT 30,
    enabled BOOLEAN NOT NULL DEFAULT true,
    test_sent_at TIMESTAMPTZ,
    test_status TEXT,                             -- 'success', 'failed'
    test_error TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_smtp_settings_brand ON smtp_settings(brand_id);

-- Email templates (reusable templates with variable substitution)
CREATE TABLE IF NOT EXISTS email_templates (
    id TEXT PRIMARY KEY,
    brand_id TEXT,
    name TEXT NOT NULL UNIQUE,
    subject TEXT NOT NULL,
    body_html TEXT NOT NULL,
    body_text TEXT NOT NULL DEFAULT '',
    template_type TEXT NOT NULL DEFAULT 'transactional',  -- 'transactional', 'marketing', 'notification'
    category TEXT NOT NULL DEFAULT 'general',             -- 'otp', 'welcome', 'password_reset', 'order', etc.
    language TEXT NOT NULL DEFAULT 'en',
    enabled BOOLEAN NOT NULL DEFAULT true,
    is_default BOOLEAN NOT NULL DEFAULT false,            -- system default templates
    variables TEXT NOT NULL DEFAULT '[]',                 -- JSON array of variable names
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_email_templates_brand ON email_templates(brand_id);
CREATE INDEX IF NOT EXISTS idx_email_templates_category ON email_templates(category);
CREATE INDEX IF NOT EXISTS idx_email_templates_type ON email_templates(template_type);

-- Email send log (audit trail for all sent emails)
CREATE TABLE IF NOT EXISTS email_send_log (
    id TEXT PRIMARY KEY,
    brand_id TEXT,
    template_id TEXT,
    template_name TEXT,
    to_address TEXT NOT NULL,
    from_address TEXT NOT NULL,
    subject TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending',              -- 'pending', 'sent', 'failed', 'bounced'
    error_message TEXT,
    smtp_response TEXT,
    message_id TEXT,                                     -- SMTP Message-ID header
    sent_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_email_send_log_brand ON email_send_log(brand_id);
CREATE INDEX IF NOT EXISTS idx_email_send_log_status ON email_send_log(status);
CREATE INDEX IF NOT EXISTS idx_email_send_log_to ON email_send_log(to_address);
CREATE INDEX IF NOT EXISTS idx_email_send_log_created ON email_send_log(created_at);

-- Default email templates seeded on first run
-- OTP template
INSERT INTO email_templates (id, name, subject, body_html, body_text, template_type, category, language, is_default, variables)
VALUES (
    'tpl-otp-verify',
    'OTP Verification',
    'Your verification code: {{code}}',
    '<!DOCTYPE html><html><head><meta charset="utf-8"><style>body{font-family:Arial,sans-serif;background:#f4f4f4;margin:0;padding:0}.container{max-width:600px;margin:20px auto;background:#fff;border-radius:8px;padding:40px}.code{font-size:36px;font-weight:bold;letter-spacing:8px;color:#2563eb;text-align:center;padding:20px;background:#f0f7ff;border-radius:8px;margin:20px 0}.header{color:#1e293b;font-size:24px;margin-bottom:10px}.footer{color:#64748b;font-size:12px;margin-top:30px;text-align:center}</style></head><body><div class="container"><h1 class="header">Verification Code</h1><p>Hello {{name}},</p><p>Your verification code is:</p><div class="code">{{code}}</div><p>This code expires in {{expiry_minutes}} minutes. If you did not request this code, please ignore this email.</p><div class="footer"><p>&copy; {{year}} {{brand_name}}. All rights reserved.</p></div></div></body></html>',
    'Your verification code is: {{code}}. This code expires in {{expiry_minutes}} minutes. If you did not request this code, please ignore this email.',
    'transactional',
    'otp',
    'en',
    true,
    '["code", "name", "expiry_minutes", "year", "brand_name"]'
) ON CONFLICT (name) DO NOTHING;

-- Welcome email template
INSERT INTO email_templates (id, name, subject, body_html, body_text, template_type, category, language, is_default, variables)
VALUES (
    'tpl-welcome',
    'Welcome Email',
    'Welcome to {{brand_name}}, {{name}}!',
    '<!DOCTYPE html><html><head><meta charset="utf-8"><style>body{font-family:Arial,sans-serif;background:#f4f4f4;margin:0;padding:0}.container{max-width:600px;margin:20px auto;background:#fff;border-radius:8px;padding:40px}.header{color:#1e293b;font-size:24px;margin-bottom:10px}.button{display:inline-block;padding:12px 24px;background:#2563eb;color:#fff;text-decoration:none;border-radius:6px;margin:20px 0}.footer{color:#64748b;font-size:12px;margin-top:30px;text-align:center}</style></head><body><div class="container"><h1 class="header">Welcome, {{name}}!</h1><p>Thank you for joining {{brand_name}}. We''re excited to have you on board.</p><p>You can get started by visiting your dashboard:</p><a href="{{dashboard_url}}" class="button">Go to Dashboard</a><div class="footer"><p>&copy; {{year}} {{brand_name}}. All rights reserved.</p></div></div></body></html>',
    'Welcome to {{brand_name}}, {{name}}! Thank you for joining us. Visit your dashboard: {{dashboard_url}}',
    'transactional',
    'welcome',
    'en',
    true,
    '["name", "brand_name", "dashboard_url", "year"]'
) ON CONFLICT (name) DO NOTHING;

-- Password reset template
INSERT INTO email_templates (id, name, subject, body_html, body_text, template_type, category, language, is_default, variables)
VALUES (
    'tpl-password-reset',
    'Password Reset',
    'Reset your password for {{brand_name}}',
    '<!DOCTYPE html><html><head><meta charset="utf-8"><style>body{font-family:Arial,sans-serif;background:#f4f4f4;margin:0;padding:0}.container{max-width:600px;margin:20px auto;background:#fff;border-radius:8px;padding:40px}.header{color:#1e293b;font-size:24px;margin-bottom:10px}.button{display:inline-block;padding:12px 24px;background:#2563eb;color:#fff;text-decoration:none;border-radius:6px;margin:20px 0}.footer{color:#64748b;font-size:12px;margin-top:30px;text-align:center}</style></head><body><div class="container"><h1 class="header">Password Reset</h1><p>Hello {{name}},</p><p>We received a request to reset your password. Click the button below to create a new password:</p><a href="{{reset_url}}" class="button">Reset Password</a><p>This link expires in {{expiry_minutes}} minutes. If you did not request a password reset, please ignore this email.</p><div class="footer"><p>&copy; {{year}} {{brand_name}}. All rights reserved.</p></div></div></body></html>',
    'Reset your password: {{reset_url}}. This link expires in {{expiry_minutes}} minutes.',
    'transactional',
    'password_reset',
    'en',
    true,
    '["name", "brand_name", "reset_url", "expiry_minutes", "year"]'
) ON CONFLICT (name) DO NOTHING;

-- Order confirmation template
INSERT INTO email_templates (id, name, subject, body_html, body_text, template_type, category, language, is_default, variables)
VALUES (
    'tpl-order-confirmation',
    'Order Confirmation',
    'Order #{{order_number}} confirmed',
    '<!DOCTYPE html><html><head><meta charset="utf-8"><style>body{font-family:Arial,sans-serif;background:#f4f4f4;margin:0;padding:0}.container{max-width:600px;margin:20px auto;background:#fff;border-radius:8px;padding:40px}.header{color:#1e293b;font-size:24px;margin-bottom:10px}.order-details{background:#f8fafc;padding:16px;border-radius:6px;margin:16px 0}.footer{color:#64748b;font-size:12px;margin-top:30px;text-align:center}</style></head><body><div class="container"><h1 class="header">Order Confirmed</h1><p>Hello {{name}},</p><p>Your order has been confirmed and is being processed.</p><div class="order-details"><p><strong>Order:</strong> #{{order_number}}</p><p><strong>Total:</strong> {{order_total}}</p><p><strong>Date:</strong> {{order_date}}</p></div><p>We''ll send you another email when your order ships.</p><div class="footer"><p>&copy; {{year}} {{brand_name}}. All rights reserved.</p></div></div></body></html>',
    'Order #{{order_number}} confirmed. Total: {{order_total}}. Date: {{order_date}}.',
    'transactional',
    'order',
    'en',
    true,
    '["name", "brand_name", "order_number", "order_total", "order_date", "year"]'
) ON CONFLICT (name) DO NOTHING;
