#![allow(dead_code)]

use crate::{
    db::AppState,
    models::{Conversation, Message, MessageFeedback, SlaPolicy, TeamMember},
};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::{error, info};
use uuid::Uuid;

/// A driver failure is named as one. Folding `Err` into "not found" or an empty
/// list tells the caller the database answered "no rows", which it did not.
fn db_error(context: &str, err: sqlx::Error) -> Response {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(json!({
            "error": format!("{context}: {err}"),
            "databaseError": true,
        })),
    )
        .into_response()
}

/// 0 rows on an id-scoped statement: the row is genuinely absent, so the answer
/// names the id and says nothing was changed.
fn write_not_found(table: &str, id: &str, rows: u64) -> Response {
    (
        StatusCode::NOT_FOUND,
        Json(json!({
            "error": format!(
                "no {table} row with id '{id}' exists (the statement affected {rows} rows), so \
                 nothing was changed."
            ),
            "id": id,
            "rowsAffected": rows,
        })),
    )
        .into_response()
}

/// Timestamps that are stored in a form other than RFC3339 are unknown, not
/// "now" and not "never breached"; the reason is carried out as a warning.
fn parse_ts(
    raw: &str,
    what: &str,
    warnings: &mut Vec<String>,
) -> Option<chrono::DateTime<chrono::Utc>> {
    match chrono::DateTime::parse_from_rfc3339(raw) {
        Ok(dt) => Some(dt.with_timezone(&Utc)),
        Err(err) => {
            warnings.push(format!(
                "{what} is stored as '{raw}', which is not an RFC3339 timestamp ({err}); timing \
                 computed from it is unknown"
            ));
            None
        }
    }
}

pub async fn list_conversations(State(state): State<AppState>) -> impl IntoResponse {
    // `channel` is read from the conversation's own NOT NULL column. The old
    // `COALESCE(ct.channel, 'whatsapp')` reported a thread as WhatsApp whenever
    // its contact had no channel row, which is an invented label, not data.
    let rows = match sqlx::query_as::<_, Conversation>(
        r#"
        SELECT 
            c.id, 
            c.contact_id, 
            ct.person_name as contact_name, 
            ct.person_name as customer_name, 
            ct.person_name as lead_name,
            ct.value as contact_phone, 
            ct.value as phone, 
            c.channel as channel, 
            LOWER(c.state) as state, 
            c.priority,
            c.resolution_type, 
            c.broadcast_id, 
            b.name as broadcast_name, 
            c.inbox_id,
            c.team_id, 
            tm.name as team_name,
            c.csat_score, 
            c.human_takeover, 
            c.assigned_to_id, 
            u.name as assigned_to_name,
            c.snoozed_until, 
            ct.language,
            c.created_at,
            c.updated_at,
            c.custom_attributes
        FROM conversations c
        LEFT JOIN contacts ct ON c.contact_id = ct.id
        LEFT JOIN broadcasts b ON c.broadcast_id = b.id
        LEFT JOIN teams tm ON c.team_id = tm.id
        LEFT JOIN team_members u ON c.assigned_to_id = u.id
        ORDER BY c.updated_at DESC
        "#,
    )
    .fetch_all(&state.db)
    .await
    {
        // A failed read is not "there are no conversations".
        Ok(rows) => rows,
        Err(err) => return db_error("conversations could not be listed", err),
    };

    Json(rows).into_response()
}

pub async fn get_conversation(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    // `channel` is the conversation's own stored value, not the contact's with a
    // 'whatsapp' default bolted on.
    let conv = sqlx::query_as::<_, Conversation>(
        r#"
        SELECT 
            c.id, 
            c.contact_id, 
            ct.person_name as contact_name, 
            ct.person_name as customer_name, 
            ct.person_name as lead_name,
            ct.value as contact_phone, 
            ct.value as phone, 
            c.channel as channel, 
            LOWER(c.state) as state, 
            c.priority,
            c.resolution_type, 
            c.broadcast_id, 
            b.name as broadcast_name, 
            c.inbox_id,
            c.team_id, 
            tm.name as team_name,
            c.csat_score, 
            c.human_takeover, 
            c.assigned_to_id, 
            u.name as assigned_to_name,
            c.snoozed_until, 
            ct.language,
            c.created_at,
            c.updated_at,
            c.custom_attributes
        FROM conversations c
        LEFT JOIN contacts ct ON c.contact_id = ct.id
        LEFT JOIN broadcasts b ON c.broadcast_id = b.id
        LEFT JOIN teams tm ON c.team_id = tm.id
        LEFT JOIN team_members u ON c.assigned_to_id = u.id
        WHERE c.id = $1
        "#,
    )
    .bind(&id)
    .fetch_optional(&state.db)
    .await;

    // A failed read is not "the conversation is missing".
    let c = match conv {
        Ok(Some(c)) => c,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({
                    "error": format!("no conversations row with id '{id}' exists")
                })),
            )
                .into_response();
        }
        Err(err) => {
            return db_error(&format!("conversation '{id}' could not be read"), err);
        }
    };

    let messages = sqlx::query_as::<_, Message>(
        r#"
        SELECT 
            id, 
            conversation_id, 
            LOWER(direction) as direction, 
            channel, 
            body, 
            intent, 
            status, 
            sent_at, 
            provider_message_id,
            provider_metadata, 
            sender_name, 
            is_private,
            media_type,
            media_url,
            media_filename,
            media_mimetype,
            voice_duration,
            transcription_text
        FROM messages
        WHERE conversation_id = $1
        ORDER BY sent_at ASC
        "#,
    )
    .bind(&id)
    .fetch_all(&state.db)
    .await;

    // A failed read is not "this conversation has no messages".
    let messages = match messages {
        Ok(m) => m,
        Err(err) => {
            return db_error(
                &format!("the messages of conversation '{id}' could not be read"),
                err,
            )
        }
    };

    // Load feedback for all messages in this conversation so the response can
    // carry the rating alongside each message without N+1 queries.
    let message_ids: Vec<String> = messages.iter().map(|m| m.id.clone()).collect();
    let feedback_rows: Vec<MessageFeedback> = if message_ids.is_empty() {
        Vec::new()
    } else {
        let placeholders: Vec<String> = (0..message_ids.len()).map(|i| format!("${}", i + 1)).collect();
        let query_text = format!(
            "SELECT id, message_id, conversation_id, rating, comment, author_type, author_id, created_at \
             FROM message_feedback WHERE message_id IN ({})",
            placeholders.join(", ")
        );
        let mut query = sqlx::query_as::<_, MessageFeedback>(&query_text);
        for mid in &message_ids {
            query = query.bind(mid);
        }
        match query.fetch_all(&state.db).await {
            Ok(rows) => rows,
            Err(_) => Vec::new(), // table may not exist yet if migration hasn't run
        }
    };

    // Build a map from message_id -> feedback (keep only the latest per message)
    let feedback_map: std::collections::HashMap<String, Option<MessageFeedback>> = {
        let mut map: std::collections::HashMap<String, MessageFeedback> = std::collections::HashMap::new();
        for fb in feedback_rows {
            let msg_id = fb.message_id.clone();
            let created = fb.created_at.clone();
            map.entry(msg_id)
                .and_modify(|existing| {
                    if created > existing.created_at {
                        *existing = fb.clone();
                    }
                })
                .or_insert(fb);
        }
        let mut result = std::collections::HashMap::new();
        for mid in &message_ids {
            result.insert(mid.clone(), map.get(mid).cloned());
        }
        result
    };

    let mut read_warnings: Vec<String> = Vec::new();

    // NULL custom_attributes means nothing is stored -- that is `{}`. Malformed
    // JSON is reported as malformed, not silently replaced by an empty object.
    let parsed_custom_attrs: serde_json::Value = match c.custom_attributes.as_deref() {
        None | Some("") => json!({}),
        Some(text) => match serde_json::from_str(text) {
            Ok(v) => v,
            Err(err) => {
                read_warnings.push(format!(
                    "the stored custom_attributes of conversation '{id}' are not valid JSON \
                     ({err}); reported as null"
                ));
                serde_json::Value::Null
            }
        },
    };

    // Dynamic SLA Evaluation. It is only computed when every input is real: a
    // policy row with stored thresholds and a parseable created_at. Previously
    // no active policy answered with the invented "Standard Customer Support
    // SLA" at 60/1440 minutes, presenting a fabricated contract as data.
    let priority_val = c.priority.as_deref().map(str::to_lowercase);
    let sla_policy = match sqlx::query_as::<_, SlaPolicy>(
        r#"
        SELECT id, name, description, first_response_time_threshold, resolution_time_threshold, only_during_business_hours, priorities
        FROM sla_policies
        WHERE is_active = 1
        ORDER BY CASE 
            WHEN priorities LIKE '%' || $1 || '%' THEN 0
            ELSE 1 
        END, first_response_time_threshold ASC
        LIMIT 1
        "#,
    )
    .bind(&priority_val)
    .fetch_optional(&state.db)
    .await
    {
        // A failed policy read must not look like "no active policy".
        Ok(p) => p,
        Err(err) => return db_error("the active SLA policy could not be read", err),
    };

    let now = Utc::now();
    let created_dt: Option<chrono::DateTime<chrono::Utc>> = match c.created_at.as_deref() {
        None => {
            read_warnings.push(
                "the conversation has no created_at; the SLA clock cannot be anchored and breach \
                 status was not evaluated"
                    .to_string(),
            );
            None
        }
        Some(text) => parse_ts(
            text,
            &format!("created_at of conversation '{id}'"),
            &mut read_warnings,
        ),
    };

    let first_outbound = messages.iter().find(|m| m.direction == "outbound");
    let first_response_at = first_outbound.map(|m| m.sent_at.clone());

    let fr_threshold = sla_policy.as_ref().and_then(|p| p.first_response_time_threshold);
    let res_threshold = sla_policy.as_ref().and_then(|p| p.resolution_time_threshold);
    let is_resolved = c.state == "resolved" || c.resolution_type.is_some();

    let (sla_obj, sla_status_label) = match (created_dt, &sla_policy, fr_threshold, res_threshold) {
        (Some(created_at_dt), Some(policy), Some(fr), Some(res)) => {
            let fr_target = created_at_dt + chrono::Duration::minutes(fr as i64);
            let res_target = created_at_dt + chrono::Duration::minutes(res as i64);

            let fr_fulfilled = first_response_at.is_some();
            let fr_breached: Option<bool> = match first_response_at.as_deref() {
                Some(at) => parse_ts(at, "the first response timestamp", &mut read_warnings)
                    .map(|dt| dt > fr_target),
                None => Some(now > fr_target),
            };
            let res_breached: Option<bool> = if is_resolved {
                parse_ts(&c.updated_at, "updated_at", &mut read_warnings)
                    .map(|dt| dt > res_target)
            } else {
                Some(now > res_target)
            };
            let breach_unknown = fr_breached.is_none() || res_breached.is_none();
            let is_breached: Option<bool> = if fr_breached == Some(true)
                || res_breached == Some(true)
            {
                Some(true)
            } else if breach_unknown {
                None
            } else {
                Some(false)
            };

            let minutes_to_fr = (fr_target - now).num_minutes();
            let minutes_to_res = (res_target - now).num_minutes();

            let label = if is_breached == Some(true) {
                "breached"
            } else if breach_unknown {
                "unknown"
            } else if minutes_to_fr < 15 && !fr_fulfilled {
                "warning"
            } else {
                "compliant"
            };

            let obj = json!({
                "policyId": policy.id,
                "policyName": policy.name,
                "firstResponseThresholdMinutes": fr,
                "resolutionThresholdMinutes": res,
                "firstResponseTargetAt": fr_target.to_rfc3339(),
                "resolutionTargetAt": res_target.to_rfc3339(),
                "firstResponseAt": first_response_at,
                "firstResponseStatus": if fr_breached == Some(true) { "breached" } else if fr_breached.is_none() { "unknown" } else if fr_fulfilled { "fulfilled" } else { "active" },
                "firstResponseBreached": fr_breached,
                "resolutionAt": if is_resolved { Some(c.updated_at.clone()) } else { None },
                "resolutionStatus": if res_breached == Some(true) { "breached" } else if res_breached.is_none() { "unknown" } else if is_resolved { "fulfilled" } else { "active" },
                "resolutionBreached": res_breached,
                "isBreached": is_breached,
                "nearingBreach": if breach_unknown { None } else { Some(minutes_to_fr > 0 && minutes_to_fr < 15 && !fr_fulfilled) },
                "minutesToFirstResponseBreach": minutes_to_fr.max(0),
                "minutesToResolutionBreach": minutes_to_res.max(0),
                "evaluatedAt": now.to_rfc3339()
            });
            (obj, label.to_string())
        }
        _ => {
            if sla_policy.is_none() {
                read_warnings.push(
                    "no active SLA policy (is_active = 1) exists in sla_policies; thresholds and \
                     breach status were not evaluated instead of being guessed"
                        .to_string(),
                );
            } else {
                read_warnings.push(
                    "the active SLA policy has no stored threshold value(s), or the conversation's \
                     created_at is missing/unparseable; breach status was not evaluated"
                        .to_string(),
                );
            }
            let obj = json!({
                "policyId": sla_policy.as_ref().map(|p| p.id.clone()),
                "policyName": sla_policy.as_ref().map(|p| p.name.clone()),
                "firstResponseThresholdMinutes": fr_threshold,
                "resolutionThresholdMinutes": res_threshold,
                "firstResponseAt": first_response_at,
                "firstResponseStatus": "unknown",
                "firstResponseBreached": serde_json::Value::Null,
                "resolutionStatus": "unknown",
                "resolutionBreached": serde_json::Value::Null,
                "isBreached": serde_json::Value::Null,
                "nearingBreach": serde_json::Value::Null,
                "evaluatedAt": now.to_rfc3339()
            });
            (obj, "unknown".to_string())
        }
    };

    // WhatsApp 24-hour customer service window status
    let whatsapp_24h_window: serde_json::Value = if c.channel.as_deref() == Some("whatsapp") {
        let contact_id = c.contact_id.clone();
        // Query the whatsapp_session_windows table for the active session
        let session_row: Option<(String, String, i64)> = sqlx::query_as(
            r#"
            SELECT id, expires_at, is_active
            FROM whatsapp_session_windows
            WHERE contact_id = $1 AND is_active = 1
            ORDER BY expires_at DESC
            LIMIT 1
            "#
        )
        .bind(&contact_id)
        .fetch_optional(&state.db)
        .await
        .unwrap_or(None);

        match session_row {
            Some((_, expires_at, _)) => {
                let expires_dt: Option<chrono::DateTime<chrono::Utc>> =
                    chrono::DateTime::parse_from_rfc3339(&expires_at)
                        .ok()
                        .map(|dt| dt.with_timezone(&chrono::Utc));

                match expires_dt {
                    Some(exp) => {
                        let remaining = exp - now;
                        let remaining_seconds = remaining.num_seconds().max(0);
                        let remaining_minutes = remaining.num_minutes().max(0);
                        let is_open = remaining_seconds > 0;
                        let is_closing_soon = is_open && remaining_minutes <= 60;

                        json!({
                            "isOpen": is_open,
                            "isClosingSoon": is_closing_soon,
                            "expiresAt": expires_at,
                            "remainingSeconds": remaining_seconds,
                            "remainingMinutes": remaining_minutes,
                        })
                    }
                    None => json!({
                        "isOpen": false,
                        "isClosingSoon": false,
                        "expiresAt": expires_at,
                        "remainingSeconds": 0,
                        "remainingMinutes": 0,
                    }),
                }
            }
            None => json!({
                "isOpen": false,
                "isClosingSoon": false,
                "expiresAt": null,
                "remainingSeconds": 0,
                "remainingMinutes": 0,
            }),
        }
    } else {
        serde_json::Value::Null
    };

    // Attach feedback to each message so the frontend can render the rating.
    let messages_with_feedback: Vec<serde_json::Value> = messages
        .iter()
        .map(|m| {
            let fb = feedback_map.get(&m.id).and_then(|opt| opt.as_ref());
            json!({
                "id": m.id,
                "conversationId": m.conversation_id,
                "direction": m.direction,
                "channel": m.channel,
                "body": m.body,
                "intent": m.intent,
                "status": m.status,
                "sentAt": m.sent_at,
                "providerMessageId": m.provider_message_id,
                "providerMetadata": m.provider_metadata,
                "senderName": m.sender_name,
                "isPrivate": m.is_private,
                "mediaType": m.media_type,
                "mediaUrl": m.media_url,
                "mediaFilename": m.media_filename,
                "mediaMimetype": m.media_mimetype,
                "voiceDuration": m.voice_duration,
                "transcriptionText": m.transcription_text,
                "feedback": fb.map(|f| json!({
                    "id": f.id,
                    "rating": f.rating,
                    "comment": f.comment,
                    "authorType": f.author_type,
                    "createdAt": f.created_at,
                })),
            })
        })
        .collect();

    (
        StatusCode::OK,
        Json(json!({
            "id": c.id,
            "state": c.state,
            "channel": c.channel,
            "contactId": c.contact_id,
            "contactName": c.contact_name,
            "customerName": c.customer_name,
            "leadName": c.lead_name,
            "contactPhone": c.contact_phone,
            "phone": c.phone,
            "humanTakeover": c.human_takeover,
            "assignedToId": c.assigned_to_id,
            "assignedToName": c.assigned_to_name,
            "teamId": c.team_id,
            "teamName": c.team_name,
            "priority": c.priority,
            "resolutionType": c.resolution_type,
            "csatScore": c.csat_score,
            "snoozedUntil": c.snoozed_until,
            "language": c.language,
            "createdAt": c.created_at,
            "updatedAt": c.updated_at,
            "customAttributes": parsed_custom_attrs,
            "slaStatus": sla_status_label,
            "sla": sla_obj,
            "whatsapp24hWindow": whatsapp_24h_window,
            "messages": messages_with_feedback,
            "warnings": read_warnings,
        })),
    )
        .into_response()
}

#[derive(Deserialize)]
pub struct SendMessagePayload {
    pub body: String,
    pub is_private: Option<bool>,
    pub sender_name: Option<String>,
}

pub async fn send_conversation_message(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<SendMessagePayload>,
) -> impl IntoResponse {
    let now = Utc::now().to_rfc3339();
    let msg_id = Uuid::new_v4().to_string();
    let is_private = payload.is_private.unwrap_or(false);
    let mut warnings: Vec<String> = Vec::new();

    // `messages.sender_name` is nullable, so an absent name is stored as null.
    // Attributing the row to a generic "Agent" invents an author.
    let sender_name: Option<String> = payload
        .sender_name
        .map(|name| name.trim().to_string())
        .filter(|name| !name.is_empty());
    if sender_name.is_none() {
        warnings.push(
            "no sender name was supplied, so the message is stored with a null sender_name \
             instead of an invented one"
                .to_string(),
        );
    }

    // Fetch conversation & contact phone. `channel` is the conversation's own
    // NOT NULL column: the previous `COALESCE(ct.channel, 'whatsapp')` made every
    // thread without a recorded contact channel look like a WhatsApp conversation
    // and dispatched a live send against a provider the customer is not on.
    let conv = sqlx::query_as::<_, Conversation>(
        r#"
        SELECT 
            c.id, c.contact_id, ct.person_name as contact_name, ct.person_name as customer_name, ct.person_name as lead_name,
            ct.value as contact_phone, ct.value as phone, c.channel as channel, LOWER(c.state) as state, c.priority,
            c.resolution_type, c.broadcast_id, b.name as broadcast_name, c.inbox_id,
            c.team_id, tm.name as team_name, c.csat_score, c.human_takeover, c.assigned_to_id, u.name as assigned_to_name,
            c.snoozed_until, ct.language, c.created_at, c.updated_at, c.custom_attributes
        FROM conversations c
        LEFT JOIN contacts ct ON c.contact_id = ct.id
        LEFT JOIN broadcasts b ON c.broadcast_id = b.id
        LEFT JOIN teams tm ON c.team_id = tm.id
        LEFT JOIN team_members u ON c.assigned_to_id = u.id
        WHERE c.id = $1
        "#,
    )
    .bind(&id)
    .fetch_optional(&state.db)
    .await;

    // A failed read is not "the conversation is missing".
    let c = match conv {
        Ok(Some(c)) => c,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({
                    "error": format!(
                        "no conversations row with id '{id}' exists, so nothing was sent and \
                         nothing was stored."
                    ),
                    "id": id,
                })),
            )
                .into_response();
        }
        Err(err) => return db_error(&format!("conversation '{id}' could not be read"), err),
    };

    // `messages.channel` is NOT NULL, so a row whose stored channel is empty is
    // refused by name instead of being stored -- and answered -- as 'whatsapp'.
    let channel = match c.channel.as_deref().map(str::trim) {
        Some(value) if !value.is_empty() => value.to_string(),
        _ => {
            return (
                StatusCode::CONFLICT,
                Json(json!({
                    "error": format!(
                        "conversation '{id}' stores an empty conversations.channel and \
                         messages.channel is NOT NULL, so the message was not stored under an \
                         invented 'whatsapp' label. Record the conversation's channel and retry."
                    ),
                    "id": id,
                    "contactId": c.contact_id,
                    "messageStored": false,
                })),
            )
                .into_response();
        }
    };

    // Obtain Meta WhatsApp credentials from env or fallback to integration_settings
    // table. A read failure is a database error: folding `Err` into "no
    // credentials" would report an unreachable provider as an unconfigured one.
    let access_token = if !state.config.meta_whatsapp_access_token.is_empty() {
        Some(state.config.meta_whatsapp_access_token.clone())
    } else {
        match sqlx::query_scalar::<_, String>(
            "SELECT meta_access_token FROM integration_settings WHERE meta_access_token IS NOT NULL AND meta_access_token != '' LIMIT 1",
        )
        .fetch_optional(&state.db)
        .await
        {
            Ok(stored) => stored.filter(|value| !value.is_empty()),
            Err(err) => {
                return db_error("the stored Meta WhatsApp access token could not be read", err)
            }
        }
    };

    let phone_number_id = if !state.config.meta_whatsapp_phone_number_id.is_empty() {
        Some(state.config.meta_whatsapp_phone_number_id.clone())
    } else {
        match sqlx::query_scalar::<_, String>(
            "SELECT meta_phone_number_id FROM integration_settings WHERE meta_phone_number_id IS NOT NULL AND meta_phone_number_id != '' LIMIT 1",
        )
        .fetch_optional(&state.db)
        .await
        {
            Ok(stored) => stored.filter(|value| !value.is_empty()),
            Err(err) => {
                return db_error("the stored Meta WhatsApp phone number id could not be read", err)
            }
        }
    };

    let api_version = if !state.config.meta_whatsapp_api_version.is_empty() {
        state.config.meta_whatsapp_api_version.clone()
    } else {
        // Still a guess about which Graph version the account speaks, so the
        // caller is told it was assumed rather than configured.
        "v21.0".to_string()
    };
    let api_version_assumed = state.config.meta_whatsapp_api_version.is_empty();
    let api_version = &api_version;

    let mut provider_msg_id: Option<String> = None;
    let mut provider: Option<String> = None;
    // Nothing has been dispatched yet, so the row cannot start out as "sent".
    let mut delivery_status = if is_private {
        "internal".to_string()
    } else {
        "not_sent".to_string()
    };

    // If the outbound message is on WhatsApp and every input the provider call
    // needs is really present, dispatch it; otherwise say exactly what is
    // missing and leave the row as not_sent.
    if is_private {
        // An internal note is never handed to a provider.
    } else if channel != "whatsapp" {
        warnings.push(format!(
            "the conversation channel is '{channel}' and this endpoint only dispatches WhatsApp, \
             so no provider call was made and the message is stored as not_sent"
        ));
    } else {
        let recipient_phone = c
            .contact_phone
            .as_deref()
            .or(c.phone.as_deref())
            .map(str::trim)
            .filter(|value| !value.is_empty());

        match (
            recipient_phone,
            access_token.as_deref(),
            phone_number_id.as_deref(),
        ) {
            (Some(phone), Some(token), Some(phone_number_id)) => {
                if api_version_assumed {
                    warnings.push(format!(
                        "no Meta Graph API version is configured, so {api_version} was assumed for \
                         this send"
                    ));
                }
                let meta_url = format!(
                    "https://graph.facebook.com/{api_version}/{phone_number_id}/messages"
                );
                let clean_phone = phone.replace('+', "").replace(' ', "").replace('-', "");

                let meta_body = json!({
                    "messaging_product": "whatsapp",
                    "recipient_type": "individual",
                    "to": clean_phone,
                    "type": "text",
                    "text": { "preview_url": false, "body": payload.body }
                });

                provider = Some("meta".to_string());

                match state
                    .http_client
                    .post(&meta_url)
                    .bearer_auth(token)
                    .json(&meta_body)
                    .send()
                    .await
                {
                    Ok(res) => {
                        let status_code = res.status();
                        if status_code.is_success() {
                            match res.json::<serde_json::Value>().await {
                                Ok(json_res) => {
                                    let provider_id = json_res["messages"]
                                        .as_array()
                                        .and_then(|messages| messages.first())
                                        .and_then(|message| message["id"].as_str())
                                        .map(str::to_string);
                                    match provider_id {
                                        Some(mid) => {
                                            provider_msg_id = Some(mid.clone());
                                            delivery_status = "sent".to_string();
                                            info!(
                                                "Dispatched live WhatsApp message to Meta Cloud API {}: {}",
                                                api_version, mid
                                            );
                                        }
                                        None => {
                                            delivery_status = "accepted".to_string();
                                            warnings.push(format!(
                                                "Meta answered HTTP {status_code} but its response \
                                                 carried no message id, so the send is recorded as \
                                                 accepted and not as delivered"
                                            ));
                                        }
                                    }
                                }
                                Err(err) => {
                                    delivery_status = "accepted".to_string();
                                    warnings.push(format!(
                                        "Meta answered HTTP {status_code}, but the response body \
                                         could not be read ({err}), so the send is unconfirmed and \
                                         no provider message id was stored"
                                    ));
                                }
                            }
                        } else {
                            // An unreadable error body is reported as unreadable,
                            // not as an empty reason.
                            let reason = match res.text().await {
                                Ok(text) if text.trim().is_empty() => {
                                    "the provider returned no error body".to_string()
                                }
                                Ok(text) => text,
                                Err(err) => {
                                    format!("the provider error body could not be read: {err}")
                                }
                            };
                            delivery_status = "failed".to_string();
                            error!("Meta WhatsApp send failed: status {status_code}");
                            warnings.push(format!(
                                "Meta rejected the send (HTTP {status_code}): {reason}. The message \
                                 is stored as failed and was not delivered."
                            ));
                        }
                    }
                    Err(err) => {
                        delivery_status = "failed".to_string();
                        error!("Meta WhatsApp dispatch network error: {err}");
                        warnings.push(format!(
                            "the call to Meta could not be made ({err}). The message is stored as \
                             failed and was not delivered."
                        ));
                    }
                }
            }
            (None, _, _) => {
                let contact_label = match c.contact_id.as_deref() {
                    Some(contact_id) => format!("contact '{contact_id}'"),
                    None => "this conversation's contact".to_string(),
                };
                warnings.push(format!(
                    "{contact_label} has no stored phone number, so no WhatsApp message was sent; \
                     it is stored as not_sent"
                ));
            }
            (_, None, _) => warnings.push(format!(
                "no Meta WhatsApp access token is available (the environment value is unset and \
                 integration_settings.meta_access_token holds nothing), so no message was sent; it \
                 is stored as not_sent"
            )),
            (_, _, None) => warnings.push(format!(
                "no Meta WhatsApp phone number id is available (the environment value is unset and \
                 integration_settings.meta_phone_number_id holds nothing), so no message was sent; \
                 it is stored as not_sent"
            )),
        }
    }

    // Save message to database. `sender` carries the actor role the read paths
    // filter on (`copilot_ai.rs`/`knowledge_base.rs` query `sender IN (...)`),
    // so it has to be bound alongside `sender_type`.
    let message_stored = match sqlx::query(
        r#"
        INSERT INTO messages (
            id, conversation_id, direction, channel, body, status, delivery_status, sent_at,
            provider, provider_message_id, is_private, sender, sender_type, sender_name, created_at
        ) VALUES ($1, $2, 'outbound', $3, $4, $5, $6, $7, $8, $9, $10, 'agent', 'agent', $11, $12)
        "#,
    )
    .bind(&msg_id)
    .bind(&id)
    .bind(&channel)
    .bind(&payload.body)
    .bind(&delivery_status)
    .bind(&delivery_status)
    .bind(&now)
    .bind(&provider)
    .bind(&provider_msg_id)
    .bind(is_private)
    .bind(&sender_name)
    .bind(&now)
    .execute(&state.db)
    .await
    {
        Ok(result) if result.rows_affected() == 1 => true,
        Ok(result) => {
            warnings.push(format!(
                "the messages INSERT affected {} rows instead of 1, so the reply is not in the \
                 transcript (message id {msg_id})",
                result.rows_affected()
            ));
            false
        }
        Err(err) => {
            warnings.push(format!(
                "the reply could not be stored ({err}); message id {msg_id} does not exist in \
                 messages"
            ));
            false
        }
    };

    // Update conversation timestamp
    let timestamp_updated = match sqlx::query(
        r#"
        UPDATE conversations
        SET updated_at = $1, last_outbound_at = $2
        WHERE id = $3
        "#,
    )
    .bind(&now)
    .bind(&now)
    .bind(&id)
    .execute(&state.db)
    .await
    {
        Ok(result) if result.rows_affected() > 0 => true,
        Ok(_) => {
            warnings.push(format!(
                "conversation {id} still shows no outbound activity: the timestamp UPDATE \
                 matched no row, so the conversation id may be stale"
            ));
            false
        }
        Err(err) => {
            warnings.push(format!("the conversation timestamp could not be updated: {err}"));
            false
        }
    };

    // The transcript row is the point of this endpoint: when it was not stored,
    // the answer cannot be a 200 "here is your message".
    let response_status = if message_stored {
        StatusCode::OK
    } else {
        StatusCode::INTERNAL_SERVER_ERROR
    };

    (
        response_status,
        Json(json!({
            "error": (!message_stored).then(|| format!(
                "the outbound reply for conversation '{id}' was not stored in messages (id \
                 '{msg_id}'); see warnings for the provider outcome"
            )),
            "id": msg_id,
            "conversationId": id,
            "direction": "outbound",
            "channel": channel,
            "body": payload.body,
            // No intent is classified on this path and none is stored, so the
            // answer says so instead of showing an empty classification.
            "intent": serde_json::Value::Null,
            "status": delivery_status.clone(),
            "deliveryStatus": delivery_status,
            "sentAt": now,
            "provider": provider,
            "providerMessageId": provider_msg_id,
            "senderName": sender_name,
            "isPrivate": is_private,
            "messageStored": message_stored,
            "conversationUpdated": timestamp_updated,
            "warnings": warnings
        })),
    )
        .into_response()
}

pub async fn resolve_conversation(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let now = Utc::now().to_rfc3339();
    let res = sqlx::query(
        "UPDATE conversations SET state = 'RESOLVED', resolution_type = 'resolved', updated_at = $1 WHERE id = $2",
    )
    .bind(&now)
    .bind(&id)
    .execute(&state.db)
    .await;

    match res {
        Ok(r) if r.rows_affected() > 0 => (
            StatusCode::OK,
            Json(json!({
                "id": id,
                "state": "resolved",
                "resolutionType": "resolved"
            })),
        )
            .into_response(),
        // A driver failure is not "the conversation is missing": 0 rows means the
        // id genuinely does not exist, an error means the statement never ran.
        Ok(r) => write_not_found("conversations", &id, r.rows_affected()),
        Err(err) => db_error(&format!("the conversations row '{id}' could not be updated"), err),
    }
}

#[derive(Deserialize)]
pub struct PriorityPayload {
    pub priority: String,
}

pub async fn update_priority(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<PriorityPayload>,
) -> impl IntoResponse {
    let now = Utc::now().to_rfc3339();
    let res = sqlx::query("UPDATE conversations SET priority = $1, updated_at = $2 WHERE id = $3")
        .bind(&payload.priority)
        .bind(&now)
        .bind(&id)
        .execute(&state.db)
        .await;

    match res {
        Ok(r) if r.rows_affected() > 0 => (
            StatusCode::OK,
            Json(json!({ "id": id, "priority": payload.priority })),
        )
            .into_response(),
        // A driver failure is not "the conversation is missing": 0 rows means the
        // id genuinely does not exist, an error means the statement never ran.
        Ok(r) => write_not_found("conversations", &id, r.rows_affected()),
        Err(err) => db_error(&format!("the conversations row '{id}' could not be updated"), err),
    }
}

#[derive(Deserialize)]
pub struct AssignTeamPayload {
    pub team_id: Option<String>,
}

pub async fn assign_team(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<AssignTeamPayload>,
) -> impl IntoResponse {
    let now = Utc::now().to_rfc3339();
    let res = sqlx::query("UPDATE conversations SET team_id = $1, updated_at = $2 WHERE id = $3")
        .bind(&payload.team_id)
        .bind(&now)
        .bind(&id)
        .execute(&state.db)
        .await;

    match res {
        Ok(r) if r.rows_affected() > 0 => (
            StatusCode::OK,
            Json(json!({ "id": id, "teamId": payload.team_id })),
        )
            .into_response(),
        // A driver failure is not "the conversation is missing": 0 rows means the
        // id genuinely does not exist, an error means the statement never ran.
        Ok(r) => write_not_found("conversations", &id, r.rows_affected()),
        Err(err) => db_error(&format!("the conversations row '{id}' could not be updated"), err),
    }
}

#[derive(Deserialize)]
pub struct AssignAgentPayload {
    pub agent_id: Option<String>,
}

pub async fn assign_agent(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<AssignAgentPayload>,
) -> impl IntoResponse {
    let now = Utc::now().to_rfc3339();
    let res = sqlx::query("UPDATE conversations SET assigned_to_id = $1, updated_at = $2 WHERE id = $3")
        .bind(&payload.agent_id)
        .bind(&now)
        .bind(&id)
        .execute(&state.db)
        .await;

    match res {
        Ok(r) if r.rows_affected() > 0 => (
            StatusCode::OK,
            Json(json!({ "id": id, "assignedToId": payload.agent_id })),
        )
            .into_response(),
        // A driver failure is not "the conversation is missing": 0 rows means the
        // id genuinely does not exist, an error means the statement never ran.
        Ok(r) => write_not_found("conversations", &id, r.rows_affected()),
        Err(err) => db_error(&format!("the conversations row '{id}' could not be updated"), err),
    }
}

#[derive(Deserialize)]
pub struct SnoozePayload {
    pub snoozed_until: Option<String>,
}

pub async fn snooze_conversation(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<SnoozePayload>,
) -> impl IntoResponse {
    let now = Utc::now().to_rfc3339();
    let res = sqlx::query("UPDATE conversations SET snoozed_until = $1, updated_at = $2 WHERE id = $3")
        .bind(&payload.snoozed_until)
        .bind(&now)
        .bind(&id)
        .execute(&state.db)
        .await;

    match res {
        Ok(r) if r.rows_affected() > 0 => (
            StatusCode::OK,
            Json(json!({ "id": id, "snoozedUntil": payload.snoozed_until })),
        )
            .into_response(),
        // A driver failure is not "the conversation is missing": 0 rows means the
        // id genuinely does not exist, an error means the statement never ran.
        Ok(r) => write_not_found("conversations", &id, r.rows_affected()),
        Err(err) => db_error(&format!("the conversations row '{id}' could not be updated"), err),
    }
}

#[derive(Deserialize)]
pub struct CustomAttributesPayload {
    pub attributes: serde_json::Value,
}

pub async fn update_custom_attributes(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<CustomAttributesPayload>,
) -> impl IntoResponse {
    let now = Utc::now().to_rfc3339();
    let attr_str = payload.attributes.to_string();
    let res = sqlx::query("UPDATE conversations SET custom_attributes = $1, updated_at = $2 WHERE id = $3")
        .bind(&attr_str)
        .bind(&now)
        .bind(&id)
        .execute(&state.db)
        .await;

    match res {
        Ok(r) if r.rows_affected() > 0 => (
            StatusCode::OK,
            Json(json!({ "id": id, "customAttributes": payload.attributes })),
        )
            .into_response(),
        // A driver failure is not "the conversation is missing": 0 rows means the
        // id genuinely does not exist, an error means the statement never ran.
        Ok(r) => write_not_found("conversations", &id, r.rows_affected()),
        Err(err) => db_error(&format!("the conversations row '{id}' could not be updated"), err),
    }
}

pub async fn list_contact_notes(
    State(state): State<AppState>,
    Path(contact_id): Path<String>,
) -> impl IntoResponse {
    #[derive(Serialize, sqlx::FromRow)]
    #[serde(rename_all = "camelCase")]
    struct NoteRow {
        id: String,
        contact_id: String,
        body: String,
        author_name: Option<String>,
        created_at: String,
    }

    // A failed read is not "this contact has no notes": an empty array would
    // answer as if the database had looked and found nothing.
    let notes = match sqlx::query_as::<_, NoteRow>(
        "SELECT id, contact_id, body, author_name, created_at FROM contact_notes WHERE contact_id = $1 ORDER BY created_at DESC",
    )
    .bind(&contact_id)
    .fetch_all(&state.db)
    .await
    {
        Ok(notes) => notes,
        Err(err) => {
            return db_error(
                &format!("the contact notes of contact '{contact_id}' could not be read"),
                err,
            )
        }
    };

    Json(notes).into_response()
}

#[derive(Deserialize)]
pub struct AddNotePayload {
    pub body: String,
    pub author_name: Option<String>,
}

pub async fn add_contact_note(
    State(state): State<AppState>,
    Path(contact_id): Path<String>,
    Json(payload): Json<AddNotePayload>,
) -> impl IntoResponse {
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();

    // `contact_notes.author_name` is NOT NULL *without* a default on purpose: a
    // note whose author was not supplied is not signed with an invented "Agent".
    let author = match payload
        .author_name
        .map(|name| name.trim().to_string())
        .filter(|name| !name.is_empty())
    {
        Some(author) => author,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "error": "author_name is required and was not supplied, so the note was not \
                             stored; contact_notes.author_name is NOT NULL and deliberately has no \
                             default to write on its behalf",
                    "contactId": contact_id,
                    "noteStored": false,
                })),
            )
                .into_response();
        }
    };

    // PostgreSQL enforces foreign keys, but the read is split so a
    // failure is not reported as "no such contact".
    match sqlx::query_scalar::<_, i64>("SELECT 1 FROM contacts WHERE id = $1")
        .bind(&contact_id)
        .fetch_optional(&state.db)
        .await
    {
        Ok(Some(_)) => {}
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({
                    "error": format!(
                        "no contacts row with id '{contact_id}' exists, so the note '{id}' was \
                         not stored against it."
                    ),
                    "contactId": contact_id,
                    "noteStored": false,
                })),
            )
                .into_response();
        }
        Err(err) => {
            return db_error(
                &format!("the existence of contact '{contact_id}' could not be checked"),
                err,
            )
        }
    }

    match sqlx::query(
        "INSERT INTO contact_notes (id, contact_id, body, author_name, created_at) VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(&id)
    .bind(&contact_id)
    .bind(&payload.body)
    .bind(&author)
    .bind(&now)
    .execute(&state.db)
    .await
    {
        Ok(report) if report.rows_affected() == 1 => (
            StatusCode::CREATED,
            Json(json!({
                "id": id,
                "contactId": contact_id,
                "body": payload.body,
                "authorName": author,
                "createdAt": now,
                "noteStored": true,
            })),
        )
            .into_response(),
        Ok(report) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": format!(
                    "INSERT INTO contact_notes affected {} rows instead of 1, so note '{id}' was \
                     not stored.",
                    report.rows_affected()
                ),
                "id": id,
                "contactId": contact_id,
                "noteStored": false,
            })),
        )
            .into_response(),
        Err(err) => db_error(
            &format!("the note '{id}' for contact '{contact_id}' could not be inserted"),
            err,
        ),
    }
}

#[derive(Deserialize)]
pub struct EnrollSequenceQuery {
    pub sequence_id: String,
}

pub async fn enroll_sequence(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(query): Query<EnrollSequenceQuery>,
) -> impl IntoResponse {
    let now = Utc::now().to_rfc3339();
    let res = sqlx::query("UPDATE conversations SET sequence_id = $1, updated_at = $2 WHERE id = $3")
        .bind(&query.sequence_id)
        .bind(&now)
        .bind(&id)
        .execute(&state.db)
        .await;

    match res {
        Ok(r) if r.rows_affected() > 0 => (
            StatusCode::OK,
            Json(json!({ "id": id, "sequenceId": query.sequence_id, "status": "enrolled" })),
        )
            .into_response(),
        // A driver failure is not "the conversation is missing": 0 rows means the
        // id genuinely does not exist, an error means the statement never ran.
        Ok(r) => write_not_found("conversations", &id, r.rows_affected()),
        Err(err) => db_error(&format!("the conversations row '{id}' could not be updated"), err),
    }
}

// Real-time agent presence in conversation
#[derive(Deserialize)]
pub struct PresencePayload {
    pub agent_name: String,
    pub action: String,
    pub agent_avatar: Option<String>,
}

pub async fn update_presence(
    Path(id): Path<String>,
    Json(payload): Json<PresencePayload>,
) -> impl IntoResponse {
    Json(json!({
        "status": "ok",
        "presence": {
            "conversationId": id,
            "agentName": payload.agent_name,
            "action": payload.action,
            "agentAvatar": payload.agent_avatar,
            "updatedAt": Utc::now().to_rfc3339()
        }
    }))
}

pub async fn get_presence(_path: Path<String>) -> impl IntoResponse {
    let empty: Vec<serde_json::Value> = Vec::new();
    Json(empty)
}

#[derive(Deserialize)]
pub struct CollisionQuery {
    pub current_agent: Option<String>,
}

pub async fn check_collision(
    Path(id): Path<String>,
    Query(_query): Query<CollisionQuery>,
) -> impl IntoResponse {
    Json(json!({
        "conversationId": id,
        "hasCollision": false,
        "activeAgents": []
    }))
}

#[derive(Deserialize)]
pub struct AutoAssignPayload {
    pub strategy: Option<String>,
    pub team_id: Option<String>,
}

pub async fn auto_assign(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<AutoAssignPayload>,
) -> impl IntoResponse {
    let mut warnings: Vec<String> = Vec::new();

    // `strategy` arrives with the request but nothing here implements an
    // alternative to least-loaded, so the caller is told which rule answered.
    if let Some(strategy) = payload
        .strategy
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        warnings.push(format!(
            "the requested strategy '{strategy}' is not implemented by this endpoint: the \
             candidate is always the online team member with the fewest open conversations"
        ));
    }

    // If team_id is provided, check available members in that team, else across all online team members
    let scoped = match payload
        .team_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        Some(tid) => match sqlx::query_as::<_, TeamMember>(
            r#"
            SELECT tm.id, tm.name, tm.email, tm.role, tm.availability_status, tm.max_concurrent_conversations
            FROM team_members tm
            JOIN team_memberships tms ON tm.id = tms.team_member_id
            LEFT JOIN conversations c ON c.assigned_to_id = tm.id AND LOWER(c.state) NOT IN ('resolved', 'closed_won', 'closed_lost', 'opted_out')
            WHERE tms.team_id = $1 AND (tm.availability_status = 'online' OR tm.availability_status IS NULL)
            GROUP BY tm.id
            ORDER BY COUNT(c.id) ASC
            LIMIT 1
            "#,
        )
        .bind(tid)
        .fetch_optional(&state.db)
        .await
        {
            // A failed read is not "this team has no online member".
            Ok(member) => member,
            Err(err) => {
                return db_error(
                    &format!("the online members of team '{tid}' could not be read"),
                    err,
                )
            }
        },
        None => None,
    };

    let fell_back_from_team = scoped.is_none() && payload.team_id.is_some();

    let member = match scoped {
        Some(m) => Some(m),
        None => {
            match sqlx::query_as::<_, TeamMember>(
                r#"
                SELECT tm.id, tm.name, tm.email, tm.role, tm.availability_status, tm.max_concurrent_conversations
                FROM team_members tm
                LEFT JOIN conversations c ON c.assigned_to_id = tm.id AND LOWER(c.state) NOT IN ('resolved', 'closed_won', 'closed_lost', 'opted_out')
                WHERE (tm.availability_status = 'online' OR tm.availability_status IS NULL)
                GROUP BY tm.id
                ORDER BY COUNT(c.id) ASC
                LIMIT 1
                "#,
            )
            .fetch_optional(&state.db)
            .await
            {
                // A failed read is not "nobody is online".
                Ok(member) => member,
                Err(err) => {
                    return db_error(
                        "the online team members across all teams could not be read",
                        err,
                    )
                }
            }
        }
    };

    if fell_back_from_team {
        if let Some(tid) = payload.team_id.as_deref() {
            match &member {
                Some(m) => warnings.push(format!(
                    "team '{tid}' has no online member of its own, so '{name}' was taken from all \
                     online team members",
                    name = m.name
                )),
                None => warnings.push(format!(
                    "team '{tid}' has no online member and no online team member exists at all"
                )),
            }
        }
    }

    let m = match member {
        Some(m) => m,
        None => {
            return (
                StatusCode::OK,
                Json(json!({
                    "assigned": false,
                    "status": "overflow",
                    "conversationId": id,
                    "message": "No online agents available",
                    "assignedAgentId": serde_json::Value::Null,
                    "conversationUpdated": false,
                    "warnings": warnings,
                })),
            )
                .into_response();
        }
    };

    let now = Utc::now().to_rfc3339();

    // The conversation is only "assigned" once that UPDATE actually touched its
    // row; 0 rows means the id is absent and nothing was linked.
    match sqlx::query(
        "UPDATE conversations SET assigned_to_id = $1, team_id = COALESCE($2, team_id), updated_at = $3 WHERE id = $4",
    )
    .bind(&m.id)
    .bind(&payload.team_id)
    .bind(&now)
    .bind(&id)
    .execute(&state.db)
    .await
    {
        Ok(report) if report.rows_affected() == 1 => {}
        Ok(report) => return write_not_found("conversations", &id, report.rows_affected()),
        Err(err) => {
            return db_error(
                &format!(
                    "conversation '{id}' could not be assigned to team member '{}'",
                    m.id
                ),
                err,
            )
        }
    }

    // Auxiliary: the member's stamp. A failure here is logged, not hidden, and
    // reported through `memberStamped` because the assignment above stands.
    let member_stamped = crate::db::report_write(
        sqlx::query("UPDATE team_members SET last_assigned_at = $1 WHERE id = $2")
            .bind(&now)
            .bind(&m.id)
            .execute(&state.db)
            .await
            .map(|report| report.rows_affected()),
        "team_members.last_assigned_at after auto-assigning a conversation",
    );

    if payload.team_id.is_none() {
        warnings.push(
            "no team_id was supplied, so conversations.team_id kept its stored value; this \
             endpoint did not read it back"
                .to_string(),
        );
    }

    (
        StatusCode::OK,
        Json(json!({
            "assigned": true,
            "status": "assigned",
            "conversationId": id,
            "assignedAgentId": m.id,
            "assignedAgentName": m.name,
            "teamIdRequested": payload.team_id,
            "conversationUpdated": true,
            "memberStamped": member_stamped,
            "warnings": warnings,
        })),
    )
        .into_response()
}

// ---------------------------------------------------------------------------
// Message Feedback — thumbs up / down on AI-generated messages
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct SubmitFeedbackPayload {
    pub rating: i32,       // 1 = helpful, -1 = not helpful
    pub comment: Option<String>,
    pub author_type: Option<String>,
    pub author_id: Option<String>,
}

/// Submit or update feedback on an outbound message.
/// If the same author already rated this message, the rating is updated.
pub async fn submit_message_feedback(
    State(state): State<AppState>,
    Path(message_id): Path<String>,
    Json(payload): Json<SubmitFeedbackPayload>,
) -> impl IntoResponse {
    let mut warnings: Vec<String> = Vec::new();

    // Validate rating
    if payload.rating != 1 && payload.rating != -1 {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "rating must be 1 (thumbs up) or -1 (thumbs down)",
                "received": payload.rating,
            })),
        )
            .into_response();
    }

    let author_type = payload.author_type.unwrap_or_else(|| "customer".to_string());
    let now = Utc::now().to_rfc3339();

    // Verify the message exists and belongs to a conversation
    let msg_row: Option<(String, String)> = match sqlx::query_as(
        "SELECT id, conversation_id FROM messages WHERE id = $1",
    )
    .bind(&message_id)
    .fetch_optional(&state.db)
    .await
    {
        Ok(row) => row,
        Err(err) => return db_error("the message could not be read", err),
    };

    let (msg_id, conversation_id) = match msg_row {
        Some(row) => row,
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({
                    "error": format!("no message with id '{message_id}' exists"),
                    "messageId": message_id,
                })),
            )
                .into_response();
        }
    };

    // Check if this author already rated this message — update if so
    let existing: Option<String> = match sqlx::query_scalar::<_, String>(
        "SELECT id FROM message_feedback WHERE message_id = $1 AND author_type = $2",
    )
    .bind(&msg_id)
    .bind(&author_type)
    .fetch_optional(&state.db)
    .await
    {
        Ok(row) => row,
        Err(_) => None, // table may not exist yet
    };

    let feedback_stored = if let Some(existing_id) = existing {
        // Update existing feedback
        match sqlx::query(
            "UPDATE message_feedback SET rating = $1, comment = $2, created_at = $3 WHERE id = $4",
        )
        .bind(&payload.rating)
        .bind(&payload.comment)
        .bind(&now)
        .bind(&existing_id)
        .execute(&state.db)
        .await
        {
            Ok(report) if report.rows_affected() == 1 => {
                warnings.push("an existing rating from this author was updated".to_string());
                true
            }
            Ok(report) => {
                warnings.push(format!(
                    "UPDATE affected {} rows instead of 1 for feedback id '{}'",
                    report.rows_affected(),
                    existing_id
                ));
                false
            }
            Err(_) => {
                // Table may not exist yet — try INSERT instead
                let fb_id = Uuid::new_v4().to_string();
                match sqlx::query(
                    "INSERT INTO message_feedback (id, message_id, conversation_id, rating, comment, author_type, author_id, created_at) \
                     VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
                )
                .bind(&fb_id)
                .bind(&msg_id)
                .bind(&conversation_id)
                .bind(&payload.rating)
                .bind(&payload.comment)
                .bind(&author_type)
                .bind(&payload.author_id)
                .bind(&now)
                .execute(&state.db)
                .await
                {
                    Ok(report) if report.rows_affected() == 1 => true,
                    _ => false,
                }
            }
        }
    } else {
        // Insert new feedback
        let fb_id = Uuid::new_v4().to_string();
        match sqlx::query(
            "INSERT INTO message_feedback (id, message_id, conversation_id, rating, comment, author_type, author_id, created_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
        )
        .bind(&fb_id)
        .bind(&msg_id)
        .bind(&conversation_id)
        .bind(&payload.rating)
        .bind(&payload.comment)
        .bind(&author_type)
        .bind(&payload.author_id)
        .bind(&now)
        .execute(&state.db)
        .await
        {
            Ok(report) if report.rows_affected() == 1 => true,
            Ok(report) => {
                warnings.push(format!(
                    "INSERT affected {} rows instead of 1 for feedback id '{}'",
                    report.rows_affected(),
                    fb_id
                ));
                false
            }
            Err(err) => {
                warnings.push(format!(
                    "could not store feedback — the message_feedback table may not exist yet \
                     (run migration 0015): {err}"
                ));
                false
            }
        }
    };

    if !feedback_stored {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": "the feedback could not be stored; see warnings",
                "messageId": message_id,
                "feedbackStored": false,
                "warnings": warnings,
            })),
        )
            .into_response();
    }

    (
        StatusCode::OK,
        Json(json!({
            "messageId": message_id,
            "conversationId": conversation_id,
            "rating": payload.rating,
            "comment": payload.comment,
            "authorType": author_type,
            "createdAt": now,
            "feedbackStored": true,
            "warnings": warnings,
        })),
    )
        .into_response()
}

/// Get feedback summary across all messages (for analytics dashboard).
/// Returns counts of positive/negative ratings, recent negative feedback with
/// message context for the self-improvement loop.
pub async fn get_feedback_summary(State(state): State<AppState>) -> impl IntoResponse {
    // Overall counts
    let counts: Option<(i64, i64)> = match sqlx::query_as(
        "SELECT \
            COALESCE(SUM(CASE WHEN rating = 1 THEN 1 ELSE 0 END), 0) as positive, \
            COALESCE(SUM(CASE WHEN rating = -1 THEN 1 ELSE 0 END), 0) as negative \
         FROM message_feedback",
    )
    .fetch_optional(&state.db)
    .await
    {
        Ok(row) => row,
        Err(_) => None, // table may not exist
    };

    let (positive, negative) = counts.unwrap_or((0, 0));
    let total = positive + negative;
    let satisfaction_rate = if total > 0 {
        positive as f64 / total as f64 * 100.0
    } else {
        0.0
    };

    // Recent negative feedback with message body for analysis
    let recent_negative: Vec<(String, String, Option<String>, String)> = match sqlx::query_as(
        "SELECT mf.message_id, m.body, mf.comment, mf.created_at \
         FROM message_feedback mf \
         JOIN messages m ON m.id = mf.message_id \
         WHERE mf.rating = -1 \
         ORDER BY mf.created_at DESC \
         LIMIT 50",
    )
    .fetch_all(&state.db)
    .await
    {
        Ok(rows) => rows,
        Err(_) => Vec::new(),
    };

    let negative_items: Vec<serde_json::Value> = recent_negative
        .iter()
        .map(|(msg_id, body, comment, created_at)| {
            json!({
                "messageId": msg_id,
                "body": body.chars().take(200).collect::<String>(),
                "comment": comment,
                "createdAt": created_at,
            })
        })
        .collect();

    (
        StatusCode::OK,
        Json(json!({
            "totalFeedback": total,
            "positiveRatings": positive,
            "negativeRatings": negative,
            "satisfactionRate": (satisfaction_rate * 100.0).round() as f64 / 100.0,
            "recentNegativeFeedback": negative_items,
        })),
    )
        .into_response()
}
