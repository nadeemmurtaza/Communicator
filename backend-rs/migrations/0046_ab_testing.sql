-- Phase 38: A/B Testing & Phased Deployments
-- Enables data-driven experimentation on AI configurations before full deployment

CREATE TABLE IF NOT EXISTS ab_experiments (
    id TEXT PRIMARY KEY DEFAULT (gen_random_uuid()::TEXT),
    name TEXT NOT NULL,
    description TEXT,
    experiment_type TEXT NOT NULL DEFAULT 'confidence_threshold', -- confidence_threshold, retrieval_strategy, response_format, escalation_trigger, tone_style, model
    status TEXT NOT NULL DEFAULT 'draft', -- draft, running, paused, completed, failed
    traffic_split FLOAT NOT NULL DEFAULT 0.5, -- 0.0 to 1.0 for control group
    start_at TIMESTAMPTZ,
    end_at TIMESTAMPTZ,
    winner_variant_id TEXT,
    created_by TEXT,
    org_id TEXT NOT NULL DEFAULT 'org-default',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_ab_experiments_status ON ab_experiments(status);
CREATE INDEX IF NOT EXISTS idx_ab_experiments_type ON ab_experiments(experiment_type);
CREATE INDEX IF NOT EXISTS idx_ab_experiments_org ON ab_experiments(org_id);

CREATE TABLE IF NOT EXISTS ab_variants (
    id TEXT PRIMARY KEY DEFAULT (gen_random_uuid()::TEXT),
    experiment_id TEXT NOT NULL REFERENCES ab_experiments(id) ON DELETE CASCADE,
    variant_name TEXT NOT NULL, -- control, variant_a, variant_b, variant_c
    config JSONB NOT NULL DEFAULT '{}'::jsonb, -- variant-specific config (threshold, strategy, format, etc.)
    is_control BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_ab_variants_experiment ON ab_variants(experiment_id);

CREATE TABLE IF NOT EXISTS ab_assignments (
    id TEXT PRIMARY KEY DEFAULT (gen_random_uuid()::TEXT),
    experiment_id TEXT NOT NULL REFERENCES ab_experiments(id) ON DELETE CASCADE,
    variant_id TEXT NOT NULL REFERENCES ab_variants(id) ON DELETE CASCADE,
    conversation_id TEXT NOT NULL,
    contact_id TEXT,
    assigned_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_ab_assignments_experiment ON ab_assignments(experiment_id);
CREATE INDEX IF NOT EXISTS idx_ab_assignments_variant ON ab_assignments(variant_id);
CREATE INDEX IF NOT EXISTS idx_ab_assignments_conversation ON ab_assignments(conversation_id);

CREATE TABLE IF NOT EXISTS ab_results (
    id TEXT PRIMARY KEY DEFAULT (gen_random_uuid()::TEXT),
    experiment_id TEXT NOT NULL REFERENCES ab_experiments(id) ON DELETE CASCADE,
    variant_id TEXT NOT NULL REFERENCES ab_variants(id) ON DELETE CASCADE,
    assignment_id TEXT REFERENCES ab_assignments(id) ON DELETE CASCADE,
    metric_name TEXT NOT NULL, -- resolution_rate, satisfaction, response_time, feedback_score, validation_score
    metric_value FLOAT NOT NULL,
    metadata JSONB DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_ab_results_experiment ON ab_results(experiment_id);
CREATE INDEX IF NOT EXISTS idx_ab_results_variant ON ab_results(variant_id);
CREATE INDEX IF NOT EXISTS idx_ab_results_metric ON ab_results(metric_name);

CREATE TABLE IF NOT EXISTS ab_experiment_summary (
    id TEXT PRIMARY KEY DEFAULT (gen_random_uuid()::TEXT),
    experiment_id TEXT NOT NULL REFERENCES ab_experiments(id) ON DELETE CASCADE,
    total_assignments INTEGER NOT NULL DEFAULT 0,
    total_results INTEGER NOT NULL DEFAULT 0,
    variant_stats JSONB NOT NULL DEFAULT '{}'::jsonb, -- per-variant aggregated stats
    statistical_significance FLOAT DEFAULT 0, -- 0 to 1
    recommended_winner TEXT,
    snapshot_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_ab_summary_experiment ON ab_experiment_summary(experiment_id);

CREATE TABLE IF NOT EXISTS ab_settings (
    id TEXT PRIMARY KEY DEFAULT (gen_random_uuid()::TEXT),
    org_id TEXT NOT NULL DEFAULT 'org-default',
    enabled BOOLEAN NOT NULL DEFAULT true,
    auto_detect_winner BOOLEAN NOT NULL DEFAULT false,
    min_sample_size INTEGER NOT NULL DEFAULT 100,
    significance_threshold FLOAT NOT NULL DEFAULT 0.95,
    max_concurrent_experiments INTEGER NOT NULL DEFAULT 5,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Seed default settings
INSERT INTO ab_settings (id, org_id, enabled, auto_detect_winner, min_sample_size, significance_threshold)
VALUES ('default-ab-settings', 'org-default', true, false, 100, 0.95)
ON CONFLICT (id) DO NOTHING;

-- Seed sample experiment
INSERT INTO ab_experiments (id, name, description, experiment_type, status, traffic_split, org_id)
VALUES ('sample-ab-experiment', 'Confidence Threshold Test', 'Test if lowering confidence threshold improves resolution rate', 'confidence_threshold', 'draft', 0.5, 'org-default')
ON CONFLICT (id) DO NOTHING;

INSERT INTO ab_variants (id, experiment_id, variant_name, config, is_control)
VALUES 
('sample-control', 'sample-ab-experiment', 'Control (0.7)', '{"confidence_threshold": 0.7}'::jsonb, true),
('sample-variant-a', 'sample-ab-experiment', 'Variant A (0.5)', '{"confidence_threshold": 0.5}'::jsonb, false)
ON CONFLICT (id) DO NOTHING;
