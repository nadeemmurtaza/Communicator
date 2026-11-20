mod erp_inventory {
    include!(concat!(env!("OUT_DIR"), "/newax.erp_inventory.v1.rs"));
}

use erp_inventory::erp_inventory_service_server::{ErpInventoryService, ErpInventoryServiceServer};
use erp_inventory::{JsonBody, JsonResponse,
    ListLocationsRequest, GetLocationRequest, CreateLocationRequest, UpdateLocationRequest,
    GetStockByLocationRequest, GetStockByVariantRequest, UpdateVariantStockRequest, GetTotalStockRequest,
    ListStockMovementsRequest, GetStockMovementRequest, CreateStockMovementRequest, GetMovementsByVariantRequest,
    ListStockVariancesRequest, GetStockVarianceRequest, CreateStockVarianceRequest, ResolveVarianceRequest,
    ListSpotCheckPoliciesRequest, CreateSpotCheckPolicyRequest,
    ListSpotChecksRequest, GetSpotCheckRequest, CreateSpotCheckRequest, StartSpotCheckRequest, CompleteSpotCheckRequest,
    ListSpotCheckLinesRequest, CreateSpotCheckLineRequest, UpdateSpotCheckLineRequest,
    GetVariantSalesStatRequest, ListSlowMoversRequest,
    GetVariantForecastRequest, CreateVariantForecastRequest, ListForecastsRequest,
    ListMarkdownSuggestionsRequest, CreateMarkdownSuggestionRequest, UpdateMarkdownSuggestionStatusRequest,
    ListLotsRequest, GetLotRequest, GetExpiringLotsRequest, CreateLotRequest,
    ListLotStockRequest, CreateLotStockRequest,
    ListLotAllocationsRequest, CreateLotAllocationRequest,
    ListSerializedUnitsRequest, GetSerializedUnitRequest, GetSerializedUnitBySerialRequest, CreateSerializedUnitRequest, UpdateSerializedUnitStatusRequest,
    ListStockCountsRequest, GetStockCountRequest, CreateStockCountRequest, StartStockCountRequest, CompleteStockCountRequest, CancelStockCountRequest,
    ListStockCountLinesRequest, CreateStockCountLineRequest, UpdateStockCountLineRequest,
    GetInventoryStatsRequest};
use newax_erp_inventory::ErpInventoryService as BusinessService;
use sqlx::PgPool;
use tonic::{Request, Response, Status};

#[derive(Clone)]
struct ErpInventoryServer {
    service: BusinessService,
}

impl ErpInventoryServer {
    fn new(pool: PgPool) -> Self {
        Self {
            service: BusinessService::new(pool),
        }
    }
}

#[tonic::async_trait]
impl ErpInventoryService for ErpInventoryServer {
    // Locations
    async fn list_locations(&self, request: Request<ListLocationsRequest>) -> Result<Response<JsonResponse>, Status> {
        let req = request.into_inner();
        let is_active = if req.has_is_active { Some(req.is_active) } else { None };
        let json = self.service.list_locations(is_active).await.map_err(|e| Status::internal(e))?;
        Ok(Response::new(JsonResponse { json }))
    }

    async fn get_location(&self, request: Request<GetLocationRequest>) -> Result<Response<JsonResponse>, Status> {
        let req = request.into_inner();
        let json = self.service.get_location(&req.id).await.map_err(|e| Status::not_found(e))?;
        Ok(Response::new(JsonResponse { json }))
    }

    async fn create_location(&self, request: Request<CreateLocationRequest>) -> Result<Response<JsonResponse>, Status> {
        let req = request.into_inner();
        let json = self.service.create_location(&req.body.unwrap().json).await.map_err(|e| Status::invalid_argument(e))?;
        Ok(Response::new(JsonResponse { json }))
    }

    async fn update_location(&self, request: Request<UpdateLocationRequest>) -> Result<Response<JsonResponse>, Status> {
        let req = request.into_inner();
        let json = self.service.update_location(&req.id, &req.body.unwrap().json).await.map_err(|e| Status::invalid_argument(e))?;
        Ok(Response::new(JsonResponse { json }))
    }

    // Variant Location Stock
    async fn get_stock_by_location(&self, request: Request<GetStockByLocationRequest>) -> Result<Response<JsonResponse>, Status> {
        let req = request.into_inner();
        let json = self.service.get_stock_by_location(&req.location_id).await.map_err(|e| Status::internal(e))?;
        Ok(Response::new(JsonResponse { json }))
    }

    async fn get_stock_by_variant(&self, request: Request<GetStockByVariantRequest>) -> Result<Response<JsonResponse>, Status> {
        let req = request.into_inner();
        let json = self.service.get_stock_by_variant(&req.variant_id).await.map_err(|e| Status::internal(e))?;
        Ok(Response::new(JsonResponse { json }))
    }

    async fn update_variant_stock(&self, request: Request<UpdateVariantStockRequest>) -> Result<Response<JsonResponse>, Status> {
        let req = request.into_inner();
        let json = self.service.update_variant_location_stock(&req.variant_id, &req.location_id, req.quantity).await.map_err(|e| Status::internal(e))?;
        Ok(Response::new(JsonResponse { json }))
    }

    async fn get_total_stock(&self, request: Request<GetTotalStockRequest>) -> Result<Response<JsonResponse>, Status> {
        let req = request.into_inner();
        let json = self.service.get_total_stock(&req.variant_id).await.map_err(|e| Status::internal(e))?;
        Ok(Response::new(JsonResponse { json }))
    }

    // Stock Movements
    async fn list_stock_movements(&self, request: Request<ListStockMovementsRequest>) -> Result<Response<JsonResponse>, Status> {
        let req = request.into_inner();
        let variant_id = if req.variant_id.is_empty() { None } else { Some(req.variant_id.as_str()) };
        let reason = if req.reason.is_empty() { None } else { Some(req.reason.as_str()) };
        let location_id = if req.location_id.is_empty() { None } else { Some(req.location_id.as_str()) };
        let json = self.service.list_stock_movements(variant_id, reason, location_id, req.limit, req.offset).await.map_err(|e| Status::internal(e))?;
        Ok(Response::new(JsonResponse { json }))
    }

    async fn get_stock_movement(&self, request: Request<GetStockMovementRequest>) -> Result<Response<JsonResponse>, Status> {
        let req = request.into_inner();
        let json = self.service.get_stock_movement(&req.id).await.map_err(|e| Status::not_found(e))?;
        Ok(Response::new(JsonResponse { json }))
    }

    async fn create_stock_movement(&self, request: Request<CreateStockMovementRequest>) -> Result<Response<JsonResponse>, Status> {
        let req = request.into_inner();
        let json = self.service.create_stock_movement(&req.body.unwrap().json).await.map_err(|e| Status::invalid_argument(e))?;
        Ok(Response::new(JsonResponse { json }))
    }

    async fn get_movements_by_variant(&self, request: Request<GetMovementsByVariantRequest>) -> Result<Response<JsonResponse>, Status> {
        let req = request.into_inner();
        let json = self.service.get_movements_by_variant(&req.variant_id, req.limit).await.map_err(|e| Status::internal(e))?;
        Ok(Response::new(JsonResponse { json }))
    }

    // Stock Variances
    async fn list_stock_variances(&self, request: Request<ListStockVariancesRequest>) -> Result<Response<JsonResponse>, Status> {
        let req = request.into_inner();
        let status = if req.status.is_empty() { None } else { Some(req.status.as_str()) };
        let variant_id = if req.variant_id.is_empty() { None } else { Some(req.variant_id.as_str()) };
        let json = self.service.list_stock_variances(status, variant_id, req.limit, req.offset).await.map_err(|e| Status::internal(e))?;
        Ok(Response::new(JsonResponse { json }))
    }

    async fn get_stock_variance(&self, request: Request<GetStockVarianceRequest>) -> Result<Response<JsonResponse>, Status> {
        let req = request.into_inner();
        let json = self.service.get_stock_variance(&req.id).await.map_err(|e| Status::not_found(e))?;
        Ok(Response::new(JsonResponse { json }))
    }

    async fn create_stock_variance(&self, request: Request<CreateStockVarianceRequest>) -> Result<Response<JsonResponse>, Status> {
        let req = request.into_inner();
        let json = self.service.create_stock_variance(&req.body.unwrap().json).await.map_err(|e| Status::invalid_argument(e))?;
        Ok(Response::new(JsonResponse { json }))
    }

    async fn resolve_variance(&self, request: Request<ResolveVarianceRequest>) -> Result<Response<JsonResponse>, Status> {
        let req = request.into_inner();
        let json = self.service.resolve_variance(&req.id, &req.body.unwrap().json).await.map_err(|e| Status::invalid_argument(e))?;
        Ok(Response::new(JsonResponse { json }))
    }

    // Spot Check Policies
    async fn list_spot_check_policies(&self, request: Request<ListSpotCheckPoliciesRequest>) -> Result<Response<JsonResponse>, Status> {
        let req = request.into_inner();
        let is_active = if req.has_is_active { Some(req.is_active) } else { None };
        let json = self.service.list_spot_check_policies(is_active).await.map_err(|e| Status::internal(e))?;
        Ok(Response::new(JsonResponse { json }))
    }

    async fn create_spot_check_policy(&self, request: Request<CreateSpotCheckPolicyRequest>) -> Result<Response<JsonResponse>, Status> {
        let req = request.into_inner();
        let json = self.service.create_spot_check_policy(&req.body.unwrap().json).await.map_err(|e| Status::invalid_argument(e))?;
        Ok(Response::new(JsonResponse { json }))
    }

    // Spot Checks
    async fn list_spot_checks(&self, request: Request<ListSpotChecksRequest>) -> Result<Response<JsonResponse>, Status> {
        let req = request.into_inner();
        let location_id = if req.location_id.is_empty() { None } else { Some(req.location_id.as_str()) };
        let status = if req.status.is_empty() { None } else { Some(req.status.as_str()) };
        let json = self.service.list_spot_checks(location_id, status, req.limit, req.offset).await.map_err(|e| Status::internal(e))?;
        Ok(Response::new(JsonResponse { json }))
    }

    async fn get_spot_check(&self, request: Request<GetSpotCheckRequest>) -> Result<Response<JsonResponse>, Status> {
        let req = request.into_inner();
        let json = self.service.get_spot_check(&req.id).await.map_err(|e| Status::not_found(e))?;
        Ok(Response::new(JsonResponse { json }))
    }

    async fn create_spot_check(&self, request: Request<CreateSpotCheckRequest>) -> Result<Response<JsonResponse>, Status> {
        let req = request.into_inner();
        let json = self.service.create_spot_check(&req.body.unwrap().json).await.map_err(|e| Status::invalid_argument(e))?;
        Ok(Response::new(JsonResponse { json }))
    }

    async fn start_spot_check(&self, request: Request<StartSpotCheckRequest>) -> Result<Response<JsonResponse>, Status> {
        let req = request.into_inner();
        let json = self.service.start_spot_check(&req.id).await.map_err(|e| Status::internal(e))?;
        Ok(Response::new(JsonResponse { json }))
    }

    async fn complete_spot_check(&self, request: Request<CompleteSpotCheckRequest>) -> Result<Response<JsonResponse>, Status> {
        let req = request.into_inner();
        let json = self.service.complete_spot_check(&req.id).await.map_err(|e| Status::internal(e))?;
        Ok(Response::new(JsonResponse { json }))
    }

    // Spot Check Lines
    async fn list_spot_check_lines(&self, request: Request<ListSpotCheckLinesRequest>) -> Result<Response<JsonResponse>, Status> {
        let req = request.into_inner();
        let json = self.service.list_spot_check_lines(&req.spot_check_id).await.map_err(|e| Status::internal(e))?;
        Ok(Response::new(JsonResponse { json }))
    }

    async fn create_spot_check_line(&self, request: Request<CreateSpotCheckLineRequest>) -> Result<Response<JsonResponse>, Status> {
        let req = request.into_inner();
        let json = self.service.create_spot_check_line(&req.body.unwrap().json).await.map_err(|e| Status::invalid_argument(e))?;
        Ok(Response::new(JsonResponse { json }))
    }

    async fn update_spot_check_line(&self, request: Request<UpdateSpotCheckLineRequest>) -> Result<Response<JsonResponse>, Status> {
        let req = request.into_inner();
        let json = self.service.update_spot_check_line(&req.id, &req.body.unwrap().json).await.map_err(|e| Status::invalid_argument(e))?;
        Ok(Response::new(JsonResponse { json }))
    }

    // Variant Sales Stats
    async fn get_variant_sales_stat(&self, request: Request<GetVariantSalesStatRequest>) -> Result<Response<JsonResponse>, Status> {
        let req = request.into_inner();
        let json = self.service.get_variant_sales_stat(&req.variant_id).await.map_err(|e| Status::not_found(e))?;
        Ok(Response::new(JsonResponse { json }))
    }

    async fn list_slow_movers(&self, request: Request<ListSlowMoversRequest>) -> Result<Response<JsonResponse>, Status> {
        let req = request.into_inner();
        let json = self.service.list_slow_movers(req.limit, req.offset).await.map_err(|e| Status::internal(e))?;
        Ok(Response::new(JsonResponse { json }))
    }

    // Variant Forecasts
    async fn get_variant_forecast(&self, request: Request<GetVariantForecastRequest>) -> Result<Response<JsonResponse>, Status> {
        let req = request.into_inner();
        let json = self.service.get_variant_forecast(&req.variant_id).await.map_err(|e| Status::not_found(e))?;
        Ok(Response::new(JsonResponse { json }))
    }

    async fn create_variant_forecast(&self, request: Request<CreateVariantForecastRequest>) -> Result<Response<JsonResponse>, Status> {
        let req = request.into_inner();
        let json = self.service.create_variant_forecast(&req.body.unwrap().json).await.map_err(|e| Status::invalid_argument(e))?;
        Ok(Response::new(JsonResponse { json }))
    }

    async fn list_forecasts(&self, request: Request<ListForecastsRequest>) -> Result<Response<JsonResponse>, Status> {
        let req = request.into_inner();
        let json = self.service.list_forecasts(req.limit, req.offset).await.map_err(|e| Status::internal(e))?;
        Ok(Response::new(JsonResponse { json }))
    }

    // Markdown Suggestions
    async fn list_markdown_suggestions(&self, request: Request<ListMarkdownSuggestionsRequest>) -> Result<Response<JsonResponse>, Status> {
        let req = request.into_inner();
        let status = if req.status.is_empty() { None } else { Some(req.status.as_str()) };
        let json = self.service.list_markdown_suggestions(status, req.limit, req.offset).await.map_err(|e| Status::internal(e))?;
        Ok(Response::new(JsonResponse { json }))
    }

    async fn create_markdown_suggestion(&self, request: Request<CreateMarkdownSuggestionRequest>) -> Result<Response<JsonResponse>, Status> {
        let req = request.into_inner();
        let json = self.service.create_markdown_suggestion(&req.body.unwrap().json).await.map_err(|e| Status::invalid_argument(e))?;
        Ok(Response::new(JsonResponse { json }))
    }

    async fn update_markdown_suggestion_status(&self, request: Request<UpdateMarkdownSuggestionStatusRequest>) -> Result<Response<JsonResponse>, Status> {
        let req = request.into_inner();
        let json = self.service.update_markdown_suggestion_status(&req.id, &req.body.unwrap().json).await.map_err(|e| Status::invalid_argument(e))?;
        Ok(Response::new(JsonResponse { json }))
    }

    // Lots
    async fn list_lots(&self, request: Request<ListLotsRequest>) -> Result<Response<JsonResponse>, Status> {
        let req = request.into_inner();
        let product_variant_id = if req.product_variant_id.is_empty() { None } else { Some(req.product_variant_id.as_str()) };
        let json = self.service.list_lots(product_variant_id, req.limit, req.offset).await.map_err(|e| Status::internal(e))?;
        Ok(Response::new(JsonResponse { json }))
    }

    async fn get_lot(&self, request: Request<GetLotRequest>) -> Result<Response<JsonResponse>, Status> {
        let req = request.into_inner();
        let json = self.service.get_lot(&req.id).await.map_err(|e| Status::not_found(e))?;
        Ok(Response::new(JsonResponse { json }))
    }

    async fn get_expiring_lots(&self, request: Request<GetExpiringLotsRequest>) -> Result<Response<JsonResponse>, Status> {
        let req = request.into_inner();
        let json = self.service.get_expiring_lots(req.days).await.map_err(|e| Status::internal(e))?;
        Ok(Response::new(JsonResponse { json }))
    }

    async fn create_lot(&self, request: Request<CreateLotRequest>) -> Result<Response<JsonResponse>, Status> {
        let req = request.into_inner();
        let json = self.service.create_lot(&req.body.unwrap().json).await.map_err(|e| Status::invalid_argument(e))?;
        Ok(Response::new(JsonResponse { json }))
    }

    // Lot Stock
    async fn list_lot_stock(&self, request: Request<ListLotStockRequest>) -> Result<Response<JsonResponse>, Status> {
        let req = request.into_inner();
        let json = self.service.list_lot_stock(&req.lot_id).await.map_err(|e| Status::internal(e))?;
        Ok(Response::new(JsonResponse { json }))
    }

    async fn create_lot_stock(&self, request: Request<CreateLotStockRequest>) -> Result<Response<JsonResponse>, Status> {
        let req = request.into_inner();
        let json = self.service.create_lot_stock(&req.body.unwrap().json).await.map_err(|e| Status::invalid_argument(e))?;
        Ok(Response::new(JsonResponse { json }))
    }

    // Lot Allocations
    async fn list_lot_allocations(&self, request: Request<ListLotAllocationsRequest>) -> Result<Response<JsonResponse>, Status> {
        let req = request.into_inner();
        let json = self.service.list_lot_allocations(&req.lot_id).await.map_err(|e| Status::internal(e))?;
        Ok(Response::new(JsonResponse { json }))
    }

    async fn create_lot_allocation(&self, request: Request<CreateLotAllocationRequest>) -> Result<Response<JsonResponse>, Status> {
        let req = request.into_inner();
        let json = self.service.create_lot_allocation(&req.body.unwrap().json).await.map_err(|e| Status::invalid_argument(e))?;
        Ok(Response::new(JsonResponse { json }))
    }

    // Serialized Units
    async fn list_serialized_units(&self, request: Request<ListSerializedUnitsRequest>) -> Result<Response<JsonResponse>, Status> {
        let req = request.into_inner();
        let product_variant_id = if req.product_variant_id.is_empty() { None } else { Some(req.product_variant_id.as_str()) };
        let status = if req.status.is_empty() { None } else { Some(req.status.as_str()) };
        let json = self.service.list_serialized_units(product_variant_id, status, req.limit, req.offset).await.map_err(|e| Status::internal(e))?;
        Ok(Response::new(JsonResponse { json }))
    }

    async fn get_serialized_unit(&self, request: Request<GetSerializedUnitRequest>) -> Result<Response<JsonResponse>, Status> {
        let req = request.into_inner();
        let json = self.service.get_serialized_unit(&req.id).await.map_err(|e| Status::not_found(e))?;
        Ok(Response::new(JsonResponse { json }))
    }

    async fn get_serialized_unit_by_serial(&self, request: Request<GetSerializedUnitBySerialRequest>) -> Result<Response<JsonResponse>, Status> {
        let req = request.into_inner();
        let json = self.service.get_serialized_unit_by_serial(&req.serial_number).await.map_err(|e| Status::not_found(e))?;
        Ok(Response::new(JsonResponse { json }))
    }

    async fn create_serialized_unit(&self, request: Request<CreateSerializedUnitRequest>) -> Result<Response<JsonResponse>, Status> {
        let req = request.into_inner();
        let json = self.service.create_serialized_unit(&req.body.unwrap().json).await.map_err(|e| Status::invalid_argument(e))?;
        Ok(Response::new(JsonResponse { json }))
    }

    async fn update_serialized_unit_status(&self, request: Request<UpdateSerializedUnitStatusRequest>) -> Result<Response<JsonResponse>, Status> {
        let req = request.into_inner();
        let json = self.service.update_serialized_unit_status(&req.id, &req.body.unwrap().json).await.map_err(|e| Status::invalid_argument(e))?;
        Ok(Response::new(JsonResponse { json }))
    }

    // Stock Counts
    async fn list_stock_counts(&self, request: Request<ListStockCountsRequest>) -> Result<Response<JsonResponse>, Status> {
        let req = request.into_inner();
        let status = if req.status.is_empty() { None } else { Some(req.status.as_str()) };
        let json = self.service.list_stock_counts(status, req.limit, req.offset).await.map_err(|e| Status::internal(e))?;
        Ok(Response::new(JsonResponse { json }))
    }

    async fn get_stock_count(&self, request: Request<GetStockCountRequest>) -> Result<Response<JsonResponse>, Status> {
        let req = request.into_inner();
        let json = self.service.get_stock_count(&req.id).await.map_err(|e| Status::not_found(e))?;
        Ok(Response::new(JsonResponse { json }))
    }

    async fn create_stock_count(&self, request: Request<CreateStockCountRequest>) -> Result<Response<JsonResponse>, Status> {
        let req = request.into_inner();
        let json = self.service.create_stock_count(&req.body.unwrap().json).await.map_err(|e| Status::invalid_argument(e))?;
        Ok(Response::new(JsonResponse { json }))
    }

    async fn start_stock_count(&self, request: Request<StartStockCountRequest>) -> Result<Response<JsonResponse>, Status> {
        let req = request.into_inner();
        let json = self.service.start_stock_count(&req.id).await.map_err(|e| Status::internal(e))?;
        Ok(Response::new(JsonResponse { json }))
    }

    async fn complete_stock_count(&self, request: Request<CompleteStockCountRequest>) -> Result<Response<JsonResponse>, Status> {
        let req = request.into_inner();
        let json = self.service.complete_stock_count(&req.id).await.map_err(|e| Status::internal(e))?;
        Ok(Response::new(JsonResponse { json }))
    }

    async fn cancel_stock_count(&self, request: Request<CancelStockCountRequest>) -> Result<Response<JsonResponse>, Status> {
        let req = request.into_inner();
        let json = self.service.cancel_stock_count(&req.id).await.map_err(|e| Status::internal(e))?;
        Ok(Response::new(JsonResponse { json }))
    }

    // Stock Count Lines
    async fn list_stock_count_lines(&self, request: Request<ListStockCountLinesRequest>) -> Result<Response<JsonResponse>, Status> {
        let req = request.into_inner();
        let json = self.service.list_stock_count_lines(&req.stock_count_id).await.map_err(|e| Status::internal(e))?;
        Ok(Response::new(JsonResponse { json }))
    }

    async fn create_stock_count_line(&self, request: Request<CreateStockCountLineRequest>) -> Result<Response<JsonResponse>, Status> {
        let req = request.into_inner();
        let json = self.service.create_stock_count_line(&req.body.unwrap().json).await.map_err(|e| Status::invalid_argument(e))?;
        Ok(Response::new(JsonResponse { json }))
    }

    async fn update_stock_count_line(&self, request: Request<UpdateStockCountLineRequest>) -> Result<Response<JsonResponse>, Status> {
        let req = request.into_inner();
        let json = self.service.update_stock_count_line(&req.id, &req.body.unwrap().json).await.map_err(|e| Status::invalid_argument(e))?;
        Ok(Response::new(JsonResponse { json }))
    }

    // Stats
    async fn get_inventory_stats(&self, _request: Request<GetInventoryStatsRequest>) -> Result<Response<JsonResponse>, Status> {
        let json = self.service.get_inventory_stats().await.map_err(|e| Status::internal(e))?;
        Ok(Response::new(JsonResponse { json }))
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();
    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPool::connect(&db_url).await.expect("Failed to connect to database");

    let addr = "0.0.0.0:50101".parse().expect("Invalid address");
    let server = ErpInventoryServer::new(pool);

    println!("ERP Inventory service listening on {}", addr);
    tonic::transport::Server::builder()
        .add_service(ErpInventoryServiceServer::new(server))
        .serve(addr)
        .await?;

    Ok(())
}
