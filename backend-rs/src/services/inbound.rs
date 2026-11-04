#![allow(dead_code)]

use crate::db::DbPool;
use crate::services::compliance::{self, ConsentStatus};
use sqlx::Row;

const TERMINAL_STATES: [&str; 3] = ["opted_out", "closed_won", "closed_lost"];

#[derive(Debug)]
pub enum InboundError {
    UnknownContact(String),
    Db(sqlx::Error),
}

impl From<sqlx::Error> for InboundError {
    fn from(err: sqlx::Error) -> Self {
        Self::Db(err)
    }
}

pub struct InboundMessage<'a> {
    pub channel: &'a str,
    pub from_value: &'a str,
    pub body: &'a str,
    pub provider: &'a str,
    pub provider_message_id: &'a str,
    pub sender_name: &'a str,
}

struct ResolvedContact {
    id: String,
    tenant_key: Option<String>,
}

/// Placeholder names written by earlier import passes; a real display name from
/// the channel payload replaces them, but a genuine human name is never
/// overwritten by whatever the next payload happens to claim.
fn is_placeholder_name(name: &str) -> bool {
    name.is_empty()
        || name.starts_with("Contact ")
        || name.starts_with("Customer ")
        || name.ends_with(" User")
        || name.ends_with(" Lead")
        || (name.contains(" Contact ")
            && name.chars().next_back().is_some_and(|c| c.is_ascii_digit()))
}

async fn find_contact(
    pool: &DbPool,
    channel: &str,
    value: &str,
    sender_name: &str,
) -> Result<ResolvedContact, InboundError> {    let row = sqlx::query("SELECT id, person_name, tenant_key FROM contacts WHERE channel = ? AND value = ? LIMIT 1")
        .bind(channel)
        .bind(value)
        .fetch_optional(pool)
        .await?;

    let Some(row) = row else {
        return Err(InboundError::UnknownContact(format!(
            "No Contact found for {channel}:{value}"
        )));
    };

    let id: String = row.try_get("id")?;
    let person_name: Option<String> = row.try_get("person_name").ok().flatten();
    let tenant_key: Option<String> = row.try_get("tenant_key").ok().flatten();

    let trimmed = sender_name.trim();
    if !trimmed.is_empty() && person_name.as_deref().map(is_placeholder_name).unwrap_or(true) {
        sqlx::query("UPDATE contacts SET person_name = ?, updated_at = ? WHERE id = ?")
            .bind(trimmed)
            .bind(chrono::Utc::now().to_rfc3339())
            .bind(&id)
            .execute(pool)
            .await?;
    }

    Ok(ResolvedContact { id, tenant_key })
}

async fn get_or_create_conversation(
    pool: &DbPool,
    contact: &ResolvedContact,
    channel: &str,
) -> Result<(String, String), InboundError> {
    let existing = sqlx::query(
        "SELECT id, state FROM conversations WHERE contact_id = ? \
         AND state NOT IN ('opted_out', 'closed_won', 'closed_lost') \
         ORDER BY created_at DESC LIMIT 1",
    )
    .bind(&contact.id)
    .fetch_optional(pool)
    .await?;

    if let Some(row) = existing {
        return Ok((
            row.try_get("id")?,
            row.try_get::<String, _>("state").unwrap_or_default(),
        ));
    }

    let id = uuid::Uuid::new_v4().simple().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    sqlx::query(
        "INSERT INTO conversations (id, contact_id, channel, state, priority, human_takeover, created_at, updated_at) \
         VALUES (?, ?, ?, 'engaged', 'medium', 0, ?, ?)",
    )
    .bind(&id)
    .bind(&contact.id)
    .bind(channel)
    .bind(&now)
    .bind(&now)
    .execute(pool)
    .await?;

    Ok((id, "engaged".to_string()))
}

/// Records one inbound message across every channel: deduplicates on the
/// provider id, applies the consent gate before anything else, then persists
/// the Message and moves the Conversation state. The second value is true when
/// the provider id had already been ingested.
pub async fn process_inbound_message(
    pool: &DbPool,
    inbound: &InboundMessage<'_>,
) -> Result<(String, bool), InboundError> {
    if !inbound.provider_message_id.is_empty() {
        let existing = sqlx::query("SELECT id FROM messages WHERE provider_message_id = ? LIMIT 1")
            .bind(inbound.provider_message_id)
            .fetch_optional(pool)
            .await?;
        if let Some(row) = existing {
            return Ok((row.try_get("id")?, true));
        }
    }

    let contact = find_contact(pool, inbound.channel, inbound.from_value, inbound.sender_name).await?;
    let (conversation_id, conversation_state) =
        get_or_create_conversation(pool, &contact, inbound.channel).await?;

    // Compliance gate first: a STOP-class body is recorded before the message is
    // stored, so a later crash cannot leave us free to keep messaging them.
    let consent_change =
        compliance::process_inbound_consent_signal(pool, &contact.id, inbound.body).await?;

    let now = chrono::Utc::now().to_rfc3339();
    let message_id = uuid::Uuid::new_v4().simple().to_string();
    sqlx::query(
        "INSERT INTO messages (id, conversation_id, direction, channel, body, status, provider, \
         provider_message_id, sender_type, sender_name, sender, is_private, created_at, sent_at) \
         VALUES (?, ?, 'inbound', ?, ?, 'received', ?, ?, 'customer', ?, 'customer', 0, ?, ?)",
    )
    .bind(&message_id)
    .bind(&conversation_id)
    .bind(inbound.channel)
    .bind(inbound.body)
    .bind(inbound.provider)
    .bind(inbound.provider_message_id)
    .bind(inbound.sender_name)
    .bind(&now)
    .bind(&now)
    .execute(pool)
    .await?;

    let new_state = match consent_change {
        Some(ConsentStatus::OptedOut) => Some("opted_out"),
        _ if !TERMINAL_STATES.contains(&conversation_state.as_str()) => Some("engaged"),
        _ => None,
    };
    if let Some(state) = new_state {
        sqlx::query(
            "UPDATE conversations SET state = ?, last_inbound_at = ?, updated_at = ? WHERE id = ?",
        )
        .bind(state)
        .bind(&now)
        .bind(&now)
        .bind(&conversation_id)
        .execute(pool)
        .await?;
    } else {
        sqlx::query("UPDATE conversations SET last_inbound_at = ?, updated_at = ? WHERE id = ?")
            .bind(&now)
            .bind(&now)
            .bind(&conversation_id)
            .execute(pool)
            .await?;
    }

    Ok((message_id, false))
}

/// Social, email and voice providers deliver identities we have never seen
/// before. Provisioning writes a real Lead plus Contact and an explicit
/// OPTED_IN consent record attributed to the channel, so a later send has an
/// actual consent row to check rather than a compiled-in assumption that the
/// contact is reachable.
pub async fn get_or_create_contact(
    pool: &DbPool,
    channel: &str,
    value: &str,
    sender_name: &str,
    tenant_key: &str,
) -> Result<String, InboundError> {
    let value = value.trim();
    if value.is_empty() {
        return Err(InboundError::UnknownContact(format!(
            "No sender identity on {channel} inbound"
        )));
    }

    if let Some(row) = sqlx::query("SELECT id, person_name FROM contacts WHERE channel = ? AND value = ? LIMIT 1")
        .bind(channel)
        .bind(value)
        .fetch_optional(pool)
        .await?
    {
        let id: String = row.try_get("id")?;
        let person_name: Option<String> = row.try_get("person_name").ok().flatten();
        let trimmed = sender_name.trim();
        if !trimmed.is_empty() && person_name.as_deref().map(is_placeholder_name).unwrap_or(true) {
            sqlx::query("UPDATE contacts SET person_name = ?, updated_at = ? WHERE id = ?")
                .bind(trimmed)
                .bind(chrono::Utc::now().to_rfc3339())
                .bind(&id)
                .execute(pool)
                .await?;
        }
        return Ok(id);
    }

    let title = channel_title(channel);
    let provided = sender_name.trim();
    let short = value.chars().rev().take(4).collect::<Vec<_>>();
    let tail: String = short.iter().rev().collect();
    let fallback = format!("{title} Contact {tail}");
    let lead_name = if provided.is_empty() { fallback.clone() } else { provided.to_string() };
    let contact_name = if provided.is_empty() { format!("{title} User") } else { provided.to_string() };

    let now = chrono::Utc::now().to_rfc3339();
    let lead_id = uuid::Uuid::new_v4().simple().to_string();
    let institution_prefix: String = value.chars().take(16).collect();
    sqlx::query(
        "INSERT INTO leads (id, tenant_key, harvester_institution_id, name, institution_type, \
         city, qualification_rationale, synced_at, created_at, updated_at) \
         VALUES (?, ?, ?, ?, ?, '', '', ?, ?, ?)",
    )
    .bind(&lead_id)
    .bind(tenant_key)
    .bind(format!("{channel}_{institution_prefix}"))
    .bind(&lead_name)
    .bind(format!("{title} Contact"))
    .bind(&now)
    .bind(&now)
    .bind(&now)
    .execute(pool)
    .await?;

    let contact_id = uuid::Uuid::new_v4().simple().to_string();
    sqlx::query(
        "INSERT INTO contacts (id, value, channel, person_name, lead_id, tenant_key, created_at, updated_at) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&contact_id)
    .bind(value)
    .bind(channel)
    .bind(&contact_name)
    .bind(&lead_id)
    .bind(tenant_key)
    .bind(&now)
    .bind(&now)
    .execute(pool)
    .await?;

    compliance::record_consent(
        pool,
        &contact_id,
        ConsentStatus::OptedIn,
        &format!("{channel}_webhook_inbound"),
        "",
    )
    .await?;

    Ok(contact_id)
}

fn channel_title(channel: &str) -> String {
    let mut chars = channel.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}

/// Conversation scoped to a specific inbox, used by the channels that carry
/// more than one number or mailbox. Reopens a thread only once the previous one
/// has been won or lost; an opted-out contact stays opted out.
pub async fn get_or_create_inbox_conversation(
    pool: &DbPool,
    contact_id: &str,
    channel: &str,
    inbox_id: Option<&str>,
    subject: Option<&str>,
) -> Result<String, InboundError> {
    let existing = sqlx::query(
        "SELECT id, state FROM conversations WHERE contact_id = ? AND \
         (inbox_id IS ? OR inbox_id = ?) ORDER BY created_at DESC LIMIT 1",
    )
    .bind(contact_id)
    .bind(inbox_id)
    .bind(inbox_id.unwrap_or_default())
    .fetch_optional(pool)
    .await?;

    if let Some(row) = existing {
        let state: Option<String> = row.try_get("state").ok().flatten();
        if !matches!(state.as_deref(), Some("closed_won") | Some("closed_lost")) {
            return Ok(row.try_get("id")?);
        }
    }

    let id = uuid::Uuid::new_v4().simple().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    let attributes = subject
        .filter(|value| !value.is_empty())
        .map(|value| serde_json::json!({ "subject": value }).to_string());
    sqlx::query(
        "INSERT INTO conversations (id, contact_id, channel, state, priority, human_takeover, \
         inbox_id, custom_attributes, created_at, updated_at) \
         VALUES (?, ?, ?, 'engaged', 'medium', 0, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(contact_id)
    .bind(channel)
    .bind(inbox_id)
    .bind(attributes)
    .bind(&now)
    .bind(&now)
    .execute(pool)
    .await?;

    Ok(id)
}

/// Direct inbound persistence for payloads that are not simple message text
/// (voice call events, threaded emails), where the provider payload itself is
/// the record and the consent pipeline has nothing to parse.
#[allow(clippy::too_many_arguments)]
pub async fn record_raw_inbound_message(
    pool: &DbPool,
    conversation_id: &str,
    channel: &str,
    body: &str,
    provider: &str,
    provider_message_id: &str,
    sender_name: &str,
    metadata: Option<serde_json::Value>,
    attachments: Option<serde_json::Value>,
) -> Result<String, InboundError> {
    if !provider_message_id.is_empty() {
        if let Some(row) = sqlx::query("SELECT id FROM messages WHERE provider_message_id = ? LIMIT 1")
            .bind(provider_message_id)
            .fetch_optional(pool)
            .await?
        {
            return Ok(row.try_get("id")?);
        }
    }

    let now = chrono::Utc::now().to_rfc3339();
    let id = uuid::Uuid::new_v4().simple().to_string();
    sqlx::query(
        "INSERT INTO messages (id, conversation_id, direction, channel, body, status, provider, \
         provider_message_id, sender_type, sender_name, sender, is_private, provider_metadata, \
         attachments, created_at, sent_at) \
         VALUES (?, ?, 'inbound', ?, ?, 'received', ?, ?, 'customer', ?, 'customer', 0, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(conversation_id)
    .bind(channel)
    .bind(body)
    .bind(provider)
    .bind(provider_message_id)
    .bind(sender_name)
    .bind(metadata.map(|value| value.to_string()))
    .bind(attachments.map(|value| value.to_string()))
    .bind(&now)
    .bind(&now)
    .execute(pool)
    .await?;

    sqlx::query("UPDATE conversations SET last_inbound_at = ?, updated_at = ? WHERE id = ?")
        .bind(&now)
        .bind(&now)
        .bind(conversation_id)
        .execute(pool)
        .await?;

    Ok(id)
}

/// Appends provider-side detail (call duration, recordings, transcripts) onto
/// the metadata of an inbound event we already stored.
pub async fn amend_message_metadata(
    pool: &DbPool,
    provider_message_id: &str,
    additions: serde_json::Value,
    body_suffix: &str,
) -> Result<bool, InboundError> {
    if provider_message_id.is_empty() {
        return Ok(false);
    }
    let Some(row) = sqlx::query("SELECT id, body, provider_metadata FROM messages WHERE provider_message_id = ? LIMIT 1")
        .bind(provider_message_id)
        .fetch_optional(pool)
        .await?
    else {
        return Ok(false);
    };

    let id: String = row.try_get("id")?;
    let existing_body: String = row.try_get("body").unwrap_or_default();
    let existing: serde_json::Value = row
        .try_get::<Option<String>, _>("provider_metadata")
        .ok()
        .flatten()
        .and_then(|raw| serde_json::from_str(&raw).ok())
        .unwrap_or_else(|| serde_json::json!({}));

    let mut merged = match existing {
        serde_json::Value::Object(map) => map,
        _ => serde_json::Map::new(),
    };
    if let serde_json::Value::Object(extra) = additions {
        for (key, value) in extra {
            merged.insert(key, value);
        }
    }

    let body = if body_suffix.is_empty() {
        existing_body
    } else {
        format!("{existing_body}{body_suffix}")
    };

    sqlx::query("UPDATE messages SET body = ?, provider_metadata = ? WHERE id = ?")
        .bind(&body)
        .bind(serde_json::Value::Object(merged).to_string())
        .bind(&id)
        .execute(pool)
        .await?;
    Ok(true)
}

/// Applies a delivery/status receipt from the provider onto the outbound
/// message it refers to. Unknown ids and already-terminal states are ignored,
/// so a late or out-of-order receipt cannot resurrect a failed send.
pub async fn apply_delivery_status(
    pool: &DbPool,
    provider_message_id: &str,
    status: &str,
    metadata: Option<serde_json::Value>,
) -> Result<bool, InboundError> {
    if provider_message_id.is_empty() {
        return Ok(false);
    }
    let row = sqlx::query(
        "SELECT id, status FROM messages WHERE provider_message_id = ? AND direction = 'outbound' LIMIT 1",
    )
    .bind(provider_message_id)
    .fetch_optional(pool)
    .await?;
    let Some(row) = row else {
        return Ok(false);
    };
    let current: Option<String> = row.try_get("status").ok().flatten();
    if matches!(current.as_deref(), Some("failed")) && status != "failed" {
        return Ok(false);
    }

    let id: String = row.try_get("id")?;
    let metadata_text = metadata.map(|v| v.to_string());
    sqlx::query(
        "UPDATE messages SET status = ?, delivery_status = ?, provider_metadata = COALESCE(?, provider_metadata) \
         WHERE id = ?",
    )
    .bind(status)
    .bind(status)
    .bind(metadata_text)
    .bind(&id)
    .execute(pool)
    .await?;
    Ok(true)
}
