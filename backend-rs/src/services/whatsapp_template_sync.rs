#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::time::Duration;

use crate::config::Config;
use crate::services::whatsapp::WhatsAppCredentials;

// ---------------------------------------------------------------------------
// Data structures
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TemplateInfo {
    pub name: String,
    pub status: String,
    pub category: String,
    pub language: String,
    pub body: String,
    pub components: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MessageLimitsInfo {
    pub quality_rating: String,
    pub messaging_limit_tier: String,
    pub messaging_limit: i64,
    pub utilitarian: i64,
    pub marketing: i64,
    pub authentication: i64,
    pub two_sided_conversation: i64,
    pub period_start: String,
    pub period_end: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncResult {
    pub synced_count: i64,
    pub updated_count: i64,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LimitsUpdateResult {
    pub brand_id: String,
    pub quality_rating: String,
    pub messaging_tier: String,
    pub total_limit: i64,
    pub utility_used: i64,
    pub marketing_used: i64,
    pub authentication_used: i64,
    pub customer_service_used: i64,
    pub remaining: i64,
    pub period_start: String,
    pub period_end: String,
}

// ---------------------------------------------------------------------------
// Template Sync Service
// ---------------------------------------------------------------------------

pub struct TemplateSyncService {
    pool: PgPool,
    config: Config,
    client: reqwest::Client,
}

impl TemplateSyncService {
    pub fn new(pool: PgPool, config: Config) -> Self {
        Self {
            pool,
            config,
            client: reqwest::Client::builder()
                .timeout(Duration::from_secs(60))
                .build()
                .unwrap_or_default(),
        }
    }

    /// Resolve WhatsApp credentials for a brand (same pattern as whatsapp.rs)
    async fn resolve_credentials(&self, brand_id: &str) -> Result<WhatsAppCredentials, String> {
        let row: Option<(String, String)> = sqlx::query_as(
            "SELECT credentials, channel_type FROM brand_credentials WHERE brand_id = ?1 AND channel_type = 'whatsapp' AND is_active = 1 LIMIT 1"
        )
        .bind(brand_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("DB error: {}", e))?;

        if let Some((credentials_json, _)) = row {
            if let Ok(creds) = serde_json::from_str::<serde_json::Value>(&credentials_json) {
                let token = creds["access_token"].as_str().unwrap_or("");
                let phone_id = creds["phone_number_id"].as_str().unwrap_or("");
                if !token.is_empty() && !phone_id.is_empty() {
                    return Ok(WhatsAppCredentials {
                        access_token: token.to_string(),
                        phone_number_id: phone_id.to_string(),
                        api_version: creds["api_version"].as_str().map(|s| s.to_string()),
                    });
                }
            }
        }

        let row: Option<(String,)> = sqlx::query_as(
            "SELECT credentials FROM integration_settings WHERE key = 'whatsapp_meta' LIMIT 1"
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("DB error: {}", e))?;

        if let Some((credentials_json,)) = row {
            if let Ok(creds) = serde_json::from_str::<serde_json::Value>(&credentials_json) {
                let token = creds["access_token"].as_str().unwrap_or("");
                let phone_id = creds["phone_number_id"].as_str().unwrap_or("");
                if !token.is_empty() && !phone_id.is_empty() {
                    return Ok(WhatsAppCredentials {
                        access_token: token.to_string(),
                        phone_number_id: phone_id.to_string(),
                        api_version: creds["api_version"].as_str().map(|s| s.to_string()),
                    });
                }
            }
        }

        if !self.config.meta_whatsapp_access_token.is_empty()
            && !self.config.meta_whatsapp_phone_number_id.is_empty()
        {
            return Ok(WhatsAppCredentials {
                access_token: self.config.meta_whatsapp_access_token.clone(),
                phone_number_id: self.config.meta_whatsapp_phone_number_id.clone(),
                api_version: if self.config.meta_whatsapp_api_version.is_empty() {
                    None
                } else {
                    Some(self.config.meta_whatsapp_api_version.clone())
                },
            });
        }

        Err("No WhatsApp credentials found for this brand".to_string())
    }

    fn graph_url(&self, creds: &WhatsAppCredentials, resource: &str) -> String {
        let version = creds.api_version.as_deref().unwrap_or("v18.0");
        format!(
            "https://graph.facebook.com/{}/{}{}",
            version, creds.phone_number_id, resource
        )
    }

    async fn graph_get(&self, url: &str, token: &str) -> Result<serde_json::Value, String> {
        let mut delay = Duration::from_millis(500);
        let max_retries = 3;

        for attempt in 0..=max_retries {
            match self
                .client
                .get(url)
                .header("Authorization", format!("Bearer {}", token))
                .send()
                .await
            {
                Ok(response) => {
                    let status = response.status();
                    let body = response.text().await.unwrap_or_default();

                    if status.is_success() {
                        return Ok(serde_json::from_str(&body)
                            .unwrap_or_else(|_| serde_json::json!({"raw": body})));
                    }

                    let err_json: serde_json::Value =
                        serde_json::from_str(&body).unwrap_or_else(|_| serde_json::json!({}));
                    let err_msg = err_json["error"]["message"]
                        .as_str()
                        .unwrap_or(&body);

                    if (status == 429 || status.is_server_error()) && attempt < max_retries {
                        tokio::time::sleep(delay).await;
                        delay *= 2;
                        continue;
                    }

                    return Err(format!("Meta API error {}: {}", status, err_msg));
                }
                Err(e) => {
                    if attempt < max_retries {
                        tokio::time::sleep(delay).await;
                        delay *= 2;
                        continue;
                    }
                    return Err(format!("Network error: {}", e));
                }
            }
        }

        Err("Max retries exceeded".to_string())
    }

    // ---------------------------------------------------------------------------
    // Sync templates from Meta
    // ---------------------------------------------------------------------------

    pub async fn sync_templates_from_meta(&self, brand_id: &str) -> Result<SyncResult, String> {
        let creds = self.resolve_credentials(brand_id).await?;
        let url = self.graph_url(&creds, "/message_templates");

        let body = self.graph_get(&url, &creds.access_token).await?;

        let mut synced = 0i64;
        let mut updated = 0i64;
        let mut errors = Vec::new();

        let now = chrono::Utc::now().to_rfc3339();

        if let Some(templates) = body.get("data").and_then(|d| d.as_array()) {
            for template in templates {
                let name = template.get("name").and_then(|v| v.as_str()).unwrap_or("");
                let status = template.get("status").and_then(|v| v.as_str()).unwrap_or("");
                let category = template.get("category").and_then(|v| v.as_str()).unwrap_or("");
                let language = template.get("language").and_then(|v| v.as_str()).unwrap_or("");
                let body_text = template
                    .get("body")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let components = template.get("components").map(|v| v.to_string());

                if name.is_empty() {
                    continue;
                }

                let existing: Option<(String,)> = sqlx::query_as(
                    "SELECT id FROM whatsapp_templates WHERE brand_id = ?1 AND name = ?2 LIMIT 1"
                )
                .bind(brand_id)
                .bind(name)
                .fetch_optional(&self.pool)
                .await
                .map_err(|e| format!("DB error: {}", e))?;

                if existing.is_some() {
                    match sqlx::query(
                        "UPDATE whatsapp_templates SET status = ?, category = ?, language = ?, body = ?, components = ?, last_synced_at = ?, updated_at = ? WHERE brand_id = ? AND name = ?"
                    )
                    .bind(status)
                    .bind(category)
                    .bind(language)
                    .bind(&body_text)
                    .bind(components.as_ref().map(|c| c.as_str()))
                    .bind(&now)
                    .bind(&now)
                    .bind(brand_id)
                    .bind(name)
                    .execute(&self.pool)
                    .await
                    {
                        Ok(_) => updated += 1,
                        Err(e) => errors.push(format!("Update template {}: {}", name, e)),
                    }
                } else {
                    let id = uuid::Uuid::new_v4().to_string();
                    match sqlx::query(
                        "INSERT INTO whatsapp_templates (id, brand_id, name, category, language, status, body, components, created_at, updated_at, last_synced_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
                    )
                    .bind(&id)
                    .bind(brand_id)
                    .bind(name)
                    .bind(category)
                    .bind(language)
                    .bind(status)
                    .bind(&body_text)
                    .bind(components.as_ref().map(|c| c.as_str()))
                    .bind(&now)
                    .bind(&now)
                    .bind(&now)
                    .execute(&self.pool)
                    .await
                    {
                        Ok(_) => synced += 1,
                        Err(e) => errors.push(format!("Insert template {}: {}", name, e)),
                    }
                }
            }
        } else {
            errors.push("No templates found in Meta response".to_string());
        }

        Ok(SyncResult {
            synced_count: synced,
            updated_count: updated,
            errors,
        })
    }

    // ---------------------------------------------------------------------------
    // Fetch message limits from Meta
    // ---------------------------------------------------------------------------

    pub async fn fetch_message_limits_from_meta(
        &self,
        brand_id: &str,
    ) -> Result<MessageLimitsInfo, String> {
        let creds = self.resolve_credentials(brand_id).await?;
        let url = self.graph_url(
            &creds,
            "/?fields=messaging_limit_tier,messaging_limit,quality_rating",
        );

        let body = self.graph_get(&url, &creds.access_token).await?;

        let quality_rating = body
            .get("quality_rating")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let messaging_tier = body
            .get("messaging_limit_tier")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let messaging_limit = body
            .get("messaging_limit")
            .and_then(|v| v.as_i64())
            .unwrap_or(1000);

        let period_start = body
            .get("period_start")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let period_end = body
            .get("period_end")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let utilitarian = body.get("utilitarian").and_then(|v| v.as_i64()).unwrap_or(0);
        let marketing = body.get("marketing").and_then(|v| v.as_i64()).unwrap_or(0);
        let authentication = body
            .get("authentication")
            .and_then(|v| v.as_i64())
            .unwrap_or(0);
        let two_sided = body
            .get("two_sided_conversation")
            .and_then(|v| v.as_i64())
            .unwrap_or(0);

        Ok(MessageLimitsInfo {
            quality_rating,
            messaging_limit_tier: messaging_tier,
            messaging_limit,
            utilitarian,
            marketing,
            authentication,
            two_sided_conversation: two_sided,
            period_start,
            period_end,
        })
    }

    // ---------------------------------------------------------------------------
    // Update message limits in database
    // ---------------------------------------------------------------------------

    pub async fn update_message_limits_in_db(
        &self,
        brand_id: &str,
    ) -> Result<LimitsUpdateResult, String> {
        let limits = self.fetch_message_limits_from_meta(brand_id).await?;

        let now = chrono::Utc::now();
        let period_start = if limits.period_start.is_empty() {
            chrono::DateTime::parse_from_rfc3339(&format!("{}T00:00:00+00:00", now.format("%Y-%m-%d"))).unwrap().to_rfc3339()
        } else {
            limits.period_start.clone()
        };
        let period_end = if limits.period_end.is_empty() {
            let tomorrow = now + chrono::Duration::days(1);
            chrono::DateTime::parse_from_rfc3339(&format!("{}T00:00:00+00:00", tomorrow.format("%Y-%m-%d"))).unwrap().to_rfc3339()
        } else {
            limits.period_end.clone()
        };

        let counts: (i64, i64, i64, i64) = sqlx::query_as(
            r#"
            SELECT
                COALESCE(SUM(CASE WHEN status = 'sent' THEN 1 ELSE 0 END), 0),
                COALESCE(SUM(CASE WHEN status = 'sent' THEN 1 ELSE 0 END), 0),
                COALESCE(SUM(CASE WHEN status = 'sent' THEN 1 ELSE 0 END), 0),
                COALESCE(SUM(CASE WHEN status = 'sent' THEN 1 ELSE 0 END), 0)
            FROM whatsapp_template_log
            WHERE brand_id = ?1 AND created_at >= ?2 AND created_at < ?3
            "#
        )
        .bind(brand_id)
        .bind(&period_start)
        .bind(&period_end)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("DB error: {}", e))?
        .unwrap_or((0, 0, 0, 0));

        let total_used = counts.0 + counts.1 + counts.2 + counts.3;
        let remaining = (limits.messaging_limit as i64 - total_used).max(0);

        let _ = sqlx::query(
            "INSERT INTO message_limits (id, brand_id, period_start, period_end, utility_count, marketing_count, authentication_count, customer_service_count, total_limit, remaining, quality_rating, messaging_tier, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
             ON CONFLICT(brand_id, period_start) DO UPDATE SET
             utility_count = excluded.utility_count, marketing_count = excluded.marketing_count,
             authentication_count = excluded.authentication_count, customer_service_count = excluded.customer_service_count,
             total_limit = excluded.total_limit, remaining = excluded.remaining,
             quality_rating = excluded.quality_rating, messaging_tier = excluded.messaging_tier,
             updated_at = excluded.updated_at"
        )
        .bind(uuid::Uuid::new_v4().to_string())
        .bind(brand_id)
        .bind(&period_start)
        .bind(&period_end)
        .bind(counts.0)
        .bind(counts.1)
        .bind(counts.2)
        .bind(counts.3)
        .bind(limits.messaging_limit)
        .bind(remaining)
        .bind(&limits.quality_rating)
        .bind(&limits.messaging_limit_tier)
        .bind(&now.to_rfc3339())
        .bind(&now.to_rfc3339())
        .execute(&self.pool)
        .await
        .map_err(|e| format!("DB error: {}", e))?;

        Ok(LimitsUpdateResult {
            brand_id: brand_id.to_string(),
            quality_rating: limits.quality_rating,
            messaging_tier: limits.messaging_limit_tier,
            total_limit: limits.messaging_limit,
            utility_used: counts.0,
            marketing_used: counts.1,
            authentication_used: counts.2,
            customer_service_used: counts.3,
            remaining,
            period_start,
            period_end,
        })
    }

    // ---------------------------------------------------------------------------
    // Check message limits before sending
    // ---------------------------------------------------------------------------

    pub async fn check_message_limit(&self, brand_id: &str) -> Result<bool, String> {
        let now = chrono::Utc::now().to_rfc3339();
        let row: Option<(i64, i64)> = sqlx::query_as(
            "SELECT total_limit, remaining FROM message_limits WHERE brand_id = ?1 AND period_start <= ?2 AND period_end > ?2 ORDER BY period_start DESC LIMIT 1"
        )
        .bind(brand_id)
        .bind(&now)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("DB error: {}", e))?;

        match row {
            Some((total, remaining)) => Ok(remaining > 0 && total > 0),
            None => Ok(true),
        }
    }

    pub async fn get_remaining_limits(&self, brand_id: &str) -> Result<serde_json::Value, String> {
        let now = chrono::Utc::now().to_rfc3339();
        let row: Option<(String, String, i64, i64, i64, i64, i64, i64, String, String)> =
            sqlx::query_as(
                "SELECT quality_rating, messaging_tier, total_limit, remaining, utility_count, marketing_count, authentication_count, customer_service_count, period_start, period_end
                 FROM message_limits WHERE brand_id = ?1 AND period_start <= ?2 AND period_end > ?2
                 ORDER BY period_start DESC LIMIT 1"
            )
            .bind(brand_id)
            .bind(&now)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| format!("DB error: {}", e))?;

        match row {
            Some((qr, tier, total, remaining, util, mkt, auth, cs, ps, pe)) => Ok(serde_json::json!({
                "qualityRating": qr,
                "messagingTier": tier,
                "totalLimit": total,
                "remaining": remaining,
                "utilityUsed": util,
                "marketingUsed": mkt,
                "authenticationUsed": auth,
                "customerServiceUsed": cs,
                "periodStart": ps,
                "periodEnd": pe
            })),
            None => Ok(serde_json::json!({
                "qualityRating": "unknown",
                "messagingTier": "unknown",
                "totalLimit": 0,
                "remaining": 0,
                "utilityUsed": 0,
                "marketingUsed": 0,
                "authenticationUsed": 0,
                "customerServiceUsed": 0,
                "periodStart": "",
                "periodEnd": ""
            })),
        }
    }

    // ---------------------------------------------------------------------------
    // List cached templates
    // ---------------------------------------------------------------------------

    pub async fn list_templates(
        &self,
        brand_id: &str,
        category: Option<&str>,
        status: Option<&str>,
    ) -> Result<Vec<serde_json::Value>, String> {
        let rows: Vec<(String, String, Option<String>, Option<String>, String, String, Option<String>)> =
            sqlx::query_as(
                "SELECT id, name, category, language, status, body, last_synced_at FROM whatsapp_templates WHERE brand_id = ?1 ORDER BY name LIMIT 500"
            )
            .bind(brand_id)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| format!("DB error: {}", e))?;

        let mut result = Vec::new();
        for (id, name, cat, lang, status_val, body, synced) in rows {
            let cat_str = cat.unwrap_or_default();
            let status_str = status_val;

            if let Some(c) = category {
                if cat_str != c {
                    continue;
                }
            }
            if let Some(s) = status {
                if status_str != *s {
                    continue;
                }
            }

            result.push(serde_json::json!({
                "id": id,
                "name": name,
                "category": cat_str,
                "language": lang.unwrap_or_default(),
                "status": status_str,
                "body": body,
                "lastSyncedAt": synced.unwrap_or_default()
            }));
        }

        Ok(result)
    }

    // ---------------------------------------------------------------------------
    // Sync customer phone numbers from Meta
    // ---------------------------------------------------------------------------

    pub async fn sync_customer_phone_numbers(
        &self,
        brand_id: &str,
    ) -> Result<Vec<String>, String> {
        let creds = self.resolve_credentials(brand_id).await?;
        let url = self.graph_url(&creds, "/contacts");

        let body = self.graph_get(&url, &creds.access_token).await?;

        let mut phones = Vec::new();
        if let Some(data) = body.get("data").and_then(|d| d.as_array()) {
            for entry in data {
                let phone = entry.get("phone").and_then(|v| v.as_str()).unwrap_or("");
                if !phone.is_empty() {
                    phones.push(phone.to_string());
                }
            }
        }

        Ok(phones)
    }
}
