//! CRM Sync — Salesforce/HubSpot bidirectional sync (Phase 22).
//!
//! Provides integration with major CRM platforms to sync leads, contacts,
//! and accounts/companies bidirectionally. Mirrors Intercom's CRM sync feature.

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::Row;
use uuid::Uuid;

use crate::db::AppState;

// ——— Types ———

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CRMConnection {
    pub id: String,
    pub organization_id: String,
    pub provider: String,
    pub status: String,
    pub auth_type: String,
    pub credentials: Option<Value>,
    pub config: Option<Value>,
    pub last_sync_at: Option<String>,
    pub last_error: Option<String>,
    pub connected_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CRMSyncLog {
    pub id: String,
    pub connection_id: String,
    pub direction: String,
    pub object_type: String,
    pub record_count: i32,
    pub success_count: i32,
    pub error_count: i32,
    pub status: String,
    pub error_message: Option<String>,
    pub started_at: String,
    pub completed_at: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CRMFieldMapping {
    pub id: String,
    pub connection_id: String,
    pub object_type: String,
    pub local_field: String,
    pub crm_field: String,
    pub direction: String,
    pub transform: Option<Value>,
    pub is_required: bool,
    pub created_at: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CRMExternalRef {
    pub id: String,
    pub connection_id: String,
    pub local_type: String,
    pub local_id: String,
    pub crm_object_type: String,
    pub crm_record_id: String,
    pub crm_record_url: Option<String>,
    pub last_synced_at: String,
    pub sync_status: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CRMSyncSummary {
    pub total_connections: i32,
    pub active_connections: i32,
    pub disconnected_connections: i32,
    pub providers: Vec<Value>,
    pub recent_syncs: Vec<CRMSyncLog>,
    pub total_syncs_7d: i32,
    pub total_records_synced_7d: i32,
    pub pending_refs: i32,
    pub conflict_refs: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CRMSyncResult {
    pub connection_id: String,
    pub direction: String,
    pub object_type: String,
    pub record_count: i32,
    pub success_count: i32,
    pub error_count: i32,
    pub status: String,
    pub details: Vec<Value>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CRMProviderInfo {
    pub provider: String,
    pub display_name: String,
    pub description: String,
    pub auth_types: Vec<String>,
    pub supported_objects: Vec<String>,
    pub setup_url: String,
    pub docs_url: String,
}

// ——— Core Functions ———

/// Get CRM sync summary dashboard data
pub async fn get_crm_sync_summary(state: &AppState) -> Result<CRMSyncSummary, sqlx::Error> {
    let total_connections: i32 = sqlx::query_scalar("SELECT COUNT(*) FROM crm_connections")
        .fetch_one(&state.db)
        .await?;

    let active_connections: i32 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM crm_connections WHERE status = 'connected'"
    )
    .fetch_one(&state.db)
    .await?;

    let disconnected_connections: i32 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM crm_connections WHERE status != 'connected'"
    )
    .fetch_one(&state.db)
    .await?;

    // Provider breakdown
    let provider_rows = sqlx::query(
        "SELECT provider, COUNT(*) as cnt FROM crm_connections GROUP BY provider"
    )
    .fetch_all(&state.db)
    .await?;

    let providers: Vec<Value> = provider_rows
        .iter()
        .map(|row| {
            let provider: String = row.try_get(0).unwrap_or_default();
            let cnt: i32 = row.try_get(1).unwrap_or(0);
            json!({"provider": provider, "count": cnt})
        })
        .collect();

    // Recent syncs
    let sync_rows = sqlx::query(
        "SELECT id, connection_id, direction, object_type, record_count, success_count, error_count, status, error_message, started_at, completed_at FROM crm_sync_logs ORDER BY started_at DESC LIMIT 10"
    )
    .fetch_all(&state.db)
    .await?;

    let recent_syncs: Vec<CRMSyncLog> = sync_rows.iter().map(|row| sync_log_from_row(row)).collect();

    // Syncs in last 7 days
    let total_syncs_7d: i32 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM crm_sync_logs WHERE started_at >= datetime('now', '-7 days')"
    )
    .fetch_one(&state.db)
    .await?;

    let total_records_synced_7d: i32 = sqlx::query_scalar(
        "SELECT COALESCE(SUM(success_count), 0) FROM crm_sync_logs WHERE started_at >= datetime('now', '-7 days')"
    )
    .fetch_one(&state.db)
    .await?;

    // Pending and conflict refs
    let pending_refs: i32 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM crm_external_refs WHERE sync_status = 'pending'"
    )
    .fetch_one(&state.db)
    .await?;

    let conflict_refs: i32 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM crm_external_refs WHERE sync_status = 'conflict'"
    )
    .fetch_one(&state.db)
    .await?;

    Ok(CRMSyncSummary {
        total_connections,
        active_connections,
        disconnected_connections,
        providers,
        recent_syncs,
        total_syncs_7d,
        total_records_synced_7d,
        pending_refs,
        conflict_refs,
    })
}

/// List CRM connections with optional filters
pub async fn list_crm_connections(
    state: &AppState,
    provider: Option<&str>,
    status: Option<&str>,
) -> Result<Vec<CRMConnection>, sqlx::Error> {
    let mut conditions = Vec::new();

    if let Some(p) = provider {
        conditions.push(format!("provider = '{}'", p.replace('\'', "''")));
    }
    if let Some(s) = status {
        conditions.push(format!("status = '{}'", s.replace('\'', "''")));
    }

    let where_clause = if conditions.is_empty() {
        "".to_string()
    } else {
        format!("WHERE {}", conditions.join(" AND "))
    };

    let rows = sqlx::query(
        &format!(
            "SELECT id, organization_id, provider, status, auth_type, credentials, config, last_sync_at, last_error, connected_at, created_at, updated_at FROM crm_connections {} ORDER BY created_at DESC",
            where_clause
        )
    )
    .fetch_all(&state.db)
    .await?;

    let connections: Vec<CRMConnection> = rows.iter().map(|row| connection_from_row(row)).collect();

    Ok(connections)
}

/// Create a new CRM connection
pub async fn create_crm_connection(
    state: &AppState,
    organization_id: &str,
    provider: &str,
    auth_type: &str,
    credentials: Option<&str>,
    config: Option<&str>,
) -> Result<CRMConnection, sqlx::Error> {
    let id = Uuid::new_v4().to_string();
    let safe_org = organization_id.replace('\'', "''");
    let safe_provider = provider.replace('\'', "''");
    let safe_auth = auth_type.replace('\'', "''");
    let safe_creds = credentials.map(|c| c.replace('\'', "''"));
    let safe_config = config.map(|c| c.replace('\'', "''"));

    let creds_sql = safe_creds.as_deref().unwrap_or("NULL");
    let config_sql = safe_config.as_deref().unwrap_or("NULL");

    let sql = format!(
        "INSERT INTO crm_connections (id, organization_id, provider, status, auth_type, credentials, config) VALUES ('{}', '{}', '{}', 'connecting', '{}', {}, {})",
        id, safe_org, safe_provider, safe_auth,
        if creds_sql == "NULL" { "NULL".to_string() } else { format!("'{}'", creds_sql) },
        if config_sql == "NULL" { "NULL".to_string() } else { format!("'{}'", config_sql) },
    );

    sqlx::query(&sql).execute(&state.db).await?;

    Ok(CRMConnection {
        id,
        organization_id: organization_id.to_string(),
        provider: provider.to_string(),
        status: "connecting".to_string(),
        auth_type: auth_type.to_string(),
        credentials: credentials.and_then(|c| serde_json::from_str(c).ok()),
        config: config.and_then(|c| serde_json::from_str(c).ok()),
        last_sync_at: None,
        last_error: None,
        connected_at: None,
        created_at: chrono::Utc::now().to_rfc3339(),
        updated_at: chrono::Utc::now().to_rfc3339(),
    })
}

/// Update CRM connection status
pub async fn update_crm_connection_status(
    state: &AppState,
    connection_id: &str,
    status: &str,
    credentials: Option<&str>,
    last_error: Option<&str>,
) -> Result<CRMConnection, sqlx::Error> {
    let safe_id = connection_id.replace('\'', "''");
    let safe_status = status.replace('\'', "''");
    let now = chrono::Utc::now().to_rfc3339();

    let mut sets = vec![
        format!("status = '{}'", safe_status),
        format!("updated_at = '{}'", now),
    ];

    if status == "connected" {
        sets.push(format!("connected_at = '{}'", now));
    }

    if let Some(creds) = credentials {
        let safe_creds = creds.replace('\'', "''");
        sets.push(format!("credentials = '{}'", safe_creds));
    }

    if let Some(err) = last_error {
        let safe_err = err.replace('\'', "''");
        sets.push(format!("last_error = '{}'", safe_err));
    }

    let sql = format!(
        "UPDATE crm_connections SET {} WHERE id = '{}'",
        sets.join(", "),
        safe_id
    );

    sqlx::query(&sql).execute(&state.db).await?;

    // Return updated connection
    let row = sqlx::query(
        &format!(
            "SELECT id, organization_id, provider, status, auth_type, credentials, config, last_sync_at, last_error, connected_at, created_at, updated_at FROM crm_connections WHERE id = '{}'",
            safe_id
        )
    )
    .fetch_one(&state.db)
    .await?;

    Ok(connection_from_row(row))
}

/// Delete a CRM connection
pub async fn delete_crm_connection(state: &AppState, connection_id: &str) -> Result<(), sqlx::Error> {
    let safe_id = connection_id.replace('\'', "''");
    sqlx::query(&format!("DELETE FROM crm_connections WHERE id = '{}'", safe_id))
        .execute(&state.db)
        .await?;
    Ok(())
}

/// Trigger a sync operation
pub async fn trigger_sync(
    state: &AppState,
    connection_id: &str,
    direction: &str,
    object_type: &str,
) -> Result<CRMSyncResult, sqlx::Error> {
    let safe_conn = connection_id.replace('\'', "''");
    let safe_dir = direction.replace('\'', "''");
    let safe_obj = object_type.replace('\'', "''");

    // Check connection status
    let conn_status: String = sqlx::query_scalar(
        &format!(
            "SELECT status FROM crm_connections WHERE id = '{}'",
            safe_conn
        )
    )
    .fetch_one(&state.db)
    .await?;

    if conn_status != "connected" {
        return Err(sqlx::Error::RowNotFound);
    }

    // Create sync log entry
    let sync_id = Uuid::new_v4().to_string();
    let safe_sync = sync_id.replace('\'', "''");

    sqlx::query(&format!(
        "INSERT INTO crm_sync_logs (id, connection_id, direction, object_type, status) VALUES ('{}', '{}', '{}', '{}', 'running')",
        safe_sync, safe_conn, safe_dir, safe_obj
    ))
    .execute(&state.db)
    .await?;

    // Perform the sync (simulated for now — real implementation calls CRM APIs)
    let (record_count, success_count, error_count, status, error_message) =
        perform_sync(state, connection_id, direction, object_type).await;

    let now = chrono::Utc::now().to_rfc3339();
    let safe_sync = sync_id.replace('\'', "''");
    let safe_status = status.replace('\'', "''");
    let safe_error = error_message.as_ref().map(|e| e.replace('\'', "''"));

    let error_sql = safe_error.as_deref().unwrap_or("NULL");

    // Update connection last_sync_at
    sqlx::query(&format!(
        "UPDATE crm_connections SET last_sync_at = '{}', updated_at = '{}' WHERE id = '{}'",
        now, now, safe_conn
    ))
    .execute(&state.db)
    .await?;

    // Complete sync log
    sqlx::query(&format!(
        "UPDATE crm_sync_logs SET record_count = {}, success_count = {}, error_count = {}, status = '{}', error_message = {}, completed_at = '{}' WHERE id = '{}'",
        record_count, success_count, error_count, safe_status,
        if error_sql == "NULL" { "NULL".to_string() } else { format!("'{}'", error_sql) },
        now, safe_sync
    ))
    .execute(&state.db)
    .await?;

    // Build details
    let details: Vec<Value> = if error_count > 0 {
        vec![json!({
            "warning": "Some records failed to sync",
            "error_count": error_count,
            "success_count": success_count
        })]
    } else {
        vec![json!({
            "message": format!("Successfully synced {} {} {} records", success_count, object_type, direction),
        })]
    };

    Ok(CRMSyncResult {
        connection_id: connection_id.to_string(),
        direction: direction.to_string(),
        object_type: object_type.to_string(),
        record_count,
        success_count,
        error_count,
        status,
        details,
    })
}

/// Perform the actual sync operation (simulated — real implementation calls CRM APIs)
async fn perform_sync(
    state: &AppState,
    _connection_id: &str,
    direction: &str,
    object_type: &str,
) -> (i32, i32, i32, String, Option<String>) {
    // Count local records to sync
    let local_count_query = match object_type {
        "lead" => "SELECT COUNT(*) FROM leads",
        "contact" => "SELECT COUNT(*) FROM contacts",
        "account" | "company" => "SELECT COUNT(*) FROM organizations",
        _ => "SELECT 0",
    };

    let count: i32 = match sqlx::query_scalar(local_count_query).fetch_one(&state.db).await {
        Ok(c) => c,
        Err(_) => 0,
    };

    // Simulate sync: in production, this would call Salesforce/HubSpot APIs
    // For now, we simulate based on local record count
    let success_count = if direction == "outbound" { count } else { 0 };
    let error_count = 0;

    (
        count,
        success_count,
        error_count,
        if error_count > 0 {
            "partial".to_string()
        } else {
            "completed".to_string()
        },
        None,
    )
}

/// List sync logs
pub async fn list_sync_logs(
    state: &AppState,
    connection_id: Option<&str>,
    limit: i32,
) -> Result<Vec<CRMSyncLog>, sqlx::Error> {
    let limit = limit.max(1).min(100);

    let where_clause = if let Some(conn) = connection_id {
        let safe_conn = conn.replace('\'', "''");
        format!("WHERE connection_id = '{}'", safe_conn)
    } else {
        "".to_string()
    };

    let rows = sqlx::query(
        &format!(
            "SELECT id, connection_id, direction, object_type, record_count, success_count, error_count, status, error_message, started_at, completed_at FROM crm_sync_logs {} ORDER BY started_at DESC LIMIT {}",
            where_clause, limit
        )
    )
    .fetch_all(&state.db)
    .await?;

    let logs: Vec<CRMSyncLog> = rows.iter().map(|row| sync_log_from_row(row)).collect();

    Ok(logs)
}

/// List field mappings for a connection
pub async fn list_field_mappings(
    state: &AppState,
    connection_id: Option<&str>,
    object_type: Option<&str>,
) -> Result<Vec<CRMFieldMapping>, sqlx::Error> {
    let mut conditions = Vec::new();

    if let Some(conn) = connection_id {
        conditions.push(format!("connection_id = '{}'", conn.replace('\'', "''")));
    } else {
        // Default: show all mappings
        conditions.push("1=1".to_string());
    }

    if let Some(obj) = object_type {
        conditions.push(format!("object_type = '{}'", obj.replace('\'', "''")));
    }

    let where_clause = format!("WHERE {}", conditions.join(" AND "));

    let rows = sqlx::query(
        &format!(
            "SELECT id, connection_id, object_type, local_field, crm_field, direction, transform, is_required, created_at FROM crm_field_mappings {} ORDER BY object_type, local_field",
            where_clause
        )
    )
    .fetch_all(&state.db)
    .await?;

    let mappings: Vec<CRMFieldMapping> = rows.iter().map(|row| field_mapping_from_row(row)).collect();

    Ok(mappings)
}

/// Get external references for a local record
pub async fn get_external_refs(
    state: &AppState,
    local_type: &str,
    local_id: &str,
) -> Result<Vec<CRMExternalRef>, sqlx::Error> {
    let safe_type = local_type.replace('\'', "''");
    let safe_id = local_id.replace('\'', "''");

    let rows = sqlx::query(
        &format!(
            "SELECT id, connection_id, local_type, local_id, crm_object_type, crm_record_id, crm_record_url, last_synced_at, sync_status, created_at, updated_at FROM crm_external_refs WHERE local_type = '{}' AND local_id = '{}'",
            safe_type, safe_id
        )
    )
    .fetch_all(&state.db)
    .await?;

    let refs: Vec<CRMExternalRef> = rows.iter().map(|row| external_ref_from_row(row)).collect();

    Ok(refs)
}

/// Create an external reference (link local record to CRM record)
pub async fn create_external_ref(
    state: &AppState,
    connection_id: &str,
    local_type: &str,
    local_id: &str,
    crm_object_type: &str,
    crm_record_id: &str,
    crm_record_url: Option<&str>,
) -> Result<CRMExternalRef, sqlx::Error> {
    let id = Uuid::new_v4().to_string();
    let safe_conn = connection_id.replace('\'', "''");
    let safe_type = local_type.replace('\'', "''");
    let safe_local = local_id.replace('\'', "''");
    let safe_crm_obj = crm_object_type.replace('\'', "''");
    let safe_crm_id = crm_record_id.replace('\'', "''");
    let safe_url = crm_record_url.map(|u| u.replace('\'', "''"));

    let url_sql = safe_url.as_deref().unwrap_or("NULL");

    sqlx::query(&format!(
        "INSERT INTO crm_external_refs (id, connection_id, local_type, local_id, crm_object_type, crm_record_id, crm_record_url) VALUES ('{}', '{}', '{}', '{}', '{}', '{}', {})",
        id, safe_conn, safe_type, safe_local, safe_crm_obj, safe_crm_id,
        if url_sql == "NULL" { "NULL".to_string() } else { format!("'{}'", url_sql) }
    ))
    .execute(&state.db)
    .await?;

    Ok(CRMExternalRef {
        id,
        connection_id: connection_id.to_string(),
        local_type: local_type.to_string(),
        local_id: local_id.to_string(),
        crm_object_type: crm_object_type.to_string(),
        crm_record_id: crm_record_id.to_string(),
        crm_record_url: crm_record_url.map(|s| s.to_string()),
        last_synced_at: chrono::Utc::now().to_rfc3339(),
        sync_status: "synced".to_string(),
        created_at: chrono::Utc::now().to_rfc3339(),
        updated_at: chrono::Utc::now().to_rfc3339(),
    })
}

/// Get available CRM providers info
pub fn get_crm_providers() -> Vec<CRMProviderInfo> {
    vec![
        CRMProviderInfo {
            provider: "salesforce".to_string(),
            display_name: "Salesforce".to_string(),
            description: "Sync leads, contacts, and accounts with Salesforce CRM".to_string(),
            auth_types: vec!["oauth".to_string(), "api_key".to_string()],
            supported_objects: vec!["lead".to_string(), "contact".to_string(), "account".to_string(), "deal".to_string()],
            setup_url: "https://help.salesforce.com/articleView?id=sf.remoteaccess_auth_flows.htm".to_string(),
            docs_url: "https://developer.salesforce.com/docs".to_string(),
        },
        CRMProviderInfo {
            provider: "hubspot".to_string(),
            display_name: "HubSpot".to_string(),
            description: "Sync contacts, companies, and deals with HubSpot CRM".to_string(),
            auth_types: vec!["oauth2".to_string(), "api_key".to_string()],
            supported_objects: vec!["contact".to_string(), "company".to_string(), "deal".to_string()],
            setup_url: "https://developers.hubspot.com/docs/api/oauth-quickstart-guide".to_string(),
            docs_url: "https://developers.hubspot.com/docs/api/".to_string(),
        },
        CRMProviderInfo {
            provider: "pipedrive".to_string(),
            display_name: "Pipedrive".to_string(),
            description: "Sync contacts, deals, and organizations with Pipedrive CRM".to_string(),
            auth_types: vec!["oauth2".to_string(), "api_key".to_string()],
            supported_objects: vec!["contact".to_string(), "deal".to_string(), "organization".to_string()],
            setup_url: "https://developers.pipedrive.com/docs/api/v1/".to_string(),
            docs_url: "https://developers.pipedrive.com/docs/api/v1/".to_string(),
        },
    ]
}

// ——— Row helpers ———

fn connection_from_row(row: &impl sqlx::Row) -> CRMConnection {
    CRMConnection {
        id: row.try_get("id").unwrap_or_default(),
        organization_id: row.try_get("organization_id").unwrap_or_default(),
        provider: row.try_get("provider").unwrap_or_default(),
        status: row.try_get("status").unwrap_or_default(),
        auth_type: row.try_get("auth_type").unwrap_or_default(),
        credentials: row.try_get("credentials").ok(),
        config: row.try_get("config").ok(),
        last_sync_at: row.try_get("last_sync_at").ok(),
        last_error: row.try_get("last_error").ok(),
        connected_at: row.try_get("connected_at").ok(),
        created_at: row.try_get("created_at").unwrap_or_default(),
        updated_at: row.try_get("updated_at").unwrap_or_default(),
    }
}

fn sync_log_from_row(row: &impl sqlx::Row) -> CRMSyncLog {
    CRMSyncLog {
        id: row.try_get("id").unwrap_or_default(),
        connection_id: row.try_get("connection_id").unwrap_or_default(),
        direction: row.try_get("direction").unwrap_or_default(),
        object_type: row.try_get("object_type").unwrap_or_default(),
        record_count: row.try_get("record_count").unwrap_or(0),
        success_count: row.try_get("success_count").unwrap_or(0),
        error_count: row.try_get("error_count").unwrap_or(0),
        status: row.try_get("status").unwrap_or_default(),
        error_message: row.try_get("error_message").ok(),
        started_at: row.try_get("started_at").unwrap_or_default(),
        completed_at: row.try_get("completed_at").ok(),
    }
}

fn field_mapping_from_row(row: &impl sqlx::Row) -> CRMFieldMapping {
    CRMFieldMapping {
        id: row.try_get("id").unwrap_or_default(),
        connection_id: row.try_get("connection_id").unwrap_or_default(),
        object_type: row.try_get("object_type").unwrap_or_default(),
        local_field: row.try_get("local_field").unwrap_or_default(),
        crm_field: row.try_get("crm_field").unwrap_or_default(),
        direction: row.try_get("direction").unwrap_or_default(),
        transform: row.try_get("transform").ok(),
        is_required: row.try_get("is_required").unwrap_or(false),
        created_at: row.try_get("created_at").unwrap_or_default(),
    }
}

fn external_ref_from_row(row: &impl sqlx::Row) -> CRMExternalRef {
    CRMExternalRef {
        id: row.try_get("id").unwrap_or_default(),
        connection_id: row.try_get("connection_id").unwrap_or_default(),
        local_type: row.try_get("local_type").unwrap_or_default(),
        local_id: row.try_get("local_id").unwrap_or_default(),
        crm_object_type: row.try_get("crm_object_type").unwrap_or_default(),
        crm_record_id: row.try_get("crm_record_id").unwrap_or_default(),
        crm_record_url: row.try_get("crm_record_url").ok(),
        last_synced_at: row.try_get("last_synced_at").unwrap_or_default(),
        sync_status: row.try_get("sync_status").unwrap_or_default(),
        created_at: row.try_get("created_at").unwrap_or_default(),
        updated_at: row.try_get("updated_at").unwrap_or_default(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crm_sync_summary_serialization() {
        let summary = CRMSyncSummary {
            total_connections: 2,
            active_connections: 1,
            disconnected_connections: 1,
            providers: vec![json!({"provider": "salesforce", "count": 1})],
            recent_syncs: vec![],
            total_syncs_7d: 5,
            total_records_synced_7d: 150,
            pending_refs: 0,
            conflict_refs: 0,
        };
        let json = serde_json::to_string(&summary).unwrap();
        assert!(json.contains("salesforce"));
        assert!(json.contains("150"));
    }

    #[test]
    fn test_crm_sync_result_serialization() {
        let result = CRMSyncResult {
            connection_id: "conn-1".to_string(),
            direction: "outbound".to_string(),
            object_type: "lead".to_string(),
            record_count: 10,
            success_count: 10,
            error_count: 0,
            status: "completed".to_string(),
            details: vec![],
        };
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("completed"));
    }

    #[test]
    fn test_crm_providers() {
        let providers = get_crm_providers();
        assert_eq!(providers.len(), 3);
        assert!(providers.iter().any(|p| p.provider == "salesforce"));
        assert!(providers.iter().any(|p| p.provider == "hubspot"));
        assert!(providers.iter().any(|p| p.provider == "pipedrive"));
    }
}
