//! CSAT Surveys service Ã¢â‚¬â€ bounded context.
//!
//! Provides survey template CRUD, survey creation/responses, and analytics
//! (summary, snapshots, trends).

use sqlx::{PgPool, Row};
use serde_json::{json, Value};
use chrono::{Duration, Utc};
use uuid::Uuid;

/// Application state for the CSAT surveys service.
#[derive(Clone)]
pub struct CSATState {
    pub pool: PgPool,
}

impl CSATState {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

// ---------------------------------------------------------------------------
// Query params
// ---------------------------------------------------------------------------

#[derive(Default)]
pub struct CSATQueryParams {
    pub status: Option<String>,
    pub channel: Option<String>,
    pub template_id: Option<String>,
    pub conversation_id: Option<String>,
    pub contact_id: Option<String>,
    pub limit: Option<i32>,
    pub offset: Option<i32>,
}

fn parse_query_params(json_str: &str) -> CSATQueryParams {
    let mut params = CSATQueryParams::default();
    if let Ok(val) = serde_json::from_str::<Value>(json_str) {
        if let Some(obj) = val.as_object() {
            params.status = obj.get("status").and_then(|v| v.as_str()).map(String::from);
            params.channel = obj.get("channel").and_then(|v| v.as_str()).map(String::from);
            params.template_id = obj.get("template_id").and_then(|v| v.as_str()).map(String::from);
            params.conversation_id = obj.get("conversation_id").and_then(|v| v.as_str()).map(String::from);
            params.contact_id = obj.get("contact_id").and_then(|v| v.as_str()).map(String::from);
            params.limit = obj.get("limit").and_then(|v| v.as_i64()).map(|v| v as i32);
            params.offset = obj.get("offset").and_then(|v| v.as_i64()).map(|v| v as i32);
        }
    }
    params
}

// ---------------------------------------------------------------------------
// Template CRUD
// ---------------------------------------------------------------------------

pub async fn list_templates(state: &CSATState, params: &CSATQueryParams) -> Result<Value, String> {
    let mut query = r#"
        SELECT id, name, description, question, scale_type, min_score, max_score,
               include_comment, comment_prompt, trigger_type, trigger_delay_minutes,
               channel_filter, enabled, created_at, updated_at
        FROM csat_survey_templates
        WHERE 1=1
    "#.to_string();

    if let Some(ref enabled_filter) = params.status {
        if enabled_filter == "enabled" {
            query.push_str(" AND enabled = 1");
        } else if enabled_filter == "disabled" {
            query.push_str(" AND enabled = 0");
        }
    }
    query.push_str(" ORDER BY created_at DESC");

    let rows = if let Some(limit) = params.limit {
        let q = format!("{} LIMIT {}", query, limit);
        sqlx::query(&q).fetch_all(&state.pool).await.map_err(|e| e.to_string())?
    } else {
        sqlx::query(&query).fetch_all(&state.pool).await.map_err(|e| e.to_string())?
    };

    let templates: Vec<Value> = rows.iter().map(|row| template_json(row)).collect();
    Ok(json!({
        "templates": templates,
        "total": templates.len(),
    }))
}

pub async fn get_template(state: &CSATState, id: &str) -> Result<Value, String> {
    let row = sqlx::query(
        "SELECT id, name, description, question, scale_type, min_score, max_score,
         include_comment, comment_prompt, trigger_type, trigger_delay_minutes,
         channel_filter, enabled, created_at, updated_at
         FROM csat_survey_templates WHERE id = $1"
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await.map_err(|e| e.to_string())?;

    match row {
        Some(row) => Ok(template_json(&row)),
        None => Err("Template not found".to_string()),
    }
}

pub async fn create_template(state: &CSATState, body: &Value) -> Result<Value, String> {
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
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)"
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
    .execute(&state.pool)
    .await.map_err(|e| e.to_string())?;

    Ok(json!({
        "id": id,
        "name": name,
        "question": question,
        "scale_type": scale_type,
        "enabled": enabled,
        "message": "Template created successfully",
    }))
}

pub async fn update_template(state: &CSATState, id: &str, body: &Value) -> Result<Value, String> {
    let exists = sqlx::query("SELECT COUNT(*) FROM csat_survey_templates WHERE id = $1")
        .bind(id)
        .fetch_one(&state.pool)
        .await.map_err(|e| e.to_string())?;

    if exists.try_get::<i64, _>(0).unwrap_or(0) == 0 {
        return Err("Template not found".to_string());
    }

    let mut sets: Vec<String> = Vec::new();
    let mut binds: Vec<Value> = Vec::new();
    let mut idx = 1;

    if let Some(name) = body["name"].as_str() {
        sets.push(format!("name = ${idx}"));
        binds.push(json!(name));
        idx += 1;
    }
    if let Some(description) = body["description"].as_str() {
        sets.push(format!("description = ${idx}"));
        binds.push(json!(description));
        idx += 1;
    }
    if let Some(question) = body["question"].as_str() {
        sets.push(format!("question = ${idx}"));
        binds.push(json!(question));
        idx += 1;
    }
    if let Some(scale_type) = body["scale_type"].as_str() {
        sets.push(format!("scale_type = ${idx}"));
        binds.push(json!(scale_type));
        idx += 1;
    }
    if let Some(min_score) = body["min_score"].as_i64() {
        sets.push(format!("min_score = ${idx}"));
        binds.push(json!(min_score));
        idx += 1;
    }
    if let Some(max_score) = body["max_score"].as_i64() {
        sets.push(format!("max_score = ${idx}"));
        binds.push(json!(max_score));
        idx += 1;
    }
    if let Some(include_comment) = body["include_comment"].as_bool() {
        sets.push(format!("include_comment = ${idx}"));
        binds.push(json!(include_comment));
        idx += 1;
    }
    if let Some(comment_prompt) = body["comment_prompt"].as_str() {
        sets.push(format!("comment_prompt = ${idx}"));
        binds.push(json!(comment_prompt));
        idx += 1;
    }
    if let Some(trigger_type) = body["trigger_type"].as_str() {
        sets.push(format!("trigger_type = ${idx}"));
        binds.push(json!(trigger_type));
        idx += 1;
    }
    if let Some(trigger_delay) = body["trigger_delay_minutes"].as_i64() {
        sets.push(format!("trigger_delay_minutes = ${idx}"));
        binds.push(json!(trigger_delay));
        idx += 1;
    }
    if let Some(channel_filter) = body["channel_filter"].as_str() {
        sets.push(format!("channel_filter = ${idx}"));
        binds.push(json!(channel_filter));
        idx += 1;
    }
    if let Some(enabled) = body["enabled"].as_bool() {
        sets.push(format!("enabled = ${idx}"));
        binds.push(json!(enabled));
        idx += 1;
    }

    if sets.is_empty() {
        return Ok(json!({"message": "No fields to update"}));
    }

    sets.push(format!("updated_at = ${idx}"));
    idx += 1;
    binds.push(json!(Utc::now().format("%Y-%m-%dT%H:%M:%fZ").to_string()));
    binds.push(json!(id));

    let sql = format!("UPDATE csat_survey_templates SET {} WHERE id = ${idx}", sets.join(", "));

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

    query_builder.execute(&state.pool).await.map_err(|e| e.to_string())?;

    Ok(json!({
        "id": id,
        "message": "Template updated successfully",
    }))
}

pub async fn delete_template(state: &CSATState, id: &str) -> Result<Value, String> {
    sqlx::query("DELETE FROM csat_survey_templates WHERE id = $1")
        .bind(id)
        .execute(&state.pool)
        .await.map_err(|e| e.to_string())?;

    Ok(json!({"message": "Template deleted successfully"}))
}

fn template_json(row: &sqlx::postgres::PgRow) -> Value {
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
}

// ---------------------------------------------------------------------------
// Survey operations
// ---------------------------------------------------------------------------

pub async fn list_surveys(state: &CSATState, params: &CSATQueryParams) -> Result<Value, String> {
    let mut query = r#"
        SELECT s.id, s.template_id, s.conversation_id, s.contact_id, s.channel,
               s.contact_identifier, s.status, s.score, s.comment, s.sent_at,
               s.responded_at, s.expires_at, s.created_at
        FROM csat_surveys s
        WHERE 1=1
    "#.to_string();
    let mut binds: Vec<String> = Vec::new();
    let mut idx = 1;

    if let Some(ref status) = params.status {
        query.push_str(&format!(" AND s.status = ${idx}"));
        binds.push(status.clone());
        idx += 1;
    }
    if let Some(ref channel) = params.channel {
        query.push_str(&format!(" AND s.channel = ${idx}"));
        binds.push(channel.clone());
        idx += 1;
    }
    if let Some(ref template_id) = params.template_id {
        query.push_str(&format!(" AND s.template_id = ${idx}"));
        binds.push(template_id.clone());
        idx += 1;
    }
    if let Some(ref conversation_id) = params.conversation_id {
        query.push_str(&format!(" AND s.conversation_id = ${idx}"));
        binds.push(conversation_id.clone());
        idx += 1;
    }
    if let Some(ref contact_id) = params.contact_id {
        query.push_str(&format!(" AND s.contact_id = ${idx}"));
        binds.push(contact_id.clone());
        idx += 1;
    }

    query.push_str(" ORDER BY s.created_at DESC");

    let limit = params.limit.unwrap_or(50);
    let offset = params.offset.unwrap_or(0);
    query.push_str(&format!(" LIMIT ${idx} OFFSET ${}", idx + 1));
    binds.push(limit.to_string());
    binds.push(offset.to_string());

    let mut qb = sqlx::query(&query);
    for b in &binds {
        qb = qb.bind(b.as_str());
    }

    let rows = qb.fetch_all(&state.pool).await.map_err(|e| e.to_string())?;
    let surveys: Vec<Value> = rows.iter().map(|row| {
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
    }).collect();

    Ok(json!({
        "surveys": surveys,
        "total": surveys.len(),
    }))
}

pub async fn create_survey(state: &CSATState, body: &Value) -> Result<Value, String> {
    let id = Uuid::new_v4().to_string();
    let template_id = body["template_id"].as_str().unwrap_or("default").to_string();
    let conversation_id = body["conversation_id"].as_str()
        .ok_or_else(|| "conversation_id is required".to_string())?.to_string();
    let contact_id = body["contact_id"].as_str()
        .ok_or_else(|| "contact_id is required".to_string())?.to_string();
    let channel = body["channel"].as_str().unwrap_or("email").to_string();
    let contact_identifier = body["contact_identifier"].as_str().unwrap_or("").to_string();

    let template = sqlx::query(
        "SELECT trigger_delay_minutes FROM csat_survey_templates WHERE id = $1"
    )
    .bind(&template_id)
    .fetch_optional(&state.pool)
    .await.map_err(|e| e.to_string())?;

    let delay_minutes = template.and_then(|r| r.try_get::<i32, _>("trigger_delay_minutes").ok()).unwrap_or(0);
    let expires_at = if delay_minutes > 0 {
        let expires = Utc::now() + Duration::minutes(delay_minutes as i64 * 60);
        Some(expires.format("%Y-%m-%dT%H:%M:%fZ").to_string())
    } else {
        let expires = Utc::now() + Duration::days(7);
        Some(expires.format("%Y-%m-%dT%H:%M:%fZ").to_string())
    };

    sqlx::query(
        "INSERT INTO csat_surveys (id, template_id, conversation_id, contact_id, channel,
         contact_identifier, status, expires_at)
         VALUES ($1, $2, $3, $4, $5, $6, 'pending', $7)"
    )
    .bind(&id)
    .bind(&template_id)
    .bind(&conversation_id)
    .bind(&contact_id)
    .bind(&channel)
    .bind(&contact_identifier)
    .bind(&expires_at)
    .execute(&state.pool)
    .await.map_err(|e| e.to_string())?;

    Ok(json!({
        "id": id,
        "template_id": template_id,
        "conversation_id": conversation_id,
        "status": "pending",
        "expires_at": expires_at,
        "message": "Survey created successfully",
    }))
}

pub async fn respond_to_survey(state: &CSATState, id: &str, body: &Value) -> Result<Value, String> {
    let survey = sqlx::query(
        "SELECT id, status, template_id FROM csat_surveys WHERE id = $1"
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await.map_err(|e| e.to_string())?;

    let survey = match survey {
        Some(s) => s,
        None => return Err("Survey not found".to_string()),
    };

    let status: String = survey.try_get("status").unwrap_or("pending".to_string());
    if status == "responded" {
        return Err("Survey already responded".to_string());
    }
    if status == "expired" {
        return Err("Survey has expired".to_string());
    }

    let template_id: String = survey.try_get("template_id").unwrap_or("default".to_string());
    let template = sqlx::query(
        "SELECT min_score, max_score FROM csat_survey_templates WHERE id = $1"
    )
    .bind(&template_id)
    .fetch_optional(&state.pool)
    .await.map_err(|e| e.to_string())?;

    let score = body["score"].as_i64().unwrap_or(0) as i32;
    if let Some(t) = &template {
        let min_s: i32 = t.try_get("min_score").unwrap_or(1);
        let max_s: i32 = t.try_get("max_score").unwrap_or(5);
        if score < min_s || score > max_s {
            return Err(format!("Score must be between {} and {}", min_s, max_s));
        }
    }

    let comment = body["comment"].as_str().map(String::from);
    let now = Utc::now().format("%Y-%m-%dT%H:%M:%fZ").to_string();

    sqlx::query(
        "UPDATE csat_surveys SET status = 'responded', score = $1, comment = $2, responded_at = $3
         WHERE id = $4"
    )
    .bind(score)
    .bind(&comment)
    .bind(&now)
    .bind(id)
    .execute(&state.pool)
    .await.map_err(|e| e.to_string())?;

    Ok(json!({
        "id": id,
        "score": score,
        "comment": comment,
        "responded_at": now,
        "message": "Response recorded successfully",
    }))
}

pub async fn send_survey(state: &CSATState, id: &str) -> Result<Value, String> {
    let now = Utc::now().format("%Y-%m-%dT%H:%M:%fZ").to_string();

    let result = sqlx::query(
        "UPDATE csat_surveys SET status = 'sent', sent_at = $1
         WHERE id = $2 AND status = 'pending'"
    )
    .bind(&now)
    .bind(id)
    .execute(&state.pool)
    .await.map_err(|e| e.to_string())?;

    if result.rows_affected() == 0 {
        return Err("Survey not found or already sent".to_string());
    }

    Ok(json!({
        "id": id,
        "status": "sent",
        "sent_at": now,
        "message": "Survey marked as sent",
    }))
}

// ---------------------------------------------------------------------------
// Analytics
// ---------------------------------------------------------------------------

pub async fn get_csat_summary(state: &CSATState, params: &CSATQueryParams) -> Result<Value, String> {
    let days = params.limit.unwrap_or(30);
    let window_start = (Utc::now() - Duration::days(days as i64)).format("%Y-%m-%dT%H:%M:%fZ").to_string();

    let total_sent: i64 = sqlx::query(
        "SELECT COUNT(*) FROM csat_surveys WHERE created_at >= $1 AND status != 'cancelled'"
    )
    .bind(&window_start)
    .fetch_one(&state.pool)
    .await.map_err(|e| e.to_string())?
    .try_get(0).unwrap_or(0);

    let total_responses: i64 = sqlx::query(
        "SELECT COUNT(*) FROM csat_surveys WHERE created_at >= $1 AND status = 'responded'"
    )
    .bind(&window_start)
    .fetch_one(&state.pool)
    .await.map_err(|e| e.to_string())?
    .try_get(0).unwrap_or(0);

    let avg_row = sqlx::query(
        "SELECT AVG(score) as avg_score FROM csat_surveys WHERE created_at >= $1 AND status = 'responded' AND score IS NOT NULL"
    )
    .bind(&window_start)
    .fetch_optional(&state.pool)
    .await.map_err(|e| e.to_string())?;

    let average_score = avg_row.and_then(|r| r.try_get::<f64, _>("avg_score").ok());

    let csat_row = sqlx::query(
        "SELECT COUNT(*) FROM csat_surveys WHERE created_at >= $1 AND status = 'responded' AND score >= 4"
    )
    .bind(&window_start)
    .fetch_one(&state.pool)
    .await.map_err(|e| e.to_string())?;

    let csat_count: i64 = csat_row.try_get(0).unwrap_or(0);
    let csat_percentage = if total_responses > 0 {
        Some((csat_count as f64 / total_responses as f64) * 100.0)
    } else {
        None
    };

    let dist_rows = sqlx::query(
        "SELECT score, COUNT(*) as count FROM csat_surveys
         WHERE created_at >= $1 AND status = 'responded' AND score IS NOT NULL
         GROUP BY score ORDER BY score"
    )
    .bind(&window_start)
    .fetch_all(&state.pool)
    .await.map_err(|e| e.to_string())?;

    let mut score_distribution = serde_json::Map::new();
    for row in &dist_rows {
        let score: i32 = row.try_get("score").unwrap_or(0);
        let count: i64 = row.try_get("count").unwrap_or(0);
        score_distribution.insert(score.to_string(), json!(count));
    }

    let channel_rows = sqlx::query(
        "SELECT channel, AVG(score) as avg_score, COUNT(*) as count
         FROM csat_surveys WHERE created_at >= $1 AND status = 'responded' AND score IS NOT NULL
         GROUP BY channel"
    )
    .bind(&window_start)
    .fetch_all(&state.pool)
    .await.map_err(|e| e.to_string())?;

    let mut by_channel = serde_json::Map::new();
    for row in &channel_rows {
        let channel: String = row.try_get("channel").unwrap_or("unknown".to_string());
        let avg: f64 = row.try_get("avg_score").unwrap_or(0.0);
        let count: i64 = row.try_get("count").unwrap_or(0);
        by_channel.insert(channel, json!({"avg": (avg * 100.0).round() / 100.0, "count": count}));
    }

    let comment_rows = sqlx::query(
        "SELECT s.id, s.score, s.comment, s.contact_identifier, s.channel, s.responded_at
         FROM csat_surveys s
         WHERE created_at >= $1 AND status = 'responded' AND s.comment IS NOT NULL AND s.comment != ''
         ORDER BY s.responded_at DESC LIMIT 10"
    )
    .bind(&window_start)
    .fetch_all(&state.pool)
    .await.map_err(|e| e.to_string())?;

    let recent_comments: Vec<Value> = comment_rows
        .iter()
        .filter_map(|row| {
            Some(json!({
                "survey_id": row.try_get::<String, _>("id").ok()?,
                "score": row.try_get::<i32, _>("score").ok()?,
                "comment": row.try_get::<String, _>("comment").ok()?,
                "contact_identifier": row.try_get::<String, _>("contact_identifier").ok()?,
                "channel": row.try_get::<String, _>("channel").ok()?,
                "responded_at": row.try_get::<String, _>("responded_at").ok()?,
            }))
        })
        .collect();

    let trend_7d = compute_trend(&state.pool).await.ok();

    let response_rate = if total_sent > 0 {
        (total_responses as f64 / total_sent as f64) * 100.0
    } else {
        0.0
    };

    Ok(json!({
        "overall_csat": csat_percentage,
        "overall_average_score": average_score,
        "total_surveys_sent": total_sent as i32,
        "total_responses": total_responses as i32,
        "response_rate": (response_rate * 100.0).round() / 100.0,
        "score_distribution": Value::Object(score_distribution),
        "by_channel": Value::Object(by_channel),
        "trend_7d": trend_7d,
        "recent_comments": recent_comments,
    }))
}

async fn compute_trend(pool: &PgPool) -> Result<f64, String> {
    let now = Utc::now();
    let week_ago = (now - Duration::days(7)).format("%Y-%m-%dT%H:%M:%fZ").to_string();
    let two_weeks_ago = (now - Duration::days(14)).format("%Y-%m-%dT%H:%M:%fZ").to_string();

    let current = sqlx::query(
        "SELECT AVG(score) FROM csat_surveys WHERE created_at >= $1 AND created_at < $2 AND status = 'responded' AND score IS NOT NULL"
    )
    .bind(&week_ago)
    .bind(&now.format("%Y-%m-%dT%H:%M:%fZ").to_string())
    .fetch_optional(pool)
    .await.map_err(|e| e.to_string())?;

    let previous = sqlx::query(
        "SELECT AVG(score) FROM csat_surveys WHERE created_at >= $1 AND created_at < $2 AND status = 'responded' AND score IS NOT NULL"
    )
    .bind(&two_weeks_ago)
    .bind(&week_ago)
    .fetch_optional(pool)
    .await.map_err(|e| e.to_string())?;

    let current_avg = current.and_then(|r| r.try_get::<f64, _>(0).ok());
    let previous_avg = previous.and_then(|r| r.try_get::<f64, _>(0).ok());

    match (current_avg, previous_avg) {
        (Some(c), Some(p)) if p > 0.0 => Ok(((c - p) / p) * 100.0),
        _ => Ok(0.0),
    }
}

pub async fn generate_snapshot(state: &CSATState) -> Result<Value, String> {
    let id = Uuid::new_v4().to_string();
    let now = Utc::now();
    let window_start = (now - Duration::days(7)).format("%Y-%m-%dT%H:%M:%fZ").to_string();
    let window_end = now.format("%Y-%m-%dT%H:%M:%fZ").to_string();

    let total_sent: i64 = sqlx::query(
        "SELECT COUNT(*) FROM csat_surveys WHERE created_at >= $1 AND created_at < $2 AND status != 'cancelled'"
    )
    .bind(&window_start)
    .bind(&window_end)
    .fetch_one(&state.pool)
    .await.map_err(|e| e.to_string())?
    .try_get(0).unwrap_or(0);

    let total_responses: i64 = sqlx::query(
        "SELECT COUNT(*) FROM csat_surveys WHERE created_at >= $1 AND created_at < $2 AND status = 'responded'"
    )
    .bind(&window_start)
    .bind(&window_end)
    .fetch_one(&state.pool)
    .await.map_err(|e| e.to_string())?
    .try_get(0).unwrap_or(0);

    let avg_row = sqlx::query(
        "SELECT AVG(score) as avg_score FROM csat_surveys WHERE created_at >= $1 AND created_at < $2 AND status = 'responded' AND score IS NOT NULL"
    )
    .bind(&window_start)
    .bind(&window_end)
    .fetch_optional(&state.pool)
    .await.map_err(|e| e.to_string())?;

    let average_score = avg_row.and_then(|r| r.try_get::<f64, _>("avg_score").ok());

    let csat_count: i64 = sqlx::query(
        "SELECT COUNT(*) FROM csat_surveys WHERE created_at >= $1 AND created_at < $2 AND status = 'responded' AND score >= 4"
    )
    .bind(&window_start)
    .bind(&window_end)
    .fetch_one(&state.pool)
    .await.map_err(|e| e.to_string())?
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

    let dist_rows = sqlx::query(
        "SELECT score, COUNT(*) as count FROM csat_surveys
         WHERE created_at >= $1 AND created_at < $2 AND status = 'responded' AND score IS NOT NULL
         GROUP BY score ORDER BY score"
    )
    .bind(&window_start)
    .bind(&window_end)
    .fetch_all(&state.pool)
    .await.map_err(|e| e.to_string())?;

    let mut score_distribution = serde_json::Map::new();
    for row in &dist_rows {
        let score: i32 = row.try_get("score").unwrap_or(0);
        let count: i64 = row.try_get("count").unwrap_or(0);
        score_distribution.insert(score.to_string(), json!(count));
    }

    let channel_rows = sqlx::query(
        "SELECT channel, AVG(score) as avg_score, COUNT(*) as count
         FROM csat_surveys WHERE created_at >= $1 AND created_at < $2 AND status = 'responded' AND score IS NOT NULL
         GROUP BY channel"
    )
    .bind(&window_start)
    .bind(&window_end)
    .fetch_all(&state.pool)
    .await.map_err(|e| e.to_string())?;

    let mut by_channel = serde_json::Map::new();
    for row in &channel_rows {
        let channel: String = row.try_get("channel").unwrap_or("unknown".to_string());
        let avg: f64 = row.try_get("avg_score").unwrap_or(0.0);
        let count: i64 = row.try_get("count").unwrap_or(0);
        by_channel.insert(channel, json!({"avg": (avg * 100.0).round() / 100.0, "count": count}));
    }

    let comment_rows = sqlx::query(
        "SELECT id, score, comment, contact_identifier, channel, responded_at
         FROM csat_surveys WHERE created_at >= $1 AND created_at < $2 AND status = 'responded'
         AND comment IS NOT NULL AND comment != ''
         ORDER BY responded_at DESC LIMIT 10"
    )
    .bind(&window_start)
    .bind(&window_end)
    .fetch_all(&state.pool)
    .await.map_err(|e| e.to_string())?;

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

    sqlx::query(
        "INSERT INTO csat_snapshots (id, window_start, window_end, total_surveys_sent, total_responses,
         response_rate, average_score, csat_percentage, score_distribution, by_channel, top_comments)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)"
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
    .execute(&state.pool)
    .await.map_err(|e| e.to_string())?;

    Ok(json!({
        "snapshot": {
            "id": id,
            "window_start": window_start,
            "window_end": window_end,
            "total_surveys_sent": total_sent as i32,
            "total_responses": total_responses as i32,
            "response_rate": response_rate,
            "average_score": average_score,
            "csat_percentage": csat_percentage,
            "created_at": now.format("%Y-%m-%dT%H:%M:%fZ").to_string(),
        }
    }))
}

pub async fn list_snapshots(state: &CSATState) -> Result<Value, String> {
    let rows = sqlx::query(
        "SELECT id, window_start, window_end, total_surveys_sent, total_responses,
         response_rate, average_score, csat_percentage, score_distribution, by_channel,
         top_comments, created_at
         FROM csat_snapshots ORDER BY window_end DESC LIMIT 52"
    )
    .fetch_all(&state.pool)
    .await.map_err(|e| e.to_string())?;

    let snapshots: Vec<Value> = rows.iter().map(|row| {
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
    }).collect();

    Ok(json!({
        "snapshots": snapshots,
        "total": snapshots.len(),
    }))
}
