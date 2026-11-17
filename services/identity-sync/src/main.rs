//! Standalone worker: no gRPC surface, just a NATS JetStream consumer that
//! keeps `synced_tenant`/`synced_organization`/`synced_brand`/
//! `synced_customer` current. See `newax-identity-sync`'s doc comment for
//! the ownership/idempotency rules this implements.

use sqlx::postgres::PgPoolOptions;
use std::time::Duration;
use tracing::{error, info};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::try_new("info,svc_identity_sync=debug").unwrap())
        .init();

    let database_url = std::env::var("DATABASE_URL")
        .map_err(|_| "DATABASE_URL must be set (this repo's own Communicator Postgres instance)")?;
    let db = PgPoolOptions::new().max_connections(5).connect(&database_url).await?;

    info!("svc-identity-sync starting (NATS JetStream consumer, no gRPC)");

    // A dropped NATS connection ends `run()`; reconnect with backoff rather
    // than exiting, since this is a background worker with nothing else
    // depending on its process lifetime.
    loop {
        if let Err(e) = newax_identity_sync::run(db.clone()).await {
            error!(error = %e, "svc-identity-sync: run() exited, reconnecting in 5s");
        }
        tokio::time::sleep(Duration::from_secs(5)).await;
    }
}
