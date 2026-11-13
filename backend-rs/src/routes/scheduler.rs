//! Phase 4 scheduled background runners (port of `api/routes/scheduler.py`).
//!
//! The Python module is honest about one thing: nothing in this repo owns a
//! task queue, so an external cron is expected to POST here. This port keeps
//! that contract and adds what the Python version did not have:
//!
//! * a real `scheduler_jobs` row per job (interval, enabled flag, last run,
//!   last status, and a `next_run_at` computed with chrono), so the schedule
//!   survives restarts and is readable by the UI;
//! * a real `scheduler_runs` history row per invocation, including duration;
//! * candidate selection with the same SQL predicates the Python services use,
//!   evaluated against this schema's real columns;
//! * actual provider dispatch through the gateway layer (`send_text_via_gateway_row`)
//!   for follow-ups, auto-approved drafts and inactivity nudges. Every count
//!   named `sent`/`nudged`/`dispatched` is the number of *successful* provider
//!   round-trips; refusals (no gateway row, no credential, no destination) and
//!   failures are returned with the provider's own message, never folded into
//!   `sent`, and -- outside a dry run -- stored on the conversation as a
//!   `failed`/`not_sent` message row carrying the reason;
//! * candidate selection and dispatch scoped to the request's tenant (the
//!   `Tenant` extractor: X-Tenant-Key header, else DEFAULT_TENANT_KEY);
//!
//! When something cannot be done for real, the response says so instead of
//! pretending: `executed: false` plus the reason, and `dryRun` when the caller
//! (or `SCHEDULER_DRY_RUN=1`) asked for a rehearsal.

use crate::db::{report_write, AppState};
use crate::routes::auth::Tenant;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use chrono::{DateTime, Datelike, Duration as ChronoDuration, NaiveDateTime, TimeZone, Timelike, Utc};
use serde::Deserialize;
use serde_json::{json, Value};
use std::sync::OnceLock;
use tracing::debug;
use uuid::Uuid;

use super::gateways_webhooks_stripe::{gateway_row_for_channel, send_text_via_gateway_row};

/// Jobs this process knows how to run. Keys are stable identifiers stored in
/// `scheduler_jobs.job_key`, not display strings.
const JOBS: &[(&str, &str, i64)] = &[
    (
        "followups",
        "Sends the next step of a conversation's follow-up sequence once next_followup_at is due",
        900,
    ),
    (
        "auto_approved_drafts",
        "Dispatches copilot replies the reply engine already marked approved",
        300,
    ),
    (
        "inactivity_nudges",
        "Nudges WhatsApp conversations sitting in the 18-23.5h inbound window, before the 24h session closes",
        600,
    ),
    (
        "campaigns",
        "Dispatches broadcasts whose scheduled_at has passed",
        300,
    ),
];

const SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS scheduler_jobs (
    id TEXT PRIMARY KEY,
    job_key TEXT NOT NULL UNIQUE,
    description TEXT NOT NULL DEFAULT '',
    interval_seconds INTEGER NOT NULL,
    is_enabled INTEGER NOT NULL DEFAULT 1,
    last_run_at TEXT,
    next_run_at TEXT,
    last_status TEXT NOT NULL DEFAULT 'never',
    last_candidates INTEGER NOT NULL DEFAULT 0,
    last_executed INTEGER NOT NULL DEFAULT 0,
    last_error TEXT NOT NULL DEFAULT '',
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS scheduler_runs (
    id TEXT PRIMARY KEY,
    job_key TEXT NOT NULL,
    started_at TEXT NOT NULL,
    finished_at TEXT,
    duration_ms INTEGER NOT NULL DEFAULT 0,
    candidates INTEGER NOT NULL DEFAULT 0,
    executed INTEGER NOT NULL DEFAULT 0,
    failed INTEGER NOT NULL DEFAULT 0,
    status TEXT NOT NULL DEFAULT '',
    detail TEXT NOT NULL DEFAULT '',
    dry_run INTEGER NOT NULL DEFAULT 0
);
CREATE INDEX IF NOT EXISTS ix_scheduler_runs_job ON scheduler_runs (job_key, started_at);
"#;

static SCHEMA_READY: OnceLock<()> = OnceLock::new();

// ---------------------------------------------------------------------------
// Small shared helpers
// ---------------------------------------------------------------------------

fn now_iso() -> String {
    Utc::now().to_rfc3339()
}

/// Writes the run's reported counts and status depend on. Unlike
/// `crate::db::report_write`, logging is not enough here: this returns the
/// reason a statement did not land -- an error, or a row count other than the
/// single row expected -- so the caller can put it in the response's `notes`
/// and stop claiming the work succeeded.
fn write_failure(
    result: Result<sqlx::postgres::PgQueryResult, sqlx::Error>,
    context: &str,
) -> Option<String> {
    match result {
        Ok(report) if report.rows_affected() == 1 => None,
        Ok(report) => Some(format!(
            "{context} affected {} rows (expected 1)",
            report.rows_affected()
        )),
        Err(err) => Some(format!("{context} failed: {err}")),
    }
}

/// A count the response publishes as a number. `Some(0)` is the measurement
/// "nothing is due"; `None` means the read itself failed, and rendering that as
/// a zero would tell the operator the queue is empty when it is unknown.
fn measured(
    result: Result<i64, sqlx::Error>,
    label: &str,
    notes: &mut Vec<String>,
) -> Option<i64> {
    match result {
        Ok(value) => Some(value),
        Err(err) => {
            notes.push(format!(
                "{label} could not be read ({err}); it is reported as unknown, not as zero"
            ));
            None
        }
    }
}

/// Secrets/tuning this module needs but `config.rs` does not expose. Read from
/// the process environment only; an empty value is treated as unset.
fn env_value(name: &str) -> Option<String> {
    std::env::var(name)
        .ok()
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
}

fn env_int(name: &str) -> Option<i64> {
    env_value(name).and_then(|v| v.parse::<i64>().ok())
}

/// Both the RFC-3339 form this backend writes and the `YYYY-MM-DD HH:MM:SS`
/// form seeded rows use. A timestamp that cannot be parsed is never treated as
/// "due" -- that would silently send traffic against a row we cannot read.
fn parse_ts(raw: Option<&str>) -> Option<DateTime<Utc>> {
    let raw = raw?.trim();
    if raw.is_empty() {
        return None;
    }
    if let Ok(dt) = DateTime::parse_from_rfc3339(raw) {
        return Some(dt.with_timezone(&Utc));
    }
    let normalised = raw.replace('T', " ").replace('Z', "").trim().to_string();
    for fmt in ["%Y-%m-%d %H:%M:%S%.f", "%Y-%m-%d %H:%M"] {
        if let Ok(ndt) = NaiveDateTime::parse_from_str(&normalised, fmt) {
            return Some(Utc.from_utc_datetime(&ndt));
        }
    }
    None
}

async fn table_exists(state: &AppState, table: &str) -> Option<bool> {
    match sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM information_schema.tables WHERE table_schema='public' AND table_name=$1")
        .bind(table)
        .fetch_one(&state.db)
        .await
    {
        Ok(count) => Some(count > 0),
        // Answering `Some(false)` here would let the response claim the table is
        // missing when all that happened is that the check failed.
        Err(_) => None,
    }
}

/// Creates the table pair on first use. `db/mod.rs` is not this module's to
/// edit, so the DDL also has to run here; it is `IF NOT EXISTS` and only
/// executed once per process.
async fn ensure_schema(state: &AppState) -> Vec<String> {
    let mut notes = Vec::new();
    if SCHEMA_READY.get().is_some() {
        return notes;
    }
    match sqlx::query(SCHEMA).execute(&state.db).await {
        Ok(_) => {
            // Two runners can pass the `get()` guard together, so losing the
            // `set` is an expected outcome, not a failure to swallow: the call
            // that won has already marked the same `IF NOT EXISTS` DDL ready.
            if SCHEMA_READY.set(()).is_err() {
                debug!("scheduler schema was marked ready by a concurrent run");
            }
        }
        Err(err) => notes.push(format!(
            "scheduler_jobs / scheduler_runs could not be created ({err}); nothing about this run is persisted"
        )),
    }
    notes
}

/// The cadence a job is registered at: the operator's env override when set,
/// otherwise the `JOBS` registry value. `None` means this process has no
/// runner for the key, and inventing a cadence for a job that cannot run is
/// worse than refusing to register it.
fn default_interval(job_key: &str) -> Option<i64> {
    let registry_secs = JOBS
        .iter()
        .find(|(key, _, _)| *key == job_key)
        .map(|(_, _, secs)| *secs)?;
    let env_name = format!("SCHEDULER_INTERVAL_SECONDS_{}", job_key.to_ascii_uppercase());
    Some(
        env_int(&env_name)
            .filter(|v| *v >= 30)
            .unwrap_or(registry_secs),
    )
}

#[derive(Debug, Clone, sqlx::FromRow)]
struct JobRow {
    id: String,
    job_key: String,
    description: String,
    interval_seconds: i64,
    is_enabled: bool,
    last_run_at: Option<String>,
    next_run_at: Option<String>,
    last_status: String,
    last_candidates: i64,
    last_executed: i64,
    last_error: String,
    created_at: String,
    updated_at: String,
}

const SELECT_JOB: &str = "SELECT id, job_key, description, interval_seconds, is_enabled, last_run_at, next_run_at, last_status, last_candidates, last_executed, last_error, created_at, updated_at FROM scheduler_jobs WHERE job_key = ?";

impl JobRow {
    fn json(&self, now: DateTime<Utc>) -> Value {
        let next = parse_ts(self.next_run_at.as_deref());
        json!({
            "id": self.id,
            "jobKey": self.job_key,
            "job_key": self.job_key,
            "description": self.description,
            "intervalSeconds": self.interval_seconds,
            "interval_seconds": self.interval_seconds,
            "isEnabled": self.is_enabled,
            "is_enabled": self.is_enabled,
            "lastRunAt": self.last_run_at,
            "last_run_at": self.last_run_at,
            "nextRunAt": self.next_run_at,
            "next_run_at": self.next_run_at,
            // A job with no readable `next_run_at` is not due, it is unknown:
            // answering from `is_enabled` would invent a due/not-due state.
            "dueNow": next.map(|n| n <= now),
            "lastStatus": self.last_status,
            "lastCandidates": self.last_candidates,
            "lastExecuted": self.last_executed,
            "lastError": self.last_error,
            "createdAt": self.created_at,
            "updatedAt": self.updated_at,
        })
    }
}

/// `Ok(None)` is the fact "never registered"; `Err` is "the read itself failed"
/// -- collapsing the two would let a broken table read as a missing job and
/// send the caller off to INSERT over the same failure.
async fn load_job(state: &AppState, job_key: &str) -> Result<Option<JobRow>, sqlx::Error> {
    sqlx::query_as::<_, JobRow>(SELECT_JOB)
        .bind(job_key)
        .fetch_optional(&state.db)
        .await
}

/// Reads the schedule row, creating it (with the real registry cadence) when
/// the job has never been registered. This is the "persist a real schedule
/// row" half of the module: after one run the row exists and `next_run_at` is
/// a concrete chrono-computed instant, not a rendered string. An `Err` carries
/// the exact reason the schedule is unknown or unstored -- a read failure, an
/// unknown job key (refused, never registered at a made-up cadence), or an
/// INSERT/read-back that did not land -- so every caller can put it in `notes`
/// instead of advertising a `next_run_at` the database does not hold.
async fn ensure_job(state: &AppState, job_key: &str) -> Result<JobRow, String> {
    match load_job(state, job_key).await {
        Ok(Some(row)) => return Ok(row),
        Ok(None) => {}
        Err(err) => {
            return Err(format!(
                "the scheduler_jobs row for {job_key} could not be read ({err}); whether the job \
                 is registered is unknown, so no row was inserted and no schedule is stored"
            ));
        }
    }

    let Some((_, registry_description, registry_secs)) =
        JOBS.iter().find(|(key, _, _)| *key == job_key)
    else {
        return Err(format!(
            "{job_key} names no runner in this process's JOBS registry: it was not registered and \
             will not be scheduled, because its cadence would have to be invented"
        ));
    };
    let interval = default_interval(job_key).unwrap_or(*registry_secs);

    let now = Utc::now();
    let next = now + ChronoDuration::seconds(interval);
    let insert = sqlx::query(
        "INSERT INTO scheduler_jobs (id, job_key, description, interval_seconds, is_enabled, last_run_at, next_run_at, last_status, last_candidates, last_executed, last_error, created_at, updated_at) VALUES (?, ?, ?, ?, 1, NULL, ?, 'never', 0, 0, '', ?, ?)",
    )
    .bind(Uuid::new_v4().simple().to_string())
    .bind(job_key)
    .bind(registry_description)
    .bind(interval)
    .bind(next.to_rfc3339())
    .bind(now.to_rfc3339())
    .bind(now.to_rfc3339())
    .execute(&state.db)
    .await;
    if let Some(reason) = write_failure(insert, "INSERT INTO scheduler_jobs (registering the job)") {
        return Err(format!(
            "{reason}: the {job_key} schedule is not stored, so it will not survive a restart"
        ));
    }

    match load_job(state, job_key).await {
        Ok(Some(row)) => Ok(row),
        Ok(None) => Err(format!(
            "the scheduler_jobs row for {job_key} was inserted but reads back as absent; the \
             schedule reported for this job is not confirmed in the database"
        )),
        Err(err) => Err(format!(
            "the scheduler_jobs row for {job_key} was inserted but could not be read back ({err}); \
             the schedule reported for this job is not confirmed in the database"
        )),
    }
}

/// Writes the history row and rolls the schedule forward. `status` is derived
/// from what actually happened, never asserted. Both writes are part of what
/// `persisted` claims below, so a failure goes into the caller's `notes` as
/// well as flipping that flag.
async fn finish_run(
    state: &AppState,
    job_key: &str,
    started: DateTime<Utc>,
    candidates: usize,
    executed: usize,
    failed: usize,
    status: &str,
    detail: &str,
    dry_run: bool,
    notes: &mut Vec<String>,
) -> Value {
    let now = Utc::now();
    let duration_ms = (now - started).num_milliseconds();
    let history = sqlx::query(
        "INSERT INTO scheduler_runs (id, job_key, started_at, finished_at, duration_ms, candidates, executed, failed, status, detail, dry_run) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(Uuid::new_v4().simple().to_string())
    .bind(job_key)
    .bind(started.to_rfc3339())
    .bind(now.to_rfc3339())
    .bind(duration_ms)
    .bind(candidates as i64)
    .bind(executed as i64)
    .bind(failed as i64)
    .bind(status)
    .bind(detail)
    .bind(dry_run)
    .execute(&state.db)
    .await;
    let history_failure = write_failure(history, "INSERT INTO scheduler_runs (the history row)");
    if let Some(reason) = &history_failure {
        notes.push(format!(
            "{reason}: this {job_key} run is missing from scheduler_runs, so its counts are only \
             in this response"
        ));
    }

    let job = ensure_job(state, job_key).await;
    if let Err(reason) = &job {
        notes.push(reason.clone());
    }
    let next = match job.as_ref() {
        Ok(job) if job.is_enabled => Some(now + ChronoDuration::seconds(job.interval_seconds.max(30))),
        // A disabled job has no next run; a job whose row could not be read or
        // registered has no stored cadence to compute one from either.
        Ok(_) | Err(_) => None,
    };
    let roll_failure = match job.as_ref() {
        Ok(_) => {
            let roll = sqlx::query(
                "UPDATE scheduler_jobs SET last_run_at = ?, last_status = ?, last_candidates = ?, last_executed = ?, last_error = ?, next_run_at = ?, updated_at = ? WHERE job_key = ?",
            )
            .bind(now.to_rfc3339())
            .bind(status)
            .bind(candidates as i64)
            .bind(executed as i64)
            .bind(if failed > 0 || status == "error" { detail } else { "" })
            .bind(next.as_ref().map(|d| d.to_rfc3339()))
            .bind(now.to_rfc3339())
            .bind(job_key)
            .execute(&state.db)
            .await;
            write_failure(roll, "UPDATE scheduler_jobs (last_run_at / next_run_at)")
        }
        // There is no confirmed row to roll; saying so beats an UPDATE that
        // would report 0 rowsAffected over a failure already explained.
        Err(_) => None,
    };
    if let Some(reason) = &roll_failure {
        notes.push(format!(
            "{reason}: the stored schedule was not rolled forward, so nextRunAt in this response is \
             not what scheduler_jobs holds"
        ));
    }

    json!({
        "jobKey": job_key,
        "ranAt": now.to_rfc3339(),
        "nextRunAt": next.map(|d| d.to_rfc3339()),
        "durationMs": duration_ms,
        "persisted": job.as_ref().ok().is_some_and(|job| !job.id.is_empty())
            && history_failure.is_none()
            && roll_failure.is_none(),
    })
}

/// Caller-level knobs shared by all four runners.
#[derive(Debug, Default, Deserialize)]
pub struct RunQuery {
    #[serde(default, alias = "dryRun")]
    pub dry_run: Option<bool>,
    /// Hard ceiling on how many sends one invocation may perform. A cron that
    /// fires after an outage should not stampede a provider.
    #[serde(default, alias = "limit", alias = "maxItems")]
    pub max: Option<i64>,
    /// Ignore the configured working-hours window for this invocation.
    #[serde(default)]
    pub force: Option<bool>,
}

impl RunQuery {
    /// `SCHEDULER_DRY_RUN=1` makes every runner count and report candidates
    /// without touching a provider. The caller's explicit flag wins.
    fn dry_run(&self) -> bool {
        self.dry_run.unwrap_or_else(|| {
            matches!(
                env_value("SCHEDULER_DRY_RUN").unwrap_or_default().as_str(),
                "1" | "true" | "yes" | "on"
            )
        })
    }

    fn max(&self) -> usize {
        self.max.unwrap_or(50).clamp(1, 200) as usize
    }

    fn force(&self) -> bool {
        self.force.unwrap_or(false)
    }
}

/// True when `now` sits inside the configured working-hours window. The window
/// comes from the environment (`FOLLOW_UP_QUIET_HOURS_START` / `_END`, the same
/// names the Python settings use); when unset, no gate is applied rather than
/// inventing one.
fn working_hours(now: DateTime<Utc>) -> (bool, String) {
    let start = env_int("FOLLOW_UP_QUIET_HOURS_START");
    let end = env_int("FOLLOW_UP_QUIET_HOURS_END");
    let hour = now.hour() as i64;
    match (start, end) {
        (Some(s), Some(e)) => {
            let weekend = now.weekday().num_days_from_monday() >= 5;
            if weekend {
                return (
                    false,
                    format!("{} is outside the Mon-Fri window", now.weekday()),
                );
            }
            let open = s <= hour && hour < e;
            (
                open,
                format!("local hour {hour}, configured window {s}..{e} (UTC){}", if open { "" } else { " -- deferred" }),
            )
        }
        _ => (
            true,
            "FOLLOW_UP_QUIET_HOURS_START/END not set; no working-hours gate applied".to_string(),
        ),
    }
}

// ---------------------------------------------------------------------------
// Real dispatch
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
struct SendOutcome {
    ok: bool,
    attempted: bool,
    /// The gateway row's display name, kept for detail/reporting use.
    gateway: Option<String>,
    /// The `provider` column of the gateway row that carried (or would have
    /// carried) the traffic -- the real value stored in `messages.provider`.
    provider: Option<String>,
    detail: String,
    provider_message_id: Option<String>,
}

impl SendOutcome {
    fn refused(detail: impl Into<String>) -> Self {
        Self {
            ok: false,
            attempted: false,
            gateway: None,
            provider: None,
            detail: detail.into(),
            provider_message_id: None,
        }
    }

    fn json(&self, target: &str, body: &str) -> Value {
        json!({
            "target": target,
            "ok": self.ok,
            "liveCallMade": self.attempted,
            "gateway": self.gateway,
            "provider": self.provider,
            "detail": self.detail,
            "providerMessageId": self.provider_message_id,
            "bodyPreview": truncate(body, 160),
        })
    }
}

fn truncate(input: &str, max_chars: usize) -> String {
    let count = input.chars().count();
    if count <= max_chars {
        return input.to_string();
    }
    let mut out: String = input.chars().take(max_chars).collect();
    out.push('…');
    out
}

/// Sends one text through the stored gateway connection for the channel. Every
/// branch reports the truth: `attempted` means a provider request was actually
/// made, `ok` means it answered 2xx.
async fn attempt_send(state: &AppState, channel: &str, target: &str, body: &str, dry_run: bool) -> SendOutcome {
    let target = target.trim();
    if target.is_empty() {
        return SendOutcome::refused(format!(
            "no destination address on the {channel} contact row; refusing to send to a substitute"
        ));
    }
    if target.contains('•') {
        return SendOutcome::refused(
            "the stored destination is a masked display value, not an address; refusing to transmit",
        );
    }
    if body.trim().is_empty() {
        return SendOutcome::refused("nothing to send: the resolved message body is empty");
    }

    let Some(row) = gateway_row_for_channel(state, channel).await else {
        return SendOutcome::refused(format!(
            "no active row in channel_gateway_connections for channel {channel}; the message was not sent"
        ));
    };

    if dry_run {
        return SendOutcome {
            ok: false,
            attempted: false,
            gateway: Some(row.name.clone()),
            provider: Some(row.provider.clone()),
            detail: format!(
                "dry run: would have sent {} characters to {target} through {} ({})",
                body.chars().count(),
                row.name,
                row.provider
            ),
            provider_message_id: None,
        };
    }

    let probe = send_text_via_gateway_row(state, &row, target, body).await;
    let ok = probe.get("ok").and_then(|v| v.as_bool()).unwrap_or(false);
    let attempted = probe
        .get("live_call_made")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let detail = probe
        .get("refusal")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .or_else(|| probe.get("detail").and_then(|v| v.as_str()).map(|s| s.to_string()))
        .unwrap_or_else(|| {
            // "HTTP 0" would read like a status code the provider sent.
            match probe.get("http_status").and_then(|v| v.as_i64()) {
                Some(code) => format!("provider returned HTTP {code}"),
                None => "the provider response carried no ok flag, no refusal and no status code"
                    .to_string(),
            }
        });
    SendOutcome {
        ok,
        attempted,
        gateway: Some(row.name.clone()),
        provider: Some(row.provider.clone()),
        detail,
        provider_message_id: probe
            .get("remote_id")
            .and_then(|v| v.as_str())
            .filter(|s| !s.trim().is_empty())
            .map(|s| s.to_string()),
    }
}

/// One outbound message row, written with the state the provider's answer
/// proves -- mirroring `services::messaging`'s `is_accepted()`/
/// `message_status()`: `sent` only when a live call was made and answered
/// 2xx, `failed` when the provider was reached and refused, `not_sent` when
/// nothing left the process. The caller's `sent` count claims a `sent` row
/// exists, so a failed INSERT is returned as a reason for the run's notes
/// instead of being logged and forgotten; returns `true` when the row landed.
async fn record_outbound(
    state: &AppState,
    conversation_id: &str,
    channel: &str,
    body: &str,
    outcome: &SendOutcome,
    intent: &str,
    sender_name: &str,
    notes: &mut Vec<String>,
) -> bool {
    let accepted = outcome.ok && outcome.attempted;
    let status = if accepted {
        "sent"
    } else if outcome.attempted {
        "failed"
    } else {
        "not_sent"
    };
    let now = now_iso();
    let sent_at = if accepted { Some(now.clone()) } else { None };
    let metadata = json!({
        "sendStatus": status,
        "liveCallMade": outcome.attempted,
        "detail": outcome.detail,
        "gateway": outcome.gateway,
    })
    .to_string();
    let insert = sqlx::query(
        "INSERT INTO messages (id, conversation_id, sender, sender_name, direction, channel, body, status, delivery_status, provider, provider_message_id, intent, provider_metadata, sent_at, created_at) VALUES (?, ?, 'bot', ?, 'outbound', ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(Uuid::new_v4().simple().to_string())
    .bind(conversation_id)
    .bind(sender_name)
    .bind(channel)
    .bind(body)
    .bind(status)
    .bind(status)
    .bind(outcome.provider.as_deref())
    .bind(outcome.provider_message_id.as_deref())
    .bind(intent)
    .bind(&metadata)
    .bind(&sent_at)
    .bind(&now)
    .execute(&state.db)
    .await;
    match write_failure(insert, "INSERT INTO messages (the outbound message row)") {
        None => {
            if !accepted {
                notes.push(format!(
                    "conversation {conversation_id}: messages row stored with status '{status}' \
                     (delivery_status '{status}') -- reason: {}",
                    outcome.detail
                ));
            }
            true
        }
        Some(reason) => {
            if accepted {
                notes.push(format!(
                    "{reason}: the provider accepted this send but conversation {conversation_id} \
                     has no outbound message row"
                ));
            } else {
                notes.push(format!(
                    "{reason}: conversation {conversation_id} has no '{status}' row carrying this \
                     reason: {}",
                    outcome.detail
                ));
            }
            false
        }
    }
}

/// The candidate rows the runners share: a conversation, where to reach it, and
/// the body to send.
#[derive(Debug, Clone, Default, sqlx::FromRow)]
struct Candidate {
    conversation_id: String,
    channel: String,
    /// The source row's own state: `conversations.state` for follow-ups and
    /// nudges, the `status` column for approval/draft candidates. Reported in
    /// every result row so the caller can see which state the dispatch was made
    /// from rather than inferring it afterwards.
    state_name: String,
    /// Which table `item_id` names. The drafts runner dispatches from two
    /// tables at once, and a `status = 'sent'` write has to go to the one the
    /// candidate was actually read from.
    #[sqlx(default)]
    source: Option<String>,
    target: Option<String>,
    contact_name: Option<String>,
    #[sqlx(default)]
    next_followup_at: Option<String>,
    #[sqlx(default)]
    sequence_step: i64,
    #[sqlx(default)]
    last_inbound_at: Option<String>,
    #[sqlx(default)]
    item_id: Option<String>,
    #[sqlx(default)]
    body: Option<String>,
}

// ---------------------------------------------------------------------------
// 1. Follow-up sequences
// ---------------------------------------------------------------------------

/// Resolves the step body for a conversation. Only usable when exactly one
/// active sequence exists, because this schema's `conversations` table has no
/// `sequence_id` column to pin a conversation to a specific one -- with two or
/// more active sequences the assignment is genuinely ambiguous and saying so is
/// better than picking one at random.
async fn sequence_step_body(
    state: &AppState,
    step_index: usize,
    name: Option<&str>,
) -> (Option<String>, Option<i64>, Vec<String>) {
    let mut notes = Vec::new();
    let sequences: Vec<(String, String)> = match sqlx::query_as(
        "SELECT id, name FROM follow_up_sequences WHERE is_active = 1 ORDER BY created_at ASC",
    )
    .fetch_all(&state.db)
    .await
    {
        Ok(rows) => rows,
        // A failed read must not be reported as "there are 0 active sequences",
        // which is a different fact than not knowing.
        Err(err) => {
            notes.push(format!(
                "follow_up_sequences could not be read ({err}); no step body could be resolved and \
                 nothing was sent"
            ));
            return (None, None, notes);
        }
    };

    if sequences.len() != 1 {
        notes.push(format!(
            "{} active follow_up_sequences row(s): without conversations.sequence_id the conversation \
             cannot be attributed to one of them, so no step body was composed",
            sequences.len()
        ));
        return (None, None, notes);
    }
    let (sequence_id, sequence_name) = sequences.into_iter().next().unwrap();

    let steps: Vec<(i64, i64, Option<String>)> = match sqlx::query_as(
        "SELECT step_number, delay_days, template FROM sequence_steps WHERE sequence_id = ? ORDER BY step_number ASC",
    )
    .bind(&sequence_id)
    .fetch_all(&state.db)
    .await
    {
        Ok(rows) => rows,
        Err(err) => {
            notes.push(format!(
                "sequence_steps for sequence {sequence_name} could not be read ({err}); nothing was \
                 sent for this conversation"
            ));
            return (None, None, notes);
        }
    };

    if steps.is_empty() {
        notes.push(format!(
            "sequence {sequence_name} has no sequence_steps rows; nothing to send"
        ));
        return (None, None, notes);
    }

    let Some((_, _, template)) = steps.get(step_index) else {
        // Sequence exhausted for this conversation.
        return (None, None, notes);
    };
    let Some(raw) = template.as_ref().filter(|t| !t.trim().is_empty()) else {
        notes.push(format!(
            "step {} of sequence {sequence_name} has an empty template; refusing to invent copy",
            step_index + 1
        ));
        return (None, None, notes);
    };

    // A contact with no stored name gets the template's own words with the
    // name token emptied -- never a substitute name like "there", which would
    // be an invented personalization token.
    let name = name.unwrap_or("");
    let body = raw.replace("{{1}}", name).replace("{{name}}", name);
    let next_delay = steps.get(step_index + 1).map(|(_, days, _)| *days);
    (Some(body), next_delay, notes)
}

async fn run_followups_inner(state: &AppState, opts: &RunQuery, tenant_key: &str) -> Value {
    let started = Utc::now();
    let mut notes = ensure_schema(state).await;
    let dry_run = opts.dry_run();
    notes.push(format!(
        "this run is scoped to tenant {tenant_key}: conversations of other tenants were not \
         considered"
    ));

    let rows: Vec<Candidate> = match sqlx::query_as(
        r#"
        SELECT c.id AS conversation_id,
               COALESCE(c.channel, '') AS channel,
               COALESCE(c.state, '') AS state_name,
               COALESCE(ct.value, c.contact_phone, c.phone) AS target,
               COALESCE(ct.person_name, c.contact_name, c.customer_name) AS contact_name,
               c.next_followup_at AS next_followup_at,
               COALESCE(c.sequence_step, 0) AS sequence_step
        FROM conversations c
        LEFT JOIN contacts ct ON ct.id = c.contact_id
        WHERE c.next_followup_at IS NOT NULL AND c.next_followup_at <> ''
          AND COALESCE(c.human_takeover, 0) = 0
          AND COALESCE(c.state, '') IN ('new', 'awaiting_reply', 'engaged')
          AND COALESCE(c.tenant_key, '') = ?
        ORDER BY c.next_followup_at ASC
        LIMIT ?
        "#,
    )
    .bind(tenant_key)
    .bind(opts.max() as i64)
    .fetch_all(&state.db)
    .await
    {
        Ok(rows) => rows,
        // Reporting `candidates: 0, status: "idle"` would claim nothing is due
        // when all that happened is that the due list could not be read.
        Err(err) => {
            let detail = format!("follow-up candidates could not be read: {err}");
            notes.push(detail.clone());
            let persisted = finish_run(
                state,
                "followups",
                started,
                0,
                0,
                0,
                "error",
                &detail,
                dry_run,
                &mut notes,
            )
            .await;
            return json!({
                "jobKey": "followups",
                "tenantKey": tenant_key,
                "candidates": 0,
                "sent": 0,
                "failed": 0,
                "readFailed": true,
                "executed": false,
                "results": [],
                "notes": notes,
                "status": "error",
                "detail": detail,
                "schedule": persisted,
            });
        }
    };

    let due: Vec<Candidate> = rows
        .iter()
        .filter(|c| parse_ts(c.next_followup_at.as_deref()).map(|t| t <= started).unwrap_or(false))
        .cloned()
        .collect();

    if due.is_empty() {
        let persisted = finish_run(state, "followups", started, 0, 0, 0, "idle", "no conversation had next_followup_at at or before this instant", dry_run, &mut notes).await;
        return json!({
            "jobKey": "followups",
            "tenantKey": tenant_key,
            "candidates": 0,
            "sent": 0,
            "failed": 0,
            "executed": false,
            "results": [],
            "notes": if notes.is_empty() { Value::Array(vec![]) } else { json!(notes) },
            "status": "idle",
            "schedule": persisted,
        });
    }

    let (open, window_note) = working_hours(started);
    if !open && !opts.force() {
        notes.push(window_note.clone());
        let persisted = finish_run(state, "followups", started, due.len(), 0, 0, "deferred", &window_note, dry_run, &mut notes).await;
        return json!({
            "jobKey": "followups",
            "tenantKey": tenant_key,
            "candidates": due.len(),
            "sent": 0,
            "failed": 0,
            "executed": false,
            "deferred": true,
            "results": [],
            "notes": notes,
            "status": "deferred",
            "schedule": persisted,
        });
    }
    notes.push(window_note);

    let mut results = Vec::new();
    let mut sent = 0usize;
    let mut failed = 0usize;
    let mut record_failures = 0usize;
    let mut messages_recorded = 0usize;

    for candidate in &due {
        let name = candidate
            .contact_name
            .as_deref()
            .filter(|n| !n.trim().is_empty());
        let index = candidate.sequence_step.max(0) as usize;
        let (body, next_delay, mut step_notes) = sequence_step_body(state, index, name).await;
        notes.append(&mut step_notes);

        let Some(body) = body else {
            failed += 1;
            results.push(json!({
                "conversationId": candidate.conversation_id,
                "ok": false,
                "liveCallMade": false,
                "step": index + 1,
                "detail": "no step body could be resolved for this conversation",
            }));
            continue;
        };

        let outcome = attempt_send(state, &candidate.channel, candidate.target.as_deref().unwrap_or(""), &body, dry_run).await;
        let accepted = outcome.ok && outcome.attempted;

        // Outside a dry run every attempt leaves a row behind -- `sent`,
        // `failed` or `not_sent` -- so the conversation shows why nothing
        // arrived, not just the successes.
        if !dry_run {
            let recorded = record_outbound(
                state,
                &candidate.conversation_id,
                &candidate.channel,
                &body,
                &outcome,
                "followup",
                "followup_scheduler",
                &mut notes,
            )
            .await;
            if recorded {
                messages_recorded += 1;
            } else {
                record_failures += 1;
            }
        }

        if accepted {
            sent += 1;
            // Only a delivered step advances the schedule.
            let next = match next_delay {
                Some(days) => Some((started + ChronoDuration::days(days)).to_rfc3339()),
                None => None,
            };
            let advance = write_failure(
                sqlx::query(
                    "UPDATE conversations SET sequence_step = COALESCE(sequence_step, 0) + 1, next_followup_at = ?, last_outbound_at = ?, updated_at = ? WHERE id = ?",
                )
                .bind(&next)
                .bind(started.to_rfc3339())
                .bind(started.to_rfc3339())
                .bind(&candidate.conversation_id)
                .execute(&state.db)
                .await,
                "UPDATE conversations SET sequence_step / next_followup_at",
            );
            match advance {
                Some(reason) => {
                    record_failures += 1;
                    notes.push(format!(
                        "{reason}: conversation {} was sent but its schedule did not advance, so the \
                         next run will send this step again",
                        candidate.conversation_id
                    ));
                }
                None => {
                    if next.is_none() {
                        notes.push(format!(
                            "conversation {} finished its sequence: next_followup_at cleared",
                            candidate.conversation_id
                        ));
                    }
                }
            }
        } else {
            failed += 1;
        }

        let mut row = outcome.json(candidate.target.as_deref().unwrap_or(""), &body);
        row["conversationId"] = json!(candidate.conversation_id);
        row["stateName"] = json!(candidate.state_name);
        row["state_name"] = json!(candidate.state_name);
        row["step"] = json!(index + 1);
        results.push(row);
    }

    let status = if sent > 0 && failed == 0 && record_failures == 0 {
        "sent"
    } else if sent > 0 {
        "partial"
    } else if dry_run {
        "dry_run"
    } else {
        "error"
    };
    let mut detail = format!("{sent} of {} follow-up(s) accepted by the provider", due.len());
    if record_failures > 0 {
        detail.push_str(&format!(
            "; {record_failures} write(s) behind those sends did not land"
        ));
    }
    let persisted = finish_run(state, "followups", started, due.len(), sent, failed, status, &detail, dry_run, &mut notes).await;

    json!({
        "jobKey": "followups",
        "tenantKey": tenant_key,
        "candidates": due.len(),
        "sent": sent,
        "failed": failed,
        "recordFailures": record_failures,
        "messagesRecorded": messages_recorded,
        "messages_recorded": messages_recorded,
        "executed": sent > 0,
        "dryRun": dry_run,
        "results": results,
        "notes": notes,
        "status": status,
        "detail": detail,
        "schedule": persisted,
    })
}

pub async fn run_followups(
    State(state): State<AppState>,
    Tenant(tenant_key): Tenant,
    Query(opts): Query<RunQuery>,
) -> impl IntoResponse {
    Json(run_followups_inner(&state, &opts, &tenant_key).await)
}

// ---------------------------------------------------------------------------
// 2. Auto-approved drafts
// ---------------------------------------------------------------------------

async fn dispatch_drafts_inner(state: &AppState, opts: &RunQuery, tenant_key: &str) -> Value {
    let started = Utc::now();
    let mut notes = ensure_schema(state).await;
    let dry_run = opts.dry_run();
    notes.push(format!(
        "this run is scoped to tenant {tenant_key}: approval and draft rows of other tenants were \
         not considered"
    ));

    // `drafts` is read by routes/drafts.rs but has no DDL in db/mod.rs, so on a
    // fresh database the table genuinely does not exist. Report it rather than
    // letting the query fail silently and returning "0 drafts".
    let drafts_table = table_exists(state, "drafts").await;
    match drafts_table {
        Some(false) => notes.push(
            "table `drafts` is missing (no CREATE TABLE in db/mod.rs); only copilot_approvals rows were considered"
                .to_string(),
        ),
        // Not being able to ask the question is not an answer that the table is
        // absent, so the response says which of the two happened.
        None => notes.push(
            "table `drafts` could not be checked (the information_schema read failed); only copilot_approvals rows were considered"
                .to_string(),
        ),
        Some(true) => {}
    }

    let approvals: Vec<Candidate> = match sqlx::query_as(
        r#"
        SELECT a.id AS item_id,
               a.conversation_id AS conversation_id,
               COALESCE(c.channel, '') AS channel,
               COALESCE(a.status, '') AS state_name,
               'copilot_approval' AS source,
               COALESCE(ct.value, c.contact_phone, c.phone) AS target,
               COALESCE(ct.person_name, c.contact_name) AS contact_name,
               COALESCE(NULLIF(a.amended_reply, ''), NULLIF(a.proposed_reply, '')) AS body
        FROM copilot_approvals a
        LEFT JOIN conversations c ON c.id = a.conversation_id
        LEFT JOIN contacts ct ON ct.id = c.contact_id
        WHERE a.status = 'approved'
          AND COALESCE(a.dispatched_message_id, '') = ''
          AND COALESCE(c.tenant_key, '') = ?
        ORDER BY a.updated_at ASC
        LIMIT ?
        "#,
    )
    .bind(tenant_key)
    .bind(opts.max() as i64)
    .fetch_all(&state.db)
    .await
    {
        Ok(rows) => rows,
        Err(err) => {
            let detail = format!("approved copilot rows could not be read: {err}");
            notes.push(detail.clone());
            let persisted = finish_run(
                state,
                "auto_approved_drafts",
                started,
                0,
                0,
                0,
                "error",
                &detail,
                dry_run,
                &mut notes,
            )
            .await;
            return json!({
                "jobKey": "auto_approved_drafts",
                "tenantKey": tenant_key,
                "candidates": 0,
                "sent": 0,
                "failed": 0,
                "readFailed": true,
                "executed": false,
                "results": [],
                "notes": notes,
                "status": "error",
                "detail": detail,
                "schedule": persisted,
            });
        }
    };

    let mut candidates = approvals;
    if drafts_table == Some(true) {
        let extra: Vec<Candidate> = match sqlx::query_as(
            r#"
            SELECT d.id AS item_id,
                   d.conversation_id AS conversation_id,
                   COALESCE(c.channel, '') AS channel,
                   COALESCE(d.status, '') AS state_name,
                   'draft' AS source,
                   COALESCE(ct.value, c.contact_phone, c.phone) AS target,
                   COALESCE(ct.person_name, c.contact_name) AS contact_name,
                   d.body AS body
            FROM drafts d
            LEFT JOIN conversations c ON c.id = d.conversation_id
            LEFT JOIN contacts ct ON ct.id = c.contact_id
            WHERE d.status IN ('auto_approved', 'approved')
              AND COALESCE(c.tenant_key, '') = ?
            ORDER BY d.created_at ASC
            LIMIT ?
            "#,
        )
        .bind(tenant_key)
        .bind(opts.max() as i64)
        .fetch_all(&state.db)
        .await
        {
            Ok(rows) => rows,
            // The approvals list is still real, so this run continues -- but the
            // drafts half of it is unknown, and saying so beats reporting only
            // the approvals as if that were everything awaiting dispatch.
            Err(err) => {
                notes.push(format!(
                    "drafts awaiting auto-dispatch could not be read ({err}); only copilot_approvals \
                     rows were considered by this run"
                ));
                Vec::new()
            }
        };
        candidates.extend(extra);
    }

    if candidates.is_empty() {
        let persisted = finish_run(state, "auto_approved_drafts", started, 0, 0, 0, "idle", "nothing awaiting auto-dispatch", dry_run, &mut notes).await;
        return json!({
            "jobKey": "auto_approved_drafts",
            "tenantKey": tenant_key,
            "candidates": 0,
            "sent": 0,
            "failed": 0,
            "executed": false,
            "results": [],
            "notes": if notes.is_empty() { Value::Array(vec![]) } else { json!(notes) },
            "status": "idle",
            "schedule": persisted,
        });
    }

    let mut results = Vec::new();
    let mut sent = 0usize;
    let mut failed = 0usize;
    let mut record_failures = 0usize;
    let mut messages_recorded = 0usize;

    for candidate in &candidates {
        let body = candidate.body.clone().unwrap_or_default();
        let outcome = attempt_send(state, &candidate.channel, candidate.target.as_deref().unwrap_or(""), &body, dry_run).await;
        let accepted = outcome.ok && outcome.attempted;

        // Outside a dry run the attempt leaves a row behind whatever the
        // provider answered, so the conversation shows the reason nothing
        // arrived instead of only the successes.
        if !dry_run {
            let recorded = record_outbound(
                state,
                &candidate.conversation_id,
                &candidate.channel,
                &body,
                &outcome,
                "auto_approved",
                "AI Copilot",
                &mut notes,
            )
            .await;
            if recorded {
                messages_recorded += 1;
            } else {
                record_failures += 1;
            }
        }

        if accepted {
            sent += 1;
            let now = started.to_rfc3339();
            match candidate.item_id.as_deref().filter(|id| !id.trim().is_empty()) {
                // Without the row id there is nothing to mark, and an unmarked
                // row is dispatched again by the next run.
                None => {
                    record_failures += 1;
                    notes.push(format!(
                        "a dispatch candidate from '{}' carried no row id: the provider accepted the \
                         send for conversation {} but no source row can be marked dispatched",
                        candidate.source.as_deref().unwrap_or("unknown"),
                        candidate.conversation_id
                    ));
                }
                Some(item_id) => {
                    // A row that stops looking dispatchable is what keeps a
                    // re-run from sending the same reply twice, so the marker
                    // goes to the table the candidate was actually read from.
                    // `dispatched_message_id` holds a real provider id or
                    // NULL -- never a synthetic stand-in that reads like one.
                    // Re-dispatch is prevented by `status = 'sent'` below, not
                    // by this column.
                    let provider_id = outcome.provider_message_id.as_deref();
                    if provider_id.is_none() {
                        notes.push(format!(
                            "row {item_id}: the provider accepted the send but returned no message \
                             id, so dispatched_message_id is stored as NULL"
                        ));
                    }
                    let mark = match candidate.source.as_deref() {
                        Some("draft") => write_failure(
                            sqlx::query("UPDATE drafts SET status = 'sent' WHERE id = ?")
                                .bind(item_id)
                                .execute(&state.db)
                                .await,
                            "UPDATE drafts SET status = 'sent'",
                        ),
                        Some("copilot_approval") => write_failure(
                            sqlx::query(
                                "UPDATE copilot_approvals SET status = 'sent', dispatched_message_id = ?, resolved_at = ?, updated_at = ? WHERE id = ?",
                            )
                            .bind(provider_id)
                            .bind(&now)
                            .bind(&now)
                            .bind(item_id)
                            .execute(&state.db)
                            .await,
                            "UPDATE copilot_approvals SET status = 'sent'",
                        ),
                        Some(other) => Some(format!(
                            "the candidate came from an unrecognised source table '{other}'"
                        )),
                        None => Some("the candidate named no source table".to_string()),
                    };
                    if let Some(reason) = mark {
                        record_failures += 1;
                        notes.push(format!(
                            "{reason}: row {item_id} was sent but is still marked dispatchable, so a \
                             later run will send it again"
                        ));
                    }
                }
            }
        } else {
            failed += 1;
            if !outcome.attempted && !dry_run {
                notes.push(format!(
                    "draft {} stayed approved and unsent",
                    candidate.item_id.as_deref().unwrap_or("<no row id>")
                ));
            }
        }

        let mut row = outcome.json(candidate.target.as_deref().unwrap_or(""), &body);
        row["draftId"] = json!(candidate.item_id);
        row["source"] = json!(candidate.source);
        row["conversationId"] = json!(candidate.conversation_id);
        row["stateName"] = json!(candidate.state_name);
        row["state_name"] = json!(candidate.state_name);
        results.push(row);
    }

    let status = if sent > 0 && failed == 0 && record_failures == 0 {
        "sent"
    } else if sent > 0 {
        "partial"
    } else if dry_run {
        "dry_run"
    } else {
        "error"
    };
    let mut detail = format!("{sent} of {} draft(s) actually transmitted", candidates.len());
    if record_failures > 0 {
        detail.push_str(&format!(
            "; {record_failures} write(s) behind those sends did not land"
        ));
    }
    let persisted = finish_run(
        state,
        "auto_approved_drafts",
        started,
        candidates.len(),
        sent,
        failed,
        status,
        &detail,
        dry_run,
        &mut notes,
    )
    .await;

    json!({
        "jobKey": "auto_approved_drafts",
        "tenantKey": tenant_key,
        "candidates": candidates.len(),
        "sent": sent,
        "failed": failed,
        "recordFailures": record_failures,
        "messagesRecorded": messages_recorded,
        "messages_recorded": messages_recorded,
        "executed": sent > 0,
        "dryRun": dry_run,
        "results": results,
        "notes": notes,
        "status": status,
        "detail": detail,
        "schedule": persisted,
    })
}

pub async fn dispatch_auto_approved(
    State(state): State<AppState>,
    Tenant(tenant_key): Tenant,
    Query(opts): Query<RunQuery>,
) -> impl IntoResponse {
    Json(dispatch_drafts_inner(&state, &opts, &tenant_key).await)
}

// ---------------------------------------------------------------------------
// 3. 24-hour inactivity nudges
// ---------------------------------------------------------------------------

/// Renders the operator-configured `INACTIVITY_NUDGE_BODY` template. There is
/// no fallback copy: an unsent message body is never invented, and the caller
/// only reaches this with a real template. A contact with no stored name gets
/// the name token emptied, not replaced with a substitute.
fn nudge_body(template: &str, name: Option<&str>) -> String {
    let name = name.unwrap_or("");
    template.replace("{{name}}", name).replace("{{1}}", name)
}

async fn run_nudges_inner(state: &AppState, opts: &RunQuery, tenant_key: &str) -> Value {
    let started = Utc::now();
    let mut notes = ensure_schema(state).await;
    let dry_run = opts.dry_run();
    let window_start = started - ChronoDuration::seconds(23 * 3600 + 1800);
    let window_end = started - ChronoDuration::hours(18);
    notes.push(format!(
        "this run is scoped to tenant {tenant_key}: conversations of other tenants were not \
         considered"
    ));

    let rows: Vec<Candidate> = match sqlx::query_as(
        r#"
        SELECT c.id AS conversation_id,
               COALESCE(c.channel, '') AS channel,
               COALESCE(c.state, '') AS state_name,
               COALESCE(ct.value, c.contact_phone, c.phone) AS target,
               COALESCE(ct.person_name, c.contact_name, c.customer_name) AS contact_name,
               c.last_inbound_at AS last_inbound_at
        FROM conversations c
        LEFT JOIN contacts ct ON ct.id = c.contact_id
        WHERE COALESCE(c.human_takeover, 0) = 0
          AND c.last_inbound_at IS NOT NULL AND c.last_inbound_at <> ''
          AND COALESCE(c.state, '') IN ('engaged', 'awaiting_reply')
          AND COALESCE(c.tenant_key, '') = ?
        ORDER BY c.last_inbound_at ASC
        LIMIT ?
        "#,
    )
    .bind(tenant_key)
    .bind(opts.max() as i64)
    .fetch_all(&state.db)
    .await
    {
        Ok(rows) => rows,
        Err(err) => {
            let detail = format!("inactivity-nudge candidates could not be read: {err}");
            notes.push(detail.clone());
            let persisted = finish_run(
                state,
                "inactivity_nudges",
                started,
                0,
                0,
                0,
                "error",
                &detail,
                dry_run,
                &mut notes,
            )
            .await;
            return json!({
                "jobKey": "inactivity_nudges",
                "tenantKey": tenant_key,
                "nudged": 0,
                "candidates": 0,
                "conversationIds": [],
                "readFailed": true,
                "windowStart": window_start.to_rfc3339(),
                "windowEnd": window_end_iso(window_end),
                "executed": false,
                "results": [],
                "notes": notes,
                "status": "error",
                "detail": detail,
                "schedule": persisted,
            });
        }
    };

    let mut due: Vec<Candidate> = Vec::new();
    for row in rows {
        let Some(inbound) = parse_ts(row.last_inbound_at.as_deref()) else {
            notes.push(format!(
                "conversation {} has an unreadable last_inbound_at ({:?}); skipped",
                row.conversation_id, row.last_inbound_at
            ));
            continue;
        };
        if inbound < window_start || inbound > window_end {
            continue;
        }
        // Do not nag twice inside the same session window. The dedup key is
        // the row's `intent`, because `provider` now holds the real provider
        // name; a `not_sent` row carries no `sent_at` and so never suppresses
        // a later genuine attempt.
        let already = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM messages WHERE conversation_id = ? AND intent = 'inactivity_nudge' AND sent_at > ?",
        )
        .bind(&row.conversation_id)
        .bind(window_start.to_rfc3339())
        .fetch_one(&state.db)
        .await;
        match already {
            // A count that could not be read is not a zero: nudging anyway risks
            // the second message this check exists to prevent.
            Err(err) => notes.push(format!(
                "conversation {}: the earlier-nudge check could not be read ({err}); skipped rather \
                 than risking a second nudge in one session window",
                row.conversation_id
            )),
            Ok(0) => due.push(row),
            Ok(_) => {}
        }
    }

    if due.is_empty() {
        let persisted = finish_run(state, "inactivity_nudges", started, 0, 0, 0, "idle", "no conversation sat in the 18-23.5 hour inbound window", dry_run, &mut notes).await;
        return json!({
            "jobKey": "inactivity_nudges",
            "tenantKey": tenant_key,
            "nudged": 0,
            "conversationIds": [],
            "windowStart": window_start.to_rfc3339(),
            "windowEnd": window_end_iso(window_end),
            "executed": false,
            "notes": if notes.is_empty() { Value::Array(vec![]) } else { json!(notes) },
            "status": "idle",
            "schedule": persisted,
        });
    }

    // An unsent message body is never invented: with no configured nudge copy
    // the runner refuses for every candidate, records each one as not sent,
    // and says which env var has to be set.
    let Some(template) = env_value("INACTIVITY_NUDGE_BODY") else {
        let reason = "no nudge body configured: set INACTIVITY_NUDGE_BODY";
        let mut results = Vec::new();
        let mut messages_recorded = 0usize;
        let mut record_failures = 0usize;
        for candidate in &due {
            if !dry_run {
                let outcome = SendOutcome::refused(reason);
                if record_outbound(
                    state,
                    &candidate.conversation_id,
                    &candidate.channel,
                    "",
                    &outcome,
                    "inactivity_nudge",
                    "inactivity_nudge",
                    &mut notes,
                )
                .await
                {
                    messages_recorded += 1;
                } else {
                    record_failures += 1;
                }
            }
            results.push(json!({
                "conversationId": candidate.conversation_id,
                "stateName": candidate.state_name,
                "state_name": candidate.state_name,
                "lastInboundAt": candidate.last_inbound_at,
                "ok": false,
                "liveCallMade": false,
                "detail": reason,
            }));
        }
        notes.push(format!(
            "{reason}; {} candidate(s) recorded as not sent and nothing was dispatched",
            due.len()
        ));
        let detail = format!("{reason}: {} in-window conversation(s) left un-nudged", due.len());
        let persisted = finish_run(
            state,
            "inactivity_nudges",
            started,
            due.len(),
            0,
            due.len(),
            "error",
            &detail,
            dry_run,
            &mut notes,
        )
        .await;
        return json!({
            "jobKey": "inactivity_nudges",
            "tenantKey": tenant_key,
            "nudged": 0,
            "candidates": due.len(),
            "failed": due.len(),
            "recordFailures": record_failures,
            "messagesRecorded": messages_recorded,
            "messages_recorded": messages_recorded,
            "conversationIds": [],
            "executed": false,
            "dryRun": dry_run,
            "windowStart": window_start.to_rfc3339(),
            "windowEnd": window_end_iso(window_end),
            "results": results,
            "notes": notes,
            "warnings": [reason],
            "status": "error",
            "detail": detail,
            "schedule": persisted,
        });
    };

    let mut results = Vec::new();
    let mut ids = Vec::new();
    let mut sent = 0usize;
    let mut failed = 0usize;
    let mut record_failures = 0usize;
    let mut messages_recorded = 0usize;

    for candidate in &due {
        let name = candidate
            .contact_name
            .as_deref()
            .filter(|n| !n.trim().is_empty());
        let body = nudge_body(&template, name);
        let outcome = attempt_send(state, &candidate.channel, candidate.target.as_deref().unwrap_or(""), &body, dry_run).await;
        let accepted = outcome.ok && outcome.attempted;

        if !dry_run {
            let recorded = record_outbound(
                state,
                &candidate.conversation_id,
                &candidate.channel,
                &body,
                &outcome,
                "inactivity_nudge",
                "inactivity_nudge",
                &mut notes,
            )
            .await;
            if recorded {
                messages_recorded += 1;
            } else {
                record_failures += 1;
            }
        }

        if accepted {
            sent += 1;
            ids.push(json!(candidate.conversation_id));
            // Auxiliary only: the nudge itself is stored in `messages`, and the
            // earlier-nudge check reads that table, so a missed timestamp bump
            // cannot cause a second nudge.
            report_write(
                sqlx::query("UPDATE conversations SET last_outbound_at = ?, updated_at = ? WHERE id = ?")
                    .bind(started.to_rfc3339())
                    .bind(started.to_rfc3339())
                    .bind(&candidate.conversation_id)
                    .execute(&state.db)
                    .await
                    .map(|report| report.rows_affected()),
                "UPDATE conversations SET last_outbound_at (inactivity nudge)",
            );
        } else {
            failed += 1;
        }

        let mut row = outcome.json(candidate.target.as_deref().unwrap_or(""), &body);
        row["conversationId"] = json!(candidate.conversation_id);
        row["stateName"] = json!(candidate.state_name);
        row["state_name"] = json!(candidate.state_name);
        row["lastInboundAt"] = json!(candidate.last_inbound_at);
        results.push(row);
    }

    let status = if sent > 0 && failed == 0 && record_failures == 0 {
        "sent"
    } else if sent > 0 {
        "partial"
    } else if dry_run {
        "dry_run"
    } else {
        "error"
    };
    let mut detail = format!("{sent} of {} in-window conversation(s) nudged for real", due.len());
    if record_failures > 0 {
        detail.push_str(&format!(
            "; {record_failures} write(s) behind those nudges did not land"
        ));
    }
    let persisted = finish_run(state, "inactivity_nudges", started, due.len(), sent, failed, status, &detail, dry_run, &mut notes).await;

    json!({
        "jobKey": "inactivity_nudges",
        "tenantKey": tenant_key,
        "nudged": sent,
        "candidates": due.len(),
        "failed": failed,
        "recordFailures": record_failures,
        "messagesRecorded": messages_recorded,
        "messages_recorded": messages_recorded,
        "conversationIds": ids,
        "executed": sent > 0,
        "dryRun": dry_run,
        "windowStart": window_start.to_rfc3339(),
        "windowEnd": window_end_iso(window_end),
        "results": results,
        "notes": notes,
        "status": status,
        "detail": detail,
        "schedule": persisted,
    })
}

fn window_end_iso(value: DateTime<Utc>) -> String {
    value.to_rfc3339()
}

pub async fn run_nudges(
    State(state): State<AppState>,
    Tenant(tenant_key): Tenant,
    Query(opts): Query<RunQuery>,
) -> impl IntoResponse {
    Json(run_nudges_inner(&state, &opts, &tenant_key).await)
}

// ---------------------------------------------------------------------------
// 4. Scheduled campaigns
// ---------------------------------------------------------------------------

async fn process_campaigns_inner(state: &AppState, opts: &RunQuery, tenant_key: &str) -> Value {
    let started = Utc::now();
    let mut notes = ensure_schema(state).await;
    let dry_run = opts.dry_run();

    // The tenant comes from the request's X-Tenant-Key header (or the
    // deployment's DEFAULT_TENANT_KEY via the `Tenant` extractor). Guessing
    // one would risk dispatching a different tenant's campaign list.
    let tenant_key = tenant_key.trim();
    if tenant_key.is_empty() {
        let detail = "the campaign sweep was given no tenant, so the scheduler cannot tell whose \
                      campaigns are due; nothing was dispatched."
            .to_string();
        notes.push(detail.clone());
        let persisted = finish_run(state, "campaigns", started, 0, 0, 0, "error", &detail, dry_run, &mut notes).await;
        return json!({
            "jobKey": "campaigns",
            "dispatched": 0,
            "campaigns": [],
            "executed": false,
            "notes": json!(notes),
            "status": "error",
            "detail": detail,
            "schedule": persisted,
        });
    }

    let rows = match sqlx::query_as::<_, (String, String, Option<String>)>(
        "SELECT id, name, scheduled_at FROM broadcasts \
         WHERE status = 'scheduled' AND COALESCE(tenant_key, '') = ? \
         AND scheduled_at IS NOT NULL AND scheduled_at <> '' \
         ORDER BY scheduled_at ASC LIMIT ?",
    )
    .bind(tenant_key)
    .bind(opts.max() as i64)
    .fetch_all(&state.db)
    .await
    {
        Ok(rows) => rows,
        Err(err) => {
            let detail = format!("the due campaigns could not be read: {err}");
            notes.push(detail.clone());
            let persisted =
                finish_run(state, "campaigns", started, 0, 0, 0, "error", &detail, dry_run, &mut notes).await;
            return json!({
                "jobKey": "campaigns",
                "tenantKey": tenant_key,
                "dispatched": 0,
                "campaigns": [],
                "readFailed": true,
                "executed": false,
                "notes": json!(notes),
                "status": "error",
                "detail": detail,
                "schedule": persisted,
            });
        }
    };

    let due: Vec<(String, String, Option<String>)> = rows
        .into_iter()
        .filter(|(_, _, scheduled)| parse_ts(scheduled.as_deref()).map(|t| t <= started).unwrap_or(false))
        .collect();

    if due.is_empty() {
        let persisted = finish_run(state, "campaigns", started, 0, 0, 0, "idle", "no broadcast had scheduled_at at or before this instant", dry_run, &mut notes).await;
        return json!({
            "jobKey": "campaigns",
            "tenantKey": tenant_key,
            "dispatched": 0,
            "campaigns": [],
            "executed": false,
            "notes": if notes.is_empty() { Value::Array(vec![]) } else { json!(notes) },
            "status": "idle",
            "schedule": persisted,
        });
    }

    let mut campaigns = Vec::new();
    let mut dispatched = 0usize;
    let mut failed = 0usize;

    for (id, name, scheduled_at) in &due {
        if dry_run {
            campaigns.push(json!({
                "id": id,
                "name": name,
                "scheduledAt": scheduled_at,
                "ok": false,
                "liveCallMade": false,
                "detail": "dry run: recipient dispatch not invoked",
            }));
            continue;
        }

        // The dispatch itself lives in routes/broadcasts.rs and is the one code
        // path the operator's Dispatch button uses, so the scheduler calls it and
        // reads the outcome it returns instead of re-deriving it from the table.
        match super::broadcasts::run_dispatch(state, tenant_key, id).await {
            Ok(body) => {
                let count = |key: &str| body.get(key).and_then(Value::as_i64);
                let sent = count("sentCount");
                match sent {
                    Some(value) if value > 0 => dispatched += 1,
                    Some(_) => {
                        failed += 1;
                        if let Some(reason) = body
                            .get("warnings")
                            .and_then(Value::as_array)
                            .and_then(|items| items.first())
                            .and_then(Value::as_str)
                        {
                            notes.push(format!("campaign {name} sent nothing: {reason}"));
                        }
                    }
                    None => {
                        failed += 1;
                        notes.push(format!(
                            "campaign {name} was dispatched but its response carried no sentCount"
                        ));
                    }
                }
                campaigns.push(json!({
                    "id": id,
                    "name": name,
                    "scheduledAt": scheduled_at,
                    "statusAfter": body.get("status").and_then(Value::as_str),
                    "sent": sent,
                    "delivered": count("deliveredCount"),
                    "read": count("readCount"),
                    "failed": count("failedCount"),
                    "skipped": count("skippedCount"),
                    "ok": sent.unwrap_or(0) > 0,
                    // The dispatcher answers `not_sent` for recipients it never
                    // reached, so a live call is only claimed when something was
                    // actually sent or failed at the provider.
                    "liveCallMade": sent.unwrap_or(0) > 0
                        || count("failedCount").unwrap_or(0) > 0,
                    "warnings": body.get("warnings").cloned().unwrap_or_else(|| json!([])),
                }));
            }
            Err((status, body)) => {
                failed += 1;
                notes.push(format!(
                    "campaign {name} was not dispatched (HTTP {}): {}",
                    status.as_u16(),
                    body.get("error")
                        .and_then(Value::as_str)
                        .unwrap_or("the dispatcher returned no reason")
                ));
                campaigns.push(json!({
                    "id": id,
                    "name": name,
                    "scheduledAt": scheduled_at,
                    "ok": false,
                    "liveCallMade": false,
                    "httpStatus": status.as_u16(),
                    "detail": body.get("error").cloned().unwrap_or(Value::Null),
                }));
            }
        }
    }

    let status = if dispatched > 0 && failed == 0 {
        "dispatched"
    } else if dispatched > 0 {
        "partial"
    } else if dry_run {
        "dry_run"
    } else {
        "error"
    };
    let detail = format!("{dispatched} of {} due campaign(s) reported sends", due.len());
    let persisted = finish_run(state, "campaigns", started, due.len(), dispatched, failed, status, &detail, dry_run, &mut notes).await;

    json!({
        "jobKey": "campaigns",
        "tenantKey": tenant_key,
        "dispatched": dispatched,
        "candidates": due.len(),
        "failed": failed,
        "campaigns": campaigns,
        "executed": dispatched > 0,
        "dryRun": dry_run,
        "notes": notes,
        "status": status,
        "detail": detail,
        "schedule": persisted,
    })
}

pub async fn process_campaigns(
    State(state): State<AppState>,
    Tenant(tenant_key): Tenant,
    Query(opts): Query<RunQuery>,
) -> impl IntoResponse {
    Json(process_campaigns_inner(&state, &opts, &tenant_key).await)
}

// ---------------------------------------------------------------------------
// Unified run + schedule surface
// ---------------------------------------------------------------------------

/// POST /scheduler/run-all -- replaces the fixed-zero response the admin module
/// returns for the same path.
pub async fn run_all_schedulers(
    State(state): State<AppState>,
    Tenant(tenant_key): Tenant,
    Query(opts): Query<RunQuery>,
) -> impl IntoResponse {
    let schema_notes = ensure_schema(&state).await;
    let followups = run_followups_inner(&state, &opts, &tenant_key).await;
    let drafts = dispatch_drafts_inner(&state, &opts, &tenant_key).await;
    let nudges = run_nudges_inner(&state, &opts, &tenant_key).await;
    let campaigns = process_campaigns_inner(&state, &opts, &tenant_key).await;

    let mut notes: Vec<String> = schema_notes;
    notes.extend(
        [&followups, &drafts, &nudges, &campaigns]
            .iter()
            .flat_map(|v| {
                v.get("notes")
                    .and_then(|n| n.as_array())
                    .cloned()
                    .unwrap_or_default()
                    .into_iter()
                    .filter_map(|n| n.as_str().map(|s| s.to_string()))
            }),
    );

    let read_failed = [&followups, &drafts, &nudges, &campaigns]
        .iter()
        .any(|v| v.get("readFailed").and_then(Value::as_bool).unwrap_or(false));

    Json(json!({
        "tenantKey": tenant_key,
        "followupsSent": followups["sent"].as_i64().unwrap_or(0),
        "followupsCandidates": followups["candidates"].as_i64().unwrap_or(0),
        "draftsAutoSent": drafts["sent"].as_i64().unwrap_or(0),
        "draftsCandidates": drafts["candidates"].as_i64().unwrap_or(0),
        "inactivityNudges": nudges["nudged"].as_i64().unwrap_or(0),
        "nudgesCandidates": nudges["candidates"].as_i64().unwrap_or(0),
        "campaignsDispatched": campaigns["dispatched"].as_i64().unwrap_or(0),
        "campaignsCandidates": campaigns["candidates"].as_i64().unwrap_or(0),
        "dryRun": opts.dry_run(),
        // A runner whose candidate list could not be read contributes nothing to
        // these counts, so the response says which half of the run is unknown.
        "readFailed": read_failed,
        "executed": followups["sent"].as_i64().unwrap_or(0)
            + drafts["sent"].as_i64().unwrap_or(0)
            + nudges["nudged"].as_i64().unwrap_or(0)
            + campaigns["dispatched"].as_i64().unwrap_or(0)
            > 0,
        "notes": notes,
        "jobs": { "followups": followups, "autoApprovedDrafts": drafts, "inactivityNudges": nudges, "campaigns": campaigns },
    }))
}

/// GET /scheduler/jobs -- the real schedule table, with defaults materialised
/// so the UI shows the cadence that will actually be used.
pub async fn list_jobs(State(state): State<AppState>) -> Response {
    let mut notes = ensure_schema(&state).await;
    let now = Utc::now();
    let mut jobs = Vec::new();
    for (key, _, _) in JOBS {
        match ensure_job(&state, *key).await {
            Ok(job) => jobs.push(job.json(now)),
            Err(reason) => {
                notes.push(reason.clone());
                jobs.push(json!({
                    "jobKey": *key,
                    "job_key": *key,
                    "registered": false,
                    "scheduleKnown": false,
                    "detail": reason,
                }));
            }
        }
    }
    let runs: Vec<Value> = match sqlx::query_as::<_, RunRow>(
        "SELECT id, job_key, started_at, finished_at, duration_ms, candidates, executed, failed, status, detail, dry_run FROM scheduler_runs ORDER BY started_at DESC LIMIT 20",
    )
    .fetch_all(&state.db)
    .await
    {
        Ok(rows) => rows.iter().map(|r| r.json()).collect(),
        Err(err) => {
            notes.push(format!(
                "scheduler_runs history could not be read ({err}); recentRuns is empty because the \
                 read failed, not because no run has happened"
            ));
            Vec::new()
        }
    };

    Json(json!({
        "jobs": jobs,
        "recentRuns": runs,
        "inProcessTicker": false,
        "notes": {
            "ticker": "no background loop is registered in main.rs: this surface is driven by an external cron/queue hitting the POST endpoints",
            "schemaNotes": notes,
        },
    }))
    .into_response()
}

/// GET /scheduler/jobs/{key}
pub async fn get_job(State(state): State<AppState>, Path(key): Path<String>) -> Response {
    let known = JOBS.iter().any(|(k, _, _)| *k == key);
    let mut notes = ensure_schema(&state).await;
    let mut unreadable = false;
    let job = match load_job(&state, &key).await {
        Ok(Some(row)) => Some(row),
        Ok(None) if known => match ensure_job(&state, &key).await {
            Ok(job) => Some(job),
            Err(reason) => {
                notes.push(reason);
                unreadable = true;
                None
            }
        },
        // Ok(None) on an unknown key is the real "no such job"; Err means the
        // read failed and answering 404 would claim a fact we could not check.
        Ok(None) => None,
        Err(err) => {
            notes.push(format!(
                "the scheduler_jobs row for {key} could not be read ({err})"
            ));
            unreadable = true;
            None
        }
    };
    let Some(job) = job else {
        if unreadable {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(json!({
                    "error": format!("the schedule state of {key} could not be read"),
                    "detail": "scheduler_jobs is not answerable; whether this job exists is unknown",
                    "notes": notes,
                })),
            )
                .into_response();
        }
        return (
            StatusCode::NOT_FOUND,
            Json(json!({
                "error": format!("no scheduler job named {key}"),
                "knownJobs": JOBS.iter().map(|(k, _, _)| *k).collect::<Vec<_>>(),
            })),
        )
            .into_response();
    };

    let runs: Vec<Value> = match sqlx::query_as::<_, RunRow>(
        "SELECT id, job_key, started_at, finished_at, duration_ms, candidates, executed, failed, status, detail, dry_run FROM scheduler_runs WHERE job_key = ? ORDER BY started_at DESC LIMIT 20",
    )
    .bind(&key)
    .fetch_all(&state.db)
    .await
    {
        Ok(rows) => rows.iter().map(|r| r.json()).collect(),
        Err(err) => {
            notes.push(format!(
                "scheduler_runs history for {key} could not be read ({err}); recentRuns is empty \
                 because the read failed, not because no run has happened"
            ));
            Vec::new()
        }
    };

    let next = parse_ts(job.next_run_at.as_deref());
    Json(
        json!({
            "job": job.json(Utc::now()),
            "secondsUntilNextRun": next.map(|n| (n - Utc::now()).num_seconds()),
            "recentRuns": runs,
            "notes": notes,
        }),
    )
        .into_response()
}

#[derive(Debug, Clone, sqlx::FromRow)]
struct RunRow {
    id: String,
    job_key: String,
    started_at: String,
    finished_at: Option<String>,
    duration_ms: i64,
    candidates: i64,
    executed: i64,
    failed: i64,
    status: String,
    detail: String,
    dry_run: Option<bool>,
}

impl RunRow {
    fn json(&self) -> Value {
        json!({
            "id": self.id,
            "jobKey": self.job_key,
            "startedAt": self.started_at,
            "finishedAt": self.finished_at,
            "durationMs": self.duration_ms,
            "candidates": self.candidates,
            "executed": self.executed,
            "failed": self.failed,
            "status": self.status,
            "detail": self.detail,
            "dryRun": self.dry_run.unwrap_or(false),
        })
    }
}

#[derive(Debug, Default, Deserialize)]
pub struct SchedulePayload {
    #[serde(default, alias = "intervalSeconds")]
    pub interval_seconds: Option<i64>,
    #[serde(default, alias = "intervalMinutes")]
    pub interval_minutes: Option<i64>,
    #[serde(default, alias = "intervalHours")]
    pub interval_hours: Option<i64>,
    #[serde(default, alias = "isEnabled")]
    pub is_enabled: Option<bool>,
    pub cron: Option<String>,
    #[serde(default, alias = "runNow")]
    pub run_now: Option<bool>,
}

/// PUT /scheduler/jobs/{key} -- persists the cadence and recomputes
/// `next_run_at` with chrono. A cron expression is refused rather than stored
/// and then ignored, because no cron parser is in Cargo.toml.
pub async fn update_job(
    State(state): State<AppState>,
    Path(key): Path<String>,
    Json(payload): Json<SchedulePayload>,
) -> Response {
    if !JOBS.iter().any(|(k, _, _)| *k == key) {
        return (
            StatusCode::NOT_FOUND,
            Json(json!({
                "error": format!("no scheduler job named {key}"),
                "knownJobs": JOBS.iter().map(|(k, _, _)| *k).collect::<Vec<_>>(),
            })),
        )
            .into_response();
    }

    let mut notes = ensure_schema(&state).await;
    if payload.cron.as_deref().map(|c| !c.trim().is_empty()).unwrap_or(false) {
        notes.push(
            "cron expressions are not parsed (no cron crate in Cargo.toml); set interval_seconds/minutes/hours instead"
                .to_string(),
        );
    }

    let resolved = payload
        .interval_seconds
        .map(|v| v.max(30))
        .or_else(|| payload.interval_minutes.map(|v| v.max(1) * 60))
        .or_else(|| payload.interval_hours.map(|v| v.max(1) * 3600));

    let job = match ensure_job(&state, &key).await {
        Ok(job) => job,
        Err(reason) => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(json!({
                    "error": "the current schedule could not be read or registered",
                    "detail": reason,
                    "notes": notes,
                })),
            )
                .into_response();
        }
    };
    let interval = resolved.unwrap_or(job.interval_seconds.max(30));
    let enabled = payload.is_enabled.unwrap_or(job.is_enabled);
    let now = Utc::now();
    let next = if enabled {
        Some((now + ChronoDuration::seconds(interval)).to_rfc3339())
    } else {
        None
    };

    let result = sqlx::query(
        "UPDATE scheduler_jobs SET interval_seconds = ?, is_enabled = ?, next_run_at = ?, updated_at = ? WHERE job_key = ?",
    )
    .bind(interval)
    .bind(enabled)
    .bind(&next)
    .bind(now.to_rfc3339())
    .bind(&key)
    .execute(&state.db)
    .await;

    match result {
        Ok(report) if report.rows_affected() == 1 => {
            let mut body = match ensure_job(&state, &key).await {
                Ok(saved) => json!({
                    "status": "saved",
                    "persisted": saved.id.as_str() != "",
                    "job": saved.json(now),
                    "notes": notes,
                }),
                Err(reason) => {
                    notes.push(format!(
                        "{reason}: the UPDATE reported one row changed, but the stored schedule \
                         could not be read back to confirm it"
                    ));
                    json!({
                        "status": "saved",
                        "persisted": false,
                        "job": null,
                        "notes": notes,
                    })
                }
            };
            if payload.run_now.unwrap_or(false) {
                body["runNowNotSupported"] = json!(
                    "this endpoint only stores the schedule; run the matching POST /scheduler/* endpoint to execute it"
                );
            }
            Json(body).into_response()
        }
        // The statement succeeded but matched no row: answering "saved" would
        // claim a cadence change that the database does not hold.
        Ok(report) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": "the schedule was not stored",
                "detail": format!(
                    "UPDATE scheduler_jobs SET interval_seconds/is_enabled for {key} affected {} rows (expected 1)",
                    report.rows_affected()
                ),
                "notes": notes,
            })),
        )
            .into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": "could not persist the schedule",
                "detail": err.to_string(),
                "notes": notes,
            })),
        )
            .into_response(),
    }
}

/// GET /scheduler/status -- what a schedule page needs before trusting the
/// numbers: is the schema present, is a ticker running, is dry-run on, are the
/// gateways the runners depend on configured at all. The conversation,
/// broadcast and approval counts are scoped to the request's tenant: they are
/// read from `conversations.tenant_key` / `broadcasts.tenant_key`, and
/// `copilot_approvals` (which has no tenant column of its own) through its
/// conversation.
pub async fn scheduler_status(
    State(state): State<AppState>,
    Tenant(tenant_key): Tenant,
) -> Response {
    let mut notes = ensure_schema(&state).await;
    let now = Utc::now();

    let gateway_count = measured(
        sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM channel_gateway_connections WHERE is_active = 1",
        )
        .fetch_one(&state.db)
        .await,
        "the active gateway count",
        &mut notes,
    );
    let due_followups = measured(
        sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM conversations WHERE next_followup_at IS NOT NULL AND next_followup_at <> '' AND COALESCE(human_takeover, 0) = 0 AND COALESCE(tenant_key, '') = ?",
        )
        .bind(&tenant_key)
        .fetch_one(&state.db)
        .await,
        "the count of conversations with a follow-up scheduled",
        &mut notes,
    );
    let due_campaigns = measured(
        sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM broadcasts WHERE status = 'scheduled' AND scheduled_at IS NOT NULL AND scheduled_at <> '' AND COALESCE(tenant_key, '') = ?",
        )
        .bind(&tenant_key)
        .fetch_one(&state.db)
        .await,
        "the count of scheduled broadcasts",
        &mut notes,
    );
    let pending_drafts = measured(
        sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM copilot_approvals a \
             LEFT JOIN conversations c ON c.id = a.conversation_id \
             WHERE a.status = 'approved' AND COALESCE(a.dispatched_message_id, '') = '' \
             AND COALESCE(c.tenant_key, '') = ?",
        )
        .bind(&tenant_key)
        .fetch_one(&state.db)
        .await,
        "the count of approved drafts awaiting dispatch",
        &mut notes,
    );

    let jobs: Vec<Value> = {
        let mut out = Vec::new();
        for (key, _, _) in JOBS {
            match ensure_job(&state, *key).await {
                Ok(job) => out.push(job.json(now)),
                Err(reason) => {
                    notes.push(reason.clone());
                    out.push(json!({
                        "jobKey": *key,
                        "job_key": *key,
                        "registered": false,
                        "scheduleKnown": false,
                        "detail": reason,
                    }));
                }
            }
        }
        out
    };

    let (open, window) = working_hours(now);

    let mut warnings: Vec<String> = notes;
    if gateway_count == Some(0) {
        warnings.push(
            "no active channel_gateway_connections row: every runner will refuse to send until a gateway is configured"
                .to_string(),
        );
    }
    if matches!(due_followups, Some(count) if count > 0) && !open {
        warnings.push(format!("follow-ups are outside the working-hours window ({window})"));
    }
    match table_exists(&state, "drafts").await {
        Some(false) => warnings.push("table `drafts` has no DDL in db/mod.rs; auto-approved dispatch reads copilot_approvals only".to_string()),
        None => warnings.push("the existence of table `drafts` could not be checked (the information_schema read failed)".to_string()),
        Some(true) => {}
    }
    warnings.push(format!(
        "the conversation, broadcast and approval counts are scoped to tenant {tenant_key}; \
         rows of other tenants are not included"
    ));

    Json(json!({
        "serverTime": now.to_rfc3339(),
        "tenantKey": tenant_key,
        "inProcessTicker": false,
        "dryRun": RunQuery::default().dry_run(),
        "workingHours": { "open": open, "detail": window },
        "counts": {
            "conversationsWithFollowupScheduled": due_followups,
            "campaignsScheduled": due_campaigns,
            "approvedDraftsAwaitingDispatch": pending_drafts,
            "activeGateways": gateway_count
        },
        "jobs": jobs,
        "warnings": warnings,
    }))
        .into_response()
}
