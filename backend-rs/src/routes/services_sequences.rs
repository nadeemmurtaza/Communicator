use crate::db::AppState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use uuid::Uuid;

use super::auth::Tenant;

async fn table_columns(pool: &sqlx::Pool<sqlx::Sqlite>, table: &str) -> Vec<String> {
    sqlx::query_scalar::<_, String>(&format!("SELECT name FROM pragma_table_info('{table}')"))
        .fetch_all(pool)
        .await
        .unwrap_or_default()
}

fn parse_string_array(raw: Option<&str>) -> Vec<String> {
    match raw {
        Some(s) => serde_json::from_str::<Vec<String>>(s).unwrap_or_default(),
        None => vec![],
    }
}

fn parse_faq(raw: Option<&str>) -> Value {
    match raw {
        Some(s) => serde_json::from_str::<Value>(s).unwrap_or_else(|_| Value::Array(vec![])),
        None => Value::Array(vec![]),
    }
}

const SERVICE_COLUMNS: &str =
    "id, name, summary, details, target_institution_types, faq, pricing_notes, created_at, updated_at";

#[derive(Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ServiceFullRow {
    pub id: String,
    pub name: String,
    pub summary: Option<String>,
    pub details: Option<String>,
    pub target_institution_types: Option<String>,
    pub faq: Option<String>,
    pub pricing_notes: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

fn service_json(s: &ServiceFullRow) -> Value {
    json!({
        "id": s.id,
        "name": s.name,
        "summary": s.summary.clone().unwrap_or_default(),
        "details": s.details.clone().unwrap_or_default(),
        "targetInstitutionTypes": parse_string_array(s.target_institution_types.as_deref()),
        "faq": parse_faq(s.faq.as_deref()),
        "pricingNotes": s.pricing_notes.clone().unwrap_or_default(),
        "createdAt": s.created_at,
        "updatedAt": s.updated_at,
    })
}

#[derive(Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ServiceItem {
    pub id: String,
    pub name: String,
    pub summary: String,
    pub target_institution_types: Option<String>,
}

pub async fn list_services(
    State(state): State<AppState>,
    Tenant(tenant): Tenant,
) -> Response {
    let services = match sqlx::query_as::<_, ServiceItem>(
        "SELECT id, name, COALESCE(summary, '') AS summary, target_institution_types FROM services WHERE COALESCE(tenant_key, '') = ? ORDER BY name ASC",
    )
    .bind(&tenant)
    .fetch_all(&state.db)
    .await
    {
        Ok(rows) => rows,
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": format!("could not read the services table: {err}") })),
            )
                .into_response()
        }
    };

    let result: Vec<Value> = services
        .into_iter()
        .map(|s| {
            json!({
                "id": s.id,
                "name": s.name,
                "summary": s.summary,
                "targetInstitutionTypes": parse_string_array(s.target_institution_types.as_deref())
            })
        })
        .collect();

    (StatusCode::OK, Json(result)).into_response()
}

/// GET /services/{service_id} -- real row or a genuine 404.
pub async fn get_service(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Response {
    let row = sqlx::query_as::<_, ServiceFullRow>(&format!(
        "SELECT {SERVICE_COLUMNS} FROM services WHERE id = ? LIMIT 1"
    ))
    .bind(&id)
    .fetch_optional(&state.db)
    .await;

    match row {
        Ok(Some(s)) => (StatusCode::OK, Json(service_json(&s))).into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": format!("Service '{id}' does not exist in the services table.") })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": format!("Service lookup failed: {e}") })),
        )
            .into_response(),
    }
}

#[derive(Deserialize)]
pub struct CreateServicePayload {
    pub name: String,
    #[serde(default)]
    pub summary: Option<String>,
    #[serde(default)]
    pub details: Option<String>,
    pub target_institution_types: Option<Vec<String>>,
    #[serde(alias = "targetInstitutionTypes")]
    pub target_institution_types_camel: Option<Vec<String>>,
    #[serde(default)]
    pub faq: Option<Vec<Value>>,
    #[serde(alias = "pricingNotes")]
    pub pricing_notes: Option<String>,
}

pub async fn create_service(
    State(state): State<AppState>,
    Tenant(tenant): Tenant,
    Json(payload): Json<CreateServicePayload>,
) -> Response {
    let name = payload.name.trim();
    if name.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Service name must not be empty." })),
        )
            .into_response();
    }

    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();
    let targets = payload
        .target_institution_types
        .or(payload.target_institution_types_camel)
        .unwrap_or_default();
    let targets_json = serde_json::to_string(&targets).unwrap_or_else(|_| "[]".to_string());
    let faq = payload.faq.unwrap_or_default();
    let faq_json = serde_json::to_string(&faq).unwrap_or_else(|_| "[]".to_string());
    let pricing = payload.pricing_notes.unwrap_or_default();

    let res = sqlx::query(
        "INSERT INTO services (id, name, summary, details, target_institution_types, faq, pricing_notes, created_at, updated_at, tenant_key) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(name)
    .bind(payload.summary.as_deref().unwrap_or(""))
    .bind(payload.details.as_deref().unwrap_or(""))
    .bind(&targets_json)
    .bind(&faq_json)
    .bind(&pricing)
    .bind(&now)
    .bind(&now)
    .bind(&tenant)
    .execute(&state.db)
    .await;

    match res {
        Ok(_) => (
            StatusCode::CREATED,
            Json(json!({
                "id": id,
                "name": name,
                "summary": payload.summary.as_deref().unwrap_or(""),
                "details": payload.details.as_deref().unwrap_or(""),
                "targetInstitutionTypes": targets,
                "faq": faq,
                "pricingNotes": pricing,
                "createdAt": now,
            })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": format!("Service could not be written to the services table: {e}") })),
        )
            .into_response(),
    }
}

#[derive(Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct SeqRow {
    pub id: String,
    pub name: String,
    pub is_active: bool,
}

pub async fn list_sequences(State(state): State<AppState>) -> Response {
    let seqs = match sqlx::query_as::<_, SeqRow>(
        "SELECT id, name, is_active FROM follow_up_sequences ORDER BY name ASC",
    )
    .fetch_all(&state.db)
    .await
    {
        Ok(rows) => rows,
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": format!("could not read the follow_up_sequences table: {err}") })),
            )
                .into_response()
        }
    };

    (
        StatusCode::OK,
        Json(serde_json::to_value(&seqs).unwrap_or_else(|_| json!([]))),
    )
        .into_response()
}

#[derive(Deserialize)]
pub struct CreateSequencePayload {
    pub name: String,
    pub description: Option<String>,
    pub is_active: Option<bool>,
}

pub async fn create_sequence(
    State(state): State<AppState>,
    Json(payload): Json<CreateSequencePayload>,
) -> impl IntoResponse {
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();
    let is_active = payload.is_active.unwrap_or(true);
    let name = payload.name.trim().to_string();
    if name.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Sequence name must not be empty." })),
        )
            .into_response();
    }

    let res = sqlx::query(
        "INSERT INTO follow_up_sequences (id, name, description, is_active, created_at) VALUES (?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(&name)
    .bind(&payload.description)
    .bind(is_active)
    .bind(&now)
    .execute(&state.db)
    .await;

    match res {
        Ok(_) => (
            StatusCode::CREATED,
            Json(json!({
                "id": id,
                "name": name,
                "description": payload.description,
                "isActive": is_active,
                "createdAt": now,
                "stepCount": 0,
                "note": "Sequence row written; add steps via sequence_steps before enrolling conversations.",
            })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": format!("Sequence could not be written to follow_up_sequences: {e}") })),
        )
            .into_response(),
    }
}

/// Per-variant A/B results, counted from real `conversations` rows only.
/// Variants declared in `sequence_steps` are reported with zero counts when no
/// conversation has been enrolled in them -- never with invented numbers.
pub async fn sequence_variant_stats(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Response {
    let seq: Option<(String, String)> =
        sqlx::query_as("SELECT id, name FROM follow_up_sequences WHERE id = ? LIMIT 1")
            .bind(&id)
            .fetch_optional(&state.db)
            .await
            .unwrap_or(None);

    let Some((seq_id, seq_name)) = seq else {
        return (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": format!("Sequence '{id}' does not exist in follow_up_sequences.") })),
        )
            .into_response();
    };

    let declared: Vec<Option<String>> = sqlx::query_scalar(
        "SELECT variant FROM sequence_steps WHERE sequence_id = ? GROUP BY variant",
    )
    .bind(&seq_id)
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    let cols = table_columns(&state.db, "conversations").await;
    let has_link = cols.iter().any(|c| c == "sequence_id") && cols.iter().any(|c| c == "sequence_variant");

    if !has_link {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({
                "sequenceId": seq_id,
                "sequenceName": seq_name,
                "declaredVariants": declared.iter().map(|v| v.clone().unwrap_or_else(|| "(no variant)".to_string())).collect::<Vec<String>>(),
                "variants": [],
                "error": "Enrolment columns are missing: conversations.sequence_id and conversations.sequence_variant are not present in this database, so no real per-variant counts exist.",
                "requiredDdl": [
                    "ALTER TABLE conversations ADD COLUMN sequence_id TEXT",
                    "ALTER TABLE conversations ADD COLUMN sequence_variant TEXT"
                ],
                "note": "Refusing to report invented variant statistics. Apply the DDL above (see WIRING NEEDED) and enrol conversations via POST /conversations/{id}/enroll-sequence.",
            })),
        )
            .into_response();
    }

    let rows: Vec<(String, i64, i64, i64)> = sqlx::query_as(
        "SELECT COALESCE(sequence_variant, '(no variant)') AS variant, \
                COUNT(*) AS enrolled, \
                COALESCE(SUM(CASE WHEN UPPER(state) IN ('ENGAGED','HOT_LEAD','HANDED_OFF','CLOSED_WON','CLOSED_LOST') OR last_inbound_at IS NOT NULL THEN 1 ELSE 0 END), 0) AS replied, \
                COALESCE(SUM(CASE WHEN UPPER(state) = 'CLOSED_WON' OR resolution_type = 'won' THEN 1 ELSE 0 END), 0) AS closed_won \
         FROM conversations WHERE sequence_id = ? GROUP BY COALESCE(sequence_variant, '(no variant)')",
    )
    .bind(&seq_id)
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    let mut variants: Vec<Value> = Vec::new();
    for (variant, enrolled, replied, closed) in &rows {
        variants.push(json!({
            "variant": variant,
            "enrolled": enrolled,
            "replied": replied,
            "closedWon": closed,
            "replyRate": if *enrolled > 0 { (*replied as f64 * 100.0 / *enrolled as f64).round() / 100.0 } else { 0.0 },
            "measured": true,
        }));
    }

    for d in &declared {
        let label = d.clone().unwrap_or_else(|| "(no variant)".to_string());
        if !variants.iter().any(|v| v["variant"].as_str() == Some(label.as_str())) {
            variants.push(json!({
                "variant": label,
                "enrolled": 0,
                "replied": 0,
                "closedWon": 0,
                "replyRate": null,
                "measured": false,
                "note": "Declared in sequence_steps but no conversation is enrolled in it yet.",
            }));
        }
    }

    let total_enrolled: i64 = variants
        .iter()
        .filter_map(|v| v["enrolled"].as_i64())
        .sum();

    (
        StatusCode::OK,
        Json(json!({
            "sequenceId": seq_id,
            "sequenceName": seq_name,
            "variants": variants,
            "totalEnrolled": total_enrolled,
            "dataSource": "conversations.sequence_id + conversations.sequence_variant",
            "note": if total_enrolled == 0 {
                "No conversation is enrolled in this sequence yet; every variant below shows real zero counts."
            } else {
                "Counts are live aggregates over enrolled conversations; reply uses state/last_inbound_at evidence."
            },
        })),
    )
        .into_response()
}
