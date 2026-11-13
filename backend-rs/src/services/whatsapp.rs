#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::time::Duration;

use crate::config::Config;

// ---------------------------------------------------------------------------
// Result types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SendResult {
    pub provider: String,
    pub provider_message_id: String,
    pub raw_response: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WhatsAppCredentials {
    pub access_token: String,
    pub phone_number_id: String,
    pub api_version: Option<String>,
}

// ---------------------------------------------------------------------------
// WhatsApp Service
// ---------------------------------------------------------------------------

pub struct WhatsAppService {
    pool: PgPool,
    config: Config,
    client: reqwest::Client,
}

impl WhatsAppService {
    pub fn new(pool: PgPool, config: Config) -> Self {
        Self {
            pool,
            config,
            client: reqwest::Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .unwrap_or_default(),
        }
    }

    /// Resolve WhatsApp credentials for a brand, falling back to global config
    async fn resolve_credentials(&self, brand_id: &str) -> Result<WhatsAppCredentials, String> {
        // Try brand_credentials table first
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

        // Fall back to integration_settings
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

        // Fall back to env vars
        if !self.config.meta_whatsapp_access_token.is_empty() && !self.config.meta_whatsapp_phone_number_id.is_empty() {
            return Ok(WhatsAppCredentials {
                access_token: self.config.meta_whatsapp_access_token.clone(),
                phone_number_id: self.config.meta_whatsapp_phone_number_id.clone(),
                api_version: if self.config.meta_whatsapp_api_version.is_empty() { None } else { Some(self.config.meta_whatsapp_api_version.clone()) },
            });
        }

        Err("No WhatsApp credentials found for this brand".to_string())
    }

    fn messages_url(&self, creds: &WhatsAppCredentials) -> String {
        let version = creds.api_version.as_deref().unwrap_or("v18.0");
        format!(
            "https://graph.facebook.com/{}//{}/messages",
            version, creds.phone_number_id
        )
    }

    async fn post_with_retry(
        &self,
        url: &str,
        payload: &serde_json::Value,
        token: &str,
    ) -> Result<serde_json::Value, String> {
        let mut delay = Duration::from_millis(500);
        let max_retries = 3;

        for attempt in 0..=max_retries {
            match self
                .client
                .post(url)
                .header("Authorization", format!("Bearer {}", token))
                .header("Content-Type", "application/json")
                .json(payload)
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

                    // Parse error
                    let err_json: serde_json::Value =
                        serde_json::from_str(&body).unwrap_or_else(|_| serde_json::json!({}));
                    let err_msg = err_json["error"]["message"]
                        .as_str()
                        .unwrap_or(&body);

                    // Retry on 429 or 5xx
                    if (status == 429 || status.is_server_error()) && attempt < max_retries {
                        tokio::time::sleep(delay).await;
                        delay *= 2;
                        continue;
                    }

                    return Err(format!(
                        "Meta API error {}: {}",
                        status, err_msg
                    ));
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

    fn extract_message_id(response: &serde_json::Value) -> String {
        response["messages"][0]["id"]
            .as_str()
            .unwrap_or("")
            .to_string()
    }

    // ---------------------------------------------------------------------------
    // Text messages
    // ---------------------------------------------------------------------------

    pub async fn send_text(
        &self,
        brand_id: &str,
        to: &str,
        body: &str,
        preview_url: bool,
    ) -> Result<SendResult, String> {
        let creds = self.resolve_credentials(brand_id).await?;
        let payload = serde_json::json!({
            "messaging_product": "whatsapp",
            "recipient_type": "individual",
            "to": to.trim_start_matches('+'),
            "type": "text",
            "text": {
                "preview_url": preview_url,
                "body": body.chars().take(4096).collect::<String>()
            }
        });
        let response = self
            .post_with_retry(&self.messages_url(&creds), &payload, &creds.access_token)
            .await?;
        Ok(SendResult {
            provider: "meta".to_string(),
            provider_message_id: Self::extract_message_id(&response),
            raw_response: response,
        })
    }

    // ---------------------------------------------------------------------------
    // Interactive buttons (up to 3)
    // ---------------------------------------------------------------------------

    pub async fn send_interactive_buttons(
        &self,
        brand_id: &str,
        to: &str,
        body_text: &str,
        buttons: &[serde_json::Value],
        header_text: Option<&str>,
        footer_text: Option<&str>,
    ) -> Result<SendResult, String> {
        let creds = self.resolve_credentials(brand_id).await?;

        let formatted_buttons: Vec<serde_json::Value> = buttons
            .iter()
            .take(3)
            .enumerate()
            .map(|(i, btn)| {
                let id = btn["id"].as_str().unwrap_or("btn").to_string();
                let id_final = if id == "btn" { format!("btn_{}", i) } else { id };
                let title = btn["title"].as_str().unwrap_or("").to_string();
                serde_json::json!({
                    "type": "reply",
                    "reply": {
                        "id": id_final.chars().take(256).collect::<String>(),
                        "title": title.chars().take(20).collect::<String>()
                    }
                })
            })
            .collect();

        let mut interactive = serde_json::json!({
            "type": "button",
            "body": { "text": body_text.chars().take(1024).collect::<String>() },
            "action": { "buttons": formatted_buttons }
        });

        if let Some(header) = header_text {
            interactive["header"] = serde_json::json!({
                "type": "text",
                "text": header.chars().take(60).collect::<String>()
            });
        }
        if let Some(footer) = footer_text {
            interactive["footer"] = serde_json::json!({
                "text": footer.chars().take(60).collect::<String>()
            });
        }

        let payload = serde_json::json!({
            "messaging_product": "whatsapp",
            "recipient_type": "individual",
            "to": to.trim_start_matches('+'),
            "type": "interactive",
            "interactive": interactive
        });

        let response = self
            .post_with_retry(&self.messages_url(&creds), &payload, &creds.access_token)
            .await?;
        Ok(SendResult {
            provider: "meta".to_string(),
            provider_message_id: Self::extract_message_id(&response),
            raw_response: response,
        })
    }

    // ---------------------------------------------------------------------------
    // Interactive list (up to 10 rows)
    // ---------------------------------------------------------------------------

    pub async fn send_interactive_list(
        &self,
        brand_id: &str,
        to: &str,
        body_text: &str,
        button_title: &str,
        sections: &[serde_json::Value],
        header_text: Option<&str>,
        footer_text: Option<&str>,
    ) -> Result<SendResult, String> {
        let creds = self.resolve_credentials(brand_id).await?;

        let mut formatted_sections = Vec::new();
        let mut total_rows = 0;

        for sec in sections {
            let mut rows = Vec::new();
            if let Some(sec_rows) = sec["rows"].as_array() {
                for r in sec_rows {
                    if total_rows >= 10 {
                        break;
                    }
                    let mut row = serde_json::json!({
                        "id": r["id"].as_str().unwrap_or("").chars().take(200).collect::<String>(),
                        "title": r["title"].as_str().unwrap_or("").chars().take(24).collect::<String>()
                    });
                    if let Some(desc) = r["description"].as_str() {
                        row["description"] =
                            serde_json::json!(desc.chars().take(72).collect::<String>());
                    }
                    rows.push(row);
                    total_rows += 1;
                }
            }
            if !rows.is_empty() {
                formatted_sections.push(serde_json::json!({
                    "title": sec["title"].as_str().unwrap_or("").chars().take(24).collect::<String>(),
                    "rows": rows
                }));
            }
        }

        let mut interactive = serde_json::json!({
            "type": "list",
            "body": { "text": body_text.chars().take(1024).collect::<String>() },
            "action": {
                "button": button_title.chars().take(20).collect::<String>(),
                "sections": formatted_sections
            }
        });

        if let Some(header) = header_text {
            interactive["header"] = serde_json::json!({
                "type": "text",
                "text": header.chars().take(60).collect::<String>()
            });
        }
        if let Some(footer) = footer_text {
            interactive["footer"] = serde_json::json!({
                "text": footer.chars().take(60).collect::<String>()
            });
        }

        let payload = serde_json::json!({
            "messaging_product": "whatsapp",
            "recipient_type": "individual",
            "to": to.trim_start_matches('+'),
            "type": "interactive",
            "interactive": interactive
        });

        let response = self
            .post_with_retry(&self.messages_url(&creds), &payload, &creds.access_token)
            .await?;
        Ok(SendResult {
            provider: "meta".to_string(),
            provider_message_id: Self::extract_message_id(&response),
            raw_response: response,
        })
    }

    // ---------------------------------------------------------------------------
    // CTA URL
    // ---------------------------------------------------------------------------

    pub async fn send_interactive_cta_url(
        &self,
        brand_id: &str,
        to: &str,
        body_text: &str,
        display_text: &str,
        url: &str,
        header_text: Option<&str>,
        footer_text: Option<&str>,
    ) -> Result<SendResult, String> {
        let creds = self.resolve_credentials(brand_id).await?;

        let mut interactive = serde_json::json!({
            "type": "cta_url",
            "body": { "text": body_text.chars().take(1024).collect::<String>() },
            "action": {
                "name": "cta_url",
                "parameters": {
                    "display_text": display_text.chars().take(20).collect::<String>(),
                    "url": url
                }
            }
        });

        if let Some(header) = header_text {
            interactive["header"] = serde_json::json!({
                "type": "text",
                "text": header.chars().take(60).collect::<String>()
            });
        }
        if let Some(footer) = footer_text {
            interactive["footer"] = serde_json::json!({
                "text": footer.chars().take(60).collect::<String>()
            });
        }

        let payload = serde_json::json!({
            "messaging_product": "whatsapp",
            "recipient_type": "individual",
            "to": to.trim_start_matches('+'),
            "type": "interactive",
            "interactive": interactive
        });

        let response = self
            .post_with_retry(&self.messages_url(&creds), &payload, &creds.access_token)
            .await?;
        Ok(SendResult {
            provider: "meta".to_string(),
            provider_message_id: Self::extract_message_id(&response),
            raw_response: response,
        })
    }

    // ---------------------------------------------------------------------------
    // Location request
    // ---------------------------------------------------------------------------

    pub async fn send_location_request(
        &self,
        brand_id: &str,
        to: &str,
        body_text: &str,
    ) -> Result<SendResult, String> {
        let creds = self.resolve_credentials(brand_id).await?;
        let payload = serde_json::json!({
            "messaging_product": "whatsapp",
            "recipient_type": "individual",
            "to": to.trim_start_matches('+'),
            "type": "interactive",
            "interactive": {
                "type": "location_request_message",
                "body": { "text": body_text.chars().take(1024).collect::<String>() },
                "action": { "name": "send_location" }
            }
        });
        let response = self
            .post_with_retry(&self.messages_url(&creds), &payload, &creds.access_token)
            .await?;
        Ok(SendResult {
            provider: "meta".to_string(),
            provider_message_id: Self::extract_message_id(&response),
            raw_response: response,
        })
    }

    // ---------------------------------------------------------------------------
    // Catalog message
    // ---------------------------------------------------------------------------

    pub async fn send_catalog_message(
        &self,
        brand_id: &str,
        to: &str,
        body_text: &str,
        catalog_id: &str,
        thumbnail_product_retailer_id: Option<&str>,
        footer_text: Option<&str>,
    ) -> Result<SendResult, String> {
        let creds = self.resolve_credentials(brand_id).await?;

        let mut action = serde_json::json!({ "name": "catalog_message" });
        if let Some(thumb) = thumbnail_product_retailer_id {
            action["parameters"] = serde_json::json!({ "thumbnail_product_retailer_id": thumb });
        }

        let mut interactive = serde_json::json!({
            "type": "catalog_message",
            "body": { "text": body_text.chars().take(1024).collect::<String>() },
            "action": action
        });

        if let Some(footer) = footer_text {
            interactive["footer"] = serde_json::json!({
                "text": footer.chars().take(60).collect::<String>()
            });
        }

        let payload = serde_json::json!({
            "messaging_product": "whatsapp",
            "recipient_type": "individual",
            "to": to.trim_start_matches('+'),
            "type": "interactive",
            "interactive": interactive
        });

        let response = self
            .post_with_retry(&self.messages_url(&creds), &payload, &creds.access_token)
            .await?;
        Ok(SendResult {
            provider: "meta".to_string(),
            provider_message_id: Self::extract_message_id(&response),
            raw_response: response,
        })
    }

    // ---------------------------------------------------------------------------
    // Product message
    // ---------------------------------------------------------------------------

    pub async fn send_product_message(
        &self,
        brand_id: &str,
        to: &str,
        catalog_id: &str,
        product_retailer_id: &str,
        body_text: Option<&str>,
        footer_text: Option<&str>,
    ) -> Result<SendResult, String> {
        let creds = self.resolve_credentials(brand_id).await?;

        let mut interactive = serde_json::json!({
            "type": "product",
            "body": { "text": body_text.unwrap_or("").chars().take(1024).collect::<String>() },
            "action": {
                "name": "product",
                "parameters": {
                    "product_retailer_id": product_retailer_id,
                    "catalog_id": catalog_id
                }
            }
        });

        if let Some(footer) = footer_text {
            interactive["footer"] = serde_json::json!({
                "text": footer.chars().take(60).collect::<String>()
            });
        }

        let payload = serde_json::json!({
            "messaging_product": "whatsapp",
            "recipient_type": "individual",
            "to": to.trim_start_matches('+'),
            "type": "interactive",
            "interactive": interactive
        });

        let response = self
            .post_with_retry(&self.messages_url(&creds), &payload, &creds.access_token)
            .await?;
        Ok(SendResult {
            provider: "meta".to_string(),
            provider_message_id: Self::extract_message_id(&response),
            raw_response: response,
        })
    }

    // ---------------------------------------------------------------------------
    // Media messages
    // ---------------------------------------------------------------------------

    pub async fn send_media(
        &self,
        brand_id: &str,
        to: &str,
        media_type: &str,
        link: &str,
        caption: Option<&str>,
        filename: Option<&str>,
    ) -> Result<SendResult, String> {
        let creds = self.resolve_credentials(brand_id).await?;

        let mut media_obj = serde_json::json!({
            "link": link
        });
        if let Some(fname) = filename {
            media_obj["filename"] = serde_json::json!(fname);
        }

        let mut message = serde_json::json!({
            "messaging_product": "whatsapp",
            "recipient_type": "individual",
            "to": to.trim_start_matches('+'),
            "type": media_type
        });

        message[media_type] = serde_json::json!({
            "link": link,
            "caption": caption.unwrap_or(""),
            "filename": filename.unwrap_or("")
        });

        let response = self
            .post_with_retry(&self.messages_url(&creds), &message, &creds.access_token)
            .await?;
        Ok(SendResult {
            provider: "meta".to_string(),
            provider_message_id: Self::extract_message_id(&response),
            raw_response: response,
        })
    }

    // ---------------------------------------------------------------------------
    // Location
    // ---------------------------------------------------------------------------

    pub async fn send_location(
        &self,
        brand_id: &str,
        to: &str,
        latitude: f64,
        longitude: f64,
        name: Option<&str>,
        address: Option<&str>,
    ) -> Result<SendResult, String> {
        let creds = self.resolve_credentials(brand_id).await?;

        let mut location = serde_json::json!({
            "latitude": latitude,
            "longitude": longitude
        });
        if let Some(n) = name {
            location["name"] = serde_json::json!(n);
        }
        if let Some(a) = address {
            location["address"] = serde_json::json!(a);
        }

        let payload = serde_json::json!({
            "messaging_product": "whatsapp",
            "recipient_type": "individual",
            "to": to.trim_start_matches('+'),
            "type": "location",
            "location": location
        });

        let response = self
            .post_with_retry(&self.messages_url(&creds), &payload, &creds.access_token)
            .await?;
        Ok(SendResult {
            provider: "meta".to_string(),
            provider_message_id: Self::extract_message_id(&response),
            raw_response: response,
        })
    }

    // ---------------------------------------------------------------------------
    // Reaction
    // ---------------------------------------------------------------------------

    pub async fn send_reaction(
        &self,
        brand_id: &str,
        to: &str,
        message_id: &str,
        emoji: &str,
    ) -> Result<SendResult, String> {
        let creds = self.resolve_credentials(brand_id).await?;
        let payload = serde_json::json!({
            "messaging_product": "whatsapp",
            "recipient_type": "individual",
            "to": to.trim_start_matches('+'),
            "type": "reaction",
            "reaction": {
                "message_id": message_id,
                "emoji": emoji
            }
        });
        let response = self
            .post_with_retry(&self.messages_url(&creds), &payload, &creds.access_token)
            .await?;
        Ok(SendResult {
            provider: "meta".to_string(),
            provider_message_id: Self::extract_message_id(&response),
            raw_response: response,
        })
    }

    // ---------------------------------------------------------------------------
    // Mark as read
    // ---------------------------------------------------------------------------

    pub async fn mark_as_read(
        &self,
        brand_id: &str,
        message_id: &str,
    ) -> Result<serde_json::Value, String> {
        let creds = self.resolve_credentials(brand_id).await?;
        let payload = serde_json::json!({
            "messaging_product": "whatsapp",
            "status": "read",
            "message_id": message_id
        });
        let version = creds.api_version.as_deref().unwrap_or("v18.0");
        let url = format!(
            "https://graph.facebook.com/{}/{}",
            version, creds.phone_number_id
        );
        self.post_with_retry(&url, &payload, &creds.access_token)
            .await
    }

    // ---------------------------------------------------------------------------
    // Template messages
    // ---------------------------------------------------------------------------

    pub async fn send_template(
        &self,
        brand_id: &str,
        to: &str,
        template_name: &str,
        language: &str,
        components: Option<&serde_json::Value>,
    ) -> Result<SendResult, String> {
        let creds = self.resolve_credentials(brand_id).await?;

        let mut interactive = serde_json::json!({
            "type": "template",
            "template": {
                "name": template_name,
                "language": { "code": language }
            }
        });

        if let Some(comp) = components {
            interactive["template"]["components"] = comp.clone();
        }

        let payload = serde_json::json!({
            "messaging_product": "whatsapp",
            "recipient_type": "individual",
            "to": to.trim_start_matches('+'),
            "type": "template",
            "template": interactive["template"].clone()
        });

        let response = self
            .post_with_retry(&self.messages_url(&creds), &payload, &creds.access_token)
            .await?;

        // Log template send
        let msg_id = Self::extract_message_id(&response);
        let log_id = uuid::Uuid::new_v4().to_string();
        let _ = sqlx::query(
            "INSERT OR IGNORE INTO whatsapp_template_log (id, brand_id, template_name, message_id, status, created_at) VALUES (?1, ?2, ?3, ?4, 'sent', NOW()::TEXT)"
        )
        .bind(&log_id)
        .bind(brand_id)
        .bind(template_name)
        .bind(&msg_id)
        .execute(&self.pool)
        .await;

        Ok(SendResult {
            provider: "meta".to_string(),
            provider_message_id: msg_id,
            raw_response: response,
        })
    }

    // ---------------------------------------------------------------------------
    // Session window management
    // ---------------------------------------------------------------------------

    pub async fn update_session_window(
        &self,
        brand_id: &str,
        contact_id: &str,
    ) -> Result<(), String> {
        let now = chrono::Utc::now();
        let expires = now + chrono::Duration::hours(24);
        let started = now - chrono::Duration::seconds(0);

        let _ = sqlx::query(
            "INSERT INTO whatsapp_session_windows (id, brand_id, contact_id, started_at, expires_at, is_active, message_count, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, 1, 1, ?6)"
        )
        .bind(uuid::Uuid::new_v4().to_string())
        .bind(brand_id)
        .bind(contact_id)
        .bind(started.to_rfc3339())
        .bind(expires.to_rfc3339())
        .bind(now.to_rfc3339())
        .execute(&self.pool)
        .await
        .map_err(|e| format!("DB error: {}", e))?;

        // Deactivate old sessions
        let _ = sqlx::query(
            "UPDATE whatsapp_session_windows SET is_active = 0 WHERE brand_id = ?1 AND contact_id = ?2 AND id != ?3"
        )
        .bind(brand_id)
        .bind(contact_id)
        .bind(uuid::Uuid::new_v4().to_string())
        .execute(&self.pool)
        .await;

        Ok(())
    }

    pub async fn check_session_active(
        &self,
        brand_id: &str,
        contact_id: &str,
    ) -> Result<bool, String> {
        let now = chrono::Utc::now().to_rfc3339();
        let row: Option<(i64,)> = sqlx::query_as(
            "SELECT is_active FROM whatsapp_session_windows WHERE brand_id = ?1 AND contact_id = ?2 AND is_active = 1 AND expires_at > ?3 LIMIT 1"
        )
        .bind(brand_id)
        .bind(contact_id)
        .bind(&now)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("DB error: {}", e))?;

        Ok(row.is_some())
    }

    // ---------------------------------------------------------------------------
    // Automation state
    // ---------------------------------------------------------------------------

    pub async fn get_automation_state(
        &self,
        brand_id: &str,
        conversation_id: &str,
    ) -> Result<Option<(String, bool, bool, Option<String>, Option<String>, Option<String>)>, String> {
        let row: Option<(String, bool, bool, Option<String>, Option<String>, Option<String>)> =
            sqlx::query_as(
                "SELECT id, greeting_sent, csat_sent, last_menu, current_flow, metadata FROM whatsapp_automation_state WHERE brand_id = ?1 AND conversation_id = ?2 LIMIT 1"
            )
            .bind(brand_id)
            .bind(conversation_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| format!("DB error: {}", e))?;
        Ok(row)
    }

    pub async fn upsert_automation_state(
        &self,
        brand_id: &str,
        conversation_id: &str,
        greeting_sent: bool,
        csat_sent: bool,
        last_menu: Option<&str>,
        current_flow: Option<&str>,
        metadata: Option<&str>,
    ) -> Result<(), String> {
        let _ = sqlx::query(
            "INSERT INTO whatsapp_automation_state (id, brand_id, conversation_id, greeting_sent, csat_sent, last_menu, current_flow, metadata, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, NOW()::TEXT, NOW()::TEXT)
             ON CONFLICT(brand_id, conversation_id) DO UPDATE SET
             greeting_sent = ?4, csat_sent = ?5, last_menu = ?6, current_flow = ?7, metadata = ?8, updated_at = NOW()::TEXT"
        )
        .bind(uuid::Uuid::new_v4().to_string())
        .bind(brand_id)
        .bind(conversation_id)
        .bind(greeting_sent)
        .bind(csat_sent)
        .bind(last_menu)
        .bind(current_flow)
        .bind(metadata)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("DB error: {}", e))?;
        Ok(())
    }
}
