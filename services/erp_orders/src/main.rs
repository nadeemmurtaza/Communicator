use newax_proto::communicator::erp_orders::v1::erp_orders_service_server::ErpOrdersServiceServer;
use newax_proto::communicator::erp_orders::v1::*;
use newax_erp_orders::ErpOrdersService;
use sqlx::PgPool;
use tonic::{Request, Response, Status};

#[derive(Clone)]
pub struct ErpOrdersServiceImpl {
    inner: ErpOrdersService,
}

impl ErpOrdersServiceImpl {
    pub fn new(pool: PgPool) -> Self {
        Self {
            inner: ErpOrdersService::new(pool),
        }
    }
}

#[tonic::async_trait]
impl ErpOrdersServiceServer for ErpOrdersServiceImpl {
    // Orders
    async fn list_orders(&self, request: Request<ListOrdersRequest>) -> Result<Response<JsonResponse>, Status> {
        let req = request.into_inner();
        let status = req.status.as_deref();
        let channel = req.channel.as_deref();
        let limit = req.limit;
        let offset = req.offset;
        match self.inner.list_orders(status, channel, limit, offset).await {
            Ok(result) => Ok(Response::new(JsonResponse { json: serde_json::to_string(&result).unwrap_or_default() })),
            Err(e) => Err(Status::internal(e)),
        }
    }

    async fn get_order(&self, request: Request<IdRequest>) -> Result<Response<JsonResponse>, Status> {
        let id = request.into_inner().id;
        match self.inner.get_order(&id).await {
            Ok(result) => Ok(Response::new(JsonResponse { json: serde_json::to_string(&result).unwrap_or_default() })),
            Err(e) => Err(Status::not_found(e)),
        }
    }

    async fn get_order_by_number(&self, request: Request<IdRequest>) -> Result<Response<JsonResponse>, Status> {
        let number = request.into_inner().id;
        match self.inner.get_order_by_number(&number).await {
            Ok(result) => Ok(Response::new(JsonResponse { json: serde_json::to_string(&result).unwrap_or_default() })),
            Err(e) => Err(Status::not_found(e)),
        }
    }

    async fn create_order(&self, request: Request<CreateOrderRequest>) -> Result<Response<JsonResponse>, Status> {
        let req = request.into_inner();
        let parsed: newax_erp_orders::CreateOrderRequest = serde_json::from_str(&req.json).map_err(|e| Status::invalid_argument(e.to_string()))?;
        match self.inner.create_order(&parsed).await {
            Ok(result) => Ok(Response::new(JsonResponse { json: serde_json::to_string(&result).unwrap_or_default() })),
            Err(e) => Err(Status::invalid_argument(e)),
        }
    }

    async fn update_order_status(&self, request: Request<UpdateOrderStatusRequest>) -> Result<Response<JsonResponse>, Status> {
        let req = request.into_inner();
        let parsed: newax_erp_orders::UpdateOrderStatusRequest = serde_json::from_str(&req.json).map_err(|e| Status::invalid_argument(e.to_string()))?;
        match self.inner.update_order_status(&req.order_id, &parsed).await {
            Ok(result) => Ok(Response::new(JsonResponse { json: serde_json::to_string(&result).unwrap_or_default() })),
            Err(e) => Err(Status::invalid_argument(e)),
        }
    }

    async fn cancel_order(&self, request: Request<CancelOrderRequest>) -> Result<Response<JsonResponse>, Status> {
        let req = request.into_inner();
        match self.inner.cancel_order(&req.order_id, &req.reason, req.actor_id.as_deref()).await {
            Ok(result) => Ok(Response::new(JsonResponse { json: serde_json::to_string(&result).unwrap_or_default() })),
            Err(e) => Err(Status::invalid_argument(e)),
        }
    }

    async fn get_order_items(&self, request: Request<IdRequest>) -> Result<Response<JsonResponse>, Status> {
        let id = request.into_inner().id;
        match self.inner.get_order_items(&id).await {
            Ok(result) => Ok(Response::new(JsonResponse { json: serde_json::to_string(&result).unwrap_or_default() })),
            Err(e) => Err(Status::not_found(e)),
        }
    }

    async fn get_order_status_events(&self, request: Request<IdRequest>) -> Result<Response<JsonResponse>, Status> {
        let id = request.into_inner().id;
        match self.inner.get_order_status_events(&id).await {
            Ok(result) => Ok(Response::new(JsonResponse { json: serde_json::to_string(&result).unwrap_or_default() })),
            Err(e) => Err(Status::not_found(e)),
        }
    }

    async fn get_order_stats(&self, _request: Request<Empty>) -> Result<Response<JsonResponse>, Status> {
        match self.inner.get_order_stats().await {
            Ok(result) => Ok(Response::new(JsonResponse { json: serde_json::to_string(&result).unwrap_or_default() })),
            Err(e) => Err(Status::internal(e)),
        }
    }

    // Carts
    async fn list_carts(&self, request: Request<ListCartsRequest>) -> Result<Response<JsonResponse>, Status> {
        let req = request.into_inner();
        let user_id = req.user_id.as_deref();
        let status = req.status.as_deref();
        match self.inner.list_carts(user_id, status).await {
            Ok(result) => Ok(Response::new(JsonResponse { json: serde_json::to_string(&result).unwrap_or_default() })),
            Err(e) => Err(Status::internal(e)),
        }
    }

    async fn get_cart(&self, request: Request<IdRequest>) -> Result<Response<JsonResponse>, Status> {
        let id = request.into_inner().id;
        match self.inner.get_cart(&id).await {
            Ok(result) => Ok(Response::new(JsonResponse { json: serde_json::to_string(&result).unwrap_or_default() })),
            Err(e) => Err(Status::not_found(e)),
        }
    }

    async fn create_cart(&self, request: Request<CreateCartRequest>) -> Result<Response<JsonResponse>, Status> {
        let req = request.into_inner();
        let parsed: newax_erp_orders::CreateCartRequest = serde_json::from_str(&req.json).map_err(|e| Status::invalid_argument(e.to_string()))?;
        match self.inner.create_cart(&parsed).await {
            Ok(result) => Ok(Response::new(JsonResponse { json: serde_json::to_string(&result).unwrap_or_default() })),
            Err(e) => Err(Status::invalid_argument(e)),
        }
    }

    async fn delete_cart(&self, request: Request<IdRequest>) -> Result<Response<Empty>, Status> {
        let id = request.into_inner().id;
        match self.inner.delete_cart(&id).await {
            Ok(()) => Ok(Response::new(Empty {})),
            Err(e) => Err(Status::internal(e)),
        }
    }

    async fn get_cart_items(&self, request: Request<IdRequest>) -> Result<Response<JsonResponse>, Status> {
        let id = request.into_inner().id;
        match self.inner.get_cart_items(&id).await {
            Ok(result) => Ok(Response::new(JsonResponse { json: serde_json::to_string(&result).unwrap_or_default() })),
            Err(e) => Err(Status::not_found(e)),
        }
    }

    async fn add_cart_item(&self, request: Request<AddCartItemRequest>) -> Result<Response<JsonResponse>, Status> {
        let req = request.into_inner();
        let parsed: newax_erp_orders::CreateCartItemRequest = serde_json::from_str(&req.json).map_err(|e| Status::invalid_argument(e.to_string()))?;
        match self.inner.add_cart_item(&parsed).await {
            Ok(result) => Ok(Response::new(JsonResponse { json: serde_json::to_string(&result).unwrap_or_default() })),
            Err(e) => Err(Status::invalid_argument(e)),
        }
    }

    async fn update_cart_item(&self, request: Request<UpdateCartItemRequest>) -> Result<Response<JsonResponse>, Status> {
        let req = request.into_inner();
        let parsed: newax_erp_orders::UpdateCartItemRequest = serde_json::from_str(&req.json).map_err(|e| Status::invalid_argument(e.to_string()))?;
        match self.inner.update_cart_item(&req.item_id, parsed.quantity).await {
            Ok(result) => Ok(Response::new(JsonResponse { json: serde_json::to_string(&result).unwrap_or_default() })),
            Err(e) => Err(Status::invalid_argument(e)),
        }
    }

    async fn remove_cart_item(&self, request: Request<IdRequest>) -> Result<Response<Empty>, Status> {
        let id = request.into_inner().id;
        match self.inner.remove_cart_item(&id).await {
            Ok(()) => Ok(Response::new(Empty {})),
            Err(e) => Err(Status::internal(e)),
        }
    }

    async fn checkout_cart(&self, request: Request<CheckoutCartRequest>) -> Result<Response<JsonResponse>, Status> {
        let req = request.into_inner();
        let parsed: newax_erp_orders::CheckoutCartRequest = serde_json::from_str(&req.json).map_err(|e| Status::invalid_argument(e.to_string()))?;
        match self.inner.checkout_cart(&parsed).await {
            Ok(result) => Ok(Response::new(JsonResponse { json: serde_json::to_string(&result).unwrap_or_default() })),
            Err(e) => Err(Status::invalid_argument(e)),
        }
    }

    // Shipments
    async fn list_shipments(&self, request: Request<ListShipmentsRequest>) -> Result<Response<JsonResponse>, Status> {
        let req = request.into_inner();
        let order_id = req.order_id.as_deref();
        let status = req.status.as_deref();
        match self.inner.list_shipments(order_id, status).await {
            Ok(result) => Ok(Response::new(JsonResponse { json: serde_json::to_string(&result).unwrap_or_default() })),
            Err(e) => Err(Status::internal(e)),
        }
    }

    async fn get_shipment(&self, request: Request<IdRequest>) -> Result<Response<JsonResponse>, Status> {
        let id = request.into_inner().id;
        match self.inner.get_shipment(&id).await {
            Ok(result) => Ok(Response::new(JsonResponse { json: serde_json::to_string(&result).unwrap_or_default() })),
            Err(e) => Err(Status::not_found(e)),
        }
    }

    async fn create_shipment(&self, request: Request<CreateShipmentRequest>) -> Result<Response<JsonResponse>, Status> {
        let req = request.into_inner();
        let parsed: newax_erp_orders::CreateShipmentRequest = serde_json::from_str(&req.json).map_err(|e| Status::invalid_argument(e.to_string()))?;
        match self.inner.create_shipment(&parsed).await {
            Ok(result) => Ok(Response::new(JsonResponse { json: serde_json::to_string(&result).unwrap_or_default() })),
            Err(e) => Err(Status::invalid_argument(e)),
        }
    }

    async fn update_shipment_status(&self, request: Request<UpdateShipmentStatusRequest>) -> Result<Response<JsonResponse>, Status> {
        let req = request.into_inner();
        let parsed: newax_erp_orders::UpdateShipmentStatusRequest = serde_json::from_str(&req.json).map_err(|e| Status::invalid_argument(e.to_string()))?;
        match self.inner.update_shipment_status(&req.shipment_id, &parsed).await {
            Ok(result) => Ok(Response::new(JsonResponse { json: serde_json::to_string(&result).unwrap_or_default() })),
            Err(e) => Err(Status::invalid_argument(e)),
        }
    }

    async fn get_shipment_events(&self, request: Request<IdRequest>) -> Result<Response<JsonResponse>, Status> {
        let id = request.into_inner().id;
        match self.inner.get_shipment_events(&id).await {
            Ok(result) => Ok(Response::new(JsonResponse { json: serde_json::to_string(&result).unwrap_or_default() })),
            Err(e) => Err(Status::not_found(e)),
        }
    }

    async fn create_shipment_event(&self, request: Request<CreateShipmentEventRequest>) -> Result<Response<JsonResponse>, Status> {
        let req = request.into_inner();
        let parsed: newax_erp_orders::CreateShipmentEventRequest = serde_json::from_str(&req.json).map_err(|e| Status::invalid_argument(e.to_string()))?;
        match self.inner.create_shipment_event(&parsed).await {
            Ok(result) => Ok(Response::new(JsonResponse { json: serde_json::to_string(&result).unwrap_or_default() })),
            Err(e) => Err(Status::invalid_argument(e)),
        }
    }

    // Back in Stock
    async fn list_back_in_stock_requests(&self, request: Request<ListBackInStockRequestsRequest>) -> Result<Response<JsonResponse>, Status> {
        let req = request.into_inner();
        let product_id = req.product_id.as_deref();
        let status = req.status.as_deref();
        match self.inner.list_back_in_stock_requests(product_id, status).await {
            Ok(result) => Ok(Response::new(JsonResponse { json: serde_json::to_string(&result).unwrap_or_default() })),
            Err(e) => Err(Status::internal(e)),
        }
    }

    async fn create_back_in_stock_request(&self, request: Request<CreateBackInStockRequest>) -> Result<Response<JsonResponse>, Status> {
        let req = request.into_inner();
        let parsed: newax_erp_orders::CreateBackInStockRequest = serde_json::from_str(&req.json).map_err(|e| Status::invalid_argument(e.to_string()))?;
        match self.inner.create_back_in_stock_request(&parsed).await {
            Ok(result) => Ok(Response::new(JsonResponse { json: serde_json::to_string(&result).unwrap_or_default() })),
            Err(e) => Err(Status::invalid_argument(e)),
        }
    }

    async fn notify_back_in_stock(&self, request: Request<IdRequest>) -> Result<Response<JsonResponse>, Status> {
        let product_id = request.into_inner().id;
        match self.inner.notify_back_in_stock(&product_id).await {
            Ok(count) => Ok(Response::new(JsonResponse { json: serde_json::to_string(&serde_json::json!({"notified": count})).unwrap_or_default() })),
            Err(e) => Err(Status::internal(e)),
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let pool = sqlx::PgPool::connect(std::env::var("DATABASE_URL").unwrap_or_default().as_str())
        .await
        .expect("Failed to create pool");

    let addr = "0.0.0.0:50099".parse().expect("Invalid address");
    let service = ErpOrdersServiceImpl::new(pool);

    tonic::transport::Server::builder()
        .add_service(ErpOrdersServiceServer::new(service))
        .serve(addr)
        .await?;

    Ok(())
}
