//! Enterprise SSO / SAML 2.0 Service Provider surface (port of
//! api/routes/sso.py). Consumed by frontend/src/pages/Team.tsx, which links
//! straight to `/api/v1/sso/metadata.xml` and posts the IdP's form to
//! `/api/v1/sso/saml/acs`.
//!
//! What is real here: the configuration row, the generated SP metadata, the
//! AuthnRequest, and a live reachability probe of the IdP SSO URL.
//!
//! What is NOT available: verifying an inbound SAML assertion. Rust has no XML
//! reader and no RSA/X.509 verifier in Cargo.toml, and an unsigned assertion is
//! a login anyone can forge -- so the ACS endpoint answers 503 and names the
//! exact missing crates instead of provisioning a session.

use crate::db::AppState;
use axum::body::Bytes;
use axum::extract::{Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use base64::Engine as _;
use chrono::Utc;
use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;

const MISSING_SAML_DEPENDENCIES: &[&str] = &[
    "quick-xml (or roxmltree) to parse the Response document",
    "xmlsec (or rsa + x509-cert + sha1) to validate the XML-DSig enveloped signature and the reference digest",
    "flate2 to decode a DEFLATE-compressed HTTP-Redirect SAMLRequest/SAMLResponse",
];

// ---------------------------------------------------------------------------
// Config storage
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, sqlx::FromRow)]
struct SsoRow {
    id: String,
    is_enabled: Option<bool>,
    provider_type: Option<String>,
    idp_entity_id: Option<String>,
    idp_sso_url: Option<String>,
    idp_x509_cert: Option<String>,
    sp_entity_id: Option<String>,
    sp_acs_url: Option<String>,
    enforced_domains: Option<String>,
    allow_unlinked_passwords: Option<bool>,
    jit_provisioning_enabled: Option<bool>,
    default_role: Option<String>,
    attribute_mapping: Option<String>,
    last_tested_at: Option<String>,
    last_test_status: Option<String>,
    last_test_message: Option<String>,
    created_at: Option<String>,
    updated_at: Option<String>,
}

fn parse_string_array(raw: Option<String>) -> Vec<String> {
    raw.and_then(|v| serde_json::from_str::<Vec<String>>(&v).ok())
        .unwrap_or_default()
}

fn parse_object(raw: Option<String>) -> Value {
    raw.and_then(|v| serde_json::from_str(&v).ok()).unwrap_or_else(|| json!({}))
}

fn to_text(raw: &str) -> String {
    // Meta/Okta PEM blocks arrive with real newlines; keep them intact but
    // strip the surrounding whitespace the UI tends to add.
    raw.trim().to_string()
}

impl SsoRow {
    /// What an unconfigured deployment looks like: nothing has ever been saved,
    /// so every operator-settable field is null rather than an empty string or a
    /// default that would look like a choice somebody made.
    fn unconfigured() -> Self {
        Self {
            id: String::new(),
            is_enabled: Some(false),
            provider_type: None,
            idp_entity_id: None,
            idp_sso_url: None,
            idp_x509_cert: None,
            sp_entity_id: None,
            sp_acs_url: None,
            enforced_domains: None,
            allow_unlinked_passwords: None,
            jit_provisioning_enabled: None,
            default_role: None,
            attribute_mapping: None,
            last_tested_at: None,
            last_test_status: None,
            last_test_message: None,
            created_at: None,
            updated_at: None,
        }
    }

    fn json(&self) -> Value {
        json!({
            // The DDL for sso_configs carries literal column defaults; rowExists
            // is what tells the UI whether any of this was actually saved.
            "id": self.id,
            "rowExists": !self.id.is_empty(),
            "is_enabled": self.is_enabled.unwrap_or(false),
            "provider_type": self.provider_type.clone(),
            "idp_entity_id": self.idp_entity_id.clone(),
            "idp_sso_url": self.idp_sso_url.clone(),
            "idp_x509_cert": self.idp_x509_cert.clone(),
            "sp_entity_id": self.sp_entity_id.clone(),
            "sp_acs_url": self.sp_acs_url.clone(),
            "enforced_domains": parse_string_array(self.enforced_domains.clone()),
            "jit_provisioning_enabled": self.jit_provisioning_enabled,
            "default_role": self.default_role.clone(),
            "allow_unlinked_passwords": self.allow_unlinked_passwords,
            "attribute_mapping": parse_object(self.attribute_mapping.clone()),
            "last_tested_at": self.last_tested_at,
            "last_test_status": self.last_test_status.clone(),
            "last_test_message": self.last_test_message.clone(),
            "created_at": self.created_at.clone(),
            "updated_at": self.updated_at.clone(),
        })
    }
}

const SELECT_SSO: &str = "SELECT id, is_enabled, provider_type, idp_entity_id, idp_sso_url, idp_x509_cert, sp_entity_id, sp_acs_url, enforced_domains, allow_unlinked_passwords, jit_provisioning_enabled, default_role, attribute_mapping, last_tested_at, last_test_status, last_test_message, created_at, updated_at FROM sso_configs ORDER BY created_at ASC LIMIT 1";

async fn fetch_config(state: &AppState) -> Result<Option<SsoRow>, sqlx::Error> {
    sqlx::query_as::<_, SsoRow>(SELECT_SSO)
        .fetch_optional(&state.db)
        .await
}

/// Reads the config without writing anything. When no row has ever been saved the
/// caller gets `SsoRow::unconfigured()`, whose fields serialise as null, and a
/// database failure is a failure -- not silently "unconfigured".
async fn read_config(state: &AppState) -> Result<SsoRow, sqlx::Error> {
    Ok(fetch_config(state)
        .await?
        .unwrap_or_else(SsoRow::unconfigured))
}

fn config_read_failure(err: sqlx::Error) -> Response {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(json!({
            "error": "the SSO configuration could not be read",
            "detail": err.to_string(),
        })),
    )
        .into_response()
}


fn field<'a>(payload: &'a Value, snake: &str, camel: &str) -> Option<&'a Value> {
    payload
        .get(snake)
        .or_else(|| payload.get(camel))
        .filter(|v| !v.is_null())
}

fn field_str(payload: &Value, snake: &str, camel: &str) -> Option<String> {
    field(payload, snake, camel).and_then(|v| v.as_str()).map(to_text)
}

fn field_bool(payload: &Value, snake: &str, camel: &str) -> Option<bool> {
    field(payload, snake, camel).and_then(|v| v.as_bool())
}

pub async fn get_config(State(state): State<AppState>) -> Response {
    let config = match read_config(&state).await {
        Ok(config) => config,
        Err(err) => return config_read_failure(err),
    };
    let mut body = config.json();
    if config.id.is_empty() {
        body["note"] = json!(
            "No SSO configuration has ever been saved for this deployment, so every null field is unset rather than a default. SSO stays disabled until an IdP is configured."
        );
    }
    Json(json!({ "sso_config": body })).into_response()
}

pub async fn update_config(
    State(state): State<AppState>,
    Json(payload): Json<Value>,
) -> Response {
    let mut config = match read_config(&state).await {
        Ok(config) => config,
        Err(err) => return config_read_failure(err),
    };
    let existing_row = !config.id.is_empty();

    if let Some(v) = field_bool(&payload, "is_enabled", "isEnabled") {
        config.is_enabled = Some(v);
    }
    if let Some(v) = field_str(&payload, "provider_type", "providerType") {
        config.provider_type = Some(v);
    }
    if let Some(v) = field_str(&payload, "idp_entity_id", "idpEntityId") {
        config.idp_entity_id = Some(v);
    }
    if let Some(v) = field_str(&payload, "idp_sso_url", "idpSsoUrl") {
        config.idp_sso_url = Some(v);
    }
    if let Some(v) = field_str(&payload, "idp_x509_cert", "idpX509Cert") {
        config.idp_x509_cert = Some(v);
    }
    if let Some(v) = field_str(&payload, "sp_entity_id", "spEntityId") {
        config.sp_entity_id = Some(v);
    }
    if let Some(v) = field_str(&payload, "sp_acs_url", "spAcsUrl") {
        config.sp_acs_url = Some(v);
    }
    if let Some(v) = field(&payload, "enforced_domains", "enforcedDomains") {
        let normalized: Vec<String> = match v {
            Value::String(text) => text
                .split(',')
                .map(|d| d.trim().to_lowercase())
                .filter(|d| !d.is_empty())
                .collect(),
            Value::Array(items) => items
                .iter()
                .filter_map(|i| i.as_str())
                .map(|d| d.trim().to_lowercase())
                .filter(|d| !d.is_empty())
                .collect(),
            _ => Vec::new(),
        };
        config.enforced_domains = Some(serde_json::to_string(&normalized).unwrap_or_else(|_| "[]".to_string()));
    }
    if let Some(v) = field_bool(&payload, "jit_provisioning_enabled", "jitProvisioningEnabled") {
        config.jit_provisioning_enabled = Some(v);
    }
    if let Some(v) = field_str(&payload, "default_role", "defaultRole") {
        config.default_role = Some(v);
    }
    if let Some(v) = field_bool(&payload, "allow_unlinked_passwords", "allowUnlinkedPasswords") {
        config.allow_unlinked_passwords = Some(v);
    }
    if let Some(v) = field(&payload, "attribute_mapping", "attributeMapping") {
        config.attribute_mapping = Some(v.to_string());
    }

    let now = Utc::now().to_rfc3339();
    config.updated_at = Some(now.clone());
    if !existing_row {
        config.id = Uuid::new_v4().to_string();
        config.created_at = Some(now.clone());
    }

    // Upsert, not UPDATE: an unconfigured deployment has no row, and the first
    // save has to create it. Nothing is defaulted into a value the operator never
    // chose -- unset nullable columns are written as NULL, and the only non-null
    // substitutes are `false` / empty string, which genuinely mean "off" and "not
    // set".
    let result = sqlx::query(
        r#"
        INSERT INTO sso_configs
            (id, is_enabled, provider_type, idp_entity_id, idp_sso_url, idp_x509_cert,
             sp_entity_id, sp_acs_url, enforced_domains, jit_provisioning_enabled,
             default_role, allow_unlinked_passwords, attribute_mapping,
             created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        ON CONFLICT(id) DO UPDATE SET
            is_enabled = excluded.is_enabled,
            provider_type = excluded.provider_type,
            idp_entity_id = excluded.idp_entity_id,
            idp_sso_url = excluded.idp_sso_url,
            idp_x509_cert = excluded.idp_x509_cert,
            sp_entity_id = excluded.sp_entity_id,
            sp_acs_url = excluded.sp_acs_url,
            enforced_domains = excluded.enforced_domains,
            jit_provisioning_enabled = excluded.jit_provisioning_enabled,
            default_role = excluded.default_role,
            allow_unlinked_passwords = excluded.allow_unlinked_passwords,
            attribute_mapping = excluded.attribute_mapping,
            updated_at = excluded.updated_at
        "#,
    )
    .bind(&config.id)
    .bind(config.is_enabled.unwrap_or(false))
    .bind(config.provider_type.clone().unwrap_or_default())
    .bind(config.idp_entity_id.clone())
    .bind(config.idp_sso_url.clone())
    .bind(config.idp_x509_cert.clone())
    .bind(config.sp_entity_id.clone())
    .bind(config.sp_acs_url.clone())
    .bind(config.enforced_domains.clone())
    .bind(config.jit_provisioning_enabled.unwrap_or(false))
    .bind(config.default_role.clone())
    .bind(config.allow_unlinked_passwords.unwrap_or(false))
    .bind(config.attribute_mapping.clone())
    .bind(config.created_at.clone().unwrap_or(now.clone()))
    .bind(config.updated_at.clone().unwrap_or(now.clone()))
    .execute(&state.db)
    .await;

    match result {
        Ok(_) => (
            StatusCode::OK,
            Json(json!({
                "sso_config": config.json(),
                "created": !existing_row,
            })),
        )
            .into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "detail": format!("SSO configuration could not be saved: {err}"),
                "error": err.to_string(),
            })),
        )
            .into_response(),
    }
}

// ---------------------------------------------------------------------------
// Request helpers
// ---------------------------------------------------------------------------

fn header<'a>(headers: &'a HeaderMap, name: &str) -> Option<&'a str> {
    headers.get(name).and_then(|v| v.to_str().ok())
}

/// Absolute base URL of this service, proxy-aware, because the IdP has to be
/// given URLs the browser can actually reach.
fn base_url(headers: &HeaderMap) -> String {
    let scheme = header(headers, "x-forwarded-proto").unwrap_or("http");
    let host = header(headers, "x-forwarded-host")
        .or_else(|| header(headers, "host"))
        .unwrap_or("localhost");
    format!("{scheme}://{host}")
}

fn absolute_acs(config: &SsoRow, base: &str) -> String {
    let stored = config
        .sp_acs_url
        .clone()
        .unwrap_or_else(|| "/api/v1/sso/saml/acs".to_string());
    if stored.starts_with("http://") || stored.starts_with("https://") {
        stored
    } else if stored.starts_with('/') {
        format!("{}{stored}", base.trim_end_matches('/'))
    } else {
        format!("{}/{}", base.trim_end_matches('/'), stored)
    }
}

fn escape_xml(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for ch in input.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&apos;"),
            other => out.push(other),
        }
    }
    out
}

// ---------------------------------------------------------------------------
// SP metadata
// ---------------------------------------------------------------------------

/// SAML 2.0 EntityDescriptor for import into Okta / Azure AD / Google
/// Workspace / OneLogin. Generated from the stored config, never a template.
pub async fn sp_metadata(State(state): State<AppState>, headers: HeaderMap) -> Response {
    let config = match read_config(&state).await {
        Ok(config) => config,
        Err(err) => return config_read_failure(err),
    };
    let base = base_url(&headers);
    let acs = absolute_acs(&config, &base);
    let entity_id = config
        .sp_entity_id
        .clone()
        .filter(|v| !v.trim().is_empty())
        .unwrap_or_else(|| format!("{}/api/v1/sso/metadata", base.trim_end_matches('/')));

    let xml = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<md:EntityDescriptor xmlns:md="urn:oasis:names:tc:SAML:2.0:metadata" entityID="{entity}">
  <md:SPSSODescriptor AuthnRequestsSigned="false" WantAssertionsSigned="true" protocolSupportEnumeration="urn:oasis:names:tc:SAML:2.0:protocol">
    <md:NameIDFormat>urn:oasis:names:tc:SAML:1.1:nameid-format:emailAddress</md:NameIDFormat>
    <md:NameIDFormat>urn:oasis:names:tc:SAML:2.0:nameid-format:persistent</md:NameIDFormat>
    <md:NameIDFormat>urn:oasis:names:tc:SAML:2.0:nameid-format:transient</md:NameIDFormat>
    <md:AssertionConsumerService Binding="urn:oasis:names:tc:SAML:2.0:bindings:HTTP-POST" Location="{acs}" index="1" isDefault="true"/>
  </md:SPSSODescriptor>
  <md:Organization>
    <md:OrganizationName xml:lang="en">Communicator</md:OrganizationName>
    <md:OrganizationDisplayName xml:lang="en">Communicator Enterprise Platform</md:OrganizationDisplayName>
    <md:OrganizationURL xml:lang="en">{base}</md:OrganizationURL>
  </md:Organization>
</md:EntityDescriptor>"#,
        entity = escape_xml(&entity_id),
        acs = escape_xml(&acs),
        base = escape_xml(&base),
    );

    (
        [(axum::http::header::CONTENT_TYPE, "application/xml")],
        xml,
    )
        .into_response()
}

// ---------------------------------------------------------------------------
// AuthnRequest / login redirect
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct LoginQuery {
    #[serde(default)]
    email: Option<String>,
    #[serde(default)]
    relay_state: Option<String>,
}

/// Builds a real SAML AuthnRequest from the stored config and returns the
/// IdP redirect. The request is unsigned, matching the SP metadata above
/// (AuthnRequestsSigned="false").
pub async fn initiate_login(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(params): Query<LoginQuery>,
) -> Response {
    let config = match read_config(&state).await {
        Ok(config) => config,
        Err(err) => return config_read_failure(err),
    };
    let enabled = config.is_enabled.unwrap_or(false);
    let idp_sso_url = config.idp_sso_url.clone().unwrap_or_default();

    if !enabled || idp_sso_url.trim().is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "detail": "Enterprise Single Sign-On is not configured or enabled.",
                "ssoEnabled": enabled,
                "idpSsoUrl": idp_sso_url,
            })),
        )
            .into_response();
    }

    let base = base_url(&headers);
    let acs = absolute_acs(&config, &base);
    let request_id = format!("ID_{}", Uuid::new_v4().simple());
    let issue_instant = Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
    let relay_state = params
        .relay_state
        .filter(|v| !v.trim().is_empty())
        .unwrap_or_else(|| "/".to_string());

    let xml_payload = format!(
        r#"<samlp:AuthnRequest xmlns:samlp="urn:oasis:names:tc:SAML:2.0:protocol"
  xmlns:saml="urn:oasis:names:tc:SAML:2.0:assertion"
  ID="{request_id}"
  Version="2.0"
  IssueInstant="{issue_instant}"
  Destination="{destination}"
  AssertionConsumerServiceURL="{acs}"
  ProtocolBinding="urn:oasis:names:tc:SAML:2.0:bindings:HTTP-POST">
  <saml:Issuer>{issuer}</saml:Issuer>
  <samlp:NameIDPolicy Format="urn:oasis:names:tc:SAML:1.1:nameid-format:emailAddress" AllowCreate="true"/>
</samlp:AuthnRequest>"#,
        destination = escape_xml(idp_sso_url.trim()),
        acs = escape_xml(&acs),
        issuer = escape_xml(config.sp_entity_id.as_deref().unwrap_or("")),
    );

    let b64 = base64::engine::general_purpose::STANDARD.encode(xml_payload.as_bytes());
    let separator = if idp_sso_url.contains('?') { '&' } else { '?' };
    let redirect_url = format!(
        "{idp_sso_url}{separator}SAMLRequest={saml}&RelayState={relay}",
        saml = urlencode(&b64),
        relay = urlencode(&relay_state),
    );

    // A DEFLATE-encoded HTTP-Redirect binding needs flate2, which is not a
    // dependency; the base64 (uncompressed) form is what we can honestly send.
    Json(json!({
        "request_id": request_id,
        "issue_instant": issue_instant,
        "saml_request_b64": b64,
        "saml_request_deflated": false,
        "redirect_url": redirect_url,
        "acs_url": acs,
        "relay_state": relay_state,
        "email": params.email,
        "note": "SAMLRequest is base64 without DEFLATE: flate2 is not in Cargo.toml, so the HTTP-Redirect binding's compressed form is not used.",
    }))
    .into_response()
}

fn urlencode(input: &str) -> String {
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

// ---------------------------------------------------------------------------
// Assertion Consumer Service
// ---------------------------------------------------------------------------

fn percent_decode(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'+' => {
                out.push(b' ');
                i += 1;
            }
            b'%' if i + 2 < bytes.len() => {
                let hex = std::str::from_utf8(&bytes[i + 1..i + 3]).unwrap_or("");
                match u8::from_str_radix(hex, 16) {
                    Ok(byte) => {
                        out.push(byte);
                        i += 3;
                    }
                    Err(_) => {
                        out.push(bytes[i]);
                        i += 1;
                    }
                }
            }
            byte => {
                out.push(byte);
                i += 1;
            }
        }
    }
    String::from_utf8_lossy(&out).to_string()
}

fn parse_form(bytes: &[u8]) -> Vec<(String, String)> {
    String::from_utf8_lossy(bytes)
        .split('&')
        .filter(|pair| !pair.is_empty())
        .map(|pair| {
            let (k, v) = pair.split_once('=').unwrap_or((pair, ""));
            (percent_decode(k), percent_decode(v))
        })
        .collect()
}

/// The ACS endpoint. It accepts the IdP's POST (form or JSON), and answers 503:
/// without an XML parser and an XML-DSig verifier an assertion cannot be
/// trusted, and provisioning a session from unverified input would be a forged
/// login.
pub async fn assertion_consumer_service(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    let content_type = header(&headers, "content-type").unwrap_or_default();
    let mut saml_response = String::new();
    let mut relay_state = "/".to_string();

    if content_type.contains("application/json") {
        if let Ok(parsed) = serde_json::from_slice::<Value>(&body) {
            saml_response = parsed
                .get("saml_response")
                .or_else(|| parsed.get("SAMLResponse"))
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string();
            relay_state = parsed
                .get("relay_state")
                .or_else(|| parsed.get("RelayState"))
                .and_then(|v| v.as_str())
                .filter(|v| !v.trim().is_empty())
                .unwrap_or("/")
                .to_string();
        }
    } else {
        for (key, value) in parse_form(&body) {
            match key.as_str() {
                "SAMLResponse" => saml_response = value,
                "RelayState" => {
                    if !value.trim().is_empty() {
                        relay_state = value;
                    }
                }
                _ => {}
            }
        }
    }

    if saml_response.trim().is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "detail": "Missing SAMLResponse in the request body.",
                "expected": "application/x-www-form-urlencoded with SAMLResponse (and optional RelayState), or JSON with saml_response.",
            })),
        )
            .into_response();
    }

    let config = match read_config(&state).await {
        Ok(config) => config,
        Err(err) => return config_read_failure(err),
    };

    // Distinguish "not base64/XML at all" (a client bug -> 400) from the real
    // blocker, which is the missing verifier (-> 503).
    let decoded_ok = base64::engine::general_purpose::STANDARD
        .decode(saml_response.trim())
        .map(|bytes| String::from_utf8_lossy(&bytes).contains("<"))
        .unwrap_or_else(|_| saml_response.trim().starts_with('<'));
    if !decoded_ok {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "detail": "SAMLResponse is neither valid Base64 containing XML nor raw XML.",
            })),
        )
            .into_response();
    }

    let blocked = json!({
        "detail": "The SAML assertion was received but NOT processed: this build cannot validate an XML-DSig enveloped signature, and accepting an unverified assertion would let anyone sign in as any user. No session was issued and no user was provisioned.",
        "status": "unsupported",
        "live_signature_check": false,
        "receivedBytes": saml_response.trim().len(),
        "relayState": relay_state,
        "configuredProvider": config.provider_type,
        "wantAssertionsSigned": true,
        "missingDependencies": MISSING_SAML_DEPENDENCIES,
        "acsUrl": absolute_acs(&config, &base_url(&headers)),
        "unblock": "Add the crates above to backend-rs/Cargo.toml, then implement verify_assertion() against the stored idp_x509_cert (also check Audience, Recipient, Destination, and NotBefore/NotOnOrAfter).",
    });

    (StatusCode::SERVICE_UNAVAILABLE, Json(blocked)).into_response()
}

// ---------------------------------------------------------------------------
// Diagnostics
// ---------------------------------------------------------------------------

/// Validates the stored configuration and then actually reaches the IdP. The
/// reachability result is the provider's real HTTP answer; signature handling
/// is reported as unavailable rather than "verified".
pub async fn test_connection(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<Value>,
) -> Response {
    let config = match read_config(&state).await {
        Ok(config) => config,
        Err(err) => return config_read_failure(err),
    };
    let base = base_url(&headers);
    let now = Utc::now().to_rfc3339();

    let idp_sso_url = config.idp_sso_url.clone().unwrap_or_default();
    let idp_entity_id = config.idp_entity_id.clone().unwrap_or_default();
    let cert = config.idp_x509_cert.clone().unwrap_or_default();
    let sp_entity_id = config.sp_entity_id.clone().unwrap_or_default();
    let enabled = config.is_enabled.unwrap_or(false);

    let mut checks = vec![json!({
        "name": "SSO Master Status",
        "passed": enabled,
        "message": if enabled { "SSO is enabled in the stored configuration." } else { "SSO is currently disabled." },
    })];

    let url_valid = idp_sso_url.starts_with("https://") || idp_sso_url.starts_with("http://");
    checks.push(json!({
        "name": "Identity Provider SSO URL",
        "passed": url_valid,
        "message": if url_valid { idp_sso_url.clone() } else { "Missing or invalid IdP HTTP/HTTPS URL.".to_string() },
    }));
    checks.push(json!({
        "name": "Identity Provider Entity ID",
        "passed": !idp_entity_id.trim().is_empty(),
        "message": if idp_entity_id.trim().is_empty() { "Missing IdP Entity ID / Issuer URI.".to_string() } else { idp_entity_id.clone() },
    }));
    let cert_present = cert.contains("CERTIFICATE");
    checks.push(json!({
        "name": "X.509 Certificate",
        "passed": cert_present,
        "message": if cert_present { "PEM certificate stored; it cannot be parsed or used for verification in this build (no XML-DSig verifier)." } else { "No PEM certificate stored, so assertions could not be verified even with the parser crates added." },
    }));
    checks.push(json!({
        "name": "Service Provider Entity ID",
        "passed": !sp_entity_id.trim().is_empty(),
        "message": if sp_entity_id.trim().is_empty() { "Missing SP Entity ID.".to_string() } else { sp_entity_id.clone() },
    }));

    // The one live check we can honestly perform: is the IdP endpoint there?
    let (reachability, probe_passed) = if !url_valid {
        (
            "No HTTP request was made because the IdP SSO URL is not set.".to_string(),
            false,
        )
    } else {
        match state
            .http_client
            .get(&idp_sso_url)
            .timeout(std::time::Duration::from_secs(8))
            .send()
            .await
        {
            Ok(resp) => {
                let status = resp.status();
                (
                    format!("GET {idp_sso_url} answered HTTP {}.", status.as_u16()),
                    status.is_success() || status.is_redirection() || status.as_u16() == 405,
                )
            }
            Err(err) => (format!("GET {idp_sso_url} failed: {err}"), false),
        }
    };
    checks.push(json!({
        "name": "IdP Endpoint Reachability",
        "passed": probe_passed,
        "message": reachability,
    }));

    // A supplied sample assertion can only be decoded, never trusted.
    let sample = payload
        .get("sample_xml")
        .or_else(|| payload.get("sampleXml"))
        .and_then(|v| v.as_str())
        .map(|s| s.trim().to_string());
    if let Some(sample) = sample {
        let looks_like_response = sample.contains("Assertion") || sample.contains("samlp:Response");
        checks.push(json!({
            "name": "Sample Assertion Decode",
            "passed": false,
            "message": format!(
                "The sample {} base64/XML-shaped input, but it was NOT validated: no signature or audience check is available without the crates listed in this response.",
                if looks_like_response { "looks like a SAML" } else { "does not look like a SAML" }
            ),
        }));
    }

    let config_errors = !url_valid || idp_entity_id.trim().is_empty();
    let status = if config_errors {
        "error"
    } else if !probe_passed || !enabled {
        "warning"
    } else {
        // Even a reachable IdP is not a working login until assertions verify.
        "warning"
    };
    let message = if config_errors {
        "SSO configuration is incomplete; no login would succeed.".to_string()
    } else {
        "Configuration stored and the IdP endpoint was reached. Logins are NOT yet possible: assertion signature verification is not implemented (see missingDependencies).".to_string()
    };

    // Recording the probe outcome is a write, so its failure is reported instead
    // of being ignored with `let _ =`.
    let test_result_persistence = if config.id.is_empty() {
        json!({
            "ok": false,
            "reason": "No configuration row exists yet, so this probe was not recorded. Save the SSO configuration first.",
        })
    } else {
        match sqlx::query(
            "UPDATE sso_configs SET last_tested_at = ?, last_test_status = ?, last_test_message = ? WHERE id = ?",
        )
        .bind(&now)
        .bind(status)
        .bind(&message)
        .bind(&config.id)
        .execute(&state.db)
        .await
        {
            Ok(res) if res.rows_affected() == 1 => json!({ "ok": true, "rowId": config.id }),
            Ok(res) => json!({
                "ok": false,
                "reason": format!("The UPDATE matched {} rows for id {}, so this probe was not recorded.", res.rows_affected(), config.id),
            }),
            Err(err) => json!({ "ok": false, "reason": err.to_string() }),
        }
    };

    Json(json!({
        "status": status,
        "message": message,
        "tested_at": now,
        "checks": checks,
        "idpReachable": probe_passed,
        "assertionVerification": "unavailable",
        "missingDependencies": MISSING_SAML_DEPENDENCIES,
        "metadataUrl": format!("{}/api/v1/sso/metadata.xml", base.trim_end_matches('/')),
        "acsUrl": absolute_acs(&config, &base),
        "testResultPersistence": test_result_persistence,
    }))
    .into_response()
}

/// Parity of `/sso/check`: does this email's domain have to use SSO?
#[derive(Deserialize)]
pub struct CheckEmailPayload {
    #[serde(default)]
    email: Option<String>,
}

pub async fn check_email(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<CheckEmailPayload>,
) -> Response {
    let email = payload.email.unwrap_or_default().trim().to_lowercase();
    let domain = email.split('@').nth(1).unwrap_or("");
    let config = match read_config(&state).await {
        Ok(config) => config,
        Err(err) => return config_read_failure(err),
    };
    let base = base_url(&headers);

    let enabled = config.is_enabled.unwrap_or(false);
    let enforced = parse_string_array(config.enforced_domains.clone())
        .into_iter()
        .map(|d| d.to_lowercase())
        .collect::<Vec<String>>();
    let domain_enforced = enabled && !domain.is_empty() && enforced.iter().any(|d| d == domain);

    Json(json!({
        "email": email,
        "domain": domain,
        "sso_enabled": enabled,
        "ssoRequired": domain_enforced,
        "enforcedDomains": enforced,
        "providerType": config.provider_type,
        "loginUrl": if domain_enforced { format!("{}/api/v1/sso/login?email={}", base.trim_end_matches('/'), urlencode(&email)) } else { String::new() },
        "idpSsoUrl": config.idp_sso_url,
        // Null means the operator never chose, which the client must treat as
        // "not allowed" rather than as an enabled fallback.
        "allowUnlinkedPasswords": config.allow_unlinked_passwords,
        "jitProvisioningEnabled": config.jit_provisioning_enabled,
        "note": if domain_enforced {
            "SSO is required for this domain, but the ACS endpoint cannot verify assertions in this build yet (see /sso/test)."
        } else if !enabled {
            "Single Sign-On is not enabled, so this address uses password login."
        } else {
            "This domain is not in the enforced list, so password login remains allowed."
        },
    }))
    .into_response()
}
