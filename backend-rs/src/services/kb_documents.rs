//! Knowledge Base Document Upload & URL Crawling Service (Phase 23).
//!
//! Handles document uploads (chunked + mirrored into kb_document_chunks),
//! URL source registration/crawling (mirrored into kb_url_chunks), and
//! the per-organization import settings that control chunk sizing and
//! crawl behavior.

use crate::db::AppState;
use regex::Regex;
use serde_json::{json, Value};
use sqlx::Row;
use uuid::Uuid;

const DEFAULT_CHUNK_SIZE: usize = 2000;
const DEFAULT_CHUNK_OVERLAP: usize = 200;

// ---------------------------------------------------------------------------
// Summary
// ---------------------------------------------------------------------------

pub async fn get_kb_import_summary(state: &AppState) -> Result<Value, String> {
    let doc_rows = sqlx::query("SELECT status, COUNT(*) as count FROM kb_documents GROUP BY status")
        .fetch_all(&state.db)
        .await
        .map_err(|e| e.to_string())?;
    let doc_status: Value = json!(doc_rows
        .iter()
        .map(|r| {
            let status: String = r.try_get("status").unwrap_or_default();
            let count: i64 = r.try_get("count").unwrap_or(0);
            (status, count)
        })
        .collect::<std::collections::HashMap<_, _>>());

    let url_rows = sqlx::query("SELECT status, COUNT(*) as count FROM kb_url_sources GROUP BY status")
        .fetch_all(&state.db)
        .await
        .map_err(|e| e.to_string())?;
    let url_status: Value = json!(url_rows
        .iter()
        .map(|r| {
            let status: String = r.try_get("status").unwrap_or_default();
            let count: i64 = r.try_get("count").unwrap_or(0);
            (status, count)
        })
        .collect::<std::collections::HashMap<_, _>>());

    let total_documents: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM kb_documents")
        .fetch_one(&state.db)
        .await
        .map_err(|e| e.to_string())?;
    let total_chunks: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM kb_document_chunks")
        .fetch_one(&state.db)
        .await
        .map_err(|e| e.to_string())?;
    let total_url_sources: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM kb_url_sources")
        .fetch_one(&state.db)
        .await
        .map_err(|e| e.to_string())?;
    let total_url_chunks: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM kb_url_chunks")
        .fetch_one(&state.db)
        .await
        .map_err(|e| e.to_string())?;

    Ok(json!({
        "documents": {
            "total": total_documents,
            "total_chunks": total_chunks,
            "by_status": doc_status,
        },
        "url_sources": {
            "total": total_url_sources,
            "total_chunks": total_url_chunks,
            "by_status": url_status,
        },
    }))
}

// ---------------------------------------------------------------------------
// Documents
// ---------------------------------------------------------------------------

pub async fn list_documents(
    state: &AppState,
    organization_id: Option<&str>,
    status: Option<&str>,
) -> Result<Value, String> {
    let mut query = String::from(
        "SELECT id, organization_id, file_name, file_type, file_size, status, error_message, \
         pages_count, chunks_count, created_at, updated_at FROM kb_documents WHERE 1=1",
    );
    let mut idx = 1;
    if organization_id.is_some() {
        query.push_str(&format!(" AND organization_id = ${}", idx));
        idx += 1;
    }
    if status.is_some() {
        query.push_str(&format!(" AND status = ${}", idx));
    }
    query.push_str(" ORDER BY created_at DESC LIMIT 200");

    let mut q = sqlx::query(&query);
    if let Some(org) = organization_id {
        q = q.bind(org);
    }
    if let Some(s) = status {
        q = q.bind(s);
    }

    let rows = q.fetch_all(&state.db).await.map_err(|e| e.to_string())?;
    Ok(json!(rows.iter().map(document_row_to_json).collect::<Vec<_>>()))
}

pub async fn get_document_chunks(state: &AppState, doc_id: &str) -> Result<Value, String> {
    let rows = sqlx::query(
        "SELECT id, document_id, chunk_index, title, content, page_number, kb_entry_id, created_at \
         FROM kb_document_chunks WHERE document_id = $1 ORDER BY chunk_index ASC",
    )
    .bind(doc_id)
    .fetch_all(&state.db)
    .await
    .map_err(|e| e.to_string())?;

    Ok(json!(rows
        .iter()
        .map(|r| {
            json!({
                "id": r.try_get::<String, _>("id").unwrap_or_default(),
                "document_id": r.try_get::<String, _>("document_id").unwrap_or_default(),
                "chunk_index": r.try_get::<i32, _>("chunk_index").unwrap_or(0),
                "title": r.try_get::<String, _>("title").unwrap_or_default(),
                "content": r.try_get::<String, _>("content").unwrap_or_default(),
                "page_number": r.try_get::<Option<i32>, _>("page_number").unwrap_or(None),
                "kb_entry_id": r.try_get::<Option<String>, _>("kb_entry_id").unwrap_or(None),
            })
        })
        .collect::<Vec<_>>()))
}

pub async fn delete_document(state: &AppState, doc_id: &str) -> Result<(), String> {
    sqlx::query("DELETE FROM kb_document_chunks WHERE document_id = $1")
        .bind(doc_id)
        .execute(&state.db)
        .await
        .map_err(|e| e.to_string())?;

    let result = sqlx::query("DELETE FROM kb_documents WHERE id = $1")
        .bind(doc_id)
        .execute(&state.db)
        .await
        .map_err(|e| e.to_string())?;

    if result.rows_affected() == 0 {
        return Err(format!("Document '{}' not found", doc_id));
    }
    Ok(())
}

/// Upload a document's raw bytes, store it, chunk the text content, and
/// mirror the chunks into kb_document_chunks.
pub async fn upload_document(
    state: &AppState,
    organization_id: &str,
    file_name: &str,
    file_type: &str,
    file_data: &[u8],
) -> Result<Value, String> {
    let settings = get_or_create_settings(state, organization_id).await?;

    let id = Uuid::new_v4().to_string();
    let file_size = file_data.len() as i32;

    // Only plain-text-ish formats are actually parsed into chunks here;
    // binary formats (pdf/docx) are still accepted and stored but chunking
    // is best-effort on whatever is valid UTF-8 in them.
    let text = String::from_utf8_lossy(file_data).to_string();

    sqlx::query(
        "INSERT INTO kb_documents (id, organization_id, file_name, file_type, file_size, status) \
         VALUES ($1, $2, $3, $4, $5, 'processing')",
    )
    .bind(&id)
    .bind(organization_id)
    .bind(file_name)
    .bind(file_type)
    .bind(file_size)
    .execute(&state.db)
    .await
    .map_err(|e| format!("Failed to create document record: {}", e))?;

    let chunks = chunk_text(&text, settings.chunk_size, settings.chunk_overlap);
    for (i, chunk) in chunks.iter().enumerate() {
        let chunk_id = Uuid::new_v4().to_string();
        sqlx::query(
            "INSERT INTO kb_document_chunks (id, document_id, chunk_index, title, content) \
             VALUES ($1, $2, $3, $4, $5)",
        )
        .bind(&chunk_id)
        .bind(&id)
        .bind(i as i32)
        .bind(format!("{} (part {})", file_name, i + 1))
        .bind(chunk)
        .execute(&state.db)
        .await
        .map_err(|e| format!("Failed to store chunk {}: {}", i, e))?;
    }

    sqlx::query(
        "UPDATE kb_documents SET status = 'completed', chunks_count = $1, updated_at = NOW() WHERE id = $2",
    )
    .bind(chunks.len() as i32)
    .bind(&id)
    .execute(&state.db)
    .await
    .map_err(|e| e.to_string())?;

    let row = sqlx::query(
        "SELECT id, organization_id, file_name, file_type, file_size, status, error_message, \
         pages_count, chunks_count, created_at, updated_at FROM kb_documents WHERE id = $1",
    )
    .bind(&id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| e.to_string())?;

    Ok(document_row_to_json(&row))
}

fn document_row_to_json(r: &sqlx::postgres::PgRow) -> Value {
    json!({
        "id": r.try_get::<String, _>("id").unwrap_or_default(),
        "organization_id": r.try_get::<String, _>("organization_id").unwrap_or_default(),
        "file_name": r.try_get::<String, _>("file_name").unwrap_or_default(),
        "file_type": r.try_get::<String, _>("file_type").unwrap_or_default(),
        "file_size": r.try_get::<i32, _>("file_size").unwrap_or(0),
        "status": r.try_get::<String, _>("status").unwrap_or_default(),
        "error_message": r.try_get::<Option<String>, _>("error_message").unwrap_or(None),
        "pages_count": r.try_get::<Option<i32>, _>("pages_count").unwrap_or(None),
        "chunks_count": r.try_get::<Option<i32>, _>("chunks_count").unwrap_or(None),
    })
}

// ---------------------------------------------------------------------------
// URL sources
// ---------------------------------------------------------------------------

pub async fn add_url_source(
    state: &AppState,
    organization_id: &str,
    url: &str,
    crawl_interval_hours: Option<i32>,
) -> Result<Value, String> {
    if url.is_empty() {
        return Err("url is required".to_string());
    }

    let id = Uuid::new_v4().to_string();
    let interval = crawl_interval_hours.unwrap_or(168);

    sqlx::query(
        "INSERT INTO kb_url_sources (id, organization_id, url, crawl_interval_hours) \
         VALUES ($1, $2, $3, $4)",
    )
    .bind(&id)
    .bind(organization_id)
    .bind(url)
    .bind(interval)
    .execute(&state.db)
    .await
    .map_err(|e| format!("Failed to add URL source: {}", e))?;

    get_url_source_json(state, &id).await
}

pub async fn list_url_sources(
    state: &AppState,
    organization_id: Option<&str>,
    status: Option<&str>,
) -> Result<Value, String> {
    let mut query = String::from(
        "SELECT id, organization_id, url, title, status, error_message, content_length, \
         chunks_count, last_crawled_at, next_crawl_at, crawl_interval_hours, created_at, updated_at \
         FROM kb_url_sources WHERE 1=1",
    );
    let mut idx = 1;
    if organization_id.is_some() {
        query.push_str(&format!(" AND organization_id = ${}", idx));
        idx += 1;
    }
    if status.is_some() {
        query.push_str(&format!(" AND status = ${}", idx));
    }
    query.push_str(" ORDER BY created_at DESC LIMIT 200");

    let mut q = sqlx::query(&query);
    if let Some(org) = organization_id {
        q = q.bind(org);
    }
    if let Some(s) = status {
        q = q.bind(s);
    }

    let rows = q.fetch_all(&state.db).await.map_err(|e| e.to_string())?;
    Ok(json!(rows.iter().map(url_source_row_to_json).collect::<Vec<_>>()))
}

pub async fn delete_url_source(state: &AppState, source_id: &str) -> Result<(), String> {
    sqlx::query("DELETE FROM kb_url_chunks WHERE url_source_id = $1")
        .bind(source_id)
        .execute(&state.db)
        .await
        .map_err(|e| e.to_string())?;
    sqlx::query("DELETE FROM kb_url_crawl_logs WHERE url_source_id = $1")
        .bind(source_id)
        .execute(&state.db)
        .await
        .map_err(|e| e.to_string())?;

    let result = sqlx::query("DELETE FROM kb_url_sources WHERE id = $1")
        .bind(source_id)
        .execute(&state.db)
        .await
        .map_err(|e| e.to_string())?;

    if result.rows_affected() == 0 {
        return Err(format!("URL source '{}' not found", source_id));
    }
    Ok(())
}

/// Fetch a URL source's page, strip it down to text, chunk it, and mirror
/// the chunks into kb_url_chunks. Logs the attempt to kb_url_crawl_logs.
pub async fn crawl_url(state: &AppState, source_id: &str) -> Result<Value, String> {
    let organization_id: String = sqlx::query_scalar(
        "SELECT organization_id FROM kb_url_sources WHERE id = $1",
    )
    .bind(source_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| e.to_string())?
    .ok_or_else(|| format!("URL source '{}' not found", source_id))?;

    let url: String = sqlx::query_scalar("SELECT url FROM kb_url_sources WHERE id = $1")
        .bind(source_id)
        .fetch_one(&state.db)
        .await
        .map_err(|e| e.to_string())?;

    let settings = get_or_create_settings(state, &organization_id).await?;
    let log_id = Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO kb_url_crawl_logs (id, url_source_id, status) VALUES ($1, $2, 'running')",
    )
    .bind(&log_id)
    .bind(source_id)
    .execute(&state.db)
    .await
    .map_err(|e| e.to_string())?;

    let response = state
        .http_client
        .get(&url)
        .header("User-Agent", &settings.user_agent)
        .send()
        .await;

    let (status_code, body) = match response {
        Ok(resp) => {
            let code = resp.status().as_u16() as i32;
            let text = resp.text().await.unwrap_or_default();
            (code, text)
        }
        Err(e) => {
            sqlx::query(
                "UPDATE kb_url_crawl_logs SET status = 'failed', error_message = $1, completed_at = NOW()::text WHERE id = $2",
            )
            .bind(e.to_string())
            .bind(&log_id)
            .execute(&state.db)
            .await
            .ok();
            sqlx::query(
                "UPDATE kb_url_sources SET status = 'failed', error_message = $1, updated_at = NOW() WHERE id = $2",
            )
            .bind(e.to_string())
            .bind(source_id)
            .execute(&state.db)
            .await
            .ok();
            return Err(format!("Failed to fetch URL: {}", e));
        }
    };

    let text = html_to_text(&body);
    let title = extract_title(&body).unwrap_or_else(|| url.clone());

    // Replace previous chunks for this source with the freshly crawled ones.
    sqlx::query("DELETE FROM kb_url_chunks WHERE url_source_id = $1")
        .bind(source_id)
        .execute(&state.db)
        .await
        .map_err(|e| e.to_string())?;

    let chunks = chunk_text(&text, settings.chunk_size, settings.chunk_overlap);
    for (i, chunk) in chunks.iter().enumerate() {
        let chunk_id = Uuid::new_v4().to_string();
        sqlx::query(
            "INSERT INTO kb_url_chunks (id, url_source_id, chunk_index, title, content) \
             VALUES ($1, $2, $3, $4, $5)",
        )
        .bind(&chunk_id)
        .bind(source_id)
        .bind(i as i32)
        .bind(format!("{} (part {})", title, i + 1))
        .bind(chunk)
        .execute(&state.db)
        .await
        .map_err(|e| format!("Failed to store chunk {}: {}", i, e))?;
    }

    sqlx::query(
        "UPDATE kb_url_sources SET status = 'completed', title = $1, content_length = $2, \
         chunks_count = $3, error_message = NULL, last_crawled_at = NOW()::text, \
         next_crawl_at = (NOW() + (crawl_interval_hours * INTERVAL '1 hour'))::text, \
         updated_at = NOW() WHERE id = $4",
    )
    .bind(&title)
    .bind(text.len() as i32)
    .bind(chunks.len() as i32)
    .bind(source_id)
    .execute(&state.db)
    .await
    .map_err(|e| e.to_string())?;

    sqlx::query(
        "UPDATE kb_url_crawl_logs SET status = 'completed', status_code = $1, content_length = $2, \
         chunks_created = true, completed_at = NOW()::text WHERE id = $3",
    )
    .bind(status_code)
    .bind(text.len() as i32)
    .bind(&log_id)
    .execute(&state.db)
    .await
    .ok();

    get_url_source_json(state, source_id).await
}

async fn get_url_source_json(state: &AppState, source_id: &str) -> Result<Value, String> {
    let row = sqlx::query(
        "SELECT id, organization_id, url, title, status, error_message, content_length, \
         chunks_count, last_crawled_at, next_crawl_at, crawl_interval_hours, created_at, updated_at \
         FROM kb_url_sources WHERE id = $1",
    )
    .bind(source_id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| e.to_string())?;

    Ok(url_source_row_to_json(&row))
}

fn url_source_row_to_json(r: &sqlx::postgres::PgRow) -> Value {
    json!({
        "id": r.try_get::<String, _>("id").unwrap_or_default(),
        "organization_id": r.try_get::<String, _>("organization_id").unwrap_or_default(),
        "url": r.try_get::<String, _>("url").unwrap_or_default(),
        "title": r.try_get::<Option<String>, _>("title").unwrap_or(None),
        "status": r.try_get::<String, _>("status").unwrap_or_default(),
        "error_message": r.try_get::<Option<String>, _>("error_message").unwrap_or(None),
        "content_length": r.try_get::<Option<i32>, _>("content_length").unwrap_or(None),
        "chunks_count": r.try_get::<Option<i32>, _>("chunks_count").unwrap_or(None),
        "last_crawled_at": r.try_get::<Option<String>, _>("last_crawled_at").unwrap_or(None),
        "next_crawl_at": r.try_get::<Option<String>, _>("next_crawl_at").unwrap_or(None),
        "crawl_interval_hours": r.try_get::<i32, _>("crawl_interval_hours").unwrap_or(168),
    })
}

// ---------------------------------------------------------------------------
// Settings
// ---------------------------------------------------------------------------

struct ImportSettings {
    chunk_size: usize,
    chunk_overlap: usize,
    user_agent: String,
}

async fn get_or_create_settings(state: &AppState, organization_id: &str) -> Result<ImportSettings, String> {
    let row = sqlx::query(
        "SELECT chunk_size, chunk_overlap, user_agent FROM kb_import_settings WHERE organization_id = $1",
    )
    .bind(organization_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| e.to_string())?;

    if let Some(r) = row {
        return Ok(ImportSettings {
            chunk_size: r.try_get::<i32, _>("chunk_size").unwrap_or(DEFAULT_CHUNK_SIZE as i32) as usize,
            chunk_overlap: r.try_get::<i32, _>("chunk_overlap").unwrap_or(DEFAULT_CHUNK_OVERLAP as i32) as usize,
            user_agent: r
                .try_get::<String, _>("user_agent")
                .unwrap_or_else(|_| "Communicator-KB-Crawler/1.0".to_string()),
        });
    }

    let id = Uuid::new_v4().to_string();
    sqlx::query("INSERT INTO kb_import_settings (id, organization_id) VALUES ($1, $2)")
        .bind(&id)
        .bind(organization_id)
        .execute(&state.db)
        .await
        .map_err(|e| e.to_string())?;

    Ok(ImportSettings {
        chunk_size: DEFAULT_CHUNK_SIZE,
        chunk_overlap: DEFAULT_CHUNK_OVERLAP,
        user_agent: "Communicator-KB-Crawler/1.0".to_string(),
    })
}

pub async fn get_settings(state: &AppState, organization_id: &str) -> Result<Value, String> {
    // Ensure a row exists, then read it back in full.
    get_or_create_settings(state, organization_id).await?;

    let row = sqlx::query(
        "SELECT id, organization_id, max_file_size_mb, allowed_file_types, chunk_size, \
         chunk_overlap, auto_create_kb_entries, crawl_interval_hours, max_url_depth, user_agent \
         FROM kb_import_settings WHERE organization_id = $1",
    )
    .bind(organization_id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| e.to_string())?;

    Ok(settings_row_to_json(&row))
}

pub async fn update_settings(
    state: &AppState,
    organization_id: &str,
    payload: &Value,
) -> Result<Value, String> {
    get_or_create_settings(state, organization_id).await?;

    let max_file_size_mb = payload.get("max_file_size_mb").and_then(|v| v.as_i64());
    let allowed_file_types = payload.get("allowed_file_types").map(|v| v.to_string());
    let chunk_size = payload.get("chunk_size").and_then(|v| v.as_i64());
    let chunk_overlap = payload.get("chunk_overlap").and_then(|v| v.as_i64());
    let auto_create_kb_entries = payload.get("auto_create_kb_entries").and_then(|v| v.as_bool());
    let crawl_interval_hours = payload.get("crawl_interval_hours").and_then(|v| v.as_i64());
    let max_url_depth = payload.get("max_url_depth").and_then(|v| v.as_i64());
    let user_agent = payload.get("user_agent").and_then(|v| v.as_str());

    let mut sets: Vec<String> = Vec::new();
    let mut idx = 1;
    macro_rules! add_set {
        ($present:expr, $col:expr) => {
            if $present {
                sets.push(format!("{} = ${}", $col, idx));
                idx += 1;
            }
        };
    }
    add_set!(max_file_size_mb.is_some(), "max_file_size_mb");
    add_set!(allowed_file_types.is_some(), "allowed_file_types");
    add_set!(chunk_size.is_some(), "chunk_size");
    add_set!(chunk_overlap.is_some(), "chunk_overlap");
    add_set!(auto_create_kb_entries.is_some(), "auto_create_kb_entries");
    add_set!(crawl_interval_hours.is_some(), "crawl_interval_hours");
    add_set!(max_url_depth.is_some(), "max_url_depth");
    add_set!(user_agent.is_some(), "user_agent");

    if !sets.is_empty() {
        sets.push("updated_at = NOW()".to_string());
        let query = format!(
            "UPDATE kb_import_settings SET {} WHERE organization_id = ${}",
            sets.join(", "),
            idx
        );

        let mut q = sqlx::query(&query);
        if let Some(v) = max_file_size_mb {
            q = q.bind(v as i32);
        }
        if let Some(ref v) = allowed_file_types {
            q = q.bind(v);
        }
        if let Some(v) = chunk_size {
            q = q.bind(v as i32);
        }
        if let Some(v) = chunk_overlap {
            q = q.bind(v as i32);
        }
        if let Some(v) = auto_create_kb_entries {
            q = q.bind(v);
        }
        if let Some(v) = crawl_interval_hours {
            q = q.bind(v as i32);
        }
        if let Some(v) = max_url_depth {
            q = q.bind(v as i32);
        }
        if let Some(v) = user_agent {
            q = q.bind(v);
        }
        q = q.bind(organization_id);

        q.execute(&state.db).await.map_err(|e| e.to_string())?;
    }

    get_settings(state, organization_id).await
}

fn settings_row_to_json(r: &sqlx::postgres::PgRow) -> Value {
    json!({
        "id": r.try_get::<String, _>("id").unwrap_or_default(),
        "organization_id": r.try_get::<String, _>("organization_id").unwrap_or_default(),
        "max_file_size_mb": r.try_get::<i32, _>("max_file_size_mb").unwrap_or(10),
        "allowed_file_types": r.try_get::<String, _>("allowed_file_types").unwrap_or_default(),
        "chunk_size": r.try_get::<i32, _>("chunk_size").unwrap_or(DEFAULT_CHUNK_SIZE as i32),
        "chunk_overlap": r.try_get::<i32, _>("chunk_overlap").unwrap_or(DEFAULT_CHUNK_OVERLAP as i32),
        "auto_create_kb_entries": r.try_get::<bool, _>("auto_create_kb_entries").unwrap_or(true),
        "crawl_interval_hours": r.try_get::<i32, _>("crawl_interval_hours").unwrap_or(168),
        "max_url_depth": r.try_get::<i32, _>("max_url_depth").unwrap_or(3),
        "user_agent": r.try_get::<String, _>("user_agent").unwrap_or_default(),
    })
}

// ---------------------------------------------------------------------------
// Text helpers
// ---------------------------------------------------------------------------

/// Split text into overlapping chunks of roughly `chunk_size` characters.
fn chunk_text(text: &str, chunk_size: usize, overlap: usize) -> Vec<String> {
    let text = text.trim();
    if text.is_empty() {
        return Vec::new();
    }
    if text.len() <= chunk_size {
        return vec![text.to_string()];
    }

    let chars: Vec<char> = text.chars().collect();
    let step = chunk_size.saturating_sub(overlap).max(1);
    let mut chunks = Vec::new();
    let mut start = 0;
    while start < chars.len() {
        let end = (start + chunk_size).min(chars.len());
        chunks.push(chars[start..end].iter().collect::<String>());
        if end == chars.len() {
            break;
        }
        start += step;
    }
    chunks
}

/// Very small HTML→text reducer: drops script/style blocks and tags,
/// then collapses whitespace. Good enough for crawled article/help pages.
fn html_to_text(html: &str) -> String {
    let scripts = Regex::new(r"(?is)<(script|style)[^>]*>.*?</\1>").unwrap();
    let without_scripts = scripts.replace_all(html, " ");
    let tags = Regex::new(r"(?s)<[^>]+>").unwrap();
    let without_tags = tags.replace_all(&without_scripts, " ");
    let decoded = without_tags
        .replace("&nbsp;", " ")
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'");
    let ws = Regex::new(r"\s+").unwrap();
    ws.replace_all(decoded.trim(), " ").to_string()
}

fn extract_title(html: &str) -> Option<String> {
    let re = Regex::new(r"(?is)<title[^>]*>(.*?)</title>").ok()?;
    re.captures(html)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str().trim().to_string())
        .filter(|s| !s.is_empty())
}
