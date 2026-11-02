use crate::db::AppState;
use crate::services::nid_client::{NidError, SessionClaims};
use axum::extract::{FromRequestParts, Query, State};
use axum::http::{header::AUTHORIZATION, request::Parts, HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;

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
