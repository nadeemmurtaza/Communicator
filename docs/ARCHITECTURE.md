# Communicator — Architecture & Delivery Plan

## 1. Goal

Given a lead sitting in Harvester's database, Communicator should be able to,
without a human touching every message:

1. **Fetch** the lead and its contact points (email / phone / WhatsApp) from
   Harvester.
2. **Send** a first-touch message on the appropriate channel(s).
3. **Receive** replies on that same channel (inbound email, inbound SMS,
   inbound WhatsApp).
4. **Respond** using the full conversation history, the current message, and
   a catalog of the services we sell — answering questions, handling
   objections, and routing hot leads to a human.
5. **Follow up automatically** on a cadence if the lead goes quiet, and stop
   cleanly the moment they reply, convert, or opt out.
6. **Report outcomes back to Harvester** so lead scoring and sales visibility
   stay in one place.

Everything below is designed to be built incrementally (see §9, Roadmap) —
none of it needs to exist on day one to ship value.

## 2. Non-negotiable constraints

These shape the design more than anything else:

- **Consent and opt-out are handled before any AI logic runs.** STOP/
  UNSUBSCRIBE-class replies on any channel immediately and permanently
  suppress that contact on that channel, full stop — no model, no queue, no
  "let me just check the intent first."
- **A human can always see and, initially, must approve** what the AI is
  about to send. This mirrors Harvester's existing `OutreachDraft` /
  `AccountBrief` approval gate — we are extending the same philosophy to
  two-way conversations rather than reinventing it.
- **Channel rules are legally different** (CAN-SPAM for email, TCPA-style
  consent for SMS, Meta's WhatsApp Business session/template rules) and must
  be modeled explicitly, not bolted on later.
- **No hallucinated promises.** The model only answers from the service
  catalog and lead-specific facts we give it; pricing, contractual terms, and
  anything it isn't confident about get escalated to a human instead of
  guessed.

## 3. High-level architecture

```
                     ┌─────────────────────────┐
                     │        Harvester         │
                     │  (leads, institutions,    │
                     │   contact points, drafts) │
                     └───────────┬───────────────┘
                                 │ REST API (pull leads, push outcomes)
                                 ▼
 ┌───────────────────────────────────────────────────────────────────┐
 │                            Communicator                            │
 │                                                                     │
 │  ┌────────────┐   ┌────────────────┐   ┌──────────────────────┐   │
 │  │ Lead Sync   │──▶│  Conversation   │◀──│  Inbound Webhooks     │   │
 │  │ Worker      │   │  Store (PG)     │   │  (email/SMS/WhatsApp) │   │
 │  └────────────┘   └───────┬────────┘   └──────────▲────────────┘   │
 │                            │                        │                │
 │                    ┌───────▼────────┐      ┌────────┴────────┐      │
 │                    │  Reply Engine   │      │ Channel Adapters │      │
 │                    │ (intent + RAG   │      │ Email / SMS /    │      │
 │                    │  + LLM draft)   │      │ WhatsApp senders │      │
 │                    └───────┬────────┘      └────────▲────────┘      │
 │                            │                          │              │
 │                    ┌───────▼────────┐        ┌────────┴────────┐    │
 │                    │ Approval Queue  │───────▶│  Send Dispatcher │    │
 │                    │ (human review,  │        │  (rate limits,   │    │
 │                    │  auto-send gate)│        │  quiet hours)    │    │
 │                    └────────────────┘        └─────────────────┘    │
 │                                                                       │
 │                    ┌────────────────┐                                │
 │                    │ Follow-up       │  (scheduled worker, evaluates │
 │                    │ Scheduler       │   silent conversations)       │
 │                    └────────────────┘                                │
 └───────────────────────────────────────────────────────────────────┘
```

Communicator is a standalone Python/FastAPI service (same stack as
Harvester, for shared tooling and ops familiarity) with its own Postgres
database, a Redis-backed task queue for scheduling/retries, and one FastAPI
router per inbound provider webhook.

## 4. Data model (Communicator's own DB)

Mirrors Harvester's SQLAlchemy conventions (UUID PKs, JSONB for flexible
fields, `created_at`/`updated_at`).

- **`Lead`** — a local cache of a Harvester `CanonicalInstitution` +
  its qualification summary: `harvester_institution_id`, `name`,
  `institution_type`, `city`, `qualification_rationale`, `synced_at`.
  Communicator never treats this as the source of truth — it's a read
  cache refreshed by the Lead Sync Worker.
- **`Contact`** — one reachable person/number at a `Lead`:
  `channel` (email/sms/whatsapp), `value`, `person_name`, `role`,
  `harvester_contact_point_id`.
- **`ConsentRecord`** — per `Contact` + channel: `status`
  (opted_in/opted_out/unknown), `source`, `recorded_at`. Checked before
  every send; written to on every STOP-class inbound message.
- **`Conversation`** — one thread per `Contact` (a lead may have parallel
  conversations on different channels): `state` (`NEW`,
  `AWAITING_REPLY`, `ENGAGED`, `HOT_LEAD`, `HANDED_OFF`, `OPTED_OUT`,
  `CLOSED_WON`, `CLOSED_LOST`), `sequence_id`, `sequence_step`,
  `next_followup_at`.
- **`Message`** — every inbound/outbound message: `conversation_id`,
  `direction`, `channel`, `body`, `provider`, `provider_message_id`,
  `status` (queued/sent/delivered/failed/received), `sent_at`.
- **`Service`** — the catalog the AI is allowed to talk about: `name`,
  `summary`, `details` (markdown), `target_institution_types`,
  `faq` (JSONB list of Q/A), `pricing_notes`. This is the grounding data
  for "explain about services and answer queries."
- **`FollowUpSequence`** / **`SequenceStep`** — templated cadences
  (e.g. day 0 intro, day 3 nudge, day 7 case study, day 14 breakup),
  per channel, with stop conditions (any reply, opt-out, max steps).
- **`Draft`** — an AI-generated candidate reply awaiting send:
  `conversation_id`, `body`, `confidence`, `status`
  (`pending_review`/`approved`/`auto_approved`/`rejected`/`sent`),
  `reviewer`, `reviewer_feedback` — directly modeled on Harvester's
  `OutreachDraft`.
- **`AuditLog`** — every AI decision (intent classification, draft
  generated, auto-send vs. escalate) and every human action, for
  compliance and debugging.

## 5. Integration with Harvester

Communicator talks to Harvester **only through its existing (and
lightly extended) FastAPI API** — not by connecting directly to
Harvester's Postgres database. Rationale: independent scaling/deploys,
least-privilege access, and Harvester's schema stays free to evolve without
breaking Communicator.

Pull side (Communicator → Harvester, polling or webhook-triggered):
- `GET /api/v1/institutions?qualified=true&updated_since=...` — incremental
  sync of leads ready for outreach (exists today, needs a `qualified`
  filter — see the integration doc in the Harvester repo).
- `GET /api/v1/institutions/{id}` — full dossier (contact points,
  qualification rationale) used as AI context.
- `GET /api/v1/drafts` (`status=APPROVED`) — an alternative "ready to
  contact" signal, reusing Harvester's existing human-approval gate for the
  *first* touch; Communicator owns the reply loop after that.

Push side (Communicator → Harvester):
- `POST /api/v1/outreach-outcomes` *(new endpoint — see integration doc)* —
  reports `opened`/`replied`/`positive_reply`/`closed_won` back onto
  Harvester's `OutreachOutcome` so lead scoring and sales dashboards reflect
  real engagement, not just "we sent something."

Auth: reuse Harvester's existing bearer-token auth (`get_current_user`),
issuing Communicator a dedicated service-role token scoped to read
institutions/drafts and write outcomes only.

## 6. Channel adapters

A single `ChannelProvider` interface (`send(contact, body) -> ProviderRef`,
`parse_inbound(payload) -> InboundMessage`) with three implementations:

- **Email** — SES or Postmark/SendGrid (need inbound-parse webhooks, which
  raw SMTP doesn't give us; Harvester's current `smtplib` sender is
  outbound-only and insufficient here). Requires unsubscribe link + physical
  address footer for CAN-SPAM.
- **SMS** — Twilio. Requires documented consent before first send (TCPA) and
  `STOP`/`HELP` keyword handling (Twilio provides this at the platform level
  as a backstop; we still record it ourselves).
- **WhatsApp** — Twilio's WhatsApp API or Meta's WhatsApp Business Cloud API
  directly. Key constraint: outside a 24-hour customer-initiated session
  window, only pre-approved **template messages** can be sent — this directly
  shapes the follow-up scheduler (first-touch and any re-engagement after 24h
  silence must use an approved template, not free text).

Recommendation: start with **Twilio for SMS + WhatsApp** (one vendor, one
webhook shape, one bill) and **Postmark for email** (best-in-class inbound
parsing). Swappable later behind the `ChannelProvider` interface.

## 7. Reply engine (the "respond to messages" piece)

Runs whenever an inbound `Message` arrives, or a follow-up step fires.

1. **Compliance gate** (cheap, deterministic, runs first): does the message
   match an opt-out/opt-in keyword for this channel? If so, update
   `ConsentRecord` and stop — no LLM call.
2. **Intent classification** (fast/cheap model or rules): `faq_question`,
   `objection`, `buying_signal`, `scheduling_request`, `complaint`,
   `unrelated`. Buying signals and complaints are flagged `HOT_LEAD` /
   `HANDED_OFF` for immediate human attention regardless of what the draft
   step produces.
3. **Context assembly** for the drafting model:
   - Full `Conversation` message history (in order).
   - The current inbound message.
   - The `Lead` profile (institution type, city, qualification rationale
     from Harvester).
   - Relevant `Service` catalog entries, retrieved by keyword match against
     `target_institution_types` initially; upgrade to embedding search once
     the catalog is large enough to need it.
   - A system prompt fixing tone, allowed claims, and hard escalation rules
     (never discuss pricing beyond `pricing_notes`, never make legal/
     contractual commitments, always offer a human call when asked).
4. **Draft generation** (Claude) produces the candidate reply + a
   `confidence` score and a `citations` list (which `Service`/FAQ entries it
   drew from), stored as a `Draft`.
5. **Approval gate**: below a configured confidence threshold, or for any
   `HOT_LEAD`/`complaint`/off-catalog question, the draft sits in
   `pending_review` for a human (reusing the review-queue pattern from
   Harvester's `DraftReviewService`). Above the threshold and once the team
   trusts the system, low-risk FAQ replies can be configured to auto-send.
   This threshold is a per-deployment config knob, not a fixed rule — start
   conservative (100% human review) and loosen deliberately.

## 8. Automatic follow-up

A scheduled worker (Celery/APScheduler + Redis) scans conversations in
`AWAITING_REPLY` whose `next_followup_at` has passed:

- Advances to the next `SequenceStep` for that `Conversation`'s
  `FollowUpSequence`, respecting: channel-specific quiet hours/working
  hours (reusing Harvester's `within_working_hours` concept), per-lead
  weekly message caps, and WhatsApp's 24h template-vs-freeform constraint.
- Stops immediately on any inbound reply, opt-out, or `CLOSED_*` state —
  checked at send time, not just at scheduling time, to avoid racing an
  in-flight reply.
- Generated follow-up copy goes through the same `Draft` + approval gate as
  reactive replies.

## 9. Roadmap (phased)

- **Phase 0 — Foundations**: repo scaffold (FastAPI + SQLAlchemy async +
  Alembic, matching Harvester's layout), Communicator DB schema, Harvester
  API client, `ConsentRecord`/opt-out handling, `AuditLog`. No sending yet.
- **Phase 1 — Outbound MVP**: email + SMS send of a manually-templated
  first-touch message for Harvester leads with `APPROVED` drafts; delivery
  status tracking; outcome push back to Harvester. No AI yet — proves the
  send/receive plumbing and compliance gate.
- **Phase 2 — Omnichannel inbound**: add WhatsApp; inbound webhooks for all
  three channels; unified `Conversation`/`Message` timeline; still
  human-written replies, now with real conversation history to reply from.
- **Phase 3 — AI reply engine**: `Service` catalog, intent classification,
  RAG-grounded draft generation, human approval queue.
- **Phase 4 — Automated follow-up + auto-send**: `FollowUpSequence`
  scheduler; confidence-gated auto-send for low-risk FAQ replies; outcome
  feedback loop enriches Harvester's lead scoring.
- **Phase 5 — Optimization**: sequence A/B testing, sentiment-based hot-lead
  alerts to sales, multi-language support.
- **Phase 6 — Feedback Collection Loop**: thumbs up/down on every outbound
  message; `message_feedback` table stores per-author ratings with optional
  comments; `GET /feedback/summary` provides aggregated analytics (satisfaction
  rate, recent negative feedback with message context). This closes the first
  loop of the self-improvement flywheel: every rating feeds content-gap
  detection, model tuning, and agent coaching.
- **Phase 7 — Validation Layer**: before an AI-generated draft is stored or
  auto-sent, a second independent Claude call validates the reply on two
  dimensions — **relevance** (does it answer the customer's question?) and
  **grounding** (are all factual claims supported by the knowledge base?).
  If either score falls below the configured threshold, the draft is forced
  to `pending_review` regardless of the confidence score. Configured via
  `VALIDATION_ENABLED`, `VALIDATION_RELEVANCE_THRESHOLD`, and
  `VALIDATION_GROUNDING_THRESHOLD` environment variables. The validation
  result (scores, issues, latency) is returned with the draft and displayed
  in the conversation detail UI so the human reviewer understands why the
  draft was held.
- **Phase 8 — Monitoring & Alerting**: comprehensive observability layer with
  three database tables — `system_metrics` (time-series metric snapshots),
  `alert_rules` (configurable threshold-based rules with cooldowns and
  webhook notifications), and `alert_events` (fired alerts with
  acknowledgment and resolution tracking). Metric collection aggregates
  feedback rates, validation scores, conversation counts, and draft
  generation stats from existing data. Health checks cover database
  connectivity, Claude API reachability, and active alert status. Five
  default alert rules monitor validation score drops, error rate spikes,
  slow response times, negative feedback surges, and high draft generation
  latency. The monitoring dashboard (at `/monitoring`) provides four views:
  overview (health status, component health, key metrics, recent negative
  feedback), alerts (event list with acknowledge/resolve actions), rules
  (CRUD for alert rules with enable/disable), and metrics (detailed metric
  cards with range visualization). API endpoints at `/monitoring/*` support
  all operations including manual metric collection and alert evaluation.
- **Phase 9 — AI-Powered Suggestions**: closes the self-improvement flywheel
  by connecting negative feedback (Phase 6) to automatic suggestion generation.
  When a customer gives thumbs-down feedback, Claude analyzes the full
  conversation context and generates structured suggestions for: new knowledge
  base entries for uncovered questions, FAQ additions for recurring topics,
  and agent behavior improvements. Each suggestion has a status lifecycle
  (suggested → approved/dismissed/applied) with duplicate detection to prevent
  the same suggestion from being created multiple times. The `suggestions`
  table stores suggestion type, title, description, suggested content, source
  feedback reference, confidence score, and category. Batch analysis endpoint
  (`POST /suggestions/batch-analyze`) processes all unprocessed negative
  feedback from the last N hours. Single feedback analysis
  (`POST /suggestions/analyze-feedback/{id}`) triggers immediate analysis.
  Suggestions can be approved or dismissed via the API. Summary analytics
  (`GET /suggestions/summary`) provides counts by type, category, and status
  with recent suggestions. The dedicated suggestions dashboard (at `/suggestions`)
  provides three views: overview (summary cards, breakdown by type/category,
  recent suggestions), all suggestions (filterable list with approve/dismiss
  actions, search, detail modal), and batch analysis (trigger and track progress).
  If Claude is unavailable, suggestions are skipped gracefully without blocking
  the feedback submission.
- **Phase 10 — Copilot for Human Reviewers**: enhances the draft approval queue
  with an AI-generated context package so human reviewers can make faster, more
  informed decisions. When a draft is in `pending_review` status, the copilot
  provides: (1) **Conversation Summary** — Claude condenses the full conversation
  history into 2-3 sentences focusing on what the customer wants and where the
  conversation stands; (2) **Customer Profile Enrichment** — lead info, contact
  details, conversation statistics, and historical engagement data collected from
  the database without requiring AI; (3) **Key Topics** — 3-5 topic strings
  extracted from the conversation; (4) **Customer Sentiment** — overall sentiment
  indicator (positive, neutral, negative, frustrated); (5) **Suggested Edits** —
  Claude reviews the draft body and suggests specific improvements categorized by
  kind (tone, clarity, completeness, accuracy, style) with priority levels (high,
  medium, low). Context is cached in the `copilot_context` table for 1 hour to
  avoid regenerating on every page refresh. API endpoints at `/copilot-reviewer/*`
  support getting context per draft, batch fetching for all pending drafts, and
  force-refresh. The copilot panel appears in the conversation detail UI as a
  collapsible purple section with sentiment badge, edit count indicator, and
  refresh button. If Claude is unavailable, the profile enrichment still works
  since it uses only database data.
- **Phase 11 — Topic Categorization**: auto-categorizes every conversation into
  topics and subtopics using Claude for trend analysis and content gap identification.
  Each conversation is classified into one of 10 top-level topics (product_inquiry,
  pricing, support, sales, complaint, refund, scheduling, account, feedback, general)
  with a specific subtopic. The `conversation_topics` table stores the categorization
  with confidence scores, extracted keywords, and override support. If Claude is
  unavailable, a keyword-based heuristic fallback categorizes conversations. The
  `topic_trends` table tracks aggregated analytics over time windows. API endpoints
  at `/topics/*` support getting topic summary (distribution, trending topics, top
  keywords), categorizing individual conversations, batch categorization of all
  uncategorized conversations, and manual topic override by human reviewers. The
  frontend dashboard provides four views: overview (summary cards, top topics,
  trending), topic distribution (bar chart), trending topics (7-day change table),
  and keyword cloud. This enables identifying which topics are growing, which have
  high negative feedback (content gaps), and which agents struggle with.
- **Phase 12 — Simulation & Testing**: pre-deployment testing of AI responses against
  sample conversations before going live. The `agent_policies` table stores guardrails
  and behavior rules (name, category, description, version, guardrail text). The
  `simulation_scenarios` table models customer personas with expected conversation flows
  and customer messages. The `simulation_runs` table records batch test results with
  pass/fail verdicts, scores, and recommendations. An interactive sandbox endpoint
  (`POST /simulations/sandbox/chat`) allows real-time testing of agent responses.
  Batch simulation (`POST /simulations/batch`) runs all scenarios against policies.
  Statistics endpoint (`GET /simulations/stats`) provides pass rates and average scores.
- **Phase 13 — Voice Channel**: voice and WebRTC calling integration for phone support
  directly inside the agent inbox. Supports Twilio Voice and WebRTC softphone with IVR,
  dual-track audio recording, and real-time transcription. The voice integration page
  at `/channels/voice` provides call controls, recording management, transcription
  display, and voice channel configuration.
- **Phase 14 — Web Chat Widget**: live chat messenger integration for website visitors,
  not just outbound leads. Supports real-time chat with visitor tracking, proactive
  messaging, and co-browsing capabilities. The live chat integration page at
  `/channels/live-chat` provides widget configuration, active visitor monitoring,
  and chat management.
- **Phase 16 — CX Scoring & Automated QA**: AI-powered conversation quality scoring
  that evaluates support quality across every conversation without requiring customer
  surveys, providing 5x more coverage than traditional CSAT. Each conversation is
  scored on 5 dimensions using Claude: accuracy (factual correctness), empathy (tone
  and care), resolution (clear outcome), compliance (policy adherence), and timeliness
  (response speed). The `qa_scores` table stores per-conversation scores with evaluation
  summary, issues found, and recommendations. The `cx_score_snapshots` table aggregates
  scores over time windows for trend analysis. The `qa_scoring_rules` table configures
  quality checks with severity levels and score impact. If Claude is unavailable,
  keyword-based heuristics provide baseline scores. API endpoints at `/cx-qa/*` support
  getting CX summary (overall score, dimension averages, 7-day trends, top issues),
  scoring individual conversations, batch scoring of all unresolved conversations,
  and managing QA rules (list, enable/disable). The frontend dashboard provides three
  views: overview (CX score cards, dimension scores with progress bars, trend table,
  top issues), QA rules (enable/disable toggle), and batch scoring (progress tracking).
- **Phase 15 — CSAT Surveys**: customer-facing satisfaction surveys that complement the
  AI-powered CX scoring (Phase 16) with direct customer feedback. The `csat_survey_templates`
  table defines survey structure: question text, scale type (1-5, 1-10, emoji, binary),
  comment prompt, trigger type (manual, auto_on_close, auto_on_send, scheduled), and channel
  filter. The `csat_surveys` table stores individual survey instances with status lifecycle
  (pending → sent → responded/expired), score, and optional comment. The `csat_snapshots`
  table aggregates metrics over time windows for trend analysis. CSAT summary provides overall
  CSAT percentage (% of 4-5 scores), average score, response rate, score distribution,
  breakdown by channel, 7-day trend comparison, and recent comments. API endpoints at `/csat/*`
  support template CRUD, survey creation, response submission, send action, summary analytics,
  and snapshot management. The frontend dashboard (at `/csat`) provides three views: overview
  (CSAT score cards with trend indicator, score distribution bars, by-channel breakdown,
  recent comments), all surveys (filterable list with send/respond actions), and templates.
- **Phase 17 — Fin Operator**: meta-agent for autonomous self-improvement that monitors
  system metrics (validation scores, feedback rates, error rates, resolution rates,
  response times), detects anomalies by comparing current metrics to previous snapshots,
  and generates structured improvement proposals using Claude. The `operator_proposals`
  table stores proposals with type (knowledge_update, procedure_change, policy_update,
  prompt_tuning, threshold_adjustment, content_creation, escalation_rule, general),
  status lifecycle (proposed → reviewing → approved → testing → deployed/dismissed),
  priority, confidence score, and source data. The `operator_performance_log` table
  tracks every operator action (scan, analyze, propose, test, deploy, rollback, monitor)
  with result and duration. The `operator_config` table controls behavior: scan interval,
  auto-deploy for low-risk changes, confidence thresholds, and metrics to monitor.
  The `operator_metrics_snapshot` table stores periodic metric snapshots with anomaly
  detection and trend analysis. API endpoints at `/operator/*` support summary dashboard,
  config CRUD, scan trigger, proposal listing/filtering/approval/dismissal, logs, and
  snapshots. The frontend dashboard (at `/operator`) provides four views: overview
  (metrics cards, anomalies, proposals by status, recent activity), proposals (filterable
  list with approve/dismiss actions, expandable details with rationale and implementation
  plan), activity log (table of all operator actions with result and duration), and
  configuration (enable/disable, scan interval, auto-deploy settings, confidence threshold).
- **Phase 19 — Unified Inbox**: customer-centric inbox view that merges conversations
  across all channels (WhatsApp, Email, SMS, Facebook, Instagram, Telegram, LINE, Voice,
  TikTok, Web Chat) for the same customer into a single unified timeline. Uses the existing
  customer_channel_ids table for identity resolution. The `unified_customer_mapping` table
  links conversations to their resolved customer_id for cross-channel grouping. The
  `unified_inbox_settings` table configures per-organization preferences (group by customer/
  contact/channel, merge timeline, show channel badges, auto-resolve identity). A database
  trigger auto-populates customer mappings when conversations are created and the contact has
  a known customer_id. API endpoints at `/unified-inbox/*` support listing customers with
  their conversations grouped, getting unified customer detail with merged timeline, resolving
  customer identity from channel + identifier, creating manual mappings, and managing settings.
  The frontend page (at `/unified-inbox`) provides a customer list with search and channel/state
  filters, customer detail view showing profile info and all conversations across channels, and
  a merged timeline view showing all messages chronologically with channel indicators.
- **Phase 20 — Visual Workflow Builder**: no-code automation engine for creating multi-step
  workflows with conditional branching, assignment rules, and trigger-based execution, mirroring
  Intercom's visual workflow builder. The `workflows` table stores workflow definitions with type
  (automation, assignment, notification, escalation, custom), status (draft, active, paused,
  archived), trigger type (event, schedule, manual, webhook), and JSON step definitions. The
  `workflow_steps` table stores ordered steps with types (wait, send_message, assign, tag,
  condition, action, notify, update_field, create_ticket, custom) and branching configuration.
  The `workflow_conditions` table configures conditional branching with operators (equals,
  not_equals, contains, greater_than, less_than, is_empty, is_not_empty) and logic operators
  (and, or). The `workflow_executions` table tracks each execution with step-by-step results.
  The `workflow_triggers` table defines event-based triggers (conversation_created, message_received,
  state_changed, feedback_received, etc.). Step execution engine processes each step type, with
  actual database updates for assign steps and simulated execution for others. Condition evaluation
  supports AND/OR logic with multiple operators. API endpoints at `/workflows/*` support full CRUD,
  step/condition/trigger management, manual execution, condition evaluation, and execution history.
  The frontend dashboard (at `/workflows`) provides three views: overview (summary cards, by-type
  breakdown, recent executions), workflows (filterable list with create/activate/pause/delete/
  execute actions), and executions (execution history with status). Default workflows seeded for
  common use cases: new conversation notification, auto-assign by topic, and escalate unresolved.
- **Phase 21 — Public Help Center**: customer-facing self-service portal with categorized
  articles, full-text search, and view analytics, mirroring Intercom's Help Center. The
  `help_center_categories` table organizes articles with icons, descriptions, display order,
  and visibility toggles. The `help_center_articles` table stores articles with draft/published/
  archived lifecycle, pinning, SEO metadata (title, description), and helpful/not-helpful
  feedback counts. The `help_center_searches` table tracks customer search queries with result
  counts and clicked articles for analytics. The `help_center_settings` table configures the
  public help center: name, tagline, logo, primary color, search enablement, article feedback,
  and footer text. A public endpoint (`GET /help-center/public`) returns the entire help center
  for embedding on external websites. Search uses LIKE-based full-text matching on title and
  content, recording each query for analytics. Default categories seeded: Getting Started,
  How-To Guides, FAQ, and Troubleshooting. API endpoints at `/help-center/*` support summary
  dashboard, public overview, category/article CRUD, search, article feedback, and settings.
  The frontend dashboard (at `/help-center`) provides four views: overview (summary cards,
  quick search, top articles by views, popular searches), articles (filterable list with
  create/edit/publish/unpublish/pin/delete actions), categories (list with create modal),
  and settings (name, tagline, color picker, search/feedback toggles, footer text).
- **Phase 22 — CRM Sync**: Salesforce, HubSpot, and Pipedrive bidirectional sync for leads,
  contacts, and accounts/companies, mirroring Intercom's CRM sync. The `crm_connections` table
  stores per-organization CRM integrations with provider, auth type, encrypted credentials,
  sync config, and status lifecycle (disconnected → connecting → connected → error). The
  `crm_sync_logs` table tracks each sync operation with direction (inbound/outbound), object
  type, record counts, success/error counts, and status. The `crm_field_mappings` table defines
  how local fields map to CRM fields per object type with direction (inbound/outbound/both)
  and optional transforms. The `crm_external_refs` table links local records to CRM records
  for identity resolution with sync status tracking. Default field mappings seeded for
  Salesforce (Company, Email, Phone, FirstName, LastName) and HubSpot (company, email).
  API endpoints at `/crm-sync/*` support summary dashboard, provider listing, connection
  CRUD, sync triggering (per direction and object type), sync logs, field mappings, and
  external reference management. The frontend dashboard (at `/crm-sync`) provides four views:
  overview (summary cards, provider breakdown, pending/conflict alerts, recent syncs),
  connections (filterable list with create/sync/delete actions), logs (sync history table),
  and mappings (field mapping table).
- **Phase 23 — KB Document Upload & URL Crawling**: upload PDF, DOCX, TXT, HTML,
  and MD files to automatically populate the knowledge base, and crawl webpages for
  content extraction, mirroring Intercom's document upload and URL crawling. The
  `kb_documents` table stores uploaded files with metadata (name, type, size, SHA256
  hash for duplicate detection, status lifecycle). The `kb_document_chunks` table
  stores extracted text segments linked to documents and KB entries. The `kb_url_sources`
  table tracks webpages for crawling with configurable intervals. The `kb_url_crawl_logs`
  table records each crawl operation with status codes and chunk counts. The `kb_url_chunks`
  table stores extracted text from crawled pages. The `kb_import_settings` table configures
  chunk size, overlap, max file size, auto-create KB entries, crawl interval, and allowed
  file types. PDF extraction uses lopdf, DOCX parsing uses zip+XML extraction, and HTML
  text extraction strips scripts/styles/tags. Text is chunked with configurable size and
  overlap, with sentence-boundary detection. API endpoints at `/kb-import/*` support
  summary dashboard, document upload/list/delete/chunks, URL source add/crawl/delete,
  and settings CRUD. The frontend dashboard (at `/kb-import`) provides four views:
  overview (summary cards, recent uploads, recent crawls), documents (file upload area,
  filterable list with delete), URL sources (add URL with crawl interval, crawl now,
  delete), and settings (chunk size, overlap, max file size, auto-create toggle,
  allowed file types).
- **Phase 24 — Discord Channel**: Discord bot integration for two-way messaging,
  mirroring Intercom's Discord channel support. The `discord_commands` table stores
  slash commands (help, agent, status) with active/inactive status. The `discord_events`
  table tracks incoming webhook events (MESSAGE_CREATE, INTERACTION, GUILD_JOIN) with
  raw payloads and processing status. The `discord_settings` table configures auto-respond,
  greeting/away messages, allowed channels, blocked words, and event logging. The Discord
  adapter (`DiscordAdapter` in `channels.rs`) sends messages via Discord API v10 using
  bot token authentication. Bot profile is fetched live from `/@me` endpoint for
  verification. API endpoints at `/inboxes/channels/discord/*` support get/update channel
  config, verify connection, send test message, and list events. The frontend page
  (at `/channels/discord`) provides four views: credentials (bot token, client ID,
  interaction secret, bot profile), settings (auto-respond toggle, greeting/away messages),
  commands (list of slash commands with active/inactive status), and diagnostics (connection
  health, test message sender, recent events).
- **Phase 25 — Fin Vision**: AI-powered image understanding that analyzes
  images/screenshots sent by customers using Claude's multi-modal vision capabilities,
  mirroring Intercom's Fin Vision. The `vision_images` table stores uploaded images with
  metadata (MIME type, file size, base64 data, conversation/contact/channel linkage,
  status lifecycle: uploaded → analyzing → completed/failed). The `vision_analyses` table
  stores per-image AI analysis results with analysis type (general, screenshot, document,
  error_code, product), confidence scores, objects detected, extracted text (OCR), summary,
  processing time, and error messages. The `vision_settings` table configures per-organization
  preferences: enabled state, auto-analyze on upload, default analysis type, Claude model,
  max file size, allowed MIME types, OCR, and object detection. Five analysis prompts are
  tailored to each type: screenshot analysis identifies apps, errors, and UI elements;
  document analysis extracts text and summarizes key information; error_code analysis
  identifies error messages and suggests troubleshooting; product analysis identifies
  products, features, and labels; general analysis provides a broad description. Images
  are sent to Claude's API as base64-encoded data or URLs using the multi-modal content
  block format. Graceful fallback provides basic analysis when Claude is unavailable.
  API endpoints at `/vision/*` support summary dashboard, image upload, analyze, list
  images with status filter, get image with analyses, delete, and settings CRUD. The
  frontend dashboard (at `/fin-vision`) provides four views: overview (summary cards,
  analysis type breakdown with progress bars, recent analyses, vision status), images
  (upload button, filterable list with view/analyze/delete actions), analysis detail
  (image info, re-analyze with different type, full analysis results with objects as
  tags, extracted text, summary, and error messages), and settings (toggles for enabled,
  auto-analyze, OCR, object detection, model selection, file size limit).
- **Phase 26 — Audience KB Scoping**: links customer segments (audiences) to
  specific knowledge base subsets so different customer groups see different AI
  knowledge, mirroring Intercom's "Audiences control what Fin knows." The
  `audience_kb_links` table maps segments to KB entries with priority ordering.
  The `audience_kb_settings` table configures per-organization preferences:
  enabled state, fallback to global KB when segment has no links, merge segments
  (union of all segment KB entries vs highest-priority only), and default scope.
  The `audience_kb_usage_log` table tracks which KB entries were used for which
  conversations to measure the impact of audience-based scoping. The
  `segment_contact_memberships` table links contacts to segments with membership
  type (static/dynamic) for explicit membership management. The core engine
  provides scoped KB retrieval per contact: when a contact belongs to segments,
  only the KB entries linked to those segments are returned to the AI. Settings
  control fallback behavior (return all KB if segment has no links) and merge
  mode (union of all segment KB entries vs single highest-priority segment).
  API endpoints at `/audience-kb/*` support summary dashboard, settings CRUD,
  KB link CRUD (link/unlink KB entries to segments), scoped KB retrieval per
  contact, segment membership management (add/remove contacts from segments),
  usage logging, and usage log retrieval. The frontend dashboard (at
  `/audience-kb`) provides four views: overview (summary cards, top segments
  by KB links, top KB entries by usage), links (filterable list with create/
  delete, view usage logs), memberships (contact-to-segment membership list
  with add/remove, usage log viewer), and settings (toggles for enabled,
  fallback, merge segments, default scope).
- **Phase 27 — Brand Overrides in Workflows**: enables dynamic brand switching
  mid-conversation or mid-workflow execution, mirroring Intercom's "Brand
  overrides in workflows" capability. The `brand_override_rules` table defines
  when and how a brand switch occurs with trigger types (workflow, keyword,
  segment, manual), from/to brand IDs, conditions, and auto-switch toggles for
  KB, AI config, and messenger. The `brand_switch_log` table tracks every brand
  switch event for auditing with conversation, contact, trigger type, and rule
  references. The `brand_override_settings` table configures per-organization
  preferences: enabled state, cross-org brand switching, approval requirement,
  and logging. The `brand_hierarchy` table models parent-brand to sub-brand
  relationships for cascading overrides. The workflow builder includes a new
  "switch_brand" step type that updates the conversation's brand during
  workflow execution. API endpoints at `/brand-overrides/*` support summary
  dashboard, rule CRUD, brand switching, switch log retrieval, settings CRUD,
  hierarchy management, and matching rule discovery. The frontend dashboard
  (at `/brand-overrides`) provides four views: overview (summary cards, top
  switched-to brands with progress bars, recent switches timeline), rules
  (filterable list with create/enable/disable/delete), hierarchy (parent-sub-brand
  relationship management), and settings (toggles for enabled, cross-org,
  approval, logging).
- **Phase 28 — MCP Protocol Support**: Model Context Protocol support for
  extensible AI tool connections, mirroring Intercom's "MCP connections to
  Jira, Sentry, Glean, Guru, Slite, Pendo, Fullstory." The `mcp_servers` table
  stores external MCP-compatible server connections with URL, transport type
  (stdio/HTTP), headers, environment variables, and connection status. The
  `mcp_tools` table stores discovered tools from connected servers with input
  schema, enable/disable state, and AI accessibility flags. The `mcp_sessions`
  table tracks active MCP sessions for stateful connections. The `mcp_call_logs`
  table records every tool call with input/output data, status, duration, and
  tokens used for auditing and analytics. The `mcp_settings` table configures
  per-organization preferences: enabled state, auto-discover tools, log all
  calls, max concurrent calls, timeout, and allowed/blocked tool lists. The
  `mcp_server_templates` table provides pre-built configurations for common
  integrations (Sentry, Guru, Pendo, Fullstory, Jira, GitHub, Slack) with
  default URLs, required environment variables, and default tool lists. Tool
  discovery infers available tools from server URL patterns. API endpoints at
  `/mcp/*` support summary dashboard, server CRUD, tool discovery, tool calling,
  template listing, settings CRUD, and call log retrieval. The frontend dashboard
  (at `/mcp`) provides six views: overview (summary cards, top called tools with
  progress bars, recent calls table), servers (create/manage connections with
  discover/enable/disable/delete actions), tools (discovered tool grid with
  call counts and AI accessibility), templates (pre-built server configurations
  with one-click "Use Template" button), calls (call log history table), and
  settings (MCP behavior controls with toggles and numeric inputs).
- **Phase 29 — Autonomous Resolution**: Enables the AI agent to resolve
  conversations autonomously without human approval, mirroring Intercom's
  "Fin AI Agent" that resolves conversations autonomously across all channels.
  The `resolution_config` table stores per-organization settings: enabled state,
  auto-send confidence threshold, validation requirements, max autonomous attempts,
  low-confidence escalation, sensitive topic detection, allowed channels, and
  cooldown period. The `escalation_rules` table defines when and how to escalate
  from AI to human with condition types (confidence_below, topic_match, keyword_match),
  action types (assign_to_agent, assign_to_team, notify_webhook), and priority ordering.
  The `resolution_logs` table tracks every autonomous resolution attempt with AI
  confidence, validation score, outcome (auto_sent/escalated), escalation reason,
  and customer feedback. The `resolution_analytics` table aggregates resolution rate,
  escalation rate, and top escalation reasons over configurable time windows. The
  `agent_policies` table provides fine-grained guardrails (no hallucination, sensitive
  topics, max attempts) with condition/action pairs. The core engine evaluates each
  draft against config thresholds, validation scores, sensitive topics, and escalation
  rules to decide whether to auto-send or escalate. Disabled by default — enable when
  ready for autonomous operation. API endpoints at `/autonomous-resolution/*` support
  summary dashboard, config CRUD, escalation rules CRUD, policy listing, decision
  evaluation, attempt logging, log retrieval, and analytics snapshot generation.
  The frontend dashboard (at `/autonomous-resolution`) provides four views: overview
  (status banner, metrics cards, live config editing, analytics snapshot, top escalation
  reasons, recent logs), escalation rules (CRUD with enable/disable/edit/delete),
  policies (agent behavior guardrails display), and logs (resolution attempt history
  table with status, confidence, validation, outcome, and reason).
- **Phase 30 — Agent Performance Analytics**: Per-agent metrics, coaching insights,
  and team benchmarks. The `agent_performance_snapshots` table stores aggregated
  per-agent metrics over time windows: conversations handled/resolved, average
  response time, resolution rate, customer satisfaction, feedback counts, draft
  approval rate, escalations, SLA breaches, CX/QA scores. The `agent_coaching_insights`
  table stores AI-generated or manual coaching suggestions with severity, category,
  and recommendation. The `agent_comparison_metrics` table provides team-wide
  benchmarks for leaderboards. The `agent_performance_settings` table configures
  benchmarks (response time, resolution rate, satisfaction, CX score), insight
  frequency, and leaderboard visibility. API endpoints at `/agent-performance/*`
  support summary dashboard, snapshot CRUD, insight CRUD with address workflow,
  comparison metrics generation, and settings CRUD. The frontend dashboard (at
  `/agent-performance`) provides four views: overview (summary cards, benchmark
  cards, recent insights, team comparison), snapshots (table with progress bars),
  insights (filterable list with create/address), and settings (toggles and
  benchmark inputs).
- **Phase 31 — Real-time Notifications & WebSockets**: Push notifications,
  WebSocket broadcast, and notification preferences for live updates across
  the platform. The `notification_queue` table stores all notifications with
  status lifecycle (pending → sent → read/dismissed), notification type,
  channel, and JSON data payload. The `notification_preferences` table stores
  per-user settings for 14 event types (new messages, draft approvals/rejections,
  alerts, assignments, escalations, workflow completion, CSAT responses,
  suggestions, operator proposals) plus sound, desktop notifications, and
  quiet hours. The `websocket_connections` table tracks active WebSocket
  connections with heartbeat monitoring for broadcast targeting. The
  `notification_broadcast_log` table records every broadcast with recipient
  count and delivery status. The `notification_settings` table configures
  per-organization preferences: enabled state, WebSocket support, email
  notifications, rate limiting, digest mode, and retention days. API endpoints
  at `/notifications/*` support summary dashboard, list/queue/broadcast
  notifications, mark read/dismiss, preferences CRUD, settings CRUD, connection
  listing, and old notification cleanup. The frontend dashboard (at
  `/notifications`) provides four views: overview (metrics cards, by-type
  breakdown, recent notifications, active WebSocket connections), notifications
  (searchable/filterable list with mark read/dismiss actions), preferences
  (toggle grid for all event types, quiet hours configuration), and settings
  (global notification configuration).
- **Phase 32 — In-App Messaging & Proactive Triggers**: Behavioral event-driven
  messaging system for triggered messages based on user actions, page views, time
  on site, and custom events. Mirrors Intercom's proactive outreach capabilities
  ("Are you stuck?" popups, onboarding sequences, re-engagement messages). The
  `inapp_messages` table stores message definitions with type (popup, banner,
  carousel, spotlight, modal), trigger configuration (event, page view, time delay,
  segment), frequency controls (once per session, once per user, every visit, once
  per day), and analytics counters (views, clicks, dismisses). The `inapp_message_events`
  table tracks when messages are shown, clicked, or dismissed with automatic stat
  updates. The `proactive_triggers` table defines when messages fire: page view with
  URL matching, custom events, exit intent, inactivity, and time on page — each with
  JSON conditions, delay, and daily trigger limits. The `behavioral_events` table
  records user behavioral events (page views, clicks, custom events) for trigger
  evaluation and analytics. The `inapp_settings` table configures per-organization
  preferences: enabled state, max concurrent messages, behavioral event tracking,
  and event retention. Trigger evaluation API allows testing triggers against
  simulated events before deployment. API endpoints at `/inapp/*` support summary
  dashboard, message CRUD, trigger CRUD, behavioral event recording/listing, message
  event recording, trigger evaluation, and settings CRUD. The frontend dashboard
  (at `/inapp-messaging`) provides five views: overview (metrics cards, by-type
  breakdown with progress bars, recent events), messages (searchable/filterable list
  with create/activate/delete modals), triggers (list with create/enable/delete,
  test trigger evaluation modal), events (behavioral event table with record test
  event button), and settings (toggles and numeric inputs).
- **Phase 34 — Translation Dashboard**: Completed the Translation feature by adding
  the missing frontend dashboard. The translation backend already existed with language
  detection (29 languages, script-level markers, keyword analysis), phrase-based
  translation (exact phrases for Spanish, French, German), external translation API
  integration (configurable URL and key), and auto-translation in conversations.
  The new dashboard provides five views: Overview (stats cards showing total/curated
  languages, enabled status, auto-translate toggle, supported languages list, current
  settings summary, how-it-works guide), Languages (full table of all 29 supported
  languages with code, name, native name, direction, and curated/detect-only status),
  Detect Language (interactive test for language detection with confidence scores),
  Translate (test translation to any supported language with source/target display),
  and Settings (enable/disable translation, auto-translate, language detection, show
  original, default target language, fallback language, API URL and key configuration).
   Access at `/translation`.

- **Phase 35 — Embed Widget**: Added embeddable help center and live chat widgets
   for external websites. Four widget types: help center, live chat, search, and
   combined. Create widget configs with custom styles and trigger settings, generate
   embed scripts for easy installation, track widget instances and events (opened,
   closed, message sent, search performed, article viewed), and manage per-org
   settings (enable/disable, custom domains, analytics, max instances). The public
   endpoint (`/embed-widget/public/{config_id}`) serves widget data including help
   center articles for embedding. Access at `/embed-widget`.

- **Phase 36 — Unified Analytics Dashboard**: Comprehensive analytics layer aggregating
   all metrics from every phase into one unified dashboard. The `saved_reports` table
   stores report definitions with 7 types (unified, conversation, agent, cx_qa, csat,
   feedback, topic), configurable time ranges, and filters. The `scheduled_reports`
   table manages automated report delivery with daily/weekly/monthly schedules,
   recipient lists, and format options (CSV, JSON, PDF). The `report_templates` table
   provides pre-built report configurations for quick creation. The `dashboard_widgets`
   table stores customizable widget layouts with position, size, and type (metric_card,
   line_chart, bar_chart, pie_chart, table, trend). The `analytics_snapshots` table
   records periodic metric snapshots for time-series trend analysis. The
   `unified_analytics_settings` table configures per-org preferences: enabled state,
   auto-snapshot frequency, retention days, and default dashboard. The summary endpoint
   aggregates data from conversations, feedback, validation, CX scores, CSAT, topics,
   suggestions, alerts, autonomous resolution, procedures, workflows, and agent
   performance into a single response. API endpoints at `/analytics/*` support summary
   dashboard, saved report CRUD and run-on-demand, scheduled report CRUD, template
   listing, widget CRUD, snapshot creation and listing, settings CRUD, and report
   export. The frontend dashboard (at `/unified-analytics`) provides four views:
   overview (6 key metric cards, feedback/suggestions/alerts summary, channel and
   status breakdowns with progress bars, topic distribution tags, autonomous/procedure/
   workflow stats, agent leaderboard table), saved reports (list with run/delete,
   create modal with type/time range selection, template gallery), scheduled reports
   (list with enable/pause/delete, create modal with schedule/format/recipients),
   and settings (enable/disable toggles, snapshot frequency, retention days, default
   dashboard).

- **Phase 37 — Reply Engine Integration**: Connected all previously siloed knowledge
   features into the AI reply generation pipeline. The `reply_integration` service
   orchestrates: (1) Knowledge Base entries with audience-based scoping per contact
   segment, (2) Help Center published articles, (3) Uploaded document chunks from
   PDFs/DOCX/crawled URLs, and (4) Tone of Voice + Guidance Rules injected into the
   AI system prompt. The `reply_knowledge_sources` table tracks which sources were
   used in each AI-generated reply for transparency and analytics. The
   `reply_integration_settings` table configures per-source toggles and limits
   (max entries, chunks, articles). The reply engine's `build_user_prompt` now calls
   `ReplyIntegrationService::build_context_package()` to assemble the unified context
   before each reply, falling back to legacy context if integration fails. Knowledge
   sources are tracked after each draft is written. API endpoints at `/reply-integration/*`
   support summary dashboard, settings CRUD, context build test, and knowledge source
   listing. The frontend dashboard (at `/reply-integration`) provides four views:
   overview (status cards for each integration point, tracked sources by type with
   progress bars, how-it-works guide), settings (toggle grid for each source type,
   numeric limits for max entries/chunks/articles), test build (enter a sample message
   to see the exact context package the AI would receive), and knowledge sources
   (usage log table showing which sources were used in which drafts).

- **Phase 38 — A/B Testing & Phased Deployments**: Data-driven experimentation on
   AI configurations before full deployment. The `ab_experiments` table stores
   experiment definitions with 6 types (confidence_threshold, retrieval_strategy,
   response_format, escalation_trigger, tone_style, model), status lifecycle
   (draft, running, paused, completed, failed), and traffic split for control group.
   The `ab_variants` table stores per-experiment variants with JSON-configurable
   settings and a control flag. The `ab_assignments` table tracks which variant each
   conversation was assigned to. The `ab_results` table records per-assignment metrics
   (resolution_rate, satisfaction, response_time, feedback_score, validation_score).
   The `ab_experiment_summary` table stores aggregated snapshots with per-variant
   statistics, statistical significance, and recommended winner. The `ab_settings`
   table configures per-org preferences: enabled state, auto-detect winner, min
   sample size, significance threshold, and max concurrent experiments. The service
   provides randomized assignment based on traffic split, result recording, summary
   generation with significance calculation, and winner detection. API endpoints at
   `/ab-testing/*` support summary dashboard, experiment CRUD, variant listing,
   conversation assignment, result recording, summary generation, and settings CRUD.
   The frontend dashboard (at `/ab-testing`) provides three views: experiments
   (filterable list with status/type filters, start/pause/resume/delete actions,
   type breakdown chart), experiment detail (variant comparison, summary with
   significance and winner detection, recent results), and settings (enable/disable,
   auto-detect winner, min sample size, significance threshold, max concurrent
   experiments). A sample experiment is seeded on first run to demonstrate the
   confidence threshold testing pattern.

- **Phase 39 — ERP Fulfillment Dashboard**: Completed the ERP Fulfillment feature
   with PostgreSQL migration and frontend dashboard. The `erp_shipping_zones` table
   stores geographic delivery areas with city lists and catch-all support. The
   `erp_shipping_methods` table configures courier services with ETA ranges. The
   `erp_shipping_rates` table defines pricing rules per zone/method with rate types
   (FLAT, BY_WEIGHT, BY_PRICE, BY_ITEM). The `erp_return_requests` table manages
   customer return requests with full lifecycle tracking (REQUESTED → APPROVED →
   IN_TRANSIT → RECEIVED → INSPECTED → REFUNDED → CLOSED). The `erp_return_request_items`
   table tracks individual items within each return, including restockability. The
   `erp_manifests` table groups shipments by courier for dispatch. The `erp_return_reason_codes`
   table provides predefined return reasons. Migration uses PostgreSQL syntax with
   `gen_random_uuid()`, `TIMESTAMPTZ`, and `ON CONFLICT DO NOTHING`. The backend service
   (`erp_fulfilment.rs`) provides summary analytics, CRUD for zones/methods/rates,
   return request management with status updates, and manifest dispatch. API endpoints
   at `/erp/fulfilment/*` support summary, shipping zone/method/rate CRUD, return
   reason listing, return request CRUD with status updates, return stats, and manifest
   listing/dispatch. The frontend dashboard (at `/fulfillment`) provides four views:
   overview (summary cards for zones, methods, rates, returns, manifests; return
   statistics; how-it-works guide), shipping (zone and method creation/deletion,
   rate table with type and fee display), returns/RMA (status filter, return list
   with detail view showing items, approve/reject actions), and manifests (list with
   dispatch button for open manifests).

- **Phase 40 — Scheduled Tasks & Cron Jobs**: Automates recurring operations across
  the platform with configurable cron schedules. The `scheduled_tasks` table stores
  task definitions with 7 types (metric_collection, alert_evaluation, csat_send,
  kb_crawl, analytics_snapshot, operator_scan, cleanup), cron schedule expressions,
  JSON config, and active/inactive toggle. The `scheduled_task_runs` table logs
  each execution with status (running, success, failed, skipped), duration, records
  processed, and error messages. The `scheduled_task_queue` table manages pending
  tasks triggered by scheduler, manual action, or API. The `scheduler_settings`
  table configures per-org preferences: enabled state, max concurrent tasks, task
  timeout, retry policy (enabled, count, delay), log retention days, and failure
  webhook URL. The service provides summary analytics, task CRUD, manual triggering,
  queue processing, run history, settings management, and old log cleanup. Seven
  default tasks are seeded on first run: metric collection (every 6 hours), alert
  evaluation (every hour), CSAT survey sending (daily at 9 AM), KB URL re-crawling
  (weekly), analytics snapshot (daily), Fin Operator scan (every 12 hours), and
  old log cleanup (weekly). API endpoints at `/scheduled-tasks/*` support summary
  dashboard, task CRUD, trigger, process queue, run listing, queue listing, settings
  CRUD, cleanup, and available task types. The frontend dashboard (at `/scheduled-tasks`)
  provides four views: overview (metrics cards, task type breakdown with progress
  bars, recent runs, task queue, process queue and cleanup buttons), tasks (filterable
  list with type filter, create modal with name/description/type/cron/config,
  toggle active/inactive, trigger now, delete), runs (status filter, execution
  history table with status, task ID, started time, duration, records, errors),
  and settings (enable/disable scheduler, max concurrent tasks, timeout, retry
  policy, log retention, webhook URL).

- **Phase 41 — Compliance & Data Privacy Dashboard**: Comprehensive compliance
  management for legally regulated multi-channel communication. The `suppression_list`
  table tracks contacts who should not receive messages with per-channel and per-reason
  tracking (opt_out, bounce, complaint, gdpr_deletion, manual, blocked). The
  `gdpr_requests` table manages data subject requests (access, deletion, portability,
  rectification, objection) with full lifecycle (pending → in_progress → completed/denied)
  and auto-expiry after configurable response period (default 30 days). The
  `compliance_policies` table stores regulatory rules per channel for CAN-SPAM,
  TCPA, GDPR, WhatsApp Business, and CCPA with JSON-based rule definitions. The
  `compliance_audit_log` table records all compliance-related actions with severity
  levels (info, warning, critical) for full audit trail. The `compliance_settings`
  table configures per-org preferences: GDPR response days, consent requirements,
  suppression checks, CAN-SPAM unsubscribe links, TCPA consent, WhatsApp opt-in,
  and data retention. Pre-send compliance checks verify suppression status and
  channel-specific requirements (unsubscribe links for email, STOP instructions
  for SMS). Five default policies are seeded on first run. API endpoints at
  `/compliance/*` support summary dashboard, suppression list CRUD and check,
  GDPR request CRUD, policy CRUD, audit log retrieval, compliance check, and
  settings CRUD. The frontend dashboard (at `/compliance`) provides six views:
  overview (metrics cards for suppressed contacts, pending GDPR requests, active
  policies, critical violations; suppression by reason and GDPR by type charts;
  recent audit activity), suppressions (filterable list with channel/reason filters,
  add suppression form, remove action), GDPR requests (status filter, create request
  form, lifecycle actions: start processing, mark completed, deny), policies
  (create policy form, enable/disable/delete actions), audit log (severity filter,
  full audit trail table), and settings (GDPR response days, data retention,
  consent toggles, CAN-SPAM/TCPA/WhatsApp settings, physical address).

- **Phase 42 — Conversation Search & Discovery**: Full-text search across
  conversations, messages, and contacts with saved searches, search history,
  and analytics. The `saved_searches` table stores user-defined search queries
  with name, query text, search type (conversation, message, contact, all),
  JSON filters, pin status, and usage count. The `search_history` table tracks
  all searches with query, type, result count, filters, duration, and timestamp.
  The `search_analytics` table aggregates search metrics over time windows:
  total searches, average result count, average duration, zero-result searches,
  top queries, and top search types. The `search_settings` table configures
  per-org preferences: max results, search scope toggles (message content,
  contact details, conversation metadata), history tracking, auto-suggest,
  minimum query length, and match highlighting. The service provides LIKE-based
  full-text search across conversations (subject, contact_id), messages (body),
  and contacts (phone, email, first_name, last_name) with case-insensitive
  matching. Each search is automatically recorded in history with result count
  and duration. Saved searches support pin, run, and delete actions with usage
  tracking. Analytics snapshots aggregate 7-day search metrics. API endpoints
  at `/search/*` support summary dashboard, full-text search, saved search CRUD
  and pin/use, search history retrieval and clear, analytics snapshot generation,
  and settings CRUD. The frontend dashboard (at `/conversation-search`) provides
  four views: search (search bar with type filter, summary cards for searches
  today/7d/avg results/zero result rate, results grouped by type with snippets
  and matched fields, top queries chart), saved searches (filterable list with
  pin/run/delete actions, create modal for new saved searches), history (search
  history table with query, type, result count, duration, timestamp; clear
  history button), and settings (toggle grid for search scope, history tracking,
  auto-suggest, match highlighting; numeric inputs for max results and minimum
  query length).

- **Phase 43 — Slack Channel Integration**: Added Slack as a supported channel
  in the unified inbox, enabling two-way messaging via Slack App (bot) integration.
  Mirrors Intercom's Slack channel support. The `slack_commands` table stores custom
  slash commands (/help, /agent, /status) with active/inactive status. The
  `slack_events` table tracks incoming webhook events (message, app_mention,
  reaction_added) with channel ID, user info, message content, raw payload, and
  processed status. The `slack_settings` table configures per-org preferences:
  auto-respond, greeting/away messages, allowed channels, blocked words, and event
  logging. Backend routes in `inboxes.rs` handle get/update channel config, live bot
  verification via Slack API (api.test), test message sending via chat.postMessage,
  and event listing. The frontend dashboard (at `/channels/slack`) provides four
  views: credentials (bot token, app ID, signing secret, connection status), settings
  (auto-respond toggle, greeting/away messages), commands (slash command list with
  active/inactive status), and diagnostics (test message sender, recent events).

- **Phase 44 — System Health Check & Onboarding Wizard**: Guided setup flow for
  new organizations to configure all Communicator features and verify every component
  is working. The `onboarding_steps` table defines 16 setup steps across 6 categories
  (general, channels, AI, analytics, compliance, integrations) with display order,
  required/optional flag, and route links to relevant dashboards. The
  `onboarding_progress` table tracks per-organization completion status with
  timestamps and completed-by tracking. The `health_check_results` table stores
  component health status (database, Claude API, channel inboxes) with status
  (healthy/degraded/unhealthy), messages, and check duration. The
  `onboarding_settings` table configures per-org preferences: show onboarding,
  auto health check, and check interval. The service provides onboarding summary
  with completion percentage, category breakdown, and next recommended steps.
  Health checks verify database connectivity, Claude API key configuration, and
  inbox status for all configured channels. API endpoints at `/system-health/*`
  support summary dashboard, step listing, progress tracking, step completion,
  health check execution, health summary, and settings CRUD. The frontend dashboard
  (at `/system-health`) provides three views: onboarding wizard (progress cards,
  category breakdown with progress bars, recommended next steps, full step list
  with mark-done and navigate actions), system health (overall status badge,
  component count cards, detailed component health table with run-check button),
   and settings (onboarding visibility toggle, auto health check toggle, check
   interval input).

- **Phase 46 — Omni-Channel Broadcast System**: Design and send rich media broadcasts
  across all channels in one click. The `broadcasts` table stores broadcast definitions
  with title, status (draft/scheduled/sending/sent/failed/paused/cancelled), body text,
  HTML body, subject line, preview text, media type (text/image/video/link/mixed),
  primary media URL, media caption, link preview (URL, title, description, image),
  targeting (contacts/segments/all_contacts with target IDs), channel selection (JSON
  array of channel codes), per-channel formatting overrides, and analytics counters
  (total recipients, sent, delivered, failed, read, clicked, replied). The
  `broadcast_media` table tracks uploaded media files with type, file name, size,
  MIME type, URL, thumbnail, and processing status. The `broadcast_channels` table
  stores per-channel formatting and delivery status: formatted body, formatted media
  URL, formatted subject, recipient/sent/failed counts, and timestamps. The
  `broadcast_recipients` table tracks individual recipient delivery: contact ID,
  channel, phone/email/external ID, status (pending/queued/sending/sent/delivered/
  read/failed/skipped), message ID, and event timestamps. The `broadcast_logs` table
  provides an audit trail of all broadcast operations. The `broadcast_settings` table
  configures per-org preferences: enabled, max recipients, send delay, rate limit,
  scheduling, approval workflow, auto-format, and analytics tracking. The service
  provides per-channel auto-formatting: WhatsApp (markdown, 4096 chars), Email (full
  HTML), SMS (plain text, 160 chars), Instagram (500 chars), Facebook (64000 chars),
  Telegram (MarkdownV2, 4096 chars), Discord (markdown, 2000 chars), Slack (4000 chars).
  One-click send creates channel records, recipient records, and simulates sending
  to all selected channels. Format preview shows how the message will appear on each
  channel. API endpoints at `/broadcasts/*` support summary, CRUD, status update,
  send, format preview, available channels, media upload/list/delete, channel/recipient/
  log listing, and settings CRUD. The frontend dashboard (at `/broadcast`) provides
  five views: overview (summary cards, analytics row, by-channel breakdown with
  progress bars, recent broadcasts, how-it-works guide), broadcasts (filterable list
  with view/send/delete), new broadcast (campaign type selection screen, then either
  Omni-Channel or Channel-by-Channel composer), channels (available channel grid with
  capabilities), and settings (toggle grid and numeric inputs).

  **Campaign Type Selection**: When creating a new broadcast, users first choose between
  two pathways: (1) **Omni-Channel Campaign** — write one message with text, images,
  videos, and link previews; the system auto-formats and sends to all selected channels
  in one click; or (2) **Channel-by-Channel Campaign** — select multiple channels and
  compose a separate tailored message for each channel, with per-channel media, audience
  targeting, and scheduling. The Channel-by-Channel composer uses a tabbed interface
  where each selected channel gets its own tab with a full message composer (body text
  with character limit, media type selector, media URL and caption, link preview,
  target audience, and schedule). Each channel tab shows a completion indicator (green
  check if body text is filled, yellow dot if empty). Creating a Channel-by-Channel
  campaign creates separate broadcast records for each channel, each with its own
  tailored content and delivery tracking.

  **HTML Auto-Format**: When the user pastes HTML into the HTML Body field, the system
  automatically converts it to each channel's native format. The backend provides
  `POST /broadcasts/html-preview` for live preview without creating a broadcast. The
  conversion engine includes: `html_to_plain_text()` (strips tags, preserves links as
  text, decodes HTML entities, adds newlines for block elements), `html_to_whatsapp_markdown()`
  (converts <b>/<strong> to *bold*, <i>/<em> to _italic_, <s>/<del> to ~strikethrough~,
  <code> to ```code```), `html_to_telegram_markdown()` (MarkdownV2 with special char
  escaping), `html_to_discord_markdown()` (**bold**, *italic*, ~~strikethrough~~, `code`),
  `html_to_slack_markdown()` (*bold*, _italic_, ~strikethrough~, `code`). Email keeps the
  original HTML. The frontend shows a "Preview on all channels" button that opens a
  collapsible panel displaying the formatted output for each selected channel with
  character count, channel capabilities (HTML/Images/Video badges), and subject line.

- **Phase 45 — Email Templates & SMTP Management Dashboard**: Dedicated dashboard
   for managing transactional email templates and SMTP configuration. The backend
   (migration 0052, smtp_email service/routes) already existed with full
   functionality — SMTP settings CRUD, email template management with variable
   substitution, templated email sending, and send log audit trail. This phase adds
   the missing frontend: a five-tab dashboard at `/email-templates` with Overview
   (summary cards, category breakdown, recent activity), SMTP Config (create/edit/
   test SMTP server connections), Templates (search/filter/create/edit with HTML
   preview), Send Logs (audit table), and Test Send (send a templated email with
   variable substitution). A new summary endpoint (`GET /smtp/summary`) aggregates
   SMTP config counts, template counts by category, send log stats, and recent
   activity. Four default templates are seeded: OTP verification, welcome, password
   reset, and order confirmation. API endpoints at `/smtp/*`, `/email-templates/*`,
   `/email/send`, and `/email/logs` support all operations.

- **Phase 33 — Tone of Voice & Guidance**: Configurable AI personality, tone,
  answer length control, and custom instructions that coach the AI on support
  policies and communication style. Core part of Intercom's TRAIN phase. The
  `tone_config` table stores per-organization settings: tone style (professional,
  friendly, humorous, casual, formal), personality (helpful, enthusiastic, playful,
  direct, thorough), answer length (short, medium, long), language, formality level
  (1-10), humor enabled, empathy level (1-10), conciseness level (1-10), custom
  instructions, forbidden/required phrases, and greeting/closing/escalation templates.
  The `guidance_rules` table stores custom instructions with priority ordering, rule
  types (style, policy, safety, escalation), and enable/disable toggle — these are
  injected into the AI system prompt in priority order. The `tone_presets` table
  provides pre-built configurations (Professional, Friendly, Humorous, Concise,
  Detailed) for one-click application. The `tone_analytics` table tracks how tone
  settings affect satisfaction and resolution rates over time windows. The core
  `get_prompt_additions()` function assembles the tone instructions and guidance rules
  into formatted text blocks that are injected into the AI system prompt before each
  reply generation. API endpoints at `/tone-voice/*` support summary dashboard, config
  CRUD, preset application, guidance rule CRUD, prompt additions retrieval, and
  analytics snapshot creation. The frontend dashboard (at `/tone-of-voice`) provides
  five views: overview (status, current tone/personality/length, guidance rule count,
  active rules list), configuration (tone style/personality/length selectors,
  formality/empathy/conciseness sliders, humor toggle, template editors, custom
  instructions), presets (one-click apply for pre-built tone configurations), guidance
  rules (CRUD with enable/disable/edit/delete, priority ordering), and prompt preview
  (shows exactly what tone instructions and guidance rules are injected into the AI
  system prompt).

- **Issue Fixes (Post Phase 46)**: Resolved 15 identified issues across the codebase:
  (1) Fixed 10 duplicate migration numbers (renamed to 0062–0071), (2) Resolved `/broadcasts`
  route conflict in main.rs by renaming old WhatsApp-only routes to `/whatsapp-broadcasts`,
  (3) Consolidated frontend broadcast pages — removed old `/broadcasts` route and Broadcasts.tsx,
  keeping only `/broadcast` with BroadcastDashboard, (4) Reorganized Intelligence navigation from
  37 flat items into 9 visual groups (Knowledge, AI Engine, Analytics, Operations, Inbox,
  Outreach, Integrations, System) using group separators in the nav component, (5) Fixed
  routes/mod.rs lines 73-74 — replaced `pub use services::` with `pub mod` and created proper
  route files for nid_organization_sync and nid_brand_sync, (6) Created KNOWN_LIMITATIONS.md
  documenting all simulated/stub implementations, deferred features, and missing frontend pages.
  Full details in `docs/KNOWN_LIMITATIONS.md`.

## 10. Open decisions

Flagging rather than guessing on the items that materially change the
design:

- **Provider choice**: Twilio for SMS/WhatsApp + Postmark for email assumed
  above — confirm, since this affects webhook contracts and cost.
- **LLM provider**: Harvester already depends on `openai`; this plan assumes
  Claude for reply drafting for quality/safety on customer-facing text.
  Either is a drop-in behind the drafting interface.
- **Auto-send trust threshold**: who signs off on moving a reply category
  from "always human-reviewed" to "auto-send," and what's the rollback
  trigger if it goes wrong.
- **Data residency/compliance region**: affects where Communicator's DB and
  message content can be hosted (PII in chat transcripts).
