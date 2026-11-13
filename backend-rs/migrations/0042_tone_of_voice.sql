-- Migration 0042: Tone of Voice & Guidance (PostgreSQL)
-- Configurable AI personality, tone, answer length control, and custom instructions
-- that coach the AI on support policies and communication style.
-- Core part of Intercom's TRAIN phase.

-- ============================================================================
-- 1. TONE CONFIGURATION
-- ============================================================================
-- Per-organization tone settings for the AI agent.
CREATE TABLE IF NOT EXISTS tone_config (
    id TEXT PRIMARY KEY DEFAULT gen_random_uuid()::TEXT,
    organization_id TEXT NOT NULL,
    tone_style TEXT NOT NULL DEFAULT 'professional',
    personality TEXT NOT NULL DEFAULT 'helpful',
    answer_length TEXT NOT NULL DEFAULT 'medium',
    language TEXT NOT NULL DEFAULT 'en',
    formality_level INTEGER NOT NULL DEFAULT 5,
    humor_enabled BOOLEAN NOT NULL DEFAULT false,
    empathy_level INTEGER NOT NULL DEFAULT 5,
    conciseness_level INTEGER NOT NULL DEFAULT 5,
    custom_instructions TEXT,
    forbidden_phrases TEXT NOT NULL DEFAULT '[]',
    required_phrases TEXT NOT NULL DEFAULT '[]',
    greeting_template TEXT NOT NULL DEFAULT 'Hello! How can I help you today?',
    closing_template TEXT NOT NULL DEFAULT 'Is there anything else I can help you with?',
    escalation_template TEXT NOT NULL DEFAULT 'Let me connect you with a team member who can help.',
    enabled BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    FOREIGN KEY (organization_id) REFERENCES organizations(id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_tone_config_org ON tone_config(organization_id);

-- ============================================================================
-- 2. GUIDANCE RULES
-- ============================================================================
-- Custom instructions that coach the AI on support policies and communication style.
CREATE TABLE IF NOT EXISTS guidance_rules (
    id TEXT PRIMARY KEY DEFAULT gen_random_uuid()::TEXT,
    organization_id TEXT NOT NULL,
    name TEXT NOT NULL,
    description TEXT,
    rule_type TEXT NOT NULL DEFAULT 'style',
    instruction TEXT NOT NULL,
    priority INTEGER NOT NULL DEFAULT 0,
    applies_to TEXT NOT NULL DEFAULT 'all',
    enabled BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    FOREIGN KEY (organization_id) REFERENCES organizations(id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_guidance_org ON guidance_rules(organization_id);
CREATE INDEX IF NOT EXISTS idx_guidance_enabled ON guidance_rules(enabled);
CREATE INDEX IF NOT EXISTS idx_guidance_priority ON guidance_rules(priority DESC);

-- ============================================================================
-- 3. TONE ANALYTICS
-- ============================================================================
-- Tracks how tone settings affect customer satisfaction and resolution rates.
CREATE TABLE IF NOT EXISTS tone_analytics (
    id TEXT PRIMARY KEY DEFAULT gen_random_uuid()::TEXT,
    organization_id TEXT NOT NULL,
    tone_style TEXT NOT NULL,
    conversation_count INTEGER NOT NULL DEFAULT 0,
    avg_satisfaction DOUBLE PRECISION NOT NULL DEFAULT 0,
    avg_resolution_rate DOUBLE PRECISION NOT NULL DEFAULT 0,
    avg_response_length INTEGER NOT NULL DEFAULT 0,
    thumbs_up_count INTEGER NOT NULL DEFAULT 0,
    thumbs_down_count INTEGER NOT NULL DEFAULT 0,
    window_start TIMESTAMPTZ NOT NULL,
    window_end TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_tone_analytics_org ON tone_analytics(organization_id);
CREATE INDEX IF NOT EXISTS idx_tone_analytics_style ON tone_analytics(tone_style);
CREATE INDEX IF NOT EXISTS idx_tone_analytics_window ON tone_analytics(window_start, window_end);

-- ============================================================================
-- 4. TONE PRESETS
-- ============================================================================
-- Pre-built tone presets for quick configuration.
CREATE TABLE IF NOT EXISTS tone_presets (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,
    tone_style TEXT NOT NULL,
    personality TEXT NOT NULL,
    answer_length TEXT NOT NULL,
    formality_level INTEGER NOT NULL,
    humor_enabled BOOLEAN NOT NULL,
    empathy_level INTEGER NOT NULL,
    conciseness_level INTEGER NOT NULL,
    greeting_template TEXT NOT NULL,
    closing_template TEXT NOT NULL,
    escalation_template TEXT NOT NULL,
    custom_instructions TEXT
);

-- Seed default tone config
INSERT INTO tone_config (id, organization_id, tone_style, personality, answer_length)
VALUES ('tone-config-default', 'org-default', 'professional', 'helpful', 'medium')
ON CONFLICT (id) DO NOTHING;

-- Seed default guidance rules
INSERT INTO guidance_rules (id, organization_id, name, rule_type, instruction, priority, enabled)
VALUES ('guidance-1', 'org-default', 'Be Empathetic', 'style', 'Always acknowledge the customer''s feelings and show understanding before providing solutions.', 10, true)
ON CONFLICT (id) DO NOTHING;

INSERT INTO guidance_rules (id, organization_id, name, rule_type, instruction, priority, enabled)
VALUES ('guidance-2', 'org-default', 'Be Concise', 'style', 'Keep responses clear and concise. Avoid unnecessary jargon or overly technical language unless the customer demonstrates technical expertise.', 9, true)
ON CONFLICT (id) DO NOTHING;

INSERT INTO guidance_rules (id, organization_id, name, rule_type, instruction, priority, enabled)
VALUES ('guidance-3', 'org-default', 'Escalate When Unsure', 'policy', 'If you are not confident in your answer or the question is outside your knowledge base, escalate to a human agent immediately. Never guess or make up information.', 11, true)
ON CONFLICT (id) DO NOTHING;

INSERT INTO guidance_rules (id, organization_id, name, rule_type, instruction, priority, enabled)
VALUES ('guidance-4', 'org-default', 'No Promises on Refunds', 'policy', 'Never promise a refund directly. Always direct refund requests to a human agent or follow the established refund procedure.', 8, true)
ON CONFLICT (id) DO NOTHING;

INSERT INTO guidance_rules (id, organization_id, name, rule_type, instruction, priority, enabled)
VALUES ('guidance-5', 'org-default', 'Use Customer Name', 'style', 'When you know the customer''s name, use it naturally in the conversation to personalize the experience.', 7, true)
ON CONFLICT (id) DO NOTHING;

-- Seed tone presets
INSERT INTO tone_presets (id, name, description, tone_style, personality, answer_length, formality_level, humor_enabled, empathy_level, conciseness_level, greeting_template, closing_template, escalation_template)
VALUES (
    'preset-professional',
    'Professional',
    'Formal and business-appropriate tone for enterprise customers.',
    'professional', 'helpful', 'medium', 8, false, 6, 7,
    'Hello! How may I assist you today?',
    'Please let me know if you need any further assistance.',
    'I will connect you with a team member who can better assist you.'
)
ON CONFLICT (id) DO NOTHING;

INSERT INTO tone_presets (id, name, description, tone_style, personality, answer_length, formality_level, humor_enabled, empathy_level, conciseness_level, greeting_template, closing_template, escalation_template)
VALUES (
    'preset-friendly',
    'Friendly',
    'Warm and approachable tone for consumer-facing support.',
    'friendly', 'enthusiastic', 'medium', 4, false, 8, 5,
    'Hi there! How can I help you today? 😊',
    'Is there anything else I can help with? Happy to assist!',
    'Let me get someone who can help you out right away!'
)
ON CONFLICT (id) DO NOTHING;

INSERT INTO tone_presets (id, name, description, tone_style, personality, answer_length, formality_level, humor_enabled, empathy_level, conciseness_level, greeting_template, closing_template, escalation_template)
VALUES (
    'preset-humorous',
    'Humorous',
    'Light-hearted and fun tone for brands with a playful personality.',
    'humorous', 'playful', 'short', 3, true, 7, 6,
    'Hey! What can I do for you today? 🚀',
    'Anything else on your mind? I''m all ears!',
    'Time to bring in the big guns! Let me get someone who can help.'
)
ON CONFLICT (id) DO NOTHING;

INSERT INTO tone_presets (id, name, description, tone_style, personality, answer_length, formality_level, humor_enabled, empathy_level, conciseness_level, greeting_template, closing_template, escalation_template)
VALUES (
    'preset-concise',
    'Concise',
    'Short and direct responses for customers who want quick answers.',
    'professional', 'direct', 'short', 6, false, 5, 9,
    'Hello. How can I help?',
    'Need anything else?',
    'Connecting you with an agent now.'
)
ON CONFLICT (id) DO NOTHING;

INSERT INTO tone_presets (id, name, description, tone_style, personality, answer_length, formality_level, humor_enabled, empathy_level, conciseness_level, greeting_template, closing_template, escalation_template)
VALUES (
    'preset-detailed',
    'Detailed',
    'Thorough and comprehensive responses for complex technical support.',
    'professional', 'thorough', 'long', 7, false, 6, 3,
    'Hello! I''m here to provide detailed assistance. What can I help you with?',
    'Please review the information above and let me know if you have any questions.',
    'I''m connecting you with a specialist who can provide more detailed assistance.'
)
ON CONFLICT (id) DO NOTHING;
