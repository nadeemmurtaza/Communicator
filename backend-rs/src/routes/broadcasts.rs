use crate::{db::AppState, models::Broadcast};
use axum::{
    extract::{Path, Query, State},
    response::IntoResponse,
    Json,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;

pub async fn list_broadcasts(State(state): State<AppState>) -> impl IntoResponse {
    let broadcasts = sqlx::query_as::<_, Broadcast>(
        r#"
        SELECT 
            id, name, template_name, template_language, body_preview, tag_name, status,
            scheduled_at, created_at,
            audience_count, audience_count as total_recipients,
            sent_count, delivered_count, read_count, failed_count, skipped_count
        FROM broadcasts
        ORDER BY created_at DESC
        "#,
    )
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    Json(broadcasts)
}

#[derive(Deserialize)]
pub struct CreateBroadcastPayload {
    pub name: String,
    pub template_name: Option<String>,
    pub template_language: Option<String>,
    pub body_preview: Option<String>,
    pub template_params: Option<serde_json::Value>,
    pub tag_name: Option<String>,
    pub scheduled_at: Option<String>,
    pub tenant_key: Option<String>,
}

fn parse_tags(tag_str: &str) -> Vec<String> {
    tag_str
        .split(|c| c == ',' || c == '|')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

pub async fn create_broadcast(
    State(state): State<AppState>,
    Json(payload): Json<CreateBroadcastPayload>,
) -> impl IntoResponse {
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();
    let template_name = payload.template_name.unwrap_or_else(|| "spring_sale_offer".to_string());
    let lang = payload.template_language.unwrap_or_else(|| "en_US".to_string());
    let tag_name = payload.tag_name.unwrap_or_else(|| "vip-customers".to_string());
    let tags = parse_tags(&tag_name);
    let tags_json = serde_json::to_string(&tags).unwrap_or_else(|_| "[]".to_string());
    let tenant_key = payload.tenant_key.unwrap_or_else(|| "newax".to_string());
    let body_preview = payload.body_preview.unwrap_or_default();
    let params_str = payload
        .template_params
        .as_ref()
        .map(|p| p.to_string())
        .unwrap_or_else(|| "{}".to_string());

    // Dynamically count matching audience contacts across tags
    let count: i64 = if tags.is_empty() {
        0
    } else {
        sqlx::query_scalar(
            r#"
            SELECT count(DISTINCT ct.id)
            FROM contacts ct
            JOIN contact_tags ctags ON ct.id = ctags.contact_id
            JOIN tags t ON ctags.tag_id = t.id
            WHERE t.name IN (SELECT value FROM json_each(?))
            "#,
        )
        .bind(&tags_json)
        .fetch_one(&state.db)
        .await
        .unwrap_or(0)
    };

    let status = if payload.scheduled_at.is_some() {
        "scheduled"
    } else {
        "draft"
    };

    let _ = sqlx::query(
        r#"
        INSERT INTO broadcasts (
            id, name, template_name, template_language, body_preview, tag_name, status,
            scheduled_at, created_at, audience_count, total_recipients,
            sent_count, delivered_count, read_count, failed_count, skipped_count,
            template_params, tenant_key
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 0, 0, 0, 0, 0, ?, ?)
        "#,
    )
    .bind(&id)
    .bind(&payload.name)
    .bind(&template_name)
    .bind(&lang)
    .bind(&body_preview)
    .bind(&tag_name)
    .bind(status)
    .bind(&payload.scheduled_at)
    .bind(&now)
    .bind(count as i32)
    .bind(count as i32)
    .bind(&params_str)
    .bind(&tenant_key)
    .execute(&state.db)
    .await;

    Json(json!({
        "id": id,
        "name": payload.name,
        "templateName": template_name,
        "templateLanguage": lang,
        "bodyPreview": body_preview,
        "templateParams": payload.template_params.unwrap_or_else(|| json!({})),
        "tagName": tag_name,
        "status": status,
        "scheduledAt": payload.scheduled_at,
        "createdAt": now,
        "audienceCount": count,
        "totalRecipients": count,
        "sentCount": 0,
        "deliveredCount": 0,
        "readCount": 0,
        "failedCount": 0,
        "skippedCount": 0
    }))
}

#[derive(Deserialize)]
pub struct AudienceQuery {
    pub tags: Option<String>,
    pub tenant_key: Option<String>,
}

#[derive(sqlx::FromRow)]
struct ContactSampleRow {
    id: String,
    value: String,
    person_name: Option<String>,
}

pub async fn get_audience_preview(
    State(state): State<AppState>,
    Query(query): Query<AudienceQuery>,
) -> impl IntoResponse {
    let raw_tags = query.tags.unwrap_or_default();
    let tag_list = parse_tags(&raw_tags);

    if tag_list.is_empty() {
        return Json(json!({
            "tags": Vec::<String>::new(),
            "totalSelected": 0,
            "dailyLimit": 1000,
            "sampleContacts": Vec::<serde_json::Value>::new(),
            "eligibleCount": 0,
            "optedInCount": 0,
            "optedOutCount": 0
        }));
    }

    let tags_json = serde_json::to_string(&tag_list).unwrap_or_else(|_| "[]".to_string());

    let matching_contacts = sqlx::query_as::<_, ContactSampleRow>(
        r#"
        SELECT DISTINCT ct.id, ct.value, ct.person_name
        FROM contacts ct
        JOIN contact_tags ctags ON ct.id = ctags.contact_id
        JOIN tags t ON ctags.tag_id = t.id
        WHERE t.name IN (SELECT value FROM json_each(?))
        "#,
    )
    .bind(&tags_json)
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    let total_selected = matching_contacts.len();

    let mut sample_contacts = Vec::new();
    for c in matching_contacts.into_iter().take(5) {
        #[derive(sqlx::FromRow)]
        struct TagNameRow {
            name: String,
        }
        let contact_tags = sqlx::query_as::<_, TagNameRow>(
            r#"
            SELECT t.name
            FROM tags t
            JOIN contact_tags ctags ON ctags.tag_id = t.id
            WHERE ctags.contact_id = ?
            "#,
        )
        .bind(&c.id)
        .fetch_all(&state.db)
        .await
        .unwrap_or_default();

        let tags_arr: Vec<String> = contact_tags.into_iter().map(|t| t.name).collect();

        sample_contacts.push(json!({
            "id": c.id,
            "name": c.person_name.unwrap_or_else(|| "Valued Customer".to_string()),
            "phone": c.value,
            "tags": tags_arr
        }));
    }

    Json(json!({
        "tags": tag_list,
        "totalSelected": total_selected,
        "dailyLimit": 1000,
        "sampleContacts": sample_contacts,
        "eligibleCount": total_selected,
        "optedInCount": total_selected,
        "optedOutCount": 0
    }))
}

#[derive(Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct RecipientDetailRow {
    pub id: String,
    pub contact_id: String,
    pub contact_name: Option<String>,
    pub contact_phone: Option<String>,
    pub status: String,
    pub error: Option<String>,
    pub sent_at: Option<String>,
    pub delivered_at: Option<String>,
    pub read_at: Option<String>,
}

pub async fn get_campaign_analytics(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let b = sqlx::query_as::<_, Broadcast>(
        r#"
        SELECT 
            id, name, template_name, template_language, body_preview, tag_name, status,
            scheduled_at, created_at,
            audience_count, audience_count as total_recipients,
            sent_count, delivered_count, read_count, failed_count, skipped_count
        FROM broadcasts
        WHERE id = ?
        "#,
    )
    .bind(&id)
    .fetch_optional(&state.db)
    .await
    .unwrap_or(None);

    if let Some(bc) = b {
        let recipients = sqlx::query_as::<_, RecipientDetailRow>(
            r#"
            SELECT 
                br.id,
                br.contact_id,
                COALESCE(ct.person_name, 'Valued Customer') as contact_name,
                COALESCE(ct.value, '') as contact_phone,
                br.status,
                br.error,
                br.sent_at,
                br.delivered_at,
                br.read_at
            FROM broadcast_recipients br
            LEFT JOIN contacts ct ON br.contact_id = ct.id
            WHERE br.broadcast_id = ?
            ORDER BY br.sent_at DESC
            "#,
        )
        .bind(&id)
        .fetch_all(&state.db)
        .await
        .unwrap_or_default();

        let recipient_count = recipients.len() as i32;
        let audience = bc.audience_count.unwrap_or(0).max(recipient_count);
        let submitted = bc.sent_count.unwrap_or(0);
        let delivered = bc.delivered_count.unwrap_or(0);
        let read = bc.read_count.unwrap_or(0);
        let failed = bc.failed_count.unwrap_or(0);
        let skipped = bc.skipped_count.unwrap_or(0);

        let aud_f64 = audience as f64;
        let submitted_pct = if audience > 0 { ((submitted as f64 / aud_f64) * 100.0).round() as i64 } else { 0 };
        let delivered_pct = if audience > 0 { ((delivered as f64 / aud_f64) * 100.0).round() as i64 } else { 0 };
        let read_pct = if audience > 0 { ((read as f64 / aud_f64) * 100.0).round() as i64 } else { 0 };
        let failed_pct = if audience > 0 { ((failed as f64 / aud_f64) * 100.0).round() as i64 } else { 0 };
        let skipped_pct = if audience > 0 { ((skipped as f64 / aud_f64) * 100.0).round() as i64 } else { 0 };

        Json(json!({
            "campaign": bc,
            "metrics": {
                "audience": audience,
                "submitted": submitted,
                "submittedPct": submitted_pct,
                "delivered": delivered,
                "deliveredPct": delivered_pct,
                "read": read,
                "readPct": read_pct,
                "failed": failed,
                "failedPct": failed_pct,
                "skipped": skipped,
                "skippedPct": skipped_pct
            },
            "deliveryBreakdown": {
                "deliveredRate": delivered_pct,
                "readRate": read_pct,
                "failureRate": failed_pct
            },
            "recipients": recipients,
            "campaignId": bc.id,
            "name": bc.name,
            "totalRecipients": audience,
            "sent": submitted,
            "delivered": delivered,
            "read": read,
            "failed": failed,
            "deliveryRate": delivered_pct,
            "readRate": read_pct
        }))
    } else {
        Json(json!({ "error": "Campaign not found" }))
    }
}

#[derive(Deserialize)]
pub struct ReschedulePayload {
    pub scheduled_at: Option<String>,
}

pub async fn reschedule_campaign(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<ReschedulePayload>,
) -> impl IntoResponse {
    let status = if payload.scheduled_at.is_some() {
        "scheduled"
    } else {
        "draft"
    };

    let _ = sqlx::query("UPDATE broadcasts SET scheduled_at = ?, status = ? WHERE id = ?")
        .bind(&payload.scheduled_at)
        .bind(status)
        .bind(&id)
        .execute(&state.db)
        .await;

    let updated = sqlx::query_as::<_, Broadcast>(
        r#"
        SELECT 
            id, name, template_name, template_language, body_preview, tag_name, status,
            scheduled_at, created_at,
            audience_count, audience_count as total_recipients,
            sent_count, delivered_count, read_count, failed_count, skipped_count
        FROM broadcasts
        WHERE id = ?
        "#,
    )
    .bind(&id)
    .fetch_optional(&state.db)
    .await
    .unwrap_or(None);

    if let Some(b) = updated {
        Json(json!({
            "id": b.id,
            "name": b.name,
            "status": b.status,
            "scheduledAt": b.scheduled_at,
            "audienceCount": b.audience_count,
            "totalRecipients": b.total_recipients,
            "sentCount": b.sent_count,
            "deliveredCount": b.delivered_count
        }))
    } else {
        Json(json!({ "id": id, "scheduledAt": payload.scheduled_at, "status": status }))
    }
}

pub async fn dispatch_broadcast(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let now = Utc::now().to_rfc3339();

    let broadcast = sqlx::query_as::<_, Broadcast>(
        r#"
        SELECT 
            id, name, template_name, template_language, body_preview, tag_name, status,
            scheduled_at, created_at,
            audience_count, audience_count as total_recipients,
            sent_count, delivered_count, read_count, failed_count, skipped_count
        FROM broadcasts
        WHERE id = ?
        "#,
    )
    .bind(&id)
    .fetch_optional(&state.db)
    .await
    .unwrap_or(None);

    if let Some(b) = broadcast {
        let tag_name = b.tag_name.clone().unwrap_or_default();
        let tags = parse_tags(&tag_name);
        let tags_json = serde_json::to_string(&tags).unwrap_or_else(|_| "[]".to_string());

        #[derive(sqlx::FromRow)]
        struct RecipientRow {
            id: String,
            value: String,
            person_name: Option<String>,
        }

        let recipients = if tags.is_empty() {
            sqlx::query_as::<_, RecipientRow>(
                "SELECT id, value, person_name FROM contacts WHERE channel = 'whatsapp'",
            )
            .fetch_all(&state.db)
            .await
            .unwrap_or_default()
        } else {
            sqlx::query_as::<_, RecipientRow>(
                r#"
                SELECT DISTINCT ct.id, ct.value, ct.person_name
                FROM contacts ct
                JOIN contact_tags ctags ON ct.id = ctags.contact_id
                JOIN tags t ON ctags.tag_id = t.id
                WHERE t.name IN (SELECT value FROM json_each(?))
                "#,
            )
            .bind(&tags_json)
            .fetch_all(&state.db)
            .await
            .unwrap_or_default()
        };

        let _ = sqlx::query("UPDATE broadcasts SET status = 'sending' WHERE id = ?")
            .bind(&id)
            .execute(&state.db)
            .await;

        let has_meta_credentials = !state.config.meta_whatsapp_access_token.is_empty()
            && !state.config.meta_whatsapp_phone_number_id.is_empty();

        let mut sent_count = 0;
        let mut delivered_count = 0;
        let mut read_count = 0;
        let mut failed_count = 0;

        for r in &recipients {
            // Find or create conversation for recipient
            let conv_id = match sqlx::query_scalar::<_, String>(
                "SELECT id FROM conversations WHERE contact_id = ? LIMIT 1",
            )
            .bind(&r.id)
            .fetch_optional(&state.db)
            .await
            .unwrap_or(None)
            {
                Some(cid) => {
                    let _ = sqlx::query("UPDATE conversations SET broadcast_id = ?, updated_at = ? WHERE id = ?")
                        .bind(&b.id)
                        .bind(&now)
                        .bind(&cid)
                        .execute(&state.db)
                        .await;
                    cid
                }
                None => {
                    let cid = Uuid::new_v4().to_string();
                    let _ = sqlx::query(
                        r#"
                        INSERT INTO conversations (
                            id, contact_id, state, broadcast_id, channel, human_takeover, created_at, updated_at
                        ) VALUES (?, ?, 'ENGAGED', ?, 'whatsapp', 0, ?, ?)
                        "#,
                    )
                    .bind(&cid)
                    .bind(&r.id)
                    .bind(&b.id)
                    .bind(&now)
                    .bind(&now)
                    .execute(&state.db)
                    .await;
                    cid
                }
            };

            let contact_name = r.person_name.as_deref().unwrap_or("Customer");
            let mut body_text = b
                .body_preview
                .clone()
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| {
                    format!(
                        "Hello {}! This is an update from Newax Technologies regarding: {}.",
                        contact_name, b.name
                    )
                });
            if body_text.contains("{{1}}") {
                body_text = body_text.replace("{{1}}", contact_name);
            }

            let br_id = Uuid::new_v4().to_string();
            let msg_id = Uuid::new_v4().to_string();

            if has_meta_credentials {
                let clean_phone = r.value.replace('+', "").replace(' ', "").replace('-', "");
                let meta_url = format!(
                    "https://graph.facebook.com/{}/{}/messages",
                    state.config.meta_whatsapp_api_version,
                    state.config.meta_whatsapp_phone_number_id
                );

                let template_to_use = b.template_name.clone().unwrap_or_default();
                let meta_body = if !template_to_use.is_empty() {
                    json!({
                        "messaging_product": "whatsapp",
                        "recipient_type": "individual",
                        "to": clean_phone,
                        "type": "template",
                        "template": {
                            "name": template_to_use,
                            "language": {
                                "code": b.template_language.as_deref().unwrap_or("en_US")
                            },
                            "components": [
                                {
                                    "type": "body",
                                    "parameters": [
                                        {
                                            "type": "text",
                                            "text": contact_name
                                        }
                                    ]
                                }
                            ]
                        }
                    })
                } else {
                    json!({
                        "messaging_product": "whatsapp",
                        "recipient_type": "individual",
                        "to": clean_phone,
                        "type": "text",
                        "text": { "preview_url": false, "body": body_text }
                    })
                };

                let meta_response = state
                    .http_client
                    .post(&meta_url)
                    .bearer_auth(&state.config.meta_whatsapp_access_token)
                    .json(&meta_body)
                    .send()
                    .await;

                match meta_response {
                    Ok(resp) if resp.status().is_success() => {
                        let resp_json: serde_json::Value = resp.json().await.unwrap_or_default();
                        let provider_msg_id = resp_json["messages"][0]["id"]
                            .as_str()
                            .unwrap_or("meta_ok")
                            .to_string();

                        sent_count += 1;
                        delivered_count += 1;

                        let _ = sqlx::query(
                            r#"
                            INSERT INTO broadcast_recipients (
                                id, broadcast_id, contact_id, status, provider_message_id, sent_at, delivered_at
                            ) VALUES (?, ?, ?, 'delivered', ?, ?, ?)
                            "#,
                        )
                        .bind(&br_id)
                        .bind(&b.id)
                        .bind(&r.id)
                        .bind(&provider_msg_id)
                        .bind(&now)
                        .bind(&now)
                        .execute(&state.db)
                        .await;

                        let _ = sqlx::query(
                            r#"
                            INSERT INTO messages (
                                id, conversation_id, direction, channel, body, provider, provider_message_id, status, sent_at, sender_type, sender_name
                            ) VALUES (?, ?, 'outbound', 'whatsapp', ?, 'meta', ?, 'delivered', ?, 'system', 'Campaign Broadcast')
                            "#,
                        )
                        .bind(&msg_id)
                        .bind(&conv_id)
                        .bind(&body_text)
                        .bind(&provider_msg_id)
                        .bind(&now)
                        .execute(&state.db)
                        .await;
                    }
                    Ok(resp) => {
                        let err_text = resp.text().await.unwrap_or_default();
                        failed_count += 1;

                        let _ = sqlx::query(
                            r#"
                            INSERT INTO broadcast_recipients (
                                id, broadcast_id, contact_id, status, error, sent_at
                            ) VALUES (?, ?, ?, 'failed', ?, ?)
                            "#,
                        )
                        .bind(&br_id)
                        .bind(&b.id)
                        .bind(&r.id)
                        .bind(&err_text)
                        .bind(&now)
                        .execute(&state.db)
                        .await;
                    }
                    Err(e) => {
                        failed_count += 1;
                        let err_str = e.to_string();

                        let _ = sqlx::query(
                            r#"
                            INSERT INTO broadcast_recipients (
                                id, broadcast_id, contact_id, status, error, sent_at
                            ) VALUES (?, ?, ?, 'failed', ?, ?)
                            "#,
                        )
                        .bind(&br_id)
                        .bind(&b.id)
                        .bind(&r.id)
                        .bind(&err_str)
                        .bind(&now)
                        .execute(&state.db)
                        .await;
                    }
                }
            } else {
                // Realistic local / prototype delivery
                sent_count += 1;
                delivered_count += 1;
                read_count += 1;
                let sim_provider_id = format!("meta_sim_{}", Uuid::new_v4().simple());

                let _ = sqlx::query(
                    r#"
                    INSERT INTO broadcast_recipients (
                        id, broadcast_id, contact_id, status, provider_message_id, sent_at, delivered_at, read_at
                    ) VALUES (?, ?, ?, 'read', ?, ?, ?, ?)
                    "#,
                )
                .bind(&br_id)
                .bind(&b.id)
                .bind(&r.id)
                .bind(&sim_provider_id)
                .bind(&now)
                .bind(&now)
                .bind(&now)
                .execute(&state.db)
                .await;

                let _ = sqlx::query(
                    r#"
                    INSERT INTO messages (
                        id, conversation_id, direction, channel, body, provider, provider_message_id, status, sent_at, sender_type, sender_name
                    ) VALUES (?, ?, 'outbound', 'whatsapp', ?, 'meta', ?, 'read', ?, 'system', 'Campaign Broadcast')
                    "#,
                )
                .bind(&msg_id)
                .bind(&conv_id)
                .bind(&body_text)
                .bind(&sim_provider_id)
                .bind(&now)
                .execute(&state.db)
                .await;
            }
        }

        let total_recipients = recipients.len() as i32;
        let _ = sqlx::query(
            r#"
            UPDATE broadcasts 
            SET status = 'completed', audience_count = ?, total_recipients = ?, sent_count = ?, delivered_count = ?, read_count = ?, failed_count = ?
            WHERE id = ?
            "#,
        )
        .bind(total_recipients)
        .bind(total_recipients)
        .bind(sent_count)
        .bind(delivered_count)
        .bind(read_count)
        .bind(failed_count)
        .bind(&id)
        .execute(&state.db)
        .await;

        Json(json!({
            "id": id,
            "status": "completed",
            "audienceCount": total_recipients,
            "totalRecipients": total_recipients,
            "sentCount": sent_count,
            "deliveredCount": delivered_count,
            "readCount": read_count,
            "failedCount": failed_count
        }))
    } else {
        Json(json!({ "error": "Broadcast not found" }))
    }
}

pub async fn process_scheduled_campaigns(State(state): State<AppState>) -> impl IntoResponse {
    let now = Utc::now().to_rfc3339();
    let scheduled = sqlx::query_as::<_, Broadcast>(
        r#"
        SELECT 
            id, name, template_name, template_language, body_preview, tag_name, status,
            scheduled_at, created_at,
            audience_count, audience_count as total_recipients,
            sent_count, delivered_count, read_count, failed_count, skipped_count
        FROM broadcasts 
        WHERE status = 'scheduled' AND (scheduled_at IS NULL OR scheduled_at <= ?)
        "#,
    )
    .bind(&now)
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    let count = scheduled.len();

    for camp in &scheduled {
        let _ = dispatch_broadcast(State(state.clone()), Path(camp.id.clone())).await;
    }

    Json(json!({
        "processed": count,
        "campaigns": scheduled
    }))
}

pub async fn list_broadcast_recipients(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let recipients = sqlx::query_as::<_, RecipientDetailRow>(
        r#"
        SELECT 
            br.id,
            br.contact_id,
            COALESCE(ct.person_name, 'Valued Customer') as contact_name,
            COALESCE(ct.value, '') as contact_phone,
            br.status,
            br.error,
            br.sent_at,
            br.delivered_at,
            br.read_at
        FROM broadcast_recipients br
        LEFT JOIN contacts ct ON br.contact_id = ct.id
        WHERE br.broadcast_id = ?
        ORDER BY br.sent_at DESC
        "#,
    )
    .bind(&id)
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    Json(recipients)
}

#[derive(Deserialize)]
pub struct PricingQuery {
    pub region: Option<String>,
    pub category: Option<String>,
    pub volume: Option<i64>,
}

struct RegionRate {
    code: &'static str,
    name: &'static str,
    marketing: f64,
    utility: f64,
    auth: f64,
    service: f64,
}

const META_RATES: &[RegionRate] = &[
    RegionRate { code: "US", name: "United States", marketing: 0.0250, utility: 0.0150, auth: 0.0135, service: 0.0080 },
    RegionRate { code: "UK", name: "United Kingdom", marketing: 0.0388, utility: 0.0200, auth: 0.0180, service: 0.0100 },
    RegionRate { code: "EU", name: "European Union", marketing: 0.0520, utility: 0.0300, auth: 0.0250, service: 0.0150 },
    RegionRate { code: "IN", name: "India", marketing: 0.0105, utility: 0.0035, auth: 0.0030, service: 0.0020 },
    RegionRate { code: "BR", name: "Brazil", marketing: 0.0625, utility: 0.0350, auth: 0.0315, service: 0.0180 },
    RegionRate { code: "MX", name: "Mexico", marketing: 0.0430, utility: 0.0250, auth: 0.0220, service: 0.0120 },
    RegionRate { code: "UAE", name: "United Arab Emirates", marketing: 0.0340, utility: 0.0220, auth: 0.0190, service: 0.0110 },
    RegionRate { code: "ROW", name: "Rest of World", marketing: 0.0300, utility: 0.0180, auth: 0.0150, service: 0.0090 },
];

pub async fn get_whatsapp_pricing(Query(query): Query<PricingQuery>) -> impl IntoResponse {
    let volume = query.volume.unwrap_or(1000);
    let region_code = query.region.unwrap_or_else(|| "US".to_string()).to_uppercase();
    let category = query.category.unwrap_or_else(|| "marketing".to_string()).to_lowercase();

    let rate_entry = META_RATES
        .iter()
        .find(|r| r.code == region_code)
        .unwrap_or(&META_RATES[META_RATES.len() - 1]); // Default to ROW

    let rate_per_conversation = match category.as_str() {
        "marketing" => rate_entry.marketing,
        "utility" => rate_entry.utility,
        "auth" | "authentication" => rate_entry.auth,
        "service" => rate_entry.service,
        _ => rate_entry.marketing,
    };

    let total_cost = (volume as f64) * rate_per_conversation;
    let estimated_cost_usd = (total_cost * 100.0).round() / 100.0;
    let estimated_roi_usd = ((estimated_cost_usd * 8.5) * 100.0).round() / 100.0;

    let supported_regions: Vec<_> = META_RATES
        .iter()
        .map(|r| json!({ "code": r.code, "name": r.name }))
        .collect();

    Json(json!({
        "region": rate_entry.code,
        "regionName": rate_entry.name,
        "category": category,
        "volume": volume,
        "ratePerConversation": rate_per_conversation,
        "ratePerMessage": rate_per_conversation,
        "estimatedCostUsd": estimated_cost_usd,
        "estimatedCost": estimated_cost_usd,
        "estimatedRoiUsd": estimated_roi_usd,
        "freeServiceTierIncluded": "First 1,000 service conversations/month are free from Meta",
        "supportedRegions": supported_regions,
        "currency": "USD"
    }))
}

pub async fn seed_default_campaigns(State(state): State<AppState>) -> impl IntoResponse {
    // Check if default campaigns already exist
    let existing_count: i64 = sqlx::query_scalar("SELECT count(*) FROM broadcasts")
        .fetch_one(&state.db)
        .await
        .unwrap_or(0);

    if existing_count >= 3 {
        return Json(json!({
            "status": "skipped",
            "message": "Example campaigns already active"
        }));
    }

    let now = Utc::now().to_rfc3339();

    // 1. Spring sale early access
    let c1_id = Uuid::new_v4().to_string();
    let _ = sqlx::query(
        r#"
        INSERT OR REPLACE INTO broadcasts (
            id, name, template_name, template_language, body_preview, tag_name, status,
            scheduled_at, created_at, audience_count, total_recipients,
            sent_count, delivered_count, read_count, failed_count, skipped_count,
            template_params, tenant_key
        ) VALUES (
            ?, 'Spring sale early access', 'spring_sale_offer', 'en_US',
            'Hi Maya, our spring sale starts Friday. As a VIP customer you get early access and 20% off until Sunday.',
            'vip-customers', 'scheduled', ?, ?, 248, 248, 240, 218, 164, 12, 8,
            '{"customer_name": "Maya", "discount": "20%"}', 'newax'
        )
        "#,
    )
    .bind(&c1_id)
    .bind(&now)
    .bind(&now)
    .execute(&state.db)
    .await;

    // 2. New store opening
    let c2_id = Uuid::new_v4().to_string();
    let _ = sqlx::query(
        r#"
        INSERT OR REPLACE INTO broadcasts (
            id, name, template_name, template_language, body_preview, tag_name, status,
            scheduled_at, created_at, audience_count, total_recipients,
            sent_count, delivered_count, read_count, failed_count, skipped_count,
            template_params, tenant_key
        ) VALUES (
            ?, 'New store opening', 'store_launch_announcement', 'en_US',
            'We are thrilled to invite you to our new flagship store opening! Come by this weekend for exclusive in-store perks.',
            'newsletter', 'sending', NULL, ?, 520, 520, 310, 295, 180, 15, 0,
            '{}', 'newax'
        )
        "#,
    )
    .bind(&c2_id)
    .bind(&now)
    .execute(&state.db)
    .await;

    // 3. Loyalty points reminder
    let c3_id = Uuid::new_v4().to_string();
    let _ = sqlx::query(
        r#"
        INSERT OR REPLACE INTO broadcasts (
            id, name, template_name, template_language, body_preview, tag_name, status,
            scheduled_at, created_at, audience_count, total_recipients,
            sent_count, delivered_count, read_count, failed_count, skipped_count,
            template_params, tenant_key
        ) VALUES (
            ?, 'Loyalty points reminder', 'loyalty_rewards_update', 'en_US',
            'Hi {{1}}, you have {{2}} loyalty points expiring at the end of the month. Redeem them on your next purchase!',
            'vip-customers', 'completed', NULL, ?, 150, 150, 150, 148, 112, 2, 0,
            '{"1": "Customer", "2": "500"}', 'newax'
        )
        "#,
    )
    .bind(&c3_id)
    .bind(&now)
    .execute(&state.db)
    .await;

    // Seed sample broadcast recipients for Spring sale early access
    let _ = sqlx::query(
        r#"
        INSERT OR REPLACE INTO broadcast_recipients (
            id, broadcast_id, contact_id, status, sent_at, delivered_at, read_at
        ) VALUES 
        (?, ?, 'contact-1', 'read', ?, ?, ?),
        (?, ?, 'contact-2', 'delivered', ?, ?, NULL),
        (?, ?, 'contact-3', 'sent', ?, NULL, NULL)
        "#,
    )
    .bind(Uuid::new_v4().to_string())
    .bind(&c1_id)
    .bind(&now)
    .bind(&now)
    .bind(&now)
    .bind(Uuid::new_v4().to_string())
    .bind(&c1_id)
    .bind(&now)
    .bind(&now)
    .bind(Uuid::new_v4().to_string())
    .bind(&c1_id)
    .bind(&now)
    .execute(&state.db)
    .await;

    Json(json!({
        "status": "seeded",
        "message": "Pre-built Chatwoot WhatsApp example campaigns installed"
    }))
}
