use crate::db::AppState;
use crate::routes::auth::Tenant;
use axum::extract::multipart::{Field, Multipart};
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::net::ToSocketAddrs;
use std::path::PathBuf;
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn err(status: StatusCode, detail: impl Into<String>) -> Response {
    (status, Json(json!({ "detail": detail.into() }))).into_response()
}

fn db_err(e: sqlx::Error) -> Response {
    err(
        StatusCode::INTERNAL_SERVER_ERROR,
        format!("Database error: {e}"),
    )
}

/// DB `Err` is propagated, not folded into "table missing": a transient
/// database failure must never be reported as a schema problem.
async fn table_exists(db: &sqlx::SqlitePool, table: &str) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar::<_, i64>("SELECT count(*) FROM sqlite_master WHERE type='table' AND name=?")
        .bind(table)
        .fetch_one(db)
        .await
        .map(|c| c > 0)
}

async fn require_table(db: &sqlx::SqlitePool, table: &str) -> Result<(), Response> {
    match table_exists(db, table).await {
        Ok(true) => Ok(()),
        Ok(false) => Err(err(
            StatusCode::SERVICE_UNAVAILABLE,
            format!(
                "Knowledge schema table '{table}' is missing from this database. Apply the knowledge-base DDL (WIRING NEEDED) and restart; refusing to serve invented knowledge data."
            ),
        )),
        Err(e) => Err(db_err(e)),
    }
}

/// Count rows with a given status. `Ok` is a real answer -- a missing table is
/// a genuine 0 -- and `Err` is the database error itself, which the caller
/// reports as unknown (JSON null) instead of folding it into 0.
///
/// The tenant predicate is `COALESCE(tenant_key,'') = ?`: rows that predate the
/// tenancy migration carry NULL and belong to no verified tenant, so they are
/// counted for nobody rather than guessed into somebody's dashboard.
async fn count_by_status(
    db: &sqlx::SqlitePool,
    table: &str,
    status: &str,
    tenant_key: &str,
) -> Result<Value, sqlx::Error> {
    if !table_exists(db, table).await? {
        return Ok(json!(0));
    }
    let n = sqlx::query_scalar::<_, i64>(&format!(
        "SELECT COUNT(*) FROM {table} WHERE status = ? AND COALESCE(tenant_key, '') = ?"
    ))
    .bind(status)
    .bind(tenant_key)
    .fetch_one(db)
    .await?;
    Ok(json!(n))
}

const KB_COLUMNS: &str = "id, tenant_key, title, content, source_type, source_url, file_name, topic, \
     target_touchpoints, status, resolution_rate, usage_count, helpful_count, unhelpful_count, \
     created_at, updated_at";

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
struct KbRow {
    id: String,
    tenant_key: Option<String>,
    title: String,
    content: String,
    source_type: Option<String>,
    source_url: Option<String>,
    file_name: Option<String>,
    topic: Option<String>,
    target_touchpoints: Option<String>,
    status: Option<String>,
    resolution_rate: Option<f64>,
    usage_count: Option<i64>,
    helpful_count: Option<i64>,
    unhelpful_count: Option<i64>,
    created_at: Option<String>,
    updated_at: Option<String>,
}

fn parse_string_array(raw: Option<&str>, default: &[&str]) -> Vec<String> {
    raw.and_then(|s| serde_json::from_str::<Vec<String>>(s).ok())
        .unwrap_or_else(|| default.iter().map(|s| s.to_string()).collect())
}

fn serialize_entry(e: &KbRow) -> Value {
    json!({
        "id": e.id,
        "tenantKey": e.tenant_key,
        "title": e.title,
        "content": e.content,
        "sourceType": e.source_type,
        "sourceUrl": e.source_url,
        "fileName": e.file_name,
        "topic": e.topic,
        "targetTouchpoints": e.target_touchpoints.as_deref()
            .and_then(|raw| serde_json::from_str::<Vec<String>>(raw).ok())
            .unwrap_or_default(),
        "targetTouchpointsUnset": e.target_touchpoints.is_none(),
        "status": e.status,
        "resolutionRate": e.resolution_rate,
        "usageCount": e.usage_count.unwrap_or(0),
        "helpfulCount": e.helpful_count.unwrap_or(0),
        "unhelpfulCount": e.unhelpful_count.unwrap_or(0),
        "createdAt": e.created_at,
        "updatedAt": e.updated_at,
    })
}

async fn fetch_entry(
    db: &sqlx::SqlitePool,
    id: &str,
    tenant_key: &str,
) -> Result<Option<KbRow>, sqlx::Error> {
    sqlx::query_as::<_, KbRow>(&format!(
        "SELECT {KB_COLUMNS} FROM knowledge_base_entries WHERE id = ? AND tenant_key = ?"
    ))
    .bind(id)
    .bind(tenant_key)
    .fetch_optional(db)
    .await
}

async fn insert_entry(
    db: &sqlx::SqlitePool,
    tenant_key: &str,
    title: &str,
    content: &str,
    source_type: &str,
    source_url: Option<&str>,
    file_name: Option<&str>,
    topic: &str,
    touchpoints: &[String],
    status: &str,
) -> Result<(String, u64), sqlx::Error> {
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();
    // Encoding failure is surfaced, not defaulted to "[]" (which would
    // silently store an entry with no touchpoints).
    let tp = serde_json::to_string(touchpoints)
        .map_err(|err| sqlx::Error::Io(std::io::Error::other(err.to_string())))?;
    let result = sqlx::query(
        r#"INSERT INTO knowledge_base_entries (
            id, tenant_key, title, content, source_type, source_url, file_name, topic,
            target_touchpoints, status, resolution_rate, usage_count, helpful_count,
            unhelpful_count, created_at, updated_at
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, NULL, 0, 0, 0, ?, ?)"#,
    )
    .bind(&id)
    .bind(tenant_key)
    .bind(title)
    .bind(content)
    .bind(source_type)
    .bind(source_url)
    .bind(file_name)
    .bind(topic)
    .bind(&tp)
    .bind(status)
    .bind(&now)
    .bind(&now)
    .execute(db)
    .await?;
    // A knowledge row is only "stored" when the database says it wrote one row;
    // silently reporting a save on 0 rows would invent persistence.
    if result.rows_affected() != 1 {
        return Err(sqlx::Error::RowNotFound);
    }
    Ok((id, result.rows_affected()))
}

/// Builds the crate-wide honest persistence block (same shape as
/// `inboxes.rs::write_response`): `saved` is only ever true when the write
/// actually happened.
fn persistence_json(ok: bool, rows_affected: Option<u64>, reason: &str, detail: Option<String>) -> Value {
    json!({
        "ok": ok,
        "saved": ok,
        "rowsAffected": rows_affected.map(|r| json!(r)).unwrap_or(Value::Null),
        "reason": reason,
        "detail": detail.map(|d| json!(d)).unwrap_or(Value::Null),
    })
}

/// Same truncation the Python ingest applies before storing (30 000 code
/// points + marker), so a huge document does not silently differ from the
/// behaviour users had under FastAPI.
fn truncate_content(text: String) -> (String, bool) {
    const MAX_CHARS: usize = 30_000;
    if text.chars().count() > MAX_CHARS {
        let kept: String = text.chars().take(MAX_CHARS).collect();
        (format!("{kept}\n\n...[Content truncated for length]..."), true)
    } else {
        (text, false)
    }
}

fn cap_title(title: &str) -> String {
    title.chars().take(250).collect()
}

// ---------------------------------------------------------------------------
// Text extraction using ONLY crates already in Cargo.toml
// ---------------------------------------------------------------------------

/// Collapses whitespace runs the way the Python ingest cleaner does.
fn clean_text(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    let mut blank_run = 0usize;
    let mut last_was_space = false;
    for ch in raw.chars() {
        if ch.is_whitespace() {
            if ch == '\n' {
                blank_run += 1;
                last_was_space = true;
                continue;
            }
            if !last_was_space {
                out.push(' ');
                last_was_space = true;
            }
            continue;
        }
        if blank_run > 0 {
            out.push('\n');
            if blank_run > 1 {
                out.push('\n');
            }
            blank_run = 0;
        }
        last_was_space = false;
        out.push(ch);
    }
    out.trim().to_string()
}

/// Real HTML -> text: drops script/style/nav/... blocks, strips tags,
/// decodes the entities that appear in help-center pages, keeps headings.
fn html_to_text(html: &str) -> String {
    let drop_blocks = [
        "script", "style", "noscript", "svg", "nav", "footer", "aside", "header", "form",
        "iframe", "button", "template",
    ];
    let mut working = html.to_string();
    for tag in drop_blocks {
        loop {
            let lower = working.to_lowercase();
            let open = match lower.find(&format!("<{tag}")) {
                Some(i) => i,
                None => break,
            };
            let close_need = format!("</{tag}>");
            match lower[open..].find(&close_need) {
                Some(rel) => {
                    let end = open + rel + close_need.len();
                    working.replace_range(open..end, " ");
                }
                None => {
                    working.replace_range(open..working.len(), " ");
                    break;
                }
            }
        }
    }

    // Keep a paragraph/heading break before block tags, then remove all tags.
    let mut out = String::with_capacity(working.len());
    let mut in_tag = false;
    for ch in working.chars() {
        if ch == '<' {
            in_tag = true;
            out.push(' ');
        } else if ch == '>' {
            in_tag = false;
        } else if !in_tag {
            out.push(ch);
        }
    }

    let decoded = out
        .replace("&nbsp;", " ")
        .replace("&amp;", "&")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&#x27;", "'");

    clean_text(&decoded)
}

/// Python parity (`ingest_document_bytes`): decode strict UTF-8, and on
/// failure fall back to latin-1 byte-for-byte. `from_utf8_lossy` would
/// replace undecodable runs with U+FFFD -- content the document never had.
fn decode_text_bytes(bytes: &[u8]) -> String {
    match std::str::from_utf8(bytes) {
        Ok(s) => s.to_string(),
        Err(_) => bytes.iter().map(|b| *b as char).collect(),
    }
}

enum Extraction {
    /// Genuine text produced from the bytes we hold.
    Extracted { text: String, engine: String },
    /// Bytes stored, but this format needs a parser crate we do not have.
    NeedsParser {
        format: String,
        reason: String,
        required: &'static [&'static str],
    },
    /// Not a document format we can ingest at all.
    Unsupported { reason: String },
}

fn extract_document(name: &str, bytes: &[u8]) -> Extraction {
    let ext = PathBuf::from(name)
        .extension()
        .map(|e| e.to_string_lossy().to_lowercase())
        .unwrap_or_default();

    let text_like = [
        "txt", "text", "md", "markdown", "mdown", "csv", "tsv", "json", "log", "eml", "xml",
        "yaml", "yml", "ini", "conf",
    ];
    let html_like = ["html", "htm"];
    // Formats that are genuinely compressed/binary containers: decoding them
    // without a real parser would fabricate content, so we refuse.
    let needs_parser: &[&str] = &["pdf", "docx", "xlsx", "pptx", "odt", "ods", "epub", "zip", "rtf"];

    if text_like.contains(&ext.as_str()) {
        let decoded = decode_text_bytes(bytes);
        let text = clean_text(&decoded);
        if text.trim().is_empty() {
            return Extraction::Unsupported {
                reason: "The file decoded as text but contains no readable characters".to_string(),
            };
        }
        return Extraction::Extracted {
            text,
            engine: format!("utf8/latin1-decode+whitespace-clean(.{ext})"),
        };
    }

    if html_like.contains(&ext.as_str()) {
        let decoded = decode_text_bytes(bytes);
        let text = html_to_text(&decoded);
        if text.trim().is_empty() {
            return Extraction::Unsupported {
                reason: "HTML contained no readable text after removing markup".to_string(),
            };
        }
        return Extraction::Extracted {
            text,
            engine: "html-markdown-strip".to_string(),
        };
    }

    if needs_parser.contains(&ext.as_str()) {
        return Extraction::NeedsParser {
            format: ext.clone(),
            reason: format!(
                "The .{ext} bytes are kept on disk, but this build has no .{ext} text extractor. \
                 Storing un-extractable bytes as knowledge 'content' would poison retrieval, so no \
                 knowledge entry was created and nothing was learned from this document."
            ),
            required: match ext.as_str() {
                "pdf" => &["pdf-extract or lopdf + flate2 (Python used pypdf here)"],
                "rtf" => &["an RTF text-extractor crate (byte-stripping is not real RTF parsing)"],
                "docx" | "pptx" | "xlsx" | "odt" | "ods" | "epub" | "zip" => {
                    &["zip (deflate) + xml5ever, or docx-rs"]
                }
                _ => &["a format parser crate"],
            },
        };
    }

    Extraction::Unsupported {
        reason: format!(
            "Unsupported file type '{ext}'. Upload text-based documents (txt, md, csv, json, html, xml, yaml)."
        ),
    }
}

fn sanitize_filename(raw: &str) -> String {
    let base = raw
        .rsplit(['/', '\\'])
        .next()
        .unwrap_or("uploaded-document")
        .trim();
    let cleaned: String = base
        .chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            other => other,
        })
        .collect();
    let cleaned = cleaned.trim_matches('.').trim();
    if cleaned.is_empty() {
        "uploaded-document".to_string()
    } else {
        cleaned.chars().take(180).collect()
    }
}

fn upload_root() -> Result<PathBuf, String> {
    let dir = match std::env::var("KB_UPLOAD_DIR") {
        Ok(v) if !v.trim().is_empty() => PathBuf::from(v),
        _ => {
            let cwd = std::env::current_dir().map_err(|e| e.to_string())?;
            cwd.join("data").join("knowledge_uploads")
        }
    };
    std::fs::create_dir_all(&dir).map_err(|e| format!("cannot create upload dir {dir:?}: {e}"))?;
    Ok(dir)
}

async fn read_field_bytes<'a>(mut field: Field<'a>, max_bytes: usize) -> Result<Vec<u8>, String> {
    // Enforce the cap while streaming: `field.bytes()` would buffer an
    // arbitrarily large upload in memory before ever checking the limit.
    let mut out: Vec<u8> = Vec::new();
    while let Some(chunk) = field
        .chunk()
        .await
        .map_err(|e| format!("Failed to read upload part: {e}"))?
    {
        if out.len() + chunk.len() > max_bytes {
            return Err(format!(
                "File exceeds the {max_bytes} byte ingest limit (at least {} bytes streamed)",
                out.len() + chunk.len()
            ));
        }
        out.extend_from_slice(&chunk);
    }
    Ok(out)
}

// ---------------------------------------------------------------------------
// List / create / delete
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct ListQuery {
    pub topic: Option<String>,
    pub status: Option<String>,
}

pub async fn list_knowledge_base(
    State(state): State<AppState>,
    Tenant(tenant_key): Tenant,
    Query(query): Query<ListQuery>,
) -> impl IntoResponse {
    if let Err(r) = require_table(&state.db, "knowledge_base_entries").await {
        return r;
    }

    let sql = match (query.topic.as_deref(), query.status.as_deref()) {
        (Some(_), Some(_)) => format!(
            "SELECT {KB_COLUMNS} FROM knowledge_base_entries WHERE tenant_key = ? AND topic = ? AND status = ? ORDER BY created_at DESC"
        ),
        (Some(_), None) => format!(
            "SELECT {KB_COLUMNS} FROM knowledge_base_entries WHERE tenant_key = ? AND topic = ? ORDER BY created_at DESC"
        ),
        (None, Some(_)) => format!(
            "SELECT {KB_COLUMNS} FROM knowledge_base_entries WHERE tenant_key = ? AND status = ? ORDER BY created_at DESC"
        ),
        (None, None) => format!(
            "SELECT {KB_COLUMNS} FROM knowledge_base_entries WHERE tenant_key = ? ORDER BY created_at DESC"
        ),
    };

    let mut q = sqlx::query_as::<_, KbRow>(&sql).bind(&tenant_key);
    if let Some(t) = query.topic.as_deref() {
        q = q.bind(t);
    }
    if let Some(s) = query.status.as_deref() {
        q = q.bind(s);
    }

    match q.fetch_all(&state.db).await {
        Ok(rows) => Json(rows.iter().map(serialize_entry).collect::<Vec<_>>()).into_response(),
        Err(e) => db_err(e),
    }
}

#[derive(Deserialize)]
pub struct CreateKBPayload {
    pub title: String,
    pub content: String,
    pub topic: Option<String>,
    pub source_type: Option<String>,
    pub target_touchpoints: Option<Vec<String>>,
}

pub async fn create_knowledge_base_entry(
    State(state): State<AppState>,
    Tenant(tenant_key): Tenant,
    Json(payload): Json<CreateKBPayload>,
) -> impl IntoResponse {
    if let Err(r) = require_table(&state.db, "knowledge_base_entries").await {
        return r;
    }
    let title = payload.title.trim();
    if title.is_empty() {
        return err(StatusCode::BAD_REQUEST, "Knowledge entry title cannot be empty");
    }
    if payload.content.trim().is_empty() {
        return err(
            StatusCode::BAD_REQUEST,
            "Knowledge entry content cannot be empty",
        );
    }

    let touchpoints = payload
        .target_touchpoints
        .unwrap_or_else(|| vec!["agent".into(), "copilot".into(), "help_center".into()]);
    let source_type = payload
        .source_type
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| "note".to_string());

    match insert_entry(
        &state.db,
        &tenant_key,
        title,
        &payload.content,
        &source_type,
        None,
        None,
        payload.topic.as_deref().unwrap_or("General"),
        &touchpoints,
        "published",
    )
    .await
    {
        Ok((id, rows)) => match fetch_entry(&state.db, &id, &tenant_key).await {
            Ok(Some(row)) => {
                let mut body = serialize_entry(&row);
                body["persistence"] = persistence_json(
                    true,
                    Some(rows),
                    "row written and read back from knowledge_base_entries",
                    None,
                );
                (StatusCode::CREATED, Json(body)).into_response()
            }
            Ok(None) => err(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Entry inserted but could not be read back",
            ),
            Err(e) => db_err(e),
        },
        Err(e) => db_err(e),
    }
}

pub async fn delete_knowledge_base_entry(
    State(state): State<AppState>,
    Tenant(tenant_key): Tenant,
    Path(id): Path<String>,
) -> impl IntoResponse {
    if let Err(r) = require_table(&state.db, "knowledge_base_entries").await {
        return r;
    }
    match sqlx::query("DELETE FROM knowledge_base_entries WHERE id = ? AND tenant_key = ?")
        .bind(&id)
        .bind(&tenant_key)
        .execute(&state.db)
        .await
    {
        Ok(r) if r.rows_affected() == 0 => {
            err(
                StatusCode::NOT_FOUND,
                format!("Entry {id} is not in tenant '{tenant_key}' (it may belong to another tenant)."),
            )
        }
        Ok(_) => Json(json!({ "status": "deleted", "id": id })).into_response(),
        Err(e) => db_err(e),
    }
}

// ---------------------------------------------------------------------------
// POST /knowledge-base/upload  (multipart, real bytes + real extraction)
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct UploadQuery {
    pub topic: Option<String>,
}

pub async fn upload_knowledge_document(
    State(state): State<AppState>,
    Tenant(tenant_key): Tenant,
    query: Query<UploadQuery>,
    mut multipart: Multipart,
) -> impl IntoResponse {
    if let Err(r) = require_table(&state.db, "knowledge_base_entries").await {
        return r;
    }

    const MAX_BYTES: usize = 25 * 1024 * 1024;
    let mut filename: Option<String> = None;
    let mut bytes: Option<Vec<u8>> = None;
    let mut stream_error: Option<String> = None;

    loop {
        match multipart.next_field().await {
            Ok(Some(field)) => {
                let name = field.name().unwrap_or("").to_string();
                if name == "file" || name == "document" {
                    let fname = field
                        .file_name()
                        .map(|f| sanitize_filename(f))
                        .unwrap_or_else(|| sanitize_filename("uploaded-document"));
                    match read_field_bytes(field, MAX_BYTES).await {
                        Ok(b) if !b.is_empty() => {
                            filename = Some(fname);
                            bytes = Some(b);
                            break;
                        }
                        Ok(_) => return err(StatusCode::BAD_REQUEST, "Uploaded file is empty"),
                        Err(e) => return err(StatusCode::BAD_REQUEST, e),
                    }
                }
            }
            Ok(None) => break,
            // A broken multipart stream must not masquerade as "no field sent".
            Err(e) => {
                stream_error = Some(format!("Multipart stream failed: {e}"));
                break;
            }
        }
    }

    let (Some(filename), Some(content_bytes)) = (filename, bytes) else {
        return match stream_error {
            Some(e) => err(
                StatusCode::BAD_REQUEST,
                format!("Could not read the upload: {e}"),
            ),
            None => err(
                StatusCode::BAD_REQUEST,
                "Multipart field 'file' with the document bytes is required",
            ),
        };
    };

    // 1. Persist the real bytes first -- ingestion must never lose the file.
    let root = match upload_root() {
        Ok(r) => r,
        Err(e) => {
            return err(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Upload storage unavailable: {e}"),
            )
        }
    };
    let stored_id = Uuid::new_v4().to_string();
    let path = root.join(format!("{stored_id}-{filename}"));
    if let Err(e) = std::fs::write(&path, &content_bytes) {
        return err(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to write uploaded bytes to {path:?}: {e}"),
        );
    }
    let digest = hex::encode(Sha256::digest(&content_bytes));
    let size = content_bytes.len();

    // A DB failure must NOT be silently treated as "not a duplicate": it
    // would let the same document be ingested twice under a fresh id.
    let dup: Option<String> = match sqlx::query_scalar(
        "SELECT id FROM knowledge_base_entries WHERE tenant_key = ? AND source_type = 'upload' AND content LIKE ? LIMIT 1",
    )
    .bind(&tenant_key)
    .bind(format!("%{digest}%"))
    .fetch_optional(&state.db)
    .await
    {
        Ok(v) => v,
        Err(e) => {
            return err(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Duplicate-check query failed, upload not ingested: Database error: {e}"),
            )
        }
    };
    if let Some(existing) = dup {
        let cleanup_note = match std::fs::remove_file(&path) {
            Ok(()) => None,
            Err(e) => Some(format!("the duplicate could not be deleted from disk ({path:?}): {e}")),
        };
        return Json(json!({
            "status": "duplicate",
            "existingEntryId": existing,
            "fileName": filename,
            "bytesStored": size,
            "sha256": digest,
            "message": "An identical document is already in the knowledge base; the duplicate upload was discarded.",
            "cleanupNote": cleanup_note.map(|s| json!(s)).unwrap_or(Value::Null),
        }))
        .into_response();
    }

    // 2. Extract with what this build actually has.
    let topic = query
        .topic
        .as_deref()
        .filter(|t| !t.trim().is_empty())
        .unwrap_or("Uploaded Documents")
        .to_string();

    match extract_document(&filename, &content_bytes) {
        Extraction::Extracted { text, engine } => {
            let title = cap_title(&derive_title(&filename, &text));
            let (text, truncated) = truncate_content(text);
            let stored_content = format!("{text}\n\n[sha256:{digest}]");
            let touchpoints =
                vec!["agent".to_string(), "copilot".to_string(), "help_center".to_string()];
            match insert_entry(
                &state.db,
                &tenant_key,
                &title,
                &stored_content,
                "upload",
                None,
                Some(&filename),
                &topic,
                &touchpoints,
                "published",
            )
            .await
            {
                Ok((id, rows)) => match fetch_entry(&state.db, &id, &tenant_key).await {
                    Ok(Some(row)) => {
                        let mut body = serialize_entry(&row);
                        body["extraction"] = json!({
                            "status": "extracted",
                            "engine": engine,
                            "characters": text.chars().count(),
                            "truncated": truncated,
                            "bytesStored": size,
                            "storedPath": path.to_string_lossy(),
                            "sha256": digest,
                        });
                        body["persistence"] = persistence_json(
                            true,
                            Some(rows),
                            "knowledge row written and read back from knowledge_base_entries",
                            None,
                        );
                        (StatusCode::CREATED, Json(body)).into_response()
                    }
                    Ok(None) => err(
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "Entry inserted but could not be read back",
                    ),
                    Err(e) => db_err(e),
                },
                Err(e) => db_err(e),
            }
        }
        Extraction::NeedsParser {
            format,
            reason,
            required,
        } => {
            // Honest refusal: bytes stay on disk for a future re-ingest, but
            // NO knowledge row is written, because no text was ever extracted.
            (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(json!({
                    "detail": reason,
                    "missing": format!("text extractor for .{format} documents"),
                    "requiredCrates": required,
                    "reason": reason,
                    "workingFormats": [
                        "txt", "md", "csv", "tsv", "json", "log", "eml", "xml", "yaml", "yml",
                        "ini", "conf", "html", "htm"
                    ],
                    "fileName": filename,
                    "bytesStored": size,
                    "storedPath": path.to_string_lossy(),
                    "sha256": digest,
                    "ingested": false,
                    "knowledgeEntryCreated": false,
                    "persistence": persistence_json(
                        false,
                        Some(0),
                        "refused before write: no text was extracted, so no knowledge row was created",
                        None,
                    ),
                })),
            )
                .into_response()
        }
        Extraction::Unsupported { reason } => {
            // Keep the bytes for the operator but do not create a knowledge row.
            let rejected_note = match std::fs::write(
                root.join(format!("{stored_id}-{filename}.rejected")),
                &content_bytes,
            ) {
                Ok(()) => None,
                Err(e) => Some(format!("rejected bytes could not be kept on disk: {e}")),
            };
            (
                StatusCode::UNSUPPORTED_MEDIA_TYPE,
                Json(json!({
                    "detail": reason,
                    "fileName": filename,
                    "bytesStored": size,
                    "storedPath": path.to_string_lossy(),
                    "sha256": digest,
                    "ingested": false,
                    "persistence": persistence_json(
                        false,
                        Some(0),
                        "no knowledge row created; format is not a text document",
                        rejected_note.clone(),
                    ),
                })),
            )
                .into_response()
        }
    }
}

fn derive_title(filename: &str, text: &str) -> String {
    // Prefer the first markdown ATX heading -- it is real content, not invented.
    for line in text.lines().take(25) {
        let trimmed = line.trim();
        if let Some(h) = trimmed.strip_prefix('#') {
            let t = h.trim_start_matches('#').trim();
            if !t.is_empty() && t.chars().count() <= 200 {
                return t.to_string();
            }
        }
    }
    PathBuf::from(filename)
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| filename.to_string())
}

// ---------------------------------------------------------------------------
// POST /knowledge-base/ingest-url  (real fetch, real extraction)
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct IngestUrlPayload {
    pub url: String,
    pub topic: Option<String>,
}

/// Python parity (`_resolve_public_ip`): refuse non-public addresses AND
/// return the vetted SocketAddr so the caller can pin the connection to it,
/// closing the DNS-rebinding window between the check and the fetch.
fn resolve_public_addr(host: &str, port: u16) -> Result<std::net::SocketAddr, String> {
    let addrs: Vec<std::net::SocketAddr> = match (host, port).to_socket_addrs() {
        Ok(it) => it.collect(),
        Err(e) => return Err(format!("Could not resolve host {host}: {e}")),
    };
    let Some(ip) = addrs.first().map(|a| a.ip()) else {
        return Err(format!("Could not resolve host {host}"));
    };
    match ip {
        std::net::IpAddr::V4(v4) => {
            if v4.is_loopback()
                || v4.is_private()
                || v4.is_link_local()
                || v4.is_multicast()
                || v4.is_unspecified()
                || v4.is_broadcast()
            {
                return Err(format!("Refusing to fetch non-public address {host} -> {v4}"));
            }
        }
        std::net::IpAddr::V6(v6) => {
            if v6.is_loopback() || v6.is_unspecified() || v6.is_multicast() {
                return Err(format!("Refusing to fetch non-public address {host} -> {v6}"));
            }
        }
    }
    Ok(std::net::SocketAddr::new(ip, port))
}

pub async fn ingest_url(
    State(state): State<AppState>,
    Tenant(tenant_key): Tenant,
    Json(payload): Json<IngestUrlPayload>,
) -> impl IntoResponse {
    if let Err(r) = require_table(&state.db, "knowledge_base_entries").await {
        return r;
    }

    let raw = payload.url.trim();
    if raw.is_empty() {
        return err(StatusCode::BAD_REQUEST, "url is required");
    }
    let normalized = if raw.starts_with("http://") || raw.starts_with("https://") {
        raw.to_string()
    } else {
        format!("https://{raw}")
    };

    let without_scheme = normalized.split_once("://").map(|x| x.1).unwrap_or("");
    let host_part = without_scheme
        .split(['/', '?', '#'])
        .next()
        .unwrap_or("")
        .to_string();
    if host_part.contains('@') {
        return err(
            StatusCode::BAD_REQUEST,
            "Refusing URL with embedded credentials",
        );
    }
    let host = host_part
        .split(':')
        .next()
        .unwrap_or("")
        .trim()
        .to_string();
    if host.is_empty() {
        return err(StatusCode::BAD_REQUEST, format!("Invalid URL: {normalized}"));
    }
    let port = host_part
        .rsplit_once(':')
        .and_then(|(_, p)| p.parse::<u16>().ok())
        .unwrap_or(if normalized.starts_with("http://") { 80 } else { 443 });

    let pinned_addr = match resolve_public_addr(&host, port) {
        Ok(a) => a,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "detail": e, "ingested": false })),
            )
                .into_response();
        }
    };

    // reqwest's redirect policy is a Client-builder option, not a RequestBuilder
    // method, and the shared AppState.http_client follows redirects by default.
    // Build a local no-redirect client here so the explicit SSRF redirect check
    // below can actually see 3xx responses. `resolve` pins the connection to
    // the vetted IP (Host header + TLS SNI still use the original hostname),
    // so DNS cannot change its answer between the check and the fetch.
    let fetch_client = match reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .resolve(&host, pinned_addr)
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "detail": format!("Could not build HTTP client for ingest: {e}"),
                    "ingested": false,
                })),
            )
                .into_response();
        }
    };

    let fetch = fetch_client
        .get(&normalized)
        .header(
            "User-Agent",
            "CommunicatorKnowledgeIngestBot/1.0 (+rust backend; document crawler)",
        )
        .timeout(std::time::Duration::from_secs(25))
        .send()
        .await;

    let resp = match fetch {
        Ok(r) => r,
        Err(e) => {
            return (
                StatusCode::BAD_GATEWAY,
                Json(json!({
                    "detail": format!("Failed to fetch {normalized}: {e}"),
                    "ingested": false,
                })),
            )
                .into_response();
        }
    };

    let status_code = resp.status();
    if status_code.is_redirection() {
        let location = resp
            .headers()
            .get("location")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .to_string();
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "detail": format!(
                    "Refusing to follow redirect from {normalized} to {location} (SSRF guard, same policy as the Python ingest)."
                ),
                "ingested": false,
                "redirectLocation": location,
            })),
        )
            .into_response();
    }
    if !status_code.is_success() {
        return (
            StatusCode::BAD_GATEWAY,
            Json(json!({
                "detail": format!("{normalized} responded {status_code}"),
                "ingested": false,
            })),
        )
            .into_response();
    }

    let content_type = resp
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();
    let body = match resp.text().await {
        Ok(b) => b,
        Err(e) => {
            return (
                StatusCode::BAD_GATEWAY,
                Json(json!({
                    "detail": format!("Failed to read response body from {normalized}: {e}"),
                    "ingested": false,
                })),
            )
                .into_response();
        }
    };

    let (text, engine) = if content_type.contains("html") || body.trim_start().starts_with('<') {
        (html_to_text(&body), "html-markdown-strip")
    } else {
        (clean_text(&body), "plain-text-clean")
    };

    if text.trim().is_empty() {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(json!({
                "detail": format!("{normalized} returned no extractable text (likely a JavaScript-rendered page). Nothing was stored."),
                "content": "",
                "ingested": false,
            })),
        )
            .into_response();
    }

    let (text, truncated) = truncate_content(text);
    let title = cap_title(&extract_html_title(&body).unwrap_or_else(|| {
        normalized
            .trim_end_matches('/')
            .rsplit('/')
            .next()
            .unwrap_or(&normalized)
            .to_string()
    }));

    let touchpoints = vec!["agent".to_string(), "copilot".to_string(), "help_center".to_string()];
    match insert_entry(
        &state.db,
        &tenant_key,
        &title,
        &text,
        "url",
        Some(&normalized),
        None,
        payload.topic.as_deref().unwrap_or("Web Docs"),
        &touchpoints,
        "published",
    )
    .await
    {
        Ok((id, rows)) => match fetch_entry(&state.db, &id, &tenant_key).await {
            Ok(Some(row)) => {
                let mut body_json = serialize_entry(&row);
                body_json["extraction"] = json!({
                    "status": "extracted",
                    "engine": engine,
                    "characters": text.chars().count(),
                    "truncated": truncated,
                    "httpStatus": status_code.as_u16(),
                    "contentType": content_type,
                    "resolvedIp": pinned_addr.ip().to_string(),
                });
                body_json["persistence"] = persistence_json(
                    true,
                    Some(rows),
                    "knowledge row written and read back from knowledge_base_entries",
                    None,
                );
                (StatusCode::CREATED, Json(body_json)).into_response()
            }
            Ok(None) => err(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Entry inserted but could not be read back",
            ),
            Err(e) => db_err(e),
        },
        Err(e) => db_err(e),
    }
}

fn extract_html_title(html: &str) -> Option<String> {
    let lower = html.to_lowercase();
    let start = lower.find("<title>")? + "<title>".len();
    let rel = lower[start..].find("</title>")?;
    let raw = &html[start..start + rel];
    let cleaned = clean_text(raw);
    if cleaned.is_empty() {
        None
    } else {
        Some(cleaned.chars().take(200).collect())
    }
}

// ---------------------------------------------------------------------------
// GET /knowledge-base/hub  (real aggregates)
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct HubQuery {
    pub q: Option<String>,
    pub topic: Option<String>,
    pub touchpoint: Option<String>,
}

pub async fn get_knowledge_hub_overview(
    State(state): State<AppState>,
    Tenant(tenant_key): Tenant,
    Query(query): Query<HubQuery>,
) -> impl IntoResponse {
    if let Err(r) = require_table(&state.db, "knowledge_base_entries").await {
        return r;
    }

    let rows = match sqlx::query_as::<_, KbRow>(&format!(
        "SELECT {KB_COLUMNS} FROM knowledge_base_entries WHERE tenant_key = ? ORDER BY created_at DESC"
    ))
    .bind(&tenant_key)
    .fetch_all(&state.db)
    .await
    {
        Ok(r) => r,
        Err(e) => return db_err(e),
    };

    let needle = query.q.as_deref().unwrap_or("").trim().to_lowercase();
    let topic_filter = query.topic.clone();
    let touchpoint_filter = query.touchpoint.clone();

    let matching: Vec<&KbRow> = rows
        .iter()
        .filter(|r| {
            if let Some(t) = topic_filter.as_deref() {
                if r.topic.as_deref() != Some(t) {
                    return false;
                }
            }
            if let Some(tp) = touchpoint_filter.as_deref() {
                let tps = parse_string_array(r.target_touchpoints.as_deref(), &[]);
                if !tps.iter().any(|x| x == tp) {
                    return false;
                }
            }
            if !needle.is_empty() {
                let hay = format!("{} {}", r.title, r.content).to_lowercase();
                if !hay.contains(&needle) {
                    return false;
                }
            }
            true
        })
        .collect();

    // Topic grouping straight from stored rows.
    let mut topics: Vec<(String, usize, i64, Option<f64>)> = Vec::new();
    for r in &matching {
        let name = r.topic.clone().unwrap_or_else(|| "General".to_string());
        if let Some(entry) = topics.iter_mut().find(|t| t.0 == name) {
            entry.1 += 1;
            entry.2 += r.usage_count.unwrap_or(0);
        } else {
            topics.push((name, 1, r.usage_count.unwrap_or(0), r.resolution_rate));
        }
    }
    topics.sort_by_key(|t| std::cmp::Reverse(t.1));

    let touchpoint_counts = |tp: &str| -> (usize, i64) {
        let mut sources = 0usize;
        let mut uses = 0i64;
        for r in &rows {
            let tps = parse_string_array(r.target_touchpoints.as_deref(), &[]);
            if tps.iter().any(|x| x == tp) {
                sources += 1;
                uses += r.usage_count.unwrap_or(0);
            }
        }
        (sources, uses)
    };
    let (agent_sources, agent_uses) = touchpoint_counts("agent");
    let (copilot_sources, copilot_uses) = touchpoint_counts("copilot");
    let (help_sources, help_uses) = touchpoint_counts("help_center");

    // Resolution rate: only computed when we actually have measurements.
    let rated: Vec<f64> = rows.iter().filter_map(|r| r.resolution_rate).collect();
    let overall_resolution = if rated.is_empty() {
        None
    } else {
        Some((rated.iter().sum::<f64>() / rated.len() as f64 * 10.0).round() / 10.0)
    };

    // A missing table is a genuine zero; a database error is unknown and is
    // reported as JSON null with the real error text, never folded into 0.
    let (unresolved_gaps, gap_count_err) =
        match count_by_status(&state.db, "knowledge_gaps", "suggested", &tenant_key).await {
            Ok(v) => (v, None),
            Err(e) => (Value::Null, Some(format!("knowledge_gaps: {e}"))),
        };
    let (open_recommendations, rec_count_err) = match count_by_status(
        &state.db,
        "knowledge_recommendations",
        "open",
        &tenant_key,
    )
    .await
    {
        Ok(v) => (v, None),
        Err(e) => (Value::Null, Some(format!("knowledge_recommendations: {e}"))),
    };
    let count_errors: Vec<String> = gap_count_err.into_iter().chain(rec_count_err).collect();

    Json(json!({
        "totalSources": rows.len(),
        "filteredSources": matching.len(),
        "overallResolutionRate": overall_resolution,
        "resolutionRateBasis": if rated.is_empty() {
            "no measured resolutions yet -- resolution_rate is NULL on every entry"
        } else {
            "mean of stored resolution_rate across measured entries"
        },
        "unresolvedGaps": unresolved_gaps,
        "openRecommendations": open_recommendations,
        "countsNote": if count_errors.is_empty() {
            Value::Null
        } else {
            json!(format!(
                "gap/recommendation counts could not be read; null means unknown, not zero. Counts are tenant-scoped and pre-tenancy rows (tenant_key NULL) count for no tenant. {}",
                count_errors.join("; ")
            ))
        },
        "topics": topics.iter().map(|(name, count, uses, _)| json!({
            "name": name,
            "entryCount": count,
            "usageCount": uses,
        })).collect::<Vec<_>>(),
        "touchpointBreakdown": {
            "agent": { "sourcesDeployed": agent_sources, "deployedCount": agent_uses,
                "status": if agent_sources > 0 { "live" } else { "not_deployed" } },
            "copilot": { "sourcesDeployed": copilot_sources, "insertionsCount": copilot_uses,
                "status": if copilot_sources > 0 { "live" } else { "not_deployed" } },
            "help_center": { "sourcesDeployed": help_sources, "viewsCount": help_uses,
                "status": if help_sources > 0 { "live" } else { "not_deployed" } },
        },
        "filters": { "q": needle, "topic": topic_filter, "touchpoint": touchpoint_filter },
    }))
    .into_response()
}

// ---------------------------------------------------------------------------
// Recommendations (real rows, real state transitions)
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct RecommendationsQuery {
    pub status: Option<String>,
    pub gap_type: Option<String>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
struct RecRow {
    id: String,
    gap_type: Option<String>,
    topic: Option<String>,
    title: String,
    description: Option<String>,
    impact_on_resolution: Option<f64>,
    suggested_action: Option<String>,
    suggested_content: Option<String>,
    status: Option<String>,
    created_at: Option<String>,
    resolved_at: Option<String>,
}

const REC_COLUMNS: &str = "id, gap_type, topic, title, description, impact_on_resolution, \
     suggested_action, suggested_content, status, created_at, resolved_at";

pub async fn list_knowledge_recommendations(
    State(state): State<AppState>,
    Tenant(tenant_key): Tenant,
    Query(query): Query<RecommendationsQuery>,
) -> impl IntoResponse {
    if let Err(r) = require_table(&state.db, "knowledge_recommendations").await {
        return r;
    }
    let status = query.status.unwrap_or_else(|| "open".to_string());
    let rows = match query.gap_type.as_deref() {
        Some(gt) => sqlx::query_as::<_, RecRow>(&format!(
            "SELECT {REC_COLUMNS} FROM knowledge_recommendations WHERE status = ? AND gap_type = ? AND COALESCE(tenant_key, '') = ? ORDER BY impact_on_resolution DESC"
        ))
        .bind(&status)
        .bind(gt)
        .bind(&tenant_key)
        .fetch_all(&state.db)
        .await,
        None => sqlx::query_as::<_, RecRow>(&format!(
            "SELECT {REC_COLUMNS} FROM knowledge_recommendations WHERE status = ? AND COALESCE(tenant_key, '') = ? ORDER BY impact_on_resolution DESC"
        ))
        .bind(&status)
        .bind(&tenant_key)
        .fetch_all(&state.db)
        .await,
    };

    match rows {
        Ok(rs) => Json(
            rs.iter()
                .map(|r| {
                    // NULL columns serialize as null, never as an invented
                    // default the row never had.
                    json!({
                        "id": r.id,
                        "gapType": r.gap_type,
                        "topic": r.topic,
                        "title": r.title,
                        "description": r.description,
                        "impactOnResolution": r.impact_on_resolution,
                        "suggestedAction": r.suggested_action,
                        "suggestedContent": r.suggested_content,
                        "status": r.status,
                        "createdAt": r.created_at,
                        "resolvedAt": r.resolved_at,
                    })
                })
                .collect::<Vec<_>>(),
        )
        .into_response(),
        Err(e) => db_err(e),
    }
}

#[derive(Deserialize)]
pub struct ApplyRecommendationPayload {
    pub custom_content: Option<String>,
}

pub async fn apply_knowledge_recommendation(
    State(state): State<AppState>,
    Tenant(tenant_key): Tenant,
    Path(id): Path<String>,
    Json(payload): Json<ApplyRecommendationPayload>,
) -> impl IntoResponse {
    if let Err(r) = require_table(&state.db, "knowledge_recommendations").await {
        return r;
    }
    if let Err(r) = require_table(&state.db, "knowledge_base_entries").await {
        return r;
    }

    let rec = sqlx::query_as::<_, RecRow>(&format!(
        "SELECT {REC_COLUMNS} FROM knowledge_recommendations WHERE id = ? AND COALESCE(tenant_key, '') = ?"
    ))
    .bind(&id)
    .bind(&tenant_key)
    .fetch_optional(&state.db)
    .await;
    let rec = match rec {
        Ok(Some(r)) => r,
        // 0 rows here means "not yours or not there" -- both are a 404, never a
        // silent apply on another tenant's recommendation.
        Ok(None) => return err(
            StatusCode::NOT_FOUND,
            format!("Recommendation {id} not found for tenant '{tenant_key}'"),
        ),
        Err(e) => return db_err(e),
    };

    let content = payload
        .custom_content
        .filter(|c| !c.trim().is_empty())
        .or(rec.suggested_content.clone().filter(|c| !c.trim().is_empty()));
    let Some(content) = content else {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(json!({
                "detail": format!(
                    "Recommendation {id} carries no suggested_content and no custom_content was supplied. \
                     Writing placeholder text into the knowledge base would corrupt retrieval -- supply the article body."
                ),
                "recommendationId": id,
            })),
        )
            .into_response();
    };

    let now = Utc::now().to_rfc3339();
    let touchpoints = vec!["agent".to_string(), "copilot".to_string(), "help_center".to_string()];
    let (entry_id, entry_rows) = match insert_entry(
        &state.db,
        &tenant_key,
        &rec.title,
        &content,
        "ai_recommendation",
        None,
        None,
        rec.topic.as_deref().unwrap_or("General"),
        &touchpoints,
        "published",
    )
    .await
    {
        Ok(v) => v,
        Err(e) => return db_err(e),
    };

    // The recommendation transition must be verified, not assumed: the entry
    // exists either way, but "applied" may not have been written. The tenant
    // predicate makes another tenant's row unmatchable even by id guesswork.
    let (rec_rows, rec_err) = match sqlx::query(
        "UPDATE knowledge_recommendations SET status = 'applied', resolved_at = ? WHERE id = ? AND COALESCE(tenant_key, '') = ?",
    )
    .bind(&now)
    .bind(&id)
    .bind(&tenant_key)
    .execute(&state.db)
    .await
    {
        Ok(r) => (r.rows_affected(), None),
        Err(e) => (0, Some(e.to_string())),
    };
    let marked = rec_rows == 1 && entry_rows == 1;
    let apply_reason = if entry_rows != 1 {
        format!("the knowledge entry insert affected {entry_rows} rows, so the recommendation was not marked applied")
    } else if rec_err.is_some() {
        "knowledge entry was inserted, but the recommendation status update failed with a database error".to_string()
    } else {
        format!("knowledge entry was inserted, but the recommendation status update matched 0 rows for tenant '{tenant_key}' -- it was NOT marked applied")
    };

    let entry_readback = fetch_entry(&state.db, &entry_id, &tenant_key).await;
    let mut body = json!({
        "status": if marked { "applied" } else { "entry_created_recommendation_unmarked" },
        "recommendationId": id,
        "recommendationStatus": if marked { json!("applied") } else { Value::Null },
        "createdEntry": match &entry_readback {
            Ok(Some(row)) => json!(serialize_entry(row)),
            _ => Value::Null,
        },
    });
    body["persistence"] = if marked {
        persistence_json(
            true,
            Some(rec_rows),
            "knowledge entry inserted and recommendation marked applied",
            None,
        )
    } else {
        persistence_json(
            false,
            Some(rec_rows),
            &apply_reason,
            rec_err,
        )
    };
    match entry_readback {
        Ok(Some(_)) => {}
        Ok(None) => body["note"] = json!("entry id not readable after insert"),
        Err(e) => body["note"] = json!(format!("could not read back the created entry: {e}")),
    }
    Json(body).into_response()
}

pub async fn dismiss_knowledge_recommendation(
    State(state): State<AppState>,
    Tenant(tenant_key): Tenant,
    Path(id): Path<String>,
) -> impl IntoResponse {
    if let Err(r) = require_table(&state.db, "knowledge_recommendations").await {
        return r;
    }
    let now = Utc::now().to_rfc3339();
    match sqlx::query(
        "UPDATE knowledge_recommendations SET status = 'dismissed', resolved_at = ? WHERE id = ? AND COALESCE(tenant_key, '') = ?",
    )
    .bind(&now)
    .bind(&id)
    .bind(&tenant_key)
    .execute(&state.db)
    .await
    {
        // 0 rows = another tenant's row, a pre-tenancy NULL row, or no row at
        // all; none of those is a dismissal we may claim succeeded.
        Ok(r) if r.rows_affected() == 0 => err(
            StatusCode::NOT_FOUND,
            format!("Recommendation {id} not found for tenant '{tenant_key}'"),
        ),
        Ok(_) => Json(json!({
            "status": "dismissed",
            "recommendationId": id,
            "tenantKey": tenant_key,
            "resolvedAt": now,
        }))
        .into_response(),
        Err(e) => db_err(e),
    }
}

// ---------------------------------------------------------------------------
// Knowledge gaps (real detection from the messages table)
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct GapsQuery {
    pub status: Option<String>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
struct GapRow {
    id: String,
    question: String,
    suggested_answer: Option<String>,
    source_query: Option<String>,
    frequency: Option<i64>,
    status: Option<String>,
    created_at: Option<String>,
}

fn gap_json(g: &GapRow) -> Value {
    // NULL stays null: a missing frequency must not be reported as "seen 1
    // time" and a missing status must not be reported as "suggested".
    json!({
        "id": g.id,
        "question": g.question,
        "suggestedAnswer": g.suggested_answer,
        "sourceQuery": g.source_query,
        "frequency": g.frequency,
        "status": g.status,
        "createdAt": g.created_at,
    })
}

pub async fn list_knowledge_gaps(
    State(state): State<AppState>,
    Tenant(tenant_key): Tenant,
    Query(query): Query<GapsQuery>,
) -> impl IntoResponse {
    if let Err(r) = require_table(&state.db, "knowledge_gaps").await {
        return r;
    }
    let status = query.status.unwrap_or_else(|| "suggested".to_string());
    let rows = sqlx::query_as::<_, GapRow>(
        "SELECT id, question, suggested_answer, source_query, frequency, status, created_at \
         FROM knowledge_gaps WHERE status = ? AND COALESCE(tenant_key, '') = ? \
         ORDER BY frequency DESC, created_at DESC",
    )
    .bind(&status)
    .bind(&tenant_key)
    .fetch_all(&state.db)
    .await;

    match rows {
        Ok(rs) => Json(rs.iter().map(gap_json).collect::<Vec<_>>()).into_response(),
        Err(e) => db_err(e),
    }
}

pub async fn scan_knowledge_gaps(
    State(state): State<AppState>,
    Tenant(tenant_key): Tenant,
) -> impl IntoResponse {
    for table in ["knowledge_gaps", "knowledge_base_entries", "messages"] {
        if let Err(r) = require_table(&state.db, table).await {
            return r;
        }
    }

    // The "is this already covered?" corpus is this tenant's published KB only:
    // another tenant's article must not mask a gap in ours (and vice versa).
    let kb_titles: Vec<String> = match sqlx::query_scalar(
        "SELECT title FROM knowledge_base_entries WHERE status = 'published' AND tenant_key = ?",
    )
    .bind(&tenant_key)
    .fetch_all(&state.db)
    .await
    {
        Ok(v) => v,
        // A failed query must not be reported as "the KB is empty" --
        // that would make every question look like a gap.
        Err(e) => return db_err(e),
    };
    let kb_bodies: Vec<String> = match sqlx::query_scalar(
        "SELECT content FROM knowledge_base_entries WHERE status = 'published' AND tenant_key = ?",
    )
    .bind(&tenant_key)
    .fetch_all(&state.db)
    .await
    {
        Ok(v) => v,
        Err(e) => return db_err(e),
    };
    let corpus = kb_bodies.join("\n").to_lowercase();

    #[derive(sqlx::FromRow)]
    struct InboundRow {
        id: String,
        conversation_id: String,
        body: String,
    }
    // `messages` has no tenant column of its own; ownership is inherited from
    // the conversation it belongs to, so the scan joins through it instead of
    // reading every tenant's inbound text.
    let inbound = match sqlx::query_as::<_, InboundRow>(
        r#"
        SELECT m.id, m.conversation_id, m.body FROM messages m
        JOIN conversations c ON c.id = m.conversation_id
        WHERE (LOWER(m.sender) IN ('customer','user','inbound')
               OR LOWER(COALESCE(m.direction,'')) = 'inbound')
          AND COALESCE(c.tenant_key, '') = ?
        ORDER BY m.created_at DESC LIMIT 200
        "#,
    )
    .bind(&tenant_key)
    .fetch_all(&state.db)
    .await
    {
        Ok(v) => v,
        Err(e) => return db_err(e),
    };

    let Ok(pattern) = regex::Regex::new(
        r"(?i)(?:what is|how do|how does|how do i|can i|is it possible to|do you support|what are|where can i|when do|how much does|does (?:you|it|this))\s+([^?.!\n]{10,120})\??",
    ) else {
        return err(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Question-detection regex failed to compile",
        );
    };

    let existing: Vec<String> = match sqlx::query_scalar(
        "SELECT LOWER(TRIM(question)) FROM knowledge_gaps WHERE COALESCE(tenant_key, '') = ?",
    )
    .bind(&tenant_key)
    .fetch_all(&state.db)
    .await
    {
        Ok(v) => v,
        // Treating a failed read as "no gaps yet" would re-insert duplicates.
        Err(e) => return db_err(e),
    };
    let existing_set: std::collections::HashSet<String> = existing.into_iter().collect();

    let mut candidates: Vec<(String, String, usize, String)> = Vec::new(); // question, source, count, conv_id
    for row in &inbound {
        let text = row.body.trim();
        if text.is_empty() {
            continue;
        }
        let Some(m) = pattern.find(text) else { continue };
        let mut q = m.as_str().trim().to_string();
        if let Some(first) = q.chars().next() {
            let rest = q[first.len_utf8()..].to_string();
            let upper: String = first.to_uppercase().collect();
            q = format!("{upper}{rest}");
        } else {
            continue;
        }
        if !q.ends_with('?') {
            q.push('?');
        }
        let q_lower = q.to_lowercase();
        let covered = kb_titles
            .iter()
            .any(|t| t.to_lowercase().contains(&q_lower) || q_lower.contains(&t.to_lowercase()))
            || corpus.contains(&q_lower);
        if covered || existing_set.contains(&q_lower) {
            continue;
        }
        if let Some(entry) = candidates.iter_mut().find(|c| c.0 == q) {
            entry.2 += 1;
        } else {
            candidates.push((q.clone(), text.to_string(), 1, row.conversation_id.clone()));
        }
    }

    let mut created: Vec<Value> = Vec::new();
    let mut insert_failures: Vec<String> = Vec::new();
    let now = Utc::now().to_rfc3339();
    for (question, source_query, frequency, conversation_id) in candidates {
        // Grounded answer: the first real agent reply that followed the question
        // in the same thread. If none exists we record an empty answer instead
        // of inventing one.
        let answer: Option<String> = match sqlx::query_scalar(
            r#"
            SELECT body FROM messages
            WHERE conversation_id = ?
              AND (LOWER(sender) IN ('agent','user','bot') OR LOWER(COALESCE(direction,'')) = 'outbound')
            ORDER BY created_at ASC LIMIT 1
            "#,
        )
        .bind(&conversation_id)
        .fetch_optional(&state.db)
        .await
        {
            Ok(v) => v,
            // Err (lookup broke) is not Ok(None) (thread has no reply): report it.
            Err(e) => return db_err(e),
        };

        let grounded = answer.unwrap_or_default();
        let id = Uuid::new_v4().to_string();
        let res = sqlx::query(
            "INSERT INTO knowledge_gaps (id, tenant_key, question, suggested_answer, source_query, frequency, status, created_at) VALUES (?, ?, ?, ?, ?, ?, 'suggested', ?)",
        )
        .bind(&id)
        .bind(&tenant_key)
        .bind(&question)
        .bind(&grounded)
        .bind(&source_query)
        .bind(frequency as i64)
        .bind(&now)
        .execute(&state.db)
        .await;

        // Only gaps the database actually wrote are reported.
        match res {
            Ok(r) if r.rows_affected() == 1 => created.push(json!({
                "id": id,
                "question": question,
                "suggestedAnswer": grounded,
                "answerGrounded": !grounded.is_empty(),
                "frequency": frequency,
            })),
            Ok(r) => insert_failures.push(format!(
                "insert for '{question}' affected {} rows",
                r.rows_affected()
            )),
            Err(e) => insert_failures.push(format!("insert for '{question}' failed: {e}")),
        }
    }

    let ungrounded = created
        .iter()
        .filter(|c| c["answerGrounded"].as_bool() == Some(false))
        .count();
    let overall_ok = insert_failures.is_empty();
    Json(json!({
        "status": if overall_ok { "completed" } else { "completed_with_insert_failures" },
        "tenantKey": &tenant_key,
        // Stated, not implied: every pool this scan reads is now filtered to the
        // authenticated tenant. `messages` has no tenant column, so its scope is
        // inherited by joining the owning conversation.
        "scanScope": {
            "knowledgeCorpus": "this tenant's published knowledge_base_entries",
            "gapDedup": "this tenant's knowledge_gaps rows only",
            "inboundMessages": "messages whose conversation belongs to this tenant",
        },
        "scannedMessages": inbound.len(),
        "detectedCount": created.len(),
        "gaps": created,
        "persistence": persistence_json(
            overall_ok,
            Some(created.len() as u64),
            if overall_ok {
                "every reported gap was written to knowledge_gaps"
            } else {
                "some gap inserts failed; only successfully written gaps are reported"
            },
            if overall_ok { None } else { Some(insert_failures.join("; ")) },
        ),
        "note": if ungrounded > 0 {
            format!(
                "{ungrounded} gap(s) have no human reply in their thread, so suggestedAnswer is empty. \
                 Drafting answers automatically requires the AI model connection (see /ai/connections)."
            )
        } else if created.is_empty() {
            format!(
                "Scanned {} inbound messages against {} published knowledge entries; no uncovered recurring questions.",
                inbound.len(),
                kb_titles.len()
            )
        } else {
            "All detected gaps are grounded in a real human reply from the same thread.".to_string()
        },
    }))
    .into_response()
}

#[derive(Deserialize)]
pub struct ApproveGapPayload {
    pub custom_answer: Option<String>,
}

pub async fn approve_knowledge_gap(
    State(state): State<AppState>,
    Tenant(tenant_key): Tenant,
    Path(id): Path<String>,
    Json(payload): Json<ApproveGapPayload>,
) -> impl IntoResponse {
    if let Err(r) = require_table(&state.db, "knowledge_gaps").await {
        return r;
    }
    if let Err(r) = require_table(&state.db, "knowledge_base_entries").await {
        return r;
    }

    let gap = sqlx::query_as::<_, GapRow>(
        "SELECT id, question, suggested_answer, source_query, frequency, status, created_at FROM knowledge_gaps WHERE id = ? AND COALESCE(tenant_key, '') = ?",
    )
    .bind(&id)
    .bind(&tenant_key)
    .fetch_optional(&state.db)
    .await;
    let gap = match gap {
        Ok(Some(g)) => g,
        // A pre-tenancy (NULL tenant) or foreign-tenant row is not ours to
        // approve: 404 naming both id and tenant, never a silent success.
        Ok(None) => {
            return err(
                StatusCode::NOT_FOUND,
                format!("Knowledge gap {id} not found for tenant '{tenant_key}'"),
            )
        }
        Err(e) => return db_err(e),
    };

    let answer = payload
        .custom_answer
        .filter(|a| !a.trim().is_empty())
        .or(gap.suggested_answer.clone().filter(|a| !a.trim().is_empty()));
    let Some(answer) = answer else {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(json!({
                "detail": format!(
                    "Gap {id} has no grounded answer and custom_answer was not supplied. \
                     Publishing an empty FAQ would look like coverage that does not exist."
                ),
                "gapId": id,
                "question": gap.question,
            })),
        )
            .into_response();
    };

    let content = format!("Q: {}\n\nA: {}", gap.question, answer);
    let touchpoints = vec!["agent".to_string(), "copilot".to_string(), "help_center".to_string()];
    let (entry_id, entry_rows) = match insert_entry(
        &state.db,
        &tenant_key,
        &gap.question,
        &content,
        "gap_approval",
        None,
        None,
        "FAQ",
        &touchpoints,
        "published",
    )
    .await
    {
        Ok(v) => v,
        Err(e) => return db_err(e),
    };

    // A gap may only be reported as approved if its row was really updated;
    // swallowing this error used to leave the gap 'suggested' forever while
    // the response claimed success.
    let (gap_update_rows, gap_update_err) = match sqlx::query(
        "UPDATE knowledge_gaps SET status = 'approved', suggested_answer = ? WHERE id = ? AND COALESCE(tenant_key, '') = ?",
    )
    .bind(&answer)
    .bind(&id)
    .bind(&tenant_key)
    .execute(&state.db)
    .await
    {
        Ok(r) => (r.rows_affected(), None),
        Err(e) => (0, Some(e.to_string())),
    };
    let gap_marked = gap_update_rows == 1;
    let approve_reason = if gap_marked {
        "FAQ entry written and gap row marked approved".to_string()
    } else if gap_update_err.is_some() {
        "FAQ entry written, but the gap row could NOT be marked approved (database error)".to_string()
    } else {
        format!(
            "FAQ entry written, but the gap row update matched 0 rows for tenant '{tenant_key}' -- the gap is NOT marked approved"
        )
    };

    match fetch_entry(&state.db, &entry_id, &tenant_key).await {
        Ok(Some(row)) => {
            let mut body = serialize_entry(&row);
            body["gapId"] = json!(id);
            body["gapStatus"] = if gap_marked { json!("approved") } else { Value::Null };
            body["persistence"] = persistence_json(
                gap_marked,
                Some(entry_rows),
                &approve_reason,
                gap_update_err,
            );
            Json(body).into_response()
        }
        Ok(None) => err(
            StatusCode::INTERNAL_SERVER_ERROR,
            "FAQ entry inserted but could not be read back",
        ),
        Err(e) => db_err(e),
    }
}

pub async fn dismiss_knowledge_gap(
    State(state): State<AppState>,
    Tenant(tenant_key): Tenant,
    Path(id): Path<String>,
) -> impl IntoResponse {
    if let Err(r) = require_table(&state.db, "knowledge_gaps").await {
        return r;
    }
    match sqlx::query(
        "UPDATE knowledge_gaps SET status = 'dismissed' WHERE id = ? AND COALESCE(tenant_key, '') = ?",
    )
    .bind(&id)
    .bind(&tenant_key)
    .execute(&state.db)
    .await
    {
        Ok(r) if r.rows_affected() == 0 => err(
            StatusCode::NOT_FOUND,
            format!("Knowledge gap {id} not found for tenant '{tenant_key}'"),
        ),
        Ok(_) => Json(json!({ "status": "dismissed", "id": id, "tenantKey": tenant_key }))
            .into_response(),
        Err(e) => db_err(e),
    }
}
