//! Unified Inbox (Phase 19) — customer-centric view merging conversations
//! across channels, with manual/auto channel-identity resolution.

use crate::db::AppState;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Data types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct ConversationSummary {
    pub id: String,
    pub channel: Option<String>,
    pub state: String,
    pub priority: Option<String>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct UnifiedInboxEntry {
    pub customer_id: String,
    pub display_name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub channels: Vec<String>,
    pub conversation_count: i64,
    pub open_count: i64,
    pub last_activity_at: Option<DateTime<Utc>>,
    pub conversations: Vec<ConversationSummary>,
}

#[derive(Debug, Clone, Serialize)]
pub struct UnifiedInboxSummary {
    pub customers: Vec<UnifiedInboxEntry>,
    pub total: i64,
    pub limit: i32,
    pub offset: i32,
}

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct CustomerMapping {
    pub id: String,
    pub customer_id: String,
    pub contact_id: String,
    pub channel: String,
    pub channel_identifier: String,
    pub confidence: f32,
    pub resolved_by: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UnifiedInboxSettings {
    pub id: String,
    pub organization_id: String,
    pub enabled: bool,
    pub group_by: String,
    pub merge_timeline: bool,
    pub show_channel_badges: bool,
    pub auto_resolve: bool,
}

#[derive(FromRow)]
struct CustomerRow {
    id: String,
    display_name: String,
    email: Option<String>,
    phone: Option<String>,
}

// ---------------------------------------------------------------------------
// Inbox listing
// ---------------------------------------------------------------------------

/// List customers that have at least one conversation, grouped with their
/// conversation set, newest activity first. Filters are optional.
pub async fn get_unified_inbox_summary(
    state: &AppState,
    limit: i32,
    offset: i32,
    channel: Option<&str>,
    conv_state: Option<&str>,
    search: Option<&str>,
) -> Result<UnifiedInboxSummary, sqlx::Error> {
    let pool = &state.db;

    // Matching customer ids, newest activity first, with pagination.
    let customer_ids: Vec<(String,)> = sqlx::query_as(
        r#"SELECT cu.id
           FROM customers cu
           JOIN contacts ct ON ct.customer_id = cu.id
           JOIN conversations co ON co.contact_id = ct.id
           WHERE ($1::text IS NULL OR co.channel = $1)
             AND ($2::text IS NULL OR co.state = $2)
             AND ($3::text IS NULL OR cu.display_name ILIKE '%' || $3 || '%' OR cu.email ILIKE '%' || $3 || '%')
           GROUP BY cu.id
           ORDER BY MAX(co.updated_at) DESC NULLS LAST
           LIMIT $4 OFFSET $5"#,
    )
    .bind(channel)
    .bind(conv_state)
    .bind(search)
    .bind(limit as i64)
    .bind(offset as i64)
    .fetch_all(pool)
    .await?;

    let total: i64 = sqlx::query_scalar(
        r#"SELECT COUNT(DISTINCT cu.id)
           FROM customers cu
           JOIN contacts ct ON ct.customer_id = cu.id
           JOIN conversations co ON co.contact_id = ct.id
           WHERE ($1::text IS NULL OR co.channel = $1)
             AND ($2::text IS NULL OR co.state = $2)
             AND ($3::text IS NULL OR cu.display_name ILIKE '%' || $3 || '%' OR cu.email ILIKE '%' || $3 || '%')"#,
    )
    .bind(channel)
    .bind(conv_state)
    .bind(search)
    .fetch_one(pool)
    .await?;

    let mut customers = Vec::with_capacity(customer_ids.len());
    for (customer_id,) in customer_ids {
        customers.push(load_customer_entry(pool, &customer_id).await?);
    }

    Ok(UnifiedInboxSummary {
        customers,
        total,
        limit,
        offset,
    })
}

/// Get the unified view (customer + merged conversation timeline) for a
/// single customer. Returns `sqlx::Error::RowNotFound` if the customer
/// doesn't exist.
pub async fn get_unified_customer(
    state: &AppState,
    customer_id: &str,
) -> Result<UnifiedInboxEntry, sqlx::Error> {
    load_customer_entry(&state.db, customer_id).await
}

async fn load_customer_entry(
    pool: &sqlx::PgPool,
    customer_id: &str,
) -> Result<UnifiedInboxEntry, sqlx::Error> {
    let customer: CustomerRow = sqlx::query_as(
        "SELECT id, display_name, email, phone FROM customers WHERE id = $1",
    )
    .bind(customer_id)
    .fetch_one(pool)
    .await?;

    let conversations: Vec<ConversationSummary> = sqlx::query_as(
        r#"SELECT co.id, co.channel, co.state, co.priority, co.updated_at
           FROM conversations co
           JOIN contacts ct ON co.contact_id = ct.id
           WHERE ct.customer_id = $1
           ORDER BY co.updated_at DESC"#,
    )
    .bind(customer_id)
    .fetch_all(pool)
    .await?;

    let mut channels: Vec<String> = conversations
        .iter()
        .filter_map(|c| c.channel.clone())
        .collect();
    channels.sort();
    channels.dedup();

    let open_count = conversations
        .iter()
        .filter(|c| c.state != "resolved" && c.state != "closed")
        .count() as i64;
    let last_activity_at = conversations.first().map(|c| c.updated_at);

    Ok(UnifiedInboxEntry {
        customer_id: customer.id,
        display_name: customer.display_name,
        email: customer.email,
        phone: customer.phone,
        channels,
        conversation_count: conversations.len() as i64,
        open_count,
        last_activity_at,
        conversations,
    })
}

// ---------------------------------------------------------------------------
// Channel identity resolution
// ---------------------------------------------------------------------------

/// Look up a known customer mapping for a channel + channel-specific
/// identifier (e.g. a WhatsApp phone number or an email address).
pub async fn resolve_customer(
    state: &AppState,
    channel: &str,
    identifier: &str,
) -> Result<Option<CustomerMapping>, sqlx::Error> {
    sqlx::query_as(
        r#"SELECT id, customer_id, contact_id, channel, channel_identifier, confidence, resolved_by
           FROM unified_customer_mapping
           WHERE channel = $1 AND channel_identifier = $2"#,
    )
    .bind(channel)
    .bind(identifier)
    .fetch_optional(&state.db)
    .await
}

/// Create (or replace) a manual mapping between a customer, a contact, and
/// a channel identifier. Manual mappings are trusted at full confidence.
pub async fn create_customer_mapping(
    state: &AppState,
    customer_id: &str,
    contact_id: &str,
    channel: &str,
    channel_identifier: &str,
) -> Result<CustomerMapping, sqlx::Error> {
    let id = Uuid::new_v4().to_string();

    sqlx::query(
        r#"INSERT INTO unified_customer_mapping (id, customer_id, contact_id, channel, channel_identifier, confidence, resolved_by)
           VALUES ($1, $2, $3, $4, $5, 1.0, 'manual')
           ON CONFLICT (contact_id, channel) DO UPDATE
             SET customer_id = EXCLUDED.customer_id,
                 channel_identifier = EXCLUDED.channel_identifier,
                 confidence = 1.0,
                 resolved_by = 'manual',
                 resolved_at = NOW()"#,
    )
    .bind(&id)
    .bind(customer_id)
    .bind(contact_id)
    .bind(channel)
    .bind(channel_identifier)
    .execute(&state.db)
    .await?;

    sqlx::query_as(
        r#"SELECT id, customer_id, contact_id, channel, channel_identifier, confidence, resolved_by
           FROM unified_customer_mapping
           WHERE contact_id = $1 AND channel = $2"#,
    )
    .bind(contact_id)
    .bind(channel)
    .fetch_one(&state.db)
    .await
}

// ---------------------------------------------------------------------------
// Settings
// ---------------------------------------------------------------------------

/// Get an organization's unified inbox settings, creating the default row
/// the first time it's requested.
pub async fn get_unified_inbox_settings(
    state: &AppState,
    organization_id: &str,
) -> Result<UnifiedInboxSettings, sqlx::Error> {
    let existing: Option<UnifiedInboxSettings> = sqlx::query_as(
        r#"SELECT id, organization_id, enabled, group_by, merge_timeline, show_channel_badges, auto_resolve
           FROM unified_inbox_settings WHERE organization_id = $1"#,
    )
    .bind(organization_id)
    .fetch_optional(&state.db)
    .await?;

    if let Some(settings) = existing {
        return Ok(settings);
    }

    let id = Uuid::new_v4().to_string();
    sqlx::query(
        r#"INSERT INTO unified_inbox_settings (id, organization_id)
           VALUES ($1, $2)"#,
    )
    .bind(&id)
    .bind(organization_id)
    .execute(&state.db)
    .await?;

    sqlx::query_as(
        r#"SELECT id, organization_id, enabled, group_by, merge_timeline, show_channel_badges, auto_resolve
           FROM unified_inbox_settings WHERE organization_id = $1"#,
    )
    .bind(organization_id)
    .fetch_one(&state.db)
    .await
}

/// Update an organization's unified inbox settings from a partial JSON
/// payload. Only the fields present in `payload` are changed.
pub async fn update_unified_inbox_settings(
    state: &AppState,
    organization_id: &str,
    payload: &serde_json::Value,
) -> Result<UnifiedInboxSettings, sqlx::Error> {
    // Make sure a row exists before trying to patch it.
    get_unified_inbox_settings(state, organization_id).await?;

    let enabled = payload.get("enabled").and_then(|v| v.as_bool());
    let group_by = payload.get("group_by").and_then(|v| v.as_str());
    let merge_timeline = payload.get("merge_timeline").and_then(|v| v.as_bool());
    let show_channel_badges = payload.get("show_channel_badges").and_then(|v| v.as_bool());
    let auto_resolve = payload.get("auto_resolve").and_then(|v| v.as_bool());

    let mut sets: Vec<String> = Vec::new();
    let mut idx = 1;
    if enabled.is_some() {
        sets.push(format!("enabled = ${}", idx));
        idx += 1;
    }
    if group_by.is_some() {
        sets.push(format!("group_by = ${}", idx));
        idx += 1;
    }
    if merge_timeline.is_some() {
        sets.push(format!("merge_timeline = ${}", idx));
        idx += 1;
    }
    if show_channel_badges.is_some() {
        sets.push(format!("show_channel_badges = ${}", idx));
        idx += 1;
    }
    if auto_resolve.is_some() {
        sets.push(format!("auto_resolve = ${}", idx));
        idx += 1;
    }

    if !sets.is_empty() {
        sets.push("updated_at = NOW()".to_string());
        let query = format!(
            "UPDATE unified_inbox_settings SET {} WHERE organization_id = ${}",
            sets.join(", "),
            idx
        );

        let mut q = sqlx::query(&query);
        if let Some(v) = enabled {
            q = q.bind(v);
        }
        if let Some(v) = group_by {
            q = q.bind(v);
        }
        if let Some(v) = merge_timeline {
            q = q.bind(v);
        }
        if let Some(v) = show_channel_badges {
            q = q.bind(v);
        }
        if let Some(v) = auto_resolve {
            q = q.bind(v);
        }
        q = q.bind(organization_id);

        q.execute(&state.db).await?;
    }

    get_unified_inbox_settings(state, organization_id).await
}
