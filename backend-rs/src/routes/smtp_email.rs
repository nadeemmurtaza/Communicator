//! Thin gRPC client forwarding SMTP/email requests to `svc-smtp-email` (port 50092).

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Deserialize;
use serde_json::Value;

use crate::db::AppState;

use newax_proto::smtp_email::v1::{
    smtp_email_service_client::SmtpEmailServiceClient,
    Empty,
};

fn svc_addr() -> &'static str {
    option_env!("SMTP_EMAIL_GRPC").unwrap_or("http://127.0.0.1:50092")
}

async fn client() -> SmtpEmailServiceClient<tonic::transport::Channel> {
    SmtpEmailServiceClient::connect(svc_addr().to_string())
        .await
        .expect("smtp-email gRPC service unavailable")
}

fn grpc_json(s: String) -> Value {
    serde_json::from_str(&s).unwrap_or_else(|_| serde_json::json!({ "error": "invalid gRPC response" }))
}

fn ok_response(v: Value) -> Response {
    (StatusCode::OK, Json(v)).into_response()
}

fn err_response(status: StatusCode, message: &str) -> Response {
    (status, Json(serde_json::json!({ "error": message }))).into_response()
}

#[derive(Deserialize, Debug)]
pub struct TemplatesQuery {
    pub category: Option<String>,
    pub brand_id: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct SendLogsQuery {
    pub limit: Option<i64>,
}

#[derive(Deserialize, Debug)]
pub struct CreateSmtpPayload {
    pub brand_id: Option<String>,
    pub host: String,
    pub port: i64,
    pub security: String,
    pub auth_method: String,
    pub username: String,
    pub password: String,
    pub from_address: String,
    pub from_name: String,
    pub reply_to_address: Option<String>,
    pub connection_timeout_seconds: Option<i64>,
}

#[derive(Deserialize, Debug)]
pub struct UpdateSmtpPayload {
    pub host: Option<String>,
    pub port: Option<i64>,
    pub security: Option<String>,
    pub auth_method: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub from_address: Option<String>,
    pub from_name: Option<String>,
    pub reply_to_address: Option<String>,
    pub connection_timeout_seconds: Option<i64>,
    pub enabled: Option<bool>,
}

#[derive(Deserialize, Debug)]
pub struct TestSmtpPayload {
    pub to_address: String,
}

#[derive(Deserialize, Debug)]
pub struct CreateTemplatePayload {
    pub brand_id: Option<String>,
    pub name: String,
    pub subject: String,
    pub body_html: String,
    pub body_text: String,
    pub template_type: String,
    pub category: String,
    pub language: Option<String>,
    pub variables: Option<Value>,
}

#[derive(Deserialize, Debug)]
pub struct UpdateTemplatePayload {
    pub subject: Option<String>,
    pub body_html: Option<String>,
    pub body_text: Option<String>,
    pub template_type: Option<String>,
    pub category: Option<String>,
    pub language: Option<String>,
    pub enabled: Option<bool>,
    pub variables: Option<Value>,
}

#[derive(Deserialize, Debug)]
pub struct SendEmailPayload {
    pub to_address: String,
    pub template_name: String,
    pub variables: Option<std::collections::HashMap<String, String>>,
    pub brand_id: Option<String>,
}

pub async fn get_summary(State(_state): State<AppState>) -> Response {
    match client().await.get_summary(Empty {}).await {
        Ok(resp) => ok_response(grpc_json(resp.into_inner().json)),
        Err(e) => err_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

pub async fn list_smtp_settings(State(_state): State<AppState>) -> Response {
    match client().await.list_smtp_settings(Empty {}).await {
        Ok(resp) => ok_response(grpc_json(resp.into_inner().json)),
        Err(e) => err_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

pub async fn get_smtp_settings(
    State(_state): State<AppState>,
    Path(id): Path<String>,
) -> Response {
    let mut req = newax_proto::smtp_email::v1::GetSmtpSettingsRequest::default();
    req.id = id;
    match client().await.get_smtp_settings(req).await {
        Ok(resp) => ok_response(grpc_json(resp.into_inner().json)),
        Err(e) => err_response(StatusCode::NOT_FOUND, &e.to_string()),
    }
}

pub async fn create_smtp_settings(
    State(_state): State<AppState>,
    Json(body): Json<CreateSmtpPayload>,
) -> Response {
    let mut req = newax_proto::smtp_email::v1::CreateSmtpSettingsRequest::default();
    req.brand_id = body.brand_id.unwrap_or_default();
    req.host = body.host;
    req.port = body.port;
    req.security = body.security;
    req.auth_method = body.auth_method;
    req.username = body.username;
    req.password = body.password;
    req.from_address = body.from_address;
    req.from_name = body.from_name;
    req.reply_to_address = body.reply_to_address.unwrap_or_default();
    req.connection_timeout_seconds = body.connection_timeout_seconds.unwrap_or(30);
    match client().await.create_smtp_settings(req).await {
        Ok(resp) => ok_response(grpc_json(resp.into_inner().json)),
        Err(e) => err_response(StatusCode::BAD_REQUEST, &e.to_string()),
    }
}

pub async fn update_smtp_settings(
    State(_state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<UpdateSmtpPayload>,
) -> Response {
    let mut updates = serde_json::Map::new();
    if let Some(v) = body.host { updates.insert("host".into(), serde_json::Value::String(v)); }
    if let Some(v) = body.port { updates.insert("port".into(), serde_json::Value::Number(v.into())); }
    if let Some(v) = body.security { updates.insert("security".into(), serde_json::Value::String(v)); }
    if let Some(v) = body.auth_method { updates.insert("auth_method".into(), serde_json::Value::String(v)); }
    if let Some(v) = body.username { updates.insert("username".into(), serde_json::Value::String(v)); }
    if let Some(v) = body.password { updates.insert("password".into(), serde_json::Value::String(v)); }
    if let Some(v) = body.from_address { updates.insert("from_address".into(), serde_json::Value::String(v)); }
    if let Some(v) = body.from_name { updates.insert("from_name".into(), serde_json::Value::String(v)); }
    if let Some(v) = body.reply_to_address { updates.insert("reply_to_address".into(), serde_json::Value::String(v)); }
    if let Some(v) = body.connection_timeout_seconds { updates.insert("connection_timeout_seconds".into(), serde_json::Value::Number(v.into())); }
    if let Some(v) = body.enabled { updates.insert("enabled".into(), serde_json::Value::Bool(v)); }
    let mut req = newax_proto::smtp_email::v1::UpdateSmtpSettingsRequest::default();
    req.id = id;
    req.updates_json = serde_json::to_string(&serde_json::Value::Object(updates)).unwrap_or_default();
    match client().await.update_smtp_settings(req).await {
        Ok(resp) => ok_response(grpc_json(resp.into_inner().json)),
        Err(e) => err_response(StatusCode::BAD_REQUEST, &e.to_string()),
    }
}

pub async fn delete_smtp_settings(
    State(_state): State<AppState>,
    Path(id): Path<String>,
) -> Response {
    let mut req = newax_proto::smtp_email::v1::DeleteSmtpSettingsRequest::default();
    req.id = id;
    match client().await.delete_smtp_settings(req).await {
        Ok(resp) => ok_response(grpc_json(resp.into_inner().json)),
        Err(e) => err_response(StatusCode::BAD_REQUEST, &e.to_string()),
    }
}

pub async fn test_smtp_connection(
    State(_state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<TestSmtpPayload>,
) -> Response {
    let mut req = newax_proto::smtp_email::v1::TestSmtpConnectionRequest::default();
    req.id = id;
    req.to_address = body.to_address;
    match client().await.test_smtp_connection(req).await {
        Ok(resp) => ok_response(grpc_json(resp.into_inner().json)),
        Err(e) => err_response(StatusCode::BAD_REQUEST, &e.to_string()),
    }
}

pub async fn list_email_templates(
    State(_state): State<AppState>,
    Query(params): Query<TemplatesQuery>,
) -> Response {
    let mut req = newax_proto::smtp_email::v1::ListEmailTemplatesRequest::default();
    req.category = params.category.clone().unwrap_or_default();
    req.brand_id = params.brand_id.clone().unwrap_or_default();
    match client().await.list_email_templates(req).await {
        Ok(resp) => ok_response(grpc_json(resp.into_inner().json)),
        Err(e) => err_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

pub async fn create_email_template(
    State(_state): State<AppState>,
    Json(body): Json<CreateTemplatePayload>,
) -> Response {
    let mut req = newax_proto::smtp_email::v1::CreateEmailTemplateRequest::default();
    req.brand_id = body.brand_id.unwrap_or_default();
    req.name = body.name;
    req.subject = body.subject;
    req.body_html = body.body_html;
    req.body_text = body.body_text;
    req.template_type = body.template_type;
    req.category = body.category;
    req.language = body.language.unwrap_or_else(|| "en".to_string());
    req.variables_json = serde_json::to_string(&body.variables.unwrap_or_else(|| serde_json::json!({}))).unwrap_or_default();
    match client().await.create_email_template(req).await {
        Ok(resp) => ok_response(grpc_json(resp.into_inner().json)),
        Err(e) => err_response(StatusCode::BAD_REQUEST, &e.to_string()),
    }
}

pub async fn get_email_template(
    State(_state): State<AppState>,
    Path(id): Path<String>,
) -> Response {
    let mut req = newax_proto::smtp_email::v1::GetEmailTemplateRequest::default();
    req.id = id;
    match client().await.get_email_template(req).await {
        Ok(resp) => ok_response(grpc_json(resp.into_inner().json)),
        Err(e) => err_response(StatusCode::NOT_FOUND, &e.to_string()),
    }
}

pub async fn update_email_template(
    State(_state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<UpdateTemplatePayload>,
) -> Response {
    let mut updates = serde_json::Map::new();
    if let Some(v) = body.subject { updates.insert("subject".into(), serde_json::Value::String(v)); }
    if let Some(v) = body.body_html { updates.insert("body_html".into(), serde_json::Value::String(v)); }
    if let Some(v) = body.body_text { updates.insert("body_text".into(), serde_json::Value::String(v)); }
    if let Some(v) = body.template_type { updates.insert("template_type".into(), serde_json::Value::String(v)); }
    if let Some(v) = body.category { updates.insert("category".into(), serde_json::Value::String(v)); }
    if let Some(v) = body.language { updates.insert("language".into(), serde_json::Value::String(v)); }
    if let Some(v) = body.enabled { updates.insert("enabled".into(), serde_json::Value::Bool(v)); }
    if let Some(v) = body.variables { updates.insert("variables".into(), v); }
    let mut req = newax_proto::smtp_email::v1::UpdateEmailTemplateRequest::default();
    req.id = id;
    req.updates_json = serde_json::to_string(&serde_json::Value::Object(updates)).unwrap_or_default();
    match client().await.update_email_template(req).await {
        Ok(resp) => ok_response(grpc_json(resp.into_inner().json)),
        Err(e) => err_response(StatusCode::BAD_REQUEST, &e.to_string()),
    }
}

pub async fn delete_email_template(
    State(_state): State<AppState>,
    Path(id): Path<String>,
) -> Response {
    let mut req = newax_proto::smtp_email::v1::DeleteEmailTemplateRequest::default();
    req.id = id;
    match client().await.delete_email_template(req).await {
        Ok(resp) => ok_response(grpc_json(resp.into_inner().json)),
        Err(e) => err_response(StatusCode::BAD_REQUEST, &e.to_string()),
    }
}

pub async fn send_email(
    State(_state): State<AppState>,
    Json(body): Json<SendEmailPayload>,
) -> Response {
    let mut req = newax_proto::smtp_email::v1::SendEmailRequest::default();
    req.to_address = body.to_address;
    req.template_name = body.template_name;
    req.brand_id = body.brand_id.unwrap_or_default();
    if let Some(vars) = body.variables {
        let map: serde_json::Map<String, serde_json::Value> = vars.into_iter()
            .map(|(k, v)| (k, serde_json::Value::String(v)))
            .collect();
        req.variables_json = serde_json::to_string(&serde_json::Value::Object(map)).unwrap_or_default();
    }
    match client().await.send_email(req).await {
        Ok(resp) => ok_response(grpc_json(resp.into_inner().json)),
        Err(e) => err_response(StatusCode::BAD_REQUEST, &e.to_string()),
    }
}

pub async fn get_send_logs(
    State(_state): State<AppState>,
    Query(params): Query<SendLogsQuery>,
) -> Response {
    let mut req = newax_proto::smtp_email::v1::GetSendLogsRequest::default();
    req.limit = params.limit.unwrap_or(50);
    match client().await.get_send_logs(req).await {
        Ok(resp) => ok_response(grpc_json(resp.into_inner().json)),
        Err(e) => err_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}
