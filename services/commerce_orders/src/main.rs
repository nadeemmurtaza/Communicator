use newax_commerce_orders as commerce_orders;
use sqlx::postgres::PgPoolOptions;
use tonic::{transport::Server, Request, Response, Status};
use tracing::info;

pub mod proto {
    tonic::include_proto!("newax.commerce_orders.v1");
}

use proto::{
    commerce_orders_service_server::{CommerceOrdersService, CommerceOrdersServiceServer},
    JsonResponse,
};

const PORT: u16 = 50111;

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
impl CommerceOrdersService for Svc {
    async fn checkout(
        &self,
        req: Request<proto::CheckoutRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let body: newax_commerce_orders::CheckoutRequest =
            serde_json::from_str(&r.json).map_err(to_status)?;
        let service = commerce_orders::CommerceOrdersService::new(self.pool.clone());
        let result = service.checkout(&r.cart_id, &body).await.map_err(to_status)?;
        Ok(Response::new(json_response(serde_json::to_value(result).map_err(to_status)?)))
    }

    async fn list_orders(
        &self,
        req: Request<proto::ListOrdersRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let service = commerce_orders::CommerceOrdersService::new(self.pool.clone());
        let cid = if r.customer_id.is_empty() { None } else { Some(r.customer_id.as_str()) };
        let st = if r.status.is_empty() { None } else { Some(r.status.as_str()) };
        let ch = if r.channel_type.is_empty() { None } else { Some(r.channel_type.as_str()) };
        let result = service.list_orders(&r.brand_id, cid, st, ch).await.map_err(to_status)?;
        Ok(Response::new(json_response(serde_json::to_value(result).map_err(to_status)?)))
    }

    async fn get_order(
        &self,
        req: Request<proto::GetOrderRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let service = commerce_orders::CommerceOrdersService::new(self.pool.clone());
        let result = service.get_order_with_items(&r.id).await.map_err(to_status)?;
        Ok(Response::new(json_response(serde_json::to_value(result).map_err(to_status)?)))
    }

    async fn update_order_status(
        &self,
        req: Request<proto::UpdateOrderStatusRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let body = newax_commerce_orders::UpdateOrderStatusRequest { status: r.status };
        let service = commerce_orders::CommerceOrdersService::new(self.pool.clone());
        let result = service.update_order_status(&r.id, &body).await.map_err(to_status)?;
        Ok(Response::new(json_response(serde_json::to_value(result).map_err(to_status)?)))
    }

    async fn get_order_stats(
        &self,
        req: Request<proto::GetOrderStatsRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let service = commerce_orders::CommerceOrdersService::new(self.pool.clone());
        let result = service.get_order_stats(&r.brand_id).await.map_err(to_status)?;
        Ok(Response::new(json_response(result)))
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

    let port: u16 = std::env::var("COMMERCE_ORDERS_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(PORT);
    let addr = format!("0.0.0.0:{port}").parse()?;

    info!(port, "svc-commerce-orders starting (gRPC)");

    Server::builder()
        .add_service(CommerceOrdersServiceServer::new(Svc { pool }))
        .serve(addr)
        .await?;

    Ok(())
}
