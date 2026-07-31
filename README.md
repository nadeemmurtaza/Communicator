# Communicator

Automated, multi-channel communication engine for NEWAX leads.

Communicator takes qualified leads produced by [Harvester](https://github.com/Newax-Technologies/harvester),
reaches out to them over **email, SMS, and WhatsApp**, and carries on the
resulting conversation: answering questions about our services, handling
objections, and following up on a schedule until the lead replies, converts,
or opts out.

See [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) for the full architecture
and phased delivery plan.

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

**Phases 0-3 merged.** In place: project scaffold, the full data model
(leads/contacts/consent/conversations/messages/services/follow-up
sequences/drafts/audit log), Alembic migrations, a Harvester API client,
the opt-out/consent compliance gate, channel adapters (email/SMS/
WhatsApp) with outbound send, inbound webhooks and a unified conversation
timeline for all three channels, and a Claude-backed reply engine (intent
classification, service-catalog grounding, confidence-scored drafts) with
a human approval queue.

Not yet built (Phase 4-5): drafts marked `AUTO_APPROVED` still require a
human to hit `/drafts/{id}/approve` — nothing actually auto-sends yet; no
follow-up scheduler exists to advance `FollowUpSequence` steps or evaluate
`next_followup_at` (no Celery/APScheduler dependency at all); no quiet-hours,
working-hours, or per-lead rate caps; no WhatsApp 24h session/template-window
enforcement; the outcome push to Harvester is a hardcoded stub, and the
Harvester endpoints it targets (`qualified` filter, `/outreach-outcomes`)
don't exist yet on that side. See the roadmap in `docs/ARCHITECTURE.md` for
what's next.
