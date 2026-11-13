#![allow(dead_code)]

//! Outbound channel adapters: one per provider the product connects to.
//!
//! Port of `src/communicator/channels/*.py`. The Python adapters were written
//! around a "simulated dispatch" idea: a missing or test-shaped credential, or
//! even a live HTTP failure, produced a fabricated message id with
//! `simulated: true` and `status: "queued"`, which the caller then stored as
//! SENT. This module inverts that:
//!
//! * every adapter makes the real request with the real credential;
//! * a credential that is absent, masked (`••••1234`) or recognibly fake
//!   (`mock_…`, `test_…`, `AC_…`) yields `SendStatus::NotSent` naming the exact
//!   field and every place it was looked for;
//! * a provider that answers with an error yields `SendStatus::Rejected` with
//!   that provider's own error text and HTTP status;
//! * nothing in this file invents a message id.
//!
//! Credentials resolve in the order the rest of the backend uses: the inbox the
//! tenant connected, then the gateway connection, then `integration_settings`,
//! then the vault-applied `Config`, then the process environment.

use crate::db::AppState;
use crate::services::{
    integration_settings,
    messaging::{SendResult, SendStatus},
};
use serde_json::{json, Value};
use sqlx::Row;
use std::collections::HashMap;
use std::env;
use std::future::Future;
use std::pin::Pin;
use std::time::Duration;

/// boxed so one `&'static dyn ChannelAdapter` table can hold every provider
pub type SendFuture<'a> = Pin<Box<dyn Future<Output = SendResult> + Send + 'a>>;

#[derive(Debug, Clone)]
pub struct CredValue {
    pub value: String,
    /// Where it came from, so the refusal can tell the operator what to fix.
    pub source: String,
}

#[derive(Debug, Clone, Default)]
pub struct Credentials {
    pub channel: String,
    pub values: HashMap<String, CredValue>,
    /// Every place consulted, in order, for audit output.
    pub searched: Vec<String>,
}

impl Credentials {
    pub fn get(&self, key: &str) -> Option<&CredValue> {
        self.values.get(key)
    }

    pub fn str_or_default(&self, key: &str) -> String {
        self.values.get(key).map(|v| v.value.clone()).unwrap_or_default()
    }

    pub fn has(&self, key: &str) -> bool {
        self.values.get(key).is_some_and(|v| !v.value.is_empty())
    }

    /// True when a key holds a non-empty value -- including one the resolver
    /// rejected as a mask or fixture, which is why `has` is not enough for an
    /// SMTP-vs-Postmark style branch.
    pub fn present(&self, key: &str) -> bool {
        self.values.get(key).is_some_and(|v| !v.value.trim().is_empty())
    }

    /// The first of `keys` that resolves to a non-empty value.
    pub fn first_any(&self, keys: &[&str]) -> Option<&CredValue> {
        keys.iter().filter_map(|key| self.get(key)).find(|cred| !cred.value.is_empty())
    }

    pub fn str_any_or_empty(&self, keys: &[&str]) -> String {
        self.first_any(keys).map(|cred| cred.value.clone()).unwrap_or_default()
    }

    /// The credential, or a refusal that names every field that would have
    /// satisfied it, the inbox row and the env vars that were searched.
    pub fn require(&self, key: &str) -> Result<String, String> {
        self.require_any(std::slice::from_ref(&key))
    }

    /// `require` across the alias spellings the routes accept for the same
    /// logical field, so an inbox that IS configured is never reported as
    /// "not configured" because it stored `botToken` instead of `accessToken`.
    pub fn require_any(&self, keys: &[&str]) -> Result<String, String> {
        if let Some(cred) = self.first_any(keys) {
            return Ok(cred.value.clone());
        }
        let empties: Vec<String> = keys
            .iter()
            .filter_map(|key| {
                self.get(key).filter(|cred| cred.value.trim().is_empty()).map(|cred| {
                    format!("'{key}' is present but empty (source: {})", cred.source)
                })
            })
            .collect();
        let names = keys
            .iter()
            .map(|key| format!("'{key}'"))
            .collect::<Vec<_>>()
            .join(" or ");
        let mut detail = format!(
            "Credential {names} is not configured for channel '{}'. Looked in: {}.",
            self.channel,
            if self.searched.is_empty() {
                format!("no active inbox row for channel '{}'", self.channel)
            } else {
                self.searched.join("; ")
            }
        );
        if !empties.is_empty() {
            detail.push_str(&format!(" {}", empties.join("; ")));
        }
        Err(format!("{detail} Nothing was sent."))
    }
}

/// The alias spellings each logical credential field is stored under, taken
/// from the key lists `routes/inboxes.rs` reads and writes on the same inbox
/// rows. `absorb` already normalises camelCase to snake_case, so only genuinely
/// different field names belong here -- never a fabricated default.
pub fn credential_aliases(key: &str) -> Option<&'static [&'static str]> {
    match key {
        "access_token" => Some(&["access_token", "app_access_token"]),
        "bot_token" => Some(&["bot_token", "telegram_bot_token", "access_token"]),
        "discord_bot_token" => Some(&["discord_bot_token", "bot_token", "access_token"]),
        "discord_client_id" => Some(&["discord_client_id", "client_id", "application_id"]),
        "page_access_token" => Some(&["page_access_token", "access_token"]),
        "ig_id" => Some(&["ig_id", "instagram_user_id", "user_id"]),
        "channel_access_token" => Some(&["channel_access_token"]),
        "api_key" => Some(&["api_key", "postmark_server_token", "postmark_api_key", "server_token"]),
        "from_address" => Some(&["from_address", "email_from_address"]),
        "from_name" => Some(&["from_name", "email_from_name"]),
        "reply_to" => Some(&["reply_to", "email_reply_to"]),
        "from_number" => Some(&["from_number", "phone_number", "twilio_from"]),
        "caller_id" => Some(&["caller_id", "voice_number", "from_number", "phone_number"]),
        "messaging_service_sid" => Some(&["messaging_service_sid"]),
        "business_account_id" => Some(&["business_account_id", "business_id", "advertiser_id"]),
        "website_token" => Some(&["website_token"]),
        "widget_hmac_secret" => Some(&["widget_hmac_secret"]),
        _ => None,
    }
}

/// Every spelling a logical credential field may be stored under, always
/// including the field's own name, so an unlisted key still resolves instead of
/// silently becoming an empty search.
pub fn credential_keys(key: &str) -> Vec<&str> {
    match credential_aliases(key) {
        Some(aliases) => aliases.to_vec(),
        None => vec![key],
    }
}

/// Values that look like credentials but are masks or fixtures. The Python
/// adapters used these same shapes to *fake a success*; here they only ever
/// produce a refusal, which is the honest outcome for a test fixture.
fn describe_fake_shape(value: &str) -> &'static str {
    let lower = value.to_ascii_lowercase();
    if value.contains('•') || value.contains("****") {
        "a masked display value"
    } else if lower.starts_with("mock") || lower.contains("_mock") || lower.contains("mock_") {
        "a mock fixture"
    } else if lower.starts_with("test") || lower.contains("_test") || lower.contains("test_") {
        "a test fixture"
    } else if lower.contains("example") || lower.contains("placeholder") || lower.contains("changeme")
    {
        "a placeholder"
    } else if lower.starts_with("your_") || lower.starts_with("your-") {
        "an untouched template value"
    } else if value.starts_with("AC_") || lower.starts_with("sk-ant-api03-default") {
        "a non-real account/token id"
    } else {
        "an unusable value"
    }
}

fn is_usable_secret(value: &str) -> bool {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return false;
    }
    !(trimmed.contains('•')
        || trimmed.contains("****")
        || describe_fake_shape(trimmed) != "an unusable value")
}

fn camel_to_snake(input: &str) -> String {
    let mut out = String::with_capacity(input.len() + 4);
    for ch in input.chars() {
        if ch.is_ascii_uppercase() && !out.is_empty() && !out.ends_with('_') {
            out.push('_');
        }
        out.push(ch.to_ascii_lowercase());
    }
    out
}

/// Channel-qualified environment variable names, newest contract first. These
/// are the names documented for the deployment; nothing here is invented at
/// runtime.
///
/// Every arm is scoped to the channels that can actually use the variable: a
/// Meta WhatsApp token adopted as a Telegram bot token or an SMS number adopted
/// as a voice caller id would produce a provider error that looks like a bad
/// credential while the real problem is a missing one, so unmapped channels get
/// an empty list and the adapter refuses with the field name instead.
fn env_names(channel: &str, key: &str) -> Vec<&'static str> {
    let channel = canonical_channel(channel);
    let channel = channel.as_str();
    match key {
        "access_token" => match channel {
            "instagram" => vec!["INSTAGRAM_ACCESS_TOKEN", "META_INSTAGRAM_ACCESS_TOKEN"],
            "tiktok" => vec!["TIKTOK_ACCESS_TOKEN"],
            "whatsapp" => vec!["META_WHATSAPP_ACCESS_TOKEN"],
            _ => vec![],
        },
        "phone_number_id" if channel == "whatsapp" => vec!["META_WHATSAPP_PHONE_NUMBER_ID"],
        "api_version" if matches!(channel, "whatsapp" | "facebook" | "instagram") => {
            vec!["META_WHATSAPP_API_VERSION", "META_GRAPH_API_VERSION"]
        }
        "app_secret" => match channel {
            "tiktok" => vec!["TIKTOK_APP_SECRET"],
            "instagram" => vec!["META_INSTAGRAM_APP_SECRET"],
            "whatsapp" | "facebook" => vec!["META_APP_SECRET"],
            _ => vec![],
        },
        "webhook_verify_token" => match channel {
            "tiktok" => vec!["TIKTOK_WEBHOOK_VERIFY_TOKEN"],
            "whatsapp" | "facebook" | "instagram" => vec!["META_WEBHOOK_VERIFY_TOKEN"],
            _ => vec![],
        },
        "page_access_token" if matches!(channel, "facebook" | "instagram") => {
            vec!["META_PAGE_ACCESS_TOKEN", "FACEBOOK_PAGE_ACCESS_TOKEN"]
        }
        "bot_token" if channel == "telegram" => vec!["TELEGRAM_BOT_TOKEN", "TELEGRAM_BOT_API_TOKEN"],
        "bot_token" if channel == "discord" => vec!["DISCORD_BOT_TOKEN"],
        "discord_client_id" if channel == "discord" => vec!["DISCORD_CLIENT_ID", "DISCORD_APPLICATION_ID"],
        "channel_access_token" | "channel_secret" if channel == "line" => {
            if key == "channel_access_token" {
                vec!["LINE_CHANNEL_ACCESS_TOKEN"]
            } else {
                vec!["LINE_CHANNEL_SECRET"]
            }
        }
        "account_sid" | "auth_token" | "messaging_service_sid"
            if matches!(channel, "sms" | "whatsapp" | "whatsapp_twilio" | "voice") =>
        {
            match key {
                "account_sid" => vec!["TWILIO_ACCOUNT_SID"],
                "auth_token" => vec!["TWILIO_AUTH_TOKEN"],
                _ => vec!["TWILIO_MESSAGING_SERVICE_SID"],
            }
        }
        "from_number" | "caller_id"
            if matches!(channel, "sms" | "whatsapp" | "whatsapp_twilio" | "voice") =>
        {
            match (channel, key) {
                ("whatsapp" | "whatsapp_twilio", _) => vec!["TWILIO_WHATSAPP_FROM", "TWILIO_SMS_FROM"],
                ("voice", _) => vec!["TWILIO_CALLER_ID", "TWILIO_VOICE_FROM"],
                (_, "caller_id") => vec!["TWILIO_CALLER_ID", "TWILIO_VOICE_FROM"],
                _ => vec!["TWILIO_SMS_FROM"],
            }
        }
        "api_key" if channel == "email" => vec!["POSTMARK_SERVER_TOKEN", "EMAIL_PROVIDER_API_KEY"],
        "from_address" | "reply_to" if channel == "email" => {
            if key == "from_address" {
                vec!["EMAIL_FROM_ADDRESS"]
            } else {
                vec!["EMAIL_REPLY_TO"]
            }
        }
        "smtp_host" | "smtp_password" | "smtp_username" if channel == "email" => match key {
            "smtp_host" => vec!["SMTP_HOST"],
            "smtp_password" => vec!["SMTP_PASSWORD"],
            _ => vec!["SMTP_USERNAME"],
        },
        "business_account_id" if channel == "tiktok" => vec!["TIKTOK_BUSINESS_ACCOUNT_ID"],
        "website_token" if channel == "webchat" => vec!["WEBCHAT_WEBSITE_TOKEN"],
        "widget_hmac_secret" if channel == "webchat" => vec!["WEBCHAT_HMAC_SECRET", "CHATWOOT_HMAC_SECRET"],
        _ => vec![],
    }
}

/// Only the credential families the vault loader can populate into `Config`.
/// Reading `Config` rather than `env` means a vault secret wins over a stale
/// process environment -- `main.rs` applies `vault_loader::apply_vault_secrets`
/// onto `Config` at startup, so `Config` is the authoritative source and the
/// environment is only ever the fallback. `access_token` is deliberately scoped
/// per channel: a Meta number token is not an Instagram, Messenger or TikTok
/// token, and vice versa.
fn config_value(state: &AppState, channel: &str, key: &str) -> Option<String> {
    let channel = canonical_channel(channel);
    let channel = channel.as_str();
    let value = match key {
        "access_token" if channel == "whatsapp" => state.config.meta_whatsapp_access_token.clone(),
        "access_token" if channel == "tiktok" => state.config.tiktok_access_token.clone(),
        "phone_number_id" if channel == "whatsapp" => state.config.meta_whatsapp_phone_number_id.clone(),
        "api_version" if channel == "whatsapp" => state.config.meta_whatsapp_api_version.clone(),
        "app_secret" if channel == "whatsapp" || channel == "facebook" => {
            state.config.meta_app_secret.clone()
        }
        "app_secret" if channel == "instagram" => state.config.meta_instagram_app_secret.clone(),
        "webhook_verify_token" if channel == "whatsapp" => state.config.meta_webhook_verify_token.clone(),
        "account_sid" if is_twilio(channel) => state.config.twilio_account_sid.clone(),
        "auth_token" if is_twilio(channel) => state.config.twilio_auth_token.clone(),
        // An SMS number is not a voice caller id, so `from_number` only comes
        // from Config for the text-bearing Twilio channels.
        "from_number" if matches!(channel, "sms" | "whatsapp_twilio") => state.config.twilio_sms_from.clone(),
        "messaging_service_sid" if is_twilio(channel) => state.config.twilio_messaging_service_sid.clone(),
        _ => return None,
    };
    // A masked Config value (the UI writing "••••1234" back) is treated as
    // absent, exactly as `routes/inboxes.rs::config_val` does.
    is_usable_secret(&value).then_some(value)
}

fn is_twilio(channel: &str) -> bool {
    matches!(channel, "sms" | "whatsapp" | "whatsapp_twilio" | "voice")
}

fn env_value(channel: &str, key: &str) -> Option<(String, String)> {
    for name in env_names(channel, key) {
        if let Ok(raw) = env::var(name) {
            if !raw.trim().is_empty() {
                return Some((raw, format!("env {name}")));
            }
        }
    }
    None
}

/// Copies one credential blob (a `credentials` JSON column) into the map,
/// recording where each value came from and which values were rejected.
fn absorb(
    map: &mut HashMap<String, CredValue>,
    searched: &mut Vec<String>,
    raw: &Value,
    label: &str,
    table: &str,
) {
    let Some(object) = raw.as_object() else { return };
    searched.push(format!("{label} [{table}]"));
    for (key, value) in object {
        let text = match value {
            Value::String(text) => text.clone(),
            Value::Number(num) => num.to_string(),
            Value::Bool(flag) => flag.to_string(),
            _ => continue,
        };
        let normalized = camel_to_snake(key);
        for candidate in [normalized.clone(), key.clone()] {
            if map.contains_key(&candidate) {
                continue;
            }
            if !is_usable_secret(&text) {
                // Record the rejection so the refusal names the exact row.
                searched.push(format!(
                    "{key} from {label} [{table}] rejected as {}",
                    describe_fake_shape(&text)
                ));
                continue;
            }
            map.insert(
                candidate,
                CredValue { value: text.clone(), source: format!("{label} ({table})") },
            );
        }
    }
}

/// A text column read only to label the audit trail. `None` means the column is
/// missing, NULL or undecodable, and the refusal says "(unreadable)" rather
/// than showing an empty name as if the row really had one.
fn read_column(row: &sqlx::postgres::PgRow, column: &str) -> Option<String> {
    match row.try_get::<Option<String>, _>(column) {
        Ok(value) => value,
        Err(err) => {
            tracing::debug!("credential row column '{column}' unreadable: {err}");
            None
        }
    }
}

fn quote_or_unknown(value: Option<String>) -> String {
    match value.filter(|text| !text.trim().is_empty()) {
        Some(text) => format!("'{text}'"),
        None => "(unreadable)".to_string(),
    }
}

/// The `inboxes.channel_type` / `channel_gateway_connections.gateway_type`
/// strings that mean this transport. Taken from the values the routes and the
/// SQLite data actually use (`whatsapp`, `facebook`, `instagram`, `telegram`,
/// `line`, `sms`, `email`, `voice`, `tiktok`, `website`/`live-chat`), plus the
/// historical aliases, so an older row is still found.
fn inbox_channel_types(channel: &str) -> Vec<&'static str> {
    match canonical_channel(channel).as_str() {
        "whatsapp" => vec!["whatsapp", "whatsapp_cloud", "whatsapp_meta"],
        "whatsapp_twilio" => vec!["whatsapp_twilio", "twilio_whatsapp"],
        "facebook" => vec!["facebook", "messenger"],
        "instagram" => vec!["instagram", "ig", "instagram_dm"],
        "telegram" => vec!["telegram"],
        "discord" => vec!["discord"],
        "line" => vec!["line"],
        "sms" => vec!["sms"],
        "email" => vec!["email"],
        "tiktok" => vec!["tiktok", "tiktok_dm"],
        "voice" => vec!["voice", "voice_sip", "telephony"],
        "webchat" => vec!["website", "live-chat", "live_chat", "webchat"],
        _ => vec![],
    }
}

/// Reads the real credential set for one outbound channel.
pub async fn resolve_credentials(
    state: &AppState,
    channel: &str,
    inbox_id: Option<&str>,
) -> Credentials {
    let mut map: HashMap<String, CredValue> = HashMap::new();
    let mut searched: Vec<String> = Vec::new();

    // 1. The inbox the tenant connected (by id when the conversation carries
    //    one, else the active inbox for that channel type).
    // A DB error is recorded as an error, never flattened into "no inbox row":
    // the refusal has to tell the operator the read failed.
    let inbox_row = if let Some(id) = inbox_id.filter(|id| !id.is_empty()) {
        sqlx::query("SELECT id, name, credentials FROM inboxes WHERE id = ? LIMIT 1")
            .bind(id)
            .fetch_optional(&state.db)
            .await
            .map_err(|err| format!("inboxes read for id '{id}' failed: {err}"))
    } else {
        let types = inbox_channel_types(channel);
        if types.is_empty() {
            searched.push(format!("channel '{channel}' has no inbox channel_type mapping"));
            Err(format!("no inbox channel_type mapping for '{channel}'; the inboxes table was not queried"))
        } else {
            let placeholders = types.iter().map(|_| "?").collect::<Vec<_>>().join(",");
            let sql = format!(
                "SELECT id, name, credentials FROM inboxes WHERE channel_type IN ({placeholders}) \
                 AND is_active = 1 ORDER BY updated_at DESC LIMIT 1"
            );
            let mut query = sqlx::query(&sql);
            for value in &types {
                query = query.bind(*value);
            }
            query
                .fetch_optional(&state.db)
                .await
                .map_err(|err| format!("inboxes read for channel '{channel}' failed: {err}"))
        }
    };

    match inbox_row {
        Ok(Some(row)) => {
            let name = read_column(&row, "name");
            let id = read_column(&row, "id");
            let label = format!("inbox {} ({})", quote_or_unknown(name), quote_or_unknown(id));
            let raw = row
                .try_get::<Option<String>, _>("credentials")
                .map_err(|err| err.to_string())
                .ok()
                .flatten();
            match raw.and_then(|text| serde_json::from_str::<Value>(&text).ok()) {
                Some(text) => absorb(&mut map, &mut searched, &text, &label, "inboxes.credentials"),
                None => searched.push(format!("{label} has no JSON credentials blob (inboxes.credentials)")),
            }
        }
        Ok(None) => searched.push(format!(
            "no active inbox row for channel '{channel}'{} [inboxes]",
            inbox_id
                .filter(|id| !id.is_empty())
                .map(|id| format!(" with id '{id}'"))
                .unwrap_or_default()
        )),
        Err(err) => searched.push(err),
    }

    // 2. A gateway connection (Twilio/Postmark/SMTP numbers bought separately).
    let gateway_types = inbox_channel_types(channel);
    let mut gateway_names: Vec<&str> = gateway_types.clone();
    gateway_names.push(channel);
    gateway_names.retain(|value| !value.is_empty());
    let gateway_row = if gateway_names.is_empty() {
        Ok(None)
    } else {
        let gateway_placeholders = gateway_names.iter().map(|_| "?").collect::<Vec<_>>().join(",");
        let gateway_sql = format!(
            "SELECT provider, name, credentials FROM channel_gateway_connections \
             WHERE gateway_type IN ({gateway_placeholders}) AND is_active = 1 \
             ORDER BY is_default DESC, updated_at DESC LIMIT 1"
        );
        let mut gateway_query = sqlx::query(&gateway_sql);
        for value in &gateway_names {
            gateway_query = gateway_query.bind(*value);
        }
        gateway_query
            .fetch_optional(&state.db)
            .await
            .map_err(|err| format!("channel_gateway_connections read failed: {err}"))
    };
    if let Some(row) = gateway_row.as_ref().ok().and_then(|row| row.as_ref()) {
        let name = read_column(row, "name");
        let provider = read_column(row, "provider");
        let label = format!("gateway {}", quote_or_unknown(name));
        if let Some(provider) = provider.as_deref() {
            if is_usable_secret(provider) && !map.contains_key("provider") {
                map.insert(
                    "provider".to_string(),
                    CredValue {
                        value: provider.to_string(),
                        source: "channel_gateway_connections.provider".to_string(),
                    },
                );
            }
        }
        let raw: Option<String> = row.try_get("credentials").ok().flatten();
        match raw.and_then(|text| serde_json::from_str::<Value>(&text).ok()) {
            Some(text) => absorb(
                &mut map,
                &mut searched,
                &text,
                &label,
                "channel_gateway_connections.credentials",
            ),
            None => searched.push(format!("{label} has no JSON credentials blob")),
        }
    } else if let Err(err) = gateway_row {
        searched.push(err);
    }

    // 3. integration_settings: the Connections page's Meta credentials.
    let settings = match integration_settings::load(&state.db, state.config.default_tenant_key.as_str()).await
    {
        Ok(Some(row)) => Some(row),
        // A deployment that never sets DEFAULT_TENANT_KEY still has one
        // Connections row; read it rather than losing the credentials.
        Ok(None) => integration_settings::load_any(&state.db).await.ok().flatten(),
        Err(err) => {
            searched.push(format!("integration_settings could not be read: {err}"));
            None
        }
    };

    if let Some(settings) = settings {
        searched.push("integration_settings row [integration_settings]".to_string());
        let pairs = [
            ("meta_access_token", settings.meta_access_token.clone()),
            ("meta_phone_number_id", settings.meta_phone_number_id.clone()),
            ("meta_app_secret", settings.meta_app_secret.clone()),
            ("meta_webhook_verify_token", settings.meta_webhook_verify_token.clone()),
            ("whatsapp_provider", settings.whatsapp_provider.clone()),
        ];
        for (key, value) in pairs {
            if !map.contains_key(key) && is_usable_secret(&value) {
                map.insert(
                    key.to_string(),
                    CredValue { value, source: "integration_settings".to_string() },
                );
            }
        }
        // The Meta WhatsApp token doubles as the Cloud API credential only for
        // the whatsapp channel; Messenger/Instagram need their own page or
        // business-login token, so it is never substituted here.
        if channel == "whatsapp" {
            for key in ["access_token", "phone_number_id"] {
                let source_key = format!("meta_{key}");
                if let Some(cred) = map.get(&source_key) {
                    if !map.contains_key(key) {
                        map.insert(
                            key.to_string(),
                            CredValue { value: cred.value.clone(), source: "integration_settings".to_string() },
                        );
                    }
                }
            }
        }
    } else {
        searched.push("no integration_settings row (Connections page never saved)".to_string());
    }

    // 4. Vault-applied Config, then the process environment, for every field
    //    the adapters can ask for.
    for key in ENV_KEY_CANDIDATES {
        if map.contains_key(*key) {
            continue;
        }
        if let Some(value) = config_value(state, channel, key) {
            if is_usable_secret(&value) {
                map.insert(
                    key.to_string(),
                    CredValue { value, source: "Config (vault secret or boot env)".to_string() },
                );
                continue;
            }
        }
        if let Some((value, source)) = env_value(channel, key) {
            if is_usable_secret(&value) {
                map.insert(key.to_string(), CredValue { value, source });
            }
        }
    }

    Credentials { channel: channel.to_string(), values: map, searched }
}

/// Every credential field a live adapter may need, so each one gets the
/// environment fallback rather than only the Meta/Twilio subset.
const ENV_KEY_CANDIDATES: &[&str] = &[
    "access_token",
    "phone_number_id",
    "api_version",
    "app_secret",
    "webhook_verify_token",
    "page_access_token",
    "ig_id",
    "bot_token",
    "channel_access_token",
    "channel_secret",
    "account_sid",
    "auth_token",
    "from_number",
    "caller_id",
    "messaging_service_sid",
    "api_key",
    "from_address",
    "reply_to",
    "business_account_id",
    "website_token",
    "widget_hmac_secret",
];

// ---------------------------------------------------------------------------
// HTTP plumbing
// ---------------------------------------------------------------------------

struct Reply {
    /// Which provider was dialled -- quoted in every refusal so an operator
    /// reading `HTTP 400` knows whose 400 it is.
    host: String,
    status: Option<u16>,
    body: Value,
    /// The response text when it was not JSON. A provider that answers a bad
    /// token with an HTML or plain-text page still gets reported with its own
    /// words instead of "no error body".
    raw_body: Option<String>,
    transport_error: Option<String>,
}

impl Reply {
    fn ok(&self) -> bool {
        self.status.is_some_and(|code| (200..300).contains(&code))
    }

    fn has_json_body(&self) -> bool {
        !self.body.is_null()
    }

    fn text(&self) -> String {
        match &self.transport_error {
            Some(err) => err.clone(),
            None => {
                let code = self
                    .status
                    .map(|code| code.to_string())
                    .unwrap_or_else(|| "no response".to_string());
                let detail = if self.has_json_body() {
                    self.body.to_string()
                } else {
                    self.raw_body.clone().unwrap_or_else(|| "(no body)".to_string())
                };
                format!("HTTP {code} from {}: {detail}", self.host)
            }
        }
    }
}

async fn read_reply(response: reqwest::Response, host: &str) -> Reply {
    let status = response.status().as_u16();
    // Read the text once: parsing from the same slice keeps a non-JSON refusal
    // (HTML error pages, `400 Bad Request` plain text) reportable.
    let (body, raw_body) = match response.text().await {
        Ok(text) => {
            let trimmed = text.trim().to_string();
            let parsed = serde_json::from_str::<Value>(&text).unwrap_or(Value::Null);
            (parsed, (!trimmed.is_empty()).then(|| truncate(&trimmed, 600)))
        }
        Err(err) => (
            Value::Null,
            Some(format!("(the response body could not be read: {err})")),
        ),
    };
    Reply { host: host.to_string(), status: Some(status), body, raw_body, transport_error: None }
}

fn transport_error(host: &str, err: reqwest::Error) -> Reply {
    Reply {
        host: host.to_string(),
        status: None,
        body: Value::Null,
        raw_body: None,
        transport_error: Some(format!(
            "Could not reach {host}: {err}. The request never arrived, so the message was not sent."
        )),
    }
}

fn truncate(value: &str, limit: usize) -> String {
    if value.chars().count() <= limit {
        return value.to_string();
    }
    let mut out: String = value.chars().take(limit).collect();
    out.push_str("…(truncated)");
    out
}

// ---------------------------------------------------------------------------
// Provider error envelopes
// ---------------------------------------------------------------------------

/// The one mapping from a provider's refusal to the text the caller sees.
/// Every adapter's failure path goes through it, so no provider error can be
/// flattened into a generic "send failed": transport failures keep their
/// connect error, JSON bodies go through that provider's own envelope, and a
/// body the provider did not send as JSON is quoted verbatim.
fn provider_error(provider: &str, reply: &Reply) -> String {
    if let Some(err) = &reply.transport_error {
        return err.clone();
    }
    let status = reply.status.unwrap_or(0);
    if !reply.has_json_body() {
        let detail = reply
            .raw_body
            .clone()
            .filter(|text| !text.trim().is_empty())
            .unwrap_or_else(|| "(the provider returned no body)".to_string());
        return format!("{} returned HTTP {status} from {} with a non-JSON body: {detail}", provider, reply.host);
    }
    match provider {
        "meta" | "messenger" | "instagram" => meta_error(&reply.body, status),
        "twilio" | "twilio_voice" => twilio_error(&reply.body, status),
        "telegram" => telegram_error(&reply.body, status),
        "discord" => discord_error(&reply.body, status),
        "postmark" => postmark_error(&reply.body, status),
        "line" => line_error(&reply.body, status),
        "tiktok" => tiktok_error(&reply.body, status),
        other => format!("{other} returned HTTP {status}: {}", reply.text()),
    }
}

/// The rejection every adapter returns for a provider that answered and said
/// no: the provider's own message, its HTTP status, and its payload as `raw`.
fn reject(provider: &str, reply: Reply) -> SendResult {
    let raw = if reply.has_json_body() {
        reply.body.clone()
    } else {
        json!({
            "httpStatus": reply.status,
            "host": reply.host,
            "rawBody": reply.raw_body,
            "note": "the provider replied with a body that was not JSON",
        })
    };
    SendResult::rejected(provider, reply.status, provider_error(provider, &reply), raw)
}

/// Meta's error envelope plus the codes that change what the operator must do.
fn meta_error(body: &Value, status: u16) -> String {
    let err = body.get("error");
    let message = err.and_then(|e| e.get("message")).and_then(|v| v.as_str()).unwrap_or("");
    let code = err.and_then(|e| e.get("code")).and_then(Value::as_i64);
    let subcode = err.and_then(|e| e.get("error_subcode")).and_then(Value::as_i64);
    let hint = code.map(explain_meta_code).unwrap_or_default();
    // `fbtrace_id` is what Meta support asks for when a Graph send is refused.
    let trace = err
        .and_then(|e| e.get("fbtrace_id"))
        .and_then(Value::as_str)
        .or_else(|| body.get("fbtrace_id").and_then(Value::as_str))
        .map(|id| format!(" [fbtrace_id: {id}]"))
        .unwrap_or_default();
    match (message.is_empty(), code) {
        (false, Some(code)) => {
            format!("Meta API error {code}{}: {message}{hint}{trace}", subcode.map(|s| format!("/{s}")).unwrap_or_default())
        }
        (false, _) => format!("{message}{trace}"),
        (true, Some(code)) => format!("Meta API error {code} (HTTP {status}){hint}{trace}"),
        // A 2xx carrying `success: false` is Meta's way of refusing without an
        // error envelope; the flag has to survive into the reported reason.
        (true, _) if body.get("success").and_then(Value::as_bool) == Some(false) => {
            format!("Meta reported success=false (HTTP {status}) with no error envelope{trace}")
        }
        (true, _) => format!("Meta returned HTTP {status} with no error envelope{trace}"),
    }
}

fn explain_meta_code(code: i64) -> String {
    let note = match code {
        131026 => " (WhatsApp could not deliver to this recipient: they may be banned, unregistered, or the number is invalid)",
        131047 | 131048 => " (the 24-hour customer service window is closed; send a pre-approved template instead)",
        131049 => " (Meta rate-limited this number; retry after the backoff)",
        130472 => " (sender is not permitted to message this recipient)",
        131030 => " (recipient cannot receive this message)",
        13301 => " (unknown message id)",
        10 | 190 => " (access token invalid or expired: reconnect the inbox)",
        368 => " (the action attempted is too spammy for this account)",
        131000 => " (required parameter missing from the payload)",
        _ => "",
    };
    if note.is_empty() { String::new() } else { note.to_string() }
}

/// Twilio exposes `20002`/`21211`/`11200`/`21612`… with a human `errorMessage`.
fn twilio_error(body: &Value, status: u16) -> String {
    let message = body
        .get("message")
        .and_then(Value::as_str)
        .or_else(|| body.get("ErrorMessage").and_then(Value::as_str))
        .unwrap_or("");
    let code = body.get("code").and_then(Value::as_i64);
    let more = body.get("more_info").and_then(Value::as_str).unwrap_or("");
    let note = code
        .and_then(|code| match code {
            20002 => Some(" (the account SID is malformed -- check the credential shape)"),
            21211 => Some(" (the From number is not valid or not owned by this account)"),
            11200 => Some(" (no messaging service or From number could route this message)"),
            21612 => Some(" (the recipient has opted out of messages from this number)"),
            21610 => Some(" (the recipient is unsubscribed)"),
            6303 => Some(" (carrier rejected the message)"),
            30002 => Some(" (recipient number invalid)"),
            20429 => Some(" (Twilio rate limited the request)"),
            _ => None,
        })
        .map(str::to_string)
        .unwrap_or_default();
    match (message.is_empty(), code) {
        (false, Some(code)) => format!("Twilio error {code}: {message}{note} [more_info: {more}]"),
        (false, _) => format!("Twilio error (HTTP {status}): {message}"),
        (true, Some(code)) => format!("Twilio error {code} (HTTP {status}){note}"),
        (true, _) => format!("Twilio returned HTTP {status} with no error message"),
    }
}

fn discord_error(body: &Value, status: u16) -> String {
    let message = body.get("message").and_then(Value::as_str).unwrap_or("");
    let code = body.get("code").and_then(Value::as_i64);
    match code {
        Some(50035) => format!("Discord: invalid form body (50035): {message}"),
        Some(50013) => format!("Discord: missing access/permissions (50013): {message}"),
        Some(50027) => format!("Discord: bot not in guild (50027): {message}"),
        Some(50034) => format!("Discord: channel not found (50034): {message}"),
        Some(401) => "Discord: bot token is invalid or revoked (401)".to_string(),
        Some(403) => format!("Discord: missing permissions (403): {message}"),
        Some(429) => format!("Discord rate limit (429): {message}"),
        Some(c) => format!("Discord error {c} (HTTP {status}): {message}"),
        None => format!("Discord error (HTTP {status}): {message}"),
    }
}

fn telegram_error(body: &Value, status: u16) -> String {
    let description = body.get("description").and_then(Value::as_str).unwrap_or("");
    let code = body
        .get("error_code")
        .and_then(Value::as_i64)
        .or_else(|| body.get("code").and_then(Value::as_i64));
    match code {
        Some(400) => format!("Telegram rejected the request (400): {description}"),
        Some(401) => "Telegram: bot token is invalid or revoked (401)".to_string(),
        Some(403) => format!("Telegram: the bot is blocked by this chat or it was deleted (403): {description}"),
        Some(429) => format!("Telegram rate limit (429): {description}"),
        Some(code) => format!("Telegram error {code} (HTTP {status}): {description}"),
        // Telegram answers refusals as HTTP 200 `{"ok":false,"description":…}`,
        // so an absent error_code still carries the provider's own reason.
        None if !description.is_empty() => {
            let ok = body.get("ok").and_then(Value::as_bool);
            let ok = ok.map(|flag| flag.to_string()).unwrap_or_else(|| "unknown".to_string());
            format!("Telegram refused the send (ok={ok}): {description}")
        }
        None => format!("Telegram returned HTTP {status} with no description"),
    }
}

fn postmark_error(body: &Value, status: u16) -> String {
    let message = body.get("Message").or_else(|| body.get("message")).and_then(Value::as_str).unwrap_or("");
    let code = body.get("ErrorCode").or_else(|| body.get("errorCode")).and_then(Value::as_i64);
    match (message, code) {
        ("", None) => format!("Postmark returned HTTP {status} with no Message field"),
        (message, Some(code)) => format!("Postmark error {code}: {message}"),
        (message, None) => message.to_string(),
    }
}

fn line_error(body: &Value, status: u16) -> String {
    let message = body.get("message").and_then(Value::as_str).unwrap_or("");
    let details = body
        .get("details")
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(|d| d.get("message").and_then(Value::as_str))
                .collect::<Vec<_>>()
                .join(", ")
        })
        .unwrap_or_default();
    if message.is_empty() && details.is_empty() {
        return format!("LINE returned HTTP {status} with no message");
    }
    format!("LINE error (HTTP {status}): {message}{}", if details.is_empty() { String::new() } else { format!(" — {details}") })
}

/// TikTok has two live error shapes: the Display/DM API nests
/// `{"error":{"code":"ok"|"…","message":"…"}}` with a STRING code, and the
/// API for Business answers `{"code":<int>,"message":"…"}` at the top level.
/// Both are read, and an absent code is reported as absent -- never as `0`,
/// which is the value that means success.
fn tiktok_error(body: &Value, status: u16) -> String {
    let nested = body.get("error");
    let message = nested
        .and_then(|e| e.get("message"))
        .and_then(Value::as_str)
        .or_else(|| body.get("message").and_then(Value::as_str))
        .unwrap_or("");
    let code = nested
        .and_then(|e| e.get("code"))
        .and_then(|v| v.as_str().map(str::to_string).or_else(|| v.as_i64().map(|n| n.to_string())))
        .or_else(|| body.get("code").and_then(|v| v.as_str().map(str::to_string).or_else(|| v.as_i64().map(|n| n.to_string()))));
    match (message, code) {
        ("", None) => format!("TikTok returned HTTP {status} with no error code or message"),
        ("", Some(code)) => format!("TikTok error code {code} (HTTP {status}) with no message"),
        (message, Some(code)) => format!("TikTok error {code} (HTTP {status}): {message}"),
        (message, None) => format!("TikTok error (HTTP {status}): {message}"),
    }
}

/// True only when TikTok's own envelope says success: HTTP 2xx AND the error
/// block is absent, `ok`, or code 0. A `code` that cannot be read is NOT a
/// success -- previously a string code such as `"permission_denied"` parsed as
/// no integer and the send was reported as accepted.
fn tiktok_reports_ok(body: &Value) -> bool {
    let nested = body.get("error");
    if let Some(nested) = nested {
        // A nested `error` object with no code at all still means the call
        // failed if it carries a message.
        if nested.is_object() {
            let code = nested.get("code");
            let message = nested.get("message").and_then(Value::as_str).unwrap_or("");
            return match code {
                Some(Value::String(text)) => text.eq_ignore_ascii_case("ok") && message.is_empty(),
                Some(other) => other.as_i64().is_some_and(|n| n == 0) && message.is_empty(),
                None => message.is_empty(),
            };
        }
        // `"error": null` / absent => no error block.
        return nested.is_null();
    }
    match body.get("code") {
        Some(Value::String(text)) => text.eq_ignore_ascii_case("ok"),
        Some(other) => other.as_i64().is_some_and(|n| n == 0),
        None => true,
    }
}

// ---------------------------------------------------------------------------
// Adapters
// ---------------------------------------------------------------------------

pub struct SendContext {
    pub destination: String,
    pub body: String,
    pub subject: Option<String>,
    pub inbox_id: Option<String>,
    /// Set only by callers that are sending a pre-approved provider template.
    pub template: Option<TemplateSpec>,
}

/// A provider template the operator chose. WhatsApp marketing messages outside
/// the 24-hour customer-service window can only be delivered as templates, so
/// the adapter needs the name, language and body parameters that were really
/// configured -- the wording is never composed in code.
#[derive(Clone, Debug)]
pub struct TemplateSpec {
    pub name: String,
    pub language: String,
    /// Fills `{{1}}`, `{{2}}`, ... in the template body, in order.
    pub body_params: Vec<String>,
}

pub trait ChannelAdapter: Send + Sync {
    fn channel(&self) -> &'static str;
    fn provider(&self) -> &'static str;
    /// Credential fields this adapter cannot run without -- used by
    /// `/inboxes` health checks to say what is missing before anyone tries.
    fn required_credentials(&self) -> &'static [&'static str];
    fn send<'a>(
        &'a self,
        state: &'a AppState,
        creds: &'a Credentials,
        ctx: &'a SendContext,
    ) -> SendFuture<'a>;
}

macro_rules! need {
    ($creds:expr, $key:expr, $provider:expr) => {
        match credential_aliases($key)
            .map(|aliases| $creds.require_any(aliases))
            .unwrap_or_else(|| $creds.require($key))
        {
            Ok(value) => value,
            Err(reason) => return SendResult::not_sent($provider, reason),
        }
    };
}

// --- WhatsApp via Meta Cloud API -------------------------------------------

pub struct WhatsAppMetaAdapter;

impl ChannelAdapter for WhatsAppMetaAdapter {
    fn channel(&self) -> &'static str {
        "whatsapp"
    }
    fn provider(&self) -> &'static str {
        "meta"
    }
    fn required_credentials(&self) -> &'static [&'static str] {
        &["access_token", "phone_number_id"]
    }
    fn send<'a>(&'a self, state: &'a AppState, creds: &'a Credentials, ctx: &'a SendContext) -> SendFuture<'a> {
        Box::pin(async move {
            let token = need!(creds, "access_token", "meta");
            let phone_id = need!(creds, "phone_number_id", "meta");
            let Some(api_version) = graph_version(state, creds) else {
                return no_graph_version("meta");
            };

            let payload = match &ctx.template {
                Some(template) => json!({
                    "messaging_product": "whatsapp",
                    "recipient_type": "individual",
                    "to": ctx.destination.trim().trim_start_matches('+'),
                    "type": "template",
                    "template": {
                        "name": template.name,
                        "language": { "code": template.language },
                        "components": [{
                            "type": "body",
                            "parameters": template
                                .body_params
                                .iter()
                                .map(|value| json!({ "type": "text", "text": value }))
                                .collect::<Vec<Value>>()
                        }]
                    }
                }),
                None => json!({
                    "messaging_product": "whatsapp",
                    "recipient_type": "individual",
                    "to": ctx.destination.trim().trim_start_matches('+'),
                    "type": "text",
                    "text": { "preview_url": false, "body": clamp(&ctx.body, 4096) },
                }),
            };
            send_meta_message(state, &api_version, &phone_id, &token, &payload).await
        })
    }
}

async fn send_meta_message(
    state: &AppState,
    api_version: &str,
    phone_id: &str,
    token: &str,
    payload: &Value,
) -> SendResult {
    let url = format!("https://graph.facebook.com/{api_version}/{phone_id}/messages");
    let mut delay_ms = 500u64;
    let mut attempt = 0u32;

    // Ported retry: 429 and 5xx are the only transient cases Meta returns.
    // The loop cannot exit without returning, so there is no synthetic Reply to
    // fall back on at the end.
    loop {
        attempt += 1;
        let response = state
            .http_client
            .post(&url)
            .bearer_auth(token)
            .json(payload)
            .timeout(Duration::from_secs(30))
            .send()
            .await;

        let reply = match response {
            Ok(response) => read_reply(response, "graph.facebook.com").await,
            Err(err) => transport_error("graph.facebook.com", err),
        };

        let retriable = match reply.status {
            Some(code) => matches!(code, 429 | 500 | 502 | 503 | 504),
            None => true,
        };
        if reply.ok() || !retriable || attempt >= 4 {
            return classify_meta_reply(reply);
        }
        tracing::warn!(
            "Meta attempt {attempt} failed ({}) -- backing off {delay_ms}ms",
            reply.text()
        );
        tokio::time::sleep(Duration::from_millis(delay_ms)).await;
        delay_ms *= 2;
    }
}

fn classify_meta_reply(reply: Reply) -> SendResult {
    if reply.ok() {
        // Meta hands back messages[0].id; a response without one is not a
        // receipt we can track, but the send did succeed, so the id stays empty
        // rather than being invented.
        let id = reply
            .body
            .get("messages")
            .and_then(Value::as_array)
            .and_then(|items| items.first())
            .and_then(|first| first.get("id"))
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string();
        let status = reply.status.unwrap_or(200);
        if reply.body.get("success").and_then(Value::as_bool) == Some(false) {
            return reject("meta", reply);
        }
        return SendResult::accepted("meta", id, status, reply.body);
    }
    reject("meta", reply)
}

fn clamp(value: &str, limit: usize) -> String {
    value.chars().take(limit).collect()
}

// --- Twilio (WhatsApp, SMS, voice) -----------------------------------------

/// Twilio account SIDs are `AC` + 32 hex characters. A value that is not that
/// shape can never authenticate, so the adapters refuse before dialling instead
/// of letting Twilio's `20002` arrive as an unexplained failure.
fn bad_twilio_sid(sid: &str) -> Option<SendResult> {
    (!sid.starts_with("AC") || sid.len() < 10).then(|| {
        SendResult::not_sent(
            "twilio",
            format!(
                "Twilio account SID '{sid}' is not the shape Twilio issues (AC + 32 hex chars). \
                 Nothing was sent."
            ),
        )
    })
}

pub struct TwilioWhatsAppAdapter;

impl ChannelAdapter for TwilioWhatsAppAdapter {
    fn channel(&self) -> &'static str {
        "whatsapp_twilio"
    }
    fn provider(&self) -> &'static str {
        "twilio"
    }
    fn required_credentials(&self) -> &'static [&'static str] {
        &["account_sid", "auth_token", "from_number"]
    }
    fn send<'a>(&'a self, state: &'a AppState, creds: &'a Credentials, ctx: &'a SendContext) -> SendFuture<'a> {
        Box::pin(async move {
            let sid = need!(creds, "account_sid", "twilio");
            let token = need!(creds, "auth_token", "twilio");
            let from = need!(creds, "from_number", "twilio");
            if let Some(refusal) = bad_twilio_sid(&sid) {
                return refusal;
            }
            let url = format!("https://api.twilio.com/2010-04-01/Accounts/{sid}/Messages.json");
            let response = state
                .http_client
                .post(&url)
                .basic_auth(&sid, Some(&token))
                .form(&[
                    ("From", format!("whatsapp:{from}")),
                    ("To", format!("whatsapp:{}", ctx.destination.trim_start_matches('+'))),
                    ("Body", ctx.body.clone()),
                ])
                .timeout(Duration::from_secs(30))
                .send()
                .await;
            classify_twilio_reply(
                match response {
                    Ok(response) => read_reply(response, "api.twilio.com").await,
                    Err(err) => transport_error("api.twilio.com", err),
                },
                "twilio",
            )
        })
    }
}

// --- SMS (Twilio) ----------------------------------------------------------

pub struct TwilioSmsAdapter;

impl ChannelAdapter for TwilioSmsAdapter {
    fn channel(&self) -> &'static str {
        "sms"
    }
    fn provider(&self) -> &'static str {
        "twilio"
    }
    fn required_credentials(&self) -> &'static [&'static str] {
        &["account_sid", "auth_token"]
    }
    fn send<'a>(&'a self, state: &'a AppState, creds: &'a Credentials, ctx: &'a SendContext) -> SendFuture<'a> {
        Box::pin(async move {
            let sid = need!(creds, "account_sid", "twilio");
            let token = need!(creds, "auth_token", "twilio");
            let service = creds.str_any_or_empty(&credential_keys("messaging_service_sid"));
            let from = creds.str_any_or_empty(&credential_keys("from_number"));
            if service.is_empty() && from.is_empty() {
                return SendResult::not_sent(
                    "twilio",
                    format!(
                        "SMS sender is not configured: neither a messaging service SID nor a \
                         Twilio From number is set. Looked in: {}. Nothing was sent.",
                        creds.searched.join("; ")
                    ),
                );
            }
            if let Some(refusal) = bad_twilio_sid(&sid) {
                return refusal;
            }

            let url = format!("https://api.twilio.com/2010-04-01/Accounts/{sid}/Messages.json");
            let mut form: Vec<(String, String)> =
                vec![("To".to_string(), ctx.destination.clone()), ("Body".to_string(), ctx.body.clone())];
            if !service.is_empty() {
                form.push(("MessagingServiceSid".to_string(), service));
            } else {
                form.push(("From".to_string(), from));
            }

            let response = state
                .http_client
                .post(&url)
                .basic_auth(&sid, Some(&token))
                .form(&form)
                .timeout(Duration::from_secs(30))
                .send()
                .await;
            classify_twilio_reply(
                match response {
                    Ok(response) => read_reply(response, "api.twilio.com").await,
                    Err(err) => transport_error("api.twilio.com", err),
                },
                "twilio",
            )
        })
    }
}

fn classify_twilio_reply(reply: Reply, provider: &str) -> SendResult {
    if reply.ok() {
        // Twilio's Messages.json and Calls.json both answer with `sid`; the
        // snake_case spelling is kept as a twin because the audit rows expose
        // both shapes to the UI.
        let sid = reply
            .body
            .get("sid")
            .or_else(|| reply.body.get("message_sid"))
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string();
        let provider_status = reply
            .body
            .get("status")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string();
        // Twilio answers 201 with status queued/scheduled: the message left us
        // and Twilio owns it, so it is Accepted -- but keep their status so the
        // UI can show "queued" rather than "delivered".
        let status = reply.status.unwrap_or(200);
        let mut result = SendResult::accepted(provider, sid, status, reply.body.clone());
        if let Some(raw) = result.raw.as_object_mut() {
            raw.insert("providerStatus".to_string(), Value::String(provider_status.clone()));
            raw.insert("provider_status".to_string(), Value::String(provider_status));
        }
        return result;
    }
    reject(provider, reply)
}

// --- Messenger (Facebook Page) ---------------------------------------------

pub struct MessengerAdapter;

impl ChannelAdapter for MessengerAdapter {
    fn channel(&self) -> &'static str {
        "facebook"
    }
    fn provider(&self) -> &'static str {
        "messenger"
    }
    fn required_credentials(&self) -> &'static [&'static str] {
        &["page_access_token"]
    }
    fn send<'a>(&'a self, state: &'a AppState, creds: &'a Credentials, ctx: &'a SendContext) -> SendFuture<'a> {
        Box::pin(async move {
            let token = need!(creds, "page_access_token", "messenger");
            let Some(api_version) = graph_version(state, creds) else {
                return no_graph_version("messenger");
            };
            send_graph_messages(
                state, &api_version, "me", &token, &ctx.destination, &ctx.body, "messenger",
            )
            .await
        })
    }
}

// --- Instagram -------------------------------------------------------------

pub struct InstagramAdapter;

impl ChannelAdapter for InstagramAdapter {
    fn channel(&self) -> &'static str {
        "instagram"
    }
    fn provider(&self) -> &'static str {
        "instagram"
    }
    fn required_credentials(&self) -> &'static [&'static str] {
        &["access_token"]
    }
    fn send<'a>(&'a self, state: &'a AppState, creds: &'a Credentials, ctx: &'a SendContext) -> SendFuture<'a> {
        Box::pin(async move {
            // Two real connection flows exist. Instagram Business Login tokens
            // only work against graph.instagram.com/<ig-user-id>/messages; a
            // page-linked account uses the Page endpoint. Which one applies is
            // read from the inbox row (`connected_page_id`, the key
            // `routes/inboxes.rs` writes for a page-linked Instagram inbox)
            // instead of defaulting to whichever one used to "simulate".
            let ig_user_id = creds.str_any_or_empty(&credential_keys("ig_id"));
            let page_token = creds.str_any_or_empty(&credential_keys("page_access_token"));
            let page_linked = !creds.str_any_or_empty(&["connected_page_id", "page_id"]).trim().is_empty();

            if !ig_user_id.is_empty() && !page_linked {
                let token = need!(creds, "access_token", "instagram");
                let Some(api_version) = graph_version(state, creds) else {
                    return no_graph_version("instagram");
                };
                let url = format!("https://graph.instagram.com/{api_version}/{ig_user_id}/messages");
                let response = state
                    .http_client
                    .post(&url)
                    .bearer_auth(&token)
                    .json(&json!({
                        "recipient": { "id": ctx.destination },
                        "message": { "text": clamp(&ctx.body, 4000) },
                    }))
                    .timeout(Duration::from_secs(15))
                    .send()
                    .await;
                return classify_graph_reply(
                    match response {
                        Ok(response) => read_reply(response, "graph.instagram.com").await,
                        Err(err) => transport_error("graph.instagram.com", err),
                    },
                    "instagram",
                );
            }

            if page_linked || !page_token.is_empty() {
                let token = match creds.first_any(&credential_keys("page_access_token")) {
                    Some(cred) => cred.value.clone(),
                    None => {
                        return SendResult::not_sent(
                            "instagram",
                            format!(
                                "This Instagram inbox is page-linked (a page id is stored) but \
                                 carries no page access token under {} . Looked in: {}. Nothing \
                                 was sent.",
                                credential_keys("page_access_token")
                                    .iter()
                                    .map(|key| format!("'{key}'"))
                                    .collect::<Vec<_>>()
                                    .join(" / "),
                                creds.searched.join("; ")
                            ),
                        );
                    }
                };
                let Some(api_version) = graph_version(state, creds) else {
                    return no_graph_version("instagram");
                };
                return send_graph_messages(
                    state, &api_version, "me", &token, &ctx.destination, &ctx.body, "instagram",
                )
                .await;
            }

            let token_refusal = creds.require_any(&credential_keys("access_token")).err();
            // Token present but no IG-scoped id and no page token: the
            // Instagram Business Login flow was never completed.
            SendResult::not_sent(
                "instagram",
                format!(
                    "Instagram is connected but neither flow can be taken: no ig user id \
                     (looked for {} in the inbox credentials) and no page access token for the \
                     page-linked flow. Reconnect the Instagram inbox so the scoped user id or the \
                     page token is stored. {}",
                    credential_keys("ig_id").iter().map(|key| format!("'{key}'")).collect::<Vec<_>>().join(" / "),
                    token_refusal.unwrap_or_else(|| "An Instagram access token was found.".to_string())
                ),
            )
        })
    }
}

/// The Graph API version to call. Taken from the inbox credentials, then the
/// vault-applied `Config`, then the environment -- never guessed per provider,
/// and when none of them answer the caller refuses instead of building a URL
/// with an invented version segment.
fn graph_version(state: &AppState, creds: &Credentials) -> Option<String> {
    if let Some(cred) = creds.first_any(&credential_keys("api_version")) {
        let version = cred.value.trim();
        if !version.is_empty() {
            return Some(version.to_string());
        }
    }
    // `meta_whatsapp_api_version` is the only Graph version field `Config`
    // carries; it is Meta's own global version, so the graph adapters read it
    // rather than each provider keeping a separate copy.
    let configured = state.config.meta_whatsapp_api_version.trim();
    if !configured.is_empty() {
        return Some(configured.to_string());
    }
    for name in ["META_GRAPH_API_VERSION", "GRAPH_API_VERSION", "META_API_VERSION"] {
        if let Ok(value) = env::var(name) {
            let value = value.trim();
            if !value.is_empty() {
                return Some(value.to_string());
            }
        }
    }
    None
}

/// The refusal both Meta-graph adapters share when no version is configured.
fn no_graph_version(provider: &str) -> SendResult {
    SendResult::not_sent(
        provider,
        "No Graph API version configured: set meta_whatsapp_api_version \
         (META_WHATSAPP_API_VERSION / META_GRAPH_API_VERSION) or store api_version \
         on the inbox. Graph has no 'latest' alias, so the request cannot be built \
         and nothing was sent.",
    )
}

async fn send_graph_messages(
    state: &AppState,
    api_version: &str,
    node: &str,
    token: &str,
    recipient: &str,
    body: &str,
    provider: &str,
) -> SendResult {
    let url = format!("https://graph.facebook.com/{api_version}/{node}/messages");
    let response = state
        .http_client
        .post(&url)
        .bearer_auth(token)
        .json(&json!({
            "recipient": { "id": recipient },
            "message": { "text": clamp(body, 4000) },
            "messaging_type": "RESPONSE",
        }))
        .timeout(Duration::from_secs(15))
        .send()
        .await;
    classify_graph_reply(
        match response {
            Ok(response) => read_reply(response, "graph.facebook.com").await,
            Err(err) => transport_error("graph.facebook.com", err),
        },
        provider,
    )
}

fn classify_graph_reply(reply: Reply, provider: &str) -> SendResult {
    if reply.ok() {
        // The Messenger/Instagram graph endpoints answer `{"message_id": …}`;
        // no id in the payload means no receipt, not a locally generated one.
        let id = reply
            .body
            .get("message_id")
            .or_else(|| reply.body.get("messageId"))
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string();
        return SendResult::accepted(provider, id, reply.status.unwrap_or(200), reply.body);
    }
    reject(provider, reply)
}

// --- Telegram --------------------------------------------------------------

pub struct TelegramAdapter;

impl ChannelAdapter for TelegramAdapter {
    fn channel(&self) -> &'static str {
        "telegram"
    }
    fn provider(&self) -> &'static str {
        "telegram"
    }
    fn required_credentials(&self) -> &'static [&'static str] {
        &["bot_token"]
    }
    fn send<'a>(&'a self, state: &'a AppState, creds: &'a Credentials, ctx: &'a SendContext) -> SendFuture<'a> {
        Box::pin(async move {
            let token = need!(creds, "bot_token", "telegram");
            let url = format!("https://api.telegram.org/bot{token}/sendMessage");
            let response = state
                .http_client
                .post(&url)
                .json(&json!({
                    "chat_id": ctx.destination,
                    "text": clamp(&ctx.body, 4096),
                }))
                .timeout(Duration::from_secs(15))
                .send()
                .await;

            let reply = match response {
                Ok(response) => read_reply(response, "api.telegram.org").await,
                Err(err) => transport_error("api.telegram.org", err),
            };
            // Telegram answers 200 {"ok":false,...} for most refusals, so the
            // `ok` flag decides, not the status code.
            if reply.ok() && reply.body.get("ok").and_then(Value::as_bool) != Some(false) {
                let id = reply
                    .body
                    .get("result")
                    .and_then(|r| r.get("message_id"))
                    .map(|v| match v {
                        Value::Number(num) => num.to_string(),
                        Value::String(text) => text.clone(),
                        _ => String::new(),
                    })
                    .unwrap_or_default();
                return SendResult::accepted("telegram", id, reply.status.unwrap_or(200), reply.body);
            }
            reject("telegram", reply)
        })
    }
}

// --- LINE ------------------------------------------------------------------

pub struct LineAdapter;

impl ChannelAdapter for LineAdapter {
    fn channel(&self) -> &'static str {
        "line"
    }
    fn provider(&self) -> &'static str {
        "line"
    }
    fn required_credentials(&self) -> &'static [&'static str] {
        &["channel_access_token"]
    }
    fn send<'a>(&'a self, state: &'a AppState, creds: &'a Credentials, ctx: &'a SendContext) -> SendFuture<'a> {
        Box::pin(async move {
            let token = need!(creds, "channel_access_token", "line");
            let url = "https://api.line.me/v2/bot/message/push";
            let response = state
                .http_client
                .post(url)
                .bearer_auth(&token)
                .json(&json!({
                    "to": ctx.destination,
                    "messages": [{ "type": "text", "text": clamp(&ctx.body, 5000) }],
                }))
                .timeout(Duration::from_secs(15))
                .send()
                .await;

            let reply = match response {
                Ok(response) => read_reply(response, "api.line.me").await,
                Err(err) => transport_error("api.line.me", err),
            };
            // LINE's push API returns 200 with an empty body and no id: the id
            // is deliberately left empty instead of being fabricated.
            if reply.ok() {
                return SendResult {
                    provider: "line".to_string(),
                    provider_message_id: String::new(),
                    status: SendStatus::Accepted,
                    http_status: reply.status,
                    error: None,
                    simulated: false,
                    raw: json!({
                        "note": "LINE push API returns no message id; delivery receipts come from the webhook",
                        "body": reply.body,
                    }),
                };
            }
            reject("line", reply)
        })
    }
}

// --- TikTok ----------------------------------------------------------------

pub struct TikTokAdapter;

/// The default host of TikTok's API for Business. Only the host is compiled in:
/// the DM send path is not guessed -- `message_send_url` (or `api_base` +
/// `message_send_path`) on the inbox picks the endpoint, because the path has
/// moved between API versions and a wrong one answers 404 in a way that looks
/// like a bad token.
const TIKTOK_BUSINESS_API_HOST: &str = "https://business-api.tiktok.com";

fn tiktok_send_url(creds: &Credentials) -> Option<String> {
    let explicit = creds.str_any_or_empty(&["message_send_url", "send_url", "endpoint_url"]);
    if !explicit.trim().is_empty() {
        return Some(explicit.trim().to_string());
    }
    let base = creds.str_any_or_empty(&["api_base", "api_base_url"]).trim().trim_end_matches('/').to_string();
    let path = creds.str_any_or_empty(&["message_send_path"]).trim().trim_matches('/').to_string();
    if base.is_empty() || path.is_empty() {
        return None;
    }
    Some(format!("{base}/{path}/"))
}

impl ChannelAdapter for TikTokAdapter {
    fn channel(&self) -> &'static str {
        "tiktok"
    }
    fn provider(&self) -> &'static str {
        "tiktok"
    }
    fn required_credentials(&self) -> &'static [&'static str] {
        &["access_token", "business_account_id", "conversation_id"]
    }
    fn send<'a>(&'a self, state: &'a AppState, creds: &'a Credentials, ctx: &'a SendContext) -> SendFuture<'a> {
        Box::pin(async move {
            // The token comes from the inbox row, then `Config.tiktok_access_token`
            // (vault or TIKTOK_ACCESS_TOKEN) -- never `tiktok_app_secret`, which
            // only signs inbound webhooks.
            let token = need!(creds, "access_token", "tiktok");
            let business_account = need!(creds, "business_account_id", "tiktok");
            let conversation_id = need!(creds, "conversation_id", "tiktok");
            let Some(url) = tiktok_send_url(creds) else {
                return SendResult::not_sent(
                    "tiktok",
                    format!(
                        "TikTok DM send endpoint is not configured for channel 'tiktok': the inbox \
                         needs `messageSendUrl`, or `apiBase` plus `messageSendPath`, pointing at \
                         TikTok's Business Messaging send endpoint (default host {TIKTOK_BUSINESS_API_HOST}). \
                         This build does not guess the path -- TikTok moves it between API \
                         versions and a guessed one answers 404. Nothing was sent to \
                         '{destination}'.",
                        destination = ctx.destination
                    ),
                );
            };
            let payload = json!({
                "business_id": business_account,
                "conversation_id": conversation_id,
                "recipient_open_id": ctx.destination,
                "message": { "type": "TEXT", "content": clamp(&ctx.body, 4000) },
            });
            let response = state
                .http_client
                .post(&url)
                // The API for Business authenticates with `Access-Token`; the
                // Display API uses a bearer. Both are sent so a token works
                // against whichever endpoint the operator configured.
                .header("Access-Token", &token)
                .bearer_auth(&token)
                .json(&payload)
                .timeout(Duration::from_secs(15))
                .send()
                .await;

            let host = url
                .split('/')
                .nth(2)
                .unwrap_or("business-api.tiktok.com")
                .to_string();
            let reply = match response {
                Ok(response) => read_reply(response, &host).await,
                Err(err) => transport_error(&host, err),
            };
            if reply.ok() && tiktok_reports_ok(&reply.body) {
                let id = reply
                    .body
                    .get("data")
                    .and_then(|d| {
                        d.get("message_id")
                            .or_else(|| d.get("messageId"))
                            .or_else(|| d.get("id"))
                    })
                    .map(|v| match v {
                        Value::String(text) => text.clone(),
                        Value::Number(num) => num.to_string(),
                        _ => String::new(),
                    })
                    .unwrap_or_default();
                return SendResult::accepted("tiktok", id, reply.status.unwrap_or(200), reply.body);
            }
            // A 2xx whose envelope says otherwise, and any non-2xx, is the
            // provider's own refusal.
            reject("tiktok", reply)
        })
    }
}

// --- Email -----------------------------------------------------------------

pub struct PostmarkEmailAdapter;

impl ChannelAdapter for PostmarkEmailAdapter {
    fn channel(&self) -> &'static str {
        "email"
    }
    fn provider(&self) -> &'static str {
        "postmark"
    }
    fn required_credentials(&self) -> &'static [&'static str] {
        &["api_key", "from_address"]
    }
    fn send<'a>(&'a self, state: &'a AppState, creds: &'a Credentials, ctx: &'a SendContext) -> SendFuture<'a> {
        Box::pin(async move {
            let requested = creds.str_or_default("provider");
            let normalized = requested.to_ascii_lowercase();
            if !normalized.is_empty() && normalized != "postmark" {
                return SendResult::not_sent(
                    "postmark",
                    format!(
                        "Email provider is configured as '{requested}'. This build has a live \
                         Postmark adapter only; {} has no HTTP adapter (and SMTP needs a client \
                         crate that Cargo.toml does not carry), so nothing was sent.",
                        if normalized == "smtp" { "SMTP" } else { requested.as_str() }
                    ),
                );
            }
            let api_key = need!(creds, "api_key", "postmark");
            let from_address = need!(creds, "from_address", "postmark");
            let from_name = creds.str_or_default("from_name");
            let reply_to = creds.str_or_default("reply_to");
            let from = if from_name.is_empty() {
                from_address.clone()
            } else {
                format!("{from_name} <{from_address}>")
            };
            let subject = ctx.subject.clone().unwrap_or_default();
            if subject.trim().is_empty() {
                return SendResult::not_sent(
                    "postmark",
                    "Email requires a subject; the request carried none, and Postmark rejects an \
                     empty one -- not inventing a default subject.",
                );
            }

            let mut payload = json!({
                "From": from,
                "To": ctx.destination,
                "Subject": subject,
                "TextBody": ctx.body,
                "HtmlBody": format!(
                    "<div style=\"font-family:sans-serif;line-height:1.5\">{}</div>",
                    html_escape(&ctx.body).replace('\n', "<br/>")
                ),
            });
            // Postmark's stream is a per-server setting; only send it when the
            // inbox names one, otherwise let Postmark use the server's default
            // rather than assuming a stream called "outbound" exists.
            let stream = creds.str_or_default("message_stream");
            let mut extras: Vec<(&str, Value)> = Vec::new();
            if !stream.is_empty() {
                extras.push(("MessageStream", Value::String(stream)));
            }
            if !reply_to.is_empty() {
                extras.push(("ReplyTo", Value::String(reply_to)));
            }
            if let Some(map) = payload.as_object_mut() {
                for (key, value) in extras {
                    map.insert(key.to_string(), value);
                }
            }

            let response = state
                .http_client
                .post("https://api.postmarkapp.com/email")
                .header("X-Postmark-Server-Token", api_key)
                .header("Accept", "application/json")
                .json(&payload)
                .timeout(Duration::from_secs(30))
                .send()
                .await;

            let reply = match response {
                Ok(response) => read_reply(response, "api.postmarkapp.com").await,
                Err(err) => transport_error("api.postmarkapp.com", err),
            };
            if reply.ok() {
                let id = reply
                    .body
                    .get("MessageID")
                    .or_else(|| reply.body.get("messageId"))
                    .map(|v| match v {
                        Value::String(text) => text.clone(),
                        other => other.to_string(),
                    })
                    .unwrap_or_default();
                return SendResult::accepted("postmark", id, reply.status.unwrap_or(200), reply.body);
            }
            SendResult::rejected(
                "postmark",
                reply.status,
                if reply.transport_error.is_some() { reply.text() } else { postmark_error(&reply.body, reply.status.unwrap_or(0)) },
                reply.body,
            )
        })
    }
}

fn html_escape(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '&' => out.push_str("&amp;"),
            '"' => out.push_str("&quot;"),
            _ => out.push(ch),
        }
    }
    out
}

// --- Voice -----------------------------------------------------------------

pub struct TwilioVoiceAdapter;

impl ChannelAdapter for TwilioVoiceAdapter {
    fn channel(&self) -> &'static str {
        "voice"
    }
    fn provider(&self) -> &'static str {
        "twilio_voice"
    }
    fn required_credentials(&self) -> &'static [&'static str] {
        &["account_sid", "auth_token", "caller_id"]
    }
    fn send<'a>(&'a self, state: &'a AppState, creds: &'a Credentials, ctx: &'a SendContext) -> SendFuture<'a> {
        Box::pin(async move {
            let requested = creds.str_or_default("provider");
            let normalized = requested.to_ascii_lowercase();
            if !normalized.is_empty() && !normalized.starts_with("twilio") {
                return SendResult::not_sent(
                    "twilio_voice",
                    format!(
                        "Voice gateway is configured as '{requested}'. Only Twilio Calls has a live \
                         adapter in this build; Telnyx/Plivo/Vonage/SIP trunks are not ported, so \
                         no call was placed and none will be reported as queued."
                    ),
                );
            }
            let sid = need!(creds, "account_sid", "twilio_voice");
            let token = need!(creds, "auth_token", "twilio_voice");
            let caller_id = creds.str_or_default("caller_id");
            let caller_id = if caller_id.is_empty() {
                match creds.require("from_number") {
                    Ok(value) => value,
                    Err(_) => return SendResult::not_sent("twilio_voice", format!(
                        "No caller-id / voice number for outbound calls (searched: {}).",
                        creds.searched.join("; ")
                    )),
                }
            } else {
                caller_id
            };
            if !sid.starts_with("AC") || sid.len() < 10 {
                return SendResult::not_sent(
                    "twilio_voice",
                    format!("Twilio account SID '{sid}' is not the shape Twilio issues (AC + 32 chars)."),
                );
            }

            let twiml = format!(
                "<Response><Say language=\"en-US\">{}</Say></Response>",
                html_escape(&clamp(&ctx.body, 400))
            );
            let url = format!("https://api.twilio.com/2010-04-01/Accounts/{sid}/Calls.json");
            let response = state
                .http_client
                .post(&url)
                .basic_auth(&sid, Some(&token))
                .form(&[
                    ("To", ctx.destination.clone()),
                    ("From", caller_id),
                    ("Twiml", twiml),
                    ("Record", "true".to_string()),
                ])
                .timeout(Duration::from_secs(15))
                .send()
                .await;
            classify_twilio_reply(
                match response {
                    Ok(response) => read_reply(response, "api.twilio.com").await,
                    Err(err) => transport_error("api.twilio.com", err),
                },
                "twilio_voice",
            )
        })
    }
}

// --- Discord ---------------------------------------------------------------

pub struct DiscordAdapter;

impl ChannelAdapter for DiscordAdapter {
    fn channel(&self) -> &'static str {
        "discord"
    }
    fn provider(&self) -> &'static str {
        "discord"
    }
    fn required_credentials(&self) -> &'static [&'static str] {
        &["bot_token", "discord_client_id"]
    }
    fn send<'a>(&'a self, state: &'a AppState, creds: &'a Credentials, ctx: &'a SendContext) -> SendFuture<'a> {
        Box::pin(async move {
            let token = need!(creds, "bot_token", "discord");
            let channel_id = ctx.destination.trim();
            let url = format!("https://discord.com/api/v10/channels/{channel_id}/messages");
            let response = state
                .http_client
                .post(&url)
                .header("Authorization", format!("Bot {token}"))
                .json(&json!({
                    "content": clamp(&ctx.body, 2000),
                }))
                .timeout(Duration::from_secs(15))
                .send()
                .await;

            let reply = match response {
                Ok(response) => read_reply(response, "discord.com").await,
                Err(err) => transport_error("discord.com", err),
            };
            if reply.ok() {
                let id = reply
                    .body
                    .get("id")
                    .map(|v| v.to_string())
                    .unwrap_or_default();
                return SendResult::accepted("discord", id, reply.status.unwrap_or(200), reply.body);
            }
            reject("discord", reply)
        })
    }
}

// --- Web chat --------------------------------------------------------------

pub struct WebChatAdapter;

impl ChannelAdapter for WebChatAdapter {
    fn channel(&self) -> &'static str {
        "webchat"
    }
    fn provider(&self) -> &'static str {
        "web_widget"
    }
    fn required_credentials(&self) -> &'static [&'static str] {
        &["website_token", "widget_hmac_secret"]
    }
    fn send<'a>(&'a self, _state: &'a AppState, creds: &'a Credentials, ctx: &'a SendContext) -> SendFuture<'a> {
        Box::pin(async move {
            // The Python "provider" only logged the send and returned
            // delivered: true. Nothing better is possible here yet either: there
            // is no widget transport to call -- the `ws` feature is compiled into
            // axum but no hub is mounted, and a website inbox row carries no
            // message-push endpoint -- so `Accepted` would report a delivery no
            // transport actually performed.
            //
            // The credentials are resolved instead of discarded so the refusal
            // names the smallest fixable gap first: an unconfigured
            // `website_token` / `widget_hmac_secret` is operator-fixable today,
            // while the missing transport is a build gap. Only field names and
            // the places they were searched are reported, never the values.
            let credential_gaps: Vec<String> = self
                .required_credentials()
                .iter()
                .filter_map(|key| creds.require(*key).err())
                .collect();
            let no_transport = format!(
                "Webchat reply to visitor '{visitor}' was not delivered: this build has no widget \
                 message transport to send through -- no websocket hub is mounted and the website \
                 inbox row defines no message-push endpoint, so there is no provider URL to call. \
                 The reply exists only as a `messages` row the visitor can pick up by polling \
                 GET /widget/conversations/<conversation id>/messages/since; nothing was confirmed \
                 received.",
                visitor = ctx.destination,
            );
            if !credential_gaps.is_empty() {
                return SendResult::not_sent(
                    "web_widget",
                    format!("{} | {}", credential_gaps.join(" | "), no_transport),
                );
            }
            SendResult::not_sent("web_widget", no_transport)
        })
    }
}

// ---------------------------------------------------------------------------
// Registry + dispatch
// ---------------------------------------------------------------------------

static ADAPTERS: [&dyn ChannelAdapter; 12] = [
    &WhatsAppMetaAdapter,
    &TwilioWhatsAppAdapter,
    &TwilioSmsAdapter,
    &MessengerAdapter,
    &InstagramAdapter,
    &TelegramAdapter,
    &LineAdapter,
    &PostmarkEmailAdapter,
    &TikTokAdapter,
    &TwilioVoiceAdapter,
    &WebChatAdapter,
    &DiscordAdapter,
];

/// Channel aliases the rest of the product uses for the same transport.
fn canonical_channel(channel: &str) -> String {
    let lower = channel.trim().to_ascii_lowercase();
    match lower.as_str() {
        "whatsapp_cloud" | "whatsapp_meta" | "meta" => "whatsapp".to_string(),
        "whatsapp_twilio" | "twilio_whatsapp" | "twilio whatsapp" => "whatsapp_twilio".to_string(),
        "facebook" | "messenger" | "fb" => "facebook".to_string(),
        "ig" | "instagram_dm" | "meta_instagram" => "instagram".to_string(),
        "tg" => "telegram".to_string(),
        "email" | "mail" | "smtp" | "postmark" => "email".to_string(),
        "tt" | "tiktok_dm" => "tiktok".to_string(),
        "live_chat" | "livechat" | "web_widget" | "website" | "webchat" | "chat" => "webchat".to_string(),
        "voice_call" | "telephony" | "voice_sip" => "voice".to_string(),
        "text" | "sms_twilio" => "sms".to_string(),
        other => other.to_string(),
    }
}

pub fn adapter_for(channel: &str) -> Option<&'static dyn ChannelAdapter> {
    let target = canonical_channel(channel);
    ADAPTERS
        .iter()
        .copied()
        .find(|adapter| adapter.channel() == target)
}

pub fn registered_channels() -> Vec<&'static str> {
    ADAPTERS.iter().map(|adapter| adapter.channel()).collect()
}

/// What a channel would refuse, checked without touching the network. Used by
/// the Connections health endpoint so "configured" means "usable".
pub fn missing_credentials_report<'a>(
    state: &'a AppState,
    channel: &'a str,
    inbox_id: Option<&'a str>,
) -> SendFuture<'a> {
    Box::pin(async move {
        match adapter_for(channel) {
            None => SendResult::not_sent(
                channel,
                format!(
                    "No outbound adapter is registered for channel '{channel}'. Registered: {}.",
                    registered_channels().join(", ")
                ),
            ),
            Some(adapter) => {
                let creds = resolve_credentials(state, adapter.channel(), inbox_id).await;
                let mut missing = Vec::new();
                for key in adapter.required_credentials() {
                    if !creds.has(key) {
                        missing.push(creds.require(key).unwrap_err());
                    }
                }
                if missing.is_empty() {
                    SendResult::not_sent(
                        adapter.provider(),
                        format!(
                            "Credential pre-check only: {} are all configured, but no message was \
                             sent by this check.",
                            adapter.required_credentials().join(", ")
                        ),
                    )
                } else {
                    SendResult::not_sent(adapter.provider(), missing.join(" | "))
                }
            }
        }
    })
}

/// The single outbound entry point every route should use.
pub async fn send(
    state: &AppState,
    channel: &str,
    destination: &str,
    body: &str,
    subject: Option<&str>,
    inbox_id: Option<&str>,
) -> SendResult {
    let ctx = SendContext {
        destination: destination.to_string(),
        body: body.to_string(),
        subject: subject.map(str::to_string),
        inbox_id: inbox_id.map(str::to_string),
        template: None,
    };
    send_with(state, channel, &ctx).await
}

/// The transport for a caller that already holds a built context -- used by
/// broadcasts, which send a provider template rather than free text.
pub async fn send_with(state: &AppState, channel: &str, ctx: &SendContext) -> SendResult {
    let mut adapter = match adapter_for(channel) {
        Some(adapter) => adapter,
        None => {
            return SendResult::not_sent(
                channel,
                format!(
                    "No outbound adapter is registered for channel '{channel}'. Registered: {}. \
                     The message was not delivered anywhere.",
                    registered_channels().join(", ")
                ),
            )
        }
    };

    if ctx.destination.trim().is_empty() {
        return SendResult::not_sent(
            adapter.provider(),
            format!(
                "No recipient on channel '{}': the contact row has an empty value, so there is \
                 nothing to address the message to.",
                adapter.channel()
            ),
        );
    }

    let mut creds = resolve_credentials(state, adapter.channel(), ctx.inbox_id.as_deref()).await;

    // WhatsApp has two live transports and the tenant's Connections row picks
    // one (`whatsapp_provider` = meta | twilio). Reading it here means a Twilio
    // WhatsApp number is never pushed at the Meta Graph endpoint.
    if adapter.channel() == "whatsapp" {
        let chosen = {
            let raw = creds.str_or_default("whatsapp_provider");
            if raw.is_empty() { creds.str_or_default("provider") } else { raw }
        };
        if chosen.eq_ignore_ascii_case("twilio") {
            adapter = &TwilioWhatsAppAdapter;
            creds = resolve_credentials(state, "whatsapp_twilio", ctx.inbox_id.as_deref()).await;
        }
    }
    let result = adapter.send(state, &creds, ctx).await;
    if !result.is_accepted() {
        tracing::warn!(
            channel = adapter.channel(),
            provider = %result.provider,
            status = result.status.as_str(),
            "outbound send did not reach the recipient: {}",
            result.reason()
        );
    }
    result
}

// ---------------------------------------------------------------------------
// Receipt parsing
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct StatusReceipt {
    pub provider: String,
    pub provider_message_id: String,
    /// Normalised: sent | delivered | read | failed | queued | bounced.
    pub status: &'static str,
    pub provider_status: String,
    pub recipient: String,
    pub timestamp: Option<String>,
    pub error_code: Option<i64>,
    pub error: Option<String>,
    pub raw: Value,
}

fn normalize_status(channel: &str, raw: &str) -> &'static str {
    match raw.to_ascii_lowercase().as_str() {
        "sent" => "sent",
        "delivered" => "delivered",
        "read" | "seen" => "read",
        "failed" | "undelivered" | "error" | "rejected" | "blocked" => "failed",
        "bounced" | "expired" => "bounced",
        "queued" | "scheduled" | "accepted" | "in_progress" | "initiated" | "ringing" | "answering" => "queued",
        "canceled" | "busy" | "noanswer" => "failed",
        "completed" => "delivered",
        other => {
            tracing::debug!("unmapped {channel} receipt status '{other}' -- kept as queued");
            "queued"
        }
    }
}

/// Turns a provider webhook/notification body into the receipts it reports.
/// Unknown shapes yield no receipts rather than a guessed one.
pub fn parse_receipts(channel: &str, body: &Value) -> Vec<StatusReceipt> {
    let channel = canonical_channel(channel);
    let mut receipts = Vec::new();

    match channel.as_str() {
        "whatsapp" => {
            for entry in body.get("entry").and_then(Value::as_array).into_iter().flatten() {
                for change in entry.get("changes").and_then(Value::as_array).into_iter().flatten() {
                    let value = change.get("value");
                    let Some(phone) = value
                        .and_then(|v| v.get("metadata"))
                        .and_then(|m| m.get("phone_number_id"))
                        .and_then(Value::as_str)
                    else { continue };
                    let Some(statuses) = value.and_then(|v| v.get("statuses")).and_then(Value::as_array) else { continue };
                    for status_event in statuses {
                        let raw_status = status_event.get("status").and_then(Value::as_str).unwrap_or_default();
                        let error = status_event.get("errors").and_then(Value::as_array).and_then(|errs| errs.first());
                        receipts.push(StatusReceipt {
                            provider: "meta".to_string(),
                            provider_message_id: status_event
                                .get("id")
                                .and_then(Value::as_str)
                                .unwrap_or_default()
                                .to_string(),
                            status: normalize_status("whatsapp", raw_status),
                            provider_status: raw_status.to_string(),
                            recipient: status_event
                                .get("recipient_id")
                                .and_then(Value::as_str)
                                .unwrap_or(phone)
                                .to_string(),
                            timestamp: status_event
                                .get("timestamp")
                                .and_then(|t| match t {
                                    Value::String(text) => Some(text.clone()),
                                    Value::Number(num) => Some(num.to_string()),
                                    _ => None,
                                }),
                            error_code: error.and_then(|e| e.get("code")).and_then(Value::as_i64),
                            error: error.map(|e| {
                                let title = e.get("title").and_then(Value::as_str).unwrap_or("");
                                let message = e.get("message").and_then(Value::as_str).unwrap_or("");
                                format!("{title}: {message}")
                            }),
                            raw: status_event.clone(),
                        });
                    }
                }
            }
        }
        // Twilio status callbacks (form-encoded) and message JSON both land
        // here; the field names differ between the two shapes.
        "sms" | "whatsapp_twilio" | "voice" => {
            let id = first_string(body, &["MessageSid", "message_sid", "CallSid", "call_sid", "Sid", "sid"]);
            let raw_status = first_string(body, &["Status", "status", "CallStatus"]);
            if !id.is_empty() && !raw_status.is_empty() {
                receipts.push(StatusReceipt {
                    provider: "twilio".to_string(),
                    provider_message_id: id,
                    status: normalize_status(&channel, &raw_status),
                    provider_status: raw_status.clone(),
                    recipient: first_string(body, &["To", "to", "ToCountry"]),
                    timestamp: Some(first_string(body, &["DateSent", "date_sent", "EndTime", "StartTime"])),
                    error_code: body
                        .get("ErrorCode")
                        .or_else(|| body.get("error_code"))
                        .and_then(|v| v.as_i64().or_else(|| v.as_str().and_then(|s| s.parse().ok()))),
                    error: non_empty(body.get("ErrorMessage").or_else(|| body.get("error_message")).and_then(Value::as_str)),
                    raw: body.clone(),
                });
            }
        }
        "email" => {
            let kind = first_string(body, &["Type", "type", "BounceType"]);
            let id = first_string(body, &["MessageID", "ID", "Id", "InboundMessageId"]);
            if !id.is_empty() {
                let mapped = match kind.to_ascii_lowercase().as_str() {
                    "bounce" | "softbounce" | "expired" | "complaint" => "failed",
                    "spamcomplainer" => "bounced",
                    "delivery" => "delivered",
                    "open" => "read",
                    _ => "sent",
                };
                receipts.push(StatusReceipt {
                    provider: "postmark".to_string(),
                    provider_message_id: id,
                    status: mapped,
                    provider_status: kind.clone(),
                    recipient: first_string(body, &["Email", "Recipient"]),
                    timestamp: non_empty(body.get("DeliveredAt").and_then(Value::as_str)),
                    error_code: None,
                    error: non_empty(
                        body.get("Description")
                            .or_else(|| body.get("BouncedAt"))
                            .and_then(|v| v.as_str()),
                    ),
                    raw: body.clone(),
                });
            }
        }
        "telegram" => {
            // Telegram has no delivery receipts for bots; an edited/delete
            // notification is the only state change it reports.
        }
        "line" => {
            for item in body.get("events").and_then(Value::as_array).into_iter().flatten() {
                let kind = item.get("type").and_then(Value::as_str).unwrap_or_default();
                let reply_token = item.get("reply").and_then(|r| r.get("replyToken")).and_then(Value::as_str);
                if matches!(kind, "message" | "follow") {
                    receipts.push(StatusReceipt {
                        provider: "line".to_string(),
                        provider_message_id: reply_token.unwrap_or_default().to_string(),
                        status: if kind == "follow" { "delivered" } else { "sent" },
                        provider_status: kind.to_string(),
                        recipient: item
                            .get("source")
                            .and_then(|s| s.get("userId"))
                            .and_then(Value::as_str)
                            .unwrap_or_default()
                            .to_string(),
                        timestamp: item.get("timestamp").and_then(|t| t.as_i64()).map(|t| t.to_string()),
                        error_code: None,
                        error: None,
                        raw: item.clone(),
                    });
                }
            }
        }
        _ => {}
    }

    receipts
}

fn first_string(body: &Value, keys: &[&str]) -> String {
    for key in keys {
        if let Some(value) = body.get(key) {
            match value {
                Value::String(text) if !text.is_empty() => return text.clone(),
                Value::Number(num) => return num.to_string(),
                _ => {}
            }
        }
    }
    String::new()
}

fn non_empty(value: Option<&str>) -> Option<String> {
    value.map(str::to_string).filter(|v| !v.is_empty())
}
