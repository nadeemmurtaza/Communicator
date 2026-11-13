#![allow(dead_code)]

use crate::config::Config;
use crate::db::DbPool;
use crate::services::otp_engine::{GenerateOtpRequest, OtpEngine, VerifyOtpRequest};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use tracing::warn;
use uuid::Uuid;

const STATE_TTL_SECS: u64 = 600;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SessionClaims {
    pub account_id: String,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub phone_verified: bool,
    pub access_token: String,
}

#[derive(Clone, Debug)]
pub struct LoginStart {
    pub authorize_url: String,
    pub poll_token: String,
    pub state: String,
}

#[derive(Clone, Debug)]
pub struct PendingState {
    created_at: u64,
    redirect_uri: String,
}

/// Result of a successful nID account sign-up (email + OTP verification).
#[derive(Clone, Debug, Serialize)]
pub struct SignUpResult {
    pub session_token: String,
    pub account_id: String,
    pub email: String,
    pub otp_id: String,
}

#[derive(Debug)]
pub enum NidError {
    NotConfigured,
    Unreachable(String),
    Rejected(u16),
    MalformedResponse,
    InvalidState,
    MissingIdentity,
    OtpFailed(String),
}

/// Pending OTP sign-up request — maps email -> otp_id so verify_sign_up can find it.
#[derive(Clone, Debug)]
struct PendingOtp {
    otp_id: String,
    created_at: u64,
}

pub struct NidClient {
    http: reqwest::Client,
    pool: DbPool,
    base_url: String,
    client_id: String,
    client_secret: String,
    redirect_uri: String,
    allow_mock_auth: bool,
    /// Brand ID used when calling the OTP engine for nID sign-up OTPs.
    otp_brand_id: String,
    pending: RwLock<HashMap<String, PendingState>>,
    sessions: RwLock<HashMap<String, SessionClaims>>,
    /// Maps email -> pending OTP info for sign-up flow.
    pending_otp: RwLock<HashMap<String, PendingOtp>>,
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn random_hex(bytes: usize) -> String {
    let mut out = String::with_capacity(bytes * 2);
    while out.len() < bytes * 2 {
        out.push_str(&Uuid::new_v4().simple().to_string());
    }
    out.truncate(bytes * 2);
    out
}

fn fnv1a_hex(text: &str) -> String {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in text.bytes() {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    format!("{hash:08x}")
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Origin {
    scheme: String,
    host: String,
    port: u16,
}

/// Parses the scheme/host/port of a URI without a URL crate. Splitting the
/// authority at the *last* '@' is deliberate: `http://localhost@evil.test`
/// must resolve to `evil.test`, not `localhost`.
fn parse_origin(raw: &str) -> Option<Origin> {
    let (scheme, rest) = raw.split_once("://")?;
    let scheme = scheme.to_ascii_lowercase();
    let default_port = match scheme.as_str() {
        "http" => 80,
        "https" => 443,
        _ => return None,
    };
    let authority = rest.split(['/', '?', '#']).next().unwrap_or("");
    let host_port = authority.rsplit('@').next().unwrap_or(authority);
    if host_port.is_empty() {
        return None;
    }
    let (host, port) = match host_port.strip_prefix('[') {
        Some(v6) => {
            let (addr, tail) = v6.split_once(']')?;
            let port = match tail.strip_prefix(':') {
                Some(p) => p.parse().ok()?,
                None => default_port,
            };
            (addr.to_ascii_lowercase(), port)
        }
        None => match host_port.rsplit_once(':') {
            Some((h, p)) => (h.to_ascii_lowercase(), p.parse().ok()?),
            None => (host_port.to_ascii_lowercase(), default_port),
        },
    };
    if host.is_empty() {
        return None;
    }
    Some(Origin {
        scheme,
        host,
        port,
    })
}

impl NidClient {
    pub fn from_config(pool: &DbPool, config: &Config) -> Self {
        Self {
            http: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(10))
                .build()
                .unwrap_or_default(),
            pool: pool.clone(),
            base_url: config.nid_base_url.trim_end_matches('/').to_string(),
            client_id: config.nid_client_id.clone(),
            client_secret: config.nid_client_secret.clone(),
            redirect_uri: config.nid_redirect_uri.clone(),
            allow_mock_auth: config.nid_allow_mock_auth,
            otp_brand_id: config.nid_otp_brand_id.clone(),
            pending: RwLock::new(HashMap::new()),
            sessions: RwLock::new(HashMap::new()),
            pending_otp: RwLock::new(HashMap::new()),
        }
    }

    pub fn is_configured(&self) -> bool {
        !self.client_id.is_empty() && !self.client_secret.is_empty()
    }

    pub fn mock_auth_allowed(&self) -> bool {
        self.allow_mock_auth
    }

    fn is_allowed_redirect_uri(&self, candidate: &str) -> bool {
        let Some(parsed) = parse_origin(candidate) else {
            return false;
        };
        if parsed.host == "localhost" {
            return true;
        }
        match parse_origin(&self.redirect_uri) {
            Some(configured) => configured == parsed,
            None => false,
        }
    }

    async fn prune_expired(&self) {
        let cutoff = now_secs().saturating_sub(STATE_TTL_SECS);
        self.pending.write().await.retain(|_, v| v.created_at > cutoff);
    }

    pub async fn start_login(&self, custom_redirect_uri: Option<&str>) -> Result<LoginStart, NidError> {
        if !self.is_configured() {
            if self.allow_mock_auth {
                return self.local_login_start().await;
            }
            return Err(NidError::NotConfigured);
        }

        self.prune_expired().await;

        let state = random_hex(16);
        let mut redirect_uri = self.redirect_uri.clone();
        if let Some(candidate) = custom_redirect_uri {
            if self.is_allowed_redirect_uri(candidate) {
                redirect_uri = candidate.to_string();
            }
        }

        let payload = serde_json::json!({
            "clientId": self.client_id,
            "redirectUri": redirect_uri,
            "state": state,
        });

        let response = self
            .http
            .post(format!("{}/v1/oauth/login-requests", self.base_url))
            .json(&payload)
            .send()
            .await;

        let data = match response {
            Err(err) => {
                warn!("nID login-request endpoint unreachable: {}", err);
                if self.allow_mock_auth {
                    return self.local_login_start().await;
                }
                return Err(NidError::Unreachable(err.to_string()));
            }
            Ok(response) => {
                let status = response.status();
                if !status.is_success() {
                    warn!(
                        "nID login-request endpoint rejected the attempt: status {}",
                        status.as_u16()
                    );
                    if self.allow_mock_auth {
                        return self.local_login_start().await;
                    }
                    return Err(NidError::Rejected(status.as_u16()));
                }
                match response.json::<serde_json::Value>().await {
                    Ok(data) => data,
                    Err(_) => return Err(NidError::MalformedResponse),
                }
            }
        };

        let authorize_url = data
            .get("authorizeUrl")
            .and_then(|v| v.as_str())
            .ok_or(NidError::MalformedResponse)?
            .to_string();
        let poll_token = data
            .get("pollToken")
            .and_then(|v| v.as_str())
            .map(ToString::to_string)
            .unwrap_or_else(|| format!("poll_{state}"));

        self.pending.write().await.insert(
            state.clone(),
            PendingState {
                created_at: now_secs(),
                redirect_uri,
            },
        );

        Ok(LoginStart {
            authorize_url,
            poll_token,
            state,
        })
    }

    /// Registers a login attempt locally so a developer who has explicitly set
    /// NID_ALLOW_MOCK_AUTH can finish the flow without a reachable nID. The
    /// claims it yields are marked unverified and are never produced when the
    /// flag is off.
    async fn local_login_start(&self) -> Result<LoginStart, NidError> {
        self.prune_expired().await;
        let state = random_hex(16);
        let poll_token = format!("mock_poll_{state}");
        self.pending.write().await.insert(
            state.clone(),
            PendingState {
                created_at: now_secs(),
                redirect_uri: self.redirect_uri.clone(),
            },
        );
        Ok(LoginStart {
            authorize_url: String::new(),
            poll_token,
            state,
        })
    }

    pub async fn exchange_code(&self, code: &str, state: &str) -> Result<SessionClaims, NidError> {
        self.prune_expired().await;
        let state_record = self
            .pending
            .write()
            .await
            .remove(state)
            .ok_or(NidError::InvalidState)?;

        if self.is_configured() {
            match self.exchange_with_provider(code, &state_record).await {
                Ok(claims) => {
                    self.sessions
                        .write()
                        .await
                        .insert(claims.access_token.clone(), claims.clone());
                    return Ok(claims);
                }
                Err(err) => warn!("nID code exchange failed: {:?}", err),
            }
        }

        if self.allow_mock_auth {
            let claims = self.mock_claims(code);
            self.sessions
                .write()
                .await
                .insert(claims.access_token.clone(), claims.clone());
            return Ok(claims);
        }

        if !self.is_configured() {
            return Err(NidError::NotConfigured);
        }
        Err(NidError::Rejected(401))
    }

    async fn exchange_with_provider(
        &self,
        code: &str,
        state_record: &PendingState,
    ) -> Result<SessionClaims, NidError> {
        let payload = serde_json::json!({
            "grant_type": "authorization_code",
            "client_id": self.client_id,
            "client_secret": self.client_secret,
            "code": code,
            "redirect_uri": state_record.redirect_uri,
        });

        let response = self
            .http
            .post(format!("{}/v1/oauth/token", self.base_url))
            .json(&payload)
            .send()
            .await
            .map_err(|err| NidError::Unreachable(err.to_string()))?;

        let status = response.status();
        if !status.is_success() {
            return Err(NidError::Rejected(status.as_u16()));
        }

        let data: serde_json::Value =
            response.json().await.map_err(|_| NidError::MalformedResponse)?;
        let account = data.get("account").unwrap_or(&serde_json::Value::Null);

        let account_id = account
            .get("id")
            .and_then(|v| v.as_str())
            .filter(|v| !v.is_empty())
            .ok_or(NidError::MissingIdentity)?
            .to_string();
        let access_token = data
            .get("access_token")
            .and_then(|v| v.as_str())
            .filter(|v| !v.is_empty())
            .ok_or(NidError::MissingIdentity)?
            .to_string();

        Ok(SessionClaims {
            account_id,
            phone: account
                .get("phone")
                .and_then(|v| v.as_str())
                .map(ToString::to_string),
            email: account
                .get("email")
                .and_then(|v| v.as_str())
                .map(ToString::to_string),
            phone_verified: account
                .get("phoneVerified")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
            access_token,
        })
    }

    fn mock_claims(&self, code: &str) -> SessionClaims {
        let account_id = format!("acc_{}", fnv1a_hex(code));
        SessionClaims {
            account_id: account_id.clone(),
            phone: None,
            email: Some(format!("{account_id}@newax.local")),
            phone_verified: false,
            access_token: random_hex(32),
        }
    }

    pub async fn validate_session(&self, token: &str) -> Option<SessionClaims> {
        self.sessions.read().await.get(token).cloned()
    }

    // ------------------------------------------------------------------
    // Sign Up — Account creation via Communicator OTP engine + email
    // ------------------------------------------------------------------

    /// Sends an OTP code to the given email address using the local OTP engine.
    /// The OTP is dispatched via email (SMTP) and tracked so verify_sign_up can validate it.
    pub async fn start_sign_up(&self, email: &str) -> Result<String, NidError> {
        // Prune expired pending OTPs (older than STATE_TTL_SECS)
        self.prune_pending_otp().await;

        // Check for rate limiting: reject if a recent OTP already exists for this email
        {
            let pending = self.pending_otp.read().await;
            if let Some(existing) = pending.get(email) {
                if now_secs() - existing.created_at < 60 {
                    return Err(NidError::OtpFailed(
                        "An OTP was recently sent to this email. Please wait before requesting another.".to_string(),
                    ));
                }
            }
        }

        // Use the local OTP engine to generate and send the OTP via email
        let brand_id = self.otp_brand_id.as_str();
        if brand_id.is_empty() {
            return Err(NidError::OtpFailed(
                "OTP engine is not configured. Set NID_OTP_BRAND_ID.".to_string(),
            ));
        }

        let req = GenerateOtpRequest {
            recipient: email.to_string(),
            recipient_type: Some("email".to_string()),
            purpose: Some("sign_up".to_string()),
            channel: Some("email".to_string()),
            metadata: None,
            ip_address: None,
            user_agent: None,
        };

        let result = OtpEngine::generate_and_send(&self.pool, brand_id, &req)
            .await
            .map_err(|err| NidError::OtpFailed(err))?;

        if !result.success {
            return Err(NidError::OtpFailed(
                result.error.unwrap_or_else(|| "OTP generation failed".to_string()),
            ));
        }

        // Store the otp_id so verify_sign_up can find it
        let pending = PendingOtp {
            otp_id: result.otp_id.clone(),
            created_at: now_secs(),
        };
        self.pending_otp.write().await.insert(email.to_string(), pending);

        Ok(result.otp_id)
    }

    /// Verifies the OTP code using the local OTP engine and creates the session.
    /// Returns a session token and account identity on success.
    pub async fn verify_sign_up(&self, email: &str, code: &str) -> Result<SignUpResult, NidError> {
        // Look up the pending OTP for this email
        let otp_id = {
            let pending = self.pending_otp.read().await;
            let pending_otp = pending.get(email).ok_or_else(|| {
                NidError::OtpFailed("No pending OTP for this email. Please request a new code.".to_string())
            })?;
            // Check expiry (same TTL as STATE_TTL_SECS = 10 minutes)
            if now_secs() - pending_otp.created_at > STATE_TTL_SECS {
                return Err(NidError::OtpFailed(
                    "OTP request has expired. Please request a new code.".to_string(),
                ));
            }
            pending_otp.otp_id.clone()
        };

        // Verify via the local OTP engine
        let verify_req = VerifyOtpRequest {
            otp_id: otp_id.clone(),
            code: code.to_string(),
            ip_address: None,
            user_agent: None,
        };

        let result = OtpEngine::verify(&self.pool, &verify_req)
            .await
            .map_err(|err| NidError::OtpFailed(err))?;

        if !result.verified {
            return Err(NidError::OtpFailed(
                result.error.unwrap_or_else(|| "OTP verification failed".to_string()),
            ));
        }

        // Remove the pending OTP entry
        self.pending_otp.write().await.remove(email);

        // Generate a session token and account ID for the new user
        let session_token = random_hex(32);
        let account_id = format!("acc_{}", fnv1a_hex(email));

        // Store the session so validate_session can find it later.
        let claims = SessionClaims {
            account_id: account_id.clone(),
            phone: None,
            email: Some(email.to_string()),
            phone_verified: false,
            access_token: session_token.clone(),
        };
        self.sessions
            .write()
            .await
            .insert(session_token.clone(), claims);

        Ok(SignUpResult {
            session_token,
            account_id,
            email: email.to_string(),
            otp_id,
        })
    }

    /// Prune expired pending OTP entries.
    async fn prune_pending_otp(&self) {
        let cutoff = now_secs().saturating_sub(STATE_TTL_SECS);
        self.pending_otp.write().await.retain(|_, v| v.created_at > cutoff);
    }
}
