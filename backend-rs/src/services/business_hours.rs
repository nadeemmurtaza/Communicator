//! Business / working hours, computed from the row the operator actually saved.
//!
//! Port of `services/business_hours.py`. The Python version was three lines:
//! if gating is on, compare `business_hours_start <= now.hour < end` in
//! `ZoneInfo(row.business_timezone)`, and on *any* timezone failure it caught
//! the exception and used UTC -- so a mistyped zone quietly moved a tenant's
//! open window by hours. Here the zone is resolved explicitly and an
//! unresolvable zone is reported as such instead of being replaced by a guess.
//!
//! `Cargo.toml` has `chrono` but **not** `chrono-tz`, so IANA names
//! (`Asia/Kolkata`) cannot be resolved. What *is* supported, and what the
//! callers get an honest verdict for:
//!
//! * `UTC`, `GMT`, `Z`, `Etc/UTC`, `+05:30`, `-8`, `UTC+5:30`, `GMT-0800`
//! * `local` / `server` -- the operating-system offset, read from chrono at
//!   call time (real data, not a constant);
//! * anything else -> [`HoursVerdict::TimezoneUnresolved`], which never blocks
//!   a send (the caller cannot claim a state it did not compute) and tells the
//!   operator to store an offset.
//!
//! Two real sources of hours exist and both are read: `integration_settings`
//! (the tenant-wide Connections window) and `inboxes` (the per-channel
//! `working_hours_*` columns with their `out_of_office_message`).

use crate::db::{AppState, DbPool};
use crate::services::integration_settings;
use chrono::{FixedOffset, Local, Offset, TimeZone, Timelike, Utc};
use sqlx::Row;

/// One configured open window, in whole hours `[start, end)`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorkingHours {
    pub enabled: bool,
    /// SQLite integers arrive as `i64` (`integration_settings` stores these
    /// columns as `INTEGER`), so the window is kept at the source width and
    /// never narrowed by a lossy cast; [`WorkingHours::bounds_valid`] is what
    /// decides whether a value is a real hour at all.
    pub start_hour: i64,
    pub end_hour: i64,
    pub timezone: String,
    /// Where the row came from, so a verdict can be traced ("Connections page",
    /// "inbox 'support' (abc123)").
    pub source: String,
}

impl WorkingHours {
    /// The tenant-wide window from the Connections row.
    pub fn from_settings(row: &integration_settings::IntegrationSettings, source: &str) -> Self {
        Self {
            enabled: row.business_hours_enabled,
            start_hour: row.business_hours_start,
            end_hour: row.business_hours_end,
            timezone: row.business_timezone.clone(),
            source: source.to_string(),
        }
    }

    /// A window is only usable when both bounds were saved; `integration_settings`
    /// defaults them to -1 ("never configured"), and `business_profile.rs` writes
    /// real hours, so -1 must not be read as "open all day".
    fn bounds_valid(&self) -> bool {
        (0..=23).contains(&self.start_hour) && (0..=23).contains(&self.end_hour)
    }

    pub fn window(&self) -> String {
        format!("{:02}:00-{:02}:00", self.start_hour, self.end_hour)
    }

    /// `0..24` in the configured zone, or the zone's resolution failure.
    pub fn evaluate(&self, now_utc: chrono::DateTime<Utc>) -> HoursVerdict {
        if !self.enabled {
            return HoursVerdict::GatingDisabled {
                window: self.window(),
                timezone: self.timezone.clone(),
                source: self.source.clone(),
            };
        }
        if !self.bounds_valid() {
            return HoursVerdict::NotConfigured {
                reason: format!(
                    "{} has business hours {} ({}) -- the window was never saved, so no verdict \
                     can be computed and nothing is being deferred.",
                    self.source,
                    self.window(),
                    if self.timezone.is_empty() { "no timezone" } else { &self.timezone }
                ),
                window: self.window(),
            };
        }

        let (offset, label) = match resolve_offset(&self.timezone) {
            Ok(resolved) => resolved,
            Err(reason) => {
                return HoursVerdict::TimezoneUnresolved { reason, window: self.window() };
            }
        };

        let local = offset.from_utc_datetime(&now_utc.naive_utc());
        let hour = local.hour();
        let hour_value = i64::from(hour);
        // Python's `start <= hour < end` silently means "always closed" for an
        // overnight window (20 -> 6). A night shift is a real configuration, so
        // the wrap is handled instead of reported as a closed office.
        let open = if self.start_hour <= self.end_hour {
            (self.start_hour..self.end_hour).contains(&hour_value)
        } else {
            hour_value >= self.start_hour || hour_value < self.end_hour
        };
        let now_local = local.format("%Y-%m-%d %H:%M:%S %:z").to_string();

        if open {
            HoursVerdict::Open {
                local_hour: hour,
                window: self.window(),
                timezone: label,
                now_local,
                source: self.source.clone(),
            }
        } else {
            HoursVerdict::Closed {
                local_hour: hour,
                window: self.window(),
                timezone: label,
                now_local,
                source: self.source.clone(),
            }
        }
    }
}

/// What the hours check produced. Every variant carries enough to tell the
/// operator *why*, which is the whole point of the module.
#[derive(Clone, Debug)]
pub enum HoursVerdict {
    /// Inside the saved window.
    Open {
        local_hour: u32,
        window: String,
        timezone: String,
        now_local: String,
        source: String,
    },
    /// Outside it -- the state that defers an outbound send.
    Closed {
        local_hour: u32,
        window: String,
        timezone: String,
        now_local: String,
        source: String,
    },
    /// The operator switched gating off: sends go out at any hour.
    GatingDisabled { window: String, timezone: String, source: String },
    /// No hours saved yet (start/end still the -1 "unknown" default).
    NotConfigured { reason: String, window: String },
    /// A window exists but its zone cannot be resolved without `chrono-tz`.
    TimezoneUnresolved { reason: String, window: String },
    /// No row at all -- the Connections page was never opened for this tenant.
    MissingSettings { reason: String },
}

impl HoursVerdict {
    /// The only verdict that stops a send. `Open`, `GatingDisabled`,
    /// `NotConfigured`, `TimezoneUnresolved` and `MissingSettings` do not: an
    /// uncomputed window must never silently swallow a message.
    pub fn blocks_send(&self) -> bool {
        matches!(self, Self::Closed { .. })
    }

    pub fn is_closed(&self) -> bool {
        self.blocks_send()
    }

    pub fn state(&self) -> &'static str {
        match self {
            Self::Open { .. } => "open",
            Self::Closed { .. } => "closed",
            Self::GatingDisabled { .. } => "gating_disabled",
            Self::NotConfigured { .. } => "not_configured",
            Self::TimezoneUnresolved { .. } => "timezone_unresolved",
            Self::MissingSettings { .. } => "missing_settings",
        }
    }

    /// One-line, human-readable status for API responses and logs.
    pub fn describe(&self) -> String {
        match self {
            Self::Open { local_hour, window, timezone, now_local, source } => {
                format!("OPEN: local hour {local_hour} is inside {window} ({timezone}) at {now_local} [{source}]")
            }
            Self::Closed { local_hour, window, timezone, now_local, source } => {
                format!("CLOSED: local hour {local_hour} is outside {window} ({timezone}) at {now_local} [{source}]")
            }
            Self::GatingDisabled { source, .. } => {
                format!("business-hours gating is off for {source}; every hour is open")
            }
            Self::NotConfigured { reason, .. } => reason.clone(),
            Self::TimezoneUnresolved { reason, .. } => reason.clone(),
            Self::MissingSettings { reason } => reason.clone(),
        }
    }

    pub fn to_json(&self) -> serde_json::Value {
        use serde_json::json;
        match self {
            Self::Open { local_hour, window, timezone, now_local, source }
            | Self::Closed { local_hour, window, timezone, now_local, source } => json!({
                "state": self.state(),
                "open": !self.blocks_send(),
                "blocksSend": self.blocks_send(),
                "localHour": local_hour,
                "window": window,
                "timezone": timezone,
                "nowLocal": now_local,
                "source": source,
                "detail": self.describe(),
            }),
            Self::GatingDisabled { window, timezone, source } => json!({
                "state": self.state(), "open": true, "blocksSend": false,
                "window": window, "timezone": timezone, "source": source,
                "detail": self.describe(),
            }),
            Self::NotConfigured { window, .. } | Self::TimezoneUnresolved { window, .. } => json!({
                "state": self.state(), "open": true, "blocksSend": false,
                "window": window, "detail": self.describe(),
            }),
            Self::MissingSettings { .. } => json!({
                "state": self.state(), "open": true, "blocksSend": false,
                "detail": self.describe(),
            }),
        }
    }
}

/// Resolves the stored zone string to a concrete offset. `Err` carries the
/// operator-facing reason; the second tuple element of `Ok` is the label that
/// goes into the verdict so the log shows which zone was actually used.
pub fn resolve_offset(name: &str) -> Result<(FixedOffset, String), String> {
    let raw = name.trim();
    if raw.is_empty() || matches!(raw.to_ascii_uppercase().as_str(), "UTC" | "GMT" | "Z" | "ETC/UTC" | "ETC/GMT")
    {
        // Zero offset is in range by definition; `expect` documents that rather
        // than silently falling back to a different zone.
        let utc = FixedOffset::east_opt(0).expect("UTC is a valid fixed offset");
        return Ok((utc, "UTC".to_string()));
    }

    if matches!(raw.to_ascii_lowercase().as_str(), "local" | "server" | "system") {
        let offset = Local::now().offset().fix();
        return Ok((offset, format!("local ({})", offset)));
    }

    if let Some(offset) = parse_offset(raw) {
        return Ok((offset, raw.to_string()));
    }

    Err(format!(
        "Timezone '{raw}' cannot be resolved. This build links `chrono` only (no `chrono-tz` in \
         Cargo.toml), so IANA zone names are not available: store a fixed offset such as \
         '+05:30' or 'UTC-8', or use 'local' for the server offset. Business hours are NOT being \
         enforced for this window rather than guessed against UTC."
    ))
}

/// `+05:30`, `-8`, `UTC+5:30`, `GMT-0800`, `Etc/GMT+5` (inverted sign, per the
/// POSIX convention that zone uses), `IST=+05:30`-style explicit overrides.
fn parse_offset(input: &str) -> Option<FixedOffset> {
    let upper = input.to_ascii_uppercase().replace(' ', "");
    let rest = upper
        .strip_prefix("UTC")
        .or_else(|| upper.strip_prefix("GMT"))
        .or_else(|| upper.strip_prefix("UT"))
        .unwrap_or(upper.as_str());

    // "IST=+05:30" / "Asia/Kolkata=+05:30": the operator pinned the offset.
    if let Some((_, explicit)) = rest.split_once('=') {
        if let Some(offset) = parse_sign_offset(explicit) {
            return Some(offset);
        }
    }

    // Etc/GMT±N has an inverted sign compared to the ISO convention.
    if let Some(posix) = upper.strip_prefix("ETC/GMT") {
        if let Some(hours) = posix.parse::<i32>().ok() {
            return FixedOffset::east_opt(-hours * 3600);
        }
    }

    if rest.starts_with('+')
        || rest.starts_with('-')
        || matches!(rest.chars().next(), Some(first) if first.is_ascii_digit())
    {
        return parse_sign_offset(rest);
    }
    None
}

fn parse_sign_offset(text: &str) -> Option<FixedOffset> {
    let text = text.trim();
    let (sign, digits) = match text.as_bytes().first() {
        Some(b'-') => (-1i32, &text[1..]),
        Some(b'+') => (1i32, &text[1..]),
        _ => (1i32, text),
    };
    let digits = digits.trim();
    if digits.is_empty() {
        return None;
    }
    let (hours, minutes) = match digits.split_once(':') {
        Some((h, m)) => (h.parse::<i32>().ok()?, m.parse::<i32>().ok()?),
        None if digits.len() > 2 && digits.chars().all(|c| c.is_ascii_digit()) => {
            let (h, m) = digits.split_at(digits.len() - 2);
            (h.parse::<i32>().ok()?, m.parse::<i32>().ok()?)
        }
        None => (digits.parse::<i32>().ok()?, 0),
    };
    if minutes.abs() > 59 || hours.abs() > 14 {
        return None;
    }
    FixedOffset::east_opt(sign * (hours * 3600 + sign * minutes * 60))
}

// ---------------------------------------------------------------------------
// Real data access
// ---------------------------------------------------------------------------

fn int_flag(row: &sqlx::sqlite::SqliteRow, column: &str) -> Option<i64> {
    if let Ok(value) = row.try_get::<i64, _>(column) {
        return Some(value);
    }
    if let Ok(value) = row.try_get::<bool, _>(column) {
        return Some(i64::from(value));
    }
    if let Ok(Some(value)) = row.try_get::<Option<String>, _>(column) {
        return match value.to_ascii_lowercase().as_str() {
            "1" | "true" | "yes" | "on" => Some(1),
            "0" | "false" | "no" | "off" => Some(0),
            other => other.parse::<i64>().ok(),
        };
    }
    if let Ok(Some(value)) = row.try_get::<Option<i64>, _>(column) {
        return Some(value);
    }
    None
}

fn opt_text(row: &sqlx::sqlite::SqliteRow, column: &str) -> String {
    row.try_get::<String, _>(column)
        .or_else(|_| row.try_get::<Option<String>, _>(column).map(|v| v.unwrap_or_default()))
        .unwrap_or_default()
}

/// The tenant's Connections window, read live. `Ok(None)` means no row exists.
pub async fn tenant_hours(pool: &DbPool, tenant_key: &str) -> Option<WorkingHours> {
    let source = format!("integration_settings tenant '{tenant_key}'");
    if let Ok(Some(row)) = integration_settings::load(pool, tenant_key).await {
        return Some(WorkingHours::from_settings(&row, &source));
    }
    // Single-tenant deployments never set DEFAULT_TENANT_KEY; the one row is
    // still the operator's answer.
    let row = integration_settings::load_any(pool).await.ok().flatten()?;
    Some(WorkingHours::from_settings(&row, "integration_settings (only row)"))
}

/// Evaluate the tenant window right now.
pub async fn check_tenant_hours(state: &AppState, tenant_key: &str) -> HoursVerdict {
    let now = Utc::now();
    match tenant_hours(&state.db, tenant_key).await {
        Some(hours) => hours.evaluate(now),
        None => HoursVerdict::MissingSettings {
            reason: format!(
                "No integration_settings row for tenant '{tenant_key}', so no business-hours window \
                 exists; the send is not deferred on hours and the Connections page should be saved."
            ),
        },
    }
}

/// Convenience for callers that only need the boolean the Python helper gave.
pub async fn tenant_is_open(state: &AppState, tenant_key: &str) -> bool {
    !check_tenant_hours(state, tenant_key).await.blocks_send()
}

/// The away message the operator configured, but only while the tenant is
/// actually closed -- replying "we are offline" during office hours is the kind
/// of fabricated state this module exists to prevent. An empty `away_message`
/// yields `None` rather than a canned line.
pub async fn tenant_away_message(state: &AppState, tenant_key: &str) -> Option<String> {
    let settings = match integration_settings::load(&state.db, tenant_key).await {
        Ok(Some(row)) => Some(row),
        _ => integration_settings::load_any(&state.db).await.ok().flatten(),
    }?;

    let hours = WorkingHours::from_settings(
        &settings,
        &format!("integration_settings tenant '{tenant_key}'"),
    );
    if !hours.evaluate(Utc::now()).blocks_send() {
        return None;
    }
    if settings.away_message.trim().is_empty() {
        tracing::info!(
            "business hours are closed for tenant '{tenant_key}' but away_message is empty; \
             no auto-reply is sent"
        );
        return None;
    }
    Some(settings.away_message)
}

/// Per-inbox working hours (`inboxes.working_hours_*`), which is what the widget
/// and channel inboxes expose in the UI, with the matching out-of-office text.
pub async fn check_inbox_hours(state: &AppState, inbox_id: &str) -> Option<(HoursVerdict, String)> {
    if inbox_id.is_empty() {
        return None;
    }
    let row = sqlx::query(
        "SELECT id, name, working_hours_enabled, working_hours_start, working_hours_end, \
         timezone, out_of_office_enabled, out_of_office_message FROM inboxes WHERE id = ? LIMIT 1",
    )
    .bind(inbox_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|err| {
        tracing::warn!("inbox working hours read failed for {inbox_id}: {err}");
        err
    })
    .ok()?;
    let row = row?;

    let id = opt_text(&row, "id");
    let name = opt_text(&row, "name");
    let enabled = int_flag(&row, "working_hours_enabled").unwrap_or(0) != 0;
    let start = int_flag(&row, "working_hours_start").unwrap_or(-1);
    let end = int_flag(&row, "working_hours_end").unwrap_or(-1);
    let timezone = opt_text(&row, "timezone");
    let ooo_enabled = int_flag(&row, "out_of_office_enabled").unwrap_or(0) != 0;
    let ooo_message = opt_text(&row, "out_of_office_message");

    let hours = WorkingHours {
        enabled,
        start_hour: start,
        end_hour: end,
        timezone,
        source: format!("inbox '{name}' ({id})"),
    };
    let verdict = hours.evaluate(Utc::now());
    let away = if ooo_enabled { ooo_message } else { String::new() };
    Some((verdict, away))
}

/// Next time the window opens, as a plain description the away message can use.
/// Computed from the real offset and the saved hours, not from a constant.
pub fn next_open_text(hours: &WorkingHours, now_utc: chrono::DateTime<Utc>) -> Option<String> {
    if !hours.enabled || !hours.bounds_valid() {
        return None;
    }
    let (offset, _) = resolve_offset(&hours.timezone).ok()?;
    let local = offset.from_utc_datetime(&now_utc.naive_utc());
    // `Timelike::hour()`/`minute()` hand back `u32`; the window bounds are
    // `i64`, so widen the clock values instead of narrowing the saved hours.
    let hour = i64::from(local.hour());
    let minute = i64::from(local.minute());
    let mut delta = hours.start_hour - hour;
    if delta <= 0 {
        delta += 24;
    }
    let total_minutes = delta * 60 - minute;
    let days = total_minutes / (24 * 60);
    let remaining = total_minutes % (24 * 60);
    Some(format!(
        "opens in {}{} minutes ({:02}:00 {})",
        if days > 0 { format!("{days} day(s) ") } else { String::new() },
        remaining,
        hours.start_hour,
        if days > 0 { "tomorrow" } else { "today" }
    ))
}
