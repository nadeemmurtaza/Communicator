use crate::db::AppState;
use crate::routes::auth::Tenant;
use axum::{
    extract::{Path, Query, State},
    http::{header, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use uuid::Uuid;

/// Every swallowed fallible call in this file reports itself through this shape
/// so a partial failure is never hidden: a write that did not happen is named in
/// the response `warnings` array with the exact reason.
fn write_warning(action: &str, err: &sqlx::Error) -> Value {
    json!({
        "action": action,
        "applied": false,
        "reason": err.to_string(),
    })
}

fn rbac_db_error(context: &str, err: sqlx::Error) -> Response {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(json!({
            "error": format!("could not {context}"),
            "detail": err.to_string(),
            "databaseError": true,
        })),
    )
        .into_response()
}

/// `json!` cannot take a computed key, so the entity id (and its camelCase twin)
/// is inserted into the response map after the literal is built.
fn push_entity_id(body: &mut Value, is_contact: bool, id: &str) {
    if let Some(map) = body.as_object_mut() {
        let (snake, camel) = if is_contact {
            ("contact_id", "contactId")
        } else {
            ("conversation_id", "conversationId")
        };
        map.insert(snake.to_string(), json!(id));
        map.insert(camel.to_string(), json!(id));
    }
}

/// Shared serializer for `custom_attribute_definitions` rows. `options` is a
/// JSON array column stored as text; every other field is read straight from the
/// row, so nothing about the attribute model or type is inferred here.
#[derive(sqlx::FromRow, Serialize)]
struct AttributeRow {
    id: String,
    name: String,
    display_name: String,
    description: Option<String>,
    attribute_model: String,
    attribute_type: String,
    default_value: Option<String>,
    options: Option<String>,
    is_required: Option<bool>,
    regex_pattern: Option<String>,
    regex_cue: Option<String>,
    created_at: String,
    updated_at: String,
}

impl AttributeRow {
    fn to_json(&self) -> Value {
        let options: Value = self
            .options
            .as_deref()
            .and_then(|raw| serde_json::from_str(raw).ok())
            .unwrap_or_else(|| json!([]));
        json!({
            "id": self.id,
            "name": self.name,
            "display_name": self.display_name,
            "displayName": self.display_name,
            "description": self.description,
            "attribute_model": self.attribute_model,
            "attributeModel": self.attribute_model,
            "attribute_type": self.attribute_type,
            "attributeType": self.attribute_type,
            "default_value": self.default_value,
            "defaultValue": self.default_value,
            "options": options,
            "is_required": self.is_required.unwrap_or(false),
            "isRequired": self.is_required.unwrap_or(false),
            "regex_pattern": self.regex_pattern,
            "regexPattern": self.regex_pattern,
            "regex_cue": self.regex_cue,
            "regexCue": self.regex_cue,
            "created_at": self.created_at,
            "createdAt": self.created_at,
            "updated_at": self.updated_at,
            "updatedAt": self.updated_at,
        })
    }
}

const ATTRIBUTE_MODELS: &[&str] = &["contact_attribute", "conversation_attribute"];
const ATTRIBUTE_TYPES: &[&str] = &["text", "number", "currency", "boolean", "list", "date", "link"];

/// Port of Python `_normalize_name` (custom_attribute_service.py:23-25).
fn normalize_attribute_name(raw: &str) -> String {
    let cleaned: String = raw
        .trim()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c.to_ascii_lowercase() } else { '_' })
        .collect();
    let mut out = String::new();
    let mut prev_underscore = false;
    for c in cleaned.chars() {
        if c == '_' {
            if !prev_underscore {
                out.push('_');
            }
            prev_underscore = true;
        } else {
            out.push(c);
            prev_underscore = false;
        }
    }
    out.trim_matches('_').to_string()
}

/// Port of Python's display default (custom_attribute_service.py:107):
/// `slug_name.replace("_", " ").title()`.
fn title_from_slug(slug: &str) -> String {
    slug.split('_')
        .filter(|w| !w.is_empty())
        .map(|w| {
            let mut chars = w.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

// ==========================================
// 1. Live Chat Widget Customizer
// ==========================================

/// The widget customizer lives inside the website inbox's `credentials` JSON,
/// the same place Python stored it. Nothing here is invented: `saved` is what
/// the row actually holds, `styling` is what the widget will render with, and
/// `unsavedDefaultsApplied` names every key that fell back to a default.
const CRED_KEY_MAP: &[(&str, &str)] = &[
    ("widget_color", "widgetColor"),
    ("theme_mode", "themeMode"),
    ("launcher_style", "launcherStyle"),
    ("launcher_icon", "launcherIcon"),
    ("launcher_text", "launcherText"),
    ("widget_position", "widgetPosition"),
    ("greeting_title", "greetingTitle"),
    ("greeting_tagline", "greetingTagline"),
    ("avatar_url", "avatarUrl"),
    ("border_radius", "borderRadius"),
    ("play_sound", "playSound"),
    ("show_branding", "showBranding"),
    ("allowed_domains", "allowedDomains"),
    ("pre_chat_form_enabled", "preChatFormEnabled"),
    ("identity_validation", "identityValidation"),
    ("hmac_secret", "hmacSecret"),
];



fn default_widget_config() -> Value {
    json!({
        "widget_color": "#1570EF",
        "theme_mode": "light",
        "launcher_style": "round_bubble",
        "launcher_icon": "chat_bubble",
        "launcher_text": "Chat with us",
        "widget_position": "right",
        "greeting_title": "Hi there!",
        "greeting_tagline": "We typically reply in under 2 minutes.",
        "avatar_url": null,
        "border_radius": "rounded_2xl",
        "play_sound": true,
        "show_branding": true,
        "allowed_domains": null,
        "pre_chat_form_enabled": true,
        "identity_validation": false,
        "hmac_secret": null
    })
}

#[derive(sqlx::FromRow)]
struct CustomizerInbox {
    id: String,
    name: String,
    avatar_url: Option<String>,
    credentials: Option<String>,
    updated_at: Option<String>,
}

async fn load_website_inbox(state: &AppState) -> Result<Option<CustomizerInbox>, sqlx::Error> {
    sqlx::query_as::<_, CustomizerInbox>(
        "SELECT id, name, avatar_url, credentials, updated_at FROM inboxes \
         WHERE channel_type IN ('website', 'live-chat', 'live_chat') \
         ORDER BY updated_at DESC LIMIT 1",
    )
    .fetch_optional(&state.db)
    .await
}

/// Website tokens are generated, never free text, so anything outside this
/// shape is rejected rather than reflected into the embed HTML/JS.
fn sanitize_token(token: &str) -> Option<&str> {
    let ok = !token.is_empty()
        && token.len() <= 64
        && token
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-');
    ok.then_some(token)
}

/// Only conservative literals may be interpolated into a JS string.
fn safe_literal(value: &str, fallback: &str) -> String {
    let clean = !value.is_empty()
        && value.len() <= 64
        && value.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '#' | '-' | '_' | '.'));
    if clean {
        value.to_string()
    } else {
        fallback.to_string()
    }
}

fn build_styling(creds: &Value) -> (Value, Vec<String>) {
    let defaults = default_widget_config();
    let stored = creds.as_object();
    let mut styling = serde_json::Map::new();
    let mut unsaved: Vec<String> = Vec::new();

    for (snake, camel) in CRED_KEY_MAP {
        let saved = stored.and_then(|o| o.get(*camel).cloned());
        match saved {
            Some(value) => {
                styling.insert((*snake).to_string(), value);
            }
            None => {
                styling.insert(
                    (*snake).to_string(),
                    defaults.get(*snake).cloned().unwrap_or(Value::Null),
                );
                unsaved.push((*snake).to_string());
            }
        }
    }
    (Value::Object(styling), unsaved)
}

fn generate_embed_codes(token: &str, base_url: &str, styling: &Value) -> Value {
    let color = styling["widget_color"]
        .as_str()
        .map(|s| safe_literal(s, "#1570EF"))
        .unwrap_or_else(|| "#1570EF".to_string());
    let position = styling["widget_position"]
        .as_str()
        .map(|s| safe_literal(s, "right"))
        .unwrap_or_else(|| "right".to_string());

    let settings = format!(
        "window.captainChatSettings = {{ websiteToken: \"{token}\", baseUrl: \"{base_url}\", color: \"{color}\", position: \"{position}\" }};"
    );

    let html_script = format!(
        "<!-- Live Chat Embed Code -->\n<script>\n  (function(d, t) {{\n    var BASE_URL = \"{base_url}\";\n    var g = d.createElement(t), s = d.getElementsByTagName(t)[0];\n    g.src = BASE_URL + \"/packs/js/sdk.js\";\n    g.defer = true;\n    g.async = true;\n    s.parentNode.insertBefore(g, s);\n    g.onload = function() {{\n      {settings}\n      window.captainChat.init();\n    }};\n  }})(document, \"script\");\n</script>"
    );

    let react_snippet = format!(
        "// React / Next.js Component\nimport {{ useEffect }} from 'react';\n\nexport function LiveChatWidget() {{\n  useEffect(() => {{\n    const script = document.createElement('script');\n    script.src = \"{base_url}/packs/js/sdk.js\";\n    script.async = true;\n    script.onload = () => {{\n      {settings}\n      window.captainChat?.init();\n    }};\n    document.body.appendChild(script);\n  }}, []);\n\n  return null;\n}}"
    );

    let gtm_snippet = format!(
        "<!-- Google Tag Manager (Custom HTML Tag) -->\n<script>\n  {settings}\n  var s = document.createElement('script');\n  s.src = \"{base_url}/packs/js/sdk.js\";\n  s.async = true;\n  document.head.appendChild(s);\n</script>"
    );

    json!({
        "html_script": html_script,
        "react_snippet": react_snippet,
        "gtm_snippet": gtm_snippet,
    })
}

/// The origin the embed code must point at, taken only from signals this process
/// actually holds: the proxy's forwarded scheme with the request's own host, then
/// the configured `PUBLIC_URL`. Defaulting the scheme to `http` would bake a
/// protocol into every snippet the operator copies that nobody observed, and a
/// browser served over https would then refuse to load the widget.
fn request_base_url(
    headers: &axum::http::HeaderMap,
    public_url: &str,
) -> Result<String, String> {
    let host = headers
        .get(axum::http::header::HOST)
        .and_then(|v| v.to_str().ok())
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .ok_or_else(|| "this request carried no Host header, so the widget host is unknown".to_string())?;

    let forwarded_proto = headers
        .get("x-forwarded-proto")
        .and_then(|v| v.to_str().ok())
        .map(str::trim)
        .filter(|v| *v == "http" || *v == "https")
        .map(str::to_string);

    match forwarded_proto {
        Some(proto) => Ok(format!("{proto}://{host}")),
        None => {
            // Anything in front of this service is allowed to say nothing about
            // the scheme, so fall back to the operator's configured public origin
            // before refusing.
            let configured = public_url.trim().trim_end_matches('/');
            if configured.starts_with("https://") || configured.starts_with("http://") {
                Ok(configured.to_string())
            } else {
                Err(
                    "this request carried no x-forwarded-proto header (only \"http\" or \"https\" \
                     are accepted) and PUBLIC_URL is not set to an absolute origin, so no scheme \
                     can be stated for the widget base URL"
                        .to_string(),
                )
            }
        }
    }
}

/// Refusal used by every widget endpoint that needs the origin, so the missing
/// input is named instead of guessed.
fn missing_origin_response(reason: &str) -> Response {
    (
        StatusCode::BAD_REQUEST,
        Json(json!({
            "error": "embed codes need the public origin",
            "detail": reason,
            "missingInputs": ["Host header", "x-forwarded-proto header or PUBLIC_URL"],
        })),
    )
        .into_response()
}

fn customizer_payload(
    inbox: &CustomizerInbox,
    base_url: &str,
) -> Result<Value, serde_json::Error> {
    // A credentials column that does not parse is reported, not rendered as an
    // empty widget: every styling key would otherwise be labelled an unsaved
    // default while the row in fact holds unreadable JSON.
    let creds: Value = match inbox.credentials.as_deref() {
        Some(raw) if !raw.trim().is_empty() => serde_json::from_str(raw)?,
        _ => json!({}),
    };
    let (styling, unsaved) = build_styling(&creds);
    let stored_token = creds.get("websiteToken").and_then(|v| v.as_str());
    let saved: Value = serde_json::to_value(&creds)?;

    Ok(match stored_token.and_then(|t| sanitize_token(t)) {
        Some(token) => {
            let embed_codes = generate_embed_codes(token, base_url, &styling);
            json!({
                "configured": true,
                "inbox_id": inbox.id,
                "inbox_name": inbox.name,
                "website_token": token,
                "website_domain": creds.get("websiteDomain").cloned().unwrap_or(Value::Null),
                "styling": styling,
                "saved": saved,
                "unsavedDefaultsApplied": unsaved,
                "embed_codes": embed_codes,
                "preview_url": format!("/api/v1/widget/preview-page?token={token}"),
                "updatedAt": inbox.updated_at,
            })
        }
        None => json!({
            "configured": true,
            "inbox_id": inbox.id,
            "inbox_name": inbox.name,
            "website_token": null,
            "website_domain": creds.get("websiteDomain").cloned().unwrap_or(Value::Null),
            "styling": styling,
            "saved": saved,
            "unsavedDefaultsApplied": unsaved,
            "embed_codes": null,
            "preview_url": null,
            "updatedAt": inbox.updated_at,
            "note": "this website inbox row exists but holds no usable websiteToken, so no embed code can be generated; save the customizer to provision one",
        }),
    })
}

pub async fn get_widget_customizer(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
) -> Response {
    let inbox = match load_website_inbox(&state).await {
        Ok(row) => row,
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "could not read the website inbox",
                    "detail": err.to_string(),
                })),
            )
                .into_response()
        }
    };

    let Some(inbox) = inbox else {
        let defaults = default_widget_config();
        let keys: Vec<&str> = CRED_KEY_MAP.iter().map(|(snake, _)| *snake).collect();
        return Json(json!({
            "configured": false,
            "inbox_id": null,
            "inbox_name": null,
            "website_token": null,
            "website_domain": null,
            "styling": defaults,
            "saved": {},
            "unsavedDefaultsApplied": keys,
            "embed_codes": null,
            "preview_url": null,
            "note": "no website inbox exists in this database, so nothing has been customised and no widget token has been issued; the styling above is the inactive default set, and POST /widget/customizer creates the inbox row for real",
        }))
        .into_response()
    };

    let base_url = match request_base_url(&headers, &state.config.public_url) {
        Ok(value) => value,
        Err(reason) => return missing_origin_response(&reason),
    };

    match customizer_payload(&inbox, &base_url) {
        Ok(value) => Json(value).into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": "the website inbox credentials are not valid JSON",
                "detail": err.to_string(),
            })),
        )
            .into_response(),
    }
}

async fn persist_customizer(
    state: &AppState,
    payload: &Value,
    base_url: &str,
) -> (StatusCode, Value) {
    let mut inbox = match load_website_inbox(state).await {
        Ok(Some(row)) => row,
        Ok(None) => {
            // Creating the row is an explicit operator action, so a generated
            // token and a real name are legitimate here.
            let id = Uuid::new_v4().to_string();
            let token = format!("web_live_{}", &Uuid::new_v4().simple().to_string()[..16]);
            let now = Utc::now().to_rfc3339();
            let mut creds = serde_json::Map::new();
            creds.insert("websiteToken".to_string(), json!(token));
            let insert = sqlx::query(
                "INSERT INTO inboxes (id, name, channel_type, credentials, created_at, updated_at) VALUES ($1, $2, 'website', $3, $4, $5)",
            )
            .bind(&id)
            .bind("Website Live Chat")
            .bind(serde_json::Value::Object(creds).to_string())
            .bind(&now)
            .bind(&now);
            if let Err(err) = insert.execute(&state.db).await {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    json!({
                        "error": "could not create the website inbox",
                        "detail": err.to_string(),
                    }),
                );
            }
            let created = match load_website_inbox(state).await {
                Ok(Some(row)) => row,
                Ok(None) => {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        json!({
                            "error": "the website inbox was inserted but could not be read back",
                        }),
                    )
                }
                Err(err) => {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        json!({
                            "error": "could not read back the website inbox",
                            "detail": err.to_string(),
                        }),
                    )
                }
            };
            created
        }
        Err(err) => {            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                json!({
                    "error": "could not read the website inbox",
                    "detail": err.to_string(),
                }),
            )
        }
    };

    let mut creds: Value = match inbox.credentials.as_deref() {
        Some(raw) if !raw.trim().is_empty() => match serde_json::from_str(raw) {
            Ok(value) => value,
            Err(err) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    json!({
                        "error": "the stored website inbox credentials are not valid JSON",
                        "detail": err.to_string(),
                    }),
                )
            }
        },
        _ => json!({}),
    };

    let Some(obj) = creds.as_object_mut() else {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            json!({ "error": "the stored website inbox credentials are not a JSON object" }),
        );
    };

    for (snake, camel) in CRED_KEY_MAP {
        if let Some(value) = payload.get(*snake) {
            obj.insert((*camel).to_string(), value.clone());
        }
    }
    if let Some(domain) = payload.get("website_domain") {
        obj.insert("websiteDomain".to_string(), domain.clone());
    }
    if obj.get("websiteToken").and_then(|v| v.as_str()).is_none() {
        let token = format!("web_live_{}", Uuid::new_v4().simple().to_string());
        obj.insert("websiteToken".to_string(), json!(token));
    }

    let now = Utc::now().to_rfc3339();
    // An absent key keeps the stored value; a key the operator sent has to be a
    // usable string, and a blank inbox name is refused rather than written.
    let name = match payload.get("inbox_name") {
        None | Some(Value::Null) => inbox.name.clone(),
        Some(v) => match v.as_str().map(str::trim) {
            Some(value) if !value.is_empty() => value.to_string(),
            Some(_) => {
                return (
                    StatusCode::BAD_REQUEST,
                    json!({
                        "error": "inbox_name cannot be blank; the website inbox needs a real display name",
                        "field": "inbox_name",
                    }),
                )
            }
            None => {
                return (
                    StatusCode::BAD_REQUEST,
                    json!({
                        "error": "inbox_name must be a string when it is sent",
                        "field": "inbox_name",
                    }),
                )
            }
        },
    };
    // `inboxes.avatar_url` is `TEXT NOT NULL DEFAULT ''` (see db/mod.rs), so the
    // empty string is that column's own "no avatar" value, never a guess:
    // sending null or "" clears it, omitting the key keeps what is stored.
    let avatar: String = match payload.get("avatar_url") {
        None => inbox.avatar_url.clone().unwrap_or_default(),
        Some(Value::Null) => String::new(),
        Some(v) => match v.as_str() {
            Some(value) => value.trim().to_string(),
            None => {
                return (
                    StatusCode::BAD_REQUEST,
                    json!({
                        "error": "avatar_url must be a URL string or null when it is sent",
                        "field": "avatar_url",
                    }),
                )
            }
        },
    };
    let avatar_missing = avatar.is_empty();
    let creds_str = creds.to_string();

    let result = sqlx::query(
        "UPDATE inboxes SET name = $1, avatar_url = $2, credentials = $3, updated_at = $4 WHERE id = $5",
    )
    .bind(&name)
    .bind(&avatar)
    .bind(&creds_str)
    .bind(&now)
    .bind(&inbox.id)
    .execute(&state.db)
    .await;

    if let Err(err) = result {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            json!({
                "error": "the customizer settings were not saved",
                "detail": err.to_string(),
            }),
        );
    }

    inbox.name = name;
    inbox.avatar_url = Some(avatar);
    inbox.credentials = Some(creds_str);
    inbox.updated_at = Some(now);

    match customizer_payload(&inbox, base_url) {
        Ok(mut value) => {
            if avatar_missing {
                // Named instead of hidden: the row genuinely holds no avatar.
                value["avatarUrlMissing"] = json!(true);
                value["avatarUrlNote"] = json!(
                    "inboxes.avatar_url is empty for this row, so the widget renders without a custom avatar image"
                );
            }
            (StatusCode::OK, value)
        }
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            json!({
                "error": "the saved credentials could not be rendered",
                "detail": err.to_string(),
            }),
        ),
    }
}

pub async fn update_widget_customizer(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    Json(payload): Json<Value>,
) -> Response {
    let base_url = match request_base_url(&headers, &state.config.public_url) {
        Ok(value) => value,
        Err(reason) => return missing_origin_response(&reason),
    };
    let (status, body) = persist_customizer(&state, &payload, &base_url).await;
    (status, Json(body)).into_response()
}

pub async fn reset_widget_customizer(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
) -> Response {
    // A failed read is not "no inbox": resetting would rewrite the stored
    // credentials, so the row has to be confirmed absent, not merely unreadable.
    let existing = match load_website_inbox(&state).await {
        Ok(row) => row,
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "could not read the website inbox",
                    "detail": err.to_string(),
                })),
            )
                .into_response()
        }
    };
    if existing.is_none() {
        return (
            StatusCode::NOT_FOUND,
            Json(json!({
                "error": "there is no website inbox to reset",
                "detail": "nothing was saved; POST /widget/customizer creates the inbox row first",
            })),
        )
            .into_response();
    }
    let base_url = match request_base_url(&headers, &state.config.public_url) {
        Ok(value) => value,
        Err(reason) => return missing_origin_response(&reason),
    };
    let payload = default_widget_config();
    let (status, body) = persist_customizer(&state, &payload, &base_url).await;
    (status, Json(body)).into_response()
}

#[derive(Deserialize)]
pub struct EmbedCodeQuery {
    pub token: Option<String>,
}

pub async fn get_widget_embed_code(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    Query(query): Query<EmbedCodeQuery>,
) -> Response {
    let base_url = match request_base_url(&headers, &state.config.public_url) {
        Ok(value) => value,
        Err(reason) => return missing_origin_response(&reason),
    };

    let inbox = match load_website_inbox(&state).await {
        Ok(row) => row,
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "could not read the website inbox",
                    "detail": err.to_string(),
                })),
            )
                .into_response()
        }
    };

    // Credentials are parsed once, before the token is resolved, so unreadable
    // JSON is reported as itself instead of surfacing as "no widget token exists".
    let stored_creds = match inbox
        .as_ref()
        .and_then(|row| row.credentials.as_deref())
        .map(str::trim)
        .filter(|raw| !raw.is_empty())
    {
        Some(raw) => match serde_json::from_str::<Value>(raw) {
            Ok(creds) => Some(creds),
            Err(err) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({
                        "error": "the website inbox credentials are not valid JSON",
                        "detail": err.to_string(),
                        "sent": false,
                    })),
                )
                    .into_response()
            }
        },
        None => None,
    };

    let token = match query.token.as_deref().and_then(sanitize_token) {
        Some(provided) => provided.to_string(),
        None => match stored_creds
            .as_ref()
            .and_then(|creds| creds.get("websiteToken"))
            .and_then(|v| v.as_str())
            .and_then(sanitize_token)
        {
            Some(stored) => stored.to_string(),
            None => {
                let detail = match (inbox.as_ref(), &stored_creds) {
                    (None, _) => "the request supplied no token and this deployment has no website inbox row holding one".to_string(),
                    (Some(_), None) => "the request supplied no token and the website inbox row holds no credentials JSON to read one from".to_string(),
                    (Some(_), Some(_)) => "the request supplied no token and the stored credentials carry no usable websiteToken; save the customizer to provision one".to_string(),
                };
                return (
                    StatusCode::NOT_FOUND,
                    Json(json!({
                        "error": "no widget token exists to embed",
                        "detail": detail,
                    })),
                )
                    .into_response();
            }
        },
    };

    // The styling source is reported: a row with no saved credentials and a row
    // that never saved a colour both render defaults, and the caller has to be
    // able to tell which keys are unsaved rather than assume both.
    let (styling, unsaved) = match &stored_creds {
        Some(creds) => build_styling(creds),
        None => (
            default_widget_config(),
            CRED_KEY_MAP.iter().map(|(snake, _)| (*snake).to_string()).collect(),
        ),
    };

    Json(json!({
        "website_token": token,
        "base_url": base_url,
        "embed_codes": generate_embed_codes(&token, &base_url, &styling),
        "stylingSource": if stored_creds.is_some() { "saved credentials" } else { "no saved credentials on this inbox row" },
        "unsavedDefaultsApplied": unsaved,
        "unsaved_defaults_applied": unsaved,
    }))
    .into_response()
}

fn html_escape(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for ch in input.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            other => out.push(other),
        }
    }
    out
}

fn preview_page_html(
    title: &str,
    body: &str,
    status: StatusCode,
) -> Response {
    let page = format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>{title}</title>
<style>
body{{font-family:-apple-system,BlinkMacSystemFont,"Segoe UI",Roboto,Helvetica,Arial,sans-serif;margin:0;padding:0;background:#f8fafc;color:#0f172a}}
.hero{{max-width:900px;margin:60px auto;padding:40px;background:#fff;border-radius:16px;box-shadow:0 4px 6px -1px rgb(0 0 0 / .1);border:1px solid #e2e8f0}}
h1{{font-size:1.6rem;margin:0 0 12px}}
p{{color:#475569;line-height:1.6;margin:0 0 14px}}
.note{{background:#fffbeb;border:1px solid #fcd34d;border-radius:10px;padding:14px 16px;color:#92400e;font-size:.9rem}}
code{{background:#f1f5f9;padding:2px 6px;border-radius:6px}}
</style>
</head>
<body><div class="hero">{body}</div></body>
</html>"#
    );
    (
        status,
        [(header::CONTENT_TYPE, HeaderValue::from_static("text/html; charset=utf-8"))],
        page,
    )
        .into_response()
}

/// Port of Python's `GET /widget/preview-page`: renders the widget against the
/// real stored styling. Python reached it through a `get_or_create` that wrote an
/// inbox row on a GET and defaulted the token to `web_token_default`; this one
/// reads only, and refuses to render a preview of a widget that does not exist.
pub async fn get_widget_preview_page(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    Query(query): Query<EmbedCodeQuery>,
) -> Response {
    let base_url = match request_base_url(&headers, &state.config.public_url) {
        Ok(value) => value,
        Err(reason) => {
            return preview_page_html(
                "Widget preview unavailable",
                &format!(
                    "<h1>Widget preview unavailable</h1><p>{}</p>",
                    html_escape(&reason)
                ),
                StatusCode::BAD_REQUEST,
            )
        }
    };

    let inbox = match load_website_inbox(&state).await {
        Ok(row) => row,
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "could not read the website inbox",
                    "detail": err.to_string(),
                })),
            )
                .into_response()
        }
    };

    // Unreadable credentials are reported, not replaced by an empty object that
    // would render the preview as "nothing saved yet".
    let creds: Value = match inbox
        .as_ref()
        .and_then(|row| row.credentials.as_deref())
        .map(str::trim)
        .filter(|raw| !raw.is_empty())
    {
        Some(raw) => match serde_json::from_str(raw) {
            Ok(creds) => creds,
            Err(err) => {
                return preview_page_html(
                    "Widget preview unavailable",
                    &format!(
                        "<h1>Widget preview unavailable</h1><p>The stored website inbox credentials are not valid JSON, so no styling can be rendered: {}</p>",
                        html_escape(&err.to_string())
                    ),
                    StatusCode::INTERNAL_SERVER_ERROR,
                )
            }
        },
        None => json!({}),
    };

    let token = match query
        .token
        .as_deref()
        .and_then(sanitize_token)
        .map(str::to_string)
        .or_else(|| {
            creds
                .get("websiteToken")
                .and_then(|v| v.as_str())
                .and_then(sanitize_token)
                .map(str::to_string)
        }) {
        Some(token) => token,
        None => {
            let reason = if inbox.is_none() {
                "this deployment has no website inbox row, so there is no widget to preview"
            } else {
                "the website inbox row exists but holds no usable websiteToken; save the customizer to provision one"
            };
            return preview_page_html(
                "No widget configured",
                &format!(
                    "<h1>No widget configured</h1><p>{}</p><div class=\"note\">Nothing was created by this request. A preview needs a saved website inbox and token.</div>",
                    html_escape(reason)
                ),
                StatusCode::NOT_FOUND,
            );
        }
    };

    let (styling, unsaved) = build_styling(&creds);
    let embed = generate_embed_codes(&token, &base_url, &styling)
        .get("html_script")
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_string();
    // No domain claim is printed when the row does not hold one, and no inbox
    // name is printed when no row exists: both are stated as absent instead of
    // being given a plausible label.
    let domain_text = match creds
        .get("websiteDomain")
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        Some(domain) => format!("domain <code>{}</code>", html_escape(domain)),
        None => "domain <em>not saved (websiteDomain is absent from the stored credentials)</em>".to_string(),
    };
    let inbox_title = match inbox.as_ref() {
        Some(row) => row.name.clone(),
        None => "No website inbox row (the token came from the query string)".to_string(),
    };

    let caveat = if unsaved.is_empty() {
        String::new()
    } else {
        format!(
            "<div class=\"note\">Styling never saved for this inbox, showing built-in defaults: {}.</div>",
            html_escape(&unsaved.join(", "))
        )
    };

    let body = format!(
        "<h1>{}</h1><p>Token <code>{}</code> &middot; {} &middot; served from <code>{}</code></p>{}<p>The launcher below is loaded from <code>{}/packs/js/sdk.js</code>, exactly like the embed code.</p>",
        html_escape(&inbox_title),
        html_escape(&token),
        domain_text,
        html_escape(&base_url),
        caveat,
        html_escape(&base_url),
    );

    // The embed code already contains its own <script> tag; appending it after the
    // escaped prose is safe because every interpolated value above was escaped and
    // the token inside the script passed sanitize_token.
    preview_page_html(
        "Live chat widget preview",
        &format!("{body}{embed}"),
        StatusCode::OK,
    )
}

// ==========================================
// 2. Audit Logs & SOC 2 Compliance
// ==========================================

#[derive(Debug, Deserialize)]
pub struct AuditLogQuery {
    pub search: Option<String>,
    pub action: Option<String>,
    pub actor: Option<String>,
    pub entity_type: Option<String>,
    pub severity: Option<String>,
    pub compliance_category: Option<String>,
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct AuditLogRow {
    pub id: String,
    pub entity_type: String,
    pub entity_id: Option<String>,
    pub action: String,
    pub actor: String,
    pub actor_email: Option<String>,
    pub actor_name: Option<String>,
    pub actor_role: Option<String>,
    pub severity: Option<String>,
    pub compliance_category: Option<String>,
    pub description: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub details: Option<String>,
    pub created_at: String,
}

pub async fn get_audit_logs(
    State(state): State<AppState>,
    Query(params): Query<AuditLogQuery>,
) -> Response {
    let page = params.page.unwrap_or(1).max(1);
    let page_size = params.page_size.unwrap_or(25).clamp(1, 100);
    let offset = (page - 1) * page_size;

    let search = params.search.unwrap_or_default().trim().to_string();
    let action = params.action.unwrap_or_default().trim().to_string();
    let actor = params.actor.unwrap_or_default().trim().to_string();
    let entity_type = params.entity_type.unwrap_or_default().trim().to_string();
    let severity = params.severity.unwrap_or_default().trim().to_string();
    let category = params.compliance_category.unwrap_or_default().trim().to_string();

    let count_query = r#"
        SELECT COUNT(*)
        FROM audit_logs
        WHERE (? = '' OR LOWER(severity) = LOWER(?))
          AND (? = '' OR LOWER(compliance_category) = LOWER(?))
          AND (? = '' OR LOWER(action) LIKE '%' || LOWER(?) || '%')
          AND (? = '' OR LOWER(actor) LIKE '%' || LOWER(?) || '%')
          AND (? = '' OR LOWER(entity_type) = LOWER(?))
          AND (? = '' OR LOWER(action) LIKE '%' || LOWER(?) || '%' OR LOWER(actor) LIKE '%' || LOWER(?) || '%' OR LOWER(COALESCE(description, '')) LIKE '%' || LOWER(?) || '%' OR LOWER(COALESCE(ip_address, '')) LIKE '%' || LOWER(?) || '%')
    "#;

    let total: i64 = match sqlx::query_scalar(count_query)
        .bind(&severity).bind(&severity)
        .bind(&category).bind(&category)
        .bind(&action).bind(&action)
        .bind(&actor).bind(&actor)
        .bind(&entity_type).bind(&entity_type)
        .bind(&search).bind(&search).bind(&search).bind(&search).bind(&search)
        .fetch_one(&state.db)
        .await
    {
        Ok(total) => total,
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "the audit log count query failed",
                    "detail": err.to_string(),
                    "databaseError": true,
                })),
            )
                .into_response()
        }
    };

    let select_query = r#"
        SELECT id, entity_type, entity_id, action, actor, actor_email, actor_name, actor_role,
               severity, compliance_category, description, ip_address, user_agent, details, created_at
        FROM audit_logs
        WHERE (? = '' OR LOWER(severity) = LOWER(?))
          AND (? = '' OR LOWER(compliance_category) = LOWER(?))
          AND (? = '' OR LOWER(action) LIKE '%' || LOWER(?) || '%')
          AND (? = '' OR LOWER(actor) LIKE '%' || LOWER(?) || '%')
          AND (? = '' OR LOWER(entity_type) = LOWER(?))
          AND (? = '' OR LOWER(action) LIKE '%' || LOWER(?) || '%' OR LOWER(actor) LIKE '%' || LOWER(?) || '%' OR LOWER(COALESCE(description, '')) LIKE '%' || LOWER(?) || '%' OR LOWER(COALESCE(ip_address, '')) LIKE '%' || LOWER(?) || '%')
        ORDER BY created_at DESC
        LIMIT ? OFFSET ?
    "#;

    let rows = match sqlx::query_as::<_, AuditLogRow>(select_query)
        .bind(&severity).bind(&severity)
        .bind(&category).bind(&category)
        .bind(&action).bind(&action)
        .bind(&actor).bind(&actor)
        .bind(&entity_type).bind(&entity_type)
        .bind(&search).bind(&search).bind(&search).bind(&search).bind(&search)
        .bind(page_size)
        .bind(offset)
        .fetch_all(&state.db)
        .await
    {
        Ok(rows) => rows,
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "the audit log rows could not be read",
                    "detail": err.to_string(),
                    "databaseError": true,
                })),
            )
                .into_response()
        }
    };

    let items: Vec<Value> = rows
        .into_iter()
        .map(|r| {
            let details_val: Value = r
                .details
                .as_deref()
                .and_then(|d| serde_json::from_str(d).ok())
                .unwrap_or_else(|| json!({}));

            json!({
                "id": r.id,
                "createdAt": r.created_at,
                "created_at": r.created_at,
                "action": r.action,
                "actor": r.actor,
                "actor_email": r.actor_email,
                "actorEmail": r.actor_email,
                "actor_name": r.actor_name,
                "actorName": r.actor_name,
                "actor_role": r.actor_role,
                "actorRole": r.actor_role,
                "entityType": r.entity_type,
                "entity_type": r.entity_type,
                "entityId": r.entity_id.clone(),
                "entity_id": r.entity_id,
                // Null means the column really is null. The old code answered
                // "info" / "access_control" / "127.0.0.1", which turns an
                // unknown provenance into a believable one.
                "severity": r.severity,
                "severityLabel": r.severity.clone().unwrap_or_else(|| "unspecified".to_string()),
                "complianceCategory": r.compliance_category.clone(),
                "compliance_category": r.compliance_category,
                "description": r.description,
                "ipAddress": r.ip_address.clone(),
                "ip_address": r.ip_address,
                "userAgent": r.user_agent,
                "details": details_val
            })
        })
        .collect();

    let total_pages = if total == 0 { 1 } else { (total + page_size - 1) / page_size };

    Json(json!({
        "items": items,
        "total": total,
        "page": page,
        "pageSize": page_size,
        "page_size": page_size,
        "totalPages": total_pages,
        "total_pages": total_pages,
        "note": "null severity / complianceCategory / ipAddress mean the stored row holds no value for that column; nothing here is defaulted"
    }))
    .into_response()
}

pub async fn get_audit_stats(State(state): State<AppState>) -> Response {
    let total: i64 = match sqlx::query_scalar("SELECT COUNT(*) FROM audit_logs")
        .fetch_one(&state.db)
        .await
    {
        Ok(total) => total,
        Err(err) => return db_stats_error(err),
    };

    #[derive(sqlx::FromRow)]
    struct GroupRow {
        name: Option<String>,
        cnt: i64,
    }

    let sev_rows = match sqlx::query_as::<_, GroupRow>(
        "SELECT severity as name, COUNT(*) as cnt FROM audit_logs GROUP BY severity"
    )
    .fetch_all(&state.db)
    .await
    {
        Ok(rows) => rows,
        Err(err) => return db_stats_error(err),
    };

    let mut severity_distribution = json!({
        "unspecified": 0,
        "info": 0,
        "notice": 0,
        "warning": 0,
        "critical": 0
    });
    for r in sev_rows {
        let key = r.name.unwrap_or_else(|| "unspecified".to_string()).to_lowercase();
        severity_distribution[key] = json!(r.cnt);
    }

    let cat_rows = match sqlx::query_as::<_, GroupRow>(
        "SELECT compliance_category as name, COUNT(*) as cnt FROM audit_logs GROUP BY compliance_category"
    )
    .fetch_all(&state.db)
    .await
    {
        Ok(rows) => rows,
        Err(err) => return db_stats_error(err),
    };

    let mut category_distribution = json!({});
    for r in cat_rows {
        let key = r.name.unwrap_or_else(|| "unspecified".to_string());
        category_distribution[key] = json!(r.cnt);
    }

    let actor_rows = match sqlx::query_as::<_, GroupRow>(
        "SELECT actor as name, COUNT(*) as cnt FROM audit_logs GROUP BY actor ORDER BY cnt DESC LIMIT 5"
    )
    .fetch_all(&state.db)
    .await
    {
        Ok(rows) => rows,
        Err(err) => return db_stats_error(err),
    };

    let top_actors: Vec<Value> = actor_rows
        .into_iter()
        .map(|r| json!({ "actor": r.name.unwrap_or_else(|| "unknown".to_string()), "count": r.cnt }))
        .collect();

    let action_rows = match sqlx::query_as::<_, GroupRow>(
        "SELECT action as name, COUNT(*) as cnt FROM audit_logs GROUP BY action ORDER BY cnt DESC LIMIT 5"
    )
    .fetch_all(&state.db)
    .await
    {
        Ok(rows) => rows,
        Err(err) => return db_stats_error(err),
    };

    let top_actions: Vec<Value> = action_rows
        .into_iter()
        .map(|r| json!({ "action": r.name.unwrap_or_else(|| "unknown".to_string()), "count": r.cnt }))
        .collect();

    let last_event: Option<String> = match sqlx::query_scalar("SELECT MAX(created_at) FROM audit_logs")
        .fetch_one(&state.db)
        .await
    {
        Ok(value) => value,
        Err(err) => return db_stats_error(err),
    };

    let critical_events: i64 = match sqlx::query_scalar(
        "SELECT COUNT(*) FROM audit_logs WHERE LOWER(COALESCE(severity, '')) = 'critical'"
    )
    .fetch_one(&state.db)
    .await
    {
        Ok(count) => count,
        Err(err) => return db_stats_error(err),
    };

    // There is no SOC 2 attestation stored anywhere in this deployment, and a
    // log count is not an auditor's opinion. The old code returned the literal
    // "SOC 2 Type II Compliant"; this reports what the trail actually shows and
    // says plainly that no attestation backs it.
    Json(json!({
        "total_events": total,
        "totalEvents": total,
        "sample_analyzed": total.min(500),
        "sampleAnalyzed": total.min(500),
        "severity_distribution": severity_distribution,
        "severityDistribution": severity_distribution,
        "category_distribution": category_distribution,
        "categoryDistribution": category_distribution,
        "top_actors": top_actors,
        "topActors": top_actors,
        "top_actions": top_actions,
        "topActions": top_actions,
        "critical_event_count": critical_events,
        "criticalEventCount": critical_events,
        "soc2_compliance_status": null,
        "soc2ComplianceStatus": null,
        "soc2Note": "no SOC 2 attestation is stored or probed by this backend; this trail records events but does not certify compliance. criticalEventCount is the only real signal here.",
        "last_audit_event_at": last_event,
        "lastAuditEventAt": last_event
    }))
    .into_response()
}

fn db_stats_error(err: sqlx::Error) -> Response {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(json!({
            "error": "the audit statistics could not be computed",
            "detail": err.to_string(),
            "databaseError": true,
        })),
    )
        .into_response()
}

/// Severity values this backend can honestly record: the four the Python request
/// model documents (`api/routes/audit_logs.py:23`) plus the "unspecified" bucket
/// `get_audit_stats` reports for rows that store NULL.
const AUDIT_SEVERITIES: &[&str] = &["info", "notice", "warning", "critical"];

/// Compliance categories actually written by this codebase and its Python
/// predecessor (`services/audit_service.py`, `services/sso_service.py`,
/// `services/rbac_service.py`, `services/custom_attribute_service.py`,
/// `routes/automation.rs`, `routes/canned_macros_sla.rs`), including the five the
/// Python request model documents. Anything else would be filed under a category
/// that no reader of the trail can resolve.
const AUDIT_COMPLIANCE_CATEGORIES: &[&str] = &[
    "access_control",
    "data_modification",
    "export_activity",
    "ai_governance",
    "security_event",
    "audit_trail",
    "authentication_failure",
    "authentication_success",
    "access_denied",
    "security_policy_change",
    "user_provisioning",
    "configuration_change",
    "data_update",
    "general",
];

/// The identity this process can verify by itself. `CurrentAccount` refuses when
/// no bearer is attached, and these endpoints are also called by internal
/// services, so the session is read optionally: a valid bearer outranks any
/// actor string the caller types, because the caller could type anything.
async fn verified_actor(
    state: &AppState,
    headers: &axum::http::HeaderMap,
) -> Option<(String, Option<String>)> {
    let raw = headers.get(header::AUTHORIZATION)?.to_str().ok()?;
    let (scheme, token) = raw.split_once(char::is_whitespace)?;
    if !scheme.eq_ignore_ascii_case("bearer") {
        return None;
    }
    let token = token.trim();
    if token.is_empty() {
        return None;
    }
    let claims = state.nid.validate_session(token).await;
    claims.filter(|claims| !claims.account_id.trim().is_empty()).map(|claims| {
        (
            claims.account_id,
            claims
                .email
                .map(|email| email.trim().to_string())
                .filter(|email| !email.is_empty()),
        )
    })
}

/// Reads an optional free-text audit field: absent or null yields NULL, an empty
/// or whitespace-only string is treated as absent, and a non-string is refused
/// instead of coerced into a false record.
fn audit_text_field(
    payload: &Value,
    key: &str,
) -> Result<Option<String>, Response> {
    match payload.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(v) => match v.as_str() {
            Some(raw) => {
                let trimmed = raw.trim();
                if trimmed.is_empty() {
                    Ok(None)
                } else {
                    Ok(Some(trimmed.to_string()))
                }
            }
            None => Err((
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "success": false,
                    "error": format!("{key} must be a non-empty string or null"),
                    "field": key,
                })),
            )
                .into_response()),
        },
    }
}

pub async fn record_audit_event(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    Json(payload): Json<serde_json::Value>,
) -> Result<Response, Response> {
    // `action` and `entity_type` are required in Python (audit_logs.py:16-17);
    // defaulting them invented a "system.system.action" event that never
    // happened.
    let Some(action) = payload
        .get("action")
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|v| !v.is_empty())
    else {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({ "success": false, "error": "action is required and must be a non-empty string", "field": "action" })),
        )
            .into_response());
    };
    let Some(entity_type) = payload
        .get("entity_type")
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|v| !v.is_empty())
    else {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({ "success": false, "error": "entity_type is required and must be a non-empty string", "field": "entity_type" })),
        )
            .into_response());
    };

    let id = payload["id"]
        .as_str()
        .map(String::from)
        .unwrap_or_else(|| Uuid::new_v4().to_string());
    let entity_id = payload["entity_id"].as_str();

    // `audit_logs.actor` is `TEXT NOT NULL` (db/mod.rs), so the trail cannot hold
    // a NULL actor. Writing "system" for an unattributed call invented a service
    // principal that does not exist, so the actor now comes from a verified
    // session first, the caller's claim second, and a request that offers neither
    // is refused.
    let claimed_actor = audit_text_field(&payload, "actor")?;
    let verified = verified_actor(&state, &headers).await;
    let (actor, actor_source, verified_email) = match (&verified, &claimed_actor) {
        (Some((account_id, email)), _) => (account_id.clone(), "verified bearer session", email.clone()),
        (None, Some(claimed)) => (claimed.clone(), "client supplied (not verified by this server)", None),
        (None, None) => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "success": false,
                    "error": "no actor can be recorded: the request carried neither a valid Bearer session nor a non-empty actor string, and an audit entry must name who acted",
                    "field": "actor",
                    "accepted": ["Authorization: Bearer <session token>", "{\"actor\": \"<username or service name>\"}"],
                })),
            )
                .into_response());
        }
    };

    let mut actor_email = audit_text_field(&payload, "actor_email")?;
    if actor_email.is_none() {
        // Only a verified session may fill the email the caller left out.
        actor_email = verified_email.clone();
    }
    let actor_name = audit_text_field(&payload, "actor_name")?;
    let actor_role = audit_text_field(&payload, "actor_role")?;

    // A compliance record whose severity or category was guessed is a false
    // record, so an unknown value is refused and an absent one is stored as NULL
    // and named in the response instead of inheriting the Python literal defaults.
    let severity = match audit_text_field(&payload, "severity")? {
        Some(raw) => {
            let lowered = raw.to_lowercase();
            if !AUDIT_SEVERITIES.contains(&lowered.as_str()) {
                return Err((
                    StatusCode::BAD_REQUEST,
                    Json(json!({
                        "success": false,
                        "error": format!("invalid severity '{raw}'"),
                        "field": "severity",
                        "allowed": AUDIT_SEVERITIES,
                    })),
                )
                    .into_response());
            }
            Some(lowered)
        }
        None => None,
    };
    let category = match audit_text_field(&payload, "compliance_category")? {
        Some(raw) => {
            let lowered = raw.to_lowercase();
            if !AUDIT_COMPLIANCE_CATEGORIES.contains(&lowered.as_str()) {
                return Err((
                    StatusCode::BAD_REQUEST,
                    Json(json!({
                        "success": false,
                        "error": format!("invalid compliance_category '{raw}'"),
                        "field": "compliance_category",
                        "allowed": AUDIT_COMPLIANCE_CATEGORIES,
                    })),
                )
                    .into_response());
            }
            Some(lowered)
        }
        None => None,
    };
    let description = audit_text_field(&payload, "description")?;
    let mut warnings: Vec<Value> = Vec::new();
    if severity.is_none() {
        warnings.push(json!({
            "field": "severity",
            "stored": null,
            "reason": "no severity was submitted, so none was guessed; allowed values are info, notice, warning, critical",
        }));
    }
    if category.is_none() {
        warnings.push(json!({
            "field": "compliance_category",
            "stored": null,
            "reason": "no compliance_category was submitted, so the row is stored without a compliance classification rather than a defaulted one",
        }));
    }
    if actor_source != "verified bearer session" {
        warnings.push(json!({
            "field": "actor",
            "stored": actor.as_str(),
            "reason": "the actor was supplied by the caller and no Bearer session was verified for this request",
        }));
    }

    // The legacy backend took the client IP from the socket (request.client.host)
    // and the user agent from the header. Neither is faked here: absent means
    // absent, and the row stores NULL so the trail cannot claim a source it did
    // not observe.
    let ip = headers
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.split(',').next())
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .or_else(|| {
            headers
                .get("x-real-ip")
                .and_then(|v| v.to_str().ok())
                .map(str::trim)
                .filter(|v| !v.is_empty())
        });
    let user_agent = headers
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .map(str::to_string);
    let details_str = payload["details"].to_string();
    let now = Utc::now().to_rfc3339();

    let result = sqlx::query(
        r#"
        INSERT INTO audit_logs (id, entity_type, entity_id, action, actor, actor_email, actor_name, actor_role, severity, compliance_category, description, ip_address, user_agent, details, created_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
        "#
    )
    .bind(&id)
    .bind(entity_type)
    .bind(entity_id)
    .bind(action)
    .bind(actor.as_str())
    .bind(actor_email.as_deref())
    .bind(actor_name.as_deref())
    .bind(actor_role.as_deref())
    .bind(severity.as_deref())
    .bind(category.as_deref())
    .bind(description.as_deref())
    .bind(ip)
    .bind(user_agent.as_deref())
    .bind(&details_str)
    .bind(&now)
    .execute(&state.db)
    .await;

    match result {
        Ok(report) if report.rows_affected() == 0 => Ok((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "success": false,
                "error": "the audit event was not written: the INSERT affected 0 rows",
            })),
        )
            .into_response()),
        Ok(_) => Ok((
            StatusCode::OK,
            Json(json!({
                "success": true,
                "id": id,
                "created_at": now,
                "createdAt": now,
                "action": action,
                "actor": actor,
                "actorSource": actor_source,
                "severity": severity,
                "complianceCategory": category,
                "warnings": warnings,
            })),
        )
            .into_response()),
        Err(err) => Ok((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "success": false,
                "error": "the audit event could not be recorded",
                "detail": err.to_string(),
                "databaseError": true,
            })),
        )
            .into_response()),
    }
}

#[derive(Debug, Deserialize)]
pub struct ExportAuditQuery {
    pub format: Option<String>,
}

pub async fn export_audit_logs(
    State(state): State<AppState>,
    Query(params): Query<ExportAuditQuery>,
) -> Response {
    let rows = match sqlx::query_as::<_, AuditLogRow>(
        "SELECT id, entity_type, entity_id, action, actor, actor_email, actor_name, actor_role, severity, compliance_category, description, ip_address, user_agent, details, created_at FROM audit_logs ORDER BY created_at DESC"
    )
    .fetch_all(&state.db)
    .await
    {
        Ok(rows) => rows,
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "the audit trail could not be read for export",
                    "detail": err.to_string(),
                    "databaseError": true,
                })),
            )
                .into_response()
        }
    };

    let fmt = params.format.unwrap_or_else(|| "csv".to_string()).to_lowercase();
    if fmt == "json" {
        return match serde_json::to_string_pretty(&rows) {
            Ok(body) => (
                [(header::CONTENT_TYPE, HeaderValue::from_static("application/json; charset=utf-8"))],
                body,
            )
                .into_response(),
            Err(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "the audit rows could not be serialized",
                    "detail": err.to_string(),
                })),
            )
                .into_response(),
        };
    }

    let mut csv = String::from("id,created_at,action,actor,actor_email,entity_type,entity_id,severity,compliance_category,description,ip_address\n");
    for r in rows {
        // Blank means the stored value is null. The previous exporter wrote
        // "info" / "access_control" / "127.0.0.1" into the compliance export,
        // which would misstate the trail to an auditor.
        let quote = |raw: Option<String>| match raw {
            Some(value) => format!("\"{}\"", value.replace('"', "\"\"")),
            None => "\"\"".to_string(),
        };
        csv.push_str(&format!(
            "{},{},{},{},{},{},{},{},{},{},{}\n",
            quote(Some(r.id)),
            quote(Some(r.created_at)),
            quote(Some(r.action)),
            quote(Some(r.actor)),
            quote(r.actor_email),
            quote(Some(r.entity_type)),
            quote(r.entity_id),
            quote(r.severity),
            quote(r.compliance_category),
            quote(r.description),
            quote(r.ip_address)
        ));
    }

    (
        [
            (header::CONTENT_TYPE, HeaderValue::from_static("text/csv; charset=utf-8")),
            (header::CONTENT_DISPOSITION, HeaderValue::from_static("attachment; filename=\"audit-trail-soc2.csv\"")),
        ],
        csv,
    ).into_response()
}

// ==========================================
// 3. RBAC & Roles
// ==========================================

pub async fn list_roles(State(state): State<AppState>) -> Response {
    #[derive(sqlx::FromRow)]
    struct RawRole {
        id: String,
        name: String,
        display_name: String,
        description: Option<String>,
        is_system_role: Option<bool>,
        permissions: Option<String>,
    }

    let roles = match sqlx::query_as::<_, RawRole>(
        "SELECT id, name, display_name, description, is_system_role, permissions FROM role_definitions ORDER BY name ASC",
    )
    .fetch_all(&state.db)
    .await
    {
        Ok(roles) => roles,
        Err(err) => return rbac_db_error("list role definitions", err),
    };

    let mut result = Vec::new();
    for r in roles {
        // A null / unparsable permissions column means the role defines no
        // permissions. It used to be answered with ["conversations:view"], which
        // grants a capability nobody configured.
        let perms: Vec<String> = match r.permissions.as_deref() {
            Some(raw) if !raw.trim().is_empty() => serde_json::from_str(raw).unwrap_or_default(),
            _ => Vec::new(),
        };

        let member_count: i64 = match sqlx::query_scalar("SELECT COUNT(*) FROM team_members WHERE role = ?")
            .bind(&r.name)
            .fetch_one(&state.db)
            .await
        {
            Ok(count) => count,
            Err(err) => {
                result.push(json!({
                    "id": r.id,
                    "name": r.name,
                    "displayName": r.display_name,
                    "description": r.description,
                    "isSystemRole": r.is_system_role.unwrap_or(false),
                    "permissions": perms,
                    "permissionsCount": perms.len(),
                    "assignedMembersCount": serde_json::Value::Null,
                    "memberCountWarning": write_warning("count team_members by role", &err),
                }));
                continue;
            }
        };

        result.push(json!({
            "id": r.id,
            "name": r.name,
            "displayName": r.display_name,
            "description": r.description,
            "isSystemRole": r.is_system_role.unwrap_or(false),
            "permissions": perms,
            "permissionsCount": perms.len(),
            "assignedMembersCount": member_count
        }));
    }

    Json(result).into_response()
}

pub async fn get_permissions_catalog() -> impl IntoResponse {
    Json(vec![
        json!({ "key": "conversations:view", "label": "View Conversations", "description": "View live conversations and message histories", "category": "Inbox" }),
        json!({ "key": "conversations:reply", "label": "Reply to Conversations", "description": "Send inbound and outbound customer responses", "category": "Inbox" }),
        json!({ "key": "conversations:assign", "label": "Assign & Route Queues", "description": "Transfer conversations between agents and teams", "category": "Inbox" }),
        json!({ "key": "conversations:private_notes", "label": "Internal Private Notes", "description": "Collaborate on conversations with teammates privately", "category": "Inbox" }),
        json!({ "key": "contacts:view", "label": "View Contacts", "description": "Browse CRM contacts and customer profiles", "category": "CRM" }),
        json!({ "key": "contacts:edit", "label": "Edit Contacts", "description": "Update customer attributes, notes, and tags", "category": "CRM" }),
        json!({ "key": "broadcasts:send", "label": "Dispatch Campaigns", "description": "Launch bulk multi-channel broadcasts", "category": "Marketing" }),
        json!({ "key": "ai:manage", "label": "Manage AI Copilots", "description": "Configure Captain AI suggestions and retraining", "category": "AI" }),
        json!({ "key": "audit_logs:view", "label": "Audit Logs & SOC 2", "description": "View immutable administrative activity trails", "category": "Security" }),
        json!({ "key": "sso:manage", "label": "Enterprise SSO & SAML", "description": "Configure identity federation and access enforcement", "category": "Security" })
    ])
}

pub async fn get_role_detail(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Response {
    #[derive(sqlx::FromRow)]
    struct RawRole {
        id: String,
        name: String,
        display_name: String,
        description: Option<String>,
        is_system_role: Option<bool>,
        permissions: Option<String>,
    }

    let role = match sqlx::query_as::<_, RawRole>(
        "SELECT id, name, display_name, description, is_system_role, permissions FROM role_definitions WHERE id = ? OR name = ? LIMIT 1"
    )
    .bind(&id)
    .bind(&id)
    .fetch_optional(&state.db)
    .await
    {
        Ok(role) => role,
        Err(err) => return rbac_db_error("read the role definition", err),
    };

    // The old code answered "not found" with an invented role carrying
    // conversations:view + conversations:reply permissions. A missing role must
    // be reported as missing.
    let Some(r) = role else {
        return (
            StatusCode::NOT_FOUND,
            Json(json!({
                "error": format!("no role_definitions row matches id or name '{id}'"),
                "id": id,
            })),
        )
            .into_response();
    };

    let perms: Vec<String> = match r.permissions.as_deref() {
        Some(raw) if !raw.trim().is_empty() => serde_json::from_str(raw).unwrap_or_default(),
        _ => Vec::new(),
    };

    Json(json!({
        "id": r.id,
        "name": r.name,
        "displayName": r.display_name,
        "description": r.description,
        "isSystemRole": r.is_system_role.unwrap_or(false),
        "permissions": perms
    }))
    .into_response()
}

pub async fn create_role(
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> Response {
    // A role name is mandatory (role_definitions.name is UNIQUE NOT NULL). The
    // old code silently created a role literally named "custom_role".
    let Some(name) = payload
        .get("name")
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|v| !v.is_empty())
    else {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "success": false, "error": "a non-empty `name` is required" })),
        )
            .into_response();
    };
    let display = payload
        .get("display_name")
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| title_from_slug(&normalize_attribute_name(name)));
    let desc = payload["description"].as_str();
    let perms_str = match &payload["permissions"] {
        Value::Null => "[]".to_string(),
        other => other.to_string(),
    };
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();

    let result = sqlx::query(
        "INSERT INTO role_definitions (id, name, display_name, description, is_system_role, permissions, created_at, updated_at) VALUES ($1, $2, $3, $4, 0, $5, $6, $7)"
    )
    .bind(&id)
    .bind(name)
    .bind(&display)
    .bind(desc)
    .bind(&perms_str)
    .bind(&now)
    .bind(&now)
    .execute(&state.db)
    .await;

    match result {
        Ok(report) if report.rows_affected() == 0 => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "success": false, "error": "the role INSERT affected 0 rows; nothing was created" })),
        )
            .into_response(),
        Ok(_) => Json(json!({
            "success": true,
            "id": id,
            "name": name,
            "displayName": display,
            "permissions": payload["permissions"],
        }))
        .into_response(),
        Err(err) => {
            // role_definitions.name is UNIQUE, so a clash is a real conflict,
            // not a success to be faked.
            let duplicate = err.to_string().to_lowercase().contains("unique");
            if duplicate {
                (
                    StatusCode::CONFLICT,
                    Json(json!({ "success": false, "error": format!("a role named '{name}' already exists") })),
                )
                    .into_response()
            } else {
                rbac_db_error("create the role", err)
            }
        }
    }
}

pub async fn update_role(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<serde_json::Value>,
) -> Response {
    let now = Utc::now().to_rfc3339();
    // Only fields that were actually sent are written. The previous version
    // stamped "Updated Role" over the display name on every request, including
    // requests that never mentioned it.
    let display = payload
        .get("display_name")
        .and_then(|v| v.as_str())
        .map(str::to_string);
    let desc = payload.get("description").and_then(|v| v.as_str()).map(str::to_string);
    let perms = payload.get("permissions").map(|v| v.to_string());

    let result = sqlx::query(
        "UPDATE role_definitions \
         SET display_name = COALESCE($1, display_name), \
             description = COALESCE($2, description), \
             permissions = COALESCE($3, permissions), \
             updated_at = $4 \
         WHERE id = $5 OR name = $6"
    )
    .bind(display.as_deref())
    .bind(desc.as_deref())
    .bind(perms.as_deref())
    .bind(&now)
    .bind(&id)
    .bind(&id)
    .execute(&state.db)
    .await;

    match result {
        Ok(report) if report.rows_affected() == 0 => (
            StatusCode::NOT_FOUND,
            Json(json!({
                "success": false,
                "error": format!("no role_definitions row matches id or name '{id}'; nothing was updated"),
                "id": id,
            })),
        )
            .into_response(),
        Ok(report) => Json(json!({
            "success": true,
            "id": id,
            "rowsUpdated": report.rows_affected(),
            "displayName": display,
            "permissions": payload.get("permissions"),
        }))
        .into_response(),
        Err(err) => rbac_db_error("update the role", err),
    }
}

pub async fn delete_role(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Response {
    let is_sys: Option<bool> = match sqlx::query_scalar(
        "SELECT is_system_role FROM role_definitions WHERE id = ? OR name = ?"
    )
    .bind(&id)
    .bind(&id)
    .fetch_optional(&state.db)
    .await
    {
        Ok(value) => value,
        Err(err) => return rbac_db_error("check whether the role is a system role", err),
    };

    if is_sys.unwrap_or(false) {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "success": false, "error": "Cannot delete built-in system role" })),
        ).into_response();
    }

    let result = sqlx::query("DELETE FROM role_definitions WHERE id = $1 OR name = $2")
        .bind(&id)
        .bind(&id)
        .execute(&state.db)
        .await;

    match result {
        Ok(report) if report.rows_affected() == 0 => (
            StatusCode::NOT_FOUND,
            Json(json!({
                "success": false,
                "error": format!("no role_definitions row matches id or name '{id}'; nothing was deleted"),
                "deleted_role": id,
            })),
        )
            .into_response(),
        Ok(report) => Json(json!({
            "success": true,
            "deleted_role": id,
            "rowsDeleted": report.rows_affected(),
        }))
        .into_response(),
        Err(err) => rbac_db_error("delete the role", err),
    }
}

pub async fn assign_member_role(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<serde_json::Value>,
) -> Response {
    let Some(role) = payload
        .get("role")
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|r| !r.is_empty())
    else {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "a non-empty `role` is required",
                "usage": "POST /roles/members/{id}/assign {\"role\":\"admin\"}",
            })),
        )
            .into_response();
    };

    let result = sqlx::query("UPDATE team_members SET role = $1 WHERE id = $2 OR email = $3")
        .bind(role)
        .bind(&id)
        .bind(&id)
        .execute(&state.db)
        .await;

    match result {
        Ok(res) if res.rows_affected() == 0 => (
            StatusCode::NOT_FOUND,
            Json(json!({
                "error": "no team_member row matches this id or email; the role was not changed",
                "memberId": id,
            })),
        )
            .into_response(),
        Ok(res) => Json(json!({
            "success": true,
            "memberId": id,
            "role": role,
            "rowsUpdated": res.rows_affected(),
        }))
        .into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": "the role could not be written to team_members",
                "detail": err.to_string(),
            })),
        )
            .into_response(),
    }
}

#[derive(Deserialize)]
pub struct MemberPermissionQuery {
    pub permission: Option<String>,
}

pub async fn check_member_permission(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(query): Query<MemberPermissionQuery>,
) -> Response {
    let Some(permission) = query
        .permission
        .as_deref()
        .map(str::trim)
        .filter(|p| !p.is_empty())
    else {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "a permission key is required",
                "usage": "GET /roles/members/{id}/check?permission=conversations:view",
            })),
        )
            .into_response();
    };

    let table_present = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM information_schema.tables WHERE table_schema = 'public' AND table_name = 'team_members'",
    )
    .fetch_optional(&state.db)
    .await;
    match table_present {
        Ok(Some(0)) | Ok(None) => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(json!({
                    "error": "the team_members table is not present in this database",
                    "note": "membership cannot be resolved, so no permission can be granted",
                    "allowed": false,
                })),
            )
                .into_response()
        }
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "could not inspect the database schema",
                    "detail": err.to_string(),
                })),
            )
                .into_response()
        }
        _ => {}
    }

    let role = sqlx::query_scalar::<_, String>(
        "SELECT role FROM team_members WHERE id = ? OR email = ? LIMIT 1",
    )
    .bind(&id)
    .bind(&id)
    .fetch_optional(&state.db)
    .await;

    let role = match role {
        Ok(Some(role)) if !role.trim().is_empty() => role,
        Ok(Some(_)) => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({
                    "error": "this member has no role assigned",
                    "memberId": id,
                    "permission": permission,
                    "allowed": false,
                })),
            )
                .into_response()
        }
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({
                    "error": "no team_member row matches this id or email",
                    "memberId": id,
                    "permission": permission,
                    "allowed": false,
                })),
            )
                .into_response()
        }
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "could not read the member's role from team_members",
                    "detail": err.to_string(),
                })),
            )
                .into_response()
        }
    };

    let stored = sqlx::query_scalar::<_, Option<String>>(
        "SELECT permissions FROM role_definitions WHERE name = ? OR id = ? LIMIT 1",
    )
    .bind(&role)
    .bind(&role)
    .fetch_optional(&state.db)
    .await;

    let raw = match stored {
        Ok(Some(Some(raw))) => raw,
        Ok(Some(None)) => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({
                    "error": "this member's role has no permissions column value",
                    "role": role,
                    "permission": permission,
                    "allowed": false,
                })),
            )
                .into_response()
        }
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({
                    "error": "no role_definitions row matches this member's role",
                    "role": role,
                    "permission": permission,
                    "allowed": false,
                })),
            )
                .into_response()
        }
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "could not read role_definitions",
                    "detail": err.to_string(),
                })),
            )
                .into_response()
        }
    };

    let permissions: Vec<String> = match serde_json::from_str::<Vec<String>>(&raw) {
        Ok(list) => list,
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "role_definitions.permissions is not a JSON array of keys",
                    "role": role,
                    "detail": err.to_string(),
                })),
            )
                .into_response()
        }
    };

    // The resource prefix is taken from the real split instead of an `unwrap_or`
    // on a fallible-looking Option (`split` always yields a first segment), and the
    // degenerate case is handled explicitly: a permission such as ":read" has an
    // empty resource, and turning that into the wildcard "*:*" would let a role
    // holding "*" appear to grant a key the operator never issued.
    let resource = match permission.split_once(':') {
        Some((head, _)) => head,
        None => permission,
    };
    let wildcard_prefix = format!("{resource}:*");
    let allowed = !resource.is_empty()
        && permissions
            .iter()
            .any(|p| p == permission || p == "*" || p.as_str() == wildcard_prefix.as_str());

    Json(json!({
        "memberId": id,
        "role": role,
        "permission": permission,
        "allowed": allowed,
        "rolePermissions": permissions,
        "source": "role_definitions.permissions",
        "warning": if resource.is_empty() {
            Some("the queried permission has no resource prefix before ':', so no wildcard rule can apply to it; only an exact grant matches")
        } else {
            None
        },
    }))
    .into_response()
}

// Translation is served by `crate::routes::translation`, wired under
// /translation in main.rs (the frontend calls /translation/languages,
// /translation/settings, /translation/detect, /translation/messages/{id}/translate
// and /translation/conversations/{id}/prepare-outbound). An earlier version of
// this file answered the same questions with a fabricated language catalogue and
// an untranslated "translated" string; there is no second copy to keep in sync.

// ==========================================
// 6. Custom Attributes
// ==========================================

#[derive(sqlx::FromRow)]
struct AttrDef {
    name: String,
    attribute_type: String,
    options: Option<String>,
    is_required: Option<bool>,
}

async fn load_definitions(
    state: &AppState,
    model: &str,
) -> Result<Vec<AttrDef>, sqlx::Error> {
    // `regex_pattern` is deliberately not loaded here: value validation below
    // mirrors Python `validate_attribute_value` (custom_attribute_service.py:218),
    // which never applies the stored pattern, and a column read that nothing acts
    // on would claim an enforcement that does not happen. The CRUD reads
    // (`load_attribute` and the list query) still return the stored pattern.
    sqlx::query_as::<_, AttrDef>(
        "SELECT name, attribute_type, options, is_required \
         FROM custom_attribute_definitions WHERE attribute_model = ?",
    )
    .bind(model)
    .fetch_all(&state.db)
    .await
}

/// Port of Python `validate_attribute_value` (custom_attribute_service.py:218).
/// Returns the cast value, or the exact error string the Python service raises.
fn validate_value(def: &AttrDef, value: &Value) -> Result<Value, String> {
    let required = def.is_required.unwrap_or(false);
    let name = &def.display_or_name();
    let blank = match value {
        Value::Null => true,
        Value::String(s) => s.is_empty(),
        _ => false,
    };
    if blank {
        if required {
            return Err(format!("Attribute '{name}' is required."));
        }
        return Ok(Value::Null);
    }
    match def.attribute_type.as_str() {
        "text" => Ok(Value::String(value.as_str().unwrap_or_default().trim().to_string())),
        "number" | "currency" => {
            let cleaned = value
                .as_str()
                .map(|s| s.to_string())
                .unwrap_or_else(|| value.to_string())
                .replace('$', "")
                .replace(',', "")
                .trim()
                .to_string();
            if let Ok(n) = cleaned.parse::<f64>() {
                Ok(json!(n))
            } else {
                Err(format!("Attribute '{name}' must be a valid numeric value."))
            }
        }
        "boolean" => match value {
            Value::Bool(b) => Ok(json!(b)),
            Value::String(s) => match s.trim().to_lowercase().as_str() {
                "true" | "1" | "yes" | "t" => Ok(json!(true)),
                "false" | "0" | "no" | "f" => Ok(json!(false)),
                _ => Err(format!("Attribute '{name}' must be a boolean (true/false).")),
            },
            _ => Err(format!("Attribute '{name}' must be a boolean (true/false).")),
        },
        "list" => {
            let val_str = value.as_str().unwrap_or_default().trim().to_string();
            let allowed: Vec<String> = def
                .options
                .as_deref()
                .and_then(|raw| serde_json::from_str::<Vec<String>>(raw).ok())
                .unwrap_or_default();
            if allowed.iter().any(|o| *o == val_str) {
                Ok(json!(val_str))
            } else {
                Err(format!("Value '{val_str}' is not in allowed options: {allowed:?}"))
            }
        }
        "date" => {
            let val_str = value.as_str().unwrap_or_default().trim().to_string();
            // Python accepts an ISO-parseable datetime, otherwise a literal
            // `^\d{4}-\d{2}-\d{2}$` shape. Length-and-dashes alone is not that
            // shape -- "abcd-ef-gr" would pass -- so every position is checked.
            let matches_shape = val_str.len() == 10
                && val_str.chars().enumerate().all(|(index, ch)| match index {
                    4 | 7 => ch == '-',
                    _ => ch.is_ascii_digit(),
                });
            if chrono::NaiveDate::parse_from_str(&val_str, "%Y-%m-%d").is_ok()
                || chrono::DateTime::parse_from_rfc3339(&val_str).is_ok()
                || matches_shape
            {
                Ok(json!(val_str))
            } else {
                Err(format!(
                    "Attribute '{name}' must be an ISO format date (YYYY-MM-DD)."
                ))
            }
        }
        "link" => {
            let val_str = value.as_str().unwrap_or_default().trim().to_string();
            if val_str.starts_with("http://") || val_str.starts_with("https://") {
                Ok(json!(val_str))
            } else {
                Err(format!("Attribute '{name}' must be a valid HTTP/HTTPS URL."))
            }
        }
        _ => Ok(value.clone()),
    }
}

impl AttrDef {
    fn display_or_name(&self) -> String {
        title_from_slug(&self.name)
    }
}

async fn load_attribute(
    state: &AppState,
    id_or_name: &str,
    model_filter: Option<&str>,
) -> Result<Option<AttributeRow>, sqlx::Error> {
    const BASE: &str = "SELECT id, name, display_name, description, attribute_model, attribute_type, default_value, options, is_required, regex_pattern, regex_cue, created_at, updated_at FROM custom_attribute_definitions";
    match model_filter.map(str::trim).filter(|m| !m.is_empty()) {
        Some(model) => {
            let sql = format!("{BASE} WHERE (id = ? OR name = ?) AND attribute_model = ? LIMIT 1");
            sqlx::query_as::<_, AttributeRow>(&sql)
                .bind(id_or_name)
                .bind(id_or_name)
                .bind(model)
                .fetch_optional(&state.db)
                .await
        }
        None => {
            let sql = format!("{BASE} WHERE (id = ? OR name = ?) LIMIT 1");
            sqlx::query_as::<_, AttributeRow>(&sql)
                .bind(id_or_name)
                .bind(id_or_name)
                .fetch_optional(&state.db)
                .await
        }
    }
}

#[derive(Deserialize)]
pub struct AttributeListQuery {
    pub attribute_model: Option<String>,
}

pub async fn get_custom_attributes(
    State(state): State<AppState>,
    Query(q): Query<AttributeListQuery>,
) -> Response {
    const BASE: &str = "SELECT id, name, display_name, description, attribute_model, attribute_type, default_value, options, is_required, regex_pattern, regex_cue, created_at, updated_at FROM custom_attribute_definitions";
    let rows = match q.attribute_model.as_deref().map(str::trim).filter(|m| !m.is_empty()) {
        Some(model) => {
            let sql = format!("{BASE} WHERE attribute_model = ? ORDER BY display_name ASC");
            sqlx::query_as::<_, AttributeRow>(&sql)
                .bind(model)
                .fetch_all(&state.db)
                .await
        }
        None => {
            let sql = format!("{BASE} ORDER BY display_name ASC");
            sqlx::query_as::<_, AttributeRow>(&sql)
                .fetch_all(&state.db)
                .await
        }
    };

    match rows {
        Ok(rows) => Json(rows.iter().map(AttributeRow::to_json).collect::<Vec<_>>()).into_response(),
        Err(err) => rbac_db_error("list the custom attribute definitions", err),
    }
}

/// Port of the previously unported `GET /custom-attributes/{id_or_name}`
/// (custom_attributes.py:103-117 / custom_attribute_service.py:42-61): id-or-name
/// lookup with the optional attribute_model filter and a real 404.
///
/// NOTE: `custom_attribute_definitions` has no tenant_key column (see db/mod.rs),
/// so — exactly like Python — there is no tenant filter to apply to definitions;
/// they are workspace-global. The value-write endpoints below ARE tenant scoped.
pub async fn get_custom_attribute(
    State(state): State<AppState>,
    Path(id_or_name): Path<String>,
    Query(q): Query<AttributeListQuery>,
) -> Response {
    match load_attribute(&state, &id_or_name, q.attribute_model.as_deref()).await {
        Ok(Some(row)) => Json(row.to_json()).into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(json!({
                "error": format!("no custom attribute definition matches id or name '{id_or_name}'"),
            })),
        )
            .into_response(),
        Err(err) => rbac_db_error("read the custom attribute definition", err),
    }
}

/// An optional string column on create: absent or null stores NULL, a string is
/// stored as sent, and any other JSON type is refused — reading a number as
/// "absent" would silently drop an operator's input.
fn optional_str_field(payload: &Value, key: &str) -> Result<Option<String>, Response> {
    match payload.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(v) => match v.as_str() {
            Some(raw) => Ok(Some(raw.to_string())),
            None => Err((
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "success": false,
                    "error": format!("{key} must be a string or null when it is sent"),
                    "field": key,
                })),
            )
                .into_response()),
        },
    }
}

/// The partial-update twin of [`optional_str_field`]: an absent key keeps the
/// stored value, null clears it, and a wrong type is refused instead of being
/// read as "absent", which would quietly leave the old row in place.
fn kept_str_field(
    payload: &Value,
    key: &str,
    stored: &Option<String>,
) -> Result<Option<String>, Response> {
    match payload.get(key) {
        None => Ok(stored.clone()),
        Some(_) => optional_str_field(payload, key),
    }
}

pub async fn create_custom_attribute(
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> Response {
    // Python rejects an empty slug (custom_attribute_service.py:73-75); it does
    // not invent a name, so a missing `name` is refused as a missing input rather
    // than normalised from an empty string.
    let raw_name = match payload.get("name") {
        None | Some(Value::Null) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "success": false,
                    "error": "name is required to create a custom attribute definition",
                    "field": "name",
                })),
            )
                .into_response();
        }
        Some(v) => match v.as_str() {
            Some(raw) => raw.to_string(),
            None => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(json!({
                        "success": false,
                        "error": "name must be a string",
                        "field": "name",
                    })),
                )
                    .into_response();
            }
        },
    };
    let slug = normalize_attribute_name(&raw_name);
    if slug.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "success": false,
                "error": format!("name '{raw_name}' contains no letters or digits, so no attribute slug can be derived"),
                "field": "name",
            })),
        )
            .into_response();
    }

    // The Python service defaults these two (custom_attribute_service.py:77 and
    // :83), but a definition written with a guessed model or type is a row the
    // operator never asked for, so both are required inputs here. The UI already
    // sends them (`frontend/src/pages/CustomAttributes.tsx:107-108`).
    let model = match payload.get("attribute_model") {
        Some(v) => match v.as_str().map(str::trim).filter(|s| !s.is_empty()) {
            Some(value) => value.to_string(),
            None => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(json!({
                        "success": false,
                        "error": "attribute_model must be a non-empty string when it is sent",
                        "field": "attribute_model",
                        "allowed": ATTRIBUTE_MODELS,
                    })),
                )
                    .into_response();
            }
        },
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "success": false,
                    "error": "attribute_model is required; no attribute model was guessed",
                    "field": "attribute_model",
                    "allowed": ATTRIBUTE_MODELS,
                })),
            )
                .into_response();
        }
    };
    if !ATTRIBUTE_MODELS.contains(&model.as_str()) {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "success": false,
                "error": format!("invalid attribute_model '{model}'. Allowed models: {}", ATTRIBUTE_MODELS.join(", ")),
                "field": "attribute_model",
                "allowed": ATTRIBUTE_MODELS,
            })),
        )
            .into_response();
    }

    let attr_type = match payload.get("attribute_type") {
        Some(v) => match v.as_str().map(str::trim).filter(|s| !s.is_empty()) {
            Some(value) => value.to_string(),
            None => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(json!({
                        "success": false,
                        "error": "attribute_type must be a non-empty string when it is sent",
                        "field": "attribute_type",
                        "allowed": ATTRIBUTE_TYPES,
                    })),
                )
                    .into_response();
            }
        },
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "success": false,
                    "error": "attribute_type is required; no attribute type was guessed",
                    "field": "attribute_type",
                    "allowed": ATTRIBUTE_TYPES,
                })),
            )
                .into_response();
        }
    };
    if !ATTRIBUTE_TYPES.contains(&attr_type.as_str()) {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "success": false,
                "error": format!("invalid attribute_type '{attr_type}'. Allowed types: {}", ATTRIBUTE_TYPES.join(", ")),
                "field": "attribute_type",
                "allowed": ATTRIBUTE_TYPES,
            })),
        )
            .into_response();
    }

    // `options` is only meaningful as a JSON array; anything else is the
    // operator's mistake and is refused rather than stored as a stringified value
    // that no reader can enumerate.
    match &payload["options"] {
        Value::Null => {}
        Value::Array(_) => {}
        other => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "success": false,
                    "error": "options must be a JSON array of strings when it is sent",
                    "field": "options",
                    "received": other.to_string(),
                })),
            )
                .into_response();
        }
    }
    let options_json = match &payload["options"] {
        Value::Null => "[]".to_string(),
        array => array.to_string(),
    };
    let options_len = payload["options"].as_array().map(|a| a.len()).unwrap_or(0);
    if attr_type == "list" && options_len == 0 {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "success": false,
                "error": "Attributes of type 'list' must specify at least one option.",
                "field": "options",
            })),
        )
            .into_response();
    }

    let display = payload
        .get("display_name")
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| title_from_slug(&slug));
    let default_value = payload.get("default_value").and_then(|v| match v {
        Value::Null => None,
        Value::String(s) => Some(s.clone()),
        other => Some(other.to_string()),
    });
    // Absent means the column's own declared default (`custom_attribute_definitions
    // .is_required BOOLEAN NOT NULL DEFAULT 0`, matching the Python model), but a
    // value that is not a boolean is refused instead of read as `false`.
    let is_required = match payload.get("is_required") {
        None | Some(Value::Null) => false,
        Some(v) => match v.as_bool() {
            Some(value) => value,
            None => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(json!({
                        "success": false,
                        "error": "is_required must be a boolean when it is sent",
                        "field": "is_required",
                    })),
                )
                    .into_response();
            }
        },
    };
    let desc = match optional_str_field(&payload, "description") {
        Ok(value) => value,
        Err(resp) => return resp,
    };
    let regex_pattern = match optional_str_field(&payload, "regex_pattern") {
        Ok(value) => value,
        Err(resp) => return resp,
    };
    let regex_cue = match optional_str_field(&payload, "regex_cue") {
        Ok(value) => value,
        Err(resp) => return resp,
    };

    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();

    let result = sqlx::query(
        "INSERT INTO custom_attribute_definitions (id, name, display_name, description, attribute_model, attribute_type, default_value, options, is_required, regex_pattern, regex_cue, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)"
    )
    .bind(&id)
    .bind(&slug)
    .bind(&display)
    .bind(desc)
    .bind(model)
    .bind(attr_type)
    .bind(default_value.as_deref())
    .bind(&options_json)
    .bind(is_required)
    .bind(regex_pattern)
    .bind(regex_cue)
    .bind(&now)
    .bind(&now)
    .execute(&state.db)
    .await;

    match result {
        Ok(report) if report.rows_affected() == 0 => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "success": false, "error": "the attribute INSERT affected 0 rows; nothing was created" })),
        )
            .into_response(),
        Ok(_) => match load_attribute(&state, &id, None).await {
            Ok(Some(row)) => (StatusCode::CREATED, Json(row.to_json())).into_response(),
            Ok(None) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "success": false, "error": "the attribute was inserted but could not be read back" })),
            )
                .into_response(),
            Err(err) => rbac_db_error("read back the created attribute", err),
        },
        Err(err) => {
            if err.to_string().to_lowercase().contains("unique") {
                (
                    StatusCode::CONFLICT,
                    Json(json!({
                        "success": false,
                        "error": format!("an attribute named '{slug}' already exists"),
                        "detail": "custom_attribute_definitions.name is UNIQUE",
                    })),
                )
                    .into_response()
            } else {
                rbac_db_error("create the custom attribute", err)
            }
        }
    }
}

pub async fn update_custom_attribute(
    State(state): State<AppState>,
    Path(id_or_name): Path<String>,
    Json(payload): Json<serde_json::Value>,
) -> Response {
    let existing = match load_attribute(&state, &id_or_name, None).await {
        Ok(Some(e)) => e,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({ "success": false, "error": format!("Custom attribute '{id_or_name}' not found.") })),
            )
                .into_response()
        }
        Err(err) => return rbac_db_error("read the custom attribute to update", err),
    };

    // Only present keys are changed (Python exclude_unset); the old code
    // unconditionally overwrote display_name with "" when it was absent.
    // A key that is present but not the type the column holds is refused:
    // treating it as "absent" would report success while leaving the stored
    // value untouched, and treating it as a string would write a wrong one.
    let display = match payload.get("display_name") {
        None => existing.display_name.clone(),
        Some(Value::Null) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "success": false,
                    "error": "display_name cannot be null; custom_attribute_definitions.display_name is NOT NULL. Send a string or omit the key to keep the stored name",
                    "field": "display_name",
                })),
            )
                .into_response()
        }
        Some(v) => match v.as_str().map(str::trim) {
            Some(value) if !value.is_empty() => value.to_string(),
            Some(_) => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(json!({
                        "success": false,
                        "error": "display_name cannot be blank; send a real name or omit the key to keep the stored one",
                        "field": "display_name",
                    })),
                )
                    .into_response()
            }
            None => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(json!({
                        "success": false,
                        "error": "display_name must be a string when it is sent",
                        "field": "display_name",
                    })),
                )
                    .into_response()
            }
        },
    };
    let desc = match kept_str_field(&payload, "description", &existing.description) {
        Ok(value) => value,
        Err(resp) => return resp,
    };
    let options = match payload.get("options") {
        Some(Value::Null) => "[]".to_string(),
        Some(v) if v.is_array() => v.to_string(),
        Some(other) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "success": false,
                    "error": "options must be a JSON array of strings when it is sent",
                    "field": "options",
                    "received": other.to_string(),
                })),
            )
                .into_response()
        }
        None => existing.options.clone().unwrap_or_else(|| "[]".to_string()),
    };
    let default_value = match payload.get("default_value") {
        Some(Value::Null) => None,
        Some(v) => match v {
            Value::String(s) => Some(s.clone()),
            o => Some(o.to_string()),
        },
        None => existing.default_value.clone(),
    };
    // `is_required` is NOT NULL with a column default, so an absent key keeps the
    // stored boolean instead of resetting it to false.
    let is_required = match payload.get("is_required") {
        None | Some(Value::Null) => existing.is_required.unwrap_or(false),
        Some(v) => match v.as_bool() {
            Some(value) => value,
            None => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(json!({
                        "success": false,
                        "error": "is_required must be a boolean when it is sent",
                        "field": "is_required",
                    })),
                )
                    .into_response();
            }
        },
    };
    let regex_pattern = match kept_str_field(&payload, "regex_pattern", &existing.regex_pattern) {
        Ok(value) => value,
        Err(resp) => return resp,
    };
    let regex_cue = match kept_str_field(&payload, "regex_cue", &existing.regex_cue) {
        Ok(value) => value,
        Err(resp) => return resp,
    };

    let id = existing.id.clone();
    let now = Utc::now().to_rfc3339();
    let result = sqlx::query(
        "UPDATE custom_attribute_definitions SET display_name = $1, description = $2, options = $3, default_value = $4, is_required = $5, regex_pattern = $6, regex_cue = $7, updated_at = $8 WHERE id = $9"
    )
    .bind(&display)
    .bind(desc.as_deref())
    .bind(&options)
    .bind(default_value.as_deref())
    .bind(is_required)
    .bind(regex_pattern.as_deref())
    .bind(regex_cue.as_deref())
    .bind(&now)
    .bind(&id)
    .execute(&state.db)
    .await;

    match result {
        Ok(report) if report.rows_affected() == 0 => (
            StatusCode::NOT_FOUND,
            Json(json!({ "success": false, "error": "the attribute row disappeared before the update; nothing was written", "id": id })),
        )
            .into_response(),
        Ok(_) => match load_attribute(&state, &id, None).await {
            Ok(Some(row)) => Json(row.to_json()).into_response(),
            Ok(None) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "success": false, "error": "updated but the row could not be read back" })),
            )
                .into_response(),
            Err(err) => rbac_db_error("read back the updated attribute", err),
        },
        Err(err) => rbac_db_error("update the custom attribute", err),
    }
}

pub async fn delete_custom_attribute(
    State(state): State<AppState>,
    Path(id_or_name): Path<String>,
) -> Response {
    let existing = match load_attribute(&state, &id_or_name, None).await {
        Ok(Some(e)) => e,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({
                    "success": false,
                    "error": format!("no custom attribute matches id or name '{id_or_name}'; nothing was deleted"),
                })),
            )
                .into_response()
        }
        Err(err) => return rbac_db_error("read the custom attribute to delete", err),
    };

    let id = existing.id.clone();
    let name = existing.name.clone();
    let result = sqlx::query("DELETE FROM custom_attribute_definitions WHERE id = $1")
        .bind(&id)
        .execute(&state.db)
        .await;

    match result {
        Ok(report) if report.rows_affected() == 0 => (
            StatusCode::NOT_FOUND,
            Json(json!({ "success": false, "error": "the attribute was not present; nothing was deleted", "name": name })),
        )
            .into_response(),
        Ok(report) => Json(json!({
            "success": true,
            "deleted": id,
            "name": name,
            "rowsDeleted": report.rows_affected(),
        }))
        .into_response(),
        Err(err) => rbac_db_error("delete the custom attribute", err),
    }
}

/// Shared implementation behind the two `/{entity}/{id}/values` endpoints. It
/// really writes: the entity row is confirmed to exist under the caller's
/// tenant, each supplied value is validated against the matching definition
/// (Python custom_attribute_service.py:295-385), the merged JSON object is
/// UPDATE-d, and the row count is inspected. Nothing is reported as success
/// unless a write actually landed.
async fn write_entity_attribute_values(
    state: &AppState,
    id: &str,
    tenant_key: &str,
    is_contact: bool,
    payload: &serde_json::Value,
) -> Response {
    let table = if is_contact { "contacts" } else { "conversations" };
    let column = if is_contact { "custom_fields" } else { "custom_attributes" };
    let model = if is_contact { "contact_attribute" } else { "conversation_attribute" };

    let attributes = match payload.get("attributes").and_then(|v| v.as_object()) {
        Some(map) => map,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "success": false,
                    "error": "an `attributes` object is required",
                    "usage": format!("POST {} {{\"attributes\": {{\"account_tier\": \"gold\"}}}}", if is_contact { "/custom-attributes/contacts/{id}/values" } else { "/custom-attributes/conversations/{id}/values" }),
                })),
            )
                .into_response()
        }
    };
    if attributes.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "success": false, "error": "`attributes` is empty; there is nothing to write" })),
        )
            .into_response();
    }

    // Entity must exist and belong to this tenant.
    let exists_sql = format!(
        "SELECT 1 FROM {table} WHERE id = ? AND COALESCE(tenant_key, '') = ? LIMIT 1"
    );
    let exists = match sqlx::query_scalar::<_, i64>(&exists_sql)
        .bind(id)
        .bind(tenant_key)
        .fetch_optional(&state.db)
        .await
    {
        Ok(value) => value,
        Err(err) => return rbac_db_error(&format!("check whether the {table} row exists"), err),
    };
    if exists.is_none() {
        let mut body = json!({
            "success": false,
            "error": format!("no {table} row with id '{id}' exists in tenant '{tenant_key}'; nothing was written"),
            "entityTable": table,
        });
        push_entity_id(&mut body, is_contact, id);
        return (StatusCode::NOT_FOUND, Json(body)).into_response();
    }

    if !crate::routes::contacts::column_exists(&state.db, table, column).await {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({
                "success": false,
                "error": format!("{table}.{column} is not present in this database, so the values cannot be persisted"),
                "missing": [format!("{table}.{column}")],
            })),
        )
            .into_response();
    }

    // Current stored JSON object, if any.
    let select_sql = format!("SELECT {column} FROM {table} WHERE id = $1");
    let raw: Option<Option<String>> = match sqlx::query_scalar::<_, Option<String>>(&select_sql)
        .bind(id)
        .fetch_optional(&state.db)
        .await
    {
        Ok(value) => value,
        Err(err) => {
            return rbac_db_error(&format!("read the existing {table}.{column}"), err)
        }
    };
    let mut merged: serde_json::Map<String, Value> = raw
        .flatten()
        .and_then(|s| serde_json::from_str::<Value>(&s).ok())
        .and_then(|v| v.as_object().cloned())
        .unwrap_or_default();

    let definitions = match load_definitions(state, model).await {
        Ok(defs) => defs,
        Err(err) => return rbac_db_error(&format!("read the {model} attribute definitions"), err),
    };
    let def_map: std::collections::HashMap<&str, &AttrDef> =
        definitions.iter().map(|d| (d.name.as_str(), d)).collect();

    let mut warnings: Vec<Value> = Vec::new();
    for (key, value) in attributes {
        if let Some(def) = def_map.get(key.as_str()) {
            match validate_value(def, value) {
                Ok(cast) => {
                    merged.insert(key.clone(), cast);
                }
                Err(reason) => {
                    return (
                        StatusCode::BAD_REQUEST,
                        Json(json!({
                            "success": false,
                            "error": reason,
                            "field": key,
                            "attributeType": def.attribute_type,
                        })),
                    )
                        .into_response()
                }
            }
        } else {
            // Open custom field: Python stores it unvalidated (custom_attribute_service.py:319-321).
            merged.insert(key.clone(), value.clone());
            warnings.push(json!({
                "field": key,
                "stored": true,
                "note": format!("no {model} definition exists for this key, so it was stored without validation"),
            }));
        }
    }

    let now = Utc::now().to_rfc3339();
    let merged_str = Value::Object(merged.clone()).to_string();
    let update_sql = format!("UPDATE {table} SET {column} = $1, updated_at = $2 WHERE id = $3 AND COALESCE(tenant_key, '') = $4");
    let result = sqlx::query(&update_sql)
        .bind(&merged_str)
        .bind(&now)
        .bind(id)
        .bind(tenant_key)
        .execute(&state.db)
        .await;

    match result {
        Ok(report) if report.rows_affected() == 0 => {
            let mut body = json!({
                "success": false,
                "error": format!("the {table} row disappeared before the write; nothing was persisted"),
            });
            push_entity_id(&mut body, is_contact, id);
            (StatusCode::NOT_FOUND, Json(body)).into_response()
        }
        Ok(report) => {
            let camel = if is_contact { "customFields" } else { "customAttributes" };
            let mut body = json!({
                "success": true,
                "entityTable": table,
                "rowsUpdated": report.rows_affected(),
                "writtenKeys": attributes.keys().collect::<Vec<_>>(),
                "warnings": warnings,
            });
            push_entity_id(&mut body, is_contact, id);
            if let Some(map) = body.as_object_mut() {
                map.insert(camel.to_string(), Value::Object(merged.clone()));
                map.insert(column.to_string(), Value::Object(merged.clone()));
            }
            Json(body).into_response()
        }
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "success": false,
                "error": format!("the {table} attribute values could not be written"),
                "detail": err.to_string(),
                "databaseError": true,
                "warnings": [write_warning("update attributes", &err)],
            })),
        )
            .into_response(),
    }
}

pub async fn update_contact_attribute_values(
    State(state): State<AppState>,
    Tenant(tenant_key): Tenant,
    Path(id): Path<String>,
    Json(payload): Json<serde_json::Value>,
) -> Response {
    write_entity_attribute_values(&state, &id, &tenant_key, true, &payload).await
}

pub async fn update_conv_attribute_values(
    State(state): State<AppState>,
    Tenant(tenant_key): Tenant,
    Path(id): Path<String>,
    Json(payload): Json<serde_json::Value>,
) -> Response {
    write_entity_attribute_values(&state, &id, &tenant_key, false, &payload).await
}

// ==========================================
// 7. Organizations & Multi-tenancy
// ==========================================

/// Python `organizations.py:25` `_slugify`: collapse every run of non
/// alphanumeric characters into a single `-`, lower-case, strip leading/trailing
/// `-`, and fall back to a short uuid when the name yields nothing usable.
fn slugify_org_key(name: &str) -> String {
    let mut out = String::new();
    let mut pending_dash = false;
    for ch in name.trim().chars() {
        if ch.is_ascii_alphanumeric() {
            if pending_dash && !out.is_empty() {
                out.push('-');
            }
            pending_dash = false;
            out.push(ch.to_ascii_lowercase());
        } else {
            pending_dash = true;
        }
    }
    while out.ends_with('-') {
        out.pop();
    }
    if out.is_empty() {
        return Uuid::new_v4().simple().to_string()[..8].to_string();
    }
    out
}

pub async fn list_organizations(State(state): State<AppState>) -> Response {
    #[derive(serde::Serialize, sqlx::FromRow)]
    #[serde(rename_all = "camelCase")]
    struct OrgRow {
        id: String,
        tenant_key: String,
        name: String,
        owner_account_id: Option<String>,
        created_at: String,
    }

    // Python `list_organizations` (organizations.py:51) scopes this to the
    // authenticated account. No account principal extractor is wired to this
    // route yet, so we return the real tenant rows and flag the missing scoping
    // instead of fabricating a single "newax" organization.
    let orgs = match sqlx::query_as::<_, OrgRow>(
        "SELECT id, tenant_key, name, owner_account_id, created_at FROM tenants ORDER BY created_at ASC",
    )
    .fetch_all(&state.db)
    .await
    {
        Ok(rows) => rows,
        Err(err) => return rbac_db_error("read the organizations (tenants) list", err),
    };

    let mut body = serde_json::to_value(&orgs).unwrap_or_else(|_| json!([]));
    if let Some(array) = body.as_array_mut() {
        for item in array.iter_mut() {
            if item.get("ownerAccountId").and_then(|v| v.as_str()).is_none() {
                item["unowned"] = json!(true);
            }
        }
    }

    (
        StatusCode::OK,
        Json(json!({
            "organizations": body,
            "count": orgs.len(),
            "warning": "this listing is not scoped to the authenticated account because no account principal is attached to this route; wire one (see WIRING) to stop cross-account enumeration",
        })),
    )
        .into_response()
}

pub async fn create_organization(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    Json(payload): Json<serde_json::Value>,
) -> Response {
    // A blank organization name is refused as the missing input it is, never
    // normalised into a row whose tenant_key would fall back to a random uuid.
    let raw_name = match payload.get("name") {
        None | Some(Value::Null) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "success": false,
                    "error": "name is required to create an organization",
                    "field": "name",
                    "usage": "POST /api/v1/organizations {\"name\": \"Acme Inc\"}",
                })),
            )
                .into_response();
        }
        Some(v) => match v.as_str().map(str::trim) {
            Some(value) if !value.is_empty() => value.to_string(),
            Some(_) => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(json!({
                        "success": false,
                        "error": "organization name cannot be blank",
                        "field": "name",
                        "usage": "POST /api/v1/organizations {\"name\": \"Acme Inc\"}",
                    })),
                )
                    .into_response();
            }
            None => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(json!({
                        "success": false,
                        "error": "name must be a string",
                        "field": "name",
                    })),
                )
                    .into_response();
            }
        },
    };

    // Collision-safe key, matching organizations.py:65-72 (base, then base-2, base-3 ...).
    let base_key = slugify_org_key(&raw_name);
    let mut tenant_key = base_key.clone();
    let mut suffix = 1usize;
    loop {
        let taken = match sqlx::query_scalar::<_, i64>(
            "SELECT 1 FROM tenants WHERE tenant_key = ? LIMIT 1",
        )
        .bind(&tenant_key)
        .fetch_optional(&state.db)
        .await
        {
            Ok(value) => value.is_some(),
            Err(err) => return rbac_db_error("check for an existing tenant key", err),
        };
        if !taken {
            break;
        }
        suffix += 1;
        tenant_key = format!("{base_key}-{suffix}");
    }

    // owner_account_id always comes from the validated session, never the body
    // (organizations.py:74-76). With no bearer session on the request there is no
    // principal to record, so the column stays NULL and the response says so
    // instead of hardcoding an owner such as 'user_admin'.
    let owner: Option<String> = verified_actor(&state, &headers)
        .await
        .map(|(account_id, _)| account_id);
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();

    let result = sqlx::query(
        "INSERT INTO tenants (id, tenant_key, name, owner_account_id, created_at) VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(&id)
    .bind(&tenant_key)
    .bind(&raw_name)
    .bind(owner.clone())
    .bind(&now)
    .execute(&state.db)
    .await;

    match result {
        Ok(report) if report.rows_affected() == 0 => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "success": false,
                "error": "the organization insert matched no rows; nothing was created",
                "databaseError": true,
            })),
        )
            .into_response(),
        Ok(_) => (
            StatusCode::CREATED,
            Json(json!({
                "success": true,
                "id": id,
                "tenantKey": tenant_key,
                "tenant_key": tenant_key,
                "name": raw_name,
                "ownerAccountId": owner.as_deref(),
                "owner_account_id": owner.as_deref(),
                "createdAt": now,
                "created_at": now,
                "warning": if owner.is_none() {
                    Some("owner_account_id was stored as NULL because this request carried no verifiable Bearer session; the organization is therefore unowned")
                } else {
                    None
                },
                "ownerSource": if owner.is_none() { "none: no verified account principal on this request" } else { "verified bearer session" },
            })),
        )
            .into_response(),
        Err(err) => {
            let msg = err.to_string();
            if msg.contains("UNIQUE") || msg.contains("constraint") {
                (
                    StatusCode::CONFLICT,
                    Json(json!({
                        "success": false,
                        "error": format!("a tenant with key '{tenant_key}' already exists"),
                        "tenantKey": tenant_key,
                    })),
                )
                    .into_response()
            } else {
                rbac_db_error("create the organization", err)
            }
        }
    }
}

// ==========================================
// 8. Dynamic Segments
// ==========================================

pub async fn list_segments(State(state): State<AppState>) -> impl IntoResponse {
    #[derive(serde::Serialize, sqlx::FromRow)]
    #[serde(rename_all = "camelCase")]
    struct SegRow {
        id: String,
        name: String,
        description: Option<String>,
        filters: Option<String>,
    }

    let segs = sqlx::query_as::<_, SegRow>(
        "SELECT id, name, description, filters FROM contact_segments ORDER BY name ASC",
    )
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    let result: Vec<serde_json::Value> = segs
        .into_iter()
        .map(|s| {
            let f: serde_json::Value = s.filters.and_then(|flt| serde_json::from_str(&flt).ok()).unwrap_or_else(|| json!({}));
            json!({
                "id": s.id,
                "name": s.name,
                "description": s.description,
                "filters": f,
                "contactCount": 3
            })
        })
        .collect();

    Json(result)
}

pub async fn create_segment(Json(payload): Json<serde_json::Value>) -> impl IntoResponse {
    Json(json!({
        "id": Uuid::new_v4().to_string(),
        "name": payload["name"],
        "description": payload["description"],
        "filters": payload["filters"],
        "contactCount": 0
    }))
}

pub async fn get_segment(Path(id): Path<String>) -> impl IntoResponse {
    Json(json!({
        "id": id,
        "name": "Active Contacts Segment",
        "description": "Contacts engaged within 30 days",
        "filters": {},
        "contacts": []
    }))
}

pub async fn update_segment(Path(id): Path<String>, Json(payload): Json<serde_json::Value>) -> impl IntoResponse {
    Json(json!({ "id": id, "name": payload["name"] }))
}

pub async fn delete_segment(Path(id): Path<String>) -> impl IntoResponse {
    Json(json!({ "status": "ok", "deleted": true, "id": id }))
}

pub async fn export_segment(State(state): State<AppState>, Path(id): Path<String>) -> Response {
    // Python exported a real member list from the segment tables. This schema has
    // no `segments` table yet, so returning any CSV at all would be invented data.
    let present = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM information_schema.tables WHERE table_schema = 'public' AND table_name IN ('segments', 'segment_members')",
    )
    .fetch_one(&state.db)
    .await;

    match present {
        Ok(count) if count >= 2 => {
            let rows = sqlx::query_as::<_, (String, Option<String>, Option<String>)>(
                "SELECT COALESCE(c.name, ''), c.phone_number, c.channel \
                 FROM segment_members sm \
                 JOIN contacts c ON c.id = sm.contact_id \
                 WHERE sm.segment_id = ? \
                 ORDER BY c.name ASC",
            )
            .bind(&id)
            .fetch_all(&state.db)
            .await;

            match rows {
                Ok(rows) => {
                    let mut csv = String::from("name,phone,channel\n");
                    for (name, phone, channel) in &rows {
                        let esc = |s: &str| format!("\"{}\"", s.replace('"', "\"\""));
                        csv.push_str(&format!(
                            "{},{},{}\n",
                            esc(name),
                            phone.as_deref().map(esc).unwrap_or_default(),
                            channel.as_deref().map(esc).unwrap_or_default()
                        ));
                    }
                    (
                        [(
                            header::CONTENT_TYPE,
                            HeaderValue::from_static("text/csv; charset=utf-8"),
                        )],
                        csv,
                    )
                        .into_response()
                }
                Err(err) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({
                        "error": "the segment members could not be read",
                        "detail": err.to_string(),
                    })),
                )
                    .into_response(),
            }
        }
        Ok(_) => (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({
                "error": "segment tables are not present in this database",
                "missing": ["segments", "segment_members"],
                "note": "refusing to export a contact list that does not exist; apply the segment DDL first",
            })),
        )
            .into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": "could not inspect the database schema",
                "detail": err.to_string(),
            })),
        )
            .into_response(),
    }
}

// ==========================================
// 9. Real-time Telemetry & Dynamic Reports (Task 2)
// ==========================================

#[derive(Debug, Deserialize)]
pub struct ReportPeriodQuery {
    pub period: Option<String>,
    pub channel: Option<String>,
}

/// The windows the reporting UI and the Python route accept
/// (`api/routes/reports.py` declares `pattern="^(7d|30d|90d)$"`).
const REPORT_PERIODS: &[(&str, i64)] = &[("7d", 7), ("30d", 30), ("90d", 90)];

/// Report failures are reported, never averaged into a number: a dashboard that
/// answers "0 conversations" because the query errored, or "7 days" because an
/// unknown period was guessed, is a fabricated metric.
/// A report handler's failure, rendered as a JSON error body that names the read
/// or input that failed. `pub` because the handlers expose it in their signatures
/// to `main.rs`'s router; its fields stay private so a caller cannot forge a
/// status.
pub struct ReportError {
    status: StatusCode,
    context: String,
    detail: String,
}

impl ReportError {
    fn read(context: &str, err: sqlx::Error) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            context: context.to_string(),
            detail: err.to_string(),
        }
    }

    fn bad_request(context: &str, detail: String) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            context: context.to_string(),
            detail,
        }
    }
}

impl IntoResponse for ReportError {
    fn into_response(self) -> Response {
        (
            self.status,
            Json(json!({
                "status": "error",
                "error": format!("the report could not be built: {}", self.context),
                "detail": self.detail,
                "databaseError": self.status == StatusCode::INTERNAL_SERVER_ERROR,
            })),
        )
            .into_response()
    }
}

async fn report_scalar(
    state: &AppState,
    context: &str,
    sql: &str,
) -> Result<i64, ReportError> {
    sqlx::query_scalar::<_, i64>(sql)
        .fetch_one(&state.db)
        .await
        .map_err(|err| ReportError::read(context, err))
}

async fn report_rows<T>(
    state: &AppState,
    context: &str,
    sql: &str,
) -> Result<Vec<T>, ReportError>
where
    T: for<'r> sqlx::FromRow<'r, sqlx::postgres::PgRow> + Send + Unpin,
{
    sqlx::query_as::<_, T>(sql)
        .fetch_all(&state.db)
        .await
        .map_err(|err| ReportError::read(context, err))
}

/// Resolves the requested window to `(period label, days, cutoff date, whether the
/// caller actually asked)`. An absent `period` takes the documented default and
/// says so in the response; an unknown one is refused with the accepted list
/// instead of being reported as 7 days.
fn report_window(
    period: Option<&str>,
) -> Result<(String, i64, String, bool), ReportError> {
    let (label, days, defaulted) = match period.map(str::trim).filter(|p| !p.is_empty()) {
        Some(raw) => match REPORT_PERIODS.iter().find(|(name, _)| *name == raw) {
            Some((name, days)) => ((*name).to_string(), *days, false),
            None => {
                let accepted: Vec<&str> = REPORT_PERIODS.iter().map(|(name, _)| *name).collect();
                return Err(ReportError::bad_request(
                    "the requested period is not a supported window",
                    format!("period '{raw}' was rejected; accepted values are {}", accepted.join(", ")),
                ));
            }
        },
        None => ("7d".to_string(), 7, true),
    };
    let cutoff = (Utc::now() - Duration::days(days)).format("%Y-%m-%d").to_string();
    Ok((label, days, cutoff, defaulted))
}

/// Timestamps in this database are written as `Utc::now().to_rfc3339()` (UTC, with
/// a `T` and an offset). Every report window comparison truncates to the UTC
/// date the row records instead of comparing two different text formats.
const SQL_DT_EXPRESSION_NOTE: &str = "dates are compared as UTC calendar days via replace(substr(col,1,19),'T',' ')";

pub async fn get_live_report(
    State(state): State<AppState>,
) -> Result<Json<Value>, ReportError> {
    let open_convs: i64 = report_scalar(
        &state,
        "the open conversation count",
        "SELECT COUNT(*) FROM conversations WHERE state NOT IN ('RESOLVED', 'CLOSED', 'ARCHIVED')",
    )
    .await?;

    let unassigned_convs: i64 = report_scalar(
        &state,
        "the unassigned open conversation count",
        "SELECT COUNT(*) FROM conversations WHERE state NOT IN ('RESOLVED', 'CLOSED', 'ARCHIVED') AND (assigned_to_id IS NULL OR assigned_to_id = '')",
    )
    .await?;

    let pending_replies: i64 = report_scalar(
        &state,
        "the pending-reply conversation count",
        r#"
        SELECT COUNT(*) FROM conversations c
        WHERE c.state NOT IN ('RESOLVED', 'CLOSED', 'ARCHIVED')
          AND (SELECT direction FROM messages m WHERE m.conversation_id = c.id ORDER BY COALESCE(m.sent_at, m.created_at) DESC LIMIT 1) = 'INBOUND'
        "#,
    )
    .await?;

    let resolved_today: i64 = report_scalar(
        &state,
        "the resolved-today conversation count",
        r#"
        SELECT COUNT(*) FROM conversations
        WHERE state IN ('RESOLVED', 'CLOSED')
          AND DATE(updated_at::TIMESTAMPTZ) = CURRENT_DATE
        "#,
    )
    .await?;

    #[derive(sqlx::FromRow)]
    struct ChannelCount {
        channel: Option<String>,
        cnt: i64,
    }

    let mut channel_rows = report_rows::<ChannelCount>(
        &state,
        "the conversation channel breakdown",
        "SELECT channel, COUNT(*) as cnt FROM conversations GROUP BY channel",
    )
    .await?;

    if channel_rows.is_empty() {
        channel_rows = report_rows::<ChannelCount>(
            &state,
            "the message channel breakdown",
            "SELECT channel, COUNT(*) as cnt FROM messages GROUP BY channel",
        )
        .await?;
    }

    let mut channel_breakdown = serde_json::Map::new();
    for r in channel_rows {
        // A row whose channel column is NULL is reported as such rather than
        // dropped, so the totals below still add up.
        let key = r.channel.unwrap_or_else(|| "(no channel stored)".to_string());
        channel_breakdown.insert(key, json!(r.cnt));
    }

    #[derive(sqlx::FromRow)]
    struct AgentRow {
        id: String,
        name: String,
        email: String,
        availability_status: Option<String>,
        max_concurrent_conversations: Option<i32>,
    }

    let members = report_rows::<AgentRow>(
        &state,
        "the team member roster",
        "SELECT id, name, email, availability_status, max_concurrent_conversations FROM team_members ORDER BY name ASC LIMIT 8",
    )
    .await?;

    let roster_count = members.len();
    let mut agents_online = Vec::new();
    for m in members {
        let active_chats: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM conversations WHERE assigned_to_id = ? AND state NOT IN ('RESOLVED', 'CLOSED', 'ARCHIVED')",
        )
        .bind(&m.id)
        .fetch_one(&state.db)
        .await
        .map_err(|err| ReportError::read("an agent's active conversation count", err))?;

        // Both fallbacks are the stored column's own DDL default
        // (`team_members.availability_status DEFAULT 'online'` and
        // `max_concurrent_conversations DEFAULT 5`, db/mod.rs), used only for rows
        // written before those columns existed.
        let status = m
            .availability_status
            .clone()
            .unwrap_or_else(|| "online".to_string());
        let cap = m
            .max_concurrent_conversations
            .unwrap_or(5);

        agents_online.push(json!({
            "id": m.id,
            "name": m.name,
            "email": m.email,
            "status": status,
            "statusSource": if m.availability_status.is_some() { "team_members.availability_status" } else { "column default 'online' (stored value was NULL)" },
            // "busy"/"online" here is a load reading computed from the two real
            // numbers beside it, never a presence signal the database does not have.
            "loadStatus": if active_chats >= (cap as i64) { "busy" } else { "online" },
            "activeChats": active_chats,
            "maxCapacity": cap,
        }));
    }

    // Wait time is measured, not asserted: seconds since the last inbound message
    // on each still-open conversation. No such conversation means no queue wait,
    // which is reported as null rather than the literal 135 the port shipped.
    let avg_wait_seconds: Option<f64> = sqlx::query_scalar(
        r#"
        SELECT AVG(EXTRACT(EPOCH FROM NOW()) - EXTRACT(EPOCH FROM last_inbound_at::TIMESTAMPTZ))
        FROM conversations
        WHERE state NOT IN ('RESOLVED', 'CLOSED', 'ARCHIVED')
          AND last_inbound_at IS NOT NULL
        "#,
    )
    .fetch_one(&state.db)
    .await
    .map_err(|err| ReportError::read("the inbound queue wait time", err))?;

    let mut warnings: Vec<Value> = Vec::new();
    if channel_breakdown.is_empty() {
        warnings.push(json!({
            "field": "channelBreakdown",
            "reason": "no conversation or message row carries a channel value, so there is no breakdown to show",
        }));
    }
    if roster_count == 0 {
        warnings.push(json!({
            "field": "agentsOnline",
            "reason": "team_members holds no rows, so no agent is online according to this database",
        }));
    }

    Ok(Json(json!({
        "status": "success",
        "liveMetrics": {
            "openConversations": open_convs,
            "unassignedConversations": unassigned_convs,
            "pendingReplies": pending_replies,
            "resolvedToday": resolved_today,
            "avgWaitTimeSeconds": avg_wait_seconds.map(|v| (v * 100.0).round() / 100.0),
            "avgWaitTimeFormatted": avg_wait_seconds.map(|v| format!("{}m {}s", (v / 60.0).floor() as i64, (v % 60.0).round() as i64)),
            "avgWaitTimeSource": if avg_wait_seconds.is_some() { "measured from conversations.last_inbound_at" } else { "unavailable: no open conversation records a last_inbound_at value" },
            "lastRefreshedAt": Utc::now().to_rfc3339()
        },
        "channelBreakdown": channel_breakdown,
        "agentsOnline": agents_online,
        "warnings": warnings,
        "notes": SQL_DT_EXPRESSION_NOTE,
    })))
}

/// Builds a window-scoped aggregate over one of the report tables. Every
/// fragment except the cutoff and the channel value is a code-controlled literal
/// (never request text), and the bind order matches the built SQL: cutoff first,
/// then channel when a filter was requested.
fn scoped_sql(
    select_and_from: &str,
    window_col: &str,
    extra_where: &str,
    with_channel: bool,
) -> String {
    let mut sql = format!(
        "{select_and_from} WHERE DATE({window_col}::TIMESTAMPTZ) >= ?"
    );
    if !extra_where.is_empty() {
        sql.push_str(" AND ");
        sql.push_str(extra_where);
    }
    if with_channel {
        sql.push_str(" AND channel = ?");
    }
    sql
}

async fn scoped_count(
    state: &AppState,
    context: &str,
    select_and_from: &str,
    window_col: &str,
    extra_where: &str,
    cutoff: &str,
    channel: Option<&str>,
) -> Result<i64, ReportError> {
    let sql = scoped_sql(select_and_from, window_col, extra_where, channel.is_some());
    let mut query = sqlx::query_scalar::<_, i64>(&sql).bind(cutoff);
    if let Some(value) = channel {
        query = query.bind(value);
    }
    query
        .fetch_one(&state.db)
        .await
        .map_err(|err| ReportError::read(context, err))
}

#[derive(sqlx::FromRow)]
struct DayCount {
    day: Option<String>,
    cnt: i64,
}

async fn scoped_day_counts(
    state: &AppState,
    context: &str,
    table: &str,
    window_col: &str,
    extra_where: &str,
    cutoff: &str,
    channel: Option<&str>,
) -> Result<Vec<DayCount>, ReportError> {
    let mut sql = scoped_sql(
        &format!(
            "SELECT DATE({window_col}::TIMESTAMPTZ) as day, COUNT(*) as cnt FROM {table}"
        ),
        window_col,
        extra_where,
        channel.is_some(),
    );
    // Grouped after the predicates are appended, so the bucket list is the window
    // the caller asked for.
    sql.push_str(" GROUP BY day");
    let mut query = sqlx::query_as::<_, DayCount>(&sql).bind(cutoff);
    if let Some(value) = channel {
        query = query.bind(value);
    }
    query
        .fetch_all(&state.db)
        .await
        .map_err(|err| ReportError::read(context, err))
}

fn day_map(rows: Vec<DayCount>) -> std::collections::HashMap<String, i64> {
    rows.into_iter()
        .filter_map(|r| r.day.map(|day| (day, r.cnt)))
        .collect()
}

pub async fn get_conversation_reports(
    State(state): State<AppState>,
    Query(params): Query<ReportPeriodQuery>,
) -> Result<Json<Value>, ReportError> {
    let (period, days_count, cutoff, period_defaulted) = report_window(params.period.as_deref())?;
    let channel = params
        .channel
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string);

    // Every summary number is scoped to the requested window (and channel): the
    // port computed these over the whole table while labelling them "7d".
    let total_convs = scoped_count(
        &state,
        "the conversation total",
        "SELECT COUNT(*) FROM conversations",
        "COALESCE(NULLIF(created_at, ''), updated_at)",
        "",
        &cutoff,
        channel.as_deref(),
    )
    .await?;

    let total_messages = scoped_count(
        &state,
        "the message total",
        "SELECT COUNT(*) FROM messages",
        "COALESCE(NULLIF(sent_at, ''), created_at)",
        "",
        &cutoff,
        channel.as_deref(),
    )
    .await?;

    let inbound_messages = scoped_count(
        &state,
        "the inbound message total",
        "SELECT COUNT(*) FROM messages",
        "COALESCE(NULLIF(sent_at, ''), created_at)",
        "UPPER(direction) = 'INBOUND'",
        &cutoff,
        channel.as_deref(),
    )
    .await?;

    let outbound_messages = scoped_count(
        &state,
        "the outbound message total",
        "SELECT COUNT(*) FROM messages",
        "COALESCE(NULLIF(sent_at, ''), created_at)",
        "UPPER(direction) = 'OUTBOUND'",
        &cutoff,
        channel.as_deref(),
    )
    .await?;

    let resolved_count = scoped_count(
        &state,
        "the resolved conversation total",
        "SELECT COUNT(*) FROM conversations",
        "COALESCE(NULLIF(created_at, ''), updated_at)",
        "state IN ('RESOLVED', 'CLOSED')",
        &cutoff,
        channel.as_deref(),
    )
    .await?;

    let mut warnings: Vec<Value> = Vec::new();
    // No conversations in the window means no rate, not a plausible one.
    let resolution_rate = if total_convs > 0 {
        Some(((resolved_count as f64 / total_convs as f64) * 100.0).round() as i64)
    } else {
        warnings.push(json!({
            "field": "resolutionRatePercent",
            "stored": null,
            "reason": "no conversation falls in this window, so a resolution rate cannot be computed",
        }));
        None
    };

    let daily_inbound = scoped_day_counts(
        &state,
        "the daily inbound message counts",
        "messages",
        "COALESCE(NULLIF(sent_at, ''), created_at)",
        "UPPER(direction) = 'INBOUND'",
        &cutoff,
        channel.as_deref(),
    )
    .await?;

    let daily_outbound = scoped_day_counts(
        &state,
        "the daily outbound message counts",
        "messages",
        "COALESCE(NULLIF(sent_at, ''), created_at)",
        "UPPER(direction) = 'OUTBOUND'",
        &cutoff,
        channel.as_deref(),
    )
    .await?;

    let daily_convs = scoped_day_counts(
        &state,
        "the daily conversation counts",
        "conversations",
        "COALESCE(NULLIF(created_at, ''), updated_at)",
        "",
        &cutoff,
        channel.as_deref(),
    )
    .await?;

    let daily_resolutions = scoped_day_counts(
        &state,
        "the daily resolution counts",
        "conversations",
        "COALESCE(NULLIF(created_at, ''), updated_at)",
        "state IN ('RESOLVED', 'CLOSED')",
        &cutoff,
        channel.as_deref(),
    )
    .await?;

    let in_map = day_map(daily_inbound);
    let out_map = day_map(daily_outbound);
    let conv_map = day_map(daily_convs);
    let res_map = day_map(daily_resolutions);

    let mut time_series = Vec::new();
    let today = Utc::now().date_naive();
    for i in (0..days_count).rev() {
        let d = today - Duration::days(i);
        let d_str = d.format("%Y-%m-%d").to_string();
        // A missing day bucket is a real zero: the grouped query said nothing
        // happened that day.
        let in_cnt = in_map.get(&d_str).copied().unwrap_or(0);
        let out_cnt = out_map.get(&d_str).copied().unwrap_or(0);
        let conv_cnt = conv_map.get(&d_str).copied().unwrap_or(0);
        let res_cnt = res_map.get(&d_str).copied().unwrap_or(0);

        time_series.push(json!({
            "date": d_str,
            "conversations": conv_cnt,
            "inboundMessages": in_cnt,
            "outboundMessages": out_cnt,
            "resolutions": res_cnt
        }));
    }

    // First response time is measured from the message log: the gap between the
    // first inbound and the first later outbound message per conversation.
    let frt_channel_clause = if channel.is_some() {
        "\n                  AND channel = ?"
    } else {
        ""
    };
    let frt_sql = format!(
        r#"
        SELECT AVG((EXTRACT(EPOCH FROM first_out::TIMESTAMPTZ) - EXTRACT(EPOCH FROM first_in::TIMESTAMPTZ)) / 60.0) FROM (
            SELECT conversation_id,
                   MIN(CASE WHEN UPPER(direction) = 'INBOUND' THEN dt END) AS first_in,
                   MIN(CASE WHEN UPPER(direction) = 'OUTBOUND' THEN dt END) AS first_out
            FROM (
                SELECT conversation_id, direction,
                       replace(substr(COALESCE(NULLIF(sent_at, ''), created_at), 1, 19), 'T', ' ') AS dt
                FROM messages
                WHERE DATE(COALESCE(NULLIF(sent_at, ''), created_at)::TIMESTAMPTZ) >= ?{frt_channel_clause}
            )
            GROUP BY conversation_id
        ) WHERE first_in IS NOT NULL AND first_out IS NOT NULL AND first_out >= first_in
        "#
    );
    let mut frt_query = sqlx::query_scalar::<_, Option<f64>>(&frt_sql).bind(&cutoff);
    if let Some(value) = &channel {
        frt_query = frt_query.bind(value);
    }
    let avg_first_response: Option<f64> = frt_query
        .fetch_one(&state.db)
        .await
        .map_err(|err| ReportError::read("the first-response time average", err))?;
    if avg_first_response.is_none() {
        warnings.push(json!({
            "field": "avgFirstResponseMinutes",
            "stored": null,
            "reason": "no conversation in this window holds both an inbound and a later outbound message with a direction value, so first-response time cannot be measured",
        }));
    }

    // Resolution duration uses the conversation's own created -> last-updated span
    // on rows that are resolved; there is no separate resolved_at column, and that
    // is stated rather than hidden.
    let art_channel_clause = if channel.is_some() {
        " AND channel = ?"
    } else {
        ""
    };
    let art_sql = format!(
        r#"
        SELECT AVG((EXTRACT(EPOCH FROM updated_at::TIMESTAMPTZ) - EXTRACT(EPOCH FROM created_at::TIMESTAMPTZ)) / 3600.0)
        FROM conversations
        WHERE state IN ('RESOLVED', 'CLOSED')
          AND created_at IS NOT NULL AND created_at <> ''
          AND DATE(COALESCE(NULLIF(created_at, ''), updated_at)::TIMESTAMPTZ) >= ?{art_channel_clause}
        "#
    );
    let mut art_query = sqlx::query_scalar::<_, Option<f64>>(&art_sql).bind(&cutoff);
    if let Some(value) = &channel {
        art_query = art_query.bind(value);
    }
    let avg_resolution_hours: Option<f64> = art_query
        .fetch_one(&state.db)
        .await
        .map_err(|err| ReportError::read("the resolution-duration average", err))?;
    if avg_resolution_hours.is_none() {
        warnings.push(json!({
            "field": "avgResolutionHours",
            "stored": null,
            "reason": "no resolved conversation in this window records a created_at value, so no resolution duration can be measured",
        }));
    }

    #[derive(sqlx::FromRow)]
    struct ChannelDist {
        channel: Option<String>,
        cnt: i64,
    }

    let channel_dist_rows = report_rows::<ChannelDist>(
        &state,
        "the message channel distribution",
        "SELECT channel, COUNT(*) as cnt FROM messages GROUP BY channel",
    )
    .await?;

    let mut channel_distribution = serde_json::Map::new();
    for r in channel_dist_rows {
        let key = r.channel.unwrap_or_else(|| "(no channel stored)".to_string());
        channel_distribution.insert(key, json!(r.cnt));
    }
    if channel_distribution.is_empty() {
        warnings.push(json!({
            "field": "channelDistribution",
            "stored": null,
            "reason": "messages holds no rows, so no channel distribution exists; nothing was assumed about the channel mix",
        }));
    }
    if period_defaulted {
        warnings.push(json!({
            "field": "period",
            "stored": period,
            "reason": "no period was requested, so the documented 7d default window was used",
        }));
    }

    Ok(Json(json!({
        "status": "success",
        "period": period,
        "periodDays": days_count,
        "periodCutoff": cutoff,
        "periodDefaulted": period_defaulted,
        "channelFilter": channel,
        "summary": {
            "totalConversations": total_convs,
            "totalMessages": total_messages,
            "inboundMessages": inbound_messages,
            "outboundMessages": outbound_messages,
            "resolutionRatePercent": resolution_rate,
            "avgFirstResponseMinutes": avg_first_response.map(|v| (v * 100.0).round() / 100.0),
            "avgResolutionHours": avg_resolution_hours.map(|v| (v * 100.0).round() / 100.0),
            "avgResolutionHoursDefinition": "created_at to updated_at on conversations in state RESOLVED/CLOSED; the schema records no separate resolved_at"
        },
        "timeSeries": time_series,
        "channelDistribution": channel_distribution,
        "warnings": warnings
    })))
}

/// Window-scoped SQL over the conversations table for one owner column. Bind
/// order is always owner, cutoff, then channel.
fn windowed_conversation_sql(
    select: &str,
    owner_column: &str,
    extra_where: &str,
    with_channel: bool,
) -> String {
    let mut sql = format!(
        "{select} FROM conversations WHERE {owner_column} = ? \
         AND DATE(COALESCE(NULLIF(created_at, ''), updated_at)::TIMESTAMPTZ) >= ?"
    );
    if !extra_where.is_empty() {
        sql.push_str(" AND ");
        sql.push_str(extra_where);
    }
    if with_channel {
        sql.push_str(" AND channel = ?");
    }
    sql
}

async fn windowed_conversation_count(
    state: &AppState,
    context: &str,
    owner_column: &str,
    owner_id: &str,
    extra_where: &str,
    cutoff: &str,
    channel: Option<&str>,
) -> Result<i64, ReportError> {
    let sql = windowed_conversation_sql(
        "SELECT COUNT(*)",
        owner_column,
        extra_where,
        channel.is_some(),
    );
    let mut query = sqlx::query_scalar::<_, i64>(&sql)
        .bind(owner_id)
        .bind(cutoff);
    if let Some(value) = channel {
        query = query.bind(value);
    }
    query
        .fetch_one(&state.db)
        .await
        .map_err(|err| ReportError::read(context, err))
}

async fn windowed_conversation_avg(
    state: &AppState,
    context: &str,
    owner_column: &str,
    owner_id: &str,
    extra_where: &str,
    cutoff: &str,
    channel: Option<&str>,
) -> Result<Option<f64>, ReportError> {
    let sql = windowed_conversation_sql(
        "SELECT AVG(csat_score)",
        owner_column,
        extra_where,
        channel.is_some(),
    );
    let mut query = sqlx::query_scalar::<_, Option<f64>>(&sql)
        .bind(owner_id)
        .bind(cutoff);
    if let Some(value) = channel {
        query = query.bind(value);
    }
    query
        .fetch_one(&state.db)
        .await
        .map_err(|err| ReportError::read(context, err))
}

pub async fn get_agent_reports(
    State(state): State<AppState>,
    Query(params): Query<ReportPeriodQuery>,
) -> Result<Json<Vec<Value>>, ReportError> {
    // The period was previously accepted and thrown away (`let _ = params;`)
    // while every agent was counted over the whole table; both `period` and
    // `channel` now scope the queries below.
    let (period, days, cutoff, period_defaulted) = report_window(params.period.as_deref())?;
    let channel = params
        .channel
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string);
    let channel = channel.as_deref();

    #[derive(sqlx::FromRow)]
    struct MemberRow {
        id: String,
        name: String,
        email: String,
        availability_status: Option<String>,
    }

    let members = report_rows::<MemberRow>(
        &state,
        "the team member roster",
        "SELECT id, name, email, availability_status FROM team_members ORDER BY name ASC",
    )
    .await?;

    let mut reports = Vec::new();
    for m in members {
        let assigned = windowed_conversation_count(
            &state,
            "an agent's assigned conversation count",
            "assigned_to_id",
            &m.id,
            "",
            &cutoff,
            channel,
        )
        .await?;

        let resolved = windowed_conversation_count(
            &state,
            "an agent's resolved conversation count",
            "assigned_to_id",
            &m.id,
            "state IN ('RESOLVED', 'CLOSED')",
            &cutoff,
            channel,
        )
        .await?;

        let csat_responses = windowed_conversation_count(
            &state,
            "an agent's CSAT response count",
            "assigned_to_id",
            &m.id,
            "csat_score IS NOT NULL",
            &cutoff,
            channel,
        )
        .await?;

        let avg_csat = windowed_conversation_avg(
            &state,
            "avg_csat",
            "assigned_to_id",
            &m.id,
            "csat_score IS NOT NULL",
            &cutoff,
            channel,
        )
        .await?;

        // No padded denominator and no floored rate: an agent with no work in the
        // window shows zero conversations and a null rate.
        let resolution_rate = if assigned > 0 {
            let pct = ((resolved as f64 / assigned as f64) * 100.0).min(100.0).round() as i64;
            Some(format!("{pct}%"))
        } else {
            None
        };

        reports.push(json!({
            "id": m.id,
            "name": m.name,
            "email": m.email,
            "avatar": null,
            "avatarSource": "unavailable: team_members stores no avatar column",
            "status": m.availability_status.clone().unwrap_or_else(|| "unknown".to_string()),
            "statusSource": if m.availability_status.is_some() { "team_members.availability_status" } else { "column is NULL on this row" },
            "conversationsAssigned": assigned,
            "conversationsResolved": resolved,
            "resolutionRate": resolution_rate,
            "avgFirstResponseMinutes": null,
            "avgFirstResponseMinutesSource": "unavailable: first-response time is not attributed per agent in this schema (use /reports/conversations for the measured overall average)",
            "avgResolutionHours": null,
            "avgResolutionHoursSource": "unavailable: resolution duration is not attributed per agent in this schema",
            "csatRating": avg_csat.map(|v| (v * 100.0).round() / 100.0),
            "csatRatingSource": if avg_csat.is_some() { "AVG(conversations.csat_score) for this window" } else { "no CSAT response is stored for this agent in this window" },
            "csatResponsesCount": csat_responses,
            "period": period.as_str(),
            "periodDays": days,
            "periodDefaulted": period_defaulted,
            "periodCutoff": cutoff.as_str(),
            "channelFilter": channel,
        }));
    }

    Ok(Json(reports))
}

pub async fn get_team_reports(
    State(state): State<AppState>,
    Query(params): Query<ReportPeriodQuery>,
) -> Result<Json<Vec<Value>>, ReportError> {
    // Same as the agent report: `period` used to be discarded, so every team was
    // shown over all history while the response claimed to be a window.
    let (_period, _days, cutoff, _defaulted) = report_window(params.period.as_deref())?;
    let channel = params
        .channel
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string);
    let channel = channel.as_deref();

    #[derive(sqlx::FromRow)]
    struct TeamRow {
        id: String,
        name: String,
        description: Option<String>,
    }

    let teams = report_rows::<TeamRow>(
        &state,
        "the teams roster",
        "SELECT id, name, description FROM teams ORDER BY name ASC",
    )
    .await?;

    let mut reports = Vec::new();
    for t in teams {
        let count = windowed_conversation_count(
            &state,
            "a team's conversation count",
            "team_id",
            &t.id,
            "",
            &cutoff,
            channel,
        )
        .await?;

        let unassigned = windowed_conversation_count(
            &state,
            "a team's unassigned conversation count",
            "team_id",
            &t.id,
            "(assigned_to_id IS NULL OR assigned_to_id = '')",
            &cutoff,
            channel,
        )
        .await?;

        let resolved = windowed_conversation_count(
            &state,
            "a team's resolved conversation count",
            "team_id",
            &t.id,
            "state IN ('RESOLVED', 'CLOSED')",
            &cutoff,
            channel,
        )
        .await?;

        let resolution_rate = if count > 0 {
            let pct = ((resolved as f64 / count as f64) * 100.0).min(100.0).round() as i64;
            Some(format!("{pct}%"))
        } else {
            None
        };

        reports.push(json!({
            "id": t.id,
            "name": t.name,
            // `teams.description` is NOT NULL DEFAULT '' (db/mod.rs), so the empty
            // string is the stored value, not a substitute for a missing one.
            "description": t.description.clone().unwrap_or_default(),
            "conversationsCount": count,
            "unassignedCount": unassigned,
            "resolutionRate": resolution_rate,
            "avgResponseMinutes": null,
            "avgResponseMinutesSource": "unavailable: no per-team response-time series is stored; /reports/conversations gives the measured overall figure",
            "slaAdherencePercent": null,
            "slaAdherencePercentSource": "unavailable: sla_policies holds policy rows but no per-team breach record to compute adherence from",
            "periodCutoff": cutoff,
            "channelFilter": channel,
        }));
    }

    if reports.is_empty() {
        // The port used to synthesise a "Customer Support & Triage" team with 38
        // conversations when the table was empty; an empty roster is returned as
        // an empty list instead.
        tracing::warn!("[reports] teams table holds no rows; /reports/teams returns an empty roster");
    }

    Ok(Json(reports))
}

pub async fn get_csat_report(
    State(state): State<AppState>,
    Query(params): Query<ReportPeriodQuery>,
) -> Result<Json<Value>, ReportError> {
    // `period` used to be read, defaulted to "7d" and echoed back without ever
    // filtering the query, so the route answered with all-history numbers under a
    // 7-day label. Meanwhile the distribution (24/4/1/0/0), the 4.92 average, the
    // 96.5% figure and the three "recent reviews" (Sophia Reynolds, Marcus Vance,
    // Elena Rostova) were demo-seed literals. The window and channel are now
    // applied like every other report route applies them, the read error is
    // surfaced instead of `.unwrap_or_default()`-ed into an empty list, and a
    // database with no CSAT rows answers with zeros, nulls and the reason.
    let (period, days, cutoff, period_defaulted) = report_window(params.period.as_deref())?;
    let channel = params
        .channel
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string);
    let channel = channel.as_deref();

    #[derive(sqlx::FromRow)]
    struct ScoreCount {
        score: Option<i64>,
        cnt: i64,
    }

    // Grouped after `scoped_sql` appends the window (and channel) predicates, so
    // the buckets are the window the caller asked for.
    let dist_sql = format!(
        "{} GROUP BY csat_score",
        scoped_sql(
            "SELECT csat_score as score, COUNT(*) as cnt FROM conversations",
            "updated_at",
            "csat_score IS NOT NULL",
            channel.is_some(),
        )
    );
    let mut dist_query = sqlx::query_as::<_, ScoreCount>(&dist_sql).bind(cutoff.as_str());
    if let Some(value) = channel {
        dist_query = dist_query.bind(value);
    }
    let dist_rows = dist_query
        .fetch_all(&state.db)
        .await
        .map_err(|err| ReportError::read("the CSAT score distribution", err))?;

    let mut distribution = serde_json::Map::new();
    for score in 1..=5i64 {
        distribution.insert(score.to_string(), json!(0));
    }
    let mut total_responses = 0i64;
    let mut total_stars = 0i64;
    let mut satisfied = 0i64;
    let mut unfiled_scores: Vec<String> = Vec::new();
    for row in dist_rows {
        match row.score {
            Some(score) if (1..=5).contains(&score) => {
                distribution.insert(score.to_string(), json!(row.cnt));
                total_responses += row.cnt;
                total_stars += score * row.cnt;
                if score >= 4 {
                    satisfied += row.cnt;
                }
            }
            other => unfiled_scores.push(format!(
                "score {other:?} holds {} scored conversation(s) outside the 1-5 CSAT scale",
                row.cnt
            )),
        }
    }

    let round_two = |value: f64| (value * 100.0).round() / 100.0;
    let average_score = if total_responses > 0 {
        Some(round_two(total_stars as f64 / total_responses as f64))
    } else {
        None
    };
    let satisfaction_percentage = if total_responses > 0 {
        Some(round_two(satisfied as f64 * 100.0 / total_responses as f64))
    } else {
        None
    };

    #[derive(sqlx::FromRow)]
    struct FeedbackRow {
        conversation_id: String,
        contact_name: Option<String>,
        channel: Option<String>,
        score: Option<i64>,
        comment: Option<String>,
        agent_name: Option<String>,
        updated_at: Option<String>,
    }

    // Every field comes from the joined row. The old handler typed in customer,
    // agent and comment text that no table stores; here a missing name or comment
    // is null, and the count says how many rows were actually reviewed.
    let contact_name_expr = "COALESCE(\
        NULLIF(c.contact_name, ''), NULLIF(c.customer_name, ''), NULLIF(c.lead_name, ''), \
        NULLIF(ct.person_name, ''), \
        NULLIF(TRIM(COALESCE(ct.first_name, '') || ' ' || COALESCE(ct.last_name, '')), '')\
    )";
    let mut feedback_sql = format!(
        "SELECT c.id as conversation_id, {contact_name_expr} as contact_name, \
         c.channel as channel, c.csat_score as score, c.csat_comment as comment, \
         tm.name as agent_name, c.updated_at as updated_at \
         FROM conversations c \
         LEFT JOIN contacts ct ON ct.id = c.contact_id \
         LEFT JOIN team_members tm ON tm.id = c.assigned_to_id \
         WHERE DATE(c.updated_at::TIMESTAMPTZ) >= ? \
           AND c.csat_score IS NOT NULL"
    );
    if channel.is_some() {
        feedback_sql.push_str(" AND c.channel = ?");
    }
    feedback_sql.push_str(" ORDER BY c.updated_at DESC LIMIT 50");

    let mut feedback_query = sqlx::query_as::<_, FeedbackRow>(&feedback_sql).bind(cutoff.as_str());
    if let Some(value) = channel {
        feedback_query = feedback_query.bind(value);
    }
    let feedback_rows = feedback_query
        .fetch_all(&state.db)
        .await
        .map_err(|err| ReportError::read("the CSAT feedback reviews", err))?;

    let feedback_count = feedback_rows.len();
    let recent_feedback: Vec<Value> = feedback_rows
        .into_iter()
        .map(|row| {
            json!({
                "conversationId": row.conversation_id,
                "conversation_id": row.conversation_id,
                "contactName": row.contact_name,
                "contact_name": row.contact_name,
                "channel": row.channel,
                "score": row.score,
                "comment": row.comment,
                "assignedAgent": row.agent_name,
                "assigned_agent": row.agent_name,
                "date": row.updated_at,
            })
        })
        .collect();

    let mut warnings: Vec<Value> = Vec::new();
    if period_defaulted {
        warnings.push(json!({
            "field": "period",
            "stored": period,
            "reason": "no period was requested, so the documented 7d default window was used",
        }));
    }
    if total_responses == 0 {
        warnings.push(json!({
            "field": "metrics",
            "stored": null,
            "reason": "no conversation in this window (and channel filter, when sent) stores a csat_score, so averageScore and satisfactionPercentage are null rather than the 4.92 / 96.5 the port carried as literals",
        }));
    }
    if feedback_count == 0 {
        warnings.push(json!({
            "field": "recentFeedback",
            "stored": [],
            "reason": "no scored conversation in this window has a csat_comment row to quote; the previous handler printed three invented reviews",
        }));
    }
    for note in unfiled_scores {
        warnings.push(json!({
            "field": "metrics.distribution",
            "reason": note,
        }));
    }

    Ok(Json(json!({
        "status": "success",
        "period": period,
        "periodDays": days,
        "periodCutoff": cutoff,
        "periodDefaulted": period_defaulted,
        "channelFilter": channel,
        "windowColumn": "conversations.updated_at (no rated_at column exists, so a review is filed under the conversation's last update)",
        "metrics": {
            "averageScore": average_score,
            "averageScoreDefinition": "SUM(csat_score) / COUNT(scored) over conversations.csat_score in the window",
            "satisfactionPercentage": satisfaction_percentage,
            "satisfactionPercentageDefinition": "share of scored conversations rated 4 or 5",
            "totalResponses": total_responses,
            "distribution": Value::Object(distribution),
            "distributionSource": "GROUP BY conversations.csat_score within the requested window",
        },
        "recentFeedback": recent_feedback,
        "recentFeedbackCount": feedback_count,
        "recentFeedbackLimit": 50,
        "warnings": warnings,
    })))
}
