# Communicator

Automated, multi-channel communication engine for NEWAX leads.

Communicator takes qualified leads produced by [Harvester](https://github.com/Newax-Technologies/harvester),
reaches out to them over **email, SMS, and WhatsApp**, and carries on the
resulting conversation: answering questions about our services, handling
objections, and following up on a schedule until the lead replies, converts,
or opts out.

See [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) for the full architecture
and phased delivery plan.

## Part of the NEWAX ecosystem

This is one of several connected NEWAX repos. See [`ECOSYSTEM.md`](ECOSYSTEM.md)
for what this repo owns, what it must never own, and the rules for how it
talks to the others (short version: never directly — everything routes
through core-newax's Module Gateway/event bus).

## Running locally

```bash
python -m venv .venv
source .venv/bin/activate
pip install -e ".[dev]"
cp .env.example .env  # fill in DATABASE_URL at minimum

alembic upgrade head        # create schema
uvicorn communicator.api.app:app --reload  # http://localhost:8000/docs

pytest                      # runs without a live DB (compliance-gate + health tests)
```

## In one sentence

Fetch a qualified lead from Harvester → send a first-touch message on the
right channel → when the lead replies (or goes quiet), use their chat
history + our service catalog to draft the next message → get a human
approval (initially) or auto-send (once trusted) → repeat on a follow-up
cadence until closed, converted, or opted out.

## Relationship to Harvester

Communicator is a **separate service with its own database**, integrated
with Harvester over its REST API rather than direct DB access, so the two
systems can evolve and scale independently. See
[`docs/ARCHITECTURE.md#integration-with-harvester`](docs/ARCHITECTURE.md#integration-with-harvester)
for the exact contract, and
[`harvester/docs/communicator-integration.md`](../harvester/docs/communicator-integration.md)
for the endpoints Harvester needs to expose.

## Status

**Phases 0–46, Campaign Pathways, and HTML Auto-Format complete.** See the roadmap in `docs/ARCHITECTURE.md` for
full phase descriptions.

### Phase 46: Omni-Channel Broadcast System

Design and send rich media broadcasts across all channels. When creating a new
broadcast, users first choose between two campaign pathways:

**Omni-Channel Campaign** — Write one message with text, images, videos, and link
previews. Select multiple channels (WhatsApp, Email, SMS, Instagram, Facebook,
Telegram, Discord, Slack), target individual contacts or segments, and optionally
schedule for later. The system auto-formats the message for each channel's capabilities
and sends to all selected channels simultaneously.

**Channel-by-Channel Campaign** — Select multiple channels and compose a separate
tailored message for each channel. Each channel gets its own tab with a full message
composer: body text with character limit, media type selector, media URL and caption,
link preview, per-channel target audience, and per-channel scheduling. Creating a
Channel-by-Channel campaign creates separate broadcast records for each channel, each
with its own tailored content and delivery tracking.

Per-channel format preview shows exactly how the message will appear on each platform.
Full delivery tracking with per-recipient status, channel delivery stats, and activity
logs. Configurable settings for rate limiting, send delay, approval workflow, and
analytics tracking. Access at `/broadcast`.

### Phase 45: Email Templates & SMTP Management Dashboard

Added a dedicated dashboard for managing transactional email templates and SMTP
configuration. The backend (migration 0052, smtp_email service/routes) already
existed with full functionality — SMTP settings CRUD, email template management
with variable substitution, templated email sending, and send log audit trail.
This phase adds the missing frontend: a five-tab dashboard at `/email-templates`
with Overview (summary cards, category breakdown, recent activity), SMTP Config
(create/edit/test SMTP server connections), Templates (search/filter/create/edit
with HTML preview), Send Logs (audit table), and Test Send (send a templated
email with variable substitution). Four default templates are seeded: OTP
verification, welcome, password reset, and order confirmation.

### Phase 34: Translation Dashboard

Completed the Translation feature by adding the missing frontend dashboard.

### Phase 35: Embed Widget

Added embeddable help center and live chat widgets for external websites. Create
widget configs (help center, live chat, search, combined types), generate embed
scripts, track widget instances and events, and manage settings. Access at
`/embed-widget`.

### Phase 37: Reply Engine Integration

Connected all previously siloed knowledge features into the AI reply generation
pipeline. The reply engine now uses ReplyIntegrationService to combine:
(1) Knowledge Base entries with audience-based scoping, (2) Help Center published
articles, (3) Uploaded document chunks (PDFs, DOCX, crawled URLs), and (4) Tone of
Voice + Guidance Rules injected into the AI system prompt. Each AI-generated reply
tracks which knowledge sources were used, enabling transparency and analytics.
Configurable per-source toggles and limits. Test context build shows exactly what
the AI receives before generating a reply. Access at `/reply-integration`.

### Phase 39: ERP Fulfillment Dashboard

Completed the ERP Fulfillment feature with PostgreSQL migration and frontend dashboard.
Manages shipping zones (geographic delivery areas), shipping methods (courier services
with ETA), shipping rates (pricing rules per zone/method), return requests/RMA (full
lifecycle: requested → approved → in_transit → received → inspected → refunded → closed),
and manifests (shipment grouping by courier for dispatch). Return stats track total returns,
pending returns, and refunded amounts. Access at `/fulfillment`.

### Phase 40: Scheduled Tasks & Cron Jobs

Automates recurring operations across the platform with configurable cron schedules.
Seven built-in task types: metric_collection (collect system metrics), alert_evaluation
(evaluate alert rules), csat_send (send CSAT surveys), kb_crawl (re-crawl URL sources),
analytics_snapshot (generate analytics snapshots), operator_scan (run Fin Operator scan),
and cleanup (remove old logs). Each task has a cron schedule, JSON config, active/inactive
toggle, and execution history with run logs. Task queue for manual triggering and batch
processing. Scheduler settings control concurrency, timeout, retry policy, log retention,
and failure webhook notifications. Default tasks are seeded on first run. Access at
`/scheduled-tasks`.

### Phase 41: Compliance & Data Privacy Dashboard

Comprehensive compliance management for legally regulated multi-channel communication.
Manages suppression lists (contacts who should not receive messages) with per-channel
and per-reason tracking (opt_out, bounce, complaint, gdpr_deletion, manual, blocked).
GDPR data request handling with full lifecycle (access, deletion, portability,
rectification, objection) and auto-expiry after configurable response period.
Compliance policies for CAN-SPAM, TCPA, GDPR, WhatsApp Business, and CCPA with
per-channel rules. Pre-send compliance checks verify suppression status and channel-specific
requirements (unsubscribe links for email, STOP instructions for SMS). Full audit trail
logging all compliance actions with severity levels. Configurable settings for consent
requirements, data retention, and regulatory toggles. Access at `/compliance`.

### Phase 42: Conversation Search & Discovery

Full-text search across conversations, messages, and contacts with saved searches,
search history, and analytics. Search interface with type filter (all, conversations,
messages, contacts) and result snippets. Saved searches with pin, run, and delete
actions. Search history tracking with query, result count, and duration. Search
analytics with top queries, search type breakdown, and zero-result rate. Configurable
settings for max results, search scope, history tracking, and auto-suggest. Access at
`/conversation-search`.

### Phase 44: System Health Check & Onboarding Wizard

Guided setup flow for new organizations to configure all Communicator features
and verify every component is working. Onboarding wizard with 16 steps across
6 categories (general, channels, AI, analytics, compliance, integrations),
each linking to the relevant dashboard. Progress tracking with completion
percentage, required vs optional steps, and category breakdown. System health
check verifies database, Claude API, and all channel inboxes are functioning.
Configurable settings for onboarding visibility, auto health checks, and check
intervals. Access at `/system-health`.

### Phase 43: Slack Channel Integration

Added Slack as a supported channel in the unified inbox, enabling two-way messaging
via Slack App (bot) integration. Mirrors Intercom's Slack channel support. Features:
Slack bot credentials management (bot token, app ID, signing secret), live bot
verification via Slack API (api.test), slash commands (/help, /agent, /status),
auto-respond settings with greeting and away messages, test message sender via
chat.postMessage API, and event logging. Connection status shows bot user ID,
configuration state, and missing credentials. Access at `/channels/slack`.

### Phase 38: A/B Testing & Phased Deployments

Data-driven experimentation on AI configurations before full deployment. Create
experiments to test different confidence thresholds, retrieval strategies, response
formats, escalation triggers, tone styles, or models. Each experiment has variants
with JSON-configurable settings and a traffic split controlling how much traffic
goes to the control group. Conversations are randomly assigned to variants, and
metrics (resolution rate, satisfaction, response time, etc.) are recorded per
assignment. Generate summary snapshots to see statistical significance and the
recommended winning variant. Settings control min sample size, significance
threshold, and max concurrent experiments. Access at `/ab-testing`.

### Phase 36: Unified Analytics Dashboard

Comprehensive analytics layer aggregating all metrics from every phase into one
unified dashboard. Features: overview with key metrics (conversations, resolution
rate, CX score, CSAT, feedback satisfaction, validation pass rate), channel and
status breakdowns, topic distribution, autonomous resolution stats, procedure and
workflow stats, and agent leaderboard. Saved reports with 7 report types (unified,
conversation, agent, CX QA, CSAT, feedback, topic) and run-on-demand execution.
Scheduled reports with daily/weekly/monthly delivery to email recipients in CSV,
JSON, or PDF format. Report templates for quick report creation. Analytics snapshots
for time-series trend analysis. Configurable settings for auto-snapshot frequency
and data retention. Access at `/unified-analytics`.

The translation backend already existed with language detection, phrase-based
translation, external API integration, and auto-translation in conversations,
but had no dedicated management interface. The new dashboard provides five views:
Overview (stats cards, supported languages, current settings, how-it-works guide),
Languages (full table of all 29 supported languages with curated/detect-only status),
Detect Language (interactive language detection test), Translate (test translation
to any supported language), and Settings (enable/disable translation, auto-translate,
language detection, show original, default target language, fallback language,
API URL and key configuration). Access at `/translation`.

### Phase 33: Tone of Voice & Guidance

Configurable AI personality, tone, answer length control, and custom instructions
that coach the AI on support policies and communication style. Core part of
Intercom's TRAIN phase. Features: tone presets (Professional, Friendly, Humorous,
Concise, Detailed), guidance rules with priority ordering, formality/empathy/conciseness
sliders, greeting/closing/escalation templates, and prompt preview showing exactly
what gets injected into the AI system prompt. Access at `/tone-of-voice`.

- **Phase 0 (Foundations)** — done. Project scaffold, full data model
  (leads/contacts/consent/conversations/messages/services/follow-up
  sequences/drafts/audit log), Alembic migrations, Harvester API client,
  opt-out/consent compliance gate.
- **Phase 1 (Outbound MVP)** — done. Email/SMS/WhatsApp channel adapters,
  lead sync worker, first-touch send pipeline, outcome push to Harvester.
- **Phase 2 (Omnichannel inbound)** — done. Inbound webhooks for all three
  channels, unified conversation/message timeline.
- **Phase 3 (AI reply engine)** — done. Service catalog, Claude-based
  intent classification + draft generation, human approval queue
  (`/drafts`).
- **Phase 4 (Automated follow-up + auto-send)** — done. Follow-up
  sequence scheduler (quiet hours, weekly caps, WhatsApp 24h session-window
  awareness), auto-send dispatch for high-confidence drafts. Exposed as
  `/scheduler/*` endpoints for now — swap in a Celery/APScheduler beat once
  this needs to run unattended rather than via cron/manual trigger.
- **Phase 5 (Optimization)** — done. `SequenceStep` variants + `GET
  /sequences/{id}/variant-stats` for A/B testing; sentiment-based hot-lead
  alerts posted to a configurable Slack-compatible webhook
  (`SALES_ALERT_WEBHOOK_URL`); language detection + in-kind replies and
  translated follow-up templates (`Contact.language`).
- **Phase 6 (Feedback Collection Loop)** — done. Thumbs up/down on every
  outbound message (`POST /messages/{id}/feedback`); feedback stored in
  `message_feedback` table with per-author ratings; aggregated analytics via
  `GET /feedback/summary` (satisfaction rate, negative feedback with context).
  Feeds the self-improvement flywheel: content-gap detection, model tuning,
  and agent coaching.
- **Phase 7 (Validation Layer)** — done. Before an AI-generated draft is
  stored or auto-sent, a second independent Claude call validates the reply
  on two dimensions: **relevance** (does it answer the question?) and
  **grounding** (are claims supported by the knowledge base?). If either
  score falls below the configured threshold (`VALIDATION_RELEVANCE_THRESHOLD`,
  `VALIDATION_GROUNDING_THRESHOLD`, default 0.7), the draft is forced to
  `pending_review` regardless of confidence. Disable with
  `VALIDATION_ENABLED=false`. Validation results (scores, issues, latency)
  are returned with the draft and displayed in the conversation detail UI.
  - **Phase 8 (Monitoring & Alerting)** — done. Comprehensive observability
  layer with time-series metrics collection (feedback rates, validation scores,
  conversation counts, draft generation stats), configurable alert rules with
  threshold-based conditions and cooldown periods, and fired alert events with
  acknowledgment and resolution tracking. Health checks cover database, Claude API,
  and active alert status. Dashboard provides overview, alerts management, rule
  configuration, and metric detail views. Default alert rules monitor validation
  score, error rate, response time, negative feedback spike, and draft latency.
- **Phase 9 (AI-Powered Suggestions)** — done. When a customer gives negative
  feedback (thumbs down), the system automatically analyzes the conversation
  using Claude and generates structured suggestions for: new knowledge base
  entries for uncovered questions, FAQ additions for recurring topics, and
  agent behavior improvements. Each suggestion has a status lifecycle
  (suggested → approved/dismissed) with duplicate detection. Batch analysis
  processes all unprocessed negative feedback. API endpoints at `/suggestions/*`
  support listing, filtering, approving, dismissing, and triggering analysis.
- **Phase 10 (Copilot for Human Reviewers)** — done. When a draft sits in the
  approval queue, the human reviewer sees an AI-generated context package:
  conversation summary (Claude condenses the full history into 2-3 sentences),
  customer profile enrichment (lead info, contact details, conversation stats),
  key topics extracted from the conversation, customer sentiment indicator,
  and suggested edits for the draft (tone, clarity, completeness, accuracy,
  style). Context is cached for 1 hour to avoid regenerating on every refresh.
  API endpoints at `/copilot-reviewer/*` support getting context per draft,
  batch fetching for all pending drafts, and force-refresh. The copilot panel
  appears in the conversation detail UI as a collapsible purple section.
- **Phase 11 (Topic Categorization)** — done. Every conversation is auto-categorized
  into topics and subtopics using Claude (10 top-level topics: product_inquiry,
  pricing, support, sales, complaint, refund, scheduling, account, feedback, general)
  with confidence scores and extracted keywords. Keyword-based heuristic fallback
  when Claude is unavailable. Batch categorization of all uncategorized conversations.
  Manual topic override by human reviewers. Topic summary analytics include distribution,
  trending topics (7-day change), and top keywords. API endpoints at `/topics/*`.
  Frontend dashboard provides four views: overview, topic distribution, trending,
  and keyword cloud. Enables trend analysis and content gap identification.
- **Phase 12 (Simulation & Testing)** — done. Pre-deployment testing of AI responses
  against sample conversations before going live. Agent policies define guardrails and
  behavior rules. Simulation scenarios model customer personas and conversation flows.
  Interactive sandbox allows real-time testing of agent responses. Batch simulation runs
  test all scenarios against policies with pass/fail verdicts and detailed scoring.
  API endpoints at `/simulations/*`. Frontend dashboard provides three views: sandbox
  (interactive chat), scenarios (run and review), and policies (create and manage).
- **Phase 13 (Voice Channel)** — done. Voice and WebRTC calling integration for phone
  support directly inside the agent inbox. Supports Twilio Voice and WebRTC softphone
  with IVR, dual-track audio recording, and real-time transcription. Frontend page at
  `/channels/voice` with call controls, recording management, and transcription display.
- **Phase 14 (Web Chat Widget)** — done. Live chat messenger integration for website
  visitors, not just outbound leads. Supports real-time chat with visitor tracking,
  proactive messaging, and co-browsing capabilities. Frontend page at `/channels/live-chat`
  with widget configuration, active visitor monitoring, and chat management.
- **Phase 16 (CX Scoring & Automated QA)** — done. Every conversation is automatically
  scored on 5 dimensions using Claude: accuracy, empathy, resolution, compliance,
  and timeliness. Provides 5x more coverage than traditional CSAT surveys because it
  evaluates every conversation, not just the ones customers choose to rate. QA scoring
  rules are configurable with severity levels and score impact. CX summary provides
  overall score, dimension averages, trend analysis (7-day change), and top issues.
  Batch scoring processes all resolved/closed conversations without QA scores.
  If Claude is unavailable, keyword-based heuristics provide baseline scores.
  API endpoints at `/cx-qa/*`. Frontend dashboard provides three views: overview
  (CX score, dimension scores, trends, top issues), QA rules (enable/disable),
  and batch scoring.
- **Phase 15 (CSAT Surveys)** — done. Customer-facing satisfaction surveys complement
  the AI-powered CX scoring (Phase 16) with direct customer feedback. Survey templates
  define question, scale type (1-5, 1-10, emoji, binary), comment prompt, and trigger
  type (manual, auto_on_close, auto_on_send, scheduled). Individual survey instances
  track status (pending → sent → responded/expired). CSAT summary provides overall CSAT
  percentage, average score, response rate, score distribution, breakdown by channel,
  7-day trend comparison, and recent comments. Snapshot generation aggregates metrics
  over a 7-day window. API endpoints at `/csat/*`. Frontend dashboard at `/csat`
  provides three views: overview (CSAT score cards, score distribution bars, by-channel
  breakdown, recent comments), all surveys (filterable list with send/respond actions),
  and templates (survey template list).
- **Phase 17 (Fin Operator)** — done. Meta-agent for autonomous self-improvement that
  monitors system metrics (validation scores, feedback rates, error rates, resolution
  rates, response times), detects anomalies by comparing current metrics to previous
  snapshots, and generates structured improvement proposals using Claude. Proposals
  follow a review workflow (proposed → reviewing → approved → testing → deployed)
  with human approval before deployment. Configurable scan interval, auto-deploy for
  low-risk changes, confidence thresholds, and metrics selection. Performance log
  tracks every operator action. API endpoints at `/operator/*`. Frontend dashboard
  at `/operator` provides four views: overview (metrics cards, anomalies, proposals
  by status, recent activity), proposals (filterable list with approve/dismiss actions,
  expandable details), activity log (table of all operator actions), and configuration
  (enable/disable, scan interval, auto-deploy settings, confidence threshold).
- **Phase 18 (AI Procedures & Actions)** — done. Multi-step AI workflows that teach
  the AI how to handle complex processes like refunds, troubleshooting, and order
  changes. Procedures consist of ordered steps that can ask the customer for input,
  call external actions, or evaluate conditions. Actions are reusable API connectors
  (HTTP requests, webhooks, Stripe, internal APIs) that procedures invoke to perform
  real operations. Keyword-based procedure matching detects when a customer message
  triggers a procedure. Execution tracking records each step with input/output data.
  Default actions seeded for Stripe refunds, customer lookup, and internal order
  operations. Example refund procedure included. API endpoints at `/procedures/*`
  and `/actions/*`. Frontend dashboard at `/procedures` provides three views:
  overview (summary cards, category breakdown, recent executions), procedures
  (filterable list with create/delete), and actions (CRUD with enable/disable).
- **Phase 19 (Unified Inbox)** — done. Customer-centric inbox view that merges
  conversations across all channels (WhatsApp, Email, SMS, Facebook, Instagram,
  Telegram, LINE, Voice, TikTok, Web Chat) for the same customer into a single
  unified timeline. Uses the existing customer_channel_ids table for identity
  resolution. Provides summary dashboard with customer list, per-customer detail
  view showing all conversations and a merged message timeline with channel badges.
  Configurable settings per organization (group by customer/contact/channel,
  merge timeline, show channel badges, auto-resolve identity). API endpoints at
  `/unified-inbox/*` support listing customers, getting unified customer detail,
  resolving customer identity from channel + identifier, creating manual mappings,
  and managing settings. Frontend page at `/unified-inbox` provides customer list
  with search and filters, customer detail with profile info and conversations,
  and merged timeline view showing all messages chronologically with channel
  indicators.
- **Phase 20 (Visual Workflow Builder)** — done. No-code automation engine for creating multi-step
  workflows with conditional branching, assignment rules, and trigger-based execution. Workflows
  consist of ordered steps (wait, send_message, assign, tag, condition, notify, update_field,
  create_ticket, custom action) with configurable conditions for branching. Trigger types include
  event-based (conversation_created, message_received, state_changed, etc.), scheduled, manual,
  and webhook. Execution tracking records each step's result. Default workflows seeded for common
  use cases (new conversation notification, auto-assign by topic, escalate unresolved). API
  endpoints at `/workflows/*` support full CRUD, step/condition/trigger management, manual
  execution, and condition evaluation. Frontend dashboard at `/workflows` provides three views:
  overview (summary cards, by-type breakdown, recent executions), workflows (filterable list with
  create/activate/pause/delete/execute actions), and executions (execution history with status).
- **Phase 21 (Public Help Center)** — done. Customer-facing self-service portal with categorized
  articles, full-text search, and view analytics. Categories organize articles (Getting Started,
  How-To Guides, FAQ, Troubleshooting) with icons, descriptions, and display order. Articles
  support draft/published/archived lifecycle, pinning, SEO metadata, and helpful/not-helpful
  feedback tracking. Search records queries and result counts for analytics. Public endpoint
  returns the entire help center for embedding on external websites. Settings control name,
  tagline, primary color, search enablement, and article feedback. API endpoints at
  `/help-center/*` support summary, public overview, category/article CRUD, search, feedback,
  and settings. Frontend dashboard at `/help-center` provides four views: overview (summary
  cards, quick search, top articles, popular searches), articles (filterable list with
  create/edit/publish/pin/delete), categories (list with create modal), and settings
  (name, tagline, color, toggles, footer text).
- **Phase 22 (CRM Sync)** — done. Salesforce, HubSpot, and Pipedrive bidirectional sync
  for leads, contacts, and accounts/companies. CRM connections store provider, auth type,
  credentials, and sync config. Sync logs track each operation with record counts, success
  rates, and error details. Field mappings define how local fields map to CRM fields per
  object type (lead, contact, account) with direction (inbound/outbound/both). External
  references link local records to CRM records for identity resolution. Default field
  mappings seeded for Salesforce and HubSpot. API endpoints at `/crm-sync/*` support
  summary, provider listing, connection CRUD, sync triggering, sync logs, and field
  mappings. Frontend dashboard at `/crm-sync` provides four views: overview (summary
  cards, provider breakdown, pending/conflict alerts, recent syncs), connections
  (filterable list with create/sync/delete), logs (sync history table), and mappings
  (field mapping table).
- **Phase 23 (KB Document Upload & URL Crawling)** — done. Upload PDF, DOCX, TXT,
  HTML, and MD files to automatically populate the knowledge base. Documents are parsed,
  text is extracted, chunked with configurable size and overlap, and auto-converted to
  KB entries. Duplicate detection via SHA256 file hashing. URL crawling fetches webpages,
  extracts text from HTML, and creates KB entries from crawled content. Crawl scheduling
  with configurable intervals. Import settings control chunk size, overlap, max file size,
  auto-create KB entries, and crawl interval. API endpoints at `/kb-import/*` support
  summary, document upload/list/delete, URL source add/crawl/delete, and settings CRUD.
  Frontend dashboard at `/kb-import` provides four views: overview (summary cards, recent
  uploads, recent crawls), documents (file upload area, filterable list with delete),
  URL sources (add URL with crawl interval, crawl now, delete), and settings (chunk size,
  overlap, max file size, auto-create toggle, allowed file types).
- **Phase 24 (Discord Channel)** — done. Discord bot integration for two-way messaging
  via Discord API. Bot credentials (token, client ID, interaction secret) stored in inbox
  configuration. Bot profile fetched live from Discord API for verification. Slash commands
  (help, agent, status) seeded by default. Settings control auto-respond, greeting message,
  away message, allowed channels, blocked words, and event logging. Webhook endpoint for
  receiving Discord events. Test message sending to any Discord channel ID. API endpoints
  at `/inboxes/channels/discord/*` support get/update channel config, verify connection,
  send test message, and list events. Frontend page at `/channels/discord` provides four
  views: credentials (bot token, client ID, interaction secret, bot profile), settings
  (auto-respond toggle, greeting/away messages), commands (list of slash commands), and
  diagnostics (connection health, test message sender, recent events).
- **Phase 25 (Fin Vision)** — done. AI-powered image understanding that analyzes
  images/screenshots sent by customers using Claude's multi-modal vision capabilities.
  Supports 5 analysis types: general, screenshot, document, error_code, and product.
- **Phase 26 (Audience KB Scoping)** — done. Links customer segments (audiences)
   to specific knowledge base subsets so different customer groups see different AI
   knowledge. Mirrors Intercom's "Audiences control what Fin knows" capability.
- **Phase 27 (Brand Overrides)** — done. Enables dynamic brand switching mid-conversation
   or mid-workflow execution with override rules, switch logging, parent-sub-brand
   hierarchy, and settings. Mirrors Intercom's "Brand overrides in workflows" capability.
- **Phase 28 (MCP Protocol)** — done. Model Context Protocol support for extensible AI
  tool connections. Connect to MCP-compatible servers (Sentry, Guru, Pendo, Fullstory,
  Jira, GitHub, Slack) to enable the AI to use external tools during conversations.
  Server templates provide pre-built configurations for common integrations. Tool
  discovery automatically finds available tools from connected servers. Call logging
  tracks every tool execution for auditing and analytics. Configurable settings control
  auto-discovery, call logging, concurrency limits, and timeouts. API endpoints at
  `/mcp/*` support server CRUD, tool discovery, tool calling, template listing, settings
  management, and call log retrieval. Frontend dashboard at `/mcp` provides six views:
  overview (summary cards, top tools, recent calls), servers (create/manage connections),
  tools (discovered tool list), templates (pre-built server configs), calls (call log
  history), and settings (MCP behavior controls).
- **Phase 29 (Autonomous Resolution)** — done. Enables the AI agent to resolve conversations
  autonomously without human approval, with confidence-based auto-send thresholds, smart
  escalation rules, and resolution analytics. Mirrors Intercom's "Fin AI Agent" that resolves
  conversations autonomously across all channels. Resolution config controls auto-send threshold,
  validation requirements, max attempts, low-confidence escalation, and sensitive topic detection.
  Escalation rules define when and how to escalate from AI to human (confidence below threshold,
  topic match, keyword match). Agent policies provide fine-grained guardrails (no hallucination,
  sensitive topics, max attempts). Resolution logs track every autonomous attempt with confidence,
  validation score, outcome, and escalation reason. Analytics snapshots aggregate resolution rate,
  escalation rate, and top escalation reasons over configurable time windows. Disabled by default —
  enable when ready for autonomous operation. API endpoints at `/autonomous-resolution/*`.
  Frontend dashboard at `/autonomous-resolution` provides four views: overview (metrics cards,
  config, analytics snapshot, top escalation reasons, recent logs), escalation rules (CRUD with
  enable/disable), policies (agent behavior guardrails), and logs (resolution attempt history).
- **Phase 30 (Agent Performance Analytics)** — done. Per-agent metrics, coaching insights,
  and team benchmark comparison. Complements AI-powered CX scoring with human agent analytics.
  Performance snapshots track conversations handled, resolution rate, response times, satisfaction,
  feedback, drafts, escalations, and SLA compliance. Coaching insights suggest improvements based
  on performance data. Comparison metrics generate team-wide benchmarks and leaderboards.
  Configurable benchmarks for response time, resolution rate, satisfaction, and CX score.
  API endpoints at `/agent-performance/*`. Frontend dashboard at `/agent-performance` provides
  four views: overview (summary cards, benchmarks, insights, team comparison), snapshots
  (table with progress bars), insights (filterable list with create/address), and settings.
- **Phase 31 (Real-time Notifications & WebSockets)** — done. Push notifications, WebSocket
  broadcast, and notification preferences for live updates. Notification queue stores all
  notifications with status lifecycle (pending → sent → read/dismissed). Per-user preferences
  control which event types trigger push/email notifications (new messages, draft approvals,
  alerts, assignments, escalations, workflow completion, CSAT responses, suggestions, operator
  proposals). WebSocket connection tracking for broadcast targeting. Broadcast logging records
  recipient count and delivery status. Global settings control enabled state, WebSocket support,
  email notifications, rate limiting, digest mode, and retention. Quiet hours support for
  per-user notification scheduling. API endpoints at `/notifications/*` support summary,
  list/queue/broadcast notifications, mark read/dismiss, preferences CRUD, settings CRUD,
  connection listing, and cleanup. Frontend dashboard at `/notifications` provides four views:
  overview (metrics cards, by-type breakdown, recent notifications, active connections),
  notifications (searchable/filterable list with mark read/dismiss), preferences (toggle grid
  for all event types, quiet hours), and settings (global config).
- **Phase 32 (In-App Messaging & Proactive Triggers)** — done. Behavioral event-driven
  messaging system for triggered messages based on user actions, page views, time on site,
  and custom events. Enables proactive outreach like "Are you stuck?" popups, onboarding
  sequences, and re-engagement messages. In-app messages support multiple types (popup,
  banner, carousel, spotlight, modal) with configurable triggers (event, page view, time
  delay, segment) and frequency controls (once per session, once per user, every visit,
  once per day). Proactive triggers define when messages fire: page view with URL matching,
  custom events, exit intent, inactivity, and time on page. Trigger evaluation API allows
  testing triggers against simulated events. Behavioral events track user actions for
  trigger evaluation and analytics. Message events log shown/clicked/dismissed with
  automatic stat updates. Settings control enabled state, max concurrent messages,
  behavioral event tracking, and event retention. API endpoints at `/inapp/*` support
  summary, message CRUD, trigger CRUD, behavioral event recording/listing, message event
  recording, trigger evaluation, and settings CRUD. Frontend dashboard at `/inapp-messaging`
  provides five views: overview (metrics cards, by-type breakdown, recent events), messages
  (searchable/filterable list with create/activate/delete), triggers (list with create/
  enable/delete, test trigger evaluation), events (behavioral event table, record test
  event), and settings (toggles and inputs).

- **Phase 25 (Fin Vision)** details — Each analysis extracts objects detected, text via
  OCR, and generates a human-readable summary with confidence scores. Images are uploaded
  as base64-encoded data with MIME type validation and file size limits. Auto-analyze on
  upload is configurable. Settings control enabled state, auto-analyze, default analysis
  type, Claude model, max file size, OCR, and object detection. Graceful fallback when
  Claude is unavailable. API endpoints
  at `/vision/*` support summary, upload, analyze, list images, get image with analyses,
  delete, and settings CRUD. Frontend dashboard at `/fin-vision` provides four views:
  overview (summary cards, analysis type breakdown, recent analyses, status), images
  (upload button, filterable list with view/analyze/delete actions), analysis detail
  (image info, re-analyze with different type, full analysis results with objects,
  extracted text, and summary), and settings (toggles, model selection, file size limit).
