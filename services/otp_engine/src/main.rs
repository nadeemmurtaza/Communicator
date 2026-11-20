use newax_otp_engine as otp_engine;
use sqlx::postgres::PgPoolOptions;
use tonic::{transport::Server, Request, Response, Status};
use tracing::info;

pub mod proto {
    tonic::include_proto!("newax.otp_engine.v1");
}

use proto::{
    otp_engine_service_server::{OtpEngineService as OtpEngineServiceServerTrait, OtpEngineServiceServer},
    JsonResponse,
};

struct Svc {
    db: sqlx::PgPool,
}

fn to_status(e: impl std::fmt::Display) -> Status {
    Status::internal(e.to_string())
}

fn json_response(v: serde_json::Value) -> JsonResponse {
    JsonResponse { json: v.to_string() }
}

#[tonic::async_trait]
impl OtpEngineServiceServerTrait for Svc {
    async fn generate_and_send(
        &self,
        req: Request<proto::GenerateAndSendRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let metadata: Option<std::collections::HashMap<String, String>> = if r.metadata.is_empty() {
            None
        } else {
            serde_json::from_str(&r.metadata).unwrap_or(None)
        };
        let req_inner = otp_engine::GenerateOtpRequest {
            recipient: r.recipient,
            recipient_type: if r.recipient_type.is_empty() { None } else { Some(r.recipient_type) },
            purpose: if r.purpose.is_empty() { None } else { Some(r.purpose) },
            channel: if r.channel.is_empty() { None } else { Some(r.channel) },
            metadata,
            ip_address: if r.ip_address.is_empty() { None } else { Some(r.ip_address) },
            user_agent: if r.user_agent.is_empty() { None } else { Some(r.user_agent) },
        };
        let result = otp_engine::OtpEngine::generate_and_send(&self.db, &r.brand_id, &req_inner)
            .await
            .map_err(to_status)?;
        Ok(Response::new(json_response(serde_json::to_value(result).map_err(to_status)?)))
    }

    async fn verify(
        &self,
        req: Request<proto::VerifyRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let req_inner = otp_engine::VerifyOtpRequest {
            otp_id: r.otp_id,
            code: r.code,
            ip_address: if r.ip_address.is_empty() { None } else { Some(r.ip_address) },
            user_agent: if r.user_agent.is_empty() { None } else { Some(r.user_agent) },
        };
        let result = otp_engine::OtpEngine::verify(&self.db, &req_inner)
            .await
            .map_err(to_status)?;
        Ok(Response::new(json_response(serde_json::to_value(result).map_err(to_status)?)))
    }

    async fn get_channel_config(
        &self,
        req: Request<proto::GetChannelConfigRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let result = otp_engine::OtpEngine::get_channel_config(&self.db, &r.brand_id)
            .await
            .map_err(to_status)?;
        Ok(Response::new(json_response(serde_json::to_value(result).map_err(to_status)?)))
    }

    async fn update_channel_config(
        &self,
        req: Request<proto::UpdateChannelConfigRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let mut body = otp_engine::UpdateChannelConfigRequest::default();
        if r.has_whatsapp_enabled { body.whatsapp_enabled = Some(r.whatsapp_enabled.parse().unwrap_or(false)); }
        if r.has_email_enabled { body.email_enabled = Some(r.email_enabled.parse().unwrap_or(false)); }
        if r.has_sms_enabled { body.sms_enabled = Some(r.sms_enabled.parse().unwrap_or(false)); }
        if r.has_in_app_enabled { body.in_app_enabled = Some(r.in_app_enabled.parse().unwrap_or(false)); }
        if r.has_default_channel { body.default_channel = Some(r.default_channel); }
        if r.has_code_length { body.code_length = Some(r.code_length.parse().unwrap_or(6)); }
        if r.has_code_charset { body.code_charset = Some(r.code_charset); }
        if r.has_ttl_seconds { body.ttl_seconds = Some(r.ttl_seconds.parse().unwrap_or(300)); }
        if r.has_max_attempts { body.max_attempts = Some(r.max_attempts.parse().unwrap_or(3)); }
        if r.has_rate_limit_per_minute { body.rate_limit_per_minute = Some(r.rate_limit_per_minute.parse().unwrap_or(5)); }
        if r.has_rate_limit_per_hour { body.rate_limit_per_hour = Some(r.rate_limit_per_hour.parse().unwrap_or(20)); }
        if r.has_cooldown_seconds { body.cooldown_seconds = Some(r.cooldown_seconds.parse().unwrap_or(30)); }
        if r.has_lockout_seconds { body.lockout_seconds = Some(r.lockout_seconds.parse().unwrap_or(900)); }
        if r.has_template_name { body.template_name = Some(r.template_name); }
        if r.has_whatsapp_message_template { body.whatsapp_message_template = Some(r.whatsapp_message_template); }
        if r.has_sms_message_template { body.sms_message_template = Some(r.sms_message_template); }

        let result = otp_engine::OtpEngine::update_channel_config(&self.db, &r.brand_id, &body)
            .await
            .map_err(to_status)?;
        Ok(Response::new(json_response(serde_json::to_value(result).map_err(to_status)?)))
    }

    async fn list_records(
        &self,
        req: Request<proto::ListRecordsRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let result = otp_engine::OtpEngine::list_records(&self.db, &r.brand_id, r.limit as i64)
            .await
            .map_err(to_status)?;
        Ok(Response::new(json_response(serde_json::to_value(result).map_err(to_status)?)))
    }

    async fn get_audit_log(
        &self,
        req: Request<proto::GetAuditLogRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let result = otp_engine::OtpEngine::get_audit_log(&self.db, &r.brand_id, r.limit as i64)
            .await
            .map_err(to_status)?;
        Ok(Response::new(json_response(serde_json::to_value(result).map_err(to_status)?)))
    }

    async fn get_stats(
        &self,
        req: Request<proto::GetStatsRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let result = otp_engine::OtpEngine::get_stats(&self.db, &r.brand_id)
            .await
            .map_err(to_status)?;
        Ok(Response::new(json_response(result)))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let db_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&db_url)
        .await
        .expect("failed to connect to database");

    let addr = "0.0.0.0:50099".parse()?;
    let svc = Svc { db: pool };

    info!("OTP Engine gRPC server listening on {}", addr);
    Server::builder()
        .add_service(OtpEngineServiceServer::new(svc))
        .serve(addr)
        .await?;

    Ok(())
}
