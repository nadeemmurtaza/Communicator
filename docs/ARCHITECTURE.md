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
