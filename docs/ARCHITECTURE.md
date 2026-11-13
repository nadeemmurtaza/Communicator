# Communicator Architecture and Delivery Plan

Status: **Current foundation plus planned standalone-product generalization**

## 1. Goal

Communicator enables an organization to connect approved customer and business data, define communication-ready profiles and audiences, send messages across supported channels, receive replies, preserve complete conversation history, generate grounded responses, follow up safely, and measure engagement.

Communicator must work as a standalone product.

Harvester and NEWAX Core are optional integrations. A customer may instead use manual entry, a CRM, an ERP, a relational database, a document database, an API, a file import, a message queue, or another approved connector.

## 2. Locked product boundary

Communicator owns:

- Its operational database and migrations
- Authentication, authorization, and tenant isolation
- Connection and credential profiles
- Source schemas, mappings, and normalization
- Organization, person, and contact profiles
- Contact-channel eligibility
- Consent and suppression
- Dynamic groups and static groups
- Frozen campaign audiences
- Campaigns and sequences
- Conversations and messages
- Delivery, reply, bounce, complaint, and engagement history
- Draft generation, approval, auto-send, and escalation
- Communication context
- Audit, retention, export, backup, and restore

Communicator does not require:

- Harvester for profile creation
- NEWAX Core for identity
- A NEWAX-owned database
- Harvester field names
- Core Registry field names
- Direct access to another application's tables

## 3. Product combinations

```text
Communicator only
Customer database, CRM, API, file, or manual entry
→ Communicator
→ groups, audiences, campaigns, conversations, and replies
```

```text
Harvester + Communicator
Harvester publishing rule
→ signed API, webhook, or event delivery
→ Communicator source adapter
→ Communicator mapping and normalization
```

```text
Communicator + NEWAX Core
Optional Core identity, relationships, source facets, and governed context
→ Communicator Core adapter
```

```text
Full NEWAX deployment
Harvester lead intelligence
+ optional Core identity and governance
+ Communicator communication and engagement
```

No deployment mode changes Communicator's ownership of messages, conversations, audiences, consent, or response context.

## 4. Formatting independence

The same external record may have different meanings in each product:

```text
Harvester
Observation, evidence, resolved lead entity, qualification, score, and dossier

NEWAX Core
Existence, canonical identity, relationships, provenance, and governed source facets

Communicator
Communication profile, channel, group, audience, conversation, and engagement
```

Communicator's normalization flow is:

```text
Source-native record
→ immutable intake record
→ source-schema validation
→ mapping
→ Communicator candidate
→ duplicate and identity matching
→ organization, person, and contact profile
→ consent and channel eligibility
→ group evaluation
```

A common outer event envelope may standardize delivery. It must not become a shared internal schema.

## 5. High-level architecture

```text
┌──────────────────────────────────────────────────────────────────────┐
│                         Approved Data Sources                         │
│                                                                      │
│ Customer DB  CRM/ERP  APIs  Files  Webhooks  Harvester  NEWAX Core │
└────────────────────────────────┬─────────────────────────────────────┘
                                 │
                                 ▼
┌──────────────────────────────────────────────────────────────────────┐
│                         Connector Gateway                            │
│                                                                      │
│ Connection profiles  Credential resolver  Adapter registry          │
│ Schema discovery      Capability validation  Checkpoints             │
└────────────────────────────────┬─────────────────────────────────────┘
                                 │
                                 ▼
┌──────────────────────────────────────────────────────────────────────┐
│                    Intake, Mapping, and Profiles                      │
│                                                                      │
│ Raw intake  Schema registry  Mapping engine  Normalization           │
│ Profile matching  Provenance  Consent  Suppression  Eligibility      │
└───────────────────────┬───────────────────────────────┬──────────────┘
                        │                               │
                        ▼                               ▼
┌─────────────────────────────────┐   ┌────────────────────────────────┐
│ Groups and Audiences            │   │ Communication Context          │
│                                 │   │                                │
│ Dynamic groups                  │   │ Identity and organization      │
│ Static groups                   │   │ Recent conversation history    │
│ Frozen campaign audiences       │   │ Objections and commitments     │
│ Inclusion/exclusion reasons     │   │ Source-owned external context  │
└────────────────┬────────────────┘   └────────────────┬───────────────┘
                 │                                     │
                 └──────────────────┬──────────────────┘
                                    ▼
┌──────────────────────────────────────────────────────────────────────┐
│                     Communication Orchestration                      │
│                                                                      │
│ Campaigns  Sequences  Conversations  Messages  Reply engine          │
│ Approval  Auto-send  Follow-up  Escalation  Audit                   │
└────────────────────────────────┬─────────────────────────────────────┘
                                 │
                                 ▼
┌──────────────────────────────────────────────────────────────────────┐
│                         Channel Providers                            │
│                                                                      │
│ Email  SMS  WhatsApp  Future approved channels                      │
└──────────────────────────────────────────────────────────────────────┘
```

## 6. Current foundation

The current codebase contains the Harvester-first communication path through Phases 0–5:

- Core conversation, message, contact, consent, sequence, draft, and audit models
- Harvester API client and lead synchronization
- Email, SMS, and WhatsApp sending adapters
- Inbound webhooks
- Unified message history
- Intent classification and draft generation
- Human approval
- Follow-up scheduling and confidence-gated auto-send
- Sequence variants
- Hot-lead alerts
- Language detection and translated follow-ups

This is a useful implementation foundation. It is not proof that generic connectors, multi-tenancy, resale packaging, source-schema mappings, or optional-Core behavior are complete.

## 7. Target profile model

The current Harvester-oriented `Lead` cache should evolve through migrations into communication-oriented profile concepts.

### `OrganizationProfile`

```text
id
tenant_id
display_name
organization_type
status
summary
primary_location
attributes
created_at
updated_at
```

### `PersonProfile`

```text
id
tenant_id
organization_profile_id
name
role
summary
attributes
created_at
updated_at
```

### `ContactChannel`

```text
id
person_profile_id
organization_profile_id
channel_type
address
verification_status
preferred
source_reference
created_at
updated_at
```

### `ExternalReference`

```text
id
tenant_id
profile_type
profile_id
source_system
source_installation
source_entity_id
source_schema
source_revision
last_synced_at
```

### `SourceAttribute`

Stores bounded source-owned context without pretending it is Communicator-owned truth.

```text
id
profile_type
profile_id
source_system
attribute_key
attribute_value
source_revision
freshness
sensitivity
model_use_allowed
```

Existing conversations and contacts require an explicit migration and compatibility strategy. Renaming a table and hoping every foreign key develops emotional resilience is not a strategy.

## 8. Connection profiles

Route:

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

Connection metadata:

```text
id
name
tenant_id
connector_type
engine_or_provider
environment
host
port
database_or_resource
schema_or_namespace
ssl_mode
network_mode
access_mode
secret_reference
status
capability_snapshot
last_validated_at
last_successful_sync_at
created_at
updated_at
```

Secret values remain outside normal connection metadata and are never returned after storage.

## 9. Credential system

Production secret resolution order:

```text
Deployment secret manager
→ encrypted application vault
→ explicitly enabled local development adapter
```

Supported secret material may include:

- Username and password
- API key
- OAuth client and refresh credentials
- Bearer token
- Private key
- Certificate
- SSH tunnel material
- Webhook signing secret
- Channel-provider credentials

The frontend shows only configured state, status, environment, last validation, last rotation, and a short fingerprint.

## 10. Database adapters

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

Later adapters may include Oracle, DynamoDB, Firebase, Elasticsearch, Snowflake, BigQuery, ClickHouse, and approved customer-specific systems.

"Any database" means an adapter framework with certified capabilities. It does not mean arbitrary access to every engine, version, stored procedure, trigger, extension, and home-grown schema humans have accumulated since the invention of regret.

## 11. Adapter contract

```python
class ConnectionAdapter:
    connector_type: str
    adapter_version: str

    async def validate(self, profile, credential) -> ValidationResult: ...
    async def discover_schema(self, profile) -> SourceSchema: ...
    async def preview(self, profile, cursor=None) -> PreviewBatch: ...
    async def read(self, profile, cursor=None) -> SourceBatch: ...
    async def write(self, profile, records) -> WriteResult: ...
    async def checkpoint(self, run) -> Checkpoint: ...
```

Every adapter declares:

- Engine and supported versions
- Read, write, polling, webhook, export, and CDC capability
- Schema discovery behavior
- Transaction behavior
- Cursor and checkpoint semantics
- Identifier and timestamp handling
- Batch and payload limits
- TLS and network requirements
- Safe validation operation
- Required permissions

Default access is read-only.

## 12. Source-schema registry

```text
SourceSchemaDefinition
SourceSchemaVersion
ConnectorVersion
MappingProfile
MappingVersion
NormalizationVersion
CompatibilityState
```

Examples:

```text
customer_crm.contacts.v3
customer_postgres.accounts.v2
harvester.full_lead_context.v1
core.communication_context.v1
partner_api.person.v4
```

Unknown or incompatible schemas enter quarantine or review. They do not receive the charitable assumption that similarly named fields mean the same thing.

## 13. Mapping engine

Supported operations:

- Direct mapping
- Rename
- Constant
- Conditional mapping
- Enumeration mapping
- Split
- Combine
- Lookup
- Normalize
- Ignore
- Preserve as source metadata
- Require review
- Redact
- Deny import

Example CRM mapping:

```text
crm.account_name
→ organization profile display name

crm.primary_email
→ email contact channel

crm.do_not_contact
→ suppression status

crm.account_owner
→ source-owned relationship context
```

Example Harvester mapping:

```text
harvester.organization_name
→ organization profile display name

harvester.person_name and role
→ person profile

harvester.verified_email
→ email contact channel

harvester.lead_score
→ source-owned lead context

harvester.dossier_reference
→ external context reference
```

Mappings are versioned, previewable, testable, auditable, and reversible.

## 14. Intake pipeline

```text
Connection profile
→ credential resolution
→ adapter validation
→ source schema discovery
→ source-native read
→ immutable intake persistence
→ schema validation
→ mapping
→ Communicator normalization
→ duplicate and identity matching
→ profile services
→ consent and channel eligibility
→ group evaluation
→ local commit
→ optional outbound event
```

Optional outbound integration never participates in the local profile or message transaction.

## 15. Synchronization modes

Target modes:

```text
Manual import
Scheduled import
Continuous polling
Webhook-triggered synchronization
Change-data-capture where approved
Manual export
Scheduled export
Event-driven publication
Bidirectional synchronization for explicitly approved fields
```

Every run stores:

- Connection, schema, mapping, and normalization versions
- Cursor or checkpoint
- Records examined
- Created, updated, skipped, quarantined, and rejected counts
- Conflict decisions
- Retry and recovery state
- Actor or scheduler identity
- Error details without secrets

## 16. Groups and filters

### Dynamic group

A saved query evaluated against current Communicator profile data.

### Static group

A manually maintained collection.

### Frozen campaign audience

An immutable snapshot created when a campaign is approved.

```text
Profile data
→ filter evaluation
→ dynamic or static group
→ audience preview
→ suppression and eligibility checks
→ approval
→ frozen campaign audience
```

Each inclusion and exclusion must be explainable.

## 17. Campaigns and conversations

A campaign references a frozen audience, objective, sequence, channels, schedule, limits, approval state, and sending policy.

A conversation belongs to a communication profile and channel context. Conversation history remains local even when the originating source is disconnected.

Current state examples:

```text
NEW
AWAITING_REPLY
ENGAGED
HOT_LEAD
HANDED_OFF
OPTED_OUT
CLOSED_WON
CLOSED_LOST
```

State names and transitions should become configurable where resale requirements justify it, without weakening compliance states.

## 18. Channel providers

A stable `ChannelProvider` interface supports sending, inbound parsing, delivery updates, identity validation, and capability metadata.

Initial channels:

- Email
- SMS
- WhatsApp

Future channels require explicit legal, security, webhook, identity, and opt-out review before activation.

Channel-specific rules remain explicit. The application must not flatten legally different channels into one optimistic `send()` button and let compliance clean up afterward.

## 19. Consent and suppression

Consent and opt-out are evaluated before model logic and before every send.

Required behavior:

- Per profile and channel status
- Source and evidence
- Effective time
- Tenant scope
- Global and campaign-specific suppression where policy permits
- STOP and unsubscribe handling
- Complaint handling
- Conflict resolution across connected sources
- Audit and export

Core synchronization may share suppression outcomes when configured. Communicator remains capable of enforcing suppression locally without Core.

## 20. Communication Context Profile

```text
CommunicationContextProfile
├── Identity summary
├── Organization summary
├── Person and role summary
├── Contact-channel state
├── Source-owned lead or relationship context
├── Recent conversation history
├── Questions and objections
├── Preferences and commitments
├── Current intent and sentiment
├── Relevant service or knowledge context
├── Consent and suppression
└── Source references and freshness
```

Context sources:

- Local profiles
- Local conversation history
- Service and knowledge catalog
- Harvester adapter
- Core adapter
- Customer CRM or ERP
- Customer knowledge base
- Approved imported records

Context retrieval is layered:

1. Identity and current conversation summary
2. Recent messages
3. Open questions, objections, promises, and preferences
4. Relevant organization and source context
5. Older records retrieved on demand
6. Raw source material available for verification

Every context item has source, freshness, sensitivity, retention, purpose, and model-use permission.

## 21. Reply engine

The current reply engine remains conceptually valid but receives context through the generalized context builder.

```text
Inbound message
→ deterministic compliance gate
→ intent and risk classification
→ context assembly
→ grounded draft
→ confidence and citation record
→ approval or permitted auto-send
→ message dispatch
→ audit
```

Hard escalation examples:

- Pricing or contractual commitments outside approved knowledge
- Complaint or legal threat
- Sensitive-data request
- Uncertain identity
- Conflicting source context
- Low-confidence answer
- High-value buying signal requiring a person

## 22. Follow-up engine

Follow-up remains conversation-owned and source-independent.

It evaluates:

- Conversation state
- Last inbound and outbound activity
- Sequence step
- Quiet hours and timezone
- Weekly and channel limits
- Consent and suppression
- Provider restrictions
- Campaign state
- Human handoff

A disconnected source must not erase scheduled work or conversation history.

## 23. Optional Harvester integration

Harvester may publish:

- Minimal Contact
- Standard Lead
- Full Lead Context
- Custom mapped payload

Communicator maps these payloads into local profiles. It does not treat Harvester's model as its own.

Feedback to Harvester may include:

- Sent
- Delivered
- Reply received
- Positive reply
- Hard or soft bounce
- Complaint
- Opt-out
- Invalid contact
- Closed won or lost
- Verification or refresh request

Direct integration works without Core.

## 24. Optional NEWAX Core integration

Core may provide:

- Canonical organization and person IDs
- Relationships
- Contact-channel updates
- Consent and suppression state
- Customer or partner context
- Approved source facets
- Controlled context packages
- Merge and split notifications

Communicator may publish:

- Communication profile links
- Contact-channel state
- Consent and suppression outcomes
- Delivery, bounce, complaint, reply, and engagement events
- Conversation-state summaries

Core is not required to preserve a message, create a draft, or send a reply.

## 25. Reliability

Use:

- Transactional local outbox
- Durable inbound inbox
- Stable event and command IDs
- Payload hashes
- Idempotent processing
- Exponential retry with jitter
- Dead letter
- Circuit breaker
- Backpressure
- Durable checkpoints
- Reconciliation

No distributed transaction spans Communicator and another product.

## 26. Security

- Tenant-scoped connections and data access
- Least-privilege credentials
- Write-only secrets
- TLS validation
- SSRF protections
- Parameterized and adapter-owned queries
- Schema, table, collection, and operation allowlists
- Payload and response limits
- Signed product events
- Timestamp and replay validation
- Consent checks before sending
- Context authorization before model use
- Redaction and sensitivity controls
- Audit without secrets
- No shared database credentials

## 27. Resale architecture

Required capabilities:

- Tenant provisioning and isolation
- Tenant-specific connection and channel credentials
- Tenant-specific sender identities
- Tenant-specific service and knowledge catalog
- Subscription and license state
- Usage, channel, and model quotas
- Feature flags
- Branding and terminology
- Custom domains
- Data residency
- Import and export portability
- Retention and deletion
- Backup and restore
- Upgrade compatibility
- Controlled white-labeling

NEWAX remains the default product brand. White-labeling does not permit weakening consent, audit, or security controls.

## 28. Administration routes

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
/campaigns
/conversations
/drafts
/operations
```

## 29. Delivery plan

The current Phase 0–5 implementation remains the foundation.

Standalone-product generalization is delivered through the roadmap in `docs/ROADMAP.md`:

```text
CSP-0   Product boundary and migration inventory
CSP-1   Generic profile foundation
CSP-2   Connection and credential foundation
CSP-3   Database and source adapter kernel
CSP-4   Source-schema registry and mapping
CSP-5   Communicator normalization engine
CSP-6   Groups, filters, and frozen audiences
CSP-7   Communication Context Profile
CSP-8   Direct Harvester adapter
CSP-9   Optional NEWAX Core adapter
CSP-10  Synchronization engine
CSP-11  Administration UI
CSP-12  Multi-tenant resale foundation
CSP-13  Packaging and operations
CSP-14  Verification and rollout
```

## 30. Acceptance criteria

The target architecture is not complete until:

1. Communicator operates with Harvester and Core disabled.
2. Profiles can originate from manual entry or approved customer systems.
3. Each source is mapped through a versioned Communicator mapping.
4. Source-native records are preserved before transformation.
5. External schemas do not become the internal schema.
6. Consent and suppression are source-independent and enforced before every send.
7. Dynamic groups and frozen audiences remain distinct.
8. Conversation history survives source disconnection.
9. Strong reply context can be assembled without Core.
10. Direct Harvester integration works without Core.
11. Optional Core integration works without Harvester.
12. Customer databases use certified, least-privilege adapters.
13. Optional integration outages create backlogs, not message loss.
14. Product events are idempotent and replay-safe.
15. Context access is permissioned and auditable.
16. Tenants remain isolated across data, connections, messages, mappings, groups, and context.
17. Backup, restore, export, deletion, licensing, and upgrade paths are verified.
18. Exact-head tests, migrations, security checks, runtime scenarios, and CI pass before activation.

## 31. Locked rule

> Communicator is the communication and relationship product. Sources may enrich its profiles and context, but Communicator independently owns how data becomes an audience, a conversation, and a response.