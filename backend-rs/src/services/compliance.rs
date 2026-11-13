#![allow(dead_code)]

use crate::db::DbPool;
use regex::Regex;
use sqlx::Row;
use std::sync::OnceLock;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConsentStatus {
    Unknown,
    OptedIn,
    OptedOut,
}

impl ConsentStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Unknown => "unknown",
            Self::OptedIn => "opted_in",
            Self::OptedOut => "opted_out",
        }
    }

    pub fn parse(value: &str) -> Self {
        match value {
            "opted_in" => Self::OptedIn,
            "opted_out" => Self::OptedOut,
            _ => Self::Unknown,
        }
    }
}

fn opt_out_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"(?i)\b(stop|unsubscribe|opt\s*out|cancel|remove\s*me)\b").unwrap()
    })
}

fn opt_in_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"(?i)\b(start|subscribe|opt\s*in|yes)\b").unwrap()
    })
}

pub fn is_opt_out_message(body: &str) -> bool {
    opt_out_re().is_match(body)
}

pub fn is_opt_in_message(body: &str) -> bool {
    opt_in_re().is_match(body)
}

pub async fn get_consent_status(pool: &DbPool, contact_id: &str) -> Result<ConsentStatus, sqlx::Error> {
    let row = sqlx::query(
        "SELECT status FROM consent_records WHERE contact_id = ? ORDER BY recorded_at DESC LIMIT 1",
    )
    .bind(contact_id)
    .fetch_optional(pool)
    .await?;
    Ok(row
        .and_then(|r| r.try_get::<String, _>("status").ok())
        .map(|s| ConsentStatus::parse(&s))
        .unwrap_or(ConsentStatus::Unknown))
}

pub async fn record_consent(
    pool: &DbPool,
    contact_id: &str,
    status: ConsentStatus,
    source: &str,
    note: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO consent_records (id, contact_id, status, source, note, recorded_at) \
         VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(uuid::Uuid::new_v4().simple().to_string())
    .bind(contact_id)
    .bind(status.as_str())
    .bind(source)
    .bind(note)
    .bind(chrono::Utc::now().to_rfc3339())
    .execute(pool)
    .await?;
    Ok(())
}

/// Unknown is deliberately not sendable: a Contact must have an explicit
/// OPTED_IN record before anything goes out.
pub async fn is_sendable(pool: &DbPool, contact_id: &str) -> Result<bool, sqlx::Error> {
    Ok(get_consent_status(pool, contact_id).await? == ConsentStatus::OptedIn)
}

/// Runs before any reply logic on every inbound message. Returns the new
/// status when the message changed consent.
pub async fn process_inbound_consent_signal(
    pool: &DbPool,
    contact_id: &str,
    body: &str,
) -> Result<Option<ConsentStatus>, sqlx::Error> {
    if is_opt_out_message(body) {
        record_consent(pool, contact_id, ConsentStatus::OptedOut, "inbound_stop_keyword", "").await?;
        return Ok(Some(ConsentStatus::OptedOut));
    }
    if is_opt_in_message(body) {
        record_consent(pool, contact_id, ConsentStatus::OptedIn, "inbound_start_keyword", "").await?;
        return Ok(Some(ConsentStatus::OptedIn));
    }
    Ok(None)
}
