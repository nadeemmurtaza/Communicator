//! Unified Inbox API routes (Phase 19).
//!
//! Provides customer-centric inbox view merging conversations across channels.

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;
use serde_json::json;

use crate::db::AppState;
use crate::services::unified_inbox;

/// Query parameters for unified inbox listing
#[derive(Deserialize)]
struct UnifiedInboxQuery {
    limit: Option<i32>,
    offset: Option<i32>,
    channel: Option<String>,
    state: Option<String>,
    search: Option<String>,
}

/// GET /unified-inbox — List customers with their conversations grouped
pub async fn list_unified_inbox(
    State(state): State<AppState>,
    Query(params): Query<UnifiedInboxQuery>,
) -> Result<axum::Json<unified_inbox::UnifiedInboxSummary>, (StatusCode, axum::Json<serde_json::Value>)> {
    let limit = params.limit.unwrap_or(20);
    let offset = params.offset.unwrap_or(0);

    match unified_inbox::get_unified_inbox_summary(
        &state,
        limit,
        offset,
        params.channel.as_deref(),
        params.state.as_deref(),
        params.search.as_deref(),
    )
    .await
    {
        Ok(summary) => Ok(Json(summary)),
        Err(err) => {
            eprintln!("Unified inbox list error: {}", err);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": format!("Failed to list unified inbox: {}", err) })),
            ))
        }
    }
}

/// GET /unified-inbox/{customer_id} — Get unified view for a specific customer
pub async fn get_unified_customer(
    State(state): State<AppState>,
    Path(customer_id): Path<String>,
) -> Result<axum::Json<unified_inbox::UnifiedInboxEntry>, (StatusCode, axum::Json<serde_json::Value>)> {
    match unified_inbox::get_unified_customer(&state, &customer_id).await {
        Ok(entry) => Ok(Json(entry)),
        Err(sqlx::Error::RowNotFound) => Err((
            StatusCode::NOT_FOUND,
            Json(json!({ "error": format!("Customer '{}' not found", customer_id) })),
        )),
        Err(err) => {
            eprintln!("Unified customer error: {}", err);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": format!("Failed to get customer: {}", err) })),
            ))
        }
    }
}

/// POST /unified-inbox/resolve — Resolve customer identity from channel + identifier
pub async fn resolve_customer(
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> Result<axum::Json<serde_json::Value>, (StatusCode, axum::Json<serde_json::Value>)> {
    let channel = payload.get("channel").and_then(|v| v.as_str()).unwrap_or("");
    let identifier = payload.get("identifier").and_then(|v| v.as_str()).unwrap_or("");

    if channel.is_empty() || identifier.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "channel and identifier are required" })),
        ));
    }

    match unified_inbox::resolve_customer(&state, channel, identifier).await {
        Ok(Some(customer)) => Ok(Json(serde_json::to_value(customer).unwrap_or_else(|_| json!({})))),
        Ok(None) => Ok(Json(json!({ "found": false, "message": "No customer found for this channel + identifier" }))),
        Err(err) => {
            eprintln!("Resolve customer error: {}", err);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": format!("Failed to resolve customer: {}", err) })),
            ))
        }
    }
}

/// POST /unified-inbox/mapping — Create manual customer mapping
pub async fn create_customer_mapping(
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> Result<axum::Json<unified_inbox::CustomerMapping>, (StatusCode, axum::Json<serde_json::Value>)> {
    let customer_id = payload.get("customer_id").and_then(|v| v.as_str()).unwrap_or("");
    let contact_id = payload.get("contact_id").and_then(|v| v.as_str()).unwrap_or("");
    let channel = payload.get("channel").and_then(|v| v.as_str()).unwrap_or("");
    let channel_identifier = payload.get("channel_identifier").and_then(|v| v.as_str()).unwrap_or("");

    if customer_id.is_empty() || contact_id.is_empty() || channel.is_empty() || channel_identifier.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "customer_id, contact_id, channel, and channel_identifier are required" })),
        ));
    }

    match unified_inbox::create_customer_mapping(
        &state, customer_id, contact_id, channel, channel_identifier,
    )
    .await
    {
        Ok(mapping) => Ok(Json(mapping)),
        Err(err) => {
            eprintln!("Create mapping error: {}", err);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": format!("Failed to create mapping: {}", err) })),
            ))
        }
    }
}

/// GET /unified-inbox/settings — Get unified inbox settings
pub async fn get_unified_inbox_settings(
    State(state): State<AppState>,
    Query(params): Query<UnifiedInboxQuery>,
) -> Result<axum::Json<unified_inbox::UnifiedInboxSettings>, (StatusCode, axum::Json<serde_json::Value>)> {
    // Get organization ID from query or use default
    let org_id = params.search.as_deref().unwrap_or("default");

    match unified_inbox::get_unified_inbox_settings(&state, org_id).await {
        Ok(settings) => Ok(Json(settings)),
        Err(err) => {
            eprintln!("Get settings error: {}", err);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": format!("Failed to get settings: {}", err) })),
            ))
        }
    }
}

/// PUT /unified-inbox/settings — Update unified inbox settings
pub async fn update_unified_inbox_settings(
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> Result<axum::Json<unified_inbox::UnifiedInboxSettings>, (StatusCode, axum::Json<serde_json::Value>)> {
    let org_id = payload.get("organization_id").and_then(|v| v.as_str()).unwrap_or("default");

    match unified_inbox::update_unified_inbox_settings(&state, org_id, &payload).await {
        Ok(settings) => Ok(Json(settings)),
        Err(err) => {
            eprintln!("Update settings error: {}", err);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": format!("Failed to update settings: {}", err) })),
            ))
        }
    }
}
