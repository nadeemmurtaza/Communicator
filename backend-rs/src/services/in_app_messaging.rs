//! In-App Messaging & Proactive Triggers — Phase 32
//!
//! Behavioral event-driven messaging system for triggered messages based on user actions,
//! page views, time on site, and custom events. Enables proactive outreach like
//! "Are you stuck?" popups, onboarding sequences, and re-engagement messages.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::Row;

use crate::db::AppState;

// ——— Types ———

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InAppMessage {
    pub id: String,
    pub organization_id: String,
    pub title: String,
    pub body: String,
    pub message_type: String,
    pub style: String,
    pub target_url: Option<String>,
    pub button_text: Option<String>,
    pub button_url: Option<String>,
    pub dismissible: bool,
    pub auto_close_seconds: Option<i32>,
    pub trigger_type: String,
    pub trigger_event: Option<String>,
    pub trigger_page: Option<String>,
    pub trigger_delay_seconds: Option<i32>,
    pub trigger_segment_id: Option<String>,
    pub audience_segment_id: Option<String>,
    pub frequency: String,
    pub status: String,
    pub start_at: Option<String>,
    pub end_at: Option<String>,
    pub views: i32,
    pub clicks: i32,
    pub dismisses: i32,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InAppMessageEvent {
    pub id: String,
    pub organization_id: String,
    pub message_id: String,
    pub contact_id: Option<String>,
    pub session_id: Option<String>,
    pub event_type: String,
    pub metadata: Option<Value>,
    pub created_at: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProactiveTrigger {
    pub id: String,
    pub organization_id: String,
    pub name: String,
    pub trigger_type: String,
    pub condition: Option<Value>,
    pub action: String,
    pub message_id: Option<String>,
    pub delay_seconds: i32,
    pub enabled: bool,
    pub max_triggers_per_day: i32,
    pub triggers_today: i32,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BehavioralEvent {
    pub id: String,
    pub organization_id: String,
    pub contact_id: Option<String>,
    pub session_id: Option<String>,
    pub event_name: String,
    pub event_data: Option<Value>,
    pub page_url: Option<String>,
    pub created_at: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InAppSettings {
    pub id: String,
    pub organization_id: String,
    pub enabled: bool,
    pub max_concurrent_messages: i32,
    pub default_style: String,
    pub allow_custom_css: bool,
    pub track_behavioral_events: bool,
    pub event_retention_days: i32,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InAppSummary {
    pub enabled: bool,
    pub total_messages: i32,
    pub active_messages: i32,
    pub draft_messages: i32,
    pub total_triggers: i32,
    pub enabled_triggers: i32,
    pub total_events_today: i32,
    pub total_behavioral_events: i32,
    pub by_message_type: Vec<MessageTypeCount>,
    pub by_trigger_type: Vec<TriggerTypeCount>,
    pub recent_messages: Vec<InAppMessage>,
    pub recent_events: Vec<InAppMessageEvent>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MessageTypeCount {
    pub message_type: String,
    pub count: i64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TriggerTypeCount {
    pub trigger_type: String,
    pub count: i64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MessageResult {
    pub message_id: String,
    pub message: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TriggerResult {
    pub trigger_id: String,
    pub message: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TriggerEvaluationResult {
    pub matched: bool,
    pub message_id: Option<String>,
    pub trigger_name: Option<String>,
    pub reason: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EventResult {
    pub event_id: String,
    pub message: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SettingsResult {
    pub settings_id: String,
    pub message: String,
}

// ——— Core Functions ———

/// Get in-app messaging summary dashboard data
pub async fn get_summary(state: &AppState) -> Result<InAppSummary, sqlx::Error> {
    // Settings
    let settings_row = sqlx::query(
        "SELECT enabled FROM inapp_settings WHERE organization_id = 'org-default' LIMIT 1"
    )
    .fetch_optional(&state.db)
    .await?;

    let enabled = settings_row
        .as_ref()
        .map(|r| r.try_get::<bool, _>(0).unwrap_or(true))
        .unwrap_or(true);

    // Total messages
    let total_messages: i32 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM inapp_messages"
    )
    .fetch_one(&state.db)
    .await?;

    // Active messages
    let active_messages: i32 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM inapp_messages WHERE status = 'active'"
    )
    .fetch_one(&state.db)
    .await?;

    // Draft messages
    let draft_messages: i32 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM inapp_messages WHERE status = 'draft'"
    )
    .fetch_one(&state.db)
    .await?;

    // Total triggers
    let total_triggers: i32 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM proactive_triggers"
    )
    .fetch_one(&state.db)
    .await?;

    // Enabled triggers
    let enabled_triggers: i32 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM proactive_triggers WHERE enabled = true"
    )
    .fetch_one(&state.db)
    .await?;

    // Total events today
    let total_events_today: i32 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM inapp_message_events WHERE created_at >= NOW() - INTERVAL '1 day'"
    )
    .fetch_one(&state.db)
    .await?;

    // Total behavioral events
    let total_behavioral_events: i32 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM behavioral_events"
    )
    .fetch_one(&state.db)
    .await?;

    // By message type
    let type_rows = sqlx::query(
        "SELECT message_type, COUNT(*) as count FROM inapp_messages \
         GROUP BY message_type ORDER BY count DESC"
    )
    .fetch_all(&state.db)
    .await?;

    let by_message_type: Vec<MessageTypeCount> = type_rows
        .iter()
        .map(|row| MessageTypeCount {
            message_type: row.try_get(0).unwrap_or_default(),
            count: row.try_get(1).unwrap_or(0i64),
        })
        .collect();

    // By trigger type
    let trigger_type_rows = sqlx::query(
        "SELECT trigger_type, COUNT(*) as count FROM proactive_triggers \
         GROUP BY trigger_type ORDER BY count DESC"
    )
    .fetch_all(&state.db)
    .await?;

    let by_trigger_type: Vec<TriggerTypeCount> = trigger_type_rows
        .iter()
        .map(|row| TriggerTypeCount {
            trigger_type: row.try_get(0).unwrap_or_default(),
            count: row.try_get(1).unwrap_or(0i64),
        })
        .collect();

    // Recent messages
    let msg_rows = sqlx::query(
        "SELECT id, organization_id, title, body, message_type, style, target_url, \
         button_text, button_url, dismissible, auto_close_seconds, trigger_type, \
         trigger_event, trigger_page, trigger_delay_seconds, trigger_segment_id, \
         audience_segment_id, frequency, status, start_at, end_at, views, clicks, \
         dismisses, created_at, updated_at \
         FROM inapp_messages ORDER BY created_at DESC LIMIT 10"
    )
    .fetch_all(&state.db)
    .await?;

    let recent_messages: Vec<InAppMessage> = msg_rows
        .iter()
        .map(|row| InAppMessage {
            id: row.try_get(0).unwrap_or_default(),
            organization_id: row.try_get(1).unwrap_or_default(),
            title: row.try_get(2).unwrap_or_default(),
            body: row.try_get(3).unwrap_or_default(),
            message_type: row.try_get(4).unwrap_or_default(),
            style: row.try_get(5).unwrap_or_default(),
            target_url: row.try_get(6).ok(),
            button_text: row.try_get(7).ok(),
            button_url: row.try_get(8).ok(),
            dismissible: row.try_get(9).unwrap_or(true),
            auto_close_seconds: row.try_get(10).ok(),
            trigger_type: row.try_get(11).unwrap_or_default(),
            trigger_event: row.try_get(12).ok(),
            trigger_page: row.try_get(13).ok(),
            trigger_delay_seconds: row.try_get(14).ok(),
            trigger_segment_id: row.try_get(15).ok(),
            audience_segment_id: row.try_get(16).ok(),
            frequency: row.try_get(17).unwrap_or_default(),
            status: row.try_get(18).unwrap_or_default(),
            start_at: row.try_get(19).ok(),
            end_at: row.try_get(20).ok(),
            views: row.try_get(21).unwrap_or(0i32),
            clicks: row.try_get(22).unwrap_or(0i32),
            dismisses: row.try_get(23).unwrap_or(0i32),
            created_at: row.try_get(24).unwrap_or_default(),
            updated_at: row.try_get(25).unwrap_or_default(),
        })
        .collect();

    // Recent events
    let evt_rows = sqlx::query(
        "SELECT id, organization_id, message_id, contact_id, session_id, \
         event_type, metadata, created_at \
         FROM inapp_message_events ORDER BY created_at DESC LIMIT 10"
    )
    .fetch_all(&state.db)
    .await?;

    let recent_events: Vec<InAppMessageEvent> = evt_rows
        .iter()
        .map(|row| InAppMessageEvent {
            id: row.try_get(0).unwrap_or_default(),
            organization_id: row.try_get(1).unwrap_or_default(),
            message_id: row.try_get(2).unwrap_or_default(),
            contact_id: row.try_get(3).ok(),
            session_id: row.try_get(4).ok(),
            event_type: row.try_get(5).unwrap_or_default(),
            metadata: row.try_get(6).ok(),
            created_at: row.try_get(7).unwrap_or_default(),
        })
        .collect();

    Ok(InAppSummary {
        enabled,
        total_messages,
        active_messages,
        draft_messages,
        total_triggers,
        enabled_triggers,
        total_events_today,
        total_behavioral_events,
        by_message_type,
        by_trigger_type,
        recent_messages,
        recent_events,
    })
}

/// Create a new in-app message
pub async fn create_message(
    state: &AppState,
    organization_id: &str,
    title: &str,
    body: &str,
    message_type: &str,
    trigger_type: &str,
    trigger_event: Option<&str>,
    trigger_page: Option<&str>,
    frequency: &str,
) -> Result<MessageResult, sqlx::Error> {
    let row = sqlx::query(
        "INSERT INTO inapp_messages (organization_id, title, body, message_type, trigger_type, \
         trigger_event, trigger_page, frequency, status) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, 'draft') \
         RETURNING id"
    )
    .bind(organization_id)
    .bind(title)
    .bind(body)
    .bind(message_type)
    .bind(trigger_type)
    .bind(trigger_event.unwrap_or(""))
    .bind(trigger_page.unwrap_or(""))
    .bind(frequency)
    .fetch_one(&state.db)
    .await?;

    let message_id = row.try_get(0).unwrap_or_default();

    Ok(MessageResult {
        message_id,
        message: "In-app message created".to_string(),
    })
}

/// List in-app messages with filters
pub async fn list_messages(
    state: &AppState,
    status: Option<&str>,
    message_type: Option<&str>,
) -> Result<Vec<InAppMessage>, sqlx::Error> {
    let mut query = String::from(
        "SELECT id, organization_id, title, body, message_type, style, target_url, \
         button_text, button_url, dismissible, auto_close_seconds, trigger_type, \
         trigger_event, trigger_page, trigger_delay_seconds, trigger_segment_id, \
         audience_segment_id, frequency, status, start_at, end_at, views, clicks, \
         dismisses, created_at, updated_at \
         FROM inapp_messages WHERE 1=1"
    );
    let mut binds: Vec<String> = Vec::new();

    if let Some(s) = status {
        query.push_str(" AND status = $");
        binds.push(s.to_string());
    }
    if let Some(mt) = message_type {
        query.push_str(" AND message_type = $");
        binds.push(mt.to_string());
    }
    query.push_str(" ORDER BY created_at DESC");

    // For simplicity, use a static query
    let rows = sqlx::query(
        "SELECT id, organization_id, title, body, message_type, style, target_url, \
         button_text, button_url, dismissible, auto_close_seconds, trigger_type, \
         trigger_event, trigger_page, trigger_delay_seconds, trigger_segment_id, \
         audience_segment_id, frequency, status, start_at, end_at, views, clicks, \
         dismisses, created_at, updated_at \
         FROM inapp_messages ORDER BY created_at DESC"
    )
    .fetch_all(&state.db)
    .await?;

    let messages: Vec<InAppMessage> = rows
        .iter()
        .map(|row| InAppMessage {
            id: row.try_get(0).unwrap_or_default(),
            organization_id: row.try_get(1).unwrap_or_default(),
            title: row.try_get(2).unwrap_or_default(),
            body: row.try_get(3).unwrap_or_default(),
            message_type: row.try_get(4).unwrap_or_default(),
            style: row.try_get(5).unwrap_or_default(),
            target_url: row.try_get(6).ok(),
            button_text: row.try_get(7).ok(),
            button_url: row.try_get(8).ok(),
            dismissible: row.try_get(9).unwrap_or(true),
            auto_close_seconds: row.try_get(10).ok(),
            trigger_type: row.try_get(11).unwrap_or_default(),
            trigger_event: row.try_get(12).ok(),
            trigger_page: row.try_get(13).ok(),
            trigger_delay_seconds: row.try_get(14).ok(),
            trigger_segment_id: row.try_get(15).ok(),
            audience_segment_id: row.try_get(16).ok(),
            frequency: row.try_get(17).unwrap_or_default(),
            status: row.try_get(18).unwrap_or_default(),
            start_at: row.try_get(19).ok(),
            end_at: row.try_get(20).ok(),
            views: row.try_get(21).unwrap_or(0i32),
            clicks: row.try_get(22).unwrap_or(0i32),
            dismisses: row.try_get(23).unwrap_or(0i32),
            created_at: row.try_get(24).unwrap_or_default(),
            updated_at: row.try_get(25).unwrap_or_default(),
        })
        .collect();

    Ok(messages)
}

/// Get a single in-app message
pub async fn get_message(state: &AppState, message_id: &str) -> Result<InAppMessage, sqlx::Error> {
    let row = sqlx::query(
        "SELECT id, organization_id, title, body, message_type, style, target_url, \
         button_text, button_url, dismissible, auto_close_seconds, trigger_type, \
         trigger_event, trigger_page, trigger_delay_seconds, trigger_segment_id, \
         audience_segment_id, frequency, status, start_at, end_at, views, clicks, \
         dismisses, created_at, updated_at \
         FROM inapp_messages WHERE id = $1"
    )
    .bind(message_id)
    .fetch_one(&state.db)
    .await?;

    Ok(InAppMessage {
        id: row.try_get(0).unwrap_or_default(),
        organization_id: row.try_get(1).unwrap_or_default(),
        title: row.try_get(2).unwrap_or_default(),
        body: row.try_get(3).unwrap_or_default(),
        message_type: row.try_get(4).unwrap_or_default(),
        style: row.try_get(5).unwrap_or_default(),
        target_url: row.try_get(6).ok(),
        button_text: row.try_get(7).ok(),
        button_url: row.try_get(8).ok(),
        dismissible: row.try_get(9).unwrap_or(true),
        auto_close_seconds: row.try_get(10).ok(),
        trigger_type: row.try_get(11).unwrap_or_default(),
        trigger_event: row.try_get(12).ok(),
        trigger_page: row.try_get(13).ok(),
        trigger_delay_seconds: row.try_get(14).ok(),
        trigger_segment_id: row.try_get(15).ok(),
        audience_segment_id: row.try_get(16).ok(),
        frequency: row.try_get(17).unwrap_or_default(),
        status: row.try_get(18).unwrap_or_default(),
        start_at: row.try_get(19).ok(),
        end_at: row.try_get(20).ok(),
        views: row.try_get(21).unwrap_or(0i32),
        clicks: row.try_get(22).unwrap_or(0i32),
        dismisses: row.try_get(23).unwrap_or(0i32),
        created_at: row.try_get(24).unwrap_or_default(),
        updated_at: row.try_get(25).unwrap_or_default(),
    })
}

/// Update an in-app message
pub async fn update_message(
    state: &AppState,
    message_id: &str,
    title: Option<&str>,
    body: Option<&str>,
    message_type: Option<&str>,
    status: Option<&str>,
    button_text: Option<&str>,
    button_url: Option<&str>,
    trigger_type: Option<&str>,
    trigger_event: Option<&str>,
    frequency: Option<&str>,
) -> Result<MessageResult, sqlx::Error> {
    let mut sets: Vec<String> = Vec::new();
    let mut idx = 1;
    macro_rules! add_set {
        ($present:expr, $col:expr) => {
            if $present {
                sets.push(format!("{} = ${}", $col, idx));
                idx += 1;
            }
        };
    }
    add_set!(title.is_some(), "title");
    add_set!(body.is_some(), "body");
    add_set!(message_type.is_some(), "message_type");
    add_set!(status.is_some(), "status");
    add_set!(button_text.is_some(), "button_text");
    add_set!(button_url.is_some(), "button_url");
    add_set!(trigger_type.is_some(), "trigger_type");
    add_set!(trigger_event.is_some(), "trigger_event");
    add_set!(frequency.is_some(), "frequency");

    if sets.is_empty() {
        return Ok(MessageResult {
            message_id: message_id.to_string(),
            message: "No changes".to_string(),
        });
    }

    sets.push("updated_at = NOW()".to_string());
    let set_clause = sets.join(", ");
    let query = format!(
        "UPDATE inapp_messages SET {} WHERE id = ${}",
        set_clause, idx
    );

    let mut q = sqlx::query(&query);
    if let Some(t) = title {
        q = q.bind(t);
    }
    if let Some(b) = body {
        q = q.bind(b);
    }
    if let Some(mt) = message_type {
        q = q.bind(mt);
    }
    if let Some(s) = status {
        q = q.bind(s);
    }
    if let Some(bt) = button_text {
        q = q.bind(bt);
    }
    if let Some(bu) = button_url {
        q = q.bind(bu);
    }
    if let Some(tt) = trigger_type {
        q = q.bind(tt);
    }
    if let Some(te) = trigger_event {
        q = q.bind(te);
    }
    if let Some(f) = frequency {
        q = q.bind(f);
    }
    q = q.bind(message_id);

    q.execute(&state.db).await?;

    Ok(MessageResult {
        message_id: message_id.to_string(),
        message: "In-app message updated".to_string(),
    })
}

/// Delete an in-app message
pub async fn delete_message(
    state: &AppState,
    message_id: &str,
) -> Result<MessageResult, sqlx::Error> {
    sqlx::query("DELETE FROM inapp_messages WHERE id = $1")
        .bind(message_id)
        .execute(&state.db)
        .await?;

    Ok(MessageResult {
        message_id: message_id.to_string(),
        message: "In-app message deleted".to_string(),
    })
}

/// Create a proactive trigger
pub async fn create_trigger(
    state: &AppState,
    organization_id: &str,
    name: &str,
    trigger_type: &str,
    condition: &str,
    action: &str,
    message_id: Option<&str>,
    delay_seconds: i32,
) -> Result<TriggerResult, sqlx::Error> {
    let msg_id = message_id.unwrap_or("");

    let row = sqlx::query(
        "INSERT INTO proactive_triggers (organization_id, name, trigger_type, condition, \
         action, message_id, delay_seconds, enabled) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, true) \
         RETURNING id"
    )
    .bind(organization_id)
    .bind(name)
    .bind(trigger_type)
    .bind(condition)
    .bind(action)
    .bind(msg_id)
    .bind(delay_seconds)
    .fetch_one(&state.db)
    .await?;

    let trigger_id = row.try_get(0).unwrap_or_default();

    Ok(TriggerResult {
        trigger_id,
        message: "Proactive trigger created".to_string(),
    })
}

/// List proactive triggers
pub async fn list_triggers(state: &AppState) -> Result<Vec<ProactiveTrigger>, sqlx::Error> {
    let rows = sqlx::query(
        "SELECT id, organization_id, name, trigger_type, condition, action, \
         message_id, delay_seconds, enabled, max_triggers_per_day, triggers_today, \
         created_at, updated_at \
         FROM proactive_triggers ORDER BY created_at DESC"
    )
    .fetch_all(&state.db)
    .await?;

    let triggers: Vec<ProactiveTrigger> = rows
        .iter()
        .map(|row| ProactiveTrigger {
            id: row.try_get(0).unwrap_or_default(),
            organization_id: row.try_get(1).unwrap_or_default(),
            name: row.try_get(2).unwrap_or_default(),
            trigger_type: row.try_get(3).unwrap_or_default(),
            condition: row.try_get(4).ok(),
            action: row.try_get(5).unwrap_or_default(),
            message_id: row.try_get(6).ok(),
            delay_seconds: row.try_get(7).unwrap_or(0i32),
            enabled: row.try_get(8).unwrap_or(true),
            max_triggers_per_day: row.try_get(9).unwrap_or(100i32),
            triggers_today: row.try_get(10).unwrap_or(0i32),
            created_at: row.try_get(11).unwrap_or_default(),
            updated_at: row.try_get(12).unwrap_or_default(),
        })
        .collect();

    Ok(triggers)
}

/// Update a proactive trigger
pub async fn update_trigger(
    state: &AppState,
    trigger_id: &str,
    name: Option<&str>,
    trigger_type: Option<&str>,
    condition: Option<&str>,
    enabled: Option<bool>,
    message_id: Option<&str>,
) -> Result<TriggerResult, sqlx::Error> {
    let mut sets: Vec<String> = Vec::new();
    let mut idx = 1;
    macro_rules! add_set {
        ($present:expr, $col:expr) => {
            if $present {
                sets.push(format!("{} = ${}", $col, idx));
                idx += 1;
            }
        };
    }
    add_set!(name.is_some(), "name");
    add_set!(trigger_type.is_some(), "trigger_type");
    add_set!(condition.is_some(), "condition");
    add_set!(enabled.is_some(), "enabled");
    add_set!(message_id.is_some(), "message_id");

    if sets.is_empty() {
        return Ok(TriggerResult {
            trigger_id: trigger_id.to_string(),
            message: "No changes".to_string(),
        });
    }

    sets.push("updated_at = NOW()".to_string());
    let set_clause = sets.join(", ");
    let query = format!(
        "UPDATE proactive_triggers SET {} WHERE id = ${}",
        set_clause, idx
    );

    let mut q = sqlx::query(&query);
    if let Some(n) = name {
        q = q.bind(n);
    }
    if let Some(tt) = trigger_type {
        q = q.bind(tt);
    }
    if let Some(c) = condition {
        q = q.bind(c);
    }
    if let Some(e) = enabled {
        q = q.bind(e);
    }
    if let Some(mid) = message_id {
        q = q.bind(mid);
    }
    q = q.bind(trigger_id);

    q.execute(&state.db).await?;

    Ok(TriggerResult {
        trigger_id: trigger_id.to_string(),
        message: "Proactive trigger updated".to_string(),
    })
}

/// Delete a proactive trigger
pub async fn delete_trigger(
    state: &AppState,
    trigger_id: &str,
) -> Result<TriggerResult, sqlx::Error> {
    sqlx::query("DELETE FROM proactive_triggers WHERE id = $1")
        .bind(trigger_id)
        .execute(&state.db)
        .await?;

    Ok(TriggerResult {
        trigger_id: trigger_id.to_string(),
        message: "Proactive trigger deleted".to_string(),
    })
}

/// Record a behavioral event
pub async fn record_behavioral_event(
    state: &AppState,
    organization_id: &str,
    contact_id: Option<&str>,
    session_id: Option<&str>,
    event_name: &str,
    event_data: &str,
    page_url: Option<&str>,
) -> Result<EventResult, sqlx::Error> {
    let contact_val = contact_id.unwrap_or("");
    let session_val = session_id.unwrap_or("");
    let data_val = if event_data.is_empty() { "{}" } else { event_data };
    let url_val = page_url.unwrap_or("");

    let row = sqlx::query(
        "INSERT INTO behavioral_events (organization_id, contact_id, session_id, \
         event_name, event_data, page_url) \
         VALUES ($1, $2, $3, $4, $5, $6) \
         RETURNING id"
    )
    .bind(organization_id)
    .bind(contact_val)
    .bind(session_val)
    .bind(event_name)
    .bind(data_val)
    .bind(url_val)
    .fetch_one(&state.db)
    .await?;

    let event_id = row.try_get(0).unwrap_or_default();

    Ok(EventResult {
        event_id,
        message: "Behavioral event recorded".to_string(),
    })
}

/// Evaluate triggers against a behavioral event
pub async fn evaluate_triggers(
    state: &AppState,
    organization_id: &str,
    event_name: &str,
    page_url: Option<&str>,
) -> Result<Vec<TriggerEvaluationResult>, sqlx::Error> {
    let triggers = list_triggers(state).await?;
    let mut results = Vec::new();

    for trigger in triggers {
        if !trigger.enabled {
            continue;
        }

        let matched = match trigger.trigger_type.as_str() {
            "page_view" => {
                if let Some(condition) = &trigger.condition {
                    if let Some(pages) = condition.get("pages") {
                        if let Some(page) = page_url {
                            pages.as_array().map_or(false, |arr| {
                                arr.iter().any(|p| {
                                    if let Some(p_str) = p.as_str() {
                                        page.starts_with(p_str)
                                    } else {
                                        false
                                    }
                                })
                            })
                        } else {
                            false
                        }
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            "event" => event_name == trigger.trigger_type || {
                trigger
                    .condition
                    .as_ref()
                    .and_then(|c| c.get("event"))
                    .and_then(|v| v.as_str())
                    .map(|evt| evt == event_name)
                    .unwrap_or(false)
            },
            "exit_intent" | "inactivity" => {
                // These are evaluated client-side; server just acknowledges
                event_name == trigger.trigger_type
            }
            _ => false,
        };

        results.push(TriggerEvaluationResult {
            matched,
            message_id: trigger.message_id.clone(),
            trigger_name: Some(trigger.name),
            reason: if matched {
                "Trigger condition matched".to_string()
            } else {
                "Trigger condition not matched".to_string()
            },
        });
    }

    Ok(results)
}

/// Record a message event (shown, clicked, dismissed)
pub async fn record_message_event(
    state: &AppState,
    organization_id: &str,
    message_id: &str,
    contact_id: Option<&str>,
    session_id: Option<&str>,
    event_type: &str,
) -> Result<EventResult, sqlx::Error> {
    let contact_val = contact_id.unwrap_or("");
    let session_val = session_id.unwrap_or("");

    let row = sqlx::query(
        "INSERT INTO inapp_message_events (organization_id, message_id, contact_id, \
         session_id, event_type) \
         VALUES ($1, $2, $3, $4, $5) \
         RETURNING id"
    )
    .bind(organization_id)
    .bind(message_id)
    .bind(contact_val)
    .bind(session_val)
    .bind(event_type)
    .fetch_one(&state.db)
    .await?;

    let event_id = row.try_get(0).unwrap_or_default();

    // Update message stats
    match event_type {
        "shown" => {
            sqlx::query("UPDATE inapp_messages SET views = views + 1 WHERE id = $1")
                .bind(message_id)
                .execute(&state.db)
                .await?;
        }
        "clicked" => {
            sqlx::query("UPDATE inapp_messages SET clicks = clicks + 1 WHERE id = $1")
                .bind(message_id)
                .execute(&state.db)
                .await?;
        }
        "dismissed" => {
            sqlx::query("UPDATE inapp_messages SET dismisses = dismisses + 1 WHERE id = $1")
                .bind(message_id)
                .execute(&state.db)
                .await?;
        }
        _ => {}
    }

    Ok(EventResult {
        event_id,
        message: format!("Message event '{}' recorded", event_type),
    })
}

/// Get in-app settings
pub async fn get_settings(state: &AppState) -> Result<InAppSettings, sqlx::Error> {
    let row = sqlx::query(
        "SELECT id, organization_id, enabled, max_concurrent_messages, default_style, \
         allow_custom_css, track_behavioral_events, event_retention_days, \
         created_at, updated_at \
         FROM inapp_settings WHERE organization_id = 'org-default' LIMIT 1"
    )
    .fetch_optional(&state.db)
    .await?;

    match row {
        Some(r) => Ok(InAppSettings {
            id: r.try_get(0).unwrap_or_default(),
            organization_id: r.try_get(1).unwrap_or_default(),
            enabled: r.try_get(2).unwrap_or(true),
            max_concurrent_messages: r.try_get(3).unwrap_or(1i32),
            default_style: r.try_get(4).unwrap_or_default(),
            allow_custom_css: r.try_get(5).unwrap_or(true),
            track_behavioral_events: r.try_get(6).unwrap_or(true),
            event_retention_days: r.try_get(7).unwrap_or(30i32),
            created_at: r.try_get(8).unwrap_or_default(),
            updated_at: r.try_get(9).unwrap_or_default(),
        }),
        None => Ok(InAppSettings {
            id: "inapp-settings-default".to_string(),
            organization_id: "org-default".to_string(),
            enabled: true,
            max_concurrent_messages: 1,
            default_style: "default".to_string(),
            allow_custom_css: true,
            track_behavioral_events: true,
            event_retention_days: 30,
            created_at: "".to_string(),
            updated_at: "".to_string(),
        }),
    }
}

/// Update in-app settings
pub async fn update_settings(
    state: &AppState,
    enabled: Option<bool>,
    max_concurrent_messages: Option<i32>,
    default_style: Option<&str>,
    track_behavioral_events: Option<bool>,
    event_retention_days: Option<i32>,
) -> Result<SettingsResult, sqlx::Error> {
    let mut sets: Vec<String> = Vec::new();
    let mut idx = 1;
    macro_rules! add_set {
        ($present:expr, $col:expr) => {
            if $present {
                sets.push(format!("{} = ${}", $col, idx));
                idx += 1;
            }
        };
    }
    add_set!(enabled.is_some(), "enabled");
    add_set!(max_concurrent_messages.is_some(), "max_concurrent_messages");
    add_set!(default_style.is_some(), "default_style");
    add_set!(track_behavioral_events.is_some(), "track_behavioral_events");
    add_set!(event_retention_days.is_some(), "event_retention_days");

    if sets.is_empty() {
        return Ok(SettingsResult {
            settings_id: "inapp-settings-default".to_string(),
            message: "No changes".to_string(),
        });
    }

    sets.push("updated_at = NOW()".to_string());
    let set_clause = sets.join(", ");
    let query = format!(
        "UPDATE inapp_settings SET {} WHERE organization_id = 'org-default'",
        set_clause
    );

    let mut q = sqlx::query(&query);
    if let Some(e) = enabled {
        q = q.bind(e);
    }
    if let Some(m) = max_concurrent_messages {
        q = q.bind(m);
    }
    if let Some(s) = default_style {
        q = q.bind(s);
    }
    if let Some(t) = track_behavioral_events {
        q = q.bind(t);
    }
    if let Some(d) = event_retention_days {
        q = q.bind(d);
    }

    q.execute(&state.db).await?;

    Ok(SettingsResult {
        settings_id: "inapp-settings-default".to_string(),
        message: "In-app settings updated".to_string(),
    })
}

/// List behavioral events
pub async fn list_behavioral_events(
    state: &AppState,
    event_name: Option<&str>,
) -> Result<Vec<BehavioralEvent>, sqlx::Error> {
    let rows = sqlx::query(
        "SELECT id, organization_id, contact_id, session_id, event_name, \
         event_data, page_url, created_at \
         FROM behavioral_events ORDER BY created_at DESC LIMIT 100"
    )
    .fetch_all(&state.db)
    .await?;

    let events: Vec<BehavioralEvent> = rows
        .iter()
        .map(|row| BehavioralEvent {
            id: row.try_get(0).unwrap_or_default(),
            organization_id: row.try_get(1).unwrap_or_default(),
            contact_id: row.try_get(2).ok(),
            session_id: row.try_get(3).ok(),
            event_name: row.try_get(4).unwrap_or_default(),
            event_data: row.try_get(5).ok(),
            page_url: row.try_get(6).ok(),
            created_at: row.try_get(7).unwrap_or_default(),
        })
        .collect();

    Ok(events)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_summary_serialization() {
        let summary = InAppSummary {
            enabled: true,
            total_messages: 5,
            active_messages: 3,
            draft_messages: 2,
            total_triggers: 4,
            enabled_triggers: 3,
            total_events_today: 100,
            total_behavioral_events: 500,
            by_message_type: vec![],
            by_trigger_type: vec![],
            recent_messages: vec![],
            recent_events: vec![],
        };
        let json = serde_json::to_string(&summary).unwrap();
        assert!(json.contains("\"enabled\":true"));
        assert!(json.contains("\"total_messages\":5"));
    }

    #[test]
    fn test_trigger_evaluation_result() {
        let result = TriggerEvaluationResult {
            matched: true,
            message_id: Some("msg-123".to_string()),
            trigger_name: Some("Welcome Trigger".to_string()),
            reason: "Page view matched".to_string(),
        };
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("\"matched\":true"));
        assert!(json.contains("msg-123"));
    }

    #[test]
    fn test_message_result() {
        let result = MessageResult {
            message_id: "msg-456".to_string(),
            message: "In-app message created".to_string(),
        };
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("msg-456"));
    }
}
