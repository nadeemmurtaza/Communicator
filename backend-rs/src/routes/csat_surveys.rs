use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::Row;
use uuid::Uuid;

use crate::db::AppState;

// ——— Types ———

#[derive(Serialize, Deserialize, Debug)]
pub struct CSATSurveyTemplate {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub question: String,
    pub scale_type: String,
    pub min_score: i32,
    pub max_score: i32,
    pub include_comment: bool,
    pub comment_prompt: Option<String>,
    pub trigger_type: String,
    pub trigger_delay_minutes: i32,
    pub channel_filter: Option<String>,
    pub enabled: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CSATSurvey {
    pub id: String,
    pub template_id: String,
    pub conversation_id: String,
    pub contact_id: String,
    pub channel: String,
    pub contact_identifier: String,
    pub status: String,
    pub score: Option<i32>,
    pub comment: Option<String>,
    pub sent_at: Option<String>,
    pub responded_at: Option<String>,
    pub expires_at: Option<String>,
    pub created_at: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CSATSnapshot {
    pub id: String,
    pub window_start: String,
    pub window_end: String,
    pub total_surveys_sent: i32,
    pub total_responses: i32,
    pub response_rate: f64,
    pub average_score: Option<f64>,
    pub csat_percentage: Option<f64>,
    pub score_distribution: Option<Value>,
    pub by_channel: Option<Value>,
    pub by_template: Option<Value>,
    pub top_comments: Option<Value>,
    pub created_at: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CSATSummary {
    pub overall_csat: Option<f64>,
    pub overall_average_score: Option<f64>,
    pub total_surveys_sent: i32,
    pub total_responses: i32,
    pub response_rate: f64,
    pub score_distribution: Value,
    pub by_channel: Value,
    pub trend_7d: Option<f64>,
    pub recent_comments: Vec<CSATComment>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CSATComment {
    pub survey_id: String,
    pub score: i32,
    pub comment: String,
    pub contact_identifier: String,
    pub channel: String,
    pub responded_at: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CSATResponse {
    pub score: i32,
    pub comment: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct CSATQueryParams {
    pub status: Option<String>,
    pub channel: Option<String>,
    pub template_id: Option<String>,
    pub conversation_id: Option<String>,
    pub contact_id: Option<String>,
    pub limit: Option<i32>,
    pub offset: Option<i32>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CSATSnapshotResult {
    pub snapshot: CSATSnapshot,
}

// ——— Template CRUD ———

pub async fn list_templates(
    State(state): State<AppState>,
    Query(params): Query<CSATQueryParams>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let db = &state.db;
    let mut query = r#"
        SELECT id, name, description, question, scale_type, min_score, max_score,
               include_comment, comment_prompt, trigger_type, trigger_delay_minutes,
               channel_filter, enabled, created_at, updated_at
        FROM csat_survey_templates
        WHERE 1=1
    "#.to_string();
    let mut binds: Vec<String> = Vec::new();

    if let Some(ref enabled_filter) = params.status {
        // reuse status param for enabled filter
        if enabled_filter == "enabled" {
            query.push_str(" AND enabled = 1");
        } else if enabled_filter == "disabled" {
            query.push_str(" AND enabled = 0");
        }
    }

    query.push_str(" ORDER BY created_at DESC");

    if let Some(limit) = params.limit {
        query.push_str(" LIMIT ?");
        binds.push(limit.to_string());
    }

    let rows = sqlx::query(&query)
        .fetch_all(db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?;

    let templates = rows
        .iter()
        .map(|row| {
            json!({
                "id": row.try_get::<String, _>("id").ok(),
                "name": row.try_get::<String, _>("name").ok(),
                "description": row.try_get::<Option<String>, _>("description").ok(),
                "question": row.try_get::<String, _>("question").ok(),
                "scale_type": row.try_get::<String, _>("scale_type").ok(),
                "min_score": row.try_get::<i32, _>("min_score").ok(),
                "max_score": row.try_get::<i32, _>("max_score").ok(),
                "include_comment": row.try_get::<bool, _>("include_comment").ok(),
                "comment_prompt": row.try_get::<Option<String>, _>("comment_prompt").ok(),
                "trigger_type": row.try_get::<String, _>("trigger_type").ok(),
                "trigger_delay_minutes": row.try_get::<i32, _>("trigger_delay_minutes").ok(),
                "channel_filter": row.try_get::<Option<String>, _>("channel_filter").ok(),
                "enabled": row.try_get::<bool, _>("enabled").ok(),
                "created_at": row.try_get::<String, _>("created_at").ok(),
                "updated_at": row.try_get::<String, _>("updated_at").ok(),
            })
        })
        .collect::<Vec<_>>();

    Ok(Json(json!({
        "templates": templates,
        "total": templates.len(),
    })))
}

pub async fn get_template(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let db = &state.db;
    let row = sqlx::query(
        "SELECT id, name, description, question, scale_type, min_score, max_score,
         include_comment, comment_prompt, trigger_type, trigger_delay_minutes,
         channel_filter, enabled, created_at, updated_at
         FROM csat_survey_templates WHERE id = ?"
    )
    .bind(&id)
    .fetch_optional(db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?;

    match row {
        Some(row) => Ok(Json(json!({
            "id": row.try_get::<String, _>("id").ok(),
            "name": row.try_get::<String, _>("name").ok(),
            "description": row.try_get::<Option<String>, _>("description").ok(),
            "question": row.try_get::<String, _>("question").ok(),
            "scale_type": row.try_get::<String, _>("scale_type").ok(),
            "min_score": row.try_get::<i32, _>("min_score").ok(),
            "max_score": row.try_get::<i32, _>("max_score").ok(),
            "include_comment": row.try_get::<bool, _>("include_comment").ok(),
            "comment_prompt": row.try_get::<Option<String>, _>("comment_prompt").ok(),
            "trigger_type": row.try_get::<String, _>("trigger_type").ok(),
            "trigger_delay_minutes": row.try_get::<i32, _>("trigger_delay_minutes").ok(),
            "channel_filter": row.try_get::<Option<String>, _>("channel_filter").ok(),
            "enabled": row.try_get::<bool, _>("enabled").ok(),
            "created_at": row.try_get::<String, _>("created_at").ok(),
            "updated_at": row.try_get::<String, _>("updated_at").ok(),
        }))),
        None => Err((StatusCode::NOT_FOUND, Json(json!({"error": "Template not found"})))),
    }
}

pub async fn create_template(
    State(state): State<AppState>,
    Json(body): Json<Value>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let db = &state.db;
    let id = Uuid::new_v4().to_string();
    let name = body["name"].as_str().unwrap_or("Custom Survey").to_string();
    let description = body["description"].as_str().map(String::from);
    let question = body["question"].as_str().unwrap_or("How would you rate this conversation?").to_string();
    let scale_type = body["scale_type"].as_str().unwrap_or("1-5").to_string();
    let min_score = body["min_score"].as_i64().unwrap_or(1) as i32;
    let max_score = body["max_score"].as_i64().unwrap_or(5) as i32;
    let include_comment = body["include_comment"].as_bool().unwrap_or(true);
    let comment_prompt = body["comment_prompt"].as_str().map(String::from);
    let trigger_type = body["trigger_type"].as_str().unwrap_or("manual").to_string();
    let trigger_delay_minutes = body["trigger_delay_minutes"].as_i64().unwrap_or(0) as i32;
    let channel_filter = body["channel_filter"].as_str().map(String::from);
    let enabled = body["enabled"].as_bool().unwrap_or(true);

    sqlx::query(
        "INSERT INTO csat_survey_templates (id, name, description, question, scale_type, min_score, max_score,
         include_comment, comment_prompt, trigger_type, trigger_delay_minutes, channel_filter, enabled)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(&id)
    .bind(&name)
    .bind(&description)
    .bind(&question)
    .bind(&scale_type)
    .bind(min_score)
    .bind(max_score)
    .bind(include_comment)
    .bind(&comment_prompt)
    .bind(&trigger_type)
    .bind(trigger_delay_minutes)
    .bind(&channel_filter)
    .bind(enabled)
    .execute(db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?;

    Ok(Json(json!({
        "id": id,
        "name": name,
        "question": question,
        "scale_type": scale_type,
        "enabled": enabled,
        "message": "Template created successfully",
    })))
}

pub async fn update_template(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<Value>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let db = &state.db;

    // Check exists
    let exists = sqlx::query("SELECT COUNT(*) FROM csat_survey_templates WHERE id = ?")
        .bind(&id)
        .fetch_one(db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?;

    if exists.try_get::<i64, _>(0).unwrap_or(0) == 0 {
        return Err((StatusCode::NOT_FOUND, Json(json!({"error": "Template not found"}))));
    }

    let mut sets = Vec::new();
    let mut binds: Vec<Value> = Vec::new();

    if let Some(name) = body["name"].as_str() {
        sets.push("name = ?");
        binds.push(json!(name));
    }
    if let Some(description) = body["description"].as_str() {
        sets.push("description = ?");
        binds.push(json!(description));
    }
    if let Some(question) = body["question"].as_str() {
        sets.push("question = ?");
        binds.push(json!(question));
    }
    if let Some(scale_type) = body["scale_type"].as_str() {
        sets.push("scale_type = ?");
        binds.push(json!(scale_type));
    }
    if let Some(min_score) = body["min_score"].as_i64() {
        sets.push("min_score = ?");
        binds.push(json!(min_score));
    }
    if let Some(max_score) = body["max_score"].as_i64() {
        sets.push("max_score = ?");
        binds.push(json!(max_score));
    }
    if let Some(include_comment) = body["include_comment"].as_bool() {
        sets.push("include_comment = ?");
        binds.push(json!(include_comment));
    }
    if let Some(comment_prompt) = body["comment_prompt"].as_str() {
        sets.push("comment_prompt = ?");
        binds.push(json!(comment_prompt));
    }
    if let Some(trigger_type) = body["trigger_type"].as_str() {
        sets.push("trigger_type = ?");
        binds.push(json!(trigger_type));
    }
    if let Some(trigger_delay) = body["trigger_delay_minutes"].as_i64() {
        sets.push("trigger_delay_minutes = ?");
        binds.push(json!(trigger_delay));
    }
    if let Some(channel_filter) = body["channel_filter"].as_str() {
        sets.push("channel_filter = ?");
        binds.push(json!(channel_filter));
    }
    if let Some(enabled) = body["enabled"].as_bool() {
        sets.push("enabled = ?");
        binds.push(json!(enabled));
    }

    if sets.is_empty() {
        return Ok(Json(json!({"message": "No fields to update"})));
    }

    sets.push("updated_at = ?");
    binds.push(json!(Utc::now().format("%Y-%m-%dT%H:%M:%fZ").to_string()));
    binds.push(json!(id));

    let sql = format!("UPDATE csat_survey_templates SET {} WHERE id = ?", sets.join(", "));

    let mut query_builder = sqlx::query(&sql);
    for b in &binds {
        match b {
            Value::String(s) => { query_builder = query_builder.bind(s.as_str()); }
            Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    query_builder = query_builder.bind(i);
                }
            }
            Value::Bool(b) => { query_builder = query_builder.bind(*b); }
            _ => {}
        }
    }

    query_builder
        .execute(db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?;

    Ok(Json(json!({
        "id": id,
        "message": "Template updated successfully",
    })))
}

pub async fn delete_template(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let db = &state.db;

    sqlx::query("DELETE FROM csat_survey_templates WHERE id = ?")
        .bind(&id)
        .execute(db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?;

    Ok(Json(json!({"message": "Template deleted successfully"})))
}

// ——— Survey operations ———

pub async fn list_surveys(
    State(state): State<AppState>,
    Query(params): Query<CSATQueryParams>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let db = &state.db;
    let mut query = r#"
        SELECT s.id, s.template_id, s.conversation_id, s.contact_id, s.channel,
               s.contact_identifier, s.status, s.score, s.comment, s.sent_at,
               s.responded_at, s.expires_at, s.created_at
        FROM csat_surveys s
        WHERE 1=1
    "#.to_string();
    let mut binds: Vec<String> = Vec::new();

    if let Some(ref status) = params.status {
        query.push_str(" AND s.status = ?");
        binds.push(status.clone());
    }
    if let Some(ref channel) = params.channel {
        query.push_str(" AND s.channel = ?");
        binds.push(channel.clone());
    }
    if let Some(ref template_id) = params.template_id {
        query.push_str(" AND s.template_id = ?");
        binds.push(template_id.clone());
    }
    if let Some(ref conversation_id) = params.conversation_id {
        query.push_str(" AND s.conversation_id = ?");
        binds.push(conversation_id.clone());
    }
    if let Some(ref contact_id) = params.contact_id {
        query.push_str(" AND s.contact_id = ?");
        binds.push(contact_id.clone());
    }

    query.push_str(" ORDER BY s.created_at DESC");

    let limit = params.limit.unwrap_or(50);
    let offset = params.offset.unwrap_or(0);
    query.push_str(" LIMIT ? OFFSET ?");
    binds.push(limit.to_string());
    binds.push(offset.to_string());

    let mut qb = sqlx::query(&query);
    for b in &binds {
        qb = qb.bind(b.as_str());
    }

    let rows = qb
        .fetch_all(db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?;

    let surveys = rows
        .iter()
        .map(|row| {
            json!({
                "id": row.try_get::<String, _>("id").ok(),
                "template_id": row.try_get::<String, _>("template_id").ok(),
                "conversation_id": row.try_get::<String, _>("conversation_id").ok(),
                "contact_id": row.try_get::<String, _>("contact_id").ok(),
                "channel": row.try_get::<String, _>("channel").ok(),
                "contact_identifier": row.try_get::<String, _>("contact_identifier").ok(),
                "status": row.try_get::<String, _>("status").ok(),
                "score": row.try_get::<Option<i32>, _>("score").ok(),
                "comment": row.try_get::<Option<String>, _>("comment").ok(),
                "sent_at": row.try_get::<Option<String>, _>("sent_at").ok(),
                "responded_at": row.try_get::<Option<String>, _>("responded_at").ok(),
                "expires_at": row.try_get::<Option<String>, _>("expires_at").ok(),
                "created_at": row.try_get::<String, _>("created_at").ok(),
            })
        })
        .collect::<Vec<_>>();

    Ok(Json(json!({
        "surveys": surveys,
        "total": surveys.len(),
    })))
}

/// Create a new survey for a conversation
pub async fn create_survey(
    State(state): State<AppState>,
    Json(body): Json<Value>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let db = &state.db;
    let id = Uuid::new_v4().to_string();
    let template_id = body["template_id"].as_str().unwrap_or("default").to_string();
    let conversation_id = body["conversation_id"].as_str()
        .ok_or((StatusCode::BAD_REQUEST, Json(json!({"error": "conversation_id is required"}))))?
        .to_string();
    let contact_id = body["contact_id"].as_str()
        .ok_or((StatusCode::BAD_REQUEST, Json(json!({"error": "contact_id is required"}))))?
        .to_string();
    let channel = body["channel"].as_str().unwrap_or("email").to_string();
    let contact_identifier = body["contact_identifier"].as_str().unwrap_or("").to_string();

    // Get template to determine expiry
    let template = sqlx::query(
        "SELECT trigger_delay_minutes FROM csat_survey_templates WHERE id = ?"
    )
    .bind(&template_id)
    .fetch_optional(db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?;

    let delay_minutes = template.and_then(|r| r.try_get::<i32, _>("trigger_delay_minutes").ok()).unwrap_or(0);
    let expires_at = if delay_minutes > 0 {
        let expires = Utc::now() + Duration::minutes(delay_minutes as i64 * 60);
        Some(expires.format("%Y-%m-%dT%H:%M:%fZ").to_string())
    } else {
        // Default: expire in 7 days
        let expires = Utc::now() + Duration::days(7);
        Some(expires.format("%Y-%m-%dT%H:%M:%fZ").to_string())
    };

    sqlx::query(
        "INSERT INTO csat_surveys (id, template_id, conversation_id, contact_id, channel,
         contact_identifier, status, expires_at)
         VALUES (?, ?, ?, ?, ?, ?, 'pending', ?)"
    )
    .bind(&id)
    .bind(&template_id)
    .bind(&conversation_id)
    .bind(&contact_id)
    .bind(&channel)
    .bind(&contact_identifier)
    .bind(&expires_at)
    .execute(db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?;

    Ok(Json(json!({
        "id": id,
        "template_id": template_id,
        "conversation_id": conversation_id,
        "status": "pending",
        "expires_at": expires_at,
        "message": "Survey created successfully",
    })))
}

/// Submit a response to a survey
pub async fn respond_to_survey(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<CSATResponse>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let db = &state.db;

    // Check survey exists and is pending/sent
    let survey = sqlx::query(
        "SELECT id, status, template_id FROM csat_surveys WHERE id = ?"
    )
    .bind(&id)
    .fetch_optional(db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?;

    let survey = match survey {
        Some(s) => s,
        None => return Err((StatusCode::NOT_FOUND, Json(json!({"error": "Survey not found"})))),
    };

    let status: String = survey.try_get("status").unwrap_or("pending".to_string());
    if status == "responded" {
        return Err((StatusCode::BAD_REQUEST, Json(json!({"error": "Survey already responded"}))));
    }
    if status == "expired" {
        return Err((StatusCode::BAD_REQUEST, Json(json!({"error": "Survey has expired"}))));
    }

    // Validate score range
    let template_id: String = survey.try_get("template_id").unwrap_or("default".to_string());
    let template = sqlx::query(
        "SELECT min_score, max_score FROM csat_survey_templates WHERE id = ?"
    )
    .bind(&template_id)
    .fetch_optional(db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?;

    if let Some(t) = &template {
        let min_s: i32 = t.try_get("min_score").unwrap_or(1);
        let max_s: i32 = t.try_get("max_score").unwrap_or(5);
        if body.score < min_s || body.score > max_s {
            return Err((StatusCode::BAD_REQUEST, Json(json!({
                "error": format!("Score must be between {} and {}", min_s, max_s)
            }))));
        }
    }

    let now = Utc::now().format("%Y-%m-%dT%H:%M:%fZ").to_string();

    sqlx::query(
        "UPDATE csat_surveys SET status = 'responded', score = ?, comment = ?, responded_at = ?
         WHERE id = ?"
    )
    .bind(body.score)
    .bind(&body.comment)
    .bind(&now)
    .bind(&id)
    .execute(db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?;

    Ok(Json(json!({
        "id": id,
        "score": body.score,
        "comment": body.comment,
        "responded_at": now,
        "message": "Response recorded successfully",
    })))
}

/// Send a survey (mark as sent)
pub async fn send_survey(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let db = &state.db;
    let now = Utc::now().format("%Y-%m-%dT%H:%M:%fZ").to_string();

    let result = sqlx::query(
        "UPDATE csat_surveys SET status = 'sent', sent_at = ?
         WHERE id = ? AND status = 'pending'"
    )
    .bind(&now)
    .bind(&id)
    .execute(db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?;

    if result.rows_affected() == 0 {
        return Err((StatusCode::NOT_FOUND, Json(json!({"error": "Survey not found or already sent"}))));
    }

    Ok(Json(json!({
        "id": id,
        "status": "sent",
        "sent_at": now,
        "message": "Survey marked as sent",
    })))
}

// ——— Analytics ———

pub async fn get_csat_summary(
    State(state): State<AppState>,
    Query(params): Query<CSATQueryParams>,
) -> Result<Json<CSATSummary>, (StatusCode, Json<Value>)> {
    let db = &state.db;

    // Time window: last 30 days by default
    let days = params.limit.unwrap_or(30);
    let window_start = (Utc::now() - Duration::days(days as i64)).format("%Y-%m-%dT%H:%M:%fZ").to_string();

    // Total sent
    let total_sent: i64 = sqlx::query(
        "SELECT COUNT(*) FROM csat_surveys WHERE created_at >= ? AND status != 'cancelled'"
    )
    .bind(&window_start)
    .fetch_one(db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?
    .try_get(0).unwrap_or(0);

    // Total responses
    let total_responses: i64 = sqlx::query(
        "SELECT COUNT(*) FROM csat_surveys WHERE created_at >= ? AND status = 'responded'"
    )
    .bind(&window_start)
    .fetch_one(db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?
    .try_get(0).unwrap_or(0);

    // Average score
    let avg_row = sqlx::query(
        "SELECT AVG(score) as avg_score FROM csat_surveys WHERE created_at >= ? AND status = 'responded' AND score IS NOT NULL"
    )
    .bind(&window_start)
    .fetch_optional(db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?;

    let average_score = avg_row.and_then(|r| r.try_get::<f64, _>("avg_score").ok());

    // CSAT percentage (% of 4-5 scores for 1-5 scale)
    let csat_row = sqlx::query(
        "SELECT COUNT(*) FROM csat_surveys WHERE created_at >= ? AND status = 'responded' AND score >= 4"
    )
    .bind(&window_start)
    .fetch_one(db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?;

    let csat_count: i64 = csat_row.try_get(0).unwrap_or(0);
    let csat_percentage = if total_responses > 0 {
        Some((csat_count as f64 / total_responses as f64) * 100.0)
    } else {
        None
    };

    // Score distribution
    let dist_rows = sqlx::query(
        "SELECT score, COUNT(*) as count FROM csat_surveys
         WHERE created_at >= ? AND status = 'responded' AND score IS NOT NULL
         GROUP BY score ORDER BY score"
    )
    .bind(&window_start)
    .fetch_all(db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?;

    let mut score_distribution = serde_json::Map::new();
    for row in &dist_rows {
        let score: i32 = row.try_get("score").unwrap_or(0);
        let count: i64 = row.try_get("count").unwrap_or(0);
        score_distribution.insert(score.to_string(), json!(count));
    }

    // By channel
    let channel_rows = sqlx::query(
        "SELECT channel, AVG(score) as avg_score, COUNT(*) as count
         FROM csat_surveys WHERE created_at >= ? AND status = 'responded' AND score IS NOT NULL
         GROUP BY channel"
    )
    .bind(&window_start)
    .fetch_all(db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?;

    let mut by_channel = serde_json::Map::new();
    for row in &channel_rows {
        let channel: String = row.try_get("channel").unwrap_or("unknown".to_string());
        let avg: f64 = row.try_get("avg_score").unwrap_or(0.0);
        let count: i64 = row.try_get("count").unwrap_or(0);
        by_channel.insert(channel, json!({"avg": (avg * 100.0).round() / 100.0, "count": count}));
    }

    // Recent comments
    let comment_rows = sqlx::query(
        "SELECT s.id, s.score, s.comment, s.contact_identifier, s.channel, s.responded_at
         FROM csat_surveys s
         WHERE created_at >= ? AND status = 'responded' AND s.comment IS NOT NULL AND s.comment != ''
         ORDER BY s.responded_at DESC LIMIT 10"
    )
    .bind(&window_start)
    .fetch_all(db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?;

    let recent_comments: Vec<CSATComment> = comment_rows
        .iter()
        .filter_map(|row| {
            let survey_id = row.try_get::<String, _>("id").ok()?;
            let score = row.try_get::<i32, _>("score").ok()?;
            let comment = row.try_get::<String, _>("comment").ok()?;
            let contact_identifier = row.try_get::<String, _>("contact_identifier").ok()?;
            let channel = row.try_get::<String, _>("channel").ok()?;
            let responded_at = row.try_get::<String, _>("responded_at").ok()?;
            Some(CSATComment {
                survey_id,
                score,
                comment,
                contact_identifier,
                channel,
                responded_at,
            })
        })
        .collect();

    // Trend: compare last 7 days to previous 7 days
    let trend_7d = compute_trend(db).await.ok();

    let response_rate = if total_sent > 0 {
        (total_responses as f64 / total_sent as f64) * 100.0
    } else {
        0.0
    };

    Ok(Json(CSATSummary {
        overall_csat: csat_percentage,
        overall_average_score: average_score,
        total_surveys_sent: total_sent as i32,
        total_responses: total_responses as i32,
        response_rate: (response_rate * 100.0).round() / 100.0,
        score_distribution: Value::Object(score_distribution),
        by_channel: Value::Object(by_channel),
        trend_7d,
        recent_comments,
    }))
}

async fn compute_trend(db: &sqlx::PgPool) -> Result<f64, sqlx::Error> {
    let now = Utc::now();
    let week_ago = (now - Duration::days(7)).format("%Y-%m-%dT%H:%M:%fZ").to_string();
    let two_weeks_ago = (now - Duration::days(14)).format("%Y-%m-%dT%H:%M:%fZ").to_string();

    // Current week avg
    let current = sqlx::query(
        "SELECT AVG(score) FROM csat_surveys WHERE created_at >= ? AND created_at < ? AND status = 'responded' AND score IS NOT NULL"
    )
    .bind(&week_ago)
    .bind(&now.format("%Y-%m-%dT%H:%M:%fZ").to_string())
    .fetch_optional(db)
    .await?;

    // Previous week avg
    let previous = sqlx::query(
        "SELECT AVG(score) FROM csat_surveys WHERE created_at >= ? AND created_at < ? AND status = 'responded' AND score IS NOT NULL"
    )
    .bind(&two_weeks_ago)
    .bind(&week_ago)
    .fetch_optional(db)
    .await?;

    let current_avg = current.and_then(|r| r.try_get::<f64, _>(0).ok());
    let previous_avg = previous.and_then(|r| r.try_get::<f64, _>(0).ok());

    match (current_avg, previous_avg) {
        (Some(c), Some(p)) if p > 0.0 => Ok(((c - p) / p) * 100.0),
        _ => Ok(0.0),
    }
}

/// Generate a snapshot for a time window
pub async fn generate_snapshot(
    State(state): State<AppState>,
) -> Result<Json<CSATSnapshotResult>, (StatusCode, Json<Value>)> {
    let db = &state.db;
    let id = Uuid::new_v4().to_string();
    let now = Utc::now();
    let window_start = (now - Duration::days(7)).format("%Y-%m-%dT%H:%M:%fZ").to_string();
    let window_end = now.format("%Y-%m-%dT%H:%M:%fZ").to_string();

    // Reuse summary logic for the window
    let total_sent: i64 = sqlx::query(
        "SELECT COUNT(*) FROM csat_surveys WHERE created_at >= ? AND created_at < ? AND status != 'cancelled'"
    )
    .bind(&window_start)
    .bind(&window_end)
    .fetch_one(db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?
    .try_get(0).unwrap_or(0);

    let total_responses: i64 = sqlx::query(
        "SELECT COUNT(*) FROM csat_surveys WHERE created_at >= ? AND created_at < ? AND status = 'responded'"
    )
    .bind(&window_start)
    .bind(&window_end)
    .fetch_one(db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?
    .try_get(0).unwrap_or(0);

    let avg_row = sqlx::query(
        "SELECT AVG(score) as avg_score FROM csat_surveys WHERE created_at >= ? AND created_at < ? AND status = 'responded' AND score IS NOT NULL"
    )
    .bind(&window_start)
    .bind(&window_end)
    .fetch_optional(db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?;

    let average_score = avg_row.and_then(|r| r.try_get::<f64, _>("avg_score").ok());

    let csat_count: i64 = sqlx::query(
        "SELECT COUNT(*) FROM csat_surveys WHERE created_at >= ? AND created_at < ? AND status = 'responded' AND score >= 4"
    )
    .bind(&window_start)
    .bind(&window_end)
    .fetch_one(db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?
    .try_get(0).unwrap_or(0);

    let csat_percentage = if total_responses > 0 {
        Some((csat_count as f64 / total_responses as f64) * 100.0)
    } else {
        None
    };

    let response_rate = if total_sent > 0 {
        (total_responses as f64 / total_sent as f64) * 100.0
    } else {
        0.0
    };

    // Score distribution
    let dist_rows = sqlx::query(
        "SELECT score, COUNT(*) as count FROM csat_surveys
         WHERE created_at >= ? AND created_at < ? AND status = 'responded' AND score IS NOT NULL
         GROUP BY score ORDER BY score"
    )
    .bind(&window_start)
    .bind(&window_end)
    .fetch_all(db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?;

    let mut score_distribution = serde_json::Map::new();
    for row in &dist_rows {
        let score: i32 = row.try_get("score").unwrap_or(0);
        let count: i64 = row.try_get("count").unwrap_or(0);
        score_distribution.insert(score.to_string(), json!(count));
    }

    // By channel
    let channel_rows = sqlx::query(
        "SELECT channel, AVG(score) as avg_score, COUNT(*) as count
         FROM csat_surveys WHERE created_at >= ? AND created_at < ? AND status = 'responded' AND score IS NOT NULL
         GROUP BY channel"
    )
    .bind(&window_start)
    .bind(&window_end)
    .fetch_all(db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?;

    let mut by_channel = serde_json::Map::new();
    for row in &channel_rows {
        let channel: String = row.try_get("channel").unwrap_or("unknown".to_string());
        let avg: f64 = row.try_get("avg_score").unwrap_or(0.0);
        let count: i64 = row.try_get("count").unwrap_or(0);
        by_channel.insert(channel, json!({"avg": (avg * 100.0).round() / 100.0, "count": count}));
    }

    // Top comments
    let comment_rows = sqlx::query(
        "SELECT id, score, comment, contact_identifier, channel, responded_at
         FROM csat_surveys WHERE created_at >= ? AND created_at < ? AND status = 'responded'
         AND comment IS NOT NULL AND comment != ''
         ORDER BY responded_at DESC LIMIT 10"
    )
    .bind(&window_start)
    .bind(&window_end)
    .fetch_all(db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?;

    let top_comments: Vec<Value> = comment_rows
        .iter()
        .filter_map(|row| {
            Some(json!({
                "survey_id": row.try_get::<String, _>("id").ok()?,
                "score": row.try_get::<i32, _>("score").ok()?,
                "comment": row.try_get::<String, _>("comment").ok()?,
                "contact": row.try_get::<String, _>("contact_identifier").ok()?,
                "channel": row.try_get::<String, _>("channel").ok()?,
            }))
        })
        .collect();

    // Save snapshot
    sqlx::query(
        "INSERT INTO csat_snapshots (id, window_start, window_end, total_surveys_sent, total_responses,
         response_rate, average_score, csat_percentage, score_distribution, by_channel, top_comments)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(&id)
    .bind(&window_start)
    .bind(&window_end)
    .bind(total_sent as i32)
    .bind(total_responses as i32)
    .bind(response_rate)
    .bind(average_score)
    .bind(csat_percentage)
    .bind(&Value::Object(score_distribution).to_string())
    .bind(&Value::Object(by_channel).to_string())
    .bind(&Value::Array(top_comments).to_string())
    .execute(db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?;

    Ok(Json(CSATSnapshotResult {
        snapshot: CSATSnapshot {
            id,
            window_start,
            window_end,
            total_surveys_sent: total_sent as i32,
            total_responses: total_responses as i32,
            response_rate,
            average_score,
            csat_percentage,
            score_distribution: Some(Value::Object(serde_json::Map::new())),
            by_channel: Some(Value::Object(serde_json::Map::new())),
            by_template: None,
            top_comments: Some(Value::Array(Vec::new())),
            created_at: now.format("%Y-%m-%dT%H:%M:%fZ").to_string(),
        },
    }))
}

/// List historical snapshots
pub async fn list_snapshots(
    State(state): State<AppState>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let db = &state.db;
    let rows = sqlx::query(
        "SELECT id, window_start, window_end, total_surveys_sent, total_responses,
         response_rate, average_score, csat_percentage, score_distribution, by_channel,
         top_comments, created_at
         FROM csat_snapshots ORDER BY window_end DESC LIMIT 52"
    )
    .fetch_all(db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?;

    let snapshots = rows
        .iter()
        .map(|row| {
            json!({
                "id": row.try_get::<String, _>("id").ok(),
                "window_start": row.try_get::<String, _>("window_start").ok(),
                "window_end": row.try_get::<String, _>("window_end").ok(),
                "total_surveys_sent": row.try_get::<i32, _>("total_surveys_sent").ok(),
                "total_responses": row.try_get::<i32, _>("total_responses").ok(),
                "response_rate": row.try_get::<f64, _>("response_rate").ok(),
                "average_score": row.try_get::<Option<f64>, _>("average_score").ok(),
                "csat_percentage": row.try_get::<Option<f64>, _>("csat_percentage").ok(),
                "created_at": row.try_get::<String, _>("created_at").ok(),
            })
        })
        .collect::<Vec<_>>();

    Ok(Json(json!({
        "snapshots": snapshots,
        "total": snapshots.len(),
    })))
}

// ——— Unit tests ———

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_csat_summary_serialization() {
        let summary = CSATSummary {
            overall_csat: Some(85.5),
            overall_average_score: Some(4.2),
            total_surveys_sent: 100,
            total_responses: 50,
            response_rate: 50.0,
            score_distribution: json!({"1": 2, "2": 3, "3": 10, "4": 15, "5": 20}),
            by_channel: json!({"email": {"avg": 4.2, "count": 30}, "sms": {"avg": 3.8, "count": 20}}),
            trend_7d: Some(5.2),
            recent_comments: vec![],
        };
        let json = serde_json::to_string(&summary).unwrap();
        assert!(json.contains("85.5"));
        assert!(json.contains("4.2"));
    }

    #[test]
    fn test_csat_response_validation() {
        let response = CSATResponse {
            score: 5,
            comment: Some("Great service!".to_string()),
        };
        assert!(response.score >= 1 && response.score <= 5);
    }

    #[test]
    fn test_csat_query_params_defaults() {
        let params = CSATQueryParams::default();
        assert!(params.status.is_none());
        assert!(params.channel.is_none());
        assert_eq!(params.limit, None);
    }
}
