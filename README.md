# Communicator

Communicator is a standalone, multi-channel communication and relationship application.

It connects customer data, contact channels, conversation history, approved business context, and communication automation so organizations can manage audiences, campaigns, replies, follow-ups, consent, and engagement from one controlled system.

> **Product principle:** Bring the right context to every conversation while keeping communication reliable, permissioned, and under operational control.

## Product boundary

Communicator is a separate, resellable application with its own:

- Database and migrations
- Authentication, authorization, and tenant isolation
- Connection and credential system
- Source-schema registry
- Mapping and normalization engine
- Organization, person, and contact profiles
- Groups and frozen campaign audiences
- Conversations and message history
- Consent and suppression
- Drafting, approval, sending, follow-up, and escalation
- Audit, retention, backup, restore, and export

Communicator does not require Harvester or NEWAX Core.

Both are optional integrations:

```text
Communicator only
Customer database, CRM, API, file, or manual entry
→ Communicator
→ groups, audiences, campaigns, conversations, and replies

Harvester + Communicator
Harvester publishing rule
→ direct adapter
→ Communicator mapping and normalization

Communicator + NEWAX Core
Optional Core identity and governed context
→ Communicator

Full NEWAX deployment
Harvester lead intelligence
+ optional Core identity and governance
+ Communicator conversation and engagement
```

## Independent formatting

Communicator formats incoming information according to communication needs:

```text
Source-native record
→ raw intake record
→ mapped organization, person, or contact candidate
→ Communicator profile
→ channel eligibility
→ group membership
→ frozen campaign audience
→ conversation and engagement
```

This remains separate from:

```text
Harvester
Observations, evidence, resolved leads, qualification, scoring, and research

NEWAX Core
Existence, canonical identity, relationships, provenance, and governed source facets
```

A shared source or common event envelope does not create a shared internal database schema.

## Data sources and connections

Communicator is planned to support credentialed adapters for:

- PostgreSQL
- MySQL
- MariaDB
- Microsoft SQL Server
- SQLite for bounded local use
- MongoDB
- Supabase
- Neon
- Amazon RDS-compatible relational services
- Google Cloud SQL-compatible relational services
- Azure SQL
- REST APIs
- GraphQL APIs
- CSV and JSON files
- Webhooks
- Message queues
- Object storage
- Harvester
- NEWAX Core
- Customer CRM and ERP systems
- Approved custom connectors

"Any database" is implemented through versioned adapters with explicit capabilities, least-privilege credentials, schema discovery, mapping, checkpoints, validation, and audit. It is not permission to execute improvised SQL against whatever server happens to answer.

## Groups, filters, and audiences

Communicator owns audience selection:

- **Dynamic group:** a saved filter whose membership changes with profile data.
- **Static group:** a manually curated collection.
- **Frozen campaign audience:** the approved recipient snapshot used by a campaign.

```text
Connected source
→ Communicator mapping
→ filters
→ dynamic or static group
→ review
→ frozen campaign audience
→ send workflow
```

Source changes never silently rewrite an audience after a campaign begins.

## Communication context

Communicator builds a local `CommunicationContextProfile` from permitted sources:

- Identity and organization summary
- Person and role summary
- Contact-channel state
- Source-owned lead or relationship context
- Recent conversation history
- Questions, objections, promises, and preferences
- Intent and sentiment
- Relevant product, service, or knowledge context
- Consent and suppression
- Source references and freshness

Communicator can build strong replies without Core. When Harvester or Core is connected, their approved context is added through adapters rather than by reading their databases.

## Current implementation

The current repository contains the Harvester-first communication path through Phases 0–5:

- **Phase 0: Foundations** — data model, migrations, Harvester API client, compliance gate, and audit.
- **Phase 1: Outbound MVP** — email, SMS, WhatsApp adapters, lead synchronization, sending, and Harvester outcome callbacks.
- **Phase 2: Omnichannel inbound** — provider webhooks and unified conversation history.
- **Phase 3: AI reply engine** — service catalog, intent classification, draft generation, and human approval.
- **Phase 4: Automated follow-up and auto-send** — scheduled follow-ups, limits, quiet hours, and confidence-gated dispatch.
- **Phase 5: Optimization** — sequence variants, hot-lead alerts, language detection, and translated follow-ups.

These capabilities are current foundations. The generic connector, multi-source mapping, standalone profile, resale, and optional-Core architecture remains planned until implemented and verified.

## Running locally

```bash
python -m venv .venv
source .venv/bin/activate
pip install -e ".[dev]"
cp .env.example .env

alembic upgrade head
uvicorn communicator.api.app:app --reload
pytest
```

A local PostgreSQL connection is currently required through `DATABASE_URL`. The broader connection and credential framework is planned work.

## Documentation

- [Architecture and Delivery Plan](docs/ARCHITECTURE.md)
- [Standalone Product Plan](docs/STANDALONE-PRODUCT-PLAN.md)
- [Roadmap](docs/ROADMAP.md)

## Locked rule

> Communicator owns communication profiles, groups, audiences, conversations, consent, engagement, and response context. Harvester and NEWAX Core may enrich it through optional adapters, but neither is required for Communicator to operate.
