//! Zero-trust mesh ingress and egress.
//!
//! Inbound envelopes are verified and decrypted by `CryptoMesh`, then handed to
//! the same dispatcher the plaintext ecosystem webhook uses, so an event cannot
//! claim more completed work because it arrived encrypted. Outbound events are
//! sealed with the ingress key and posted to the relay; anything the relay has
//! not accepted is stored as a real outbox row rather than reported as queued.

use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::{json, Value};

use crate::db::AppState;
use crate::services::crypto_mesh::{CryptoMesh, MeshError, TransitEnvelope};

fn rejected(status: StatusCode, detail: impl Into<String>) -> Response {
    (status, Json(json!({ "success": false, "error": detail.into() }))).into_response()
}

fn mesh_unavailable(err: &MeshError) -> Response {
    rejected(StatusCode::SERVICE_UNAVAILABLE, err.detail())
}

fn value_to_text(value: &Value) -> String {
    match value {
        Value::String(text) => text.clone(),
        Value::Number(num) => num.to_string(),
        Value::Bool(flag) => flag.to_string(),
        _ => String::new(),
    }
}

fn field_text(payload: &Value, key: &str) -> String {
    payload.get(key).map(value_to_text).unwrap_or_default()
}

/// `POST /crypto-mesh/ingress`
pub async fn ingress(State(state): State<AppState>, Json(envelope): Json<TransitEnvelope>) -> Response {
    let mesh = match CryptoMesh::from_config(&state.config) {
        Ok(mesh) => mesh,
        Err(err) => return mesh_unavailable(&err),
    };

    let decrypted = match mesh.decrypt_from_relay(&envelope) {
        Ok(payload) => payload,
        Err(err) => {
            tracing::warn!("CryptoMesh ingress refused: {}", err.detail());
            return match err {
                MeshError::Replay(_) => rejected(StatusCode::UNAUTHORIZED, err.detail()),
                MeshError::Config(_) => mesh_unavailable(&err),
                MeshError::Integrity(_) => rejected(StatusCode::BAD_REQUEST, err.detail()),
            };
        }
    };
    if !decrypted.is_object() {
        return rejected(StatusCode::BAD_REQUEST, "Decrypted payload must be a JSON object.");
    }

    let event_name = [field_text(&decrypted, "name"), field_text(&decrypted, "event")]
        .into_iter()
        .find(|value| !value.is_empty())
        .unwrap_or_default();
    let event_data = match decrypted.get("payload") {
        Some(value) if value.is_object() => value.clone(),
        _ => decrypted.clone(),
    };
    if event_name.is_empty() {
        return rejected(
            StatusCode::BAD_REQUEST,
            "Missing required 'name' in decrypted event payload.",
        );
    }

    tracing::info!(
        "CryptoMesh ingress accepted event '{event_name}' from '{}' (tenant: {})",
        envelope.sender_id,
        envelope.tenant_id
    );

    let outcome =
        super::webhooks::dispatch_ecosystem_event(&state, &event_name, &event_data, &envelope.tenant_id).await;

    let stored = sqlx::query(
        "INSERT INTO ecosystem_events (id, event_name, source, tenant_id, payload, outcome, received_at) \
         VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(uuid::Uuid::new_v4().simple().to_string())
    .bind(&event_name)
    .bind("crypto_mesh")
    .bind(&envelope.tenant_id)
    .bind(event_data.to_string())
    .bind(&outcome)
    .bind(chrono::Utc::now().to_rfc3339())
    .execute(&state.db)
    .await;

    let store_note = match stored {
        Ok(_) => None,
        Err(err) => {
            tracing::error!("CryptoMesh event {event_name} was processed but could not be stored: {err}");
            Some(format!("the event was processed but not persisted: {err}"))
        }
    };

    Json(json!({
        "success": store_note.is_none(),
        "status": if store_note.is_none() { "processed" } else { "processed_unstored" },
        "event": event_name,
        "sender": envelope.sender_id,
        "keyId": envelope.key_id,
        "action": outcome,
        "warning": store_note,
    }))
    .into_response()
}

/// `POST /crypto-mesh/egress` — seal an event with the ingress key and hand it
/// to the relay. The relay's own response is what decides `dispatched`.
pub async fn egress(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    Json(body): Json<Value>,
) -> Response {
    let mesh = match CryptoMesh::from_config(&state.config) {
        Ok(mesh) => mesh,
        Err(err) => return mesh_unavailable(&err),
    };

    let event_name = field_text(&body, "name");
    if event_name.is_empty() {
        return rejected(StatusCode::BAD_REQUEST, "Missing required 'name' for the outbound event.");
    }
    let tenant_id = [
        field_text(&body, "tenantId"),
        headers
            .get("x-tenant-id")
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default()
            .to_string(),
    ]
    .into_iter()
    .find(|value| !value.is_empty())
    .unwrap_or_else(|| mesh.tenant_id().to_string());
    if tenant_id != mesh.tenant_id() {
        return rejected(
            StatusCode::FORBIDDEN,
            format!(
                "Refusing to seal an event for tenant '{tenant_id}'; this backend is configured for '{}'.",
                mesh.tenant_id()
            ),
        );
    }
    let recipient_id = [
        field_text(&body, "recipientId"),
        field_text(&body, "targetModule"),
    ]
    .into_iter()
    .find(|value| !value.is_empty())
    .unwrap_or_default();
    if recipient_id.is_empty() {
        return rejected(
            StatusCode::BAD_REQUEST,
            "Missing 'recipientId': an envelope with no addressee cannot be signed.",
        );
    }

    let event_id = uuid::Uuid::new_v4().simple().to_string();
    let organization_id = field_text(&body, "organizationId");
    let correlation_id = {
        let raw = field_text(&body, "correlationId");
        if raw.is_empty() { event_id.clone() } else { raw }
    };
    let envelope_body = json!({
        "eventId": event_id,
        "name": event_name,
        "occurredAt": chrono::Utc::now().to_rfc3339(),
        "source": mesh.entity_id(),
        "tenantId": tenant_id,
        "organizationId": if organization_id.is_empty() { Value::Null } else { json!(organization_id) },
        "correlationId": correlation_id,
        "schemaVersion": 1,
        "payload": body.get("payload").cloned().unwrap_or_else(|| json!({})),
    });

    let envelope = match mesh.encrypt_for_relay(&recipient_id, &envelope_body) {
        Ok(envelope) => envelope,
        Err(err) => {
            return if err.is_config() {
                mesh_unavailable(&err)
            } else {
                rejected(StatusCode::BAD_REQUEST, err.detail())
            }
        }
    };

    deliver_or_store(&state, &event_id, &event_name, &recipient_id, &tenant_id, &organization_id, &correlation_id, &envelope)
        .await
}

/// Posts a sealed envelope to the relay. A relay that answers non-2xx or is
/// unreachable leaves the envelope in `ecosystem_egress_outbox` with the real
/// error recorded, so `status` never claims more than happened.
async fn deliver_or_store(
    state: &AppState,
    event_id: &str,
    event_name: &str,
    recipient_id: &str,
    tenant_id: &str,
    organization_id: &str,
    correlation_id: &str,
    envelope: &TransitEnvelope,
) -> Response {
    let sealed = match serde_json::to_string(envelope) {
        Ok(sealed) => sealed,
        Err(err) => return rejected(StatusCode::INTERNAL_SERVER_ERROR, format!("envelope could not be stored: {err}")),
    };

    if state.config.neura_relay_gateway_url.trim().is_empty() {
        let outbox_id = store_outbox(state, event_id, event_name, recipient_id, tenant_id, organization_id, correlation_id, &sealed, "NEURA_RELAY_GATEWAY_URL is not configured")
            .await;
        return match outbox_id {
            Some(id) => Json(json!({
                "success": true,
                "status": "stored_for_retry",
                "eventId": event_id,
                "recipient": recipient_id,
                "outboxId": id,
                "detail": "No relay gateway is configured, so the sealed envelope was written to ecosystem_egress_outbox and nothing was sent.",
            }))
            .into_response(),
            None => rejected(
                StatusCode::SERVICE_UNAVAILABLE,
                "No relay gateway is configured and the envelope could not be written to the outbox either.",
            ),
        };
    }

    let mut request = state
        .http_client
        .post(state.config.neura_relay_gateway_url.trim())
        .header("Content-Type", "application/json")
        .header("X-Newax-Source", "communicator")
        .header("X-Tenant-Id", tenant_id);
    if !organization_id.is_empty() {
        request = request.header("X-Organization-Id", organization_id);
    }

    match request.json(envelope).send().await {
        Ok(response) => {
            let status = response.status();
            if status.is_success() {
                // The relay accepted it; mark any earlier pending row for this
                // event as sent so the outbox does not replay a delivered one.
                // WHY: `Ok(0 rows)` is the normal first-try dispatch (no earlier
                // pending row existed), but an `Err` leaves that row pending, so
                // the next flush would re-deliver an event the relay already
                // accepted -- the operator has to see that and the response says
                // so instead of reporting a clean dispatch.
                let superseded = sqlx::query(
                    "UPDATE ecosystem_egress_outbox SET status = 'superseded' WHERE id = ? AND status = 'pending'",
                )
                .bind(event_id)
                .execute(&state.db)
                .await;
                let replay_risk = match superseded {
                    Ok(_) => None,
                    Err(err) => {
                        tracing::error!(
                            "Outbox row {event_id} could not be marked superseded after the relay \
                             accepted it: {err} -- a later flush would re-deliver this event"
                        );
                        Some(format!(
                            "the relay accepted the event, but its outbox row was NOT marked superseded \
                             and can be replayed: {err}"
                        ))
                    }
                };
                let mut body = json!({
                    "success": replay_risk.is_none(),
                    "status": "dispatched",
                    "eventId": event_id,
                    "recipient": recipient_id,
                    "relayStatus": status.as_u16(),
                });
                if let Some(risk) = replay_risk {
                    body["warning"] = json!(risk);
                }
                return Json(body).into_response();
            }
            let text = response
                .text()
                .await
                .unwrap_or_else(|err| format!("<relay response body could not be read: {err}>"));
            let reason = format!("relay gateway returned status {}: {}", status.as_u16(), truncate(&text, 400));
            let outbox_id = store_outbox(state, event_id, event_name, recipient_id, tenant_id, organization_id, correlation_id, &sealed, &reason).await;
            Json(json!({
                "success": false,
                "status": if outbox_id.is_some() { "rejected_stored_for_retry" } else { "rejected_not_stored" },
                "eventId": event_id,
                "recipient": recipient_id,
                "relayStatus": status.as_u16(),
                "outboxId": outbox_id,
                "error": reason,
            }))
            .into_response()
        }
        Err(err) => {
            let reason = format!("relay gateway request failed: {err}");
            let outbox_id = store_outbox(state, event_id, event_name, recipient_id, tenant_id, organization_id, correlation_id, &sealed, &reason).await;
            Json(json!({
                "success": false,
                "status": if outbox_id.is_some() { "stored_for_retry" } else { "failed_not_stored" },
                "eventId": event_id,
                "recipient": recipient_id,
                "outboxId": outbox_id,
                "error": reason,
            }))
            .into_response()
        }
    }
}

#[allow(clippy::too_many_arguments)]
async fn store_outbox(
    state: &AppState,
    event_id: &str,
    event_name: &str,
    recipient_id: &str,
    tenant_id: &str,
    organization_id: &str,
    correlation_id: &str,
    sealed: &str,
    reason: &str,
) -> Option<String> {
    let result = sqlx::query(
        "INSERT INTO ecosystem_egress_outbox \
         (id, event_name, recipient_id, tenant_id, organization_id, correlation_id, envelope, status, attempts, last_error, created_at) \
         VALUES (?, ?, ?, ?, ?, ?, ?, 'pending', 1, ?, ?)",
    )
    .bind(event_id)
    .bind(event_name)
    .bind(recipient_id)
    .bind(tenant_id)
    .bind(if organization_id.is_empty() { None } else { Some(organization_id) })
    .bind(correlation_id)
    .bind(sealed)
    .bind(reason)
    .bind(chrono::Utc::now().to_rfc3339())
    .execute(&state.db)
    .await;
    match result {
        Ok(_) => Some(event_id.to_string()),
        Err(err) => {
            tracing::error!("Outbox row for {event_id} could not be written: {err}");
            None
        }
    }
}

/// `GET /crypto-mesh/outbox`
pub async fn outbox(State(state): State<AppState>) -> Response {
    let rows = sqlx::query_as::<_, (String, String, String, String, String, i64, String, String, Option<String>)>(
        "SELECT id, event_name, recipient_id, tenant_id, status, attempts, last_error, created_at, sent_at \
         FROM ecosystem_egress_outbox ORDER BY created_at DESC LIMIT 200",
    )
    .fetch_all(&state.db)
    .await;

    match rows {
        Ok(rows) => {
            let pending = rows.iter().filter(|row| row.4 == "pending").count();
            Json(json!({
                "success": true,
                "pending": pending,
                "count": rows.len(),
                "entries": rows
                    .into_iter()
                    .map(|row| json!({
                        "id": row.0,
                        "eventName": row.1,
                        "recipientId": row.2,
                        "tenantId": row.3,
                        "status": row.4,
                        "attempts": row.5,
                        "lastError": row.6,
                        "createdAt": row.7,
                        "sentAt": row.8,
                    }))
                    .collect::<Vec<Value>>(),
            }))
            .into_response()
        }
        Err(err) => rejected(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("The egress outbox could not be read: {err}"),
        ),
    }
}

/// `POST /crypto-mesh/outbox/flush` — replay every pending envelope once.
pub async fn flush_outbox(State(state): State<AppState>) -> Response {
    if state.config.neura_relay_gateway_url.trim().is_empty() {
        return rejected(
            StatusCode::SERVICE_UNAVAILABLE,
            "No relay gateway is configured (NEURA_RELAY_GATEWAY_URL), so there is nothing to flush to.",
        );
    }

    let pending = sqlx::query_as::<_, (String, String, String, String, Option<String>, String, i64)>(
        "SELECT id, event_name, recipient_id, tenant_id, organization_id, envelope, attempts \
         FROM ecosystem_egress_outbox WHERE status = 'pending' ORDER BY created_at ASC LIMIT 50",
    )
    .fetch_all(&state.db)
    .await;

    let Ok(pending) = pending else {
        return rejected(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Pending outbox rows could not be read.".to_string(),
        );
    };

    let mut results = Vec::with_capacity(pending.len());
    let mut delivered = 0usize;
    let mut unrecorded = 0usize;
    for (id, event_name, recipient_id, tenant_id, organization_id, envelope_raw, attempts) in pending {
        let envelope: TransitEnvelope = match serde_json::from_str(&envelope_raw) {
            Ok(envelope) => envelope,
            Err(err) => {
                let reason = format!("stored envelope is unreadable: {err}");
                let recorded = record_attempt(&state, &id, &reason, attempts, false).await;
                unrecorded += usize::from(!recorded);
                results.push(json!({ "id": id, "eventName": event_name, "status": audit_status("unreadable", recorded), "error": reason, "attemptRecorded": recorded }));
                continue;
            }
        };

        // A sealed envelope carries its own timestamp; the relay window may have
        // closed since it was stored, and saying so is better than resending.
        let skew = chrono::DateTime::parse_from_rfc3339(&envelope.timestamp)
            .map(|stamp| (chrono::Utc::now() - stamp.with_timezone(&chrono::Utc)).num_seconds())
            .unwrap_or(i64::MAX);
        if skew.abs() > state.config.neura_max_clock_skew_seconds {
            let reason = format!("envelope left the {}s replay window before it could be sent", state.config.neura_max_clock_skew_seconds);
            let recorded = record_attempt(&state, &id, &reason, attempts, false).await;
            unrecorded += usize::from(!recorded);
            results.push(json!({ "id": id, "eventName": event_name, "status": audit_status("expired", recorded), "error": reason, "attemptRecorded": recorded }));
            continue;
        }

        let mut request = state
            .http_client
            .post(state.config.neura_relay_gateway_url.trim())
            .header("Content-Type", "application/json")
            .header("X-Newax-Source", "communicator")
            .header("X-Tenant-Id", &tenant_id);
        if let Some(organization) = organization_id.filter(|value| !value.is_empty()) {
            request = request.header("X-Organization-Id", organization);
        }

        match request.json(&envelope).send().await {
            Ok(response) => {
                let status = response.status();
                if status.is_success() {
                    delivered += 1;
                    let recorded = record_attempt(&state, &id, "", attempts + 1, true).await;
                    unrecorded += usize::from(!recorded);
                    results.push(json!({ "id": id, "eventName": event_name, "recipientId": recipient_id, "status": audit_status("dispatched", recorded), "relayStatus": status.as_u16(), "attemptRecorded": recorded }));
                } else {
                    let text = response
                        .text()
                        .await
                        .unwrap_or_else(|err| format!("<relay response body could not be read: {err}>"));
                    let reason = format!("relay gateway returned status {}: {}", status.as_u16(), truncate(&text, 200));
                    let recorded = record_attempt(&state, &id, &reason, attempts + 1, false).await;
                    unrecorded += usize::from(!recorded);
                    results.push(json!({ "id": id, "eventName": event_name, "recipientId": recipient_id, "status": audit_status("rejected", recorded), "error": reason, "attemptRecorded": recorded }));
                }
            }
            Err(err) => {
                let reason = format!("relay gateway request failed: {err}");
                let recorded = record_attempt(&state, &id, &reason, attempts + 1, false).await;
                unrecorded += usize::from(!recorded);
                results.push(json!({ "id": id, "eventName": event_name, "recipientId": recipient_id, "status": audit_status("retryable", recorded), "error": reason, "attemptRecorded": recorded }));
            }
        }
    }

    Json(json!({
        // WHY: an attempt whose outbox write failed is a real relay call with no
        // audit trail, so the flush cannot report an unqualified success.
        "success": unrecorded == 0,
        "attempted": results.len(),
        "delivered": delivered,
        "attemptsUnrecorded": unrecorded,
        "warning": if unrecorded == 0 {
            Value::Null
        } else {
            json!(format!(
                "{unrecorded} attempt outcome(s) were NOT written to ecosystem_egress_outbox; those \
                 rows still carry their previous state, so a later flush will replay them"
            ))
        },
        "results": results,
    }))
    .into_response()
}

/// WHY: the outbox row is the audit trail for every relay attempt. `Err` and
/// `Ok(0 rows)` both mean the attempt happened but is not on record, so each is
/// logged at error level with the outbox id and the reason, and the caller is
/// told rather than allowed to report the outcome as stored.
async fn record_attempt(state: &AppState, id: &str, reason: &str, attempts: i64, delivered: bool) -> bool {
    let outcome = if delivered { "sent" } else { "pending" };
    match mark_attempt(state, id, reason, attempts, delivered).await {
        Ok(rows) if rows == 1 => true,
        Ok(rows) => {
            tracing::error!(
                "Outbox attempt ({outcome}) for {id} was NOT recorded: the UPDATE affected {rows} \
                 rows (expected 1); reason: {reason}"
            );
            false
        }
        Err(err) => {
            tracing::error!(
                "Outbox attempt ({outcome}) for {id} could not be written: {err}; reason being \
                 recorded: {reason}"
            );
            false
        }
    }
}

fn audit_status(outcome: &str, recorded: bool) -> String {
    if recorded {
        return outcome.to_string();
    }
    format!("{outcome}_audit_unrecorded")
}

async fn mark_attempt(
    state: &AppState,
    id: &str,
    reason: &str,
    attempts: i64,
    delivered: bool,
) -> Result<u64, sqlx::Error> {
    sqlx::query(
        "UPDATE ecosystem_egress_outbox SET attempts = ?, last_error = ?, status = ?, sent_at = ? WHERE id = ?",
    )
    .bind(attempts)
    .bind(reason)
    .bind(if delivered { "sent" } else { "pending" })
    .bind(delivered.then(|| chrono::Utc::now().to_rfc3339()))
    .bind(id)
    .execute(&state.db)
    .await
    .map(|report| report.rows_affected())
}

fn truncate(value: &str, max: usize) -> String {
    if value.len() <= max {
        return value.to_string();
    }
    let mut cut = max;
    while cut > 0 && !value.is_char_boundary(cut) {
        cut -= 1;
    }
    format!("{}…", &value[..cut])
}
