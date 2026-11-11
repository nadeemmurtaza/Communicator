//! Phase 37: Reply Engine Integration Service
//!
//! Orchestrates all knowledge sources (KB entries, help center articles, document chunks,
//! audience-scoped KB, tone of voice, guidance rules) into a single context package
//! for the AI reply engine.

use serde::{Deserialize, Serialize};
use sqlx::PgPool;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ReplyContextPackage {
    /// Combined knowledge context string for the AI prompt
    pub knowledge_context: String,
    /// Tone and style instructions from tone_of_voice
    pub tone_instructions: String,
    /// Guidance rules from tone_of_voice
    pub guidance_instructions: String,
    /// List of knowledge sources used (for tracking)
    pub sources_used: Vec<KnowledgeSourceUsed>,
    /// Whether audience scoping was applied
    pub audience_scoped: bool,
    /// Contact ID used for scoping (if any)
    pub contact_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct KnowledgeSourceUsed {
    pub source_type: String,
    pub source_id: Option<String>,
    pub source_title: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ReplyIntegrationSettings {
    pub id: String,
    pub org_id: String,
    pub use_tone_of_voice: bool,
    pub use_guidance_rules: bool,
    pub use_audience_scoping: bool,
    pub use_help_center_articles: bool,
    pub use_document_chunks: bool,
    pub max_knowledge_entries: i32,
    pub max_document_chunks: i32,
    pub max_help_center_articles: i32,
    pub track_knowledge_sources: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ReplyIntegrationSummary {
    pub tone_of_voice_enabled: bool,
    pub guidance_rules_active: i32,
    pub audience_scoping_enabled: bool,
    pub help_center_enabled: bool,
    pub document_chunks_enabled: bool,
    pub total_knowledge_sources_tracked: i64,
    pub sources_by_type: Vec<SourceTypeCount>,
    pub settings: ReplyIntegrationSettings,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SourceTypeCount {
    pub source_type: String,
    pub count: i64,
}

pub struct ReplyIntegrationService {
    pub pool: PgPool,
}

impl ReplyIntegrationService {
    pub fn new(pool: &PgPool) -> Self {
        Self { pool: pool.clone() }
    }

    /// Build the complete reply context package by combining all knowledge sources.
    /// This is the main entry point for the reply engine.
    pub async fn build_context_package(
        &self,
        contact_id: Option<&str>,
        last_inbound: &str,
        tenant_key: &str,
    ) -> Result<ReplyContextPackage, anyhow::Error> {
        let settings = self.get_settings().await.unwrap_or_else(|| ReplyIntegrationSettings {
            id: "default".to_string(),
            org_id: "default".to_string(),
            use_tone_of_voice: true,
            use_guidance_rules: true,
            use_audience_scoping: true,
            use_help_center_articles: true,
            use_document_chunks: true,
            max_knowledge_entries: 30,
            max_document_chunks: 15,
            max_help_center_articles: 10,
            track_knowledge_sources: true,
        });

        let mut sources_used = Vec::new();
        let mut knowledge_blocks = Vec::new();

        // 1. Knowledge Base entries (with audience scoping if enabled)
        let kb_entries = if settings.use_audience_scoping {
            self.get_scoped_kb_entries(contact_id, tenant_key, settings.max_knowledge_entries).await
        } else {
            self.get_all_kb_entries(tenant_key, settings.max_knowledge_entries).await
        };

        let audience_scoped = settings.use_audience_scoping && contact_id.is_some();

        if let Ok(entries) = kb_entries {
            for entry in &entries {
                knowledge_blocks.push(format!("### {}\n{}", entry.title, entry.content));
                sources_used.push(KnowledgeSourceUsed {
                    source_type: if audience_scoped { "audience_scoped" } else { "knowledge_base" }.to_string(),
                    source_id: Some(entry.id.clone()),
                    source_title: entry.title.clone(),
                });
            }
        }

        // 2. Help Center articles
        if settings.use_help_center_articles {
            if let Ok(articles) = self.get_help_center_articles(last_inbound, settings.max_help_center_articles).await {
                for article in &articles {
                    knowledge_blocks.push(format!("### [Help Center] {}\n{}", article.title, article.content));
                    sources_used.push(KnowledgeSourceUsed {
                        source_type: "help_center".to_string(),
                        source_id: Some(article.id.clone()),
                        source_title: article.title.clone(),
                    });
                }
            }
        }

        // 3. Document chunks (from uploaded PDFs, DOCX, etc.)
        if settings.use_document_chunks {
            if let Ok(chunks) = self.get_relevant_document_chunks(last_inbound, settings.max_document_chunks).await {
                for chunk in &chunks {
                    knowledge_blocks.push(format!("### [Document: {}]\n{}", chunk.document_title, chunk.text));
                    sources_used.push(KnowledgeSourceUsed {
                        source_type: "document_chunk".to_string(),
                        source_id: Some(chunk.chunk_id.clone()),
                        source_title: chunk.document_title.clone(),
                    });
                }
            }
        }

        let knowledge_context = if knowledge_blocks.is_empty() {
            String::new()
        } else {
            format!("KNOWLEDGE BASE:\n{}\n\n", knowledge_blocks.join("\n\n"))
        };

        // 4. Tone of Voice & Guidance Rules
        let (tone_instructions, guidance_instructions) = if settings.use_tone_of_voice {
            self.get_tone_and_guidance().await.unwrap_or((String::new(), String::new()))
        } else {
            (String::new(), String::new())
        };

        Ok(ReplyContextPackage {
            knowledge_context,
            tone_instructions,
            guidance_instructions,
            sources_used,
            audience_scoped,
            contact_id: contact_id.map(|s| s.to_string()),
        })
    }

    /// Get all KB entries for a tenant (no scoping)
    async fn get_all_kb_entries(
        &self,
        tenant_key: &str,
        limit: i32,
    ) -> Result<Vec<KBEntry>, anyhow::Error> {
        let rows = sqlx::query(
            "SELECT id, title, content FROM knowledge_base_entries \
             WHERE COALESCE(tenant_key, '') = $1 \
             ORDER BY updated_at DESC LIMIT $2"
        )
        .bind(tenant_key)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        let mut entries = Vec::new();
        for row in rows {
            let id: String = row.try_get("id").unwrap_or_default();
            let title: String = row.try_get("title").unwrap_or_default();
            let content: String = row.try_get("content").unwrap_or_default();
            if !content.trim().is_empty() {
                entries.push(KBEntry { id, title, content });
            }
        }
        Ok(entries)
    }

    /// Get audience-scoped KB entries for a contact
    async fn get_scoped_kb_entries(
        &self,
        contact_id: Option<&str>,
        tenant_key: &str,
        limit: i32,
    ) -> Result<Vec<KBEntry>, anyhow::Error> {
        // If no contact_id, fall back to all entries
        let Some(contact_id) = contact_id else {
            return self.get_all_kb_entries(tenant_key, limit).await;
        };

        // Check if audience scoping is enabled and has links
        let enabled: Option<bool> = sqlx::query_scalar(
            "SELECT enabled FROM audience_kb_settings WHERE organization_id = 'org-default' LIMIT 1"
        ).fetch_optional(&self.pool).await?;

        if !enabled.unwrap_or(false) {
            return self.get_all_kb_entries(tenant_key, limit).await;
        }

        // Get segments for this contact
        let segment_rows = sqlx::query(
            "SELECT segment_id FROM segment_contact_memberships WHERE contact_id = $1"
        ).bind(contact_id).fetch_all(&self.pool).await?;

        let segment_ids: Vec<String> = segment_rows.iter().map(|r| {
            r.try_get::<String, _>("segment_id").unwrap_or_default()
        }).collect();

        if segment_ids.is_empty() {
            // Check fallback setting
            let fallback: Option<bool> = sqlx::query_scalar(
                "SELECT fallback_to_all_kb FROM audience_kb_settings WHERE organization_id = 'org-default' LIMIT 1"
            ).fetch_optional(&self.pool).await?;

            if fallback.unwrap_or(true) {
                return self.get_all_kb_entries(tenant_key, limit).await;
            }
            return Ok(Vec::new());
        }

        // Get KB entries linked to these segments
        let mut entries = Vec::new();
        for segment_id in &segment_ids {
            let rows = sqlx::query(
                "SELECT kbe.id, kbe.title, kbe.content FROM knowledge_base_entries kbe \
                 JOIN audience_kb_links aul ON aul.kb_entry_id = kbe.id \
                 WHERE aul.segment_id = $1 \
                 ORDER BY aul.priority DESC LIMIT $2"
            ).bind(segment_id).bind(limit).fetch_all(&self.pool).await?;

            for row in rows {
                let id: String = row.try_get("id").unwrap_or_default();
                let title: String = row.try_get("title").unwrap_or_default();
                let content: String = row.try_get("content").unwrap_or_default();
                if !content.trim().is_empty() && !entries.iter().any(|e| e.id == id) {
                    entries.push(KBEntry { id, title, content });
                }
            }
        }

        Ok(entries)
    }

    /// Get help center articles relevant to the inbound message
    async fn get_help_center_articles(
        &self,
        _last_inbound: &str,
        limit: i32,
    ) -> Result<Vec<HelpCenterArticle>, anyhow::Error> {
        // Get published articles, ordered by helpfulness
        let rows = sqlx::query(
            "SELECT id, title, content FROM help_center_articles \
             WHERE status = 'published' \
             ORDER BY helpful_count DESC, created_at DESC LIMIT $1"
        ).bind(limit).fetch_all(&self.pool).await?;

        let mut articles = Vec::new();
        for row in rows {
            let id: String = row.try_get("id").unwrap_or_default();
            let title: String = row.try_get("title").unwrap_or_default();
            let content: String = row.try_get("content").unwrap_or_default();
            if !content.trim().is_empty() {
                articles.push(HelpCenterArticle { id, title, content });
            }
        }
        Ok(articles)
    }

    /// Get document chunks relevant to the inbound message
    async fn get_relevant_document_chunks(
        &self,
        _last_inbound: &str,
        limit: i32,
    ) -> Result<Vec<DocumentChunk>, anyhow::Error> {
        // Get the most recent document chunks
        let rows = sqlx::query(
            "SELECT kdc.id, kdc.text, kdc.document_id, kbd.title as document_title \
             FROM kb_document_chunks kdc \
             JOIN kb_documents kbd ON kdc.document_id = kbd.id \
             ORDER BY kbd.created_at DESC LIMIT $1"
        ).bind(limit).fetch_all(&self.pool).await?;

        let mut chunks = Vec::new();
        for row in rows {
            let id: String = row.try_get("id").unwrap_or_default();
            let text: String = row.try_get("text").unwrap_or_default();
            let document_id: String = row.try_get("document_id").unwrap_or_default();
            let document_title: String = row.try_get("document_title").unwrap_or_default();
            if !text.trim().is_empty() {
                chunks.push(DocumentChunk {
                    chunk_id: id,
                    text,
                    document_id,
                    document_title,
                });
            }
        }
        Ok(chunks)
    }

    /// Get tone instructions and guidance rules
    async fn get_tone_and_guidance(&self) -> Result<(String, String), anyhow::Error> {
        // Get tone config
        let config_row = sqlx::query(
            "SELECT tone_style, personality, answer_length, formality_level, humor_enabled, \
             empathy_level, conciseness_level, custom_instructions, enabled \
             FROM tone_config WHERE organization_id = 'org-default' LIMIT 1"
        ).fetch_optional(&self.pool).await?;

        let enabled = config_row.as_ref().map(|r| r.try_get::<bool, _>(8).unwrap_or(true)).unwrap_or(true);

        if !enabled {
            return Ok((String::new(), String::new()));
        }

        let mut tone_parts = Vec::new();

        if let Some(r) = &config_row {
            let tone_style: String = r.try_get(0).unwrap_or_default();
            let personality: String = r.try_get(1).unwrap_or_default();
            let answer_length: String = r.try_get(2).unwrap_or_default();
            let formality_level: i32 = r.try_get(3).unwrap_or(5i32);
            let humor_enabled: bool = r.try_get(4).unwrap_or(false);
            let empathy_level: i32 = r.try_get(5).unwrap_or(5i32);
            let conciseness_level: i32 = r.try_get(6).unwrap_or(5i32);
            let custom_instructions: Option<String> = r.try_get(7).ok();

            tone_parts.push(format!("- Tone style: {}", tone_style));
            tone_parts.push(format!("- Personality: {}", personality));

            match answer_length.as_str() {
                "short" => tone_parts.push("- Answer length: Keep responses brief and to the point (1-3 sentences)".to_string()),
                "long" => tone_parts.push("- Answer length: Provide thorough, detailed responses with examples".to_string()),
                _ => tone_parts.push("- Answer length: Provide balanced responses (3-5 sentences)".to_string()),
            }

            if formality_level >= 7 {
                tone_parts.push("- Formality: Use formal, professional language".to_string());
            } else if formality_level <= 3 {
                tone_parts.push("- Formality: Use casual, conversational language".to_string());
            } else {
                tone_parts.push("- Formality: Use a balanced, semi-formal tone".to_string());
            }

            if humor_enabled {
                tone_parts.push("- Humor: Light humor is appropriate when it feels natural".to_string());
            } else {
                tone_parts.push("- Humor: Avoid humor; stay professional".to_string());
            }

            if empathy_level >= 7 {
                tone_parts.push("- Empathy: Always acknowledge feelings and show deep understanding".to_string());
            } else if empathy_level <= 3 {
                tone_parts.push("- Empathy: Focus on solutions rather than emotional acknowledgment".to_string());
            } else {
                tone_parts.push("- Empathy: Show moderate empathy when appropriate".to_string());
            }

            if conciseness_level >= 7 {
                tone_parts.push("- Conciseness: Be very direct; avoid unnecessary elaboration".to_string());
            } else if conciseness_level <= 3 {
                tone_parts.push("- Conciseness: Elaborate and provide context; don't worry about length".to_string());
            } else {
                tone_parts.push("- Conciseness: Balance detail with brevity".to_string());
            }

            if let Some(custom) = custom_instructions {
                tone_parts.push(format!("- Custom: {}", custom));
            }
        }

        // Get guidance rules
        let guidance_rows = sqlx::query(
            "SELECT instruction FROM guidance_rules WHERE enabled = true ORDER BY priority DESC"
        ).fetch_all(&self.pool).await?;

        let mut guidance_parts = Vec::new();
        for row in &guidance_rows {
            let instruction: String = row.try_get(0).unwrap_or_default();
            guidance_parts.push(instruction);
        }

        let tone_instructions = if tone_parts.is_empty() {
            String::new()
        } else {
            format!("\n\nTONE AND STYLE:\n{}\n", tone_parts.join("\n"))
        };

        let guidance_instructions = if guidance_parts.is_empty() {
            String::new()
        } else {
            format!("\n\nGUIDANCE RULES:\n{}\n", guidance_parts.join("\n"))
        };

        Ok((tone_instructions, guidance_instructions))
    }

    /// Track knowledge sources used in a reply
    pub async fn track_knowledge_sources(
        &self,
        draft_id: &str,
        conversation_id: &str,
        sources: &[KnowledgeSourceUsed],
    ) -> Result<(), anyhow::Error> {
        let settings = self.get_settings().await;
        if !settings.map(|s| s.track_knowledge_sources).unwrap_or(true) {
            return Ok(());
        }

        for source in sources {
            sqlx::query(
                "INSERT INTO reply_knowledge_sources (draft_id, conversation_id, source_type, source_id, source_title) \
                 VALUES ($1, $2, $3, $4, $5)"
            )
            .bind(draft_id)
            .bind(conversation_id)
            .bind(&source.source_type)
            .bind(&source.source_id)
            .bind(&source.source_title)
            .execute(&self.pool)
            .await?;
        }

        Ok(())
    }

    /// Get settings
    pub async fn get_settings(&self) -> Result<ReplyIntegrationSettings, anyhow::Error> {
        let row = sqlx::query(
            "SELECT id, org_id, use_tone_of_voice, use_guidance_rules, use_audience_scoping, \
             use_help_center_articles, use_document_chunks, max_knowledge_entries, \
             max_document_chunks, max_help_center_articles, track_knowledge_sources \
             FROM reply_integration_settings WHERE org_id = 'default' LIMIT 1"
        ).fetch_one(&self.pool).await?;

        Ok(ReplyIntegrationSettings {
            id: row.try_get(0).unwrap_or_default(),
            org_id: row.try_get(1).unwrap_or_default(),
            use_tone_of_voice: row.try_get(2).unwrap_or(true),
            use_guidance_rules: row.try_get(3).unwrap_or(true),
            use_audience_scoping: row.try_get(4).unwrap_or(true),
            use_help_center_articles: row.try_get(5).unwrap_or(true),
            use_document_chunks: row.try_get(6).unwrap_or(true),
            max_knowledge_entries: row.try_get(7).unwrap_or(30i32),
            max_document_chunks: row.try_get(8).unwrap_or(15i32),
            max_help_center_articles: row.try_get(9).unwrap_or(10i32),
            track_knowledge_sources: row.try_get(10).unwrap_or(true),
        })
    }

    /// Update settings
    pub async fn update_settings(&self, settings: &ReplyIntegrationSettings) -> Result<ReplyIntegrationSettings, anyhow::Error> {
        sqlx::query(
            "UPDATE reply_integration_settings SET \
             use_tone_of_voice = $1, use_guidance_rules = $2, use_audience_scoping = $3, \
             use_help_center_articles = $4, use_document_chunks = $5, \
             max_knowledge_entries = $6, max_document_chunks = $7, max_help_center_articles = $8, \
             track_knowledge_sources = $9, updated_at = NOW() \
             WHERE id = $10"
        )
        .bind(settings.use_tone_of_voice)
        .bind(settings.use_guidance_rules)
        .bind(settings.use_audience_scoping)
        .bind(settings.use_help_center_articles)
        .bind(settings.use_document_chunks)
        .bind(settings.max_knowledge_entries)
        .bind(settings.max_document_chunks)
        .bind(settings.max_help_center_articles)
        .bind(settings.track_knowledge_sources)
        .bind(&settings.id)
        .execute(&self.pool)
        .await?;

        self.get_settings().await
    }

    /// Get summary dashboard data
    pub async fn get_summary(&self) -> Result<ReplyIntegrationSummary, anyhow::Error> {
        let settings = self.get_settings().await.unwrap_or_else(|_| ReplyIntegrationSettings {
            id: "default".to_string(),
            org_id: "default".to_string(),
            use_tone_of_voice: true,
            use_guidance_rules: true,
            use_audience_scoping: true,
            use_help_center_articles: true,
            use_document_chunks: true,
            max_knowledge_entries: 30,
            max_document_chunks: 15,
            max_help_center_articles: 10,
            track_knowledge_sources: true,
        });

        // Tone of voice enabled
        let tone_enabled: Option<bool> = sqlx::query_scalar(
            "SELECT enabled FROM tone_config WHERE organization_id = 'org-default' LIMIT 1"
        ).fetch_optional(&self.pool).await.unwrap_or(Some(false));

        // Active guidance rules
        let guidance_active: i32 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM guidance_rules WHERE enabled = true"
        ).fetch_one(&self.pool).await.unwrap_or(0);

        // Audience scoping enabled
        let audience_enabled: Option<bool> = sqlx::query_scalar(
            "SELECT enabled FROM audience_kb_settings WHERE organization_id = 'org-default' LIMIT 1"
        ).fetch_optional(&self.pool).await.unwrap_or(false);

        // Help center enabled (check if articles exist)
        let help_center_count: i32 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM help_center_articles WHERE status = 'published'"
        ).fetch_one(&self.pool).await.unwrap_or(0);

        // Document chunks enabled (check if chunks exist)
        let document_chunks_count: i32 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM kb_document_chunks"
        ).fetch_one(&self.pool).await.unwrap_or(0);

        // Total knowledge sources tracked
        let total_tracked: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM reply_knowledge_sources"
        ).fetch_one(&self.pool).await.unwrap_or(0);

        // Sources by type
        let sources_by_type = sqlx::query_as::<_, SourceTypeCount>(
            "SELECT source_type, COUNT(*) as count FROM reply_knowledge_sources GROUP BY source_type ORDER BY count DESC"
        ).fetch_all(&self.pool).await.unwrap_or_default();

        Ok(ReplyIntegrationSummary {
            tone_of_voice_enabled: tone_enabled.unwrap_or(false),
            guidance_rules_active: guidance_active,
            audience_scoping_enabled: audience_enabled.unwrap_or(false),
            help_center_enabled: help_center_count > 0,
            document_chunks_enabled: document_chunks_count > 0,
            total_knowledge_sources_tracked: total_tracked,
            sources_by_type,
            settings,
        })
    }
}

#[derive(Debug, Clone)]
struct KBEntry {
    id: String,
    title: String,
    content: String,
}

#[derive(Debug, Clone)]
struct HelpCenterArticle {
    id: String,
    title: String,
    content: String,
}

#[derive(Debug, Clone)]
struct DocumentChunk {
    chunk_id: String,
    text: String,
    document_id: String,
    document_title: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialization() {
        let pkg = ReplyContextPackage {
            knowledge_context: "test".to_string(),
            tone_instructions: "be nice".to_string(),
            guidance_instructions: "be accurate".to_string(),
            sources_used: vec![],
            audience_scoped: false,
            contact_id: None,
        };
        let json = serde_json::to_string(&pkg).unwrap();
        assert!(json.contains("test"));
        assert!(json.contains("be nice"));
    }

    #[test]
    fn test_settings_serialization() {
        let settings = ReplyIntegrationSettings {
            id: "test".to_string(),
            org_id: "default".to_string(),
            use_tone_of_voice: true,
            use_guidance_rules: true,
            use_audience_scoping: true,
            use_help_center_articles: true,
            use_document_chunks: true,
            max_knowledge_entries: 30,
            max_document_chunks: 15,
            max_help_center_articles: 10,
            track_knowledge_sources: true,
        };
        let json = serde_json::to_string(&settings).unwrap();
        assert!(json.contains("test"));
    }

    #[test]
    fn test_summary_serialization() {
        let summary = ReplyIntegrationSummary {
            tone_of_voice_enabled: true,
            guidance_rules_active: 5,
            audience_scoping_enabled: true,
            help_center_enabled: true,
            document_chunks_enabled: false,
            total_knowledge_sources_tracked: 100,
            sources_by_type: vec![],
            settings: ReplyIntegrationSettings {
                id: "test".to_string(),
                org_id: "default".to_string(),
                use_tone_of_voice: true,
                use_guidance_rules: true,
                use_audience_scoping: true,
                use_help_center_articles: true,
                use_document_chunks: true,
                max_knowledge_entries: 30,
                max_document_chunks: 15,
                max_help_center_articles: 10,
                track_knowledge_sources: true,
            },
        };
        let json = serde_json::to_string(&summary).unwrap();
        assert!(json.contains("100"));
    }
}
