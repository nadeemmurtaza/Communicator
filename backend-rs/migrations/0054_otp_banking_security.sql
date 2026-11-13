-- Phase 54: OTP Engine Banking-Grade Security Hardening
-- Fixes all 7 critical/important security gaps identified in audit.
--
-- Changes:
-- 1. otp_code column now stores masked value only (plaintext never persisted)
-- 2. metadata JSONB already stores ip_address and user_agent_hash (IP/device binding)
-- 3. Per-IP rate limiting uses existing otp_rate_limit table with "ip:{address}" as recipient
-- 4. In-app messages store masked OTP only
-- 5. Session tokens generated on verification (no schema change needed)
-- 6. Rejection sampling in code generation (no schema change needed)
-- 7. Per-IP rate limiting (no schema change needed)
--
-- This migration adds a session_token column to otp_records for audit trail
-- and ensures the metadata column is properly indexed for IP binding lookups.

-- Add session_token column to otp_records for tracking issued tokens
ALTER TABLE otp_records ADD COLUMN IF NOT EXISTS session_token TEXT;
ALTER TABLE otp_records ADD COLUMN IF NOT EXISTS session_token_issued_at TIMESTAMPTZ;

CREATE INDEX IF NOT EXISTS idx_otp_records_metadata_ip ON otp_records USING GIN (metadata);

-- Add comment documenting the security model
COMMENT ON TABLE otp_records IS 'OTP records: plaintext code NEVER stored. otp_code column holds masked display value. Verification uses otp_hash (HMAC-SHA256 with per-OTP salt). IP/device binding via metadata JSONB.';
COMMENT ON COLUMN otp_records.otp_code IS 'Masked display value only (e.g., A7x••••••9mK). Plaintext OTP exists only in memory during dispatch.';
COMMENT ON COLUMN otp_records.metadata IS 'Contains ip_address and user_agent_hash for IP/device binding verification.';
