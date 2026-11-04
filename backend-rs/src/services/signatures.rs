#![allow(dead_code)]

use base64::Engine;
use hmac::{Hmac, Mac};
use sha1::Sha1;
use sha2::Sha256;

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

/// Shared-secret comparison for handshake tokens and query-string auth.
pub fn secrets_match(expected: &str, provided: Option<&str>) -> bool {
    if expected.is_empty() {
        return false;
    }
    constant_time_eq(expected.as_bytes(), provided.unwrap_or("").as_bytes())
}

/// Meta's X-Hub-Signature-256: HMAC-SHA256 over the raw request body keyed
/// with the app secret. Without it, anyone who learns the webhook URL can
/// forge inbound messages (fake STOP, impersonated contacts, bogus outcomes).
pub fn validate_meta_signature(app_secret: &str, body: &[u8], signature_header: Option<&str>) -> bool {
    let Some(raw) = signature_header else {
        return false;
    };
    let provided = raw.trim().strip_prefix("sha256=");
    let Some(provided) = provided else {
        return false;
    };
    if app_secret.is_empty() {
        return false;
    }
    let Ok(mut mac) = Hmac::<Sha256>::new_from_slice(app_secret.as_bytes()) else {
        return false;
    };
    mac.update(body);
    let expected = hex::encode(mac.finalize().into_bytes());
    constant_time_eq(expected.as_bytes(), provided.as_bytes())
}

fn twilio_data_string(url: &str, params: &[(String, String)]) -> String {
    let mut sorted: Vec<&(String, String)> = params.iter().collect();
    sorted.sort_by(|a, b| a.0.cmp(&b.0));
    let mut data = String::from(url);
    for (key, value) in sorted {
        data.push_str(key);
        data.push_str(value);
    }
    data
}

/// Twilio's X-Twilio-Signature: HMAC-SHA1 over the full request URL followed
/// by each POST parameter's key+value sorted by key, base64 encoded.
pub fn validate_twilio_signature(
    auth_token: &str,
    url: &str,
    params: &[(String, String)],
    signature: Option<&str>,
) -> bool {
    let Some(provided) = signature else {
        return false;
    };
    if auth_token.is_empty() {
        return false;
    }
    let Ok(mut mac) = Hmac::<Sha1>::new_from_slice(auth_token.as_bytes()) else {
        return false;
    };
    mac.update(twilio_data_string(url, params).as_bytes());
    let expected = base64::engine::general_purpose::STANDARD.encode(mac.finalize().into_bytes());
    constant_time_eq(expected.as_bytes(), provided.trim().as_bytes())
}

/// LINE's X-Line-Signature: Base64 HMAC-SHA256 over the raw body.
pub fn validate_line_signature(
    channel_secret: &str,
    body: &[u8],
    signature: Option<&str>,
) -> bool {
    let Some(provided) = signature else {
        return false;
    };
    if channel_secret.is_empty() {
        return false;
    }
    let Ok(mut mac) = Hmac::<Sha256>::new_from_slice(channel_secret.as_bytes()) else {
        return false;
    };
    mac.update(body);
    let expected = base64::engine::general_purpose::STANDARD.encode(mac.finalize().into_bytes());
    constant_time_eq(expected.as_bytes(), provided.trim().as_bytes())
}

/// TikTok's X-TikTok-Signature: hex HMAC-SHA256 over the raw body, with no
/// algorithm prefix (unlike Meta's `sha256=` form).
pub fn validate_tiktok_signature(
    app_secret: &str,
    body: &[u8],
    signature: Option<&str>,
) -> bool {
    let Some(provided) = signature else {
        return false;
    };
    if app_secret.is_empty() {
        return false;
    }
    let Ok(mut mac) = Hmac::<Sha256>::new_from_slice(app_secret.as_bytes()) else {
        return false;
    };
    mac.update(body);
    let expected = hex::encode(mac.finalize().into_bytes());
    constant_time_eq(expected.as_bytes(), provided.trim().as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;


    #[test]
    fn meta_signature_matches_known_vector() {
        let secret = "test_app_secret";
        let body = br#"{"entry":[]}"#;
        let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes()).unwrap();
        mac.update(body);
        let header = format!("sha256={}", hex::encode(mac.finalize().into_bytes()));
        assert!(validate_meta_signature(secret, body, Some(&header)));
        assert!(!validate_meta_signature(secret, body, Some("sha256=deadbeef")));
        assert!(!validate_meta_signature(secret, body, None));
        assert!(!validate_meta_signature("", body, Some(&header)));
    }

    #[test]
    fn twilio_signature_sorts_params_by_key() {
        let params = vec![
            ("Body".to_string(), "hi".to_string()),
            ("AccountSid".to_string(), "AC1".to_string()),
        ];
        let mut mac = Hmac::<Sha1>::new_from_slice(b"auth_token").unwrap();
        mac.update(b"https://example.test/hAccountSidAC1Bodyhi");
        let signature = base64::engine::general_purpose::STANDARD.encode(mac.finalize().into_bytes());
        assert!(validate_twilio_signature(
            "auth_token",
            "https://example.test/h",
            &params,
            Some(&signature)
        ));
        assert!(!validate_twilio_signature(
            "auth_token",
            "https://example.test/h",
            &params,
            Some("AAAA")
        ));
    }
}
