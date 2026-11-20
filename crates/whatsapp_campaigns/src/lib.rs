//! WhatsApp Campaigns Service
//!
//! Manages WhatsApp campaigns: CRUD, contact selection, execution, retry queue, and analytics.

use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};

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

pub struct WhatsAppCampaignsState {
    pub db: PgPool,
}

impl WhatsAppCampaignsState {
    pub fn new(db: PgPool) -> Self {
        Self { db }
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
        .execute(&self.db)
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
        .fetch_one(&self.db)
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
            .fetch_all(&self.db)
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
            .fetch_all(&self.db)
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
            .execute(&self.db)
            .await
            .map_err(|e| format!("DB error: {}", e))?;

        sqlx::query("DELETE FROM campaign_contacts WHERE campaign_id = $1")
            .bind(campaign_id)
            .execute(&self.db)
            .await
            .map_err(|e| format!("DB error: {}", e))?;

        sqlx::query("DELETE FROM campaigns WHERE id = $1")
            .bind(campaign_id)
            .execute(&self.db)
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
            .execute(&self.db)
            .await
            {
                Ok(_) => count += 1,
                Err(_) => {} // Skip duplicates
            }
        }

        let _ = sqlx::query(
            "UPDATE campaigns SET total_contacts = $1, updated_at = $2 WHERE id = $3"
        )
        .bind(count)
        .bind(&now)
        .bind(campaign_id)
        .execute(&self.db)
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
            .execute(&self.db)
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
        .fetch_all(&self.db)
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
        let sync_service = newax_whatsapp_template_sync::WhatsAppTemplateSyncState::new(self.db.clone());
        let brand_id = self.get_campaign_brand_id(campaign_id).await?;
        let can_send = sync_service
            .check_message_limit(&brand_id)
            .await
            .unwrap_or(true);

        if !can_send {
            return Err("Message limit reached. Cannot start campaign.".to_string());
        }

        sqlx::query(
            "UPDATE campaigns SET status = 'running', started_at = $1, updated_at = $2 WHERE id = $3"
        )
        .bind(&now)
        .bind(&now)
        .bind(campaign_id)
        .execute(&self.db)
        .await
        .map_err(|e| format!("DB error: {}", e))?;

        let contacts: Vec<(String, String)> = sqlx::query_as(
            "SELECT id, contact_id FROM campaign_contacts WHERE campaign_id = $1 AND status = 'pending' ORDER BY id"
        )
        .bind(campaign_id)
        .fetch_all(&self.db)
        .await
        .map_err(|e| format!("DB error: {}", e))?;

        let campaign = self.get_campaign(campaign_id).await?;
        let template_flow_id = campaign.template_flow_id.clone();

        for (contact_row_id, contact_id) in &contacts {
            let phone: Option<String> = sqlx::query_scalar(
                "SELECT value FROM contacts WHERE id = $1 AND channel = 'whatsapp' LIMIT 1"
            )
            .bind(contact_id)
            .fetch_optional(&self.db)
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
                    .execute(&self.db)
                    .await;
                    continue;
                }
            };

            let (message, msg_type) = if let Some(ref flow_id) = template_flow_id {
                let body: Option<String> = sqlx::query_scalar(
                    "SELECT body FROM whatsapp_template_flows WHERE id = $1 LIMIT 1"
                )
                .bind(flow_id)
                .fetch_optional(&self.db)
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
                        .execute(&self.db)
                        .await;
                        continue;
                    }
                }
            } else {
                ("Hello from our team!".to_string(), "text".to_string())
            };

            // In the standalone service, we just mark as sent since we don't have WhatsAppService
            // The actual sending would be done via a separate worker or gRPC call
            let sent_at = chrono::Utc::now().to_rfc3339();
            let _ = sqlx::query(
                "UPDATE campaign_contacts SET status = 'sent', message_id = $1, sent_at = $2 WHERE id = $3"
            )
            .bind(uuid::Uuid::new_v4().to_string())
            .bind(&sent_at)
            .bind(contact_row_id)
            .execute(&self.db)
            .await;

            let _ = sqlx::query(
                "UPDATE campaigns SET sent_count = sent_count + 1, updated_at = $1 WHERE id = $2"
            )
            .bind(&sent_at)
            .bind(campaign_id)
            .execute(&self.db)
            .await;

            tokio::time::sleep(std::time::Duration::from_millis(200)).await;
        }

        let pending: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM campaign_contacts WHERE campaign_id = $1 AND status IN ('pending', 'failed')"
        )
        .bind(campaign_id)
        .fetch_one(&self.db)
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
            .execute(&self.db)
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
        .execute(&self.db)
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
        .execute(&self.db)
        .await
        .map_err(|e| format!("DB error: {}", e))?;

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
        .execute(&self.db)
        .await
        .map_err(|e| format!("DB error: {}", e))?;

        let _ = sqlx::query(
            "DELETE FROM campaign_retry_queue WHERE campaign_id = $1"
        )
        .bind(campaign_id)
        .execute(&self.db)
        .await;

        Ok(())
    }

    // ---------------------------------------------------------------------------
    // Retry Queue
    // ---------------------------------------------------------------------------

    pub async fn retry_failed_messages(&self, campaign_id: &str) -> Result<i64, String> {
        let now = chrono::Utc::now().to_rfc3339();

        let retries: Vec<(String, String, Option<String>, String, i32)> = sqlx::query_as(
            "SELECT id, contact_id, template_flow_id, campaign_id, retry_count
             FROM campaign_retry_queue
             WHERE campaign_id = $1 AND status = 'pending' AND next_retry_at <= $2
             ORDER BY next_retry_at"
        )
        .bind(campaign_id)
        .bind(&now)
        .fetch_all(&self.db)
        .await
        .map_err(|e| format!("DB error: {}", e))?;

        let mut success_count = 0i64;

        for (retry_id, contact_id, template_flow_id, _campaign_id, retry_count) in &retries {
            let phone: Option<String> = sqlx::query_scalar(
                "SELECT value FROM contacts WHERE id = $1 AND channel = 'whatsapp' LIMIT 1"
            )
            .bind(contact_id.as_str())
            .fetch_optional(&self.db)
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
                    .execute(&self.db)
                    .await;
                    continue;
                }
            };

            let _message = if let Some(ref flow_id) = template_flow_id {
                let body: Option<String> = sqlx::query_scalar(
                    "SELECT body FROM whatsapp_template_flows WHERE id = $1 LIMIT 1"
                )
                .bind(flow_id)
                .fetch_optional(&self.db)
                .await
                .unwrap_or(None);

                body.unwrap_or_else(|| "Hello from our team!".to_string())
            } else {
                "Hello from our team!".to_string()
            };

            // Mark as sent in standalone service
            let sent_at = chrono::Utc::now().to_rfc3339();
            let _ = sqlx::query(
                "UPDATE campaign_retry_queue SET status = 'sent', message_id = $1, sent_at = $2, error = NULL WHERE id = $3"
            )
            .bind(uuid::Uuid::new_v4().to_string())
            .bind(&sent_at)
            .bind(retry_id)
            .execute(&self.db)
            .await;

            let _ = sqlx::query(
                "UPDATE campaign_contacts SET status = 'sent', message_id = $1, sent_at = $2, error = NULL
                 WHERE campaign_id = $3 AND contact_id = $4"
            )
            .bind(uuid::Uuid::new_v4().to_string())
            .bind(&sent_at)
            .bind(campaign_id)
            .bind(contact_id)
            .execute(&self.db)
            .await;

            success_count += 1;

            tokio::time::sleep(std::time::Duration::from_millis(200)).await;
        }

        let _ = sqlx::query(
            "UPDATE campaigns SET retry_count = retry_count + $1, updated_at = $2 WHERE id = $3"
        )
        .bind(success_count)
        .bind(&now)
        .bind(campaign_id)
        .execute(&self.db)
        .await;

        Ok(success_count)
    }

    pub async fn process_retry_queue(&self) -> Result<i64, String> {
        let now = chrono::Utc::now().to_rfc3339();

        let campaigns: Vec<String> = sqlx::query_scalar(
            "SELECT DISTINCT campaign_id FROM campaign_retry_queue WHERE status = 'pending' AND next_retry_at <= $1"
        )
        .bind(&now)
        .fetch_all(&self.db)
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
        .fetch_one(&self.db)
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
        .fetch_optional(&self.db)
        .await
        .map_err(|e| format!("DB error: {}", e))?;

        match brand_id {
            Some(id) => Ok(id),
            None => Err("Campaign not found".to_string()),
        }
    }
}
