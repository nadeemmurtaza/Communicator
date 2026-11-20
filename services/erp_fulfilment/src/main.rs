use newax_erp_fulfilment::ErpFulfilmentService;
use serde_json::json;
use sqlx::PgPool;
use tonic::{Request, Response, Status};

pub mod proto {
    tonic::include_proto!("newax.erp_fulfilment.v1");
}

use proto::erp_fulfilment_service_server::{ErpFulfilmentService as ErpFulfilmentServiceTrait, ErpFulfilmentServiceServer};
use proto::*;

pub struct ErpFulfilmentServiceImpl {
    inner: newax_erp_fulfilment::ErpFulfilmentService,
}

impl ErpFulfilmentServiceImpl {
    pub fn new(pool: PgPool) -> Self {
        Self {
            inner: newax_erp_fulfilment::ErpFulfilmentService::new(pool),
        }
    }
}

#[tonic::async_trait]
impl ErpFulfilmentServiceTrait for ErpFulfilmentServiceImpl {
    async fn get_summary(&self, _request: Request<Empty>) -> Result<Response<JsonResponse>, Status> {
        match self.inner.get_summary().await {
            Ok(result) => Ok(Response::new(JsonResponse { json: serde_json::to_string(&result).unwrap_or_default() })),
            Err(e) => Err(Status::internal(e.to_string())),
        }
    }

    async fn list_shipping_zones(&self, request: Request<ListShippingZonesRequest>) -> Result<Response<JsonResponse>, Status> {
        let active_only = request.into_inner().active_only;
        match self.inner.list_shipping_zones(active_only).await {
            Ok(result) => Ok(Response::new(JsonResponse { json: serde_json::to_string(&result).unwrap_or_default() })),
            Err(e) => Err(Status::internal(e)),
        }
    }

    async fn get_shipping_zone(&self, request: Request<IdRequest>) -> Result<Response<JsonResponse>, Status> {
        let id = request.into_inner().id;
        match self.inner.get_shipping_zone(&id).await {
            Ok(result) => Ok(Response::new(JsonResponse { json: serde_json::to_string(&result).unwrap_or_default() })),
            Err(e) => Err(Status::not_found(e)),
        }
    }

    async fn create_shipping_zone(&self, request: Request<CreateShippingZoneRequest>) -> Result<Response<JsonResponse>, Status> {
        let req = request.into_inner();
        let parsed: newax_erp_fulfilment::CreateShippingZoneRequest = serde_json::from_str(&req.json).map_err(|e| Status::invalid_argument(e.to_string()))?;
        match self.inner.create_shipping_zone(&parsed).await {
            Ok(result) => Ok(Response::new(JsonResponse { json: serde_json::to_string(&result).unwrap_or_default() })),
            Err(e) => Err(Status::invalid_argument(e)),
        }
    }

    async fn update_shipping_zone(&self, request: Request<UpdateShippingZoneRequest>) -> Result<Response<JsonResponse>, Status> {
        let req = request.into_inner();
        let parsed: newax_erp_fulfilment::UpdateShippingZoneRequest = serde_json::from_str(&req.json).map_err(|e| Status::invalid_argument(e.to_string()))?;
        match self.inner.update_shipping_zone(&req.zone_id, &parsed).await {
            Ok(result) => Ok(Response::new(JsonResponse { json: serde_json::to_string(&result).unwrap_or_default() })),
            Err(e) => Err(Status::invalid_argument(e)),
        }
    }

    async fn delete_shipping_zone(&self, request: Request<IdRequest>) -> Result<Response<Empty>, Status> {
        let id = request.into_inner().id;
        match self.inner.delete_shipping_zone(&id).await {
            Ok(()) => Ok(Response::new(Empty {})),
            Err(e) => Err(Status::internal(e)),
        }
    }

    async fn list_shipping_methods(&self, request: Request<ListShippingMethodsRequest>) -> Result<Response<JsonResponse>, Status> {
        let active_only = request.into_inner().active_only;
        match self.inner.list_shipping_methods(active_only).await {
            Ok(result) => Ok(Response::new(JsonResponse { json: serde_json::to_string(&result).unwrap_or_default() })),
            Err(e) => Err(Status::internal(e)),
        }
    }

    async fn get_shipping_method(&self, request: Request<IdRequest>) -> Result<Response<JsonResponse>, Status> {
        let id = request.into_inner().id;
        match self.inner.get_shipping_method(&id).await {
            Ok(result) => Ok(Response::new(JsonResponse { json: serde_json::to_string(&result).unwrap_or_default() })),
            Err(e) => Err(Status::not_found(e)),
        }
    }

    async fn create_shipping_method(&self, request: Request<CreateShippingMethodRequest>) -> Result<Response<JsonResponse>, Status> {
        let req = request.into_inner();
        let parsed: newax_erp_fulfilment::CreateShippingMethodRequest = serde_json::from_str(&req.json).map_err(|e| Status::invalid_argument(e.to_string()))?;
        match self.inner.create_shipping_method(&parsed).await {
            Ok(result) => Ok(Response::new(JsonResponse { json: serde_json::to_string(&result).unwrap_or_default() })),
            Err(e) => Err(Status::invalid_argument(e)),
        }
    }

    async fn update_shipping_method(&self, request: Request<UpdateShippingMethodRequest>) -> Result<Response<JsonResponse>, Status> {
        let req = request.into_inner();
        let parsed: newax_erp_fulfilment::UpdateShippingMethodRequest = serde_json::from_str(&req.json).map_err(|e| Status::invalid_argument(e.to_string()))?;
        match self.inner.update_shipping_method(&req.method_id, &parsed).await {
            Ok(result) => Ok(Response::new(JsonResponse { json: serde_json::to_string(&result).unwrap_or_default() })),
            Err(e) => Err(Status::invalid_argument(e)),
        }
    }

    async fn delete_shipping_method(&self, request: Request<IdRequest>) -> Result<Response<Empty>, Status> {
        let id = request.into_inner().id;
        match self.inner.delete_shipping_method(&id).await {
            Ok(()) => Ok(Response::new(Empty {})),
            Err(e) => Err(Status::internal(e)),
        }
    }

    async fn list_shipping_rates(&self, _request: Request<Empty>) -> Result<Response<JsonResponse>, Status> {
        match self.inner.list_shipping_rates(None, None).await {
            Ok(result) => Ok(Response::new(JsonResponse { json: serde_json::to_string(&result).unwrap_or_default() })),
            Err(e) => Err(Status::internal(e)),
        }
    }

    async fn create_shipping_rate(&self, request: Request<CreateShippingRateRequest>) -> Result<Response<JsonResponse>, Status> {
        let req = request.into_inner();
        let parsed: newax_erp_fulfilment::CreateShippingRateRequest = serde_json::from_str(&req.json).map_err(|e| Status::invalid_argument(e.to_string()))?;
        match self.inner.create_shipping_rate(&parsed).await {
            Ok(result) => Ok(Response::new(JsonResponse { json: serde_json::to_string(&result).unwrap_or_default() })),
            Err(e) => Err(Status::invalid_argument(e)),
        }
    }

    async fn get_shipping_rate(&self, request: Request<IdRequest>) -> Result<Response<JsonResponse>, Status> {
        let id = request.into_inner().id;
        match self.inner.get_shipping_rate(&id).await {
            Ok(result) => Ok(Response::new(JsonResponse { json: serde_json::to_string(&result).unwrap_or_default() })),
            Err(e) => Err(Status::not_found(e)),
        }
    }

    async fn update_shipping_rate(&self, request: Request<UpdateShippingRateRequest>) -> Result<Response<JsonResponse>, Status> {
        let req = request.into_inner();
        let parsed: newax_erp_fulfilment::CreateShippingRateRequest = serde_json::from_str(&req.json).map_err(|e| Status::invalid_argument(e.to_string()))?;
        match self.inner.update_shipping_rate(&req.rate_id, &parsed).await {
            Ok(result) => Ok(Response::new(JsonResponse { json: serde_json::to_string(&result).unwrap_or_default() })),
            Err(e) => Err(Status::invalid_argument(e)),
        }
    }

    async fn delete_shipping_rate(&self, request: Request<IdRequest>) -> Result<Response<Empty>, Status> {
        let id = request.into_inner().id;
        match self.inner.delete_shipping_rate(&id).await {
            Ok(()) => Ok(Response::new(Empty {})),
            Err(e) => Err(Status::internal(e)),
        }
    }

    async fn list_return_reasons(&self, _request: Request<Empty>) -> Result<Response<JsonResponse>, Status> {
        match self.inner.list_return_reasons().await {
            Ok(result) => Ok(Response::new(JsonResponse { json: serde_json::to_string(&result).unwrap_or_default() })),
            Err(e) => Err(Status::internal(e)),
        }
    }

    async fn list_return_requests(&self, _request: Request<Empty>) -> Result<Response<JsonResponse>, Status> {
        match self.inner.list_return_requests(None, None).await {
            Ok(result) => Ok(Response::new(JsonResponse { json: serde_json::to_string(&result).unwrap_or_default() })),
            Err(e) => Err(Status::internal(e)),
        }
    }

    async fn get_return_request(&self, request: Request<IdRequest>) -> Result<Response<JsonResponse>, Status> {
        let id = request.into_inner().id;
        match self.inner.get_return_request(&id).await {
            Ok((request, items)) => Ok(Response::new(JsonResponse { json: serde_json::to_string(&json!({"request": request, "items": items})).unwrap_or_default() })),
            Err(e) => Err(Status::not_found(e)),
        }
    }

    async fn create_return_request(&self, request: Request<CreateReturnRequestRequest>) -> Result<Response<JsonResponse>, Status> {
        let req = request.into_inner();
        let parsed: newax_erp_fulfilment::CreateReturnRequestRequest = serde_json::from_str(&req.json).map_err(|e| Status::invalid_argument(e.to_string()))?;
        match self.inner.create_return_request(&parsed).await {
            Ok(result) => Ok(Response::new(JsonResponse { json: serde_json::to_string(&result).unwrap_or_default() })),
            Err(e) => Err(Status::invalid_argument(e)),
        }
    }

    async fn update_return_status(&self, request: Request<UpdateReturnStatusRequest>) -> Result<Response<JsonResponse>, Status> {
        let req = request.into_inner();
        let parsed: newax_erp_fulfilment::UpdateReturnStatusRequest = serde_json::from_str(&req.json).map_err(|e| Status::invalid_argument(e.to_string()))?;
        match self.inner.update_return_status(&req.return_id, &parsed).await {
            Ok(result) => Ok(Response::new(JsonResponse { json: serde_json::to_string(&result).unwrap_or_default() })),
            Err(e) => Err(Status::invalid_argument(e)),
        }
    }

    async fn inspect_return_item(&self, request: Request<InspectReturnItemRequest>) -> Result<Response<JsonResponse>, Status> {
        let req = request.into_inner();
        let parsed: newax_erp_fulfilment::InspectReturnItemRequest = serde_json::from_str(&req.json).map_err(|e| Status::invalid_argument(e.to_string()))?;
        match self.inner.inspect_return_item(&parsed).await {
            Ok(result) => Ok(Response::new(JsonResponse { json: serde_json::to_string(&result).unwrap_or_default() })),
            Err(e) => Err(Status::invalid_argument(e)),
        }
    }

    async fn get_return_stats(&self, _request: Request<Empty>) -> Result<Response<JsonResponse>, Status> {
        match self.inner.get_return_stats().await {
            Ok(result) => Ok(Response::new(JsonResponse { json: serde_json::to_string(&result).unwrap_or_default() })),
            Err(e) => Err(Status::internal(e)),
        }
    }

    async fn list_manifests(&self, _request: Request<Empty>) -> Result<Response<JsonResponse>, Status> {
        match self.inner.list_manifests(None, None).await {
            Ok(result) => Ok(Response::new(JsonResponse { json: serde_json::to_string(&result).unwrap_or_default() })),
            Err(e) => Err(Status::internal(e)),
        }
    }

    async fn dispatch_manifest(&self, request: Request<IdRequest>) -> Result<Response<JsonResponse>, Status> {
        let id = request.into_inner().id;
        match self.inner.dispatch_manifest(&id).await {
            Ok(result) => Ok(Response::new(JsonResponse { json: serde_json::to_string(&result).unwrap_or_default() })),
            Err(e) => Err(Status::invalid_argument(e)),
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let pool = sqlx::PgPool::connect(std::env::var("DATABASE_URL").unwrap_or_default().as_str())
        .await
        .expect("Failed to create pool");

    let addr = "0.0.0.0:50097".parse().expect("Invalid address");
    let service = ErpFulfilmentServiceImpl::new(pool);

    tonic::transport::Server::builder()
        .add_service(ErpFulfilmentServiceServer::new(service))
        .serve(addr)
        .await?;

    Ok(())
}
