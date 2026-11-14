//! Discovery-flow routes (WhatsApp business-inquiry funnel).
//!
//! Ported from `src/communicator/api/routes/discovery.py`. Every response below
//! is either backed by real rows / a real provider call, or refuses with the
//! exact missing table, column, or credential. Nothing here invents inquiries,
//! templates, or Meta approval statuses.

use crate::db::AppState;
use crate::routes::auth::Tenant;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use chrono::Utc;
use serde::Deserialize;
use serde_json::{json, Value};

const INQUIRY_COLUMNS: &str = "id, tenant_key, conversation_id, contact_id, lead_id, \
    organization_name, organization_status, industry, contact_role, org_size_range, locations_count, \
    problem_description, problem_categories, reported_impact, frequency, impact_level, \
    current_workaround, limitations, desired_outcome, existing_systems, integration_required, \
    capability_mapping, timeline, decision_process, budget_range, qualification_score, \
    qualification_factors, status, next_action, created_at, updated_at";

const CAMPAIGN_COLUMNS: &str = "id, tenant_key, key, channel, name, purpose, meta_template_name, \
    meta_language, meta_category, meta_status, body_preview, variables, is_active, created_at, updated_at";

const FLOW_COLUMNS: &str = "id, tenant_key, name, is_active, steps, problem_category_keywords, \
    capability_keywords, capability_labels, scoring_weights, intro_message, closing_message, \
    created_at, updated_at";

fn db_error(e: sqlx::Error, context: &str) -> Response {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(json!({
            "error": format!("{context} failed: {e}"),
            "hint": "The table or one of the selected columns is missing from this database; see WIRING NEEDED for the exact DDL.",
        })),
    )
        .into_response()
}

async fn has_table(pool: &sqlx::PgPool, table: &str) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM information_schema.tables WHERE table_schema='public' AND table_name = $1",
    )
    .bind(table)
    .fetch_one(pool)
    .await
    .map(|c| c > 0)
}

/// A failed `information_schema` read is not the same fact as "table absent", and a
/// missing table must not become an invented listing either: both answer with
/// the DDL the operator has to apply. Returns `Some(refusal)` when the caller
/// must stop, `None` when the table is really there.
async fn require_table(
    pool: &sqlx::PgPool,
    table: &str,
    ddl: &str,
    wanted: &str,
) -> Option<Response> {
    match has_table(pool, table).await {
        Ok(true) => None,
        Ok(false) => Some(missing_table_response(table, ddl, wanted)),
        Err(e) => Some(db_error(e, &format!("information_schema lookup for '{table}'"))),
    }
}

/// Every read and write below is scoped to the authenticated tenant, so an
/// explicit `tenantKey` that differs from it is a request for another
/// organization's rows. Refuse it rather than honour or silently drop it.
fn tenant_mismatch(requested: Option<&str>, authenticated: &str) -> Option<Response> {
    requested
        .map(str::trim)
        .filter(|t| !t.is_empty() && *t != authenticated)
        .map(|t| {
            (
                StatusCode::FORBIDDEN,
                Json(json!({
                    "error": format!(
                        "The authenticated tenant is '{authenticated}' but the request asked for tenant '{t}'."
                    ),
                    "authenticatedTenantKey": authenticated,
                    "requestedTenantKey": t,
                    "note": "Tenant scope comes from the X-Tenant-Key header (or DEFAULT_TENANT_KEY); it cannot be widened by a query parameter.",
                })),
            )
                .into_response()
        })
}

fn missing_table_response(table: &str, ddl: &str, wanted: &str) -> Response {
    (
        StatusCode::SERVICE_UNAVAILABLE,
        Json(json!({
            "error": format!("Table '{table}' is not present in this database."),
            "missingTable": table,
            "wanted": wanted,
            "requiredDdl": ddl,
            "note": format!("Refusing to return invented {table} rows. Apply the DDL (see WIRING NEEDED) and restart the backend."),
        })),
    )
        .into_response()
}

fn json_array_len(raw: Option<&str>) -> usize {
    raw.and_then(|s| serde_json::from_str::<Value>(s).ok())
        .and_then(|v| v.as_array().map(|a| a.len()))
        .unwrap_or(0)
}

fn json_value(raw: Option<&str>) -> Value {
    raw.and_then(|s| serde_json::from_str::<Value>(s).ok())
        .unwrap_or(Value::Null)
}

fn str_or_empty(raw: Option<String>) -> String {
    raw.unwrap_or_default()
}

#[derive(Debug, sqlx::FromRow)]
struct InquiryRow {
    id: String,
    tenant_key: Option<String>,
    conversation_id: Option<String>,
    contact_id: Option<String>,
    lead_id: Option<String>,
    organization_name: Option<String>,
    organization_status: Option<String>,
    industry: Option<String>,
    contact_role: Option<String>,
    org_size_range: Option<String>,
    locations_count: Option<String>,
    problem_description: Option<String>,
    problem_categories: Option<String>,
    reported_impact: Option<String>,
    frequency: Option<String>,
    impact_level: Option<String>,
    current_workaround: Option<String>,
    limitations: Option<String>,
    desired_outcome: Option<String>,
    existing_systems: Option<String>,
    integration_required: Option<String>,
    capability_mapping: Option<String>,
    timeline: Option<String>,
    decision_process: Option<String>,
    budget_range: Option<String>,
    qualification_score: Option<i64>,
    qualification_factors: Option<String>,
    status: Option<String>,
    next_action: Option<String>,
    created_at: Option<String>,
    updated_at: Option<String>,
}

impl InquiryRow {
    fn serialize(&self) -> Value {
        json!({
            "id": self.id,
            "tenantKey": str_or_empty(self.tenant_key.clone()),
            "conversationId": self.conversation_id,
            "contactId": self.contact_id,
            "leadId": self.lead_id,
            "organizationName": str_or_empty(self.organization_name.clone()),
            "organizationStatus": str_or_empty(self.organization_status.clone()),
            "industry": str_or_empty(self.industry.clone()),
            "contactRole": str_or_empty(self.contact_role.clone()),
            "orgSizeRange": str_or_empty(self.org_size_range.clone()),
            "locationsCount": str_or_empty(self.locations_count.clone()),
            "problemDescription": str_or_empty(self.problem_description.clone()),
            "problemCategories": json_value(self.problem_categories.as_deref()),
            "reportedImpact": json_value(self.reported_impact.as_deref()),
            "frequency": str_or_empty(self.frequency.clone()),
            "impactLevel": str_or_empty(self.impact_level.clone()),
            "currentWorkaround": str_or_empty(self.current_workaround.clone()),
            "limitations": json_value(self.limitations.as_deref()),
            "desiredOutcome": str_or_empty(self.desired_outcome.clone()),
            "existingSystems": json_value(self.existing_systems.as_deref()),
            "integrationRequired": str_or_empty(self.integration_required.clone()),
            "capabilityMapping": json_value(self.capability_mapping.as_deref()),
            "timeline": str_or_empty(self.timeline.clone()),
            "decisionProcess": str_or_empty(self.decision_process.clone()),
            "budgetRange": str_or_empty(self.budget_range.clone()),
            "qualificationScore": self.qualification_score,
            "qualificationFactors": json_value(self.qualification_factors.as_deref()),
            "status": str_or_empty(self.status.clone()),
            "nextAction": self.next_action.clone(),
            "createdAt": self.created_at,
            "updatedAt": self.updated_at,
        })
    }
}

#[derive(Debug, sqlx::FromRow)]
struct CampaignRow {
    id: String,
    tenant_key: Option<String>,
    key: Option<String>,
    channel: Option<String>,
    name: Option<String>,
    purpose: Option<String>,
    meta_template_name: Option<String>,
    meta_language: Option<String>,
    meta_category: Option<String>,
    meta_status: Option<String>,
    body_preview: Option<String>,
    variables: Option<String>,
    is_active: Option<bool>,
    created_at: Option<String>,
    updated_at: Option<String>,
}

impl CampaignRow {
    fn serialize(&self) -> Value {
        json!({
            "id": self.id,
            "tenantKey": str_or_empty(self.tenant_key.clone()),
            "key": str_or_empty(self.key.clone()),
            "channel": if self.channel.as_deref().unwrap_or("").is_empty() { "shared" } else { self.channel.as_deref().unwrap() },
            "channelRaw": str_or_empty(self.channel.clone()),
            "name": str_or_empty(self.name.clone()),
            "purpose": str_or_empty(self.purpose.clone()),
            "metaTemplateName": str_or_empty(self.meta_template_name.clone()),
            "metaLanguage": str_or_empty(self.meta_language.clone()),
            "metaCategory": str_or_empty(self.meta_category.clone()),
            "metaStatus": str_or_empty(self.meta_status.clone()),
            "bodyPreview": str_or_empty(self.body_preview.clone()),
            "variables": json_value(self.variables.as_deref()),
            "isActive": self.is_active.unwrap_or(true),
            "createdAt": self.created_at,
            "updatedAt": self.updated_at,
        })
    }
}

#[derive(Debug, sqlx::FromRow)]
struct FlowRow {
    id: String,
    tenant_key: Option<String>,
    name: Option<String>,
    is_active: Option<bool>,
    steps: Option<String>,
    intro_message: Option<String>,
    closing_message: Option<String>,
    created_at: Option<String>,
    updated_at: Option<String>,
}

// ---------------------------------------------------------------------------
// GET /discovery/templates
// ---------------------------------------------------------------------------

pub async fn list_discovery_templates(
    State(state): State<AppState>,
    Tenant(tenant_key): Tenant,
) -> Response {
    if let Some(refusal) = require_table(
        &state.db,
        "discovery_flow_templates",
        DISCOVERY_FLOW_DDL,
        "this tenant's discovery-flow template with its step list",
    )
    .await
    {
        return refusal;
    }

    let rows = sqlx::query_as::<_, FlowRow>(&format!(
        "SELECT {FLOW_COLUMNS} FROM discovery_flow_templates WHERE tenant_key = $1 ORDER BY created_at ASC"
    ))
    .bind(&tenant_key)
    .fetch_all(&state.db)
    .await;

    match rows {
        Ok(rows) => {
            let out: Vec<Value> = rows
                .iter()
                .map(|t| {
                    json!({
                        "id": t.id,
                        "tenantKey": str_or_empty(t.tenant_key.clone()),
                        "name": str_or_empty(t.name.clone()),
                        "isActive": t.is_active.unwrap_or(true),
                        "stepCount": json_array_len(t.steps.as_deref()),
                        "introMessage": str_or_empty(t.intro_message.clone()),
                        "closingMessage": str_or_empty(t.closing_message.clone()),
                        "createdAt": t.created_at,
                        "updatedAt": t.updated_at,
                    })
                })
                .collect();
            let count = out.len();
            let warnings = if count == 0 {
                vec![format!(
                    "discovery_flow_templates holds no row for tenant '{tenant_key}'. No route in this backend writes that table: the flow step definitions (questions, option sets, keyword maps, scoring weights) exist only in the Python src/communicator/services/seed_discovery_templates.py, so a Rust writer would have to invent the questions instead of using the tenant's own flow."
                )]
            } else {
                Vec::new()
            };
            (
                StatusCode::OK,
                Json(json!({
                    "templates": out,
                    "count": count,
                    "tenantKey": tenant_key,
                    "warnings": warnings,
                    "note": if count == 0 {
                        "Real read: the table exists and holds no row for the authenticated tenant."
                    } else {
                        "Real rows from discovery_flow_templates for the authenticated tenant."
                    },
                })),
            )
                .into_response()
        }
        Err(e) => db_error(e, "SELECT FROM discovery_flow_templates"),
    }
}

// ---------------------------------------------------------------------------
// GET /discovery/inquiries, GET /discovery/inquiries/{id}
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, Default)]
pub struct InquiryQuery {
    #[serde(alias = "tenantKey", alias = "tenant_key")]
    pub tenant_key: Option<String>,
    pub status: Option<String>,
}

pub async fn list_inquiries(
    State(state): State<AppState>,
    Tenant(tenant_key): Tenant,
    Query(q): Query<InquiryQuery>,
) -> Response {
    if let Some(refusal) = tenant_mismatch(q.tenant_key.as_deref(), &tenant_key) {
        return refusal;
    }
    if let Some(refusal) = require_table(
        &state.db,
        "inquiries",
        INQUIRIES_DDL,
        "structured discovery inquiries",
    )
    .await
    {
        return refusal;
    }

    // `tenant_key = ?` is unconditional: inquiries.tenant_key is NOT NULL with
    // no schema default, so an unscoped read would blend two organizations'
    // funnels into one list.
    let mut sql = format!("SELECT {INQUIRY_COLUMNS} FROM inquiries WHERE tenant_key = $1");
    let status = q.status.clone().filter(|s| !s.trim().is_empty());
    if status.is_some() {
        sql.push_str(" AND status = $2");
    }
    sql.push_str(" ORDER BY created_at DESC");

    let mut query = sqlx::query_as::<_, InquiryRow>(&sql).bind(&tenant_key);
    if let Some(s) = &status {
        query = query.bind(s.clone());
    }

    match query.fetch_all(&state.db).await {
        Ok(rows) => {
            let out: Vec<Value> = rows.iter().map(|r| r.serialize()).collect();
            let count = out.len();
            (
                StatusCode::OK,
                Json(json!({
                    "inquiries": out,
                    "count": count,
                    "filters": { "tenantKey": tenant_key, "status": status },
                })),
            )
                .into_response()
        }
        Err(e) => db_error(e, "SELECT FROM inquiries"),
    }
}

pub async fn get_inquiry(
    State(state): State<AppState>,
    Tenant(tenant_key): Tenant,
    Path(id): Path<String>,
) -> Response {
    if let Some(refusal) = require_table(
        &state.db,
        "inquiries",
        INQUIRIES_DDL,
        "one discovery inquiry",
    )
    .await
    {
        return refusal;
    }

    match sqlx::query_as::<_, InquiryRow>(&format!(
        "SELECT {INQUIRY_COLUMNS} FROM inquiries WHERE id = $1 AND tenant_key = $2 LIMIT 1"
    ))
    .bind(&id)
    .bind(&tenant_key)
    .fetch_optional(&state.db)
    .await
    {
        Ok(Some(row)) => {
            // A failed conversation read is not "this inquiry has no
            // conversation": the reason is reported so the UI never shows an
            // unlinked inquiry by accident.
            let conv_raw = sqlx::query_scalar::<_, String>(
                "SELECT json_object('id', id, 'state', state, 'channel', channel, 'assignedToName', assigned_to_name) \
                 FROM conversations WHERE id = (SELECT conversation_id FROM inquiries WHERE id = $1 AND tenant_key = $2) LIMIT 1",
            )
            .bind(&id)
            .bind(&tenant_key)
            .fetch_optional(&state.db)
            .await;

            let mut conversation = Value::Null;
            let mut conversation_error: Option<String> = None;
            match conv_raw {
                Ok(Some(text)) => match serde_json::from_str::<Value>(&text) {
                    Ok(value) => conversation = value,
                    Err(e) => {
                        conversation_error =
                            Some(format!("The conversation row exists but its json_object did not parse: {e}"))
                    }
                },
                // No conversation row behind that id means genuinely unlinked,
                // which is a different fact from the read failing.
                Ok(None) => {}
                Err(e) => {
                    conversation_error =
                        Some(format!("Could not read the linked conversation row: {e}"))
                }            }

            let warnings: Vec<String> = conversation_error.clone().into_iter().collect();

            (
                StatusCode::OK,
                Json(json!({
                    "inquiry": row.serialize(),
                    "conversation": conversation,
                    "conversationLinked": !conversation.is_null(),
                    "conversationError": conversation_error,
                    "warnings": warnings,
                })),
            )
                .into_response()
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(json!({
                "error": format!(
                    "No inquiry with id '{id}' exists in the inquiries table for tenant '{tenant_key}'."
                ),
                "tenantKey": tenant_key,
            })),
        )
            .into_response(),
        Err(e) => db_error(e, "SELECT one inquiry"),
    }
}

// ---------------------------------------------------------------------------
// GET /discovery/campaign-templates
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, Default)]
pub struct CampaignQuery {
    #[serde(alias = "tenantKey")]
    pub tenant_key: Option<String>,
    pub channel: Option<String>,
}

pub async fn list_campaign_templates(
    State(state): State<AppState>,
    Tenant(tenant_key): Tenant,
    Query(q): Query<CampaignQuery>,
) -> Response {
    if let Some(refusal) = tenant_mismatch(q.tenant_key.as_deref(), &tenant_key) {
        return refusal;
    }
    if let Some(refusal) = require_table(
        &state.db,
        "campaign_templates",
        CAMPAIGN_TEMPLATES_DDL,
        "the tenant's reusable campaign message templates",
    )
    .await
    {
        return refusal;
    }

    let mut sql = format!("SELECT {CAMPAIGN_COLUMNS} FROM campaign_templates WHERE tenant_key = $1");
    let channel = q.channel.clone().filter(|s| !s.trim().is_empty());
    // A channel filter keeps that channel's templates plus every shared one.
    if channel.is_some() {
        sql.push_str(" AND (channel = $2 OR channel = '' OR channel IS NULL)");
    }
    sql.push_str(" ORDER BY created_at ASC");

    let mut query = sqlx::query_as::<_, CampaignRow>(&sql).bind(&tenant_key);
    if let Some(c) = &channel {
        query = query.bind(c.clone());
    }

    match query.fetch_all(&state.db).await {
        Ok(rows) => {
            let out: Vec<Value> = rows.iter().map(|r| r.serialize()).collect();
            let count = out.len();
            let warnings = if count == 0 {
                empty_template_warnings(&state, &tenant_key).await
            } else {
                Vec::new()
            };
            let filters_tenant = tenant_key.clone();
            (
                StatusCode::OK,
                Json(json!({
                    "templates": out,
                    "count": count,
                    "tenantKey": tenant_key,
                    "filters": { "tenantKey": filters_tenant, "channel": channel },
                    "warnings": warnings,
                    "note": if count == 0 {
                        "campaign_templates holds no row for the authenticated tenant; the warnings say why. Nothing here substitutes a template list."
                    } else {
                        "Real rows from campaign_templates for the authenticated tenant."
                    },
                })),
            )
                .into_response()
        }
        Err(e) => db_error(e, "SELECT FROM campaign_templates"),
    }
}

/// Why a tenant's campaign_templates table is empty, stated as facts: the
/// credential situation is checked live instead of guessed, and the routes that
/// can read Meta's real list are named.
async fn empty_template_warnings(state: &AppState, tenant_key: &str) -> Vec<String> {
    let mut warnings = vec![format!(
        "no campaign template rows are stored for tenant '{tenant_key}' -- nothing in this backend inserts them: the tenant-hardcoded seeder that used to write invented template copy was removed and no route replaces it. POST /discovery/campaign-templates/sync-status only refreshes the meta_* status of rows that already exist; GET /settings/whatsapp/templates and GET /inboxes/channels/whatsapp/templates read Meta's live list without storing it."
    )];

    match resolve_meta_credentials(state, tenant_key).await {
        Ok((Some(_), Some(_), _)) => warnings.push(
            "Meta credentials resolve for this tenant, so the live template list is readable even though this table is empty.".to_string(),
        ),
        Ok((token, waba, _)) => {
            let mut missing: Vec<&str> = Vec::new();
            if token.is_none() {
                missing.push("integration_settings.meta_access_token is empty and META_WHATSAPP_ACCESS_TOKEN is unset");
            }
            if waba.is_none() {
                missing.push("integration_settings.meta_waba_id is empty and META_WHATSAPP_BUSINESS_ACCOUNT_ID is unset");
            }
            warnings.push(format!(
                "Meta credentials are not configured for tenant '{tenant_key}': {} -- no sync could have run.",
                missing.join("; ")
            ));
        }
        Err(e) => warnings.push(format!(
            "the Meta credential situation for tenant '{tenant_key}' is unknown, not absent: reading integration_settings failed with {e}"
        )),
    }

    warnings
}

/// A Meta template field this route may not substitute a value for.
fn template_field<'a>(template: &'a Value, field: &str) -> Option<&'a str> {
    template
        .get(field)
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
}

// ---------------------------------------------------------------------------
// POST /discovery/campaign-templates/sync-status -- real Meta Graph API call
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, Default)]
pub struct SyncQuery {
    #[serde(alias = "tenantKey")]
    pub tenant_key: Option<String>,
}

async fn resolve_meta_credentials(
    state: &AppState,
    tenant_key: &str,
) -> Result<(Option<String>, Option<String>, Vec<String>), sqlx::Error> {
    let mut access_token = String::new();
    let mut waba_id = String::new();
    let mut sources = Vec::new();

    // `Err` here means the credential situation is unknown, which the callers
    // report as such instead of claiming Meta was never configured.
    let row: Option<(Option<String>, Option<String>)> = sqlx::query_as(
        "SELECT meta_access_token, meta_waba_id FROM integration_settings WHERE tenant_key = $1 LIMIT 1",
    )
    .bind(tenant_key)
    .fetch_optional(&state.db)
    .await?;

    if let Some((token, waba)) = row {
        if let Some(token) = token.filter(|t| !t.trim().is_empty()) {
            access_token = token.trim().to_string();
            sources.push(format!(
                "integration_settings row for tenant '{tenant_key}'"
            ));
        }
        if let Some(waba) = waba.filter(|w| !w.trim().is_empty()) {
            waba_id = waba.trim().to_string();
        }
    }

    if access_token.is_empty() && !state.config.meta_whatsapp_access_token.is_empty() {
        access_token = state.config.meta_whatsapp_access_token.clone();
        sources.push("META_WHATSAPP_ACCESS_TOKEN".to_string());
    }
    if waba_id.is_empty() {
        if let Ok(v) = std::env::var("META_WHATSAPP_BUSINESS_ACCOUNT_ID") {
            if !v.trim().is_empty() {
                waba_id = v.trim().to_string();
            }
        }
    }

    Ok((
        (!access_token.is_empty()).then_some(access_token),
        (!waba_id.is_empty()).then_some(waba_id),
        sources,
    ))
}

/// One Meta `message_templates` read, used by the status-sync route.
struct MetaRead {
    endpoint: String,
    waba_id: String,
    credential_sources: Vec<String>,
    http_status: u16,
    latency_ms: u128,
    data: Vec<Value>,
}

/// Each failure answers with the exact credential, endpoint or body that was
/// missing; no caller of this can fall through to a stand-in template list.
async fn read_meta_templates(
    state: &AppState,
    tenant_key: &str,
    fields: &str,
) -> Result<MetaRead, Response> {
    let (access_token, waba_id, sources) = match resolve_meta_credentials(state, tenant_key).await {
        Ok(creds) => creds,
        Err(e) => {
            return Err(db_error(
                e,
                "SELECT meta_access_token, meta_waba_id FROM integration_settings",
            ))
        }
    };

    let (token, waba) = match (access_token, waba_id) {
        (Some(token), Some(waba)) => (token, waba),
        (token, waba) => {
            let mut missing: Vec<String> = Vec::new();
            if token.is_none() {
                missing.push(
                    "meta access token (integration_settings.meta_access_token is empty and META_WHATSAPP_ACCESS_TOKEN is unset)".to_string(),
                );
            }
            if waba.is_none() {
                missing.push(
                    "WhatsApp Business Account id (integration_settings.meta_waba_id is empty and META_WHATSAPP_BUSINESS_ACCOUNT_ID is unset)".to_string(),
                );
            }
            return Err((
                StatusCode::SERVICE_UNAVAILABLE,
                Json(json!({
                    "status": "not_configured",
                    "tenantKey": tenant_key,
                    "missing": missing,
                    "error": "No request was sent to Meta: this deployment has no real WhatsApp Business credentials to sync with.",
                    "checked": 0,
                    "updated": 0,
                })),
            )
                .into_response());
        }
    };

    let api_version = if state.config.meta_whatsapp_api_version.is_empty() {
        "v21.0".to_string()
    } else {
        state.config.meta_whatsapp_api_version.clone()
    };
    let endpoint = format!(
        "https://graph.facebook.com/{api_version}/{waba}/message_templates?fields={fields}&limit=200"
    );

    let started = std::time::Instant::now();
    let res = match state
        .http_client
        .get(&endpoint)
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
    {
        Ok(r) => r,
        Err(e) => {
            return Err((
                StatusCode::BAD_GATEWAY,
                Json(json!({
                    "status": "transport_error",
                    "tenantKey": tenant_key,
                    "error": format!("Could not reach graph.facebook.com: {e}"),
                    "endpoint": endpoint,
                })),
            )
                .into_response())
        }
    };

    let http_status = res.status();
    let body = match res.text().await {
        Ok(text) => text,
        Err(e) => {
            return Err((
                StatusCode::BAD_GATEWAY,
                Json(json!({
                    "status": "body_read_error",
                    "tenantKey": tenant_key,
                    "httpStatus": http_status.as_u16(),
                    "error": format!("Meta replied but its body could not be read: {e}"),
                    "endpoint": endpoint,
                })),
            )
                .into_response())
        }
    };
    let latency_ms = started.elapsed().as_millis();

    if !http_status.is_success() {
        let parsed = serde_json::from_str::<Value>(&body).ok();
        return Err((
            StatusCode::BAD_GATEWAY,
            Json(json!({
                "status": "provider_error",
                "tenantKey": tenant_key,
                "httpStatus": http_status.as_u16(),
                "providerError": parsed
                    .as_ref()
                    .and_then(|v| v.pointer("/error/message"))
                    .and_then(|m| m.as_str())
                    .map(|s| s.to_string()),
                "providerCode": parsed.as_ref().and_then(|v| v.pointer("/error/code")).cloned(),
                "rawBody": body.chars().take(600).collect::<String>(),
                "endpoint": endpoint,
                "note": "Meta rejected the request; no local template rows were modified.",
            })),
        )
            .into_response());
    }

    let parsed: Value = match serde_json::from_str(&body) {
        Ok(v) => v,
        Err(e) => {
            return Err((
                StatusCode::BAD_GATEWAY,
                Json(json!({
                    "status": "bad_provider_response",
                    "tenantKey": tenant_key,
                    "error": format!("Meta replied with HTTP {} but the body was not JSON: {e}", http_status.as_u16()),
                    "rawBody": body.chars().take(400).collect::<String>(),
                    "endpoint": endpoint,
                })),
            )
                .into_response())
        }
    };

    // A 200 body without a data array is not "zero templates" -- it is unknown.
    let data = match parsed.get("data") {
        Some(Value::Array(rows)) => rows.clone(),
        _ => {
            return Err((
                StatusCode::BAD_GATEWAY,
                Json(json!({
                    "status": "bad_provider_response",
                    "tenantKey": tenant_key,
                    "httpStatus": http_status.as_u16(),
                    "error": format!(
                        "Meta replied HTTP 200 but its body carried no data array ({}), so templates cannot be reported",
                        if body.trim().is_empty() { "no body" } else { "key \"data\" absent or not an array" }
                    ),
                    "rawBody": body.chars().take(400).collect::<String>(),
                    "endpoint": endpoint,
                })),
            )
                .into_response())
        }
    };

    Ok(MetaRead {
        endpoint,
        waba_id: waba,
        credential_sources: sources,
        http_status: http_status.as_u16(),
        latency_ms,
        data,
    })
}

pub async fn sync_campaign_template_status(
    State(state): State<AppState>,
    Tenant(tenant_key): Tenant,
    Query(q): Query<SyncQuery>,
) -> Response {
    if let Some(refusal) = tenant_mismatch(q.tenant_key.as_deref(), &tenant_key) {
        return refusal;
    }

    let read = match read_meta_templates(&state, &tenant_key, "name,status,category,language").await
    {
        Ok(read) => read,
        Err(refusal) => return refusal,
    };

    struct RemoteTemplate {
        name: String,
        status: String,
        category: String,
        language: String,
    }

    // A remote row missing any field this table stores is reported, not
    // matched with an empty string that would overwrite a real status.
    let mut remote_map: Vec<RemoteTemplate> = Vec::new();
    let mut unnamed_remote = 0usize;
    let mut incomplete_remote: Vec<Value> = Vec::new();
    for t in &read.data {
        let Some(name) = template_field(t, "name") else {
            unnamed_remote += 1;
            continue;
        };
        match (
            template_field(t, "status"),
            template_field(t, "category"),
            template_field(t, "language"),
        ) {
            (Some(status), Some(category), Some(language)) => remote_map.push(RemoteTemplate {
                name: name.to_string(),
                status: status.to_string(),
                category: category.to_string(),
                language: language.to_string(),
            }),
            absent => {
                let mut missing: Vec<&str> = Vec::new();
                if absent.0.is_none() {
                    missing.push("status");
                }
                if absent.1.is_none() {
                    missing.push("category");
                }
                if absent.2.is_none() {
                    missing.push("language");
                }
                incomplete_remote.push(json!({ "name": name, "missingFromMeta": missing }));
            }
        }
    }

    let table_present = match has_table(&state.db, "campaign_templates").await {
        Ok(present) => present,
        Err(e) => return db_error(e, "information_schema lookup for 'campaign_templates'"),
    };

    let mut local_rows: Vec<(String, String, String)> = Vec::new();
    if table_present {
        // The rows this loop writes back to, so a failed read is fatal to the
        // request rather than an "everything is up to date" answer.
        match sqlx::query_as::<_, (String, String, String)>(
            "SELECT id, meta_template_name, COALESCE(meta_status, '') FROM campaign_templates WHERE tenant_key = $1",
        )
        .bind(&tenant_key)
        .fetch_all(&state.db)
        .await
        {
            Ok(rows) => local_rows = rows,
            Err(e) => return db_error(e, "SELECT FROM campaign_templates"),
        }
    }

    let now = Utc::now().to_rfc3339();
    let mut updated = 0usize;
    let mut unchanged = 0usize;
    let mut missing_on_meta: Vec<String> = Vec::new();
    let mut write_failures: Vec<Value> = Vec::new();

    for (id, meta_name, current_status) in &local_rows {
        match remote_map.iter().find(|remote| &remote.name == meta_name) {
            Some(remote) => {
                if remote.status == *current_status {
                    unchanged += 1;
                    continue;
                }
                let res = sqlx::query(
                    "UPDATE campaign_templates SET meta_status = $1, meta_category = $2, meta_language = $3, updated_at = $4 WHERE id = $5 AND tenant_key = $6",
                )
                .bind(&remote.status)
                .bind(&remote.category)
                .bind(&remote.language)
                .bind(&now)
                .bind(id)
                .bind(&tenant_key)
                .execute(&state.db)
                .await;
                match res {
                    Ok(r) if r.rows_affected() == 1 => updated += 1,
                    Ok(r) => write_failures.push(json!({ "id": id, "error": format!("UPDATE matched {} rows (expected 1)", r.rows_affected()) })),
                    Err(e) => write_failures.push(json!({ "id": id, "error": e.to_string() })),
                }
            }
            None => missing_on_meta.push(meta_name.clone()),
        }
    }

    let mut warnings: Vec<String> = Vec::new();
    if unnamed_remote > 0 {
        warnings.push(format!(
            "{unnamed_remote} remote template(s) carried no name and were left out of the match set."
        ));
    }
    if !incomplete_remote.is_empty() {
        warnings.push(format!(
            "{} remote template(s) arrived without the status/category/language this table stores, so their local rows were left untouched.",
            incomplete_remote.len()
        ));
    }
    if !write_failures.is_empty() {
        warnings.push(format!(
            "{} local row(s) were not updated despite Meta answering.",
            write_failures.len()
        ));
    }

    let unmapped = !local_rows.is_empty() && remote_map.is_empty();
    (
        StatusCode::OK,
        Json(json!({
            "status": if table_present { "ok" } else { "remote_only" },
            "tenantKey": tenant_key,
            "wabaId": read.waba_id,
            "credentialSource": read.credential_sources,
            "endpoint": read.endpoint,
            "httpStatus": read.http_status,
            "latencyMs": read.latency_ms,
            "remoteTemplateCount": remote_map.len(),
            "checked": local_rows.len(),
            "updated": updated,
            "unchanged": unchanged,
            "notRegisteredOnMeta": missing_on_meta,
            "localTablePresent": table_present,
            "writeFailures": write_failures,
            "warnings": warnings,
            "remoteTemplatesIncomplete": incomplete_remote,
            "remoteTemplates": remote_map.iter().map(|remote| json!({ "name": remote.name, "status": remote.status, "category": remote.category, "language": remote.language })).collect::<Vec<Value>>(),
            "note": if !table_present {
                "Meta answered, but campaign_templates does not exist in this database, so nothing local was updated. The remote list above is the real provider data."
            } else if unmapped {
                "Meta returned no named template with a status for this WABA id: either none are registered or the wrong account was used. No local status was changed."
            } else {
                "Local meta_status values were overwritten from Meta's real response."
            },
        })),
    )
        .into_response()
}

// ---------------------------------------------------------------------------
// DDL the routes above expect (reported under WIRING NEEDED)
//
// `db/mod.rs` owns the authoritative schema; these strings exist so a refusal
// can tell the operator what to apply. Neither one gives `tenant_key` a
// DEFAULT: a schema default would turn a writer that forgot the tenant into
// rows belonging to some other organization.
// ---------------------------------------------------------------------------

const DISCOVERY_FLOW_DDL: &str = r#"CREATE TABLE IF NOT EXISTS discovery_flow_templates (
    id TEXT PRIMARY KEY,
    tenant_key TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT 1,
    steps TEXT NOT NULL DEFAULT '[]',
    problem_category_keywords TEXT NOT NULL DEFAULT '{}',
    capability_keywords TEXT NOT NULL DEFAULT '{}',
    capability_labels TEXT NOT NULL DEFAULT '{}',
    scoring_weights TEXT NOT NULL DEFAULT '{}',
    intro_message TEXT NOT NULL DEFAULT '',
    closing_message TEXT NOT NULL DEFAULT '',
    created_at DATETIME NOT NULL,
    updated_at DATETIME NOT NULL
);"#;

const INQUIRIES_DDL: &str = r#"CREATE TABLE IF NOT EXISTS inquiries (
    id TEXT PRIMARY KEY,
    tenant_key TEXT NOT NULL,
    conversation_id TEXT UNIQUE,
    contact_id TEXT,
    lead_id TEXT,
    organization_name TEXT NOT NULL DEFAULT '',
    organization_status TEXT NOT NULL DEFAULT '',
    industry TEXT NOT NULL DEFAULT '',
    contact_role TEXT NOT NULL DEFAULT '',
    org_size_range TEXT NOT NULL DEFAULT '',
    locations_count TEXT NOT NULL DEFAULT '',
    problem_description TEXT NOT NULL DEFAULT '',
    problem_categories TEXT NOT NULL DEFAULT '[]',
    reported_impact TEXT NOT NULL DEFAULT '[]',
    frequency TEXT NOT NULL DEFAULT '',
    impact_level TEXT NOT NULL DEFAULT '',
    current_workaround TEXT NOT NULL DEFAULT '',
    limitations TEXT NOT NULL DEFAULT '[]',
    desired_outcome TEXT NOT NULL DEFAULT '',
    existing_systems TEXT NOT NULL DEFAULT '[]',
    integration_required TEXT NOT NULL DEFAULT '',
    capability_mapping TEXT NOT NULL DEFAULT '[]',
    timeline TEXT NOT NULL DEFAULT '',
    decision_process TEXT NOT NULL DEFAULT '',
    budget_range TEXT NOT NULL DEFAULT '',
    qualification_score INTEGER,
    qualification_factors TEXT NOT NULL DEFAULT '{}',
    status TEXT NOT NULL DEFAULT 'discovery_in_progress',
    next_action TEXT,
    created_at DATETIME NOT NULL,
    updated_at DATETIME NOT NULL
);"#;

const CAMPAIGN_TEMPLATES_DDL: &str = r#"CREATE TABLE IF NOT EXISTS campaign_templates (
    id TEXT PRIMARY KEY,
    tenant_key TEXT NOT NULL,
    key TEXT NOT NULL,
    channel TEXT NOT NULL DEFAULT '',
    name TEXT NOT NULL,
    purpose TEXT NOT NULL DEFAULT '',
    meta_template_name TEXT NOT NULL,
    meta_language TEXT NOT NULL,
    meta_category TEXT NOT NULL,
    meta_status TEXT NOT NULL,
    body_preview TEXT NOT NULL DEFAULT '',
    variables TEXT NOT NULL DEFAULT '[]',
    is_active BOOLEAN NOT NULL DEFAULT 1,
    created_at DATETIME NOT NULL,
    updated_at DATETIME NOT NULL,
    UNIQUE (tenant_key, key)
);"#;
