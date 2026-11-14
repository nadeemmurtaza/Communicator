#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use sqlx::{Row, PgPool};

use crate::config::Config;
use crate::services::whatsapp::WhatsAppService;
use crate::services::whatsapp_template_sync::TemplateSyncService;

// ---------------------------------------------------------------------------
// Data structures
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Campaign {
    pub id: String,
    pub brand_id: String,
    pub name: String,
    pub campaign_type: String,
    pub template_flow_id: Option<String>,
    pub status: String,
    pub scheduled_at: Option<String>,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
    pub total_contacts: i64,
    pub sent_count: i64,
    pub delivered_count: i64,
    pub failed_count: i64,
    pub retry_count: i64,
    pub progress: Option<serde_json::Value>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CampaignContact {
    pub id: String,
    pub campaign_id: String,
    pub contact_id: String,
    pub status: String,
    pub message_id: Option<String>,
    pub error: Option<String>,
    pub sent_at: Option<String>,
    pub delivered_at: Option<String>,
    pub read_at: Option<String>,
    pub retry_count: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CampaignAnalytics {
    pub campaign_id: String,
    pub total_contacts: i64,
    pub sent_count: i64,
    pub delivered_count: i64,
    pub read_count: i64,
    pub failed_count: i64,
    pub pending_count: i64,
    pub send_rate: f64,
    pub delivery_rate: f64,
    pub read_rate: f64,
    pub failure_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RetryQueueEntry {
    pub id: String,
    pub campaign_id: String,
    pub contact_id: String,
    pub template_flow_id: Option<String>,
    pub retry_count: i32,
    pub max_retries: i32,
    pub next_retry_at: String,
    pub error: Option<String>,
    pub status: String,
}

// ---------------------------------------------------------------------------
// Campaigns Service
// ---------------------------------------------------------------------------

pub struct WhatsAppCampaignsService {
    pool: PgPool,
    config: Config,
}

impl WhatsAppCampaignsService {
    pub fn new(pool: PgPool, config: Config) -> Self {
        Self { pool, config }
    }

    // ---------------------------------------------------------------------------
    // Campaign CRUD
    // ---------------------------------------------------------------------------

    pub async fn create_campaign(
        &self,
        brand_id: &str,
        name: &str,
        campaign_type: &str,
        template_flow_id: Option<&str>,
        scheduled_at: Option<&str>,
    ) -> Result<Campaign, String> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        let now2 = now.clone();

        sqlx::query(
            "INSERT INTO campaigns (id, brand_id, name, type, template_flow_id, status, scheduled_at, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, 'draft', $6, $7, $8)"
        )
        .bind(&id)
        .bind(brand_id)
        .bind(name)
        .bind(campaign_type)
        .bind(template_flow_id)
        .bind(scheduled_at)
        .bind(&now)
        .bind(&now2)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("DB error: {}", e))?;

        Ok(Campaign {
            id,
            brand_id: brand_id.to_string(),
            name: name.to_string(),
            campaign_type: campaign_type.to_string(),
            template_flow_id: template_flow_id.map(|s| s.to_string()),
            status: "draft".to_string(),
            scheduled_at: scheduled_at.map(|s| s.to_string()),
            started_at: None,
            completed_at: None,
            total_contacts: 0,
            sent_count: 0,
            delivered_count: 0,
            failed_count: 0,
            retry_count: 0,
            progress: None,
            created_at: now,
            updated_at: now2,
        })
    }

    pub async fn get_campaign(&self, campaign_id: &str) -> Result<Campaign, String> {
        let row = sqlx::query(
            "SELECT id, brand_id, name, type, template_flow_id, status, scheduled_at, started_at,
                    completed_at, total_contacts, sent_count, delivered_count, read_count, failed_count, retry_count,
                    progress, metadata, created_by, created_at, updated_at
             FROM campaigns WHERE id = $1 LIMIT 1"
        )
        .bind(campaign_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            if matches!(&e, sqlx::Error::RowNotFound) {
                "Campaign not found".to_string()
            } else {
                format!("DB error: {}", e)
            }
        })?;

        Ok(Campaign {
            id: row.try_get("id").unwrap_or_default(),
            brand_id: row.try_get("brand_id").unwrap_or_default(),
            name: row.try_get("name").unwrap_or_default(),
            campaign_type: row.try_get("type").unwrap_or_default(),
            template_flow_id: row.try_get("template_flow_id").ok().flatten(),
            status: row.try_get("status").unwrap_or_default(),
            scheduled_at: row.try_get("scheduled_at").ok().flatten(),
            started_at: row.try_get("started_at").ok().flatten(),
            completed_at: row.try_get("completed_at").ok().flatten(),
            total_contacts: row.try_get("total_contacts").unwrap_or(0i64),
            sent_count: row.try_get("sent_count").unwrap_or(0i64),
            delivered_count: row.try_get("delivered_count").unwrap_or(0i64),
            failed_count: row.try_get("failed_count").unwrap_or(0i64),
            retry_count: row.try_get("retry_count").unwrap_or(0i64),
            progress: row.try_get("progress").ok().flatten().and_then(|s: String| serde_json::from_str(&s).ok()),
            created_at: row.try_get("created_at").unwrap_or_default(),
            updated_at: row.try_get("updated_at").unwrap_or_default(),
        })
    }

    pub async fn list_campaigns(
        &self,
        brand_id: &str,
        status: Option<&str>,
    ) -> Result<Vec<Campaign>, String> {
        let rows = if let Some(st) = status {
            sqlx::query(
                "SELECT id, brand_id, name, type, template_flow_id, status, scheduled_at, started_at,
                 completed_at, total_contacts, sent_count, delivered_count, read_count, failed_count, retry_count,
                 progress, metadata, created_by, created_at, updated_at
                 FROM campaigns WHERE brand_id = $1 AND status = $2 ORDER BY created_at DESC"
            )
            .bind(brand_id)
            .bind(st)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| format!("DB error: {}", e))?
        } else {
            sqlx::query(
                "SELECT id, brand_id, name, type, template_flow_id, status, scheduled_at, started_at,
                 completed_at, total_contacts, sent_count, delivered_count, read_count, failed_count, retry_count,
                 progress, metadata, created_by, created_at, updated_at
                 FROM campaigns WHERE brand_id = $1 ORDER BY created_at DESC"
            )
            .bind(brand_id)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| format!("DB error: {}", e))?
        };

        let mut result = Vec::new();
        for row in rows {
            result.push(Campaign {
                id: row.try_get("id").unwrap_or_default(),
                brand_id: row.try_get("brand_id").unwrap_or_default(),
                name: row.try_get("name").unwrap_or_default(),
                campaign_type: row.try_get("type").unwrap_or_default(),
                template_flow_id: row.try_get("template_flow_id").ok().flatten(),
                status: row.try_get("status").unwrap_or_default(),
                scheduled_at: row.try_get("scheduled_at").ok().flatten(),
                started_at: row.try_get("started_at").ok().flatten(),
                completed_at: row.try_get("completed_at").ok().flatten(),
                total_contacts: row.try_get("total_contacts").unwrap_or(0i64),
                sent_count: row.try_get("sent_count").unwrap_or(0i64),
                delivered_count: row.try_get("delivered_count").unwrap_or(0i64),
                failed_count: row.try_get("failed_count").unwrap_or(0i64),
                retry_count: row.try_get("retry_count").unwrap_or(0i64),
                progress: row.try_get("progress").ok().flatten().and_then(|s: String| serde_json::from_str(&s).ok()),
                created_at: row.try_get("created_at").unwrap_or_default(),
                updated_at: row.try_get("updated_at").unwrap_or_default(),
            });
        }

        Ok(result)
    }

    pub async fn delete_campaign(&self, campaign_id: &str) -> Result<(), String> {
        sqlx::query("DELETE FROM campaign_retry_queue WHERE campaign_id = $1")
            .bind(campaign_id)
            .execute(&self.pool)
            .await
            .map_err(|e| format!("DB error: {}", e))?;

        sqlx::query("DELETE FROM campaign_contacts WHERE campaign_id = $1")
            .bind(campaign_id)
            .execute(&self.pool)
            .await
            .map_err(|e| format!("DB error: {}", e))?;

        sqlx::query("DELETE FROM campaigns WHERE id = $1")
            .bind(campaign_id)
            .execute(&self.pool)
            .await
            .map_err(|e| format!("DB error: {}", e))?;

        Ok(())
    }

    // ---------------------------------------------------------------------------
    // Contact Selection
    // ---------------------------------------------------------------------------

    pub async fn select_contacts_for_campaign(
        &self,
        campaign_id: &str,
        contact_ids: &[String],
    ) -> Result<i64, String> {
        let now = chrono::Utc::now().to_rfc3339();
        let mut count = 0i64;

        for contact_id in contact_ids {
            let id = uuid::Uuid::new_v4().to_string();
            match sqlx::query(
                "INSERT INTO campaign_contacts (id, campaign_id, contact_id, status, retry_count, created_at)
                 VALUES ($1, $2, $3, 'pending', 0, $4)"
            )
            .bind(&id)
            .bind(campaign_id)
            .bind(contact_id)
            .bind(&now)
            .execute(&self.pool)
            .await
            {
                Ok(_) => count += 1,
                Err(_) => {} // Skip duplicates
            }
        }

        // Update campaign total
        let _ = sqlx::query(
            "UPDATE campaigns SET total_contacts = $1, updated_at = $2 WHERE id = $3"
        )
        .bind(count)
        .bind(&now)
        .bind(campaign_id)
        .execute(&self.pool)
        .await;

        Ok(count)
    }

    pub async fn add_contacts_to_campaign(
        &self,
        campaign_id: &str,
        contact_ids: &[String],
    ) -> Result<i64, String> {
        self.select_contacts_for_campaign(campaign_id, contact_ids).await
    }

    pub async fn remove_contacts_from_campaign(
        &self,
        campaign_id: &str,
        contact_ids: &[String],
    ) -> Result<(), String> {
        for contact_id in contact_ids {
            let _ = sqlx::query(
                "DELETE FROM campaign_contacts WHERE campaign_id = $1 AND contact_id = $2"
            )
            .bind(campaign_id)
            .bind(contact_id)
            .execute(&self.pool)
            .await;
        }

        Ok(())
    }

    pub async fn get_campaign_contacts(
        &self,
        campaign_id: &str,
    ) -> Result<Vec<CampaignContact>, String> {
        let rows: Vec<(
            String, String, String, String, Option<String>, Option<String>, Option<String>,
            Option<String>, Option<String>, i32,
        )> = sqlx::query_as(
            "SELECT id, campaign_id, contact_id, status, message_id, error, sent_at, delivered_at, read_at, retry_count
             FROM campaign_contacts WHERE campaign_id = $1 ORDER BY status, contact_id"
        )
        .bind(campaign_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("DB error: {}", e))?;

        Ok(rows
            .into_iter()
            .map(
                |(id, campaign_id, contact_id, status, message_id, error, sent_at, delivered_at, read_at, retry_count)| {
                    CampaignContact {
                        id,
                        campaign_id,
                        contact_id,
                        status,
                        message_id,
                        error,
                        sent_at,
                        delivered_at,
                        read_at,
                        retry_count,
                    }
                },
            )
            .collect())
    }

    // ---------------------------------------------------------------------------
    // Campaign Execution
    // ---------------------------------------------------------------------------

    pub async fn start_campaign(&self, campaign_id: &str) -> Result<(), String> {
        let now = chrono::Utc::now().to_rfc3339();

        // Check message limits
        let sync_service = TemplateSyncService::new(self.pool.clone(), self.config.clone());
        let can_send = sync_service
            .check_message_limit(&self.get_campaign_brand_id(campaign_id).await?)
            .await
            .unwrap_or(true);

        if !can_send {
            return Err("Message limit reached. Cannot start campaign.".to_string());
        }

        // Update campaign status
        sqlx::query(
            "UPDATE campaigns SET status = 'running', started_at = $1, updated_at = $2 WHERE id = $3"
        )
        .bind(&now)
        .bind(&now)
        .bind(campaign_id)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("DB error: {}", e))?;

        // Get all pending contacts
        let contacts: Vec<(String, String)> = sqlx::query_as(
            "SELECT id, contact_id FROM campaign_contacts WHERE campaign_id = $1 AND status = 'pending' ORDER BY id"
        )
        .bind(campaign_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("DB error: {}", e))?;

        // Get campaign details
        let campaign = self.get_campaign(campaign_id).await?;
        let brand_id = campaign.brand_id;
        let template_flow_id = campaign.template_flow_id;

        let wa_service = WhatsAppService::new(self.pool.clone(), self.config.clone());

        // Send messages to each contact
        for (contact_row_id, contact_id) in &contacts {
            // Get contact's WhatsApp number
            let phone: Option<String> = sqlx::query_scalar(
                "SELECT value FROM contacts WHERE id = $1 AND channel = 'whatsapp' LIMIT 1"
            )
            .bind(contact_id)
            .fetch_optional(&self.pool)
            .await
            .unwrap_or(None);

            let phone = match phone {
                Some(p) => p,
                None => {
                    let _ = sqlx::query(
                        "UPDATE campaign_contacts SET status = 'failed', error = $1 WHERE id = $2"
                    )
                    .bind("No WhatsApp number found")
                    .bind(contact_row_id)
                    .execute(&self.pool)
                    .await;
                    continue;
                }
            };

            // Get flow body or use template
            let (message, msg_type) = if let Some(ref flow_id) = template_flow_id {
                // Get flow body
                let body: Option<String> = sqlx::query_scalar(
                    "SELECT body FROM whatsapp_template_flows WHERE id = $1 LIMIT 1"
                )
                .bind(flow_id)
                .fetch_optional(&self.pool)
                .await
                .unwrap_or(None);

                match body {
                    Some(b) => (b, "flow".to_string()),
                    None => {
                        let _ = sqlx::query(
                            "UPDATE campaign_contacts SET status = 'failed', error = $1 WHERE id = $2"
                        )
                        .bind("Template flow not found")
                        .bind(contact_row_id)
                        .execute(&self.pool)
                        .await;
                        continue;
                    }
                }
            } else {
                (
                    "Hello from our team!".to_string(),
                    "text".to_string(),
                )
            };

            // Send the message
            match wa_service
                .send_text(&brand_id, &phone, &message, false)
                .await
            {
                Ok(result) => {
                    let sent_at = chrono::Utc::now().to_rfc3339();
                    let _ = sqlx::query(
                        "UPDATE campaign_contacts SET status = 'sent', message_id = $1, sent_at = $2 WHERE id = $3"
                    )
                    .bind(&result.provider_message_id)
                    .bind(&sent_at)
                    .bind(contact_row_id)
                    .execute(&self.pool)
                    .await;

                    // Update campaign sent count
                    let _ = sqlx::query(
                        "UPDATE campaigns SET sent_count = sent_count + 1, updated_at = $1 WHERE id = $2"
                    )
                    .bind(&sent_at)
                    .bind(campaign_id)
                    .execute(&self.pool)
                    .await;

                    // Log to template_log
                    let log_id = uuid::Uuid::new_v4().to_string();
                    let _ = sqlx::query(
                        "INSERT INTO whatsapp_template_log (id, brand_id, contact_id, template_name, status, message_id, created_at)
                         VALUES ($1, $2, $3, $4, 'sent', $5, $6)"
                    )
                    .bind(&log_id)
                    .bind(&brand_id)
                    .bind(contact_id)
                    .bind(&msg_type)
                    .bind(&result.provider_message_id)
                    .bind(&sent_at)
                    .execute(&self.pool)
                    .await;
                }
                Err(e) => {
                    // Add to retry queue
                    let retry_id = uuid::Uuid::new_v4().to_string();
                    let next_retry = (chrono::Utc::now() + chrono::Duration::minutes(5))
                        .to_rfc3339();

                    let _ = sqlx::query(
                        "INSERT INTO campaign_retry_queue (id, campaign_id, contact_id, template_flow_id, retry_count, max_retries, next_retry_at, error, status)
                         VALUES ($1, $2, $3, $4, 0, 3, $5, $6, 'pending')"
                    )
                    .bind(&retry_id)
                    .bind(campaign_id)
                    .bind(contact_id)
                    .bind(&template_flow_id)
                    .bind(&next_retry)
                    .bind(&e)
                    .execute(&self.pool)
                    .await;

                    let _ = sqlx::query(
                        "UPDATE campaign_contacts SET status = 'failed', error = $1 WHERE id = $2"
                    )
                    .bind(&e)
                    .bind(contact_row_id)
                    .execute(&self.pool)
                    .await;
                }
            }

            // Small delay between messages to avoid rate limits
            tokio::time::sleep(std::time::Duration::from_millis(200)).await;
        }

        // Check if all contacts processed
        let pending: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM campaign_contacts WHERE campaign_id = $1 AND status IN ('pending', 'failed')"
        )
        .bind(campaign_id)
        .fetch_one(&self.pool)
        .await
        .unwrap_or(0);

        if pending == 0 {
            let completed_at = chrono::Utc::now().to_rfc3339();
            let _ = sqlx::query(
                "UPDATE campaigns SET status = 'completed', completed_at = $1, updated_at = $2 WHERE id = $3"
            )
            .bind(&completed_at)
            .bind(&completed_at)
            .bind(campaign_id)
            .execute(&self.pool)
            .await;
        }

        Ok(())
    }

    pub async fn pause_campaign(&self, campaign_id: &str) -> Result<(), String> {
        let now = chrono::Utc::now().to_rfc3339();
        sqlx::query(
            "UPDATE campaigns SET status = 'paused', updated_at = $1 WHERE id = $2"
        )
        .bind(&now)
        .bind(campaign_id)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("DB error: {}", e))?;
        Ok(())
    }

    pub async fn resume_campaign(&self, campaign_id: &str) -> Result<(), String> {
        let now = chrono::Utc::now().to_rfc3339();
        sqlx::query(
            "UPDATE campaigns SET status = 'running', updated_at = $1 WHERE id = $2"
        )
        .bind(&now)
        .bind(campaign_id)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("DB error: {}", e))?;

        // Restart the campaign sending
        self.start_campaign(campaign_id).await
    }

    pub async fn cancel_campaign(&self, campaign_id: &str) -> Result<(), String> {
        let now = chrono::Utc::now().to_rfc3339();
        sqlx::query(
            "UPDATE campaigns SET status = 'cancelled', completed_at = $1, updated_at = $2 WHERE id = $3"
        )
        .bind(&now)
        .bind(&now)
        .bind(campaign_id)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("DB error: {}", e))?;

        // Clear retry queue
        let _ = sqlx::query(
            "DELETE FROM campaign_retry_queue WHERE campaign_id = $1"
        )
        .bind(campaign_id)
        .execute(&self.pool)
        .await;

        Ok(())
    }

    // ---------------------------------------------------------------------------
    // Retry Queue
    // ---------------------------------------------------------------------------

    pub async fn retry_failed_messages(&self, campaign_id: &str) -> Result<i64, String> {
        let now = chrono::Utc::now().to_rfc3339();

        // Get pending retries
        let retries: Vec<(String, String, Option<String>, String, i32)> = sqlx::query_as(
            "SELECT id, contact_id, template_flow_id, campaign_id, retry_count
             FROM campaign_retry_queue
             WHERE campaign_id = $1 AND status = 'pending' AND next_retry_at <= $2
             ORDER BY next_retry_at"
        )
        .bind(campaign_id)
        .bind(&now)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("DB error: {}", e))?;

        let mut success_count = 0i64;
        let wa_service = WhatsAppService::new(self.pool.clone(), self.config.clone());

        // Get brand_id
        let brand_id = self.get_campaign_brand_id(campaign_id).await?;

        for (retry_id, contact_id, template_flow_id, _campaign_id, retry_count) in &retries {
            // Get contact's WhatsApp number
            let phone: Option<String> = sqlx::query_scalar(
                "SELECT value FROM contacts WHERE id = $1 AND channel = 'whatsapp' LIMIT 1"
            )
            .bind(contact_id.as_str())
            .fetch_optional(&self.pool)
            .await
            .unwrap_or(None);

            let phone = match phone {
                Some(p) => p,
                None => {
                    let _ = sqlx::query(
                        "UPDATE campaign_retry_queue SET status = 'failed', error = $1 WHERE id = $2"
                    )
                    .bind("No WhatsApp number found")
                    .bind(retry_id)
                    .execute(&self.pool)
                    .await;
                    continue;
                }
            };

            // Get message body
            let message = if let Some(ref flow_id) = template_flow_id {
                let body: Option<String> = sqlx::query_scalar(
                    "SELECT body FROM whatsapp_template_flows WHERE id = $1 LIMIT 1"
                )
                .bind(flow_id)
                .fetch_optional(&self.pool)
                .await
                .unwrap_or(None);

                body.unwrap_or_else(|| "Hello from our team!".to_string())
            } else {
                "Hello from our team!".to_string()
            };

            match wa_service
                .send_text(&brand_id, &phone, &message, false)
                .await
            {
                Ok(result) => {
                    let sent_at = chrono::Utc::now().to_rfc3339();
                    let _ = sqlx::query(
                        "UPDATE campaign_retry_queue SET status = 'sent', message_id = $1, sent_at = $2, error = NULL WHERE id = $3"
                    )
                    .bind(&result.provider_message_id)
                    .bind(&sent_at)
                    .bind(retry_id)
                    .execute(&self.pool)
                    .await;

                    // Update campaign contact
                    let _ = sqlx::query(
                        "UPDATE campaign_contacts SET status = 'sent', message_id = $1, sent_at = $2, error = NULL
                         WHERE campaign_id = $3 AND contact_id = $4"
                    )
                    .bind(&result.provider_message_id)
                    .bind(&sent_at)
                    .bind(campaign_id)
                    .bind(contact_id)
                    .execute(&self.pool)
                    .await;

                    success_count += 1;
                }
                Err(e) => {
                    let new_retry_count = retry_count + 1;
                    if new_retry_count >= 3 {
                        let _ = sqlx::query(
                            "UPDATE campaign_retry_queue SET status = 'failed', error = $1 WHERE id = $2"
                        )
                        .bind(&e)
                        .bind(retry_id)
                        .execute(&self.pool)
                        .await;
                    } else {
                        let next_retry =
                            (chrono::Utc::now() + chrono::Duration::minutes(5 * (new_retry_count as i64)))
                                .to_rfc3339();
                        let _ = sqlx::query(
                            "UPDATE campaign_retry_queue SET retry_count = $1, next_retry_at = $2, error = $3 WHERE id = $4"
                        )
                        .bind(new_retry_count)
                        .bind(&next_retry)
                        .bind(&e)
                        .bind(retry_id)
                        .execute(&self.pool)
                        .await;
                    }
                }
            }

            tokio::time::sleep(std::time::Duration::from_millis(200)).await;
        }

        // Update campaign retry count
        let _ = sqlx::query(
            "UPDATE campaigns SET retry_count = retry_count + $1, updated_at = $2 WHERE id = $3"
        )
        .bind(success_count)
        .bind(&now)
        .bind(campaign_id)
        .execute(&self.pool)
        .await;

        Ok(success_count)
    }

    pub async fn process_retry_queue(&self) -> Result<i64, String> {
        let now = chrono::Utc::now().to_rfc3339();

        // Get all campaigns with pending retries
        let campaigns: Vec<String> = sqlx::query_scalar(
            "SELECT DISTINCT campaign_id FROM campaign_retry_queue WHERE status = 'pending' AND next_retry_at <= $1"
        )
        .bind(&now)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("DB error: {}", e))?;

        let mut total_success = 0i64;
        for campaign_id in &campaigns {
            let success = self.retry_failed_messages(campaign_id).await.unwrap_or(0);
            total_success += success;
        }

        Ok(total_success)
    }

    // ---------------------------------------------------------------------------
    // Analytics
    // ---------------------------------------------------------------------------

    pub async fn get_campaign_analytics(
        &self,
        campaign_id: &str,
    ) -> Result<CampaignAnalytics, String> {
        let counts: (i64, i64, i64, i64, i64) = sqlx::query_as(
            r#"
            SELECT
                COUNT(*) as total,
                COALESCE(SUM(CASE WHEN status = 'sent' THEN 1 ELSE 0 END), 0),
                COALESCE(SUM(CASE WHEN status = 'delivered' THEN 1 ELSE 0 END), 0),
                COALESCE(SUM(CASE WHEN status = 'read' THEN 1 ELSE 0 END), 0),
                COALESCE(SUM(CASE WHEN status = 'failed' THEN 1 ELSE 0 END), 0)
            FROM campaign_contacts WHERE campaign_id = $1
            "#
        )
        .bind(campaign_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("DB error: {}", e))?;

        let total = counts.0.max(1) as f64;

        Ok(CampaignAnalytics {
            campaign_id: campaign_id.to_string(),
            total_contacts: counts.0,
            sent_count: counts.1,
            delivered_count: counts.2,
            read_count: counts.3,
            failed_count: counts.4,
            pending_count: counts.0 - counts.1 - counts.4,
            send_rate: (counts.1 as f64 / total) * 100.0,
            delivery_rate: (counts.2 as f64 / total) * 100.0,
            read_rate: (counts.3 as f64 / total) * 100.0,
            failure_rate: (counts.4 as f64 / total) * 100.0,
        })
    }

    // ---------------------------------------------------------------------------
    // Helpers
    // ---------------------------------------------------------------------------

    async fn get_campaign_brand_id(&self, campaign_id: &str) -> Result<String, String> {
        let brand_id: Option<String> = sqlx::query_scalar(
            "SELECT brand_id FROM campaigns WHERE id = $1 LIMIT 1"
        )
        .bind(campaign_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("DB error: {}", e))?;

        match brand_id {
            Some(id) => Ok(id),
            None => Err("Campaign not found".to_string()),
        }
    }
}
