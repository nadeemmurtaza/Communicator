use sqlx::{sqlite::SqlitePoolOptions, Pool, Row, Sqlite};
use std::sync::Arc;
use tracing::{info, warn};

pub type DbPool = Pool<Sqlite>;

#[derive(Clone)]
pub struct AppState {
    pub db: DbPool,
    pub config: Arc<crate::config::Config>,
    pub http_client: reqwest::Client,
    pub nid: Arc<crate::services::nid_client::NidClient>,
}

/// Reads a column that the code cannot substitute a stand-in for. An empty
/// string here would become an address we send to, a conversation we write
/// into, or a body we deliver -- so the caller has to handle the failure
/// instead. Returns the column name in the error so the message says which
/// field the schema is missing.
pub fn required_text(row: &sqlx::sqlite::SqliteRow, column: &str) -> Result<String, String> {
    if let Ok(value) = row.try_get::<String, _>(column) {
        return Ok(value);
    }
    row.try_get::<Option<String>, _>(column)
        .ok()
        .flatten()
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| column.to_string())
}

/// A write whose failure nobody sees is a lie about the stored state. This is
/// the shared "best-effort bookkeeping" sink: it never panics and never blocks
/// the response, but an error or an unexpected row count is logged with the
/// statement's purpose so the operator can find the divergence.
///
/// Use it only where the write genuinely is auxiliary. Anything the caller's
/// answer depends on must match on the `Result` itself and report the outcome.
pub fn report_write(result: Result<u64, sqlx::Error>, context: &str) -> bool {
    match result {
        Ok(rows) if rows == 1 => true,
        Ok(rows) => {
            warn!("[DB] {context} affected {rows} rows (expected 1) -- stored state may differ from what was reported");
            false
        }
        Err(err) => {
            warn!("[DB] {context} failed: {err}");
            false
        }
    }
}

/// Columns that SQLite files created before this schema existed do not have, as
/// `(table, column, ALTER statement)`. `init_db` applies each one through
/// `ensure_column`, so a column the routes need is either present or logged as
/// a failure -- never silently skipped.
///
/// The `messages`/`conversations` entries exist because inbound ingestion and
/// the outbound send path INSERT those columns; the original SQLite schema
/// predated them, so every one of those INSERTs failed at runtime.
///
/// The last six entries are the columns the Python models (`models/contact.py`,
/// `models/conversation.py`) declare and the Rust routes read:
/// `routes/contacts.rs:571` (projection list), `:3024`/`:3036`
/// (`contacts.customer_id`), `:635` (`contacts.custom_fields`),
/// `routes/conversations.rs:738` (`conversations.sequence_id`) and
/// `routes/services_sequences.rs:340`-`366`
/// (`conversations.sequence_id` + `conversations.sequence_variant`).
const COLUMN_MIGRATIONS: &[(&str, &str, &str)] = &[
    ("broadcasts", "total_recipients", "ALTER TABLE broadcasts ADD COLUMN total_recipients INTEGER DEFAULT 0"),
    ("audit_logs", "actor_email", "ALTER TABLE audit_logs ADD COLUMN actor_email TEXT"),
    ("audit_logs", "actor_name", "ALTER TABLE audit_logs ADD COLUMN actor_name TEXT"),
    ("audit_logs", "actor_role", "ALTER TABLE audit_logs ADD COLUMN actor_role TEXT"),
    ("audit_logs", "severity", "ALTER TABLE audit_logs ADD COLUMN severity TEXT DEFAULT 'info'"),
    ("audit_logs", "compliance_category", "ALTER TABLE audit_logs ADD COLUMN compliance_category TEXT DEFAULT 'access_control'"),
    ("audit_logs", "description", "ALTER TABLE audit_logs ADD COLUMN description TEXT"),
    ("audit_logs", "ip_address", "ALTER TABLE audit_logs ADD COLUMN ip_address TEXT DEFAULT '127.0.0.1'"),
    ("audit_logs", "user_agent", "ALTER TABLE audit_logs ADD COLUMN user_agent TEXT"),
    ("messages", "direction", "ALTER TABLE messages ADD COLUMN direction TEXT"),
    ("messages", "status", "ALTER TABLE messages ADD COLUMN status TEXT"),
    ("messages", "sent_at", "ALTER TABLE messages ADD COLUMN sent_at DATETIME"),
    ("messages", "is_private", "ALTER TABLE messages ADD COLUMN is_private BOOLEAN DEFAULT 0"),
    ("messages", "sender_type", "ALTER TABLE messages ADD COLUMN sender_type TEXT"),
    ("messages", "sender_name", "ALTER TABLE messages ADD COLUMN sender_name TEXT"),
    ("messages", "provider", "ALTER TABLE messages ADD COLUMN provider TEXT"),
    ("messages", "provider_message_id", "ALTER TABLE messages ADD COLUMN provider_message_id TEXT"),
    ("messages", "intent", "ALTER TABLE messages ADD COLUMN intent TEXT"),
    ("messages", "provider_metadata", "ALTER TABLE messages ADD COLUMN provider_metadata TEXT"),
    ("messages", "attachments", "ALTER TABLE messages ADD COLUMN attachments TEXT"),
    ("messages", "translated_bodies", "ALTER TABLE messages ADD COLUMN translated_bodies TEXT"),
    ("conversations", "created_at", "ALTER TABLE conversations ADD COLUMN created_at DATETIME"),
    ("conversations", "last_inbound_at", "ALTER TABLE conversations ADD COLUMN last_inbound_at DATETIME"),
    ("conversations", "last_outbound_at", "ALTER TABLE conversations ADD COLUMN last_outbound_at DATETIME"),
    ("conversations", "sequence_step", "ALTER TABLE conversations ADD COLUMN sequence_step INTEGER DEFAULT 0"),
    ("conversations", "next_followup_at", "ALTER TABLE conversations ADD COLUMN next_followup_at DATETIME"),
    ("conversations", "custom_attributes", "ALTER TABLE conversations ADD COLUMN custom_attributes TEXT"),
    ("conversations", "csat_comment", "ALTER TABLE conversations ADD COLUMN csat_comment TEXT"),
    ("conversations", "tenant_key", "ALTER TABLE conversations ADD COLUMN tenant_key TEXT"),
    ("conversations", "sequence_id", "ALTER TABLE conversations ADD COLUMN sequence_id TEXT"),
    ("conversations", "sequence_variant", "ALTER TABLE conversations ADD COLUMN sequence_variant TEXT"),
    ("contacts", "tenant_key", "ALTER TABLE contacts ADD COLUMN tenant_key TEXT"),
    ("contacts", "lead_id", "ALTER TABLE contacts ADD COLUMN lead_id TEXT"),
    ("contacts", "email", "ALTER TABLE contacts ADD COLUMN email TEXT"),
    ("contacts", "language", "ALTER TABLE contacts ADD COLUMN language TEXT"),
    ("contacts", "customer_id", "ALTER TABLE contacts ADD COLUMN customer_id TEXT"),
    ("contacts", "role", "ALTER TABLE contacts ADD COLUMN role TEXT NOT NULL DEFAULT ''"),
    ("contacts", "custom_fields", "ALTER TABLE contacts ADD COLUMN custom_fields TEXT NOT NULL DEFAULT '{}'"),
    ("contacts", "harvester_contact_point_id", "ALTER TABLE contacts ADD COLUMN harvester_contact_point_id TEXT NOT NULL DEFAULT ''"),
    // Each campaign recipient row points at the `messages` row the outbound
    // pipeline wrote, so a later webhook receipt can update the campaign.
    ("broadcast_recipients", "message_id", "ALTER TABLE broadcast_recipients ADD COLUMN message_id TEXT"),
    // Nullable on purpose: gap/recommendation rows written before tenancy existed
    // keep their history instead of being dropped or relabelled to whoever runs
    // the migration. Reads match `COALESCE(tenant_key,'') = ?`, so a NULL row is
    // "nobody's" rather than guessed at for every tenant.
    ("knowledge_gaps", "tenant_key", "ALTER TABLE knowledge_gaps ADD COLUMN tenant_key TEXT"),
    ("knowledge_recommendations", "tenant_key", "ALTER TABLE knowledge_recommendations ADD COLUMN tenant_key TEXT"),
];

/// SQLite has no `ADD COLUMN IF NOT EXISTS`. Reading the live column set with
/// `PRAGMA table_info` first means the ALTER is only issued when the column is
/// genuinely absent, so a duplicate-column error can no longer be mistaken for
/// a column that was never added -- and a missing table is logged as itself.
async fn ensure_column(pool: &DbPool, table: &str, column: &str, ddl: &str) {
    if !is_identifier(table) || !is_identifier(column) {
        warn!("refusing to migrate `{table}`.`{column}`: not a plain SQLite identifier");
        return;
    }
    if !table_exists(pool, table).await {
        warn!("cannot add column `{column}`: table `{table}` does not exist in this database");
        return;
    }
    let sql = format!("SELECT name FROM pragma_table_info('{table}')");
    let present: Vec<String> = match sqlx::query_scalar::<_, String>(&sql).fetch_all(pool).await {
        Ok(names) => names,
        Err(err) => {
            warn!("could not read PRAGMA table_info({table}) for `{column}`: {err}");
            return;
        }
    };
    if present.iter().any(|name| name == column) {
        return;
    }
    if let Err(err) = sqlx::query(ddl).execute(pool).await {
        warn!("ALTER TABLE {table} ADD COLUMN {column} failed: {err}");
    }
}

/// `PRAGMA`/`sqlite_master` names cannot be bound as parameters, so the
/// identifiers reaching the formatted queries below are whitelisted instead.
fn is_identifier(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 64
        && name
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
}

async fn table_exists(pool: &DbPool, table: &str) -> bool {
    match sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = ?",
    )
    .bind(table)
    .fetch_one(pool)
    .await
    {
        Ok(count) => count > 0,
        Err(err) => {
            warn!("could not check whether table `{table}` exists: {err}");
            false
        }
    }
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
            provider_type TEXT,
            idp_entity_id TEXT,
            idp_sso_url TEXT,
            idp_x509_cert TEXT,
            sp_entity_id TEXT,
            sp_acs_url TEXT,
            enforced_domains TEXT,
            allow_unlinked_passwords BOOLEAN,
            jit_provisioning_enabled BOOLEAN,
            default_role TEXT,
            attribute_mapping TEXT,
            last_tested_at DATETIME,
            last_test_status TEXT,
            last_test_message TEXT,
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

    // Fourteen more tables are read and written by the Rust routes (grep the
    // table names under `src/routes` and `src/services`) but were never created
    // by this file, so every endpoint touching one of them died at runtime with
    // `no such table`. Column sets and nullability come from the Python
    // SQLAlchemy models in `src/communicator/models/*.py`, mapped to SQLite
    // types: UUID/DateTime -> TEXT/DATETIME, JSONB -> TEXT, Float -> REAL,
    // Boolean/Integer -> INTEGER, String(n)/Text -> TEXT.
    //
    // DDL defaults are encoded only where the Python default is neutral (an
    // empty string, an empty JSON collection, `0`, `1`, or the few the port
    // brief named: `drafts.confidence` 0.0, `campaign_templates.is_active` 1,
    // `team_members.role` 'agent', `team_members.availability_status`
    // 'online', `team_members.max_concurrent_conversations` 5). Every `DEFAULT
    // ''` / `DEFAULT '{}'` / `DEFAULT '[]'` below mirrors the literal the Python
    // model declares (`default=""`, `default=dict`, `default=list`); no default
    // was invented for a column the model leaves value-free.
    //
    // Business-value Python defaults are NOT encoded: `tenant_key` "newax"
    // (customers, contact_segments, knowledge_base_entries, inquiries),
    // `author_name` "Agent" (contact_notes), `category` "preference"
    // (contact_memories), `source_type` "note" / `topic` "General" /
    // `status` "published" / `target_touchpoints` ["agent","copilot",
    // "help_center"] / `resolution_rate` 85.0 (knowledge_base_entries),
    // `frequency` 1 (knowledge_gaps), `impact_on_resolution` 3.5 /
    // `gap_type` "content_gap" / `topic` "General" / `status` "open"
    // (knowledge_recommendations), `meta_language` "en_US" / `meta_category`
    // "MARKETING" / `meta_status` "PENDING" (campaign_templates), and the
    // lifecycle statuses `drafts.status` "pending_review" and
    // `inquiries.status` "discovery_in_progress". Those columns are NOT NULL
    // without a default: the writer has to say what it means, instead of the
    // schema silently inventing a tenant, an author, a Meta template category or
    // a metric. Each Rust INSERT already names them (`routes/drafts.rs:146`,
    // `routes/knowledge_base.rs:152`, `routes/discovery.rs:557`), so nothing
    // that exists today depends on the missing default.
    //
    // Four columns deviate from the Python models because the Rust code that
    // touches them cannot work otherwise:
    //   * `team_members.created_at` and `drafts.updated_at` are nullable --
    //     `routes/team_routing.rs:218` and `routes/drafts.rs:146` INSERT without
    //     them, which a NOT NULL column would reject.
    //   * `contact_memories.confidence` is TEXT, not REAL: `routes/contacts.rs`
    //     inserts the literal `'1.0'` (line 2029) and decodes the column with
    //     `Option<String>` (lines 1077, 1905), so REAL affinity would make every
    //     memory read fail to decode.
    //   * `knowledge_base_entries.resolution_rate` is nullable although Python
    //     declares it NOT NULL: `routes/knowledge_base.rs:156` INSERTs the
    //     literal NULL (a brand-new entry has no measured resolution rate yet)
    //     and `KbRow` decodes it as `Option<f64>` (line 88), so NOT NULL would
    //     reject every knowledge write.
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS teams (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL UNIQUE,
            description TEXT NOT NULL DEFAULT '',
            auto_assign BOOLEAN NOT NULL,
            created_at DATETIME NOT NULL
        );

        CREATE TABLE IF NOT EXISTS team_members (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            email TEXT NOT NULL DEFAULT '',
            role TEXT NOT NULL DEFAULT 'agent',
            custom_permissions TEXT NOT NULL DEFAULT '[]',
            availability_status TEXT NOT NULL DEFAULT 'online',
            max_concurrent_conversations INTEGER NOT NULL DEFAULT 5,
            last_assigned_at DATETIME,
            created_at DATETIME
        );

        CREATE TABLE IF NOT EXISTS team_memberships (
            team_id TEXT NOT NULL REFERENCES teams (id) ON DELETE CASCADE,
            team_member_id TEXT NOT NULL REFERENCES team_members (id) ON DELETE CASCADE,
            PRIMARY KEY (team_id, team_member_id)
        );

        CREATE TABLE IF NOT EXISTS customers (
            id TEXT PRIMARY KEY,
            tenant_key TEXT NOT NULL,
            lead_id TEXT REFERENCES leads (id),
            display_name TEXT NOT NULL DEFAULT '',
            created_at DATETIME NOT NULL
        );

        CREATE TABLE IF NOT EXISTS contact_notes (
            id TEXT PRIMARY KEY,
            contact_id TEXT NOT NULL REFERENCES contacts (id) ON DELETE CASCADE,
            author_name TEXT NOT NULL,
            body TEXT NOT NULL,
            created_at DATETIME NOT NULL
        );

        CREATE TABLE IF NOT EXISTS contact_memories (
            id TEXT PRIMARY KEY,
            contact_id TEXT NOT NULL REFERENCES contacts (id),
            fact TEXT NOT NULL,
            category TEXT NOT NULL,
            confidence TEXT NOT NULL,
            source_message_id TEXT REFERENCES messages (id),
            noted_at DATETIME NOT NULL,
            created_at DATETIME NOT NULL
        );

        CREATE TABLE IF NOT EXISTS contact_segments (
            id TEXT PRIMARY KEY,
            tenant_key TEXT NOT NULL,
            name TEXT NOT NULL,
            description TEXT NOT NULL DEFAULT '',
            filters TEXT NOT NULL DEFAULT '{}',
            created_at DATETIME NOT NULL,
            updated_at DATETIME NOT NULL,
            CONSTRAINT uq_segment_tenant_name UNIQUE (tenant_key, name)
        );

        CREATE TABLE IF NOT EXISTS campaign_templates (
            id TEXT PRIMARY KEY,
            tenant_key TEXT NOT NULL,
            key TEXT NOT NULL,
            channel TEXT NOT NULL DEFAULT '',
            name TEXT NOT NULL,
            purpose TEXT NOT NULL DEFAULT '',
            meta_template_name TEXT NOT NULL,
            meta_language TEXT NOT NULL,
            meta_category TEXT NOT NULL,
            meta_status TEXT NOT NULL,
            body_preview TEXT NOT NULL DEFAULT '',
            variables TEXT NOT NULL DEFAULT '[]',
            is_active BOOLEAN NOT NULL DEFAULT 1,
            created_at DATETIME NOT NULL,
            updated_at DATETIME NOT NULL,
            UNIQUE (tenant_key, key)
        );

        CREATE TABLE IF NOT EXISTS discovery_flow_templates (
            id TEXT PRIMARY KEY,
            tenant_key TEXT NOT NULL UNIQUE,
            name TEXT NOT NULL,
            is_active BOOLEAN NOT NULL DEFAULT 1,
            steps TEXT NOT NULL DEFAULT '[]',
            problem_category_keywords TEXT NOT NULL DEFAULT '{}',
            capability_keywords TEXT NOT NULL DEFAULT '{}',
            capability_labels TEXT NOT NULL DEFAULT '{}',
            scoring_weights TEXT NOT NULL DEFAULT '{}',
            intro_message TEXT NOT NULL DEFAULT '',
            closing_message TEXT NOT NULL DEFAULT '',
            created_at DATETIME NOT NULL,
            updated_at DATETIME NOT NULL
        );

        CREATE TABLE IF NOT EXISTS drafts (
            id TEXT PRIMARY KEY,
            conversation_id TEXT NOT NULL REFERENCES conversations (id),
            body TEXT NOT NULL DEFAULT '',
            confidence REAL NOT NULL DEFAULT 0.0,
            citations TEXT NOT NULL DEFAULT '[]',
            status TEXT NOT NULL DEFAULT 'pending_review',
            reviewer TEXT NOT NULL DEFAULT '',
            reviewer_feedback TEXT NOT NULL DEFAULT '',
            created_at DATETIME NOT NULL,
            updated_at DATETIME
        );

        CREATE TABLE IF NOT EXISTS inquiries (
            id TEXT PRIMARY KEY,
            tenant_key TEXT NOT NULL,
            conversation_id TEXT NOT NULL UNIQUE REFERENCES conversations (id),
            contact_id TEXT NOT NULL REFERENCES contacts (id),
            lead_id TEXT REFERENCES leads (id),
            organization_name TEXT NOT NULL DEFAULT '',
            organization_status TEXT NOT NULL DEFAULT '',
            industry TEXT NOT NULL DEFAULT '',
            contact_role TEXT NOT NULL DEFAULT '',
            org_size_range TEXT NOT NULL DEFAULT '',
            locations_count TEXT NOT NULL DEFAULT '',
            problem_description TEXT NOT NULL DEFAULT '',
            problem_categories TEXT NOT NULL DEFAULT '[]',
            reported_impact TEXT NOT NULL DEFAULT '[]',
            frequency TEXT NOT NULL DEFAULT '',
            impact_level TEXT NOT NULL DEFAULT '',
            current_workaround TEXT NOT NULL DEFAULT '',
            limitations TEXT NOT NULL DEFAULT '[]',
            desired_outcome TEXT NOT NULL DEFAULT '',
            existing_systems TEXT NOT NULL DEFAULT '[]',
            integration_required TEXT NOT NULL DEFAULT '',
            capability_mapping TEXT NOT NULL DEFAULT '[]',
            timeline TEXT NOT NULL DEFAULT '',
            decision_process TEXT NOT NULL DEFAULT '',
            budget_range TEXT NOT NULL DEFAULT '',
            qualification_score INTEGER,
            qualification_factors TEXT NOT NULL DEFAULT '{}',
            status TEXT NOT NULL DEFAULT 'discovery_in_progress',
            assigned_owner TEXT NOT NULL DEFAULT '',
            next_action TEXT NOT NULL DEFAULT '',
            created_at DATETIME NOT NULL,
            updated_at DATETIME NOT NULL
        );

        CREATE TABLE IF NOT EXISTS knowledge_base_entries (
            id TEXT PRIMARY KEY,
            tenant_key TEXT NOT NULL,
            title TEXT NOT NULL,
            content TEXT NOT NULL DEFAULT '',
            source_type TEXT NOT NULL,
            source_url TEXT,
            file_name TEXT,
            topic TEXT NOT NULL,
            target_touchpoints TEXT NOT NULL,
            status TEXT NOT NULL,
            resolution_rate REAL,
            usage_count INTEGER NOT NULL DEFAULT 0,
            helpful_count INTEGER NOT NULL DEFAULT 0,
            unhelpful_count INTEGER NOT NULL DEFAULT 0,
            created_at DATETIME NOT NULL,
            updated_at DATETIME NOT NULL
        );

        CREATE TABLE IF NOT EXISTS knowledge_gaps (
            id TEXT PRIMARY KEY,
            question TEXT NOT NULL,
            suggested_answer TEXT NOT NULL DEFAULT '',
            source_query TEXT NOT NULL DEFAULT '',
            frequency INTEGER NOT NULL,
            status TEXT NOT NULL,
            created_at DATETIME NOT NULL
        );

        CREATE TABLE IF NOT EXISTS knowledge_recommendations (
            id TEXT PRIMARY KEY,
            gap_type TEXT NOT NULL DEFAULT 'content_gap',
            topic TEXT NOT NULL DEFAULT 'General',
            title TEXT NOT NULL,
            description TEXT NOT NULL DEFAULT '',
            impact_on_resolution REAL NOT NULL,
            suggested_action TEXT NOT NULL DEFAULT '',
            suggested_content TEXT NOT NULL DEFAULT '',
            status TEXT NOT NULL DEFAULT 'open',
            created_at DATETIME NOT NULL,
            resolved_at DATETIME
        );
        "#,
    )
    .execute(&pool)
    .await?;

    // Migration safety: add columns that older SQLite files predate. Every
    // statement goes through `ensure_column`, which checks PRAGMA table_info
    // first -- the previous `let _ = sqlx::query(...)` form discarded the
    // difference between "duplicate column", "no such table" and a real failure.
    for (table, column, ddl) in COLUMN_MIGRATIONS {
        ensure_column(&pool, table, column, ddl).await;
    }

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS consent_records (
            id TEXT PRIMARY KEY,
            contact_id TEXT NOT NULL,
            status TEXT NOT NULL,
            source TEXT NOT NULL DEFAULT '',
            note TEXT NOT NULL DEFAULT '',
            recorded_at DATETIME NOT NULL
        );",
    )
    .execute(&pool)
    .await?;

    // Operator-entered Meta conversation rates. Meta does not publish rates
    // through the messages API, so the pricing endpoint reads what the operator
    // recorded from their own invoice and refuses when nothing is recorded --
    // it never quotes a built-in table. There is deliberately no seed.
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS whatsapp_pricing_rates (
            region TEXT NOT NULL,
            category TEXT NOT NULL,
            currency TEXT NOT NULL,
            rate_per_conversation REAL NOT NULL,
            source TEXT NOT NULL DEFAULT '',
            updated_at DATETIME NOT NULL,
            PRIMARY KEY (region, category)
        );",
    )
    .execute(&pool)
    .await?;

    // Channel inboxes hold the per-tenant provider credentials that the inbound
    // webhooks authenticate with. The table was declared by every /inboxes route
    // but never created, so inbox reads and webhook credential lookups both failed.
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS inboxes (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            channel_type TEXT NOT NULL,
            phone_number TEXT NOT NULL DEFAULT '',
            avatar_url TEXT NOT NULL DEFAULT '',
            is_active BOOLEAN NOT NULL DEFAULT 1,
            credentials TEXT NOT NULL DEFAULT '{}',
            greeting_enabled BOOLEAN NOT NULL DEFAULT 1,
            greeting_message TEXT NOT NULL DEFAULT '',
            out_of_office_enabled BOOLEAN NOT NULL DEFAULT 0,
            out_of_office_message TEXT NOT NULL DEFAULT '',
            csat_enabled BOOLEAN NOT NULL DEFAULT 1,
            auto_read_receipts BOOLEAN NOT NULL DEFAULT 1,
            working_hours_enabled BOOLEAN NOT NULL DEFAULT 0,
            working_hours_start INTEGER NOT NULL DEFAULT 9,
            working_hours_end INTEGER NOT NULL DEFAULT 18,
            timezone TEXT NOT NULL DEFAULT 'UTC',
            tenant_key TEXT,
            created_at DATETIME NOT NULL,
            updated_at DATETIME NOT NULL
        );",
    )
    .execute(&pool)
    .await?;

    // Social/email/voice channels provision a Lead alongside the Contact, so the
    // contacts.lead_id foreign target has to exist.
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS leads (
            id TEXT PRIMARY KEY,
            tenant_key TEXT,
            harvester_institution_id TEXT NOT NULL DEFAULT '',
            name TEXT NOT NULL DEFAULT '',
            institution_type TEXT NOT NULL DEFAULT '',
            city TEXT NOT NULL DEFAULT '',
            qualification_rationale TEXT NOT NULL DEFAULT '',
            synced_at DATETIME,
            created_at DATETIME NOT NULL,
            updated_at DATETIME NOT NULL
        );",
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS live_proof_sessions (
            id TEXT PRIMARY KEY,
            tenant_id TEXT NOT NULL,
            organization_id TEXT,
            conversation_id TEXT,
            contact_id TEXT,
            order_id TEXT,
            customer_identifier TEXT NOT NULL DEFAULT '',
            claim_type TEXT NOT NULL DEFAULT 'refund',
            session_token TEXT NOT NULL UNIQUE,
            status TEXT NOT NULL DEFAULT 'pending',
            anti_spoof_nonce TEXT NOT NULL DEFAULT '',
            max_video_seconds INTEGER NOT NULL DEFAULT 45,
            require_video BOOLEAN NOT NULL DEFAULT 1,
            require_photos BOOLEAN NOT NULL DEFAULT 1,
            expires_at DATETIME NOT NULL,
            completed_at DATETIME,
            rejection_reason TEXT,
            created_at DATETIME NOT NULL,
            updated_at DATETIME NOT NULL
        );",
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS live_proof_evidences (
            id TEXT PRIMARY KEY,
            session_id TEXT NOT NULL,
            tenant_id TEXT NOT NULL,
            media_type TEXT NOT NULL,
            file_path TEXT NOT NULL,
            file_size_bytes INTEGER NOT NULL,
            mime_type TEXT NOT NULL,
            sha256_hash TEXT NOT NULL,
            duration_seconds REAL,
            resolution TEXT,
            device_fingerprint TEXT,
            watermark_verified BOOLEAN NOT NULL,
            verification_score REAL NOT NULL,
            verification_checks TEXT,
            captured_at DATETIME NOT NULL
        );",
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS translation_settings (
            id TEXT PRIMARY KEY,
            is_enabled BOOLEAN NOT NULL DEFAULT 0,
            auto_translate_inbound BOOLEAN NOT NULL DEFAULT 0,
            auto_translate_outbound BOOLEAN NOT NULL DEFAULT 0,
            agent_primary_language TEXT NOT NULL DEFAULT '',
            preferred_engine TEXT NOT NULL DEFAULT '',
            confidence_threshold REAL NOT NULL DEFAULT 0.8,
            enabled_languages TEXT NOT NULL DEFAULT '[]',
            created_at DATETIME NOT NULL,
            updated_at DATETIME NOT NULL
        );",
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS integration_settings (
            id TEXT PRIMARY KEY,
            tenant_key TEXT,
            whatsapp_provider TEXT NOT NULL DEFAULT 'meta',
            meta_access_token TEXT NOT NULL DEFAULT '',
            meta_phone_number_id TEXT NOT NULL DEFAULT '',
            meta_waba_id TEXT NOT NULL DEFAULT '',
            meta_app_secret TEXT NOT NULL DEFAULT '',
            meta_webhook_verify_token TEXT NOT NULL DEFAULT '',
            whatsapp_verified_at DATETIME,
            whatsapp_verified_name TEXT NOT NULL DEFAULT '',
            whatsapp_display_number TEXT NOT NULL DEFAULT '',
            business_name TEXT NOT NULL DEFAULT '',
            business_description TEXT NOT NULL DEFAULT '',
            agent_guidelines TEXT NOT NULL DEFAULT '',
            auto_reply_enabled BOOLEAN NOT NULL DEFAULT 1,
            business_hours_enabled BOOLEAN NOT NULL DEFAULT 0,
            business_hours_start INTEGER NOT NULL DEFAULT 9,
            business_hours_end INTEGER NOT NULL DEFAULT 18,
            business_timezone TEXT NOT NULL DEFAULT 'UTC',
            away_message TEXT NOT NULL DEFAULT '',
            updated_at DATETIME NOT NULL
        );",
    )
    .execute(&pool)
    .await?;

    // Cross-repo events are durable before they are dispatched, so a crash
    // mid-handler cannot lose the event and a replay can be deduplicated.
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ecosystem_events (
            id TEXT PRIMARY KEY,
            event_name TEXT NOT NULL,
            source TEXT NOT NULL DEFAULT '',
            tenant_id TEXT NOT NULL DEFAULT '',
            payload TEXT NOT NULL DEFAULT '{}',
            outcome TEXT NOT NULL DEFAULT '',
            received_at DATETIME NOT NULL
        );",
    )
    .execute(&pool)
    .await?;

    // Outbound mesh events that the relay has not accepted yet. A row here is
    // the only honest meaning of "queued for retry"; it is written with the
    // sealed envelope so a flush replays exactly what was originally signed.
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS ecosystem_egress_outbox (
            id TEXT PRIMARY KEY,
            event_name TEXT NOT NULL,
            recipient_id TEXT NOT NULL,
            tenant_id TEXT NOT NULL DEFAULT '',
            organization_id TEXT,
            correlation_id TEXT NOT NULL DEFAULT '',
            envelope TEXT NOT NULL,
            status TEXT NOT NULL DEFAULT 'pending',
            attempts INTEGER NOT NULL DEFAULT 0,
            last_error TEXT NOT NULL DEFAULT '',
            created_at DATETIME NOT NULL,
            sent_at DATETIME
        );",
    )
    .execute(&pool)
    .await?;

    // Widget visitor sessions: the cursor, presence and close state a customer
    // chat runtime needs. Nothing is seeded; a row only exists once a real
    // visitor has been registered.
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS widget_sessions (
            id TEXT PRIMARY KEY,
            token_hash TEXT NOT NULL UNIQUE,
            conversation_id TEXT NOT NULL,
            contact_id TEXT NOT NULL,
            inbox_id TEXT NOT NULL,
            website_token TEXT,
            origin TEXT,
            client_ip TEXT,
            state TEXT NOT NULL DEFAULT 'open',
            last_cursor INTEGER NOT NULL DEFAULT 0,
            visitor_presence TEXT NOT NULL DEFAULT 'unknown',
            last_presence_at DATETIME,
            last_typing_at DATETIME,
            closed_at DATETIME,
            close_reason TEXT,
            created_at DATETIME NOT NULL,
            updated_at DATETIME NOT NULL
        );",
    )
    .execute(&pool)
    .await?;

    // Agent typing/presence signals, written only when an agent action actually
    // happens in this backend.
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS agent_presence (
            id TEXT PRIMARY KEY,
            conversation_id TEXT NOT NULL,
            agent_id TEXT,
            agent_name TEXT NOT NULL,
            agent_avatar TEXT,
            action TEXT NOT NULL,
            last_seen_at DATETIME NOT NULL,
            updated_at DATETIME NOT NULL
        );",
    )
    .execute(&pool)
    .await?;

    for statement in [
        "CREATE INDEX IF NOT EXISTS ix_consent_records_contact_id ON consent_records (contact_id)",
        "CREATE INDEX IF NOT EXISTS ix_messages_provider_message_id ON messages (provider_message_id)",
        "CREATE INDEX IF NOT EXISTS ix_inboxes_channel_type ON inboxes (channel_type)",
        "CREATE INDEX IF NOT EXISTS ix_contacts_channel_value ON contacts (channel, value)",
        "CREATE INDEX IF NOT EXISTS ix_live_proof_sessions_tenant_id ON live_proof_sessions (tenant_id)",
        "CREATE INDEX IF NOT EXISTS ix_ecosystem_events_event_name ON ecosystem_events (event_name)",
        "CREATE INDEX IF NOT EXISTS ix_live_proof_evidences_session_id ON live_proof_evidences (session_id)",
        "CREATE INDEX IF NOT EXISTS ix_live_proof_evidences_sha256 ON live_proof_evidences (sha256_hash)",
        "CREATE INDEX IF NOT EXISTS idx_widget_sessions_conversation ON widget_sessions (conversation_id, state)",
        "CREATE UNIQUE INDEX IF NOT EXISTS idx_agent_presence_key ON agent_presence (conversation_id, agent_name)",
        // Each index below answers a WHERE / ORDER BY that was read in the Rust
        // sources: teams.name and team_members.email/name/role are the only
        // lookup keys the routing, macro and admin paths use.
        "CREATE INDEX IF NOT EXISTS ix_team_members_email ON team_members (email)",
        "CREATE INDEX IF NOT EXISTS ix_team_members_name ON team_members (name)",
        "CREATE INDEX IF NOT EXISTS ix_team_members_role ON team_members (role)",
        // `DELETE FROM team_memberships WHERE team_member_id = ?` cannot use the
        // (team_id, team_member_id) primary key, which is team_id-first.
        "CREATE INDEX IF NOT EXISTS ix_team_memberships_team_member_id ON team_memberships (team_member_id)",
        "CREATE INDEX IF NOT EXISTS ix_contact_notes_contact_id ON contact_notes (contact_id)",
        "CREATE INDEX IF NOT EXISTS ix_contact_memories_contact_id ON contact_memories (contact_id)",
        "CREATE INDEX IF NOT EXISTS ix_drafts_conversation_id ON drafts (conversation_id)",
        "CREATE INDEX IF NOT EXISTS ix_drafts_status ON drafts (status)",
        "CREATE INDEX IF NOT EXISTS ix_inquiries_tenant_key ON inquiries (tenant_key)",
        "CREATE INDEX IF NOT EXISTS ix_inquiries_contact_id ON inquiries (contact_id)",
        "CREATE INDEX IF NOT EXISTS ix_inquiries_status ON inquiries (status)",
        "CREATE INDEX IF NOT EXISTS ix_knowledge_base_entries_topic ON knowledge_base_entries (topic)",
        "CREATE INDEX IF NOT EXISTS ix_knowledge_base_entries_status ON knowledge_base_entries (status)",
        "CREATE INDEX IF NOT EXISTS ix_knowledge_gaps_status ON knowledge_gaps (status)",
        "CREATE INDEX IF NOT EXISTS ix_knowledge_recommendations_status ON knowledge_recommendations (status, gap_type)",
        // `SELECT id FROM contacts WHERE customer_id = ?` in the relationship
        // profile path.
        "CREATE INDEX IF NOT EXISTS ix_contacts_customer_id ON contacts (customer_id)",
    ] {
        if let Err(err) = sqlx::query(statement).execute(&pool).await {
            warn!("index statement `{statement}` failed: {err}");
        }
    }

    purge_demo_rows(&pool).await;

    info!("Database schema initialized successfully.");
    Ok(pool)
}

/// The demo-content seeders are gone from the codebase, but the rows they had
/// already written still present themselves as real campaigns, services and
/// automations. Deletion is keyed on the seeders' own literal names and ids, so
/// anything an operator created for real survives.
async fn purge_demo_rows(pool: &DbPool) {
    const SEEDED_AUTOMATIONS: &[&str] = &[
        "Main Interactive Menu",
        "Live Human Support Handoff",
        "Human Agent Alias (Operator)",
        "Services Catalog Quick View",
        "Pricing & Rates Inquiry",
        "Book a Meeting / Demo CTA",
        "Office Location & Timings",
        "Post-Resolution Feedback / CSAT Survey",
        "High-Intent Lead Auto-Tagging",
        "VIP Customer Auto-Tagging",
        "Opt-Out Compliance (STOP)",
    ];
    const SEEDED_BROADCASTS: &[&str] = &[
        "Spring sale early access",
        "New store opening",
        "Loyalty points reminder",
    ];
    const SEEDED_CANNED: &[&str] = &["greet", "pricing", "hours", "refund", "followup", "closing"];
    const SEEDED_MACROS: &[&str] = &[
        "Resolve Ticket + Send CSAT + Tag 'Resolved'",
        "Escalate to Tier 2 + Mark Urgent + Assign Support Team",
        "Send Password Reset Link + Snooze 24h",
        "VIP Refund Approved + Assign Billing",
    ];
    const SEEDED_SLA_POLICIES: &[&str] = &[
        "Critical VIP Escalation SLA",
        "High Priority SLA",
        "Standard Customer Support SLA",
    ];
    const SEEDED_GATEWAYS: &[&str] = &[
        "Twilio SMS Global",
        "Vonage SMS (Nexmo)",
        "MessageBird Enterprise SMS",
        "SendGrid Production Mailer",
        "Postmark Transactional",
        "Custom Corporate SMTP",
        "Twilio Voice & WebRTC",
        "Telnyx SIP & Voice",
        "Standard SIP Trunk (Asterisk/FreeSWITCH)",
    ];
    const SEEDED_WEBHOOK_ENDPOINTS: &[&str] = &[
        "Enterprise CRM & Data Warehouse Sync",
        "Slack Incident Alert Dispatcher",
    ];
    const SEEDED_TEAMS: &[&str] = &["Sales & Growth", "Customer Support", "VIP Accounts"];

    async fn delete_in(
        pool: &DbPool,
        table: &str,
        sql_head: &str,
        sql_tail: &str,
        values: &[&str],
        label: &str,
    ) {
        if values.is_empty() {
            return;
        }
        if !table_exists(pool, table).await {
            return;
        }
        let placeholders = vec!["?"; values.len()].join(", ");
        let sql = format!("{sql_head} IN ({placeholders}){sql_tail}");
        let mut query = sqlx::query(&sql);
        for value in values {
            query = query.bind(value);
        }
        match query.execute(pool).await {
            Ok(result) => {
                let removed = result.rows_affected();
                if removed > 0 {
                    info!("purged {removed} seeded {label} rows");
                }
            }
            Err(err) => warn!("could not purge seeded {label} rows: {err}"),
        }
    }

    // Children first: recipients reference the fabricated campaigns.
    delete_in(
        pool,
        "broadcast_recipients",
        "DELETE FROM broadcast_recipients WHERE broadcast_id IN (SELECT id FROM broadcasts WHERE name",
        ")",
        SEEDED_BROADCASTS,
        "broadcast recipients",
    )
    .await;
    delete_in(
        pool,
        "broadcasts",
        "DELETE FROM broadcasts WHERE name",
        "",
        SEEDED_BROADCASTS,
        "broadcasts",
    )
    .await;
    delete_in(
        pool,
        "sequence_steps",
        "DELETE FROM sequence_steps WHERE sequence_id IN ('seq-1', 'seq-2') OR template",
        "",
        &["welcome_lead", "case_study_lead"],
        "sequence steps",
    )
    .await;
    delete_in(
        pool,
        "follow_up_sequences",
        "DELETE FROM follow_up_sequences WHERE id IN ('seq-1', 'seq-2') OR name",
        "",
        &[
            "New Inbound Lead Onboarding",
            "Enterprise Re-Engagement Sequence",
        ],
        "follow-up sequences",
    )
    .await;
    delete_in(
        pool,
        "services",
        "DELETE FROM services WHERE id IN ('srv-1', 'srv-2') OR name",
        "",
        &[
            "Enterprise Omnichannel Support",
            "AI Copilot & Autonomous Routing",
        ],
        "services",
    )
    .await;
    delete_in(
        pool,
        "automation_rules",
        "DELETE FROM automation_rules WHERE name",
        "",
        SEEDED_AUTOMATIONS,
        "automation rules",
    )
    .await;
    delete_in(
        pool,
        "canned_responses",
        "DELETE FROM canned_responses WHERE LOWER(short_code)",
        "",
        SEEDED_CANNED,
        "canned responses",
    )
    .await;
    delete_in(
        pool,
        "macros",
        "DELETE FROM macros WHERE name",
        "",
        SEEDED_MACROS,
        "macros",
    )
    .await;
    delete_in(
        pool,
        "sla_policies",
        "DELETE FROM sla_policies WHERE name",
        "",
        SEEDED_SLA_POLICIES,
        "SLA policies",
    )
    .await;
    delete_in(
        pool,
        "channel_gateway_connections",
        "DELETE FROM channel_gateway_connections WHERE name",
        "",
        SEEDED_GATEWAYS,
        "gateway connections",
    )
    .await;
    delete_in(
        pool,
        "webhook_endpoints",
        "DELETE FROM webhook_endpoints WHERE name",
        "",
        SEEDED_WEBHOOK_ENDPOINTS,
        "webhook endpoints",
    )
    .await;
    delete_in(
        pool,
        "teams",
        "DELETE FROM teams WHERE name",
        "",
        SEEDED_TEAMS,
        "teams",
    )
    .await;
    delete_in(
        pool,
        "stripe_configs",
        "DELETE FROM stripe_configs WHERE account_name",
        "",
        &["Primary Stripe Account"],
        "stripe configs",
    )
    .await;
    // The SSO read path used to insert a singleton row (the Python SINGLETON_ID)
    // whose columns were only ever the DDL's literal defaults. A row on that id
    // with no IdP values was never an operator's choice.
    delete_in(
        pool,
        "sso_configs",
        "DELETE FROM sso_configs WHERE COALESCE(idp_sso_url, '') = '' AND COALESCE(idp_entity_id, '') = '' AND id",
        "",
        &["00000000-0000-0000-0000-000000000001"],
        "SSO config shells",
    )
    .await;
}
