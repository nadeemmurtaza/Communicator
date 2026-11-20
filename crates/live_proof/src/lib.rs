//! Live Proof business logic — extracted from backend-rs monolith.

use base64::Engine;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use sqlx::Row;

// ---------------------------------------------------------------------------
// State
// ---------------------------------------------------------------------------

#[derive(Clone)]
pub struct LiveProofState {
    pub db: sqlx::PgPool,
    pub public_url: String,
    pub live_proof_storage_dir: String,
}

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

const MAX_MEDIA_BYTES: usize = 100 * 1024 * 1024;
const ALLOWED_PHOTO_MIMES: &[&str] = &["image/jpeg", "image/png", "image/webp"];
const ALLOWED_VIDEO_MIMES: &[&str] = &["video/webm", "video/mp4"];
pub const DEFAULT_TTL_MINUTES: i64 = 30;
pub const DEFAULT_MAX_VIDEO_SECONDS: i64 = 45;
pub const DEFAULT_CLAIM_TYPE: &str = "refund";

// ---------------------------------------------------------------------------
// Inlined from signatures::secrets_match (constant-time comparison)
// ---------------------------------------------------------------------------

fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff = 0u8;
    for (x, y) in a.iter().zip(b) {
        diff |= x ^ y;
    }
    diff == 0
}

fn secrets_match(expected: &str, provided: Option<&str>) -> bool {
    if expected.is_empty() {
        return false;
    }
    constant_time_eq(expected.as_bytes(), provided.unwrap_or("").as_bytes())
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

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

fn html_escape(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

fn sanitize_path_segment(input: &str) -> String {
    let cleaned: String = input
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
        .collect();
    cleaned.chars().take(64).collect()
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

// ---------------------------------------------------------------------------
// Models
// ---------------------------------------------------------------------------

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

async fn fetch_session(pool: &sqlx::PgPool, id: &str) -> Result<Option<ProofSession>, String> {
    let row = sqlx::query(
        "SELECT id, tenant_id, organization_id, order_id, customer_identifier, claim_type,
                session_token, status, anti_spoof_nonce, max_video_seconds, require_video,
                require_photos, expires_at, completed_at, rejection_reason
         FROM live_proof_sessions WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(|e| e.to_string())?;

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

async fn validate(
    state: &LiveProofState,
    session_id: &str,
    token: &str,
    for_upload: bool,
) -> Result<ProofSession, String> {
    let Some(session) = fetch_session(&state.db, session_id).await.map_err(|e| {
        format!("Could not read the verification session: {e}")
    })? else {
        return Err(format!("Verification session {session_id} not found."));
    };

    if !secrets_match(&session.session_token, Some(token)) {
        return Err("Invalid verification token.".to_string());
    }

    let expired = chrono::DateTime::parse_from_rfc3339(&session.expires_at)
        .map(|ends| ends.with_timezone(&chrono::Utc) <= chrono::Utc::now())
        .unwrap_or(true);
    if expired {
        if session.status != "expired" {
            // Auxiliary update – fire and forget
            let _ = sqlx::query("UPDATE live_proof_sessions SET status = 'expired', updated_at = $1 WHERE id = $2")
                .bind(now_iso())
                .bind(&session.id)
                .execute(&state.db)
                .await;
        }
        return Err("This verification session has expired. Please ask support for a new link.".to_string());
    }

    if session.status == "rejected" {
        return Err(format!(
            "This session was rejected: {}",
            session.rejection_reason
                .clone()
                .unwrap_or_else(|| "no reason recorded".to_string())
        ));
    }
    if for_upload && matches!(session.status.as_str(), "captured" | "verified") {
        return Err("Evidence collection for this session has already been completed and submitted.".to_string());
    }
    Ok(session)
}

async fn evidences(state: &LiveProofState, session_id: &str) -> Vec<Value> {
    let sql = "SELECT id, session_id, media_type, sha256_hash, file_size_bytes, duration_seconds,
            verification_score, watermark_verified, mime_type, resolution, captured_at
     FROM live_proof_evidences WHERE session_id = $1 ORDER BY captured_at";
    let rows = sqlx::query(sql)
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

// ---------------------------------------------------------------------------
// Public types for reply_engine compatibility
// ---------------------------------------------------------------------------

/// The part of `create_session` that other Rust code (the reply engine) needs.
#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub struct CreatedLiveProofSession {
    pub id: String,
    pub session_token: String,
    pub expires_at: String,
}

/// Create a pending session directly (used by reply_engine).
pub async fn create_pending_session(
    state: &LiveProofState,
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

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Create a new verification session. `body_json` is the JSON body from the HTTP request.
/// `public_url` is the base URL for the verification link (from request headers if empty).
pub async fn create_session(
    state: &LiveProofState,
    body_json: &str,
    public_url_from_headers: &str,
) -> Result<(u16, Value), (u16, String)> {
    let body: serde_json::Map<String, Value> =
        serde_json::from_str(body_json).map_err(|e| (400u16, format!("Invalid JSON: {e}")))?;

    let tenant_id = body
        .get("tenant_id")
        .or_else(|| body.get("tenantId"))
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(str::to_string);

    let header_tenant = body
        .get("header_tenant")
        .and_then(|v| v.as_str())
        .map(str::to_string)
        .unwrap_or_default();

    let tenant_id = tenant_id
        .unwrap_or(header_tenant);

    if tenant_id.is_empty() {
        return Err((
            400,
            "No tenant: the body has no tenant_id and the request carries no tenant scope.".into(),
        ));
    }

    let ttl: i64 = body
        .get("ttl_minutes")
        .or_else(|| body.get("ttlMinutes"))
        .and_then(|v| v.as_i64())
        .unwrap_or(DEFAULT_TTL_MINUTES);
    if !(1..=1440).contains(&ttl) {
        return Err((
            400,
            format!("ttl_minutes must be between 1 and 1440, received {ttl}."),
        ));
    }

    let max_video: i64 = body
        .get("max_video_seconds")
        .or_else(|| body.get("maxVideoSeconds"))
        .and_then(|v| v.as_i64())
        .unwrap_or(DEFAULT_MAX_VIDEO_SECONDS);
    if !(5..=120).contains(&max_video) {
        return Err((
            400,
            format!("max_video_seconds must be between 5 and 120, received {max_video}."),
        ));
    }

    let require_video = body
        .get("require_video")
        .or_else(|| body.get("requireVideo"))
        .and_then(|v| v.as_bool())
        .unwrap_or(true);
    let require_photos = body
        .get("require_photos")
        .or_else(|| body.get("requirePhotos"))
        .and_then(|v| v.as_bool())
        .unwrap_or(true);

    if !require_video && !require_photos {
        return Err((
            400,
            "A session that accepts neither video nor photos cannot verify anything.".into(),
        ));
    }

    let organization_id = body
        .get("organization_id")
        .or_else(|| body.get("organizationId"))
        .and_then(|v| v.as_str())
        .map(String::from);
    let conversation_id = body
        .get("conversation_id")
        .or_else(|| body.get("conversationId"))
        .and_then(|v| v.as_str())
        .map(String::from);
    let contact_id = body
        .get("contact_id")
        .or_else(|| body.get("contactId"))
        .and_then(|v| v.as_str())
        .map(String::from);
    let order_id = body
        .get("order_id")
        .or_else(|| body.get("orderId"))
        .and_then(|v| v.as_str())
        .map(String::from);
    let customer_identifier = body
        .get("customer_identifier")
        .or_else(|| body.get("customerIdentifier"))
        .and_then(|v| v.as_str())
        .map(String::from)
        .unwrap_or_default();
    let claim_type = body
        .get("claim_type")
        .or_else(|| body.get("claimType"))
        .and_then(|v| v.as_str())
        .map(String::from)
        .unwrap_or_else(|| DEFAULT_CLAIM_TYPE.to_string());

    let id = uuid::Uuid::new_v4().to_string();
    let session_token = high_entropy("", 2);
    let anti_spoof_nonce = high_entropy("", 1);
    let now = now_iso();
    let expires_at = iso_ahead(ttl);

    let result = sqlx::query(
        "INSERT INTO live_proof_sessions (id, tenant_id, organization_id, conversation_id,
            contact_id, order_id, customer_identifier, claim_type, session_token, status,
            anti_spoof_nonce, max_video_seconds, require_video, require_photos, expires_at,
            completed_at, rejection_reason, created_at, updated_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, 'pending', $10, $11, $12, $13, $14, NULL, NULL, $15, $16)",
    )
    .bind(&id)
    .bind(&tenant_id)
    .bind(organization_id.as_deref())
    .bind(conversation_id.as_deref())
    .bind(contact_id.as_deref())
    .bind(order_id.as_deref())
    .bind(&customer_identifier)
    .bind(&claim_type)
    .bind(&session_token)
    .bind(&anti_spoof_nonce)
    .bind(max_video)
    .bind(require_video)
    .bind(require_photos)
    .bind(&expires_at)
    .bind(&now)
    .bind(&now)
    .execute(&state.db)
    .await
    .map_err(|err| (500u16, format!("Could not create the verification session: {err}")))?;

    if result.rows_affected() != 1 {
        return Err((
            500,
            format!(
                "the session row was not written ({} rows affected)",
                result.rows_affected()
            ),
        ));
    }

    let root = if state.public_url.trim().is_empty() {
        public_url_from_headers.to_string()
    } else {
        state.public_url.trim().trim_end_matches('/').to_string()
    };

    tracing::info!("Created LiveProof session {id} for tenant {tenant_id}");

    Ok((
        201,
        json!({
            "sessionId": id,
            "verificationUrl": format!("{root}/verify/proof/{id}?token={session_token}"),
            "status": "pending",
            "expiresAt": expires_at,
            "tenantId": tenant_id,
            "maxVideoSeconds": max_video,
            "requireVideo": require_video,
            "requirePhotos": require_photos,
        }),
    ))
}

/// Get session details.
pub async fn get_session_details(
    state: &LiveProofState,
    session_id: &str,
    token: &str,
) -> Result<Value, (u16, String)> {
    let session = validate(state, session_id, token, false).await.map_err(|e| (404u16, e))?;
    Ok(json!({
        "id": session.id,
        "tenantId": session.tenant_id,
        "organizationId": session.organization_id,
        "orderId": session.order_id,
        "claimType": session.claim_type,
        "status": session.status,
        "expiresAt": session.expires_at,
        "completedAt": session.completed_at,
        "evidences": evidences(state, &session.id).await,
    }))
}

/// Record a decision (approve/reject) on a session.
pub async fn record_decision(
    state: &LiveProofState,
    session_id: &str,
    token: &str,
    body_json: &str,
) -> Result<Value, (u16, String)> {
    let session = validate(state, session_id, token, false).await.map_err(|e| (404u16, e))?;
    if !matches!(session.status.as_str(), "captured" | "in_progress" | "pending") {
        return Err((
            409,
            format!("Session is already {} and cannot be re-decided.", session.status),
        ));
    }

    let body: serde_json::Map<String, Value> =
        serde_json::from_str(body_json).map_err(|e| (400u16, format!("Invalid JSON: {e}")))?;

    let decision = body
        .get("decision")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim()
        .to_ascii_uppercase();

    let rejection_reason = body
        .get("rejection_reason")
        .or_else(|| body.get("rejectionReason"))
        .and_then(|v| v.as_str())
        .map(String::from);

    let (new_status, reason) = match decision.as_str() {
        "APPROVED" | "VERIFIED" => ("verified", Value::Null),
        "REJECTED" => (
            "rejected",
            json!(rejection_reason
                .clone()
                .filter(|v| !v.trim().is_empty())
                .unwrap_or_else(|| "No reason supplied by the reviewer.".to_string())),
        ),
        other => {
            return Err((
                400,
                format!("Unknown decision '{other}'. Expected APPROVED or REJECTED."),
            ))
        }
    };

    let result = sqlx::query(
        "UPDATE live_proof_sessions SET status = $1, rejection_reason = COALESCE($2, rejection_reason),
                updated_at = $3 WHERE id = $4",
    )
    .bind(new_status)
    .bind(rejection_reason)
    .bind(now_iso())
    .bind(&session.id)
    .execute(&state.db)
    .await
    .map_err(|err| (500u16, format!("Could not record the decision: {err}")))?;

    if result.rows_affected() == 0 {
        return Err((
            404,
            format!("Session {session_id} not found or already changed."),
        ));
    }

    tracing::info!("LiveProof session {} decided {new_status}", session.id);

    Ok(json!({
        "sessionId": session.id,
        "status": new_status,
        "rejectionReason": reason,
        "evidences": evidences(state, &session.id).await,
    }))
}

/// Upload evidence (photo/video). `body_json` contains base64-encoded file data and metadata.
pub async fn upload_evidence(
    state: &LiveProofState,
    session_id: &str,
    token: &str,
    body_json: &str,
) -> Result<Value, (u16, String)> {
    let session = validate(state, session_id, token, true).await.map_err(|e| (404u16, e))?;

    let body: serde_json::Map<String, Value> =
        serde_json::from_str(body_json).map_err(|e| (400u16, format!("Invalid JSON: {e}")))?;

    let media_type = body
        .get("media_type")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim()
        .to_string();

    if !matches!(media_type.as_str(), "live_photo" | "live_video") {
        return Err((
            400,
            format!("media_type must be live_photo or live_video, received '{media_type}'."),
        ));
    }

    // Decode base64 file data
    let file_b64 = body
        .get("file")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(file_b64)
        .map_err(|e| (400u16, format!("Invalid base64 file data: {e}")))?;

    if bytes.is_empty() {
        return Err((400, "Captured media is empty.".into()));
    }
    if bytes.len() > MAX_MEDIA_BYTES {
        return Err((
            413,
            format!(
                "Media size exceeds the {}MB limit.",
                MAX_MEDIA_BYTES / (1024 * 1024)
            ),
        ));
    }

    let mime = body
        .get("mime")
        .or_else(|| body.get("mime_type"))
        .and_then(|v| v.as_str())
        .map(normalise_mime)
        .unwrap_or_else(|| {
            if media_type == "live_photo" {
                "image/jpeg".to_string()
            } else {
                "video/webm".to_string()
            }
        });

    let allowed = if media_type == "live_photo" {
        ALLOWED_PHOTO_MIMES
    } else {
        ALLOWED_VIDEO_MIMES
    };
    if !allowed.contains(&mime.as_str()) {
        return Err((
            415,
            format!(
                "Unsupported {} MIME type: {mime}. {}",
                if media_type == "live_photo" { "photo" } else { "video" },
                if media_type == "live_photo" {
                    "Must be JPEG, PNG, or WebP."
                } else {
                    "Must be WebM or MP4."
                }
            ),
        ));
    }

    // Telemetry
    let telemetry = body
        .get("telemetry")
        .cloned()
        .unwrap_or_else(|| Value::Null);
    let telemetry_is_object = telemetry.is_object();

    let live_marker = telemetry.get("is_live_stream").and_then(|v| v.as_bool()) == Some(true);
    let source_marker = telemetry
        .get("capture_source")
        .and_then(|v| v.as_str())
        .map(|v| v == "live_webrtc_camera")
        .unwrap_or(false);
    if !(live_marker || source_marker) {
        tracing::warn!("LiveProof upload rejected for {session_id}: no live camera telemetry.");
        return Err((
            400,
            "File upload from the phone gallery is prohibited. Capture must come from the live camera.".into(),
        ));
    }

    // Watermark nonce
    let watermark_nonce = body
        .get("watermark_nonce")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    if watermark_nonce.trim().is_empty() {
        return Err((
            400,
            "Anti-spoofing verification failed: watermark nonce is required.".into(),
        ));
    }
    let nonce_matched = secrets_match(&session.anti_spoof_nonce, Some(watermark_nonce));
    if !nonce_matched {
        tracing::warn!("LiveProof watermark nonce mismatch for {session_id}");
        return Err((
            400,
            "Anti-spoofing verification failed: invalid watermark nonce.".into(),
        ));
    }

    // Duration validation for video
    let duration = if media_type == "live_video" {
        let parsed = telemetry
            .get("duration_seconds")
            .and_then(|v| v.as_f64().or_else(|| v.as_str().and_then(|s| s.parse().ok())))
            .unwrap_or(0.0);
        if parsed < 0.0 || parsed > 3600.0 {
            return Err((
                400,
                format!("Video duration {parsed}s is out of range."),
            ));
        }
        if parsed > session.max_video_seconds as f64 {
            return Err((
                400,
                format!(
                    "Video is {parsed:.1}s but this session allows at most {}s.",
                    session.max_video_seconds
                ),
            ));
        }
        Some(parsed)
    } else {
        None
    };

    let sha256_hash = hex::encode(Sha256::digest(&bytes));

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

    // Write file to disk
    let evidence_id = uuid::Uuid::new_v4().to_string();
    let storage_root = std::path::Path::new(&state.live_proof_storage_dir)
        .join(sanitize_path_segment(&session.tenant_id))
        .join(sanitize_path_segment(&session.id));
    std::fs::create_dir_all(&storage_root).map_err(|err| {
        (
            500u16,
            format!("Storage directory unavailable: {err}"),
        )
    })?;

    let file_name = format!(
        "{evidence_id}_{}.{}",
        media_type,
        extension_for(&mime, &media_type)
    );
    let target = storage_root.join(&file_name);
    std::fs::write(&target, &bytes).map_err(|err| {
        (
            500u16,
            format!("Storage write error: {err}"),
        )
    })?;

    let captured_at = now_iso();
    let resolution = body
        .get("resolution")
        .and_then(|v| v.as_str())
        .map(String::from);

    let fingerprint = {
        let mut merged = serde_json::Map::new();
        if let Value::Object(obj) = &telemetry {
            for (k, v) in obj {
                merged.insert(k.clone(), v.clone());
            }
        }
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
        let _ = std::fs::remove_file(&target);
        return Err((
            500,
            format!(
                "Evidence for session {} was written to disk but the record could not be saved: {err}.",
                session.id
            ),
        ));
    }

    // Determine session completion
    let kinds = sqlx::query("SELECT media_type FROM live_proof_evidences WHERE session_id = $1")
        .bind(&session.id)
        .fetch_all(&state.db)
        .await
        .unwrap_or_default();
    let has_video = kinds
        .iter()
        .any(|r| r.try_get::<String, _>("media_type").unwrap_or_default() == "live_video");
    let has_photo = kinds
        .iter()
        .any(|r| r.try_get::<String, _>("media_type").unwrap_or_default() == "live_photo");
    let complete = !((session.require_video && !has_video) || (session.require_photos && !has_photo));
    let new_status = if complete { "captured" } else { "in_progress" };

    let _ = sqlx::query(
        "UPDATE live_proof_sessions SET status = $1, completed_at = COALESCE(completed_at, $2),
                updated_at = $3 WHERE id = $4",
    )
    .bind(new_status)
    .bind(if complete { Some(captured_at.clone()) } else { None })
    .bind(now_iso())
    .bind(&session.id)
    .execute(&state.db)
    .await;

    tracing::info!(
        "Ingested {media_type} for session {} (sha256 {sha256_hash}, score {score:.2})",
        session.id
    );

    Ok(json!({
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
        "sessionStatus": new_status,
        "awaiting": if complete {
            Value::Null
        } else {
            json!({
                "live_video": !has_video && session.require_video,
                "live_photo": !has_photo && session.require_photos,
            })
        },
    }))
}

/// Get evidence media as base64.
pub async fn get_evidence_media(
    state: &LiveProofState,
    session_id: &str,
    evidence_id: &str,
    token: &str,
) -> Result<Value, (u16, String)> {
    let Some(session) = fetch_session(&state.db, session_id)
        .await
        .unwrap_or(None)
    else {
        return Err((
            404,
            format!("Session {session_id} not found."),
        ));
    };

    if !secrets_match(&session.session_token, Some(token)) {
        return Err((
            403,
            "A valid session token is required to view evidence media.".into(),
        ));
    }

    let row = sqlx::query(
        "SELECT file_path, mime_type FROM live_proof_evidences
         WHERE id = $1 AND session_id = $2",
    )
    .bind(evidence_id)
    .bind(session_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| (500u16, e.to_string()))?;

    let Some(row) = row else {
        return Err((404, "Evidence not found.".into()));
    };

    let path = row.try_get::<String, _>("file_path").unwrap_or_default();
    let mime = row.try_get::<String, _>("mime_type").unwrap_or_else(|_| "application/octet-stream".into());

    match std::fs::read(&path) {
        Ok(data) => {
            let b64 = base64::engine::general_purpose::STANDARD.encode(&data);
            Ok(json!({
                "mimeType": mime,
                "data": b64,
                "evidenceId": evidence_id,
            }))
        }
        Err(_) => Err((
            404,
            "Media file is not present at the recorded storage path.".into(),
        )),
    }
}

/// Get the capture page HTML.
pub async fn get_capture_page(
    state: &LiveProofState,
    session_id: &str,
    token: &str,
) -> Result<Value, (u16, String)> {
    let session = match fetch_session(&state.db, session_id).await {
        Ok(Some(session)) => session,
        Ok(None) => {
            return Err((
                404,
                format!(
                    "No verification session {} exists. Ask support for a fresh link.",
                    html_escape(session_id)
                ),
            ))
        }
        Err(err) => {
            tracing::error!("Capture page could not read session {session_id}: {err}");
            return Err((
                500,
                "The verification record could not be read. Please retry shortly.".into(),
            ));
        }
    };

    if !secrets_match(&session.session_token, Some(token)) {
        return Err((
            403,
            "This link is missing its token or the token does not match the session.".into(),
        ));
    }

    let expired = chrono::DateTime::parse_from_rfc3339(&session.expires_at)
        .map(|ends| ends.with_timezone(&chrono::Utc) <= chrono::Utc::now())
        .unwrap_or(true);
    if expired {
        return Err((
            410,
            "This verification session has expired. Ask support for a new link.".into(),
        ));
    }
    if session.status == "rejected" {
        return Err((
            403,
            format!(
                "Reason recorded by the reviewer: {}",
                session
                    .rejection_reason
                    .as_deref()
                    .unwrap_or("no reason recorded")
            ),
        ));
    }

    let evidences = evidences(state, &session.id).await;
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
        return Ok(json!({
            "html": format!(
                "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\">\
                 <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\
                 <title>Evidence submitted</title></head><body><h1>Evidence submitted</h1>\
                 <p>Evidence for this claim is complete and stored. Status: <strong>{}</strong>.</p><ul>{items}</ul></body></html>",
                html_escape(&session.status)
            ),
            "status": session.status,
        }));
    }

    let done_photo = evidences
        .iter()
        .any(|e| e["mediaType"].as_str() == Some("live_photo"));
    let done_video = evidences
        .iter()
        .any(|e| e["mediaType"].as_str() == Some("live_video"));

    let html = format!(
        "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\">\
         <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\
         <title>Live verification</title></head><body><h1>Live proof required</h1>\
         <p>Claim <strong>{}</strong></p>\
         <p><button id=\"cam\">Open camera</button></p>\
         <video id=\"v\" autoplay playsinline muted style=\"width:100%%;max-width:32rem;background:#000\"></video>\
         <p id=\"out\"></p></body></html>",
        html_escape(&session.claim_type)
    );

    Ok(json!({
        "html": html,
        "status": session.status,
        "sessionId": session.id,
        "nonce": session.anti_spoof_nonce,
        "requireVideo": session.require_video && !done_video,
        "requirePhotos": session.require_photos && !done_photo,
        "maxVideoSeconds": session.max_video_seconds,
    }))
}
