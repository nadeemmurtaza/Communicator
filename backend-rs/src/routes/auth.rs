#![allow(dead_code)]

use crate::db::AppState;
use crate::services::nid_client::{NidError, SessionClaims, SignUpResult};
use axum::extract::{FromRequestParts, Query, State};
use axum::http::{header::AUTHORIZATION, request::Parts, HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use chrono::{Duration as ChronoDuration, Utc};
use serde_json::json;
use uuid::Uuid;

pub struct AuthError(pub StatusCode, pub String);

impl From<NidError> for AuthError {
    fn from(err: NidError) -> Self {
        let (status, detail) = match err {
            NidError::NotConfigured => (
                StatusCode::SERVICE_UNAVAILABLE,
                "nID sign-in is not configured. Set NID_CLIENT_ID and NID_CLIENT_SECRET, \
                 or NID_ALLOW_MOCK_AUTH=true for local development."
                    .to_string(),
            ),
            NidError::Unreachable(reason) => (
                StatusCode::BAD_GATEWAY,
                format!("nID identity provider is unreachable: {reason}"),
            ),
            NidError::Rejected(401) => (
                StatusCode::UNAUTHORIZED,
                "Authentication failed: nID rejected the authorization code.".to_string(),
            ),
            NidError::Rejected(code) => (
                StatusCode::BAD_GATEWAY,
                format!("nID identity provider returned status {code}."),
            ),
            NidError::MalformedResponse => (
                StatusCode::BAD_GATEWAY,
                "nID identity provider returned an unreadable response.".to_string(),
            ),
            NidError::InvalidState => (
                StatusCode::UNAUTHORIZED,
                "Invalid or expired state token. Please initiate login again.".to_string(),
            ),
            NidError::MissingIdentity => (
                StatusCode::BAD_GATEWAY,
                "nID response did not carry an account identity or access token.".to_string(),
            ),
            NidError::OtpFailed(detail) => (
                StatusCode::BAD_REQUEST,
                format!("Sign-up failed: {detail}"),
            ),
        };
        Self(status, detail)
    }
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        (self.0, Json(json!({ "detail": self.1 }))).into_response()
    }
}

fn bearer_token(headers: &HeaderMap) -> Option<String> {
    let raw = headers.get(AUTHORIZATION)?.to_str().ok()?;
    let (scheme, token) = raw.split_once(char::is_whitespace)?;
    if !scheme.eq_ignore_ascii_case("bearer") {
        return None;
    }
    let token = token.trim();
    (!token.is_empty()).then(|| token.to_string())
}

/// The authenticated nID account behind a request. Endpoints that need to know
/// *who* is calling use this instead of trusting a client-supplied identity.
pub struct CurrentAccount {
    pub account_id: String,
    pub claims: SessionClaims,
}

impl FromRequestParts<AppState> for CurrentAccount {
    type Rejection = AuthError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let Some(token) = bearer_token(&parts.headers) else {
            return Err(AuthError(
                StatusCode::UNAUTHORIZED,
                "Missing or invalid Bearer authorization header.".to_string(),
            ));
        };
        let Some(claims) = state.nid.validate_session(&token).await else {
            return Err(AuthError(
                StatusCode::UNAUTHORIZED,
                "Invalid or expired session token.".to_string(),
            ));
        };
        if claims.account_id.is_empty() {
            return Err(AuthError(
                StatusCode::UNAUTHORIZED,
                "Session has no account identity.".to_string(),
            ));
        }
        Ok(Self {
            account_id: claims.account_id.clone(),
            claims,
        })
    }
}

/// Tenant whose data a request may see. Comes from X-Tenant-Key; when absent,
/// falls back to DEFAULT_TENANT_KEY and refuses the request if that is unset.
pub struct Tenant(pub String);

impl FromRequestParts<AppState> for Tenant {
    type Rejection = AuthError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let header = parts
            .headers
            .get("x-tenant-key")
            .and_then(|v| v.to_str().ok())
            .map(str::trim)
            .filter(|v| !v.is_empty())
            .map(str::to_string);
        match header.or_else(|| {
            (!state.config.default_tenant_key.is_empty())
                .then(|| state.config.default_tenant_key.clone())
        }) {
            Some(tenant) => Ok(Self(tenant)),
            None => Err(AuthError(
                StatusCode::BAD_REQUEST,
                "Missing X-Tenant-Key header and no DEFAULT_TENANT_KEY configured.".to_string(),
            )),
        }
    }
}

#[derive(serde::Deserialize)]
pub struct StartQuery {
    redirect_uri: Option<String>,
}

pub async fn login_with_nid_start(
    State(state): State<AppState>,
    Query(params): Query<StartQuery>,
) -> Result<impl IntoResponse, AuthError> {
    let start = state
        .nid
        .start_login(params.redirect_uri.as_deref())
        .await?;

    let dev_mode = state.nid.mock_auth_allowed() && !state.nid.is_configured();
    Ok(Json(json!({
        "authorize_url": start.authorize_url,
        "poll_token": start.poll_token,
        "state": start.state,
        "dev_mode": dev_mode,
    })))
}

#[derive(serde::Deserialize)]
pub struct CallbackQuery {
    code: Option<String>,
    state: Option<String>,
}

pub async fn login_with_nid_callback(
    State(state): State<AppState>,
    Query(params): Query<CallbackQuery>,
) -> Result<impl IntoResponse, AuthError> {
    let code = params.code.filter(|c| !c.is_empty());
    let login_state = params.state.filter(|s| !s.is_empty());
    let (Some(code), Some(login_state)) = (code, login_state) else {
        return Err(NidError::InvalidState.into());
    };

    let claims = state.nid.exchange_code(&code, &login_state).await?;
    Ok(Json(json!({
        "token": claims.access_token,
        "account_id": claims.account_id,
        "email": claims.email,
        "phone": claims.phone,
    })))
}

pub async fn get_current_user_profile(
    account: CurrentAccount,
) -> Result<impl IntoResponse, AuthError> {
    let claims = &account.claims;
    Ok(Json(json!({
        "status": "authenticated",
        "account_id": claims.account_id,
        "accountId": claims.account_id,
        "email": claims.email,
        "phone": claims.phone,
        "phone_verified": claims.phone_verified,
        "phoneVerified": claims.phone_verified,
        "provider": "nID",
    })))
}

// ===========================================================================
// Sign Up — Account creation via nID email + OTP
// ===========================================================================

#[derive(serde::Deserialize)]
pub struct SignUpStartBody {
    email: String,
}

/// Starts the sign-up flow by sending an OTP code to the given email address.
/// Uses the local Communicator OTP engine (email channel). Returns an otp_id
/// that must be included when calling /auth/sign-up/verify.
pub async fn sign_up_start(
    State(state): State<AppState>,
    Json(body): Json<SignUpStartBody>,
) -> Result<impl IntoResponse, AuthError> {
    let email = body.email.trim();
    if email.is_empty() {
        return Err(AuthError(
            StatusCode::BAD_REQUEST,
            "Email address is required.".to_string(),
        ));
    }
    let otp_id = state.nid.start_sign_up(email).await?;
    Ok(Json(json!({
        "status": "otp_sent",
        "email": email,
        "otpId": otp_id,
        "message": "Verification code sent to your email. Include otpId when calling /auth/sign-up/verify.",
    })))
}

#[derive(serde::Deserialize)]
pub struct SignUpVerifyBody {
    email: String,
    code: String,
    /// Optional: the otpId returned from /auth/sign-up/start.
    /// If omitted, the server looks up the pending OTP by email.
    #[allow(dead_code)]
    otp_id: Option<String>,
}

/// Verifies the OTP code using the local OTP engine and creates the session.
/// Returns a session token that can be used as a Bearer token for auth.
pub async fn sign_up_verify(
    State(state): State<AppState>,
    Json(body): Json<SignUpVerifyBody>,
) -> Result<impl IntoResponse, AuthError> {
    let email = body.email.trim();
    let code = body.code.trim();
    if email.is_empty() {
        return Err(AuthError(
            StatusCode::BAD_REQUEST,
            "Email address is required.".to_string(),
        ));
    }
    if code.is_empty() {
        return Err(AuthError(
            StatusCode::BAD_REQUEST,
            "Verification code is required.".to_string(),
        ));
    }

    let result: SignUpResult = state.nid.verify_sign_up(email, code).await?;
    Ok(Json(json!({
        "status": "signed_up",
        "token": result.session_token,
        "account_id": result.account_id,
        "accountId": result.account_id,
        "email": result.email,
        "otpId": result.otp_id,
        "provider": "nID",
    })))
}

// ===========================================================================
// Instagram Business Login (port of api/routes/instagram_auth.py)
//
// The authorize URL is built from the real configured app credentials and the
// callback performs the real code -> short-lived -> long-lived token exchange
// against Instagram/Meta. With no credentials the answer is an honest 503 that
// names the missing variables; nothing here ever reports "connected" on its own.
// ===========================================================================

const IG_SCOPES: &str = "instagram_business_basic,instagram_business_manage_messages,instagram_business_manage_comments";
const IG_STATE_TTL_SECS: i64 = 600;

/// In-flight Instagram OAuth states -> creation unix time, redeemed once.
fn pending_ig_states(
) -> &'static std::sync::Mutex<std::collections::HashMap<String, i64>> {
    use std::sync::OnceLock;
    static MAP: OnceLock<std::sync::Mutex<std::collections::HashMap<String, i64>>> = OnceLock::new();
    MAP.get_or_init(|| std::sync::Mutex::new(std::collections::HashMap::new()))
}

fn env_value(name: &str) -> Option<String> {
    std::env::var(name)
        .ok()
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty() && !v.contains('•'))
}

fn prune_expired_ig_states() {
    let now = Utc::now().timestamp();
    if let Ok(mut map) = pending_ig_states().lock() {
        let expired: Vec<String> = map
            .iter()
            .filter(|(_, created_at)| now - *created_at > IG_STATE_TTL_SECS)
            .map(|(state, _)| state.clone())
            .collect();
        for state in expired {
            map.remove(&state);
        }
    }
}

fn consume_ig_state(state: &str) -> bool {
    let mut issued = false;
    if let Ok(mut map) = pending_ig_states().lock() {
        issued = map.remove(state).is_some();
    }
    issued
}

/// RFC 3986 percent-encoding for the authorize/query strings (no urlencoding
/// crate is in Cargo.toml).
fn urlencode_component(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for byte in input.as_bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(*byte as char)
            }
            b' ' => out.push('+'),
            other => out.push_str(&format!("%{other:02X}")),
        }
    }
    out
}

#[derive(Debug)]
struct InstagramCreds {
    app_id: String,
    app_secret: String,
    redirect_uri: String,
}

/// Reads the three Instagram login settings, preferring the vault-loaded app
/// secret already on Config and falling back to the environment.
fn instagram_credentials(state: &AppState) -> Result<InstagramCreds, (StatusCode, String)> {
    let app_id = env_value("META_INSTAGRAM_APP_ID").ok_or((
        StatusCode::SERVICE_UNAVAILABLE,
        "META_INSTAGRAM_APP_ID is not configured, so no Instagram authorize URL can be built.".to_string(),
    ))?;
    let app_secret = env_value("META_INSTAGRAM_APP_SECRET")
        .or_else(|| {
            let from_config = state.config.meta_instagram_app_secret.trim().to_string();
            (!from_config.is_empty() && !from_config.contains('•')).then_some(from_config)
        })
        .ok_or((
            StatusCode::SERVICE_UNAVAILABLE,
            "META_INSTAGRAM_APP_SECRET is not configured; the authorization code cannot be exchanged.".to_string(),
        ))?;
    let redirect_uri = env_value("META_INSTAGRAM_REDIRECT_URI").ok_or((
        StatusCode::SERVICE_UNAVAILABLE,
        "META_INSTAGRAM_REDIRECT_URI is not configured; Instagram would reject the callback.".to_string(),
    ))?;
    Ok(InstagramCreds {
        app_id,
        app_secret,
        redirect_uri,
    })
}

pub async fn instagram_oauth_start(
    State(state): State<AppState>,
    _account: CurrentAccount,
) -> Result<impl IntoResponse, AuthError> {
    let creds = match instagram_credentials(&state) {
        Ok(creds) => creds,
        Err((status, detail)) => return Err(AuthError(status, detail)),
    };

    prune_expired_ig_states();
    let state_token = format!("{}{}", Uuid::new_v4().simple(), Uuid::new_v4().simple());
    if let Ok(mut map) = pending_ig_states().lock() {
        map.insert(state_token.clone(), Utc::now().timestamp());
    }

    let query = [
        ("client_id", creds.app_id.as_str()),
        ("redirect_uri", creds.redirect_uri.as_str()),
        ("response_type", "code"),
        ("scope", IG_SCOPES),
        ("state", state_token.as_str()),
    ];
    let pairs: Vec<String> = query
        .iter()
        .map(|(k, v)| format!("{k}={}", urlencode_component(v)))
        .collect();
    let authorize_url = format!(
        "https://www.instagram.com/oauth/authorize?{}",
        pairs.join("&")
    );

    Ok(Json(json!({
        "authorizeUrl": authorize_url,
        "state": state_token,
        "scopes": IG_SCOPES,
        "stateExpiresInSeconds": IG_STATE_TTL_SECS,
    })))
}

#[derive(serde::Deserialize)]
pub struct InstagramCallbackQuery {
    code: Option<String>,
    state: Option<String>,
    error: Option<String>,
    #[serde(rename = "error_description")]
    error_description: Option<String>,
    #[serde(rename = "error_reason")]
    error_reason: Option<String>,
}

/// Turns a provider error response into a readable message, keeping Meta's own
/// wording so the operator sees what Instagram actually said.
fn ig_error_body(body: &serde_json::Value, status: u16) -> String {
    if let Some(msg) = body
        .pointer("/error_message")
        .or_else(|| body.pointer("/error/message"))
        .or_else(|| body.get("error_description"))
        .or_else(|| body.get("error"))
        .and_then(|v| v.as_str())
    {
        return format!("Instagram/Meta (HTTP {status}): {msg}");
    }
    format!(
        "Instagram/Meta (HTTP {status}): {}",
        truncate_chars(&body.to_string(), 300)
    )
}

fn truncate_chars(input: &str, max: usize) -> String {
    if input.chars().count() <= max {
        return input.to_string();
    }
    let mut out: String = input.chars().take(max).collect();
    out.push('…');
    out
}

pub async fn instagram_oauth_callback(
    State(state): State<AppState>,
    Query(params): Query<InstagramCallbackQuery>,
) -> Result<impl IntoResponse, AuthError> {
    if let Some(error) = params.error.filter(|e| !e.is_empty()) {
        let detail = params
            .error_description
            .or(params.error_reason)
            .unwrap_or_else(|| format!("Instagram authorization failed: {error}"));
        return Err(AuthError(StatusCode::BAD_REQUEST, detail));
    }

    let code = params.code.filter(|c| !c.trim().is_empty());
    let login_state = params.state.filter(|s| !s.trim().is_empty());
    let (Some(code), Some(login_state)) = (code, login_state) else {
        return Err(AuthError(
            StatusCode::BAD_REQUEST,
            "Missing authorization code or state parameter.".to_string(),
        ));
    };

    prune_expired_ig_states();
    if !consume_ig_state(login_state.trim()) {
        return Err(AuthError(
            StatusCode::BAD_REQUEST,
            "Invalid, expired, or already-used OAuth state parameter.".to_string(),
        ));
    }

    let creds = match instagram_credentials(&state) {
        Ok(creds) => creds,
        Err((status, detail)) => return Err(AuthError(status, detail)),
    };

    // Step 1: code -> short-lived token.
    let short_resp = state
        .http_client
        .post("https://api.instagram.com/oauth/access_token")
        .timeout(std::time::Duration::from_secs(20))
        .form(&[
            ("client_id", creds.app_id.as_str()),
            ("client_secret", creds.app_secret.as_str()),
            ("grant_type", "authorization_code"),
            ("redirect_uri", creds.redirect_uri.as_str()),
            ("code", code.as_str()),
        ])
        .send()
        .await
        .map_err(|err| {
            AuthError(
                StatusCode::BAD_GATEWAY,
                format!("Could not reach api.instagram.com to exchange the code: {err}"),
            )
        })?;
    let short_status = short_resp.status().as_u16();
    let short_body: serde_json::Value = short_resp.json().await.unwrap_or_else(|_| json!({}));
    if !short_body.get("access_token").and_then(|v| v.as_str()).is_some() {
        return Err(AuthError(
            StatusCode::BAD_GATEWAY,
            ig_error_body(&short_body, short_status),
        ));
    }
    let ig_user_id = short_body
        .get("user_id")
        .and_then(|v| match v {
            serde_json::Value::String(s) => Some(s.clone()),
            serde_json::Value::Number(n) => Some(n.to_string()),
            _ => None,
        })
        .unwrap_or_default();
    let short_token = short_body["access_token"].as_str().unwrap_or("").to_string();

    if ig_user_id.is_empty() {
        return Err(AuthError(
            StatusCode::BAD_GATEWAY,
            "Instagram returned a token but no user id, so the account cannot be stored.".to_string(),
        ));
    }

    // Step 2: short-lived -> long-lived (60 days).
    let long_resp = state
        .http_client
        .get("https://graph.instagram.com/access_token")
        .timeout(std::time::Duration::from_secs(20))
        .query(&[
            ("grant_type", "ig_exchange_token"),
            ("client_secret", creds.app_secret.as_str()),
            ("access_token", short_token.as_str()),
        ])
        .send()
        .await
        .map_err(|err| {
            AuthError(
                StatusCode::BAD_GATEWAY,
                format!("Could not reach graph.instagram.com to extend the token: {err}"),
            )
        })?;
    let long_status = long_resp.status().as_u16();
    let long_body: serde_json::Value = long_resp.json().await.unwrap_or_else(|_| json!({}));
    let access_token = match long_body.get("access_token").and_then(|v| v.as_str()) {
        Some(token) if !token.contains('•') => token.to_string(),
        Some(_) => {
            return Err(AuthError(
                StatusCode::BAD_GATEWAY,
                "Instagram returned a masked access token; refusing to store it.".to_string(),
            ))
        }
        None => {
            return Err(AuthError(
                StatusCode::BAD_GATEWAY,
                ig_error_body(&long_body, long_status),
            ))
        }
    };
    let expires_at = long_body
        .get("expires_in")
        .and_then(|v| v.as_i64())
        .map(|secs| (Utc::now() + ChronoDuration::seconds(secs)).to_rfc3339());

    // Step 3: read the real profile so the inbox is labelled with the account
    // Instagram says it is, not with a placeholder.
    let profile_url = format!("https://graph.instagram.com/{ig_user_id}");
    let profile_resp = state
        .http_client
        .get(&profile_url)
        .timeout(std::time::Duration::from_secs(20))
        .query(&[("fields", "username"), ("access_token", access_token.as_str())])
        .send()
        .await;
    let (ig_username, profile_error) = match profile_resp {
        Ok(resp) => {
            let status = resp.status().as_u16();
            let body: serde_json::Value = resp.json().await.unwrap_or_else(|_| json!({}));
            match body.get("username").and_then(|v| v.as_str()) {
                Some(name) => (name.to_string(), None),
                None => (
                    String::new(),
                    Some(truncate_chars(&ig_error_body(&body, status), 300)),
                ),
            }
        }
        Err(err) => (String::new(), Some(format!("Profile lookup failed: {err}"))),
    };

    // Persist on the Instagram inbox, seeding one if the workspace has none.
    let now = Utc::now().to_rfc3339();
    let existing: Option<(String, String)> =
        sqlx::query_as("SELECT id, credentials FROM inboxes WHERE channel_type = 'instagram' LIMIT 1")
            .fetch_optional(&state.db)
            .await
            .unwrap_or(None);
    let had_inbox = existing.is_some();

    let display_username = if ig_username.is_empty() {
        String::new()
    } else {
        format!("@{ig_username}")
    };

    let (inbox_id, credentials_json) = match existing {
        Some((id, raw)) => {
            // Merge into whatever the inbox already holds instead of replacing it.
            let mut base: serde_json::Map<String, serde_json::Value> =
                serde_json::from_str(&raw).unwrap_or_default();
            base.insert("igId".to_string(), json!(ig_user_id));
            base.insert("igUsername".to_string(), json!(display_username));
            base.insert("accessToken".to_string(), json!(access_token));
            // Webhook payloads are signed with the app secret regardless of which
            // account sent them; without it inbound signature checks have nothing
            // to verify against.
            base.insert("appSecret".to_string(), json!(creds_app_secret(&state)));
            base.insert("tokenExpiresAt".to_string(), json!(expires_at));
            base.insert("connectedAt".to_string(), json!(now));
            (
                id,
                serde_json::to_string(&base).unwrap_or_else(|_| "{}".to_string()),
            )
        }
        None => {
            let id = Uuid::new_v4().to_string();
            let creds = json!({
                "igId": ig_user_id,
                "igUsername": display_username,
                "accessToken": access_token,
                "appSecret": creds_app_secret(&state),
                "tokenExpiresAt": expires_at,
                "connectedAt": now,
            });
            (id, serde_json::to_string(&creds).unwrap_or_else(|_| "{}".to_string()))
        }
    };

    let stored = if had_inbox {
        sqlx::query("UPDATE inboxes SET credentials = ?, is_active = 1, updated_at = ? WHERE id = ?")
            .bind(&credentials_json)
            .bind(&now)
            .bind(&inbox_id)
            .execute(&state.db)
            .await
    } else {
        sqlx::query(
            "INSERT INTO inboxes (id, name, channel_type, phone_number, is_active, credentials, created_at, updated_at) VALUES (?, 'Instagram Direct', 'instagram', '', 1, ?, ?, ?)",
        )
        .bind(&inbox_id)
        .bind(&credentials_json)
        .bind(&now)
        .bind(&now)
        .execute(&state.db)
        .await
    };

    if let Err(err) = stored {
        return Err(AuthError(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!(
                "Instagram issued a token but it could not be stored locally: {err}. Reconnect once the database is healthy."
            ),
        ));
    }

    Ok(Json(json!({
        "status": "connected",
        "igId": ig_user_id,
        "igUsername": ig_username,
        "inboxId": inbox_id,
        "tokenExpiresAt": expires_at,
        "profileLookupWarning": profile_error,
        "liveExchangePerformed": true,
    })))
}

fn creds_app_secret(state: &AppState) -> String {
    env_value("META_INSTAGRAM_APP_SECRET")
        .unwrap_or_else(|| state.config.meta_instagram_app_secret.clone())
}
