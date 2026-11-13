//! Audience-Based Knowledge Scoping API routes (Phase 26).

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;
use serde_json::json;
use sqlx::Row;

use crate::db::AppState;
use crate::services::audience_kb_scoping;

// ——— Query Params ———

#[derive(Deserialize)]
pub struct ListLinksQuery {
    segment_id: Option<String>,
}

#[derive(Deserialize)]
pub struct ListMembershipsQuery {
    segment_id: Option<String>,
    contact_id: Option<String>,
    limit: Option<i32>,
}

#[derive(Deserialize)]
pub struct ScopedKBQuery {
    contact_id: String,
}

// ——— Request Bodies ———

#[derive(Deserialize)]
pub struct LinkKBBody {
    segment_id: String,
    kb_entry_id: String,
    priority: Option<i32>,
}

#[derive(Deserialize)]
pub struct AddMembershipBody {
    segment_id: String,
    contact_id: String,
    membership_type: Option<String>,
}

#[derive(Deserialize)]
pub struct UpdateSettingsBody {
    enabled: Option<bool>,
    fallback_to_global_kb: Option<bool>,
    merge_segments: Option<bool>,
    default_scope: Option<String>,
}

#[derive(Deserialize)]
pub struct LogUsageBody {
    conversation_id: String,
    contact_id: String,
    segment_id: String,
    kb_entry_id: String,
    query_text: Option<String>,
    relevance_score: Option<f64>,
    used_in_response: Option<bool>,
}

// ——— Endpoints ———

/// GET /audience-kb/summary — Dashboard summary
pub async fn get_summary(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let summary = audience_kb_scoping::get_audience_summary(&state)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(serde_json::to_value(summary).unwrap_or_default()))
}

/// GET /audience-kb/settings — Get settings
pub async fn get_settings(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let settings = audience_kb_scoping::get_audience_kb_settings(&state).await;
    match settings {
        Some(s) => Ok(Json(serde_json::to_value(s).unwrap_or_default())),
        None => Ok(Json(json!({"enabled": true, "fallback_to_global_kb": true, "merge_segments": true, "default_scope": "all"}))),
    }
}

/// PUT /audience-kb/settings — Update settings
pub async fn update_settings(
    State(state): State<AppState>,
    Json(body): Json<UpdateSettingsBody>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let settings = audience_kb_scoping::update_audience_kb_settings(
        &state,
        body.enabled,
        body.fallback_to_global_kb,
        body.merge_segments,
        body.default_scope.as_deref(),
    )
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(serde_json::to_value(settings).unwrap_or_default()))
}

/// GET /audience-kb/links — List KB links for a segment
pub async fn list_links(
    State(state): State<AppState>,
    Query(params): Query<ListLinksQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    if let Some(segment_id) = params.segment_id {
        let links = audience_kb_scoping::get_segment_kb_links(&state, &segment_id)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        Ok(Json(json!({"links": links, "segment_id": segment_id})))
    } else {
        // Return all links grouped by segment
        let rows = sqlx::query(
            "SELECT aul.id, aul.segment_id, cs.name as segment_name, \
             aul.kb_entry_id, kbe.title as kb_title, aul.priority, aul.created_at \
             FROM audience_kb_links aul \
             LEFT JOIN contact_segments cs ON aul.segment_id = cs.id \
             LEFT JOIN knowledge_base_entries kbe ON aul.kb_entry_id = kbe.id \
             ORDER BY aul.segment_id, aul.priority DESC"
        )
        .fetch_all(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        let links: Vec<serde_json::Value> = rows
            .iter()
            .map(|row| {
                json!({
                    "id": row.try_get::<String, _>(0).unwrap_or_default(),
                    "segment_id": row.try_get::<String, _>(1).unwrap_or_default(),
                    "segment_name": row.try_get::<Option<String>, _>(2).ok(),
                    "kb_entry_id": row.try_get::<String, _>(3).unwrap_or_default(),
                    "kb_title": row.try_get::<Option<String>, _>(4).ok(),
                    "priority": row.try_get::<i32, _>(5).unwrap_or(0i32),
                    "created_at": row.try_get::<String, _>(6).unwrap_or_default(),
                })
            })
            .collect();

        Ok(Json(json!({"links": links})))
    }
}

/// POST /audience-kb/links — Link KB entry to segment
pub async fn link_kb(
    State(state): State<AppState>,
    Json(body): Json<LinkKBBody>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let result = audience_kb_scoping::link_kb_to_segment(
        &state,
        &body.segment_id,
        &body.kb_entry_id,
        body.priority.unwrap_or(0),
    )
    .await
    .map_err(|e| (StatusCode::BAD_REQUEST, e))?;
    Ok(Json(serde_json::to_value(result).unwrap_or_default()))
}

/// DELETE /audience-kb/links/:segment_id/:kb_entry_id — Unlink KB from segment
pub async fn unlink_kb(
    State(state): State<AppState>,
    Path((segment_id, kb_entry_id)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    audience_kb_scoping::unlink_kb_from_segment(&state, &segment_id, &kb_entry_id)
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, e))?;
    Ok(Json(json!({"message": "KB entry unlinked from segment"})))
}

/// GET /audience-kb/scoped — Get scoped KB for a contact
pub async fn get_scoped_kb(
    State(state): State<AppState>,
    Query(params): Query<ScopedKBQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let result = audience_kb_scoping::get_scoped_kb_for_contact(&state, &params.contact_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
    Ok(Json(serde_json::to_value(result).unwrap_or_default()))
}

/// GET /audience-kb/memberships — List segment memberships
pub async fn list_memberships(
    State(state): State<AppState>,
    Query(params): Query<ListMembershipsQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let limit = params.limit.unwrap_or(50);
    if let Some(segment_id) = params.segment_id {
        let memberships = audience_kb_scoping::get_segment_memberships(&state, &segment_id)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        Ok(Json(json!({"memberships": memberships, "segment_id": segment_id})))
    } else if let Some(contact_id) = params.contact_id {
        let memberships = audience_kb_scoping::get_contact_segment_memberships(&state, &contact_id)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        Ok(Json(json!({"memberships": memberships, "contact_id": contact_id})))
    } else {
        // Return all memberships
        let rows = sqlx::query(
            "SELECT scm.id, scm.segment_id, cs.name as segment_name, \
             scm.contact_id, c.name as contact_name, scm.membership_type, scm.matched_at \
             FROM segment_contact_memberships scm \
             LEFT JOIN contact_segments cs ON scm.segment_id = cs.id \
             LEFT JOIN contacts c ON scm.contact_id = c.id \
             ORDER BY scm.matched_at DESC LIMIT $1"
        )
        .bind(limit)
        .fetch_all(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        let memberships: Vec<serde_json::Value> = rows
            .iter()
            .map(|row| {
                json!({
                    "id": row.try_get::<String, _>(0).unwrap_or_default(),
                    "segment_id": row.try_get::<String, _>(1).unwrap_or_default(),
                    "segment_name": row.try_get::<Option<String>, _>(2).ok(),
                    "contact_id": row.try_get::<String, _>(3).unwrap_or_default(),
                    "contact_name": row.try_get::<Option<String>, _>(4).ok(),
                    "membership_type": row.try_get::<String, _>(5).unwrap_or_default(),
                    "matched_at": row.try_get::<String, _>(6).unwrap_or_default(),
                })
            })
            .collect();

        Ok(Json(json!({"memberships": memberships})))
    }
}

/// POST /audience-kb/memberships — Add contact to segment
pub async fn add_membership(
    State(state): State<AppState>,
    Json(body): Json<AddMembershipBody>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let result = audience_kb_scoping::add_contact_to_segment(
        &state,
        &body.segment_id,
        &body.contact_id,
        body.membership_type.as_deref().unwrap_or("static"),
    )
    .await
    .map_err(|e| (StatusCode::BAD_REQUEST, e))?;
    Ok(Json(serde_json::to_value(result).unwrap_or_default()))
}

/// DELETE /audience-kb/memberships/:segment_id/:contact_id — Remove contact from segment
pub async fn remove_membership(
    State(state): State<AppState>,
    Path((segment_id, contact_id)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    audience_kb_scoping::remove_contact_from_segment(&state, &segment_id, &contact_id)
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, e))?;
    Ok(Json(json!({"message": "Contact removed from segment"})))
}

/// POST /audience-kb/usage-log — Log KB usage
pub async fn log_usage(
    State(state): State<AppState>,
    Json(body): Json<LogUsageBody>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    audience_kb_scoping::log_kb_usage(
        &state,
        &body.conversation_id,
        &body.contact_id,
        &body.segment_id,
        &body.kb_entry_id,
        body.query_text.as_deref().unwrap_or(""),
        body.relevance_score.unwrap_or(0.5),
        body.used_in_response.unwrap_or(false),
    )
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
    Ok(Json(json!({"message": "Usage logged"})))
}

/// GET /audience-kb/usage/:segment_id — Get usage logs for a segment
pub async fn get_usage_logs(
    State(state): State<AppState>,
    Path(segment_id): Path<String>,
    Query(params): Query<ListMembershipsQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let limit = params.limit.unwrap_or(50);
    let logs = audience_kb_scoping::get_segment_usage_logs(&state, &segment_id, limit)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(json!({"logs": logs, "segment_id": segment_id})))
}
