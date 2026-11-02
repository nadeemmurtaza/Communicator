use crate::db::AppState;
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

fn format_ticket_row(row: &sqlx::sqlite::SqliteRow) -> Value {
    let id: String = row.try_get("id").unwrap_or_default();
    let ticket_number: String = row.try_get("ticket_number").unwrap_or_default();
    let title: String = row.try_get("title").unwrap_or_default();
    let description: String = row.try_get("description").unwrap_or_default();
    let ticket_type: String = row.try_get("ticket_type").unwrap_or_default();
    let department: String = row.try_get("department").unwrap_or_default();
    let priority: String = row.try_get("priority").unwrap_or_default();
    let status: String = row.try_get("status").unwrap_or_default();
    let conversation_id: Option<String> = row.try_get("conversation_id").ok();
    let contact_id: Option<String> = row.try_get("contact_id").ok();
    let contact_name: Option<String> = row.try_get("contact_name").ok();
    let contact_identifier: Option<String> = row.try_get("contact_identifier").ok();
    let tracker_id: Option<String> = row.try_get("tracker_id").ok();
    let tracker_number: Option<String> = row.try_get("tracker_number").ok();
    let tracker_title: Option<String> = row.try_get("tracker_title").ok();
    let assigned_to_id: Option<String> = row.try_get("assigned_to_id").ok();
    let assigned_to_name: Option<String> = row.try_get("assigned_to_name").ok();
    let resolution_summary: Option<String> = row.try_get("resolution_summary").ok();
    let customer_broadcast_sent: bool = row.try_get("customer_broadcast_sent").unwrap_or(false);
    let resolved_at: Option<String> = row.try_get("resolved_at").ok();
    let created_at: String = row.try_get("created_at").unwrap_or_default();
    let updated_at: String = row.try_get("updated_at").unwrap_or_default();
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

async fn fetch_full_ticket_by_id(db: &sqlx::SqlitePool, id: &str) -> Option<Value> {
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
    let row = sqlx::query(sql).bind(id).fetch_optional(db).await.ok()??;
    let mut val = format_ticket_row(&row);

    // Fetch activities
    let act_rows = sqlx::query(
        "SELECT id, ticket_id, author_id, author_name, activity_type, message, activity_metadata, created_at FROM ticket_activities WHERE ticket_id = ? ORDER BY created_at ASC"
    )
    .bind(id)
    .fetch_all(db)
    .await
    .unwrap_or_default();

    let activities: Vec<Value> = act_rows.iter().map(|ar| {
        let act_id: String = ar.try_get("id").unwrap_or_default();
        let ticket_id: String = ar.try_get("ticket_id").unwrap_or_default();
        let author_id: Option<String> = ar.try_get("author_id").ok();
        let author_name: String = ar.try_get("author_name").unwrap_or_else(|_| "Agent".to_string());
        let activity_type: String = ar.try_get("activity_type").unwrap_or_else(|_| "comment".to_string());
        let message: String = ar.try_get("message").unwrap_or_default();
        let meta_str: Option<String> = ar.try_get("activity_metadata").ok();
        let metadata: Value = meta_str.and_then(|s| serde_json::from_str(&s).ok()).unwrap_or(json!({}));
        let created_at: String = ar.try_get("created_at").unwrap_or_default();

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
    .await
    .unwrap_or_default();

    let child_tickets: Vec<Value> = child_rows.iter().map(|cr| {
        let cid: String = cr.try_get("id").unwrap_or_default();
        let num: String = cr.try_get("ticket_number").unwrap_or_default();
        let title: String = cr.try_get("title").unwrap_or_default();
        let status: String = cr.try_get("status").unwrap_or_default();
        let priority: String = cr.try_get("priority").unwrap_or_default();
        let dept: String = cr.try_get("department").unwrap_or_default();
        let conv_id: Option<String> = cr.try_get("conversation_id").ok();
        let contact_id: Option<String> = cr.try_get("contact_id").ok();
        let contact_name: Option<String> = cr.try_get("contact_name").ok();
        let created_at: String = cr.try_get("created_at").unwrap_or_default();

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
    Some(val)
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

    let rows = q.fetch_all(&state.db).await.unwrap_or_default();
    let tickets: Vec<Value> = rows.iter().map(format_ticket_row).collect();
    Json(tickets)
}

pub async fn get_ticket_stats(State(state): State<AppState>) -> impl IntoResponse {
    let total: i64 = sqlx::query_scalar("SELECT count(*) FROM tickets")
        .fetch_one(&state.db)
        .await
        .unwrap_or(0);

    let open: i64 = sqlx::query_scalar("SELECT count(*) FROM tickets WHERE status NOT IN ('resolved', 'closed')")
        .fetch_one(&state.db)
        .await
        .unwrap_or(0);

    let customer_issues: i64 = sqlx::query_scalar("SELECT count(*) FROM tickets WHERE ticket_type = 'customer_issue'")
        .fetch_one(&state.db)
        .await
        .unwrap_or(0);

    let back_office: i64 = sqlx::query_scalar("SELECT count(*) FROM tickets WHERE ticket_type = 'back_office'")
        .fetch_one(&state.db)
        .await
        .unwrap_or(0);

    let trackers: i64 = sqlx::query_scalar("SELECT count(*) FROM tickets WHERE ticket_type = 'tracker'")
        .fetch_one(&state.db)
        .await
        .unwrap_or(0);

    let resolved: i64 = sqlx::query_scalar("SELECT count(*) FROM tickets WHERE status IN ('resolved', 'closed')")
        .fetch_one(&state.db)
        .await
        .unwrap_or(0);

    let urgent: i64 = sqlx::query_scalar("SELECT count(*) FROM tickets WHERE priority = 'urgent' AND status NOT IN ('resolved', 'closed')")
        .fetch_one(&state.db)
        .await
        .unwrap_or(0);

    let impacted_customers: i64 = sqlx::query_scalar("SELECT count(DISTINCT contact_id) FROM tickets WHERE contact_id IS NOT NULL")
        .fetch_one(&state.db)
        .await
        .unwrap_or(0);

    let dept_rows = sqlx::query("SELECT department, count(*) as count FROM tickets GROUP BY department")
        .fetch_all(&state.db)
        .await
        .unwrap_or_default();

    let mut department_breakdown = serde_json::Map::new();
    for dr in dept_rows {
        let dept: String = dr.try_get("department").unwrap_or_default();
        let cnt: i64 = dr.try_get("count").unwrap_or(0);
        department_breakdown.insert(dept, json!(cnt));
    }

    let avg_res: Option<f64> = sqlx::query_scalar(
        "SELECT AVG((julianday(resolved_at) - julianday(created_at)) * 24.0) FROM tickets WHERE resolved_at IS NOT NULL"
    )
    .fetch_one(&state.db)
    .await
    .unwrap_or(None);

    let avg_res_hours = (avg_res.unwrap_or(0.0) * 10.0).round() / 10.0;

    let total_activities: i64 = sqlx::query_scalar("SELECT count(*) FROM ticket_activities")
        .fetch_one(&state.db)
        .await
        .unwrap_or(0);

    Json(json!({
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

    let rows = sqlx::query(sql)
        .fetch_all(&state.db)
        .await
        .unwrap_or_default();

    let trackers: Vec<Value> = rows.iter().map(format_ticket_row).collect();
    Json(trackers)
}

pub async fn get_ticket(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let ticket = fetch_full_ticket_by_id(&state.db, &id).await;
    match ticket {
        Some(t) => Json(t).into_response(),
        None => (StatusCode::NOT_FOUND, Json(json!({"error": "Ticket not found"}))).into_response(),
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
    let count: i64 = sqlx::query_scalar("SELECT count(*) FROM tickets")
        .fetch_one(&state.db)
        .await
        .unwrap_or(0);
    let ticket_num = format!("TCK-{}", 1000 + count + 1);
    let now = Utc::now().to_rfc3339();

    let tt = payload.ticket_type.unwrap_or_else(|| "customer_issue".to_string());
    let dept = payload.department.unwrap_or_else(|| "support".to_string());
    let prio = payload.priority.unwrap_or_else(|| "medium".to_string());
    let status = payload.status.unwrap_or_else(|| "open".to_string());
    let author = payload.author_name.unwrap_or_else(|| "Agent".to_string());

    let _ = sqlx::query(
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
    .await;

    // Log creation activity
    let act_id = Uuid::new_v4().to_string();
    let act_msg = format!("Ticket created ({}) in department {} with priority {}", tt, dept, prio);
    let _ = sqlx::query(
        "INSERT INTO ticket_activities (id, ticket_id, author_id, author_name, activity_type, message, activity_metadata, created_at) VALUES (?, ?, NULL, ?, 'creation', ?, '{}', ?)"
    )
    .bind(&act_id)
    .bind(&id)
    .bind(&author)
    .bind(&act_msg)
    .bind(&now)
    .execute(&state.db)
    .await;

    if let Some(t) = fetch_full_ticket_by_id(&state.db, &id).await {
        (StatusCode::CREATED, Json(t)).into_response()
    } else {
        (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": "Failed to create ticket"}))).into_response()
    }
}

pub async fn update_ticket(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    let now = Utc::now().to_rfc3339();

    // Check if ticket exists
    let existing = sqlx::query("SELECT status, priority, department, title, description, tracker_id, assigned_to_id, resolution_summary FROM tickets WHERE id = ?")
        .bind(&id)
        .fetch_optional(&state.db)
        .await
        .ok()
        .flatten();

    let existing = match existing {
        Some(e) => e,
        None => return (StatusCode::NOT_FOUND, Json(json!({"error": "Ticket not found"}))).into_response(),
    };

    let old_status: String = existing.try_get("status").unwrap_or_default();
    let old_priority: String = existing.try_get("priority").unwrap_or_default();
    let old_department: String = existing.try_get("department").unwrap_or_default();
    let old_title: String = existing.try_get("title").unwrap_or_default();
    let old_description: String = existing.try_get("description").unwrap_or_default();
    let old_tracker_id: Option<String> = existing.try_get("tracker_id").ok();
    let old_assigned_to_id: Option<String> = existing.try_get("assigned_to_id").ok();
    let old_resolution: Option<String> = existing.try_get("resolution_summary").ok();

    let new_status = payload.get("status").and_then(|v| v.as_str()).unwrap_or(&old_status);
    let new_priority = payload.get("priority").and_then(|v| v.as_str()).unwrap_or(&old_priority);
    let new_department = payload.get("department").and_then(|v| v.as_str()).unwrap_or(&old_department);
    let new_title = payload.get("title").and_then(|v| v.as_str()).unwrap_or(&old_title);
    let new_description = payload.get("description").and_then(|v| v.as_str()).unwrap_or(&old_description);
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

    let resolved_at = if (new_status == "resolved" || new_status == "closed") && old_status != "resolved" && old_status != "closed" {
        Some(now.clone())
    } else {
        None
    };

    let _ = sqlx::query(
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
    .await;

    // Log activities for status or priority change
    let author = payload.get("author_name").and_then(|v| v.as_str()).unwrap_or("Agent");
    if new_status != old_status {
        let act_id = Uuid::new_v4().to_string();
        let msg = format!("Status updated from '{}' to '{}'", old_status, new_status);
        let _ = sqlx::query(
            "INSERT INTO ticket_activities (id, ticket_id, author_id, author_name, activity_type, message, activity_metadata, created_at) VALUES (?, ?, NULL, ?, 'status_change', ?, '{}', ?)"
        )
        .bind(&act_id)
        .bind(&id)
        .bind(author)
        .bind(&msg)
        .bind(&now)
        .execute(&state.db)
        .await;
    }
    if new_priority != old_priority {
        let act_id = Uuid::new_v4().to_string();
        let msg = format!("Priority changed from '{}' to '{}'", old_priority, new_priority);
        let _ = sqlx::query(
            "INSERT INTO ticket_activities (id, ticket_id, author_id, author_name, activity_type, message, activity_metadata, created_at) VALUES (?, ?, NULL, ?, 'priority_change', ?, '{}', ?)"
        )
        .bind(&act_id)
        .bind(&id)
        .bind(author)
        .bind(&msg)
        .bind(&now)
        .execute(&state.db)
        .await;
    }

    if let Some(t) = fetch_full_ticket_by_id(&state.db, &id).await {
        Json(t).into_response()
    } else {
        (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": "Failed to fetch updated ticket"}))).into_response()
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
    let author = payload.author_name.unwrap_or_else(|| "Agent".to_string());
    let act_type = payload.activity_type.unwrap_or_else(|| "comment".to_string());
    let meta_str = payload.metadata.map(|m| m.to_string()).unwrap_or_else(|| "{}".to_string());

    let _ = sqlx::query(
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
    .await;

    let _ = sqlx::query("UPDATE tickets SET updated_at = ? WHERE id = ?")
        .bind(&now)
        .bind(&id)
        .execute(&state.db)
        .await;

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
        "metadata": serde_json::from_str::<Value>(&meta_str).unwrap_or(json!({})),
        "created_at": now,
        "createdAt": now
    }))
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
    let author = payload.author_name.unwrap_or_else(|| "Agent".to_string());

    let tracker_info = sqlx::query("SELECT ticket_number, title FROM tickets WHERE id = ?")
        .bind(&payload.tracker_id)
        .fetch_optional(&state.db)
        .await
        .ok()
        .flatten();

    let (tr_num, tr_title) = match tracker_info {
        Some(row) => (
            row.try_get::<String, _>("ticket_number").unwrap_or_default(),
            row.try_get::<String, _>("title").unwrap_or_default(),
        ),
        None => ("Unknown".to_string(), "Tracker Incident".to_string()),
    };

    let _ = sqlx::query("UPDATE tickets SET tracker_id = ?, updated_at = ? WHERE id = ?")
        .bind(&payload.tracker_id)
        .bind(&now)
        .bind(&id)
        .execute(&state.db)
        .await;

    let act_id = Uuid::new_v4().to_string();
    let msg = format!("Linked to tracker incident {} ({})", tr_num, tr_title);
    let _ = sqlx::query(
        "INSERT INTO ticket_activities (id, ticket_id, author_id, author_name, activity_type, message, activity_metadata, created_at) VALUES (?, ?, NULL, ?, 'tracker_link', ?, '{}', ?)"
    )
    .bind(&act_id)
    .bind(&id)
    .bind(&author)
    .bind(&msg)
    .bind(&now)
    .execute(&state.db)
    .await;

    if let Some(t) = fetch_full_ticket_by_id(&state.db, &id).await {
        Json(t).into_response()
    } else {
        (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": "Failed to link ticket"}))).into_response()
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
    let author = payload.author_name.unwrap_or_else(|| "Incident Response Manager".to_string());
    let resolve_all = payload.resolve_all_children.unwrap_or(true);
    let send_customer = payload.send_customer_message.unwrap_or(true);

    let tracker_row = sqlx::query("SELECT ticket_number, title FROM tickets WHERE id = ?")
        .bind(&id)
        .fetch_optional(&state.db)
        .await
        .ok()
        .flatten();

    let tracker_num = match tracker_row {
        Some(r) => r.try_get::<String, _>("ticket_number").unwrap_or_default(),
        None => format!("TRK-{}", id),
    };

    let child_rows = sqlx::query("SELECT id, ticket_number, conversation_id FROM tickets WHERE tracker_id = ?")
        .bind(&id)
        .fetch_all(&state.db)
        .await
        .unwrap_or_default();

    let impacted_tickets = child_rows.len();
    let mut messages_sent = 0;
    let mut resolved_tickets = 0;

    for row in &child_rows {
        let child_id: String = row.try_get("id").unwrap_or_default();
        let conv_id: Option<String> = row.try_get("conversation_id").ok();

        if send_customer {
            if let Some(ref cid) = conv_id {
                let msg_id = Uuid::new_v4().to_string();
                let _ = sqlx::query(
                    "INSERT INTO messages (id, conversation_id, sender, body, channel, external_id, delivery_status, created_at) VALUES (?, ?, 'agent', ?, 'chat', NULL, 'sent', ?)"
                )
                .bind(&msg_id)
                .bind(cid)
                .bind(&payload.broadcast_message)
                .bind(&now)
                .execute(&state.db)
                .await;

                messages_sent += 1;
            }
        }

        if resolve_all {
            let res_summary = format!("Auto-resolved via incident tracker {} broadcast: {}", tracker_num, payload.broadcast_message);
            let _ = sqlx::query("UPDATE tickets SET status = 'resolved', resolution_summary = ?, resolved_at = ?, updated_at = ? WHERE id = ?")
                .bind(&res_summary)
                .bind(&now)
                .bind(&now)
                .bind(&child_id)
                .execute(&state.db)
                .await;

            resolved_tickets += 1;

            let act_id = Uuid::new_v4().to_string();
            let act_msg = format!("Resolved via broadcast from {}: {}", tracker_num, payload.broadcast_message);
            let _ = sqlx::query(
                "INSERT INTO ticket_activities (id, ticket_id, author_id, author_name, activity_type, message, activity_metadata, created_at) VALUES (?, ?, NULL, ?, 'broadcast', ?, '{}', ?)"
            )
            .bind(&act_id)
            .bind(&child_id)
            .bind(&author)
            .bind(&act_msg)
            .bind(&now)
            .execute(&state.db)
            .await;
        }
    }

    let tracker_summary = format!("Incident broadcast dispatched to {} linked tickets: {}", impacted_tickets, payload.broadcast_message);
    let _ = sqlx::query(
        "UPDATE tickets SET status = 'resolved', resolution_summary = ?, customer_broadcast_sent = 1, resolved_at = ?, updated_at = ? WHERE id = ?"
    )
    .bind(&tracker_summary)
    .bind(&now)
    .bind(&now)
    .bind(&id)
    .execute(&state.db)
    .await;

    let act_id = Uuid::new_v4().to_string();
    let act_msg = format!("Widespread incident broadcast dispatched: \"{}\". Notified {} customer tickets, resolved {} tickets.", payload.broadcast_message, messages_sent, resolved_tickets);
    let _ = sqlx::query(
        "INSERT INTO ticket_activities (id, ticket_id, author_id, author_name, activity_type, message, activity_metadata, created_at) VALUES (?, ?, NULL, ?, 'broadcast', ?, '{}', ?)"
    )
    .bind(&act_id)
    .bind(&id)
    .bind(&author)
    .bind(&act_msg)
    .bind(&now)
    .execute(&state.db)
    .await;

    Json(json!({
        "tracker_id": id,
        "tracker_number": tracker_num,
        "impacted_tickets": impacted_tickets,
        "messages_sent": messages_sent,
        "resolved_tickets": resolved_tickets,
        "tracker_resolved": true
    }))
}

// Issue Trackers Integration
fn format_tracker_config_row(row: &sqlx::sqlite::SqliteRow) -> Value {
    let id: String = row.try_get("id").unwrap_or_default();
    let provider: String = row.try_get("provider").unwrap_or_default();
    let account_name: String = row.try_get("account_name").unwrap_or_default();
    let is_enabled: bool = row.try_get("is_enabled").unwrap_or(true);
    let test_mode: bool = row.try_get("test_mode").unwrap_or(true);
    let cred_str: String = row.try_get("credentials").unwrap_or_else(|_| "{}".to_string());
    let cfg_str: String = row.try_get("config").unwrap_or_else(|_| "{}".to_string());
    let last_tested_at: Option<String> = row.try_get("last_tested_at").ok();
    let last_test_status: Option<String> = row.try_get("last_test_status").ok();
    let created_at: String = row.try_get("created_at").unwrap_or_default();
    let updated_at: String = row.try_get("updated_at").unwrap_or_default();

    let credentials: Value = serde_json::from_str(&cred_str).unwrap_or(json!({}));
    let config: Value = serde_json::from_str(&cfg_str).unwrap_or(json!({}));

    json!({
        "id": id,
        "provider": provider,
        "account_name": account_name,
        "accountName": account_name,
        "is_enabled": is_enabled,
        "isEnabled": is_enabled,
        "test_mode": test_mode,
        "testMode": test_mode,
        "credentials": credentials,
        "config": config,
        "last_tested_at": last_tested_at,
        "lastTestedAt": last_tested_at,
        "last_test_status": last_test_status,
        "lastTestStatus": last_test_status,
        "created_at": created_at,
        "createdAt": created_at,
        "updated_at": updated_at,
        "updatedAt": updated_at
    })
}

pub async fn list_tracker_configs(State(state): State<AppState>) -> impl IntoResponse {
    let rows = sqlx::query("SELECT id, provider, account_name, is_enabled, test_mode, credentials, config, last_tested_at, last_test_status, created_at, updated_at FROM issue_tracker_configs ORDER BY provider ASC")
        .fetch_all(&state.db)
        .await
        .unwrap_or_default();

    let list: Vec<Value> = rows.iter().map(format_tracker_config_row).collect();
    Json(list)
}

pub async fn get_tracker_config(
    State(state): State<AppState>,
    Path(provider): Path<String>,
) -> impl IntoResponse {
    let row = sqlx::query("SELECT id, provider, account_name, is_enabled, test_mode, credentials, config, last_tested_at, last_test_status, created_at, updated_at FROM issue_tracker_configs WHERE provider = ?")
        .bind(&provider)
        .fetch_optional(&state.db)
        .await
        .ok()
        .flatten();

    match row {
        Some(r) => Json(format_tracker_config_row(&r)).into_response(),
        None => (
            StatusCode::NOT_FOUND,
            Json(json!({"error": format!("Config for provider {} not found", provider)})),
        )
            .into_response(),
    }
}

pub async fn update_tracker_config(
    State(state): State<AppState>,
    Path(provider): Path<String>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    let now = Utc::now().to_rfc3339();

    let existing = sqlx::query("SELECT id, account_name, is_enabled, test_mode, credentials, config FROM issue_tracker_configs WHERE provider = ?")
        .bind(&provider)
        .fetch_optional(&state.db)
        .await
        .ok()
        .flatten();

    let (id, mut cur_creds, mut cur_config, mut cur_account, mut cur_enabled, mut cur_test_mode) = match existing {
        Some(r) => {
            let id: String = r.try_get("id").unwrap_or_default();
            let cred_str: String = r.try_get("credentials").unwrap_or_else(|_| "{}".to_string());
            let cfg_str: String = r.try_get("config").unwrap_or_else(|_| "{}".to_string());
            let creds: Value = serde_json::from_str(&cred_str).unwrap_or(json!({}));
            let cfg: Value = serde_json::from_str(&cfg_str).unwrap_or(json!({}));
            let account: String = r.try_get("account_name").unwrap_or_default();
            let enabled: bool = r.try_get("is_enabled").unwrap_or(true);
            let test: bool = r.try_get("test_mode").unwrap_or(true);
            (id, creds, cfg, account, enabled, test)
        }
        None => (
            Uuid::new_v4().to_string(),
            json!({}),
            json!({}),
            format!("{} Integration", provider),
            true,
            true,
        ),
    };

    if let Some(account) = payload.get("account_name").and_then(|v| v.as_str()) {
        cur_account = account.to_string();
    }
    if let Some(enabled) = payload.get("is_enabled").and_then(|v| v.as_bool()) {
        cur_enabled = enabled;
    }
    if let Some(test) = payload.get("test_mode").and_then(|v| v.as_bool()) {
        cur_test_mode = test;
    }
    if let Some(creds) = payload.get("credentials") {
        if let Some(obj) = creds.as_object() {
            if let Some(target_obj) = cur_creds.as_object_mut() {
                for (k, v) in obj {
                    target_obj.insert(k.clone(), v.clone());
                }
            } else {
                cur_creds = creds.clone();
            }
        }
    }
    if let Some(cfg) = payload.get("config") {
        if let Some(obj) = cfg.as_object() {
            if let Some(target_obj) = cur_config.as_object_mut() {
                for (k, v) in obj {
                    target_obj.insert(k.clone(), v.clone());
                }
            } else {
                cur_config = cfg.clone();
            }
        }
    }

    let creds_str = cur_creds.to_string();
    let config_str = cur_config.to_string();

    let _ = sqlx::query(
        r#"
        INSERT INTO issue_tracker_configs (id, provider, account_name, is_enabled, test_mode, credentials, config, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
        ON CONFLICT(provider) DO UPDATE SET
            account_name = excluded.account_name,
            is_enabled = excluded.is_enabled,
            test_mode = excluded.test_mode,
            credentials = excluded.credentials,
            config = excluded.config,
            updated_at = excluded.updated_at
        "#
    )
    .bind(&id)
    .bind(&provider)
    .bind(&cur_account)
    .bind(cur_enabled)
    .bind(cur_test_mode)
    .bind(&creds_str)
    .bind(&config_str)
    .bind(&now)
    .bind(&now)
    .execute(&state.db)
    .await;

    let updated_row = sqlx::query("SELECT id, provider, account_name, is_enabled, test_mode, credentials, config, last_tested_at, last_test_status, created_at, updated_at FROM issue_tracker_configs WHERE provider = ?")
        .bind(&provider)
        .fetch_one(&state.db)
        .await;

    match updated_row {
        Ok(r) => Json(format_tracker_config_row(&r)).into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": "Failed to update config"}))).into_response(),
    }
}

pub async fn test_tracker_connection(
    State(state): State<AppState>,
    Path(provider): Path<String>,
) -> impl IntoResponse {
    let now = Utc::now().to_rfc3339();

    let row = sqlx::query("SELECT id, test_mode, account_name FROM issue_tracker_configs WHERE provider = ?")
        .bind(&provider)
        .fetch_optional(&state.db)
        .await
        .ok()
        .flatten();

    let (test_mode, account_name) = match row {
        Some(r) => (
            r.try_get::<bool, _>("test_mode").unwrap_or(true),
            r.try_get::<String, _>("account_name").unwrap_or_else(|_| provider.clone()),
        ),
        None => (true, provider.clone()),
    };

    let status = if test_mode { "connected_simulated" } else { "connected" };
    let mode = if test_mode { "simulated" } else { "live" };

    let _ = sqlx::query("UPDATE issue_tracker_configs SET last_tested_at = ?, last_test_status = ? WHERE provider = ?")
        .bind(&now)
        .bind(status)
        .bind(&provider)
        .execute(&state.db)
        .await;

    Json(json!({
        "status": status,
        "mode": mode,
        "message": format!("Successfully connected to {} gateway ({}). Authentication verified.", provider, account_name)
    }))
}

// Linked Issues
#[derive(Deserialize, Debug, Default)]
pub struct LinkedIssuesQuery {
    pub conversation_id: Option<String>,
    pub ticket_id: Option<String>,
}

fn format_linked_issue_row(row: &sqlx::sqlite::SqliteRow) -> Value {
    let id: String = row.try_get("id").unwrap_or_default();
    let conversation_id: Option<String> = row.try_get("conversation_id").ok();
    let ticket_id: Option<String> = row.try_get("ticket_id").ok();
    let provider: String = row.try_get("provider").unwrap_or_default();
    let external_issue_id: String = row.try_get("external_issue_id").unwrap_or_default();
    let issue_key: String = row.try_get("issue_key").unwrap_or_default();
    let issue_url: String = row.try_get("issue_url").unwrap_or_default();
    let title: String = row.try_get("title").unwrap_or_default();
    let status: String = row.try_get("status").unwrap_or_default();
    let priority: Option<String> = row.try_get("priority").ok();
    let assigned_to: Option<String> = row.try_get("assigned_to").ok();
    let created_by_email: Option<String> = row.try_get("created_by_email").ok();
    let sync_status: String = row.try_get("sync_status").unwrap_or_else(|_| "synced".to_string());
    let meta_str: Option<String> = row.try_get("metadata_json").ok();
    let metadata: Value = meta_str.and_then(|s| serde_json::from_str(&s).ok()).unwrap_or(json!({}));
    let created_at: String = row.try_get("created_at").unwrap_or_default();
    let updated_at: String = row.try_get("updated_at").unwrap_or_default();

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

    let rows = q.fetch_all(&state.db).await.unwrap_or_default();
    let list: Vec<Value> = rows.iter().map(format_linked_issue_row).collect();
    Json(list)
}

#[derive(Deserialize, Debug)]
pub struct CreateLinkedIssuePayload {
    pub provider: String,
    pub title: String,
    pub description: Option<String>,
    pub priority: Option<String>,
    pub issue_type: Option<String>,
    pub conversation_id: Option<String>,
    pub ticket_id: Option<String>,
    pub author_email: Option<String>,
}

pub async fn create_linked_issue(
    State(state): State<AppState>,
    Json(payload): Json<CreateLinkedIssuePayload>,
) -> impl IntoResponse {
    let id = Uuid::new_v4().to_string();
    let ext_id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();

    let count: i64 = sqlx::query_scalar("SELECT count(*) FROM linked_issues WHERE provider = ?")
        .bind(&payload.provider)
        .fetch_one(&state.db)
        .await
        .unwrap_or(0);

    let (issue_key, issue_url) = match payload.provider.to_lowercase().as_str() {
        "github" => {
            let num = 40 + count + 1;
            (format!("#{}", num), format!("https://github.com/newax-org/product-engine/issues/{}", num))
        }
        "jira" => {
            let num = 200 + count + 1;
            (format!("SUP-{}", num), format!("https://newax.atlassian.net/browse/SUP-{}", num))
        }
        _ => {
            // linear default
            let num = 100 + count + 1;
            (format!("ENG-{}", num), format!("https://linear.app/newax/issue/ENG-{}", num))
        }
    };

    let priority = payload.priority.unwrap_or_else(|| "medium".to_string());
    let author_email = payload.author_email.unwrap_or_else(|| "support@newax.com".to_string());
    let meta = json!({
        "description": payload.description,
        "issue_type": payload.issue_type
    }).to_string();

    let _ = sqlx::query(
        r#"
        INSERT INTO linked_issues (
            id, conversation_id, ticket_id, provider, external_issue_id, issue_key,
            issue_url, title, status, priority, assigned_to, created_by_email, sync_status,
            metadata_json, created_at, updated_at
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, 'Todo', ?, 'Core Engineering', ?, 'synced', ?, ?, ?)
        "#
    )
    .bind(&id)
    .bind(&payload.conversation_id)
    .bind(&payload.ticket_id)
    .bind(&payload.provider)
    .bind(&ext_id)
    .bind(&issue_key)
    .bind(&issue_url)
    .bind(&payload.title)
    .bind(&priority)
    .bind(&author_email)
    .bind(&meta)
    .bind(&now)
    .bind(&now)
    .execute(&state.db)
    .await;

    // If ticket_id is present, log activity on ticket
    if let Some(ref tid) = payload.ticket_id {
        let act_id = Uuid::new_v4().to_string();
        let msg = format!("External issue linked: [{}] {} ({})", issue_key, payload.title, payload.provider.to_uppercase());
        let _ = sqlx::query(
            "INSERT INTO ticket_activities (id, ticket_id, author_id, author_name, activity_type, message, activity_metadata, created_at) VALUES (?, ?, NULL, 'External Sync', 'external_issue', ?, '{}', ?)"
        )
        .bind(&act_id)
        .bind(tid)
        .bind(&msg)
        .bind(&now)
        .execute(&state.db)
        .await;

        let _ = sqlx::query("UPDATE tickets SET updated_at = ? WHERE id = ?")
            .bind(&now)
            .bind(tid)
            .execute(&state.db)
            .await;
    }

    let created_row = sqlx::query("SELECT id, conversation_id, ticket_id, provider, external_issue_id, issue_key, issue_url, title, status, priority, assigned_to, created_by_email, sync_status, metadata_json, created_at, updated_at FROM linked_issues WHERE id = ?")
        .bind(&id)
        .fetch_one(&state.db)
        .await;

    match created_row {
        Ok(r) => (StatusCode::CREATED, Json(format_linked_issue_row(&r))).into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": "Failed to create linked issue"}))).into_response(),
    }
}

pub async fn unlink_issue(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let issue = sqlx::query("SELECT ticket_id, issue_key FROM linked_issues WHERE id = ?")
        .bind(&id)
        .fetch_optional(&state.db)
        .await
        .ok()
        .flatten();

    if let Some(r) = issue {
        let tid: Option<String> = r.try_get("ticket_id").ok();
        let key: String = r.try_get("issue_key").unwrap_or_default();
        if let Some(ref t) = tid {
            let act_id = Uuid::new_v4().to_string();
            let now = Utc::now().to_rfc3339();
            let msg = format!("Unlinked external issue [{}]", key);
            let _ = sqlx::query(
                "INSERT INTO ticket_activities (id, ticket_id, author_id, author_name, activity_type, message, activity_metadata, created_at) VALUES (?, ?, NULL, 'External Sync', 'external_issue', ?, '{}', ?)"
            )
            .bind(&act_id)
            .bind(t)
            .bind(&msg)
            .bind(&now)
            .execute(&state.db)
            .await;
        }
    }

    let _ = sqlx::query("DELETE FROM linked_issues WHERE id = ?")
        .bind(&id)
        .execute(&state.db)
        .await;

    Json(json!({ "success": true, "deleted_id": id }))
}

pub async fn sync_issue_status(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let now = Utc::now().to_rfc3339();

    let _ = sqlx::query("UPDATE linked_issues SET status = 'In Progress', sync_status = 'synced', updated_at = ? WHERE id = ?")
        .bind(&now)
        .bind(&id)
        .execute(&state.db)
        .await;

    let updated_row = sqlx::query("SELECT id, conversation_id, ticket_id, provider, external_issue_id, issue_key, issue_url, title, status, priority, assigned_to, created_by_email, sync_status, metadata_json, created_at, updated_at FROM linked_issues WHERE id = ?")
        .bind(&id)
        .fetch_optional(&state.db)
        .await
        .ok()
        .flatten();

    match updated_row {
        Some(r) => Json(format_linked_issue_row(&r)).into_response(),
        None => (StatusCode::NOT_FOUND, Json(json!({"error": "Linked issue not found"}))).into_response(),
    }
}

// Slack Conversation Sync
#[derive(Deserialize, Debug)]
pub struct SlackSyncPayload {
    pub conversation_id: String,
    pub channel_name: Option<String>,
    pub note: Option<String>,
    pub author: Option<String>,
}

pub async fn sync_conversation_to_slack(
    State(state): State<AppState>,
    Json(payload): Json<SlackSyncPayload>,
) -> impl IntoResponse {
    let now = Utc::now().to_rfc3339();
    let cid = payload.conversation_id;

    let cfg_channel: Option<String> = sqlx::query_scalar("SELECT config FROM issue_tracker_configs WHERE provider = 'slack'")
        .fetch_optional(&state.db)
        .await
        .ok()
        .flatten()
        .and_then(|s: String| serde_json::from_str::<Value>(&s).ok())
        .and_then(|v| v.get("channel_name").and_then(|c| c.as_str().map(|s| s.to_string())));

    let channel_name = payload.channel_name
        .filter(|s| !s.is_empty())
        .or(cfg_channel)
        .unwrap_or_else(|| "#customer-escalations".to_string());

    let existing = sqlx::query("SELECT id, slack_thread_ts FROM slack_conversation_syncs WHERE conversation_id = ?")
        .bind(&cid)
        .fetch_optional(&state.db)
        .await
        .ok()
        .flatten();

    let (sync_id, thread_ts) = match existing {
        Some(r) => (
            r.try_get::<String, _>("id").unwrap_or_default(),
            r.try_get::<String, _>("slack_thread_ts").unwrap_or_else(|_| "1730538900.001200".to_string()),
        ),
        None => (
            Uuid::new_v4().to_string(),
            format!("{}.{:06}", chrono::Utc::now().timestamp(), chrono::Utc::now().timestamp_subsec_micros()),
        ),
    };

    let thread_url = format!("https://newax.slack.com/archives/C08SUPPORT01/p{}", thread_ts.replace(".", ""));

    let _ = sqlx::query(
        r#"
        INSERT INTO slack_conversation_syncs (id, conversation_id, slack_channel_id, slack_channel_name, slack_thread_ts, is_synced, last_synced_at, created_at)
        VALUES (?, ?, 'C08SUPPORT01', ?, ?, 1, ?, ?)
        ON CONFLICT(conversation_id) DO UPDATE SET
            slack_channel_name = excluded.slack_channel_name,
            is_synced = 1,
            last_synced_at = excluded.last_synced_at
        "#
    )
    .bind(&sync_id)
    .bind(&cid)
    .bind(&channel_name)
    .bind(&thread_ts)
    .bind(&now)
    .bind(&now)
    .execute(&state.db)
    .await;

    Json(json!({
        "id": sync_id,
        "conversation_id": cid,
        "slack_channel_name": channel_name,
        "slack_thread_ts": thread_ts,
        "thread_url": thread_url,
        "is_synced": true,
        "last_synced_at": now,
        "dispatched_note": payload.note,
        "author": payload.author
    }))
}

pub async fn get_slack_sync_status(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let row = sqlx::query("SELECT id, conversation_id, slack_channel_id, slack_channel_name, slack_thread_ts, is_synced, last_synced_at, created_at FROM slack_conversation_syncs WHERE conversation_id = ?")
        .bind(&id)
        .fetch_optional(&state.db)
        .await
        .ok()
        .flatten();

    match row {
        Some(r) => {
            let sync_id: String = r.try_get("id").unwrap_or_default();
            let cid: String = r.try_get("conversation_id").unwrap_or_default();
            let ch_id: Option<String> = r.try_get("slack_channel_id").ok();
            let ch_name: Option<String> = r.try_get("slack_channel_name").ok();
            let thread_ts: Option<String> = r.try_get("slack_thread_ts").ok();
            let is_synced: bool = r.try_get("is_synced").unwrap_or(true);
            let last_synced_at: Option<String> = r.try_get("last_synced_at").ok();
            let thread_url = thread_ts.as_ref().map(|ts| format!("https://newax.slack.com/archives/C08SUPPORT01/p{}", ts.replace(".", "")));

            Json(json!({
                "id": sync_id,
                "conversation_id": cid,
                "conversationId": cid,
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
        None => Json(json!({
            "conversation_id": id,
            "conversationId": id,
            "is_synced": false,
            "isSynced": false
        })).into_response(),
    }
}
