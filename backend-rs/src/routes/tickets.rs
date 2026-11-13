use crate::db::AppState;
use crate::services::messaging;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use chrono::Utc;
use serde::Deserialize;
use serde_json::{json, Value};
use sqlx::Row;
use uuid::Uuid;

#[derive(Deserialize, Debug, Default)]
pub struct TicketQueryParams {
    pub ticket_type: Option<String>,
    pub status: Option<String>,
    pub department: Option<String>,
    pub tracker_id: Option<String>,
    pub conversation_id: Option<String>,
    pub search: Option<String>,
}

fn format_ticket_row(row: &sqlx::postgres::PgRow) -> Value {
    // Option-typed reads: an absent or undecodable column reports null instead
    // of an empty string the UI would render as a real value.
    let id: Option<String> = row.try_get("id").ok().flatten();
    let ticket_number: Option<String> = row.try_get("ticket_number").ok().flatten();
    let title: Option<String> = row.try_get("title").ok().flatten();
    let description: Option<String> = row.try_get("description").ok().flatten();
    let ticket_type: Option<String> = row.try_get("ticket_type").ok().flatten();
    let department: Option<String> = row.try_get("department").ok().flatten();
    let priority: Option<String> = row.try_get("priority").ok().flatten();
    let status: Option<String> = row.try_get("status").ok().flatten();
    let conversation_id: Option<String> = row.try_get("conversation_id").ok().flatten();
    let contact_id: Option<String> = row.try_get("contact_id").ok().flatten();
    let contact_name: Option<String> = row.try_get("contact_name").ok().flatten();
    let contact_identifier: Option<String> = row.try_get("contact_identifier").ok().flatten();
    let tracker_id: Option<String> = row.try_get("tracker_id").ok().flatten();
    let tracker_number: Option<String> = row.try_get("tracker_number").ok().flatten();
    let tracker_title: Option<String> = row.try_get("tracker_title").ok().flatten();
    let assigned_to_id: Option<String> = row.try_get("assigned_to_id").ok().flatten();
    let assigned_to_name: Option<String> = row.try_get("assigned_to_name").ok().flatten();
    let resolution_summary: Option<String> = row.try_get("resolution_summary").ok().flatten();
    let customer_broadcast_sent: Option<bool> = row.try_get("customer_broadcast_sent").ok().flatten();
    let resolved_at: Option<String> = row.try_get("resolved_at").ok().flatten();
    let created_at: Option<String> = row.try_get("created_at").ok().flatten();
    let updated_at: Option<String> = row.try_get("updated_at").ok().flatten();
    // COUNT(*) subqueries always return an integer, so these are never unknown.
    let child_count: i64 = row.try_get("child_count").unwrap_or(0);
    let resolved_child_count: i64 = row.try_get("resolved_child_count").unwrap_or(0);

    let mut map = serde_json::Map::new();
    map.insert("id".to_string(), json!(id));
    map.insert("ticket_number".to_string(), json!(ticket_number));
    map.insert("ticketNumber".to_string(), json!(ticket_number));
    map.insert("title".to_string(), json!(title));
    map.insert("description".to_string(), json!(description));
    map.insert("ticket_type".to_string(), json!(ticket_type));
    map.insert("ticketType".to_string(), json!(ticket_type));
    map.insert("department".to_string(), json!(department));
    map.insert("priority".to_string(), json!(priority));
    map.insert("status".to_string(), json!(status));
    map.insert("conversation_id".to_string(), json!(conversation_id));
    map.insert("conversationId".to_string(), json!(conversation_id));
    map.insert("contact_id".to_string(), json!(contact_id));
    map.insert("contactId".to_string(), json!(contact_id));
    map.insert("contact_name".to_string(), json!(contact_name));
    map.insert("contactName".to_string(), json!(contact_name));
    map.insert("contact_identifier".to_string(), json!(contact_identifier));
    map.insert("contactIdentifier".to_string(), json!(contact_identifier));
    map.insert("tracker_id".to_string(), json!(tracker_id));
    map.insert("trackerId".to_string(), json!(tracker_id));
    map.insert("tracker_number".to_string(), json!(tracker_number));
    map.insert("trackerNumber".to_string(), json!(tracker_number));
    map.insert("tracker_title".to_string(), json!(tracker_title));
    map.insert("trackerTitle".to_string(), json!(tracker_title));
    map.insert("assigned_to_id".to_string(), json!(assigned_to_id));
    map.insert("assignedToId".to_string(), json!(assigned_to_id));
    map.insert("assigned_to_name".to_string(), json!(assigned_to_name));
    map.insert("assignedToName".to_string(), json!(assigned_to_name));
    map.insert("resolution_summary".to_string(), json!(resolution_summary));
    map.insert("resolutionSummary".to_string(), json!(resolution_summary));
    map.insert("customer_broadcast_sent".to_string(), json!(customer_broadcast_sent));
    map.insert("customerBroadcastSent".to_string(), json!(customer_broadcast_sent));
    map.insert("resolved_at".to_string(), json!(resolved_at));
    map.insert("resolvedAt".to_string(), json!(resolved_at));
    map.insert("created_at".to_string(), json!(created_at));
    map.insert("createdAt".to_string(), json!(created_at));
    map.insert("updated_at".to_string(), json!(updated_at));
    map.insert("updatedAt".to_string(), json!(updated_at));
    map.insert("child_count".to_string(), json!(child_count));
    map.insert("childCount".to_string(), json!(child_count));
    map.insert("resolved_child_count".to_string(), json!(resolved_child_count));
    map.insert("resolvedChildCount".to_string(), json!(resolved_child_count));

    Value::Object(map)
}

/// Ok(None) means "no such ticket"; Err means the read itself failed, so the
/// caller can tell a missing ticket apart from a broken database.
async fn fetch_full_ticket_by_id(db: &sqlx::PgPool, id: &str) -> Result<Option<Value>, sqlx::Error> {
    let sql = r#"
        SELECT
            t.id, t.ticket_number, t.title, t.description, t.ticket_type,
            t.department, t.priority, t.status, t.conversation_id, t.contact_id,
            c.person_name as contact_name, c.value as contact_identifier,
            t.tracker_id, p.ticket_number as tracker_number, p.title as tracker_title,
            t.assigned_to_id, tm.name as assigned_to_name,
            t.resolution_summary, t.customer_broadcast_sent, t.resolved_at,
            t.created_at, t.updated_at,
            (SELECT count(*) FROM tickets WHERE tracker_id = t.id) as child_count,
            (SELECT count(*) FROM tickets WHERE tracker_id = t.id AND status IN ('resolved', 'closed')) as resolved_child_count
        FROM tickets t
        LEFT JOIN contacts c ON t.contact_id = c.id
        LEFT JOIN tickets p ON t.tracker_id = p.id
        LEFT JOIN team_members tm ON t.assigned_to_id = tm.id
        WHERE t.id = ?
    "#;
    let row = match sqlx::query(sql).bind(id).fetch_optional(db).await {
        Ok(Some(r)) => r,
        Ok(None) => return Ok(None),
        Err(e) => return Err(e),
    };
    let mut val = format_ticket_row(&row);

    let act_rows = sqlx::query(
        "SELECT id, ticket_id, author_id, author_name, activity_type, message, activity_metadata, created_at FROM ticket_activities WHERE ticket_id = ? ORDER BY created_at ASC"
    )
    .bind(id)
    .fetch_all(db)
    .await?;

    let activities: Vec<Value> = act_rows.iter().map(|ar| {
        let act_id: Option<String> = ar.try_get("id").ok().flatten();
        let ticket_id: Option<String> = ar.try_get("ticket_id").ok().flatten();
        let author_id: Option<String> = ar.try_get("author_id").ok().flatten();
        // author_name is nullable: no stored author stays null rather than
        // becoming an invented "Agent".
        let author_name: Option<String> = ar.try_get("author_name").ok().flatten();
        let activity_type: Option<String> = ar.try_get("activity_type").ok().flatten();
        let message: Option<String> = ar.try_get("message").ok().flatten();
        let meta_str: Option<String> = ar.try_get("activity_metadata").ok().flatten();
        // Unparseable stored JSON is reported as unknown, not as an empty object.
        let metadata: Value = match meta_str {
            Some(s) => serde_json::from_str(&s).unwrap_or(Value::Null),
            None => Value::Null,
        };
        let created_at: Option<String> = ar.try_get("created_at").ok().flatten();

        json!({
            "id": act_id,
            "ticket_id": ticket_id,
            "ticketId": ticket_id,
            "author_id": author_id,
            "authorId": author_id,
            "author_name": author_name,
            "authorName": author_name,
            "activity_type": activity_type,
            "activityType": activity_type,
            "message": message,
            "metadata": metadata,
            "created_at": created_at,
            "createdAt": created_at
        })
    }).collect();

    // Fetch child tickets if tracker
    let child_rows = sqlx::query(
        r#"
        SELECT t.id, t.ticket_number, t.title, t.status, t.priority, t.department,
               t.conversation_id, t.contact_id, c.person_name as contact_name, t.created_at
        FROM tickets t
        LEFT JOIN contacts c ON t.contact_id = c.id
        WHERE t.tracker_id = ?
        ORDER BY t.created_at ASC
        "#
    )
    .bind(id)
    .fetch_all(db)
    .await?;

    let child_tickets: Vec<Value> = child_rows.iter().map(|cr| {
        let cid: Option<String> = cr.try_get("id").ok().flatten();
        let num: Option<String> = cr.try_get("ticket_number").ok().flatten();
        let title: Option<String> = cr.try_get("title").ok().flatten();
        let status: Option<String> = cr.try_get("status").ok().flatten();
        let priority: Option<String> = cr.try_get("priority").ok().flatten();
        let dept: Option<String> = cr.try_get("department").ok().flatten();
        let conv_id: Option<String> = cr.try_get("conversation_id").ok().flatten();
        let contact_id: Option<String> = cr.try_get("contact_id").ok().flatten();
        let contact_name: Option<String> = cr.try_get("contact_name").ok().flatten();
        let created_at: Option<String> = cr.try_get("created_at").ok().flatten();

        json!({
            "id": cid,
            "ticket_number": num,
            "ticketNumber": num,
            "title": title,
            "status": status,
            "priority": priority,
            "department": dept,
            "conversation_id": conv_id,
            "conversationId": conv_id,
            "contact_id": contact_id,
            "contactId": contact_id,
            "contact_name": contact_name,
            "contactName": contact_name,
            "created_at": created_at,
            "createdAt": created_at
        })
    }).collect();

    val["activities"] = json!(activities);
    val["child_tickets"] = json!(child_tickets);
    val["childTickets"] = json!(child_tickets);
    Ok(Some(val))
}

pub async fn list_tickets(
    State(state): State<AppState>,
    Query(query): Query<TicketQueryParams>,
) -> impl IntoResponse {
    let mut sql = String::from(r#"
        SELECT
            t.id, t.ticket_number, t.title, t.description, t.ticket_type,
            t.department, t.priority, t.status, t.conversation_id, t.contact_id,
            c.person_name as contact_name, c.value as contact_identifier,
            t.tracker_id, p.ticket_number as tracker_number, p.title as tracker_title,
            t.assigned_to_id, tm.name as assigned_to_name,
            t.resolution_summary, t.customer_broadcast_sent, t.resolved_at,
            t.created_at, t.updated_at,
            (SELECT count(*) FROM tickets WHERE tracker_id = t.id) as child_count,
            (SELECT count(*) FROM tickets WHERE tracker_id = t.id AND status IN ('resolved', 'closed')) as resolved_child_count
        FROM tickets t
        LEFT JOIN contacts c ON t.contact_id = c.id
        LEFT JOIN tickets p ON t.tracker_id = p.id
        LEFT JOIN team_members tm ON t.assigned_to_id = tm.id
        WHERE 1=1
    "#);

    if let Some(ref tt) = query.ticket_type {
        if !tt.is_empty() && tt != "all" {
            sql.push_str(" AND t.ticket_type = ?");
        }
    }
    if let Some(ref st) = query.status {
        if !st.is_empty() && st != "all" {
            if st == "resolved" {
                sql.push_str(" AND t.status IN ('resolved', 'closed')");
            } else {
                sql.push_str(" AND t.status = ?");
            }
        }
    }
    if let Some(ref dept) = query.department {
        if !dept.is_empty() && dept != "all" {
            sql.push_str(" AND t.department = ?");
        }
    }
    if let Some(ref tr_id) = query.tracker_id {
        if !tr_id.is_empty() {
            sql.push_str(" AND t.tracker_id = ?");
        }
    }
    if let Some(ref conv_id) = query.conversation_id {
        if !conv_id.is_empty() {
            sql.push_str(" AND t.conversation_id = ?");
        }
    }
    if let Some(ref s) = query.search {
        if !s.is_empty() {
            sql.push_str(" AND (t.ticket_number LIKE ? OR t.title LIKE ? OR t.description LIKE ? OR c.person_name LIKE ?)");
        }
    }
    sql.push_str(" ORDER BY t.created_at DESC");

    let mut q = sqlx::query(&sql);
    if let Some(ref tt) = query.ticket_type {
        if !tt.is_empty() && tt != "all" {
            q = q.bind(tt);
        }
    }
    if let Some(ref st) = query.status {
        if !st.is_empty() && st != "all" {
            if st != "resolved" {
                q = q.bind(st);
            }
        }
    }
    if let Some(ref dept) = query.department {
        if !dept.is_empty() && dept != "all" {
            q = q.bind(dept);
        }
    }
    if let Some(ref tr_id) = query.tracker_id {
        if !tr_id.is_empty() {
            q = q.bind(tr_id);
        }
    }
    if let Some(ref conv_id) = query.conversation_id {
        if !conv_id.is_empty() {
            q = q.bind(conv_id);
        }
    }
    if let Some(ref s) = query.search {
        if !s.is_empty() {
            let pat = format!("%{}%", s);
            q = q.bind(pat.clone()).bind(pat.clone()).bind(pat.clone()).bind(pat);
        }
    }

    let rows = match q.fetch_all(&state.db).await {
        Ok(r) => r,
        Err(e) => return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({
                "error": format!("cannot list tickets: {e}"),
                "note": "the ticket query failed; returning an empty list would look like a real empty queue",
            })),
        ).into_response(),
    };
    let tickets: Vec<Value> = rows.iter().map(format_ticket_row).collect();
    Json(tickets).into_response()
}

pub async fn get_ticket_stats(State(state): State<AppState>) -> impl IntoResponse {
    // Each aggregate is read live; a failed query aborts the whole response so a
    // read error never shows up as a believable zero count.
    let stats: Result<Value, sqlx::Error> = async {
        let total: i64 = sqlx::query_scalar("SELECT count(*) FROM tickets")
            .fetch_one(&state.db)
            .await?;

        let open: i64 = sqlx::query_scalar("SELECT count(*) FROM tickets WHERE status NOT IN ('resolved', 'closed')")
            .fetch_one(&state.db)
            .await?;

        let customer_issues: i64 = sqlx::query_scalar("SELECT count(*) FROM tickets WHERE ticket_type = 'customer_issue'")
            .fetch_one(&state.db)
            .await?;

        let back_office: i64 = sqlx::query_scalar("SELECT count(*) FROM tickets WHERE ticket_type = 'back_office'")
            .fetch_one(&state.db)
            .await?;

        let trackers: i64 = sqlx::query_scalar("SELECT count(*) FROM tickets WHERE ticket_type = 'tracker'")
            .fetch_one(&state.db)
            .await?;

        let resolved: i64 = sqlx::query_scalar("SELECT count(*) FROM tickets WHERE status IN ('resolved', 'closed')")
            .fetch_one(&state.db)
            .await?;

        let urgent: i64 = sqlx::query_scalar("SELECT count(*) FROM tickets WHERE priority = 'urgent' AND status NOT IN ('resolved', 'closed')")
            .fetch_one(&state.db)
            .await?;

        let impacted_customers: i64 = sqlx::query_scalar("SELECT count(DISTINCT contact_id) FROM tickets WHERE contact_id IS NOT NULL")
            .fetch_one(&state.db)
            .await?;

        let dept_rows = sqlx::query("SELECT department, count(*) as count FROM tickets GROUP BY department")
            .fetch_all(&state.db)
            .await?;

        let mut department_breakdown = serde_json::Map::new();
        for dr in dept_rows {
            let dept: Option<String> = dr.try_get("department").ok().flatten();
            let cnt: i64 = dr.try_get("count").unwrap_or(0);
            // A NULL department is a real bucket in the data; name it as such.
            department_breakdown.insert(dept.unwrap_or_else(|| "unassigned".to_string()), json!(cnt));
        }

        let avg_res: Option<f64> = sqlx::query_scalar(
            "SELECT AVG((julianday(resolved_at) - julianday(created_at)) * 24.0) FROM tickets WHERE resolved_at IS NOT NULL"
        )
        .fetch_one(&state.db)
        .await?;

        let total_activities: i64 = sqlx::query_scalar("SELECT count(*) FROM ticket_activities")
            .fetch_one(&state.db)
            .await?;

        // NULL when no ticket has been resolved: no history means no average, not 0h.
        let avg_res_hours: Option<f64> = avg_res.map(|h| (h * 10.0).round() / 10.0);

        Ok(json!({
            "total_tickets": total,
            "totalTickets": total,
            "open_tickets": open,
            "openTickets": open,
            "customer_issues": customer_issues,
            "customerIssues": customer_issues,
            "back_office_tickets": back_office,
            "backOfficeTickets": back_office,
            "tracker_tickets": trackers,
            "trackerTickets": trackers,
            "resolved_tickets": resolved,
            "resolvedTickets": resolved,
            "urgent_tickets": urgent,
            "urgentTickets": urgent,
            "impacted_customers_count": impacted_customers,
            "impactedCustomersCount": impacted_customers,
            "department_breakdown": department_breakdown,
            "departmentBreakdown": department_breakdown,
            "avg_resolution_time_hours": avg_res_hours,
            "avgResolutionTimeHours": avg_res_hours,
            "total_activities": total_activities,
            "totalActivities": total_activities
        }))
    }.await;

    match stats {
        Ok(v) => Json(v).into_response(),
        Err(e) => (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({
                "error": format!("cannot compute ticket stats: {e}"),
                "note": "a stats query failed; refusing to report zeros as if they were measured counts",
            })),
        ).into_response(),
    }
}

pub async fn get_trackers(State(state): State<AppState>) -> impl IntoResponse {
    let sql = r#"
        SELECT
            t.id, t.ticket_number, t.title, t.description, t.ticket_type,
            t.department, t.priority, t.status, t.conversation_id, t.contact_id,
            c.person_name as contact_name, c.value as contact_identifier,
            t.tracker_id, p.ticket_number as tracker_number, p.title as tracker_title,
            t.assigned_to_id, tm.name as assigned_to_name,
            t.resolution_summary, t.customer_broadcast_sent, t.resolved_at,
            t.created_at, t.updated_at,
            (SELECT count(*) FROM tickets WHERE tracker_id = t.id) as child_count,
            (SELECT count(*) FROM tickets WHERE tracker_id = t.id AND status IN ('resolved', 'closed')) as resolved_child_count
        FROM tickets t
        LEFT JOIN contacts c ON t.contact_id = c.id
        LEFT JOIN tickets p ON t.tracker_id = p.id
        LEFT JOIN team_members tm ON t.assigned_to_id = tm.id
        WHERE t.ticket_type = 'tracker'
        ORDER BY t.created_at DESC
    "#;

    let rows = match sqlx::query(sql).fetch_all(&state.db).await {
        Ok(r) => r,
        Err(e) => return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({
                "error": format!("cannot list trackers: {e}"),
                "note": "the tracker query failed; an empty array would look like a real empty tracker list",
            })),
        ).into_response(),
    };

    let trackers: Vec<Value> = rows.iter().map(format_ticket_row).collect();
    Json(trackers).into_response()
}

pub async fn get_ticket(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match fetch_full_ticket_by_id(&state.db, &id).await {
        Ok(Some(t)) => Json(t).into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, Json(json!({"error": "Ticket not found"}))).into_response(),
        Err(e) => (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({
                "error": format!("cannot read ticket {id}: {e}"),
                "note": "the read failed; this is not a missing ticket",
            })),
        ).into_response(),
    }
}

#[derive(Deserialize, Debug)]
pub struct CreateTicketPayload {
    pub title: String,
    pub description: String,
    pub ticket_type: Option<String>,
    pub department: Option<String>,
    pub priority: Option<String>,
    pub status: Option<String>,
    pub conversation_id: Option<String>,
    pub contact_id: Option<String>,
    pub tracker_id: Option<String>,
    pub assigned_to_id: Option<String>,
    pub author_name: Option<String>,
}

pub async fn create_ticket(
    State(state): State<AppState>,
    Json(payload): Json<CreateTicketPayload>,
) -> impl IntoResponse {
    let id = Uuid::new_v4().to_string();
    // The ticket number is derived from the live row count, so a failed count
    // must abort rather than silently mint a duplicate TCK-1001.
    let count: i64 = match sqlx::query_scalar("SELECT count(*) FROM tickets")
        .fetch_one(&state.db)
        .await
    {
        Ok(c) => c,
        Err(e) => return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({
                "error": format!("cannot count existing tickets: {e}"),
                "note": "no ticket was created; the ticket number cannot be minted without it",
            })),
        ).into_response(),
    };
    let ticket_num = format!("TCK-{}", 1000 + count + 1);
    let now = Utc::now().to_rfc3339();

    let tt = payload.ticket_type.unwrap_or_else(|| "customer_issue".to_string());
    let dept = payload.department.unwrap_or_else(|| "support".to_string());
    let prio = payload.priority.unwrap_or_else(|| "medium".to_string());
    let status = payload.status.unwrap_or_else(|| "open".to_string());
    // author_name is nullable: an unstored author stays null instead of being
    // attributed to a placeholder "Agent".
    let author: Option<String> = payload.author_name.filter(|s| !s.trim().is_empty());

    let inserted = match sqlx::query(
        r#"
        INSERT INTO tickets (
            id, ticket_number, title, description, ticket_type, department,
            priority, status, conversation_id, contact_id, tracker_id, assigned_to_id, created_at, updated_at
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(&id)
    .bind(&ticket_num)
    .bind(&payload.title)
    .bind(&payload.description)
    .bind(&tt)
    .bind(&dept)
    .bind(&prio)
    .bind(&status)
    .bind(&payload.conversation_id)
    .bind(&payload.contact_id)
    .bind(&payload.tracker_id)
    .bind(&payload.assigned_to_id)
    .bind(&now)
    .bind(&now)
    .execute(&state.db)
    .await
    {
        Ok(r) => r.rows_affected(),
        Err(e) => return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({"error": format!("ticket insert failed: {e}")})),
        ).into_response(),
    };

    if inserted == 0 {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": "ticket insert affected 0 rows",
                "ticketId": id,
                "note": "nothing was written, so no ticket was created",
            })),
        ).into_response();
    }

    // Log creation activity
    let act_id = Uuid::new_v4().to_string();
    let act_msg = format!("Ticket created ({}) in department {} with priority {}", tt, dept, prio);
    let mut activity_errors: Vec<String> = Vec::new();
    if let Err(e) = sqlx::query(
        "INSERT INTO ticket_activities (id, ticket_id, author_id, author_name, activity_type, message, activity_metadata, created_at) VALUES (?, ?, NULL, ?, 'creation', ?, '{}', ?)"
    )
    .bind(&act_id)
    .bind(&id)
    .bind(&author)
    .bind(&act_msg)
    .bind(&now)
    .execute(&state.db)
    .await
    {
        activity_errors.push(format!("creation activity not written: {e}"));
    }

    match fetch_full_ticket_by_id(&state.db, &id).await {
        Ok(Some(mut t)) => {
            t["activityErrors"] = json!(activity_errors);
            (StatusCode::CREATED, Json(t)).into_response()
        }
        Ok(None) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": "ticket insert reported success but the row cannot be read back",
                "ticketId": id,
            })),
        ).into_response(),
        Err(e) => (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({
                "error": format!("ticket was inserted but cannot be read back: {e}"),
                "ticketId": id,
            })),
        ).into_response(),
    }
}

pub async fn update_ticket(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    let now = Utc::now().to_rfc3339();

    // Check if ticket exists
    let existing = match sqlx::query("SELECT status, priority, department, title, description, tracker_id, assigned_to_id, resolution_summary FROM tickets WHERE id = ?")
        .bind(&id)
        .fetch_optional(&state.db)
        .await
    {
        Ok(Some(e)) => e,
        Ok(None) => return (StatusCode::NOT_FOUND, Json(json!({"error": "Ticket not found", "ticketId": id}))).into_response(),
        Err(e) => return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({
                "error": format!("cannot read ticket {id}: {e}"),
                "note": "the current values could not be read, so nothing was patched",
            })),
        ).into_response(),
    };

    let old_status: Option<String> = existing.try_get("status").ok().flatten();
    let old_priority: Option<String> = existing.try_get("priority").ok().flatten();
    let old_department: Option<String> = existing.try_get("department").ok().flatten();
    let old_title: Option<String> = existing.try_get("title").ok().flatten();
    let old_description: Option<String> = existing.try_get("description").ok().flatten();
    let old_tracker_id: Option<String> = existing.try_get("tracker_id").ok().flatten();
    let old_assigned_to_id: Option<String> = existing.try_get("assigned_to_id").ok().flatten();
    let old_resolution: Option<String> = existing.try_get("resolution_summary").ok().flatten();

    let new_status = payload.get("status").and_then(|v| v.as_str()).or(old_status.as_deref());
    let new_priority = payload.get("priority").and_then(|v| v.as_str()).or(old_priority.as_deref());
    let new_department = payload.get("department").and_then(|v| v.as_str()).or(old_department.as_deref());
    let new_title = payload.get("title").and_then(|v| v.as_str()).or(old_title.as_deref());
    let new_description = payload.get("description").and_then(|v| v.as_str()).or(old_description.as_deref());
    let new_tracker_id = if payload.get("tracker_id").is_some() {
        payload.get("tracker_id").and_then(|v| v.as_str().map(|s| s.to_string()))
    } else {
        old_tracker_id
    };
    let new_assigned_to_id = if payload.get("assigned_to_id").is_some() {
        payload.get("assigned_to_id").and_then(|v| v.as_str().map(|s| s.to_string()))
    } else {
        old_assigned_to_id
    };
    let new_resolution = if payload.get("resolution_summary").is_some() {
        payload.get("resolution_summary").and_then(|v| v.as_str().map(|s| s.to_string()))
    } else {
        old_resolution
    };

    // NOT NULL columns: a value neither supplied nor readable means we cannot
    // write the row honestly.
    let missing: Vec<&str> = [("status", new_status), ("priority", new_priority), ("department", new_department), ("title", new_title), ("description", new_description)]
        .iter().filter(|(_, v)| v.is_none()).map(|(k, _)| *k).collect();
    if !missing.is_empty() {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(json!({
                "error": format!("cannot resolve required field(s): {}", missing.join(", ")),
                "ticketId": id,
                "note": "neither present in the patch nor readable from the stored row; no update was written",
            })),
        ).into_response();
    }

    let resolved_at = if matches!(new_status, Some("resolved") | Some("closed"))
        && !matches!(old_status.as_deref(), Some("resolved") | Some("closed"))
    {
        Some(now.clone())
    } else {
        None
    };

    let updated = match sqlx::query(
        r#"
        UPDATE tickets
        SET status = ?, priority = ?, department = ?, title = ?, description = ?,
            tracker_id = ?, assigned_to_id = ?, resolution_summary = ?,
            resolved_at = COALESCE(?, resolved_at), updated_at = ?
        WHERE id = ?
        "#
    )
    .bind(new_status)
    .bind(new_priority)
    .bind(new_department)
    .bind(new_title)
    .bind(new_description)
    .bind(&new_tracker_id)
    .bind(&new_assigned_to_id)
    .bind(&new_resolution)
    .bind(&resolved_at)
    .bind(&now)
    .bind(&id)
    .execute(&state.db)
    .await
    {
        Ok(r) => r.rows_affected(),
        Err(e) => return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({"error": format!("ticket update failed: {e}"), "ticketId": id})),
        ).into_response(),
    };

    if updated == 0 {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": "ticket update affected 0 rows",
                "ticketId": id,
                "note": "the ticket existed at read time but no row was changed",
            })),
        ).into_response();
    }

    // Log activities for status or priority change
    let author = payload.get("author_name").and_then(|v| v.as_str()).filter(|s| !s.trim().is_empty());
    let mut activity_errors: Vec<String> = Vec::new();
    if new_status != old_status.as_deref() {
        let act_id = Uuid::new_v4().to_string();
        let msg = match &old_status {
            Some(prev) => format!("Status updated from '{prev}' to '{}'", new_status.unwrap_or("null")),
            None => format!("Status set to '{}'; previous value was unreadable", new_status.unwrap_or("null")),
        };
        if let Err(e) = sqlx::query(
            "INSERT INTO ticket_activities (id, ticket_id, author_id, author_name, activity_type, message, activity_metadata, created_at) VALUES (?, ?, NULL, ?, 'status_change', ?, '{}', ?)"
        )
        .bind(&act_id)
        .bind(&id)
        .bind(author)
        .bind(&msg)
        .bind(&now)
        .execute(&state.db)
        .await
        {
            activity_errors.push(format!("status change activity not written: {e}"));
        }
    }
    if new_priority != old_priority.as_deref() {
        let act_id = Uuid::new_v4().to_string();
        let msg = match &old_priority {
            Some(prev) => format!("Priority changed from '{prev}' to '{}'", new_priority.unwrap_or("null")),
            None => format!("Priority set to '{}'; previous value was unreadable", new_priority.unwrap_or("null")),
        };
        if let Err(e) = sqlx::query(
            "INSERT INTO ticket_activities (id, ticket_id, author_id, author_name, activity_type, message, activity_metadata, created_at) VALUES (?, ?, NULL, ?, 'priority_change', ?, '{}', ?)"
        )
        .bind(&act_id)
        .bind(&id)
        .bind(author)
        .bind(&msg)
        .bind(&now)
        .execute(&state.db)
        .await
        {
            activity_errors.push(format!("priority change activity not written: {e}"));
        }
    }

    match fetch_full_ticket_by_id(&state.db, &id).await {
        Ok(Some(mut t)) => {
            t["activityErrors"] = json!(activity_errors);
            Json(t).into_response()
        }
        Ok(None) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": "ticket update affected 0 rows", "ticketId": id})),
        ).into_response(),
        Err(e) => (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({"error": format!("ticket updated but cannot be read back: {e}"), "ticketId": id})),
        ).into_response(),
    }
}

#[derive(Deserialize, Debug)]
pub struct AddActivityPayload {
    pub message: String,
    pub activity_type: Option<String>,
    pub author_id: Option<String>,
    pub author_name: Option<String>,
    pub metadata: Option<Value>,
}

pub async fn add_ticket_activity(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<AddActivityPayload>,
) -> impl IntoResponse {
    let act_id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();
    // Nullable column: an unstated author stays null rather than being credited
    // to a placeholder "Agent".
    let author: Option<String> = payload.author_name.filter(|s| !s.trim().is_empty());
    let act_type = payload.activity_type.unwrap_or_else(|| "comment".to_string());
    let metadata: Value = payload.metadata.clone().unwrap_or(Value::Null);
    let meta_str: Option<String> = match &metadata {
        Value::Null => None,
        other => Some(other.to_string()),
    };

    // The activity row has no FK enforcement, so the ticket is checked first:
    // otherwise an activity for a missing ticket would be reported as added.
    let ticket_exists = match sqlx::query_scalar::<_, i64>("SELECT count(*) FROM tickets WHERE id = ?")
        .bind(&id)
        .fetch_one(&state.db)
        .await
    {
        Ok(c) => c > 0,
        Err(e) => return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({"error": format!("cannot verify ticket {id}: {e}"), "note": "no activity was written"})),
        ).into_response(),
    };
    if !ticket_exists {
        return (
            StatusCode::NOT_FOUND,
            Json(json!({"error": "Ticket not found", "ticketId": id, "note": "no activity was written"})),
        ).into_response();
    }

    let inserted = match sqlx::query(
        "INSERT INTO ticket_activities (id, ticket_id, author_id, author_name, activity_type, message, activity_metadata, created_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&act_id)
    .bind(&id)
    .bind(&payload.author_id)
    .bind(&author)
    .bind(&act_type)
    .bind(&payload.message)
    .bind(&meta_str)
    .bind(&now)
    .execute(&state.db)
    .await
    {
        Ok(r) => r.rows_affected(),
        Err(e) => return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({"error": format!("activity insert failed: {e}"), "ticketId": id})),
        ).into_response(),
    };

    if inserted == 0 {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": "activity insert affected 0 rows",
                "ticketId": id,
                "note": "nothing was written, so the activity does not exist",
            })),
        ).into_response();
    }

    // The touch is cosmetic: report a failure instead of swallowing it.
    let mut touch_errors: Vec<String> = Vec::new();
    match sqlx::query("UPDATE tickets SET updated_at = ? WHERE id = ?")
        .bind(&now)
        .bind(&id)
        .execute(&state.db)
        .await
    {
        Ok(r) if r.rows_affected() == 0 => touch_errors.push("ticket touch affected 0 rows".to_string()),
        Err(e) => touch_errors.push(format!("could not refresh updated_at: {e}")),
        Ok(_) => {}
    }

    Json(json!({
        "id": act_id,
        "ticket_id": id,
        "ticketId": id,
        "author_id": payload.author_id,
        "authorId": payload.author_id,
        "author_name": author,
        "authorName": author,
        "activity_type": act_type,
        "activityType": act_type,
        "message": payload.message,
        "metadata": metadata,
        "created_at": now,
        "createdAt": now,
        "activity_errors": touch_errors,
        "activityErrors": touch_errors
    })).into_response()
}

#[derive(Deserialize, Debug)]
pub struct LinkTrackerPayload {
    pub tracker_id: String,
    pub author_name: Option<String>,
}

pub async fn link_ticket_to_tracker(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<LinkTrackerPayload>,
) -> impl IntoResponse {
    let now = Utc::now().to_rfc3339();
    let author: Option<String> = payload.author_name.filter(|s| !s.trim().is_empty());

    let tracker_info = match sqlx::query("SELECT ticket_number, title FROM tickets WHERE id = ?")
        .bind(&payload.tracker_id)
        .fetch_optional(&state.db)
        .await
    {
        Ok(Some(row)) => row,
        // A tracker that does not exist has no number and no title; inventing
        // "Unknown"/"Tracker Incident" would put a fake incident in the log.
        Ok(None) => return (
            StatusCode::NOT_FOUND,
            Json(json!({
                "error": format!("tracker ticket {} does not exist", payload.tracker_id),
                "note": "nothing was linked; create the tracker ticket first",
            })),
        ).into_response(),
        Err(e) => return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({"error": format!("cannot read tracker {}: {e}", payload.tracker_id), "note": "nothing was linked"})),
        ).into_response(),
    };

    let tr_num: Option<String> = tracker_info.try_get("ticket_number").ok().flatten();
    let tr_title: Option<String> = tracker_info.try_get("title").ok().flatten();

    let linked = match sqlx::query("UPDATE tickets SET tracker_id = ?, updated_at = ? WHERE id = ?")
        .bind(&payload.tracker_id)
        .bind(&now)
        .bind(&id)
        .execute(&state.db)
        .await
    {
        Ok(r) => r.rows_affected(),
        Err(e) => return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({"error": format!("could not link ticket {id}: {e}"), "note": "the tracker link was not saved"})),
        ).into_response(),
    };

    if linked == 0 {
        return (
            StatusCode::NOT_FOUND,
            Json(json!({
                "error": "Ticket not found",
                "ticketId": id,
                "note": "the update matched 0 rows, so no tracker link was written",
            })),
        ).into_response();
    }

    let act_id = Uuid::new_v4().to_string();
    let msg = format!(
        "Linked to tracker incident {} ({})",
        tr_num.unwrap_or_else(|| "unknown number".to_string()),
        tr_title.unwrap_or_else(|| "unknown title".to_string())
    );
    let mut activity_errors: Vec<String> = Vec::new();
    if let Err(e) = sqlx::query(
        "INSERT INTO ticket_activities (id, ticket_id, author_id, author_name, activity_type, message, activity_metadata, created_at) VALUES (?, ?, NULL, ?, 'tracker_link', ?, '{}', ?)"
    )
    .bind(&act_id)
    .bind(&id)
    .bind(&author)
    .bind(&msg)
    .bind(&now)
    .execute(&state.db)
    .await
    {
        activity_errors.push(format!("tracker link activity not written: {e}"));
    }

    match fetch_full_ticket_by_id(&state.db, &id).await {
        Ok(Some(mut t)) => {
            t["activityErrors"] = json!(activity_errors);
            Json(t).into_response()
        }
        Ok(None) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": "ticket link applied but the ticket cannot be read back", "ticketId": id})),
        ).into_response(),
        Err(e) => (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({"error": format!("ticket linked but cannot be read back: {e}"), "ticketId": id})),
        ).into_response(),
    }
}

#[derive(Deserialize, Debug)]
pub struct BroadcastUpdatePayload {
    pub broadcast_message: String,
    pub resolve_all_children: Option<bool>,
    pub send_customer_message: Option<bool>,
    pub author_name: Option<String>,
}

pub async fn broadcast_tracker_update(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<BroadcastUpdatePayload>,
) -> impl IntoResponse {
    let now = Utc::now().to_rfc3339();
    let author: Option<String> = payload.author_name.filter(|s| !s.trim().is_empty());
    // An omitted flag must never message customers or close tickets on its own:
    // both actions are outward-facing and irreversible, so they happen only when
    // the request asked for them.
    let resolve_all = payload.resolve_all_children.unwrap_or(false);
    let send_customer = payload.send_customer_message.unwrap_or(false);
    let message_text = payload.broadcast_message.trim();

    let tracker_row = match sqlx::query("SELECT ticket_number, title FROM tickets WHERE id = ?")
        .bind(&id)
        .fetch_optional(&state.db)
        .await
    {
        Ok(Some(r)) => r,
        Ok(None) => return (
            StatusCode::NOT_FOUND,
            Json(json!({
                "error": format!("tracker ticket {id} does not exist"),
                "note": "no broadcast was dispatched and nothing was resolved",
            })),
        ).into_response(),
        Err(e) => return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({"error": format!("cannot read tracker {id}: {e}"), "note": "no broadcast was dispatched"})),
        ).into_response(),
    };

    let tracker_num: Option<String> = tracker_row.try_get("ticket_number").ok().flatten();
    // No stored number is labelled by id, never by an invented TRK- number.
    let tracker_num_label = tracker_num.clone().unwrap_or_else(|| format!("id {id}"));

    // An empty broadcast body would be stored against every linked conversation
    // and sent to every customer as nothing, so refuse before any write.
    if message_text.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "broadcast_message is empty: no message can be sent and no ticket was resolved",
                "trackerId": id,
                "tracker_id": id,
                "note": "nothing was written",
            })),
        )
            .into_response();
    }

    let child_rows = match sqlx::query("SELECT id, ticket_number, conversation_id, status FROM tickets WHERE tracker_id = ?")
        .bind(&id)
        .fetch_all(&state.db)
        .await
    {
        Ok(r) => r,
        Err(e) => return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({"error": format!("cannot list tickets linked to {id}: {e}"), "note": "no broadcast was dispatched"})),
        ).into_response(),
    };

    let impacted_tickets = child_rows.len();
    let mut messages_sent = 0usize;
    let mut messages_failed = 0usize;
    let mut messages_not_sent = 0usize;
    let mut resolved_tickets = 0usize;
    let mut already_resolved = 0usize;
    let mut resolve_failed = 0usize;
    let mut activity_errors: Vec<String> = Vec::new();
    let mut deliveries: Vec<Value> = Vec::new();

    for row in &child_rows {
        let child_id: Option<String> = row.try_get("id").ok().flatten();
        let conv_id: Option<String> = row.try_get("conversation_id").ok().flatten();
        let child_status: Option<String> = row.try_get("status").ok().flatten();

        if send_customer {
            let cid = conv_id.filter(|v| !v.trim().is_empty());
            match cid {
                None => {
                    messages_not_sent += 1;
                    deliveries.push(json!({
                        "ticketId": child_id.as_deref(),
                        "ticket_id": child_id.as_deref(),
                        "conversationId": Value::Null,
                        "status": "not_sent",
                        "detail": "the linked ticket row stores no conversation_id, so there is no \
                                  customer to reach and nothing to attach a message to",
                    }));
                }
                Some(cid) => {
                    // The channel the customer wrote in on, the contact the consent
                    // gate checks, and the address to dial all come from the
                    // conversation's own row. A stored message cannot be claimed as
                    // delivered by writing a row -- it needs the provider's answer.
                    let route = sqlx::query(
                        "SELECT COALESCE(c.channel, '') AS channel, COALESCE(c.contact_id, '') AS contact_id, \
                         ct.value AS contact_value, c.tenant_key AS tenant_key \
                         FROM conversations c LEFT JOIN contacts ct ON ct.id = c.contact_id WHERE c.id = ?",
                    )
                    .bind(&cid)
                    .fetch_optional(&state.db)
                    .await;

                    let route = match route {
                        Ok(Some(row)) => {
                            let channel: String = row.try_get("channel").unwrap_or_default();
                            let contact_id: String = row.try_get("contact_id").unwrap_or_default();
                            let contact_value: Option<String> =
                                row.try_get("contact_value").ok().flatten();
                            let tenant_key: Option<String> =
                                row.try_get("tenant_key").ok().flatten();
                            Some((channel, contact_id, contact_value, tenant_key))
                        }
                        Ok(None) => {
                            messages_not_sent += 1;
                            deliveries.push(json!({
                                "ticketId": child_id.as_deref(),
                                "ticket_id": child_id.as_deref(),
                                "conversationId": cid,
                                "status": "not_sent",
                                "detail": format!(
                                    "conversation {cid} linked to this ticket does not exist, so no \
                                     channel or address could be resolved"
                                ),
                            }));
                            None
                        }
                        Err(err) => {
                            messages_failed += 1;
                            deliveries.push(json!({
                                "ticketId": child_id.as_deref(),
                                "ticket_id": child_id.as_deref(),
                                "conversationId": cid,
                                "status": "error",
                                "detail": format!("conversation {cid} could not be read: {err}"),
                            }));
                            None
                        }
                    };

                    if let Some((channel, contact_id, contact_value, tenant_key)) = route {
                        let address = contact_value
                            .map(|v| v.trim().to_string())
                            .filter(|v| !v.is_empty())
                            .unwrap_or_default();
                        if channel.trim().is_empty()
                            || contact_id.trim().is_empty()
                            || address.is_empty()
                        {
                            let mut problems: Vec<&str> = Vec::new();
                            if channel.trim().is_empty() {
                                problems.push("the conversation row stores no channel, so no transport can be chosen");
                            }
                            if contact_id.trim().is_empty() {
                                problems.push("the conversation row stores no contact_id, so the consent gate cannot be evaluated");
                            }
                            if address.is_empty() {
                                problems.push("the linked contact row stores no addressable value (phone number or email)");
                            }
                            messages_not_sent += 1;
                            deliveries.push(json!({
                                "ticketId": child_id.as_deref(),
                                "ticket_id": child_id.as_deref(),
                                "conversationId": cid,
                                "status": "not_sent",
                                "channel": channel,
                                "contactId": contact_id,
                                "detail": problems.join("; "),
                                "note": "no provider was contacted and no message row was written",
                            }));
                        } else {
                            let request = messaging::OutboundRequest::new(
                                &cid,
                                &contact_id,
                                &address,
                                &channel,
                                message_text,
                            )
                            .tenant_key(tenant_key.as_deref());

                            match messaging::send_outbound(&state, &request).await {
                                Ok(outcome) => {
                                    let status = outcome.result.message_status();
                                    if outcome.was_sent() {
                                        messages_sent += 1;
                                    } else if outcome.result.status == messaging::SendStatus::Rejected
                                    {
                                        messages_failed += 1;
                                    } else {
                                        messages_not_sent += 1;
                                    }
                                    deliveries.push(json!({
                                        "ticketId": child_id.as_deref(),
                                        "ticket_id": child_id.as_deref(),
                                        "conversationId": outcome.conversation_id,
                                        "conversation_id": outcome.conversation_id,
                                        "messageId": outcome.message_id,
                                        "message_id": outcome.message_id,
                                        "status": status,
                                        "channel": channel,
                                        "provider": outcome.result.provider,
                                        "httpStatus": outcome.result.http_status,
                                        "providerMessageId": if outcome.result.provider_message_id.is_empty() {
                                            Value::Null
                                        } else {
                                            json!(outcome.result.provider_message_id)
                                        },
                                        "simulated": outcome.result.simulated,
                                        "detail": if outcome.was_sent() {
                                            Value::Null
                                        } else {
                                            json!(outcome.result.reason())
                                        },
                                    }));
                                }
                                Err(err) => {
                                    messages_failed += 1;
                                    deliveries.push(json!({
                                        "ticketId": child_id.as_deref(),
                                        "ticket_id": child_id.as_deref(),
                                        "conversationId": cid,
                                        "status": "error",
                                        "channel": channel,
                                        "detail": format!(
                                            "the outbound send did not complete: {err}"
                                        ),
                                    }));
                                }
                            }
                        }
                    }
                }
            }
        }

        if resolve_all {
            if let Some(ref child_id) = child_id {
                let res_summary = format!("Auto-resolved via incident tracker {} broadcast: {}", tracker_num_label, payload.broadcast_message);
                let res = sqlx::query("UPDATE tickets SET status = 'resolved', resolution_summary = ?, resolved_at = ?, updated_at = ? WHERE id = ?")
                    .bind(&res_summary)
                    .bind(&now)
                    .bind(&now)
                    .bind(child_id)
                    .execute(&state.db)
                    .await;

                match res {
                    // Only a row this request actually changed counts.
                    Ok(r) if r.rows_affected() == 0 => resolve_failed += 1,
                    Ok(_) if child_status.as_deref() == Some("resolved") => already_resolved += 1,
                    Ok(_) => resolved_tickets += 1,
                    Err(_) => resolve_failed += 1,
                }

                let act_id = Uuid::new_v4().to_string();
                let act_msg = format!("Resolved via broadcast from {}: {}", tracker_num_label, payload.broadcast_message);
                if let Err(e) = sqlx::query(
                    "INSERT INTO ticket_activities (id, ticket_id, author_id, author_name, activity_type, message, activity_metadata, created_at) VALUES (?, ?, NULL, ?, 'broadcast', ?, '{}', ?)"
                )
                .bind(&act_id)
                .bind(child_id)
                .bind(&author)
                .bind(&act_msg)
                .bind(&now)
                .execute(&state.db)
                .await
                {
                    activity_errors.push(format!("broadcast activity for ticket {child_id} not written: {e}"));
                }
            } else {
                activity_errors.push("a linked ticket row had no readable id and was skipped".to_string());
            }
        }
    }

    // The stored summary says what the provider accepted, not how many tickets
    // were touched -- "dispatched to N linked tickets" would read as delivered.
    let tracker_summary = format!(
        "Incident broadcast for {} linked tickets: {} accepted by their provider, {} refused, {} not \
         sent. Message: {}",
        impacted_tickets, messages_sent, messages_failed, messages_not_sent, payload.broadcast_message
    );
    // customer_broadcast_sent is only set when a customer message really landed.
    let customers_notified = send_customer && messages_sent > 0;
    let tracker_update = sqlx::query(
        "UPDATE tickets SET status = 'resolved', resolution_summary = ?, customer_broadcast_sent = ?, resolved_at = ?, updated_at = ? WHERE id = ?"
    )
    .bind(&tracker_summary)
    .bind(customers_notified)
    .bind(&now)
    .bind(&now)
    .bind(&id)
    .execute(&state.db)
    .await;

    let (tracker_resolved, tracker_resolve_error) = match tracker_update {
        Ok(r) if r.rows_affected() > 0 => (true, None),
        Ok(_) => (false, Some("the tracker row matched 0 updates".to_string())),
        Err(e) => (false, Some(format!("{e}"))),
    };

    let act_id = Uuid::new_v4().to_string();
    let act_msg = format!(
        "Incident broadcast \"{}\": {} customer message(s) accepted by a provider, {} refused, {} not \
         sent, {} ticket(s) resolved by this request.",
        payload.broadcast_message, messages_sent, messages_failed, messages_not_sent, resolved_tickets
    );
    // The per-recipient evidence travels with the activity row, so a later read
    // can tell an accepted send from a stored-but-undelivered one.
    let deliveries_raw = match serde_json::to_string(&deliveries) {
        Ok(raw) => raw,
        Err(err) => {
            activity_errors.push(format!(
                "the delivery detail could not be serialised into the activity row ({err}); the \
                 activity is written without it"
            ));
            "[]".to_string()
        }
    };
    if let Err(e) = sqlx::query(
        "INSERT INTO ticket_activities (id, ticket_id, author_id, author_name, activity_type, message, activity_metadata, created_at) VALUES (?, ?, NULL, ?, 'broadcast', ?, ?, ?)"
    )
    .bind(&act_id)
    .bind(&id)
    .bind(&author)
    .bind(&act_msg)
    .bind(&deliveries_raw)
    .bind(&now)
    .execute(&state.db)
    .await
    {
        activity_errors.push(format!("tracker broadcast activity not written: {e}"));
    }

    let customer_note = if !send_customer {
        "send_customer_message was not requested, so no customer was messaged"
    } else if messages_sent == 0 {
        "no customer message was accepted by a provider; see deliveries for each reason"
    } else {
        "only the deliveries listed as sent were accepted by a provider"
    };

    Json(json!({
        "tracker_id": id,
        "trackerId": id,
        "tracker_number": tracker_num,
        "trackerNumber": tracker_num,
        "impacted_tickets": impacted_tickets,
        "impactedTickets": impacted_tickets,
        "messages_sent": messages_sent,
        "messagesSent": messages_sent,
        "messages_failed": messages_failed,
        "messagesFailed": messages_failed,
        "messages_not_sent": messages_not_sent,
        "messagesNotSent": messages_not_sent,
        "deliveries": deliveries,
        "customer_message_requested": send_customer,
        "customerMessageRequested": send_customer,
        "resolve_all_requested": resolve_all,
        "resolveAllRequested": resolve_all,
        "resolved_tickets": resolved_tickets,
        "resolvedTickets": resolved_tickets,
        "already_resolved": already_resolved,
        "alreadyResolved": already_resolved,
        "resolve_failed": resolve_failed,
        "resolveFailed": resolve_failed,
        "customers_notified": customers_notified,
        "customersNotified": customers_notified,
        "tracker_resolved": tracker_resolved,
        "trackerResolved": tracker_resolved,
        "tracker_resolve_error": tracker_resolve_error,
        "trackerResolveError": tracker_resolve_error,
        "activity_errors": activity_errors,
        "activityErrors": activity_errors,
        "note": if tracker_resolved {
            format!("counts report only what this request actually wrote; {customer_note}")
        } else {
            format!(
                "the tracker itself was not marked resolved; see tracker_resolve_error. {customer_note}"
            )
        }
    })).into_response()
}

// Linked Issues
#[derive(Deserialize, Debug, Default)]
pub struct LinkedIssuesQuery {
    pub conversation_id: Option<String>,
    pub ticket_id: Option<String>,
}

fn format_linked_issue_row(row: &sqlx::postgres::PgRow) -> Value {
    // Option-typed reads: an unreadable column reports null instead of an empty
    // string or a defaulted "synced" that the UI would show as a real state.
    let id: Option<String> = row.try_get("id").ok().flatten();
    let conversation_id: Option<String> = row.try_get("conversation_id").ok().flatten();
    let ticket_id: Option<String> = row.try_get("ticket_id").ok().flatten();
    let provider: Option<String> = row.try_get("provider").ok().flatten();
    let external_issue_id: Option<String> = row.try_get("external_issue_id").ok().flatten();
    let issue_key: Option<String> = row.try_get("issue_key").ok().flatten();
    let issue_url: Option<String> = row.try_get("issue_url").ok().flatten();
    let title: Option<String> = row.try_get("title").ok().flatten();
    let status: Option<String> = row.try_get("status").ok().flatten();
    let priority: Option<String> = row.try_get("priority").ok().flatten();
    let assigned_to: Option<String> = row.try_get("assigned_to").ok().flatten();
    let created_by_email: Option<String> = row.try_get("created_by_email").ok().flatten();
    let sync_status: Option<String> = row.try_get("sync_status").ok().flatten();
    let meta_str: Option<String> = row.try_get("metadata_json").ok().flatten();
    let metadata: Value = match meta_str {
        Some(s) => serde_json::from_str(&s).unwrap_or(Value::Null),
        None => Value::Null,
    };
    let created_at: Option<String> = row.try_get("created_at").ok().flatten();
    let updated_at: Option<String> = row.try_get("updated_at").ok().flatten();

    json!({
        "id": id,
        "conversation_id": conversation_id,
        "conversationId": conversation_id,
        "ticket_id": ticket_id,
        "ticketId": ticket_id,
        "provider": provider,
        "external_issue_id": external_issue_id,
        "externalIssueId": external_issue_id,
        "issue_key": issue_key,
        "issueKey": issue_key,
        "issue_url": issue_url,
        "issueUrl": issue_url,
        "url": issue_url,
        "title": title,
        "status": status,
        "priority": priority,
        "assigned_to": assigned_to,
        "assignedTo": assigned_to,
        "created_by_email": created_by_email,
        "createdByEmail": created_by_email,
        "sync_status": sync_status,
        "syncStatus": sync_status,
        "metadata_json": metadata,
        "created_at": created_at,
        "createdAt": created_at,
        "updated_at": updated_at,
        "updatedAt": updated_at
    })
}

pub async fn list_linked_issues(
    State(state): State<AppState>,
    Query(query): Query<LinkedIssuesQuery>,
) -> impl IntoResponse {
    let mut sql = String::from("SELECT id, conversation_id, ticket_id, provider, external_issue_id, issue_key, issue_url, title, status, priority, assigned_to, created_by_email, sync_status, metadata_json, created_at, updated_at FROM linked_issues WHERE 1=1");

    if let Some(ref tid) = query.ticket_id {
        if !tid.is_empty() {
            sql.push_str(" AND ticket_id = ?");
        }
    } else if let Some(ref cid) = query.conversation_id {
        if !cid.is_empty() {
            sql.push_str(" AND conversation_id = ?");
        }
    }
    sql.push_str(" ORDER BY created_at DESC");

    let mut q = sqlx::query(&sql);
    if let Some(ref tid) = query.ticket_id {
        if !tid.is_empty() {
            q = q.bind(tid);
        }
    } else if let Some(ref cid) = query.conversation_id {
        if !cid.is_empty() {
            q = q.bind(cid);
        }
    }

    let rows = match q.fetch_all(&state.db).await {
        Ok(r) => r,
        Err(e) => return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({
                "error": format!("cannot list linked issues: {e}"),
                "note": "the linked_issues read failed; an empty array would look like a real empty list",
            })),
        ).into_response(),
    };
    let list: Vec<Value> = rows.iter().map(format_linked_issue_row).collect();
    Json(list).into_response()
}

pub async fn unlink_issue(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let issue = match sqlx::query("SELECT ticket_id, issue_key FROM linked_issues WHERE id = ?")
        .bind(&id)
        .fetch_optional(&state.db)
        .await
    {
        Ok(Some(r)) => Some(r),
        Ok(None) => None,
        Err(e) => return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({"error": format!("cannot read linked issue {id}: {e}"), "note": "nothing was deleted"})),
        ).into_response(),
    };

    // Delete first: an "Unlinked" activity written before a failed delete would
    // describe a change that never happened.
    let deleted = match sqlx::query("DELETE FROM linked_issues WHERE id = ?")
        .bind(&id)
        .execute(&state.db)
        .await
    {
        Ok(r) => r.rows_affected(),
        Err(e) => return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({"error": format!("delete of linked issue {id} failed: {e}")})),
        ).into_response(),
    };

    if deleted == 0 {
        return (
            StatusCode::NOT_FOUND,
            Json(json!({
                "success": false,
                "deleted_id": id,
                "error": "linked issue not found",
                "note": "the delete matched 0 rows, so nothing was unlinked",
            })),
        ).into_response();
    }

    let mut activity_errors: Vec<String> = Vec::new();
    if let Some(r) = issue {
        let tid: Option<String> = r.try_get("ticket_id").ok().flatten();
        let key: Option<String> = r.try_get("issue_key").ok().flatten();
        if let Some(ref t) = tid {
            let act_id = Uuid::new_v4().to_string();
            let now = Utc::now().to_rfc3339();
            let msg = format!("Unlinked external issue [{}]", key.unwrap_or_else(|| "unknown key".to_string()));
            if let Err(e) = sqlx::query(
                "INSERT INTO ticket_activities (id, ticket_id, author_id, author_name, activity_type, message, activity_metadata, created_at) VALUES (?, ?, NULL, 'External Sync', 'external_issue', ?, '{}', ?)"
            )
            .bind(&act_id)
            .bind(t)
            .bind(&msg)
            .bind(&now)
            .execute(&state.db)
            .await
            {
                activity_errors.push(format!("unlink activity not written: {e}"));
            }
        }
    }

    Json(json!({
        "success": true,
        "deleted_id": id,
        "rows_deleted": deleted,
        "activity_errors": activity_errors
    })).into_response()
}

// Slack Conversation Sync
#[derive(Deserialize, Debug)]
pub struct SlackSyncPayload {
    pub conversation_id: String,
    pub channel_name: Option<String>,
    pub note: Option<String>,
    pub author: Option<String>,
}

/// Reads the stored slack row. Ok(None) means no slack config row exists at all;
/// Err means the read itself failed.
async fn slack_config_row(db: &sqlx::PgPool) -> Result<Option<(Value, Value)>, sqlx::Error> {
    let row = sqlx::query("SELECT credentials, config FROM issue_tracker_configs WHERE provider = 'slack'")
        .fetch_optional(db)
        .await?;
    Ok(row.map(|r| {
        let parse = |col: &str| -> Value {
            r.try_get::<Option<String>, _>(col)
                .ok()
                .flatten()
                .and_then(|s| serde_json::from_str(&s).ok())
                .unwrap_or(Value::Null)
        };
        (parse("credentials"), parse("config"))
    }))
}

/// First non-empty, unmasked string found for any of `keys` across the two
/// stored JSON blobs.
fn slack_setting(creds: &Value, cfg: &Value, keys: &[&str]) -> Option<String> {
    keys.iter().find_map(|key| {
        [creds, cfg].iter().find_map(|src| {
            src.get(*key)
                .and_then(|v| v.as_str())
                .filter(|s| crate::services::integration_settings::usable(s))
                .map(|s| s.trim().to_string())
        })
    })
}

/// A Slack deep link needs the workspace domain, the channel id and the thread
/// ts. Without all three real parts there is no URL to hand back.
fn slack_thread_url(cfg: &Value, channel_id: Option<&str>, thread_ts: Option<&str>) -> Option<String> {
    let domain = slack_setting(cfg, &Value::Null, &["team_domain", "teamDomain", "workspace_domain"])?;
    let channel = channel_id?;
    let ts = thread_ts?;
    Some(format!("https://{domain}.slack.com/archives/{channel}/p{}", ts.replace('.', "")))
}

pub async fn sync_conversation_to_slack(
    State(state): State<AppState>,
    Json(payload): Json<SlackSyncPayload>,
) -> impl IntoResponse {
    let now = Utc::now().to_rfc3339();
    let cid = payload.conversation_id;

    let conversation_exists = match sqlx::query_scalar::<_, i64>("SELECT count(*) FROM conversations WHERE id = ?")
        .bind(&cid)
        .fetch_one(&state.db)
        .await
    {
        Ok(c) => c > 0,
        Err(e) => return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({"error": format!("cannot verify conversation {cid}: {e}"), "note": "nothing was recorded"})),
        ).into_response(),
    };
    if !conversation_exists {
        return (
            StatusCode::NOT_FOUND,
            Json(json!({"error": format!("conversation {cid} does not exist"), "note": "no sync row was written"})),
        ).into_response();
    }

    let (creds, cfg) = match slack_config_row(&state.db).await {
        Ok(Some(pair)) => pair,
        Ok(None) => (Value::Null, Value::Null),
        Err(e) => return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({"error": format!("cannot read the slack tracker config: {e}"), "note": "nothing was recorded"})),
        ).into_response(),
    };

    let existing = match sqlx::query(
        "SELECT id, slack_channel_id, slack_channel_name, slack_thread_ts, is_synced, last_synced_at FROM slack_conversation_syncs WHERE conversation_id = ?"
    )
    .bind(&cid)
    .fetch_optional(&state.db)
    .await
    {
        Ok(Some(r)) => Some(r),
        Ok(None) => None,
        Err(e) => return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({"error": format!("cannot read the existing slack sync row: {e}"), "note": "nothing was written"})),
        ).into_response(),
    };

    let stored_channel_id: Option<String> = existing.as_ref().and_then(|r| r.try_get("slack_channel_id").ok().flatten());
    let stored_channel_name: Option<String> = existing.as_ref().and_then(|r| r.try_get("slack_channel_name").ok().flatten());
    let stored_thread_ts: Option<String> = existing.as_ref().and_then(|r| r.try_get("slack_thread_ts").ok().flatten());
    let stored_is_synced: Option<bool> = existing.as_ref().and_then(|r| r.try_get("is_synced").ok().flatten());
    let stored_synced_at: Option<String> = existing.as_ref().and_then(|r| r.try_get("last_synced_at").ok().flatten());
    let sync_id = existing
        .as_ref()
        .and_then(|r| r.try_get::<String, _>("id").ok())
        .unwrap_or_else(|| Uuid::new_v4().to_string());

    // Destination comes from the request, then the stored row, then the stored
    // slack config. A guessed channel would be a fake destination.
    let channel_name = payload
        .channel_name
        .filter(|s| !s.trim().is_empty())
        .or(stored_channel_name)
        .or_else(|| slack_setting(&creds, &cfg, &["channel_name", "channelName"]));
    let channel_id = stored_channel_id.or_else(|| slack_setting(&creds, &cfg, &["channel_id", "channelId", "slack_channel_id"]));

    if channel_name.is_none() && channel_id.is_none() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "no slack destination is known",
                "reason": "the request carried no channel_name and provider 'slack' has no channel_name/channel_id in issue_tracker_configs; nothing was written",
            })),
        ).into_response();
    }

    // Only a ts that already exists in the row is kept: this endpoint performs no
    // chat.postMessage call, so it cannot mint a real thread id.
    let thread_ts = stored_thread_ts;
    let thread_url = slack_thread_url(&cfg, channel_id.as_deref(), thread_ts.as_deref());
    let has_bot_token = slack_setting(&creds, &cfg, &["bot_token", "botToken", "access_token", "token"]).is_some();

    let upserted = match sqlx::query(
        r#"
        INSERT INTO slack_conversation_syncs (id, conversation_id, slack_channel_id, slack_channel_name, slack_thread_ts, is_synced, last_synced_at, created_at)
        VALUES (?, ?, ?, ?, ?, 0, NULL, ?)
        ON CONFLICT(conversation_id) DO UPDATE SET
            slack_channel_id = excluded.slack_channel_id,
            slack_channel_name = excluded.slack_channel_name
        "#,
    )
    .bind(&sync_id)
    .bind(&cid)
    .bind(&channel_id)
    .bind(&channel_name)
    .bind(&thread_ts)
    .bind(&now)
    .execute(&state.db)
    .await
    {
        Ok(r) => r.rows_affected(),
        Err(e) => return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({"error": format!("could not store the slack sync row: {e}")})),
        ).into_response(),
    };

    if upserted == 0 {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": "slack sync upsert affected 0 rows", "conversationId": cid})),
        ).into_response();
    }

    // A first write stores is_synced = 0; an existing row keeps its recorded
    // value, and an unreadable one stays null rather than defaulting.
    let is_synced: Option<bool> = match (&existing, stored_is_synced) {
        (None, _) => Some(false),
        (_, v) => v,
    };

    Json(json!({
        "id": sync_id,
        "conversation_id": cid,
        "slack_channel_id": channel_id,
        "slackChannelId": channel_id,
        "slack_channel_name": channel_name,
        "slackChannel": channel_name,
        "slack_thread_ts": thread_ts,
        "slackThreadTs": thread_ts,
        "thread_url": thread_url,
        "threadUrl": thread_url,
        "is_synced": is_synced,
        "isSynced": is_synced,
        "last_synced_at": stored_synced_at,
        "lastSyncedAt": stored_synced_at,
        "slack_credentials_configured": has_bot_token,
        "dispatched_note": payload.note,
        "author": payload.author,
        "note": if has_bot_token {
            "This endpoint stores the routing row only; it performs no chat.postMessage call, so no Slack thread was created or updated."
        } else {
            "provider 'slack' has no usable bot_token in issue_tracker_configs, so no Slack request is possible; the routing row above is local only."
        }
    })).into_response()
}

pub async fn get_slack_sync_status(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let row = match sqlx::query("SELECT id, conversation_id, slack_channel_id, slack_channel_name, slack_thread_ts, is_synced, last_synced_at, created_at FROM slack_conversation_syncs WHERE conversation_id = ?")
        .bind(&id)
        .fetch_optional(&state.db)
        .await
    {
        Ok(Some(r)) => r,
        Ok(None) => return Json(json!({
            "conversation_id": id,
            "conversationId": id,
            "record_exists": false,
            "recordExists": false,
            "is_synced": false,
            "isSynced": false,
            "note": "no slack sync row is stored for this conversation; nothing has been routed",
        })).into_response(),
        Err(e) => return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({
                "error": format!("cannot read the slack sync row for {id}: {e}"),
                "note": "the read failed; this is not an unsynced conversation",
            })),
        ).into_response(),
    };

    let (_, cfg) = match slack_config_row(&state.db).await {
        Ok(Some(pair)) => pair,
        Ok(None) => (Value::Null, Value::Null),
        Err(e) => return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({"error": format!("cannot read the slack tracker config: {e}")})),
        ).into_response(),
    };

    let sync_id: Option<String> = row.try_get("id").ok().flatten();
    let cid: Option<String> = row.try_get("conversation_id").ok().flatten();
    let ch_id: Option<String> = row.try_get("slack_channel_id").ok().flatten();
    let ch_name: Option<String> = row.try_get("slack_channel_name").ok().flatten();
    let thread_ts: Option<String> = row.try_get("slack_thread_ts").ok().flatten();
    let is_synced: Option<bool> = row.try_get("is_synced").ok().flatten();
    let last_synced_at: Option<String> = row.try_get("last_synced_at").ok().flatten();
    let thread_url = slack_thread_url(&cfg, ch_id.as_deref(), thread_ts.as_deref());

    Json(json!({
        "id": sync_id,
        "conversation_id": cid,
        "conversationId": cid,
        "record_exists": true,
        "recordExists": true,
        "slack_channel_id": ch_id,
        "slackChannelId": ch_id,
        "slack_channel_name": ch_name,
        "slackChannel": ch_name,
        "slack_thread_ts": thread_ts,
        "slackThreadTs": thread_ts,
        "thread_url": thread_url,
        "threadUrl": thread_url,
        "is_synced": is_synced,
        "isSynced": is_synced,
        "last_synced_at": last_synced_at,
        "lastSyncedAt": last_synced_at
    })).into_response()
}
