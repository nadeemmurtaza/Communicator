use newax_notifications as notifications;
use sqlx::postgres::PgPoolOptions;
use tonic::{transport::Server, Request, Response, Status};
use tracing::info;

pub mod proto {
    tonic::include_proto!("newax.notifications.v1");
}

use proto::{
    notifications_service_server::{NotificationsService, NotificationsServiceServer},
    Empty, IdRequest, JsonResponse, ListNotificationsRequest, QueueNotificationRequest,
    BroadcastNotificationRequest, UpdatePreferencesRequest, UpdateSettingsRequest,
};

struct Svc {
    state: notifications::NotificationsState,
}

fn to_status(e: impl std::fmt::Display) -> Status {
    Status::internal(e.to_string())
}

fn serialize_response<T: serde::Serialize>(val: T) -> Result<JsonResponse, Status> {
    let v = serde_json::to_value(val).map_err(|e| Status::internal(e.to_string()))?;
    Ok(JsonResponse { json: v.to_string() })
}

fn parse_json(s: &str) -> Result<serde_json::Value, Status> {
    serde_json::from_str(s).map_err(|e| Status::invalid_argument(e.to_string()))
}

#[tonic::async_trait]
impl NotificationsService for Svc {
    async fn get_summary(&self, _req: Request<Empty>) -> Result<Response<JsonResponse>, Status> {
        let summary = notifications::get_notification_summary(&self.state)
            .await
            .map_err(to_status)?;
        Ok(Response::new(serialize_response(summary)?))
    }

    async fn list_notifications(
        &self,
        req: Request<ListNotificationsRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let items = notifications::list_notifications(
            &self.state,
            r.user_id.as_deref(),
            r.notification_type.as_deref(),
            r.status.as_deref(),
            r.limit,
        )
        .await
        .map_err(to_status)?;
        Ok(Response::new(serialize_response(items)?))
    }

    async fn queue_notification(
        &self,
        req: Request<QueueNotificationRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let result = notifications::queue_notification(
            &self.state,
            &r.organization_id,
            r.user_id.as_deref(),
            &r.notification_type,
            &r.title,
            &r.body,
            &r.data,
            &r.channel,
        )
        .await
        .map_err(to_status)?;
        Ok(Response::new(serialize_response(result)?))
    }

    async fn broadcast_notification(
        &self,
        req: Request<BroadcastNotificationRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let result = notifications::broadcast_notification(
            &self.state,
            &r.organization_id,
            &r.notification_type,
            &r.title,
            &r.body,
        )
        .await
        .map_err(to_status)?;
        Ok(Response::new(serialize_response(result)?))
    }

    async fn mark_read(&self, req: Request<IdRequest>) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let result = notifications::mark_notification_read(&self.state, &r.id)
            .await
            .map_err(to_status)?;
        Ok(Response::new(serialize_response(result)?))
    }

    async fn dismiss(&self, req: Request<IdRequest>) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let result = notifications::dismiss_notification(&self.state, &r.id)
            .await
            .map_err(to_status)?;
        Ok(Response::new(serialize_response(result)?))
    }

    async fn get_preferences(&self, req: Request<IdRequest>) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let prefs = notifications::get_preferences(&self.state, &r.id)
            .await
            .map_err(to_status)?;
        Ok(Response::new(serialize_response(prefs)?))
    }

    async fn update_preferences(
        &self,
        req: Request<UpdatePreferencesRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let prefs: notifications::NotificationPreferences =
            serde_json::from_str(&r.prefs_json).map_err(|e| Status::invalid_argument(e.to_string()))?;
        let result = notifications::update_preferences(&self.state, &r.user_id, &prefs)
            .await
            .map_err(to_status)?;
        Ok(Response::new(serialize_response(result)?))
    }

    async fn get_settings(&self, _req: Request<Empty>) -> Result<Response<JsonResponse>, Status> {
        let settings = notifications::get_settings(&self.state)
            .await
            .map_err(to_status)?;
        Ok(Response::new(serialize_response(settings)?))
    }

    async fn update_settings(
        &self,
        req: Request<UpdateSettingsRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let r = req.into_inner();
        let settings: notifications::NotificationSettings =
            serde_json::from_str(&r.settings_json).map_err(|e| Status::invalid_argument(e.to_string()))?;
        let result = notifications::update_settings(&self.state, &settings)
            .await
            .map_err(to_status)?;
        Ok(Response::new(serialize_response(result)?))
    }

    async fn list_connections(&self, _req: Request<Empty>) -> Result<Response<JsonResponse>, Status> {
        let connections = notifications::list_websocket_connections(&self.state)
            .await
            .map_err(to_status)?;
        Ok(Response::new(serialize_response(connections)?))
    }

    async fn cleanup(&self, _req: Request<Empty>) -> Result<Response<JsonResponse>, Status> {
        let count = notifications::cleanup_old_notifications(&self.state)
            .await
            .map_err(to_status)?;
        Ok(Response::new(serialize_response(serde_json::json!({
            "deleted_count": count,
            "message": format!("Cleaned up {} old notifications", count)
        }))?))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "svc-notifications=debug,tonic=info".into()),
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

    let state = notifications::NotificationsState { db: pool };

    let addr = "[::1]:50082".parse().expect("Invalid address");
    let svc = Svc { state };

    info!("Notifications service listening on {}", addr);

    Server::builder()
        .add_service(NotificationsServiceServer::new(svc))
        .serve(addr)
        .await?;

    Ok(())
}
