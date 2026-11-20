//! Business Profile Service
//!
//! Manages WhatsApp Business profile fields stored across `integration_settings`
//! and the WhatsApp inbox credentials JSON, with optional live sync to Meta Graph API.

use sqlx::postgres::PgPool;
use serde::Deserialize;
use serde_json::{json, Map, Value};
use chrono::Utc;
use tracing::{info, warn};

// ---------------------------------------------------------------------------
// State
// ---------------------------------------------------------------------------

#[derive(Clone)]
pub struct BusinessProfileState {
    pub db: PgPool,
    pub http_client: reqwest::Client,
    pub meta_whatsapp_access_token: String,
    pub meta_whatsapp_phone_number_id: String,
    pub meta_whatsapp_api_version: String,
}

// ---------------------------------------------------------------------------
// Meta Graph API types (inlined from meta_sync to avoid in-process coupling)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
struct BusinessProfile {
    pub is_connected: bool,
    pub error: Option<String>,
    pub display_phone_number: String,
    pub verified_name: String,
    pub quality_rating: String,
    pub messaging_limit_tier: String,
    pub is_verified: bool,
    pub zero_spam_flags: Option<bool>,
    pub account_mode: String,
    pub about: Option<String>,
    pub address: Option<String>,
    pub description: Option<String>,
    pub email: Option<String>,
    pub profile_picture_url: Option<String>,
    pub vertical: Option<String>,
    pub websites: Vec<String>,
}

fn unknown_profile(error: Option<String>, is_connected: bool) -> BusinessProfile {
    BusinessProfile {
        is_connected,
        error,
        display_phone_number: String::new(),
        verified_name: String::new(),
        quality_rating: String::new(),
        messaging_limit_tier: String::new(),
        is_verified: false,
        zero_spam_flags: None,
        account_mode: String::new(),
        about: None,
        address: None,
        description: None,
        email: None,
        profile_picture_url: None,
        vertical: None,
        websites: vec![],
    }
}

fn messaging_limit_label(raw_tier: Option<&str>) -> String {
    match raw_tier.unwrap_or("").trim().to_ascii_uppercase().as_str() {
        "" => String::new(),
        "TIER_UNLIMITED" | "UNLIMITED" => "Unlimited / day".to_string(),
        "TIER_100K" | "100K" => "100,000 / day".to_string(),
        "TIER_10K" | "10K" => "10,000 / day".to_string(),
        "TIER_1K" | "1K" => "1,000 / day".to_string(),
        "TIER_250" | "250" => "250 / day".to_string(),
        other => format!("{other} / day"),
    }
}

fn quality_rating_label(raw_rating: Option<&str>) -> String {
    match raw_rating.unwrap_or("").trim().to_ascii_uppercase().as_str() {
        "GREEN" => "HIGH (Green)".to_string(),
        "YELLOW" => "MEDIUM (Yellow)".to_string(),
        "RED" => "LOW (Red)".to_string(),
        other => other.to_string(),
    }
}

fn meta_error(body: &Value, status: u16) -> String {
    let err = body.get("error");
    let message = err
        .and_then(|e| e.get("message"))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let code = err.and_then(|e| e.get("code")).and_then(|v| v.as_i64());
    match (message.is_empty(), code) {
        (false, Some(code)) => format!("Meta API error {code}: {message}"),
        (false, _) => message,
        (true, Some(code)) => format!("Meta API error {code} (HTTP {status})"),
        (true, _) => format!("Meta API returned HTTP {status}"),
    }
}

async fn graph_get(
    state: &BusinessProfileState,
    url: &str,
    token: &str,
    fields: &str,
) -> Result<Value, String> {
    let response = state
        .http_client
        .get(url)
        .header("Authorization", format!("Bearer {token}"))
        .query(&[("fields", fields)])
        .send()
        .await
        .map_err(|err| format!("Could not reach graph.facebook.com: {err}"))?;

    let status = response.status();
    let body: Value = response
        .json()
        .await
        .map_err(|err| format!("Meta returned a body we could not parse: {err}"))?;

    if !status.is_success() {
        return Err(meta_error(&body, status.as_u16()));
    }
    Ok(body)
}

pub async fn fetch_meta_business_profile(
    state: &BusinessProfileState,
    access_token: &str,
    phone_number_id: &str,
    api_version: &str,
) -> BusinessProfile {
    if access_token.trim().is_empty() || phone_number_id.trim().is_empty() {
        return unknown_profile(
            Some("No Meta access token or phone number id configured.".to_string()),
            false,
        );
    }
    if access_token.contains('•') {
        return unknown_profile(
            Some("Stored Meta token is masked; reconnect the WhatsApp inbox.".to_string()),
            false,
        );
    }

    let base = format!("https://graph.facebook.com/{api_version}/{phone_number_id}");
    let phone = match graph_get(
        state,
        &base,
        access_token,
        "display_phone_number,verified_name,quality_rating,messaging_limit_tier,\
         code_verification_status,account_mode,analytics",
    )
    .await
    {
        Ok(value) => value,
        Err(error) => return unknown_profile(Some(error), false),
    };

    let raw_rating = phone.get("quality_rating").and_then(|v| v.as_str());
    let profile_url = format!("{base}/whatsapp_business_profile");
    let mut profile_fields = unknown_profile(None, true);
    match graph_get(
        state,
        &profile_url,
        access_token,
        "about,address,description,email,profile_picture_url,websites,vertical",
    )
    .await
    {
        Ok(body) => {
            if let Some(entry) = body.get("data").and_then(|v| v.as_array()).and_then(|a| a.first())
            {
                profile_fields.about =
                    entry.get("about").and_then(|v| v.as_str()).map(str::to_string);
                profile_fields.address =
                    entry.get("address").and_then(|v| v.as_str()).map(str::to_string);
                profile_fields.description =
                    entry.get("description").and_then(|v| v.as_str()).map(str::to_string);
                profile_fields.email =
                    entry.get("email").and_then(|v| v.as_str()).map(str::to_string);
                profile_fields.profile_picture_url = entry
                    .get("profile_picture_url")
                    .and_then(|v| v.as_str())
                    .map(str::to_string);
                profile_fields.vertical =
                    entry.get("vertical").and_then(|v| v.as_str()).map(str::to_string);
                profile_fields.websites = entry
                    .get("websites")
                    .and_then(|v| v.as_array())
                    .map(|sites| {
                        sites.iter().filter_map(|w| w.as_str().map(str::to_string)).collect()
                    })
                    .unwrap_or_default();
            }
        }
        Err(error) => profile_fields.error = Some(error),
    }

    BusinessProfile {
        is_connected: true,
        error: profile_fields.error,
        display_phone_number: phone
            .get("display_phone_number")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string(),
        verified_name: phone
            .get("verified_name")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string(),
        quality_rating: quality_rating_label(raw_rating),
        messaging_limit_tier: messaging_limit_label(
            phone.get("messaging_limit_tier").and_then(|v| v.as_str()),
        ),
        is_verified: !phone
            .get("verified_name")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .is_empty(),
        zero_spam_flags: raw_rating.map(|rating| {
            matches!(rating.trim().to_ascii_uppercase().as_str(), "GREEN")
        }),
        account_mode: phone
            .get("account_mode")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string(),
        about: profile_fields.about,
        address: profile_fields.address,
        description: profile_fields.description,
        email: profile_fields.email,
        profile_picture_url: profile_fields.profile_picture_url,
        vertical: profile_fields.vertical,
        websites: profile_fields.websites,
    }
}

// ---------------------------------------------------------------------------
// Persistence helper
// ---------------------------------------------------------------------------

fn persistence_json(
    ok: bool,
    rows_affected: Option<u64>,
    reason: &str,
    detail: Option<String>,
) -> Value {
    json!({
        "ok": ok,
        "saved": ok,
        "rowsAffected": rows_affected.map(|r| json!(r)).unwrap_or(Value::Null),
        "reason": reason,
        "detail": detail.map(|d| json!(d)).unwrap_or(Value::Null),
    })
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

pub async fn get_business_profile(
    state: &BusinessProfileState,
) -> Result<Value, String> {
    fetch_full_business_profile(state).await
}

#[derive(Debug, Deserialize)]
struct UpdateBusinessProfilePayload {
    business_name: Option<String>,
    display_phone: Option<String>,
    verified_name: Option<String>,
    email: Option<String>,
    description: Option<String>,
    about: Option<String>,
    address: Option<String>,
    vertical: Option<String>,
    websites: Option<Vec<String>>,
    profile_picture_url: Option<String>,
    business_hours_enabled: Option<bool>,
    business_hours_start: Option<i32>,
    business_hours_end: Option<i32>,
    business_timezone: Option<String>,
    away_message: Option<String>,
    auto_reply_enabled: Option<bool>,
    agent_guidelines: Option<String>,
    sync_with_meta: Option<bool>,
}

pub async fn update_business_profile(
    state: &BusinessProfileState,
    body: &Value,
) -> Result<Value, String> {
    let payload: UpdateBusinessProfilePayload =
        serde_json::from_value(body.clone())
            .map_err(|e| format!("invalid payload: {e}"))?;

    let now = Utc::now().to_rfc3339();
    let UpdateBusinessProfilePayload {
        business_name,
        verified_name,
        display_phone,
        email,
        description,
        about,
        address,
        vertical,
        websites,
        profile_picture_url,
        business_hours_enabled,
        business_hours_start,
        business_hours_end,
        business_timezone,
        away_message,
        auto_reply_enabled,
        agent_guidelines,
        sync_with_meta,
    } = payload;

    // 1. integration_settings
    let settings_write = sqlx::query(
        r#"
        UPDATE integration_settings
        SET business_name = COALESCE($1, business_name),
            whatsapp_verified_name = COALESCE($2, whatsapp_verified_name),
            whatsapp_display_number = COALESCE($3, whatsapp_display_number),
            business_description = COALESCE($4, business_description),
            agent_guidelines = COALESCE($5, agent_guidelines),
            auto_reply_enabled = COALESCE($6, auto_reply_enabled),
            business_hours_enabled = COALESCE($7, business_hours_enabled),
            business_hours_start = COALESCE($8, business_hours_start),
            business_hours_end = COALESCE($9, business_hours_end),
            business_timezone = COALESCE($10, business_timezone),
            away_message = COALESCE($11, away_message),
            updated_at = $12
        "#,
    )
    .bind(&business_name)
    .bind(&verified_name)
    .bind(&display_phone)
    .bind(&description)
    .bind(&agent_guidelines)
    .bind(auto_reply_enabled)
    .bind(business_hours_enabled)
    .bind(business_hours_start)
    .bind(business_hours_end)
    .bind(&business_timezone)
    .bind(&away_message)
    .bind(&now)
    .execute(&state.db)
    .await;

    let settings_persistence = match &settings_write {
        Ok(report) if report.rows_affected() == 0 => {
            return Err("UPDATE integration_settings affected 0 rows: no integration_settings row exists".to_string());
        }
        Ok(report) => persistence_json(
            true,
            Some(report.rows_affected()),
            "integration_settings business fields updated",
            None,
        ),
        Err(err) => {
            return Err(format!("the business profile could not be written to integration_settings: {err}"));
        }
    };

    // 2. WhatsApp inbox credentials JSON
    #[derive(sqlx::FromRow)]
    struct InboxRow {
        id: String,
        credentials: Option<String>,
    }

    let inbox = match sqlx::query_as::<_, InboxRow>(
        "SELECT id, credentials FROM inboxes WHERE channel_type = 'whatsapp' LIMIT 1",
    )
    .fetch_optional(&state.db)
    .await
    {
        Ok(row) => row,
        Err(err) => return Err(format!("the WhatsApp inbox row could not be read: {err}")),
    };

    let mut inbox_persistence = persistence_json(
        false,
        None,
        "no WhatsApp inbox row exists; the business profile fields were not stored",
        None,
    );

    if let Some(inbox) = inbox {
        let mut creds_val: Value = match inbox.credentials.as_deref() {
            None => json!({}),
            Some(text) => match serde_json::from_str(text) {
                Ok(value) => value,
                Err(err) => {
                    return Err(format!(
                        "the credentials of WhatsApp inbox '{}' are not valid JSON ({err}); refusing to overwrite",
                        inbox.id
                    ));
                }
            },
        };

        let mut bp = match creds_val.get("businessProfile").and_then(|v| v.as_object()) {
            Some(existing) => existing.clone(),
            None => Map::new(),
        };
        if let Some(v) = &about {
            bp.insert("about".into(), json!(v));
        }
        if let Some(v) = &address {
            bp.insert("address".into(), json!(v));
        }
        if let Some(v) = &description {
            bp.insert("description".into(), json!(v));
        }
        if let Some(v) = &email {
            bp.insert("email".into(), json!(v));
        }
        if let Some(v) = &vertical {
            bp.insert("vertical".into(), json!(v));
        }
        if let Some(v) = &websites {
            bp.insert("websites".into(), json!(v));
        }
        if let Some(v) = &profile_picture_url {
            bp.insert("profilePictureUrl".into(), json!(v));
        }
        creds_val["businessProfile"] = Value::Object(bp);

        let creds_str = match serde_json::to_string(&creds_val) {
            Ok(text) => text,
            Err(err) => {
                return Err(format!(
                    "the merged inbox credentials for inbox '{}' could not be serialized: {err}",
                    inbox.id
                ));
            }
        };

        let inbox_write = sqlx::query(
            "UPDATE inboxes SET credentials = $1, phone_number = COALESCE($2, phone_number), updated_at = $3 WHERE id = $4",
        )
        .bind(&creds_str)
        .bind(&display_phone)
        .bind(&now)
        .bind(&inbox.id)
        .execute(&state.db)
        .await;

        match &inbox_write {
            Ok(report) if report.rows_affected() == 1 => {
                inbox_persistence = persistence_json(
                    true,
                    Some(report.rows_affected()),
                    "the WhatsApp inbox credentials and phone number were stored",
                    None,
                );
            }
            Ok(report) => {
                return Err(format!(
                    "UPDATE inboxes affected {} rows for id '{}': inbox row does not exist",
                    report.rows_affected(),
                    inbox.id
                ));
            }
            Err(err) => {
                return Err(format!(
                    "UPDATE inboxes WHERE id = '{}' failed: {err}",
                    inbox.id
                ));
            }
        }
    }

    // 3. Sync live with Meta Graph API if requested
    let sync_with_meta = sync_with_meta.unwrap_or(true);
    #[derive(sqlx::FromRow)]
    struct TokenRow {
        meta_access_token: Option<String>,
        meta_phone_number_id: Option<String>,
    }

    let meta_sync_result = if !sync_with_meta {
        json!({
            "attempted": false,
            "ok": Value::Null,
            "reason": "sync_with_meta was not requested; Meta was not contacted",
        })
    } else {
        let creds = sqlx::query_as::<_, TokenRow>(
            "SELECT meta_access_token, meta_phone_number_id FROM integration_settings LIMIT 1",
        )
        .fetch_optional(&state.db)
        .await;

        match creds {
            Err(err) => json!({
                "attempted": false,
                "ok": false,
                "reason": format!("the stored Meta credentials could not be read: {err}"),
            }),
            Ok(t_row) => {
                let token = t_row
                    .as_ref()
                    .and_then(|r| r.meta_access_token.clone())
                    .filter(|t| !t.is_empty())
                    .unwrap_or_else(|| state.meta_whatsapp_access_token.clone());
                let phone_id = t_row
                    .as_ref()
                    .and_then(|r| r.meta_phone_number_id.clone())
                    .filter(|p| !p.is_empty())
                    .unwrap_or_else(|| state.meta_whatsapp_phone_number_id.clone());

                if token.is_empty() || phone_id.is_empty() {
                    json!({
                        "attempted": false,
                        "ok": Value::Null,
                        "reason": "no Meta access token or phone number id is configured; the profile was saved locally only",
                    })
                } else {
                    let url = format!(
                        "https://graph.facebook.com/{}/{}/whatsapp_business_profile",
                        state.meta_whatsapp_api_version, phone_id
                    );

                    let mut fields = Map::new();
                    fields.insert("messaging_product".into(), json!("whatsapp"));
                    if let Some(v) = &about {
                        fields.insert("about".into(), json!(v));
                    }
                    if let Some(v) = &address {
                        fields.insert("address".into(), json!(v));
                    }
                    if let Some(v) = &description {
                        fields.insert("description".into(), json!(v));
                    }
                    if let Some(v) = &email {
                        fields.insert("email".into(), json!(v));
                    }
                    if let Some(v) = &vertical {
                        fields.insert("vertical".into(), json!(v));
                    }
                    if let Some(v) = &websites {
                        fields.insert("websites".into(), json!(v));
                    }
                    let meta_body = Value::Object(fields);

                    match state.http_client
                        .post(&url)
                        .header("Authorization", format!("Bearer {}", token))
                        .json(&meta_body)
                        .send()
                        .await
                    {
                        Ok(resp) => {
                            let status = resp.status();
                            let text = match resp.text().await {
                                Ok(body) => body,
                                Err(err) => format!("<the Meta response body could not be read: {err}>"),
                            };
                            info!("Meta Graph API business profile update response: {status} - {text}");
                            json!({
                                "attempted": true,
                                "ok": status.is_success(),
                                "status": status.to_string(),
                                "response": text,
                            })
                        }
                        Err(e) => {
                            warn!("Failed to sync business profile to Meta Graph API: {}", e);
                            json!({
                                "attempted": true,
                                "ok": false,
                                "reason": format!("the Meta Graph API could not be reached: {e}"),
                            })
                        }
                    }
                }
            }
        }
    };

    // Return the fresh read
    match fetch_full_business_profile(state).await {
        Ok(mut profile_json) => {
            profile_json["persistence"] = json!({
                "integrationSettings": settings_persistence,
                "whatsappInbox": inbox_persistence,
            });
            profile_json["metaSync"] = meta_sync_result;
            Ok(profile_json)
        }
        Err(err) => {
            Err(format!(
                "the profile writes ran but the stored values could not be read back: {err}"
            ))
        }
    }
}

pub async fn sync_meta(state: &BusinessProfileState) -> Result<Value, String> {
    #[derive(sqlx::FromRow)]
    struct TokenRow {
        meta_access_token: Option<String>,
        meta_phone_number_id: Option<String>,
    }

    let token_row = match sqlx::query_as::<_, TokenRow>(
        "SELECT meta_access_token, meta_phone_number_id FROM integration_settings LIMIT 1",
    )
    .fetch_optional(&state.db)
    .await
    {
        Ok(row) => row,
        Err(err) => return Err(format!("the stored Meta credentials could not be read: {err}")),
    };

    let token = token_row
        .as_ref()
        .and_then(|r| r.meta_access_token.clone())
        .filter(|t| !t.is_empty())
        .unwrap_or_else(|| state.meta_whatsapp_access_token.clone());

    let phone_id = token_row
        .as_ref()
        .and_then(|r| r.meta_phone_number_id.clone())
        .filter(|p| !p.is_empty())
        .unwrap_or_else(|| state.meta_whatsapp_phone_number_id.clone());

    let meta_profile = fetch_meta_business_profile(
        state,
        &token,
        &phone_id,
        &state.meta_whatsapp_api_version,
    )
    .await;

    let now = Utc::now().to_rfc3339();
    let stored = sqlx::query(
        r#"
        UPDATE integration_settings
        SET whatsapp_verified_name = COALESCE(NULLIF($1, ''), whatsapp_verified_name),
            whatsapp_display_number = COALESCE(NULLIF($2, ''), whatsapp_display_number),
            whatsapp_verified_at = $3,
            updated_at = $4
        "#,
    )
    .bind(&meta_profile.verified_name)
    .bind(&meta_profile.display_phone_number)
    .bind(&now)
    .bind(&now)
    .execute(&state.db)
    .await;

    let verified_persistence = match &stored {
        Ok(report) if report.rows_affected() == 0 => {
            return Err("UPDATE integration_settings affected 0 rows: no integration_settings row exists".to_string());
        }
        Ok(report) => persistence_json(
            true,
            Some(report.rows_affected()),
            "the verified fields returned by Meta were stored on integration_settings",
            None,
        ),
        Err(err) => {
            return Err(format!(
                "Meta returned the verified fields but UPDATE integration_settings failed: {err}"
            ));
        }
    };

    match fetch_full_business_profile(state).await {
        Ok(mut profile_json) => {
            profile_json["persistence"] =
                json!({ "integrationSettingsVerifiedFields": verified_persistence });
            Ok(profile_json)
        }
        Err(err) => {
            Err(format!(
                "the verified fields were stored but the profile could not be read back: {err}"
            ))
        }
    }
}

// ---------------------------------------------------------------------------
// Fetch full business profile
// ---------------------------------------------------------------------------

async fn fetch_full_business_profile(state: &BusinessProfileState) -> Result<Value, String> {
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
        "#,
    )
    .fetch_optional(&state.db)
    .await
    .map_err(|e| format!("could not read integration_settings: {e}"))?;

    #[derive(sqlx::FromRow)]
    struct InboxRow {
        credentials: Option<String>,
    }

    let wa_inbox = sqlx::query_as::<_, InboxRow>(
        "SELECT credentials FROM inboxes WHERE channel_type = 'whatsapp' LIMIT 1",
    )
    .fetch_optional(&state.db)
    .await
    .map_err(|e| format!("could not read inboxes: {e}"))?;

    let mut warnings: Vec<String> = Vec::new();

    if settings.is_none() {
        warnings.push(
            "no integration_settings row exists; the stored business fields are unknown".to_string(),
        );
    }
    if wa_inbox.is_none() {
        warnings.push(
            "no WhatsApp inbox row exists; the stored business profile fields were never saved"
                .to_string(),
        );
    }

    let mut stored_about: Option<String> = None;
    let mut stored_email: Option<String> = None;
    let mut stored_address: Option<String> = None;
    let mut stored_vertical: Option<String> = None;
    let mut stored_websites: Vec<String> = Vec::new();
    let mut stored_pic_url: Option<String> = None;

    if let Some(inbox) = &wa_inbox {
        match inbox.credentials.as_deref() {
            None => warnings.push(
                "the WhatsApp inbox has no credentials JSON; stored business profile fields are \
                 unavailable"
                    .to_string(),
            ),
            Some(text) => match serde_json::from_str::<Value>(text) {
                Ok(c) => {
                    if let Some(bp) = c.get("businessProfile") {
                        stored_about =
                            bp.get("about").and_then(|v| v.as_str()).map(str::to_string);
                        stored_email = bp.get("email").and_then(|v| v.as_str()).map(str::to_string);
                        stored_address =
                            bp.get("address").and_then(|v| v.as_str()).map(str::to_string);
                        stored_vertical =
                            bp.get("vertical").and_then(|v| v.as_str()).map(str::to_string);
                        if let Some(sites) = bp.get("websites").and_then(|v| v.as_array()) {
                            stored_websites =
                                sites.iter().filter_map(|s| s.as_str().map(String::from)).collect();
                        }
                        stored_pic_url = bp
                            .get("profilePictureUrl")
                            .and_then(|v| v.as_str())
                            .map(str::to_string);
                    }
                }
                Err(err) => warnings.push(format!(
                    "the WhatsApp inbox credentials are not valid JSON ({err}); stored business \
                     profile fields are unavailable"
                )),
            },
        }
    }

    fn setting_or_warn<T: serde::Serialize>(
        column: &str,
        value: Option<T>,
        warnings: &mut Vec<String>,
    ) -> Value {
        match value {
            Some(v) => json!(v),
            None => {
                warnings.push(format!(
                    "{column} is not stored in integration_settings; reported as null"
                ));
                Value::Null
            }
        }
    }

    fn meta_or_stored(
        field: &str,
        meta: Option<String>,
        stored: Option<String>,
        warnings: &mut Vec<String>,
    ) -> Value {
        if let Some(v) = meta.filter(|s| !s.is_empty()) {
            return json!(v);
        }
        match stored {
            Some(v) => json!(v),
            None => {
                warnings.push(format!(
                    "{field} is neither reported by Meta nor stored in the WhatsApp inbox \
                     credentials; reported as null"
                ));
                Value::Null
            }
        }
    }

    let token = settings
        .as_ref()
        .and_then(|s| s.meta_access_token.clone())
        .filter(|t| !t.is_empty())
        .unwrap_or_else(|| state.meta_whatsapp_access_token.clone());

    let phone_id = settings
        .as_ref()
        .and_then(|s| s.meta_phone_number_id.clone())
        .filter(|p| !p.is_empty())
        .unwrap_or_else(|| state.meta_whatsapp_phone_number_id.clone());

    let meta_profile = fetch_meta_business_profile(
        state,
        &token,
        &phone_id,
        &state.meta_whatsapp_api_version,
    )
    .await;

    let stored_verified = settings
        .as_ref()
        .and_then(|s| s.whatsapp_verified_name.clone())
        .filter(|v| !v.is_empty());
    let verified_name = Some(meta_profile.verified_name.as_str())
        .filter(|v| !v.trim().is_empty())
        .map(str::to_string)
        .or(stored_verified);

    let stored_display = settings
        .as_ref()
        .and_then(|s| s.whatsapp_display_number.clone())
        .filter(|v| !v.is_empty());
    let display_phone = Some(meta_profile.display_phone_number.as_str())
        .filter(|v| !v.trim().is_empty())
        .map(str::to_string)
        .or(stored_display);

    let business_name =
        setting_or_warn("business_name", settings.as_ref().and_then(|s| s.business_name.clone()), &mut warnings);
    let description = setting_or_warn(
        "business_description",
        settings.as_ref().and_then(|s| s.business_description.clone()),
        &mut warnings,
    );
    let guidelines = setting_or_warn(
        "agent_guidelines",
        settings.as_ref().and_then(|s| s.agent_guidelines.clone()),
        &mut warnings,
    );
    let auto_reply = setting_or_warn(
        "auto_reply_enabled",
        settings.as_ref().and_then(|s| s.auto_reply_enabled),
        &mut warnings,
    );
    let hours_enabled = setting_or_warn(
        "business_hours_enabled",
        settings.as_ref().and_then(|s| s.business_hours_enabled),
        &mut warnings,
    );
    let hours_start = setting_or_warn(
        "business_hours_start",
        settings.as_ref().and_then(|s| s.business_hours_start),
        &mut warnings,
    );
    let hours_end = setting_or_warn(
        "business_hours_end",
        settings.as_ref().and_then(|s| s.business_hours_end),
        &mut warnings,
    );
    let tz = setting_or_warn(
        "business_timezone",
        settings.as_ref().and_then(|s| s.business_timezone.clone()),
        &mut warnings,
    );
    let away = setting_or_warn(
        "away_message",
        settings.as_ref().and_then(|s| s.away_message.clone()),
        &mut warnings,
    );
    let waba_id = setting_or_warn(
        "meta_waba_id",
        settings.as_ref().and_then(|s| s.meta_waba_id.clone()),
        &mut warnings,
    );
    let verified_at = setting_or_warn(
        "whatsapp_verified_at",
        settings.as_ref().and_then(|s| s.whatsapp_verified_at.clone()),
        &mut warnings,
    );
    let verified_name_value = match verified_name {
        Some(v) => json!(v),
        None => {
            warnings.push(
                "no verified display name is reported by Meta or stored in integration_settings; \
                 reported as null"
                    .to_string(),
            );
            Value::Null
        }
    };
    let display_phone_value = match display_phone {
        Some(v) => json!(v),
        None => {
            warnings.push(
                "no display phone number is reported by Meta or stored in integration_settings; \
                 reported as null"
                    .to_string(),
            );
            Value::Null
        }
    };

    Ok(json!({
        "businessName": business_name,
        "verifiedName": verified_name_value,
        "displayPhone": display_phone_value,
        "isConnected": meta_profile.is_connected,
        "connectionError": meta_profile.error.clone(),
        "isVerified": meta_profile.is_verified,
        "verificationStatus": if !meta_profile.is_connected {
            "Not connected"
        } else if meta_profile.verified_name.is_empty() {
            "Unverified display name"
        } else {
            "Verified display name"
        },
        "qualityRating": meta_profile.quality_rating,
        "messagingLimit": meta_profile.messaging_limit_tier,
        "tier": meta_profile.account_mode,
        "spamFlags": match meta_profile.zero_spam_flags {
            Some(true) => "Zero spam flags",
            Some(false) => "Quality issues reported by Meta",
            None => "Not reported by Meta",
        },
        "description": description,
        "about": meta_or_stored("about", meta_profile.about.clone(), stored_about, &mut warnings),
        "email": meta_or_stored("email", meta_profile.email.clone(), stored_email, &mut warnings),
        "address": meta_or_stored("address", meta_profile.address.clone(), stored_address, &mut warnings),
        "vertical": meta_or_stored("vertical", meta_profile.vertical.clone(), stored_vertical, &mut warnings),
        "websites": if !meta_profile.websites.is_empty() { json!(meta_profile.websites) } else { json!(stored_websites) },
        "profilePictureUrl": meta_or_stored("profilePictureUrl", meta_profile.profile_picture_url.clone(), stored_pic_url, &mut warnings),
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
        },
        "warnings": warnings,
    }))
}
