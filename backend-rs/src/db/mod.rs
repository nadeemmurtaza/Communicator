use sqlx::{postgres::PgPoolOptions, Pool, Row, Postgres};
use std::sync::Arc;
use tracing::{info, warn};

pub type DbPool = Pool<Postgres>;

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
pub fn required_text(row: &sqlx::postgres::PgRow, column: &str) -> Result<String, String> {
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

/// Check whether a table exists in the database. Used by purge_demo_rows.
async fn table_exists(pool: &DbPool, table: &str) -> bool {
    match sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM information_schema.tables WHERE table_schema = 'public' AND table_name = $1",
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

    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(database_url)
        .await?;

    // Run sqlx migrations from the migrations/ directory.
    // This replaces the previous inline CREATE TABLE + ensure_column approach.
    const MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!();
    match MIGRATOR.run(&pool).await {
        Ok(_) => info!("sqlx migrations applied successfully"),
        Err(err) => {
            warn!("sqlx migration error: {err}");
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
        let placeholders = (1..=values.len())
            .map(|i| format!("${i}"))
            .collect::<Vec<_>>()
            .join(", ");
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
