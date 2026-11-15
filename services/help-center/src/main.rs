use newax_help_center::{self as hc, HelpCenterState};
use sqlx::postgres::PgPoolOptions;
use tonic::{transport::Server, Request, Response, Status};
use tracing::info;

pub mod proto {
    tonic::include_proto!("newax.help_center.v1");
}

use proto::{
    help_center_service_server::{HelpCenterService, HelpCenterServiceServer},
    ArticleIdRequest, CreateArticleRequest, CreateCategoryRequest, DeleteResponse, Empty, FeedbackRequest,
    JsonResponse, ListArticlesRequest, ListCategoriesRequest, SearchRequest, SlugRequest, UpdateArticleRequest,
    UpdateSettingsRequest,
};

struct HelpCenterGrpc {
    state: HelpCenterState,
}

fn to_status(e: String) -> Status {
    Status::internal(e)
}

fn json_response<T: serde::Serialize>(v: &T) -> Result<JsonResponse, Status> {
    Ok(JsonResponse { json: serde_json::to_string(v).map_err(|e| Status::internal(e.to_string()))? })
}

#[tonic::async_trait]
impl HelpCenterService for HelpCenterGrpc {
    async fn get_summary(&self, _req: Request<Empty>) -> Result<Response<JsonResponse>, Status> {
        let v = hc::get_help_center_summary(&self.state).await.map_err(to_status)?;
        Ok(Response::new(json_response(&v)?))
    }

    async fn get_public(&self, _req: Request<Empty>) -> Result<Response<JsonResponse>, Status> {
        let v = hc::get_public_help_center(&self.state).await.map_err(to_status)?;
        Ok(Response::new(json_response(&v)?))
    }

    async fn list_categories(&self, req: Request<ListCategoriesRequest>) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let v = hc::list_categories(&self.state, r.organization_id.as_deref(), r.visible_only.unwrap_or(true))
            .await
            .map_err(to_status)?;
        Ok(Response::new(json_response(&v)?))
    }

    async fn create_category(&self, req: Request<CreateCategoryRequest>) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let v = hc::create_category(
            &self.state,
            &r.organization_id,
            &r.name,
            &r.slug,
            r.description.as_deref(),
            r.icon.as_deref(),
            r.display_order.unwrap_or(0),
        )
        .await
        .map_err(to_status)?;
        Ok(Response::new(json_response(&v)?))
    }

    async fn list_articles(&self, req: Request<ListArticlesRequest>) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let v = hc::list_articles(&self.state, r.category_id.as_deref(), r.status.as_deref(), r.limit, r.offset)
            .await
            .map_err(to_status)?;
        Ok(Response::new(json_response(&v)?))
    }

    async fn create_article(&self, req: Request<CreateArticleRequest>) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let v = hc::create_article(
            &self.state,
            &r.organization_id,
            &r.category_id,
            &r.title,
            &r.slug,
            &r.content,
            r.author_id.as_deref(),
        )
        .await
        .map_err(to_status)?;
        Ok(Response::new(json_response(&v)?))
    }

    async fn search_articles(&self, req: Request<SearchRequest>) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let v = hc::search_articles(&self.state, &r.q, r.limit).await.map_err(to_status)?;
        Ok(Response::new(json_response(&v)?))
    }

    async fn get_article_by_slug(&self, req: Request<SlugRequest>) -> Result<Response<JsonResponse>, Status> {
        let v = hc::get_article_by_slug(&self.state, &req.into_inner().slug).await.map_err(to_status)?;
        match v {
            Some(a) => Ok(Response::new(json_response(&a)?)),
            None => Err(Status::not_found("Article not found")),
        }
    }

    async fn update_article(&self, req: Request<UpdateArticleRequest>) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let v = hc::update_article(
            &self.state,
            &r.article_id,
            r.title.as_deref(),
            r.content.as_deref(),
            r.status.as_deref(),
            r.is_pinned,
            r.seo_title.as_deref(),
            r.seo_description.as_deref(),
        )
        .await
        .map_err(to_status)?;
        match v {
            Some(a) => Ok(Response::new(json_response(&a)?)),
            None => Err(Status::not_found("Article not found")),
        }
    }

    async fn delete_article(&self, req: Request<ArticleIdRequest>) -> Result<Response<DeleteResponse>, Status> {
        let deleted = hc::delete_article(&self.state, &req.into_inner().article_id).await.map_err(to_status)?;
        if deleted {
            Ok(Response::new(DeleteResponse { deleted: true }))
        } else {
            Err(Status::not_found("Article not found"))
        }
    }

    async fn record_feedback(&self, req: Request<FeedbackRequest>) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let v = hc::record_article_feedback(&self.state, &r.article_id, r.helpful).await.map_err(to_status)?;
        Ok(Response::new(json_response(&v)?))
    }

    async fn get_settings(&self, _req: Request<Empty>) -> Result<Response<JsonResponse>, Status> {
        let v = hc::get_settings(&self.state).await.map_err(to_status)?;
        Ok(Response::new(json_response(&v)?))
    }

    async fn update_settings(&self, req: Request<UpdateSettingsRequest>) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let v = hc::update_settings(
            &self.state,
            r.name.as_deref(),
            r.tagline.as_deref(),
            r.primary_color.as_deref(),
            r.enable_search,
            r.enable_article_feedback,
            r.footer_text.as_deref(),
        )
        .await
        .map_err(to_status)?;
        Ok(Response::new(json_response(&v)?))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::try_new("info,svc_help_center=debug").unwrap())
        .init();

    let database_url = std::env::var("DATABASE_URL")
        .map_err(|_| "DATABASE_URL must be set (shared Communicator Postgres instance)")?;
    let db = PgPoolOptions::new().max_connections(5).connect(&database_url).await?;

    let port: u16 = std::env::var("HELP_CENTER_PORT").ok().and_then(|p| p.parse().ok()).unwrap_or(50064);
    let addr = format!("0.0.0.0:{port}").parse()?;

    let grpc = HelpCenterGrpc { state: HelpCenterState { db } };

    info!(port, "svc-help-center starting (gRPC)");
    Server::builder()
        .add_service(HelpCenterServiceServer::new(grpc))
        .serve(addr)
        .await?;
    Ok(())
}
