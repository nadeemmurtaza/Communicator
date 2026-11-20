use newax_commerce_catalog as catalog;
use sqlx::postgres::PgPoolOptions;
use tonic::{transport::Server, Request, Response, Status};
use tracing::info;

pub mod proto {
    tonic::include_proto!("newax.commerce_catalog.v1");
}

use proto::{
    commerce_catalog_service_server::{CommerceCatalogService, CommerceCatalogServiceServer},
    JsonResponse,
};

const PORT: u16 = 50110;

struct Svc {
    pool: sqlx::PgPool,
}

fn to_status(e: impl std::fmt::Display) -> Status {
    Status::internal(e.to_string())
}

fn json_response(v: serde_json::Value) -> JsonResponse {
    JsonResponse { json: v.to_string() }
}

#[tonic::async_trait]
impl CommerceCatalogService for Svc {
    async fn list_categories(
        &self,
        req: Request<proto::ListCategoriesRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let service = catalog::CommerceCatalogService::new(self.pool.clone());
        let result = service
            .list_categories(&r.brand_id, r.active_only)
            .await
            .map_err(to_status)?;
        Ok(Response::new(json_response(serde_json::to_value(result).map_err(to_status)?)))
    }

    async fn get_category(
        &self,
        req: Request<proto::GetCategoryRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let service = catalog::CommerceCatalogService::new(self.pool.clone());
        let result = service.get_category(&r.id).await.map_err(to_status)?;
        Ok(Response::new(json_response(serde_json::to_value(result).map_err(to_status)?)))
    }

    async fn create_category(
        &self,
        req: Request<proto::CreateCategoryRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let parent_id = if r.parent_id.is_empty() {
            None
        } else {
            Some(r.parent_id)
        };
        let sort_order = if r.sort_order.is_empty() {
            None
        } else {
            r.sort_order.parse().ok()
        };
        let create_req = catalog::CreateCategoryRequest {
            brand_id: r.brand_id,
            name: r.name,
            parent_id,
            sort_order,
        };
        let service = catalog::CommerceCatalogService::new(self.pool.clone());
        let result = service.create_category(&create_req).await.map_err(to_status)?;
        Ok(Response::new(json_response(serde_json::to_value(result).map_err(to_status)?)))
    }

    async fn update_category(
        &self,
        req: Request<proto::UpdateCategoryRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let name = if r.name.is_empty() { None } else { Some(r.name) };
        let parent_id = if r.parent_id.is_empty() { None } else { Some(r.parent_id) };
        let sort_order = if r.sort_order.is_empty() {
            None
        } else {
            r.sort_order.parse().ok()
        };
        let is_active = if r.is_active.is_empty() {
            None
        } else {
            Some(r.is_active == "true")
        };
        let update_req = catalog::UpdateCategoryRequest {
            name,
            parent_id,
            sort_order,
            is_active,
        };
        let service = catalog::CommerceCatalogService::new(self.pool.clone());
        let result = service.update_category(&r.id, &update_req).await.map_err(to_status)?;
        Ok(Response::new(json_response(serde_json::to_value(result).map_err(to_status)?)))
    }

    async fn delete_category(
        &self,
        req: Request<proto::DeleteCategoryRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let service = catalog::CommerceCatalogService::new(self.pool.clone());
        service.delete_category(&r.id).await.map_err(to_status)?;
        Ok(Response::new(json_response(serde_json::Value::Bool(true))))
    }

    async fn list_products(
        &self,
        req: Request<proto::ListProductsRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let category_id = if r.category_id.is_empty() {
            None
        } else {
            Some(r.category_id.as_str())
        };
        let channel_type = if r.channel_type.is_empty() {
            None
        } else {
            Some(r.channel_type.as_str())
        };
        let service = catalog::CommerceCatalogService::new(self.pool.clone());
        let result = service
            .list_products(&r.brand_id, category_id, r.active_only, channel_type)
            .await
            .map_err(to_status)?;
        Ok(Response::new(json_response(serde_json::to_value(result).map_err(to_status)?)))
    }

    async fn get_product(
        &self,
        req: Request<proto::GetProductRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let service = catalog::CommerceCatalogService::new(self.pool.clone());
        let result = service.get_product(&r.id).await.map_err(to_status)?;
        Ok(Response::new(json_response(serde_json::to_value(result).map_err(to_status)?)))
    }

    async fn create_product(
        &self,
        req: Request<proto::CreateProductRequestBody>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let create_req: catalog::CreateProductRequest =
            serde_json::from_str(&r.json).map_err(to_status)?;
        let service = catalog::CommerceCatalogService::new(self.pool.clone());
        let result = service.create_product(&create_req).await.map_err(to_status)?;
        Ok(Response::new(json_response(serde_json::to_value(result).map_err(to_status)?)))
    }

    async fn update_product(
        &self,
        req: Request<proto::UpdateProductRequestBody>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let update_req: catalog::UpdateProductRequest =
            serde_json::from_str(&r.json).map_err(to_status)?;
        let service = catalog::CommerceCatalogService::new(self.pool.clone());
        let result = service.update_product(&r.id, &update_req).await.map_err(to_status)?;
        Ok(Response::new(json_response(serde_json::to_value(result).map_err(to_status)?)))
    }

    async fn delete_product(
        &self,
        req: Request<proto::DeleteProductRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let service = catalog::CommerceCatalogService::new(self.pool.clone());
        service.delete_product(&r.id).await.map_err(to_status)?;
        Ok(Response::new(json_response(serde_json::Value::Bool(true))))
    }

    async fn update_inventory(
        &self,
        req: Request<proto::UpdateInventoryRequestBody>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let update_req: catalog::UpdateInventoryRequest =
            serde_json::from_str(&r.json).map_err(to_status)?;
        let service = catalog::CommerceCatalogService::new(self.pool.clone());
        let result = service
            .update_inventory(&r.product_id, &update_req)
            .await
            .map_err(to_status)?;
        Ok(Response::new(json_response(serde_json::to_value(result).map_err(to_status)?)))
    }

    async fn get_channel_visibility(
        &self,
        req: Request<proto::GetChannelVisibilityRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let service = catalog::CommerceCatalogService::new(self.pool.clone());
        let result = service
            .get_channel_visibility(&r.product_id)
            .await
            .map_err(to_status)?;
        Ok(Response::new(json_response(serde_json::to_value(result).map_err(to_status)?)))
    }

    async fn set_channel_visibility(
        &self,
        req: Request<proto::SetChannelVisibilityRequestBody>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let set_req: catalog::SetChannelVisibilityRequest =
            serde_json::from_str(&r.json).map_err(to_status)?;
        let service = catalog::CommerceCatalogService::new(self.pool.clone());
        let result = service
            .set_channel_visibility(&r.product_id, &set_req)
            .await
            .map_err(to_status)?;
        Ok(Response::new(json_response(serde_json::to_value(result).map_err(to_status)?)))
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

    let port: u16 = std::env::var("COMMERCE_CATALOG_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(PORT);
    let addr = format!("0.0.0.0:{port}").parse()?;

    info!(port, "svc-commerce-catalog starting (gRPC)");

    Server::builder()
        .add_service(CommerceCatalogServiceServer::new(Svc { pool }))
        .serve(addr)
        .await?;

    Ok(())
}
