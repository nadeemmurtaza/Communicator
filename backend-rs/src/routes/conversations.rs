use crate::{
    db::AppState,
    models::{Conversation, Message, SlaPolicy, TeamMember},
};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::{error, info};
use uuid::Uuid;

pub async fn list_conversations(State(state): State<AppState>) -> impl IntoResponse {
    let rows = sqlx::query_as::<_, Conversation>(
        r#"
        SELECT 
            c.id, 
            c.contact_id, 
            ct.person_name as contact_name, 
            ct.person_name as customer_name, 
            ct.person_name as lead_name,
            ct.value as contact_phone, 
            ct.value as phone, 
            COALESCE(ct.channel, 'whatsapp') as channel, 
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
    .unwrap_or_default();

    Json(rows)
}

pub async fn get_conversation(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
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
            COALESCE(ct.channel, 'whatsapp') as channel, 
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
        WHERE c.id = ?
        "#,
    )
    .bind(&id)
    .fetch_optional(&state.db)
    .await
    .unwrap_or(None);

    let c = match conv {
        Some(c) => c,
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({ "error": "Conversation not found" })),
            )
                .into_response();
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
            is_private
        FROM messages
        WHERE conversation_id = ?
        ORDER BY sent_at ASC
        "#,
    )
    .bind(&id)
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    // Parse custom attributes
    let parsed_custom_attrs: serde_json::Value = c
        .custom_attributes
        .as_deref()
        .and_then(|s| serde_json::from_str(s).ok())
        .unwrap_or_else(|| json!({}));

    // Dynamic SLA Evaluation based on active policies and priority
    let priority_val = c.priority.as_deref().unwrap_or("medium").to_lowercase();
    let sla_policy = sqlx::query_as::<_, SlaPolicy>(
        r#"
        SELECT id, name, description, first_response_time_threshold, resolution_time_threshold, only_during_business_hours, priorities
        FROM sla_policies
        WHERE is_active = 1
        ORDER BY CASE 
            WHEN priorities LIKE '%' || ? || '%' THEN 0 
            ELSE 1 
        END, first_response_time_threshold ASC
        LIMIT 1
        "#,
    )
    .bind(&priority_val)
    .fetch_optional(&state.db)
    .await
    .unwrap_or(None);

    let now = Utc::now();
    let created_dt = chrono::DateTime::parse_from_rfc3339(
        c.created_at.as_deref().unwrap_or(&c.updated_at),
    )
    .map(|dt| dt.with_timezone(&Utc))
    .unwrap_or(now);

    let first_outbound = messages.iter().find(|m| m.direction == "outbound");
    let first_response_at = first_outbound.map(|m| m.sent_at.clone());

    let policy_name = sla_policy
        .as_ref()
        .map(|p| p.name.clone())
        .unwrap_or_else(|| "Standard Customer Support SLA".to_string());
    let policy_id = sla_policy
        .as_ref()
        .map(|p| p.id.clone())
        .unwrap_or_else(|| "default-sla".to_string());
    let fr_threshold = sla_policy
        .as_ref()
        .and_then(|p| p.first_response_time_threshold)
        .unwrap_or(60);
    let res_threshold = sla_policy
        .as_ref()
        .and_then(|p| p.resolution_time_threshold)
        .unwrap_or(1440);

    let fr_target = created_dt + chrono::Duration::minutes(fr_threshold as i64);
    let res_target = created_dt + chrono::Duration::minutes(res_threshold as i64);

    let fr_fulfilled = first_response_at.is_some();
    let fr_breached = if let Some(ref fr_at) = first_response_at {
        chrono::DateTime::parse_from_rfc3339(fr_at)
            .map(|dt| dt.with_timezone(&Utc) > fr_target)
            .unwrap_or(false)
    } else {
        now > fr_target
    };

    let is_resolved = c.state == "resolved" || c.resolution_type.is_some();
    let res_breached = if is_resolved {
        chrono::DateTime::parse_from_rfc3339(&c.updated_at)
            .map(|dt| dt.with_timezone(&Utc) > res_target)
            .unwrap_or(false)
    } else {
        now > res_target
    };

    let minutes_to_fr = (fr_target - now).num_minutes();
    let minutes_to_res = (res_target - now).num_minutes();

    let sla_status_label = if fr_breached || res_breached {
        "breached"
    } else if minutes_to_fr < 15 && !fr_fulfilled {
        "warning"
    } else {
        "compliant"
    };

    let sla_obj = json!({
        "policyId": policy_id,
        "policyName": policy_name,
        "firstResponseThresholdMinutes": fr_threshold,
        "resolutionThresholdMinutes": res_threshold,
        "firstResponseTargetAt": fr_target.to_rfc3339(),
        "resolutionTargetAt": res_target.to_rfc3339(),
        "firstResponseAt": first_response_at,
        "firstResponseStatus": if fr_fulfilled { if fr_breached { "breached" } else { "fulfilled" } } else if fr_breached { "breached" } else { "active" },
        "firstResponseBreached": fr_breached,
        "resolutionAt": if is_resolved { Some(c.updated_at.clone()) } else { None },
        "resolutionStatus": if is_resolved { if res_breached { "breached" } else { "fulfilled" } } else if res_breached { "breached" } else { "active" },
        "resolutionBreached": res_breached,
        "isBreached": fr_breached || res_breached,
        "nearingBreach": minutes_to_fr > 0 && minutes_to_fr < 15 && !fr_fulfilled,
        "minutesToFirstResponseBreach": minutes_to_fr.max(0),
        "minutesToResolutionBreach": minutes_to_res.max(0),
        "evaluatedAt": now.to_rfc3339()
    });

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
            "messages": messages,
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
    let sender_name = payload.sender_name.unwrap_or_else(|| "Agent".to_string());

    // Fetch conversation & contact phone
    let conv = sqlx::query_as::<_, Conversation>(
        r#"
        SELECT 
            c.id, c.contact_id, ct.person_name as contact_name, ct.person_name as customer_name, ct.person_name as lead_name,
            ct.value as contact_phone, ct.value as phone, COALESCE(ct.channel, 'whatsapp') as channel, LOWER(c.state) as state, c.priority,
            c.resolution_type, c.broadcast_id, b.name as broadcast_name, c.inbox_id,
            c.team_id, tm.name as team_name, c.csat_score, c.human_takeover, c.assigned_to_id, u.name as assigned_to_name,
            c.snoozed_until, ct.language, c.created_at, c.updated_at, c.custom_attributes
        FROM conversations c
        LEFT JOIN contacts ct ON c.contact_id = ct.id
        LEFT JOIN broadcasts b ON c.broadcast_id = b.id
        LEFT JOIN teams tm ON c.team_id = tm.id
        LEFT JOIN team_members u ON c.assigned_to_id = u.id
        WHERE c.id = ?
        "#,
    )
    .bind(&id)
    .fetch_optional(&state.db)
    .await
    .unwrap_or(None);

    let c = match conv {
        Some(c) => c,
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({ "error": "Conversation not found" })),
            )
                .into_response();
        }
    };

    // Obtain Meta WhatsApp credentials from env or fallback to integration_settings table
    let access_token = if !state.config.meta_whatsapp_access_token.is_empty() {
        state.config.meta_whatsapp_access_token.clone()
    } else {
        sqlx::query_scalar::<_, String>(
            "SELECT meta_access_token FROM integration_settings WHERE meta_access_token IS NOT NULL AND meta_access_token != '' LIMIT 1",
        )
        .fetch_optional(&state.db)
        .await
        .unwrap_or(None)
        .unwrap_or_default()
    };

    let phone_number_id = if !state.config.meta_whatsapp_phone_number_id.is_empty() {
        state.config.meta_whatsapp_phone_number_id.clone()
    } else {
        sqlx::query_scalar::<_, String>(
            "SELECT meta_phone_number_id FROM integration_settings WHERE meta_phone_number_id IS NOT NULL AND meta_phone_number_id != '' LIMIT 1",
        )
        .fetch_optional(&state.db)
        .await
        .unwrap_or(None)
        .unwrap_or_default()
    };

    let api_version = if !state.config.meta_whatsapp_api_version.is_empty() {
        &state.config.meta_whatsapp_api_version
    } else {
        "v21.0"
    };

    let mut provider_msg_id = None;
    let mut delivery_status = if is_private { "internal".to_string() } else { "sent".to_string() };

    // If outbound WhatsApp message and Meta credentials present, dispatch to Meta API v21.0
    if !is_private && c.channel.as_deref().unwrap_or("whatsapp") == "whatsapp" {
        let recipient_phone = c.contact_phone.as_ref().or(c.phone.as_ref());
        if let Some(phone) = recipient_phone {
            if !access_token.is_empty() && !phone_number_id.is_empty() {
                let meta_url = format!(
                    "https://graph.facebook.com/{}/{}/messages",
                    api_version, phone_number_id
                );
                let clean_phone = phone.replace('+', "").replace(' ', "").replace('-', "");

                let meta_body = json!({
                    "messaging_product": "whatsapp",
                    "recipient_type": "individual",
                    "to": clean_phone,
                    "type": "text",
                    "text": { "preview_url": false, "body": payload.body }
                });

                match state
                    .http_client
                    .post(&meta_url)
                    .bearer_auth(&access_token)
                    .json(&meta_body)
                    .send()
                    .await
                {
                    Ok(res) => {
                        if res.status().is_success() {
                            if let Ok(json_res) = res.json::<serde_json::Value>().await {
                                if let Some(msg_obj) =
                                    json_res["messages"].as_array().and_then(|a| a.first())
                                {
                                    if let Some(mid) = msg_obj["id"].as_str() {
                                        provider_msg_id = Some(mid.to_string());
                                        delivery_status = "delivered".to_string();
                                        info!(
                                            "Dispatched live WhatsApp message to Meta Cloud API v21.0: {}",
                                            mid
                                        );
                                    }
                                }
                            }
                        } else {
                            error!("Meta WhatsApp send failed: status {}", res.status());
                        }
                    }
                    Err(e) => {
                        error!("Meta WhatsApp dispatch network error: {}", e);
                    }
                }
            }
        }
    }

    // Save message to database
    let _ = sqlx::query(
        r#"
        INSERT INTO messages (
            id, conversation_id, direction, channel, body, status, sent_at,
            provider, provider_message_id, is_private, sender_type, sender_name
        ) VALUES (?, ?, 'outbound', 'whatsapp', ?, ?, ?, 'meta', ?, ?, 'agent', ?)
        "#,
    )
    .bind(&msg_id)
    .bind(&id)
    .bind(&payload.body)
    .bind(&delivery_status)
    .bind(&now)
    .bind(&provider_msg_id)
    .bind(is_private)
    .bind(&sender_name)
    .execute(&state.db)
    .await;

    // Update conversation timestamp
    let _ = sqlx::query(
        r#"
        UPDATE conversations 
        SET updated_at = ?, last_outbound_at = ?
        WHERE id = ?
        "#,
    )
    .bind(&now)
    .bind(&now)
    .bind(&id)
    .execute(&state.db)
    .await;

    (
        StatusCode::OK,
        Json(json!({
            "id": msg_id,
            "conversationId": id,
            "direction": "outbound",
            "channel": "whatsapp",
            "body": payload.body,
            "intent": "",
            "status": delivery_status,
            "sentAt": now,
            "providerMessageId": provider_msg_id,
            "senderName": sender_name,
            "isPrivate": is_private
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
        "UPDATE conversations SET state = 'RESOLVED', resolution_type = 'resolved', updated_at = ? WHERE id = ?",
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
        _ => (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "Conversation not found" })),
        )
            .into_response(),
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
    let res = sqlx::query("UPDATE conversations SET priority = ?, updated_at = ? WHERE id = ?")
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
        _ => (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "Conversation not found" })),
        )
            .into_response(),
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
    let res = sqlx::query("UPDATE conversations SET team_id = ?, updated_at = ? WHERE id = ?")
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
        _ => (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "Conversation not found" })),
        )
            .into_response(),
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
    let res = sqlx::query("UPDATE conversations SET assigned_to_id = ?, updated_at = ? WHERE id = ?")
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
        _ => (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "Conversation not found" })),
        )
            .into_response(),
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
    let res = sqlx::query("UPDATE conversations SET snoozed_until = ?, updated_at = ? WHERE id = ?")
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
        _ => (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "Conversation not found" })),
        )
            .into_response(),
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
    let res = sqlx::query("UPDATE conversations SET custom_attributes = ?, updated_at = ? WHERE id = ?")
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
        _ => (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "Conversation not found" })),
        )
            .into_response(),
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

    let notes = sqlx::query_as::<_, NoteRow>(
        "SELECT id, contact_id, body, author_name, created_at FROM contact_notes WHERE contact_id = ? ORDER BY created_at DESC",
    )
    .bind(&contact_id)
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    Json(notes)
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
    let author = payload.author_name.unwrap_or_else(|| "Agent".to_string());

    let _ = sqlx::query(
        "INSERT INTO contact_notes (id, contact_id, body, author_name, created_at) VALUES (?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(&contact_id)
    .bind(&payload.body)
    .bind(&author)
    .bind(&now)
    .execute(&state.db)
    .await;

    (
        StatusCode::CREATED,
        Json(json!({
            "id": id,
            "contactId": contact_id,
            "body": payload.body,
            "authorName": author,
            "createdAt": now
        })),
    )
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
    let res = sqlx::query("UPDATE conversations SET sequence_id = ?, updated_at = ? WHERE id = ?")
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
        _ => (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "Conversation not found" })),
        )
            .into_response(),
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
    // If team_id is provided, check available members in that team, else across all online team members
    let member = if let Some(ref tid) = payload.team_id {
        sqlx::query_as::<_, TeamMember>(
            r#"
            SELECT tm.id, tm.name, tm.email, tm.role, tm.availability_status, tm.max_concurrent_conversations
            FROM team_members tm
            JOIN team_memberships tms ON tm.id = tms.team_member_id
            LEFT JOIN conversations c ON c.assigned_to_id = tm.id AND LOWER(c.state) NOT IN ('resolved', 'closed_won', 'closed_lost', 'opted_out')
            WHERE tms.team_id = ? AND (tm.availability_status = 'online' OR tm.availability_status IS NULL)
            GROUP BY tm.id
            ORDER BY COUNT(c.id) ASC
            LIMIT 1
            "#,
        )
        .bind(tid)
        .fetch_optional(&state.db)
        .await
        .unwrap_or(None)
    } else {
        None
    };

    let member = match member {
        Some(m) => Some(m),
        None => {
            sqlx::query_as::<_, TeamMember>(
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
            .unwrap_or(None)
        }
    };

    if let Some(m) = member {
        let now = Utc::now().to_rfc3339();
        let _ = sqlx::query(
            "UPDATE conversations SET assigned_to_id = ?, team_id = COALESCE(?, team_id), updated_at = ? WHERE id = ?",
        )
        .bind(&m.id)
        .bind(&payload.team_id)
        .bind(&now)
        .bind(&id)
        .execute(&state.db)
        .await;

        let _ = sqlx::query("UPDATE team_members SET last_assigned_at = ? WHERE id = ?")
            .bind(&now)
            .bind(&m.id)
            .execute(&state.db)
            .await;

        Json(json!({
            "assigned": true,
            "status": "assigned",
            "conversationId": id,
            "assignedAgentId": m.id,
            "assignedAgentName": m.name
        }))
    } else {
        Json(json!({
            "assigned": false,
            "status": "overflow",
            "conversationId": id,
            "message": "No online agents available"
        }))
    }
}
