# Harvester Keyword Context Contract for Communicator

Status: **Proposed**  
Version: **1.0**  
Date: **4 August 2026**  
Implementation status: **Not implemented**

This document defines how Communicator may consume Harvester classification and opportunity context without importing Harvester's complete discovery keyword registry.

Communicator owns communication. Harvester owns discovery and opportunity judgment. Keeping those responsibilities separate prevents messaging software from quietly becoming an unreviewed lead-scoring engine, which would be efficient only at producing confusion.

## 1. Product boundary

Harvester owns:

- broad-sector, product-or-service-domain, and activity classification;
- domain and activity keyword profiles;
- exclusions and negative context;
- provider offering selection;
- evidence assessment;
- supported opportunity and lead qualification decisions.

Communicator owns:

- profiles and source references needed for communication;
- groups and frozen audiences;
- consent and communication eligibility;
- campaigns and message preparation;
- conversations, replies, engagement, and relationship context;
- delivery status, complaints, suppression inputs, and channel outcomes.

NEWAX Core is an optional integration and governance layer. Communicator must remain able to receive Harvester context directly.

## 2. What Communicator may receive

A qualified Harvester context package may include:

| Field | Communication purpose |
| --- | --- |
| `source_product` and `source_installation_id` | Source ownership and routing |
| `source_event_id` | Idempotent replay |
| `lead_id` or `organization_id` | Harvester target reference |
| `contact_reference_ids` | Bounded contact linkage |
| `sector_codes` and labels | Sector-aware communication context |
| `domain_codes` and labels | Product-or-service context |
| `activity_codes` and labels | What the target appears to do |
| `provider_offering_codes` | Services the provider actually offers |
| `opportunity_codes` | Supported Harvester opportunity context |
| `approved_outreach_themes` | Reviewed themes safe for message preparation |
| `evidence_summary` | Bounded explanation for human review |
| `confidence_dimensions` | Strength and limitations of the evidence |
| `prohibited_claims` | Statements the message must not make |
| `qualification_notes` | Bounded operational context |
| `locale` and `language_preferences` | Communication-language guidance |
| `taxonomy_version` | Sector, domain, and activity registry version |
| `keyword_registry_version` | Harvester keyword version used |
| `assessment_version` | Harvester decision-policy version |
| `assessed_at` and `expires_at` | Freshness and refresh boundary |
| `consent_state` and `suppression_state` | Communication eligibility inputs |

Communicator may transform approved context into a draft message according to its own versioned templates and review rules. It must not reinterpret the underlying opportunity as stronger than Harvester reported.

## 3. What Communicator must not import by default

Communicator must not copy or persist:

- all 1,586 domain keyword profiles;
- all 519 activity keyword profiles;
- complete exclusion dictionaries;
- internal keyword weights or ranking logic;
- semantic prompts, embeddings, or source-query plans;
- unrestricted competitor or existing-solution profiles;
- unrestricted evidence documents or website snapshots;
- search credentials, provider tokens, or private Harvester configuration.

The full registry remains a Harvester-owned dataset.

## 4. Messaging restrictions

Communicator must not:

1. treat a keyword match as proof that an organization lacks a service;
2. generate a need claim from a sector or domain label alone;
3. mention private discovery terms in customer-facing copy merely because they matched internally;
4. state that a competitor, system, or process exists without supported evidence;
5. describe uncertain evidence as confirmed absence;
6. bypass consent, suppression, complaint, quiet-hours, frequency, or channel rules;
7. reactivate an expired opportunity without refreshed Harvester context;
8. silently alter Harvester confidence or opportunity classifications.

## 5. Approved outreach themes

Harvester should publish reviewed themes rather than raw keyword lists.

Example:

```text
Sector: Healthcare
Domain: Pharmaceuticals
Activity: Distribution
Provider offering: Distributor Management System
Supported opportunity: Fragmented dealer and order operations

Approved outreach themes:
- distributor visibility
- order coordination
- dealer workflow consistency
- stock and delivery visibility

Prohibited claims:
- "Your company has no distributor system"
- "You are losing revenue"
- "Your current software is obsolete"
```

The message should invite an operational discussion, not diagnose the target with theatrical confidence.

## 6. Direct and Core-mediated modes

Communicator must support both:

```text
Harvester
→ Communicator
```

and:

```text
Harvester
→ NEWAX Core optional projection
→ Communicator
```

Both paths must preserve Harvester source IDs, versions, confidence, expiry, and opportunity ownership.

## 7. Audience and campaign rules

A Harvester-qualified context package may make a profile eligible for audience evaluation, but it must not automatically send a message.

Before activation, Communicator must still evaluate:

- tenant and workspace scope;
- current contact validity;
- consent and lawful basis where applicable;
- suppression and complaint state;
- channel eligibility;
- frequency and quiet-hour rules;
- campaign eligibility;
- template and claim approval;
- human review requirements;
- context freshness.

A frozen audience must preserve the Harvester context and Communicator policy versions used when it was created.

## 8. Feedback to Harvester

Communicator may return bounded outcome signals such as:

- delivered;
- bounced;
- complaint received;
- replied;
- positive, neutral, or negative disposition;
- opted out;
- wrong contact;
- wrong organization;
- irrelevant opportunity;
- meeting requested;
- no response after policy-defined completion.

Feedback creates new evidence. It must not rewrite Harvester's historical source evidence or keyword profile silently.

## 9. Security and privacy

- Communicator receives the minimum context required for communication.
- Free text and source evidence must be redacted and bounded.
- Sensitive personal information must not be copied from discovery sources without a valid purpose.
- Logs must not contain complete Harvester keyword profiles or private evidence payloads.
- Cross-product requests require authenticated application identities, tenant restrictions, replay protection, and audit.
- Deletion and retention rules remain independently enforceable in each product.

## 10. Acceptance criteria

- [x] Communicator receives qualified context rather than complete raw keyword registries.
- [x] Harvester remains authoritative for classification and opportunity decisions.
- [x] Communicator remains authoritative for consent, messaging, engagement, and conversations.
- [x] Direct integration works without Core.
- [x] Core-mediated integration preserves source ownership.
- [x] Message preparation uses approved themes and prohibited claims.
- [x] Keyword matches are never presented as proof of need.
- [x] Audience creation does not automatically authorize sending.
- [x] Feedback creates new evidence without rewriting history.
- [x] Runtime implementation is not falsely claimed.

## 11. Delivery boundary

This document defines a planned communication-context contract only.

It does not implement adapters, events, APIs, database tables, audience filters, message generation, consent evaluation, delivery, feedback synchronization, or production activation.
