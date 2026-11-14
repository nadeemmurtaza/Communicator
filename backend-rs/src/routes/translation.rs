use crate::db::AppState;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Deserialize;
use serde_json::{json, Value};
use sqlx::{Row, PgPool};

/// The catalog the product actually claims support for. `curated` marks the
/// codes that have more behind them than a name: script-level detection, or an
/// exact-phrase table. Everything else is advertised as detect-only so the UI
/// cannot promise a translation engine that is not there.
const LANGUAGES: &[(&str, &str, &str, &str, bool)] = &[
    ("en", "English", "English", "ltr", true),
    ("es", "Spanish", "Español", "ltr", true),
    ("fr", "French", "Français", "ltr", true),
    ("de", "German", "Deutsch", "ltr", true),
    ("it", "Italian", "Italiano", "ltr", false),
    ("pt", "Portuguese", "Português", "ltr", false),
    ("nl", "Dutch", "Nederlands", "ltr", false),
    ("ru", "Russian", "Русский", "ltr", false),
    ("zh", "Chinese (Simplified)", "中文 (简体)", "ltr", false),
    ("ja", "Japanese", "日本語", "ltr", false),
    ("ko", "Korean", "한국어", "ltr", false),
    ("ar", "Arabic", "العربية", "rtl", false),
    ("hi", "Hindi", "हिन्दी", "ltr", false),
    ("tr", "Turkish", "Türkçe", "ltr", false),
    ("pl", "Polish", "Polski", "ltr", false),
    ("sv", "Swedish", "Svenska", "ltr", false),
    ("da", "Danish", "Dansk", "ltr", false),
    ("fi", "Finnish", "Suomi", "ltr", false),
    ("no", "Norwegian", "Norsk", "ltr", false),
    ("el", "Greek", "Ελληνικά", "ltr", false),
    ("he", "Hebrew", "עברית", "rtl", false),
    ("id", "Indonesian", "Bahasa Indonesia", "ltr", false),
    ("vi", "Vietnamese", "Tiếng Việt", "ltr", false),
    ("th", "Thai", "ไทย", "ltr", false),
    ("cs", "Czech", "Čeština", "ltr", false),
    ("ro", "Romanian", "Română", "ltr", false),
    ("hu", "Hungarian", "Magyar", "ltr", false),
    ("uk", "Ukrainian", "Українська", "ltr", false),
];

/// Exact phrases verified by hand. Only whole-normalised-string matches use
/// this table; anything else must go through the configured engine.
const PHRASES: &[(&str, &str, &str, &str)] = &[
    ("es", "en", "hola", "hello"),
    ("es", "en", "gracias", "thank you"),
    ("es", "en", "por favor", "please"),
    ("es", "en", "necesito ayuda", "I need help"),
    (
        "es",
        "en",
        "hola, necesito ayuda con mi factura",
        "Hello, I need help with my invoice",
    ),
    ("es", "en", "dónde está mi pedido?", "Where is my order?"),
    ("es", "en", "cuánto cuesta esto?", "How much does this cost?"),
    ("en", "es", "hello", "hola"),
    ("en", "es", "thank you", "gracias"),
    ("en", "es", "please", "por favor"),
    ("en", "es", "i need help", "necesito ayuda"),
    (
        "en",
        "es",
        "i have refunded your payment",
        "he reembolsado tu pago",
    ),
    ("en", "es", "your order is on the way", "tu pedido está en camino"),
    (
        "en",
        "es",
        "thank you for contacting support",
        "gracias por contactar al soporte",
    ),
    ("fr", "en", "bonjour", "hello"),
    ("fr", "en", "merci", "thank you"),
    ("fr", "en", "s'il vous plaît", "please"),
    (
        "fr",
        "en",
        "bonjour, où est ma commande ?",
        "Hello, where is my order?",
    ),
    ("fr", "en", "j'ai besoin d'aide", "I need help"),
    ("en", "fr", "hello", "bonjour"),
    ("en", "fr", "thank you", "merci"),
    ("en", "fr", "your order has shipped", "votre commande a été expédiée"),
    ("de", "en", "hallo", "hello"),
    ("de", "en", "danke", "thank you"),
    ("de", "en", "bitte", "please"),
    (
        "de",
        "en",
        "hallo, wie kann ich mein passwort ändern?",
        "Hello, how can I change my password?",
    ),
    ("de", "en", "ich brauche hilfe", "I need help"),
    ("en", "de", "hello", "hallo"),
    ("en", "de", "thank you", "danke"),
    ("en", "de", "your ticket has been resolved", "Ihr Ticket wurde gelöst"),
];

const LATIN_MARKERS: &[(&str, &[&str], f64)] = &[
    (
        "es",
        &[
            "hola", "gracias", "por favor", "necesito", "ayuda", "factura", "pedido", "dónde",
            "donde", "cómo", "estoy", "cuenta", "buenos días", "buenas tardes",
        ],
        0.72,
    ),
    (
        "fr",
        &[
            "bonjour", "merci", "s'il vous", "besoin", "aide", "commande", "comment", "suis",
            "pourquoi", "salut", "bonne journée",
        ],
        0.72,
    ),
    (
        "de",
        &[
            "hallo", "guten tag", "danke", "bitte", "hilfe", "passwort", "rechnung", "bestellung",
            "warum", "ändern", "nicht",
        ],
        0.72,
    ),
    (
        "it",
        &["ciao", "grazie", "per favore", "buongiorno", "aiuto", "ordine", "fattura", "vorrei"],
        0.70,
    ),
    (
        "pt",
        &["olá", "obrigado", "obrigada", "ajuda", "pedido", "conta", "você"],
        0.70,
    ),
];

fn language_name(code: &str) -> String {
    LANGUAGES
        .iter()
        .find(|(c, ..)| *c == code)
        .map(|(_, name, ..)| (*name).to_string())
        .unwrap_or_else(|| code.to_ascii_uppercase())
}

/// Script ranges are decisive; Latin vocabulary hits are a guess and are
/// reported as such, with the confidence the evidence actually earns.
fn detect_language(text: &str) -> Value {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return json!({
            "language": "unknown",
            "name": "Unknown",
            "confidence": 0.0,
            "script": "None",
            "method": "no_text",
        });
    }

    let script_hits: [(&str, &str, &str, std::ops::RangeInclusive<u32>); 8] = [
        ("ru", "Russian", "Cyrillic", 0x0400..=0x04FF),
        ("ar", "Arabic", "Arabic", 0x0600..=0x06FF),
        ("he", "Hebrew", "Hebrew", 0x0590..=0x05FF),
        ("hi", "Hindi", "Devanagari", 0x0900..=0x097F),
        ("ja", "Japanese", "Japanese", 0x3040..=0x30FF),
        ("ko", "Korean", "Hangul", 0xAC00..=0xD7AF),
        ("zh", "Chinese", "Han", 0x4E00..=0x9FFF),
        ("el", "Greek", "Greek", 0x0370..=0x03FF),
    ];

    let mut seen: Vec<(&str, &str, &str)> = Vec::new();
    for ch in trimmed.chars() {
        let cp = ch as u32;
        for (code, name, script, range) in &script_hits {
            if range.contains(&cp) && !seen.iter().any(|(c, ..)| c == code) {
                seen.push((*code, *name, *script));
            }
        }
    }
    if let Some((code, name, script)) = seen.first() {
        return json!({
            "language": code,
            "name": name,
            // More than one script present means the text is mixed; a single
            // decisive script still is not certainty about the language.
            "confidence": if seen.len() == 1 { 0.95 } else { 0.60 },
            "script": script,
            "method": "script_range",
            "scripts_seen": seen.iter().map(|(c, ..)| *c).collect::<Vec<_>>(),
        });
    }

    let lower = trimmed.to_lowercase();
    let mut best: Option<(&str, usize)> = None;
    for (code, markers, _) in LATIN_MARKERS {
        let hits = markers
            .iter()
            .filter(|&&word| {
                if word.contains(' ') {
                    lower.contains(word)
                } else {
                    lower
                        .split(|c: char| !c.is_alphanumeric())
                        .any(|token| token == word)
                }
            })
            .count();
        if hits > 0 && best.map(|(_, n)| hits > n).unwrap_or(true) {
            best = Some((code, hits));
        }
    }
    if let Some((code, hits)) = best {
        let confidence = LATIN_MARKERS
            .iter()
            .find(|(c, ..)| *c == code)
            .map(|(_, _, conf)| *conf)
            .unwrap_or(0.6);
        return json!({
            "language": code,
            "name": language_name(code),
            "confidence": confidence,
            "script": "Latin",
            "method": "marker_vocabulary",
            "marker_hits": hits,
        });
    }

    json!({
        "language": "en",
        "name": "English",
        // Nothing matched; this is the fallback assumption, not a detection.
        "confidence": 0.40,
        "script": "Latin",
        "method": "latin_fallback",
    })
}

fn lookup_phrase(text: &str, src: &str, target: &str) -> Option<String> {
    let norm = text.trim().to_lowercase();
    PHRASES
        .iter()
        .find(|(s, t, from, _)| *s == src && *t == target && *from == norm)
        .map(|(_, _, _, to)| {
            if text.trim().starts_with(char::is_uppercase) {
                let mut capped = to.to_string();
                if let Some(first) = capped.get_mut(..1) {
                    first.make_ascii_uppercase();
                }
                capped
            } else {
                (*to).to_string()
            }
        })
}

async fn engine_translate(
    state: &AppState,
    text: &str,
    src: &str,
    target: &str,
) -> Result<Value, (StatusCode, Value)> {
    if state.config.translation_api_url.is_empty() {
        return Err((
            StatusCode::SERVICE_UNAVAILABLE,
            json!({
                "translated": false,
                "detail": format!(
                    "No translation engine is configured for {} -> {}. Set TRANSLATION_API_URL (and TRANSLATION_API_KEY if the engine needs one).",
                    src, target
                ),
            }),
        ));
    }

    let mut request = state
        .http_client
        .post(&state.config.translation_api_url)
        .timeout(std::time::Duration::from_secs(20))
        .json(&json!({
            "text": text,
            "sourceLanguage": src,
            "targetLanguage": target,
        }));
    if !state.config.translation_api_key.is_empty() {
        request = request.bearer_auth(&state.config.translation_api_key);
    }

    let response = request.send().await.map_err(|err| {
        (
            StatusCode::BAD_GATEWAY,
            json!({
                "translated": false,
                "detail": format!("Translation engine request failed: {err}"),
            }),
        )
    })?;

    let status = response.status();
    let body: Value = response.json().await.unwrap_or_else(|_| json!(null));
    if !status.is_success() {
        let detail = body
            .get("message")
            .or_else(|| body.get("error"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| format!("HTTP {status}"));
        return Err((
            StatusCode::BAD_GATEWAY,
            json!({ "translated": false, "detail": format!("Translation engine error: {detail}") }),
        ));
    }

    let translated = ["translatedText", "translated_text", "translation", "text"]
        .iter()
        .find_map(|key| body.get(*key).and_then(|v| v.as_str()))
        .filter(|s| !s.trim().is_empty());
    match translated {
        Some(text) => Ok(json!({
            "translated_text": text,
            "confidence": body.get("confidence").and_then(|v| v.as_f64()),
            "engine_detected_source": body.get("detectedSourceLanguage").and_then(|v| v.as_str()),
        })),
        None => Err((
            StatusCode::BAD_GATEWAY,
            json!({
                "translated": false,
                "detail": "Translation engine replied without a translated text field.",
                "engine_response": body,
            }),
        )),
    }
}

#[derive(Deserialize)]
pub struct TranslatePayload {
    text: String,
    #[serde(alias = "targetLanguage")]
    target_language: String,
    #[serde(alias = "sourceLanguage")]
    source_language: Option<String>,
}

#[derive(Deserialize)]
pub struct DetectPayload {
    text: String,
}

#[derive(Deserialize)]
pub struct OutboundPayload {
    text: String,
    #[serde(alias = "targetLanguage")]
    target_language: Option<String>,
}

#[derive(Deserialize)]
pub struct SettingsUpdate {
    is_enabled: Option<bool>,
    auto_translate_inbound: Option<bool>,
    auto_translate_outbound: Option<bool>,
    agent_primary_language: Option<String>,
    preferred_engine: Option<String>,
    confidence_threshold: Option<f64>,
    enabled_languages: Option<Vec<String>>,
}

#[derive(Deserialize)]
pub struct TranslateQuery {
    #[serde(alias = "targetLanguage")]
    target_language: Option<String>,
}

const SETTINGS_ID: &str = "translation_settings_default";

async fn load_settings(pool: &PgPool) -> Result<Value, sqlx::Error> {
    let row = sqlx::query(
        "SELECT id, is_enabled, auto_translate_inbound, auto_translate_outbound,
                agent_primary_language, preferred_engine, confidence_threshold,
                enabled_languages, created_at, updated_at
         FROM translation_settings WHERE id = $1",
    )
    .bind(SETTINGS_ID)
    .fetch_optional(pool)
    .await?;

    if let Some(row) = row {
        let text_of = |column: &str| -> Option<String> {
            row.try_get::<Option<String>, _>(column)
                .ok()
                .flatten()
                .filter(|v| !v.is_empty())
        };
        let enabled: Vec<String> = text_of("enabled_languages")
            .and_then(|raw| serde_json::from_str(&raw).ok())
            .unwrap_or_default();
        return Ok(json!({
            "id": text_of("id"),
            "is_enabled": row.try_get::<i64, _>("is_enabled").map(|v| v != 0).unwrap_or(false),
            "auto_translate_inbound": row.try_get::<i64, _>("auto_translate_inbound").map(|v| v != 0).unwrap_or(false),
            "auto_translate_outbound": row.try_get::<i64, _>("auto_translate_outbound").map(|v| v != 0).unwrap_or(false),
            "agent_primary_language": text_of("agent_primary_language"),
            "preferred_engine": text_of("preferred_engine"),
            "confidence_threshold": row.try_get::<f64, _>("confidence_threshold").ok(),
            "enabled_languages": enabled,
            "created_at": text_of("created_at"),
            "updated_at": text_of("updated_at"),
            "configured_engine_url": !state_translation_url().is_empty(),
        }));
    }

    // Never invent an "enabled" default: with no row the feature is off and the
    // engine is whatever is actually configured, reported as such.
    Ok(json!({
        "id": Value::Null,
        "is_enabled": false,
        "auto_translate_inbound": false,
        "auto_translate_outbound": false,
        "agent_primary_language": Value::Null,
        "preferred_engine": if state_translation_url().is_empty() { "none_configured" } else { "http_engine" },
        "confidence_threshold": Value::Null,
        "enabled_languages": [],
        "created_at": Value::Null,
        "updated_at": Value::Null,
        "configured_engine_url": !state_translation_url().is_empty(),
    }))
}

fn state_translation_url() -> String {
    std::env::var("TRANSLATION_API_URL").unwrap_or_default()
}

fn engine_url_present() -> bool {
    std::env::var("TRANSLATION_API_URL")
        .map(|v| !v.trim().is_empty())
        .unwrap_or(false)
}

pub async fn list_supported_languages() -> Json<Value> {
    let list: Vec<Value> = LANGUAGES
        .iter()
        .map(|(code, name, native, dir, curated)| {
            json!({
                "code": code,
                "name": name,
                "native": native,
                "dir": dir,
                "offline_phrases": if *curated { "exact_match_only" } else { "requires_engine" },
            })
        })
        .collect();
    Json(json!({
        "languages": list,
        "total": list.len(),
        "engine_configured": engine_url_present(),
    }))
}

pub async fn get_translation_settings(State(state): State<AppState>) -> Response {
    match load_settings(&state.db).await {
        Ok(settings) => Json(json!({ "settings": settings })).into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "detail": format!("Could not read translation settings: {err}") })),
        )
            .into_response(),
    }
}

pub async fn update_translation_settings(
    State(state): State<AppState>,
    Json(payload): Json<SettingsUpdate>,
) -> Response {
    let current = match load_settings(&state.db).await {
        Ok(value) => value,
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "detail": format!("Could not read translation settings: {err}") })),
            )
                .into_response()
        }
    };

    let now = chrono::Utc::now().to_rfc3339();
    let is_enabled = payload
        .is_enabled
        .unwrap_or_else(|| current["is_enabled"].as_bool().unwrap_or(false));
    let inbound = payload
        .auto_translate_inbound
        .unwrap_or_else(|| current["auto_translate_inbound"].as_bool().unwrap_or(false));
    let outbound = payload
        .auto_translate_outbound
        .unwrap_or_else(|| current["auto_translate_outbound"].as_bool().unwrap_or(false));
    let primary = payload
        .agent_primary_language
        .clone()
        .or_else(|| current["agent_primary_language"].as_str().map(String::from))
        .unwrap_or_default();
    let engine = payload
        .preferred_engine
        .clone()
        .or_else(|| current["preferred_engine"].as_str().map(String::from))
        .unwrap_or_default();
    let threshold = payload
        .confidence_threshold
        .or_else(|| current["confidence_threshold"].as_f64())
        .unwrap_or(0.8);
    let languages: Vec<String> = match payload.enabled_languages.clone() {
        Some(explicit) => explicit,
        None => current["enabled_languages"]
            .as_array()
            .map(|rows| {
                rows.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default(),
    };
    let languages_json = serde_json::to_string(&languages).unwrap_or_else(|_| "[]".to_string());
    let created = current["created_at"]
        .as_str()
        .unwrap_or(now.as_str())
        .to_string();

    // Enabling automatic translation with no engine configured would silently
    // stop translating, so refuse instead of storing an unreachable intent.
    if (inbound || outbound || is_enabled) && state.config.translation_api_url.is_empty() {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({
                "detail": "Cannot enable automatic translation: no translation engine is configured. Set TRANSLATION_API_URL.",
                "engine_configured": false,
            })),
        )
            .into_response();
    }

    let result = sqlx::query(
        "INSERT INTO translation_settings (id, is_enabled, auto_translate_inbound,
            auto_translate_outbound, agent_primary_language, preferred_engine,
            confidence_threshold, enabled_languages, created_at, updated_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
         ON CONFLICT(id) DO UPDATE SET
            is_enabled = excluded.is_enabled,
            auto_translate_inbound = excluded.auto_translate_inbound,
            auto_translate_outbound = excluded.auto_translate_outbound,
            agent_primary_language = excluded.agent_primary_language,
            preferred_engine = excluded.preferred_engine,
            confidence_threshold = excluded.confidence_threshold,
            enabled_languages = excluded.enabled_languages,
            updated_at = excluded.updated_at",
    )
    .bind(SETTINGS_ID)
    .bind(is_enabled)
    .bind(inbound)
    .bind(outbound)
    .bind(&primary)
    .bind(&engine)
    .bind(threshold)
    .bind(&languages_json)
    .bind(&created)
    .bind(&now)
    .execute(&state.db)
    .await;

    match result {
        Ok(_) => match load_settings(&state.db).await {
            Ok(settings) => Json(json!({ "settings": settings })).into_response(),
            Err(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "detail": format!("Saved but could not re-read settings: {err}") })),
            )
                .into_response(),
        },
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "detail": format!("Could not save translation settings: {err}") })),
        )
            .into_response(),
    }
}

pub async fn detect_language_endpoint(Json(payload): Json<DetectPayload>) -> Json<Value> {
    Json(detect_language(&payload.text))
}

pub async fn translate_text(
    State(state): State<AppState>,
    Json(payload): Json<TranslatePayload>,
) -> Response {
    let target = payload.target_language.trim().to_lowercase();
    if target.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "detail": "target_language is required." })),
        )
            .into_response();
    }
    if !LANGUAGES.iter().any(|(code, ..)| *code == target) {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "detail": format!("Language '{target}' is not in the supported catalog."),
                "supported": LANGUAGES.iter().map(|(c, ..)| *c).collect::<Vec<_>>(),
            })),
        )
            .into_response();
    }

    let source = match payload
        .source_language
        .as_deref()
        .map(|s| s.trim().to_lowercase())
        .filter(|s| !s.is_empty() && s != "auto")
    {
        Some(explicit) => (explicit, "client_supplied".to_string()),
        None => {
            let detected = detect_language(&payload.text);
            (
                detected["language"].as_str().unwrap_or("en").to_string(),
                detected["method"].as_str().unwrap_or("heuristic").to_string(),
            )
        }
    };

    if payload.text.trim().is_empty() {
        return Json(json!({
            "original_text": payload.text,
            "translated_text": payload.text,
            "source_language": source.0,
            "target_language": target,
            "provider": "noop",
            "confidence": 1.0,
            "note": "Nothing to translate.",
        }))
        .into_response();
    }

    if source.0 == target {
        return Json(json!({
            "original_text": payload.text,
            "translated_text": payload.text,
            "source_language": source.0,
            "target_language": target,
            "provider": "noop",
            "confidence": 1.0,
            "detection_method": source.1,
        }))
        .into_response();
    }

    if let Some(matched) = lookup_phrase(&payload.text, &source.0, &target) {
        return Json(json!({
            "original_text": payload.text,
            "translated_text": matched,
            "source_language": source.0,
            "target_language": target,
            "provider": "curated_phrase",
            "confidence": 1.0,
            "detection_method": source.1,
        }))
        .into_response();
    }

    match engine_translate(&state, &payload.text, &source.0, &target).await {
        Ok(result) => Json(json!({
            "original_text": payload.text,
            "translated_text": result["translated_text"],
            "source_language": source.0,
            "target_language": target,
            "provider": "http_engine",
            "confidence": result["confidence"],
            "detection_method": source.1,
        }))
        .into_response(),
        Err((status, detail)) => (status, Json(detail)).into_response(),
    }
}

pub async fn translate_message(
    State(state): State<AppState>,
    Path(message_id): Path<String>,
    Query(params): Query<TranslateQuery>,
) -> Response {
    let target = params
        .target_language
        .unwrap_or_else(|| "en".to_string())
        .trim()
        .to_lowercase();

    let row = match sqlx::query("SELECT body, channel, provider_metadata FROM messages WHERE id = $1")
        .bind(&message_id)
        .fetch_optional(&state.db)
        .await
    {
        Ok(Some(row)) => row,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({ "detail": format!("Message {message_id} not found.") })),
            )
                .into_response()
        }
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "detail": format!("Could not read message: {err}") })),
            )
                .into_response()
        }
    };

    let body = row.try_get::<String, _>("body").unwrap_or_default();
    let source = detect_language(&body)["language"]
        .as_str()
        .unwrap_or("en")
        .to_string();

    let translation = if source == target {
        json!({ "translated_text": body, "provider": "noop", "confidence": 1.0 })
    } else {
        match lookup_phrase(&body, &source, &target) {
            Some(matched) => json!({
                "translated_text": matched, "provider": "curated_phrase", "confidence": 1.0
            }),
            None => match engine_translate(&state, &body, &source, &target).await {
                Ok(result) => json!({
                    "translated_text": result["translated_text"],
                    "provider": "http_engine",
                    "confidence": result["confidence"],
                }),
                Err((status, detail)) => return (status, Json(detail)).into_response(),
            },
        }
    };

    // Persist onto the row so the work is not repeated, keyed by target code.
    let mut existing: Value = row
        .try_get::<Option<String>, _>("provider_metadata")
        .ok()
        .flatten()
        .and_then(|raw| serde_json::from_str(&raw).ok())
        .unwrap_or_else(|| json!({}));
    if !existing.is_object() {
        existing = json!({});
    }
    existing["translations"] = {
        let mut by_language = serde_json::Map::new();
        by_language.insert(target.clone(), translation.clone());
        Value::Object(by_language)
    };
    let serialized = serde_json::to_string(&existing).unwrap_or_else(|_| "{}".to_string());
    if let Err(err) = sqlx::query("UPDATE messages SET provider_metadata = $1 WHERE id = $2")
        .bind(&serialized)
        .bind(&message_id)
        .execute(&state.db)
        .await
    {
        tracing::error!("Translated text obtained but could not be stored for {message_id}: {err}");
    }

    Json(json!({
        "success": true,
        "message_id": message_id,
        "translation": {
            "original_text": body,
            "source_language": source,
            "target_language": target,
            "translated_text": translation["translated_text"],
            "provider": translation["provider"],
            "confidence": translation["confidence"],
        },
    }))
    .into_response()
}

pub async fn prepare_outbound(
    State(state): State<AppState>,
    Path(conversation_id): Path<String>,
    Json(payload): Json<OutboundPayload>,
) -> Response {
    let row = match sqlx::query(
        "SELECT c.contact_id, ct.language, ct.person_name
         FROM conversations c
         LEFT JOIN contacts ct ON ct.id = c.contact_id
         WHERE c.id = $1",
    )
    .bind(&conversation_id)
    .fetch_optional(&state.db)
    .await
    {
        Ok(Some(row)) => row,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({ "detail": format!("Conversation {conversation_id} not found.") })),
            )
                .into_response()
        }
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "detail": format!("Could not read conversation: {err}") })),
            )
                .into_response()
        }
    };

    // The customer's language is either what we stored on the contact or what
    // their most recent inbound message looks like — never a guessed default.
    let stored = row
        .try_get::<Option<String>, _>("language")
        .ok()
        .flatten()
        .filter(|v| !v.trim().is_empty() && *v != "unknown");
    let customer_language = match stored {
        Some(known) => (known, "contact_record".to_string()),
        None => {
            let last = sqlx::query(
                "SELECT body FROM messages WHERE conversation_id = $1 AND direction = 'inbound'
                 ORDER BY sent_at DESC LIMIT 1",
            )
            .bind(&conversation_id)
            .fetch_optional(&state.db)
            .await
            .ok()
            .flatten();
            match last {
                Some(message) => {
                    let body = message.try_get::<String, _>("body").unwrap_or_default();
                    let detected = detect_language(&body);
                    (
                        detected["language"].as_str().unwrap_or("en").to_string(),
                        format!("detected_{}", detected["method"].as_str().unwrap_or("heuristic")),
                    )
                }
                None => {
                    return (
                        StatusCode::SERVICE_UNAVAILABLE,
                        Json(json!({
                            "detail": "No inbound customer message in this conversation, so the customer's language cannot be established. Provide target_language explicitly.",
                            "conversation_id": conversation_id,
                        })),
                    )
                        .into_response()
                }
            }
        }
    };

    let target = payload
        .target_language
        .as_deref()
        .map(|v| v.trim().to_lowercase())
        .filter(|v| !v.is_empty())
        .unwrap_or(customer_language.0.clone());
    let agent_primary = match load_settings(&state.db).await {
        Ok(settings) => settings,
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "detail": format!("Could not read translation settings: {err}") })),
            )
                .into_response()
        }
    };
    // The agent's primary language is a stored workspace setting. Without one
    // the reply cannot be retranslated, so say so instead of assuming English.
    let source_language = match agent_primary["agent_primary_language"]
        .as_str()
        .map(|v| v.trim().to_lowercase())
        .filter(|v| !v.is_empty())
    {
        Some(declared) => declared,
        None => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(json!({
                    "detail": "No agent primary language is stored for this workspace. Set it via PUT /translation/settings before outbound translation.",
                    "conversation_id": conversation_id,
                    "customer_language": customer_language.0,
                })),
            )
                .into_response()
        }
    };

    if source_language == target {
        return Json(json!({
            "conversation_id": conversation_id,
            "customer_language": customer_language.0,
            "language_basis": customer_language.1,
            "original_text": payload.text,
            "translated_text": payload.text,
            "source_language": source_language,
            "target_language": target,
            "provider": "noop",
        }))
        .into_response();
    }

    let outcome = match lookup_phrase(&payload.text, &source_language, &target) {
        Some(matched) => json!({
            "translated_text": matched, "provider": "curated_phrase", "confidence": 1.0
        }),
        None => match engine_translate(&state, &payload.text, &source_language, &target).await {
            Ok(result) => json!({
                "translated_text": result["translated_text"],
                "provider": "http_engine",
                "confidence": result["confidence"],
            }),
            Err((status, detail)) => return (status, Json(detail)).into_response(),
        },
    };

    Json(json!({
        "conversation_id": conversation_id,
        "customer_language": customer_language.0,
        "language_basis": customer_language.1,
        "original_text": payload.text,
        "translated_text": outcome["translated_text"],
        "source_language": source_language,
        "target_language": target,
        "provider": outcome["provider"],
        "confidence": outcome["confidence"],
    }))
    .into_response()
}
