mod erp_catalog {
    include!(concat!(env!("OUT_DIR"), "/newax.erp_catalog.v1.rs"));
}

use erp_catalog::erp_catalog_service_server::{ErpCatalogService, ErpCatalogServiceServer};
use erp_catalog::{Empty, IdRequest, JsonResponse, ListCategoriesRequest, ListProductsRequest, SlugRequest,
    CreateCategoryRequest, UpdateCategoryRequest, UpsertCategoryTranslationRequest,
    CreateProductRequest, UpdateProductRequest, UpsertProductTranslationRequest,
    CreateVariantRequest, UpdateVariantRequest, CreateImageRequest,
    CreateProductOptionRequest, CreateOptionValueRequest, CreateCollectionRequest,
    AddProductToCollectionRequest, RemoveProductFromCollectionRequest,
    AdjustStockRequest, ListStockAdjustmentsRequest, AddCrossSellRequest,
    CreateRedirectRequest, ListProductsRequest as ListProductsReq};
use newax_erp_catalog::ErpCatalogService as BusinessService;
use sqlx::PgPool;
use tonic::{Request, Response, Status};

#[derive(Clone)]
struct ErpCatalogServer {
    service: BusinessService,
}

impl ErpCatalogServer {
    fn new(pool: PgPool) -> Self {
        Self {
            service: BusinessService::new(pool),
        }
    }
}

#[tonic::async_trait]
impl ErpCatalogService for ErpCatalogServer {
    async fn list_categories(&self, request: Request<ListCategoriesRequest>) -> Result<Response<JsonResponse>, Status> {
        let req = request.into_inner();
        let json = self.service.list_categories(req.active_only).await.map_err(|e| Status::internal(e))?;
        Ok(Response::new(JsonResponse { json }))
    }

    async fn get_category(&self, request: Request<IdRequest>) -> Result<Response<JsonResponse>, Status> {
        let req = request.into_inner();
        let json = self.service.get_category(&req.id).await.map_err(|e| Status::not_found(e))?;
        Ok(Response::new(JsonResponse { json }))
    }

    async fn create_category(&self, request: Request<CreateCategoryRequest>) -> Result<Response<JsonResponse>, Status> {
        let req = request.into_inner();
        let json = self.service.create_category(&req.json).await.map_err(|e| Status::invalid_argument(e))?;
        Ok(Response::new(JsonResponse { json }))
    }

    async fn update_category(&self, request: Request<UpdateCategoryRequest>) -> Result<Response<JsonResponse>, Status> {
        let req = request.into_inner();
        let json = self.service.update_category(&req.id, &req.json).await.map_err(|e| Status::invalid_argument(e))?;
        Ok(Response::new(JsonResponse { json }))
    }

    async fn delete_category(&self, request: Request<IdRequest>) -> Result<Response<Empty>, Status> {
        let req = request.into_inner();
        self.service.delete_category(&req.id).await.map_err(|e| Status::internal(e))?;
        Ok(Response::new(Empty {}))
    }

    async fn list_category_translations(&self, request: Request<IdRequest>) -> Result<Response<JsonResponse>, Status> {
        let req = request.into_inner();
        let json = self.service.list_category_translations(&req.id).await.map_err(|e| Status::internal(e))?;
        Ok(Response::new(JsonResponse { json }))
    }

    async fn upsert_category_translation(&self, request: Request<UpsertCategoryTranslationRequest>) -> Result<Response<JsonResponse>, Status> {
        let req = request.into_inner();
        let json = self.service.upsert_category_translation(&req.category_id, &req.locale, &req.json).await.map_err(|e| Status::internal(e))?;
        Ok(Response::new(JsonResponse { json }))
    }

    async fn list_products(&self, request: Request<ListProductsRequest>) -> Result<Response<JsonResponse>, Status> {
        let req = request.into_inner();
        let status = if req.status.is_empty() { None } else { Some(req.status.as_str()) };
        let kind = if req.kind.is_empty() { None } else { Some(req.kind.as_str()) };
        let json = self.service.list_products(status, kind, req.limit, req.offset).await.map_err(|e| Status::internal(e))?;
        Ok(Response::new(JsonResponse { json }))
    }

    async fn get_product(&self, request: Request<IdRequest>) -> Result<Response<JsonResponse>, Status> {
        let req = request.into_inner();
        let json = self.service.get_product(&req.id).await.map_err(|e| Status::not_found(e))?;
        Ok(Response::new(JsonResponse { json }))
    }

    async fn get_product_by_slug(&self, request: Request<SlugRequest>) -> Result<Response<JsonResponse>, Status> {
        let req = request.into_inner();
        let json = self.service.get_product_by_slug(&req.slug).await.map_err(|e| Status::not_found(e))?;
        Ok(Response::new(JsonResponse { json }))
    }

    async fn create_product(&self, request: Request<CreateProductRequest>) -> Result<Response<JsonResponse>, Status> {
        let req = request.into_inner();
        let json = self.service.create_product(&req.json).await.map_err(|e| Status::invalid_argument(e))?;
        Ok(Response::new(JsonResponse { json }))
    }

    async fn update_product(&self, request: Request<UpdateProductRequest>) -> Result<Response<JsonResponse>, Status> {
        let req = request.into_inner();
        let json = self.service.update_product(&req.id, &req.json).await.map_err(|e| Status::invalid_argument(e))?;
        Ok(Response::new(JsonResponse { json }))
    }

    async fn delete_product(&self, request: Request<IdRequest>) -> Result<Response<Empty>, Status> {
        let req = request.into_inner();
        self.service.delete_product(&req.id).await.map_err(|e| Status::internal(e))?;
        Ok(Response::new(Empty {}))
    }

    async fn list_product_translations(&self, request: Request<IdRequest>) -> Result<Response<JsonResponse>, Status> {
        let req = request.into_inner();
        let json = self.service.list_product_translations(&req.id).await.map_err(|e| Status::internal(e))?;
        Ok(Response::new(JsonResponse { json }))
    }

    async fn upsert_product_translation(&self, request: Request<UpsertProductTranslationRequest>) -> Result<Response<JsonResponse>, Status> {
        let req = request.into_inner();
        let json = self.service.upsert_product_translation(&req.product_id, &req.locale, &req.json).await.map_err(|e| Status::internal(e))?;
        Ok(Response::new(JsonResponse { json }))
    }

    async fn list_variants(&self, request: Request<IdRequest>) -> Result<Response<JsonResponse>, Status> {
        let req = request.into_inner();
        let json = self.service.list_variants(&req.id).await.map_err(|e| Status::internal(e))?;
        Ok(Response::new(JsonResponse { json }))
    }

    async fn get_variant(&self, request: Request<IdRequest>) -> Result<Response<JsonResponse>, Status> {
        let req = request.into_inner();
        let json = self.service.get_variant(&req.id).await.map_err(|e| Status::not_found(e))?;
        Ok(Response::new(JsonResponse { json }))
    }

    async fn create_variant(&self, request: Request<CreateVariantRequest>) -> Result<Response<JsonResponse>, Status> {
        let req = request.into_inner();
        let json = self.service.create_variant(&req.json).await.map_err(|e| Status::invalid_argument(e))?;
        Ok(Response::new(JsonResponse { json }))
    }

    async fn update_variant(&self, request: Request<UpdateVariantRequest>) -> Result<Response<JsonResponse>, Status> {
        let req = request.into_inner();
        let json = self.service.update_variant(&req.id, &req.json).await.map_err(|e| Status::invalid_argument(e))?;
        Ok(Response::new(JsonResponse { json }))
    }

    async fn delete_variant(&self, request: Request<IdRequest>) -> Result<Response<Empty>, Status> {
        let req = request.into_inner();
        self.service.delete_variant(&req.id).await.map_err(|e| Status::internal(e))?;
        Ok(Response::new(Empty {}))
    }

    async fn list_images(&self, request: Request<IdRequest>) -> Result<Response<JsonResponse>, Status> {
        let req = request.into_inner();
        let json = self.service.list_images(&req.id).await.map_err(|e| Status::internal(e))?;
        Ok(Response::new(JsonResponse { json }))
    }

    async fn create_image(&self, request: Request<CreateImageRequest>) -> Result<Response<JsonResponse>, Status> {
        let req = request.into_inner();
        let json = self.service.create_image(&req.json).await.map_err(|e| Status::invalid_argument(e))?;
        Ok(Response::new(JsonResponse { json }))
    }

    async fn delete_image(&self, request: Request<IdRequest>) -> Result<Response<Empty>, Status> {
        let req = request.into_inner();
        self.service.delete_image(&req.id).await.map_err(|e| Status::internal(e))?;
        Ok(Response::new(Empty {}))
    }

    async fn list_product_options(&self, request: Request<IdRequest>) -> Result<Response<JsonResponse>, Status> {
        let req = request.into_inner();
        let json = self.service.list_product_options(&req.id).await.map_err(|e| Status::internal(e))?;
        Ok(Response::new(JsonResponse { json }))
    }

    async fn create_product_option(&self, request: Request<CreateProductOptionRequest>) -> Result<Response<JsonResponse>, Status> {
        let req = request.into_inner();
        let json = self.service.create_product_option(&req.json).await.map_err(|e| Status::invalid_argument(e))?;
        Ok(Response::new(JsonResponse { json }))
    }

    async fn delete_product_option(&self, request: Request<IdRequest>) -> Result<Response<Empty>, Status> {
        let req = request.into_inner();
        self.service.delete_product_option(&req.id).await.map_err(|e| Status::internal(e))?;
        Ok(Response::new(Empty {}))
    }

    async fn list_option_values(&self, request: Request<IdRequest>) -> Result<Response<JsonResponse>, Status> {
        let req = request.into_inner();
        let json = self.service.list_option_values(&req.id).await.map_err(|e| Status::internal(e))?;
        Ok(Response::new(JsonResponse { json }))
    }

    async fn create_option_value(&self, request: Request<CreateOptionValueRequest>) -> Result<Response<JsonResponse>, Status> {
        let req = request.into_inner();
        let json = self.service.create_option_value(&req.json).await.map_err(|e| Status::invalid_argument(e))?;
        Ok(Response::new(JsonResponse { json }))
    }

    async fn delete_option_value(&self, request: Request<IdRequest>) -> Result<Response<Empty>, Status> {
        let req = request.into_inner();
        self.service.delete_option_value(&req.id).await.map_err(|e| Status::internal(e))?;
        Ok(Response::new(Empty {}))
    }

    async fn list_collections(&self, _request: Request<Empty>) -> Result<Response<JsonResponse>, Status> {
        let json = self.service.list_collections().await.map_err(|e| Status::internal(e))?;
        Ok(Response::new(JsonResponse { json }))
    }

    async fn get_collection(&self, request: Request<IdRequest>) -> Result<Response<JsonResponse>, Status> {
        let req = request.into_inner();
        let json = self.service.get_collection(&req.id).await.map_err(|e| Status::not_found(e))?;
        Ok(Response::new(JsonResponse { json }))
    }

    async fn create_collection(&self, request: Request<CreateCollectionRequest>) -> Result<Response<JsonResponse>, Status> {
        let req = request.into_inner();
        let json = self.service.create_collection(&req.json).await.map_err(|e| Status::invalid_argument(e))?;
        Ok(Response::new(JsonResponse { json }))
    }

    async fn add_product_to_collection(&self, request: Request<AddProductToCollectionRequest>) -> Result<Response<Empty>, Status> {
        let req = request.into_inner();
        self.service.add_product_to_collection(&req.collection_id, &req.json).await.map_err(|e| Status::internal(e))?;
        Ok(Response::new(Empty {}))
    }

    async fn remove_product_from_collection(&self, request: Request<RemoveProductFromCollectionRequest>) -> Result<Response<Empty>, Status> {
        let req = request.into_inner();
        self.service.remove_product_from_collection(&req.collection_id, &req.product_id).await.map_err(|e| Status::internal(e))?;
        Ok(Response::new(Empty {}))
    }

    async fn list_collection_products(&self, request: Request<IdRequest>) -> Result<Response<JsonResponse>, Status> {
        let req = request.into_inner();
        let json = self.service.list_collection_products(&req.id).await.map_err(|e| Status::internal(e))?;
        Ok(Response::new(JsonResponse { json }))
    }

    async fn adjust_stock(&self, request: Request<AdjustStockRequest>) -> Result<Response<JsonResponse>, Status> {
        let req = request.into_inner();
        let json = self.service.adjust_stock(&req.json).await.map_err(|e| Status::invalid_argument(e))?;
        Ok(Response::new(JsonResponse { json }))
    }

    async fn list_stock_adjustments(&self, request: Request<ListStockAdjustmentsRequest>) -> Result<Response<JsonResponse>, Status> {
        let req = request.into_inner();
        let json = self.service.list_stock_adjustments(&req.variant_id, req.limit).await.map_err(|e| Status::internal(e))?;
        Ok(Response::new(JsonResponse { json }))
    }

    async fn add_cross_sell(&self, request: Request<AddCrossSellRequest>) -> Result<Response<Empty>, Status> {
        let req = request.into_inner();
        self.service.add_cross_sell(&req.source_id, &req.json).await.map_err(|e| Status::internal(e))?;
        Ok(Response::new(Empty {}))
    }

    async fn list_cross_sells(&self, request: Request<IdRequest>) -> Result<Response<JsonResponse>, Status> {
        let req = request.into_inner();
        let json = self.service.list_cross_sells(&req.id).await.map_err(|e| Status::internal(e))?;
        Ok(Response::new(JsonResponse { json }))
    }

    async fn list_ai_suggestions(&self, request: Request<IdRequest>) -> Result<Response<JsonResponse>, Status> {
        let req = request.into_inner();
        let json = self.service.list_ai_suggestions(&req.id).await.map_err(|e| Status::internal(e))?;
        Ok(Response::new(JsonResponse { json }))
    }

    async fn approve_ai_suggestion(&self, request: Request<IdRequest>) -> Result<Response<Empty>, Status> {
        let req = request.into_inner();
        self.service.approve_ai_suggestion(&req.id).await.map_err(|e| Status::internal(e))?;
        Ok(Response::new(Empty {}))
    }

    async fn reject_ai_suggestion(&self, request: Request<IdRequest>) -> Result<Response<Empty>, Status> {
        let req = request.into_inner();
        self.service.reject_ai_suggestion(&req.id).await.map_err(|e| Status::internal(e))?;
        Ok(Response::new(Empty {}))
    }

    async fn list_redirects(&self, _request: Request<Empty>) -> Result<Response<JsonResponse>, Status> {
        let json = self.service.list_redirects().await.map_err(|e| Status::internal(e))?;
        Ok(Response::new(JsonResponse { json }))
    }

    async fn create_redirect(&self, request: Request<CreateRedirectRequest>) -> Result<Response<JsonResponse>, Status> {
        let req = request.into_inner();
        let json = self.service.create_redirect(&req.json).await.map_err(|e| Status::invalid_argument(e))?;
        Ok(Response::new(JsonResponse { json }))
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();
    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPool::connect(&db_url).await.expect("Failed to connect to database");

    let addr = "0.0.0.0:50100".parse().expect("Invalid address");
    let server = ErpCatalogServer::new(pool);

    println!("ERP Catalog service listening on {}", addr);
    tonic::transport::Server::builder()
        .add_service(ErpCatalogServiceServer::new(server))
        .serve(addr)
        .await?;

    Ok(())
}
