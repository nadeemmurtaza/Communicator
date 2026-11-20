//! Services & Sequences — bounded context for service catalog and follow-up sequences.
//!
//! Covers two domains:
//! - **Services**: institution service catalog with CRUD operations
//! - **Sequences**: follow-up sequences with A/B variant tracking

use sqlx::PgPool;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use chrono::Utc;
use uuid::Uuid;
use tracing::error;

// ---------------------------------------------------------------------------
// State
// ---------------------------------------------------------------------------

#[derive(Clone)]
pub struct ServicesSequencesState {
    pub pool: PgPool,
}

impl ServicesSequencesState {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

async fn table_columns(pool: &PgPool, table: &str) -> Vec<String> {
    sqlx::query_scalar::<_, String>(
        "SELECT column_name FROM information_schema.columns WHERE table_schema='public' AND table_name = $1"
    )
        .bind(table)
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

// ---------------------------------------------------------------------------
// Services
// ---------------------------------------------------------------------------

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

pub async fn list_services(state: &ServicesSequencesState, tenant_key: &str) -> Result<Value, String> {
    let services = sqlx::query_as::<_, ServiceItem>(
        "SELECT id, name, COALESCE(summary, '') AS summary, target_institution_types FROM services WHERE COALESCE(tenant_key, '') = $1 ORDER BY name ASC",
    )
    .bind(tenant_key)
    .fetch_all(&state.pool)
    .await
    .map_err(|e| format!("could not read the services table: {e}"))?;

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

    Ok(json!(result))
}

pub async fn get_service(state: &ServicesSequencesState, id: &str) -> Result<Value, String> {
    let row = sqlx::query_as::<_, ServiceFullRow>(&format!(
        "SELECT {SERVICE_COLUMNS} FROM services WHERE id = $1 LIMIT 1"
    ))
    .bind(id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| format!("Service lookup failed: {e}"))?;

    match row {
        Some(s) => Ok(service_json(&s)),
        None => Err(format!("Service '{}' does not exist in the services table.", id)),
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
    state: &ServicesSequencesState,
    payload: &CreateServicePayload,
    tenant_key: &str,
) -> Result<Value, String> {
    let name = payload.name.trim();
    if name.is_empty() {
        return Err("Service name must not be empty.".to_string());
    }

    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();
    let targets = payload
        .target_institution_types
        .clone()
        .or(payload.target_institution_types_camel.clone())
        .unwrap_or_default();
    let targets_json = serde_json::to_string(&targets).unwrap_or_else(|_| "[]".to_string());
    let faq = payload.faq.clone().unwrap_or_default();
    let faq_json = serde_json::to_string(&faq).unwrap_or_else(|_| "[]".to_string());
    let pricing = payload.pricing_notes.clone().unwrap_or_default();

    sqlx::query(
        "INSERT INTO services (id, name, summary, details, target_institution_types, faq, pricing_notes, created_at, updated_at, tenant_key) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)",
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
    .bind(tenant_key)
    .execute(&state.pool)
    .await
    .map_err(|e| format!("Service could not be written to the services table: {e}"))?;

    Ok(json!({
        "id": id,
        "name": name,
        "summary": payload.summary.as_deref().unwrap_or(""),
        "details": payload.details.as_deref().unwrap_or(""),
        "targetInstitutionTypes": targets,
        "faq": faq,
        "pricingNotes": pricing,
        "createdAt": now,
    }))
}

// ---------------------------------------------------------------------------
// Sequences
// ---------------------------------------------------------------------------

#[derive(Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct SeqRow {
    pub id: String,
    pub name: String,
    pub is_active: bool,
}

pub async fn list_sequences(state: &ServicesSequencesState) -> Result<Value, String> {
    let seqs = sqlx::query_as::<_, SeqRow>(
        "SELECT id, name, is_active FROM follow_up_sequences ORDER BY name ASC",
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|e| format!("could not read the follow_up_sequences table: {e}"))?;

    Ok(serde_json::to_value(&seqs).unwrap_or_else(|_| json!([])))
}

#[derive(Deserialize)]
pub struct CreateSequencePayload {
    pub name: String,
    pub description: Option<String>,
    pub is_active: Option<bool>,
}

pub async fn create_sequence(
    state: &ServicesSequencesState,
    payload: &CreateSequencePayload,
) -> Result<Value, String> {
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();
    let is_active = payload.is_active.unwrap_or(true);
    let name = payload.name.trim().to_string();
    if name.is_empty() {
        return Err("Sequence name must not be empty.".to_string());
    }

    sqlx::query(
        "INSERT INTO follow_up_sequences (id, name, description, is_active, created_at) VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(&id)
    .bind(&name)
    .bind(&payload.description)
    .bind(is_active)
    .bind(&now)
    .execute(&state.pool)
    .await
    .map_err(|e| format!("Sequence could not be written to follow_up_sequences: {e}"))?;

    Ok(json!({
        "id": id,
        "name": name,
        "description": payload.description,
        "isActive": is_active,
        "createdAt": now,
        "stepCount": 0,
        "note": "Sequence row written; add steps via sequence_steps before enrolling conversations.",
    }))
}

pub async fn sequence_variant_stats(
    state: &ServicesSequencesState,
    id: &str,
) -> Result<Value, String> {
    let seq: Option<(String, String)> =
        sqlx::query_as("SELECT id, name FROM follow_up_sequences WHERE id = $1 LIMIT 1")
            .bind(id)
            .fetch_optional(&state.pool)
            .await
            .unwrap_or(None);

    let Some((seq_id, seq_name)) = seq else {
        return Err(format!("Sequence '{}' does not exist in follow_up_sequences.", id));
    };

    let declared: Vec<Option<String>> = sqlx::query_scalar(
        "SELECT variant FROM sequence_steps WHERE sequence_id = $1 GROUP BY variant",
    )
    .bind(&seq_id)
    .fetch_all(&state.pool)
    .await
    .unwrap_or_default();

    let cols = table_columns(&state.pool, "conversations").await;
    let has_link = cols.iter().any(|c| c == "sequence_id") && cols.iter().any(|c| c == "sequence_variant");

    if !has_link {
        return Ok(json!({
            "sequenceId": seq_id,
            "sequenceName": seq_name,
            "declaredVariants": declared.iter().map(|v| v.clone().unwrap_or_else(|| "(no variant)".to_string())).collect::<Vec<String>>(),
            "variants": [],
            "error": "Enrolment columns are missing: conversations.sequence_id and conversations.sequence_variant are not present in this database, so no real per-variant counts exist.",
            "requiredDdl": [
                "ALTER TABLE conversations ADD COLUMN sequence_id TEXT",
                "ALTER TABLE conversations ADD COLUMN sequence_variant TEXT"
            ],
            "note": "Refusing to report invented variant statistics. Apply the DDL above and enrol conversations via POST /conversations/{id}/enroll-sequence.",
        }));
    }

    let rows: Vec<(String, i64, i64, i64)> = sqlx::query_as(
        "SELECT COALESCE(sequence_variant, '(no variant)') AS variant, \
                COUNT(*) AS enrolled, \
                COALESCE(SUM(CASE WHEN UPPER(state) IN ('ENGAGED','HOT_LEAD','HANDED_OFF','CLOSED_WON','CLOSED_LOST') OR last_inbound_at IS NOT NULL THEN 1 ELSE 0 END), 0) AS replied, \
                COALESCE(SUM(CASE WHEN UPPER(state) = 'CLOSED_WON' OR resolution_type = 'won' THEN 1 ELSE 0 END), 0) AS closed_won \
         FROM conversations WHERE sequence_id = $1 GROUP BY COALESCE(sequence_variant, '(no variant)')",
    )
    .bind(&seq_id)
    .fetch_all(&state.pool)
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

    Ok(json!({
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
    }))
}
