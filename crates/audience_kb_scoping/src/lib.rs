//! Audience-Based Knowledge Scoping Library
//!
//! Links customer segments (audiences) to specific knowledge base subsets so
//! different customer groups see different AI knowledge.

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::PgPool;

// ——— Types ———

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AudienceKBLink {
    pub id: String,
    pub segment_id: String,
    pub segment_name: Option<String>,
    pub kb_entry_id: String,
    pub kb_title: Option<String>,
    pub priority: i32,
    pub created_at: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AudienceKBSettings {
    pub id: String,
    pub organization_id: String,
    pub enabled: bool,
    pub fallback_to_global_kb: bool,
    pub merge_segments: bool,
    pub default_scope: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SegmentContactMembership {
    pub id: String,
    pub segment_id: String,
    pub segment_name: Option<String>,
    pub contact_id: String,
    pub contact_name: Option<String>,
    pub membership_type: String,
    pub matched_at: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AudienceKBUsageLog {
    pub id: String,
    pub conversation_id: String,
    pub contact_id: String,
    pub segment_id: String,
    pub kb_entry_id: String,
    pub query_text: String,
    pub relevance_score: f64,
    pub used_in_response: bool,
    pub created_at: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AudienceSummary {
    pub total_segments_with_kb: i32,
    pub total_links: i32,
    pub total_memberships: i32,
    pub total_usage_7d: i32,
    pub avg_relevance_score: f64,
    pub enabled: bool,
    pub top_segments: Vec<Value>,
    pub top_kb_entries: Vec<Value>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ScopedKBResult {
    pub kb_entry_ids: Vec<String>,
    pub segment_ids: Vec<String>,
    pub fallback_included: bool,
    pub total_entries: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LinkResult {
    pub link_id: String,
    pub segment_id: String,
    pub kb_entry_id: String,
    pub message: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MembershipResult {
    pub membership_id: String,
    pub segment_id: String,
    pub contact_id: String,
    pub message: String,
}

// ——— Service ———

pub struct AudienceKbScopingService {
    pool: PgPool,
}

impl AudienceKbScopingService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn get_audience_summary(&self) -> Result<AudienceSummary, anyhow::Error> {
        let total_segments_with_kb: i32 = sqlx::query_scalar(
            "SELECT COUNT(DISTINCT segment_id) FROM audience_kb_links"
        )
        .fetch_one(&self.pool)
        .await?;

        let total_links: i32 = sqlx::query_scalar("SELECT COUNT(*) FROM audience_kb_links")
            .fetch_one(&self.pool)
            .await?;

        let total_memberships: i32 = sqlx::query_scalar("SELECT COUNT(*) FROM segment_contact_memberships")
            .fetch_one(&self.pool)
            .await?;

        let total_usage_7d: i32 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM audience_kb_usage_log WHERE created_at >= NOW() - INTERVAL '7 days'"
        )
        .fetch_one(&self.pool)
        .await?;

        let avg_relevance_score: f64 = sqlx::query_scalar(
            "SELECT COALESCE(AVG(relevance_score), 0) FROM audience_kb_usage_log"
        )
        .fetch_one(&self.pool)
        .await?;

        let settings_row = sqlx::query(
            "SELECT enabled FROM audience_kb_settings WHERE organization_id = 'org-default' LIMIT 1"
        )
        .fetch_optional(&self.pool)
        .await?;
        let enabled = settings_row
            .map(|r| r.try_get::<bool, _>(0).unwrap_or(true))
            .unwrap_or(true);

        let top_segments_rows = sqlx::query(
            "SELECT cs.name, COUNT(aul.id) as cnt FROM audience_kb_links aul \
             JOIN contact_segments cs ON aul.segment_id = cs.id \
             GROUP BY cs.id, cs.name ORDER BY cnt DESC LIMIT 5"
        )
        .fetch_all(&self.pool)
        .await?;

        let top_segments: Vec<Value> = top_segments_rows
            .iter()
            .map(|row| {
                json!({
                    "segment_name": row.try_get::<String, _>(0).unwrap_or_default(),
                    "link_count": row.try_get::<i64, _>(1).unwrap_or(0i64),
                })
            })
            .collect();

        let top_kb_rows = sqlx::query(
            "SELECT kbe.title, COUNT(aul.id) as cnt FROM audience_kb_usage_log aul \
             JOIN knowledge_base_entries kbe ON aul.kb_entry_id = kbe.id \
             GROUP BY kbe.id, kbe.title ORDER BY cnt DESC LIMIT 5"
        )
        .fetch_all(&self.pool)
        .await?;

        let top_kb_entries: Vec<Value> = top_kb_rows
            .iter()
            .map(|row| {
                json!({
                    "kb_title": row.try_get::<String, _>(0).unwrap_or_default(),
                    "usage_count": row.try_get::<i64, _>(1).unwrap_or(0i64),
                })
            })
            .collect();

        Ok(AudienceSummary {
            total_segments_with_kb,
            total_links,
            total_memberships,
            total_usage_7d,
            avg_relevance_score,
            enabled,
            top_segments,
            top_kb_entries,
        })
    }

    pub async fn get_segment_kb_links(&self, segment_id: &str) -> Result<Vec<AudienceKBLink>, anyhow::Error> {
        let rows = sqlx::query(
            "SELECT aul.id, aul.segment_id, cs.name as segment_name, \
             aul.kb_entry_id, kbe.title as kb_title, aul.priority, aul.created_at \
             FROM audience_kb_links aul \
             LEFT JOIN contact_segments cs ON aul.segment_id = cs.id \
             LEFT JOIN knowledge_base_entries kbe ON aul.kb_entry_id = kbe.id \
             WHERE aul.segment_id = $1 \
             ORDER BY aul.priority DESC, aul.created_at DESC"
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .iter()
            .map(|row| AudienceKBLink {
                id: row.try_get(0).unwrap_or_default(),
                segment_id: row.try_get(1).unwrap_or_default(),
                segment_name: row.try_get(2).ok(),
                kb_entry_id: row.try_get(3).unwrap_or_default(),
                kb_title: row.try_get(4).ok(),
                priority: row.try_get(5).unwrap_or(0i32),
                created_at: row.try_get(6).unwrap_or_default(),
            })
            .collect())
    }

    pub async fn get_all_links(&self) -> Result<Vec<Value>, anyhow::Error> {
        let rows = sqlx::query(
            "SELECT aul.id, aul.segment_id, cs.name as segment_name, \
             aul.kb_entry_id, kbe.title as kb_title, aul.priority, aul.created_at \
             FROM audience_kb_links aul \
             LEFT JOIN contact_segments cs ON aul.segment_id = cs.id \
             LEFT JOIN knowledge_base_entries kbe ON aul.kb_entry_id = kbe.id \
             ORDER BY aul.segment_id, aul.priority DESC"
        )
        .fetch_all(&self.pool)
        .await?;

        let links: Vec<Value> = rows
            .iter()
            .map(|row| {
                json!({
                    "id": row.try_get::<String, _>(0).unwrap_or_default(),
                    "segment_id": row.try_get::<String, _>(1).unwrap_or_default(),
                    "segment_name": row.try_get::<Option<String>, _>(2).ok(),
                    "kb_entry_id": row.try_get::<String, _>(3).unwrap_or_default(),
                    "kb_title": row.try_get::<Option<String>, _>(4).ok(),
                    "priority": row.try_get::<i32, _>(5).unwrap_or(0i32),
                    "created_at": row.try_get::<String, _>(6).unwrap_or_default(),
                })
            })
            .collect();

        Ok(links)
    }

    pub async fn link_kb_to_segment(
        &self,
        segment_id: &str,
        kb_entry_id: &str,
        priority: i32,
    ) -> Result<LinkResult, String> {
        let segment_exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM contact_segments WHERE id = $1)"
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        if !segment_exists {
            return Err(format!("Segment {} not found", segment_id));
        }

        let kb_exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM knowledge_base_entries WHERE id = $1)"
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        if !kb_exists {
            return Err(format!("Knowledge base entry {} not found", kb_entry_id));
        }

        let link_id = format!("audience-kb-{}", uuid::Uuid::new_v4());

        sqlx::query(
            "INSERT INTO audience_kb_links (id, segment_id, kb_entry_id, priority) \
             VALUES ($1, $2, $3, $4) \
             ON CONFLICT (segment_id, kb_entry_id) \
             DO UPDATE SET priority = $4"
        )
        .bind(&link_id)
        .bind(segment_id)
        .bind(kb_entry_id)
        .bind(priority)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to create link: {}", e))?;

        Ok(LinkResult {
            link_id,
            segment_id: segment_id.to_string(),
            kb_entry_id: kb_entry_id.to_string(),
            message: "Knowledge base entry linked to segment".to_string(),
        })
    }

    pub async fn unlink_kb_from_segment(&self, segment_id: &str, kb_entry_id: &str) -> Result<(), String> {
        sqlx::query(
            "DELETE FROM audience_kb_links WHERE segment_id = $1 AND kb_entry_id = $2"
        )
        .bind(segment_id)
        .bind(kb_entry_id)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to remove link: {}", e))?;

        Ok(())
    }

    pub async fn get_scoped_kb_for_contact(&self, contact_id: &str) -> Result<ScopedKBResult, String> {
        let settings = self.get_audience_kb_settings().await;
        let enabled = settings.as_ref().map(|s| s.enabled).unwrap_or(true);
        let fallback = settings.as_ref().map(|s| s.fallback_to_global_kb).unwrap_or(true);
        let merge = settings.as_ref().map(|s| s.merge_segments).unwrap_or(true);

        if !enabled {
            let rows = sqlx::query(
                "SELECT id FROM knowledge_base_entries WHERE status = 'published'"
            )
            .fetch_all(&self.pool)
            .await
            .map_err(|e| e.to_string())?;

            let kb_entry_ids: Vec<String> = rows
                .iter()
                .map(|r| r.try_get(0).unwrap_or_default())
                .collect();
            let total_entries = kb_entry_ids.len() as i32;

            return Ok(ScopedKBResult {
                kb_entry_ids,
                segment_ids: vec![],
                fallback_included: true,
                total_entries,
            });
        }

        let membership_rows = sqlx::query(
            "SELECT segment_id FROM segment_contact_memberships WHERE contact_id = $1"
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        let segment_ids: Vec<String> = membership_rows
            .iter()
            .map(|r| r.try_get(0).unwrap_or_default())
            .collect();

        if segment_ids.is_empty() {
            if fallback {
                let rows = sqlx::query(
                    "SELECT id FROM knowledge_base_entries WHERE status = 'published'"
                )
                .fetch_all(&self.pool)
                .await
                .map_err(|e| e.to_string())?;

                let kb_entry_ids: Vec<String> = rows
                    .iter()
                    .map(|r| r.try_get(0).unwrap_or_default())
                    .collect();
                let total_entries = kb_entry_ids.len() as i32;

                return Ok(ScopedKBResult {
                    kb_entry_ids,
                    segment_ids: vec![],
                    fallback_included: true,
                    total_entries,
                });
            } else {
                return Ok(ScopedKBResult {
                    kb_entry_ids: vec![],
                    segment_ids: vec![],
                    fallback_included: false,
                    total_entries: 0,
                });
            }
        }

        let mut kb_entry_ids = Vec::new();

        if merge {
            let placeholders: Vec<String> = (0..segment_ids.len())
                .map(|i| format!("${}", i + 1))
                .collect();
            let placeholder_str = placeholders.join(",");

            let query = format!(
                "SELECT DISTINCT aul.kb_entry_id FROM audience_kb_links aul \
                 WHERE aul.segment_id IN ({}) \
                 ORDER BY aul.priority DESC",
                placeholder_str
            );

            let mut query_builder = sqlx::query(&query);
            for seg_id in &segment_ids {
                query_builder = query_builder.bind(seg_id);
            }

            let rows = query_builder
                .fetch_all(&self.pool)
                .await
                .map_err(|e| e.to_string())?;

            kb_entry_ids = rows
                .iter()
                .map(|r| r.try_get(0).unwrap_or_default())
                .collect();
        } else {
            let top_segment = &segment_ids[0];
            let rows = sqlx::query(
                "SELECT kb_entry_id FROM audience_kb_links WHERE segment_id = $1 ORDER BY priority DESC"
            )
            .bind(top_segment)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| e.to_string())?;

            kb_entry_ids = rows
                .iter()
                .map(|r| r.try_get(0).unwrap_or_default())
                .collect();
        }

        let fallback_included = fallback && kb_entry_ids.is_empty();

        if fallback_included {
            let rows = sqlx::query(
                "SELECT id FROM knowledge_base_entries WHERE status = 'published'"
            )
            .fetch_all(&self.pool)
            .await
            .map_err(|e| e.to_string())?;

            kb_entry_ids = rows
                .iter()
                .map(|r| r.try_get(0).unwrap_or_default())
                .collect();
        }

        let total_entries = kb_entry_ids.len() as i32;

        Ok(ScopedKBResult {
            kb_entry_ids,
            segment_ids,
            fallback_included,
            total_entries,
        })
    }

    pub async fn add_contact_to_segment(
        &self,
        segment_id: &str,
        contact_id: &str,
        membership_type: &str,
    ) -> Result<MembershipResult, String> {
        let segment_exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM contact_segments WHERE id = $1)"
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        if !segment_exists {
            return Err(format!("Segment {} not found", segment_id));
        }

        let contact_exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM contacts WHERE id = $1)"
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        if !contact_exists {
            return Err(format!("Contact {} not found", contact_id));
        }

        let membership_id = format!("seg-member-{}", uuid::Uuid::new_v4());

        sqlx::query(
            "INSERT INTO segment_contact_memberships (id, segment_id, contact_id, membership_type) \
             VALUES ($1, $2, $3, $4) \
             ON CONFLICT (segment_id, contact_id) \
             DO UPDATE SET membership_type = $4, matched_at = NOW()"
        )
        .bind(&membership_id)
        .bind(segment_id)
        .bind(contact_id)
        .bind(membership_type)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to add membership: {}", e))?;

        Ok(MembershipResult {
            membership_id,
            segment_id: segment_id.to_string(),
            contact_id: contact_id.to_string(),
            message: format!("Contact added to segment as {}", membership_type),
        })
    }

    pub async fn remove_contact_from_segment(&self, segment_id: &str, contact_id: &str) -> Result<(), String> {
        sqlx::query(
            "DELETE FROM segment_contact_memberships WHERE segment_id = $1 AND contact_id = $2"
        )
        .bind(segment_id)
        .bind(contact_id)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to remove membership: {}", e))?;

        Ok(())
    }

    pub async fn get_contact_segment_memberships(
        &self,
        contact_id: &str,
    ) -> Result<Vec<SegmentContactMembership>, anyhow::Error> {
        let rows = sqlx::query(
            "SELECT scm.id, scm.segment_id, cs.name as segment_name, \
             scm.contact_id, c.name as contact_name, scm.membership_type, scm.matched_at \
             FROM segment_contact_memberships scm \
             LEFT JOIN contact_segments cs ON scm.segment_id = cs.id \
             LEFT JOIN contacts c ON scm.contact_id = c.id \
             WHERE scm.contact_id = $1 \
             ORDER BY scm.matched_at DESC"
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .iter()
            .map(|row| SegmentContactMembership {
                id: row.try_get(0).unwrap_or_default(),
                segment_id: row.try_get(1).unwrap_or_default(),
                segment_name: row.try_get(2).ok(),
                contact_id: row.try_get(3).unwrap_or_default(),
                contact_name: row.try_get(4).ok(),
                membership_type: row.try_get(5).unwrap_or_default(),
                matched_at: row.try_get(6).unwrap_or_default(),
            })
            .collect())
    }

    pub async fn get_segment_memberships(
        &self,
        segment_id: &str,
    ) -> Result<Vec<SegmentContactMembership>, anyhow::Error> {
        let rows = sqlx::query(
            "SELECT scm.id, scm.segment_id, cs.name as segment_name, \
             scm.contact_id, c.name as contact_name, scm.membership_type, scm.matched_at \
             FROM segment_contact_memberships scm \
             LEFT JOIN contact_segments cs ON scm.segment_id = cs.id \
             LEFT JOIN contacts c ON scm.contact_id = c.id \
             WHERE scm.segment_id = $1 \
             ORDER BY scm.matched_at DESC"
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .iter()
            .map(|row| SegmentContactMembership {
                id: row.try_get(0).unwrap_or_default(),
                segment_id: row.try_get(1).unwrap_or_default(),
                segment_name: row.try_get(2).ok(),
                contact_id: row.try_get(3).unwrap_or_default(),
                contact_name: row.try_get(4).ok(),
                membership_type: row.try_get(5).unwrap_or_default(),
                matched_at: row.try_get(6).unwrap_or_default(),
            })
            .collect())
    }

    pub async fn get_all_memberships(&self, limit: i32) -> Result<Vec<Value>, anyhow::Error> {
        let rows = sqlx::query(
            "SELECT scm.id, scm.segment_id, cs.name as segment_name, \
             scm.contact_id, c.name as contact_name, scm.membership_type, scm.matched_at \
             FROM segment_contact_memberships scm \
             LEFT JOIN contact_segments cs ON scm.segment_id = cs.id \
             LEFT JOIN contacts c ON scm.contact_id = c.id \
             ORDER BY scm.matched_at DESC LIMIT $1"
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        let memberships: Vec<Value> = rows
            .iter()
            .map(|row| {
                json!({
                    "id": row.try_get::<String, _>(0).unwrap_or_default(),
                    "segment_id": row.try_get::<String, _>(1).unwrap_or_default(),
                    "segment_name": row.try_get::<Option<String>, _>(2).ok(),
                    "contact_id": row.try_get::<String, _>(3).unwrap_or_default(),
                    "contact_name": row.try_get::<Option<String>, _>(4).ok(),
                    "membership_type": row.try_get::<String, _>(5).unwrap_or_default(),
                    "matched_at": row.try_get::<String, _>(6).unwrap_or_default(),
                })
            })
            .collect();

        Ok(memberships)
    }

    pub async fn log_kb_usage(
        &self,
        conversation_id: &str,
        contact_id: &str,
        segment_id: &str,
        kb_entry_id: &str,
        query_text: &str,
        relevance_score: f64,
        used_in_response: bool,
    ) -> Result<(), String> {
        let log_id = format!("audience-usage-{}", uuid::Uuid::new_v4());

        sqlx::query(
            "INSERT INTO audience_kb_usage_log \
             (id, conversation_id, contact_id, segment_id, kb_entry_id, query_text, relevance_score, used_in_response) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8)"
        )
        .bind(&log_id)
        .bind(conversation_id)
        .bind(contact_id)
        .bind(segment_id)
        .bind(kb_entry_id)
        .bind(query_text)
        .bind(relevance_score)
        .bind(used_in_response)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to log usage: {}", e))?;

        Ok(())
    }

    pub async fn get_audience_kb_settings(&self) -> Option<AudienceKBSettings> {
        let row = sqlx::query(
            "SELECT id, organization_id, enabled, fallback_to_global_kb, merge_segments, \
             default_scope, created_at, updated_at \
             FROM audience_kb_settings WHERE organization_id = 'org-default' LIMIT 1"
        )
        .fetch_optional(&self.pool)
        .await
        .ok()??;

        Some(AudienceKBSettings {
            id: row.try_get(0).unwrap_or_default(),
            organization_id: row.try_get(1).unwrap_or_default(),
            enabled: row.try_get(2).unwrap_or(true),
            fallback_to_global_kb: row.try_get(3).unwrap_or(true),
            merge_segments: row.try_get(4).unwrap_or(true),
            default_scope: row.try_get(5).unwrap_or_default(),
            created_at: row.try_get(6).unwrap_or_default(),
            updated_at: row.try_get(7).unwrap_or_default(),
        })
    }

    pub async fn update_audience_kb_settings(
        &self,
        enabled: Option<bool>,
        fallback_to_global_kb: Option<bool>,
        merge_segments: Option<bool>,
        default_scope: Option<&str>,
    ) -> Result<AudienceKBSettings, String> {
        let settings = self.get_audience_kb_settings().await;
        let mut enabled = enabled.unwrap_or(settings.as_ref().map(|s| s.enabled).unwrap_or(true));
        let mut fallback = fallback_to_global_kb.unwrap_or(settings.as_ref().map(|s| s.fallback_to_global_kb).unwrap_or(true));
        let mut merge = merge_segments.unwrap_or(settings.as_ref().map(|s| s.merge_segments).unwrap_or(true));
        let scope = default_scope.unwrap_or("all");

        sqlx::query(
            "INSERT INTO audience_kb_settings (id, organization_id, enabled, fallback_to_global_kb, merge_segments, default_scope, updated_at) \
             VALUES ($1, 'org-default', $2, $3, $4, $5, NOW()) \
             ON CONFLICT (id) \
             DO UPDATE SET enabled = $2, fallback_to_global_kb = $3, merge_segments = $4, default_scope = $5, updated_at = NOW()"
        )
        .bind("audience-kb-default")
        .bind(enabled)
        .bind(fallback)
        .bind(merge)
        .bind(scope)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to update settings: {}", e))?;

        self.get_audience_kb_settings()
            .await
            .ok_or_else(|| "Failed to read updated settings".to_string())
    }

    pub async fn get_segment_usage_logs(
        &self,
        segment_id: &str,
        limit: i32,
    ) -> Result<Vec<AudienceKBUsageLog>, anyhow::Error> {
        let rows = sqlx::query(
            "SELECT id, conversation_id, contact_id, segment_id, kb_entry_id, \
             query_text, relevance_score, used_in_response, created_at \
             FROM audience_kb_usage_log WHERE segment_id = $1 \
             ORDER BY created_at DESC LIMIT $2"
        )
        .bind(segment_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .iter()
            .map(|row| AudienceKBUsageLog {
                id: row.try_get(0).unwrap_or_default(),
                conversation_id: row.try_get(1).unwrap_or_default(),
                contact_id: row.try_get(2).unwrap_or_default(),
                segment_id: row.try_get(3).unwrap_or_default(),
                kb_entry_id: row.try_get(4).unwrap_or_default(),
                query_text: row.try_get(5).unwrap_or_default(),
                relevance_score: row.try_get(6).unwrap_or(0.0f64),
                used_in_response: row.try_get(7).unwrap_or(false),
                created_at: row.try_get(8).unwrap_or_default(),
            })
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialization() {
        let summary = AudienceSummary {
            total_segments_with_kb: 5,
            total_links: 20,
            total_memberships: 100,
            total_usage_7d: 50,
            avg_relevance_score: 0.85,
            enabled: true,
            top_segments: vec![],
            top_kb_entries: vec![],
        };
        let json = serde_json::to_string(&summary).unwrap();
        assert!(json.contains("\"total_segments_with_kb\":5"));
    }

    #[test]
    fn test_link_result_serialization() {
        let result = LinkResult {
            link_id: "test-link".to_string(),
            segment_id: "seg-1".to_string(),
            kb_entry_id: "kb-1".to_string(),
            message: "linked".to_string(),
        };
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("\"link_id\":\"test-link\""));
    }
}
