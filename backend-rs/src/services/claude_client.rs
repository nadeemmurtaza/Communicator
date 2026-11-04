//! The Anthropic Messages client.
//!
//! Port of `ai/claude_client.py`. The Python version had three problems this
//! module fixes:
//!
//! * `settings.anthropic_api_key` defaulted to `""`, so an unconfigured
//!   deployment still POSTed an empty `x-api-key` and the 401 surfaced as an
//!   opaque `HTTPStatusError` -- here an absent key is a typed, local refusal
//!   that names the env var;
//! * `data["content"][0]["text"]` panicked on any response shape it did not
//!   expect (including `stop_reason: "max_tokens"` truncation);
//! * the model name came from a compiled-in default, so a typo in
//!   `ANTHROPIC_MODEL` produced a provider error nobody could trace. The
//!   resolved model and its source are returned with every reply.
//!
//! Provider errors are propagated as [`ClaudeError::Api`] with Anthropic's own
//! `error.type` / `error.message`, never swallowed into an empty string.

use crate::db::AppState;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::env;
use std::time::Duration;

/// The Messages API host. Overridable for gateways/proxies, never for a fake
/// local endpoint.
pub const DEFAULT_BASE_URL: &str = "https://api.anthropic.com";
/// `anthropic-version` is part of the wire contract, not tenant data.
pub const DEFAULT_API_VERSION: &str = "2023-06-01";
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(60);

#[derive(Debug, Clone)]
pub struct ClaudeConfig {
    pub api_key: String,
    /// Where the key came from, so a 401 can be traced to the right place.
    pub key_source: String,
    pub model: String,
    pub model_source: String,
    pub base_url: String,
    pub api_version: String,
    pub max_tokens: u32,
    pub timeout: Duration,
}

impl ClaudeConfig {
    /// Resolves credentials in the order the rest of the backend uses: the
    /// process environment (which the vault loader has already written into the
    /// process before `Config::from_env`), then the vault-applied `Config`, then
    /// `integration_settings` is *not* consulted -- no AI key lives in that
    /// table, and inventing a column would be exactly the fiction this port
    /// avoids.
    ///
    /// `Err` means a call cannot be made honestly: no key, or no model.
    pub fn from_state(state: &AppState) -> Result<Self, ClaudeError> {
        let (api_key, key_source) = lookup(
            &["ANTHROPIC_API_KEY", "CLAUDE_API_KEY", "ANTHROPIC_AUTH_TOKEN"],
            state.config.anthropic_api_key.as_str(),
        )
        .ok_or_else(|| {
            ClaudeError::NotConfigured(
                "ANTHROPIC_API_KEY is not set (searched ANTHROPIC_API_KEY, CLAUDE_API_KEY, \
                 ANTHROPIC_AUTH_TOKEN and Config.anthropic_api_key). No request was made to \
                 api.anthropic.com and no reply was generated."
                    .to_string(),
            )
        })?;

        // The model is required from the deployment; the compiled default in
        // `Config::from_env` is reported with its source so a wrong model is at
        // least traceable, and an explicit env value always wins.
        let (model, model_source) = lookup(
            &["ANTHROPIC_MODEL", "CLAUDE_MODEL", "ANTHROPIC_DEFAULT_MODEL"],
            state.config.anthropic_model.as_str(),
        )
        .ok_or_else(|| {
            ClaudeError::NotConfigured(
                "No Claude model configured: set ANTHROPIC_MODEL. Nothing was sent, because a \
                 guessed model name would fail at the provider anyway."
                    .to_string(),
            )
        })?;

        let base_url = env::var("ANTHROPIC_BASE_URL")
            .or_else(|_| env::var("CLAUDE_API_BASE_URL"))
            .map(|value| value.trim().to_string())
            .ok()
            .filter(|value| is_usable(value))
            .unwrap_or_else(|| DEFAULT_BASE_URL.to_string());
        // `anthropic-version` is part of the wire contract. `Config` has no
        // field for it, so only an explicit env override changes it.
        let api_version = env::var("ANTHROPIC_VERSION")
            .map(|value| value.trim().to_string())
            .ok()
            .filter(|value| is_usable(value))
            .unwrap_or_else(|| DEFAULT_API_VERSION.to_string());

        Ok(Self {
            api_key,
            key_source,
            model,
            model_source,
            base_url: base_url.trim_end_matches('/').to_string(),
            api_version,
            max_tokens: 1024,
            timeout: DEFAULT_TIMEOUT,
        })
    }

    pub fn with_max_tokens(mut self, max_tokens: u32) -> Self {
        self.max_tokens = max_tokens;
        self
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }
}

/// Env-first lookup that also rejects masked/placeholder values.
fn lookup(names: &[&str], config_value: &str) -> Option<(String, String)> {
    for name in names {
        if let Ok(value) = env::var(name) {
            let value = value.trim().to_string();
            if is_usable(&value) {
                return Some((value, format!("env {name}")));
            }
        }
    }
    let value = config_value.trim();
    if is_usable(value) {
        return Some((value.to_string(), "Config (vault or boot env)".to_string()));
    }
    None
}

fn is_usable(value: &str) -> bool {
    !value.is_empty()
        && !value.contains('•')
        && !value.contains("****")
        && !value.to_ascii_lowercase().starts_with("sk-ant-api03-placeholder")
        && !value.to_ascii_lowercase().contains("changeme")
        && !value.to_ascii_lowercase().starts_with("mock")
        && !value.to_ascii_lowercase().starts_with("test")
        && !value.eq_ignore_ascii_case("none")
        && !value.eq_ignore_ascii_case("null")
}

// ---------------------------------------------------------------------------
// Typed wire types
// ---------------------------------------------------------------------------

/// Anthropic's field names are snake_case, so no renaming is applied; the
/// request body is assembled in [`complete_with`] with those exact keys.
#[derive(Clone, Debug, Serialize)]
pub struct MessageRequest {
    pub model: String,
    pub max_tokens: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<String>,
    pub messages: Vec<WireMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    /// Requested output kind, tracked so `parse_json_reply` knows a JSON body
    /// was expected and a prose answer is reported as a real failure.
    #[serde(skip)]
    pub expect_json: bool,
}

#[derive(Clone, Debug, Serialize)]
pub struct WireMessage {
    pub role: &'static str,
    pub content: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct MessageResponse {
    pub id: String,
    #[serde(default)]
    pub model: String,
    #[serde(default)]
    pub stop_reason: Option<String>,
    #[serde(default)]
    pub usage: Option<Usage>,
    #[serde(default)]
    pub content: Vec<ContentBlock>,
    #[serde(default)]
    pub error: Option<ProviderError>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ContentBlock {
    #[serde(rename = "type", default)]
    pub block_type: String,
    #[serde(default)]
    pub text: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Usage {
    #[serde(default)]
    pub input_tokens: Option<i64>,
    #[serde(default)]
    pub output_tokens: Option<i64>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ProviderError {
    #[serde(rename = "type", default)]
    pub error_type: String,
    #[serde(default)]
    pub message: String,
}

/// A successful completion, with the fields an operator needs to audit it.
#[derive(Clone, Debug)]
pub struct ClaudeReply {
    pub text: String,
    pub provider_message_id: String,
    pub model: String,
    pub model_source: String,
    pub stop_reason: Option<String>,
    pub input_tokens: Option<i64>,
    pub output_tokens: Option<i64>,
    pub http_status: u16,
    pub latency_ms: i64,
    pub raw: Value,
}

impl ClaudeReply {
    /// `stop_reason == "max_tokens"` means the model was cut off mid-answer.
    /// Callers that ask for JSON must check this: a truncated JSON body is not
    /// a parseable one, and treating it as a success is how silent data loss
    /// starts.
    pub fn truncated(&self) -> bool {
        self.stop_reason.as_deref() == Some("max_tokens")
    }

    pub fn to_json(&self) -> Value {
        json!({
            "text": self.text,
            "providerMessageId": self.provider_message_id,
            "model": self.model,
            "modelSource": self.model_source,
            "stopReason": self.stop_reason,
            "truncated": self.truncated(),
            "inputTokens": self.input_tokens,
            "outputTokens": self.output_tokens,
            "httpStatus": self.http_status,
            "latencyMs": self.latency_ms,
        })
    }
}

/// Everything that can go wrong, as a distinct typed case.
#[derive(Debug, Clone)]
pub enum ClaudeError {
    /// Local refusal: the call was never made and the reason names the field.
    NotConfigured(String),
    /// The provider answered with an error envelope or a non-2xx status.
    Api {
        status: Option<u16>,
        error_type: String,
        message: String,
        raw: Value,
    },
    /// The socket/timeout layer failed.
    Transport(String),
    /// 2xx but no text block -- reported instead of returning `""` as if the
    /// model answered.
    EmptyCompletion { raw: Value },
    /// The reply was not the shape the caller asked for.
    Unparseable { reason: String, text: String },
}

impl ClaudeError {
    pub fn message(&self) -> String {
        match self {
            Self::NotConfigured(reason) => reason.clone(),
            Self::Api { status, error_type, message, .. } => {
                format!("Anthropic error {error_type} (HTTP {:?}): {message}", status)
            }
            Self::Transport(reason) => format!("Could not reach api.anthropic.com: {reason}"),
            Self::EmptyCompletion { .. } => {
                "Anthropic returned 2xx with no text content block".to_string()
            }
            Self::Unparseable { reason, .. } => reason.clone(),
        }
    }

    pub fn is_local_refusal(&self) -> bool {
        matches!(self, Self::NotConfigured(_))
    }

    pub fn to_json(&self) -> Value {
        json!({
            "kind": match self {
                Self::NotConfigured(_) => "not_configured",
                Self::Api { .. } => "provider_error",
                Self::Transport(_) => "transport",
                Self::EmptyCompletion { .. } => "empty_completion",
                Self::Unparseable { .. } => "unparseable",
            },
            "message": self.message(),
            "requestWasMade": !self.is_local_refusal(),
            "httpStatus": match self { Self::Api { status, .. } => status.map(|s| s as u16).map(Value::from).unwrap_or(Value::Null), _ => Value::Null },
            "raw": match self { Self::Api { raw, .. } | Self::EmptyCompletion { raw } => raw.clone(), _ => Value::Null },
        })
    }
}

impl std::fmt::Display for ClaudeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message())
    }
}

impl std::error::Error for ClaudeError {}

// ---------------------------------------------------------------------------
// The call
// ---------------------------------------------------------------------------

/// Port of `call_claude(system, user, max_tokens) -> str`, but it returns the
/// typed reply so the caller can see the model, the stop reason and the token
/// use rather than a bare string.
pub async fn complete(state: &AppState, request: &MessageRequest) -> Result<ClaudeReply, ClaudeError> {
    let config = ClaudeConfig::from_state(state)?;
    complete_with(&state.http_client, &config, request).await
}

/// Split out so a caller that already resolved a [`ClaudeConfig`] (for example
/// one pinned to a specific model) is not re-reading the environment.
pub async fn complete_with(
    client: &reqwest::Client,
    config: &ClaudeConfig,
    request: &MessageRequest,
) -> Result<ClaudeReply, ClaudeError> {
    let model = if request.model.is_empty() { config.model.clone() } else { request.model.clone() };
    let max_tokens = if request.max_tokens == 0 { config.max_tokens } else { request.max_tokens };

    let payload = json!({
        "model": model,
        "max_tokens": max_tokens,
        "messages": request.messages,
        "system": request.system,
        "temperature": request.temperature,
    });
    let payload = strip_nulls(payload);

    let url = format!("{}/v1/messages", config.base_url);
    let started = Utc::now();
    let response = client
        .post(&url)
        .header("x-api-key", &config.api_key)
        .header("anthropic-version", &config.api_version)
        .header("content-type", "application/json")
        .timeout(config.timeout)
        .json(&payload)
        .send()
        .await;

    let response = match response {
        Ok(response) => response,
        Err(err) => return Err(ClaudeError::Transport(err.to_string())),
    };

    let status = response.status();
    let body: Value = match response.json::<Value>().await {
        Ok(body) => body,
        Err(err) => {
            return Err(ClaudeError::Api {
                status: Some(status.as_u16()),
                error_type: if status.is_success() { "invalid_json_body".to_string() } else { "http_error".to_string() },
                message: format!("Anthropic answered HTTP {} with a non-JSON body: {err}", status.as_u16()),
                raw: Value::Null,
            });
        }
    };

    let latency_ms = Utc::now().signed_duration_since(started).num_milliseconds();

    // An error envelope is reported with Anthropic's own type and text; nothing
    // downstream can mistake it for a completion.
    if !status.is_success() || body.get("type").and_then(Value::as_str) == Some("error") {
        let parsed: Result<MessageResponse, serde_json::Error> = serde_json::from_value(body.clone());
        let (error_type, message) = match parsed {
            Ok(reply) => match reply.error {
                Some(err) => (err.error_type, err.message),
                None => (
                    body.get("type").and_then(Value::as_str).unwrap_or("error").to_string(),
                    body.to_string(),
                ),
            },
            Err(_) => {
                let message = body
                    .get("error")
                    .and_then(|e| e.get("message"))
                    .and_then(Value::as_str)
                    .unwrap_or("");
                (
                    body.get("error")
                        .and_then(|e| e.get("type"))
                        .and_then(Value::as_str)
                        .unwrap_or("unknown_error")
                        .to_string(),
                    if message.is_empty() { body.to_string() } else { message.to_string() },
                )
            }
        };
        tracing::warn!("Anthropic HTTP {} ({error_type}): {message}", status.as_u16());
        return Err(ClaudeError::Api {
            status: Some(status.as_u16()),
            error_type,
            message,
            raw: body,
        });
    }

    let parsed: MessageResponse = serde_json::from_value(body.clone()).map_err(|err| {
        ClaudeError::Api {
            status: Some(status.as_u16()),
            error_type: "undecodable_response".to_string(),
            message: format!("Anthropic's 2xx body did not match the Messages schema: {err}"),
            raw: body.clone(),
        }
    })?;

    // Concatenate every text block: a reply that interleaves thinking/tool
    // blocks has more than one, and taking `[0]` (the Python behaviour) drops
    // the answer.
    let text = parsed
        .content
        .iter()
        .filter(|block| block.block_type == "text" || block.block_type.is_empty())
        .map(|block| block.text.clone().unwrap_or_default())
        .collect::<Vec<_>>()
        .join("");

    if text.trim().is_empty() {
        return Err(ClaudeError::EmptyCompletion { raw: body });
    }

    let mut reply = ClaudeReply {
        text,
        provider_message_id: parsed.id,
        model: if parsed.model.is_empty() { model.clone() } else { parsed.model },
        model_source: config.model_source.clone(),
        stop_reason: parsed.stop_reason,
        input_tokens: parsed.usage.as_ref().and_then(|u| u.input_tokens),
        output_tokens: parsed.usage.as_ref().and_then(|u| u.output_tokens),
        http_status: status.as_u16(),
        latency_ms,
        raw: body,
    };
    if reply.model.trim().is_empty() {
        // The echo can be absent on gateways; report the model we asked for
        // rather than leaving the audit field blank.
        reply.model = model;
    }
    Ok(reply)
}

fn strip_nulls(value: Value) -> Value {
    match value {
        Value::Object(map) => Value::Object(
            map.into_iter()
                .filter(|(_, v)| !v.is_null())
                .map(|(k, v)| (k, strip_nulls(v)))
                .collect(),
        ),
        Value::Array(items) => Value::Array(items.into_iter().map(strip_nulls).collect()),
        other => other,
    }
}

/// The two-argument shape the rest of the ported services call.
pub async fn call_claude(
    state: &AppState,
    system: &str,
    user: &str,
    max_tokens: u32,
) -> Result<String, ClaudeError> {
    let request = MessageRequest {
        model: String::new(),
        max_tokens,
        system: Some(system.to_string()),
        messages: vec![WireMessage { role: "user", content: user.to_string() }],
        temperature: None,
        expect_json: false,
    };
    Ok(complete(state, &request).await?.text)
}

/// A completion that must parse as JSON: a cut-off or prose answer is an error,
/// not an empty object.
pub async fn ask_json(
    state: &AppState,
    system: &str,
    user: &str,
    max_tokens: u32,
) -> Result<Value, ClaudeError> {
    let request = MessageRequest {
        model: String::new(),
        max_tokens,
        system: Some(system.to_string()),
        messages: vec![WireMessage { role: "user", content: user.to_string() }],
        temperature: None,
        expect_json: true,
    };
    let reply = complete(state, &request).await?;
    if reply.truncated() {
        return Err(ClaudeError::Unparseable {
            reason: format!(
                "The model stopped at max_tokens ({}) before finishing the JSON, so the answer is \
                 incomplete and was not guessed. Raise max_tokens or shorten the prompt.",
                max_tokens
            ),
            text: reply.text,
        });
    }
    parse_json_reply(&reply.text)
}

/// Port of `parse_json_reply`, made tolerant of the three shapes a model
/// actually returns (bare JSON, a ```json fence, JSON wrapped in prose) and
/// honest about the ones it cannot recover.
pub fn parse_json_reply(text: &str) -> Result<Value, ClaudeError> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return Err(ClaudeError::Unparseable {
            reason: "The model returned an empty body.".to_string(),
            text: text.to_string(),
        });
    }

    let candidate = if trimmed.starts_with("```") {
        let without_fences = trimmed.trim_matches('`');
        match without_fences.split_once('\n') {
            Some(("json", body)) | Some(("", body)) => body.trim(),
            Some((_, body)) => body.trim(),
            None => without_fences.trim(),
        }
    } else {
        trimmed
    };

    if let Ok(value) = serde_json::from_str::<Value>(candidate) {
        return Ok(value);
    }
    // Recover a single object/array embedded in surrounding prose.
    if let Some(value) = slice_first_json(candidate) {
        if let Ok(parsed) = serde_json::from_str::<Value>(&value) {
            return Ok(parsed);
        }
    }

    Err(ClaudeError::Unparseable {
        reason: "The model reply was not JSON and no complete JSON object could be read from it."
            .to_string(),
        text: text.to_string(),
    })
}

fn slice_first_json(text: &str) -> Option<String> {
    let bytes = text.as_bytes();
    let start = text.find(['{', '['])?;
    let open = bytes[start];
    let close = if open == b'{' { b'}' } else { b']' };
    let mut depth = 0usize;
    let mut in_string = false;
    let mut escaped = false;
    for (index, byte) in bytes.iter().enumerate().skip(start) {
        if escaped {
            escaped = false;
            continue;
        }
        match byte {
            b'\\' if in_string => escaped = true,
            b'"' => in_string = !in_string,
            b'{' | b'[' if !in_string => depth += 1,
            b'}' | b']' if !in_string => {
                if depth == 0 {
                    // A closer that does not belong to anything we opened: the
                    // candidate is not balanced JSON, so stop looking.
                    return None;
                }
                depth -= 1;
                if depth == 0 && *byte == close {
                    return Some(text[start..=index].to_string());
                }
            }
            _ => {}
        }
    }
    None
}

/// Cheap, network-free status for the health endpoint: reports whether a call
/// *would* be possible and what is missing, never pretending to verify the key.
pub fn configuration_report(state: &AppState) -> Value {
    match ClaudeConfig::from_state(state) {
        Ok(config) => json!({
            "configured": true,
            "model": config.model,
            "modelSource": config.model_source,
            "keySource": config.key_source,
            "baseUrl": config.base_url,
            "apiVersion": config.api_version,
            "note": "credentials are present; the key itself is only validated by a real call",
        }),
        Err(err) => json!({
            "configured": false,
            "detail": err.to_json(),
        }),
    }
}
