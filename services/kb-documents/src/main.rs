use newax_kb_documents::{self as kb, KbDocumentsState};
use sqlx::postgres::PgPoolOptions;
use tonic::{transport::Server, Request, Response, Status};
use tracing::info;

pub mod proto {
    tonic::include_proto!("newax.kb_documents.v1");
}

use proto::{
    kb_documents_service_server::{KbDocumentsService, KbDocumentsServiceServer},
    AddUrlSourceRequest, DeleteResponse, DocumentIdRequest, GetSummaryRequest, JsonResponse,
    ListDocumentsRequest, ListUrlSourcesRequest, OrgRequest, SourceIdRequest,
    UpdateSettingsRequest, UploadDocumentRequest,
};

struct KbDocumentsGrpc {
    state: KbDocumentsState,
}

fn to_status(e: String) -> Status {
    Status::internal(e)
}

fn json_response(v: serde_json::Value) -> JsonResponse {
    JsonResponse { json: v.to_string() }
}

#[tonic::async_trait]
impl KbDocumentsService for KbDocumentsGrpc {
    async fn get_summary(
        &self,
        _req: Request<GetSummaryRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let v = kb::get_kb_import_summary(&self.state).await.map_err(to_status)?;
        Ok(Response::new(json_response(v)))
    }

    async fn list_documents(
        &self,
        req: Request<ListDocumentsRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let v = kb::list_documents(&self.state, r.organization_id.as_deref(), r.status.as_deref())
            .await
            .map_err(to_status)?;
        Ok(Response::new(json_response(v)))
    }

    async fn upload_document(
        &self,
        req: Request<UploadDocumentRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let v = kb::upload_document(&self.state, &r.organization_id, &r.file_name, &r.file_type, &r.file_data)
            .await
            .map_err(Status::invalid_argument)?;
        Ok(Response::new(json_response(v)))
    }

    async fn get_document_chunks(
        &self,
        req: Request<DocumentIdRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let v = kb::get_document_chunks(&self.state, &r.doc_id).await.map_err(to_status)?;
        Ok(Response::new(json_response(v)))
    }

    async fn delete_document(
        &self,
        req: Request<DocumentIdRequest>,
    ) -> Result<Response<DeleteResponse>, Status> {
        let r = req.into_inner();
        kb::delete_document(&self.state, &r.doc_id).await.map_err(to_status)?;
        Ok(Response::new(DeleteResponse { deleted: true }))
    }

    async fn add_url_source(
        &self,
        req: Request<AddUrlSourceRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let v = kb::add_url_source(&self.state, &r.organization_id, &r.url, r.crawl_interval_hours)
            .await
            .map_err(Status::invalid_argument)?;
        Ok(Response::new(json_response(v)))
    }

    async fn crawl_url(
        &self,
        req: Request<SourceIdRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let v = kb::crawl_url(&self.state, &r.source_id).await.map_err(Status::invalid_argument)?;
        Ok(Response::new(json_response(v)))
    }

    async fn list_url_sources(
        &self,
        req: Request<ListUrlSourcesRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let v = kb::list_url_sources(&self.state, r.organization_id.as_deref(), r.status.as_deref())
            .await
            .map_err(to_status)?;
        Ok(Response::new(json_response(v)))
    }

    async fn delete_url_source(
        &self,
        req: Request<SourceIdRequest>,
    ) -> Result<Response<DeleteResponse>, Status> {
        let r = req.into_inner();
        kb::delete_url_source(&self.state, &r.source_id).await.map_err(to_status)?;
        Ok(Response::new(DeleteResponse { deleted: true }))
    }

    async fn get_settings(
        &self,
        req: Request<OrgRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let v = kb::get_settings(&self.state, &r.organization_id).await.map_err(to_status)?;
        Ok(Response::new(json_response(v)))
    }

    async fn update_settings(
        &self,
        req: Request<UpdateSettingsRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let payload: serde_json::Value =
            serde_json::from_str(&r.settings_json).map_err(|e| Status::invalid_argument(e.to_string()))?;
        let v = kb::update_settings(&self.state, &r.organization_id, &payload)
            .await
            .map_err(Status::invalid_argument)?;
        Ok(Response::new(json_response(v)))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::try_new("info,svc_kb_documents=debug").unwrap())
        .init();

    let database_url = std::env::var("DATABASE_URL")
        .map_err(|_| "DATABASE_URL must be set (shared Communicator Postgres instance)")?;
    let db = PgPoolOptions::new().max_connections(5).connect(&database_url).await?;
    let http_client = reqwest::Client::new();

    let port: u16 = std::env::var("KB_DOCUMENTS_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(50061);
    let addr = format!("0.0.0.0:{port}").parse()?;

    let grpc = KbDocumentsGrpc {
        state: KbDocumentsState { db, http_client },
    };

    info!(port, "svc-kb-documents starting (gRPC)");
    Server::builder()
        .add_service(KbDocumentsServiceServer::new(grpc))
        .serve(addr)
        .await?;
    Ok(())
}
