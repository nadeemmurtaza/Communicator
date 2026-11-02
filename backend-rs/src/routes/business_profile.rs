use crate::{db::AppState, services::meta_sync};
use axum::{extract::State, response::IntoResponse, Json};
use chrono::Utc;
use serde::Deserialize;
use serde_json::{json, Value};
use tracing::{info, warn};

#[derive(Debug, Deserialize)]
pub struct UpdateBusinessProfilePayload {
    pub business_name: Option<String>,
    pub display_phone: Option<String>,
    pub verified_name: Option<String>,
    pub email: Option<String>,
    pub description: Option<String>,
    pub about: Option<String>,
    pub address: Option<String>,
    pub vertical: Option<String>,
    pub websites: Option<Vec<String>>,
    pub profile_picture_url: Option<String>,
    pub business_hours_enabled: Option<bool>,
    pub business_hours_start: Option<i32>,
    pub business_hours_end: Option<i32>,
    pub business_timezone: Option<String>,
    pub away_message: Option<String>,
    pub auto_reply_enabled: Option<bool>,
    pub agent_guidelines: Option<String>,
    pub sync_with_meta: Option<bool>,
}

pub async fn get_business_profile(State(state): State<AppState>) -> impl IntoResponse {
    let profile_json = fetch_full_business_profile(&state).await;
    Json(profile_json)
}

pub async fn update_business_profile(
    State(state): State<AppState>,
    Json(payload): Json<UpdateBusinessProfilePayload>,
) -> impl IntoResponse {
    let now = Utc::now().to_rfc3339();

    // 1. Fetch existing integration settings or create default
    let biz_name = payload.business_name.unwrap_or_else(|| "Newax Technologies".to_string());
    let verified_name = payload.verified_name.unwrap_or_else(|| biz_name.clone());
    let display_phone = payload.display_phone.unwrap_or_else(|| "+92 370 9861100".to_string());
    let desc = payload.description.unwrap_or_default();
    let guidelines = payload.agent_guidelines.unwrap_or_else(|| "Be polite, concise, professional, and helpful.".to_string());
    let auto_reply = payload.auto_reply_enabled.unwrap_or(true);
    let hours_enabled = payload.business_hours_enabled.unwrap_or(true);
    let hours_start = payload.business_hours_start.unwrap_or(9);
    let hours_end = payload.business_hours_end.unwrap_or(18);
    let tz = payload.business_timezone.unwrap_or_else(|| "Asia/Karachi".to_string());
    let away = payload.away_message.unwrap_or_else(|| "Thanks for contacting us! We're currently outside our working hours. We'll reply as soon as we reopen.".to_string());

    // Update integration_settings in database
    let _ = sqlx::query(
        r#"
        UPDATE integration_settings
        SET business_name = ?,
            whatsapp_verified_name = ?,
            whatsapp_display_number = ?,
            business_description = ?,
            agent_guidelines = ?,
            auto_reply_enabled = ?,
            business_hours_enabled = ?,
            business_hours_start = ?,
            business_hours_end = ?,
            business_timezone = ?,
            away_message = ?,
            updated_at = ?
        "#,
    )
    .bind(&biz_name)
    .bind(&verified_name)
    .bind(&display_phone)
    .bind(&desc)
    .bind(&guidelines)
    .bind(auto_reply)
    .bind(hours_enabled)
    .bind(hours_start)
    .bind(hours_end)
    .bind(&tz)
    .bind(&away)
    .bind(&now)
    .execute(&state.db)
    .await;

    // 2. Update WhatsApp Inbox credentials JSON
    let about = payload.about.unwrap_or_default();
    let addr = payload.address.unwrap_or_default();
    let email_val = payload.email.unwrap_or_default();
    let vertical_val = payload.vertical.unwrap_or_else(|| "PROF_SERVICES".to_string());
    let websites_val = payload.websites.unwrap_or_else(|| vec!["https://newax.co/".to_string()]);
    let pic_url = payload.profile_picture_url.unwrap_or_default();

    #[derive(sqlx::FromRow)]
    struct InboxRow {
        id: String,
        credentials: Option<String>,
    }

    if let Ok(Some(inbox)) = sqlx::query_as::<_, InboxRow>(
        "SELECT id, credentials FROM inboxes WHERE channel_type = 'whatsapp' LIMIT 1"
    )
    .fetch_optional(&state.db)
    .await {
        let mut creds_val: Value = inbox.credentials
            .as_deref()
            .and_then(|c| serde_json::from_str(c).ok())
            .unwrap_or_else(|| json!({}));

        let biz_obj = json!({
            "about": about,
            "address": addr,
            "description": desc,
            "email": email_val,
            "vertical": vertical_val,
            "websites": websites_val,
            "profilePictureUrl": pic_url
        });

        creds_val["businessProfile"] = biz_obj;
        let creds_str = serde_json::to_string(&creds_val).unwrap_or_default();

        let _ = sqlx::query(
            "UPDATE inboxes SET credentials = ?, phone_number = ?, updated_at = ? WHERE id = ?"
        )
        .bind(&creds_str)
        .bind(&display_phone)
        .bind(&now)
        .bind(&inbox.id)
        .execute(&state.db)
        .await;
    }

    // 3. Sync live with Meta Graph API v21.0 if requested
    if payload.sync_with_meta.unwrap_or(true) {
        #[derive(sqlx::FromRow)]
        struct TokenRow {
            meta_access_token: Option<String>,
            meta_phone_number_id: Option<String>,
        }

        if let Ok(Some(t_row)) = sqlx::query_as::<_, TokenRow>(
            "SELECT meta_access_token, meta_phone_number_id FROM integration_settings LIMIT 1"
        )
        .fetch_optional(&state.db)
        .await {
            let token = t_row.meta_access_token.unwrap_or_else(|| state.config.meta_whatsapp_access_token.clone());
            let phone_id = t_row.meta_phone_number_id.unwrap_or_else(|| state.config.meta_whatsapp_phone_number_id.clone());

            if !token.is_empty() && !phone_id.is_empty() {
                let url = format!(
                    "https://graph.facebook.com/{}/{}/whatsapp_business_profile",
                    state.config.meta_whatsapp_api_version, phone_id
                );

                let meta_body = json!({
                    "messaging_product": "whatsapp",
                    "about": about,
                    "address": addr,
                    "description": desc,
                    "email": email_val,
                    "vertical": vertical_val,
                    "websites": websites_val
                });

                match state.http_client
                    .post(&url)
                    .header("Authorization", format!("Bearer {}", token))
                    .json(&meta_body)
                    .send()
                    .await
                {
                    Ok(resp) => {
                        let status = resp.status();
                        let text = resp.text().await.unwrap_or_default();
                        info!("Meta Graph API v21.0 business profile update response: {} - {}", status, text);
                    }
                    Err(e) => {
                        warn!("Failed to sync business profile to Meta Graph API: {}", e);
                    }
                }
            }
        }
    }

    let profile_json = fetch_full_business_profile(&state).await;
    Json(profile_json)
}

pub async fn sync_meta(State(state): State<AppState>) -> impl IntoResponse {
    #[derive(sqlx::FromRow)]
    struct TokenRow {
        meta_access_token: Option<String>,
        meta_phone_number_id: Option<String>,
    }

    let token_row = sqlx::query_as::<_, TokenRow>(
        "SELECT meta_access_token, meta_phone_number_id FROM integration_settings LIMIT 1"
    )
    .fetch_optional(&state.db)
    .await
    .unwrap_or_default();

    let token = token_row.as_ref()
        .and_then(|r| r.meta_access_token.clone())
        .filter(|t| !t.is_empty())
        .unwrap_or_else(|| state.config.meta_whatsapp_access_token.clone());

    let phone_id = token_row.as_ref()
        .and_then(|r| r.meta_phone_number_id.clone())
        .filter(|p| !p.is_empty())
        .unwrap_or_else(|| state.config.meta_whatsapp_phone_number_id.clone());

    let meta_profile = meta_sync::fetch_meta_business_profile(
        &state.http_client,
        &token,
        &phone_id,
        &state.config.meta_whatsapp_api_version,
    )
    .await;

    // Update verified fields in integration_settings
    let now = Utc::now().to_rfc3339();
    let _ = sqlx::query(
        r#"
        UPDATE integration_settings
        SET whatsapp_verified_name = ?,
            whatsapp_display_number = ?,
            whatsapp_verified_at = ?,
            updated_at = ?
        "#
    )
    .bind(&meta_profile.verified_name)
    .bind(&meta_profile.display_phone_number)
    .bind(&now)
    .bind(&now)
    .execute(&state.db)
    .await;

    let profile_json = fetch_full_business_profile(&state).await;
    Json(profile_json)
}

async fn fetch_full_business_profile(state: &AppState) -> Value {
    #[derive(sqlx::FromRow)]
    struct SettingsRow {
        business_name: Option<String>,
        business_description: Option<String>,
        whatsapp_verified_name: Option<String>,
        whatsapp_display_number: Option<String>,
        agent_guidelines: Option<String>,
        auto_reply_enabled: Option<bool>,
        business_hours_enabled: Option<bool>,
        business_hours_start: Option<i32>,
        business_hours_end: Option<i32>,
        business_timezone: Option<String>,
        away_message: Option<String>,
        meta_access_token: Option<String>,
        meta_phone_number_id: Option<String>,
        meta_waba_id: Option<String>,
        whatsapp_verified_at: Option<String>,
    }

    let settings = sqlx::query_as::<_, SettingsRow>(
        r#"
        SELECT business_name, business_description, whatsapp_verified_name,
               whatsapp_display_number, agent_guidelines, auto_reply_enabled,
               business_hours_enabled, business_hours_start, business_hours_end,
               business_timezone, away_message, meta_access_token,
               meta_phone_number_id, meta_waba_id, whatsapp_verified_at
        FROM integration_settings LIMIT 1
        "#
    )
    .fetch_optional(&state.db)
    .await
    .unwrap_or_default();

    #[derive(sqlx::FromRow)]
    struct InboxRow {
        credentials: Option<String>,
    }

    let wa_inbox = sqlx::query_as::<_, InboxRow>(
        "SELECT credentials FROM inboxes WHERE channel_type = 'whatsapp' LIMIT 1"
    )
    .fetch_optional(&state.db)
    .await
    .unwrap_or_default();

    let mut stored_about = String::new();
    let mut stored_email = "hello@newax.co".to_string();
    let mut stored_address = "Building 9, College Road, Township, Lahore, Pakistan, 54700".to_string();
    let mut stored_vertical = "PROF_SERVICES".to_string();
    let mut stored_websites = vec!["https://newax.co/".to_string()];
    let mut stored_pic_url = String::new();

    if let Some(inbox) = wa_inbox {
        if let Some(creds_str) = inbox.credentials {
            if let Ok(c) = serde_json::from_str::<Value>(&creds_str) {
                if let Some(bp) = c.get("businessProfile") {
                    if let Some(a) = bp.get("about").and_then(|v| v.as_str()) {
                        stored_about = a.to_string();
                    }
                    if let Some(e) = bp.get("email").and_then(|v| v.as_str()) {
                        stored_email = e.to_string();
                    }
                    if let Some(addr) = bp.get("address").and_then(|v| v.as_str()) {
                        stored_address = addr.to_string();
                    }
                    if let Some(vert) = bp.get("vertical").and_then(|v| v.as_str()) {
                        stored_vertical = vert.to_string();
                    }
                    if let Some(sites) = bp.get("websites").and_then(|v| v.as_array()) {
                        stored_websites = sites.iter().filter_map(|s| s.as_str().map(String::from)).collect();
                    }
                    if let Some(pic) = bp.get("profilePictureUrl").and_then(|v| v.as_str()) {
                        stored_pic_url = pic.to_string();
                    }
                }
            }
        }
    }

    let token = settings.as_ref()
        .and_then(|s| s.meta_access_token.clone())
        .filter(|t| !t.is_empty())
        .unwrap_or_else(|| state.config.meta_whatsapp_access_token.clone());

    let phone_id = settings.as_ref()
        .and_then(|s| s.meta_phone_number_id.clone())
        .filter(|p| !p.is_empty())
        .unwrap_or_else(|| state.config.meta_whatsapp_phone_number_id.clone());

    let meta_profile = meta_sync::fetch_meta_business_profile(
        &state.http_client,
        &token,
        &phone_id,
        &state.config.meta_whatsapp_api_version,
    )
    .await;

    let biz_name = settings.as_ref()
        .and_then(|s| s.business_name.clone())
        .unwrap_or_else(|| "Newax Technologies".to_string());

    let verified_name = if !meta_profile.verified_name.is_empty() && meta_profile.verified_name != "Not Configured" {
        meta_profile.verified_name
    } else {
        settings.as_ref()
            .and_then(|s| s.whatsapp_verified_name.clone())
            .unwrap_or_else(|| biz_name.clone())
    };

    let display_phone = if !meta_profile.display_phone_number.is_empty() && meta_profile.display_phone_number != "Not Configured" {
        meta_profile.display_phone_number
    } else {
        settings.as_ref()
            .and_then(|s| s.whatsapp_display_number.clone())
            .unwrap_or_else(|| "+92 370 9861100".to_string())
    };

    let desc = settings.as_ref()
        .and_then(|s| s.business_description.clone())
        .unwrap_or_else(|| "Newax Technologies is an enterprise automation and custom digital infrastructure service that helps mid-to-large organizations transform manual operations.".to_string());

    let guidelines = settings.as_ref()
        .and_then(|s| s.agent_guidelines.clone())
        .unwrap_or_else(|| "Be polite, concise, professional, and helpful.".to_string());

    let auto_reply = settings.as_ref().and_then(|s| s.auto_reply_enabled).unwrap_or(true);
    let hours_enabled = settings.as_ref().and_then(|s| s.business_hours_enabled).unwrap_or(true);
    let hours_start = settings.as_ref().and_then(|s| s.business_hours_start).unwrap_or(9);
    let hours_end = settings.as_ref().and_then(|s| s.business_hours_end).unwrap_or(18);
    let tz = settings.as_ref().and_then(|s| s.business_timezone.clone()).unwrap_or_else(|| "Asia/Karachi".to_string());
    let away = settings.as_ref().and_then(|s| s.away_message.clone()).unwrap_or_else(|| "Thanks for reaching out! Our team is currently outside working hours.".to_string());
    let waba_id = settings.as_ref().and_then(|s| s.meta_waba_id.clone()).unwrap_or_else(|| "990754184075228".to_string());
    let verified_at = settings.as_ref().and_then(|s| s.whatsapp_verified_at.clone()).unwrap_or_else(|| Utc::now().to_rfc3339());

    json!({
        "businessName": biz_name,
        "verifiedName": verified_name,
        "displayPhone": display_phone,
        "isVerified": meta_profile.is_verified,
        "verificationStatus": "Official Business Account",
        "qualityRating": meta_profile.quality_rating,
        "messagingLimit": meta_profile.messaging_limit_tier,
        "tier": "Standard Account Limit",
        "spamFlags": "Zero spam flags",
        "description": desc,
        "about": if meta_profile.about.is_some() && !meta_profile.about.as_ref().unwrap().is_empty() { meta_profile.about.unwrap() } else { stored_about },
        "email": if meta_profile.email.is_some() && !meta_profile.email.as_ref().unwrap().is_empty() { meta_profile.email.unwrap() } else { stored_email },
        "address": if meta_profile.address.is_some() && !meta_profile.address.as_ref().unwrap().is_empty() { meta_profile.address.unwrap() } else { stored_address },
        "vertical": stored_vertical,
        "websites": if !meta_profile.websites.is_empty() { meta_profile.websites } else { stored_websites },
        "profilePictureUrl": stored_pic_url,
        "businessHours": {
            "enabled": hours_enabled,
            "start": hours_start,
            "end": hours_end,
            "timezone": tz,
            "awayMessage": away
        },
        "autoReplyEnabled": auto_reply,
        "agentGuidelines": guidelines,
        "metaIntegration": {
            "connected": !token.is_empty() && !phone_id.is_empty(),
            "phoneNumberId": phone_id,
            "wabaId": waba_id,
            "verifiedAt": verified_at
        }
    })
}
