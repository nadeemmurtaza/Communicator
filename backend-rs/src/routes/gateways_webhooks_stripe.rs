use crate::db::AppState;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use chrono::{TimeZone, Utc};
use hmac::{Hmac, Mac};
use serde::Deserialize;
use serde_json::json;
use sha2::{Digest, Sha256};
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Helpers for Masking and Signatures
// ---------------------------------------------------------------------------

fn mask_secret(val: &str) -> String {
    if val.is_empty() {
        return String::new();
    }
    if val.len() <= 6 {
        return "••••••".to_string();
    }
    format!("••••••••{}", &val[val.len() - 4..])
}

fn mask_webhook_secret(secret: &str) -> String {
    if secret.is_empty() {
        return String::new();
    }
    if secret.len() <= 10 {
        return "••••••••".to_string();
    }
    format!("{}••••••••{}", &secret[..6], &secret[secret.len() - 4..])
}

fn mask_stripe_key(key: &str) -> String {
    if key.is_empty() {
        return String::new();
    }
    if key.len() <= 12 {
        return "••••••••".to_string();
    }
    format!("{}••••••••{}", &key[..7], &key[key.len() - 4..])
}

/// `None` when there is no real secret to sign with: an unsigned delivery is
/// reported as unsigned rather than carrying a signature no one can verify.
fn compute_hmac_sha256(secret: &str, data: &[u8]) -> Option<String> {
    type HmacSha256 = Hmac<Sha256>;
    if secret.trim().is_empty() {
        return None;
    }
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).ok()?;
    mac.update(data);
    Some(format!("sha256={}", hex::encode(mac.finalize().into_bytes())))
}

/// A masked credential is a display artifact, not a secret. Never send one.
fn is_masked(value: &str) -> bool {
    value.contains('•') || value.contains("\u{2022}")
}

/// Char-boundary-safe truncation (byte slicing panics on multibyte bodies).
fn clip(input: &str, max_chars: usize) -> String {
    if input.chars().count() <= max_chars {
        return input.to_string();
    }
    let mut out: String = input.chars().take(max_chars).collect();
    out.push_str("…[truncated]");
    out
}

/// A body that cannot be read says so; it is never rendered as an empty body.
fn read_body(result: Result<String, reqwest::Error>) -> String {
    match result {
        Ok(text) => text,
        Err(err) => format!("<response body could not be read: {err}>"),
    }
}

/// The headers the endpoint actually sent, so a delivery log cannot claim a
/// content type that was never on the wire.
fn collect_response_headers(headers: &reqwest::header::HeaderMap) -> serde_json::Value {
    let mut map = serde_json::Map::new();
    for (name, value) in headers {
        let rendered = match value.to_str() {
            Ok(text) => text.to_string(),
            Err(_) => "<non-utf8 header value>".to_string(),
        };
        map.insert(name.as_str().to_string(), json!(rendered));
    }
    serde_json::Value::Object(map)
}

/// Rejects a webhook target that could never receive a delivery; no placeholder
/// URL is ever stored.
fn is_deliverable_http_url(raw: &str) -> bool {
    let Some(rest) = raw
        .strip_prefix("http://")
        .or_else(|| raw.strip_prefix("https://"))
    else {
        return false;
    };
    let host = match rest.split('/').next() {
        Some(host) => host,
        None => return false,
    };
    !host.is_empty() && !rest.chars().any(|c| c.is_whitespace() || c == '\u{0}')
}

/// A database failure is reported as a real 500, never as an empty 200.
fn db_error(context: &str, err: sqlx::Error) -> Response {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(json!({
            "error": format!("{context}: {err}"),
            "detail": format!("{err}"),
            "databaseError": true,
        })),
    )
        .into_response()
}

/// A write that succeeded but matched no row is stated, not reported as saved.
fn nothing_written(context: &str) -> Response {
    (
        StatusCode::NOT_FOUND,
        Json(json!({
            "success": false,
            "written": false,
            "error": format!("{context}: the statement affected 0 rows."),
        })),
    )
        .into_response()
}

// ---------------------------------------------------------------------------
// Live provider probes.
//
// Every outcome reported by the gateway test / ping endpoints below comes from
// an actual HTTP, TCP or UDP round-trip to the provider, or from an explicit
// `refusal` naming why no call was made (missing credential, unsupported
// provider). Nothing here sleeps and reports "connected".
// ---------------------------------------------------------------------------

const PROBE_TIMEOUT_SECS: u64 = 8;

#[derive(Debug)]
struct Probe {
    ok: bool,
    endpoint: String,
    status_code: Option<u16>,
    latency_ms: i64,
    detail: String,
    refusal: Option<String>,
    remote_id: Option<String>,
}

impl Probe {
    fn refused(endpoint: &str, reason: impl Into<String>) -> Probe {
        Probe {
            ok: false,
            endpoint: endpoint.to_string(),
            status_code: None,
            latency_ms: 0,
            detail: String::new(),
            refusal: Some(reason.into()),
            remote_id: None,
        }
    }

    fn live_call_made(&self) -> bool {
        self.refusal.is_none()
    }

    fn headline(&self) -> String {
        if let Some(ref refusal) = self.refusal {
            refusal.clone()
        } else if self.ok {
            self.detail.clone()
        } else if self.detail.is_empty() {
            format!(
                "Provider returned HTTP {}",
                self.status_code.unwrap_or(0)
            )
        } else {
            self.detail.clone()
        }
    }

    fn json(&self) -> serde_json::Value {
        json!({
            "ok": self.ok,
            "endpoint": self.endpoint,
            "http_status": self.status_code,
            "latency_ms": self.latency_ms,
            "detail": if self.detail.is_empty() { None } else { Some(self.detail.clone()) },
            "refusal": self.refusal,
            "live_call_made": self.live_call_made(),
            "remote_id": self.remote_id,
        })
    }
}

/// Read a credential that is safe to transmit: present, non-empty, unmasked.
fn usable_credential(creds: &serde_json::Value, cfg: &serde_json::Value, keys: &[&str]) -> Option<String> {
    for key in keys {
        for source in [creds, cfg] {
            if let Some(raw) = source.get(*key).and_then(|v| v.as_str()) {
                let trimmed = raw.trim();
                if !trimmed.is_empty() && !is_masked(trimmed) {
                    return Some(trimmed.to_string());
                }
            }
        }
    }
    None
}

/// True when the key exists but holds a masked display value.
fn masked_credential_present(creds: &serde_json::Value, cfg: &serde_json::Value, keys: &[&str]) -> bool {
    keys.iter().any(|key| {
        [creds, cfg].iter().any(|source| {
            source
                .get(*key)
                .and_then(|v| v.as_str())
                .map(|v| is_masked(v))
                .unwrap_or(false)
        })
    })
}

fn missing_credential(
    endpoint: &str,
    keys: &[&str],
    creds: &serde_json::Value,
    cfg: &serde_json::Value,
) -> Option<Probe> {
    let present = masked_credential_present(creds, cfg, keys);
    let names = keys.join(" / ");
    if present {
        return Some(Probe::refused(
            endpoint,
            format!(
                "Stored credential ({names}) is masked; it was saved for display only. Re-enter the real value and save it before testing."
            ),
        ));
    }
    Some(Probe::refused(
        endpoint,
        format!("No live call was made: credential field(s) {names} are not configured for this gateway."),
    ))
}

/// Pull the provider's own error text out of whichever envelope it uses.
fn provider_error_from_body(body: &serde_json::Value) -> Option<String> {
    let non_empty = |s: &str| (!s.trim().is_empty()).then(|| s.trim().to_string());

    if let Some(err) = body.get("error") {
        if let Some(msg) = err.get("message").and_then(|v| v.as_str()) {
            let code = err.get("code").and_then(|v| v.as_i64());
            return Some(match code {
                Some(code) => format!("provider error {code}: {}", non_empty(msg).unwrap_or_default()),
                None => non_empty(msg)?,
            });
        }
        if let Some(msg) = err.as_str() {
            return non_empty(msg);
        }
    }
    if let Some(list) = body.get("errors").and_then(|v| v.as_array()) {
        if let Some(first) = list.first() {
            if let Some(msg) = first.get("message").and_then(|v| v.as_str()) {
                return non_empty(msg);
            }
        }
    }
    for key in ["error_message", "errorMessage", "error_code", "fault_message", "message", "detail"] {
        if let Some(v) = body.get(key) {
            if let Some(s) = v.as_str() {
                if let Some(cleaned) = non_empty(s) {
                    // "OK"/"Success" style strings are not errors.
                    if !matches!(
                        cleaned.to_ascii_lowercase().as_str(),
                        "ok" | "success" | "successful" | "true" | "0"
                    ) {
                        return Some(cleaned);
                    }
                }
            }
        }
    }
    // Batched gateways (Vonage, Plivo, MessageBird) hide the failure inside arrays.
    if let Some(arr) = body
        .get("messages")
        .or_else(|| body.get("apiResults"))
        .or_else(|| body.get("data"))
        .and_then(|v| v.as_array())
    {
        if let Some(first) = arr.first() {
            let status = first
                .get("status")
                .and_then(|v| v.as_str())
                .unwrap_or("0");
            if status != "0" {
                let err = first
                    .get("error-text")
                    .or_else(|| first.get("errorText"))
                    .or_else(|| first.get("error_code"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("provider reported a non-zero status");
                return Some(format!("provider status {status}: {err}"));
            }
        }
    }
    if let Some(results) = body.get("results").and_then(|v| v.as_array()) {
        if let Some(first) = results.first() {
            let error = first.get("error").and_then(|v| v.as_str()).unwrap_or("");
            if !error.trim().is_empty() && !error.eq_ignore_ascii_case("none") {
                return Some(format!("provider error: {error}"));
            }
        }
    }
    None
}

fn find_remote_id(body: &serde_json::Value) -> Option<String> {
    for path in [
        body.get("sid"),
        body.get("id"),
        body.get("message_id"),
        body.get("messageId"),
        body.get("request_id"),
        body.get("messageUuid"),
    ] {
        if let Some(s) = path.and_then(|v| v.as_str()) {
            return Some(s.to_string());
        }
        if let Some(n) = path.and_then(|v| v.as_i64()) {
            return Some(n.to_string());
        }
    }
    if let Some(first) = body.get("messages").and_then(|v| v.as_array()).and_then(|a| a.first()) {
        if let Some(s) = first.get("message-id").and_then(|v| v.as_str()) {
            return Some(s.to_string());
        }
    }
    if let Some(list) = body.get("instances").and_then(|v| v.as_array()).and_then(|a| a.first()) {
        if let Some(s) = list.get("sid").and_then(|v| v.as_str()) {
            return Some(s.to_string());
        }
    }
    None
}

async fn run_http_body(
    builder: reqwest::RequestBuilder,
    endpoint: String,
) -> Probe {
    let builder = builder.timeout(std::time::Duration::from_secs(PROBE_TIMEOUT_SECS));
    let start = std::time::Instant::now();
    match builder.send().await {
        Err(err) => {
            let latency_ms = start.elapsed().as_millis() as i64;
            let kind = if err.is_timeout() {
                "timed out"
            } else if err.is_connect() {
                "could not connect"
            } else {
                "request failed"
            };
            Probe {
                ok: false,
                endpoint,
                status_code: None,
                latency_ms,
                detail: format!("Transport {kind} before any provider response: {err}"),
                refusal: None,
                remote_id: None,
            }
        }
        Ok(resp) => {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            let latency_ms = start.elapsed().as_millis() as i64;
            let parsed: Option<serde_json::Value> = serde_json::from_str(&text).ok();
            let hint = parsed.as_ref().and_then(provider_error_from_body);
            let remote_id = parsed.as_ref().and_then(find_remote_id);
            let ok = status.is_success() && hint.is_none();
            let detail = if let Some(h) = hint {
                h
            } else if status.is_success() {
                format!("HTTP {} in {latency_ms} ms", status.as_u16())
            } else {
                let excerpt = clip(text.trim(), 400);
                if excerpt.is_empty() {
                    format!("HTTP {} with an empty body", status.as_u16())
                } else {
                    excerpt
                }
            };
            Probe {
                ok,
                endpoint,
                status_code: Some(status.as_u16()),
                latency_ms,
                detail,
                refusal: None,
                remote_id,
            }
        }
    }
}

// -- AWS SigV4 (the crate set has hmac/sha2/hex but no AWS SDK) --------------

fn aws_percent_encode(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for byte in input.as_bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(*byte as char)
            }
            other => out.push_str(&format!("%{other:02X}")),
        }
    }
    out
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex::encode(hasher.finalize())
}

fn hmac_bytes(key: &[u8], data: &[u8]) -> Vec<u8> {
    type HmacSha256 = Hmac<Sha256>;
    match HmacSha256::new_from_slice(key) {
        Ok(mut mac) => {
            mac.update(data);
            mac.finalize().into_bytes().to_vec()
        }
        Err(_) => Vec::new(),
    }
}

fn form_body(mut pairs: Vec<(String, String)>) -> String {
    pairs.sort_by(|a, b| a.0.cmp(&b.0));
    pairs
        .iter()
        .map(|(k, v)| format!("{k}={}", aws_percent_encode(v)))
        .collect::<Vec<_>>()
        .join("&")
}

/// Signed query-protocol call against an AWS regional endpoint.
async fn aws_query(
    client: &reqwest::Client,
    service: &str,
    region: &str,
    access_key: &str,
    secret_key: &str,
    params: Vec<(String, String)>,
) -> Probe {
    let host = format!("{service}.{region}.amazonaws.com");
    let endpoint = format!("https://{host}/");
    let body = form_body(params);
    let payload_hash = sha256_hex(body.as_bytes());

    let now = Utc::now();
    let amz_date = now.format("%Y%m%dT%H%M%SZ").to_string();
    let short_date = now.format("%Y%m%d").to_string();
    let scope = format!("{short_date}/{region}/{service}/aws4_request");

    let canonical_request = format!(
        "POST\n/\n\nhost:{host}\nx-amz-date:{amz_date}\n\nhost;x-amz-date\n{payload_hash}"
    );
    let string_to_sign = format!(
        "AWS4-HMAC-SHA256\n{amz_date}\n{scope}\n{}",
        sha256_hex(canonical_request.as_bytes())
    );
    let k_date = hmac_bytes(format!("AWS4{secret_key}").as_bytes(), short_date.as_bytes());
    let k_region = hmac_bytes(&k_date, region.as_bytes());
    let k_service = hmac_bytes(&k_region, service.as_bytes());
    let k_signing = hmac_bytes(&k_service, b"aws4_request");
    let signature = hex::encode(hmac_bytes(&k_signing, string_to_sign.as_bytes()));
    let authorization = format!(
        "AWS4-HMAC-SHA256 Credential={access_key}/{scope}, SignedHeaders=host;x-amz-date, Signature={signature}"
    );

    let request = client
        .post(&endpoint)
        .header("host", &host)
        .header("x-amz-date", &amz_date)
        .header("authorization", authorization)
        .header("content-type", "application/x-www-form-urlencoded; charset=utf-8")
        .body(body);
    run_http_body(request, endpoint).await
}

// -- SMTP banner probe (no SMTP client crate is available) -------------------

async fn smtp_greeting_probe(host: &str, port: u16, secure: bool) -> Probe {
    let endpoint = format!("tcp://{host}:{port}");
    let timeout = std::time::Duration::from_secs(PROBE_TIMEOUT_SECS);
    let attempt = tokio::time::timeout(timeout, tokio::net::TcpStream::connect((host, port))).await;
    let mut socket = match attempt {
        Err(_) => {
            return Probe {
                ok: false,
                endpoint,
                status_code: None,
                latency_ms: timeout.as_millis() as i64,
                detail: format!("TCP connect to {host}:{port} timed out after {PROBE_TIMEOUT_SECS}s."),
                refusal: None,
                remote_id: None,
            }
        }
        Ok(Err(err)) => {
            return Probe {
                ok: false,
                endpoint,
                status_code: None,
                latency_ms: 0,
                detail: format!("TCP connect to {host}:{port} failed: {err}"),
                refusal: None,
                remote_id: None,
            }
        }
        Ok(Ok(socket)) => socket,
    };

    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    let mut buf = [0u8; 512];
    let read_res = tokio::time::timeout(timeout, socket.read(&mut buf)).await;
    let greeting = match read_res {
        Err(_) => {
            return Probe {
                ok: false,
                endpoint,
                status_code: None,
                latency_ms: timeout.as_millis() as i64,
                detail: format!("Connected to {host}:{port} but the server never sent a banner."),
                refusal: None,
                remote_id: None,
            }
        }
        Ok(Ok(n)) => String::from_utf8_lossy(&buf[..n]).trim().to_string(),
        Ok(Err(err)) => {
            return Probe {
                ok: false,
                endpoint,
                status_code: None,
                latency_ms: 0,
                detail: format!("Reading the banner failed: {err}"),
                refusal: None,
                remote_id: None,
            }
        }
    };
    let quit_note = match tokio::time::timeout(
        std::time::Duration::from_secs(2),
        socket.write_all(b"QUIT\r\n"),
    )
    .await
    {
        Ok(Ok(())) => String::new(),
        Ok(Err(err)) => format!(" The courtesy QUIT close failed: {err}."),
        Err(_) => " The courtesy QUIT close timed out.".to_string(),
    };

    let banner_ok = greeting.starts_with("220");
    let note = if secure {
        "Implicit-TLS port: banner only."
    } else {
        "Plaintext port: banner only."
    };
    Probe {
        ok: banner_ok,
        endpoint,
        status_code: None,
        latency_ms: 0,
        detail: if banner_ok {
            format!(
                "SMTP server answered \"{greeting}\". {note}{quit_note} Username/password were NOT exercised — STARTTLS + AUTH LOGIN needs an SMTP client crate (lettre + rustls), which is not in Cargo.toml."
            )
        } else {
            format!("Unexpected SMTP banner: \"{greeting}\"{quit_note}")
        },
        refusal: None,
        remote_id: None,
    }
}

// -- SIP OPTIONS probe over UDP ---------------------------------------------

async fn sip_options_probe(server: &str, port: u16, transport: &str) -> Probe {
    let endpoint = format!("sip://{server}:{port}/{transport}");
    if transport.eq_ignore_ascii_case("tcp") || transport.eq_ignore_ascii_case("tls") {
        let attempt = tokio::time::timeout(
            std::time::Duration::from_secs(PROBE_TIMEOUT_SECS),
            tokio::net::TcpStream::connect((server, port)),
        )
        .await;
        return match attempt {
            Err(_) => Probe {
                ok: false,
                endpoint,
                status_code: None,
                latency_ms: PROBE_TIMEOUT_SECS as i64 * 1000,
                detail: format!("TCP connect to {server}:{port} timed out."),
                refusal: None,
                remote_id: None,
            },
            Ok(Err(err)) => Probe {
                ok: false,
                endpoint,
                status_code: None,
                latency_ms: 0,
                detail: format!("TCP connect to {server}:{port} failed: {err}"),
                refusal: None,
                remote_id: None,
            },
            Ok(Ok(_)) => Probe {
                ok: true,
                endpoint,
                status_code: None,
                latency_ms: 0,
                detail: format!("TCP port {server}:{port} is open; no SIP transaction was sent on {transport}."),
                refusal: None,
                remote_id: None,
            },
        };
    }

    let socket = match tokio::net::UdpSocket::bind("0.0.0.0:0").await {
        Ok(socket) => socket,
        Err(err) => {
            return Probe::refused(&endpoint, format!("Could not bind a UDP socket for the SIP probe: {err}"))
        }
    };
    if let Err(err) = socket.connect((server, port)).await {
        return Probe {
            ok: false,
            endpoint,
            status_code: None,
            latency_ms: 0,
            detail: format!("UDP connect to {server}:{port} failed: {err}"),
            refusal: None,
            remote_id: None,
        };
    }

    let call_id = Uuid::new_v4().simple().to_string();
    let branch = format!("z9hG4bK-{}", &call_id[..16]);
    let request = format!(
        "OPTIONS sip:{server}:{port} SIP/2.0\r\nVia: SIP/2.0/UDP communicator.local;branch={branch};rport\r\nMax-Forwards: 0\r\nFrom: \"probe\" <sip:probe@communicator.local>;tag=probe\r\nTo: <sip:{server}>\r\nCall-ID: {call_id}@communicator.local\r\nCSeq: 1 OPTIONS\r\nContact: <sip:probe@communicator.local>\r\nUser-Agent: Communicator-Gateway-Probe\r\nContent-Length: 0\r\n\r\n"
    );
    if let Err(err) = socket.send(request.as_bytes()).await {
        return Probe {
            ok: false,
            endpoint,
            status_code: None,
            latency_ms: 0,
            detail: format!("Sending the SIP OPTIONS request failed: {err}"),
            refusal: None,
            remote_id: None,
        };
    }

    let start = std::time::Instant::now();
    let mut buf = [0u8; 2048];
    loop {
        let remaining = PROBE_TIMEOUT_SECS.saturating_sub(start.elapsed().as_secs());
        if remaining == 0 {
            return Probe {
                ok: false,
                endpoint,
                status_code: None,
                latency_ms: start.elapsed().as_millis() as i64,
                detail: format!("No SIP response from {server}:{port} within {PROBE_TIMEOUT_SECS}s (host unreachable, or the trunk ACL rejects our source IP)."),
                refusal: None,
                remote_id: None,
            };
        }
        match tokio::time::timeout(std::time::Duration::from_secs(remaining), socket.recv(&mut buf)).await {
            Err(_) => {
                return Probe {
                    ok: false,
                    endpoint,
                    status_code: None,
                    latency_ms: start.elapsed().as_millis() as i64,
                    detail: format!("No SIP response from {server}:{port} within {PROBE_TIMEOUT_SECS}s (host unreachable, or the trunk ACL rejects our source IP)."),
                    refusal: None,
                    remote_id: None,
                }
            }
            Ok(Ok(n)) => {
                let text = String::from_utf8_lossy(&buf[..n]).to_string();
                let first_line = text.lines().next().unwrap_or_default().to_string();
                let code: u16 = first_line
                    .split_whitespace()
                    .nth(1)
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(0);
                if (100..199).contains(&code) {
                    continue; // provisional; wait for the final response
                }
                return Probe {
                    ok: (200..300).contains(&code),
                    endpoint,
                    status_code: Some(code),
                    latency_ms: start.elapsed().as_millis() as i64,
                    detail: if code == 0 {
                        format!("Malformed SIP reply: \"{first_line}\"")
                    } else {
                        first_line
                    },
                    refusal: None,
                    remote_id: None,
                };
            }
            Ok(Err(err)) => {
                return Probe {
                    ok: false,
                    endpoint,
                    status_code: None,
                    latency_ms: start.elapsed().as_millis() as i64,
                    detail: format!("UDP receive failed: {err}"),
                    refusal: None,
                    remote_id: None,
                }
            }
        }
    }
}

/// Real credential verification against the configured provider.
async fn probe_gateway_connection(
    state: &AppState,
    provider: &str,
    creds: &serde_json::Value,
    cfg: &serde_json::Value,
) -> Probe {
    let client = &state.http_client;
    let p = provider.trim().to_ascii_lowercase();
    match p.as_str() {
        "twilio" | "twilio_voice" => {
            let sid_keys = ["account_sid", "sid"];
            let tok_keys = ["auth_token", "api_key"];
            let (Some(sid), Some(token)) = (
                usable_credential(creds, cfg, &sid_keys),
                usable_credential(creds, cfg, &tok_keys),
            ) else {
                return missing_credential(
                    "https://api.twilio.com",
                    &["account_sid", "auth_token"],
                    creds,
                    cfg,
                )
                .unwrap();
            };
            let url = format!("https://api.twilio.com/2010-04-01/Accounts/{sid}.json");
            run_http_body(
                client.get(&url).basic_auth(&sid, Some(&token)).header(
                    "Accept",
                    "application/json",
                ),
                url.clone(),
            )
            .await
        }
        "vonage" | "vonage_voice" => {
            let (Some(key), Some(secret)) = (
                usable_credential(creds, cfg, &["api_key"]),
                usable_credential(creds, cfg, &["api_secret"]),
            ) else {
                return missing_credential("https://rest.nexmo.com", &["api_key", "api_secret"], creds, cfg)
                    .unwrap();
            };
            let url = "https://rest.nexmo.com/account/get-balance".to_string();
            run_http_body(
                client.get(&url).query(&[("api_key", &key), ("api_secret", &secret)]),
                url.clone(),
            )
            .await
        }
        "messagebird" => {
            let Some(access_key) = usable_credential(creds, cfg, &["access_key"]) else {
                return missing_credential("https://access.messagebird.com", &["access_key"], creds, cfg)
                    .unwrap();
            };
            let url = "https://access.messagebird.com/access_keys".to_string();
            run_http_body(
                client
                    .get(&url)
                    .header("Authorization", format!("Bearer {access_key}")),
                url.clone(),
            )
            .await
        }
        "plivo" | "plivo_voice" => {
            let (Some(auth_id), Some(auth_token)) = (
                usable_credential(creds, cfg, &["auth_id"]),
                usable_credential(creds, cfg, &["auth_token"]),
            ) else {
                return missing_credential("https://api.plivo.com", &["auth_id", "auth_token"], creds, cfg)
                    .unwrap();
            };
            let url = format!("https://api.plivo.com/v1/Account/{auth_id}/");
            run_http_body(
                client.get(&url).basic_auth(&auth_id, Some(&auth_token)),
                url.clone(),
            )
            .await
        }
        "sinch" => {
            let Some(token) = usable_credential(creds, cfg, &["api_token"]) else {
                return missing_credential("https://dashboard.sinch.com", &["api_token"], creds, cfg)
                    .unwrap();
            };
            let url = "https://dashboard.sinch.com/sinch-account/api/v2/service-plans".to_string();
            run_http_body(
                client.get(&url).header("Authorization", format!("Bearer {token}")),
                url.clone(),
            )
            .await
        }
        "infobip" => {
            let Some(api_key) = usable_credential(creds, cfg, &["api_key"]) else {
                return missing_credential("https://api.infobip.com", &["api_key"], creds, cfg).unwrap();
            };
            let base = usable_credential(creds, cfg, &["base_url"])
                .unwrap_or_else(|| "https://api.infobip.com".to_string())
                .trim_end_matches('/')
                .to_string();
            let url = format!("{base}/account/1/status");
            run_http_body(
                client
                    .get(&url)
                    .header("Authorization", format!("AccessKey {api_key}")),
                url.clone(),
            )
            .await
        }
        "telnyx" => {
            let Some(api_key) = usable_credential(creds, cfg, &["api_key"]) else {
                return missing_credential("https://api.telnyx.com", &["api_key"], creds, cfg).unwrap();
            };
            let url = "https://api.telnyx.com/v2/api_key_information".to_string();
            run_http_body(
                client.get(&url).header("Authorization", format!("Bearer {api_key}")),
                url.clone(),
            )
            .await
        }
        "sendgrid" => {
            let Some(api_key) = usable_credential(creds, cfg, &["api_key"]) else {
                return missing_credential("https://api.sendgrid.com", &["api_key"], creds, cfg).unwrap();
            };
            let url = "https://api.sendgrid.com/v3/scopes".to_string();
            run_http_body(
                client.get(&url).header("Authorization", format!("Bearer {api_key}")),
                url.clone(),
            )
            .await
        }
        "mailgun" => {
            let (Some(api_key), Some(domain)) = (
                usable_credential(creds, cfg, &["api_key"]),
                usable_credential(creds, cfg, &["domain"]),
            ) else {
                return missing_credential("https://api.mailgun.net", &["api_key", "domain"], creds, cfg)
                    .unwrap();
            };
            let url = format!("https://api.mailgun.net/v3/domains/{domain}");
            run_http_body(client.get(&url).basic_auth("api", Some(&api_key)), url.clone())
                .await
        }
        "postmark" => {
            let Some(api_key) = usable_credential(creds, cfg, &["api_key", "server_token"]) else {
                return missing_credential("https://api.postmarkapp.com", &["api_key"], creds, cfg).unwrap();
            };
            let url = "https://api.postmarkapp.com/deliverability/outbound/overview".to_string();
            run_http_body(
                client
                    .get(&url)
                    .header("Accept", "application/json")
                    .header("X-Postmark-Server-Token", api_key),
                url.clone(),
            )
            .await
        }
        "resend" => {
            let Some(api_key) = usable_credential(creds, cfg, &["api_key"]) else {
                return missing_credential("https://api.resend.com", &["api_key"], creds, cfg).unwrap();
            };
            let url = "https://api.resend.com/domains".to_string();
            run_http_body(
                client.get(&url).header("Authorization", format!("Bearer {api_key}")),
                url.clone(),
            )
            .await
        }
        "aws_sns" | "aws_ses" => {
            let (Some(access_key), Some(secret_key)) = (
                usable_credential(creds, cfg, &["aws_access_key", "access_key_id"]),
                usable_credential(creds, cfg, &["aws_secret_key", "secret_access_key"]),
            ) else {
                return missing_credential(
                    "https://<service>.<region>.amazonaws.com",
                    &["aws_access_key", "aws_secret_key"],
                    creds,
                    cfg,
                )
                .unwrap();
            };
            let Some(region) = usable_credential(creds, cfg, &["region"]) else {
                return missing_credential("https://<service>.<region>.amazonaws.com", &["region"], creds, cfg)
                    .unwrap();
            };
            let (service, action) = if p == "aws_sns" {
                ("sns", "GetSMSAccountAttributes")
            } else {
                ("email", "ListIdentities")
            };
            let mut params: Vec<(String, String)> = vec![("Action".into(), action.into())];
            if service == "email" {
                params.push(("MaxItems".into(), "1".into()));
            }
            params.push((
                "Version".into(),
                if service == "sns" {
                    "2010-03-31".into()
                } else {
                    "2010-12-01".into()
                },
            ));
            aws_query(client, service, &region, &access_key, &secret_key, params).await
        }
        "smtp" => {
            let Some(host) = usable_credential(creds, cfg, &["smtp_host", "host"]) else {
                return missing_credential("smtp://host", &["smtp_host"], creds, cfg).unwrap();
            };
            let port: u16 = creds
                .get("smtp_port")
                .or_else(|| cfg.get("smtp_port"))
                .and_then(|v| v.as_i64().map(|i| i as u16).or_else(|| v.as_str().and_then(|s| s.parse().ok())))
                .unwrap_or(587);
            let secure = port == 465;
            smtp_greeting_probe(&host, port, secure).await
        }
        "sip_trunk" => {
            let Some(host) = usable_credential(creds, cfg, &["sip_server", "sip_host", "host"]) else {
                return missing_credential("sip://host", &["sip_server"], creds, cfg).unwrap();
            };
            let port: u16 = creds
                .get("sip_port")
                .or_else(|| cfg.get("sip_port"))
                .and_then(|v| v.as_i64().map(|i| i as u16).or_else(|| v.as_str().and_then(|s| s.parse().ok())))
                .unwrap_or(5060);
            let transport = usable_credential(creds, cfg, &["transport"]).unwrap_or("udp".into());
            sip_options_probe(&host, port, &transport).await
        }
        "custom_http" | "custom_webhook" | "custom_webrtc" => {
            let url_keys = ["endpoint_url", "webhook_url", "signaling_url", "url"];
            let Some(url) = usable_credential(creds, cfg, &url_keys) else {
                return missing_credential(
                    "<configured endpoint>",
                    &url_keys,
                    creds,
                    cfg,
                )
                .unwrap();
            };
            let mut builder = client.get(&url);
            if let Some(token) = usable_credential(creds, cfg, &["bearer_token", "auth_token", "auth_header"]) {
                let header_value = if token.contains(": ") {
                    token.split_once(": ").map(|(_, v)| v.to_string()).unwrap_or(token.clone())
                } else if token.to_ascii_lowercase().starts_with("bearer ") {
                    token.clone()
                } else {
                    format!("Bearer {token}")
                };
                builder = builder.header("Authorization", header_value);
            }
            run_http_body(builder, url.clone()).await
        }
        other => Probe::refused(
            "",
            format!(
                "No live probe is implemented for provider '{other}', so its credentials cannot be verified. Refusing to report an unverified connection."
            ),
        ),
    }
}

/// Sends a real test transmission through the gateway.
async fn dispatch_gateway_message(
    state: &AppState,
    provider: &str,
    gateway_type: &str,
    creds: &serde_json::Value,
    cfg: &serde_json::Value,
    target: &str,
    message: &str,
) -> Probe {
    let client = &state.http_client;
    let p = provider.trim().to_ascii_lowercase();
    let gt = gateway_type.trim().to_ascii_lowercase();

    match p.as_str() {
        "twilio" => {
            let (Some(sid), Some(token), Some(from)) = (
                usable_credential(creds, cfg, &["account_sid"]),
                usable_credential(creds, cfg, &["auth_token"]),
                usable_credential(creds, cfg, &["from_number", "from"]),
            ) else {
                return missing_credential(
                    "https://api.twilio.com",
                    &["account_sid", "auth_token", "from_number"],
                    creds,
                    cfg,
                )
                .unwrap();
            };
            let url = format!("https://api.twilio.com/2010-04-01/Accounts/{sid}/Messages.json");
            let mut form: Vec<(String, String)> =
                vec![("To".into(), target.into()), ("Body".into(), message.into())];
            if let Some(service_sid) = usable_credential(creds, cfg, &["messaging_service_sid"]) {
                form.push(("MessagingServiceSid".into(), service_sid));
            } else {
                form.push(("From".into(), from));
            }
            run_http_body(
                client
                    .post(&url)
                    .basic_auth(&sid, Some(&token))
                    .form(
                        &form
                            .iter()
                            .map(|(k, v)| (k.as_str(), v.as_str()))
                            .collect::<Vec<(&str, &str)>>(),
                    ),
                url,
            )
            .await
        }
        "vonage" => {
            let (Some(key), Some(secret), Some(from)) = (
                usable_credential(creds, cfg, &["api_key"]),
                usable_credential(creds, cfg, &["api_secret"]),
                usable_credential(creds, cfg, &["from", "from_number"]),
            ) else {
                return missing_credential(
                    "https://rest.nexmo.com/sms/json",
                    &["api_key", "api_secret", "from"],
                    creds,
                    cfg,
                )
                .unwrap();
            };
            let url = "https://rest.nexmo.com/sms/json".to_string();
            run_http_body(
                client.post(&url).form(&[
                    ("api_key", key.as_str()),
                    ("api_secret", secret.as_str()),
                    ("to", target),
                    ("from", from.as_str()),
                    ("text", message),
                ]),
                url,
            )
            .await
        }
        "messagebird" => {
            let (Some(access_key), Some(originator)) = (
                usable_credential(creds, cfg, &["access_key"]),
                usable_credential(creds, cfg, &["originator", "from_number"]),
            ) else {
                return missing_credential(
                    "https://messages.messagebird.com/v1/messages",
                    &["access_key", "originator"],
                    creds,
                    cfg,
                )
                .unwrap();
            };
            let url = "https://messages.messagebird.com/v1/messages".to_string();
            run_http_body(
                client
                    .post(&url)
                    .header("Authorization", format!("Bearer {access_key}"))
                    .json(&json!({ "originator": originator, "body": message, "recipients": [target] })),
                url,
            )
            .await
        }
        "plivo" => {
            let (Some(auth_id), Some(auth_token), Some(from)) = (
                usable_credential(creds, cfg, &["auth_id"]),
                usable_credential(creds, cfg, &["auth_token"]),
                usable_credential(creds, cfg, &["from_number", "from"]),
            ) else {
                return missing_credential(
                    "https://api.plivo.com",
                    &["auth_id", "auth_token", "from_number"],
                    creds,
                    cfg,
                )
                .unwrap();
            };
            let url = format!("https://api.plivo.com/v1/Account/{auth_id}/Message/");
            run_http_body(
                client
                    .post(&url)
                    .basic_auth(&auth_id, Some(&auth_token))
                    .json(&json!({
                        "src": from, "dst": [target], "text": message, "type": "sms"
                    })),
                url,
            )
            .await
        }
        "sinch" => {
            let (Some(plan), Some(token), Some(from)) = (
                usable_credential(creds, cfg, &["service_plan_id"]),
                usable_credential(creds, cfg, &["api_token"]),
                usable_credential(creds, cfg, &["from_number", "from"]),
            ) else {
                return missing_credential(
                    "https://portal.vilter.sinch.com",
                    &["service_plan_id", "api_token", "from_number"],
                    creds,
                    cfg,
                )
                .unwrap();
            };
            let url = format!("https://portal.vilter.sinch.com/xms/v1/{plan}/batches");
            run_http_body(
                client
                    .post(&url)
                    .header("Authorization", format!("Bearer {token}"))
                    .json(&json!({ "from": from, "to": [target], "body": message })),
                url,
            )
            .await
        }
        "infobip" => {
            let (Some(api_key), Some(from)) = (
                usable_credential(creds, cfg, &["api_key"]),
                usable_credential(creds, cfg, &["from_number", "from"]),
            ) else {
                return missing_credential(
                    "https://api.infobip.com",
                    &["api_key", "from_number"],
                    creds,
                    cfg,
                )
                .unwrap();
            };
            let base = usable_credential(creds, cfg, &["base_url"])
                .unwrap_or_else(|| "https://api.infobip.com".to_string())
                .trim_end_matches('/')
                .to_string();
            let url = format!("{base}/messaging/2/messages?dest={target}");
            run_http_body(
                client
                    .post(&url)
                    .header("Authorization", format!("AccessKey {api_key}"))
                    .json(&json!({ "messages": [{ "from": from, "text": message }] })),
                url,
            )
            .await
        }
        "telnyx" => {
            let (Some(api_key), Some(from)) = (
                usable_credential(creds, cfg, &["api_key"]),
                usable_credential(creds, cfg, &["from_number", "caller_id"]),
            ) else {
                return missing_credential("https://api.telnyx.com", &["api_key", "from_number"], creds, cfg)
                    .unwrap();
            };
            let url = "https://api.telnyx.com/v2/messages".to_string();
            run_http_body(
                client
                    .post(&url)
                    .header("Authorization", format!("Bearer {api_key}"))
                    .json(&json!({ "from": from, "to": [target], "text": message })),
                url,
            )
            .await
        }
        "aws_sns" => {
            let (Some(access_key), Some(secret_key), Some(region)) = (
                usable_credential(creds, cfg, &["aws_access_key", "access_key_id"]),
                usable_credential(creds, cfg, &["aws_secret_key", "secret_access_key"]),
                usable_credential(creds, cfg, &["region"]),
            ) else {
                return missing_credential(
                    "https://sns.<region>.amazonaws.com",
                    &["aws_access_key", "aws_secret_key", "region"],
                    creds,
                    cfg,
                )
                .unwrap();
            };
            aws_query(
                client,
                "sns",
                &region,
                &access_key,
                &secret_key,
                vec![
                    ("Action".into(), "Publish".into()),
                    ("PhoneNumber".into(), target.into()),
                    ("Message".into(), message.into()),
                    ("Version".into(), "2010-03-31".into()),
                ],
            )
            .await
        }
        "sendgrid" => {
            let (Some(api_key), Some(from)) = (
                usable_credential(creds, cfg, &["api_key"]),
                usable_credential(creds, cfg, &["from_address", "from"]),
            ) else {
                return missing_credential(
                    "https://api.sendgrid.com/v3/mail/send",
                    &["api_key", "from_address"],
                    creds,
                    cfg,
                )
                .unwrap();
            };
            let url = "https://api.sendgrid.com/v3/mail/send".to_string();
            let from_name = usable_credential(creds, cfg, &["from_name"]);
            run_http_body(
                client
                    .post(&url)
                    .header("Authorization", format!("Bearer {api_key}"))
                    .json(&json!({
                        "personalizations": [{ "to": [{ "email": target }] }],
                        "from": if let Some(n) = from_name { json!({"email": from, "name": n}) } else { json!({"email": from}) },
                        "subject": "Communicator gateway test",
                        "content": [{ "type": "text/plain", "value": message }]
                    })),
                url,
            )
            .await
        }
        "mailgun" => {
            let (Some(api_key), Some(domain), Some(from)) = (
                usable_credential(creds, cfg, &["api_key"]),
                usable_credential(creds, cfg, &["domain"]),
                usable_credential(creds, cfg, &["from_address", "from"]),
            ) else {
                return missing_credential(
                    "https://api.mailgun.net/v3/<domain>/messages",
                    &["api_key", "domain", "from_address"],
                    creds,
                    cfg,
                )
                .unwrap();
            };
            let url = format!("https://api.mailgun.net/v3/{domain}/messages");
            run_http_body(
                client
                    .post(&url)
                    .basic_auth("api", Some(&api_key))
                    .form(&[("from", from.as_str()), ("to", target), ("subject", "Communicator gateway test"), ("text", message)]),
                url,
            )
            .await
        }
        "postmark" => {
            let (Some(api_key), Some(from)) = (
                usable_credential(creds, cfg, &["api_key"]),
                usable_credential(creds, cfg, &["from_address", "from"]),
            ) else {
                return missing_credential(
                    "https://api.postmarkapp.com/email",
                    &["api_key", "from_address"],
                    creds,
                    cfg,
                )
                .unwrap();
            };
            let url = "https://api.postmarkapp.com/email".to_string();
            run_http_body(
                client
                    .post(&url)
                    .header("Accept", "application/json")
                    .header("Content-Type", "application/json")
                    .header("X-Postmark-Server-Token", api_key)
                    .json(&json!({
                        "From": from, "To": target,
                        "Subject": "Communicator gateway test",
                        "TextBody": message
                    })),
                url,
            )
            .await
        }
        "resend" => {
            let (Some(api_key), Some(from)) = (
                usable_credential(creds, cfg, &["api_key"]),
                usable_credential(creds, cfg, &["from_address", "from"]),
            ) else {
                return missing_credential(
                    "https://api.resend.com/emails",
                    &["api_key", "from_address"],
                    creds,
                    cfg,
                )
                .unwrap();
            };
            let url = "https://api.resend.com/emails".to_string();
            run_http_body(
                client
                    .post(&url)
                    .header("Authorization", format!("Bearer {api_key}"))
                    .json(&json!({
                        "from": from, "to": [target],
                        "subject": "Communicator gateway test",
                        "text": message
                    })),
                url,
            )
            .await
        }
        "aws_ses" => {
            let (Some(access_key), Some(secret_key), Some(region), Some(from)) = (
                usable_credential(creds, cfg, &["aws_access_key", "access_key_id"]),
                usable_credential(creds, cfg, &["aws_secret_key", "secret_access_key"]),
                usable_credential(creds, cfg, &["region"]),
                usable_credential(creds, cfg, &["from_address", "from"]),
            ) else {
                return missing_credential(
                    "https://email.<region>.amazonaws.com",
                    &["aws_access_key", "aws_secret_key", "region", "from_address"],
                    creds,
                    cfg,
                )
                .unwrap();
            };
            use base64::Engine as _;
            let raw = format!(
                "From: {from}\r\nTo: {target}\r\nSubject: Communicator gateway test\r\n\r\n{message}\r\n"
            );
            let encoded = base64::engine::general_purpose::STANDARD.encode(raw.as_bytes());
            aws_query(
                client,
                "email",
                &region,
                &access_key,
                &secret_key,
                vec![
                    ("Action".into(), "SendRawEmail".into()),
                    ("Destinations.member.1".into(), target.into()),
                    ("Source".into(), from),
                    ("RawMessage".into(), encoded),
                    ("Version".into(), "2010-12-01".into()),
                ],
            )
            .await
        }
        "custom_http" | "custom_webhook" => {
            let url_keys = ["endpoint_url", "webhook_url", "url"];
            let Some(url) = usable_credential(creds, cfg, &url_keys) else {
                return missing_credential("<configured endpoint>", &url_keys, creds, cfg).unwrap();
            };
            // Substitution happens inside the parsed JSON tree, never by
            // rewriting the serialized text: a message containing a quote or
            // brace would otherwise produce invalid JSON and be silently
            // replaced by a different payload than the one configured.
            let configured = cfg
                .get("payload_template")
                .or_else(|| creds.get("payload_template"))
                .cloned();
            let (template_note, body) = match configured {
                Some(template) => (
                    format!(
                        " Payload came from the configured payload_template ({}).",
                        describe_template(&template)
                    ),
                    render_template_value(&template, target, message),
                ),
                None => (
                    " No payload_template is configured for this gateway, so the built-in {to, message, type} shape was sent.".to_string(),
                    json!({ "to": target, "message": message, "type": gt }),
                ),
            };
            let mut builder = client.post(&url).json(&body);
            if let Some(token) = usable_credential(creds, cfg, &["bearer_token", "auth_token", "auth_header"]) {
                let value = if token.to_ascii_lowercase().starts_with("bearer ") {
                    token
                } else {
                    format!("Bearer {token}")
                };
                builder = builder.header("Authorization", value);
            }
            let mut probe = run_http_body(builder, url.clone()).await;
            probe.detail.push_str(&template_note);
            probe
        }
        other => Probe::refused(
            "",
            format!(
                "No live transmission adapter is ported for provider '{other}', so nothing was sent to {target}. Refusing to report a delivered message."
            ),
        ),
    }
}

/// Replace `{{target}}` / `{{message}}` inside string leaves of a configured
/// payload template, leaving the JSON structure intact.
fn render_template_value(template: &serde_json::Value, target: &str, message: &str) -> serde_json::Value {
    match template {
        serde_json::Value::String(text) => {
            let mut out = String::with_capacity(text.len());
            let mut rest = text.as_str();
            while let Some(start) = rest.find("{{") {
                out.push_str(&rest[..start]);
                match rest[start..].find("}}") {
                    Some(offset) => {
                        match &rest[start + 2..start + offset] {
                            "target" => out.push_str(target),
                            "message" => out.push_str(message),
                            other => {
                                out.push_str("{{");
                                out.push_str(other);
                                out.push_str("}}");
                            }
                        }
                        rest = &rest[start + offset + 2..];
                    }
                    None => {
                        out.push_str(&rest[start..]);
                        rest = "";
                    }
                }
            }
            out.push_str(rest);
            serde_json::Value::String(out)
        }
        serde_json::Value::Array(items) => serde_json::Value::Array(
            items
                .iter()
                .map(|item| render_template_value(item, target, message))
                .collect(),
        ),
        serde_json::Value::Object(map) => serde_json::Value::Object(
            map.iter()
                .map(|(key, value)| (key.clone(), render_template_value(value, target, message)))
                .collect(),
        ),
        other => other.clone(),
    }
}

/// Identifies the configured template well enough for a response to name the
/// shape it actually sent.
fn describe_template(template: &serde_json::Value) -> String {
    match template {
        serde_json::Value::Object(map) => format!("object with keys [{}]", map.keys().cloned().collect::<Vec<_>>().join(", ")),
        serde_json::Value::Array(items) => format!("array of {} items", items.len()),
        other => format!("non-object ({other})"),
    }
}

fn parse_json_object(raw: &str) -> serde_json::Value {
    serde_json::from_str(raw).unwrap_or_else(|_| json!({}))
}

/// Choose the stored gateway connection that should carry traffic for a channel
/// (`whatsapp`, `sms`, `email`, `voice`, ...). Preference order is real data:
/// an active connection whose `gateway_type` or `provider` names the channel,
/// then the tenant default, then any active row. `None` means nothing is
/// configured -- callers must report that instead of substituting a gateway.
pub(crate) async fn gateway_row_for_channel(
    state: &AppState,
    channel: &str,
) -> Option<GatewayDbRow> {
    let channel = channel.trim().to_ascii_lowercase();
    let preferred = if channel == "whatsapp" { "meta" } else { channel.as_str() };

    let by_channel = sqlx::query_as::<_, GatewayDbRow>(
        "SELECT id, gateway_type, provider, name, is_default, is_active, credentials, config, test_status, latency_ms, last_tested_at, created_at, updated_at \
         FROM channel_gateway_connections WHERE is_active = 1 AND (LOWER(gateway_type) = $1 OR LOWER(provider) = $2) \
         ORDER BY is_default DESC, updated_at DESC LIMIT 1",
    )
    .bind(&channel)
    .bind(preferred)
    .fetch_optional(&state.db)
    .await
    .ok()
    .flatten();

    if by_channel.is_some() {
        return by_channel;
    }

    sqlx::query_as::<_, GatewayDbRow>(
        "SELECT id, gateway_type, provider, name, is_default, is_active, credentials, config, test_status, latency_ms, last_tested_at, created_at, updated_at \
         FROM channel_gateway_connections WHERE is_active = 1 ORDER BY is_default DESC, updated_at DESC LIMIT 1",
    )
    .fetch_optional(&state.db)
    .await
    .ok()
    .flatten()
}

/// Send one text through a stored connection for real. Returns the probe as
/// JSON (`ok`, `detail`, `refusal`, `remote_id`, `http_status`, `latency_ms`,
/// `live_call_made`) so a scheduler can report the provider's answer verbatim
/// rather than claiming a delivery that never happened.
pub(crate) async fn send_text_via_gateway_row(
    state: &AppState,
    row: &GatewayDbRow,
    target: &str,
    message: &str,
) -> serde_json::Value {
    let creds = parse_json_object(&row.credentials);
    let cfg = parse_json_object(&row.config);
    dispatch_gateway_message(
        state,
        &row.provider,
        &row.gateway_type,
        &creds,
        &cfg,
        target,
        message,
    )
    .await
    .json()
}

// ---------------------------------------------------------------------------
// Database Models
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, sqlx::FromRow)]
pub struct GatewayDbRow {
    pub id: String,
    pub gateway_type: String,
    pub provider: String,
    pub name: String,
    pub is_default: bool,
    pub is_active: bool,
    pub credentials: String,
    pub config: String,
    pub test_status: String,
    pub latency_ms: i64,
    pub last_tested_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// A stored JSON blob that cannot be parsed is reported in `warnings` instead
/// of being displayed as an empty object that would read as "nothing set".
fn stored_json(raw: &str, label: &str, warnings: &mut Vec<String>) -> serde_json::Value {
    match serde_json::from_str::<serde_json::Value>(raw) {
        Ok(value) => value,
        Err(err) => {
            warnings.push(format!(
                "The stored {label} column is not valid JSON ({err}); it is displayed as empty."
            ));
            json!({})
        }
    }
}

fn format_gateway(row: GatewayDbRow) -> serde_json::Value {
    let mut warnings: Vec<String> = Vec::new();
    let creds_val = stored_json(&row.credentials, "credentials", &mut warnings);
    let mut masked_creds = serde_json::Map::new();
    if let Some(obj) = creds_val.as_object() {
        for (k, v) in obj {
            if let Some(s) = v.as_str() {
                masked_creds.insert(k.clone(), json!(mask_secret(s)));
            } else {
                masked_creds.insert(k.clone(), v.clone());
            }
        }
    }
    let config_val = stored_json(&row.config, "config", &mut warnings);

    json!({
        "id": row.id,
        "gateway_type": row.gateway_type,
        "provider": row.provider,
        "name": row.name,
        "is_default": row.is_default,
        "is_active": row.is_active,
        "credentials": masked_creds,
        "config": config_val,
        "test_status": row.test_status,
        "latency_ms": row.latency_ms,
        "last_tested_at": row.last_tested_at,
        "created_at": row.created_at,
        "updated_at": row.updated_at,
        "stored_json_warnings": warnings
    })
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, sqlx::FromRow)]
pub struct WebhookEndpointDbRow {
    pub id: String,
    pub name: String,
    pub url: String,
    pub secret: String,
    pub events: String,
    pub headers: String,
    pub is_active: bool,
    pub retry_count: i64,
    pub timeout_seconds: i64,
    pub total_delivered: i64,
    pub total_failed: i64,
    pub last_status_code: Option<i64>,
    pub last_delivered_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

fn format_webhook_endpoint(row: WebhookEndpointDbRow) -> serde_json::Value {
    let mut warnings: Vec<String> = Vec::new();
    let events_val = match serde_json::from_str::<Vec<String>>(&row.events) {
        Ok(events) => json!(events),
        Err(err) => {
            warnings.push(format!(
                "The stored events column is not a JSON string array ({err}); it is displayed as empty."
            ));
            json!([])
        }
    };
    let headers_val = stored_json(&row.headers, "headers", &mut warnings);
    let secret_masked = mask_webhook_secret(&row.secret);
    let has_secret = !row.secret.is_empty();

    json!({
        "id": row.id,
        "name": row.name,
        "url": row.url,
        "secret_masked": secret_masked,
        "has_secret": has_secret,
        "events": events_val,
        "headers": headers_val,
        "is_active": row.is_active,
        "retry_count": row.retry_count,
        "timeout_seconds": row.timeout_seconds,
        "total_delivered": row.total_delivered,
        "total_failed": row.total_failed,
        "last_status_code": row.last_status_code,
        "last_delivered_at": row.last_delivered_at,
        "created_at": row.created_at,
        "updated_at": row.updated_at,
        "stored_json_warnings": warnings
    })
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, sqlx::FromRow)]
pub struct WebhookDeliveryLogDbRow {
    pub id: String,
    pub endpoint_id: String,
    pub event: String,
    pub payload: String,
    pub status_code: i64,
    pub response_headers: String,
    pub response_body: String,
    pub duration_ms: i64,
    pub attempt: i64,
    pub success: bool,
    pub error_message: Option<String>,
    pub created_at: String,
}

fn format_delivery_log(row: WebhookDeliveryLogDbRow) -> serde_json::Value {
    let mut warnings: Vec<String> = Vec::new();
    let payload_val = stored_json(&row.payload, "payload", &mut warnings);
    let headers_val = stored_json(&row.response_headers, "response_headers", &mut warnings);

    json!({
        "id": row.id,
        "endpoint_id": row.endpoint_id,
        "event": row.event,
        "payload": payload_val,
        "status_code": row.status_code,
        "response_headers": headers_val,
        "response_body": row.response_body,
        "duration_ms": row.duration_ms,
        "attempt": row.attempt,
        "success": row.success,
        "error_message": row.error_message,
        "created_at": row.created_at,
        "stored_json_warnings": warnings
    })
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, sqlx::FromRow)]
pub struct StripeConfigDbRow {
    pub id: String,
    pub account_name: String,
    pub api_key: String,
    pub publishable_key: String,
    pub webhook_secret: String,
    pub is_enabled: bool,
    pub test_mode: bool,
    pub default_currency: String,
    pub last_tested_at: Option<String>,
    pub last_test_status: String,
    pub created_at: String,
    pub updated_at: String,
}

fn format_stripe_config(row: StripeConfigDbRow) -> serde_json::Value {
    let api_key_masked = mask_stripe_key(&row.api_key);
    let webhook_secret_masked = mask_stripe_key(&row.webhook_secret);
    let has_api_key = !row.api_key.is_empty();
    let has_webhook_secret = !row.webhook_secret.is_empty();

    json!({
        "id": row.id,
        "account_name": row.account_name,
        "api_key_masked": api_key_masked,
        "publishable_key": row.publishable_key,
        "webhook_secret_masked": webhook_secret_masked,
        "has_api_key": has_api_key,
        "has_webhook_secret": has_webhook_secret,
        "is_enabled": row.is_enabled,
        "test_mode": row.test_mode,
        "default_currency": row.default_currency,
        "last_tested_at": row.last_tested_at,
        "last_test_status": row.last_test_status,
        "created_at": row.created_at,
        "updated_at": row.updated_at
    })
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, sqlx::FromRow)]
pub struct StripeBillingActionLogDbRow {
    pub id: String,
    pub contact_id: Option<String>,
    pub customer_identifier: String,
    pub action_type: String,
    pub amount_cents: i64,
    pub currency: String,
    pub status: String,
    pub stripe_id: String,
    pub actor_email: String,
    pub reason: String,
    pub details: String,
    pub created_at: String,
}

fn format_billing_log(row: StripeBillingActionLogDbRow) -> serde_json::Value {
    let mut warnings: Vec<String> = Vec::new();
    let details_val = stored_json(&row.details, "details", &mut warnings);
    // No "$" is invented for a row whose currency was never recorded; the
    // symbol would misreport the amount.
    let currency = row.currency.trim().to_string();
    let amount_formatted = if currency.is_empty() {
        warnings.push("amount_formatted is null: this log row stores no currency.".to_string());
        serde_json::Value::Null
    } else {
        json!(format!("{} {:.2}", currency.to_uppercase(), row.amount_cents as f64 / 100.0))
    };
    let stripe_id = if row.stripe_id.trim().is_empty() {
        serde_json::Value::Null
    } else {
        json!(row.stripe_id)
    };

    json!({
        "id": row.id,
        "contact_id": row.contact_id,
        "customer_identifier": row.customer_identifier,
        "action_type": row.action_type,
        "amount_cents": row.amount_cents,
        "amount_formatted": amount_formatted,
        "currency": currency,
        "status": row.status,
        "stripe_id": stripe_id,
        "actor_email": if row.actor_email.trim().is_empty() { serde_json::Value::Null } else { json!(row.actor_email) },
        "reason": if row.reason.trim().is_empty() { serde_json::Value::Null } else { json!(row.reason) },
        "details": details_val,
        "created_at": row.created_at,
        "stored_json_warnings": warnings
    })
}

/// The Stripe row is operator-owned. Nothing here creates or repairs it: only
/// `update_stripe_config` (POST /integrations/stripe/config) may write one.
async fn load_stripe_config(
    pool: &crate::db::DbPool,
) -> Result<Option<StripeConfigDbRow>, sqlx::Error> {
    sqlx::query_as::<_, StripeConfigDbRow>(
        "SELECT id, account_name, api_key, publishable_key, webhook_secret, is_enabled, test_mode, default_currency, last_tested_at, last_test_status, created_at, updated_at FROM stripe_configs LIMIT 1",
    )
    .fetch_optional(pool)
    .await
}

/// Shape returned when no Stripe account has been saved: no keys, no currency,
/// no mode -- just the list of what the operator still has to provide.
fn stripe_not_configured(what: &str) -> serde_json::Value {
    json!({
        "success": false,
        "configured": false,
        "live_call_made": false,
        "missing": ["account_name", "api_key", "publishable_key", "webhook_secret", "default_currency", "test_mode", "is_enabled"],
        "note": format!(
            "No Stripe account row exists in stripe_configs, so {what}. Save the settings with POST /api/v1/integrations/stripe/config to unblock this."
        ),
    })
}

/// For actions that cannot run without the stored row. A missing row is a 503
/// naming the gap; a failed read is a 500 carrying the sqlx text.
async fn require_stripe_config(
    pool: &crate::db::DbPool,
    what: &str,
) -> Result<StripeConfigDbRow, Response> {
    match load_stripe_config(pool).await {
        Err(err) => Err(db_error("Could not read the Stripe configuration", err)),
        Ok(None) => Err((
            StatusCode::SERVICE_UNAVAILABLE,
            Json(stripe_not_configured(what)),
        )
            .into_response()),
        Ok(Some(row)) => Ok(row),
    }
}

/// A saved column survives unless the payload explicitly replaces it.
fn pick_str(payload: &serde_json::Value, key: &str, stored: &str) -> String {
    match payload.get(key).and_then(|v| v.as_str()) {
        Some(value) => value.to_string(),
        None => stored.to_string(),
    }
}

fn pick_bool(payload: &serde_json::Value, key: &str, stored: bool) -> bool {
    match payload.get(key).and_then(|v| v.as_bool()) {
        Some(value) => value,
        None => stored,
    }
}

/// The value the operator just sent, or the one already on the row.
fn stored_or_new(incoming: Option<String>, stored: &str) -> String {
    match incoming {
        Some(value) => value,
        None => stored.to_string(),
    }
}

// ---------------------------------------------------------------------------
// Query Parameters
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct GatewayQueryParams {
    pub gateway_type: Option<String>,
}

#[derive(Deserialize)]
pub struct LimitQuery {
    pub limit: Option<i64>,
}

// ===========================================================================
// Universal Gateways Endpoints
// ===========================================================================

pub async fn list_gateways(
    State(state): State<AppState>,
    Query(query): Query<GatewayQueryParams>,
) -> impl IntoResponse {

    let rows = if let Some(ref g_type) = query.gateway_type {
        match sqlx::query_as::<_, GatewayDbRow>(
            "SELECT id, gateway_type, provider, name, is_default, is_active, credentials, config, test_status, latency_ms, last_tested_at, created_at, updated_at FROM channel_gateway_connections WHERE gateway_type = $1 ORDER BY is_default DESC, name ASC",
        )
        .bind(g_type)
        .fetch_all(&state.db)
        .await
        {
            Ok(rows) => rows,
            Err(err) => return db_error("Could not list gateway connections", err),
        }
    } else {
        match sqlx::query_as::<_, GatewayDbRow>(
            "SELECT id, gateway_type, provider, name, is_default, is_active, credentials, config, test_status, latency_ms, last_tested_at, created_at, updated_at FROM channel_gateway_connections ORDER BY gateway_type ASC, is_default DESC, name ASC",
        )
        .fetch_all(&state.db)
        .await
        {
            Ok(rows) => rows,
            Err(err) => return db_error("Could not list gateway connections", err),
        }
    };

    Json(rows.into_iter().map(format_gateway).collect::<Vec<_>>()).into_response()
}

pub async fn get_single_gateway(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {

    let row = match sqlx::query_as::<_, GatewayDbRow>(
        "SELECT id, gateway_type, provider, name, is_default, is_active, credentials, config, test_status, latency_ms, last_tested_at, created_at, updated_at FROM channel_gateway_connections WHERE id = $1",
    )
    .bind(&id)
    .fetch_optional(&state.db)
    .await
    {
        Ok(row) => row,
        Err(err) => return db_error("Could not read the gateway connection", err),
    };

    match row {
        Some(gw) => Json(format_gateway(gw)).into_response(),
        None => (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "Gateway not found", "found": false })),
        )
            .into_response(),
    }
}

pub async fn get_supported_gateway_providers() -> impl IntoResponse {
    Json(json!({
        "sms": [
            { "provider": "twilio", "name": "Twilio SMS", "fields": ["account_sid", "auth_token", "from_number"] },
            { "provider": "vonage", "name": "Vonage (Nexmo) SMS", "fields": ["api_key", "api_secret", "from"] },
            { "provider": "messagebird", "name": "MessageBird SMS", "fields": ["access_key", "originator"] },
            { "provider": "plivo", "name": "Plivo SMS", "fields": ["auth_id", "auth_token", "from_number"] },
            { "provider": "sinch", "name": "Sinch SMS", "fields": ["service_plan_id", "api_token", "from_number"] },
            { "provider": "aws_sns", "name": "AWS SNS SMS", "fields": ["aws_access_key", "aws_secret_key", "region"] },
            { "provider": "infobip", "name": "Infobip SMS", "fields": ["api_key", "base_url", "from_number"] },
            { "provider": "custom_http", "name": "Custom HTTP REST SMS Gateway", "fields": ["endpoint_url", "bearer_token", "payload_template"] }
        ],
        "email": [
            { "provider": "smtp", "name": "SMTP Server (TLS/SSL)", "fields": ["smtp_host", "smtp_port", "smtp_username", "smtp_password", "from_address"] },
            { "provider": "sendgrid", "name": "SendGrid API", "fields": ["api_key", "from_address", "from_name"] },
            { "provider": "aws_ses", "name": "AWS SES (Simple Email Service)", "fields": ["aws_access_key", "aws_secret_key", "region", "from_address"] },
            { "provider": "mailgun", "name": "Mailgun API", "fields": ["api_key", "domain", "from_address"] },
            { "provider": "postmark", "name": "Postmark API", "fields": ["api_key", "from_address"] },
            { "provider": "resend", "name": "Resend API", "fields": ["api_key", "from_address", "from_name"] },
            { "provider": "custom_webhook", "name": "Custom Outgoing Email Webhook", "fields": ["webhook_url", "auth_header", "from_address"] }
        ],
        "voice_sip": [
            { "provider": "twilio_voice", "name": "Twilio Voice & WebRTC", "fields": ["account_sid", "auth_token", "caller_id", "twiml_app_sid"] },
            { "provider": "telnyx", "name": "Telnyx SIP & Voice", "fields": ["api_key", "sip_connection_id", "caller_id"] },
            { "provider": "plivo_voice", "name": "Plivo Voice", "fields": ["auth_id", "auth_token", "caller_id"] },
            { "provider": "vonage_voice", "name": "Vonage Voice API", "fields": ["api_key", "private_key", "application_id"] },
            { "provider": "sip_trunk", "name": "Standard SIP Trunk (Asterisk/FreeSWITCH)", "fields": ["sip_host", "sip_port", "sip_username", "sip_password"] },
            { "provider": "custom_webrtc", "name": "Custom WebRTC Gateway", "fields": ["signaling_url", "stun_turn_servers"] }
        ]
    }))
}

pub async fn create_gateway(
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    let id = Uuid::new_v4().to_string();

    // Nothing here falls back to a believable provider/name: an SMS gateway
    // invented for a request that never named one would silently start
    // carrying traffic under the wrong credentials.
    let mut missing: Vec<&str> = Vec::new();
    let gateway_type = match payload["gateway_type"].as_str().map(str::trim) {
        Some(v) if !v.is_empty() => v.to_string(),
        _ => {
            missing.push("gateway_type");
            String::new()
        }
    };
    let provider = match payload["provider"].as_str().map(str::trim) {
        Some(v) if !v.is_empty() => v.to_string(),
        _ => {
            missing.push("provider");
            String::new()
        }
    };
    let name = match payload["name"].as_str().map(str::trim) {
        Some(v) if !v.is_empty() => v.to_string(),
        _ => {
            missing.push("name");
            String::new()
        }
    };

    let mut blob_errors: Vec<&str> = Vec::new();
    let mut defaults_applied: Vec<String> = Vec::new();
    let credentials_str = match payload.get("credentials") {
        None | Some(serde_json::Value::Null) => {
            defaults_applied.push("credentials: stored as {} (the request sent none)".to_string());
            "{}".to_string()
        }
        Some(v) if v.is_object() => v.to_string(),
        Some(_) => {
            blob_errors.push("credentials must be a JSON object");
            String::new()
        }
    };
    let config_str = match payload.get("config") {
        None | Some(serde_json::Value::Null) => {
            defaults_applied.push("config: stored as {} (the request sent none)".to_string());
            "{}".to_string()
        }
        Some(v) if v.is_object() => v.to_string(),
        Some(_) => {
            blob_errors.push("config must be a JSON object");
            String::new()
        }
    };

    let is_default = payload["is_default"].as_bool().unwrap_or_else(|| {
        defaults_applied
            .push("is_default: stored as false (the request did not state it)".to_string());
        false
    });
    let is_active = match payload["is_active"].as_bool() {
        Some(v) => v,
        None => {
            // The console has no per-gateway active switch, so creation stores
            // the schema default and says so instead of pretending it was asked.
            defaults_applied
                .push("is_active: stored as true (the request did not state it)".to_string());
            true
        }
    };

    if !missing.is_empty() || !blob_errors.is_empty() {
        let mut fields: Vec<String> = missing.iter().map(|f| f.to_string()).collect();
        for e in blob_errors {
            fields.push(e.to_string());
        }
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "success": false,
                "saved": false,
                "written": false,
                "error": "The gateway was not created: required settings are missing or malformed.",
                "missing": fields,
                "required": ["gateway_type", "provider", "name", "credentials (object)", "config (object)"]
            })),
        )
            .into_response();
    }

    let now = Utc::now().to_rfc3339();
    let mut warnings: Vec<String> = Vec::new();

    if is_default {
        match sqlx::query("UPDATE channel_gateway_connections SET is_default = 0, updated_at = $1 WHERE gateway_type = $2")
            .bind(&now)
            .bind(&gateway_type)
            .execute(&state.db)
            .await
        {
            Err(err) => warnings.push(format!(
                "Could not clear the previous default for gateway_type {gateway_type}: {err}"
            )),
            Ok(res) if res.rows_affected() == 0 => warnings.push(format!(
                "No previous default needed clearing for gateway_type {gateway_type} (0 rows affected)."
            )),
            Ok(_) => {}
        }
    }

    match sqlx::query(
        r#"
        INSERT INTO channel_gateway_connections (id, gateway_type, provider, name, is_default, is_active, credentials, config, test_status, latency_ms, last_tested_at, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, 'untested', 0, NULL, $9, $10)
        "#,
    )
    .bind(&id)
    .bind(&gateway_type)
    .bind(&provider)
    .bind(&name)
    .bind(is_default)
    .bind(is_active)
    .bind(&credentials_str)
    .bind(&config_str)
    .bind(&now)
    .bind(&now)
    .execute(&state.db)
    .await
    {
        Err(err) => return db_error("Could not create the gateway connection", err),
        Ok(res) if res.rows_affected() == 0 => {
            return nothing_written("The gateway connection")
        }
        Ok(_) => {}
    }

    let created = sqlx::query_as::<_, GatewayDbRow>(
        "SELECT id, gateway_type, provider, name, is_default, is_active, credentials, config, test_status, latency_ms, last_tested_at, created_at, updated_at FROM channel_gateway_connections WHERE id = $1",
    )
    .bind(&id)
    .fetch_optional(&state.db)
    .await;

    match created {
        Ok(Some(gw)) => {
            // Contract shape kept (the gateway object itself is the body); the
            // honesty metadata rides alongside as extra keys.
            let mut body = format_gateway(gw);
            if let Some(map) = body.as_object_mut() {
                map.insert("saved".into(), json!(true));
                map.insert("defaults_applied".into(), json!(defaults_applied));
                map.insert("warnings".into(), json!(warnings));
                map.insert(
                    "test_status_note".into(),
                    json!("test_status is 'untested': this request made no live provider call."),
                );
            }
            (StatusCode::CREATED, Json(body)).into_response()
        }
        Ok(None) => nothing_written("The gateway connection created"),
        Err(err) => db_error("The gateway was written but could not be read back", err),
    }
}

pub async fn update_gateway(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    let existing = match sqlx::query_as::<_, GatewayDbRow>(
        "SELECT id, gateway_type, provider, name, is_default, is_active, credentials, config, test_status, latency_ms, last_tested_at, created_at, updated_at FROM channel_gateway_connections WHERE id = $1",
    )
    .bind(&id)
    .fetch_optional(&state.db)
    .await
    {
        Ok(row) => row,
        Err(err) => return db_error("Could not read the gateway connection", err),
    };

    let Some(mut gw) = existing else {
        return (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "Gateway not found", "found": false, "written": false })),
        )
            .into_response();
    };

    let mut warnings: Vec<String> = Vec::new();
    // A masked or blank value is display text, not a credential: it is dropped
    // (and named in the response) instead of overwriting the real secret.
    let mut ignored_fields: Vec<String> = Vec::new();

    if let Some(n) = payload["name"].as_str() {
        if n.trim().is_empty() {
            ignored_fields.push("name (blank; the stored name was kept)".to_string());
        } else {
            gw.name = n.to_string();
        }
    }
    if let Some(act) = payload["is_active"].as_bool() {
        gw.is_active = act;
    }

    if let Some(new_creds) = payload.get("credentials").and_then(|v| v.as_object()) {
        let mut cur_creds: serde_json::Map<String, serde_json::Value> =
            serde_json::from_str(&gw.credentials).unwrap_or_else(|err| {
                warnings.push(format!(
                    "The stored credentials blob is not valid JSON ({err}); it was replaced, not merged."
                ));
                serde_json::Map::new()
            });
        for (k, v) in new_creds {
            match v.as_str() {
                Some(s) if s.trim().is_empty() => {
                    ignored_fields.push(format!("credentials.{k} (blank; the stored value was kept)"));
                }
                Some(s) if is_masked(s) => {
                    ignored_fields.push(format!(
                        "credentials.{k} (masked display value; re-enter the real value to change it)"
                    ));
                }
                _ => {
                    cur_creds.insert(k.clone(), v.clone());
                }
            }
        }
        gw.credentials = serde_json::Value::Object(cur_creds).to_string();
    } else if payload.get("credentials").is_some() && !payload["credentials"].is_object() {
        ignored_fields
            .push("credentials (not a JSON object; the stored credentials were kept)".to_string());
    }

    if let Some(cfg) = payload.get("config") {
        if cfg.is_object() {
            gw.config = cfg.to_string();
        } else {
            ignored_fields.push("config (not a JSON object; the stored config was kept)".to_string());
        }
    }

    gw.updated_at = Utc::now().to_rfc3339();

    match sqlx::query(
        "UPDATE channel_gateway_connections SET name = $1, is_active = $2, credentials = $3, config = $4, updated_at = $5 WHERE id = $6",
    )
    .bind(&gw.name)
    .bind(gw.is_active)
    .bind(&gw.credentials)
    .bind(&gw.config)
    .bind(&gw.updated_at)
    .bind(&id)
    .execute(&state.db)
    .await
    {
        Err(err) => return db_error("Could not save the gateway connection", err),
        Ok(res) if res.rows_affected() == 0 => {
            return nothing_written("The gateway connection update")
        }
        Ok(_) => {}
    }

    let mut body = format_gateway(gw);
    if let Some(map) = body.as_object_mut() {
        map.insert("saved".into(), json!(true));
        map.insert("ignored_fields".into(), json!(ignored_fields));
        map.insert("warnings".into(), json!(warnings));
    }
    Json(body).into_response()
}

pub async fn delete_gateway(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match sqlx::query("DELETE FROM channel_gateway_connections WHERE id = $1")
        .bind(&id)
        .execute(&state.db)
        .await
    {
        Err(err) => db_error("Could not delete the gateway connection", err),
        Ok(res) if res.rows_affected() == 0 => (
            StatusCode::NOT_FOUND,
            Json(json!({
                "success": false,
                "deleted": false,
                "deleted_id": id,
                "error": "No gateway connection with that id exists, so nothing was deleted."
            })),
        )
            .into_response(),
        Ok(res) => Json(json!({
            "success": true,
            "deleted": true,
            "deleted_id": id,
            "rows_affected": res.rows_affected()
        }))
        .into_response(),
    }
}

pub async fn test_gateway_connection(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let existing = match sqlx::query_as::<_, GatewayDbRow>(
        "SELECT id, gateway_type, provider, name, is_default, is_active, credentials, config, test_status, latency_ms, last_tested_at, created_at, updated_at FROM channel_gateway_connections WHERE id = $1",
    )
    .bind(&id)
    .fetch_optional(&state.db)
    .await
    {
        Ok(row) => row,
        Err(err) => return db_error("Could not read the gateway connection", err),
    };

    let Some(mut gw) = existing else {
        return (
            StatusCode::NOT_FOUND,
            Json(json!({ "success": false, "error": "Gateway connection not found", "found": false })),
        )
            .into_response();
    };

    let creds = parse_json_object(&gw.credentials);
    let cfg = parse_json_object(&gw.config);
    let probe = probe_gateway_connection(&state, &gw.provider, &creds, &cfg).await;

    let now = Utc::now().to_rfc3339();
    gw.test_status = if probe.ok {
        "connected"
    } else if probe.refusal.is_some() {
        "unverified"
    } else {
        "error"
    }
    .to_string();
    gw.latency_ms = probe.latency_ms;
    gw.last_tested_at = Some(now.clone());
    gw.updated_at = now.clone();

    let mut warnings: Vec<String> = Vec::new();
    match sqlx::query(
        "UPDATE channel_gateway_connections SET test_status = $1, latency_ms = $2, last_tested_at = $3, updated_at = $4 WHERE id = $5",
    )
    .bind(&gw.test_status)
    .bind(gw.latency_ms)
    .bind(&gw.last_tested_at)
    .bind(&gw.updated_at)
    .bind(&id)
    .execute(&state.db)
    .await
    {
        Err(err) => warnings.push(format!(
            "The probe ran but the stored test_status ({}) could not be written: {err}",
            gw.test_status
        )),
        Ok(res) if res.rows_affected() == 0 => warnings.push(format!(
            "The probe ran but the stored test_status ({}) was not written (0 rows affected).",
            gw.test_status
        )),
        Ok(_) => {}
    }

    Json(json!({
        "success": probe.ok,
        "gateway_id": gw.id,
        "gateway_name": gw.name,
        "provider": gw.provider,
        "gateway_type": gw.gateway_type,
        "test_status": gw.test_status,
        "latency_ms": probe.latency_ms,
        "http_status": probe.status_code,
        "endpoint": probe.endpoint,
        "live_call_made": probe.live_call_made(),
        "error": if probe.ok { None } else { Some(probe.headline()) },
        "refusal": probe.refusal,
        "warnings": warnings,
        "checked_at": now
    }))
    .into_response()
}

pub async fn set_default_gateway(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let existing = match sqlx::query_as::<_, GatewayDbRow>(
        "SELECT id, gateway_type, provider, name, is_default, is_active, credentials, config, test_status, latency_ms, last_tested_at, created_at, updated_at FROM channel_gateway_connections WHERE id = $1",
    )
    .bind(&id)
    .fetch_optional(&state.db)
    .await
    {
        Ok(row) => row,
        Err(err) => return db_error("Could not read the gateway connection", err),
    };

    let Some(mut gw) = existing else {
        return (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "Gateway not found", "found": false, "written": false })),
        )
            .into_response();
    };

    let now = Utc::now().to_rfc3339();
    let mut warnings: Vec<String> = Vec::new();

    // Unset default on others of same gateway_type
    match sqlx::query("UPDATE channel_gateway_connections SET is_default = 0, updated_at = $1 WHERE gateway_type = $2")
        .bind(&now)
        .bind(&gw.gateway_type)
        .execute(&state.db)
        .await
    {
        Err(err) => return db_error("Could not clear the previous default gateway", err),
        Ok(res) if res.rows_affected() == 0 => warnings.push(format!(
            "No other {} gateway held the default flag (0 rows affected).",
            gw.gateway_type
        )),
        Ok(_) => {}
    }

    // Set this one as default
    gw.is_default = true;
    gw.updated_at = now.clone();

    match sqlx::query("UPDATE channel_gateway_connections SET is_default = 1, updated_at = $1 WHERE id = $2")
        .bind(&gw.updated_at)
        .bind(&id)
        .execute(&state.db)
        .await
    {
        Err(err) => return db_error("Could not store the default gateway", err),
        Ok(res) if res.rows_affected() == 0 => {
            return nothing_written("The default gateway flag")
        }
        Ok(_) => {}
    }

    let mut body = format_gateway(gw);
    if let Some(map) = body.as_object_mut() {
        map.insert("saved".into(), json!(true));
        map.insert("warnings".into(), json!(warnings));
    }
    Json(body).into_response()
}

pub async fn send_gateway_ping(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    let existing = match sqlx::query_as::<_, GatewayDbRow>(
        "SELECT id, gateway_type, provider, name, is_default, is_active, credentials, config, test_status, latency_ms, last_tested_at, created_at, updated_at FROM channel_gateway_connections WHERE id = $1",
    )
    .bind(&id)
    .fetch_optional(&state.db)
    .await
    {
        Ok(row) => row,
        Err(err) => return db_error("Could not read the gateway connection", err),
    };

    let Some(mut gw) = existing else {
        return (
            StatusCode::NOT_FOUND,
            Json(json!({ "success": false, "error": "Gateway not found", "found": false })),
        )
            .into_response();
    };

    // A test transmission is a real send: the destination and the text must be
    // supplied, we never fall back to a demo number or a canned message.
    let target = payload["target"]
        .as_str()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty() && !is_masked(s));
    let Some(target) = target else {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "success": false,
                "sent": false,
                "live_call_made": false,
                "error": "A real destination is required for a test transmission; no demo number is substituted.",
                "missing": ["target"]
            })),
        )
            .into_response();
    };
    let message = payload["message"]
        .as_str()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());
    let Some(message) = message else {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "success": false,
                "sent": false,
                "live_call_made": false,
                "error": "A message body is required; no placeholder text is sent on your behalf.",
                "missing": ["message"]
            })),
        )
            .into_response();
    };

    let creds = parse_json_object(&gw.credentials);
    let cfg = parse_json_object(&gw.config);
    let probe =
        dispatch_gateway_message(&state, &gw.provider, &gw.gateway_type, &creds, &cfg, &target, &message)
            .await;

    let now = Utc::now().to_rfc3339();
    gw.last_tested_at = Some(now.clone());
    gw.latency_ms = probe.latency_ms;
    if probe.live_call_made() {
        gw.test_status = if probe.ok { "connected" } else { "error" }.to_string();
    }
    let mut warnings: Vec<String> = Vec::new();
    match sqlx::query(
        "UPDATE channel_gateway_connections SET last_tested_at = $1, latency_ms = $2, test_status = $3, updated_at = $4 WHERE id = $5",
    )
    .bind(&now)
    .bind(gw.latency_ms)
    .bind(&gw.test_status)
    .bind(&now)
    .bind(&id)
    .execute(&state.db)
    .await
    {
        Err(err) => warnings.push(format!(
            "The transmission happened but the gateway state (test_status = {}) could not be stored: {err}",
            gw.test_status
        )),
        Ok(res) if res.rows_affected() == 0 => warnings.push(
            "The transmission happened but the gateway state was not stored (0 rows affected)."
                .to_string(),
        ),
        Ok(_) => {}
    }

    Json(json!({
        "success": probe.ok,
        "provider": gw.provider,
        "gateway_type": gw.gateway_type,
        "target": target,
        "message_id": probe.remote_id,
        "latency_ms": probe.latency_ms,
        "http_status": probe.status_code,
        "endpoint": probe.endpoint,
        "live_call_made": probe.live_call_made(),
        "simulated": false,
        "error": if probe.ok { None } else { Some(probe.headline()) },
        "refusal": probe.refusal,
        "warnings": warnings,
        "timestamp": now
    }))
    .into_response()
}

// ===========================================================================
// Outgoing Webhooks Engine Endpoints
// ===========================================================================

pub async fn list_webhook_endpoints(State(state): State<AppState>) -> impl IntoResponse {

    let rows = match sqlx::query_as::<_, WebhookEndpointDbRow>(
        "SELECT id, name, url, secret, events, headers, is_active, retry_count, timeout_seconds, total_delivered, total_failed, last_status_code, last_delivered_at, created_at, updated_at FROM webhook_endpoints ORDER BY created_at DESC",
    )
    .fetch_all(&state.db)
    .await
    {
        Ok(rows) => rows,
        Err(err) => return db_error("Could not list webhook endpoints", err),
    };

    Json(
        rows.into_iter()
            .map(format_webhook_endpoint)
            .collect::<Vec<_>>(),
    )
    .into_response()
}

pub async fn get_single_webhook_endpoint(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {

    let row = match sqlx::query_as::<_, WebhookEndpointDbRow>(
        "SELECT id, name, url, secret, events, headers, is_active, retry_count, timeout_seconds, total_delivered, total_failed, last_status_code, last_delivered_at, created_at, updated_at FROM webhook_endpoints WHERE id = $1",
    )
    .bind(&id)
    .fetch_optional(&state.db)
    .await
    {
        Ok(row) => row,
        Err(err) => return db_error("Could not read the webhook endpoint", err),
    };

    match row {
        Some(ep) => Json(format_webhook_endpoint(ep)).into_response(),
        None => (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "Webhook endpoint not found", "found": false })),
        )
            .into_response(),
    }
}

pub async fn get_webhook_catalog() -> impl IntoResponse {
    Json(vec![
        json!({
            "event": "conversation.created",
            "description": "Triggered when a new conversation thread is initiated by customer or agent.",
            "category": "Conversations"
        }),
        json!({
            "event": "conversation.status_changed",
            "description": "Triggered when conversation state transitions (open, pending, resolved, snoozed).",
            "category": "Conversations"
        }),
        json!({
            "event": "conversation.assigned",
            "description": "Triggered when conversation is assigned to an agent or team queue.",
            "category": "Conversations"
        }),
        json!({
            "event": "message.created",
            "description": "Triggered on every inbound message from customer or outbound agent reply.",
            "category": "Messages"
        }),
        json!({
            "event": "ticket.created",
            "description": "Triggered when an issue ticket or tracker outage ticket is created.",
            "category": "Ticketing"
        }),
        json!({
            "event": "ticket.updated",
            "description": "Triggered when ticket department, priority, or assignee is modified.",
            "category": "Ticketing"
        }),
        json!({
            "event": "ticket.resolved",
            "description": "Triggered when ticket is closed and marked resolved.",
            "category": "Ticketing"
        }),
        json!({
            "event": "csat.submitted",
            "description": "Triggered when customer submits satisfaction score (1-5 stars) and comment.",
            "category": "Satisfaction"
        }),
        json!({
            "event": "contact.created",
            "description": "Triggered when a new contact record is created or discovered.",
            "category": "Contacts"
        }),
        json!({
            "event": "contact.updated",
            "description": "Triggered when contact attributes, tags, or preferences change.",
            "category": "Contacts"
        })
    ])
}

pub async fn get_webhook_deliveries(
    State(state): State<AppState>,
    Query(query): Query<LimitQuery>,
) -> impl IntoResponse {
    let limit = query.limit.unwrap_or(50).clamp(1, 200);

    let rows = match sqlx::query_as::<_, WebhookDeliveryLogDbRow>(
        "SELECT id, endpoint_id, event, payload, status_code, response_headers, response_body, duration_ms, attempt, success, error_message, created_at FROM webhook_delivery_logs ORDER BY created_at DESC LIMIT $1",
    )
    .bind(limit)
    .fetch_all(&state.db)
    .await
    {
        Ok(rows) => rows,
        Err(err) => return db_error("Could not read the delivery log", err),
    };

    Json(
        rows.into_iter()
            .map(format_delivery_log)
            .collect::<Vec<_>>(),
    )
    .into_response()
}

pub async fn get_endpoint_deliveries(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(query): Query<LimitQuery>,
) -> impl IntoResponse {
    let limit = query.limit.unwrap_or(50).clamp(1, 200);

    let rows = match sqlx::query_as::<_, WebhookDeliveryLogDbRow>(
        "SELECT id, endpoint_id, event, payload, status_code, response_headers, response_body, duration_ms, attempt, success, error_message, created_at FROM webhook_delivery_logs WHERE endpoint_id = $1 ORDER BY created_at DESC LIMIT $2",
    )
    .bind(&id)
    .bind(limit)
    .fetch_all(&state.db)
    .await
    {
        Ok(rows) => rows,
        Err(err) => return db_error("Could not read the delivery log", err),
    };

    Json(
        rows.into_iter()
            .map(format_delivery_log)
            .collect::<Vec<_>>(),
    )
    .into_response()
}

pub async fn create_webhook_endpoint(
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    let id = Uuid::new_v4().to_string();

    // "https://example.com/webhook" used to be the fallback URL: every delivery
    // then "succeeded" against somebody else's domain. Both name and URL must
    // be real.
    let mut problems: Vec<String> = Vec::new();
    let name = match payload["name"].as_str().map(str::trim) {
        Some(v) if !v.is_empty() => v.to_string(),
        _ => {
            problems.push("name (a non-empty endpoint name is required)".to_string());
            String::new()
        }
    };
    let url = match payload["url"].as_str().map(str::trim) {
        Some(v) if is_deliverable_http_url(v) => v.to_string(),
        Some(v) if !v.is_empty() => {
            problems.push(format!(
                "url ({v:?} is not an absolute http(s) URL with a host; deliveries could never reach it)"
            ));
            String::new()
        }
        _ => {
            problems.push("url (an absolute http(s) URL is required)".to_string());
            String::new()
        }
    };

    let mut events_str = String::new();
    let mut events_absent = false;
    match payload.get("events") {
        None | Some(serde_json::Value::Null) => {
            events_str = "[]".to_string();
            events_absent = true;
        }
        Some(v) => match v.as_array() {
            Some(items)
                if items
                    .iter()
                    .all(|i| i.as_str().map(|s| !s.trim().is_empty()).unwrap_or(false)) =>
            {
                events_str = v.to_string();
            }
            Some(_) => problems.push("events (every entry must be a non-empty string)".to_string()),
            None => problems.push("events (must be a JSON array of event names)".to_string()),
        },
    }

    let mut headers_str = String::new();
    match payload.get("headers") {
        None | Some(serde_json::Value::Null) => headers_str = "{}".to_string(),
        Some(v) if v.is_object() => headers_str = v.to_string(),
        Some(_) => problems.push("headers (must be a JSON object)".to_string()),
    }

    if !problems.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "success": false,
                "saved": false,
                "written": false,
                "error": "The webhook endpoint was not created.",
                "missing": problems
            })),
        )
            .into_response();
    }

    let mut defaults_applied: Vec<String> = Vec::new();
    let supplied_secret = payload["secret"].as_str().map(str::trim).filter(|s| !s.is_empty());
    let generated_secret = format!("whsec_{}", Uuid::new_v4().to_string().replace('-', ""));
    let secret = match supplied_secret {
        Some(s) if is_masked(s) => {
            // A masked echo is not a secret; generate a real one instead of
            // storing the display text.
            defaults_applied.push(
                "secret: the submitted value was a masked display string, so a new real secret was generated"
                    .to_string(),
            );
            generated_secret.clone()
        }
        Some(s) => s.to_string(),
        None => {
            defaults_applied.push(
                "secret: none supplied, so a fresh HMAC signing secret was generated and is returned once in secret_plaintext"
                    .to_string(),
            );
            generated_secret.clone()
        }
    };
    let was_generated_secret = supplied_secret.is_none()
        || supplied_secret.map(is_masked).unwrap_or(false);

    let is_active = payload["is_active"].as_bool().unwrap_or_else(|| {
        defaults_applied.push("is_active: stored as true (the request did not state it)".to_string());
        true
    });
    let retry_count = payload["retry_count"].as_i64().unwrap_or_else(|| {
        defaults_applied.push("retry_count: stored as 3 (the request did not state it)".to_string());
        3
    });
    let timeout_seconds = payload["timeout_seconds"].as_i64().unwrap_or_else(|| {
        defaults_applied.push("timeout_seconds: stored as 10 (the request did not state it)".to_string());
        10
    });
    if events_absent {
        defaults_applied
            .push("events: stored as [] (the request sent none) — this endpoint receives no events".to_string());
    }
    let now = Utc::now().to_rfc3339();

    match sqlx::query(
        r#"
        INSERT INTO webhook_endpoints (id, name, url, secret, events, headers, is_active, retry_count, timeout_seconds, total_delivered, total_failed, last_status_code, last_delivered_at, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, 0, 0, NULL, NULL, $10, $11)
        "#,
    )
    .bind(&id)
    .bind(&name)
    .bind(&url)
    .bind(&secret)
    .bind(&events_str)
    .bind(&headers_str)
    .bind(is_active)
    .bind(retry_count)
    .bind(timeout_seconds)
    .bind(&now)
    .bind(&now)
    .execute(&state.db)
    .await
    {
        Err(err) => return db_error("Could not create the webhook endpoint", err),
        Ok(res) if res.rows_affected() == 0 => return nothing_written("The webhook endpoint"),
        Ok(_) => {}
    }

    let created = sqlx::query_as::<_, WebhookEndpointDbRow>(
        "SELECT id, name, url, secret, events, headers, is_active, retry_count, timeout_seconds, total_delivered, total_failed, last_status_code, last_delivered_at, created_at, updated_at FROM webhook_endpoints WHERE id = $1",
    )
    .bind(&id)
    .fetch_optional(&state.db)
    .await;

    match created {
        Ok(Some(ep)) => {
            let mut body = format_webhook_endpoint(ep);
            if let Some(map) = body.as_object_mut() {
                map.insert("saved".into(), json!(true));
                map.insert("defaults_applied".into(), json!(defaults_applied));
                // The plaintext secret is only ever returned here, once.
                map.insert(
                    "secret_plaintext".into(),
                    if was_generated_secret {
                        json!(secret)
                    } else {
                        serde_json::Value::Null
                    },
                );
                map.insert(
                    "secret_note".into(),
                    json!(if was_generated_secret {
                        "Copy secret_plaintext now: it is stored hashed-free but only displayed masked afterwards."
                    } else {
                        "The secret you supplied was stored; it is only displayed masked."
                    }),
                );
            }
            (StatusCode::CREATED, Json(body)).into_response()
        }
        Ok(None) => nothing_written("The webhook endpoint created"),
        Err(err) => db_error("The webhook endpoint was written but could not be read back", err),
    }
}

pub async fn update_webhook_endpoint(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    let existing = match sqlx::query_as::<_, WebhookEndpointDbRow>(
        "SELECT id, name, url, secret, events, headers, is_active, retry_count, timeout_seconds, total_delivered, total_failed, last_status_code, last_delivered_at, created_at, updated_at FROM webhook_endpoints WHERE id = $1",
    )
    .bind(&id)
    .fetch_optional(&state.db)
    .await
    {
        Ok(row) => row,
        Err(err) => return db_error("Could not read the webhook endpoint", err),
    };

    let Some(mut ep) = existing else {
        return (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "Webhook endpoint not found", "found": false, "written": false })),
        )
            .into_response();
    };

    let mut ignored_fields: Vec<String> = Vec::new();
    let mut warnings: Vec<String> = Vec::new();

    if let Some(n) = payload["name"].as_str() {
        if n.trim().is_empty() {
            ignored_fields.push("name (blank; the stored name was kept)".to_string());
        } else {
            ep.name = n.to_string();
        }
    }
    if let Some(u) = payload["url"].as_str() {
        if is_deliverable_http_url(u.trim()) {
            ep.url = u.trim().to_string();
        } else {
            ignored_fields.push(format!(
                "url ({u:?} is not an absolute http(s) URL; the stored URL {} was kept)",
                ep.url
            ));
        }
    }
    if let Some(v) = payload.get("events") {
        match v.as_array() {
            Some(items)
                if items
                    .iter()
                    .all(|i| i.as_str().map(|s| !s.trim().is_empty()).unwrap_or(false)) =>
            {
                ep.events = v.to_string();
            }
            Some(_) => ignored_fields.push("events (an entry is not a non-empty string; stored events kept)".to_string()),
            None => ignored_fields.push("events (not a JSON array; stored events kept)".to_string()),
        }
    }
    if let Some(v) = payload.get("headers") {
        if v.is_object() {
            ep.headers = v.to_string();
        } else {
            ignored_fields.push("headers (not a JSON object; stored headers kept)".to_string());
        }
    }
    if let Some(act) = payload["is_active"].as_bool() {
        ep.is_active = act;
    }
    if let Some(rc) = payload["retry_count"].as_i64() {
        ep.retry_count = rc;
    }
    if let Some(ts) = payload["timeout_seconds"].as_i64() {
        ep.timeout_seconds = ts;
    }

    let rotate = payload["rotate_secret"].as_bool() == Some(true);
    let mut new_secret = ep.secret.clone();
    if rotate {
        new_secret = format!("whsec_{}", Uuid::new_v4().to_string().replace('-', ""));
        ep.secret = new_secret.clone();
    } else if let Some(s) = payload["secret"].as_str() {
        let trimmed = s.trim();
        if trimmed.is_empty() || is_masked(trimmed) {
            ignored_fields.push(
                "secret (blank or masked display value; the stored secret was kept)".to_string(),
            );
        } else {
            new_secret = trimmed.to_string();
            ep.secret = new_secret.clone();
        }
    }

    ep.updated_at = Utc::now().to_rfc3339();
    if ep.events.trim() == "[]" {
        warnings.push(
            "This endpoint is subscribed to no events, so the dispatcher will never deliver to it."
                .to_string(),
        );
    }

    match sqlx::query(
        "UPDATE webhook_endpoints SET name = $1, url = $2, secret = $3, events = $4, headers = $5, is_active = $6, retry_count = $7, timeout_seconds = $8, updated_at = $9 WHERE id = $10",
    )
    .bind(&ep.name)
    .bind(&ep.url)
    .bind(&ep.secret)
    .bind(&ep.events)
    .bind(&ep.headers)
    .bind(ep.is_active)
    .bind(ep.retry_count)
    .bind(ep.timeout_seconds)
    .bind(&ep.updated_at)
    .bind(&id)
    .execute(&state.db)
    .await
    {
        Err(err) => return db_error("Could not save the webhook endpoint", err),
        Ok(res) if res.rows_affected() == 0 => return nothing_written("The webhook endpoint update"),
        Ok(_) => {}
    }

    let mut body = format_webhook_endpoint(ep);
    let rotated_plaintext = if rotate { Some(new_secret.clone()) } else { None };
    if let Some(map) = body.as_object_mut() {
        map.insert("saved".into(), json!(true));
        map.insert("ignored_fields".into(), json!(ignored_fields));
        map.insert("warnings".into(), json!(warnings));
        map.insert(
            "secret_plaintext".into(),
            match rotated_plaintext {
                Some(text) => json!(text),
                None => serde_json::Value::Null,
            },
        );
        if rotate {
            map.insert(
                "secret_note".into(),
                json!("A new signing secret was generated; copy secret_plaintext now, it is only displayed masked afterwards."),
            );
        }
    }
    Json(body).into_response()
}

pub async fn delete_webhook_endpoint(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let logs = match sqlx::query("DELETE FROM webhook_delivery_logs WHERE endpoint_id = $1")
        .bind(&id)
        .execute(&state.db)
        .await
    {
        Err(err) => return db_error("Could not delete the endpoint delivery logs", err),
        Ok(res) => res.rows_affected(),
    };

    match sqlx::query("DELETE FROM webhook_endpoints WHERE id = $1")
        .bind(&id)
        .execute(&state.db)
        .await
    {
        Err(err) => db_error("Could not delete the webhook endpoint", err),
        Ok(res) if res.rows_affected() == 0 => (
            StatusCode::NOT_FOUND,
            Json(json!({
                "success": false,
                "deleted": false,
                "deleted_id": id,
                "error": "No webhook endpoint with that id exists, so nothing was deleted.",
                "delivery_logs_deleted": logs
            })),
        )
            .into_response(),
        Ok(res) => Json(json!({
            "success": true,
            "deleted": true,
            "deleted_id": id,
            "rows_affected": res.rows_affected(),
            "delivery_logs_deleted": logs
        }))
        .into_response(),
    }
}

pub async fn test_webhook_ping(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let existing = match sqlx::query_as::<_, WebhookEndpointDbRow>(
        "SELECT id, name, url, secret, events, headers, is_active, retry_count, timeout_seconds, total_delivered, total_failed, last_status_code, last_delivered_at, created_at, updated_at FROM webhook_endpoints WHERE id = $1",
    )
    .bind(&id)
    .fetch_optional(&state.db)
    .await
    {
        Ok(row) => row,
        Err(err) => return db_error("Could not read the webhook endpoint", err),
    };

    let Some(mut ep) = existing else {
        return (
            StatusCode::NOT_FOUND,
            Json(json!({ "success": false, "error": "Endpoint not found" })),
        )
            .into_response();
    };

    let delivery_id = Uuid::new_v4().to_string();
    let now = Utc::now();
    let now_str = now.to_rfc3339();

    let payload = json!({
        "id": format!("evt_ping_{}", &Uuid::new_v4().to_string()[..12]),
        "event": "ping.test",
        "timestamp": now_str,
        "data": {
            "message": "BayWorks Outgoing Webhook Diagnostic Test Ping",
            "endpoint_id": ep.id,
            "endpoint_name": ep.name,
            "target_url": ep.url,
            "signature_algorithm": if ep.secret.trim().is_empty() { serde_json::Value::Null } else { json!("HMAC-SHA256") }
        }
    });
    let payload_str = match serde_json::to_string(&payload) {
        Ok(text) => text,
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "success": false, "error": format!("The ping payload could not be serialized: {err}") })),
            )
                .into_response();
        }
    };
    let payload_bytes = payload_str.clone().into_bytes();
    let signature = compute_hmac_sha256(&ep.secret, &payload_bytes);
    let mut warnings: Vec<String> = Vec::new();
    if signature.is_none() {
        warnings.push(
            "This endpoint has no signing secret stored, so the request was sent unsigned."
                .to_string(),
        );
    }

    // No URL is ever answered without an actual round-trip: an unreachable or
    // malformed target reports failure, never a fabricated 200.
    let (status_code, duration_ms, response_body, response_headers, success, error_message) = {
        let start = std::time::Instant::now();
        let timeout = std::time::Duration::from_secs(ep.timeout_seconds.max(1) as u64);
        let mut req = state.http_client
            .post(&ep.url)
            .timeout(timeout)
            .header("Content-Type", "application/json")
            .header("User-Agent", "BayWorks-Webhook-Dispatcher/1.0")
            .header("X-Communicator-Event", "ping.test")
            .header("X-Communicator-Delivery", &delivery_id)
            .header("X-Communicator-Timestamp", Utc::now().timestamp().to_string())
            .body(payload_bytes);
        if let Some(ref signature) = signature {
            req = req.header("X-Communicator-Signature", signature.clone());
        }

        // Add custom headers if present
        if let Ok(h_map) = serde_json::from_str::<std::collections::HashMap<String, String>>(&ep.headers) {
            for (k, v) in h_map {
                req = req.header(k, v);
            }
        }

        match req.send().await {
            Ok(resp) => {
                let code = resp.status().as_u16() as i64;
                let dur = start.elapsed().as_millis() as i64;
                let headers = collect_response_headers(resp.headers());
                let text = read_body(resp.text().await);
                let body_trunc = clip(&text, 1000);
                let is_succ = (200..300).contains(&code);
                let err = if !is_succ { Some(format!("HTTP status {code}")) } else { None };
                (code, dur, body_trunc, headers, is_succ, err)
            }
            Err(e) => {
                let dur = start.elapsed().as_millis() as i64;
                let reason = if e.is_builder() {
                    format!("Refusing to dispatch: the endpoint URL {url} is not a valid absolute http(s) URL.", url = ep.url)
                } else if e.is_timeout() {
                    format!(
                        "Timed out after {}s waiting for {url}",
                        ep.timeout_seconds.max(1),
                        url = ep.url
                    )
                } else {
                    format!("Could not reach {url}: {e}", url = ep.url)
                };
                (0, dur, String::new(), serde_json::json!({}), false, Some(reason))
            }
        }
    };

    // Insert delivery log
    let headers_str = match serde_json::to_string(&response_headers) {
        Ok(text) => text,
        Err(err) => {
            warnings.push(format!("The response headers could not be serialized for the log: {err}"));
            String::new()
        }
    };
    match sqlx::query(
        r#"
        INSERT INTO webhook_delivery_logs (id, endpoint_id, event, payload, status_code, response_headers, response_body, duration_ms, attempt, success, error_message, created_at)
        VALUES ($1, $2, 'ping.test', $3, $4, $5, $6, $7, 1, $8, $9, $10)
        "#,
    )
    .bind(&delivery_id)
    .bind(&ep.id)
    .bind(&payload_str)
    .bind(status_code)
    .bind(&headers_str)
    .bind(&response_body)
    .bind(duration_ms)
    .bind(success)
    .bind(&error_message)
    .bind(&now_str)
    .execute(&state.db)
    .await
    {
        Err(err) => warnings.push(format!("The delivery was dispatched but the log row could not be stored: {err}")),
        Ok(res) if res.rows_affected() == 0 => {
            warnings.push("The delivery was dispatched but no log row was stored (0 rows affected).".to_string())
        }
        Ok(_) => {}
    }

    // Update endpoint stats
    if success {
        ep.total_delivered += 1;
    } else {
        ep.total_failed += 1;
    }
    ep.last_status_code = Some(status_code);
    ep.last_delivered_at = Some(now_str.clone());
    ep.updated_at = now_str.clone();

    match sqlx::query(
        "UPDATE webhook_endpoints SET total_delivered = $1, total_failed = $2, last_status_code = $3, last_delivered_at = $4, updated_at = $5 WHERE id = $6",
    )
    .bind(ep.total_delivered)
    .bind(ep.total_failed)
    .bind(ep.last_status_code)
    .bind(&ep.last_delivered_at)
    .bind(&ep.updated_at)
    .bind(&id)
    .execute(&state.db)
    .await
    {
        Err(err) => warnings.push(format!("The endpoint counters could not be stored: {err}")),
        Ok(res) if res.rows_affected() == 0 => {
            warnings.push("The endpoint counters were not stored (0 rows affected).".to_string())
        }
        Ok(_) => {}
    }

    Json(json!({
        "success": success,
        "delivery_id": delivery_id,
        "endpoint_id": ep.id,
        "url": ep.url,
        "status_code": status_code,
        "duration_ms": duration_ms,
        "response_body": response_body,
        "response_headers": response_headers,
        "signed": signature.is_some(),
        "error": error_message,
        "warnings": warnings,
        "tested_at": now_str
    }))
    .into_response()
}

pub async fn resend_webhook_delivery(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let log_row =
        match sqlx::query_as::<_, WebhookDeliveryLogDbRow>(
            "SELECT id, endpoint_id, event, payload, status_code, response_headers, response_body, duration_ms, attempt, success, error_message, created_at FROM webhook_delivery_logs WHERE id = $1",
        )
        .bind(&id)
        .fetch_optional(&state.db)
        .await
        {
            Ok(row) => row,
            Err(err) => return db_error("Could not read the delivery log", err),
        };

    let Some(orig_log) = log_row else {
        return (
            StatusCode::NOT_FOUND,
            Json(json!({ "success": false, "error": "Delivery log not found" })),
        )
            .into_response();
    };

    let ep_row =
        match sqlx::query_as::<_, WebhookEndpointDbRow>(
            "SELECT id, name, url, secret, events, headers, is_active, retry_count, timeout_seconds, total_delivered, total_failed, last_status_code, last_delivered_at, created_at, updated_at FROM webhook_endpoints WHERE id = $1",
        )
        .bind(&orig_log.endpoint_id)
        .fetch_optional(&state.db)
        .await
        {
            Ok(row) => row,
            Err(err) => return db_error("Could not read the webhook endpoint", err),
        };

    let Some(mut ep) = ep_row else {
        return (
            StatusCode::NOT_FOUND,
            Json(json!({ "success": false, "error": "Webhook endpoint no longer exists" })),
        )
            .into_response();
    };

    let new_delivery_id = Uuid::new_v4().to_string();
    let now = Utc::now();
    let now_str = now.to_rfc3339();
    let payload_bytes = orig_log.payload.as_bytes();
    let signature = compute_hmac_sha256(&ep.secret, payload_bytes);
    let mut warnings: Vec<String> = Vec::new();
    if signature.is_none() {
        warnings.push(
            "This endpoint has no signing secret stored, so the re-delivery was sent unsigned."
                .to_string(),
        );
    }

    // Always a real re-delivery: the endpoint's actual HTTP answer (or the
    // transport failure) is what gets logged and returned.
    let (status_code, duration_ms, response_body, response_headers, success, error_message) = {
        let start = std::time::Instant::now();
        let timeout = std::time::Duration::from_secs(ep.timeout_seconds.max(1) as u64);
        let mut req = state.http_client
            .post(&ep.url)
            .timeout(timeout)
            .header("Content-Type", "application/json")
            .header("User-Agent", "BayWorks-Webhook-Dispatcher/1.0")
            .header("X-Communicator-Event", &orig_log.event)
            .header("X-Communicator-Delivery", &new_delivery_id)
            .header("X-Communicator-Timestamp", Utc::now().timestamp().to_string())
            .body(orig_log.payload.clone());
        if let Some(ref signature) = signature {
            req = req.header("X-Communicator-Signature", signature.clone());
        }

        if let Ok(h_map) = serde_json::from_str::<std::collections::HashMap<String, String>>(&ep.headers) {
            for (k, v) in h_map {
                req = req.header(k, v);
            }
        }

        match req.send().await {
            Ok(resp) => {
                let code = resp.status().as_u16() as i64;
                let dur = start.elapsed().as_millis() as i64;
                let headers = collect_response_headers(resp.headers());
                let text = read_body(resp.text().await);
                let body_trunc = clip(&text, 1000);
                let is_succ = (200..300).contains(&code);
                let err = if !is_succ { Some(format!("HTTP status {code}")) } else { None };
                (code, dur, body_trunc, headers, is_succ, err)
            }
            Err(e) => {
                let dur = start.elapsed().as_millis() as i64;
                let reason = if e.is_builder() {
                    format!("Refusing to dispatch: the endpoint URL {} is not a valid absolute http(s) URL.", ep.url)
                } else if e.is_timeout() {
                    format!("Timed out after {}s waiting for {}", ep.timeout_seconds.max(1), ep.url)
                } else {
                    format!("Could not reach {}: {e}", ep.url)
                };
                (0, dur, String::new(), json!({}), false, Some(reason))
            }
        }
    };

    let new_attempt = orig_log.attempt + 1;
    let headers_str = match serde_json::to_string(&response_headers) {
        Ok(text) => text,
        Err(err) => {
            warnings.push(format!("The response headers could not be serialized for the log: {err}"));
            String::new()
        }
    };
    match sqlx::query(
        r#"
        INSERT INTO webhook_delivery_logs (id, endpoint_id, event, payload, status_code, response_headers, response_body, duration_ms, attempt, success, error_message, created_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
        "#,
    )
    .bind(&new_delivery_id)
    .bind(&ep.id)
    .bind(&orig_log.event)
    .bind(&orig_log.payload)
    .bind(status_code)
    .bind(&headers_str)
    .bind(&response_body)
    .bind(duration_ms)
    .bind(new_attempt)
    .bind(success)
    .bind(&error_message)
    .bind(&now_str)
    .execute(&state.db)
    .await
    {
        Err(err) => warnings.push(format!("The re-delivery ran but its log row could not be stored: {err}")),
        Ok(res) if res.rows_affected() == 0 => {
            warnings.push("The re-delivery ran but no log row was stored (0 rows affected).".to_string())
        }
        Ok(_) => {}
    }

    if success {
        ep.total_delivered += 1;
    } else {
        ep.total_failed += 1;
    }
    ep.last_status_code = Some(status_code);
    ep.last_delivered_at = Some(now_str.clone());
    ep.updated_at = now_str.clone();

    match sqlx::query(
        "UPDATE webhook_endpoints SET total_delivered = $1, total_failed = $2, last_status_code = $3, last_delivered_at = $4, updated_at = $5 WHERE id = $6",
    )
    .bind(ep.total_delivered)
    .bind(ep.total_failed)
    .bind(ep.last_status_code)
    .bind(&ep.last_delivered_at)
    .bind(&ep.updated_at)
    .bind(&ep.id)
    .execute(&state.db)
    .await
    {
        Err(err) => warnings.push(format!("The endpoint counters could not be stored: {err}")),
        Ok(res) if res.rows_affected() == 0 => {
            warnings.push("The endpoint counters were not stored (0 rows affected).".to_string())
        }
        Ok(_) => {}
    }

    Json(json!({
        "success": success,
        "original_delivery_id": id,
        "new_delivery_id": new_delivery_id,
        "endpoint_id": ep.id,
        "url": ep.url,
        "status_code": status_code,
        "duration_ms": duration_ms,
        "response_body": response_body,
        "response_headers": response_headers,
        "attempt": new_attempt,
        "signed": signature.is_some(),
        "error": error_message,
        "warnings": warnings
    }))
    .into_response()
}

// ===========================================================================
// Stripe Billing Integration Endpoints
//
// These talk to https://api.stripe.com directly. There is no simulator: a
// missing or masked secret key is an honest 503, and every field in a success
// response is read from Stripe's own JSON.
// ===========================================================================

const STRIPE_TIMEOUT_SECS: u64 = 10;

/// A real Stripe REST outcome.
enum StripeOutcome {
    Ok(serde_json::Value),
    /// `live` is false when we never reached Stripe (no key, transport error).
    Failed {
        status: StatusCode,
        message: String,
        live: bool,
        provider_status: Option<u16>,
    },
}

/// A Stripe unix timestamp rendered as RFC3339; absent or unparseable is null.
fn stripe_timestamp(raw: Option<i64>) -> Option<String> {
    raw.and_then(|ts| Utc.timestamp_opt(ts, 0).single())
        .map(|dt| dt.to_rfc3339())
}

/// Money only ever comes from Stripe's own payload; unknown is null, not zero.
fn cents_to_string(raw: Option<i64>) -> Option<String> {
    raw.map(|cents| format!("${:.2}", cents as f64 / 100.0))
}

async fn stripe_request(
    state: &AppState,
    method: &str,
    path: &str,
    query: &[(&str, String)],
    form: &[(&str, String)],
) -> StripeOutcome {
    let cfg = match load_stripe_config(&state.db).await {
        Err(err) => {
            return StripeOutcome::Failed {
                status: StatusCode::INTERNAL_SERVER_ERROR,
                message: format!("Could not read the Stripe configuration from the database: {err}"),
                live: false,
                provider_status: None,
            }
        }
        Ok(None) => {
            return StripeOutcome::Failed {
                status: StatusCode::SERVICE_UNAVAILABLE,
                message: "No Stripe account is configured: stripe_configs has no row, so there is no secret key to call api.stripe.com with. Save the settings at /api/v1/integrations/stripe/config first.".to_string(),
                live: false,
                provider_status: None,
            }
        }
        Ok(Some(row)) => row,
    };
    let api_key = cfg.api_key.trim().to_string();

    if api_key.is_empty() || is_masked(&api_key) {
        return StripeOutcome::Failed {
            status: StatusCode::SERVICE_UNAVAILABLE,
            message: if is_masked(&api_key) {
                "The stored Stripe secret key is masked (display-only). Enter the real sk_ key and save it before using Stripe billing.".to_string()
            } else {
                "The stored Stripe account has an empty api_key. Save a live or test sk_ key before using Stripe billing.".to_string()
            },
            live: false,
            provider_status: None,
        };
    }

    let url = format!("https://api.stripe.com{path}");
    let builder = match method {
        "GET" => state.http_client.get(&url),
        "POST" => state.http_client.post(&url),
        "DELETE" => state.http_client.delete(&url),
        _ => {
            return StripeOutcome::Failed {
                status: StatusCode::INTERNAL_SERVER_ERROR,
                message: format!("Unsupported internal Stripe method {method}."),
                live: false,
                provider_status: None,
            }
        }
    };
    let builder = builder
        .timeout(std::time::Duration::from_secs(STRIPE_TIMEOUT_SECS))
        .basic_auth(api_key, None::<&str>)
        .header("User-Agent", "Communicator-Backend-RS/1.0");
    let builder = if !query.is_empty() {
        builder.query(query)
    } else {
        builder
    };
    let builder = if !form.is_empty() {
        builder.form(
            &form
                .iter()
                .map(|(k, v)| (*k, v.as_str()))
                .collect::<Vec<(&str, &str)>>(),
        )
    } else {
        builder
    };

    match builder.send().await {
        Err(err) => StripeOutcome::Failed {
            status: StatusCode::BAD_GATEWAY,
            message: format!("Could not reach api.stripe.com: {err}"),
            live: false,
            provider_status: None,
        },
        Ok(resp) => {
            let status = resp.status();
            let body: serde_json::Value = resp.json().await.unwrap_or_else(|_| json!({}));
            if status.is_success() {
                StripeOutcome::Ok(body)
            } else {
                let message = body
                    .pointer("/error/message")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Stripe rejected the request")
                    .to_string();
                let code = body
                    .pointer("/error/code")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let mapped = match status.as_u16() {
                    401 | 403 => StatusCode::UNAUTHORIZED,
                    404 => StatusCode::NOT_FOUND,
                    429 => StatusCode::TOO_MANY_REQUESTS,
                    _ => StatusCode::BAD_GATEWAY,
                };
                StripeOutcome::Failed {
                    status: mapped,
                    message: if code.is_empty() {
                        format!("Stripe (HTTP {}): {message}", status.as_u16())
                    } else {
                        format!("Stripe (HTTP {}, {code}): {message}", status.as_u16())
                    },
                    live: true,
                    provider_status: Some(status.as_u16()),
                }
            }
        }
    }
}

fn stripe_failure(outcome: &StripeOutcome) -> Option<(StatusCode, serde_json::Value)> {
    match outcome {
        StripeOutcome::Ok(_) => None,
        StripeOutcome::Failed {
            status,
            message,
            live,
            provider_status,
        } => Some((
            *status,
            json!({
                "success": false,
                "error": message,
                "detail": message,
                "live_call_made": live,
                "stripe_status": provider_status,
            }),
        )),
    }
}

/// Resolves an email / cus_ / contact id to a real Stripe customer id.
async fn resolve_stripe_customer(state: &AppState, identifier: &str) -> Result<String, (StatusCode, serde_json::Value)> {
    if identifier.starts_with("cus_") {
        return Ok(identifier.to_string());
    }

    let mut email = identifier.to_string();
    if !email.contains('@') {
        // Treat it as a local contact id and use that contact's real address.
        let row: Option<(Option<String>, Option<String>)> = match sqlx::query_as(
            "SELECT value, person_name FROM contacts WHERE id = $1 OR value = $2 LIMIT 1",
        )
        .bind(identifier)
        .bind(identifier)
        .fetch_optional(&state.db)
        .await
        {
            Ok(row) => row,
            Err(err) => {
                return Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    json!({
                        "success": false,
                        "error": format!("Could not read contact '{identifier}' from the database: {err}"),
                        "databaseError": true,
                        "live_call_made": false
                    }),
                ));
            }
        };
        match row {
            Some((Some(value), _)) if value.contains('@') => email = value,
            _ => {
                return Err((
                    StatusCode::NOT_FOUND,
                    json!({
                        "success": false,
                        "error": format!("'{identifier}' is neither a Stripe customer id (cus_...) nor a contact with an email address, so no Stripe lookup was possible."),
                        "live_call_made": false
                    }),
                ));
            }
        }
    }

    let outcome = stripe_request(
        state,
        "GET",
        "/v1/customers",
        &[("email", email.clone()), ("limit", "1".to_string())],
        &[],
    )
    .await;
    match stripe_failure(&outcome) {
        Some(err) => Err(err),
        None => match outcome {
            StripeOutcome::Ok(body) => {
                let id = body
                    .pointer("/data/0/id")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());
                match id {
                    Some(id) => Ok(id),
                    None => Err((
                        StatusCode::NOT_FOUND,
                        json!({
                            "success": false,
                            "error": format!("Stripe has no customer for {email} (searched by email; created nothing)."),
                            "live_call_made": true
                        }),
                    )),
                }
            }
            StripeOutcome::Failed { .. } => unreachable!(),
        },
    }
}

pub async fn get_stripe_billing(
    State(state): State<AppState>,
    Path(identifier): Path<String>,
) -> impl IntoResponse {
    // Every value in a success body is Stripe's own; the stored row only
    // supplies test_mode, and it is reported exactly as saved.
    let cfg = match load_stripe_config(&state.db).await {
        Err(err) => return db_error("Could not read the Stripe configuration", err),
        Ok(None) => {
            let mut body = stripe_not_configured("a customer billing profile could not be read");
            if let Some(obj) = body.as_object_mut() {
                obj.insert("contact_id".to_string(), json!(identifier));
                for key in [
                    "customer_id",
                    "name",
                    "email",
                    "currency",
                    "delinquent",
                    "has_active_subscription",
                    "subscription",
                    "mrr",
                    "mrr_formatted",
                    "lifetime_paid_cents",
                    "lifetime_paid_formatted",
                    "payment_method",
                    "test_mode",
                ] {
                    obj.insert(key.to_string(), serde_json::Value::Null);
                }
            }
            return (StatusCode::SERVICE_UNAVAILABLE, Json(body)).into_response();
        }
        Ok(Some(row)) => row,
    };

    let mut warnings: Vec<String> = Vec::new();

    let customer_id = match resolve_stripe_customer(&state, &identifier).await {
        Ok(id) => id,
        Err((status, body)) => return (status, Json(body)).into_response(),
    };

    let customer_call = stripe_request(&state, "GET", &format!("/v1/customers/{customer_id}"), &[], &[]).await;
    if let Some((status, body)) = stripe_failure(&customer_call) {
        return (status, Json(body)).into_response();
    }
    let customer = match customer_call {
        StripeOutcome::Ok(v) => v,
        _ => unreachable!(),
    };

    let subs_call = stripe_request(
        &state,
        "GET",
        "/v1/subscriptions",
        &[("customer", customer_id.clone()), ("status", "all".to_string())],
        &[],
    )
    .await;
    let subscriptions: Option<serde_json::Value> = match subs_call {
        StripeOutcome::Ok(v) => match v.get("data") {
            Some(data) => Some(data.clone()),
            None => {
                warnings.push("Stripe's subscription list response carried no `data` array.".to_string());
                None
            }
        },
        StripeOutcome::Failed { message, .. } => {
            warnings.push(format!("Subscriptions were not read: {message}"));
            None
        }
    };

    let list = subscriptions.as_ref().and_then(|s| s.as_array());
    let active = list
        .and_then(|list| {
            list.iter()
                .find(|s| s.get("status").and_then(|v| v.as_str()) == Some("active"))
                .cloned()
        })
        .or_else(|| list.and_then(|l| l.first().cloned()));

    let pm_call = stripe_request(
        &state,
        "GET",
        "/v1/payment_methods",
        &[("customer", customer_id.clone()), ("type", "card".to_string())],
        &[],
    )
    .await;
    let payment_method = match pm_call {
        StripeOutcome::Ok(v) => v.get("data").and_then(|d| d.as_array()).and_then(|a| a.first().cloned()),
        StripeOutcome::Failed { message, .. } => {
            warnings.push(format!("Payment methods were not read: {message}"));
            None
        }
    };

    let inv_call = stripe_request(
        &state,
        "GET",
        "/v1/invoices",
        &[("customer", customer_id.clone()), ("limit", "24".to_string())],
        &[],
    )
    .await;
    let lifetime_paid: Option<i64> = match inv_call {
        StripeOutcome::Ok(v) => v
            .get("data")
            .and_then(|d| d.as_array())
            .map(|list| {
                list.iter()
                    .filter(|i| i.get("status").and_then(|s| s.as_str()) == Some("paid"))
                    .filter_map(|i| i.get("amount_paid").and_then(|v| v.as_i64()))
                    .sum()
            }),
        StripeOutcome::Failed { message, .. } => {
            warnings.push(format!("Invoices were not read, so lifetime paid is unknown: {message}"));
            None
        }
    };

    let sub_json = active.clone().unwrap_or(serde_json::Value::Null);
    let price = sub_json.pointer("/items/data/0/price");
    let amount_cents = price.and_then(|p| p.get("unit_amount")).and_then(|v| v.as_i64());
    let currency = price
        .and_then(|p| p.get("currency"))
        .and_then(|v| v.as_str())
        .or_else(|| customer.get("currency").and_then(|v| v.as_str()))
        .map(|s| s.to_uppercase());
    let mrr = amount_cents.map(|cents| cents as f64 / 100.0);

    Json(json!({
        "contact_id": identifier,
        "configured": true,
        "customer_id": customer.get("id").and_then(|v| v.as_str()),
        "name": customer.get("name").and_then(|v| v.as_str()),
        "email": customer.get("email").and_then(|v| v.as_str()),
        "currency": currency,
        "delinquent": customer.get("delinquent").and_then(|v| v.as_bool()),
        "has_active_subscription": list.map(|subs| subs.iter().any(|s| s.get("status").and_then(|v| v.as_str()) == Some("active"))),
        "subscription": if sub_json.is_null() { serde_json::Value::Null } else { json!({
            "id": sub_json.get("id").and_then(|v| v.as_str()),
            "status": sub_json.get("status").and_then(|v| v.as_str()),
            "plan_name": price.and_then(|p| p.pointer("/product/name")).and_then(|v| v.as_str()).or_else(|| price.and_then(|p| p.get("nickname")).and_then(|v| v.as_str())),
            "amount_cents": amount_cents,
            "amount_formatted": cents_to_string(amount_cents),
            "interval": price.and_then(|p| p.pointer("/recurring/interval")).and_then(|v| v.as_str()),
            "current_period_end": stripe_timestamp(sub_json.get("current_period_end").and_then(|v| v.as_i64())),
            "cancel_at_period_end": sub_json.get("cancel_at_period_end").and_then(|v| v.as_bool()),
            "created_at": stripe_timestamp(sub_json.get("created").and_then(|v| v.as_i64()))
        })},
        "mrr": mrr,
        "mrr_formatted": mrr.map(|value| format!("${value:.2}/mo")),
        "lifetime_paid_cents": lifetime_paid,
        "lifetime_paid_formatted": cents_to_string(lifetime_paid),
        "payment_method": payment_method.map(|pm| json!({
            "id": pm.get("id").and_then(|v| v.as_str()),
            "brand": pm.pointer("/card/brand").and_then(|v| v.as_str()),
            "last4": pm.pointer("/card/last4").and_then(|v| v.as_str()),
            "exp_month": pm.pointer("/card/exp_month").and_then(|v| v.as_i64()),
            "exp_year": pm.pointer("/card/exp_year").and_then(|v| v.as_i64())
        })),
        "warnings": warnings,
        "live_call_made": true,
        "test_mode": cfg.test_mode
    }))
    .into_response()
}

pub async fn get_stripe_invoices(
    State(state): State<AppState>,
    Path(identifier): Path<String>,
    Query(query): Query<LimitQuery>,
) -> impl IntoResponse {
    let limit = query.limit.unwrap_or(10).clamp(1, 50);

    let customer_id = match resolve_stripe_customer(&state, &identifier).await {
        Ok(id) => id,
        Err((status, body)) => return (status, Json(body)).into_response(),
    };

    let call = stripe_request(
        &state,
        "GET",
        "/v1/invoices",
        &[
            ("customer", customer_id),
            ("limit", limit.to_string()),
            ("sort_by", "date".to_string()),
            ("status", "paid".to_string()),
        ],
        &[],
    )
    .await;

    if let Some((status, body)) = stripe_failure(&call) {
        return (status, Json(body)).into_response();
    }

    let list = match &call {
        StripeOutcome::Ok(body) => body.get("data").and_then(|d| d.as_array()).cloned(),
        StripeOutcome::Failed { .. } => None,
    };
    let Some(list) = list else {
        return (
            StatusCode::BAD_GATEWAY,
            Json(json!({
                "success": false,
                "error": "Stripe's invoice response carried no `data` array, so no invoices are reported.",
                "live_call_made": true
            })),
        )
            .into_response();
    };

    let invoices: Vec<serde_json::Value> = list
        .iter()
        .map(|inv| {
            let amount_paid = inv.get("amount_paid").and_then(|v| v.as_i64());
            let ts = |key: &str| stripe_timestamp(inv.get(key).and_then(|v| v.as_i64()));
            let refunded_cents = inv.get("amount_refunded").and_then(|v| v.as_i64());
            json!({
                "id": inv.get("id").and_then(|v| v.as_str()),
                "number": inv.get("number").and_then(|v| v.as_str()),
                "amount_paid_cents": amount_paid,
                "amount_paid_formatted": cents_to_string(amount_paid),
                "amount_due_cents": inv.get("amount_due").and_then(|v| v.as_i64()),
                "currency": inv.get("currency").and_then(|v| v.as_str()),
                "status": inv.get("status").and_then(|v| v.as_str()),
                "created_at": ts("created"),
                "period_start": ts("period_start"),
                "period_end": ts("period_end"),
                "hosted_invoice_url": inv.get("hosted_invoice_url").and_then(|v| v.as_str()),
                "invoice_pdf_url": inv.get("invoice_pdf").and_then(|v| v.as_str()),
                "refundable_amount_cents": amount_paid,
                "refunded": refunded_cents.map(|cents| cents > 0),
                "charge_id": inv.get("payment_intent").and_then(|v| v.as_str()),
                "live_call_made": true
            })
        })
        .collect();

    Json(invoices).into_response()
}

pub async fn process_stripe_refund(
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    let cfg = match require_stripe_config(&state.db, "a refund").await {
        Ok(row) => row,
        Err(response) => return response,
    };

    // A refund moves real money: the charge/payment-intent id must be supplied,
    // we never default to a test charge.
    let charge_id = match payload["charge_or_invoice_id"].as_str().map(str::trim) {
        Some(v) if !v.is_empty() && !is_masked(v) => v.to_string(),
        _ => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "success": false,
                    "error": "charge_or_invoice_id is required: a refund cannot be issued against a placeholder charge.",
                    "live_call_made": false
                })),
            )
                .into_response();
        }
    };
    let Some(amount_cents) = payload["amount_cents"].as_i64().filter(|v| *v > 0) else {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "success": false,
                "error": "amount_cents must be a positive number; nothing was refunded.",
                "live_call_made": false
            })),
        )
            .into_response();
    };
    // Only send a reason Stripe was actually given one; never assume the customer asked.
    let reason = payload["reason"]
        .as_str()
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(|s| s.to_string());
    let actor_email = payload["actor_email"]
        .as_str()
        .map(str::trim)
        .filter(|v| !v.is_empty() && !is_masked(v))
        .map(|s| s.to_string());
    let contact_id = payload["contact_id"].as_str().map(|s| s.to_string());
    let customer_identifier = payload["customer_identifier"]
        .as_str()
        .unwrap_or(&charge_id)
        .to_string();

    let param_name = if charge_id.starts_with("ch_") {
        "charge"
    } else if charge_id.starts_with("pi_") {
        "payment_intent"
    } else {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "success": false,
                "error": format!("'{charge_id}' is not a Stripe charge (ch_) or payment intent (pi_), so no refund was attempted."),
                "live_call_made": false
            })),
        )
            .into_response();
    };

    let mut form: Vec<(&str, String)> = vec![
        (param_name, charge_id.clone()),
        ("amount", amount_cents.to_string()),
    ];
    if let Some(ref reason) = reason {
        form.push(("reason", reason.clone()));
    }

    let call = stripe_request(&state, "POST", "/v1/refunds", &[], &form).await;

    let log_id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();
    let mut warnings: Vec<String> = Vec::new();

    let refund: serde_json::Value = match &call {
        StripeOutcome::Ok(v) => v.clone(),
        StripeOutcome::Failed { message, live, .. } => {
            // Log the attempt and its real failure, then surface the error.
            let mut audit_warnings: Vec<String> = Vec::new();
            let details = match serde_json::to_string(&json!({
                "charge_or_invoice_id": charge_id,
                "requested_amount_cents": amount_cents,
                "reason": reason,
                "error": message,
                "live_call_made": live,
                "mode": if cfg.test_mode { "sandbox" } else { "live" }
            })) {
                Ok(text) => text,
                Err(err) => {
                    // An audit row without details is still an audit row; the
                    // loss is reported, not hidden behind a fake "{}".
                    audit_warnings.push(format!(
                        "The failed-refund audit details could not be serialized: {err}"
                    ));
                    String::new()
                }
            };
            let stored_actor = match actor_email.as_deref() {
                Some(v) => v.to_string(),
                None => String::new(),
            };
            let stored_reason = match reason.as_deref() {
                Some(v) => v.to_string(),
                None => String::new(),
            };
            let written = sqlx::query(
                r#"
                INSERT INTO stripe_billing_action_logs (id, contact_id, customer_identifier, action_type, amount_cents, currency, status, stripe_id, actor_email, reason, details, created_at)
                VALUES ($1, $2, $3, 'refund', $4, $5, 'failed', '', $6, $7, $8, $9)
                "#,
            )
            .bind(&log_id)
            .bind(&contact_id)
            .bind(&customer_identifier)
            .bind(amount_cents)
            .bind(&cfg.default_currency)
            .bind(&stored_actor)
            .bind(&stored_reason)
            .bind(&details)
            .bind(&now)
            .execute(&state.db)
            .await;
            match written {
                Err(err) => audit_warnings.push(format!(
                    "The failed-refund audit row could not be stored: {err}"
                )),
                Ok(res) if res.rows_affected() == 0 => audit_warnings.push(
                    "The failed-refund audit row was not stored (0 rows affected).".to_string(),
                ),
                Ok(_) => {}
            }
            if actor_email.is_none() {
                audit_warnings.push("The audit row stores an empty actor_email: none was supplied.".to_string());
            }

            let status = match &call {
                StripeOutcome::Failed { status, .. } => *status,
                _ => StatusCode::BAD_GATEWAY,
            };
            return (
                status,
                Json(json!({
                    "success": false,
                    "error": message,
                    "detail": message,
                    "live_call_made": live,
                    "log_id": log_id,
                    "mode": if cfg.test_mode { "test" } else { "live" },
                    "warnings": audit_warnings
                })),
            )
                .into_response();
        }
    };

    // Everything below is Stripe's own refund object; a field they did not send
    // is reported as null plus a warning, never filled in here.
    let refund_id = refund.get("id").and_then(|v| v.as_str()).map(|s| s.to_string());
    let status = refund.get("status").and_then(|v| v.as_str()).map(|s| s.to_string());
    let refunded_amount = refund.get("amount").and_then(|v| v.as_i64());
    let currency = refund.get("currency").and_then(|v| v.as_str()).map(|s| s.to_string());
    let stripe_reason = refund.get("reason").and_then(|v| v.as_str()).map(|s| s.to_string());
    if refund_id.is_none() {
        warnings.push("Stripe's refund response carried no `id`.".to_string());
    }
    if status.is_none() {
        warnings.push("Stripe's refund response carried no `status`.".to_string());
    }
    if refunded_amount.is_none() {
        warnings.push("Stripe's refund response carried no `amount`.".to_string());
    }
    if currency.is_none() {
        warnings.push("Stripe's refund response carried no `currency`.".to_string());
    }

    // The audit columns are NOT NULL, so a value Stripe did not report is stored
    // as "" / 0 / "unknown" and named in `audit_stored` plus a warning.
    let stored_amount = match refunded_amount {
        Some(v) => v,
        None => {
            warnings.push("The audit row stores amount_cents 0 because Stripe's response had no amount.".to_string());
            0
        }
    };
    let stored_currency = match currency.as_deref() {
        Some(v) => v.to_string(),
        None => {
            warnings.push("The audit row stores an empty currency because Stripe's response had none.".to_string());
            String::new()
        }
    };
    let stored_status = match status.as_deref() {
        Some(v) => v.to_string(),
        None => {
            warnings.push("The audit row stores status 'unknown' because Stripe's response had no status.".to_string());
            "unknown".to_string()
        }
    };
    let stored_stripe_id = match refund_id.as_deref() {
        Some(v) => v.to_string(),
        None => {
            warnings.push("The audit row stores an empty stripe_id because Stripe returned no refund id.".to_string());
            String::new()
        }
    };
    let stored_actor = actor_email.clone().unwrap_or_default();
    let stored_reason = reason
        .clone()
        .or_else(|| stripe_reason.clone())
        .unwrap_or_default();

    let details_value = json!({
        "charge_or_invoice_id": charge_id,
        "refund_id": refund_id,
        "amount_cents": refunded_amount,
        "reason": reason,
        "stripe_refund": refund,
        "mode": if cfg.test_mode { "sandbox" } else { "live" }
    });
    let details_str = match serde_json::to_string(&details_value) {
        Ok(text) => text,
        Err(err) => {
            warnings.push(format!("The refund audit details could not be serialized: {err}"));
            String::new()
        }
    };

    let written = sqlx::query(
        r#"
        INSERT INTO stripe_billing_action_logs (id, contact_id, customer_identifier, action_type, amount_cents, currency, status, stripe_id, actor_email, reason, details, created_at)
        VALUES ($1, $2, $3, 'refund', $4, $5, $6, $7, $8, $9, $10, $11)
        "#,
    )
    .bind(&log_id)
    .bind(&contact_id)
    .bind(&customer_identifier)
    .bind(stored_amount)
    .bind(&stored_currency)
    .bind(&stored_status)
    .bind(&stored_stripe_id)
    .bind(&stored_actor)
    .bind(&stored_reason)
    .bind(&details_str)
    .bind(&now)
    .execute(&state.db)
    .await;
    match written {
        Err(err) => warnings.push(format!("The refund audit row could not be stored: {err}")),
        Ok(res) if res.rows_affected() == 0 => {
            warnings.push("The refund audit row was not stored (0 rows affected).".to_string())
        }
        Ok(_) => {}
    }

    Json(json!({
        "id": refund_id,
        "charge_id": refund.get("charge").and_then(|v| v.as_str()),
        "requested_charge_or_invoice_id": charge_id,
        "amount_cents": refunded_amount,
        "amount_formatted": cents_to_string(refunded_amount),
        "currency": currency,
        "status": status,
        "reason": reason.or(stripe_reason),
        "actor_email": actor_email,
        "created_at": now,
        "log_id": log_id,
        "audit_stored": {
            "amount_cents": stored_amount,
            "currency": stored_currency,
            "status": stored_status,
            "stripe_id": stored_stripe_id,
            "actor_email": stored_actor,
            "reason": stored_reason,
            "note": "NOT NULL columns: empty string / 0 / 'unknown' means Stripe did not report that field."
        },
        "mode": if cfg.test_mode { "test" } else { "live" },
        "warnings": warnings,
        "live_call_made": true
    }))
    .into_response()
}

pub async fn create_stripe_portal_session(
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    let cfg = match require_stripe_config(&state.db, "a billing portal session").await {
        Ok(row) => row,
        Err(response) => return response,
    };

    let identifier = match payload["identifier"].as_str().map(str::trim) {
        Some(v) if !v.is_empty() => v.to_string(),
        _ => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "success": false,
                    "error": "identifier (a Stripe customer id or an email with a matching customer) is required.",
                    "live_call_made": false
                })),
            )
                .into_response();
        }
    };
    // The return URL must be a real absolute URL: Stripe rejects relative ones
    // and a fabricated default would hand the customer a dead link.
    let return_url = match payload["return_url"].as_str().map(str::trim) {
        Some(v) if v.starts_with("http://") || v.starts_with("https://") => v.to_string(),
        _ => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "success": false,
                    "error": "return_url must be an absolute http(s) URL the customer is sent back to.",
                    "live_call_made": false
                })),
            )
                .into_response();
        }
    };
    let actor_email = payload["actor_email"]
        .as_str()
        .map(str::trim)
        .filter(|v| !v.is_empty() && !is_masked(v))
        .map(|s| s.to_string());

    let customer_id = match resolve_stripe_customer(&state, &identifier).await {
        Ok(id) => id,
        Err((status, body)) => return (status, Json(body)).into_response(),
    };

    let call = stripe_request(
        &state,
        "POST",
        "/v1/billing_portal/sessions",
        &[],
        &[("customer", customer_id.clone()), ("return_url", return_url.clone())],
    )
    .await;

    let log_id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();
    let mut warnings: Vec<String> = Vec::new();

    let body = match &call {
        StripeOutcome::Ok(v) => v.clone(),
        StripeOutcome::Failed { message, live, .. } => {
            let mut audit_warnings: Vec<String> = Vec::new();
            let details = match serde_json::to_string(&json!({
                "customer_id": customer_id,
                "error": message,
                "live_call_made": live,
                "mode": if cfg.test_mode { "sandbox" } else { "live" }
            })) {
                Ok(text) => text,
                Err(err) => {
                    audit_warnings.push(format!(
                        "The failed portal-session audit details could not be serialized: {err}"
                    ));
                    String::new()
                }
            };
            let stored_actor = match actor_email.as_deref() {
                Some(v) => v.to_string(),
                None => String::new(),
            };
            if actor_email.is_none() {
                audit_warnings
                    .push("The audit row stores an empty actor_email: none was supplied.".to_string());
            }
            let written = sqlx::query(
                r#"
                INSERT INTO stripe_billing_action_logs (id, contact_id, customer_identifier, action_type, amount_cents, currency, status, stripe_id, actor_email, reason, details, created_at)
                VALUES ($1, NULL, $2, 'portal_session', 0, $3, 'failed', '', $4, 'agent_generated_portal_link', $5, $6)
                "#,
            )
            .bind(&log_id)
            .bind(&identifier)
            .bind(&cfg.default_currency)
            .bind(&stored_actor)
            .bind(&details)
            .bind(&now)
            .execute(&state.db)
            .await;
            match written {
                Err(err) => audit_warnings.push(format!(
                    "The failed portal-session audit row could not be stored: {err}"
                )),
                Ok(res) if res.rows_affected() == 0 => audit_warnings.push(
                    "The failed portal-session audit row was not stored (0 rows affected).".to_string(),
                ),
                Ok(_) => {}
            }

            let status = match &call {
                StripeOutcome::Failed { status, .. } => *status,
                _ => StatusCode::BAD_GATEWAY,
            };
            // A customer with no portal configuration is Stripe's own 400; the
            // real message tells us to create one in the dashboard.
            return (
                status,
                Json(json!({
                    "success": false,
                    "error": message,
                    "detail": message,
                    "live_call_made": live,
                    "log_id": log_id,
                    "mode": if cfg.test_mode { "test" } else { "live" },
                    "warnings": audit_warnings
                })),
            )
                .into_response();
        }
    };

    let session_id = body.get("id").and_then(|v| v.as_str()).map(|s| s.to_string());
    let portal_url = body.get("url").and_then(|v| v.as_str()).map(|s| s.to_string());
    if session_id.is_none() {
        warnings.push("Stripe's portal session response carried no `id`.".to_string());
    }
    if portal_url.is_none() {
        warnings.push("Stripe's portal session response carried no `url`, so there is no link to hand out.".to_string());
    }

    let details_str = match serde_json::to_string(&json!({
        "session_id": session_id,
        "portal_url": portal_url,
        "return_url": return_url,
        "customer_id": customer_id
    })) {
        Ok(text) => text,
        Err(err) => {
            warnings.push(format!("The portal-session audit details could not be serialized: {err}"));
            String::new()
        }
    };
    let stored_stripe_id = match session_id.as_deref() {
        Some(v) => v.to_string(),
        None => {
            warnings.push("The audit row stores an empty stripe_id: Stripe returned no session id.".to_string());
            String::new()
        }
    };
    let stored_actor = match actor_email.as_deref() {
        Some(v) => v.to_string(),
        None => String::new(),
    };
    if actor_email.is_none() {
        warnings.push("The audit row stores an empty actor_email: none was supplied.".to_string());
    }

    let written = sqlx::query(
        r#"
        INSERT INTO stripe_billing_action_logs (id, contact_id, customer_identifier, action_type, amount_cents, currency, status, stripe_id, actor_email, reason, details, created_at)
        VALUES ($1, NULL, $2, 'portal_session', 0, $3, $4, $5, $6, 'agent_generated_portal_link', $7, $8)
        "#,
    )
    .bind(&log_id)
    .bind(&identifier)
    .bind(&cfg.default_currency)
    .bind(if portal_url.is_some() && session_id.is_some() { "succeeded" } else { "incomplete" })
    .bind(&stored_stripe_id)
    .bind(&stored_actor)
    .bind(&details_str)
    .bind(&now)
    .execute(&state.db)
    .await;
    match written {
        Err(err) => warnings.push(format!("The portal-session audit row could not be stored: {err}")),
        Ok(res) if res.rows_affected() == 0 => {
            warnings.push("The portal-session audit row was not stored (0 rows affected).".to_string())
        }
        Ok(_) => {}
    }

    Json(json!({
        "session_id": session_id,
        "url": portal_url,
        "customer_id": customer_id,
        "customer_identifier": identifier,
        "actor_email": actor_email,
        "created_at": now,
        "audit_stored": {
            "stripe_id": stored_stripe_id,
            "actor_email": stored_actor,
            "note": "NOT NULL columns: an empty string means that value was never reported."
        },
        "mode": if cfg.test_mode { "test" } else { "live" },
        "warnings": warnings,
        "live_call_made": true
    }))
    .into_response()
}

pub async fn get_stripe_billing_logs(
    State(state): State<AppState>,
    Path(identifier): Path<String>,
    Query(query): Query<LimitQuery>,
) -> impl IntoResponse {
    let limit = query.limit.unwrap_or(25).clamp(1, 100);

    let rows = match sqlx::query_as::<_, StripeBillingActionLogDbRow>(
        r#"
        SELECT id, contact_id, customer_identifier, action_type, amount_cents, currency, status, stripe_id, actor_email, reason, details, created_at
        FROM stripe_billing_action_logs
        WHERE customer_identifier = $1 OR contact_id = $2
        ORDER BY created_at DESC
        LIMIT $3
        "#,
    )
    .bind(&identifier)
    .bind(&identifier)
    .bind(limit)
    .fetch_all(&state.db)
    .await
    {
        Ok(rows) => rows,
        Err(err) => return db_error("Could not read the Stripe billing action logs", err),
    };

    let formatted: Vec<serde_json::Value> = rows.into_iter().map(format_billing_log).collect();
    Json(formatted).into_response()
}

pub async fn get_stripe_config(State(state): State<AppState>) -> impl IntoResponse {
    match load_stripe_config(&state.db).await {
        Err(err) => db_error("Could not read the Stripe configuration", err),
        // No row means Stripe is not configured: report that, invent nothing.
        Ok(None) => (
            StatusCode::OK,
            Json(json!({
                "id": null,
                "account_name": null,
                "api_key_masked": null,
                "publishable_key": null,
                "webhook_secret_masked": null,
                "has_api_key": false,
                "has_webhook_secret": false,
                "is_enabled": null,
                "test_mode": null,
                "default_currency": null,
                "last_tested_at": null,
                "last_test_status": null,
                "created_at": null,
                "updated_at": null,
                "configured": false,
                "note": "No Stripe account is saved. POST /api/v1/integrations/stripe/config with account_name, api_key, publishable_key, webhook_secret, default_currency, test_mode and is_enabled to configure it; no billing endpoint works until then."
            })),
        )
            .into_response(),
        Ok(Some(row)) => {
            let mut body = format_stripe_config(row);
            if let Some(obj) = body.as_object_mut() {
                obj.insert("configured".to_string(), json!(true));
            }
            Json(body).into_response()
        }
    }
}

/// Only this endpoint may create the Stripe row, because only here has the
/// operator explicitly asked for it.
pub async fn update_stripe_config(
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    let existing = match load_stripe_config(&state.db).await {
        Err(err) => return db_error("Could not read the Stripe configuration", err),
        Ok(row) => row,
    };
    let is_new_row = existing.is_none();

    // A masked value coming back from the UI is a display artifact: keep the
    // stored secret and say so instead of overwriting it with bullets.
    let mut ignored_masked: Vec<&str> = Vec::new();
    let mut defaults_applied: Vec<String> = Vec::new();

    let incoming_key = match payload["api_key"].as_str() {
        Some(k) if k.trim().is_empty() || is_masked(k) => {
            ignored_masked.push("api_key");
            None
        }
        Some(k) => Some(k.trim().to_string()),
        None => None,
    };
    let incoming_webhook_secret = match payload["webhook_secret"].as_str() {
        Some(s) if s.trim().is_empty() || is_masked(s) => {
            ignored_masked.push("webhook_secret");
            None
        }
        Some(s) => Some(s.trim().to_string()),
        None => None,
    };

    let now = Utc::now().to_rfc3339();
    let (row_id, account_name, api_key, publishable_key, webhook_secret, is_enabled, test_mode, default_currency) =
        match existing {
            Some(cfg) => (
                cfg.id.clone(),
                pick_str(&payload, "account_name", &cfg.account_name),
                stored_or_new(incoming_key.clone(), &cfg.api_key),
                pick_str(&payload, "publishable_key", &cfg.publishable_key),
                stored_or_new(incoming_webhook_secret.clone(), &cfg.webhook_secret),
                pick_bool(&payload, "is_enabled", cfg.is_enabled),
                pick_bool(&payload, "test_mode", cfg.test_mode),
                pick_str(&payload, "default_currency", &cfg.default_currency),
            ),
            None => {
                // Creating the row: every column comes from the payload, and what
                // the payload did not say is recorded instead of quietly invented.
                let key = stored_or_new(incoming_key.clone(), "");
                let test_mode = match payload.get("test_mode").and_then(|v| v.as_bool()) {
                    Some(value) => value,
                    None => {
                        let derived = if key.starts_with("sk_test_") {
                            true
                        } else if key.starts_with("sk_live_") {
                            false
                        } else {
                            true
                        };
                        defaults_applied.push(format!(
                            "test_mode was not sent; stored as {derived} ({}).",
                            if derived {
                                "sandbox, because no sk_live_ key was saved"
                            } else {
                                "live, because the saved key is an sk_live_ key"
                            }
                        ));
                        derived
                    }
                };
                let is_enabled = match payload.get("is_enabled").and_then(|v| v.as_bool()) {
                    Some(value) => value,
                    None => {
                        defaults_applied
                            .push("is_enabled was not sent; stored as false (Stripe stays off until you enable it).".to_string());
                        false
                    }
                };
                let account_name = match payload.get("account_name").and_then(|v| v.as_str()) {
                    Some(value) => value.to_string(),
                    None => {
                        defaults_applied.push("account_name was not sent; stored empty.".to_string());
                        String::new()
                    }
                };
                let publishable_key = match payload.get("publishable_key").and_then(|v| v.as_str()) {
                    Some(value) => value.trim().to_string(),
                    None => {
                        defaults_applied
                            .push("publishable_key was not sent; stored empty (no browser-side key is configured).".to_string());
                        String::new()
                    }
                };
                let default_currency = match payload.get("default_currency").and_then(|v| v.as_str()) {
                    Some(value) => value.trim().to_ascii_lowercase(),
                    None => {
                        defaults_applied.push("default_currency was not sent; stored empty (unknown).".to_string());
                        String::new()
                    }
                };
                (
                    Uuid::new_v4().to_string(),
                    account_name,
                    key,
                    publishable_key,
                    stored_or_new(incoming_webhook_secret.clone(), ""),
                    is_enabled,
                    test_mode,
                    default_currency,
                )
            }
        };

    let publishable_key = publishable_key.trim().to_string();
    let write = match is_new_row {
        false => {
            sqlx::query(
                r#"
                UPDATE stripe_configs
                SET account_name = $1, api_key = $2, publishable_key = $3, webhook_secret = $4, is_enabled = $5, test_mode = $6, default_currency = $7, updated_at = $8
                WHERE id = $9
                "#,
            )
            .bind(&account_name)
            .bind(&api_key)
            .bind(&publishable_key)
            .bind(&webhook_secret)
            .bind(is_enabled)
            .bind(test_mode)
            .bind(&default_currency)
            .bind(&now)
            .bind(&row_id)
            .execute(&state.db)
            .await
        }
        true => {
            sqlx::query(
                r#"
                INSERT INTO stripe_configs (id, account_name, api_key, publishable_key, webhook_secret, is_enabled, test_mode, default_currency, last_tested_at, last_test_status, created_at, updated_at)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, NULL, 'untested', $9, $10)
                "#,
            )
            .bind(&row_id)
            .bind(&account_name)
            .bind(&api_key)
            .bind(&publishable_key)
            .bind(&webhook_secret)
            .bind(is_enabled)
            .bind(test_mode)
            .bind(&default_currency)
            .bind(&now)
            .bind(&now)
            .execute(&state.db)
            .await
        }
    };

    match write {
        Err(err) => db_error("Stripe settings could not be saved", err),
        Ok(res) if res.rows_affected() == 0 => nothing_written("No Stripe configuration row was written"),
        Ok(_) => {
            // Read back what is actually stored rather than echoing the request.
            match sqlx::query_as::<_, StripeConfigDbRow>(
                "SELECT id, account_name, api_key, publishable_key, webhook_secret, is_enabled, test_mode, default_currency, last_tested_at, last_test_status, created_at, updated_at FROM stripe_configs WHERE id = $1",
            )
            .bind(&row_id)
            .fetch_optional(&state.db)
            .await
            {
                Err(err) => db_error("Stripe settings were saved but could not be read back", err),
                Ok(None) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({
                        "success": false,
                        "saved": true,
                        "error": "The Stripe row was written but is not readable back; the stored values are unconfirmed."
                    })),
                )
                    .into_response(),
                Ok(Some(stored)) => {
                    let mut body = format_stripe_config(stored);
                    if let Some(obj) = body.as_object_mut() {
                        obj.insert("configured".to_string(), json!(true));
                        obj.insert("saved".to_string(), json!(true));
                        if !ignored_masked.is_empty() {
                            obj.insert(
                                "ignored_masked_fields".to_string(),
                                json!(ignored_masked),
                            );
                            obj.insert(
                                "note".to_string(),
                                json!("Masked values in api_key/webhook_secret are display artifacts and were not written; the stored secrets are unchanged."),
                            );
                        }
                        if !defaults_applied.is_empty() {
                            obj.insert("defaults_applied".to_string(), json!(defaults_applied));
                        }
                    }
                    Json(body).into_response()
                }
            }
        }
    }
}

/// Stores the probe outcome on the saved row. A failed write is returned as a
/// warning so the reported verdict and the stored state cannot silently diverge.
async fn record_stripe_test_state(
    pool: &crate::db::DbPool,
    row_id: &str,
    tested_at: &str,
    status: &str,
) -> Option<String> {
    match sqlx::query("UPDATE stripe_configs SET last_tested_at = $1, last_test_status = $2 WHERE id = $3")
        .bind(tested_at)
        .bind(status)
        .bind(row_id)
        .execute(pool)
        .await
    {
        Err(err) => Some(format!("The test result could not be stored: {err}")),
        Ok(res) if res.rows_affected() == 0 => {
            Some("The test result was not stored: no stripe_configs row matched.".to_string())
        }
        Ok(_) => None,
    }
}

pub async fn test_stripe_connection(State(state): State<AppState>) -> impl IntoResponse {
    // A test never creates the row: only saving settings does.
    let cfg = match load_stripe_config(&state.db).await {
        Err(err) => return db_error("Could not read the Stripe configuration", err),
        Ok(None) => {
            let reason = "No Stripe account is configured: stripe_configs has no row, so there is no key to test. Save the settings at /api/v1/integrations/stripe/config first.";
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(json!({
                    "status": "unconfigured",
                    "success": false,
                    "configured": false,
                    "mode": null,
                    "message": reason,
                    "error": reason,
                    "live_call_made": false,
                    "tested_at": null,
                    "note": stripe_not_configured("a Stripe connection test could not be run")["note"]
                })),
            )
                .into_response();
        }
        Ok(Some(row)) => row,
    };
    let now = Utc::now().to_rfc3339();

    if cfg.api_key.starts_with("sk_live") || cfg.api_key.starts_with("sk_test") {
        match state.http_client
            .get("https://api.stripe.com/v1/balance")
            .basic_auth(&cfg.api_key, Some(""))
            .timeout(std::time::Duration::from_secs(5))
            .send()
            .await
        {
            Ok(resp) => {
                let code = resp.status();
                let body: serde_json::Value = resp.json().await.unwrap_or_else(|_| json!({}));
                if code.is_success() {
                    let warning =
                        record_stripe_test_state(&state.db, &cfg.id, &now, "connected").await;

                    // Stripe's own /v1/balance payload: what the account really
                    // holds, not an assertion that the connection "works".
                    return Json(json!({
                        "status": "connected",
                        "success": true,
                        "mode": if cfg.test_mode { "test" } else { "live" },
                        "message": format!("Stripe answered GET /v1/balance with HTTP {}.", code.as_u16()),
                        "stripe_status": code.as_u16(),
                        "live_call_made": true,
                        "available": body.get("available").cloned(),
                        "pending": body.get("pending").cloned(),
                        "tested_at": now,
                        "warnings": warning.into_iter().collect::<Vec<String>>()
                    })).into_response();
                } else {
                    let err_text = body
                        .pointer("/error/message")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string())
                        .unwrap_or_else(|| clip(&body.to_string(), 300));
                    let warning =
                        record_stripe_test_state(&state.db, &cfg.id, &now, "error").await;

                    return (
                        StatusCode::BAD_GATEWAY,
                        Json(json!({
                        "status": "error",
                        "success": false,
                        "mode": if cfg.test_mode { "test" } else { "live" },
                        "message": format!("Stripe rejected the credentials (HTTP {}): {}", code.as_u16(), err_text),
                        "error": err_text,
                        "stripe_status": code.as_u16(),
                        "live_call_made": true,
                        "tested_at": now,
                        "warnings": warning.into_iter().collect::<Vec<String>>()
                        })),
                    )
                        .into_response();
                }
            }
            Err(e) => {
                let warning = record_stripe_test_state(&state.db, &cfg.id, &now, "error").await;
                return Json(json!({
                    "status": "error",
                    "success": false,
                    "mode": if cfg.test_mode { "test" } else { "live" },
                    "message": format!("Network error reaching Stripe: {}", e),
                    "live_call_made": false,
                    "tested_at": now,
                    "warnings": warning.into_iter().collect::<Vec<String>>()
                })).into_response();
            }
        }
    }

    // No key (or a masked one) means no verdict: report the missing credential
    // instead of inventing a sandbox that "connected".
    let reason = if cfg.api_key.trim().is_empty() {
        "No Stripe secret key is stored for the saved account. Enter an sk_test_… or sk_live_… key and save it, then test again.".to_string()
    } else if is_masked(&cfg.api_key) {
        "The stored Stripe secret key is masked for display; re-enter the real key before testing.".to_string()
    } else {
        format!(
            "The stored key {:?} is not a Stripe secret key (it must start with sk_test_ or sk_live_), so no call was made.",
            mask_stripe_key(&cfg.api_key)
        )
    };

    let warning = record_stripe_test_state(&state.db, &cfg.id, &now, "unconfigured").await;

    (
        StatusCode::SERVICE_UNAVAILABLE,
        Json(json!({
            "status": "unconfigured",
            "success": false,
            "configured": true,
            "mode": if cfg.test_mode { "test" } else { "live" },
            "message": reason,
            "error": reason,
            "live_call_made": false,
            "tested_at": now,
            "warnings": warning.into_iter().collect::<Vec<String>>()
        })),
    )
        .into_response()
}
