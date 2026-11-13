use crate::{db::AppState, services::meta_sync};
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use chrono::Utc;
use serde::Deserialize;
use serde_json::{json, Map, Value};
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

fn db_error(context: &str, err: sqlx::Error) -> Response {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(json!({
            "error": format!("{context}: {err}"),
            "databaseError": true,
            "saved": false,
        })),
    )
        .into_response()
}

/// Persistence block shape used crate-wide (same fields as
/// `knowledge_base.rs::persistence_json`): `saved` is only ever true when the
/// write actually landed. Each write keeps its own block so a successful one
/// can never merge a failed one into a single "ok".
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

pub async fn get_business_profile(State(state): State<AppState>) -> impl IntoResponse {
    match fetch_full_business_profile(&state).await {
        Ok(profile_json) => Json(profile_json).into_response(),
        // A failed read is a database error, not an all-defaults profile.
        Err(err) => db_error("the stored business profile could not be read", err),
    }
}

pub async fn update_business_profile(
    State(state): State<AppState>,
    Json(payload): Json<UpdateBusinessProfilePayload>,
) -> impl IntoResponse {
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

    // 1. integration_settings. Only fields the caller actually sent are written:
    // COALESCE keeps the stored value for a NULL parameter, so an omitted field
    // is no longer blanked out to "" and then echoed back as a saved choice.
    let settings_write = sqlx::query(
        r#"
        UPDATE integration_settings
        SET business_name = COALESCE(?, business_name),
            whatsapp_verified_name = COALESCE(?, whatsapp_verified_name),
            whatsapp_display_number = COALESCE(?, whatsapp_display_number),
            business_description = COALESCE(?, business_description),
            agent_guidelines = COALESCE(?, agent_guidelines),
            auto_reply_enabled = COALESCE(?, auto_reply_enabled),
            business_hours_enabled = COALESCE(?, business_hours_enabled),
            business_hours_start = COALESCE(?, business_hours_start),
            business_hours_end = COALESCE(?, business_hours_end),
            business_timezone = COALESCE(?, business_timezone),
            away_message = COALESCE(?, away_message),
            updated_at = ?
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
            return (
                StatusCode::NOT_FOUND,
                Json(json!({
                    "error": "UPDATE integration_settings affected 0 rows: no integration_settings \
                              row exists, so nothing was changed.",
                    "saved": false,
                    "rowsAffected": 0,
                    "persistence": persistence_json(
                        false,
                        Some(0),
                        "no integration_settings row exists; nothing was changed",
                        None,
                    ),
                })),
            )
                .into_response();
        }
        Ok(report) => persistence_json(
            true,
            Some(report.rows_affected()),
            "integration_settings business fields updated",
            None,
        ),
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": format!(
                        "the business profile could not be written to integration_settings: {err}"
                    ),
                    "saved": false,
                    "databaseError": true,
                    "persistence": persistence_json(
                        false,
                        None,
                        "the integration_settings UPDATE failed",
                        Some(err.to_string()),
                    ),
                })),
            )
                .into_response();
        }
    };

    // 2. WhatsApp inbox credentials JSON.
    #[derive(sqlx::FromRow)]
    struct InboxRow {
        id: String,
        credentials: Option<String>,
    }

    // `Err` is a failed read of the inbox table; only `Ok(None)` means the
    // WhatsApp inbox genuinely does not exist.
    let inbox = match sqlx::query_as::<_, InboxRow>(
        "SELECT id, credentials FROM inboxes WHERE channel_type = 'whatsapp' LIMIT 1",
    )
    .fetch_optional(&state.db)
    .await
    {
        Ok(row) => row,
        Err(err) => return db_error("the WhatsApp inbox row could not be read", err),
    };

    let mut inbox_persistence = persistence_json(
        false,
        None,
        "no WhatsApp inbox row exists; the business profile fields were not stored",
        None,
    );
    if let Some(inbox) = inbox {
        // Unparseable stored credentials are refused, not reset to {}: writing
        // the freshly built object over them would delete the real ones.
        let mut creds_val: Value = match inbox.credentials.as_deref() {
            None => json!({}),
            Some(text) => match serde_json::from_str(text) {
                Ok(value) => value,
                Err(err) => {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(json!({
                            "error": format!(
                                "the credentials of WhatsApp inbox '{}' are not valid JSON ({err}); \
                                 refusing to overwrite them, so the business profile fields were \
                                 not stored",
                                inbox.id
                            ),
                            "saved": false,
                            "persistence": json!({
                                "integrationSettings": settings_persistence,
                                "whatsappInbox": persistence_json(
                                    false,
                                    None,
                                    "stored credentials are not valid JSON; the write was refused",
                                    Some(err.to_string()),
                                ),
                            }),
                        })),
                    )
                        .into_response();
                }
            },
        };

        // Only the fields the caller sent are merged into businessProfile; the
        // rest of the stored object keeps its previous values instead of being
        // replaced by "" stand-ins.
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
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({
                        "error": format!(
                            "the merged inbox credentials for inbox '{}' could not be serialized, \
                             so nothing was written to it: {err}",
                            inbox.id
                        ),
                        "saved": false,
                        "persistence": json!({
                            "integrationSettings": settings_persistence,
                            "whatsappInbox": persistence_json(
                                false,
                                None,
                                "the merged credentials could not be serialized; the write was refused",
                                Some(err.to_string()),
                            ),
                        }),
                    })),
                )
                    .into_response();
            }
        };

        let inbox_write = sqlx::query(
            "UPDATE inboxes SET credentials = ?, phone_number = COALESCE(?, phone_number), updated_at = ? WHERE id = ?",
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
                // The row was read moments ago and then the UPDATE hit nothing:
                // say the id and the count, do not claim a stored profile.
                return (
                    StatusCode::NOT_FOUND,
                    Json(json!({
                        "error": format!(
                            "UPDATE inboxes SET credentials WHERE id = '{}' affected {} rows: no \
                             inbox row with id '{}' exists, so the business profile fields were \
                             not stored and nothing was changed there.",
                            inbox.id,
                            report.rows_affected(),
                            inbox.id
                        ),
                        "saved": false,
                        "id": inbox.id,
                        "rowsAffected": report.rows_affected(),
                        "persistence": json!({
                            "integrationSettings": settings_persistence,
                            "whatsappInbox": persistence_json(
                                false,
                                Some(report.rows_affected()),
                                "the WhatsApp inbox row does not exist; nothing was stored there",
                                None,
                            ),
                        }),
                    })),
                )
                    .into_response();
            }
            Err(err) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({
                        "error": format!(
                            "UPDATE inboxes SET credentials WHERE id = '{}' failed: {err} -- the \
                             business profile fields were not stored on the inbox.",
                            inbox.id
                        ),
                        "saved": false,
                        "databaseError": true,
                        "id": inbox.id,
                        "persistence": json!({
                            "integrationSettings": settings_persistence,
                            "whatsappInbox": persistence_json(
                                false,
                                None,
                                "the WhatsApp inbox credentials UPDATE failed",
                                Some(err.to_string()),
                            ),
                        }),
                    })),
                )
                    .into_response();
            }
        }
    }

    // 3. Sync live with Meta Graph API if requested.
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
            // The two local writes above already happened; a failed read here
            // is reported in this block instead of silently skipping the sync.
            Err(err) => json!({
                "attempted": false,
                "ok": false,
                "reason": format!(
                    "the stored Meta credentials could not be read, so no sync was attempted: {err}"
                ),
            }),
            Ok(t_row) => {
                // Credential resolution order this backend uses: stored settings
                // row, then the process environment. Both are real values.
                let token = t_row
                    .as_ref()
                    .and_then(|r| r.meta_access_token.clone())
                    .filter(|t| !t.is_empty())
                    .unwrap_or_else(|| state.config.meta_whatsapp_access_token.clone());
                let phone_id = t_row
                    .as_ref()
                    .and_then(|r| r.meta_phone_number_id.clone())
                    .filter(|p| !p.is_empty())
                    .unwrap_or_else(|| state.config.meta_whatsapp_phone_number_id.clone());

                if token.is_empty() || phone_id.is_empty() {
                    json!({
                        "attempted": false,
                        "ok": Value::Null,
                        "reason": "no Meta access token or phone number id is configured; the profile was saved locally only",
                    })
                } else {
                    let url = format!(
                        "https://graph.facebook.com/{}/{}/whatsapp_business_profile",
                        state.config.meta_whatsapp_api_version, phone_id
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
                                // An unreadable body is named as such; "" would
                                // read like an empty answer from Meta.
                                Err(err) => {
                                    format!("<the Meta response body could not be read: {err}>")
                                }
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

    // The reply is the fresh read of the stored profile: what was written and
    // what could be read back are reported separately, never merged into ok.
    match fetch_full_business_profile(&state).await {
        Ok(mut profile_json) => {
            profile_json["persistence"] = json!({
                "integrationSettings": settings_persistence,
                "whatsappInbox": inbox_persistence,
            });
            profile_json["metaSync"] = meta_sync_result;
            Json(profile_json).into_response()
        }
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": format!(
                    "the profile writes ran but the stored values could not be read back: {err}"
                ),
                "databaseError": true,
                "persistence": json!({
                    "integrationSettings": settings_persistence,
                    "whatsappInbox": inbox_persistence,
                }),
                "metaSync": meta_sync_result,
            })),
        )
            .into_response(),
    }
}

pub async fn sync_meta(State(state): State<AppState>) -> impl IntoResponse {
    #[derive(sqlx::FromRow)]
    struct TokenRow {
        meta_access_token: Option<String>,
        meta_phone_number_id: Option<String>,
    }

    // A failed read is reported as one; `Ok(None)` alone means "no settings row".
    let token_row = match sqlx::query_as::<_, TokenRow>(
        "SELECT meta_access_token, meta_phone_number_id FROM integration_settings LIMIT 1",
    )
    .fetch_optional(&state.db)
    .await
    {
        Ok(row) => row,
        Err(err) => return db_error("the stored Meta credentials could not be read", err),
    };

    let token = token_row
        .as_ref()
        .and_then(|r| r.meta_access_token.clone())
        .filter(|t| !t.is_empty())
        .unwrap_or_else(|| state.config.meta_whatsapp_access_token.clone());

    let phone_id = token_row
        .as_ref()
        .and_then(|r| r.meta_phone_number_id.clone())
        .filter(|p| !p.is_empty())
        .unwrap_or_else(|| state.config.meta_whatsapp_phone_number_id.clone());

    let meta_profile = meta_sync::fetch_meta_business_profile(
        &state,
        &token,
        &phone_id,
        &state.config.meta_whatsapp_api_version,
    )
    .await;

    // Update verified fields in integration_settings. COALESCE(NULLIF(...))
    // keeps the stored value where Meta did not actually answer, instead of
    // storing "" and then reading it back as if Meta had verified the number.
    let now = Utc::now().to_rfc3339();
    let stored = sqlx::query(
        r#"
        UPDATE integration_settings
        SET whatsapp_verified_name = COALESCE(NULLIF(?, ''), whatsapp_verified_name),
            whatsapp_display_number = COALESCE(NULLIF(?, ''), whatsapp_display_number),
            whatsapp_verified_at = ?,
            updated_at = ?
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
            return (
                StatusCode::NOT_FOUND,
                Json(json!({
                    "error": "UPDATE integration_settings affected 0 rows: no integration_settings \
                              row exists, so the verified fields Meta returned were not stored and \
                              nothing was changed.",
                    "saved": false,
                    "rowsAffected": 0,
                    "verifiedName": meta_profile.verified_name,
                    "displayPhone": meta_profile.display_phone_number,
                    "persistence": persistence_json(
                        false,
                        Some(0),
                        "no integration_settings row exists; the Meta values were not stored",
                        None,
                    ),
                })),
            )
                .into_response();
        }
        Ok(report) => persistence_json(
            true,
            Some(report.rows_affected()),
            "the verified fields returned by Meta were stored on integration_settings",
            None,
        ),
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": format!(
                        "Meta returned the verified fields but UPDATE integration_settings failed, \
                         so they were not stored: {err}"
                    ),
                    "saved": false,
                    "databaseError": true,
                    "verifiedName": meta_profile.verified_name,
                    "displayPhone": meta_profile.display_phone_number,
                    "persistence": persistence_json(
                        false,
                        None,
                        "the integration_settings UPDATE failed",
                        Some(err.to_string()),
                    ),
                })),
            )
                .into_response();
        }
    };

    match fetch_full_business_profile(&state).await {
        Ok(mut profile_json) => {
            profile_json["persistence"] =
                json!({ "integrationSettingsVerifiedFields": verified_persistence });
            Json(profile_json).into_response()
        }
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": format!(
                    "the verified fields were stored but the profile could not be read back: {err}"
                ),
                "databaseError": true,
                "persistence": json!({
                    "integrationSettingsVerifiedFields": verified_persistence,
                }),
            })),
        )
            .into_response(),
    }
}

async fn fetch_full_business_profile(state: &AppState) -> Result<Value, sqlx::Error> {
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

    // `?` keeps Err a database error for the caller; Ok(None) is "no row",
    // reported below as missing fields rather than as invented defaults.
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
    .await?;

    #[derive(sqlx::FromRow)]
    struct InboxRow {
        credentials: Option<String>,
    }

    let wa_inbox = sqlx::query_as::<_, InboxRow>(
        "SELECT credentials FROM inboxes WHERE channel_type = 'whatsapp' LIMIT 1",
    )
    .fetch_optional(&state.db)
    .await?;

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

    // Stored profile fields stay Option: an absent value is reported as null
    // with a warning, not as an empty string that looks like a saved choice.
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

    // A missing stored column is null + a warning naming the column; a name,
    // hours window, timezone or toggle is never invented here.
    fn setting_or_warn<T: serde::Serialize>(
        column: &str,
        value: Option<T>,
        warnings: &mut Vec<String>,
    ) -> Value {
        match value {
            Some(v) => json!(v),
            None => {
                warnings.push(format!(
                    "{column} is not stored in integration_settings; reported as null instead of \
                     an invented default"
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
        .unwrap_or_else(|| state.config.meta_whatsapp_access_token.clone());

    let phone_id = settings
        .as_ref()
        .and_then(|s| s.meta_phone_number_id.clone())
        .filter(|p| !p.is_empty())
        .unwrap_or_else(|| state.config.meta_whatsapp_phone_number_id.clone());

    let meta_profile = meta_sync::fetch_meta_business_profile(
        &state,
        &token,
        &phone_id,
        &state.config.meta_whatsapp_api_version,
    )
    .await;

    // verifiedName is Meta's live answer, then the stored one -- never the
    // business name as a stand-in, which presented an unverified account as
    // having a verified display name.
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
