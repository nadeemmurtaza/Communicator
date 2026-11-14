#![allow(dead_code)]

use crate::db::AppState;
use crate::routes::auth::Tenant;
use crate::services::signatures::secrets_match;
use axum::extract::{Multipart, Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Deserialize;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use sqlx::Row;

const MAX_MEDIA_BYTES: usize = 100 * 1024 * 1024;
const ALLOWED_PHOTO_MIMES: &[&str] = &["image/jpeg", "image/png", "image/webp"];
const ALLOWED_VIDEO_MIMES: &[&str] = &["video/webm", "video/mp4"];

fn rejected(status: StatusCode, detail: impl Into<String>) -> Response {
    (
        status,
        Json(json!({ "detail": detail.into() })),
    )
        .into_response()
}

/// uuid v4 draws from the OS CSPRNG, which is what these need to be: nothing in
/// this handler derives a token from time or from a counter.
fn high_entropy(prefix: &str, parts: usize) -> String {
    let mut out = String::from(prefix);
    for _ in 0..parts {
        out.push_str(&uuid::Uuid::new_v4().simple().to_string());
    }
    out
}

fn now_iso() -> String {
    chrono::Utc::now().to_rfc3339()
}

fn iso_ahead(minutes: i64) -> String {
    chrono::Utc::now()
        .checked_add_signed(chrono::Duration::minutes(minutes))
        .map(|t| t.to_rfc3339())
        .unwrap_or_else(now_iso)
}

#[derive(Debug, Clone)]
struct ProofSession {
    id: String,
    tenant_id: String,
    organization_id: Option<String>,
    order_id: Option<String>,
    customer_identifier: String,
    claim_type: String,
    session_token: String,
    status: String,
    anti_spoof_nonce: String,
    max_video_seconds: i64,
    require_video: bool,
    require_photos: bool,
    expires_at: String,
    completed_at: Option<String>,
    rejection_reason: Option<String>,
}

async fn fetch_session(pool: &sqlx::PgPool, id: &str) -> Result<Option<ProofSession>, sqlx::Error> {
    let row = sqlx::query(
        "SELECT id, tenant_id, organization_id, order_id, customer_identifier, claim_type,
                session_token, status, anti_spoof_nonce, max_video_seconds, require_video,
                require_photos, expires_at, completed_at, rejection_reason
         FROM live_proof_sessions WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;

    let Some(row) = row else { return Ok(None) };
    let flag = |column: &str| -> bool {
        row.try_get::<Option<i64>, _>(column).ok().flatten().unwrap_or(0) != 0
    };
    let text = |column: &str| -> String {
        row.try_get::<Option<String>, _>(column)
            .ok()
            .flatten()
            .unwrap_or_default()
    };
    Ok(Some(ProofSession {
        id: text("id"),
        tenant_id: text("tenant_id"),
        organization_id: row.try_get("organization_id").ok(),
        order_id: row.try_get("order_id").ok(),
        customer_identifier: text("customer_identifier"),
        claim_type: text("claim_type"),
        session_token: text("session_token"),
        status: text("status"),
        anti_spoof_nonce: text("anti_spoof_nonce"),
        max_video_seconds: match row.try_get::<i64, _>("max_video_seconds") {
            Ok(value) => value,
            // WHY: the video-length gate must come from the stored session; an
            // unreadable column is reported and only then falls back to the
            // documented default, so a tenant never asked for it silently.
            Err(err) => {
                tracing::warn!(
                    "Session {} has no readable max_video_seconds ({err}); applying the {}s default",
                    row.try_get::<String, _>("id").unwrap_or_default(),
                    DEFAULT_MAX_VIDEO_SECONDS
                );
                DEFAULT_MAX_VIDEO_SECONDS
            }
        },
        require_video: flag("require_video"),
        require_photos: flag("require_photos"),
        expires_at: text("expires_at"),
        completed_at: row.try_get("completed_at").ok(),
        rejection_reason: row.try_get("rejection_reason").ok(),
    }))
}

/// The customer-facing checks the Python service ran, in the same order, with
/// the same status codes. Expiry is evaluated from the stored timestamp; a
/// timestamp the server cannot parse is treated as expired, because the
/// alternative is accepting a session whose deadline is unknown.
async fn validate(
    state: &AppState,
    session_id: &str,
    token: &str,
    for_upload: bool,
) -> Result<ProofSession, Response> {
    let Some(session) = fetch_session(&state.db, session_id)
        .await
        .map_err(|err| {
            rejected(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Could not read the verification session: {err}"),
            )
        })?
    else {
        return Err(rejected(
            StatusCode::NOT_FOUND,
            format!("Verification session {session_id} not found."),
        ));
    };

    if !secrets_match(&session.session_token, Some(token)) {
        return Err(rejected(
            StatusCode::FORBIDDEN,
            "Invalid verification token.",
        ));
    }

    let expired = chrono::DateTime::parse_from_rfc3339(&session.expires_at)
        .map(|ends| ends.with_timezone(&chrono::Utc) <= chrono::Utc::now())
        .unwrap_or(true);
    if expired {
        if session.status != "expired" {
            // Auxiliary: the 410 below is derived from the stored `expires_at`,
            // not from this write, so a failure is logged and reported but does
            // not change the answer the customer gets.
            crate::db::report_write(
                sqlx::query("UPDATE live_proof_sessions SET status = 'expired', updated_at = $1 WHERE id = $2")
                    .bind(now_iso())
                    .bind(&session.id)
                    .execute(&state.db)
                    .await
                    .map(|report| report.rows_affected()),
                "live proof session marked expired on first overdue request",
            );
        }
        return Err(rejected(
            StatusCode::GONE,
            "This verification session has expired. Please ask support for a new link.",
        ));
    }

    if session.status == "rejected" {
        return Err(rejected(
            StatusCode::FORBIDDEN,
            format!(
                "This session was rejected: {}",
                session
                    .rejection_reason
                    .clone()
                    .unwrap_or_else(|| "no reason recorded".to_string())
            ),
        ));
    }
    if for_upload && matches!(session.status.as_str(), "captured" | "verified") {
        return Err(rejected(
            StatusCode::CONFLICT,
            "Evidence collection for this session has already been completed and submitted.",
        ));
    }
    Ok(session)
}

#[derive(Deserialize)]
pub struct CreateSessionRequest {
    tenant_id: Option<String>,
    #[serde(alias = "organizationId")]
    organization_id: Option<String>,
    #[serde(alias = "conversationId")]
    conversation_id: Option<String>,
    #[serde(alias = "contactId")]
    contact_id: Option<String>,
    #[serde(alias = "orderId")]
    order_id: Option<String>,
    #[serde(alias = "customerIdentifier")]
    customer_identifier: Option<String>,
    #[serde(alias = "claimType")]
    claim_type: Option<String>,
    #[serde(alias = "maxVideoSeconds")]
    max_video_seconds: Option<i64>,
    #[serde(alias = "ttlMinutes")]
    ttl_minutes: Option<i64>,
    #[serde(alias = "requireVideo")]
    require_video: Option<bool>,
    #[serde(alias = "requirePhotos")]
    require_photos: Option<bool>,
}

#[derive(Deserialize)]
pub struct TokenQuery {
    token: Option<String>,
}

/// The link handed to the customer has to point back at this server, so it is
/// built from the request itself when PUBLIC_URL is not configured — never from
/// a compiled-in domain.
fn request_base(headers: &HeaderMap) -> String {
    let host = headers
        .get("x-forwarded-host")
        .or_else(|| headers.get("host"))
        .and_then(|v| v.to_str().ok())
        .unwrap_or("localhost");
    let scheme = header_str(headers, "x-forwarded-proto").unwrap_or("http");
    format!("{scheme}://{host}")
}

fn header_str<'a>(headers: &'a HeaderMap, name: &str) -> Option<&'a str> {
    headers.get(name).and_then(|v| v.to_str().ok())
}

/// Mirrors the column defaults in the `live_proof_sessions` DDL, which in turn
/// mirror the Python service's own parameter defaults.
pub const DEFAULT_TTL_MINUTES: i64 = 30;
pub const DEFAULT_MAX_VIDEO_SECONDS: i64 = 45;
/// Mirrors `claim_type TEXT NOT NULL DEFAULT 'refund'` in the DDL.
pub const DEFAULT_CLAIM_TYPE: &str = "refund";

/// The part of `create_session` that other Rust code (the reply engine) needs:
/// mint the tokens and write the pending row. Only the row — building a link
/// needs the request's own host, which is a handler concern.
pub struct NewLiveProofSession {
    pub tenant_id: String,
    pub organization_id: Option<String>,
    pub conversation_id: Option<String>,
    pub contact_id: Option<String>,
    pub order_id: Option<String>,
    pub customer_identifier: String,
    pub claim_type: String,
    pub ttl_minutes: i64,
    pub max_video_seconds: i64,
    pub require_video: bool,
    pub require_photos: bool,
}

pub struct CreatedLiveProofSession {
    pub id: String,
    pub session_token: String,
    pub expires_at: String,
}

pub async fn create_pending_session(
    state: &AppState,
    spec: &NewLiveProofSession,
) -> Result<CreatedLiveProofSession, String> {
    let id = uuid::Uuid::new_v4().to_string();
    let session_token = high_entropy("", 2);
    let anti_spoof_nonce = high_entropy("", 1);
    let now = now_iso();
    let expires_at = iso_ahead(spec.ttl_minutes);

    let result = sqlx::query(
        "INSERT INTO live_proof_sessions (id, tenant_id, organization_id, conversation_id,
            contact_id, order_id, customer_identifier, claim_type, session_token, status,
            anti_spoof_nonce, max_video_seconds, require_video, require_photos, expires_at,
            completed_at, rejection_reason, created_at, updated_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, 'pending', $10, $11, $12, $13, $14, NULL, NULL, $15, $16)",
    )
    .bind(&id)
    .bind(&spec.tenant_id)
    .bind(spec.organization_id.as_deref())
    .bind(spec.conversation_id.as_deref())
    .bind(spec.contact_id.as_deref())
    .bind(spec.order_id.as_deref())
    .bind(&spec.customer_identifier)
    .bind(&spec.claim_type)
    .bind(&session_token)
    .bind(&anti_spoof_nonce)
    .bind(spec.max_video_seconds)
    .bind(spec.require_video)
    .bind(spec.require_photos)
    .bind(&expires_at)
    .bind(&now)
    .bind(&now)
    .execute(&state.db)
    .await
    .map_err(|err| err.to_string())?;

    if result.rows_affected() != 1 {
        return Err(format!(
            "the session row was not written ({} rows affected)",
            result.rows_affected()
        ));
    }

    Ok(CreatedLiveProofSession {
        id,
        session_token,
        expires_at,
    })
}

/// `POST /live-proof/sessions`. A session belongs to the account that asked for
/// it, so the tenant comes from the body or the request's tenant scope; there is
/// no fallback tenant and a claim recorded against the wrong account is worse
/// than a refusal.
pub async fn create_session(
    State(state): State<AppState>,
    Tenant(header_tenant): Tenant,
    headers: HeaderMap,
    payload: Option<Json<CreateSessionRequest>>,
) -> Response {
    let Some(payload) = payload.map(|Json(payload)| payload) else {
        return rejected(
            StatusCode::BAD_REQUEST,
            "A JSON body is required to open a verification session.",
        );
    };

    let tenant_id = payload
        .tenant_id
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(str::to_string)
        .unwrap_or(header_tenant);
    if tenant_id.is_empty() {
        return rejected(
            StatusCode::BAD_REQUEST,
            "No tenant: the body has no tenant_id and the request carries no tenant scope.",
        );
    }

    let ttl = payload.ttl_minutes.unwrap_or(DEFAULT_TTL_MINUTES);
    if !(1..=1440).contains(&ttl) {
        return rejected(
            StatusCode::BAD_REQUEST,
            format!("ttl_minutes must be between 1 and 1440, received {ttl}."),
        );
    }
    let max_video = payload.max_video_seconds.unwrap_or(DEFAULT_MAX_VIDEO_SECONDS);
    if !(5..=120).contains(&max_video) {
        return rejected(
            StatusCode::BAD_REQUEST,
            format!("max_video_seconds must be between 5 and 120, received {max_video}."),
        );
    }

    let require_video = payload.require_video.unwrap_or(true);
    let require_photos = payload.require_photos.unwrap_or(true);
    if !require_video && !require_photos {
        return rejected(
            StatusCode::BAD_REQUEST,
            "A session that accepts neither video nor photos cannot verify anything.",
        );
    }

    let spec = NewLiveProofSession {
        tenant_id: tenant_id.clone(),
        organization_id: payload.organization_id.clone(),
        conversation_id: payload.conversation_id.clone(),
        contact_id: payload.contact_id.clone(),
        order_id: payload.order_id.clone(),
        customer_identifier: payload.customer_identifier.clone().unwrap_or_default(),
        claim_type: payload.claim_type.clone().unwrap_or_else(|| DEFAULT_CLAIM_TYPE.to_string()),
        ttl_minutes: ttl,
        max_video_seconds: max_video,
        require_video,
        require_photos,
    };

    let created = match create_pending_session(&state, &spec).await {
        Ok(created) => created,
        Err(reason) => {
            tracing::error!("LiveProof session create failed for tenant {tenant_id}: {reason}");
            return rejected(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Could not create the verification session: {reason}"),
            );
        }
    };
    let id = created.id;
    let session_token = created.session_token;
    let expires_at = created.expires_at;

    let root = if state.config.public_url.trim().is_empty() {
        request_base(&headers)
    } else {
        state.config.public_url.trim().trim_end_matches('/').to_string()
    };
    tracing::info!("Created LiveProof session {id} for tenant {tenant_id}");

    Json(json!({
        "sessionId": id,
        "verificationUrl": format!("{root}/verify/proof/{id}?token={session_token}"),
        "status": "pending",
        "expiresAt": expires_at,
        "tenantId": tenant_id,
        "maxVideoSeconds": max_video,
        "requireVideo": require_video,
        "requirePhotos": require_photos,
    }))
    .into_response()
}

fn evidence_list_sql() -> &'static str {
    "SELECT id, session_id, media_type, sha256_hash, file_size_bytes, duration_seconds,
            verification_score, watermark_verified, mime_type, resolution, captured_at
     FROM live_proof_evidences WHERE session_id = $1 ORDER BY captured_at"
}

async fn evidences(state: &AppState, session_id: &str) -> Vec<Value> {
    let rows = sqlx::query(evidence_list_sql())
        .bind(session_id)
        .fetch_all(&state.db)
        .await
        .unwrap_or_default();
    rows.into_iter()
        .map(|row| {
            json!({
                "id": row.try_get::<String, _>("id").unwrap_or_default(),
                "sessionId": row.try_get::<String, _>("session_id").unwrap_or_default(),
                "mediaType": row.try_get::<String, _>("media_type").unwrap_or_default(),
                "sha256Hash": row.try_get::<String, _>("sha256_hash").unwrap_or_default(),
                "fileSizeBytes": row.try_get::<i64, _>("file_size_bytes").unwrap_or(0),
                "durationSeconds": row.try_get::<Option<f64>, _>("duration_seconds").ok().flatten(),
                "verificationScore": row.try_get::<f64, _>("verification_score").unwrap_or(0.0),
                "watermarkVerified": row.try_get::<i64, _>("watermark_verified").map(|v| v != 0).unwrap_or(false),
                "mimeType": row.try_get::<String, _>("mime_type").unwrap_or_default(),
                "resolution": row.try_get::<Option<String>, _>("resolution").ok().flatten(),
                "capturedAt": row.try_get::<String, _>("captured_at").unwrap_or_default(),
            })
        })
        .collect()
}

pub async fn get_session_details(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
    Query(params): Query<TokenQuery>,
) -> Response {
    let token = params.token.unwrap_or_default();
    let session = match validate(&state, &session_id, &token, false).await {
        Ok(session) => session,
        Err(response) => return response,
    };
    Json(json!({
        "id": session.id,
        "tenantId": session.tenant_id,
        "organizationId": session.organization_id,
        "orderId": session.order_id,
        "claimType": session.claim_type,
        "status": session.status,
        "expiresAt": session.expires_at,
        "completedAt": session.completed_at,
        "evidences": evidences(&state, &session.id).await,
    }))
    .into_response()
}

#[derive(Deserialize)]
pub struct DecisionRequest {
    decision: String,
    #[serde(alias = "rejectionReason")]
    rejection_reason: Option<String>,
}

/// Staff verdict on a captured session. Requires the session token because this
/// backend has no authenticated staff session to check against yet.
pub async fn record_decision(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
    Query(params): Query<TokenQuery>,
    Json(payload): Json<DecisionRequest>,
) -> Response {
    let token = params.token.unwrap_or_default();
    let session = match validate(&state, &session_id, &token, false).await {
        Ok(session) => session,
        Err(response) => return response,
    };
    if !matches!(session.status.as_str(), "captured" | "in_progress" | "pending") {
        return rejected(
            StatusCode::CONFLICT,
            format!("Session is already {} and cannot be re-decided.", session.status),
        );
    }

    let decision = payload.decision.trim().to_ascii_uppercase();
    let (new_status, reason) = match decision.as_str() {
        "APPROVED" | "VERIFIED" => ("verified", Value::Null),
        "REJECTED" => (
            "rejected",
            json!(payload
                .rejection_reason
                .clone()
                .filter(|v| !v.trim().is_empty())
                .unwrap_or_else(|| "No reason supplied by the reviewer.".to_string())),
        ),
        other => {
            return rejected(
                StatusCode::BAD_REQUEST,
                format!("Unknown decision '{other}'. Expected APPROVED or REJECTED."),
            )
        }
    };

    let result = sqlx::query(
        "UPDATE live_proof_sessions SET status = $1, rejection_reason = COALESCE($2, rejection_reason),
                updated_at = $3 WHERE id = $4",
    )
    .bind(new_status)
    .bind(payload.rejection_reason.clone())
    .bind(now_iso())
    .bind(&session.id)
    .execute(&state.db)
    .await;

    match result {
        Ok(_) => {
            tracing::info!("LiveProof session {} decided {new_status}", session.id);
            Json(json!({
                "sessionId": session.id,
                "status": new_status,
                "rejectionReason": reason,
                "evidences": evidences(&state, &session.id).await,
            }))
            .into_response()
        }
        Err(err) => rejected(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Could not record the decision: {err}"),
        ),
    }
}

fn normalise_mime(raw: &str) -> String {
    raw.split(';').next().unwrap_or("").trim().to_ascii_lowercase()
}

fn extension_for(mime: &str, media_type: &str) -> &'static str {
    match mime {
        "image/jpeg" => "jpg",
        "image/png" => "png",
        "image/webp" => "webp",
        "video/webm" => "webm",
        "video/mp4" => "mp4",
        _ if media_type == "live_photo" => "jpg",
        _ => "bin",
    }
}

fn sanitize_path_segment(input: &str) -> String {
    let cleaned: String = input
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
        .collect();
    cleaned.chars().take(64).collect()
}

pub async fn upload_evidence(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
    Query(params): Query<TokenQuery>,
    headers: HeaderMap,
    mut multipart: Multipart,
) -> Response {
    let token = params.token.clone().unwrap_or_default();
    // Validate before buffering any bytes off the wire.
    let session = match validate(&state, &session_id, &token, true).await {
        Ok(session) => session,
        Err(response) => return response,
    };

    let mut media_type = String::new();
    let mut watermark_nonce = String::new();
    let mut telemetry_raw: Option<String> = None;
    let mut mime = String::new();
    let mut bytes: Vec<u8> = Vec::new();
    let mut saw_file = false;

    loop {
        let mut field = match multipart.next_field().await {
            Ok(Some(field)) => field,
            Ok(None) => break,
            Err(err) => {
                return rejected(
                    StatusCode::BAD_REQUEST,
                    format!("Malformed multipart upload: {err}"),
                )
            }
        };
        let name = field.name().unwrap_or_default().to_string();
        match name.as_str() {
            "file" => {
                saw_file = true;
                mime = normalise_mime(field.content_type().unwrap_or("application/octet-stream"));
                // Read in chunks so an oversized upload is refused mid-stream
                // rather than buffered first.
                loop {
                    match field.chunk().await {
                        Ok(Some(chunk)) => {
                            if bytes.len() + chunk.len() > MAX_MEDIA_BYTES {
                                return rejected(
                                    StatusCode::PAYLOAD_TOO_LARGE,
                                    format!(
                                        "Media size exceeds the {}MB limit.",
                                        MAX_MEDIA_BYTES / (1024 * 1024)
                                    ),
                                );
                            }
                            bytes.extend_from_slice(&chunk);
                        }
                        Ok(None) => break,
                        Err(err) => {
                            return rejected(
                                StatusCode::BAD_REQUEST,
                                format!("Upload aborted: {err}"),
                            )
                        }
                    }
                }
            }
            "media_type" => media_type = field.text().await.unwrap_or_default().trim().to_string(),
            "watermark_nonce" => watermark_nonce = field.text().await.unwrap_or_default(),
            "telemetry" => telemetry_raw = field.text().await.ok(),
            _ => {}
        }
    }

    if !saw_file {
        return rejected(StatusCode::BAD_REQUEST, "No media file part in the upload.");
    }
    if !matches!(media_type.as_str(), "live_photo" | "live_video") {
        return rejected(
            StatusCode::BAD_REQUEST,
            format!("media_type must be live_photo or live_video, received '{media_type}'."),
        );
    }
    if bytes.is_empty() {
        return rejected(StatusCode::BAD_REQUEST, "Captured media is empty.");
    }

    let allowed = if media_type == "live_photo" { ALLOWED_PHOTO_MIMES } else { ALLOWED_VIDEO_MIMES };
    if !allowed.contains(&mime.as_str()) {
        return rejected(
            StatusCode::UNSUPPORTED_MEDIA_TYPE,
            format!(
                "Unsupported {} MIME type: {mime}. {}",
                if media_type == "live_photo" { "photo" } else { "video" },
                if media_type == "live_photo" {
                    "Must be JPEG, PNG, or WebP."
                } else {
                    "Must be WebM or MP4."
                }
            ),
        );
    }

    let telemetry: Value = telemetry_raw
        .as_deref()
        .and_then(|raw| serde_json::from_str(raw).ok())
        .unwrap_or_else(|| json!(null));
    let telemetry_is_object = telemetry.is_object();

    // Anti-gallery-spoof gate: the markers must be in telemetry the client sent.
    let live_marker = telemetry.get("is_live_stream").and_then(|v| v.as_bool()) == Some(true);
    let source_marker = telemetry
        .get("capture_source")
        .and_then(|v| v.as_str())
        .map(|v| v == "live_webrtc_camera")
        .unwrap_or(false);
    if !(live_marker || source_marker) {
        tracing::warn!("LiveProof upload rejected for {session_id}: no live camera telemetry.");
        return rejected(
            StatusCode::BAD_REQUEST,
            "File upload from the phone gallery is prohibited. Capture must come from the live camera.",
        );
    }

    if watermark_nonce.trim().is_empty() {
        return rejected(
            StatusCode::BAD_REQUEST,
            "Anti-spoofing verification failed: watermark nonce is required.",
        );
    }
    let nonce_matched = secrets_match(&session.anti_spoof_nonce, Some(&watermark_nonce));
    if !nonce_matched {
        tracing::warn!("LiveProof watermark nonce mismatch for {session_id}");
        return rejected(
            StatusCode::BAD_REQUEST,
            "Anti-spoofing verification failed: invalid watermark nonce.",
        );
    }

    let duration = if media_type == "live_video" {
        let parsed = telemetry
            .get("duration_seconds")
            .and_then(|v| v.as_f64().or_else(|| v.as_str().and_then(|s| s.parse().ok())))
            .unwrap_or(0.0);
        if parsed < 0.0 || parsed > 3600.0 {
            return rejected(
                StatusCode::BAD_REQUEST,
                format!("Video duration {parsed}s is out of range."),
            );
        }
        if parsed > session.max_video_seconds as f64 {
            return rejected(
                StatusCode::BAD_REQUEST,
                format!(
                    "Video is {parsed:.1}s but this session allows at most {}s.",
                    session.max_video_seconds
                ),
            );
        }
        Some(parsed)
    } else {
        None
    };

    let sha256_hash = hex::encode(Sha256::digest(&bytes));

    // Every score component below is a check that actually ran. Nothing is
    // awarded for a step the server did not perform.
    let mut checks = vec![
        json!({ "check": "anti_spoof_nonce", "passed": nonce_matched }),
        json!({ "check": "live_camera_telemetry", "passed": live_marker || source_marker }),
        json!({ "check": "mime_allowed", "passed": true }),
        json!({ "check": "media_non_empty", "passed": true }),
        json!({ "check": "telemetry_parseable", "passed": telemetry_is_object }),
        json!({
            "check": "duration_within_limit",
            "passed": duration.map(|d| d <= session.max_video_seconds as f64).unwrap_or(true),
        }),
        json!({ "check": "sha256_computed", "passed": sha256_hash.len() == 64 }),
    ];
    let passed = checks.iter().filter(|c| c["passed"].as_bool() == Some(true)).count();
    checks.push(json!({
        "check": "frame_capture_source_reported",
        "passed": telemetry.get("facing_mode").is_some()
            || telemetry.get("resolution").and_then(|v| v.as_str()).is_some()
    }));
    let total = checks.len();
    let score = passed as f64 / total as f64;

    let evidence_id = uuid::Uuid::new_v4().to_string();
    let storage_root = std::path::Path::new(&state.config.live_proof_storage_dir)
        .canonicalize()
        .unwrap_or_else(|_| std::path::PathBuf::from(&state.config.live_proof_storage_dir))
        .join(sanitize_path_segment(&session.tenant_id))
        .join(sanitize_path_segment(&session.id));
    if let Err(err) = std::fs::create_dir_all(&storage_root) {
        tracing::error!("Could not create evidence storage dir: {err}");
        return rejected(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Storage directory unavailable: {err}"),
        );
    }
    let file_name = format!(
        "{evidence_id}_{}.{}",
        media_type,
        extension_for(&mime, &media_type)
    );
    let target = storage_root.join(&file_name);
    if let Err(err) = std::fs::write(&target, &bytes) {
        tracing::error!("Evidence write failed for {session_id}: {err}");
        return rejected(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Storage write error: {err}"),
        );
    }

    let captured_at = now_iso();
    let resolution = telemetry
        .get("resolution")
        .and_then(|v| v.as_str())
        .map(|v| v.to_string());
    let fingerprint = {
        let mut merged = telemetry.as_object().cloned().unwrap_or_default();
        merged.insert("client_ip".into(), client_ip(&headers));
        merged.insert("file_name".into(), json!(file_name));
        merged.insert("ingested_at".into(), json!(captured_at));
        serde_json::to_string(&Value::Object(merged)).unwrap_or_else(|_| "{}".to_string())
    };

    let inserted = sqlx::query(
        "INSERT INTO live_proof_evidences (id, session_id, tenant_id, media_type, file_path,
            file_size_bytes, mime_type, sha256_hash, duration_seconds, resolution,
            device_fingerprint, watermark_verified, verification_score, verification_checks,
            captured_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)",
    )
    .bind(&evidence_id)
    .bind(&session.id)
    .bind(&session.tenant_id)
    .bind(&media_type)
    .bind(target.to_string_lossy().replace('\\', "/"))
    .bind(bytes.len() as i64)
    .bind(&mime)
    .bind(&sha256_hash)
    .bind(duration)
    .bind(resolution.clone())
    .bind(&fingerprint)
    .bind(nonce_matched)
    .bind(score)
    .bind(serde_json::to_string(&checks).unwrap_or_default())
    .bind(&captured_at)
    .execute(&state.db)
    .await;

    if let Err(err) = inserted {
        tracing::error!("Evidence row failed for session {}: {err}", session.id);
        // WHY: a file that survives this cleanup is real state the operator has
        // to know about -- the 5xx used to claim the evidence was on disk even
        // when the delete had removed it, and stayed silent when it had not.
        let disk_note = match std::fs::remove_file(&target) {
            Ok(_) => "the file that was written for it has been deleted again".to_string(),
            Err(cleanup_err) if cleanup_err.kind() == std::io::ErrorKind::NotFound => {
                "no file exists on disk for it".to_string()
            }
            Err(cleanup_err) => {
                tracing::error!(
                    "Orphan live-proof file could not be deleted after the evidence row failed: {} ({cleanup_err})",
                    target.display()
                );
                format!(
                    "its file is STILL on disk at {} and could not be deleted ({cleanup_err})",
                    target.display()
                )
            }
        };
        return rejected(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!(
                "Evidence for session {} was written to disk but the record could not be saved: \
                 {err} -- {disk_note}.",
                session.id
            ),
        );
    }

    // Completion is derived from what is genuinely on file, not on assumption.
    let (kinds, roster_error) = match sqlx::query(
        "SELECT media_type FROM live_proof_evidences WHERE session_id = $1",
    )
    .bind(&session.id)
    .fetch_all(&state.db)
    .await
    {
        Ok(rows) => (rows, None),
        // WHY: an unreadable roster is not an empty roster -- used to fall back
        // to `vec![]`, which then reported which kinds were still missing as if
        // that had been read from storage.
        Err(err) => {
            tracing::error!(
                "Evidence roster for session {} could not be read after the upload was stored: {err}",
                session.id
            );
            (Vec::new(), Some(err.to_string()))
        }
    };
    let has_video = kinds
        .iter()
        .any(|r| r.try_get::<String, _>("media_type").unwrap_or_default() == "live_video");
    let has_photo = kinds
        .iter()
        .any(|r| r.try_get::<String, _>("media_type").unwrap_or_default() == "live_photo");
    let complete = roster_error.is_none()
        && !((session.require_video && !has_video) || (session.require_photos && !has_photo));
    let new_status = if complete { "captured" } else { "in_progress" };

    // WHY: the `sessionStatus` reported below is only true if this UPDATE landed,
    // so a failed or zero-row write is returned as an explicit warning instead of
    // being claimed as the session's state.
    let status_note = match &roster_error {
        Some(reason) => Some(format!(
            "the evidence is stored but the session status was not recomputed, because its evidence \
             roster could not be read: {reason}"
        )),
        None => match sqlx::query(
            "UPDATE live_proof_sessions SET status = $1, completed_at = COALESCE(completed_at, $2),
                    updated_at = $3 WHERE id = $4",
        )
        .bind(new_status)
        .bind(if complete { Some(captured_at.clone()) } else { None })
        .bind(now_iso())
        .bind(&session.id)
        .execute(&state.db)
        .await
        {
            Ok(report) if report.rows_affected() == 1 => None,
            Ok(report) => {
                tracing::error!(
                    "Live proof session {} was NOT set to '{new_status}': the UPDATE affected {} rows \
                     (expected 1) -- the stored session keeps its previous status",
                    session.id,
                    report.rows_affected()
                );
                Some(format!(
                    "the evidence is stored but session status '{new_status}' was NOT written \
                     ({} rows affected)",
                    report.rows_affected()
                ))
            }
            Err(err) => {
                tracing::error!(
                    "Live proof session {} status could not be written as '{new_status}': {err} -- \
                     the stored session keeps its previous status",
                    session.id
                );
                Some(format!(
                    "the evidence is stored but session status '{new_status}' was NOT written: {err}"
                ))
            }
        },
    };

    tracing::info!(
        "Ingested {media_type} for session {} (sha256 {sha256_hash}, score {score:.2})",
        session.id
    );

    Json(json!({
        "id": evidence_id,
        "sessionId": session.id,
        "mediaType": media_type,
        "sha256Hash": sha256_hash,
        "fileSizeBytes": bytes.len() as i64,
        "mimeType": mime,
        "durationSeconds": duration,
        "resolution": resolution,
        "verificationScore": score,
        "verificationChecks": checks,
        "watermarkVerified": nonce_matched,
        "capturedAt": captured_at,
        "sessionStatus": if status_note.is_none() { json!(new_status) } else { Value::Null },
        "sessionStatusStored": status_note.is_none(),
        "warning": status_note,
        "awaiting": match &roster_error {
            Some(_) => Value::Null,
            None if complete => Value::Null,
            None => json!({
                "live_video": !has_video && session.require_video,
                "live_photo": !has_photo && session.require_photos,
            }),
        },
    }))
    .into_response()
}

/// Only an address a proxy actually forwarded is recorded. Nothing is inferred.
fn client_ip(headers: &HeaderMap) -> Value {
    header_str(headers, "x-real-ip")
        .or_else(|| header_str(headers, "x-forwarded-for").and_then(|list| list.split(',').next()))
        .map(|value| Value::String(value.trim().to_string()))
        .unwrap_or(Value::Null)
}

pub async fn evidence_media(
    State(state): State<AppState>,
    Path((session_id, evidence_id)): Path<(String, String)>,
    Query(params): Query<TokenQuery>,
) -> Response {
    let Some(session) = fetch_session(&state.db, &session_id).await.unwrap_or(None) else {
        return rejected(
            StatusCode::NOT_FOUND,
            format!("Session {session_id} not found."),
        );
    };

    // The Python version authorised on any "Bearer " prefix and skipped checks
    // under PYTEST_CURRENT_TEST or development. Both are refused here: the only
    // credential that grants access is the session token itself.
    let token = params.token.unwrap_or_default();
    if !secrets_match(&session.session_token, Some(&token)) {
        return rejected(
            StatusCode::FORBIDDEN,
            "A valid session token is required to view evidence media.",
        );
    }

    let row = sqlx::query(
        "SELECT file_path, mime_type FROM live_proof_evidences
         WHERE id = $1 AND session_id = $2",
    )
    .bind(&evidence_id)
    .bind(&session_id)
    .fetch_optional(&state.db)
    .await;
    let Ok(Some(row)) = row else {
        return rejected(StatusCode::NOT_FOUND, "Evidence not found.");
    };

    let path = row.try_get::<String, _>("file_path").unwrap_or_default();
    let mime = row.try_get::<String, _>("mime_type").unwrap_or_else(|_| "application/octet-stream".into());
    match std::fs::read(&path) {
        Ok(data) => (
            StatusCode::OK,
            [
                ("content-type", mime),
                ("content-disposition", format!("inline; filename=\"{evidence_id}\"")),
            ],
            data,
        )
            .into_response(),
        Err(_) => rejected(
            StatusCode::NOT_FOUND,
            format!("Media file is not present at the recorded storage path."),
        ),
    }
}

#[derive(Deserialize)]
pub struct PageQuery {
    token: Option<String>,
}

/// Server-rendered capture page. The nonce is embedded because the whole point
/// of the challenge is that the page the customer actually saw carries it.
pub async fn capture_page(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
    Query(params): Query<PageQuery>,
) -> Response {
    let token = params.token.unwrap_or_default();
    let session = match fetch_session(&state.db, &session_id).await {
        Ok(Some(session)) => session,
        Ok(None) => {
            return html_page(
                StatusCode::NOT_FOUND,
                "Session not found",
                &format!(
                    "<p>No verification session <code>{}</code> exists. Ask support for a fresh link.</p>",
                    html_escape(&session_id)
                ),
            )
        }
        Err(err) => {
            tracing::error!("Capture page could not read session {session_id}: {err}");
            return html_page(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Storage unavailable",
                "<p>The verification record could not be read. Please retry shortly.</p>",
            )
        }
    };

    if !secrets_match(&session.session_token, Some(&token)) {
        return html_page(
            StatusCode::FORBIDDEN,
            "Invalid link",
            "<p>This link is missing its token or the token does not match the session.</p>",
        );
    }

    let expired = chrono::DateTime::parse_from_rfc3339(&session.expires_at)
        .map(|ends| ends.with_timezone(&chrono::Utc) <= chrono::Utc::now())
        .unwrap_or(true);
    if expired {
        return html_page(
            StatusCode::GONE,
            "Link expired",
            "<p>This verification session has expired. Ask support for a new link.</p>",
        );
    }
    if session.status == "rejected" {
        return html_page(
            StatusCode::FORBIDDEN,
            "Claim rejected",
            &format!(
                "<p>Reason recorded by the reviewer: {}</p>",
                html_escape(
                    session
                        .rejection_reason
                        .as_deref()
                        .unwrap_or("no reason recorded")
                )
            ),
        );
    }

    let evidences = evidences(&state, &session.id).await;
    if matches!(session.status.as_str(), "captured" | "verified") {
        let items = evidences
            .iter()
            .map(|e| {
                format!(
                    "<li><code>{}</code> · {} · {} bytes · verification score {:.2}</li>",
                    e["sha256Hash"]
                        .as_str()
                        .unwrap_or("")
                        .get(..12)
                        .unwrap_or(""),
                    e["mediaType"].as_str().unwrap_or("?"),
                    e["fileSizeBytes"].as_i64().unwrap_or(0),
                    e["verificationScore"].as_f64().unwrap_or(0.0)
                )
            })
            .collect::<Vec<_>>()
            .join("");
        return html_page(
            StatusCode::OK,
            "Evidence submitted",
            &format!(
                "<p>Evidence for this claim is complete and stored. Status: <strong>{}</strong>.</p><ul>{items}</ul>",
                html_escape(&session.status)
            ),
        );
    }

    html_page(
        StatusCode::OK,
        "Live verification",
        &capture_body(&session, &token, &evidences),
    )
}

fn html_escape(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

fn html_page(status: StatusCode, title: &str, body: &str) -> Response {
    (
        status,
        [(axum::http::header::CONTENT_TYPE, "text/html; charset=utf-8")],
        format!(
            "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\">\
             <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\
             <title>{title}</title><style>body{{font-family:system-ui,sans-serif;margin:2rem auto;max-width:40rem;padding:0 1rem;color:#111}}\
             .box{{border:1px solid #ddd;border-radius:.75rem;padding:1.25rem}}code{{background:#f4f4f5;padding:.1rem .3rem;border-radius:.25rem}}</style>\
             </head><body><div class=\"box\"><h1>{title}</h1>{body}</div></body></html>"
        ),
    )
        .into_response()
}

fn capture_body(session: &ProofSession, token: &str, evidences: &[Value]) -> String {
    let done_photo = evidences
        .iter()
        .any(|e| e["mediaType"].as_str() == Some("live_photo"));
    let done_video = evidences
        .iter()
        .any(|e| e["mediaType"].as_str() == Some("live_video"));
    let status_line = format!(
        "Claim <strong>{}</strong>{} · photo {}, video {}",
        html_escape(&session.claim_type),
        session
            .order_id
            .as_deref()
            .map(|o| format!(" for order <code>{}</code>", html_escape(o)))
            .unwrap_or_default(),
        if done_photo { "captured" } else { "required" },
        if done_video { "captured" } else { "required" },
    );
    let evidence_rows = if evidences.is_empty() {
        "<p>No evidence captured yet.</p>".to_string()
    } else {
        let items = evidences
            .iter()
            .map(|e| {
                format!(
                    "<li>{} · {} bytes · score {}</li>",
                    e["mediaType"].as_str().unwrap_or("?"),
                    e["fileSizeBytes"].as_i64().unwrap_or(0),
                    e["verificationScore"].as_f64().unwrap_or(0.0)
                )
            })
            .collect::<Vec<_>>()
            .join("");
        format!("<ul>{items}</ul>")
    };

    format!(
        "<h2>Live proof required</h2><p>{status_line}</p>{evidence_rows}\
         <p><button id=\"cam\">Open camera</button> <button id=\"snap\" disabled>Capture photo</button>\
         <button id=\"rec\" disabled>Record {maxv}s video</button></p>\
         <video id=\"v\" autoplay playsinline muted style=\"width:100%;max-width:32rem;background:#000\"></video>\
         <p id=\"out\"></p>\
         <script>\n\
const TOKEN = \"{token_attr}\";\n\
const NONCE = \"{nonce}\";\n\
const SESSION = \"{sid}\";\n\
const NEED_VIDEO = {need_video};\n\
const NEED_PHOTO = {need_photo};\n\
const MAXV = {maxv};\n\
const out = document.getElementById('out');\n\
let stream = null, track = null, recorder = null, chunks = [];\n\
const say = (m) => {{ out.textContent = m; }};\n\
document.getElementById('cam').onclick = async () => {{\n\
  try {{\n\
    stream = await navigator.mediaDevices.getUserMedia({{ video: {{ facingMode: 'environment' }}, audio: true }});\n\
  }} catch (e) {{ return say('Camera permission denied: ' + e.name); }}\n\
  const v = document.getElementById('v'); v.srcObject = stream; track = stream.getVideoTracks()[0];\n\
  document.getElementById('snap').disabled = false; document.getElementById('rec').disabled = false;\n\
  say('Camera live. Capture must come from this preview.');\n\
}};\n\
async function send(blob, type, telemetry) {{\n\
  const fd = new FormData();\n\
  fd.append('file', blob, type + '.bin');\n\
  fd.append('media_type', type);\n\
  fd.append('watermark_nonce', NONCE);\n\
  fd.append('telemetry', JSON.stringify(telemetry));\n\
  const r = await fetch(`/api/v1/live-proof/sessions/${{SESSION}}/evidence?token=${{TOKEN}}`, {{ method: 'POST', body: fd }});\n\
  const j = await r.json();\n\
  if (!r.ok) {{ say('Rejected: ' + (j.detail || r.status)); return; }}\n\
  say(`Stored ${{j.mediaType}} · ${{j.fileSizeBytes}} bytes · sha256 ${{j.sha256Hash.slice(0,12)}}… · score ${{j.verificationScore.toFixed(2)}} · session ${{j.sessionStatus}}`);\n\
}}\n\
function telemetry(extra) {{\n\
  const s = track && track.getSettings ? track.getSettings() : {{}};\n\
  return Object.assign({{ is_live_stream: true, capture_source: 'live_webrtc_camera',\n\
    facing_mode: s.facingMode || 'unknown', resolution: s.width && s.height ? `${{s.width}}x${{s.height}}` : null,\n\
    device_id_present: !!s.deviceId }}, extra);\n\
}}\n\
document.getElementById('snap').onclick = async () => {{\n\
  if (!stream) return say('Open the camera first.');\n\
  const v = document.getElementById('v');\n\
  const c = document.createElement('canvas'); c.width = v.videoWidth; c.height = v.videoHeight;\n\
  c.getContext('2d').drawImage(v, 0, 0);\n\
  const blob = await new Promise((r) => c.toBlob(r, 'image/jpeg', 0.92));\n\
  if (!blob) return say('Frame grab failed.');\n\
  await send(blob, 'live_photo', telemetry({{ grabbed_at: new Date().toISOString() }}));\n\
}};\n\
document.getElementById('rec').onclick = () => {{\n\
  if (!stream) return say('Open the camera first.');\n\
  chunks = []; const started = Date.now();\n\
  const types = ['video/webm;codecs=vp9,opus', 'video/webm;codecs=vp8,opus', 'video/webm'];\n\
  const mime = types.find((t) => MediaRecorder.isTypeSupported(t));\n\
  if (!mime) return say('This browser cannot record WebM video.');\n\
  recorder = new MediaRecorder(stream, {{ mimeType: mime }});\n\
  recorder.ondataavailable = (e) => e.data.size && chunks.push(e.data);\n\
  recorder.onstop = () => {{\n\
    const blob = new Blob(chunks, {{ type: mime }});\n\
    send(blob, 'live_video', telemetry({{ duration_seconds: (Date.now() - started) / 1000 }}));\n\
    document.getElementById('rec').textContent = `Record ${{MAXV}}s video`;\n\
  }};\n\
  recorder.start();\n\
  let left = MAXV;\n\
  document.getElementById('rec').disabled = true;\n\
  const tick = setInterval(() => {{\n\
    left -= 1; document.getElementById('rec').textContent = `Recording... ${{left}}s`;\n\
    if (left <= 0) {{ clearInterval(tick); recorder.stop(); document.getElementById('rec').disabled = false; }}\n\
  }}, 1000);\n\
}};\n\
if (!NEED_VIDEO) document.getElementById('rec').disabled = true;\n\
if (!NEED_PHOTO) document.getElementById('snap').disabled = true;\n\
</script>",
        token_attr = html_escape(token),
        nonce = html_escape(&session.anti_spoof_nonce),
        sid = html_escape(&session.id),
        need_video = session.require_video && !done_video,
        need_photo = session.require_photos && !done_photo,
        maxv = session.max_video_seconds,
    )
}
