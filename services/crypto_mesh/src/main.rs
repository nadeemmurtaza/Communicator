//! Crypto Mesh gRPC service — standalone binary on port 50085.

use std::env;
use std::time::Duration;

use newax_crypto_mesh::{CryptoMesh, MeshError, TransitEnvelope};
use serde_json::{json, Value};
use sqlx::{PgPool, Row};
use tonic::{Code, Request, Response, Status};
use tracing::info;

pub mod proto {
    tonic::include_proto!("newax.crypto_mesh.v1");
}

use proto::{
    crypto_mesh_service_server::{CryptoMeshService, CryptoMeshServiceServer},
    EgressRequest, Empty, IngressRequest, JsonResponse,
};

#[derive(Clone)]
struct Svc {
    mesh: CryptoMesh,
    db: PgPool,
    http_client: reqwest::Client,
    relay_url: String,
}

#[tonic::async_trait]
impl CryptoMeshService for Svc {
    async fn ingress(
        &self,
        request: Request<IngressRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let req = request.into_inner();
        let envelope = TransitEnvelope {
            version: req.version,
            tenant_id: req.tenant_id,
            sender_id: req.sender_id,
            recipient_id: req.recipient_id,
            iv: req.iv,
            ciphertext: req.ciphertext,
            auth_tag: req.auth_tag,
            key_id: req.key_id,
            timestamp: req.timestamp,
            signature: req.signature,
        };

        let decrypted = match self.mesh.decrypt_from_relay(&envelope) {
            Ok(payload) => payload,
            Err(err) => {
                tracing::warn!("CryptoMesh ingress refused: {}", err.detail());
                return Err(match err {
                    MeshError::Replay(msg) => Status::new(Code::Unauthenticated, msg),
                    MeshError::Config(msg) => Status::internal(msg),
                    MeshError::Integrity(msg) => Status::invalid_argument(msg),
                });
            }
        };

        if !decrypted.is_object() {
            return Err(Status::invalid_argument(
                "Decrypted payload must be a JSON object.",
            ));
        }

        let event_name = extract_field(&decrypted, "name")
            .or_else(|| extract_field(&decrypted, "event"))
            .unwrap_or_default();
        let event_data = match decrypted.get("payload") {
            Some(value) if value.is_object() => value.clone(),
            _ => decrypted.clone(),
        };

        if event_name.is_empty() {
            return Err(Status::invalid_argument(
                "Missing required 'name' in decrypted event payload.",
            ));
        }

        info!(
            "CryptoMesh ingress accepted event '{}' from '{}' (tenant: {})",
            event_name, envelope.sender_id, envelope.tenant_id
        );

        let stored = sqlx::query(
            "INSERT INTO ecosystem_events (id, event_name, source, tenant_id, payload, outcome, received_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7)",
        )
        .bind(uuid::Uuid::new_v4().simple().to_string())
        .bind(&event_name)
        .bind("crypto_mesh")
        .bind(&envelope.tenant_id)
        .bind(event_data.to_string())
        .bind("stored")
        .bind(chrono::Utc::now().to_rfc3339())
        .execute(&self.db)
        .await;

        let store_note = match stored {
            Ok(_) => None,
            Err(err) => {
                tracing::error!("CryptoMesh event {} could not be stored: {err}", event_name);
                Some(format!("the event was processed but not persisted: {err}"))
            }
        };

        Ok(Response::new(JsonResponse {
            json: json!({
                "success": store_note.is_none(),
                "status": if store_note.is_none() { "processed" } else { "processed_unstored" },
                "event": event_name,
                "sender": envelope.sender_id,
                "keyId": envelope.key_id,
                "action": "stored",
                "warning": store_note,
            })
            .to_string(),
        }))
    }

    async fn egress(
        &self,
        request: Request<EgressRequest>,
    ) -> Result<Response<JsonResponse>, Status> {
        let req = request.into_inner();
        let event_name = req.name;
        if event_name.is_empty() {
            return Err(Status::invalid_argument(
                "Missing required 'name' for the outbound event.",
            ));
        }

        let tenant_id = if !req.tenant_id.is_empty() {
            req.tenant_id
        } else {
            self.mesh.tenant_id().to_string()
        };

        if tenant_id != self.mesh.tenant_id() {
            return Err(Status::new(Code::PermissionDenied, format!(
                "Refusing to seal an event for tenant '{}'; this backend is configured for '{}'.",
                tenant_id,
                self.mesh.tenant_id()
            )));
        }

        let recipient_id = req.recipient_id;
        if recipient_id.is_empty() {
            return Err(Status::invalid_argument(
                "Missing 'recipientId': an envelope with no addressee cannot be signed.",
            ));
        }

        let event_id = uuid::Uuid::new_v4().simple().to_string();
        let organization_id = req.organization_id;
        let correlation_id = if !req.correlation_id.is_empty() {
            req.correlation_id
        } else {
            event_id.clone()
        };

        let payload_value: Value = if !req.payload.is_empty() {
            serde_json::from_str(&req.payload)
                .map_err(|e| Status::invalid_argument(format!("invalid payload JSON: {e}")))?
        } else {
            json!({})
        };

        let envelope_body = json!({
            "eventId": event_id,
            "name": event_name,
            "occurredAt": chrono::Utc::now().to_rfc3339(),
            "source": self.mesh.entity_id(),
            "tenantId": tenant_id,
            "organizationId": if organization_id.is_empty() { Value::Null } else { json!(organization_id) },
            "correlationId": correlation_id,
            "schemaVersion": 1,
            "payload": payload_value,
        });

        let envelope = self
            .mesh
            .encrypt_for_relay(&recipient_id, &envelope_body)
            .map_err(|err| {
                if err.is_config() {
                    Status::internal(err.detail())
                } else {
                    Status::invalid_argument(err.detail())
                }
            })?;

        let sealed = serde_json::to_string(&envelope)
            .map_err(|e| Status::internal(format!("envelope could not be serialized: {e}")))?;

        if self.relay_url.is_empty() {
            let outbox_id = store_outbox(
                &self.db,
                &event_id,
                &event_name,
                &recipient_id,
                &tenant_id,
                &organization_id,
                &correlation_id,
                &sealed,
                "NEURA_RELAY_GATEWAY_URL is not configured",
            )
            .await;

            return Ok(Response::new(JsonResponse {
                json: json!({
                    "success": true,
                    "status": "stored_for_retry",
                    "eventId": event_id,
                    "recipient": recipient_id,
                    "outboxId": outbox_id,
                    "detail": "No relay gateway is configured.",
                })
                .to_string(),
            }));
        }

        let mut request_builder = self
            .http_client
            .post(&self.relay_url)
            .header("Content-Type", "application/json")
            .header("X-Newax-Source", "communicator")
            .header("X-Tenant-Id", &tenant_id);
        if !organization_id.is_empty() {
            request_builder = request_builder.header("X-Organization-Id", &organization_id);
        }

        match request_builder.json(&envelope).send().await {
            Ok(response) => {
                let status = response.status();
                if status.is_success() {
                    let _ = sqlx::query(
                        "UPDATE ecosystem_egress_outbox SET status = 'superseded' WHERE id = $1 AND status = 'pending'",
                    )
                    .bind(&event_id)
                    .execute(&self.db)
                    .await;

                    return Ok(Response::new(JsonResponse {
                        json: json!({
                            "success": true,
                            "status": "dispatched",
                            "eventId": event_id,
                            "recipient": recipient_id,
                            "relayStatus": status.as_u16(),
                        })
                        .to_string(),
                    }));
                }

                let text = response.text().await.unwrap_or_default();
                let reason = format!("relay gateway returned status {}: {}", status.as_u16(), truncate(&text, 400));
                let outbox_id = store_outbox(
                    &self.db, &event_id, &event_name, &recipient_id, &tenant_id,
                    &organization_id, &correlation_id, &sealed, &reason,
                ).await;

                Ok(Response::new(JsonResponse {
                    json: json!({
                        "success": false,
                        "status": if outbox_id.is_some() { "rejected_stored_for_retry" } else { "rejected_not_stored" },
                        "eventId": event_id,
                        "recipient": recipient_id,
                        "relayStatus": status.as_u16(),
                        "outboxId": outbox_id,
                        "error": reason,
                    })
                    .to_string(),
                }))
            }
            Err(err) => {
                let reason = format!("relay gateway request failed: {err}");
                let outbox_id = store_outbox(
                    &self.db, &event_id, &event_name, &recipient_id, &tenant_id,
                    &organization_id, &correlation_id, &sealed, &reason,
                ).await;

                Ok(Response::new(JsonResponse {
                    json: json!({
                        "success": false,
                        "status": if outbox_id.is_some() { "stored_for_retry" } else { "failed_not_stored" },
                        "eventId": event_id,
                        "recipient": recipient_id,
                        "outboxId": outbox_id,
                        "error": reason,
                    })
                    .to_string(),
                }))
            }
        }
    }

    async fn list_outbox(
        &self,
        _request: Request<Empty>,
    ) -> Result<Response<JsonResponse>, Status> {
        let rows = sqlx::query(
            "SELECT id, event_name, recipient_id, tenant_id, status, attempts, last_error, created_at, sent_at \
             FROM ecosystem_egress_outbox ORDER BY created_at DESC LIMIT 200",
        )
        .fetch_all(&self.db)
        .await
        .map_err(|e| Status::internal(format!("The egress outbox could not be read: {e}")))?;

        let pending = rows.iter().filter(|row| {
            let status: String = row.try_get("status").unwrap_or_default();
            status == "pending"
        }).count();

        let entries: Vec<Value> = rows.iter().map(|row| {
            json!({
                "id": row.try_get::<String, _>("id").unwrap_or_default(),
                "eventName": row.try_get::<String, _>("event_name").unwrap_or_default(),
                "recipientId": row.try_get::<String, _>("recipient_id").unwrap_or_default(),
                "tenantId": row.try_get::<String, _>("tenant_id").unwrap_or_default(),
                "status": row.try_get::<String, _>("status").unwrap_or_default(),
                "attempts": row.try_get::<i64, _>("attempts").unwrap_or(0),
                "lastError": row.try_get::<String, _>("last_error").unwrap_or_default(),
                "createdAt": row.try_get::<String, _>("created_at").unwrap_or_default(),
                "sentAt": row.try_get::<Option<String>, _>("sent_at").ok().flatten(),
            })
        }).collect();

        Ok(Response::new(JsonResponse {
            json: json!({
                "success": true,
                "pending": pending,
                "count": entries.len(),
                "entries": entries,
            })
            .to_string(),
        }))
    }

    async fn flush_outbox(
        &self,
        _request: Request<Empty>,
    ) -> Result<Response<JsonResponse>, Status> {
        if self.relay_url.is_empty() {
            return Err(Status::internal(
                "No relay gateway is configured (NEURA_RELAY_GATEWAY_URL), so there is nothing to flush to.",
            ));
        }

        let rows = sqlx::query(
            "SELECT id, event_name, recipient_id, tenant_id, attempts, envelope, organization_id \
             FROM ecosystem_egress_outbox WHERE status = 'pending' ORDER BY created_at ASC LIMIT 50",
        )
        .fetch_all(&self.db)
        .await
        .map_err(|_| Status::internal("Pending outbox rows could not be read.".to_string()))?;

        let mut results = Vec::with_capacity(rows.len());
        let mut delivered = 0usize;
        let mut unrecorded = 0usize;

        for row in rows {
            let id: String = row.try_get("id").unwrap_or_default();
            let event_name: String = row.try_get("event_name").unwrap_or_default();
            let recipient_id: String = row.try_get("recipient_id").unwrap_or_default();
            let tenant_id: String = row.try_get("tenant_id").unwrap_or_default();
            let attempts: i64 = row.try_get("attempts").unwrap_or(0);
            let envelope_raw: String = row.try_get("envelope").unwrap_or_default();
            let organization_id: Option<String> = row.try_get("organization_id").ok().flatten();

            let envelope: TransitEnvelope = match serde_json::from_str(&envelope_raw) {
                Ok(e) => e,
                Err(err) => {
                    let reason = format!("stored envelope is unreadable: {err}");
                    let recorded = record_attempt(&self.db, &id, &reason, attempts, false).await;
                    unrecorded += usize::from(!recorded);
                    results.push(json!({
                        "id": id, "eventName": event_name,
                        "status": audit_status("unreadable", recorded),
                        "error": reason, "attemptRecorded": recorded,
                    }));
                    continue;
                }
            };

            let max_skew: i64 = env::var("NEURA_MAX_CLOCK_SKEW_SECONDS")
                .ok().and_then(|v| v.parse().ok()).unwrap_or(300);

            let skew = chrono::DateTime::parse_from_rfc3339(&envelope.timestamp)
                .map(|stamp| (chrono::Utc::now() - stamp.with_timezone(&chrono::Utc)).num_seconds())
                .unwrap_or(i64::MAX);

            if skew.abs() > max_skew {
                let reason = format!("envelope left the {}s replay window before it could be sent", max_skew);
                let recorded = record_attempt(&self.db, &id, &reason, attempts, false).await;
                unrecorded += usize::from(!recorded);
                results.push(json!({
                    "id": id, "eventName": event_name,
                    "status": audit_status("expired", recorded),
                    "error": reason, "attemptRecorded": recorded,
                }));
                continue;
            }

            let mut rb = self.http_client.post(&self.relay_url)
                .header("Content-Type", "application/json")
                .header("X-Newax-Source", "communicator")
                .header("X-Tenant-Id", &tenant_id);
            if let Some(org) = organization_id.filter(|v| !v.is_empty()) {
                rb = rb.header("X-Organization-Id", org);
            }

            match rb.json(&envelope).send().await {
                Ok(response) => {
                    let status = response.status();
                    if status.is_success() {
                        delivered += 1;
                        let recorded = record_attempt(&self.db, &id, "", attempts + 1, true).await;
                        unrecorded += usize::from(!recorded);
                        results.push(json!({
                            "id": id, "eventName": event_name, "recipientId": recipient_id,
                            "status": audit_status("dispatched", recorded),
                            "relayStatus": status.as_u16(), "attemptRecorded": recorded,
                        }));
                    } else {
                        let text = response.text().await.unwrap_or_default();
                        let reason = format!("relay gateway returned status {}: {}", status.as_u16(), truncate(&text, 200));
                        let recorded = record_attempt(&self.db, &id, &reason, attempts + 1, false).await;
                        unrecorded += usize::from(!recorded);
                        results.push(json!({
                            "id": id, "eventName": event_name, "recipientId": recipient_id,
                            "status": audit_status("rejected", recorded),
                            "error": reason, "attemptRecorded": recorded,
                        }));
                    }
                }
                Err(err) => {
                    let reason = format!("relay gateway request failed: {err}");
                    let recorded = record_attempt(&self.db, &id, &reason, attempts + 1, false).await;
                    unrecorded += usize::from(!recorded);
                    results.push(json!({
                        "id": id, "eventName": event_name, "recipientId": recipient_id,
                        "status": audit_status("retryable", recorded),
                        "error": reason, "attemptRecorded": recorded,
                    }));
                }
            }
        }

        Ok(Response::new(JsonResponse {
            json: json!({
                "success": unrecorded == 0,
                "attempted": results.len(),
                "delivered": delivered,
                "attemptsUnrecorded": unrecorded,
                "warning": if unrecorded == 0 { Value::Null } else {
                    json!(format!("{unrecorded} attempt outcome(s) were NOT written"))
                },
                "results": results,
            })
            .to_string(),
        }))
    }
}

fn extract_field(payload: &Value, key: &str) -> Option<String> {
    match payload.get(key) {
        Some(Value::String(t)) => Some(t.clone()),
        Some(Value::Number(n)) => Some(n.to_string()),
        Some(Value::Bool(b)) => Some(b.to_string()),
        _ => None,
    }
}

async fn store_outbox(
    db: &PgPool, event_id: &str, event_name: &str, recipient_id: &str,
    tenant_id: &str, organization_id: &str, correlation_id: &str,
    sealed: &str, reason: &str,
) -> Option<String> {
    let result = sqlx::query(
        "INSERT INTO ecosystem_egress_outbox \
         (id, event_name, recipient_id, tenant_id, organization_id, correlation_id, envelope, status, attempts, last_error, created_at) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, 'pending', 1, $8, $9)",
    )
    .bind(event_id).bind(event_name).bind(recipient_id).bind(tenant_id)
    .bind(if organization_id.is_empty() { None } else { Some(organization_id) })
    .bind(correlation_id).bind(sealed).bind(reason)
    .bind(chrono::Utc::now().to_rfc3339())
    .execute(db).await;

    match result {
        Ok(_) => Some(event_id.to_string()),
        Err(err) => {
            tracing::error!("Outbox row for {} could not be written: {err}", event_id);
            None
        }
    }
}

async fn record_attempt(db: &PgPool, id: &str, reason: &str, attempts: i64, delivered: bool) -> bool {
    let outcome = if delivered { "sent" } else { "pending" };
    match sqlx::query(
        "UPDATE ecosystem_egress_outbox SET attempts = $1, last_error = $2, status = $3, sent_at = $4 WHERE id = $5",
    )
    .bind(attempts).bind(reason).bind(outcome)
    .bind(delivered.then(|| chrono::Utc::now().to_rfc3339())).bind(id)
    .execute(db).await
    {
        Ok(report) if report.rows_affected() == 1 => true,
        Ok(report) => {
            tracing::error!("Outbox attempt ({}) for {} affected {} rows", outcome, id, report.rows_affected());
            false
        }
        Err(err) => {
            tracing::error!("Outbox attempt ({}) for {} could not be written: {err}", outcome, id);
            false
        }
    }
}

fn audit_status(outcome: &str, recorded: bool) -> String {
    if recorded { outcome.to_string() } else { format!("{outcome}_audit_unrecorded") }
}

fn truncate(value: &str, max: usize) -> String {
    if value.len() <= max { return value.to_string(); }
    let mut cut = max;
    while cut > 0 && !value.is_char_boundary(cut) { cut -= 1; }
    format!("{}…", &value[..cut])
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env()
            .add_directive("svc_crypto_mesh=info".parse()?))
        .init();

    let mesh = CryptoMesh::from_env().map_err(|e| anyhow::anyhow!("{}", e.detail()))?;
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let db = PgPool::connect(&db_url).await?;
    let relay_url = env::var("NEURA_RELAY_GATEWAY_URL").unwrap_or_default();
    let http_client = reqwest::Client::builder().timeout(Duration::from_secs(30)).build()?;

    let svc = Svc { mesh, db, http_client, relay_url };
    let addr = "0.0.0.0:50085".parse()?;
    info!("Crypto Mesh service listening on {}", addr);

    tonic::transport::Server::builder()
        .add_service(CryptoMeshServiceServer::new(svc))
        .serve(addr)
        .await?;

    Ok(())
}
