//! Subscribes to the tenant/organization/brand/customer identity events nID
//! publishes (COMMUNICATION.md section 1, async path) and keeps this
//! repo's synced read-model current.
//!
//! **Ownership, restated from `COMMUNICATION.md`:** nID is the one owner of
//! `tenant_id`, `organization_id` (and the brand/customer identity under
//! it). Every table this crate writes is a materialized, eventually
//! consistent *copy* — never treat `synced_tenant`, `synced_organization`,
//! `synced_brand`, or `synced_customer` as this repo's source of truth for
//! those IDs, and never write to them from anywhere except this consumer.
//!
//! **Interim, flagged, local-dev stand-in — not the real bus.** Same
//! situation as nID's `crates/event-bus` (see its doc comment):
//! `core-newax`'s bus doesn't exist to connect to yet, so this talks to a
//! local/dev NATS JetStream server via `NEWAX_EVENT_BUS_URL`.
//!
//! **Idempotency:** NATS JetStream is at-least-once delivery. Every event
//! carries `event_id`; `identity_sync_processed_event` is the dedupe
//! ledger — an event already recorded there is acked and dropped without
//! being re-applied (COMMUNICATION.md: "Idempotency is mandatory, not
//! optional").
//!
//! **Pre-existing, separate systems — do not confuse with this one:**
//! this repo already has `crates/nid_tenant_sync`, `nid_organization_sync`,
//! `nid_brand_sync`, and tables `nid_tenants`/`nid_tenant_mapping`/
//! `nid_organization_mapping` from an earlier, webhook-based (`REST+HMAC`)
//! design, and migration `0064_nid_sync.sql`'s views assume a literally
//! shared Postgres instance with nID — neither matches
//! `COMMUNICATION.md`'s event-bus mandate, and the webhook path has no
//! caller on nID's side today. Flagged in `RULE_VIOLATIONS_AUDIT.md`
//! rather than touched here; this crate's tables are new and distinct.

use anyhow::{Context, Result};
use async_nats::jetstream;
use futures::StreamExt;
use newax_proto::identity_events::v1::{BrandEvent, CustomerEvent, OrganizationEvent, TenantEvent};
use prost::Message;
use sqlx::PgPool;
use tracing::{info, warn};

pub const EVENT_BUS_URL_ENV: &str = "NEWAX_EVENT_BUS_URL";
const DEFAULT_URL: &str = "127.0.0.1:4222";
pub const STREAM_NAME: &str = "NEWAX_IDENTITY";
pub const CONSUMER_NAME: &str = "communicator-identity-sync";
pub const FILTER_SUBJECT: &str = "newax.identity.>";

async fn connect() -> Result<jetstream::stream::Stream> {
    let url = std::env::var(EVENT_BUS_URL_ENV).unwrap_or_else(|_| DEFAULT_URL.to_string());
    let client = async_nats::connect(&url).await.context("connecting to NATS")?;
    let js = jetstream::new(client);
    let stream = js
        .get_or_create_stream(jetstream::stream::Config {
            name: STREAM_NAME.to_string(),
            subjects: vec![FILTER_SUBJECT.to_string()],
            ..Default::default()
        })
        .await
        .context("ensuring NEWAX_IDENTITY stream")?;
    Ok(stream)
}

/// Runs the subscriber loop forever (until the connection is lost or the
/// process is killed). `services/identity-sync`'s `main` is this plus
/// retry/backoff around reconnecting.
pub async fn run(db: PgPool) -> Result<()> {
    let stream = connect().await?;
    let consumer = stream
        .get_or_create_consumer(
            CONSUMER_NAME,
            jetstream::consumer::pull::Config {
                durable_name: Some(CONSUMER_NAME.to_string()),
                filter_subject: FILTER_SUBJECT.to_string(),
                ..Default::default()
            },
        )
        .await
        .context("creating durable consumer")?;

    info!(subject = FILTER_SUBJECT, consumer = CONSUMER_NAME, "identity-sync: listening");

    let mut messages = consumer.messages().await.context("opening message stream")?;
    while let Some(msg) = messages.next().await {
        let msg = match msg {
            Ok(m) => m,
            Err(e) => {
                warn!(error = %e, "identity-sync: error pulling message");
                continue;
            }
        };
        let subject = msg.subject.to_string();
        if let Err(e) = handle_message(&db, &subject, &msg.payload).await {
            warn!(subject, error = %e, "identity-sync: failed to apply event, acking anyway (see comment)");
            // Acked regardless: a malformed/unhandled event would otherwise
            // block the whole durable consumer on redelivery forever. The
            // failure is logged for someone to go look at, not silently
            // dropped.
        }
        if let Err(e) = msg.ack().await {
            warn!(subject, error = %e, "identity-sync: ack failed");
        }
    }
    Ok(())
}

async fn handle_message(db: &PgPool, subject: &str, payload: &[u8]) -> Result<()> {
    // Subjects are `newax.identity.<entity>.<op>` — op is informational
    // (logging/filtering); the handler re-derives the actual effect (create
    // vs. update vs. delete) from the decoded event's own `deleted` field,
    // so a redelivered or reordered event is still handled correctly.
    let entity = subject.split('.').nth(2).unwrap_or("");
    match entity {
        "tenant" => {
            let event = TenantEvent::decode(payload)?;
            if claim_event(db, &event.event_id).await? {
                upsert_tenant(db, &event).await?;
            }
            Ok(())
        }
        "organization" => {
            let event = OrganizationEvent::decode(payload)?;
            if claim_event(db, &event.event_id).await? {
                upsert_organization(db, &event).await?;
            }
            Ok(())
        }
        "brand" => {
            let event = BrandEvent::decode(payload)?;
            if claim_event(db, &event.event_id).await? {
                upsert_brand(db, &event).await?;
            }
            Ok(())
        }
        "customer" => {
            let event = CustomerEvent::decode(payload)?;
            if claim_event(db, &event.event_id).await? {
                upsert_customer(db, &event).await?;
            }
            Ok(())
        }
        other => {
            warn!(entity = other, subject, "identity-sync: unknown entity in subject, ignoring");
            Ok(())
        }
    }
}

/// Claims `event_id` in the dedupe ledger. Returns `true` the first time an
/// event_id is seen (caller should apply it) and `false` on every
/// redelivery of the same event_id (caller must skip the upsert — NATS
/// JetStream is at-least-once, so redelivery is expected, not an error).
async fn claim_event(db: &PgPool, event_id: &str) -> Result<bool> {
    let result = sqlx::query(
        "INSERT INTO identity_sync_processed_event (event_id) VALUES ($1) ON CONFLICT DO NOTHING",
    )
    .bind(event_id)
    .execute(db)
    .await?;
    Ok(result.rows_affected() == 1)
}

async fn upsert_tenant(db: &PgPool, event: &TenantEvent) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO synced_tenant (id, name, slug, data_residency_region, deleted, last_event_id, last_event_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        ON CONFLICT (id) DO UPDATE SET
            name = EXCLUDED.name,
            slug = EXCLUDED.slug,
            data_residency_region = EXCLUDED.data_residency_region,
            deleted = EXCLUDED.deleted,
            last_event_id = EXCLUDED.last_event_id,
            last_event_at = EXCLUDED.last_event_at
        WHERE synced_tenant.last_event_at <= EXCLUDED.last_event_at
        "#,
    )
    .bind(&event.id)
    .bind(&event.name)
    .bind(&event.slug)
    .bind(&event.data_residency_region)
    .bind(event.deleted)
    .bind(&event.event_id)
    .bind(parse_occurred_at(&event.occurred_at))
    .execute(db)
    .await?;
    Ok(())
}

async fn upsert_organization(db: &PgPool, event: &OrganizationEvent) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO synced_organization (id, tenant_id, subject_id, legal_name, registration_number, country_code, deleted, last_event_id, last_event_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        ON CONFLICT (id) DO UPDATE SET
            tenant_id = EXCLUDED.tenant_id,
            subject_id = EXCLUDED.subject_id,
            legal_name = EXCLUDED.legal_name,
            registration_number = EXCLUDED.registration_number,
            country_code = EXCLUDED.country_code,
            deleted = EXCLUDED.deleted,
            last_event_id = EXCLUDED.last_event_id,
            last_event_at = EXCLUDED.last_event_at
        WHERE synced_organization.last_event_at <= EXCLUDED.last_event_at
        "#,
    )
    .bind(&event.id)
    .bind(&event.tenant_id)
    .bind(&event.subject_id)
    .bind(&event.legal_name)
    .bind(&event.registration_number)
    .bind(&event.country_code)
    .bind(event.deleted)
    .bind(&event.event_id)
    .bind(parse_occurred_at(&event.occurred_at))
    .execute(db)
    .await?;
    Ok(())
}

async fn upsert_brand(db: &PgPool, event: &BrandEvent) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO synced_brand (id, organization_id, name, slug, active, deleted, last_event_id, last_event_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        ON CONFLICT (id) DO UPDATE SET
            organization_id = EXCLUDED.organization_id,
            name = EXCLUDED.name,
            slug = EXCLUDED.slug,
            active = EXCLUDED.active,
            deleted = EXCLUDED.deleted,
            last_event_id = EXCLUDED.last_event_id,
            last_event_at = EXCLUDED.last_event_at
        WHERE synced_brand.last_event_at <= EXCLUDED.last_event_at
        "#,
    )
    .bind(&event.id)
    .bind(&event.organization_id)
    .bind(&event.name)
    .bind(&event.slug)
    .bind(event.active)
    .bind(event.deleted)
    .bind(&event.event_id)
    .bind(parse_occurred_at(&event.occurred_at))
    .execute(db)
    .await?;
    Ok(())
}

async fn upsert_customer(db: &PgPool, event: &CustomerEvent) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO synced_customer (id, phone, email, name, deleted, last_event_id, last_event_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        ON CONFLICT (id) DO UPDATE SET
            phone = EXCLUDED.phone,
            email = EXCLUDED.email,
            name = EXCLUDED.name,
            deleted = EXCLUDED.deleted,
            last_event_id = EXCLUDED.last_event_id,
            last_event_at = EXCLUDED.last_event_at
        WHERE synced_customer.last_event_at <= EXCLUDED.last_event_at
        "#,
    )
    .bind(&event.id)
    .bind(&event.phone)
    .bind(&event.email)
    .bind(&event.name)
    .bind(event.deleted)
    .bind(&event.event_id)
    .bind(parse_occurred_at(&event.occurred_at))
    .execute(db)
    .await?;
    Ok(())
}

fn parse_occurred_at(s: &str) -> chrono::DateTime<chrono::Utc> {
    chrono::DateTime::parse_from_rfc3339(s)
        .map(|dt| dt.with_timezone(&chrono::Utc))
        .unwrap_or_else(|_| chrono::Utc::now())
}
