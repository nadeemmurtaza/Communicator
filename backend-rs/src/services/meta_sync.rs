#![allow(dead_code)]

use crate::db::AppState;
use crate::models::BusinessProfile;
use serde_json::Value;
use sqlx::Row;

fn unknown_profile(error: Option<String>, is_connected: bool) -> BusinessProfile {
    BusinessProfile {
        is_connected,
        error,
        display_phone_number: String::new(),
        verified_name: String::new(),
        quality_rating: String::new(),
        messaging_limit_tier: String::new(),
        official_business_account: false,
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

/// Meta's `messaging_limit_tier` -> the daily cap it actually means. Meta
/// omits the tier for scaled accounts, which is reported as unknown rather
/// than assumed to be the entry-level limit.
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

/// Meta's error envelope: `{ "error": { "message", "code", "error_subcode" } }`.
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
    state: &AppState,
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

/// Live read of the WhatsApp phone-number node and business profile from the
/// Meta Graph API. Every field comes from Meta or is left empty: nothing here
/// reports a status the platform did not actually return.
pub async fn fetch_meta_business_profile(
    state: &AppState,
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
    // A masked credential stored for display is not a credential.
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
        // The phone node answered, so the channel is connected; the profile
        // detail is simply unavailable and the reason is reported.
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
        // Display-name approval, not the SMS code check, is what "verified"
        // means for a Cloud API number.
        is_verified: !phone
            .get("verified_name")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .is_empty(),
        official_business_account: phone
            .get("account_mode")
            .and_then(|v| v.as_str())
            .map(|mode| mode.eq_ignore_ascii_case("TAG"))
            .unwrap_or(false),
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

/// Credentials resolution order the rest of this backend uses: the inbox row
/// the tenant connected, then the settings row, then process environment.
pub async fn resolve_meta_credentials(state: &AppState) -> (String, String) {
    let inbox = sqlx::query("SELECT credentials FROM inboxes WHERE channel_type = 'whatsapp' LIMIT 1")
        .fetch_optional(&state.db)
        .await
        .ok()
        .flatten();

    let mut token = String::new();
    let mut phone_id = String::new();

    if let Some(row) = inbox {
        let raw: Option<String> = row.try_get("credentials").ok().flatten();
        if let Some(text) = raw.and_then(|text| serde_json::from_str::<Value>(&text).ok()) {
            token = text
                .get("accessToken")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string();
            phone_id = text
                .get("phoneNumberId")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string();
        }
    }

    if token.is_empty() || phone_id.is_empty() {
        let settings = sqlx::query("SELECT meta_access_token, meta_phone_number_id FROM integration_settings LIMIT 1")
            .fetch_optional(&state.db)
            .await
            .ok()
            .flatten();
        if let Some(row) = settings {
            if token.is_empty() {
                token = row
                    .try_get::<Option<String>, _>("meta_access_token")
                    .ok()
                    .flatten()
                    .unwrap_or_default();
            }
            if phone_id.is_empty() {
                phone_id = row
                    .try_get::<Option<String>, _>("meta_phone_number_id")
                    .ok()
                    .flatten()
                    .unwrap_or_default();
            }
        }
    }

    if token.is_empty() {
        token = state.config.meta_whatsapp_access_token.clone();
    }
    if phone_id.is_empty() {
        phone_id = state.config.meta_whatsapp_phone_number_id.clone();
    }

    (token, phone_id)
}

/// Refreshes the live Meta state after a `phone_number_quality_update`-style
/// webhook field change and persists only what Meta actually reported.
pub async fn refresh_channel_info(state: &AppState) -> Value {
    let (token, phone_id) = resolve_meta_credentials(state).await;
    let profile =
        fetch_meta_business_profile(state, &token, &phone_id, &state.config.meta_whatsapp_api_version)
            .await;

    if profile.is_connected {
        let now = chrono::Utc::now().to_rfc3339();
        let updated = sqlx::query(
            "UPDATE integration_settings SET \
               whatsapp_display_number = COALESCE(NULLIF(?, ''), whatsapp_display_number), \
               whatsapp_verified_name = COALESCE(NULLIF(?, ''), whatsapp_verified_name), \
               whatsapp_verified_at = ?, updated_at = ? \
             WHERE id = (SELECT id FROM integration_settings LIMIT 1)",
        )
        .bind(&profile.display_phone_number)
        .bind(&profile.verified_name)
        .bind(&now)
        .bind(&now)
        .execute(&state.db)
        .await;

        // `Err` and `Ok(0 rows)` are different failures and used to be merged:
        // a row that exists but could not be written was treated as "no row at
        // all" and re-INSERTed (a duplicate settings row), while both outcomes
        // were then discarded, so a sync that stored nothing still reported the
        // refreshed Meta profile. Only the genuinely empty table gets a row.
        let needs_first_row = match &updated {
            Ok(report) => report.rows_affected() == 0,
            Err(err) => {
                tracing::error!(
                    "[DB] Meta re-sync could not UPDATE integration_settings with the profile Meta \
                     returned for phone number id '{phone_id}' (verified name '{}', display number \
                     '{}'): {err} -- the stored channel info stays stale",
                    profile.verified_name,
                    profile.display_phone_number
                );
                false
            }
        };

        if needs_first_row {
            let settings_id = uuid::Uuid::new_v4().simple().to_string();
            let inserted = sqlx::query(
                "INSERT INTO integration_settings (id, whatsapp_display_number, \
                 whatsapp_verified_name, whatsapp_verified_at, updated_at) VALUES (?, ?, ?, ?, ?)",
            )
            .bind(&settings_id)
            .bind(&profile.display_phone_number)
            .bind(&profile.verified_name)
            .bind(&now)
            .bind(&now)
            .execute(&state.db)
            .await;
            match inserted {
                Ok(report) if report.rows_affected() == 1 => {
                    // The refreshed profile is the settings row now.
                }
                Ok(report) => tracing::error!(
                    "[DB] Meta re-sync INSERT INTO integration_settings (id '{settings_id}' for \
                     phone number id '{phone_id}') affected {} rows (expected 1) -- the refreshed \
                     profile was not stored",
                    report.rows_affected()
                ),
                Err(err) => tracing::error!(
                    "[DB] Meta re-sync could not INSERT the first integration_settings row (id \
                     '{settings_id}' for phone number id '{phone_id}'): {err} -- the refreshed \
                     profile was not stored"
                ),
            }
        }
    } else if let Some(error) = &profile.error {
        tracing::warn!("Meta channel re-sync failed: {error}");
    }

    serde_json::to_value(&profile).unwrap_or(Value::Null)
}
