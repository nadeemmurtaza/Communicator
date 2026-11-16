//! Security headers and rate limiting at every scope.
//!
//! Counters live in Redis (crates/rate_limit) so every instance shares one
//! limit; if the store is unreachable an in-process counter takes over, it
//! never fails open. Scopes: whole service, per client IP, route class
//! (sign-in and OTP, writes, public webhooks), per credential (hash of the
//! Authorization or X-Tenant-Key header), per phone or email on sign-in, and
//! a lockout after repeated 401/403 or malformed requests.

use axum::{
    body::Body,
    extract::ConnectInfo,
    http::{header, HeaderName, HeaderValue, Method, Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use newax_rate_limit::{Decision, RateLimiter};
use sha2::{Digest, Sha256};
use std::{
    net::SocketAddr,
    sync::{Arc, OnceLock},
    time::Duration,
};

static LIMITER: OnceLock<Arc<RateLimiter>> = OnceLock::new();
static PRODUCTION: OnceLock<bool> = OnceLock::new();

const MIN: Duration = Duration::from_secs(60);
const HOUR: Duration = Duration::from_secs(3600);

fn envn(name: &str, default: u32) -> u32 {
    std::env::var(name).ok().and_then(|v| v.parse().ok()).unwrap_or(default)
}

pub fn is_production() -> bool {
    *PRODUCTION.get().unwrap_or(&false)
}

/// Production refuses to start without Redis: per-instance counters are not a
/// limit once there are two instances.
pub async fn init(production: bool) -> Result<(), Box<dyn std::error::Error>> {
    let _ = PRODUCTION.set(production);
    let url = std::env::var("REDIS_URL").ok().filter(|u| !u.trim().is_empty());
    if production && url.is_none() {
        return Err("REDIS_URL must be set in production so rate limits are shared across instances".into());
    }
    let rl = RateLimiter::connect(url.as_deref(), "communicator").await?;
    tracing::info!(shared = rl.is_shared(), "Rate limiter ready");
    let _ = LIMITER.set(Arc::new(rl));
    Ok(())
}

async fn limiter() -> Arc<RateLimiter> {
    if let Some(l) = LIMITER.get() {
        return l.clone();
    }
    Arc::new(RateLimiter::connect(None, "communicator").await.expect("local limiter"))
}

/// The real client address. Forwarding headers are trusted only for
/// `TRUSTED_PROXY_HOPS` hops (default 0), counted from the right of
/// X-Forwarded-For so a client cannot invent its own address. Behind the
/// Cloudflare tunnel set `TRUSTED_PROXY_HOPS=1`.
pub fn client_ip(req: &Request<Body>) -> String {
    let hops = envn("TRUSTED_PROXY_HOPS", 0) as usize;
    if hops > 0 {
        if let Some(ip) = req.headers().get("cf-connecting-ip").and_then(|v| v.to_str().ok()) {
            return ip.trim().to_string();
        }
        if let Some(xff) = req.headers().get("x-forwarded-for").and_then(|v| v.to_str().ok()) {
            let parts: Vec<&str> = xff.split(',').map(str::trim).filter(|p| !p.is_empty()).collect();
            if parts.len() >= hops {
                return parts[parts.len() - hops].to_string();
            }
        }
    }
    req.extensions().get::<ConnectInfo<SocketAddr>>().map(|c| c.0.ip().to_string()).unwrap_or_else(|| "unknown".to_string())
}

fn too_many(d: &Decision, what: &str) -> Response {
    let mut res = (StatusCode::TOO_MANY_REQUESTS, format!("Too many {what}. Try again in {} seconds.", d.retry_after)).into_response();
    let h = res.headers_mut();
    for (k, v) in [("retry-after", d.retry_after.to_string()), ("ratelimit-limit", d.limit.to_string()), ("ratelimit-remaining", "0".to_string())] {
        if let Ok(v) = HeaderValue::from_str(&v) {
            h.insert(HeaderName::from_static(k), v);
        }
    }
    res
}

/// Hash of whatever credential the caller presented, so keys and tokens never
/// become Redis key names.
fn credential_id(req: &Request<Body>) -> Option<String> {
    let raw = req
        .headers()
        .get(header::AUTHORIZATION)
        .or_else(|| req.headers().get("x-tenant-key"))
        .and_then(|v| v.to_str().ok())?;
    Some(hex::encode(&Sha256::digest(raw.as_bytes())[..12]))
}

pub async fn guard(req: Request<Body>, next: Next) -> Response {
    let path = req.uri().path().to_string();
    if path.ends_with("/health") {
        return next.run(req).await;
    }
    let rl = limiter().await;
    let ip = client_ip(&req);
    let writes = !matches!(*req.method(), Method::GET | Method::HEAD | Method::OPTIONS);
    // Inbound provider webhooks (Meta, TikTok, Shopify, ...) come from the
    // provider's addresses and are signature-checked; they get their own, much
    // larger budget instead of the per-client one.
    let webhook = path.contains("webhook") || path.starts_with("/verify/proof") || path.contains("/inbound");

    if let Some(secs) = rl.blocked_for("ip", &ip).await {
        return too_many(&Decision { allowed: false, limit: 0, remaining: 0, retry_after: secs }, "failed attempts from this address");
    }
    let d = rl.hit("global", "all", envn("RL_GLOBAL_PER_MIN", 12000), MIN).await;
    if !d.allowed {
        return too_many(&d, "requests to the service");
    }
    if webhook {
        let d = rl.hit("webhook", &ip, envn("RL_WEBHOOK_PER_MIN", 3000), MIN).await;
        if !d.allowed {
            return too_many(&d, "webhook deliveries");
        }
    } else {
        let d = rl.hit("ip", &ip, envn("RL_IP_PER_MIN", 300), MIN).await;
        if !d.allowed {
            return too_many(&d, "requests from this address");
        }
        if path.contains("/auth/") {
            for (scope, limit, window) in [("auth-min", envn("RL_AUTH_PER_MIN", 10), MIN), ("auth-hour", envn("RL_AUTH_PER_HOUR", 60), HOUR)] {
                let d = rl.hit(scope, &ip, limit, window).await;
                if !d.allowed {
                    return too_many(&d, "sign-in requests");
                }
            }
        } else if writes {
            for (scope, limit, window) in [("write-min", envn("RL_WRITE_PER_MIN", 120), MIN), ("write-hour", envn("RL_WRITE_PER_HOUR", 3000), HOUR)] {
                let d = rl.hit(scope, &ip, limit, window).await;
                if !d.allowed {
                    return too_many(&d, "write requests");
                }
            }
        }
        if let Some(cred) = credential_id(&req) {
            let d = rl.hit("credential", &cred, envn("RL_CREDENTIAL_PER_MIN", 1200), MIN).await;
            if !d.allowed {
                return too_many(&d, "requests for this account");
            }
        }
    }

    let res = next.run(req).await;
    let status = res.status();
    if matches!(status, StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN) && !webhook {
        let d = rl.hit("authfail", &ip, envn("RL_AUTHFAIL_PER_10MIN", 30), Duration::from_secs(600)).await;
        if !d.allowed {
            rl.block("ip", &ip, Duration::from_secs(envn("RL_LOCKOUT_SECS", 900) as u64)).await;
            tracing::warn!(%ip, "address locked out after repeated authentication failures");
        }
    }
    if matches!(status, StatusCode::BAD_REQUEST | StatusCode::PAYLOAD_TOO_LARGE | StatusCode::UNSUPPORTED_MEDIA_TYPE | StatusCode::UNPROCESSABLE_ENTITY) && !webhook {
        let d = rl.hit("badreq", &ip, envn("RL_BADREQ_PER_10MIN", 60), Duration::from_secs(600)).await;
        if !d.allowed {
            rl.block("ip", &ip, Duration::from_secs(envn("RL_LOCKOUT_SECS", 900) as u64)).await;
            tracing::warn!(%ip, "address locked out after repeated invalid requests");
        }
    }
    res
}

/// Per phone number or email on sign-in and OTP routes, so rotating IPs does
/// not reset the budget for one account.
pub async fn identifier_guard(identifier: &str) -> Result<(), Response> {
    let id: String = identifier.trim().to_lowercase();
    let rl = limiter().await;
    for (scope, limit, window) in [("ident-10m", envn("RL_IDENT_PER_10MIN", 5), Duration::from_secs(600)), ("ident-1h", envn("RL_IDENT_PER_HOUR", 12), HOUR)] {
        let d = rl.hit(scope, &id, limit, window).await;
        if !d.allowed {
            return Err(too_many(&d, "attempts for this account"));
        }
    }
    Ok(())
}

pub async fn security_headers(req: Request<Body>, next: Next) -> Response {
    let mut res = next.run(req).await;
    let h = res.headers_mut();
    let set = |h: &mut axum::http::HeaderMap, name: HeaderName, value: &'static str| {
        h.entry(name).or_insert(HeaderValue::from_static(value));
    };
    set(h, header::X_CONTENT_TYPE_OPTIONS, "nosniff");
    set(h, header::X_FRAME_OPTIONS, "SAMEORIGIN");
    set(h, header::REFERRER_POLICY, "strict-origin-when-cross-origin");
    set(h, header::STRICT_TRANSPORT_SECURITY, "max-age=31536000; includeSubDomains");
    // Only the parts of a CSP that cannot break the single-page app; a full
    // script-src policy needs the frontend's inline scripts audited first.
    set(h, header::CONTENT_SECURITY_POLICY, "frame-ancestors 'self'; base-uri 'self'; object-src 'none'");
    // The live-proof capture page uses the camera and microphone.
    set(h, HeaderName::from_static("permissions-policy"), "camera=(self), microphone=(self), geolocation=()");
    res
}

/// Shared outbound HTTP client. reqwest's default has no timeout, so one slow
/// provider could hold a request (and its task) open forever.
pub fn http_client() -> reqwest::Client {
    static CLIENT: std::sync::OnceLock<reqwest::Client> = std::sync::OnceLock::new();
    CLIENT
        .get_or_init(|| {
            reqwest::Client::builder()
                .connect_timeout(Duration::from_secs(5))
                .timeout(Duration::from_secs(20))
                .build()
                .expect("reqwest client")
        })
        .clone()
}
