# Communicator Roadmap

Status: **Planned generalization and resale roadmap**

The existing repository already contains the Harvester-first communication path through Phases 0–5. This roadmap preserves that working foundation while generalizing Communicator into a standalone, resellable product that can connect to many customer systems and operate without Harvester or NEWAX Core.

## Product outcome

Communicator must be able to:

- Operate independently with its own database, authentication, tenants, channels, context, consent, and audit.
- Connect to approved databases, APIs, files, queues, and object stores through credentialed adapters.
- Map and normalize each source according to Communicator's own profile, audience, conversation, and engagement model.
- Build dynamic groups, static groups, and frozen campaign audiences.
- Generate high-quality replies using local history plus approved external context.
- Integrate directly with Harvester when configured.
- Integrate with NEWAX Core when configured.
- Support multi-tenant resale, licensing, controlled white-labeling, backup, restore, upgrades, and data portability.

## Delivery principles

1. Existing messaging behavior must remain operational during generalization.
2. Harvester becomes one optional source adapter.
3. NEWAX Core becomes one optional identity and context adapter.
4. Product-local message and inbound transactions commit independently.
5. Consent and suppression remain deterministic and source-independent.
6. External schemas never become Communicator's internal schema.
7. Every mapping and synchronization version is auditable.
8. Database writes are disabled by default.
9. No shared product tables or database credentials.
10. A campaign audience is frozen before sending.

# Phase CSP-0: Product boundary and migration inventory

Deliver:

- Standalone-product decision.
- Product ownership matrix.
- Existing Harvester coupling inventory.
- Migration plan for `Lead`, Harvester IDs, lead sync worker, and outcome callbacks.
- Generic source, profile, and external-reference vocabulary.
- Optional Core contract.
- Direct Harvester adapter contract.
- Current behavior regression matrix.

Acceptance:

- Existing Harvester integration is classified as an adapter, not the product identity.
- No existing Phase 0–5 feature is silently represented as already generalized.
- Current API and database compatibility risks are listed.

# Phase CSP-1: Generic profile foundation

Deliver:

- Generalize `Lead` into a communication-oriented source/profile model without losing current data.
- Organization profile.
- Person profile.
- Contact channel.
- External source reference.
- Source-owned attributes.
- Profile provenance and freshness.
- Duplicate and identity-match review.
- Migration and compatibility layer for current Harvester records.

Acceptance:

- A profile can originate from manual entry, a customer source, Harvester, or Core.
- Communicator no longer assumes every organization is a qualified lead.
- Existing conversations retain valid profile links after migration.

# Phase CSP-2: Connection and credential foundation

Deliver:

- `ConnectionProfile`.
- `CredentialReference`.
- Connector registry.
- Environment and tenant ownership.
- Read-only, import, export, write, bidirectional, and schema-inspection modes.
- Encrypted secret-provider abstraction.
- Metadata-only credential APIs.
- Validation, replacement, rotation, disable, and enable workflows.

Acceptance:

- Stored plaintext credentials never return to the frontend.
- Failed candidate validation preserves the previous active credential.
- Every connection is tenant-scoped and audited.

# Phase CSP-3: Database and source adapter kernel

Initial adapters:

- PostgreSQL
- MySQL
- MariaDB
- Microsoft SQL Server
- SQLite
- MongoDB
- Supabase
- Neon
- Amazon RDS-compatible relational services
- Google Cloud SQL-compatible relational services
- Azure SQL
- REST API
- GraphQL API
- CSV and JSON import
- Webhook
- Message queue
- Object storage

Deliver adapter contract:

- Capability declaration
- Connection validation
- Schema discovery
- Preview
- Incremental read
- Bounded write where approved
- Cursor and checkpoint
- Batch and payload limits
- Health metrics
- Error classification

Acceptance:

- Unsupported capabilities fail explicitly.
- Table, collection, schema, and operation allowlists are enforced.
- Least-privilege credentials are required.
- Connector contract tests pass against supported versions.

# Phase CSP-4: Source-schema registry and mapping

Deliver:

- Source schema definitions and versions.
- Mapping profiles and versions.
- Direct, rename, constant, conditional, enum, split, combine, lookup, normalize, ignore, metadata, review, redact, and deny operations.
- Preview with validation results.
- Mapping diff and rollback.
- Environment-specific activation.
- Test fixtures.

Acceptance:

- Incompatible schema changes cannot silently reuse an old mapping.
- Every imported profile retains source schema and mapping versions.
- Unknown fields are preserved, reviewed, or rejected according to policy rather than disappearing.

# Phase CSP-5: Communicator normalization engine

Deliver:

```text
Source-native record
→ immutable intake
→ mapping
→ Communicator candidate
→ duplicate and identity matching
→ organization/person/contact profile
→ consent and channel eligibility
→ group evaluation
```

Work:

- Route current Harvester sync through the same generic intake boundary.
- Preserve source references and source-owned attributes.
- Prevent direct connector writes into campaign or conversation tables.
- Add conflict and ownership review.

Acceptance:

- Manual, database, API, Harvester, and Core sources use one governed intake pipeline.
- Product-specific source data remains attributed.
- Consent and suppression are evaluated independently of source.

# Phase CSP-6: Groups, filters, and frozen audiences

Deliver:

- Saved filters.
- Dynamic groups.
- Static groups.
- Group membership history.
- Audience preview.
- Frozen campaign audience.
- Exclusion and suppression evaluation.
- Audience approval.
- Membership explanation.

Acceptance:

- Dynamic group changes do not alter an already frozen campaign audience.
- Every audience member has an inclusion or exclusion explanation.
- Suppressed contacts cannot enter the send queue.

# Phase CSP-7: Communication Context Profile

Deliver:

- Identity summary.
- Organization and person summary.
- Source-owned context attributes.
- Recent conversation history.
- Questions, objections, promises, and preferences.
- Intent and sentiment.
- Relevant service and knowledge retrieval.
- Source references and freshness.
- Context sensitivity and model-use permissions.
- Context preview and audit.

Acceptance:

- High-quality replies can be generated without Core.
- The model receives relevant context rather than unrestricted database dumps.
- Every external context item is source-attributed and permissioned.

# Phase CSP-8: Direct Harvester adapter

Deliver:

- Harvester connection profile and credential configuration.
- Capability and schema negotiation.
- Minimal Contact, Standard Lead, Full Lead Context, and Custom payload profiles.
- Harvester source reference storage.
- Direct profile import without Core.
- Delivery, reply, bounce, complaint, opt-out, conversion, and invalid-contact feedback.
- Verification and refresh requests.
- Reconciliation.

Acceptance:

- Harvester and Communicator work together without Core.
- Communicator maps Harvester payloads into its own model.
- Harvester lead fields remain source-owned.
- Conversation history remains Communicator-owned.

# Phase CSP-9: Optional NEWAX Core adapter

Deliver:

- Core service identity and contract.
- Canonical organization/person link.
- Relationship and contact updates.
- Consent and suppression synchronization.
- Approved source facets.
- Controlled context requests.
- Merge and split notifications.
- Engagement and conversation-state event publication.
- Core outage backlog and reconciliation.

Acceptance:

- Communicator sends and receives messages while Core is unavailable.
- Core IDs improve identity but never replace local profile IDs.
- Context access remains optional and permissioned.

# Phase CSP-10: Synchronization engine

Deliver:

- Synchronization profiles and runs.
- Cursors and checkpoints.
- Transactional inbox and outbox.
- Retry, backoff, dead letter, circuit breaker, and backpressure.
- Manual, scheduled, polling, webhook, and event-driven modes.
- Change-data-capture only for approved adapters.
- Bidirectional field ownership and conflict review.
- Reconciliation and safe replay.

Acceptance:

- Message and inbound-reply transactions do not depend on destination availability.
- Retried events are idempotent.
- Sync runs resume from durable checkpoints.
- Backfill does not starve live inbound or outbound communication.

# Phase CSP-11: Administration UI

Routes:

```text
/settings/connections
/settings/credentials
/settings/source-schemas
/settings/mappings
/settings/synchronization
/integrations/harvester
/integrations/newax-core
/groups
/audiences
/context
```

Deliver:

- Connection list and health.
- Add and validate connection.
- Write-only credential replacement.
- Schema explorer.
- Mapping designer and preview.
- Synchronization history and recovery.
- Conflict review.
- Group builder.
- Audience preview and freeze.
- Context inspector.
- Harvester and Core integration pages.
- Audit timeline.

Acceptance:

- Disabled integrations do not break primary communication workflows.
- The UI never reveals stored secret values.
- Every action is tenant- and permission-scoped.

# Phase CSP-12: Multi-tenant resale foundation

Deliver:

- Tenant provisioning and isolation.
- Tenant-specific connections and credentials.
- Tenant-specific channel providers and sender identities.
- Tenant-specific service and knowledge catalog.
- Subscription and license state.
- Usage, channel, and model quotas.
- Feature flags.
- Branding, terminology, and custom domains.
- Data residency.
- Tenant export and deletion.
- Tenant backup and restore.
- Upgrade and migration compatibility.
- Controlled white-label configuration.

Acceptance:

- No hard-coded NEWAX or Harvester tenant assumptions.
- One tenant cannot access another tenant's profiles, messages, connections, credentials, schemas, mappings, groups, audiences, or context.
- License enforcement preserves customer export rights.

# Phase CSP-13: Packaging and operations

Deliver:

- Docker-compatible deployment package.
- Environment templates.
- Supported database and connector matrix.
- Channel-provider configuration guides.
- Migration and rollback guide.
- Backup and restore runbook.
- Observability and alerting.
- Backlog, dead-letter, and reconciliation operations.
- Data-retention and deletion runbooks.
- Support diagnostics without secret or message leakage.

# Phase CSP-14: Verification and rollout

Required E2E scenarios:

```text
Communicator only with manual profiles
Communicator with customer PostgreSQL
Communicator with customer CRM API
Direct Harvester-to-Communicator
Communicator + Core without Harvester
Full Harvester + Core + Communicator
Harvester outage
Core outage
Customer database outage
Duplicate event replay
Schema incompatibility
Mapping rollback
Hard bounce and re-verification
Opt-out propagation
Audience freeze during source changes
Cross-tenant attack attempt
```

Activation requires:

- Exact-head CI.
- Migration validation.
- Existing Phase 0–5 regression tests.
- Connector contract tests.
- Tenant-isolation tests.
- Consent and suppression tests.
- Secret-leak tests.
- Retry, dead-letter, backlog, resume, and reconciliation tests.
- Context authorization and redaction tests.
- Security review.
- Operational and support readiness.

# Relationship to existing phases

Existing Phases 0–5 remain the current Harvester-first implementation foundation:

```text
Phase 0  Foundations
Phase 1  Outbound MVP
Phase 2  Omnichannel inbound
Phase 3  AI reply engine
Phase 4  Automated follow-up and auto-send
Phase 5  Optimization
```

CSP-0 through CSP-14 generalize and productize that foundation. They do not erase the current working features or falsely claim that resale infrastructure already exists.

# Final acceptance

Communicator is ready for resale only when a customer can deploy it, connect approved sources, map profiles, create groups and audiences, communicate across configured channels, preserve complete conversation history, generate controlled replies, export data, back up and restore the system, and operate it without a Harvester or NEWAX Core account.