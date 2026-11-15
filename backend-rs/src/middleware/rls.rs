//! RLS (Row-Level Security) middleware.
//!
//! Before every database query, this middleware sets PostgreSQL session-level
//! GUC variables so that RLS policies can isolate data by tenant:
//!
//! - `app.current_tenant_key` — extracted from `X-Tenant-Key` header or
//!   `DEFAULT_TENANT_KEY` config.
//! - `app.current_account_id` — extracted from the nID session token (Bearer).
//! - `app.current_user_id` — team_member UUID if resolved from account.
//!
//! The migration `0072_row_level_security.sql` creates the corresponding RLS
//! policies that reference these settings via `current_setting()`.

use axum::body::Body;
use axum::http::Request;
use axum::http::Response;
use axum::middleware::Next;
use axum::response::IntoResponse;
use sqlx::PgConnection;
use sqlx::Connection;
use tracing::debug;

use crate::db::AppState;

/// Extract tenant key from request headers.
fn extract_tenant_key(req: &Request<Body>, config: &crate::config::Config) -> String {
    req.headers()
        .get("x-tenant-key")
        .or_else(|| req.headers().get("X-Tenant-Key"))
        .and_then(|v| v.to_str().ok())
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(str::to_string)
        .or_else(|| {
            (!config.default_tenant_key.is_empty())
                .then(|| config.default_tenant_key.clone())
        })
        .unwrap_or_default()
}

/// Extract account ID from Bearer token via nID session validation.
async fn extract_account_id(state: &AppState, req: &Request<Body>) -> String {
    let auth_header = req.headers().get("authorization").and_then(|v| v.to_str().ok());
    
    if let Some(auth) = auth_header {
        let token = if auth.starts_with("Bearer ") {
            &auth[7..]
        } else if auth.starts_with("bearer ") {
            &auth[7..]
        } else {
            auth
        };
        
        if !token.is_empty() {
            if let Some(claims) = state.nid.validate_session(token).await.ok().flatten() {
                return claims.account_id;
            }
        }
    }
    
    String::new()
}

/// Tower middleware handler that sets RLS context before each request.
pub async fn rls_middleware(
    state: axum::extract::State<AppState>,
    mut req: Request<Body>,
    next: Next,
) -> impl IntoResponse {
    let tenant_key = extract_tenant_key(&req, &state.config);
    let account_id = extract_account_id(&state, &req).await;
    
    // Store the RLS context in request extensions so handlers can access it
    // and so the connection middleware can set it on the DB connection.
    req.extensions_mut().insert(RlsContext {
        tenant_key: tenant_key.clone(),
        account_id: account_id.clone(),
    });
    
    debug!(
        tenant_key = %tenant_key,
        account_id = %account_id,
        "RLS context set for request"
    );
    
    next.run(req).await
}

/// RLS context stored in request extensions.
#[derive(Clone, Debug)]
pub struct RlsContext {
    pub tenant_key: String,
    pub account_id: String,
}

/// Set the RLS context on a PostgreSQL connection.
/// This should be called at the beginning of each database transaction.
pub async fn set_rls_context(
    conn: &mut PgConnection,
    context: &RlsContext,
) -> Result<(), sqlx::Error> {
    if !context.tenant_key.is_empty() {
        sqlx::query(&format!(
            "SET LOCAL app.current_tenant_key TO '{}'",
            context.tenant_key.replace('\'', "''")
        ))
        .execute(&mut *conn)
        .await?;
    }
    
    if !context.account_id.is_empty() {
        sqlx::query(&format!(
            "SET LOCAL app.current_account_id TO '{}'",
            context.account_id.replace('\'', "''")
        ))
        .execute(&mut *conn)
        .await?;
    }
    
    Ok(())
}

/// Set the RLS context on a database pool connection using a transaction block.
/// This wraps the query execution with the RLS context.
pub async fn with_rls_context<T>(
    pool: &sqlx::PgPool,
    context: &RlsContext,
    f: impl FnOnce(&mut sqlx::Transaction<'_, sqlx::Postgres>) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<T, sqlx::Error>> + Send>>,
) -> Result<T, sqlx::Error> {
    let mut tx = pool.begin().await?;
    
    // Set the GUC variables for this transaction
    if !context.tenant_key.is_empty() {
        sqlx::query(&format!(
            "SET LOCAL app.current_tenant_key TO '{}'",
            context.tenant_key.replace('\'', "''")
        ))
        .execute(&mut *tx)
        .await?;
    }
    
    if !context.account_id.is_empty() {
        sqlx::query(&format!(
            "SET LOCAL app.current_account_id TO '{}'",
            context.account_id.replace('\'', "''")
        ))
        .execute(&mut *tx)
        .await?;
    }
    
    let result = f(&mut tx).await?;
    tx.commit().await?;
    Ok(result)
}
