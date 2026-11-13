//! Phase 42: Conversation Search & Discovery Service
//! Full-text search across conversations, messages, and contacts with saved searches
//! and search analytics.

use serde::{Deserialize, Serialize};
use sqlx::{Pool, Postgres, Row};
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Models
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SavedSearch {
    pub id: String,
    pub org_id: String,
    pub name: String,
    pub query: String,
    pub search_type: String,
    pub filters: String,
    pub created_by: String,
    pub created_at: String,
    pub updated_at: String,
    pub is_pinned: bool,
    pub usage_count: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SearchHistoryEntry {
    pub id: String,
    pub org_id: String,
    pub query: String,
    pub search_type: String,
    pub result_count: i64,
    pub filters: String,
    pub performed_by: String,
    pub performed_at: String,
    pub duration_ms: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SearchAnalytics {
    pub id: String,
    pub org_id: String,
    pub window_start: String,
    pub window_end: String,
    pub total_searches: i64,
    pub avg_result_count: f64,
    pub avg_duration_ms: f64,
    pub zero_result_searches: i64,
    pub top_queries: String,
    pub top_search_types: String,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SearchSettings {
    pub id: String,
    pub org_id: String,
    pub max_results: i32,
    pub search_message_content: bool,
    pub search_contact_details: bool,
    pub search_conversation_metadata: bool,
    pub track_search_history: bool,
    pub auto_suggest: bool,
    pub min_query_length: i32,
    pub highlight_matches: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreateSavedSearchRequest {
    pub name: String,
    pub query: String,
    pub search_type: Option<String>,
    pub filters: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SearchResult {
    pub search_type: String,
    pub id: String,
    pub title: String,
    pub snippet: String,
    pub metadata: serde_json::Value,
    pub matched_fields: Vec<String>,
    pub score: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SearchResponse {
    pub query: String,
    pub total_results: i64,
    pub conversations: Vec<SearchResult>,
    pub messages: Vec<SearchResult>,
    pub contacts: Vec<SearchResult>,
    pub duration_ms: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SearchSummary {
    pub total_saved_searches: i64,
    pub pinned_searches: i64,
    pub total_searches_today: i64,
    pub total_searches_7d: i64,
    pub avg_results_per_search: f64,
    pub zero_result_rate: f64,
    pub top_queries: Vec<QueryCount>,
    pub top_search_types: Vec<SearchTypeCount>,
    pub recent_history: Vec<SearchHistoryEntry>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct QueryCount {
    pub query: String,
    pub count: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SearchTypeCount {
    pub search_type: String,
    pub count: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SearchAnalyticsSnapshot {
    pub window_start: String,
    pub window_end: String,
    pub total_searches: i64,
    pub avg_result_count: f64,
    pub avg_duration_ms: f64,
    pub zero_result_searches: i64,
}

// ---------------------------------------------------------------------------
// Service
// ---------------------------------------------------------------------------

pub struct ConversationSearchService {
    pool: Pool<Postgres>,
}

impl ConversationSearchService {
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }

    /// Get overall search summary
    pub async fn get_summary(&self) -> Result<SearchSummary, anyhow::Error> {
        let total_saved: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM saved_searches")
            .fetch_one(&self.pool).await?;
        let pinned: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM saved_searches WHERE is_pinned = true")
            .fetch_one(&self.pool).await?;
        let today_searches: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM search_history WHERE performed_at >= CURRENT_DATE")
            .fetch_one(&self.pool).await?;
        let week_searches: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM search_history WHERE performed_at >= NOW() - INTERVAL '7 days'")
            .fetch_one(&self.pool).await?;
        
        let avg_results: f64 = sqlx::query_scalar("SELECT COALESCE(AVG(result_count), 0) FROM search_history WHERE performed_at >= NOW() - INTERVAL '7 days'")
            .fetch_one(&self.pool).await?;
        
        let zero_result: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM search_history WHERE result_count = 0 AND performed_at >= NOW() - INTERVAL '7 days'")
            .fetch_one(&self.pool).await?;
        
        let total_7d: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM search_history WHERE performed_at >= NOW() - INTERVAL '7 days'")
            .fetch_one(&self.pool).await?;
        
        let zero_rate = if total_7d > 0 { (zero_result as f64 / total_7d as f64) * 100.0 } else { 0.0 };

        let query_rows = sqlx::query(
            "SELECT query, COUNT(*) as count FROM search_history WHERE performed_at >= NOW() - INTERVAL '7 days' GROUP BY query ORDER BY count DESC LIMIT 10"
        ).fetch_all(&self.pool).await?;
        let top_queries: Vec<QueryCount> = query_rows.iter().map(|r| QueryCount {
            query: r.get("query"),
            count: r.get("count"),
        }).collect();

        let type_rows = sqlx::query(
            "SELECT search_type, COUNT(*) as count FROM search_history WHERE performed_at >= NOW() - INTERVAL '7 days' GROUP BY search_type ORDER BY count DESC"
        ).fetch_all(&self.pool).await?;
        let top_search_types: Vec<SearchTypeCount> = type_rows.iter().map(|r| SearchTypeCount {
            search_type: r.get("search_type"),
            count: r.get("count"),
        }).collect();

        let history_rows = sqlx::query(
            "SELECT id, org_id, query, search_type, result_count, filters, performed_by, performed_at, duration_ms \
             FROM search_history ORDER BY performed_at DESC LIMIT 10"
        ).fetch_all(&self.pool).await?;
        let recent_history: Vec<SearchHistoryEntry> = history_rows.iter().map(|r| SearchHistoryEntry {
            id: r.get("id"),
            org_id: r.get("org_id"),
            query: r.get("query"),
            search_type: r.get("search_type"),
            result_count: r.get("result_count"),
            filters: r.get("filters"),
            performed_by: r.get("performed_by"),
            performed_at: r.get("performed_at"),
            duration_ms: r.get("duration_ms"),
        }).collect();

        Ok(SearchSummary {
            total_saved_searches: total_saved,
            pinned_searches: pinned,
            total_searches_today: today_searches,
            total_searches_7d: week_searches,
            avg_results_per_search: avg_results,
            zero_result_rate: zero_rate,
            top_queries,
            top_search_types,
            recent_history,
        })
    }

    // ---------------------------------------------------------------------------
    // Full-Text Search
    // ---------------------------------------------------------------------------

    /// Search across conversations, messages, and contacts
    pub async fn search(&self, query: &str, search_type: &str, filters: &serde_json::Value, limit: i32) -> Result<SearchResponse, String> {
        let start = std::time::Instant::now();
        let q = query.trim();
        if q.is_empty() {
            return Ok(SearchResponse {
                query: query.to_string(),
                total_results: 0,
                conversations: vec![],
                messages: vec![],
                contacts: vec![],
                duration_ms: 0,
            });
        }

        let limit = limit.max(1).min(100) as i64;
        let like_pattern = format!("%{}%", q);

        let mut conversations = Vec::new();
        let mut messages = Vec::new();
        let mut contacts = Vec::new();

        // Search conversations
        if search_type == "all" || search_type == "conversation" {
            let rows = sqlx::query(
                "SELECT id, contact_id, channel, status, subject, created_at, updated_at \
                 FROM conversations \
                 WHERE LOWER(subject) LIKE LOWER($1) OR LOWER(contact_id) LIKE LOWER($1) \
                 ORDER BY updated_at DESC LIMIT $2"
            ).bind(&like_pattern).bind(limit).fetch_all(&self.pool).await
            .unwrap_or_else(|_| Vec::new());
            
            for r in &rows {
                conversations.push(SearchResult {
                    search_type: "conversation".to_string(),
                    id: r.get("id"),
                    title: r.get::<String, _>("subject").clone(),
                    snippet: format!("Channel: {}, Status: {}, Contact: {}", 
                        r.get::<String, _>("channel"),
                        r.get::<String, _>("status"),
                        r.get::<String, _>("contact_id")),
                    metadata: serde_json::json!({
                        "channel": r.get::<String, _>("channel"),
                        "status": r.get::<String, _>("status"),
                        "contact_id": r.get::<String, _>("contact_id"),
                        "created_at": r.get::<String, _>("created_at"),
                    }),
                    matched_fields: vec!["subject".to_string()],
                    score: 1.0,
                });
            }
        }

        // Search messages
        if search_type == "all" || search_type == "message" {
            let rows = sqlx::query(
                "SELECT id, conversation_id, sender_type, body, created_at \
                 FROM messages \
                 WHERE LOWER(body) LIKE LOWER($1) \
                 ORDER BY created_at DESC LIMIT $2"
            ).bind(&like_pattern).bind(limit).fetch_all(&self.pool).await
            .unwrap_or_else(|_| Vec::new());
            
            for r in &rows {
                let body: String = r.get("body");
                let snippet = if body.len() > 200 {
                    format!("{}...", &body[..200])
                } else {
                    body.clone()
                };
                messages.push(SearchResult {
                    search_type: "message".to_string(),
                    id: r.get("id"),
                    title: format!("Message in conversation {}", r.get::<String, _>("conversation_id")),
                    snippet,
                    metadata: serde_json::json!({
                        "conversation_id": r.get::<String, _>("conversation_id"),
                        "sender_type": r.get::<String, _>("sender_type"),
                        "created_at": r.get::<String, _>("created_at"),
                    }),
                    matched_fields: vec!["body".to_string()],
                    score: 1.0,
                });
            }
        }

        // Search contacts
        if search_type == "all" || search_type == "contact" {
            let rows = sqlx::query(
                "SELECT id, phone, email, first_name, last_name, created_at \
                 FROM contacts \
                 WHERE LOWER(phone) LIKE LOWER($1) OR LOWER(email) LIKE LOWER($1) \
                        OR LOWER(first_name) LIKE LOWER($1) OR LOWER(last_name) LIKE LOWER($1) \
                 ORDER BY created_at DESC LIMIT $2"
            ).bind(&like_pattern).bind(limit).fetch_all(&self.pool).await
            .unwrap_or_else(|_| Vec::new());
            
            for r in &rows {
                let name = format!("{} {}", 
                    r.get::<String, _>("first_name"),
                    r.get::<String, _>("last_name"));
                contacts.push(SearchResult {
                    search_type: "contact".to_string(),
                    id: r.get("id"),
                    title: if name.trim().is_empty() { r.get::<String, _>("phone").clone() } else { name.clone() },
                    snippet: format!("Phone: {}, Email: {}", 
                        r.get::<String, _>("phone"),
                        r.get::<String, _>("email")),
                    metadata: serde_json::json!({
                        "phone": r.get::<String, _>("phone"),
                        "email": r.get::<String, _>("email"),
                        "first_name": r.get::<String, _>("first_name"),
                        "last_name": r.get::<String, _>("last_name"),
                        "created_at": r.get::<String, _>("created_at"),
                    }),
                    matched_fields: vec!["name".to_string(), "phone".to_string(), "email".to_string()],
                    score: 1.0,
                });
            }
        }

        let duration = start.elapsed().as_millis() as i64;
        let total = conversations.len() + messages.len() + contacts.len();

        // Record search in history
        let history_id = Uuid::new_v4().to_string();
        sqlx::query(
            "INSERT INTO search_history (id, query, search_type, result_count, filters, duration_ms) \
             VALUES ($1, $2, $3, $4, $5, $6)"
        ).bind(&history_id).bind(q).bind(search_type).bind(total as i64)
         .bind(filters.to_string()).bind(duration)
         .execute(&self.pool).await.ok();

        Ok(SearchResponse {
            query: query.to_string(),
            total_results: total as i64,
            conversations,
            messages,
            contacts,
            duration_ms: duration,
        })
    }

    // ---------------------------------------------------------------------------
    // Saved Searches
    // ---------------------------------------------------------------------------

    pub async fn list_saved_searches(&self, search_type: Option<&str>) -> Result<Vec<SavedSearch>, String> {
        let mut query_str = String::from("SELECT id, org_id, name, query, search_type, filters, created_by, created_at, updated_at, is_pinned, usage_count FROM saved_searches WHERE 1=1");
        
        if let Some(st) = search_type {
            query_str.push_str(" AND search_type = $1");
        }
        query_str.push_str(" ORDER BY is_pinned DESC, created_at DESC");

        let rows = if search_type.is_some() {
            sqlx::query(&query_str).bind(search_type.unwrap()).fetch_all(&self.pool).await
        } else {
            sqlx::query(&query_str).fetch_all(&self.pool).await
        }
        .map_err(|e| format!("Failed to list saved searches: {e}"))?;

        rows.iter().map(|r| Ok(SavedSearch {
            id: r.get("id"),
            org_id: r.get("org_id"),
            name: r.get("name"),
            query: r.get("query"),
            search_type: r.get("search_type"),
            filters: r.get("filters"),
            created_by: r.get("created_by"),
            created_at: r.get("created_at"),
            updated_at: r.get("updated_at"),
            is_pinned: r.get("is_pinned"),
            usage_count: r.get("usage_count"),
        })).collect()
    }

    pub async fn create_saved_search(&self, req: &CreateSavedSearchRequest, created_by: &str) -> Result<SavedSearch, String> {
        let id = Uuid::new_v4().to_string();
        let search_type = req.search_type.clone().unwrap_or_else(|| "all".to_string());
        let filters = req.filters.clone().unwrap_or_else(|| serde_json::json!({}));

        sqlx::query(
            "INSERT INTO saved_searches (id, name, query, search_type, filters, created_by) \
             VALUES ($1, $2, $3, $4, $5, $6)"
        ).bind(&id).bind(&req.name).bind(&req.query).bind(&search_type)
         .bind(filters.to_string()).bind(created_by)
         .execute(&self.pool).await
         .map_err(|e| format!("Failed to create saved search: {e}"))?;

        self.get_saved_search(&id).await
    }

    pub async fn get_saved_search(&self, id: &str) -> Result<SavedSearch, String> {
        let row = sqlx::query(
            "SELECT id, org_id, name, query, search_type, filters, created_by, created_at, updated_at, is_pinned, usage_count \
             FROM saved_searches WHERE id = $1"
        ).bind(id).fetch_one(&self.pool).await
        .map_err(|_| format!("Saved search '{}' not found", id))?;

        Ok(SavedSearch {
            id: row.get("id"),
            org_id: row.get("org_id"),
            name: row.get("name"),
            query: row.get("query"),
            search_type: row.get("search_type"),
            filters: row.get("filters"),
            created_by: row.get("created_by"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            is_pinned: row.get("is_pinned"),
            usage_count: row.get("usage_count"),
        })
    }

    pub async fn delete_saved_search(&self, id: &str) -> Result<(), String> {
        sqlx::query("DELETE FROM saved_searches WHERE id = $1")
            .bind(id).execute(&self.pool).await
            .map_err(|e| format!("Failed to delete saved search: {e}"))?;
        Ok(())
    }

    pub async fn pin_saved_search(&self, id: &str, pinned: bool) -> Result<(), String> {
        sqlx::query("UPDATE saved_searches SET is_pinned = $1, updated_at = NOW() WHERE id = $2")
            .bind(pinned).bind(id).execute(&self.pool).await
            .map_err(|e| format!("Failed to update saved search: {e}"))?;
        Ok(())
    }

    pub async fn use_saved_search(&self, id: &str) -> Result<SearchResponse, String> {
        // Increment usage count
        sqlx::query("UPDATE saved_searches SET usage_count = usage_count + 1, updated_at = NOW() WHERE id = $1")
            .bind(id).execute(&self.pool).await.ok();

        let saved = self.get_saved_search(id).await?;
        self.search(&saved.query, &saved.search_type, &serde_json::from_str(&saved.filters).unwrap_or_else(|_| serde_json::json!({})), 50).await
    }

    // ---------------------------------------------------------------------------
    // Search History
    // ---------------------------------------------------------------------------

    pub async fn get_search_history(&self, limit: i32) -> Result<Vec<SearchHistoryEntry>, String> {
        let limit = limit.max(1).min(100) as i64;
        let rows = sqlx::query(
            "SELECT id, org_id, query, search_type, result_count, filters, performed_by, performed_at, duration_ms \
             FROM search_history ORDER BY performed_at DESC LIMIT $1"
        ).bind(limit).fetch_all(&self.pool).await
        .map_err(|e| format!("Failed to get search history: {e}"))?;

        rows.iter().map(|r| Ok(SearchHistoryEntry {
            id: r.get("id"),
            org_id: r.get("org_id"),
            query: r.get("query"),
            search_type: r.get("search_type"),
            result_count: r.get("result_count"),
            filters: r.get("filters"),
            performed_by: r.get("performed_by"),
            performed_at: r.get("performed_at"),
            duration_ms: r.get("duration_ms"),
        })).collect()
    }

    pub async fn clear_search_history(&self) -> Result<(), String> {
        sqlx::query("DELETE FROM search_history")
            .execute(&self.pool).await
            .map_err(|e| format!("Failed to clear search history: {e}"))?;
        Ok(())
    }

    // ---------------------------------------------------------------------------
    // Analytics
    // ---------------------------------------------------------------------------

    pub async fn get_analytics(&self) -> Result<Vec<SearchAnalytics>, String> {
        let rows = sqlx::query(
            "SELECT id, org_id, window_start, window_end, total_searches, avg_result_count, \
                    avg_duration_ms, zero_result_searches, top_queries, top_search_types, created_at \
             FROM search_analytics ORDER BY window_start DESC LIMIT 30"
        ).fetch_all(&self.pool).await
        .map_err(|e| format!("Failed to get analytics: {e}"))?;

        rows.iter().map(|r| Ok(SearchAnalytics {
            id: r.get("id"),
            org_id: r.get("org_id"),
            window_start: r.get("window_start"),
            window_end: r.get("window_end"),
            total_searches: r.get("total_searches"),
            avg_result_count: r.get("avg_result_count"),
            avg_duration_ms: r.get("avg_duration_ms"),
            zero_result_searches: r.get("zero_result_searches"),
            top_queries: r.get("top_queries"),
            top_search_types: r.get("top_search_types"),
            created_at: r.get("created_at"),
        })).collect()
    }

    pub async fn generate_analytics_snapshot(&self) -> Result<SearchAnalyticsSnapshot, String> {
        let window_start = (chrono::Utc::now() - chrono::TimeDelta::days(7)).to_rfc3339();
        let window_end = chrono::Utc::now().to_rfc3339();

        let total: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM search_history WHERE performed_at >= $1"
        ).bind(&window_start).fetch_one(&self.pool).await
        .map_err(|e| format!("Failed to count searches: {e}"))?;

        let avg_results: f64 = sqlx::query_scalar(
            "SELECT COALESCE(AVG(result_count), 0) FROM search_history WHERE performed_at >= $1"
        ).bind(&window_start).fetch_one(&self.pool).await
        .map_err(|e| format!("Failed to avg results: {e}"))?;

        let avg_duration: f64 = sqlx::query_scalar(
            "SELECT COALESCE(AVG(duration_ms), 0) FROM search_history WHERE performed_at >= $1"
        ).bind(&window_start).fetch_one(&self.pool).await
        .map_err(|e| format!("Failed to avg duration: {e}"))?;

        let zero_result: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM search_history WHERE result_count = 0 AND performed_at >= $1"
        ).bind(&window_start).fetch_one(&self.pool).await
        .map_err(|e| format!("Failed to count zero results: {e}"))?;

        // Store snapshot
        let id = Uuid::new_v4().to_string();
        sqlx::query(
            "INSERT INTO search_analytics (id, window_start, window_end, total_searches, \
             avg_result_count, avg_duration_ms, zero_result_searches) \
             VALUES ($1, $2, $3, $4, $5, $6, $7)"
        ).bind(&id).bind(&window_start).bind(&window_end).bind(total)
         .bind(avg_results).bind(avg_duration).bind(zero_result)
         .execute(&self.pool).await
         .map_err(|e| format!("Failed to store analytics snapshot: {e}"))?;

        Ok(SearchAnalyticsSnapshot {
            window_start,
            window_end,
            total_searches: total,
            avg_result_count: avg_results,
            avg_duration_ms: avg_duration,
            zero_result_searches: zero_result,
        })
    }

    // ---------------------------------------------------------------------------
    // Settings
    // ---------------------------------------------------------------------------

    pub async fn get_settings(&self) -> Result<SearchSettings, String> {
        let row = sqlx::query(
            "SELECT id, org_id, max_results, search_message_content, search_contact_details, \
                    search_conversation_metadata, track_search_history, auto_suggest, \
                    min_query_length, highlight_matches, created_at, updated_at \
             FROM search_settings WHERE id = 'default' OR org_id = 'default' LIMIT 1"
        ).fetch_one(&self.pool).await
        .map_err(|_| "Search settings not found".to_string())?;

        Ok(SearchSettings {
            id: row.get("id"),
            org_id: row.get("org_id"),
            max_results: row.get("max_results"),
            search_message_content: row.get("search_message_content"),
            search_contact_details: row.get("search_contact_details"),
            search_conversation_metadata: row.get("search_conversation_metadata"),
            track_search_history: row.get("track_search_history"),
            auto_suggest: row.get("auto_suggest"),
            min_query_length: row.get("min_query_length"),
            highlight_matches: row.get("highlight_matches"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        })
    }

    pub async fn update_settings(&self, settings: &SearchSettings) -> Result<SearchSettings, String> {
        sqlx::query(
            "UPDATE search_settings SET max_results = $1, search_message_content = $2, \
             search_contact_details = $3, search_conversation_metadata = $4, \
             track_search_history = $5, auto_suggest = $6, min_query_length = $7, \
             highlight_matches = $8, updated_at = NOW() \
             WHERE id = $9"
        ).bind(settings.max_results).bind(settings.search_message_content)
         .bind(settings.search_contact_details).bind(settings.search_conversation_metadata)
         .bind(settings.track_search_history).bind(settings.auto_suggest)
         .bind(settings.min_query_length).bind(settings.highlight_matches)
         .bind(&settings.id)
         .execute(&self.pool).await
         .map_err(|e| format!("Failed to update settings: {e}"))?;

        self.get_settings().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialization() {
        let summary = SearchSummary {
            total_saved_searches: 0,
            pinned_searches: 0,
            total_searches_today: 0,
            total_searches_7d: 0,
            avg_results_per_search: 0.0,
            zero_result_rate: 0.0,
            top_queries: vec![],
            top_search_types: vec![],
            recent_history: vec![],
        };
        let json = serde_json::to_string(&summary).unwrap();
        assert!(json.contains("totalSavedSearches"));
    }

    #[test]
    fn test_search_response() {
        let response = SearchResponse {
            query: "test".to_string(),
            total_results: 0,
            conversations: vec![],
            messages: vec![],
            contacts: vec![],
            duration_ms: 0,
        };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("totalResults"));
    }

    #[test]
    fn test_settings_serialization() {
        let settings = SearchSettings {
            id: "default".to_string(),
            org_id: "default".to_string(),
            max_results: 50,
            search_message_content: true,
            search_contact_details: true,
            search_conversation_metadata: true,
            track_search_history: true,
            auto_suggest: true,
            min_query_length: 2,
            highlight_matches: true,
            created_at: "2024-01-01T00:00:00Z".to_string(),
            updated_at: "2024-01-01T00:00:00Z".to_string(),
        };
        let json = serde_json::to_string(&settings).unwrap();
        assert!(json.contains("maxResults"));
    }
}
