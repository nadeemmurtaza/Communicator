use newax_smtp_email as smtp_email;
use sqlx::postgres::PgPoolOptions;
use tonic::{transport::Server, Request, Response, Status};
use tracing::info;

pub mod proto {
    tonic::include_proto!("newax.smtp_email.v1");
}

use proto::{
    smtp_email_service_server::{SmtpEmailService, SmtpEmailServiceServer},
    Empty, JsonResponse,
};

struct Svc {
    state: smtp_email::SmtpEmailState,
}

fn to_status(e: impl std::fmt::Display) -> Status {
    Status::internal(e.to_string())
}

fn json_response(v: serde_json::Value) -> JsonResponse {
    JsonResponse { json: v.to_string() }
}

fn parse_json(s: &str) -> Result<serde_json::Value, Status> {
    serde_json::from_str(s).map_err(|e| Status::invalid_argument(e.to_string()))
}

#[tonic::async_trait]
impl SmtpEmailService for Svc {
    async fn get_summary(&self, _req: Request<Empty>) -> Result<Response<JsonResponse>, Status> {
        let v = smtp_email::get_smtp_summary(&self.state.db).await.map_err(to_status)?;
        Ok(Response::new(json_response(serde_json::to_value(v).map_err(to_status)?)))
    }

    async fn list_smtp_settings(&self, _req: Request<Empty>) -> Result<Response<JsonResponse>, Status> {
        let v = smtp_email::SmtpSettings::list_all(&self.state.db).await.map_err(to_status)?;
        Ok(Response::new(json_response(serde_json::to_value(v).map_err(to_status)?)))
    }

    async fn get_smtp_settings(
        &self,
        req: Request<proto::GetSmtpSettingsRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let settings = smtp_email::SmtpSettings::list_all(&self.state.db).await.map_err(to_status)?;
        let setting = settings.into_iter().find(|s| s.id == r.id)
            .ok_or_else(|| Status::not_found("SMTP settings not found"))?;
        Ok(Response::new(json_response(serde_json::to_value(setting).map_err(to_status)?)))
    }

    async fn create_smtp_settings(
        &self,
        req: Request<proto::CreateSmtpSettingsRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        let brand_id = if r.brand_id.is_empty() { None } else { Some(r.brand_id) };
        let reply_to = if r.reply_to_address.is_empty() { None } else { Some(r.reply_to_address) };
        let timeout = if r.connection_timeout_seconds == 0 { 30 } else { r.connection_timeout_seconds };
        let settings = smtp_email::SmtpSettings {
            id,
            brand_id,
            host: r.host,
            port: r.port,
            security: r.security,
            auth_method: r.auth_method,
            username: r.username,
            password: r.password,
            from_address: r.from_address,
            from_name: r.from_name,
            reply_to_address: reply_to,
            connection_timeout_seconds: timeout,
            enabled: true,
            test_sent_at: None,
            test_status: None,
            test_error: None,
            created_at: now.clone(),
            updated_at: now,
        };
        let v = smtp_email::SmtpSettings::create(&self.state.db, &settings).await.map_err(to_status)?;
        Ok(Response::new(json_response(serde_json::json!({ "id": v.id }))))
    }

    async fn update_smtp_settings(
        &self,
        req: Request<proto::UpdateSmtpSettingsRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let settings = smtp_email::SmtpSettings::list_all(&self.state.db).await.map_err(to_status)?;
        let mut setting = settings.into_iter().find(|s| s.id == r.id)
            .ok_or_else(|| Status::not_found("SMTP settings not found"))?;
        let updates = parse_json(&r.updates_json)?;
        if let Some(v) = updates.get("host").and_then(|x| x.as_str()) { setting.host = v.to_string(); }
        if let Some(v) = updates.get("port").and_then(|x| x.as_i64()) { setting.port = v; }
        if let Some(v) = updates.get("security").and_then(|x| x.as_str()) { setting.security = v.to_string(); }
        if let Some(v) = updates.get("auth_method").and_then(|x| x.as_str()) { setting.auth_method = v.to_string(); }
        if let Some(v) = updates.get("username").and_then(|x| x.as_str()) { setting.username = v.to_string(); }
        if let Some(v) = updates.get("password").and_then(|x| x.as_str()) { setting.password = v.to_string(); }
        if let Some(v) = updates.get("from_address").and_then(|x| x.as_str()) { setting.from_address = v.to_string(); }
        if let Some(v) = updates.get("from_name").and_then(|x| x.as_str()) { setting.from_name = v.to_string(); }
        if let Some(v) = updates.get("reply_to_address").and_then(|x| x.as_str()) { setting.reply_to_address = Some(v.to_string()); }
        if let Some(v) = updates.get("connection_timeout_seconds").and_then(|x| x.as_i64()) { setting.connection_timeout_seconds = v; }
        if let Some(v) = updates.get("enabled").and_then(|x| x.as_bool()) { setting.enabled = v; }
        setting.updated_at = chrono::Utc::now().to_rfc3339();
        smtp_email::SmtpSettings::update(&self.state.db, &r.id, &setting).await.map_err(to_status)?;
        Ok(Response::new(json_response(serde_json::json!({ "updated": true }))))
    }

    async fn delete_smtp_settings(
        &self,
        req: Request<proto::DeleteSmtpSettingsRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        smtp_email::SmtpSettings::delete(&self.state.db, &r.id).await.map_err(to_status)?;
        Ok(Response::new(json_response(serde_json::json!({ "deleted": true }))))
    }

    async fn test_smtp_connection(
        &self,
        req: Request<proto::TestSmtpConnectionRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let v = smtp_email::SmtpSettings::test_connection(&self.state.db, &r.id, &r.to_address).await.map_err(to_status)?;
        Ok(Response::new(json_response(serde_json::to_value(v).map_err(to_status)?)))
    }

    async fn list_email_templates(
        &self,
        req: Request<proto::ListEmailTemplatesRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let cat = if r.category.is_empty() { None } else { Some(r.category) };
        let brand = if r.brand_id.is_empty() { None } else { Some(r.brand_id) };
        let v = smtp_email::EmailTemplate::list(&self.state.db, cat.as_deref(), None, brand.as_deref()).await.map_err(to_status)?;
        Ok(Response::new(json_response(serde_json::to_value(v).map_err(to_status)?)))
    }

    async fn create_email_template(
        &self,
        req: Request<proto::CreateEmailTemplateRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        let brand_id = if r.brand_id.is_empty() { None } else { Some(r.brand_id) };
        let language = if r.language.is_empty() { "en".to_string() } else { r.language };
        let variables = if r.variables_json.is_empty() {
            serde_json::json!({})
        } else {
            parse_json(&r.variables_json)?
        };
        let template = smtp_email::EmailTemplate {
            id,
            brand_id,
            name: r.name,
            subject: r.subject,
            body_html: r.body_html,
            body_text: r.body_text,
            template_type: r.template_type,
            category: r.category,
            language,
            enabled: true,
            is_default: false,
            variables,
            created_at: now.clone(),
            updated_at: now,
        };
        let v = smtp_email::EmailTemplate::create(&self.state.db, &template).await.map_err(to_status)?;
        Ok(Response::new(json_response(serde_json::json!({ "id": v.id }))))
    }

    async fn get_email_template(
        &self,
        req: Request<proto::GetEmailTemplateRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let templates = smtp_email::EmailTemplate::list(&self.state.db, None, None, None).await.map_err(to_status)?;
        let template = templates.into_iter().find(|t| t.id == r.id)
            .ok_or_else(|| Status::not_found("Template not found"))?;
        Ok(Response::new(json_response(serde_json::to_value(template).map_err(to_status)?)))
    }

    async fn update_email_template(
        &self,
        req: Request<proto::UpdateEmailTemplateRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let templates = smtp_email::EmailTemplate::list(&self.state.db, None, None, None).await.map_err(to_status)?;
        let mut template = templates.into_iter().find(|t| t.id == r.id)
            .ok_or_else(|| Status::not_found("Template not found"))?;
        let updates = parse_json(&r.updates_json)?;
        if let Some(v) = updates.get("subject").and_then(|x| x.as_str()) { template.subject = v.to_string(); }
        if let Some(v) = updates.get("body_html").and_then(|x| x.as_str()) { template.body_html = v.to_string(); }
        if let Some(v) = updates.get("body_text").and_then(|x| x.as_str()) { template.body_text = v.to_string(); }
        if let Some(v) = updates.get("template_type").and_then(|x| x.as_str()) { template.template_type = v.to_string(); }
        if let Some(v) = updates.get("category").and_then(|x| x.as_str()) { template.category = v.to_string(); }
        if let Some(v) = updates.get("language").and_then(|x| x.as_str()) { template.language = v.to_string(); }
        if let Some(v) = updates.get("enabled").and_then(|x| x.as_bool()) { template.enabled = v; }
        if let Some(v) = updates.get("variables") { template.variables = v.clone(); }
        template.updated_at = chrono::Utc::now().to_rfc3339();
        smtp_email::EmailTemplate::update(&self.state.db, &r.id, &template).await.map_err(to_status)?;
        Ok(Response::new(json_response(serde_json::json!({ "updated": true }))))
    }

    async fn delete_email_template(
        &self,
        req: Request<proto::DeleteEmailTemplateRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        smtp_email::EmailTemplate::delete(&self.state.db, &r.id).await.map_err(to_status)?;
        Ok(Response::new(json_response(serde_json::json!({ "deleted": true }))))
    }

    async fn send_email(
        &self,
        req: Request<proto::SendEmailRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let brand_id = if r.brand_id.is_empty() { None } else { Some(r.brand_id) };
        let mut variables = std::collections::HashMap::new();
        if !r.variables_json.is_empty() {
            if let Ok(map) = serde_json::from_str::<std::collections::HashMap<String, serde_json::Value>>(&r.variables_json) {
                for (k, v) in map {
                    variables.insert(k, v.to_string());
                }
            }
        }
        let request = smtp_email::SendEmailRequest {
            to_address: r.to_address,
            template_name: r.template_name,
            variables,
            brand_id,
        };
        let v = smtp_email::send_templated_email(&self.state.db, &request).await.map_err(to_status)?;
        Ok(Response::new(json_response(serde_json::to_value(v).map_err(to_status)?)))
    }

    async fn get_send_logs(
        &self,
        req: Request<proto::GetSendLogsRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let limit = if r.limit == 0 { 50 } else { r.limit };
        let v = smtp_email::EmailSendLog::recent(&self.state.db, limit).await.map_err(to_status)?;
        Ok(Response::new(json_response(serde_json::to_value(v).map_err(to_status)?)))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://postgres:postgres@localhost/communicator".to_string());

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .map_err(|e| format!("Cannot connect to database: {e}"))?;

    let state = smtp_email::SmtpEmailState::new(pool);

    let addr = "[::1]:50092".parse().unwrap();
    info!("smtp-email listening on {}", addr);

    Server::builder()
        .add_service(SmtpEmailServiceServer::new(Svc { state }))
        .serve(addr)
        .await?;

    Ok(())
}
