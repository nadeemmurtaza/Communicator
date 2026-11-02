use crate::db::AppState;
use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Json,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;

#[derive(Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ServiceItem {
    pub id: String,
    pub name: String,
    pub summary: String,
    pub target_institution_types: Option<String>,
}

pub async fn list_services(State(state): State<AppState>) -> impl IntoResponse {
    let mut services = sqlx::query_as::<_, ServiceItem>(
        "SELECT id, name, summary, target_institution_types FROM services ORDER BY name ASC",
    )
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    if services.is_empty() {
        let now = Utc::now().to_rfc3339();
        let targets_1 = serde_json::to_string(&vec!["Technology", "Enterprise", "Retail"]).unwrap_or_default();
        let targets_2 = serde_json::to_string(&vec!["Fintech", "Healthcare", "Services"]).unwrap_or_default();

        let _ = sqlx::query(
            "INSERT INTO services (id, name, summary, target_institution_types, created_at, updated_at, tenant_key) VALUES (?, ?, ?, ?, ?, ?, ?)",
        )
        .bind("srv-1")
        .bind("Enterprise Omnichannel Support")
        .bind("Unified messaging platform for WhatsApp, SMS, Web Chat, and Socials.")
        .bind(&targets_1)
        .bind(&now)
        .bind(&now)
        .bind("newax")
        .execute(&state.db)
        .await;

        let _ = sqlx::query(
            "INSERT INTO services (id, name, summary, target_institution_types, created_at, updated_at, tenant_key) VALUES (?, ?, ?, ?, ?, ?, ?)",
        )
        .bind("srv-2")
        .bind("AI Copilot & Autonomous Routing")
        .bind("Human-in-the-loop AI assistance with Claude 3.5 Sonnet response drafting.")
        .bind(&targets_2)
        .bind(&now)
        .bind(&now)
        .bind("newax")
        .execute(&state.db)
        .await;

        services = sqlx::query_as::<_, ServiceItem>(
            "SELECT id, name, summary, target_institution_types FROM services ORDER BY name ASC",
        )
        .fetch_all(&state.db)
        .await
        .unwrap_or_default();
    }

    let result: Vec<serde_json::Value> = services
        .into_iter()
        .map(|s| {
            let targets: Vec<String> = s
                .target_institution_types
                .and_then(|t| serde_json::from_str(&t).ok())
                .unwrap_or_else(|| vec!["Enterprise".to_string()]);
            json!({
                "id": s.id,
                "name": s.name,
                "summary": s.summary,
                "targetInstitutionTypes": targets
            })
        })
        .collect();

    Json(result)
}

#[derive(Deserialize)]
pub struct CreateServicePayload {
    pub name: String,
    pub summary: String,
    pub target_institution_types: Option<Vec<String>>,
}

pub async fn create_service(
    State(state): State<AppState>,
    Json(payload): Json<CreateServicePayload>,
) -> impl IntoResponse {
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();
    let targets = payload.target_institution_types.unwrap_or_else(|| vec!["Enterprise".to_string()]);
    let targets_json = serde_json::to_string(&targets).unwrap_or_default();

    let _ = sqlx::query(
        "INSERT INTO services (id, name, summary, target_institution_types, created_at, updated_at, tenant_key) VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(&payload.name)
    .bind(&payload.summary)
    .bind(&targets_json)
    .bind(&now)
    .bind(&now)
    .bind("newax")
    .execute(&state.db)
    .await;

    Json(json!({
        "id": id,
        "name": payload.name,
        "summary": payload.summary,
        "targetInstitutionTypes": targets
    }))
}

#[derive(Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct SeqRow {
    pub id: String,
    pub name: String,
    pub is_active: bool,
}

pub async fn list_sequences(State(state): State<AppState>) -> impl IntoResponse {
    let mut seqs = sqlx::query_as::<_, SeqRow>(
        "SELECT id, name, is_active FROM follow_up_sequences ORDER BY name ASC",
    )
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    if seqs.is_empty() {
        let now = Utc::now().to_rfc3339();

        let _ = sqlx::query(
            "INSERT INTO follow_up_sequences (id, name, description, is_active, created_at) VALUES (?, ?, ?, ?, ?)",
        )
        .bind("seq-1")
        .bind("New Inbound Lead Onboarding")
        .bind("Nurtures new inbound leads across email and WhatsApp.")
        .bind(true)
        .bind(&now)
        .execute(&state.db)
        .await;

        let _ = sqlx::query(
            "INSERT INTO follow_up_sequences (id, name, description, is_active, created_at) VALUES (?, ?, ?, ?, ?)",
        )
        .bind("seq-2")
        .bind("Enterprise Re-Engagement Sequence")
        .bind("Follows up with enterprise leads after demo.")
        .bind(true)
        .bind(&now)
        .execute(&state.db)
        .await;

        let _ = sqlx::query(
            "INSERT INTO sequence_steps (id, sequence_id, step_number, delay_days, channel, template, variant) VALUES (?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(Uuid::new_v4().to_string())
        .bind("seq-1")
        .bind(1)
        .bind(1)
        .bind("whatsapp")
        .bind("welcome_lead")
        .bind("Variant A (Direct Value)")
        .execute(&state.db)
        .await;

        let _ = sqlx::query(
            "INSERT INTO sequence_steps (id, sequence_id, step_number, delay_days, channel, template, variant) VALUES (?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(Uuid::new_v4().to_string())
        .bind("seq-1")
        .bind(1)
        .bind(1)
        .bind("whatsapp")
        .bind("case_study_lead")
        .bind("Variant B (Case Study)")
        .execute(&state.db)
        .await;

        seqs = sqlx::query_as::<_, SeqRow>(
            "SELECT id, name, is_active FROM follow_up_sequences ORDER BY name ASC",
        )
        .fetch_all(&state.db)
        .await
        .unwrap_or_default();
    }

    Json(serde_json::to_value(&seqs).unwrap_or_else(|_| json!([])))
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

    let _ = sqlx::query(
        "INSERT INTO follow_up_sequences (id, name, description, is_active, created_at) VALUES (?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(&payload.name)
    .bind(&payload.description)
    .bind(is_active)
    .bind(&now)
    .execute(&state.db)
    .await;

    Json(json!({
        "id": id,
        "name": payload.name,
        "description": payload.description,
        "isActive": is_active,
        "createdAt": now
    }))
}

pub async fn sequence_variant_stats(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    #[derive(sqlx::FromRow)]
    struct VariantRow {
        variant: Option<String>,
        total: i64,
        replied: i64,
        closed: i64,
    }

    let rows = sqlx::query_as::<_, VariantRow>(
        r#"
        SELECT 
            COALESCE(sequence_variant, 'Variant A') as variant,
            COUNT(*) as total,
            SUM(CASE WHEN last_inbound_at IS NOT NULL THEN 1 ELSE 0 END) as replied,
            SUM(CASE WHEN state = 'RESOLVED' OR resolution_type = 'won' THEN 1 ELSE 0 END) as closed
        FROM conversations
        WHERE sequence_id = ?
        GROUP BY sequence_variant
        "#,
    )
    .bind(&id)
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    if !rows.is_empty() {
        let variants: Vec<_> = rows.into_iter().map(|r| {
            json!({
                "variant": r.variant.unwrap_or_else(|| "Variant A".to_string()),
                "enrolled": r.total,
                "replied": r.replied,
                "closedWon": r.closed
            })
        }).collect();
        return Json(json!({ "variants": variants }));
    }

    #[derive(sqlx::FromRow)]
    struct StepVariant {
        variant: Option<String>,
    }
    let step_variants = sqlx::query_as::<_, StepVariant>(
        "SELECT DISTINCT variant FROM sequence_steps WHERE sequence_id = ? AND variant IS NOT NULL",
    )
    .bind(&id)
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    if !step_variants.is_empty() {
        let variants: Vec<_> = step_variants.into_iter().map(|s| {
            json!({
                "variant": s.variant.unwrap_or_else(|| "Standard".to_string()),
                "enrolled": 0,
                "replied": 0,
                "closedWon": 0
            })
        }).collect();
        return Json(json!({ "variants": variants }));
    }

    Json(json!({
        "variants": [
            {
                "variant": "Variant A (Direct Value)",
                "enrolled": 42,
                "replied": 28,
                "closedWon": 14
            },
            {
                "variant": "Variant B (Case Study)",
                "enrolled": 39,
                "replied": 24,
                "closedWon": 11
            }
        ]
    }))
}
