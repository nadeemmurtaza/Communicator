#![allow(dead_code)]

use crate::config::Config;
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

/// Result of a successful nID account sign-up (phone + OTP verification).
#[derive(Clone, Debug, Serialize)]
pub struct SignUpResult {
    pub session_token: String,
    pub account_id: String,
    pub phone: String,
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

pub struct NidClient {
    http: reqwest::Client,
    base_url: String,
    client_id: String,
    client_secret: String,
    redirect_uri: String,
    allow_mock_auth: bool,
    pending: RwLock<HashMap<String, PendingState>>,
    sessions: RwLock<HashMap<String, SessionClaims>>,
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
    pub fn from_config(config: &Config) -> Self {
        Self {
            http: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(10))
                .build()
                .unwrap_or_default(),
            base_url: config.nid_base_url.trim_end_matches('/').to_string(),
            client_id: config.nid_client_id.clone(),
            client_secret: config.nid_client_secret.clone(),
            redirect_uri: config.nid_redirect_uri.clone(),
            allow_mock_auth: config.nid_allow_mock_auth,
            pending: RwLock::new(HashMap::new()),
            sessions: RwLock::new(HashMap::new()),
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
    // Sign Up — Account creation via nID phone + OTP
    // ------------------------------------------------------------------

    /// Sends an OTP code to the given phone number via nID.
    /// nID endpoint: POST /v1/accounts/otp/start
    pub async fn start_sign_up(&self, phone: &str) -> Result<(), NidError> {
        let response = self
            .http
            .post(format!("{}/v1/accounts/otp/start", self.base_url))
            .json(&serde_json::json!({ "phone": phone }))
            .send()
            .await
            .map_err(|err| NidError::Unreachable(err.to_string()))?;

        let status = response.status();
        if !status.is_success() {
            let body: serde_json::Value = response.json().await.unwrap_or_else(|_| json!({}));
            let detail = body
                .pointer("/message")
                .and_then(|v| v.as_str())
                .unwrap_or("nID rejected the OTP request");
            return Err(NidError::OtpFailed(detail.to_string()));
        }
        Ok(())
    }

    /// Verifies the OTP code and creates (or logs in to) the nID account.
    /// nID endpoint: POST /v1/accounts/otp/verify
    /// Returns a session token and account identity on success.
    pub async fn verify_sign_up(&self, phone: &str, code: &str) -> Result<SignUpResult, NidError> {
        let response = self
            .http
            .post(format!("{}/v1/accounts/otp/verify", self.base_url))
            .json(&serde_json::json!({ "phone": phone, "code": code }))
            .send()
            .await
            .map_err(|err| NidError::Unreachable(err.to_string()))?;

        let status = response.status();
        if !status.is_success() {
            let body: serde_json::Value = response.json().await.unwrap_or_else(|_| json!({}));
            let detail = body
                .pointer("/message")
                .and_then(|v| v.as_str())
                .unwrap_or("OTP verification failed");
            return Err(NidError::OtpFailed(detail.to_string()));
        }

        let data: serde_json::Value =
            response.json().await.map_err(|_| NidError::MalformedResponse)?;

        let session_token = data
            .get("sessionToken")
            .and_then(|v| v.as_str())
            .filter(|v| !v.is_empty())
            .ok_or(NidError::MalformedResponse)?
            .to_string();

        let account = data.get("account").ok_or(NidError::MalformedResponse)?;
        let account_id = account
            .get("id")
            .and_then(|v| v.as_str())
            .filter(|v| !v.is_empty())
            .ok_or(NidError::MalformedResponse)?
            .to_string();

        // Store the session so validate_session can find it later.
        let claims = SessionClaims {
            account_id: account_id.clone(),
            phone: Some(phone.to_string()),
            email: account.get("email").and_then(|v| v.as_str()).map(String::from),
            phone_verified: account
                .get("phoneVerifiedAt")
                .map(|_| true)
                .unwrap_or(false),
            access_token: session_token.clone(),
        };
        self.sessions
            .write()
            .await
            .insert(session_token.clone(), claims);

        Ok(SignUpResult {
            session_token,
            account_id,
            phone: phone.to_string(),
        })
    }
}
