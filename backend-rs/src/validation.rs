//! Strict input validation in front of every endpoint (same design as nID's
//! apps/rust-api/src/core/validation.rs).
//!
//! Path segments, query string and JSON body are checked before any handler
//! runs. Unknown field names still get a strict default (bounded length, no
//! control characters, no bidi overrides); known names (phone, email, slug,
//! url, ids, ...) get type rules. Inbound provider webhooks carry arbitrary
//! signed payloads, so they keep the size and structure limits but not the
//! field-name rules.
//!
//! The browser mirrors these rules in apps/web/src/lib/formGuard.ts so a form
//! stops before it submits; the server never relies on that.

use axum::{
    body::{to_bytes, Body},
    extract::Request,
    http::{header, Method, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use regex::Regex;
use serde_json::Value;
use std::sync::LazyLock;

const MAX_BODY: usize = 2 * 1024 * 1024;
const MAX_DEPTH: usize = 10;
const MAX_NODES: usize = 20_000;
const MAX_ARRAY: usize = 2000;
const MAX_QUERY_PAIRS: usize = 60;

static RE_PHONE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^\+?[0-9]{6,15}$").unwrap());
static RE_EMAIL: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[A-Za-z0-9.!#$%&'*+/=?^_`{|}~-]+@[A-Za-z0-9](?:[A-Za-z0-9-]{0,61}[A-Za-z0-9])?(?:\.[A-Za-z0-9](?:[A-Za-z0-9-]{0,61}[A-Za-z0-9])?)+$").unwrap());
static RE_SLUG: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[a-z0-9](?:[a-z0-9_-]{0,62}[a-z0-9])?$").unwrap());
static RE_ID: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[A-Za-z0-9_.:-]{1,128}$").unwrap());
static RE_CC: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[A-Za-z]{2}$").unwrap());
static RE_CUR: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[A-Za-z]{3}$").unwrap());
static RE_OTP: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[0-9]{4,8}$").unwrap());
static RE_SEGMENT: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[A-Za-z0-9._~:@+-]{1,160}$").unwrap());
static RE_KEY: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[A-Za-z0-9_.-]{1,80}$").unwrap());
static RE_TAG: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?i)<\s*/?\s*[a-z!]").unwrap());

#[derive(Default)]
struct Problems(Vec<(String, String)>);
impl Problems {
    fn add(&mut self, path: &str, msg: impl Into<String>) {
        if self.0.len() < 20 {
            self.0.push((path.to_string(), msg.into()));
        }
    }
}

fn snake(key: &str) -> String {
    let mut out = String::with_capacity(key.len() + 4);
    for (i, c) in key.chars().enumerate() {
        if c.is_ascii_uppercase() {
            if i > 0 {
                out.push('_');
            }
            out.push(c.to_ascii_lowercase());
        } else {
            out.push(c);
        }
    }
    out
}

/// Bidi overrides, the zero-width space and line separators are used to spoof
/// text. ZWNJ and ZWJ stay allowed: Persian, Urdu and emoji sequences need them.
fn bad_chars(s: &str, allow_newlines: bool) -> bool {
    s.chars().any(|c| {
        matches!(c, '\u{202A}'..='\u{202E}' | '\u{2066}'..='\u{2069}' | '\u{200B}' | '\u{FEFF}' | '\u{2028}' | '\u{2029}')
            || (c.is_control() && !(allow_newlines && matches!(c, '\n' | '\r' | '\t')))
    })
}

fn is_secret(k: &str) -> bool {
    matches!(k, "credentials" | "password" | "secret" | "token" | "api_key" | "access_token" | "refresh_token" | "private_key" | "client_secret" | "webhook_secret" | "verify_token" | "signature" | "authorization")
        || k.ends_with("_secret") || k.ends_with("_token") || k.ends_with("_key") && k != "tenant_key" && k != "provider_key"
}

fn is_short_text(k: &str) -> bool {
    matches!(k, "name" | "display_name" | "title" | "label" | "city" | "category" | "topic" | "sku" | "template_name" | "section_type" | "channel" | "channel_type" | "provider" | "platform" | "role" | "status" | "priority" | "event_type" | "entity_type" | "action_type" | "trigger_type" | "trigger_event" | "source_type" | "language" | "locale" | "given_name" | "family_name" | "full_name" | "company")
}

fn check_url(path: &str, s: &str, k: &str, p: &mut Problems) {
    if s.len() > 2048 {
        return p.add(path, "must be at most 2048 characters");
    }
    let Ok(u) = url::Url::parse(s) else { return p.add(path, "must be a full web address such as https://example.com/path") };
    if !matches!(u.scheme(), "https" | "http") {
        return p.add(path, "must start with https://");
    }
    if !u.username().is_empty() || u.password().is_some() {
        return p.add(path, "must not contain a username or password");
    }
    let Some(host) = u.host_str() else { return p.add(path, "must include a host name") };
    // Addresses this server will call itself must be public https in production.
    if crate::hardening::is_production() && (k.contains("webhook") || k == "callback_url") {
        let h = host.trim_matches(|c| c == '[' || c == ']').to_ascii_lowercase();
        let private = h == "localhost" || h.ends_with(".local") || h.ends_with(".internal") || h.parse::<std::net::IpAddr>().map(|ip| match ip {
            std::net::IpAddr::V4(v) => v.is_private() || v.is_loopback() || v.is_link_local() || v.is_unspecified(),
            std::net::IpAddr::V6(v) => v.is_loopback() || v.is_unspecified(),
        }).unwrap_or(false);
        if u.scheme() != "https" || private {
            p.add(path, "must be a public https address");
        }
    }
}

fn check_string(path: &str, key: &str, s: &str, otp: bool, p: &mut Problems) {
    let k = snake(key);
    if is_secret(&k) {
        if s.len() > 8192 || s.contains('\u{0}') {
            p.add(path, "is not a valid secret value");
        }
        return;
    }
    // Free text keeps newlines and tabs; anything with a format does not.
    let free_text = !is_short_text(&k) && !k.ends_with("_id") && k != "id" && !k.contains("phone") && !k.contains("email") && !k.ends_with("url") && !matches!(k.as_str(), "slug" | "code" | "country_code" | "currency");
    if bad_chars(s, free_text) {
        return p.add(path, "contains characters that are not allowed (control, zero-width or direction-override characters)");
    }
    match k.as_str() {
        _ if k.contains("phone") || k == "whatsapp" => {
            let stripped: String = s.chars().filter(|c| !matches!(c, ' ' | '-' | '(' | ')' | '.')).collect();
            if !RE_PHONE.is_match(&stripped) {
                p.add(path, "must be a phone number with the country code, such as +14155550123");
            }
        }
        _ if k.contains("email") => {
            if s.len() > 254 || !RE_EMAIL.is_match(s) {
                p.add(path, "must be a valid email address");
            }
        }
        "slug" => {
            if !RE_SLUG.is_match(s) {
                p.add(path, "must be lowercase letters, numbers, hyphens or underscores");
            }
        }
        "country_code" => {
            if !RE_CC.is_match(s) {
                p.add(path, "must be a two-letter country code such as US");
            }
        }
        "currency" => {
            if !RE_CUR.is_match(s) {
                p.add(path, "must be a three-letter currency code such as USD");
            }
        }
        "code" if otp => {
            if !RE_OTP.is_match(s) {
                p.add(path, "must be the 4 to 8 digit code you received");
            }
        }
        _ if k == "url" || k.ends_with("_url") || k == "website" || k == "redirect_uri" => {
            if !s.is_empty() {
                check_url(path, s, &k, p);
            }
        }
        _ if k == "id" || k.ends_with("_id") => {
            if !RE_ID.is_match(s) {
                p.add(path, "must be 1 to 128 letters, numbers, dots, colons, dashes or underscores");
            }
        }
        _ if is_short_text(&k) => {
            if s.chars().count() > 300 {
                p.add(path, "must be at most 300 characters");
            } else if RE_TAG.is_match(s) {
                p.add(path, "must not contain HTML tags");
            }
        }
        _ => {
            if s.chars().count() > 20_000 {
                p.add(path, "must be at most 20000 characters");
            }
        }
    }
}

fn check_number(path: &str, key: &str, n: &serde_json::Number, p: &mut Problems) {
    let Some(f) = n.as_f64() else { return p.add(path, "must be a finite number") };
    if !f.is_finite() || f.abs() > 1.0e15 {
        return p.add(path, "is out of range");
    }
    let range = match snake(key).as_str() {
        "latitude" | "lat" => Some((-90.0, 90.0)),
        "longitude" | "lng" | "lon" => Some((-180.0, 180.0)),
        "percentage" | "percent" => Some((0.0, 100.0)),
        "limit" | "page_size" | "per_page" => Some((0.0, 1000.0)),
        _ => None,
    };
    if let Some((lo, hi)) = range {
        if f < lo || f > hi {
            p.add(path, format!("must be between {lo} and {hi}"));
        }
    }
}

fn walk(v: &Value, path: &str, key: &str, depth: usize, nodes: &mut usize, otp: bool, semantic: bool, p: &mut Problems) {
    *nodes += 1;
    if *nodes > MAX_NODES {
        return p.add(path, "the request is too large");
    }
    if depth > MAX_DEPTH {
        return p.add(path, "is nested too deeply");
    }
    match v {
        Value::String(s) if semantic => check_string(path, key, s, otp, p),
        Value::String(s) => {
            if s.contains('\u{0}') {
                p.add(path, "contains a NUL character");
            }
        }
        Value::Number(n) if semantic => check_number(path, key, n, p),
        Value::Array(a) => {
            if a.len() > MAX_ARRAY {
                return p.add(path, format!("must have at most {MAX_ARRAY} items"));
            }
            for (i, item) in a.iter().enumerate() {
                walk(item, &format!("{path}[{i}]"), key, depth + 1, nodes, otp, semantic, p);
            }
        }
        Value::Object(m) => {
            if m.len() > 300 {
                return p.add(path, "has too many fields");
            }
            for (k, val) in m {
                if semantic && !RE_KEY.is_match(k) {
                    p.add(path, format!("has a field name that is not allowed: {}", k.chars().take(24).collect::<String>()));
                    continue;
                }
                let child = if path.is_empty() { k.clone() } else { format!("{path}.{k}") };
                walk(val, &child, k, depth + 1, nodes, otp, semantic, p);
            }
        }
        _ => {}
    }
}

fn reject(status: StatusCode, p: Problems) -> Response {
    let fields: Vec<Value> = p.0.iter().map(|(f, m)| serde_json::json!({ "field": f, "message": m })).collect();
    let message = p.0.first().map(|(f, m)| if f.is_empty() { m.clone() } else { format!("{f} {m}") }).unwrap_or_else(|| "Invalid request".into());
    (status, Json(serde_json::json!({ "error": "Validation Error", "message": message, "fields": fields }))).into_response()
}

pub async fn gate(req: Request, next: Next) -> Response {
    let (parts, body) = req.into_parts();
    let mut p = Problems::default();
    let path = parts.uri.path().to_string();
    let webhook = path.contains("webhook") || path.starts_with("/verify/proof") || path.contains("/inbound");
    let otp = path.contains("/auth/");

    if path.len() > 768 {
        p.add("path", "is too long");
    }
    // Serving the SPA and its assets: any path under these is a file path.
    let api = path.starts_with("/api/") || path.starts_with("/verify/");
    if api {
        for seg in path.split('/').filter(|s| !s.is_empty()) {
            let decoded = urlencoding::decode(seg).map(|c| c.into_owned()).unwrap_or_default();
            if decoded.is_empty() || decoded == "." || decoded == ".." || !RE_SEGMENT.is_match(&decoded) {
                p.add("path", "contains a segment that is not allowed");
                break;
            }
        }
    }
    if let Some(q) = parts.uri.query() {
        if q.len() > 8192 {
            p.add("query", "is too long");
        } else if api && !webhook {
            let pairs: Vec<(String, String)> = url::form_urlencoded::parse(q.as_bytes()).map(|(k, v)| (k.into_owned(), v.into_owned())).collect();
            if pairs.len() > MAX_QUERY_PAIRS {
                p.add("query", "has too many parameters");
            }
            for (k, v) in pairs {
                if !RE_KEY.is_match(&k) {
                    p.add("query", "has a parameter name that is not allowed");
                    continue;
                }
                check_string(&format!("query.{k}"), &k, &v, otp, &mut p);
            }
        }
    }
    if !p.0.is_empty() {
        return reject(StatusCode::BAD_REQUEST, p);
    }

    let has_body = !matches!(parts.method, Method::GET | Method::HEAD | Method::OPTIONS);
    let ctype = parts.headers.get(header::CONTENT_TYPE).and_then(|v| v.to_str().ok()).unwrap_or("").to_ascii_lowercase();
    let declared: usize = parts.headers.get(header::CONTENT_LENGTH).and_then(|v| v.to_str().ok()).and_then(|v| v.parse().ok()).unwrap_or(0);

    let body = if has_body && ctype.starts_with("application/json") {
        let bytes = match to_bytes(body, MAX_BODY).await {
            Ok(b) => b,
            Err(_) => {
                let mut p = Problems::default();
                p.add("body", "is larger than the 2 MiB limit");
                return reject(StatusCode::PAYLOAD_TOO_LARGE, p);
            }
        };
        if !bytes.is_empty() {
            match serde_json::from_slice::<Value>(&bytes) {
                Ok(v) => {
                    let mut nodes = 0;
                    walk(&v, "", "", 0, &mut nodes, otp, !webhook, &mut p);
                    if !p.0.is_empty() {
                        return reject(StatusCode::UNPROCESSABLE_ENTITY, p);
                    }
                    // Sign-in and OTP routes: one budget per phone or email,
                    // so rotating IP addresses does not reset it.
                    if otp {
                        for field in ["phone", "email", "identifier", "username"] {
                            if let Some(id) = v.get(field).and_then(|x| x.as_str()) {
                                if let Err(res) = crate::hardening::identifier_guard(id).await {
                                    return res;
                                }
                            }
                        }
                    }
                }
                // A webhook sender may use a non-JSON body despite the header; the
                // handler's signature check decides.
                Err(_) if webhook => {}
                Err(_) => {
                    let mut p = Problems::default();
                    p.add("body", "is not valid JSON");
                    return reject(StatusCode::BAD_REQUEST, p);
                }
            }
        }
        Body::from(bytes)
    } else if has_body && declared > 0 && !webhook && !(ctype.starts_with("multipart/form-data") || ctype.starts_with("application/x-www-form-urlencoded")) {
        let mut p = Problems::default();
        p.add("body", "must be sent as application/json");
        return reject(StatusCode::UNSUPPORTED_MEDIA_TYPE, p);
    } else {
        body
    };
    next.run(Request::from_parts(parts, body)).await
}
