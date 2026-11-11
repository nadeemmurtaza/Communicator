#![allow(dead_code)]

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{delete, get, post, put},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    db::AppState,
    routes::auth::Tenant,
    models::{
        ClientContactPoint, ClientCustomSection, ClientCustomSectionData, ClientInteractionHistory,
        ClientInvoice, ClientLearningInsight, ClientLearningPattern, ClientOrder,
        ClientOrderStage, ClientPaymentDetail, ClientPreference, ClientProfileSource,
        ClientRagMapping, ClientTaskChecklist, ClientTaskItem, Customer, CustomerChannelId,
    },
};

// ---------------------------------------------------------------------------
// Request/Response types
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct CreateCustomerRequest {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub company: Option<String>,
    pub designation: Option<String>,
    pub role: Option<String>,
    pub brand_id: Option<String>,
}

#[derive(Deserialize)]
pub struct UpdateCustomerRequest {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub company: Option<String>,
    pub designation: Option<String>,
    pub role: Option<String>,
    pub notes: Option<String>,
    pub customer_tier: Option<String>,
    pub customer_segment: Option<String>,
}

#[derive(Deserialize)]
pub struct CreateChannelIdRequest {
    pub channel_type: String,
    pub channel_identifier: String,
    pub is_primary: Option<bool>,
}

#[derive(Deserialize)]
pub struct CreateLearningInsightRequest {
    pub insight_type: String,
    pub category: String,
    pub insight: String,
    pub confidence: Option<f64>,
    pub source_conversation_id: Option<String>,
    pub source_message_id: Option<String>,
}

#[derive(Deserialize)]
pub struct CreateClientPreferenceRequest {
    pub preference_type: String,
    pub preference_value: String,
    pub source: Option<String>,
}

#[derive(Deserialize)]
pub struct CreateClientOrderRequest {
    pub order_number: String,
    pub order_date: String,
    pub total_amount: f64,
    pub currency: Option<String>,
    pub items: Option<String>,
    pub payment_method: Option<String>,
    pub channel: Option<String>,
}

#[derive(Deserialize)]
pub struct CreateClientInvoiceRequest {
    pub invoice_number: String,
    pub invoice_date: String,
    pub total_amount: f64,
    pub order_id: Option<String>,
    pub due_date: Option<String>,
    pub currency: Option<String>,
}

#[derive(Deserialize)]
pub struct CreatePaymentDetailRequest {
    pub detail_type: String,
    pub account_holder: Option<String>,
    pub account_number: Option<String>,
    pub bank_name: Option<String>,
    pub routing_number: Option<String>,
    pub card_last_four: Option<String>,
    pub is_default: Option<bool>,
}

#[derive(Deserialize)]
pub struct CreateContactPointRequest {
    pub contact_type: String,
    pub contact_value: String,
    pub is_primary: Option<bool>,
}

#[derive(Deserialize)]
pub struct CreateOrderStageRequest {
    pub stage_name: String,
    pub stage_value: Option<i64>,
    pub probability: Option<f64>,
    pub expected_close_date: Option<String>,
    pub notes: Option<String>,
}

#[derive(Deserialize)]
pub struct CreateTaskChecklistRequest {
    pub checklist_name: String,
    pub checklist_type: String,
}

#[derive(Deserialize)]
pub struct CreateTaskItemRequest {
    pub task_name: String,
    pub task_description: Option<String>,
    pub priority: Option<String>,
    pub due_date: Option<String>,
    pub display_order: Option<i64>,
}

#[derive(Deserialize)]
pub struct CreateInteractionHistoryRequest {
    pub interaction_type: String,
    pub channel: Option<String>,
    pub direction: Option<String>,
    pub summary: Option<String>,
    pub sentiment: Option<String>,
    pub duration_seconds: Option<i64>,
    pub metadata: Option<String>,
}

#[derive(Deserialize)]
pub struct CreateCustomSectionRequest {
    pub section_name: String,
    pub section_type: String,
    pub section_source: Option<String>,
    pub display_order: Option<i64>,
}

#[derive(Deserialize)]
pub struct CreateCustomSectionDataRequest {
    pub field_key: String,
    pub field_value: String,
    pub field_type: Option<String>,
    pub row_key: Option<String>,
    pub display_order: Option<i64>,
}

#[derive(Deserialize)]
pub struct CreateProfileSourceRequest {
    pub field_name: String,
    pub source_type: String,
    pub source_user_name: Option<String>,
    pub source_record_id: Option<String>,
    pub source_url: Option<String>,
    pub confidence_score: Option<f64>,
}

#[derive(Deserialize)]
pub struct CreateRagMappingRequest {
    pub section_type: String,
    pub section_id: Option<String>,
    pub is_indexed: Option<bool>,
}

#[derive(Deserialize)]
pub struct ResolveConflictRequest {
    pub resolved_value: String,
    pub resolved_by: Option<String>,
}

#[derive(Serialize)]
pub struct UnifiedCustomerProfile {
    pub customer: Customer,
    pub channel_ids: Vec<CustomerChannelId>,
    pub preferences: Vec<ClientPreference>,
    pub insights: Vec<ClientLearningInsight>,
    pub patterns: Vec<ClientLearningPattern>,
    pub orders: Vec<ClientOrder>,
    pub invoices: Vec<ClientInvoice>,
    pub payment_details: Vec<ClientPaymentDetail>,
    pub contact_points: Vec<ClientContactPoint>,
    pub order_stage: Option<ClientOrderStage>,
    pub task_checklists: Vec<ClientTaskChecklist>,
    pub interaction_history: Vec<ClientInteractionHistory>,
    pub custom_sections: Vec<ClientCustomSection>,
    pub profile_sources: Vec<ClientProfileSource>,
    pub rag_mappings: Vec<ClientRagMapping>,
}

#[derive(Deserialize)]
pub struct ListCustomersQuery {
    pub search: Option<String>,
    pub brand_id: Option<String>,
    pub customer_tier: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

// ---------------------------------------------------------------------------
// Router setup
// ---------------------------------------------------------------------------

pub fn router() -> Router<AppState> {
    Router::new()
        // Unified customer resolution
        .route("/clients/resolve", post(resolve_customer))
        // Customer CRUD
        .route("/customers", get(list_customers))
        .route("/customers", post(create_customer))
        .route("/customers/{id", get(get_customer))
        .route("/customers/{id", put(update_customer))
        .route("/customers/{id", delete(delete_customer))
        .route("/customers/{id/profile", get(get_customer_profile))
        // Channel IDs
        .route("/customers/{id/channel-ids", get(list_channel_ids))
        .route("/customers/{id/channel-ids", post(create_channel_id))
        .route("/customers/{id/channel-ids/{channel_id", delete(delete_channel_id))
        // Learning insights
        .route("/customers/{id/insights", get(list_insights))
        .route("/customers/{id/insights", post(create_insight))
        .route("/customers/{id/insights/{insight_id", get(get_insight))
        .route("/customers/{id/insights/{insight_id", put(update_insight))
        .route("/customers/{id/insights/{insight_id", delete(delete_insight))
        // Learning patterns
        .route("/customers/{id/patterns", get(list_patterns))
        .route("/customers/{id/patterns", post(create_pattern))
        .route("/customers/{id/patterns/{pattern_id", delete(delete_pattern))
        // Preferences
        .route("/customers/{id/preferences", get(list_preferences))
        .route("/customers/{id/preferences", post(create_preference))
        .route("/customers/{id/preferences/{pref_id", delete(delete_preference))
        // Orders
        .route("/customers/{id/orders", get(list_orders))
        .route("/customers/{id/orders", post(create_order))
        .route("/customers/{id/orders/{order_id", get(get_order))
        .route("/customers/{id/orders/{order_id", put(update_order))
        .route("/customers/{id/orders/{order_id", delete(delete_order))
        // Invoices
        .route("/customers/{id/invoices", get(list_invoices))
        .route("/customers/{id/invoices", post(create_invoice))
        .route("/customers/{id/invoices/{invoice_id", get(get_invoice))
        .route("/customers/{id/invoices/{invoice_id", put(update_invoice))
        .route("/customers/{id/invoices/{invoice_id", delete(delete_invoice))
        // Payment details
        .route("/customers/{id/payment-details", get(list_payment_details))
        .route("/customers/{id/payment-details", post(create_payment_detail))
        .route("/customers/{id/payment-details/{detail_id", delete(delete_payment_detail))
        // Contact points
        .route("/customers/{id/contact-points", get(list_contact_points))
        .route("/customers/{id/contact-points", post(create_contact_point))
        .route("/customers/{id/contact-points/{point_id", delete(delete_contact_point))
        // Order stage
        .route("/customers/{id/order-stage", get(get_order_stage))
        .route("/customers/{id/order-stage", post(create_order_stage))
        .route("/customers/{id/order-stage/{stage_id", put(update_order_stage))
        .route("/customers/{id/order-stage/{stage_id", delete(delete_order_stage))
        // Task checklists
        .route("/customers/{id/checklists", get(list_checklists))
        .route("/customers/{id/checklists", post(create_checklist))
        .route("/customers/{id/checklists/{checklist_id", get(get_checklist))
        .route("/customers/{id/checklists/{checklist_id", delete(delete_checklist))
        // Task items
        .route("/customers/{id/checklists/{checklist_id/items", get(list_task_items))
        .route("/customers/{id/checklists/{checklist_id/items", post(create_task_item))
        .route("/customers/{id/checklists/{checklist_id/items/{item_id", put(update_task_item))
        .route("/customers/{id/checklists/{checklist_id/items/{item_id", delete(delete_task_item))
        // Interaction history
        .route("/customers/{id/interactions", get(list_interactions))
        .route("/customers/{id/interactions", post(create_interaction))
        .route("/customers/{id/interactions/{interaction_id", delete(delete_interaction))
        // Custom sections
        .route("/customers/{id/custom-sections", get(list_custom_sections))
        .route("/customers/{id/custom-sections", post(create_custom_section))
        .route("/customers/{id/custom-sections/{section_id", get(get_custom_section))
        .route("/customers/{id/custom-sections/{section_id", put(update_custom_section))
        .route("/customers/{id/custom-sections/{section_id", delete(delete_custom_section))
        // Custom section data
        .route(
            "/customers/{id/custom-sections/{section_id/data",
            get(list_custom_section_data),
        )
        .route(
            "/customers/{id/custom-sections/{section_id/data",
            post(create_custom_section_data),
        )
        .route(
            "/customers/{id/custom-sections/{section_id/data/{data_id",
            put(update_custom_section_data),
        )
        .route(
            "/customers/{id/custom-sections/{section_id/data/{data_id",
            delete(delete_custom_section_data),
        )
        // Profile sources
        .route("/customers/{id/profile-sources", get(list_profile_sources))
        .route("/customers/{id/profile-sources", post(create_profile_source))
        .route("/customers/{id/profile-sources/{source_id", delete(delete_profile_source))
        // RAG mappings
        .route("/customers/{id/rag-mappings", get(list_rag_mappings))
        .route("/customers/{id/rag-mappings", post(create_rag_mapping))
        .route("/customers/{id/rag-mappings/{mapping_id", delete(delete_rag_mapping))
}

// ---------------------------------------------------------------------------
// Unified Customer Resolution
// ---------------------------------------------------------------------------

/// Resolve a customer from any channel identifier (phone, email, telegram username, etc.)
/// Creates a new customer if one doesn't exist
async fn resolve_customer(
    State(state): State<AppState>,
    Tenant(tenant_key): Tenant,
    Json(req): Json<CreateChannelIdRequest>,
) -> Result<Json<UnifiedCustomerProfile>, (StatusCode, Json<serde_json::Value>)> {
    let db = &state.db;

    // Try to find existing channel ID mapping
    let existing: Option<CustomerChannelId> = sqlx::query_as(
        "SELECT * FROM customer_channel_ids WHERE channel_type = ? AND channel_identifier = ?",
    )
    .bind(&req.channel_type)
    .bind(&req.channel_identifier)
    .fetch_optional(db)
    .await
    .map_err(|e| (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(serde_json::json!({ "error": e.to_string() })),
    ))?;

    let customer_id = if let Some(ch) = existing {
        ch.customer_id
    } else {
        // Create new customer
        let customer_id = format!("cust_{}", Uuid::new_v4());
        let display_name = format!(
            "{} {}",
            req.channel_identifier,
            if req.channel_type == "email" { "" } else { "(Phone)" }
        );

        sqlx::query(
            "INSERT OR IGNORE INTO customers (id, tenant_key, customer_id, display_name, phone, email) 
             VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(&customer_id)
        .bind(&tenant_key)
        .bind(&customer_id)
        .bind(&display_name)
        .bind(if req.channel_type == "phone" || req.channel_type == "whatsapp" {
            &req.channel_identifier
        } else {
            ""
        })
        .bind(if req.channel_type == "email" {
            &req.channel_identifier
        } else {
            ""
        })
        .execute(db)
        .await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        ))?;

        // Create channel ID mapping
        let channel_id = format!("cc_{}", Uuid::new_v4());
        sqlx::query(
            "INSERT INTO customer_channel_ids (id, customer_id, channel_type, channel_identifier, is_primary) 
             VALUES (?, ?, ?, ?, ?)",
        )
        .bind(&channel_id)
        .bind(&customer_id)
        .bind(&req.channel_type)
        .bind(&req.channel_identifier)
        .bind(req.is_primary.unwrap_or(true))
        .execute(db)
        .await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        ))?;

        customer_id
    };

    get_customer_profile_impl(db, &customer_id).await
}

// ---------------------------------------------------------------------------
// Customer CRUD
// ---------------------------------------------------------------------------

async fn list_customers(
    State(state): State<AppState>,
    Tenant(tenant_key): Tenant,
    Query(params): Query<ListCustomersQuery>,
) -> Result<Json<Vec<Customer>>, (StatusCode, Json<serde_json::Value>)> {
    let db = &state.db;
    let mut sql = format!("SELECT * FROM customers WHERE tenant_key = '{}' ORDER BY created_at DESC", tenant_key);

    if let Some(ref search) = params.search {
        sql.push_str(&format!(
            " AND (display_name LIKE '%{}%' OR first_name LIKE '%{}%' OR last_name LIKE '%{}%' OR email LIKE '%{}%' OR phone LIKE '%{}%' OR company LIKE '%{}%')",
            search, search, search, search, search, search
        ));
    }

    if let Some(ref brand_id) = params.brand_id {
        sql.push_str(&format!(" AND brand_id = '{}'", brand_id));
    }

    if let Some(ref tier) = params.customer_tier {
        sql.push_str(&format!(" AND customer_tier = '{}'", tier));
    }

    if let Some(limit) = params.limit {
        sql.push_str(&format!(" LIMIT {}", limit));
    }

    if let Some(offset) = params.offset {
        sql.push_str(&format!(" OFFSET {}", offset));
    }

    let customers = sqlx::query_as::<_, Customer>(&sql)
        .fetch_all(db)
        .await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        ))?;

    Ok(Json(customers))
}

async fn create_customer(
    State(state): State<AppState>,
    Tenant(tenant_key): Tenant,
    Json(req): Json<CreateCustomerRequest>,
) -> Result<Json<Customer>, (StatusCode, Json<serde_json::Value>)> {
    let db = &state.db;
    let id = format!("cust_{}", Uuid::new_v4());
    let customer_id = &id;
    let display_name = format!(
        "{} {}",
        req.first_name.as_deref().unwrap_or(""),
        req.last_name.as_deref().unwrap_or("")
    );

    sqlx::query(
        "INSERT INTO customers (id, tenant_key, customer_id, display_name, first_name, last_name, 
         email, phone, company, designation, role, brand_id) 
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(&tenant_key)
    .bind(customer_id)
    .bind(&display_name)
    .bind(req.first_name.as_deref().unwrap_or(""))
    .bind(req.last_name.as_deref().unwrap_or(""))
    .bind(req.email.as_deref().unwrap_or(""))
    .bind(req.phone.as_deref().unwrap_or(""))
    .bind(req.company.as_deref().unwrap_or(""))
    .bind(req.designation.as_deref().unwrap_or(""))
    .bind(req.role.as_deref().unwrap_or(""))
    .bind(req.brand_id.as_deref().unwrap_or(""))
    .execute(db)
    .await
    .map_err(|e| (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(serde_json::json!({ "error": e.to_string() })),
    ))?;

    let customer = sqlx::query_as::<_, Customer>("SELECT * FROM customers WHERE id = ?")
        .bind(&id)
        .fetch_one(db)
        .await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        ))?;

    Ok(Json(customer))
}

async fn get_customer(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Customer>, (StatusCode, Json<serde_json::Value>)> {
    let db = &state.db;
    let customer = sqlx::query_as::<_, Customer>("SELECT * FROM customers WHERE id = ?")
        .bind(&id)
        .fetch_one(db)
        .await
        .map_err(|_| (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "Customer not found" })),
        ))?;

    Ok(Json(customer))
}

async fn update_customer(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<UpdateCustomerRequest>,
) -> Result<Json<Customer>, (StatusCode, Json<serde_json::Value>)> {
    let db = &state.db;

    // Build dynamic update query
    let mut sets = Vec::new();
    let mut binds: Vec<&str> = Vec::new();

    if let Some(ref name) = req.first_name {
        sets.push("first_name = ?");
        binds.push(name);
    }
    if let Some(ref name) = req.last_name {
        sets.push("last_name = ?");
        binds.push(name);
    }
    if let Some(ref email) = req.email {
        sets.push("email = ?");
        binds.push(email);
    }
    if let Some(ref phone) = req.phone {
        sets.push("phone = ?");
        binds.push(phone);
    }
    if let Some(ref company) = req.company {
        sets.push("company = ?");
        binds.push(company);
    }
    if let Some(ref designation) = req.designation {
        sets.push("designation = ?");
        binds.push(designation);
    }
    if let Some(ref role) = req.role {
        sets.push("role = ?");
        binds.push(role);
    }
    if let Some(ref notes) = req.notes {
        sets.push("notes = ?");
        binds.push(notes);
    }
    if let Some(ref tier) = req.customer_tier {
        sets.push("customer_tier = ?");
        binds.push(tier);
    }
    if let Some(ref segment) = req.customer_segment {
        sets.push("customer_segment = ?");
        binds.push(segment);
    }
    sets.push("updated_at = NOW()");
    binds.push("");

    if sets.is_empty() {
        return get_customer(State(state), Path(id)).await;
    }

    let query = format!("UPDATE customers SET {} WHERE id = ?", sets.join(", "));
    sqlx::query(&query)
        .bind(&id)
        .execute(db)
        .await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        ))?;

    let customer = sqlx::query_as::<_, Customer>("SELECT * FROM customers WHERE id = ?")
        .bind(&id)
        .fetch_one(db)
        .await
        .map_err(|_| (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "Customer not found" })),
        ))?;

    Ok(Json(customer))
}

async fn delete_customer(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, Json<serde_json::Value>)> {
    let db = &state.db;
    sqlx::query("DELETE FROM customers WHERE id = ?")
        .bind(&id)
        .execute(db)
        .await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        ))?;

    Ok(StatusCode::NO_CONTENT)
}

async fn get_customer_profile(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<UnifiedCustomerProfile>, (StatusCode, Json<serde_json::Value>)> {
    get_customer_profile_impl(&state.db, &id).await
}

async fn get_customer_profile_impl(
    db: &sqlx::PgPool,
    customer_id: &str,
) -> Result<Json<UnifiedCustomerProfile>, (StatusCode, Json<serde_json::Value>)> {
    let customer = sqlx::query_as::<_, Customer>("SELECT * FROM customers WHERE id = ?")
        .bind(customer_id)
        .fetch_one(db)
        .await
        .map_err(|_| (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "Customer not found" })),
        ))?;

    let channel_ids = sqlx::query_as::<_, CustomerChannelId>(
        "SELECT * FROM customer_channel_ids WHERE customer_id = ? ORDER BY is_primary DESC",
    )
    .bind(customer_id)
    .fetch_all(db)
    .await
    .unwrap_or_default();

    let preferences = sqlx::query_as::<_, ClientPreference>(
        "SELECT * FROM client_preferences WHERE customer_id = ?",
    )
    .bind(customer_id)
    .fetch_all(db)
    .await
    .unwrap_or_default();

    let insights = sqlx::query_as::<_, ClientLearningInsight>(
        "SELECT * FROM client_learning_insights WHERE customer_id = ? ORDER BY created_at DESC",
    )
    .bind(customer_id)
    .fetch_all(db)
    .await
    .unwrap_or_default();

    let patterns = sqlx::query_as::<_, ClientLearningPattern>(
        "SELECT * FROM client_learning_patterns WHERE customer_id = ?",
    )
    .bind(customer_id)
    .fetch_all(db)
    .await
    .unwrap_or_default();

    let orders = sqlx::query_as::<_, ClientOrder>(
        "SELECT * FROM client_orders WHERE customer_id = ? ORDER BY order_date DESC",
    )
    .bind(customer_id)
    .fetch_all(db)
    .await
    .unwrap_or_default();

    let invoices = sqlx::query_as::<_, ClientInvoice>(
        "SELECT * FROM client_invoices WHERE customer_id = ? ORDER BY invoice_date DESC",
    )
    .bind(customer_id)
    .fetch_all(db)
    .await
    .unwrap_or_default();

    let payment_details = sqlx::query_as::<_, ClientPaymentDetail>(
        "SELECT * FROM client_payment_details WHERE customer_id = ?",
    )
    .bind(customer_id)
    .fetch_all(db)
    .await
    .unwrap_or_default();

    let contact_points = sqlx::query_as::<_, ClientContactPoint>(
        "SELECT * FROM client_contact_points WHERE customer_id = ? ORDER BY is_primary DESC",
    )
    .bind(customer_id)
    .fetch_all(db)
    .await
    .unwrap_or_default();

    let order_stage = sqlx::query_as::<_, ClientOrderStage>(
        "SELECT * FROM client_order_stage WHERE customer_id = ? ORDER BY created_at DESC LIMIT 1",
    )
    .bind(customer_id)
    .fetch_optional(db)
    .await
    .unwrap_or_default();

    let task_checklists = sqlx::query_as::<_, ClientTaskChecklist>(
        "SELECT * FROM client_task_checklists WHERE customer_id = ? AND is_active = 1",
    )
    .bind(customer_id)
    .fetch_all(db)
    .await
    .unwrap_or_default();

    let interaction_history = sqlx::query_as::<_, ClientInteractionHistory>(
        "SELECT * FROM client_interaction_history WHERE customer_id = ? ORDER BY interaction_date DESC LIMIT 100",
    )
    .bind(customer_id)
    .fetch_all(db)
    .await
    .unwrap_or_default();

    let custom_sections = sqlx::query_as::<_, ClientCustomSection>(
        "SELECT * FROM client_custom_sections WHERE customer_id = ? AND is_active = 1 ORDER BY display_order",
    )
    .bind(customer_id)
    .fetch_all(db)
    .await
    .unwrap_or_default();

    let profile_sources = sqlx::query_as::<_, ClientProfileSource>(
        "SELECT * FROM client_profile_sources WHERE customer_id = ?",
    )
    .bind(customer_id)
    .fetch_all(db)
    .await
    .unwrap_or_default();

    let rag_mappings = sqlx::query_as::<_, ClientRagMapping>(
        "SELECT * FROM client_rag_mapping WHERE customer_id = ?",
    )
    .bind(customer_id)
    .fetch_all(db)
    .await
    .unwrap_or_default();

    Ok(Json(UnifiedCustomerProfile {
        customer,
        channel_ids,
        preferences,
        insights,
        patterns,
        orders,
        invoices,
        payment_details,
        contact_points,
        order_stage,
        task_checklists,
        interaction_history,
        custom_sections,
        profile_sources,
        rag_mappings,
    }))
}

// ---------------------------------------------------------------------------
// Channel IDs
// ---------------------------------------------------------------------------

async fn list_channel_ids(
    State(state): State<AppState>,
    Path(customer_id): Path<String>,
) -> Result<Json<Vec<CustomerChannelId>>, (StatusCode, Json<serde_json::Value>)> {
    let channel_ids = sqlx::query_as::<_, CustomerChannelId>(
        "SELECT * FROM customer_channel_ids WHERE customer_id = ?",
    )
    .bind(&customer_id)
    .fetch_all(&state.db)
    .await
    .map_err(|e| (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(serde_json::json!({ "error": e.to_string() })),
    ))?;

    Ok(Json(channel_ids))
}

async fn create_channel_id(
    State(state): State<AppState>,
    Path(customer_id): Path<String>,
    Json(req): Json<CreateChannelIdRequest>,
) -> Result<Json<CustomerChannelId>, (StatusCode, Json<serde_json::Value>)> {
    let db = &state.db;
    let id = format!("cc_{}", Uuid::new_v4());

    sqlx::query(
        "INSERT INTO customer_channel_ids (id, customer_id, channel_type, channel_identifier, is_primary) 
         VALUES (?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(&customer_id)
    .bind(&req.channel_type)
    .bind(&req.channel_identifier)
    .bind(req.is_primary.unwrap_or(false))
    .execute(db)
    .await
    .map_err(|e| (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(serde_json::json!({ "error": e.to_string() })),
    ))?;

    let channel_id = sqlx::query_as::<_, CustomerChannelId>("SELECT * FROM customer_channel_ids WHERE id = ?")
        .bind(&id)
        .fetch_one(db)
        .await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        ))?;

    Ok(Json(channel_id))
}

async fn delete_channel_id(
    State(state): State<AppState>,
    Path((customer_id, channel_id)): Path<(String, String)>,
) -> Result<StatusCode, (StatusCode, Json<serde_json::Value>)> {
    sqlx::query("DELETE FROM customer_channel_ids WHERE id = ? AND customer_id = ?")
        .bind(&channel_id)
        .bind(&customer_id)
        .execute(&state.db)
        .await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        ))?;

    Ok(StatusCode::NO_CONTENT)
}

// ---------------------------------------------------------------------------
// Learning Insights
// ---------------------------------------------------------------------------

async fn list_insights(
    State(state): State<AppState>,
    Path(customer_id): Path<String>,
) -> Result<Json<Vec<ClientLearningInsight>>, (StatusCode, Json<serde_json::Value>)> {
    let insights = sqlx::query_as::<_, ClientLearningInsight>(
        "SELECT * FROM client_learning_insights WHERE customer_id = ? ORDER BY created_at DESC",
    )
    .bind(&customer_id)
    .fetch_all(&state.db)
    .await
    .map_err(|e| (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(serde_json::json!({ "error": e.to_string() })),
    ))?;

    Ok(Json(insights))
}

async fn create_insight(
    State(state): State<AppState>,
    Path(customer_id): Path<String>,
    Json(req): Json<CreateLearningInsightRequest>,
) -> Result<Json<ClientLearningInsight>, (StatusCode, Json<serde_json::Value>)> {
    let db = &state.db;
    let id = format!("insight_{}", Uuid::new_v4());
    let now = chrono::Utc::now().to_rfc3339();

    sqlx::query(
        "INSERT INTO client_learning_insights (id, customer_id, insight_type, category, insight, 
         confidence, source_conversation_id, source_message_id, extracted_at) 
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(&customer_id)
    .bind(&req.insight_type)
    .bind(&req.category)
    .bind(&req.insight)
    .bind(req.confidence.unwrap_or(0.5))
    .bind(req.source_conversation_id.as_deref().unwrap_or(""))
    .bind(req.source_message_id.as_deref().unwrap_or(""))
    .bind(&now)
    .execute(db)
    .await
    .map_err(|e| (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(serde_json::json!({ "error": e.to_string() })),
    ))?;

    let insight = sqlx::query_as::<_, ClientLearningInsight>("SELECT * FROM client_learning_insights WHERE id = ?")
        .bind(&id)
        .fetch_one(db)
        .await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        ))?;

    Ok(Json(insight))
}

async fn get_insight(
    State(state): State<AppState>,
    Path((_customer_id, insight_id)): Path<(String, String)>,
) -> Result<Json<ClientLearningInsight>, (StatusCode, Json<serde_json::Value>)> {
    let insight = sqlx::query_as::<_, ClientLearningInsight>(
        "SELECT * FROM client_learning_insights WHERE id = ?",
    )
    .bind(&insight_id)
    .fetch_one(&state.db)
    .await
    .map_err(|_| (
        StatusCode::NOT_FOUND,
        Json(serde_json::json!({ "error": "Insight not found" })),
    ))?;

    Ok(Json(insight))
}

async fn update_insight(
    State(state): State<AppState>,
    Path((_customer_id, insight_id)): Path<(String, String)>,
    Json(req): Json<CreateLearningInsightRequest>,
) -> Result<Json<ClientLearningInsight>, (StatusCode, Json<serde_json::Value>)> {
    let db = &state.db;

    sqlx::query(
        "UPDATE client_learning_insights SET insight_type = ?, category = ?, insight = ?, 
         confidence = ?, updated_at = NOW() WHERE id = ?",
    )
    .bind(&req.insight_type)
    .bind(&req.category)
    .bind(&req.insight)
    .bind(req.confidence.unwrap_or(0.5))
    .bind(&insight_id)
    .execute(db)
    .await
    .map_err(|e| (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(serde_json::json!({ "error": e.to_string() })),
    ))?;

    let insight = sqlx::query_as::<_, ClientLearningInsight>("SELECT * FROM client_learning_insights WHERE id = ?")
        .bind(&insight_id)
        .fetch_one(db)
        .await
        .map_err(|_| (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "Insight not found" })),
        ))?;

    Ok(Json(insight))
}

async fn delete_insight(
    State(state): State<AppState>,
    Path((_customer_id, insight_id)): Path<(String, String)>,
) -> Result<StatusCode, (StatusCode, Json<serde_json::Value>)> {
    sqlx::query("DELETE FROM client_learning_insights WHERE id = ?")
        .bind(&insight_id)
        .execute(&state.db)
        .await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        ))?;

    Ok(StatusCode::NO_CONTENT)
}

// ---------------------------------------------------------------------------
// Learning Patterns
// ---------------------------------------------------------------------------

async fn list_patterns(
    State(state): State<AppState>,
    Path(customer_id): Path<String>,
) -> Result<Json<Vec<ClientLearningPattern>>, (StatusCode, Json<serde_json::Value>)> {
    let patterns = sqlx::query_as::<_, ClientLearningPattern>(
        "SELECT * FROM client_learning_patterns WHERE customer_id = ?",
    )
    .bind(&customer_id)
    .fetch_all(&state.db)
    .await
    .map_err(|e| (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(serde_json::json!({ "error": e.to_string() })),
    ))?;

    Ok(Json(patterns))
}

#[derive(Deserialize)]
pub struct CreateLearningPatternRequest {
    pub pattern_type: String,
    pub pattern_data: String,
    pub confidence: Option<f64>,
    pub sample_size: Option<i64>,
}

async fn create_pattern(
    State(state): State<AppState>,
    Path(customer_id): Path<String>,
    Json(req): Json<CreateLearningPatternRequest>,
) -> Result<Json<ClientLearningPattern>, (StatusCode, Json<serde_json::Value>)> {
    let db = &state.db;
    let id = format!("pattern_{}", Uuid::new_v4());
    let now = chrono::Utc::now().to_rfc3339();

    sqlx::query(
        "INSERT INTO client_learning_patterns (id, customer_id, pattern_type, pattern_data, 
         confidence, sample_size, last_updated) 
         VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(&customer_id)
    .bind(&req.pattern_type)
    .bind(&req.pattern_data)
    .bind(req.confidence.unwrap_or(0.5))
    .bind(req.sample_size.unwrap_or(0))
    .bind(&now)
    .execute(db)
    .await
    .map_err(|e| (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(serde_json::json!({ "error": e.to_string() })),
    ))?;

    let pattern = sqlx::query_as::<_, ClientLearningPattern>("SELECT * FROM client_learning_patterns WHERE id = ?")
        .bind(&id)
        .fetch_one(db)
        .await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        ))?;

    Ok(Json(pattern))
}

async fn delete_pattern(
    State(state): State<AppState>,
    Path((_customer_id, pattern_id)): Path<(String, String)>,
) -> Result<StatusCode, (StatusCode, Json<serde_json::Value>)> {
    sqlx::query("DELETE FROM client_learning_patterns WHERE id = ?")
        .bind(&pattern_id)
        .execute(&state.db)
        .await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        ))?;

    Ok(StatusCode::NO_CONTENT)
}

// ---------------------------------------------------------------------------
// Preferences
// ---------------------------------------------------------------------------

async fn list_preferences(
    State(state): State<AppState>,
    Path(customer_id): Path<String>,
) -> Result<Json<Vec<ClientPreference>>, (StatusCode, Json<serde_json::Value>)> {
    let preferences = sqlx::query_as::<_, ClientPreference>(
        "SELECT * FROM client_preferences WHERE customer_id = ?",
    )
    .bind(&customer_id)
    .fetch_all(&state.db)
    .await
    .map_err(|e| (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(serde_json::json!({ "error": e.to_string() })),
    ))?;

    Ok(Json(preferences))
}

async fn create_preference(
    State(state): State<AppState>,
    Path(customer_id): Path<String>,
    Json(req): Json<CreateClientPreferenceRequest>,
) -> Result<Json<ClientPreference>, (StatusCode, Json<serde_json::Value>)> {
    let db = &state.db;
    let id = format!("pref_{}", Uuid::new_v4());

    sqlx::query(
        "INSERT INTO client_preferences (id, customer_id, preference_type, preference_value, source) 
         VALUES (?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(&customer_id)
    .bind(&req.preference_type)
    .bind(&req.preference_value)
    .bind(req.source.as_deref().unwrap_or("ai_learning"))
    .execute(db)
    .await
    .map_err(|e| (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(serde_json::json!({ "error": e.to_string() })),
    ))?;

    let preference = sqlx::query_as::<_, ClientPreference>("SELECT * FROM client_preferences WHERE id = ?")
        .bind(&id)
        .fetch_one(db)
        .await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        ))?;

    Ok(Json(preference))
}

async fn delete_preference(
    State(state): State<AppState>,
    Path((_customer_id, pref_id)): Path<(String, String)>,
) -> Result<StatusCode, (StatusCode, Json<serde_json::Value>)> {
    sqlx::query("DELETE FROM client_preferences WHERE id = ?")
        .bind(&pref_id)
        .execute(&state.db)
        .await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        ))?;

    Ok(StatusCode::NO_CONTENT)
}

// ---------------------------------------------------------------------------
// Orders
// ---------------------------------------------------------------------------

async fn list_orders(
    State(state): State<AppState>,
    Path(customer_id): Path<String>,
) -> Result<Json<Vec<ClientOrder>>, (StatusCode, Json<serde_json::Value>)> {
    let orders = sqlx::query_as::<_, ClientOrder>(
        "SELECT * FROM client_orders WHERE customer_id = ? ORDER BY order_date DESC",
    )
    .bind(&customer_id)
    .fetch_all(&state.db)
    .await
    .map_err(|e| (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(serde_json::json!({ "error": e.to_string() })),
    ))?;

    Ok(Json(orders))
}

async fn create_order(
    State(state): State<AppState>,
    Path(customer_id): Path<String>,
    Json(req): Json<CreateClientOrderRequest>,
) -> Result<Json<ClientOrder>, (StatusCode, Json<serde_json::Value>)> {
    let db = &state.db;
    let id = format!("order_{}", Uuid::new_v4());

    sqlx::query(
        "INSERT INTO client_orders (id, customer_id, order_number, order_date, total_amount, 
         currency, items, payment_method, channel) 
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(&customer_id)
    .bind(&req.order_number)
    .bind(&req.order_date)
    .bind(req.total_amount)
    .bind(req.currency.as_deref().unwrap_or("USD"))
    .bind(req.items.as_deref().unwrap_or("[]"))
    .bind(req.payment_method.as_deref().unwrap_or(""))
    .bind(req.channel.as_deref().unwrap_or(""))
    .execute(db)
    .await
    .map_err(|e| (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(serde_json::json!({ "error": e.to_string() })),
    ))?;

    let order = sqlx::query_as::<_, ClientOrder>("SELECT * FROM client_orders WHERE id = ?")
        .bind(&id)
        .fetch_one(db)
        .await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        ))?;

    Ok(Json(order))
}

async fn get_order(
    State(state): State<AppState>,
    Path((_customer_id, order_id)): Path<(String, String)>,
) -> Result<Json<ClientOrder>, (StatusCode, Json<serde_json::Value>)> {
    let order = sqlx::query_as::<_, ClientOrder>("SELECT * FROM client_orders WHERE id = ?")
        .bind(&order_id)
        .fetch_one(&state.db)
        .await
        .map_err(|_| (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "Order not found" })),
        ))?;

    Ok(Json(order))
}

#[derive(Deserialize)]
pub struct UpdateClientOrderRequest {
    pub status: Option<String>,
    pub payment_status: Option<String>,
}

async fn update_order(
    State(state): State<AppState>,
    Path((_customer_id, order_id)): Path<(String, String)>,
    Json(req): Json<UpdateClientOrderRequest>,
) -> Result<Json<ClientOrder>, (StatusCode, Json<serde_json::Value>)> {
    let db = &state.db;

    let mut sets = Vec::new();
    if let Some(ref status) = req.status {
        sets.push("status = ?");
    }
    if let Some(ref ps) = req.payment_status {
        sets.push("payment_status = ?");
    }
    if sets.is_empty() {
        return get_order(State(state), Path((_customer_id, order_id))).await;
    }
    sets.push("updated_at = NOW()");

    let sql = format!("UPDATE client_orders SET {} WHERE id = ?", sets.join(", "));
    let mut query = sqlx::query(&sql);
    if let Some(ref status) = req.status {
        query = query.bind(status);
    }
    if let Some(ref ps) = req.payment_status {
        query = query.bind(ps);
    }
    query.bind(&order_id)
        .execute(db)
        .await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        ))?;

    let order = sqlx::query_as::<_, ClientOrder>("SELECT * FROM client_orders WHERE id = ?")
        .bind(&order_id)
        .fetch_one(db)
        .await
        .map_err(|_| (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "Order not found" })),
        ))?;

    Ok(Json(order))
}

async fn delete_order(
    State(state): State<AppState>,
    Path((_customer_id, order_id)): Path<(String, String)>,
) -> Result<StatusCode, (StatusCode, Json<serde_json::Value>)> {
    sqlx::query("DELETE FROM client_orders WHERE id = ?")
        .bind(&order_id)
        .execute(&state.db)
        .await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        ))?;

    Ok(StatusCode::NO_CONTENT)
}

// ---------------------------------------------------------------------------
// Invoices
// ---------------------------------------------------------------------------

async fn list_invoices(
    State(state): State<AppState>,
    Path(customer_id): Path<String>,
) -> Result<Json<Vec<ClientInvoice>>, (StatusCode, Json<serde_json::Value>)> {
    let invoices = sqlx::query_as::<_, ClientInvoice>(
        "SELECT * FROM client_invoices WHERE customer_id = ? ORDER BY invoice_date DESC",
    )
    .bind(&customer_id)
    .fetch_all(&state.db)
    .await
    .map_err(|e| (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(serde_json::json!({ "error": e.to_string() })),
    ))?;

    Ok(Json(invoices))
}

async fn create_invoice(
    State(state): State<AppState>,
    Path(customer_id): Path<String>,
    Json(req): Json<CreateClientInvoiceRequest>,
) -> Result<Json<ClientInvoice>, (StatusCode, Json<serde_json::Value>)> {
    let db = &state.db;
    let id = format!("inv_{}", Uuid::new_v4());

    sqlx::query(
        "INSERT INTO client_invoices (id, customer_id, invoice_number, order_id, invoice_date, 
         due_date, total_amount, currency) 
         VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(&customer_id)
    .bind(&req.invoice_number)
    .bind(req.order_id.as_deref().unwrap_or(""))
    .bind(&req.invoice_date)
    .bind(req.due_date.as_deref().unwrap_or(""))
    .bind(req.total_amount)
    .bind(req.currency.as_deref().unwrap_or("USD"))
    .execute(db)
    .await
    .map_err(|e| (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(serde_json::json!({ "error": e.to_string() })),
    ))?;

    let invoice = sqlx::query_as::<_, ClientInvoice>("SELECT * FROM client_invoices WHERE id = ?")
        .bind(&id)
        .fetch_one(db)
        .await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        ))?;

    Ok(Json(invoice))
}

async fn get_invoice(
    State(state): State<AppState>,
    Path((_customer_id, invoice_id)): Path<(String, String)>,
) -> Result<Json<ClientInvoice>, (StatusCode, Json<serde_json::Value>)> {
    let invoice = sqlx::query_as::<_, ClientInvoice>("SELECT * FROM client_invoices WHERE id = ?")
        .bind(&invoice_id)
        .fetch_one(&state.db)
        .await
        .map_err(|_| (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "Invoice not found" })),
        ))?;

    Ok(Json(invoice))
}

#[derive(Deserialize)]
pub struct UpdateClientInvoiceRequest {
    pub status: Option<String>,
    pub paid_amount: Option<f64>,
}

async fn update_invoice(
    State(state): State<AppState>,
    Path((_customer_id, invoice_id)): Path<(String, String)>,
    Json(req): Json<UpdateClientInvoiceRequest>,
) -> Result<Json<ClientInvoice>, (StatusCode, Json<serde_json::Value>)> {
    let db = &state.db;

    if let Some(ref status) = req.status {
        sqlx::query(
            "UPDATE client_invoices SET status = ?, updated_at = NOW() WHERE id = ?",
        )
        .bind(status)
        .bind(&invoice_id)
        .execute(db)
        .await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        ))?;
    }

    let invoice = sqlx::query_as::<_, ClientInvoice>("SELECT * FROM client_invoices WHERE id = ?")
        .bind(&invoice_id)
        .fetch_one(db)
        .await
        .map_err(|_| (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "Invoice not found" })),
        ))?;

    Ok(Json(invoice))
}

async fn delete_invoice(
    State(state): State<AppState>,
    Path((_customer_id, invoice_id)): Path<(String, String)>,
) -> Result<StatusCode, (StatusCode, Json<serde_json::Value>)> {
    sqlx::query("DELETE FROM client_invoices WHERE id = ?")
        .bind(&invoice_id)
        .execute(&state.db)
        .await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        ))?;

    Ok(StatusCode::NO_CONTENT)
}

// ---------------------------------------------------------------------------
// Payment Details
// ---------------------------------------------------------------------------

async fn list_payment_details(
    State(state): State<AppState>,
    Path(customer_id): Path<String>,
) -> Result<Json<Vec<ClientPaymentDetail>>, (StatusCode, Json<serde_json::Value>)> {
    let details = sqlx::query_as::<_, ClientPaymentDetail>(
        "SELECT * FROM client_payment_details WHERE customer_id = ?",
    )
    .bind(&customer_id)
    .fetch_all(&state.db)
    .await
    .map_err(|e| (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(serde_json::json!({ "error": e.to_string() })),
    ))?;

    Ok(Json(details))
}

async fn create_payment_detail(
    State(state): State<AppState>,
    Path(customer_id): Path<String>,
    Json(req): Json<CreatePaymentDetailRequest>,
) -> Result<Json<ClientPaymentDetail>, (StatusCode, Json<serde_json::Value>)> {
    let db = &state.db;
    let id = format!("pay_{}", Uuid::new_v4());

    sqlx::query(
        "INSERT INTO client_payment_details (id, customer_id, detail_type, account_holder, 
         account_number, bank_name, routing_number, card_last_four, is_default) 
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(&customer_id)
    .bind(&req.detail_type)
    .bind(req.account_holder.as_deref().unwrap_or(""))
    .bind(req.account_number.as_deref().unwrap_or(""))
    .bind(req.bank_name.as_deref().unwrap_or(""))
    .bind(req.routing_number.as_deref().unwrap_or(""))
    .bind(req.card_last_four.as_deref().unwrap_or(""))
    .bind(req.is_default.unwrap_or(false))
    .execute(db)
    .await
    .map_err(|e| (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(serde_json::json!({ "error": e.to_string() })),
    ))?;

    let detail = sqlx::query_as::<_, ClientPaymentDetail>("SELECT * FROM client_payment_details WHERE id = ?")
        .bind(&id)
        .fetch_one(db)
        .await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        ))?;

    Ok(Json(detail))
}

async fn delete_payment_detail(
    State(state): State<AppState>,
    Path((_customer_id, detail_id)): Path<(String, String)>,
) -> Result<StatusCode, (StatusCode, Json<serde_json::Value>)> {
    sqlx::query("DELETE FROM client_payment_details WHERE id = ?")
        .bind(&detail_id)
        .execute(&state.db)
        .await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        ))?;

    Ok(StatusCode::NO_CONTENT)
}

// ---------------------------------------------------------------------------
// Contact Points
// ---------------------------------------------------------------------------

async fn list_contact_points(
    State(state): State<AppState>,
    Path(customer_id): Path<String>,
) -> Result<Json<Vec<ClientContactPoint>>, (StatusCode, Json<serde_json::Value>)> {
    let points = sqlx::query_as::<_, ClientContactPoint>(
        "SELECT * FROM client_contact_points WHERE customer_id = ? ORDER BY is_primary DESC",
    )
    .bind(&customer_id)
    .fetch_all(&state.db)
    .await
    .map_err(|e| (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(serde_json::json!({ "error": e.to_string() })),
    ))?;

    Ok(Json(points))
}

async fn create_contact_point(
    State(state): State<AppState>,
    Path(customer_id): Path<String>,
    Json(req): Json<CreateContactPointRequest>,
) -> Result<Json<ClientContactPoint>, (StatusCode, Json<serde_json::Value>)> {
    let db = &state.db;
    let id = format!("cp_{}", Uuid::new_v4());

    sqlx::query(
        "INSERT INTO client_contact_points (id, customer_id, contact_type, contact_value, is_primary) 
         VALUES (?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(&customer_id)
    .bind(&req.contact_type)
    .bind(&req.contact_value)
    .bind(req.is_primary.unwrap_or(false))
    .execute(db)
    .await
    .map_err(|e| (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(serde_json::json!({ "error": e.to_string() })),
    ))?;

    let point = sqlx::query_as::<_, ClientContactPoint>("SELECT * FROM client_contact_points WHERE id = ?")
        .bind(&id)
        .fetch_one(db)
        .await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        ))?;

    Ok(Json(point))
}

async fn delete_contact_point(
    State(state): State<AppState>,
    Path((_customer_id, point_id)): Path<(String, String)>,
) -> Result<StatusCode, (StatusCode, Json<serde_json::Value>)> {
    sqlx::query("DELETE FROM client_contact_points WHERE id = ?")
        .bind(&point_id)
        .execute(&state.db)
        .await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        ))?;

    Ok(StatusCode::NO_CONTENT)
}

// ---------------------------------------------------------------------------
// Order Stage
// ---------------------------------------------------------------------------

async fn get_order_stage(
    State(state): State<AppState>,
    Path(customer_id): Path<String>,
) -> Result<Json<Option<ClientOrderStage>>, (StatusCode, Json<serde_json::Value>)> {
    let stage = sqlx::query_as::<_, ClientOrderStage>(
        "SELECT * FROM client_order_stage WHERE customer_id = ? ORDER BY created_at DESC LIMIT 1",
    )
    .bind(&customer_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(serde_json::json!({ "error": e.to_string() })),
    ))?;

    Ok(Json(stage))
}

async fn create_order_stage(
    State(state): State<AppState>,
    Path(customer_id): Path<String>,
    Json(req): Json<CreateOrderStageRequest>,
) -> Result<Json<ClientOrderStage>, (StatusCode, Json<serde_json::Value>)> {
    let db = &state.db;
    let id = format!("stage_{}", Uuid::new_v4());

    sqlx::query(
        "INSERT INTO client_order_stage (id, customer_id, stage_name, stage_value, probability, 
         expected_close_date, notes) 
         VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(&customer_id)
    .bind(&req.stage_name)
    .bind(req.stage_value.unwrap_or(0))
    .bind(req.probability.unwrap_or(0.0))
    .bind(req.expected_close_date.as_deref().unwrap_or(""))
    .bind(req.notes.as_deref().unwrap_or(""))
    .execute(db)
    .await
    .map_err(|e| (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(serde_json::json!({ "error": e.to_string() })),
    ))?;

    let stage = sqlx::query_as::<_, ClientOrderStage>("SELECT * FROM client_order_stage WHERE id = ?")
        .bind(&id)
        .fetch_one(db)
        .await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        ))?;

    Ok(Json(stage))
}

#[derive(Deserialize)]
pub struct UpdateOrderStageRequest {
    pub stage_name: Option<String>,
    pub stage_value: Option<i64>,
    pub probability: Option<f64>,
    pub expected_close_date: Option<String>,
    pub notes: Option<String>,
}

async fn update_order_stage(
    State(state): State<AppState>,
    Path((_customer_id, stage_id)): Path<(String, String)>,
    Json(req): Json<UpdateOrderStageRequest>,
) -> Result<Json<ClientOrderStage>, (StatusCode, Json<serde_json::Value>)> {
    let db = &state.db;

    let mut sets = Vec::new();
    let mut binds: Vec<&str> = Vec::new();

    if let Some(ref name) = req.stage_name {
        sets.push("stage_name = ?");
        binds.push(name);
    }
    if let Some(ref notes) = req.notes {
        sets.push("notes = ?");
        binds.push(notes);
    }
    sets.push("updated_at = NOW()");
    binds.push("");

    if sets.is_empty() {
        let stage = sqlx::query_as::<_, ClientOrderStage>("SELECT * FROM client_order_stage WHERE id = ?")
            .bind(&stage_id)
            .fetch_one(db)
            .await
            .map_err(|_| (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({ "error": "Stage not found" })),
            ))?;
        return Ok(Json(stage));
    }

    let query = format!("UPDATE client_order_stage SET {} WHERE id = ?", sets.join(", "));
    sqlx::query(&query)
        .bind(&stage_id)
        .execute(db)
        .await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        ))?;

    let stage = sqlx::query_as::<_, ClientOrderStage>("SELECT * FROM client_order_stage WHERE id = ?")
        .bind(&stage_id)
        .fetch_one(db)
        .await
        .map_err(|_| (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "Stage not found" })),
        ))?;

    Ok(Json(stage))
}

async fn delete_order_stage(
    State(state): State<AppState>,
    Path((_customer_id, stage_id)): Path<(String, String)>,
) -> Result<StatusCode, (StatusCode, Json<serde_json::Value>)> {
    sqlx::query("DELETE FROM client_order_stage WHERE id = ?")
        .bind(&stage_id)
        .execute(&state.db)
        .await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        ))?;

    Ok(StatusCode::NO_CONTENT)
}

// ---------------------------------------------------------------------------
// Task Checklists
// ---------------------------------------------------------------------------

async fn list_checklists(
    State(state): State<AppState>,
    Path(customer_id): Path<String>,
) -> Result<Json<Vec<ClientTaskChecklist>>, (StatusCode, Json<serde_json::Value>)> {
    let checklists = sqlx::query_as::<_, ClientTaskChecklist>(
        "SELECT * FROM client_task_checklists WHERE customer_id = ? AND is_active = 1",
    )
    .bind(&customer_id)
    .fetch_all(&state.db)
    .await
    .map_err(|e| (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(serde_json::json!({ "error": e.to_string() })),
    ))?;

    Ok(Json(checklists))
}

async fn create_checklist(
    State(state): State<AppState>,
    Path(customer_id): Path<String>,
    Json(req): Json<CreateTaskChecklistRequest>,
) -> Result<Json<ClientTaskChecklist>, (StatusCode, Json<serde_json::Value>)> {
    let db = &state.db;
    let id = format!("checklist_{}", Uuid::new_v4());

    sqlx::query(
        "INSERT INTO client_task_checklists (id, customer_id, checklist_name, checklist_type) 
         VALUES (?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(&customer_id)
    .bind(&req.checklist_name)
    .bind(&req.checklist_type)
    .execute(db)
    .await
    .map_err(|e| (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(serde_json::json!({ "error": e.to_string() })),
    ))?;

    let checklist = sqlx::query_as::<_, ClientTaskChecklist>("SELECT * FROM client_task_checklists WHERE id = ?")
        .bind(&id)
        .fetch_one(db)
        .await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        ))?;

    Ok(Json(checklist))
}

async fn get_checklist(
    State(state): State<AppState>,
    Path((_customer_id, checklist_id)): Path<(String, String)>,
) -> Result<Json<ClientTaskChecklist>, (StatusCode, Json<serde_json::Value>)> {
    let checklist = sqlx::query_as::<_, ClientTaskChecklist>(
        "SELECT * FROM client_task_checklists WHERE id = ?",
    )
    .bind(&checklist_id)
    .fetch_one(&state.db)
    .await
    .map_err(|_| (
        StatusCode::NOT_FOUND,
        Json(serde_json::json!({ "error": "Checklist not found" })),
    ))?;

    Ok(Json(checklist))
}

async fn delete_checklist(
    State(state): State<AppState>,
    Path((_customer_id, checklist_id)): Path<(String, String)>,
) -> Result<StatusCode, (StatusCode, Json<serde_json::Value>)> {
    sqlx::query("DELETE FROM client_task_checklists WHERE id = ?")
        .bind(&checklist_id)
        .execute(&state.db)
        .await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        ))?;

    Ok(StatusCode::NO_CONTENT)
}

// ---------------------------------------------------------------------------
// Task Items
// ---------------------------------------------------------------------------

async fn list_task_items(
    State(state): State<AppState>,
    Path((_customer_id, checklist_id)): Path<(String, String)>,
) -> Result<Json<Vec<ClientTaskItem>>, (StatusCode, Json<serde_json::Value>)> {
    let items = sqlx::query_as::<_, ClientTaskItem>(
        "SELECT * FROM client_task_items WHERE checklist_id = ? ORDER BY display_order",
    )
    .bind(&checklist_id)
    .fetch_all(&state.db)
    .await
    .map_err(|e| (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(serde_json::json!({ "error": e.to_string() })),
    ))?;

    Ok(Json(items))
}

async fn create_task_item(
    State(state): State<AppState>,
    Path((_customer_id, checklist_id)): Path<(String, String)>,
    Json(req): Json<CreateTaskItemRequest>,
) -> Result<Json<ClientTaskItem>, (StatusCode, Json<serde_json::Value>)> {
    let db = &state.db;
    let id = format!("task_{}", Uuid::new_v4());

    sqlx::query(
        "INSERT INTO client_task_items (id, checklist_id, task_name, task_description, priority, 
         due_date, display_order) 
         VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(&checklist_id)
    .bind(&req.task_name)
    .bind(req.task_description.as_deref().unwrap_or(""))
    .bind(req.priority.as_deref().unwrap_or("medium"))
    .bind(req.due_date.as_deref().unwrap_or(""))
    .bind(req.display_order.unwrap_or(0))
    .execute(db)
    .await
    .map_err(|e| (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(serde_json::json!({ "error": e.to_string() })),
    ))?;

    let item = sqlx::query_as::<_, ClientTaskItem>("SELECT * FROM client_task_items WHERE id = ?")
        .bind(&id)
        .fetch_one(db)
        .await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        ))?;

    Ok(Json(item))
}

#[derive(Deserialize)]
pub struct UpdateTaskItemRequest {
    pub task_name: Option<String>,
    pub task_description: Option<String>,
    pub is_completed: Option<bool>,
    pub priority: Option<String>,
    pub due_date: Option<String>,
}

async fn update_task_item(
    State(state): State<AppState>,
    Path((_customer_id, _checklist_id, item_id)): Path<(String, String, String)>,
    Json(req): Json<UpdateTaskItemRequest>,
) -> Result<Json<ClientTaskItem>, (StatusCode, Json<serde_json::Value>)> {
    let db = &state.db;

    let mut sets = Vec::new();
    let mut binds: Vec<&str> = Vec::new();

    if let Some(ref name) = req.task_name {
        sets.push("task_name = ?");
        binds.push(name);
    }
    if let Some(ref desc) = req.task_description {
        sets.push("task_description = ?");
        binds.push(desc);
    }
    if let Some(completed) = req.is_completed {
        sets.push("is_completed = ?");
        binds.push(if completed { "1" } else { "0" });
    }
    if let Some(ref priority) = req.priority {
        sets.push("priority = ?");
        binds.push(priority);
    }
    sets.push("updated_at = NOW()");
    binds.push("");

    if sets.is_empty() {
        return Ok(Json(ClientTaskItem {
            id: item_id.clone(),
            checklist_id: _checklist_id,
            task_name: String::new(),
            task_description: String::new(),
            is_completed: false,
            completed_at: None,
            completed_by: String::new(),
            due_date: None,
            priority: String::new(),
            display_order: 0,
            created_at: String::new(),
            updated_at: String::new(),
        }));
    }

    let query = format!("UPDATE client_task_items SET {} WHERE id = ?", sets.join(", "));
    sqlx::query(&query)
        .bind(&item_id)
        .execute(db)
        .await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        ))?;

    let item = sqlx::query_as::<_, ClientTaskItem>("SELECT * FROM client_task_items WHERE id = ?")
        .bind(&item_id)
        .fetch_one(db)
        .await
        .map_err(|_| (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "Task item not found" })),
        ))?;

    Ok(Json(item))
}

async fn delete_task_item(
    State(state): State<AppState>,
    Path((_customer_id, _checklist_id, item_id)): Path<(String, String, String)>,
) -> Result<StatusCode, (StatusCode, Json<serde_json::Value>)> {
    sqlx::query("DELETE FROM client_task_items WHERE id = ?")
        .bind(&item_id)
        .execute(&state.db)
        .await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        ))?;

    Ok(StatusCode::NO_CONTENT)
}

// ---------------------------------------------------------------------------
// Interaction History
// ---------------------------------------------------------------------------

async fn list_interactions(
    State(state): State<AppState>,
    Path(customer_id): Path<String>,
) -> Result<Json<Vec<ClientInteractionHistory>>, (StatusCode, Json<serde_json::Value>)> {
    let interactions = sqlx::query_as::<_, ClientInteractionHistory>(
        "SELECT * FROM client_interaction_history WHERE customer_id = ? ORDER BY interaction_date DESC LIMIT 100",
    )
    .bind(&customer_id)
    .fetch_all(&state.db)
    .await
    .map_err(|e| (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(serde_json::json!({ "error": e.to_string() })),
    ))?;

    Ok(Json(interactions))
}

async fn create_interaction(
    State(state): State<AppState>,
    Path(customer_id): Path<String>,
    Json(req): Json<CreateInteractionHistoryRequest>,
) -> Result<Json<ClientInteractionHistory>, (StatusCode, Json<serde_json::Value>)> {
    let db = &state.db;
    let id = format!("interaction_{}", Uuid::new_v4());
    let now = chrono::Utc::now().to_rfc3339();

    sqlx::query(
        "INSERT INTO client_interaction_history (id, customer_id, interaction_type, channel, 
         direction, summary, sentiment, duration_seconds, interaction_date, metadata) 
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(&customer_id)
    .bind(&req.interaction_type)
    .bind(req.channel.as_deref().unwrap_or(""))
    .bind(req.direction.as_deref().unwrap_or("inbound"))
    .bind(req.summary.as_deref().unwrap_or(""))
    .bind(req.sentiment.as_deref().unwrap_or(""))
    .bind(req.duration_seconds.unwrap_or(0))
    .bind(&now)
    .bind(req.metadata.as_deref().unwrap_or("{}"))
    .execute(db)
    .await
    .map_err(|e| (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(serde_json::json!({ "error": e.to_string() })),
    ))?;

    let interaction = sqlx::query_as::<_, ClientInteractionHistory>(
        "SELECT * FROM client_interaction_history WHERE id = ?",
    )
    .bind(&id)
    .fetch_one(db)
    .await
    .map_err(|e| (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(serde_json::json!({ "error": e.to_string() })),
    ))?;

    Ok(Json(interaction))
}

async fn delete_interaction(
    State(state): State<AppState>,
    Path((_customer_id, interaction_id)): Path<(String, String)>,
) -> Result<StatusCode, (StatusCode, Json<serde_json::Value>)> {
    sqlx::query("DELETE FROM client_interaction_history WHERE id = ?")
        .bind(&interaction_id)
        .execute(&state.db)
        .await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        ))?;

    Ok(StatusCode::NO_CONTENT)
}

// ---------------------------------------------------------------------------
// Custom Sections
// ---------------------------------------------------------------------------

async fn list_custom_sections(
    State(state): State<AppState>,
    Path(customer_id): Path<String>,
) -> Result<Json<Vec<ClientCustomSection>>, (StatusCode, Json<serde_json::Value>)> {
    let sections = sqlx::query_as::<_, ClientCustomSection>(
        "SELECT * FROM client_custom_sections WHERE customer_id = ? AND is_active = 1 ORDER BY display_order",
    )
    .bind(&customer_id)
    .fetch_all(&state.db)
    .await
    .map_err(|e| (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(serde_json::json!({ "error": e.to_string() })),
    ))?;

    Ok(Json(sections))
}

async fn create_custom_section(
    State(state): State<AppState>,
    Path(customer_id): Path<String>,
    Json(req): Json<CreateCustomSectionRequest>,
) -> Result<Json<ClientCustomSection>, (StatusCode, Json<serde_json::Value>)> {
    let db = &state.db;
    let id = format!("section_{}", Uuid::new_v4());

    sqlx::query(
        "INSERT INTO client_custom_sections (id, customer_id, section_name, section_type, 
         section_source, display_order) 
         VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(&customer_id)
    .bind(&req.section_name)
    .bind(&req.section_type)
    .bind(req.section_source.as_deref().unwrap_or("user"))
    .bind(req.display_order.unwrap_or(0))
    .execute(db)
    .await
    .map_err(|e| (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(serde_json::json!({ "error": e.to_string() })),
    ))?;

    let section = sqlx::query_as::<_, ClientCustomSection>("SELECT * FROM client_custom_sections WHERE id = ?")
        .bind(&id)
        .fetch_one(db)
        .await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        ))?;

    Ok(Json(section))
}

async fn get_custom_section(
    State(state): State<AppState>,
    Path((_customer_id, section_id)): Path<(String, String)>,
) -> Result<Json<ClientCustomSection>, (StatusCode, Json<serde_json::Value>)> {
    let section = sqlx::query_as::<_, ClientCustomSection>(
        "SELECT * FROM client_custom_sections WHERE id = ?",
    )
    .bind(&section_id)
    .fetch_one(&state.db)
    .await
    .map_err(|_| (
        StatusCode::NOT_FOUND,
        Json(serde_json::json!({ "error": "Section not found" })),
    ))?;

    Ok(Json(section))
}

#[derive(Deserialize)]
pub struct UpdateCustomSectionRequest {
    pub section_name: Option<String>,
    pub is_active: Option<bool>,
}

async fn update_custom_section(
    State(state): State<AppState>,
    Path((_customer_id, section_id)): Path<(String, String)>,
    Json(req): Json<UpdateCustomSectionRequest>,
) -> Result<Json<ClientCustomSection>, (StatusCode, Json<serde_json::Value>)> {
    let db = &state.db;

    let mut sets = Vec::new();
    if let Some(ref name) = req.section_name {
        sets.push("section_name = ?");
    }
    if let Some(active) = req.is_active {
        sets.push("is_active = ?");
    }
    sets.push("updated_at = NOW()");

    if sets.is_empty() {
        return get_custom_section(State(state), Path((_customer_id, section_id))).await;
    }

    let query = format!("UPDATE client_custom_sections SET {} WHERE id = ?", sets.join(", "));
    sqlx::query(&query)
        .bind(&section_id)
        .execute(db)
        .await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        ))?;

    let section = sqlx::query_as::<_, ClientCustomSection>("SELECT * FROM client_custom_sections WHERE id = ?")
        .bind(&section_id)
        .fetch_one(db)
        .await
        .map_err(|_| (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "Section not found" })),
        ))?;

    Ok(Json(section))
}

async fn delete_custom_section(
    State(state): State<AppState>,
    Path((_customer_id, section_id)): Path<(String, String)>,
) -> Result<StatusCode, (StatusCode, Json<serde_json::Value>)> {
    sqlx::query("DELETE FROM client_custom_sections WHERE id = ?")
        .bind(&section_id)
        .execute(&state.db)
        .await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        ))?;

    Ok(StatusCode::NO_CONTENT)
}

// ---------------------------------------------------------------------------
// Custom Section Data
// ---------------------------------------------------------------------------

async fn list_custom_section_data(
    State(state): State<AppState>,
    Path((_customer_id, section_id)): Path<(String, String)>,
) -> Result<Json<Vec<ClientCustomSectionData>>, (StatusCode, Json<serde_json::Value>)> {
    let data = sqlx::query_as::<_, ClientCustomSectionData>(
        "SELECT * FROM client_custom_section_data WHERE section_id = ? ORDER BY display_order",
    )
    .bind(&section_id)
    .fetch_all(&state.db)
    .await
    .map_err(|e| (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(serde_json::json!({ "error": e.to_string() })),
    ))?;

    Ok(Json(data))
}

async fn create_custom_section_data(
    State(state): State<AppState>,
    Path((_customer_id, section_id)): Path<(String, String)>,
    Json(req): Json<CreateCustomSectionDataRequest>,
) -> Result<Json<ClientCustomSectionData>, (StatusCode, Json<serde_json::Value>)> {
    let db = &state.db;
    let id = format!("data_{}", Uuid::new_v4());

    sqlx::query(
        "INSERT INTO client_custom_section_data (id, section_id, row_key, field_key, field_value, 
         field_type, display_order) 
         VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(&section_id)
    .bind(req.row_key.as_deref().unwrap_or(""))
    .bind(&req.field_key)
    .bind(&req.field_value)
    .bind(req.field_type.as_deref().unwrap_or("text"))
    .bind(req.display_order.unwrap_or(0))
    .execute(db)
    .await
    .map_err(|e| (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(serde_json::json!({ "error": e.to_string() })),
    ))?;

    let data = sqlx::query_as::<_, ClientCustomSectionData>("SELECT * FROM client_custom_section_data WHERE id = ?")
        .bind(&id)
        .fetch_one(db)
        .await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        ))?;

    Ok(Json(data))
}

async fn update_custom_section_data(
    State(state): State<AppState>,
    Path((_customer_id, _section_id, data_id)): Path<(String, String, String)>,
    Json(req): Json<CreateCustomSectionDataRequest>,
) -> Result<Json<ClientCustomSectionData>, (StatusCode, Json<serde_json::Value>)> {
    let db = &state.db;

    sqlx::query(
        "UPDATE client_custom_section_data SET field_key = ?, field_value = ?, field_type = ?, 
         updated_at = NOW() WHERE id = ?",
    )
    .bind(&req.field_key)
    .bind(&req.field_value)
    .bind(req.field_type.as_deref().unwrap_or("text"))
    .bind(&data_id)
    .execute(db)
    .await
    .map_err(|e| (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(serde_json::json!({ "error": e.to_string() })),
    ))?;

    let data = sqlx::query_as::<_, ClientCustomSectionData>("SELECT * FROM client_custom_section_data WHERE id = ?")
        .bind(&data_id)
        .fetch_one(db)
        .await
        .map_err(|_| (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "Data not found" })),
        ))?;

    Ok(Json(data))
}

async fn delete_custom_section_data(
    State(state): State<AppState>,
    Path((_customer_id, _section_id, data_id)): Path<(String, String, String)>,
) -> Result<StatusCode, (StatusCode, Json<serde_json::Value>)> {
    sqlx::query("DELETE FROM client_custom_section_data WHERE id = ?")
        .bind(&data_id)
        .execute(&state.db)
        .await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        ))?;

    Ok(StatusCode::NO_CONTENT)
}

// ---------------------------------------------------------------------------
// Profile Sources
// ---------------------------------------------------------------------------

async fn list_profile_sources(
    State(state): State<AppState>,
    Path(customer_id): Path<String>,
) -> Result<Json<Vec<ClientProfileSource>>, (StatusCode, Json<serde_json::Value>)> {
    let sources = sqlx::query_as::<_, ClientProfileSource>(
        "SELECT * FROM client_profile_sources WHERE customer_id = ?",
    )
    .bind(&customer_id)
    .fetch_all(&state.db)
    .await
    .map_err(|e| (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(serde_json::json!({ "error": e.to_string() })),
    ))?;

    Ok(Json(sources))
}

async fn create_profile_source(
    State(state): State<AppState>,
    Path(customer_id): Path<String>,
    Json(req): Json<CreateProfileSourceRequest>,
) -> Result<Json<ClientProfileSource>, (StatusCode, Json<serde_json::Value>)> {
    let db = &state.db;
    let id = format!("source_{}", Uuid::new_v4());

    sqlx::query(
        "INSERT INTO client_profile_sources (id, customer_id, field_name, source_type, 
         source_user_name, source_record_id, source_url, confidence_score) 
         VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(&customer_id)
    .bind(&req.field_name)
    .bind(&req.source_type)
    .bind(req.source_user_name.as_deref().unwrap_or(""))
    .bind(req.source_record_id.as_deref().unwrap_or(""))
    .bind(req.source_url.as_deref().unwrap_or(""))
    .bind(req.confidence_score.unwrap_or(1.0))
    .execute(db)
    .await
    .map_err(|e| (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(serde_json::json!({ "error": e.to_string() })),
    ))?;

    let source = sqlx::query_as::<_, ClientProfileSource>("SELECT * FROM client_profile_sources WHERE id = ?")
        .bind(&id)
        .fetch_one(db)
        .await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        ))?;

    Ok(Json(source))
}

async fn delete_profile_source(
    State(state): State<AppState>,
    Path((_customer_id, source_id)): Path<(String, String)>,
) -> Result<StatusCode, (StatusCode, Json<serde_json::Value>)> {
    sqlx::query("DELETE FROM client_profile_sources WHERE id = ?")
        .bind(&source_id)
        .execute(&state.db)
        .await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        ))?;

    Ok(StatusCode::NO_CONTENT)
}

// ---------------------------------------------------------------------------
// RAG Mappings
// ---------------------------------------------------------------------------

async fn list_rag_mappings(
    State(state): State<AppState>,
    Path(customer_id): Path<String>,
) -> Result<Json<Vec<ClientRagMapping>>, (StatusCode, Json<serde_json::Value>)> {
    let mappings = sqlx::query_as::<_, ClientRagMapping>(
        "SELECT * FROM client_rag_mapping WHERE customer_id = ?",
    )
    .bind(&customer_id)
    .fetch_all(&state.db)
    .await
    .map_err(|e| (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(serde_json::json!({ "error": e.to_string() })),
    ))?;

    Ok(Json(mappings))
}

async fn create_rag_mapping(
    State(state): State<AppState>,
    Path(customer_id): Path<String>,
    Json(req): Json<CreateRagMappingRequest>,
) -> Result<Json<ClientRagMapping>, (StatusCode, Json<serde_json::Value>)> {
    let db = &state.db;
    let id = format!("rag_{}", Uuid::new_v4());

    sqlx::query(
        "INSERT INTO client_rag_mapping (id, customer_id, section_type, section_id, is_indexed) 
         VALUES (?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(&customer_id)
    .bind(&req.section_type)
    .bind(req.section_id.as_deref().unwrap_or(""))
    .bind(req.is_indexed.unwrap_or(true))
    .execute(db)
    .await
    .map_err(|e| (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(serde_json::json!({ "error": e.to_string() })),
    ))?;

    let mapping = sqlx::query_as::<_, ClientRagMapping>("SELECT * FROM client_rag_mapping WHERE id = ?")
        .bind(&id)
        .fetch_one(db)
        .await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        ))?;

    Ok(Json(mapping))
}

async fn delete_rag_mapping(
    State(state): State<AppState>,
    Path((_customer_id, mapping_id)): Path<(String, String)>,
) -> Result<StatusCode, (StatusCode, Json<serde_json::Value>)> {
    sqlx::query("DELETE FROM client_rag_mapping WHERE id = ?")
        .bind(&mapping_id)
        .execute(&state.db)
        .await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        ))?;

    Ok(StatusCode::NO_CONTENT)
}
