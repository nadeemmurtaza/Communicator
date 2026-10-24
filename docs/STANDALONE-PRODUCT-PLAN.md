# Communicator Standalone Product Plan

Status: **Planned architecture and product generalization**

## 1. Product decision

Communicator is a standalone, resellable communication application.

It must be installable, configured, operated, upgraded, backed up, restored, licensed, and supported without requiring Harvester or NEWAX Core.

Harvester and NEWAX Core are optional integrations. A customer may instead connect Communicator to a CRM, ERP, customer database, REST API, GraphQL API, file import, message queue, object storage source, or another approved system.

> Communicator owns communication profiles, audiences, campaigns, conversations, messages, consent, suppression, engagement, and response context. External sources provide data, but they do not define Communicator's internal model.

## 2. Standalone operating modes

```text
Communicator only
Customer database, CRM, API, file, or manual entry
→ Communicator
→ groups, audiences, campaigns, conversations, and replies

Harvester + Communicator
Harvester publishing rule
→ optional direct adapter
→ Communicator mapping and normalization

Communicator + NEWAX Core
Core identity and governed context
→ optional Core adapter
→ Communicator

Full NEWAX deployment
Harvester lead intelligence
+ optional Core identity and governance
+ Communicator conversations and engagement
```

Every mode must preserve the same Communicator capabilities. Integrations improve context or reduce manual work; they do not unlock the basic ability to communicate.

## 3. Product ownership

Communicator owns:

- Organizations and people as communication context
- Contact profiles and contact channels
- Consent and suppression
- Dynamic groups
- Static groups
- Frozen campaign audiences
- Campaigns and sequences
- Conversations and messages
- Delivery, reply, bounce, complaint, and engagement history
- Drafting, approval, auto-send, and escalation
- Communication context profiles
- Communication-specific source mappings
- Communication audit and retention

Communicator does not own:

- Harvester raw research evidence and qualification logic
- NEWAX Core canonical identity governance
- Customer CRM's private internal schema
- Another application's database migrations

## 4. Communicator formatting

Communicator formats incoming information for communication work:

```text
Source-native record
→ raw intake record
→ mapped organization/person/contact candidate
→ Communicator profile
→ channel eligibility
→ group membership
→ frozen audience member
→ conversation and engagement
```

Communicator's internal model may contain:

- Organization context
- Person and role context
- Contact channel and verification state
- Consent and suppression
- Source references
- Profile attributes
- Group memberships
- Campaign participation
- Conversation summaries
- Message history
- Objections and questions
- Preferences and commitments
- Sentiment and intent
- Follow-up state
- Communication outcomes

The same source record may be interpreted differently elsewhere:

```text
Harvester
Lead evidence, qualification, score, opportunity signals, and dossier

NEWAX Core
Existence, identity, relationships, provenance, and governed source facets

Communicator
Reachable profile, audience eligibility, conversation, engagement, and reply context
```

## 5. Connection and credential system

Communicator requires a general connection framework at:

```text
/settings/connections
```

Connection categories:

```text
Database
REST API
GraphQL API
Webhook
Message queue
Object storage
File import
Harvester
NEWAX Core
CRM or ERP
Custom connector
```

A connection profile stores non-secret metadata:

```text
id
name
connector_type
engine_or_provider
environment
host
port
database_or_resource
schema_or_namespace
ssl_mode
access_mode
secret_reference
status
capability_snapshot
last_validated_at
last_successful_sync_at
created_at
updated_at
```

Secrets remain encrypted and write-only after submission. The browser may show configured state and a short fingerprint, never the stored credential.

## 6. Database connector framework

Supporting "any database" requires pluggable adapters with declared capabilities.

Initial relational adapters:

- PostgreSQL
- MySQL
- MariaDB
- Microsoft SQL Server
- SQLite for bounded local use

Initial document and managed-service adapters:

- MongoDB
- Supabase
- Neon
- Amazon RDS-compatible relational services
- Google Cloud SQL-compatible relational services
- Azure SQL

Later adapters may cover Oracle, DynamoDB, Firebase, Elasticsearch, Snowflake, BigQuery, ClickHouse, and approved customer-specific systems.

Every adapter declares:

- Engine and supported versions
- Read, write, polling, webhook, export, or change-data-capture support
- Schema discovery
- Cursor and checkpoint semantics
- Transaction behavior
- Identifier and timestamp behavior
- Batch and payload limits
- Safe validation query
- Required permissions
- TLS and network requirements

Default database access is read-only. Write-back, bidirectional synchronization, and change-data-capture require explicit configuration, least-privilege credentials, conflict policy, and approval.

## 7. Independent schema registry

Communicator owns a source-schema registry:

```text
source_system
source_schema_name
source_schema_version
connector_version
mapping_version
normalization_version
compatibility_state
active_from
active_until
```

Examples:

```text
customer_crm.contacts.v3
customer_postgres.accounts.v2
harvester.full_lead_context.v1
core.communication_context.v1
partner_api.person.v4
```

The source schema describes what arrived. It does not become Communicator's internal schema.

## 8. Mapping engine

Every source uses a versioned Communicator-owned mapping.

Supported operations:

- Direct mapping
- Rename
- Constant
- Conditional mapping
- Enumeration mapping
- Split
- Combine
- Lookup table
- Normalize
- Ignore
- Preserve as source metadata
- Require review
- Redact
- Deny import

Example:

```text
crm.primary_email
→ contact channel: email

crm.do_not_contact
→ suppression status

crm.account_name
→ organization display context

harvester.lead_score
→ source-owned lead context attribute

core.organization_id
→ optional external identity link
```

Mappings must support preview, fixtures, validation, versioning, rollback, audit, and environment-specific activation.

## 9. Independent normalization engine

```text
Source-native record
→ immutable intake record
→ mapping
→ Communicator normalization
→ duplicate and identity matching
→ organization/person/contact profile
→ consent and eligibility
→ group evaluation
```

Communicator must never write incoming records directly into campaign, conversation, or message tables without profile validation and source attribution.

## 10. Groups, filters, and audiences

Communicator owns audience selection.

### Dynamic group

A saved filter whose membership changes as profile data changes.

Example:

```text
organization_type = hospital
city = Islamabad
lead_score >= 70
verified_email = true
suppressed = false
```

### Static group

A manually curated or imported collection whose membership changes only through explicit action.

### Frozen campaign audience

An immutable snapshot of approved recipients when a campaign begins.

```text
Source data
→ Communicator mapping
→ filters
→ dynamic or static group
→ review
→ frozen campaign audience
→ send workflow
```

Campaign membership remains auditable even when the source or dynamic group changes later.

## 11. Communication Context Profile

Communicator must generate strong replies without requiring Core.

```text
CommunicationContextProfile
├── Identity summary
├── Organization summary
├── Person and role summary
├── Source-owned lead or relationship context
├── Previous conversations
├── Questions and objections
├── Preferences and commitments
├── Current intent and sentiment
├── Relevant service or knowledge context
├── Consent and suppression
└── Source references and freshness
```

Context may come from:

- Communicator's local profile and conversation history
- Harvester lead context
- NEWAX Core identity and relationship context
- Customer CRM or ERP
- Customer knowledge base
- Approved imported records
- Product and service catalog

The reply engine receives a layered working context rather than every raw row and message.

Recommended retrieval order:

1. Identity and channel summary
2. Current conversation state
3. Recent messages
4. Open questions, objections, promises, and preferences
5. Relevant organization or lead context
6. Older records retrieved only when needed
7. Raw source evidence available for verification

## 12. Optional Harvester adapter

Harvester may publish selected data through a configured rule:

- Minimal Contact
- Standard Lead
- Full Lead Context
- Custom Mapping

Communicator receives source-native Harvester payloads and maps them into its own profiles.

Communicator must not assume:

- Every profile originated in Harvester.
- Every organization is a qualified lead.
- Harvester IDs are universal IDs.
- Harvester's canonical model is Communicator's data model.

Communicator may send back:

- Delivery outcomes
- Replies
- Hard and soft bounces
- Complaints
- Opt-outs
- Invalid contact results
- Conversion or handoff outcomes
- Verification or refresh requests

Harvester remains owner of lead intelligence. Communicator remains owner of conversation history.

## 13. Optional NEWAX Core adapter

When configured, Core may provide:

- Canonical organization and person IDs
- Relationships
- Contact-channel updates
- Consent and suppression state
- Customer or partner relationship context
- Approved source facets
- Controlled communication context
- Merge and split notifications

Communicator may publish:

- Communication profile links
- Contact-channel status
- Consent and suppression outcomes
- Delivery, bounce, complaint, reply, and engagement events
- Conversation-state summaries

Core is not required to send, receive, draft, or preserve a conversation.

## 14. Synchronization modes

Target modes:

```text
Manual import
Scheduled import
Continuous polling
Webhook-triggered synchronization
Change-data-capture where safely supported
Manual export
Scheduled export
Event-driven publication
Bidirectional synchronization for explicitly approved fields
```

Every synchronization run records:

- Source and destination
- Connection, schema, mapping, and normalization versions
- Cursor or checkpoint
- Created, updated, skipped, quarantined, and rejected counts
- Conflicts and ownership decisions
- Retry and dead-letter state
- Timing and actor
- Error details without secrets

## 15. Context and data minimization

Communicator may need broad history for excellent responses, but it should not copy every available record or send unrestricted history to an AI model.

Every context source declares:

- Purpose
- Allowed fields
- Sensitivity
- Retention
- Freshness
- Retrieval conditions
- Model-use permission
- Source attribution

Raw personal data and full external datasets are not automatically eligible for AI context.

## 16. Multi-tenant and resale requirements

A resellable Communicator requires:

- Tenant isolation
- Tenant-owned connections and credentials
- Tenant-specific channels and sender identities
- Tenant-specific service and knowledge catalog
- Configurable branding and terminology
- Subscription and license state
- Usage, channel, and model quotas
- Feature flags
- Custom domains
- Import and export portability
- Tenant retention policies
- Tenant backup and restore
- Audit export
- Data residency configuration
- No hard-coded NEWAX or Harvester tenant assumptions

NEWAX remains the default product brand. Controlled enterprise white-labeling may be supported without weakening audit, consent, or security controls.

## 17. Administration pages

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

The pages show connection metadata, health, capabilities, mappings, synchronization history, conflicts, backlog, and recovery. They never reveal stored secrets.

## 18. Security requirements

- Separate runtime, read-only, write, migration, and backup credentials.
- Tenant-scoped connection resolution.
- Parameterized queries and adapter-owned query generation.
- Table, collection, schema, and operation allowlists.
- TLS validation and approved network destinations.
- SSRF protections for URL-based connectors.
- Payload and response limits.
- HMAC or asymmetric signing for product events.
- Event idempotency, replay protection, and payload hashing.
- Consent and suppression checks before every send.
- Credential rotation without revealing previous values.
- Audit logs without secret values.
- Context authorization before model access.
- No automatic write-back without ownership and conflict policy.

## 19. Acceptance criteria

The standalone architecture is not complete until:

1. Communicator starts and performs core messaging workflows with Harvester and Core disabled.
2. Profiles can be created through manual entry or an approved customer source.
3. Each tenant owns independent connections, credentials, mappings, cursors, groups, and audit records.
4. Source-native data is preserved before transformation.
5. Communicator mappings are versioned and previewable.
6. External schemas do not become Communicator's internal schema.
7. Optional integration failure cannot roll back a committed message or inbound reply.
8. Consent and suppression are enforced regardless of source.
9. Dynamic groups and frozen audiences are distinct.
10. Full reply context can be assembled without Core.
11. Direct Harvester integration works without Core.
12. Core integration works without Harvester.
13. Customer databases and APIs can be used through approved adapters.
14. Product-specific formats remain independent.
15. Secrets remain encrypted and write-only.
16. Retries are idempotent and dead-lettered when exhausted.
17. Reconciliation detects missing or stale source links.
18. Multi-tenant isolation, licensing, backup, restore, and upgrade paths are verified.
19. Exact-head tests, security checks, migrations, runtime workflows, and CI pass before activation.

## 20. Locked rule

> Communicator is a standalone communication and relationship application. It can connect to many sources, including Harvester and NEWAX Core, but it always maps and normalizes information according to Communicator's own profile, audience, conversation, consent, and engagement model.