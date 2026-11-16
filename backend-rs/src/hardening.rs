//! Security headers and API-wide rate limiting.
//!
//! The rate limit is a per-client fixed window held in process memory: enough
//! to blunt scripted abuse of one instance. Behind several instances it needs
//! a shared store (Redis) -- see PRODUCTION_READINESS_RULES.md item 7.

use axum::{
    body::Body,
    extract::{ConnectInfo, State},
    http::{header, HeaderName, HeaderValue, Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

#[derive(Clone)]
pub struct RateLimiter {
    window: Duration,
    max: u32,
    hits: Arc<Mutex<HashMap<String, (Instant, u32)>>>,
}

impl RateLimiter {
    pub fn new(max_per_window: u32, window: Duration) -> Self {
        Self { window, max: max_per_window, hits: Arc::new(Mutex::new(HashMap::new())) }
    }

    /// Ok, or the seconds until the window resets.
    fn check(&self, key: &str) -> Result<(), u64> {
        let now = Instant::now();
        let mut hits = self.hits.lock().unwrap_or_else(|p| p.into_inner());
        if hits.len() > 50_000 {
            hits.retain(|_, (start, _)| now.duration_since(*start) < self.window);
        }
        let entry = hits.entry(key.to_string()).or_insert((now, 0));
        if now.duration_since(entry.0) >= self.window {
            *entry = (now, 0);
        }
        entry.1 += 1;
        if entry.1 > self.max {
            Err(self.window.saturating_sub(now.duration_since(entry.0)).as_secs().max(1))
        } else {
            Ok(())
        }
    }
}

/// The real client address: Cloudflare's header when the request came through
/// the tunnel, else the TCP peer.
fn client_key(req: &Request<Body>) -> String {
    if let Some(ip) = req.headers().get("cf-connecting-ip").and_then(|v| v.to_str().ok()) {
        return ip.trim().to_string();
    }
    req.extensions()
        .get::<ConnectInfo<SocketAddr>>()
        .map(|c| c.0.ip().to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

pub async fn rate_limit(State(limiter): State<RateLimiter>, req: Request<Body>, next: Next) -> Response {
    // Health checks must never be throttled by the thing they monitor.
    if req.uri().path().ends_with("/health") {
        return next.run(req).await;
    }
    match limiter.check(&client_key(&req)) {
        Ok(()) => next.run(req).await,
        Err(retry_after) => {
            let mut res = (StatusCode::TOO_MANY_REQUESTS, "Too many requests. Try again shortly.").into_response();
            if let Ok(v) = HeaderValue::from_str(&retry_after.to_string()) {
                res.headers_mut().insert(header::RETRY_AFTER, v);
            }
            res
        }
    }
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
