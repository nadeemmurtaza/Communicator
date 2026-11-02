use sqlx::{sqlite::SqlitePoolOptions, Pool, Sqlite};
use std::sync::Arc;
use tracing::info;

pub type DbPool = Pool<Sqlite>;

#[derive(Clone)]
pub struct AppState {
    pub db: DbPool,
    pub config: Arc<crate::config::Config>,
    pub http_client: reqwest::Client,
    pub nid: Arc<crate::services::nid_client::NidClient>,
}

pub async fn init_db(database_url: &str) -> Result<DbPool, sqlx::Error> {
    info!("Connecting to database: {}", database_url);

    let pool = SqlitePoolOptions::new()
        .max_connections(10)
        .connect(database_url)
        .await?;

    // Auto-create required tables if not exist
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS contacts (
            id TEXT PRIMARY KEY,
            value TEXT NOT NULL,
            channel TEXT NOT NULL,
            person_name TEXT,
            first_name TEXT,
            last_name TEXT,
            quality_rating TEXT,
            consent_status TEXT,
            tags TEXT,
            created_at DATETIME NOT NULL,
            updated_at DATETIME NOT NULL
        );

        CREATE TABLE IF NOT EXISTS conversations (
            id TEXT PRIMARY KEY,
            contact_id TEXT,
            contact_name TEXT,
            customer_name TEXT,
            lead_name TEXT,
            contact_phone TEXT,
            phone TEXT,
            channel TEXT NOT NULL,
            state TEXT NOT NULL,
            priority TEXT,
            resolution_type TEXT,
            broadcast_id TEXT,
            broadcast_name TEXT,
            inbox_id TEXT,
            team_id TEXT,
            team_name TEXT,
            csat_score INTEGER,
            human_takeover BOOLEAN NOT NULL DEFAULT 0,
            assigned_to_id TEXT,
            assigned_to_name TEXT,
            snoozed_until DATETIME,
            language TEXT,
            updated_at DATETIME NOT NULL
        );

        CREATE TABLE IF NOT EXISTS messages (
            id TEXT PRIMARY KEY,
            conversation_id TEXT NOT NULL,
            sender TEXT NOT NULL,
            body TEXT NOT NULL,
            channel TEXT NOT NULL,
            external_id TEXT,
            delivery_status TEXT,
            created_at DATETIME NOT NULL
        );

        CREATE TABLE IF NOT EXISTS copilot_approvals (
            id TEXT PRIMARY KEY,
            conversation_id TEXT,
            contact_id TEXT,
            request_type TEXT,
            status TEXT NOT NULL DEFAULT 'pending',
            title TEXT NOT NULL,
            proposed_reply TEXT,
            amended_reply TEXT,
            action_data TEXT,
            internal_note TEXT,
            question_to_teammate TEXT,
            teammate_answer TEXT,
            assigned_to_id TEXT,
            reviewer_name TEXT,
            reviewer_feedback TEXT,
            dispatched_message_id TEXT,
            created_at DATETIME NOT NULL,
            updated_at DATETIME NOT NULL,
            resolved_at DATETIME
        );

        CREATE TABLE IF NOT EXISTS agent_policies (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            category TEXT,
            description TEXT,
            version TEXT NOT NULL DEFAULT '1.0',
            status TEXT NOT NULL DEFAULT 'active',
            process_steps TEXT,
            rules TEXT,
            guardrails TEXT,
            created_at DATETIME NOT NULL,
            updated_at DATETIME NOT NULL
        );

        CREATE TABLE IF NOT EXISTS simulation_scenarios (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            description TEXT,
            category TEXT,
            policy_id TEXT,
            customer_persona TEXT,
            expected_outcome TEXT,
            customer_messages TEXT,
            created_at DATETIME NOT NULL
        );

        CREATE TABLE IF NOT EXISTS simulation_runs (
            id TEXT PRIMARY KEY,
            scenario_id TEXT NOT NULL,
            policy_id TEXT,
            status TEXT NOT NULL DEFAULT 'completed',
            verdict TEXT NOT NULL,
            policy_adherence_score TEXT,
            empathy_score TEXT,
            accuracy_score TEXT,
            overall_score TEXT,
            turn_count INTEGER,
            transcript TEXT,
            evaluation_summary TEXT,
            recommendations TEXT,
            created_at DATETIME NOT NULL
        );

        CREATE TABLE IF NOT EXISTS agent_insights (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            category TEXT NOT NULL,
            description TEXT,
            human_rep_id TEXT,
            human_rep_name TEXT,
            exemplar_content TEXT,
            impact_score TEXT,
            estimated_resolution_lift TEXT,
            suggested_action TEXT,
            action_payload TEXT,
            status TEXT NOT NULL DEFAULT 'open',
            applied_at DATETIME,
            created_at DATETIME NOT NULL,
            updated_at DATETIME NOT NULL
        );

        CREATE TABLE IF NOT EXISTS agent_retraining_events (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            trigger_type TEXT,
            version_from TEXT,
            version_to TEXT,
            insight_id TEXT,
            changes_applied TEXT,
            status TEXT NOT NULL DEFAULT 'applied',
            baseline_resolution_rate TEXT,
            post_training_resolution_rate TEXT,
            resolution_lift TEXT,
            created_at DATETIME NOT NULL
        );

        CREATE TABLE IF NOT EXISTS ai_connections (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            provider TEXT NOT NULL,
            model_id TEXT NOT NULL,
            api_key TEXT,
            base_url TEXT,
            temperature TEXT DEFAULT '0.7',
            max_tokens INTEGER DEFAULT 4096,
            is_active BOOLEAN NOT NULL DEFAULT 1,
            assigned_jobs TEXT,
            last_tested_at DATETIME,
            test_status TEXT DEFAULT 'untested',
            latency_ms INTEGER,
            created_at DATETIME NOT NULL,
            updated_at DATETIME NOT NULL
        );

        CREATE TABLE IF NOT EXISTS ai_custom_policies (
            id TEXT PRIMARY KEY,
            job_type TEXT NOT NULL UNIQUE,
            tone TEXT,
            guidelines TEXT,
            rules TEXT,
            guardrails TEXT,
            auto_approval_mode TEXT DEFAULT 'review_required',
            min_csat_learning_threshold TEXT DEFAULT '4.5',
            max_autonomous_refund TEXT DEFAULT '50.0',
            created_at DATETIME NOT NULL,
            updated_at DATETIME NOT NULL
        );

        CREATE TABLE IF NOT EXISTS canned_responses (
            id TEXT PRIMARY KEY,
            short_code TEXT NOT NULL UNIQUE,
            content TEXT NOT NULL,
            category TEXT DEFAULT 'general',
            created_at DATETIME NOT NULL
        );

        CREATE TABLE IF NOT EXISTS macros (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            description TEXT,
            actions TEXT NOT NULL,
            created_at DATETIME NOT NULL,
            updated_at DATETIME NOT NULL
        );

        CREATE TABLE IF NOT EXISTS sla_policies (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            description TEXT,
            first_response_time_threshold INTEGER NOT NULL DEFAULT 60,
            resolution_time_threshold INTEGER NOT NULL DEFAULT 1440,
            only_during_business_hours BOOLEAN NOT NULL DEFAULT 0,
            priorities TEXT DEFAULT '[]',
            inbox_ids TEXT DEFAULT '[]',
            is_active BOOLEAN NOT NULL DEFAULT 1,
            created_at DATETIME NOT NULL,
            updated_at DATETIME NOT NULL
        );

        CREATE TABLE IF NOT EXISTS automation_rules (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            description TEXT,
            trigger_keyword TEXT NOT NULL,
            match_type TEXT NOT NULL,
            action_type TEXT NOT NULL,
            action_value TEXT NOT NULL,
            priority INTEGER NOT NULL DEFAULT 0,
            is_active BOOLEAN NOT NULL DEFAULT 1,
            execution_count INTEGER DEFAULT 0,
            last_triggered_at DATETIME,
            created_at DATETIME NOT NULL,
            tenant_key TEXT DEFAULT 'newax',
            channel TEXT DEFAULT ''
        );

        CREATE TABLE IF NOT EXISTS channel_gateway_connections (
            id TEXT PRIMARY KEY,
            gateway_type TEXT NOT NULL,
            provider TEXT NOT NULL,
            name TEXT NOT NULL,
            is_default BOOLEAN NOT NULL DEFAULT 0,
            is_active BOOLEAN NOT NULL DEFAULT 1,
            credentials TEXT NOT NULL DEFAULT '{}',
            config TEXT NOT NULL DEFAULT '{}',
            test_status TEXT NOT NULL DEFAULT 'untested',
            latency_ms INTEGER NOT NULL DEFAULT 0,
            last_tested_at DATETIME,
            created_at DATETIME NOT NULL,
            updated_at DATETIME NOT NULL
        );

        CREATE TABLE IF NOT EXISTS webhook_endpoints (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            url TEXT NOT NULL,
            secret TEXT NOT NULL,
            events TEXT NOT NULL DEFAULT '[]',
            headers TEXT NOT NULL DEFAULT '{}',
            is_active BOOLEAN NOT NULL DEFAULT 1,
            retry_count INTEGER NOT NULL DEFAULT 3,
            timeout_seconds INTEGER NOT NULL DEFAULT 10,
            total_delivered INTEGER NOT NULL DEFAULT 0,
            total_failed INTEGER NOT NULL DEFAULT 0,
            last_status_code INTEGER,
            last_delivered_at DATETIME,
            created_at DATETIME NOT NULL,
            updated_at DATETIME NOT NULL
        );

        CREATE TABLE IF NOT EXISTS webhook_delivery_logs (
            id TEXT PRIMARY KEY,
            endpoint_id TEXT NOT NULL,
            event TEXT NOT NULL,
            payload TEXT NOT NULL DEFAULT '{}',
            status_code INTEGER NOT NULL DEFAULT 0,
            response_headers TEXT NOT NULL DEFAULT '{}',
            response_body TEXT NOT NULL DEFAULT '',
            duration_ms INTEGER NOT NULL DEFAULT 0,
            attempt INTEGER NOT NULL DEFAULT 1,
            success BOOLEAN NOT NULL DEFAULT 0,
            error_message TEXT,
            created_at DATETIME NOT NULL
        );

        CREATE TABLE IF NOT EXISTS stripe_configs (
            id TEXT PRIMARY KEY,
            account_name TEXT NOT NULL DEFAULT 'Primary Stripe Account',
            api_key TEXT NOT NULL DEFAULT '',
            publishable_key TEXT NOT NULL DEFAULT '',
            webhook_secret TEXT NOT NULL DEFAULT '',
            is_enabled BOOLEAN NOT NULL DEFAULT 1,
            test_mode BOOLEAN NOT NULL DEFAULT 1,
            default_currency TEXT NOT NULL DEFAULT 'usd',
            last_tested_at DATETIME,
            last_test_status TEXT NOT NULL DEFAULT 'untested',
            created_at DATETIME NOT NULL,
            updated_at DATETIME NOT NULL
        );

        CREATE TABLE IF NOT EXISTS stripe_billing_action_logs (
            id TEXT PRIMARY KEY,
            contact_id TEXT,
            customer_identifier TEXT NOT NULL DEFAULT '',
            action_type TEXT NOT NULL,
            amount_cents INTEGER NOT NULL DEFAULT 0,
            currency TEXT NOT NULL DEFAULT 'usd',
            status TEXT NOT NULL DEFAULT 'succeeded',
            stripe_id TEXT NOT NULL DEFAULT '',
            actor_email TEXT NOT NULL DEFAULT 'agent@newax.com',
            reason TEXT NOT NULL DEFAULT '',
            details TEXT NOT NULL DEFAULT '{}',
            created_at DATETIME NOT NULL
        );

        CREATE TABLE IF NOT EXISTS tickets (
            id TEXT PRIMARY KEY,
            ticket_number TEXT NOT NULL,
            title TEXT NOT NULL,
            description TEXT NOT NULL,
            ticket_type TEXT NOT NULL,
            status TEXT NOT NULL,
            priority TEXT NOT NULL,
            department TEXT NOT NULL,
            conversation_id TEXT,
            contact_id TEXT,
            tracker_id TEXT,
            assigned_to_id TEXT,
            resolution_summary TEXT,
            customer_broadcast_sent BOOLEAN DEFAULT 0,
            resolved_at DATETIME,
            created_at DATETIME NOT NULL,
            updated_at DATETIME NOT NULL
        );

        CREATE TABLE IF NOT EXISTS ticket_activities (
            id TEXT PRIMARY KEY,
            ticket_id TEXT NOT NULL,
            author_id TEXT,
            author_name TEXT,
            activity_type TEXT NOT NULL,
            message TEXT NOT NULL,
            activity_metadata TEXT,
            created_at DATETIME NOT NULL
        );

        CREATE TABLE IF NOT EXISTS issue_tracker_configs (
            id TEXT PRIMARY KEY,
            provider TEXT NOT NULL UNIQUE,
            account_name TEXT,
            is_enabled BOOLEAN NOT NULL DEFAULT 1,
            test_mode BOOLEAN NOT NULL DEFAULT 1,
            credentials TEXT,
            config TEXT,
            last_tested_at DATETIME,
            last_test_status TEXT,
            created_at DATETIME NOT NULL,
            updated_at DATETIME NOT NULL
        );

        CREATE TABLE IF NOT EXISTS linked_issues (
            id TEXT PRIMARY KEY,
            conversation_id TEXT,
            ticket_id TEXT,
            provider TEXT NOT NULL,
            external_issue_id TEXT,
            issue_key TEXT NOT NULL,
            issue_url TEXT NOT NULL,
            title TEXT NOT NULL,
            status TEXT NOT NULL,
            priority TEXT,
            assigned_to TEXT,
            created_by_email TEXT,
            sync_status TEXT NOT NULL DEFAULT 'synced',
            metadata_json TEXT,
            created_at DATETIME NOT NULL,
            updated_at DATETIME NOT NULL
        );

        CREATE TABLE IF NOT EXISTS slack_conversation_syncs (
            id TEXT PRIMARY KEY,
            conversation_id TEXT NOT NULL UNIQUE,
            slack_channel_id TEXT,
            slack_channel_name TEXT,
            slack_thread_ts TEXT,
            is_synced BOOLEAN NOT NULL DEFAULT 1,
            last_synced_at DATETIME,
            created_at DATETIME NOT NULL
        );

        CREATE TABLE IF NOT EXISTS tags (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL UNIQUE,
            created_at DATETIME NOT NULL,
            tenant_key TEXT
        );

        CREATE TABLE IF NOT EXISTS contact_tags (
            contact_id TEXT NOT NULL,
            tag_id TEXT NOT NULL,
            PRIMARY KEY (contact_id, tag_id)
        );

        CREATE TABLE IF NOT EXISTS broadcasts (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            template_name TEXT,
            template_language TEXT,
            body_preview TEXT,
            tag_name TEXT,
            status TEXT NOT NULL,
            scheduled_at TEXT,
            sent_count INTEGER DEFAULT 0,
            delivered_count INTEGER DEFAULT 0,
            read_count INTEGER DEFAULT 0,
            failed_count INTEGER DEFAULT 0,
            created_at DATETIME NOT NULL,
            audience_count INTEGER DEFAULT 0,
            total_recipients INTEGER DEFAULT 0,
            skipped_count INTEGER DEFAULT 0,
            template_params TEXT,
            tenant_key TEXT,
            campaign_type TEXT,
            objective TEXT,
            campaign_template_id TEXT,
            parent_campaign_id TEXT,
            step_index INTEGER,
            step_delay_days INTEGER,
            stop_on_reply BOOLEAN DEFAULT 1
        );

        CREATE TABLE IF NOT EXISTS broadcast_recipients (
            id TEXT PRIMARY KEY,
            broadcast_id TEXT NOT NULL,
            contact_id TEXT NOT NULL,
            status TEXT NOT NULL,
            provider_message_id TEXT,
            error TEXT,
            sent_at DATETIME,
            delivered_at DATETIME,
            read_at DATETIME
        );

        CREATE TABLE IF NOT EXISTS services (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            summary TEXT,
            details TEXT,
            target_institution_types TEXT,
            faq TEXT,
            pricing_notes TEXT,
            created_at DATETIME NOT NULL,
            updated_at DATETIME NOT NULL,
            tenant_key TEXT
        );

        CREATE TABLE IF NOT EXISTS follow_up_sequences (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            description TEXT,
            is_active BOOLEAN NOT NULL DEFAULT 1,
            created_at DATETIME NOT NULL
        );

        CREATE TABLE IF NOT EXISTS sequence_steps (
            id TEXT PRIMARY KEY,
            sequence_id TEXT NOT NULL,
            step_number INTEGER NOT NULL,
            delay_days INTEGER NOT NULL,
            channel TEXT,
            template TEXT,
            variant TEXT
        );

        CREATE TABLE IF NOT EXISTS audit_logs (
            id TEXT PRIMARY KEY,
            entity_type TEXT NOT NULL,
            entity_id TEXT,
            action TEXT NOT NULL,
            actor TEXT NOT NULL,
            actor_email TEXT,
            actor_name TEXT,
            actor_role TEXT,
            severity TEXT DEFAULT 'info',
            compliance_category TEXT DEFAULT 'access_control',
            description TEXT,
            ip_address TEXT DEFAULT '127.0.0.1',
            user_agent TEXT,
            details TEXT DEFAULT '{}',
            created_at DATETIME NOT NULL
        );

        CREATE TABLE IF NOT EXISTS role_definitions (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL UNIQUE,
            display_name TEXT NOT NULL,
            description TEXT,
            is_system_role BOOLEAN NOT NULL DEFAULT 0,
            permissions TEXT NOT NULL DEFAULT '[]',
            created_at DATETIME NOT NULL,
            updated_at DATETIME NOT NULL
        );

        CREATE TABLE IF NOT EXISTS sso_configs (
            id TEXT PRIMARY KEY,
            is_enabled BOOLEAN NOT NULL DEFAULT 0,
            provider_type TEXT NOT NULL DEFAULT 'saml',
            idp_entity_id TEXT DEFAULT '',
            idp_sso_url TEXT DEFAULT '',
            idp_x509_cert TEXT DEFAULT '',
            sp_entity_id TEXT DEFAULT 'urn:communicator:sp',
            sp_acs_url TEXT DEFAULT '/api/v1/sso/saml/acs',
            enforced_domains TEXT DEFAULT '[]',
            allow_unlinked_passwords BOOLEAN NOT NULL DEFAULT 1,
            jit_provisioning_enabled BOOLEAN NOT NULL DEFAULT 1,
            default_role TEXT DEFAULT 'agent',
            attribute_mapping TEXT DEFAULT '{}',
            last_tested_at DATETIME,
            last_test_status TEXT DEFAULT 'untested',
            last_test_message TEXT DEFAULT '',
            created_at DATETIME NOT NULL,
            updated_at DATETIME NOT NULL
        );

        CREATE TABLE IF NOT EXISTS tenants (
            id TEXT PRIMARY KEY,
            tenant_key TEXT NOT NULL UNIQUE,
            name TEXT NOT NULL,
            owner_account_id TEXT,
            created_at DATETIME NOT NULL
        );

        CREATE TABLE IF NOT EXISTS custom_attribute_definitions (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL UNIQUE,
            display_name TEXT NOT NULL,
            description TEXT,
            attribute_model TEXT NOT NULL,
            attribute_type TEXT NOT NULL,
            default_value TEXT,
            options TEXT DEFAULT '[]',
            is_required BOOLEAN NOT NULL DEFAULT 0,
            regex_pattern TEXT,
            regex_cue TEXT,
            created_at DATETIME NOT NULL,
            updated_at DATETIME NOT NULL
        );
        "#,
    )
    .execute(&pool)
    .await?;

    // Migration safety: ensure total_recipients column exists on existing broadcasts table
    let _ = sqlx::query("ALTER TABLE broadcasts ADD COLUMN total_recipients INTEGER DEFAULT 0").execute(&pool).await;

    // Migration safety: ensure extra audit columns exist on existing audit_logs table
    let _ = sqlx::query("ALTER TABLE audit_logs ADD COLUMN actor_email TEXT").execute(&pool).await;
    let _ = sqlx::query("ALTER TABLE audit_logs ADD COLUMN actor_name TEXT").execute(&pool).await;
    let _ = sqlx::query("ALTER TABLE audit_logs ADD COLUMN actor_role TEXT").execute(&pool).await;
    let _ = sqlx::query("ALTER TABLE audit_logs ADD COLUMN severity TEXT DEFAULT 'info'").execute(&pool).await;
    let _ = sqlx::query("ALTER TABLE audit_logs ADD COLUMN compliance_category TEXT DEFAULT 'access_control'").execute(&pool).await;
    let _ = sqlx::query("ALTER TABLE audit_logs ADD COLUMN description TEXT").execute(&pool).await;
    let _ = sqlx::query("ALTER TABLE audit_logs ADD COLUMN ip_address TEXT DEFAULT '127.0.0.1'").execute(&pool).await;
    let _ = sqlx::query("ALTER TABLE audit_logs ADD COLUMN user_agent TEXT").execute(&pool).await;

    info!("Database schema initialized successfully.");
    Ok(pool)
}
