# Communicator

Automated, multi-channel communication engine for NEWAX leads.

Communicator takes qualified leads produced by [Harvester](https://github.com/Newax-Technologies/harvester),
reaches out to them over **email, SMS, and WhatsApp**, and carries on the
resulting conversation: answering questions about our services, handling
objections, and following up on a schedule until the lead replies, converts,
or opts out.

This repository currently contains the **architecture and delivery plan**
(see [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md)); implementation lands in
phases described there.

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

Planning stage. No application code yet — see the roadmap in
`docs/ARCHITECTURE.md` for the phased build-out.
