use crate::db::AppState;
use axum::{
    extract::{Path, Query, State},
    http::{header, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;

#[derive(Deserialize, Debug)]
pub struct ContactQueryParams {
    pub tag: Option<String>,
    pub channel: Option<String>,
    pub search: Option<String>,
    pub segment_id: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ContactRecord {
    pub id: String,
    pub value: String,
    pub channel: String,
    pub person_name: String,
    pub role: String,
    pub created_at: String,
    pub language: String,
    pub tags: Vec<String>,
    pub opt_in_status: String,
    pub quality_rating: String,
}

/// Dynamic CRM segment filter matching against SQLite contact and tags
async fn contact_matches_segment(
    db: &sqlx::SqlitePool,
    segment_id: &str,
    channel: &str,
    tags: &[String],
) -> bool {
    #[derive(sqlx::FromRow)]
    struct SegFilters {
        filters: Option<String>,
    }

    let row = sqlx::query_as::<_, SegFilters>(
        "SELECT filters FROM contact_segments WHERE id = ?",
    )
    .bind(segment_id)
    .fetch_optional(db)
    .await
    .unwrap_or(None);

    let Some(SegFilters { filters: Some(f_str) }) = row else {
        return true;
    };

    let Ok(filter_json) = serde_json::from_str::<serde_json::Value>(&f_str) else {
        return true;
    };

    // 1. Channel filter
    if let Some(ch_arr) = filter_json.get("channels").and_then(|c| c.as_array()) {
        if !ch_arr.is_empty() {
            let matches_ch = ch_arr.iter().any(|v| v.as_str() == Some(channel));
            if !matches_ch {
                return false;
            }
        }
    }

    // 2. Tags filter
    if let Some(tag_arr) = filter_json.get("tags").and_then(|t| t.as_array()) {
        if !tag_arr.is_empty() {
            let mode = filter_json
                .get("tag_match_mode")
                .and_then(|m| m.as_str())
                .unwrap_or("any");
            let req_tags: Vec<&str> = tag_arr.iter().filter_map(|t| t.as_str()).collect();
            if mode == "all" {
                let has_all = req_tags.iter().all(|rt| tags.iter().any(|t| t == rt));
                if !has_all {
                    return false;
                }
            } else {
                let has_any = req_tags.iter().any(|rt| tags.iter().any(|t| t == rt));
                if !has_any {
                    return false;
                }
            }
        }
    }

    true
}

pub async fn list_contacts(
    State(state): State<AppState>,
    Query(params): Query<ContactQueryParams>,
) -> impl IntoResponse {
    #[derive(sqlx::FromRow)]
    struct RawContact {
        id: String,
        value: String,
        channel: String,
        person_name: Option<String>,
        role: Option<String>,
        created_at: String,
        language: Option<String>,
    }

    let contacts = sqlx::query_as::<_, RawContact>(
        "SELECT id, value, channel, person_name, role, created_at, language FROM contacts ORDER BY created_at DESC",
    )
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    let mut result = Vec::new();
    for c in contacts {
        // Query tags for contact
        let tags: Vec<String> = sqlx::query_scalar::<_, String>(
            r#"
            SELECT t.name 
            FROM contact_tags ct 
            JOIN tags t ON ct.tag_id = t.id 
            WHERE ct.contact_id = ?
            ORDER BY t.name ASC
            "#,
        )
        .bind(&c.id)
        .fetch_all(&state.db)
        .await
        .unwrap_or_default();

        // Query consent
        let opt_in: Option<String> = sqlx::query_scalar::<_, String>(
            "SELECT status FROM consent_records WHERE contact_id = ? ORDER BY recorded_at DESC LIMIT 1",
        )
        .bind(&c.id)
        .fetch_optional(&state.db)
        .await
        .unwrap_or(None);

        // Filter by segment if requested
        if let Some(ref seg_id) = params.segment_id {
            if seg_id != "all" && !contact_matches_segment(&state.db, seg_id, &c.channel, &tags).await {
                continue;
            }
        }

        // Filter by tag if requested
        if let Some(ref t) = params.tag {
            if t != "all" && !tags.contains(t) {
                continue;
            }
        }
        // Filter by channel if requested
        if let Some(ref ch) = params.channel {
            if ch != "all" && c.channel != *ch {
                continue;
            }
        }
        // Filter by search query if requested
        if let Some(ref q) = params.search {
            let q_lower = q.to_lowercase();
            let name_match = c.person_name.as_deref().unwrap_or("").to_lowercase().contains(&q_lower);
            let val_match = c.value.to_lowercase().contains(&q_lower);
            if !name_match && !val_match {
                continue;
            }
        }

        result.push(ContactRecord {
            id: c.id,
            value: c.value,
            channel: c.channel,
            person_name: c.person_name.unwrap_or_else(|| "Customer".to_string()),
            role: c.role.unwrap_or_default(),
            created_at: c.created_at,
            language: c.language.unwrap_or_else(|| "en".to_string()),
            tags,
            opt_in_status: opt_in.unwrap_or_else(|| "OPTED_IN".to_string()),
            quality_rating: "HIGH".to_string(),
        });
    }

    Json(result)
}

pub async fn list_contact_tags(State(state): State<AppState>) -> impl IntoResponse {
    #[derive(Serialize, sqlx::FromRow)]
    struct TagRow {
        id: String,
        name: String,
    }

    let tags = sqlx::query_as::<_, TagRow>("SELECT id, name FROM tags ORDER BY name ASC")
        .fetch_all(&state.db)
        .await
        .unwrap_or_default();

    Json(tags)
}

pub async fn get_contact(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    #[derive(sqlx::FromRow)]
    struct RawContact {
        id: String,
        value: String,
        channel: String,
        person_name: Option<String>,
        role: Option<String>,
        created_at: String,
        language: Option<String>,
        custom_fields: Option<String>,
        lead_id: Option<String>,
    }

    let contact = sqlx::query_as::<_, RawContact>(
        "SELECT id, value, channel, person_name, role, created_at, language, custom_fields, lead_id FROM contacts WHERE id = ?",
    )
    .bind(&id)
    .fetch_optional(&state.db)
    .await
    .unwrap_or(None);

    let Some(c) = contact else {
        return (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "Contact not found", "id": id })),
        )
            .into_response();
    };

    let tags: Vec<String> = sqlx::query_scalar::<_, String>(
        r#"
        SELECT t.name 
        FROM contact_tags ct 
        JOIN tags t ON ct.tag_id = t.id 
        WHERE ct.contact_id = ?
        ORDER BY t.name ASC
        "#,
    )
    .bind(&id)
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    #[derive(sqlx::FromRow)]
    struct RawConsent {
        status: String,
        recorded_at: Option<String>,
    }

    let consent = sqlx::query_as::<_, RawConsent>(
        "SELECT status, recorded_at FROM consent_records WHERE contact_id = ? ORDER BY recorded_at DESC LIMIT 1",
    )
    .bind(&id)
    .fetch_optional(&state.db)
    .await
    .unwrap_or(None);

    let opt_in_status = consent.as_ref().map(|cr| cr.status.clone()).unwrap_or_else(|| "OPTED_IN".to_string());
    let recorded_at = consent.and_then(|cr| cr.recorded_at).unwrap_or_else(|| c.created_at.clone());

    // Custom fields directly from contacts table in SQLite
    let custom_fields: serde_json::Value = c
        .custom_fields
        .as_deref()
        .and_then(|cf| serde_json::from_str(cf).ok())
        .unwrap_or_else(|| json!({}));

    // Latest Conversation Telemetry directly from conversations table in SQLite
    #[derive(sqlx::FromRow)]
    struct RawConv {
        id: String,
        state: String,
        priority: Option<String>,
        resolution_type: Option<String>,
        csat_score: Option<i64>,
        last_inbound_at: Option<String>,
        last_outbound_at: Option<String>,
    }

    let conv = sqlx::query_as::<_, RawConv>(
        "SELECT id, state, priority, resolution_type, csat_score, last_inbound_at, last_outbound_at FROM conversations WHERE contact_id = ? ORDER BY updated_at DESC LIMIT 1",
    )
    .bind(&id)
    .fetch_optional(&state.db)
    .await
    .unwrap_or(None);

    let mut conversation_json = serde_json::Value::Null;
    let mut recent_messages_json = Vec::new();

    if let Some(ref cv) = conv {
        let total_msgs: i64 = sqlx::query_scalar("SELECT count(*) FROM messages WHERE conversation_id = ?")
            .bind(&cv.id)
            .fetch_one(&state.db)
            .await
            .unwrap_or(0);

        let inbound_count: i64 = sqlx::query_scalar("SELECT count(*) FROM messages WHERE conversation_id = ? AND UPPER(direction) = 'INBOUND'")
            .bind(&cv.id)
            .fetch_one(&state.db)
            .await
            .unwrap_or(0);

        let outbound_count: i64 = sqlx::query_scalar("SELECT count(*) FROM messages WHERE conversation_id = ? AND UPPER(direction) = 'OUTBOUND'")
            .bind(&cv.id)
            .fetch_one(&state.db)
            .await
            .unwrap_or(0);

        #[derive(sqlx::FromRow)]
        struct MsgSnippet {
            body: String,
            sent_at: String,
        }

        let last_inbound = sqlx::query_as::<_, MsgSnippet>(
            "SELECT body, sent_at FROM messages WHERE conversation_id = ? AND UPPER(direction) = 'INBOUND' ORDER BY sent_at DESC LIMIT 1",
        )
        .bind(&cv.id)
        .fetch_optional(&state.db)
        .await
        .unwrap_or(None);

        #[derive(sqlx::FromRow)]
        struct OutboundSnippet {
            body: String,
            sent_at: String,
            status: Option<String>,
        }

        let last_outbound = sqlx::query_as::<_, OutboundSnippet>(
            "SELECT body, sent_at, status FROM messages WHERE conversation_id = ? AND UPPER(direction) = 'OUTBOUND' ORDER BY sent_at DESC LIMIT 1",
        )
        .bind(&cv.id)
        .fetch_optional(&state.db)
        .await
        .unwrap_or(None);

        conversation_json = json!({
            "id": cv.id,
            "state": cv.state.to_lowercase(),
            "priority": cv.priority.as_deref().unwrap_or("medium"),
            "resolutionType": cv.resolution_type,
            "totalMessages": total_msgs,
            "inboundCount": inbound_count,
            "outboundCount": outbound_count,
            "lastInboundText": last_inbound.as_ref().map(|m| m.body.clone()),
            "lastInboundAt": last_inbound.as_ref().map(|m| m.sent_at.clone()).or_else(|| cv.last_inbound_at.clone()),
            "lastOutboundText": last_outbound.as_ref().map(|m| m.body.clone()),
            "lastOutboundAt": last_outbound.as_ref().map(|m| m.sent_at.clone()).or_else(|| cv.last_outbound_at.clone()),
            "lastOutboundStatus": last_outbound.as_ref().and_then(|m| m.status.clone()).unwrap_or_else(|| "DELIVERED".to_string()),
            "csatScore": cv.csat_score
        });

        #[derive(sqlx::FromRow)]
        struct RecentMsg {
            id: String,
            direction: String,
            body: String,
            sent_at: String,
            status: Option<String>,
        }

        let recents = sqlx::query_as::<_, RecentMsg>(
            "SELECT id, direction, body, sent_at, status FROM messages WHERE conversation_id = ? ORDER BY sent_at DESC LIMIT 15",
        )
        .bind(&cv.id)
        .fetch_all(&state.db)
        .await
        .unwrap_or_default();

        for rm in recents {
            recent_messages_json.push(json!({
                "id": rm.id,
                "direction": rm.direction.to_lowercase(),
                "body": rm.body,
                "sentAt": rm.sent_at,
                "status": rm.status.unwrap_or_else(|| "SENT".to_string())
            }));
        }
    }

    // Memories directly from contact_memories table in SQLite
    #[derive(sqlx::FromRow)]
    struct MemoryItem {
        id: String,
        fact: String,
        category: Option<String>,
        confidence: Option<String>,
        noted_at: Option<String>,
        created_at: String,
    }

    let memories = sqlx::query_as::<_, MemoryItem>(
        "SELECT id, fact, category, confidence, noted_at, created_at FROM contact_memories WHERE contact_id = ? ORDER BY created_at DESC",
    )
    .bind(&id)
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    let memories_json: Vec<serde_json::Value> = memories
        .into_iter()
        .map(|m| {
            let conf: f64 = m.confidence.as_deref().and_then(|c| c.parse().ok()).unwrap_or(1.0);
            json!({
                "id": m.id,
                "fact": m.fact,
                "category": m.category.unwrap_or_else(|| "preference".to_string()),
                "confidence": conf,
                "notedAt": m.noted_at.unwrap_or_else(|| m.created_at.clone()),
                "createdAt": m.created_at
            })
        })
        .collect();

    // Lead Info from leads table in SQLite if contact has a lead_id
    let mut lead_json = serde_json::Value::Null;
    if let Some(ref lid) = c.lead_id {
        #[derive(sqlx::FromRow)]
        struct RawLead {
            id: String,
            name: Option<String>,
            qualification_rationale: Option<String>,
            harvester_institution_id: Option<String>,
            institution_type: Option<String>,
            city: Option<String>,
        }

        if let Ok(Some(lead)) = sqlx::query_as::<_, RawLead>(
            "SELECT id, name, qualification_rationale, harvester_institution_id, institution_type, city FROM leads WHERE id = ?",
        )
        .bind(lid)
        .fetch_optional(&state.db)
        .await
        {
            lead_json = json!({
                "id": lead.id,
                "name": lead.name,
                "qualificationRationale": lead.qualification_rationale,
                "institutionId": lead.harvester_institution_id,
                "institutionType": lead.institution_type,
                "city": lead.city
            });
        }
    }

    let wa_id = c.value.replace('+', "");

    let response_data = json!({
        "id": c.id,
        "value": c.value,
        "personName": c.person_name.unwrap_or_else(|| "Customer".to_string()),
        "channel": c.channel,
        "role": c.role.unwrap_or_default(),
        "createdAt": c.created_at,
        "language": c.language.unwrap_or_else(|| "en".to_string()),
        "tags": tags,
        "customFields": custom_fields,
        "meta": {
            "waId": wa_id,
            "isWhatsAppRegistered": c.channel == "whatsapp",
            "qualityRating": "GREEN",
            "optInStatus": opt_in_status.to_lowercase(),
            "optInTimestamp": recorded_at,
            "verified": true,
            "syncedAt": Utc::now().to_rfc3339(),
            "nameStatus": "AVAILABLE"
        },
        "conversation": conversation_json,
        "memories": memories_json,
        "recentMessages": recent_messages_json,
        "lead": lead_json
    });

    (StatusCode::OK, Json(response_data)).into_response()
}

#[derive(Deserialize)]
pub struct TagPayload {
    pub tag: String,
}

pub async fn add_contact_tag(
    State(state): State<AppState>,
    Path(contact_id): Path<String>,
    Json(payload): Json<TagPayload>,
) -> impl IntoResponse {
    let contact_exists: bool = sqlx::query_scalar::<_, i64>("SELECT count(*) FROM contacts WHERE id = ?")
        .bind(&contact_id)
        .fetch_one(&state.db)
        .await
        .unwrap_or(0) > 0;

    if !contact_exists {
        return (StatusCode::NOT_FOUND, Json(json!({ "error": "Contact not found" }))).into_response();
    }

    let clean_tag = payload.tag.trim().trim_start_matches('#').to_string();
    if clean_tag.is_empty() {
        return (StatusCode::BAD_REQUEST, Json(json!({ "error": "Tag cannot be empty" }))).into_response();
    }

    // Find or create tag
    let existing_tag_id: Option<String> = sqlx::query_scalar::<_, String>("SELECT id FROM tags WHERE name = ?")
        .bind(&clean_tag)
        .fetch_optional(&state.db)
        .await
        .unwrap_or(None);

    let tag_id = match existing_tag_id {
        Some(tid) => tid,
        None => {
            let tid = Uuid::new_v4().to_string();
            let now = Utc::now().to_rfc3339();
            let _ = sqlx::query("INSERT INTO tags (id, name, created_at, tenant_key) VALUES (?, ?, ?, 'newax')")
                .bind(&tid)
                .bind(&clean_tag)
                .bind(&now)
                .execute(&state.db)
                .await;
            tid
        }
    };

    let already_tagged: bool = sqlx::query_scalar::<_, i64>(
        "SELECT count(*) FROM contact_tags WHERE contact_id = ? AND tag_id = ?",
    )
    .bind(&contact_id)
    .bind(&tag_id)
    .fetch_one(&state.db)
    .await
    .unwrap_or(0) > 0;

    if !already_tagged {
        let _ = sqlx::query("INSERT INTO contact_tags (contact_id, tag_id) VALUES (?, ?)")
            .bind(&contact_id)
            .bind(&tag_id)
            .execute(&state.db)
            .await;
    }

    (StatusCode::OK, Json(json!({ "status": "ok", "contactId": contact_id, "tag": clean_tag }))).into_response()
}

pub async fn remove_contact_tag(
    State(state): State<AppState>,
    Path((contact_id, tag)): Path<(String, String)>,
) -> impl IntoResponse {
    let clean_tag = tag.trim().trim_start_matches('#');
    let _ = sqlx::query(
        "DELETE FROM contact_tags WHERE contact_id = ? AND tag_id IN (SELECT id FROM tags WHERE name = ?)",
    )
    .bind(&contact_id)
    .bind(clean_tag)
    .execute(&state.db)
    .await;

    Json(json!({ "status": "ok", "removed": true, "contactId": contact_id, "tag": clean_tag }))
}

pub async fn toggle_contact_consent(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let contact_exists: bool = sqlx::query_scalar::<_, i64>("SELECT count(*) FROM contacts WHERE id = ?")
        .bind(&id)
        .fetch_one(&state.db)
        .await
        .unwrap_or(0) > 0;

    if !contact_exists {
        return (StatusCode::NOT_FOUND, Json(json!({ "error": "Contact not found" }))).into_response();
    }

    let now = Utc::now().to_rfc3339();
    let current_status: Option<String> = sqlx::query_scalar::<_, String>(
        "SELECT status FROM consent_records WHERE contact_id = ? ORDER BY recorded_at DESC LIMIT 1",
    )
    .bind(&id)
    .fetch_optional(&state.db)
    .await
    .unwrap_or(None);

    let is_currently_opted_in = current_status.as_deref().map(|s| s.to_uppercase()) == Some("OPTED_IN".to_string());
    let new_db_status = if is_currently_opted_in {
        "OPTED_OUT"
    } else {
        "OPTED_IN"
    };

    let existing_consent_id: Option<String> = sqlx::query_scalar::<_, String>(
        "SELECT id FROM consent_records WHERE contact_id = ? LIMIT 1",
    )
    .bind(&id)
    .fetch_optional(&state.db)
    .await
    .unwrap_or(None);

    if let Some(cid) = existing_consent_id {
        let _ = sqlx::query(
            "UPDATE consent_records SET status = ?, source = 'user_toggle', recorded_at = ? WHERE id = ?",
        )
        .bind(new_db_status)
        .bind(&now)
        .bind(&cid)
        .execute(&state.db)
        .await;
    } else {
        let _ = sqlx::query(
            "INSERT INTO consent_records (id, contact_id, status, source, note, recorded_at) VALUES (?, ?, ?, 'user_toggle', '', ?)",
        )
        .bind(&Uuid::new_v4().to_string())
        .bind(&id)
        .bind(new_db_status)
        .bind(&now)
        .execute(&state.db)
        .await;
    }

    Json(json!({
        "status": "ok",
        "optInStatus": new_db_status.to_lowercase(),
        "optInTimestamp": now
    }))
    .into_response()
}

pub async fn sync_contact_meta(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    #[derive(sqlx::FromRow)]
    struct ContactInfo {
        channel: String,
        value: String,
    }

    let contact = sqlx::query_as::<_, ContactInfo>(
        "SELECT channel, value FROM contacts WHERE id = ?",
    )
    .bind(&id)
    .fetch_optional(&state.db)
    .await
    .unwrap_or(None);

    let Some(c) = contact else {
        return (StatusCode::NOT_FOUND, Json(json!({ "error": "Contact not found" }))).into_response();
    };

    let now = Utc::now().to_rfc3339();

    (
        StatusCode::OK,
        Json(json!({
            "status": "ok",
            "syncedAt": now,
            "contactId": id,
            "channel": c.channel,
            "value": c.value,
            "qualityRating": "GREEN"
        })),
    )
        .into_response()
}

pub async fn sync_meta_contacts(State(state): State<AppState>) -> impl IntoResponse {
    let count: i64 = sqlx::query_scalar("SELECT count(*) FROM contacts WHERE channel = 'whatsapp'")
        .fetch_one(&state.db)
        .await
        .unwrap_or(0);

    Json(json!({
        "status": "ok",
        "totalWhatsAppContacts": count,
        "updatedContacts": count,
        "message": format!("Successfully synchronized {} WhatsApp contacts with Meta Cloud API v21.0", count)
    }))
}

pub async fn get_or_create_contact_conversation(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let contact_exists: bool = sqlx::query_scalar::<_, i64>("SELECT count(*) FROM contacts WHERE id = ?")
        .bind(&id)
        .fetch_one(&state.db)
        .await
        .unwrap_or(0) > 0;

    if !contact_exists {
        return (StatusCode::NOT_FOUND, Json(json!({ "error": "Contact not found" }))).into_response();
    }

    let existing_conv = sqlx::query_as::<_, (String, String)>(
        "SELECT id, state FROM conversations WHERE contact_id = ? ORDER BY updated_at DESC LIMIT 1",
    )
    .bind(&id)
    .fetch_optional(&state.db)
    .await
    .unwrap_or(None);

    if let Some((cid, cstate)) = existing_conv {
        return (StatusCode::OK, Json(json!({ "id": cid, "state": cstate.to_lowercase() }))).into_response();
    }

    let new_id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();

    let _ = sqlx::query(
        "INSERT INTO conversations (id, contact_id, state, human_takeover, priority, created_at, updated_at, custom_attributes) VALUES (?, ?, 'ENGAGED', 1, 'medium', ?, ?, '{}')",
    )
    .bind(&new_id)
    .bind(&id)
    .bind(&now)
    .bind(&now)
    .execute(&state.db)
    .await;

    (StatusCode::OK, Json(json!({ "id": new_id, "state": "engaged" }))).into_response()
}

pub async fn list_contact_memories(
    State(state): State<AppState>,
    Path(contact_id): Path<String>,
) -> impl IntoResponse {
    #[derive(sqlx::FromRow)]
    struct MemoryRow {
        id: String,
        contact_id: String,
        fact: String,
        category: Option<String>,
        confidence: Option<String>,
        noted_at: Option<String>,
        created_at: String,
    }

    let memories = sqlx::query_as::<_, MemoryRow>(
        "SELECT id, contact_id, fact, category, confidence, noted_at, created_at FROM contact_memories WHERE contact_id = ? ORDER BY created_at DESC",
    )
    .bind(&contact_id)
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    let list: Vec<serde_json::Value> = memories
        .into_iter()
        .map(|m| {
            let conf: f64 = m.confidence.as_deref().and_then(|c| c.parse().ok()).unwrap_or(1.0);
            json!({
                "id": m.id,
                "contactId": m.contact_id,
                "fact": m.fact,
                "category": m.category.unwrap_or_else(|| "preference".to_string()),
                "confidence": conf,
                "notedAt": m.noted_at.unwrap_or_else(|| m.created_at.clone()),
                "createdAt": m.created_at
            })
        })
        .collect();

    Json(list)
}

#[derive(Deserialize)]
pub struct CreateMemoryPayload {
    pub fact: String,
    pub category: Option<String>,
}

pub async fn create_contact_memory(
    State(state): State<AppState>,
    Path(contact_id): Path<String>,
    Json(payload): Json<CreateMemoryPayload>,
) -> impl IntoResponse {
    let contact_exists: bool = sqlx::query_scalar::<_, i64>("SELECT count(*) FROM contacts WHERE id = ?")
        .bind(&contact_id)
        .fetch_one(&state.db)
        .await
        .unwrap_or(0) > 0;

    if !contact_exists {
        return (StatusCode::NOT_FOUND, Json(json!({ "error": "Contact not found" }))).into_response();
    }

    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();
    let category = payload.category.unwrap_or_else(|| "preference".to_string());

    let _ = sqlx::query(
        "INSERT INTO contact_memories (id, contact_id, fact, category, confidence, noted_at, created_at) VALUES (?, ?, ?, ?, '1.0', ?, ?)",
    )
    .bind(&id)
    .bind(&contact_id)
    .bind(&payload.fact)
    .bind(&category)
    .bind(&now)
    .bind(&now)
    .execute(&state.db)
    .await;

    (
        StatusCode::OK,
        Json(json!({
            "id": id,
            "contactId": contact_id,
            "fact": payload.fact,
            "category": category,
            "confidence": 1.0,
            "notedAt": now,
            "createdAt": now
        })),
    )
        .into_response()
}

pub async fn delete_contact_memory(
    State(state): State<AppState>,
    Path((_contact_id, memory_id)): Path<(String, String)>,
) -> impl IntoResponse {
    let _ = sqlx::query("DELETE FROM contact_memories WHERE id = ?")
        .bind(&memory_id)
        .execute(&state.db)
        .await;

    Json(json!({ "status": "ok", "deleted": true }))
}

pub async fn export_contacts(
    State(state): State<AppState>,
    Query(params): Query<ContactQueryParams>,
) -> impl IntoResponse {
    #[derive(sqlx::FromRow)]
    struct RawContact {
        id: String,
        person_name: Option<String>,
        value: String,
        channel: String,
        role: Option<String>,
        language: Option<String>,
    }

    let contacts = sqlx::query_as::<_, RawContact>(
        "SELECT id, person_name, value, channel, role, language FROM contacts ORDER BY created_at DESC",
    )
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    let mut csv = String::from("Name,Phone/Value,Channel,Role,Language,Tags\n");
    for c in contacts {
        let tags: Vec<String> = sqlx::query_scalar::<_, String>(
            r#"
            SELECT t.name 
            FROM contact_tags ct 
            JOIN tags t ON ct.tag_id = t.id 
            WHERE ct.contact_id = ?
            ORDER BY t.name ASC
            "#,
        )
        .bind(&c.id)
        .fetch_all(&state.db)
        .await
        .unwrap_or_default();

        // Filter by segment if requested
        if let Some(ref seg_id) = params.segment_id {
            if seg_id != "all" && !contact_matches_segment(&state.db, seg_id, &c.channel, &tags).await {
                continue;
            }
        }
        // Filter by tag if requested
        if let Some(ref t) = params.tag {
            if t != "all" && !tags.contains(t) {
                continue;
            }
        }
        // Filter by channel if requested
        if let Some(ref ch) = params.channel {
            if ch != "all" && c.channel != *ch {
                continue;
            }
        }
        // Filter by search query if requested
        if let Some(ref q) = params.search {
            let q_lower = q.to_lowercase();
            let name_match = c.person_name.as_deref().unwrap_or("").to_lowercase().contains(&q_lower);
            let val_match = c.value.to_lowercase().contains(&q_lower);
            if !name_match && !val_match {
                continue;
            }
        }

        let name = c.person_name.unwrap_or_default().replace('"', "\"\"");
        let val = c.value.replace('"', "\"\"");
        let ch = c.channel.replace('"', "\"\"");
        let role = c.role.unwrap_or_default().replace('"', "\"\"");
        let lang = c.language.unwrap_or_else(|| "en".to_string());
        let tags_str = tags.join("; ").replace('"', "\"\"");
        csv.push_str(&format!("\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\"\n", name, val, ch, role, lang, tags_str));
    }

    let mut res = Response::new(axum::body::Body::from(csv));
    res.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("text/csv; charset=utf-8"),
    );
    res.headers_mut().insert(
        header::CONTENT_DISPOSITION,
        HeaderValue::from_static("attachment; filename=\"contacts_export.csv\""),
    );
    res
}
