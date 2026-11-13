use crate::db::AppState;
use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use chrono::Utc;
use serde_json::json;
use sqlx::Row;
use tracing::warn;

/// Liveness/readiness probe. The database is pinged for real on every call:
/// `status` is only ever `ok` when the pool actually answered, and a failed
/// probe reports the driver error instead of an optimistic success.
pub async fn health_check(State(state): State<AppState>) -> impl IntoResponse {
    let started = std::time::Instant::now();

    match sqlx::query_scalar::<_, String>("SELECT version() AS version")
        .fetch_optional(&state.db)
        .await
    {
        Ok(Some(row)) => {
            let version: String = row;
            let checked = sqlx::query("SELECT COUNT(*) AS inboxes FROM inboxes")
                .fetch_optional(&state.db)
                .await
                .ok()
                .and_then(|r| r.and_then(|r| r.try_get::<i64, _>("inboxes").ok()));

            (
                StatusCode::OK,
                Json(json!({
                    "status": "ok",
                    "database": {
                        "reachable": true,
                        "engine": "postgresql",
                        "version": version,
                        "roundTripMs": started.elapsed().as_millis() as i64,
                        "inboxCount": checked,
                    },
                    "checkedAt": Utc::now().to_rfc3339(),
                })),
            )
                .into_response()
        }
        Ok(None) => {
            warn!("Health probe: the database returned no row for a liveness query");
            (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(json!({
                    "status": "degraded",
                    "database": {
                        "reachable": false,
                        "error": "The database pool answered with an empty result set.",
                    },
                    "checkedAt": Utc::now().to_rfc3339(),
                })),
            )
                .into_response()
        }
        Err(err) => {
            warn!("Health probe: database unreachable: {err}");
            (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(json!({
                    "status": "degraded",
                    "database": {
                        "reachable": false,
                        "error": err.to_string(),
                    },
                    "checkedAt": Utc::now().to_rfc3339(),
                })),
            )
                .into_response()
        }
    }
}
