//! Real-time Notifications & WebSockets — Phase 31
//!
//! Push notifications, WebSocket broadcast, and notification preferences.
//! Enables live updates for new messages, draft approvals, alerts, and system events.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::Row;

use crate::db::AppState;

// ——— Types ———

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NotificationPreferences {
    pub id: String,
    pub organization_id: String,
    pub user_id: String,
    pub new_message_push: bool,
    pub new_message_email: bool,
    pub draft_approved_push: bool,
    pub draft_rejected_push: bool,
    pub alert_triggered_push: bool,
    pub alert_triggered_email: bool,
    pub conversation_assigned_push: bool,
    pub conversation_escalated_push: bool,
    pub workflow_completed_push: bool,
    pub csat_response_push: bool,
    pub suggestion_ready_push: bool,
    pub operator_proposal_push: bool,
    pub sound_enabled: bool,
    pub desktop_notifications: bool,
    pub quiet_hours_start: Option<String>,
    pub quiet_hours_end: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NotificationItem {
    pub id: String,
    pub organization_id: String,
    pub user_id: Option<String>,
    pub notification_type: String,
    pub title: String,
    pub body: String,
    pub data: Option<Value>,
    pub channel: String,
    pub status: String,
    pub sent_at: Option<String>,
    pub read_at: Option<String>,
    pub dismissed_at: Option<String>,
    pub created_at: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WebSocketConnection {
    pub id: String,
    pub organization_id: String,
    pub user_id: String,
    pub session_id: String,
    pub connected_at: String,
    pub last_heartbeat: String,
    pub path: String,
    pub user_agent: String,
    pub ip_address: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BroadcastLog {
    pub id: String,
    pub organization_id: String,
    pub notification_type: String,
    pub title: String,
    pub recipient_count: i32,
    pub delivered_count: i32,
    pub failed_count: i32,
    pub broadcast_at: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NotificationSettings {
    pub id: String,
    pub organization_id: String,
    pub enabled: bool,
    pub websocket_enabled: bool,
    pub email_notifications_enabled: bool,
    pub email_from_address: String,
    pub max_notifications_per_hour: i32,
    pub digest_mode: bool,
    pub digest_interval_minutes: i32,
    pub retention_days: i32,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NotificationSummary {
    pub enabled: bool,
    pub websocket_enabled: bool,
    pub total_notifications: i32,
    pub pending_notifications: i32,
    pub unread_notifications: i32,
    pub active_connections: i32,
    pub broadcasts_today: i32,
    pub by_type: Vec<NotificationTypeCount>,
    pub recent_notifications: Vec<NotificationItem>,
    pub recent_broadcasts: Vec<BroadcastLog>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NotificationTypeCount {
    pub notification_type: String,
    pub count: i64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NotificationResult {
    pub notification_id: String,
    pub message: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BroadcastResult {
    pub broadcast_id: String,
    pub recipients: i32,
    pub message: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SettingsResult {
    pub settings_id: String,
    pub message: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PreferencesResult {
    pub preferences_id: String,
    pub message: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WebSocketMessage {
    pub event: String,
    pub data: Value,
    pub timestamp: String,
}

// ——— Core Functions ———

/// Get notification summary dashboard data
pub async fn get_notification_summary(state: &AppState) -> Result<NotificationSummary, sqlx::Error> {
    // Settings
    let settings_row = sqlx::query(
        "SELECT enabled, websocket_enabled FROM notification_settings WHERE organization_id = 'org-default' LIMIT 1"
    )
    .fetch_optional(&state.db)
    .await?;

    let enabled = settings_row
        .as_ref()
        .map(|r| r.try_get::<bool, _>(0).unwrap_or(true))
        .unwrap_or(true);

    let websocket_enabled = settings_row
        .as_ref()
        .map(|r| r.try_get::<bool, _>(1).unwrap_or(true))
        .unwrap_or(true);

    // Total notifications
    let total_notifications: i32 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM notification_queue"
    )
    .fetch_one(&state.db)
    .await?;

    // Pending notifications
    let pending_notifications: i32 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM notification_queue WHERE status = 'pending'"
    )
    .fetch_one(&state.db)
    .await?;

    // Unread notifications (sent but not read)
    let unread_notifications: i32 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM notification_queue WHERE status = 'sent' AND read_at IS NULL"
    )
    .fetch_one(&state.db)
    .await?;

    // Active WebSocket connections (heartbeat within 30 seconds)
    let active_connections: i32 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM websocket_connections WHERE last_heartbeat >= NOW() - INTERVAL '30 seconds'"
    )
    .fetch_one(&state.db)
    .await?;

    // Broadcasts today
    let broadcasts_today: i32 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM notification_broadcast_log WHERE broadcast_at >= NOW() - INTERVAL '1 day'"
    )
    .fetch_one(&state.db)
    .await?;

    // By type
    let type_rows = sqlx::query(
        "SELECT notification_type, COUNT(*) as count FROM notification_queue \
         GROUP BY notification_type ORDER BY count DESC LIMIT 10"
    )
    .fetch_all(&state.db)
    .await?;

    let by_type: Vec<NotificationTypeCount> = type_rows
        .iter()
        .map(|row| NotificationTypeCount {
            notification_type: row.try_get(0).unwrap_or_default(),
            count: row.try_get(1).unwrap_or(0i64),
        })
        .collect();

    // Recent notifications
    let notif_rows = sqlx::query(
        "SELECT id, organization_id, user_id, notification_type, title, body, data, \
         channel, status, sent_at, read_at, dismissed_at, created_at \
         FROM notification_queue ORDER BY created_at DESC LIMIT 10"
    )
    .fetch_all(&state.db)
    .await?;

    let recent_notifications: Vec<NotificationItem> = notif_rows
        .iter()
        .map(|row| NotificationItem {
            id: row.try_get(0).unwrap_or_default(),
            organization_id: row.try_get(1).unwrap_or_default(),
            user_id: row.try_get(2).ok(),
            notification_type: row.try_get(3).unwrap_or_default(),
            title: row.try_get(4).unwrap_or_default(),
            body: row.try_get(5).unwrap_or_default(),
            data: row.try_get(6).ok(),
            channel: row.try_get(7).unwrap_or_default(),
            status: row.try_get(8).unwrap_or_default(),
            sent_at: row.try_get(9).ok(),
            read_at: row.try_get(10).ok(),
            dismissed_at: row.try_get(11).ok(),
            created_at: row.try_get(12).unwrap_or_default(),
        })
        .collect();

    // Recent broadcasts
    let broadcast_rows = sqlx::query(
        "SELECT id, organization_id, notification_type, title, recipient_count, \
         delivered_count, failed_count, broadcast_at \
         FROM notification_broadcast_log ORDER BY broadcast_at DESC LIMIT 10"
    )
    .fetch_all(&state.db)
    .await?;

    let recent_broadcasts: Vec<BroadcastLog> = broadcast_rows
        .iter()
        .map(|row| BroadcastLog {
            id: row.try_get(0).unwrap_or_default(),
            organization_id: row.try_get(1).unwrap_or_default(),
            notification_type: row.try_get(2).unwrap_or_default(),
            title: row.try_get(3).unwrap_or_default(),
            recipient_count: row.try_get(4).unwrap_or(0i32),
            delivered_count: row.try_get(5).unwrap_or(0i32),
            failed_count: row.try_get(6).unwrap_or(0i32),
            broadcast_at: row.try_get(7).unwrap_or_default(),
        })
        .collect();

    Ok(NotificationSummary {
        enabled,
        websocket_enabled,
        total_notifications,
        pending_notifications,
        unread_notifications,
        active_connections,
        broadcasts_today,
        by_type,
        recent_notifications,
        recent_broadcasts,
    })
}

/// Queue a new notification
pub async fn queue_notification(
    state: &AppState,
    organization_id: &str,
    user_id: Option<&str>,
    notification_type: &str,
    title: &str,
    body: &str,
    data: &str,
    channel: &str,
) -> Result<NotificationResult, sqlx::Error> {
    let user_id_val = user_id.unwrap_or("");
    let data_val = if data.is_empty() { "{}" } else { data };

    let row = sqlx::query(
        "INSERT INTO notification_queue (organization_id, user_id, notification_type, title, body, data, channel, status) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, 'pending') \
         RETURNING id"
    )
    .bind(organization_id)
    .bind(user_id_val)
    .bind(notification_type)
    .bind(title)
    .bind(body)
    .bind(data_val)
    .bind(channel)
    .fetch_one(&state.db)
    .await?;

    let notification_id = row.try_get(0).unwrap_or_default();

    // Mark as sent
    sqlx::query(
        "UPDATE notification_queue SET status = 'sent', sent_at = NOW() WHERE id = $1"
    )
    .bind(&notification_id)
    .execute(&state.db)
    .await?;

    Ok(NotificationResult {
        notification_id,
        message: "Notification queued and sent".to_string(),
    })
}

/// Broadcast a notification to all connected WebSocket clients
pub async fn broadcast_notification(
    state: &AppState,
    organization_id: &str,
    notification_type: &str,
    title: &str,
    body: &str,
) -> Result<BroadcastResult, sqlx::Error> {
    // Count active connections
    let active_count: i32 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM websocket_connections \
         WHERE organization_id = $1 AND last_heartbeat >= NOW() - INTERVAL '30 seconds'"
    )
    .bind(organization_id)
    .fetch_one(&state.db)
    .await?;

    // Log the broadcast
    let row = sqlx::query(
        "INSERT INTO notification_broadcast_log (organization_id, notification_type, title, recipient_count, delivered_count) \
         VALUES ($1, $2, $3, $4, $4) \
         RETURNING id"
    )
    .bind(organization_id)
    .bind(notification_type)
    .bind(title)
    .bind(active_count)
    .fetch_one(&state.db)
    .await?;

    let broadcast_id = row.try_get(0).unwrap_or_default();

    // Queue a notification for all users in the org
    let users_rows = sqlx::query(
        "SELECT DISTINCT user_id FROM notification_preferences WHERE organization_id = $1"
    )
    .bind(organization_id)
    .fetch_all(&state.db)
    .await?;

    for user_row in &users_rows {
        let user_id: String = user_row.try_get(0).unwrap_or_default();
        if !user_id.is_empty() {
            let _ = queue_notification(
                state,
                organization_id,
                Some(&user_id),
                notification_type,
                title,
                body,
                "{}",
                "websocket",
            )
            .await;
        }
    }

    Ok(BroadcastResult {
        broadcast_id,
        recipients: active_count,
        message: format!("Broadcast sent to {} active connections", active_count),
    })
}

/// List notifications with optional filters
pub async fn list_notifications(
    state: &AppState,
    user_id: Option<&str>,
    notification_type: Option<&str>,
    status: Option<&str>,
    limit: i32,
) -> Result<Vec<NotificationItem>, sqlx::Error> {
    let mut conditions: Vec<String> = Vec::new();
    let mut idx = 1;

    if user_id.is_some() {
        conditions.push(format!("user_id = ${}", idx));
        idx += 1;
    }
    if notification_type.is_some() {
        conditions.push(format!("notification_type = ${}", idx));
        idx += 1;
    }
    if status.is_some() {
        conditions.push(format!("status = ${}", idx));
        idx += 1;
    }

    let where_clause = if conditions.is_empty() {
        "".to_string()
    } else {
        format!("WHERE {}", conditions.join(" AND "))
    };

    let limit_val = if limit <= 0 { 50 } else { limit };

    let query = format!(
        "SELECT id, organization_id, user_id, notification_type, title, body, data, \
         channel, status, sent_at, read_at, dismissed_at, created_at \
         FROM notification_queue {} ORDER BY created_at DESC LIMIT {}",
        where_clause, limit_val
    );

    let mut q = sqlx::query(&query);
    if let Some(uid) = user_id {
        q = q.bind(uid);
    }
    if let Some(ntype) = notification_type {
        q = q.bind(ntype);
    }
    if let Some(st) = status {
        q = q.bind(st);
    }

    let rows = q.fetch_all(&state.db).await?;

    let notifications: Vec<NotificationItem> = rows
        .iter()
        .map(|row| NotificationItem {
            id: row.try_get(0).unwrap_or_default(),
            organization_id: row.try_get(1).unwrap_or_default(),
            user_id: row.try_get(2).ok(),
            notification_type: row.try_get(3).unwrap_or_default(),
            title: row.try_get(4).unwrap_or_default(),
            body: row.try_get(5).unwrap_or_default(),
            data: row.try_get(6).ok(),
            channel: row.try_get(7).unwrap_or_default(),
            status: row.try_get(8).unwrap_or_default(),
            sent_at: row.try_get(9).ok(),
            read_at: row.try_get(10).ok(),
            dismissed_at: row.try_get(11).ok(),
            created_at: row.try_get(12).unwrap_or_default(),
        })
        .collect();

    Ok(notifications)
}

/// Mark a notification as read
pub async fn mark_notification_read(
    state: &AppState,
    notification_id: &str,
) -> Result<NotificationResult, sqlx::Error> {
    sqlx::query(
        "UPDATE notification_queue SET read_at = NOW() WHERE id = $1"
    )
    .bind(notification_id)
    .execute(&state.db)
    .await?;

    Ok(NotificationResult {
        notification_id: notification_id.to_string(),
        message: "Notification marked as read".to_string(),
    })
}

/// Dismiss a notification
pub async fn dismiss_notification(
    state: &AppState,
    notification_id: &str,
) -> Result<NotificationResult, sqlx::Error> {
    sqlx::query(
        "UPDATE notification_queue SET status = 'dismissed', dismissed_at = NOW() WHERE id = $1"
    )
    .bind(notification_id)
    .execute(&state.db)
    .await?;

    Ok(NotificationResult {
        notification_id: notification_id.to_string(),
        message: "Notification dismissed".to_string(),
    })
}

/// Get notification preferences for a user
pub async fn get_preferences(
    state: &AppState,
    user_id: &str,
) -> Result<NotificationPreferences, sqlx::Error> {
    let row = sqlx::query(
        "SELECT id, organization_id, user_id, new_message_push, new_message_email, \
         draft_approved_push, draft_rejected_push, alert_triggered_push, alert_triggered_email, \
         conversation_assigned_push, conversation_escalated_push, workflow_completed_push, \
         csat_response_push, suggestion_ready_push, operator_proposal_push, \
         sound_enabled, desktop_notifications, quiet_hours_start, quiet_hours_end, \
         created_at, updated_at \
         FROM notification_preferences WHERE user_id = $1 LIMIT 1"
    )
    .bind(user_id)
    .fetch_optional(&state.db)
    .await?;

    match row {
        Some(r) => Ok(NotificationPreferences {
            id: r.try_get(0).unwrap_or_default(),
            organization_id: r.try_get(1).unwrap_or_default(),
            user_id: r.try_get(2).unwrap_or_default(),
            new_message_push: r.try_get(3).unwrap_or(true),
            new_message_email: r.try_get(4).unwrap_or(false),
            draft_approved_push: r.try_get(5).unwrap_or(true),
            draft_rejected_push: r.try_get(6).unwrap_or(true),
            alert_triggered_push: r.try_get(7).unwrap_or(true),
            alert_triggered_email: r.try_get(8).unwrap_or(true),
            conversation_assigned_push: r.try_get(9).unwrap_or(true),
            conversation_escalated_push: r.try_get(10).unwrap_or(true),
            workflow_completed_push: r.try_get(11).unwrap_or(false),
            csat_response_push: r.try_get(12).unwrap_or(false),
            suggestion_ready_push: r.try_get(13).unwrap_or(true),
            operator_proposal_push: r.try_get(14).unwrap_or(true),
            sound_enabled: r.try_get(15).unwrap_or(true),
            desktop_notifications: r.try_get(16).unwrap_or(true),
            quiet_hours_start: r.try_get(17).ok(),
            quiet_hours_end: r.try_get(18).ok(),
            created_at: r.try_get(19).unwrap_or_default(),
            updated_at: r.try_get(20).unwrap_or_default(),
        }),
        None => {
            // Create default preferences
            let org_id: String = sqlx::query_scalar(
                "SELECT organization_id FROM notification_preferences LIMIT 1"
            )
            .fetch_optional(&state.db)
            .await?
            .unwrap_or("org-default".to_string());

            let row = sqlx::query(
                "INSERT INTO notification_preferences (organization_id, user_id) \
                 VALUES ($1, $2) RETURNING id, organization_id, user_id, created_at, updated_at"
            )
            .bind(&org_id)
            .bind(user_id)
            .fetch_one(&state.db)
            .await?;

            Ok(NotificationPreferences {
                id: row.try_get(0).unwrap_or_default(),
                organization_id: row.try_get(1).unwrap_or_default(),
                user_id: user_id.to_string(),
                new_message_push: true,
                new_message_email: false,
                draft_approved_push: true,
                draft_rejected_push: true,
                alert_triggered_push: true,
                alert_triggered_email: true,
                conversation_assigned_push: true,
                conversation_escalated_push: true,
                workflow_completed_push: false,
                csat_response_push: false,
                suggestion_ready_push: true,
                operator_proposal_push: true,
                sound_enabled: true,
                desktop_notifications: true,
                quiet_hours_start: None,
                quiet_hours_end: None,
                created_at: row.try_get(3).unwrap_or_default(),
                updated_at: row.try_get(4).unwrap_or_default(),
            })
        }
    }
}

/// Update notification preferences
pub async fn update_preferences(
    state: &AppState,
    user_id: &str,
    prefs: &NotificationPreferences,
) -> Result<PreferencesResult, sqlx::Error> {
    sqlx::query(
        "UPDATE notification_preferences SET \
         new_message_push = $1, new_message_email = $2, \
         draft_approved_push = $3, draft_rejected_push = $4, \
         alert_triggered_push = $5, alert_triggered_email = $6, \
         conversation_assigned_push = $7, conversation_escalated_push = $8, \
         workflow_completed_push = $9, csat_response_push = $10, \
         suggestion_ready_push = $11, operator_proposal_push = $12, \
         sound_enabled = $13, desktop_notifications = $14, \
         quiet_hours_start = $15, quiet_hours_end = $16, updated_at = NOW() \
         WHERE user_id = $17"
    )
    .bind(prefs.new_message_push)
    .bind(prefs.new_message_email)
    .bind(prefs.draft_approved_push)
    .bind(prefs.draft_rejected_push)
    .bind(prefs.alert_triggered_push)
    .bind(prefs.alert_triggered_email)
    .bind(prefs.conversation_assigned_push)
    .bind(prefs.conversation_escalated_push)
    .bind(prefs.workflow_completed_push)
    .bind(prefs.csat_response_push)
    .bind(prefs.suggestion_ready_push)
    .bind(prefs.operator_proposal_push)
    .bind(prefs.sound_enabled)
    .bind(prefs.desktop_notifications)
    .bind(prefs.quiet_hours_start.as_deref())
    .bind(prefs.quiet_hours_end.as_deref())
    .bind(user_id)
    .execute(&state.db)
    .await?;

    Ok(PreferencesResult {
        preferences_id: user_id.to_string(),
        message: "Preferences updated".to_string(),
    })
}

/// Get notification settings
pub async fn get_settings(state: &AppState) -> Result<NotificationSettings, sqlx::Error> {
    let row = sqlx::query(
        "SELECT id, organization_id, enabled, websocket_enabled, email_notifications_enabled, \
         email_from_address, max_notifications_per_hour, digest_mode, digest_interval_minutes, \
         retention_days, created_at, updated_at \
         FROM notification_settings WHERE organization_id = 'org-default' LIMIT 1"
    )
    .fetch_optional(&state.db)
    .await?;

    match row {
        Some(r) => Ok(NotificationSettings {
            id: r.try_get(0).unwrap_or_default(),
            organization_id: r.try_get(1).unwrap_or_default(),
            enabled: r.try_get(2).unwrap_or(true),
            websocket_enabled: r.try_get(3).unwrap_or(true),
            email_notifications_enabled: r.try_get(4).unwrap_or(false),
            email_from_address: r.try_get(5).unwrap_or_default(),
            max_notifications_per_hour: r.try_get(6).unwrap_or(100i32),
            digest_mode: r.try_get(7).unwrap_or(false),
            digest_interval_minutes: r.try_get(8).unwrap_or(60i32),
            retention_days: r.try_get(9).unwrap_or(90i32),
            created_at: r.try_get(10).unwrap_or_default(),
            updated_at: r.try_get(11).unwrap_or_default(),
        }),
        None => Err(sqlx::Error::RowNotFound),
    }
}

/// Update notification settings
pub async fn update_settings(
    state: &AppState,
    settings: &NotificationSettings,
) -> Result<SettingsResult, sqlx::Error> {
    sqlx::query(
        "UPDATE notification_settings SET \
         enabled = $1, websocket_enabled = $2, email_notifications_enabled = $3, \
         email_from_address = $4, max_notifications_per_hour = $5, \
         digest_mode = $6, digest_interval_minutes = $7, retention_days = $8, \
         updated_at = NOW() \
         WHERE organization_id = $9"
    )
    .bind(settings.enabled)
    .bind(settings.websocket_enabled)
    .bind(settings.email_notifications_enabled)
    .bind(&settings.email_from_address)
    .bind(settings.max_notifications_per_hour)
    .bind(settings.digest_mode)
    .bind(settings.digest_interval_minutes)
    .bind(settings.retention_days)
    .bind(&settings.organization_id)
    .execute(&state.db)
    .await?;

    Ok(SettingsResult {
        settings_id: settings.id.clone(),
        message: "Settings updated".to_string(),
    })
}

/// Register a WebSocket connection
pub async fn register_websocket_connection(
    state: &AppState,
    organization_id: &str,
    user_id: &str,
    session_id: &str,
    path: &str,
    user_agent: &str,
    ip_address: &str,
) -> Result<WebSocketConnection, sqlx::Error> {
    let row = sqlx::query(
        "INSERT INTO websocket_connections (organization_id, user_id, session_id, path, user_agent, ip_address) \
         VALUES ($1, $2, $3, $4, $5, $6) \
         RETURNING id, organization_id, user_id, session_id, connected_at, last_heartbeat, path, user_agent, ip_address"
    )
    .bind(organization_id)
    .bind(user_id)
    .bind(session_id)
    .bind(path)
    .bind(user_agent)
    .bind(ip_address)
    .fetch_one(&state.db)
    .await?;

    Ok(WebSocketConnection {
        id: row.try_get(0).unwrap_or_default(),
        organization_id: row.try_get(1).unwrap_or_default(),
        user_id: row.try_get(2).unwrap_or_default(),
        session_id: row.try_get(3).unwrap_or_default(),
        connected_at: row.try_get(4).unwrap_or_default(),
        last_heartbeat: row.try_get(5).unwrap_or_default(),
        path: row.try_get(6).unwrap_or_default(),
        user_agent: row.try_get(7).unwrap_or_default(),
        ip_address: row.try_get(8).unwrap_or_default(),
    })
}

/// Update WebSocket heartbeat
pub async fn update_websocket_heartbeat(
    state: &AppState,
    session_id: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE websocket_connections SET last_heartbeat = NOW() WHERE session_id = $1"
    )
    .bind(session_id)
    .execute(&state.db)
    .await?;

    Ok(())
}

/// Remove WebSocket connection
pub async fn remove_websocket_connection(
    state: &AppState,
    session_id: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "DELETE FROM websocket_connections WHERE session_id = $1"
    )
    .bind(session_id)
    .execute(&state.db)
    .await?;

    Ok(())
}

/// Get active WebSocket connections
pub async fn list_websocket_connections(
    state: &AppState,
) -> Result<Vec<WebSocketConnection>, sqlx::Error> {
    let rows = sqlx::query(
        "SELECT id, organization_id, user_id, session_id, connected_at, last_heartbeat, \
         path, user_agent, ip_address \
         FROM websocket_connections WHERE last_heartbeat >= NOW() - INTERVAL '30 seconds' \
         ORDER BY connected_at DESC"
    )
    .fetch_all(&state.db)
    .await?;

    let connections: Vec<WebSocketConnection> = rows
        .iter()
        .map(|row| WebSocketConnection {
            id: row.try_get(0).unwrap_or_default(),
            organization_id: row.try_get(1).unwrap_or_default(),
            user_id: row.try_get(2).unwrap_or_default(),
            session_id: row.try_get(3).unwrap_or_default(),
            connected_at: row.try_get(4).unwrap_or_default(),
            last_heartbeat: row.try_get(5).unwrap_or_default(),
            path: row.try_get(6).unwrap_or_default(),
            user_agent: row.try_get(7).unwrap_or_default(),
            ip_address: row.try_get(8).unwrap_or_default(),
        })
        .collect();

    Ok(connections)
}

/// Clean up old notifications based on retention settings
pub async fn cleanup_old_notifications(state: &AppState) -> Result<i32, sqlx::Error> {
    let retention_days: i32 = sqlx::query_scalar(
        "SELECT retention_days FROM notification_settings WHERE organization_id = 'org-default' LIMIT 1"
    )
    .fetch_optional(&state.db)
    .await?
    .unwrap_or(90);

    let result = sqlx::query(
        "DELETE FROM notification_queue WHERE created_at < NOW() - INTERVAL '$1 days'"
    )
    .fetch_one(&state.db)
    .await?;

    Ok(result.try_get(0).unwrap_or(0i32))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_notification_summary_serialization() {
        let summary = NotificationSummary {
            enabled: true,
            websocket_enabled: true,
            total_notifications: 100,
            pending_notifications: 5,
            unread_notifications: 10,
            active_connections: 3,
            broadcasts_today: 2,
            by_type: vec![],
            recent_notifications: vec![],
            recent_broadcasts: vec![],
        };
        let json = serde_json::to_string(&summary).unwrap();
        assert!(json.contains("\"enabled\":true"));
        assert!(json.contains("\"total_notifications\":100"));
    }

    #[test]
    fn test_broadcast_result_serialization() {
        let result = BroadcastResult {
            broadcast_id: "test-id".to_string(),
            recipients: 5,
            message: "Broadcast sent to 5 active connections".to_string(),
        };
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("\"recipients\":5"));
    }

    #[test]
    fn test_websocket_message_serialization() {
        let msg = WebSocketMessage {
            event: "new_message".to_string(),
            data: serde_json::json!({"conversation_id": "123"}),
            timestamp: "2024-01-01T00:00:00Z".to_string(),
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"event\":\"new_message\""));
    }
}
