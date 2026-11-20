//! Liveness / readiness health check — bounded context.
//!
//! Pings PostgreSQL on every call and reports reachability, version,
//! round-trip latency, and inbox count.

use sqlx::PgPool;
use serde_json::json;
use chrono::Utc;
use sqlx::Row;
use tracing::warn;

/// Application state for the health service.
#[derive(Clone)]
pub struct HealthState {
    pub pool: PgPool,
}

impl HealthState {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

/// Perform a liveness/readiness check against PostgreSQL.
pub async fn health_check(state: &HealthState) -> serde_json::Value {
    let started = std::time::Instant::now();

    match sqlx::query_scalar::<_, String>("SELECT version() AS version")
        .fetch_optional(&state.pool)
        .await
    {
        Ok(Some(version)) => {
            let checked = sqlx::query("SELECT COUNT(*) AS inboxes FROM inboxes")
                .fetch_optional(&state.pool)
                .await
                .ok()
                .and_then(|r| r.and_then(|r| r.try_get::<i64, _>("inboxes").ok()));

            json!({
                "status": "ok",
                "database": {
                    "reachable": true,
                    "engine": "postgresql",
                    "version": version,
                    "roundTripMs": started.elapsed().as_millis() as i64,
                    "inboxCount": checked,
                },
                "checkedAt": Utc::now().to_rfc3339(),
            })
        }
        Ok(None) => {
            warn!("Health probe: the database returned no row for a liveness query");
            json!({
                "status": "degraded",
                "database": {
                    "reachable": false,
                    "error": "The database pool answered with an empty result set.",
                },
                "checkedAt": Utc::now().to_rfc3339(),
            })
        }
        Err(err) => {
            warn!("Health probe: database unreachable: {err}");
            json!({
                "status": "degraded",
                "database": {
                    "reachable": false,
                    "error": err.to_string(),
                },
                "checkedAt": Utc::now().to_rfc3339(),
            })
        }
    }
}
