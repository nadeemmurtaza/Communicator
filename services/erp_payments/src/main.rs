use newax_erp_payments as erp_payments;
use sqlx::postgres::PgPoolOptions;
use tonic::{transport::Server, Request, Response, Status};
use tracing::info;

pub mod proto {
    tonic::include_proto!("newax.erp_payments.v1");
}

use proto::erp_payments_service_server::{ErpPaymentsService, ErpPaymentsServiceServer};
use proto::{
    Empty, IdRequest, JsonResponse, ListPaymentsRequest, CreatePaymentRequest,
    UpdatePaymentStatusRequest, MarkAsPaidRequest, MarkAsFailedRequest,
    RefundPaymentRequest, UploadReceiptRequest, ApprovePaymentRequest,
    RejectPaymentRequest, PendingApprovalsRequest, CreatePaymentMethodRequest,
    UpdatePaymentMethodRequest,
};

struct Svc {
    pool: sqlx::PgPool,
}

fn to_status(e: impl std::fmt::Display) -> Status {
    Status::internal(e.to_string())
}

fn serialize_response<T: serde::Serialize>(val: T) -> Result<JsonResponse, Status> {
    let v = serde_json::to_value(val).map_err(|e| Status::internal(e.to_string()))?;
    Ok(JsonResponse { json: v.to_string() })
}

#[tonic::async_trait]
impl ErpPaymentsService for Svc {
    async fn list_payments(&self, req: Request<ListPaymentsRequest>) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let payments = erp_payments::ErpPaymentsService::list_payments(
            &self.pool,
            r.order_id.as_deref(),
            r.status.as_deref(),
            r.provider.as_deref(),
            r.limit,
            r.offset,
        ).await.map_err(to_status)?;
        Ok(Response::new(serialize_response(payments)?))
    }

    async fn get_payment(&self, req: Request<IdRequest>) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let payment = erp_payments::ErpPaymentsService::get_payment(&self.pool, &r.id).await.map_err(to_status)?;
        Ok(Response::new(serialize_response(payment)?))
    }

    async fn get_payments_by_order(&self, req: Request<IdRequest>) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let payments = erp_payments::ErpPaymentsService::get_payments_by_order(&self.pool, &r.id).await.map_err(to_status)?;
        Ok(Response::new(serialize_response(payments)?))
    }

    async fn get_payment_by_reference(&self, req: Request<IdRequest>) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let payment = erp_payments::ErpPaymentsService::get_payment_by_reference(&self.pool, &r.id).await.map_err(to_status)?;
        Ok(Response::new(serialize_response(payment)?))
    }

    async fn create_payment(&self, req: Request<CreatePaymentRequest>) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let meta = match r.meta_json {
            Some(ref s) if !s.is_empty() => Some(serde_json::from_str(s).map_err(|e| Status::invalid_argument(e.to_string()))?),
            _ => None,
        };
        let req_body = erp_payments::CreatePaymentRequest {
            order_id: r.order_id,
            provider: r.provider,
            amount: r.amount,
            currency: r.currency,
            reference: r.reference,
            meta,
        };
        let payment = erp_payments::ErpPaymentsService::create_payment(&self.pool, &req_body).await.map_err(to_status)?;
        Ok(Response::new(serialize_response(payment)?))
    }

    async fn update_payment_status(&self, req: Request<UpdatePaymentStatusRequest>) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let meta = match r.meta_json {
            Some(ref s) if !s.is_empty() => Some(serde_json::from_str(s).map_err(|e| Status::invalid_argument(e.to_string()))?),
            _ => None,
        };
        let req_body = erp_payments::UpdatePaymentStatusRequest {
            status: r.status,
            provider_ref: r.provider_ref,
            receipt_url: r.receipt_url,
            failure_reason: r.failure_reason,
            approved_by_id: r.approved_by_id,
            meta,
        };
        let payment = erp_payments::ErpPaymentsService::update_payment_status(&self.pool, &r.id, &req_body).await.map_err(to_status)?;
        Ok(Response::new(serialize_response(payment)?))
    }

    async fn mark_as_paid(&self, req: Request<MarkAsPaidRequest>) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let payment = erp_payments::ErpPaymentsService::mark_as_paid(&self.pool, &r.id, r.provider_ref.as_deref(), r.approved_by_id.as_deref()).await.map_err(to_status)?;
        Ok(Response::new(serialize_response(payment)?))
    }

    async fn mark_as_failed(&self, req: Request<MarkAsFailedRequest>) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let payment = erp_payments::ErpPaymentsService::mark_as_failed(&self.pool, &r.id, &r.reason).await.map_err(to_status)?;
        Ok(Response::new(serialize_response(payment)?))
    }

    async fn refund_payment(&self, req: Request<RefundPaymentRequest>) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let req_body = erp_payments::RefundPaymentRequest {
            refund_amount: r.refund_amount,
            reason: r.reason,
        };
        let payment = erp_payments::ErpPaymentsService::refund_payment(&self.pool, &r.id, &req_body).await.map_err(to_status)?;
        Ok(Response::new(serialize_response(payment)?))
    }

    async fn upload_receipt(&self, req: Request<UploadReceiptRequest>) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let meta = match r.meta_json {
            Some(ref s) if !s.is_empty() => Some(serde_json::from_str(s).map_err(|e| Status::invalid_argument(e.to_string()))?),
            _ => None,
        };
        let req_body = erp_payments::UploadReceiptRequest {
            receipt_url: r.receipt_url,
            meta,
        };
        let payment = erp_payments::ErpPaymentsService::upload_receipt(&self.pool, &r.id, &req_body).await.map_err(to_status)?;
        Ok(Response::new(serialize_response(payment)?))
    }

    async fn approve_payment(&self, req: Request<ApprovePaymentRequest>) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let payment = erp_payments::ErpPaymentsService::approve_payment(&self.pool, &r.id, &r.approved_by_id).await.map_err(to_status)?;
        Ok(Response::new(serialize_response(payment)?))
    }

    async fn reject_payment(&self, req: Request<RejectPaymentRequest>) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let payment = erp_payments::ErpPaymentsService::reject_payment(&self.pool, &r.id, &r.reason).await.map_err(to_status)?;
        Ok(Response::new(serialize_response(payment)?))
    }

    async fn get_pending_approvals(&self, req: Request<PendingApprovalsRequest>) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let payments = erp_payments::ErpPaymentsService::get_pending_approvals(&self.pool, r.limit, r.offset).await.map_err(to_status)?;
        Ok(Response::new(serialize_response(payments)?))
    }

    async fn get_payment_stats(&self, _req: Request<Empty>) -> Result<Response<JsonResponse>, Status> {
        let stats = erp_payments::ErpPaymentsService::get_payment_stats(&self.pool).await.map_err(to_status)?;
        Ok(Response::new(JsonResponse { json: stats.to_string() }))
    }

    async fn list_payment_methods(&self, _req: Request<Empty>) -> Result<Response<JsonResponse>, Status> {
        let methods = erp_payments::ErpPaymentsService::list_payment_methods(&self.pool).await.map_err(to_status)?;
        Ok(Response::new(serialize_response(methods)?))
    }

    async fn get_payment_method(&self, req: Request<IdRequest>) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let method = erp_payments::ErpPaymentsService::get_payment_method(&self.pool, &r.id).await.map_err(to_status)?;
        Ok(Response::new(serialize_response(method)?))
    }

    async fn get_payment_method_by_code(&self, req: Request<IdRequest>) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let method = erp_payments::ErpPaymentsService::get_payment_method_by_code(&self.pool, &r.id).await.map_err(to_status)?;
        Ok(Response::new(serialize_response(method)?))
    }

    async fn create_payment_method(&self, req: Request<CreatePaymentMethodRequest>) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let config = match r.config_json {
            Some(ref s) if !s.is_empty() => Some(serde_json::from_str(s).map_err(|e| Status::invalid_argument(e.to_string()))?),
            _ => None,
        };
        let req_body = erp_payments::CreatePaymentMethodRequest {
            name: r.name,
            code: r.code,
            config,
        };
        let method = erp_payments::ErpPaymentsService::create_payment_method(&self.pool, &req_body).await.map_err(to_status)?;
        Ok(Response::new(serialize_response(method)?))
    }

    async fn update_payment_method(&self, req: Request<UpdatePaymentMethodRequest>) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let config = match r.config_json {
            Some(ref s) if !s.is_empty() => Some(serde_json::from_str(s).map_err(|e| Status::invalid_argument(e.to_string()))?),
            _ => None,
        };
        let req_body = erp_payments::UpdatePaymentMethodRequest {
            name: r.name,
            config,
            is_active: r.is_active,
        };
        let method = erp_payments::ErpPaymentsService::update_payment_method(&self.pool, &r.id, &req_body).await.map_err(to_status)?;
        Ok(Response::new(serialize_response(method)?))
    }

    async fn delete_payment_method(&self, req: Request<IdRequest>) -> Result<Response<Empty>, Status> {
        let r = req.into_inner();
        erp_payments::ErpPaymentsService::delete_payment_method(&self.pool, &r.id).await.map_err(to_status)?;
        Ok(Response::new(Empty {}))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "svc-erp-payments=debug,tonic=info".into()),
        )
        .init();

    dotenvy::dotenv().ok();

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://postgres:postgres@localhost:5432/communicator".to_string());

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to create database pool");

    let port: u16 = std::env::var("ERP_PAYMENTS_PORT").ok().and_then(|p| p.parse().ok()).unwrap_or(50096);
    let addr = format!("0.0.0.0:{port}").parse()?;

    info!("ERP Payments service listening on {}", addr);

    Server::builder()
        .add_service(ErpPaymentsServiceServer::new(Svc { pool }))
        .serve(addr)
        .await?;

    Ok(())
}
