# Communicator — Agent Notes

This file carries the same ecosystem-wide architecture plan as `CLAUDE.md`
in this repo, for agents/tools that read `AGENT.md` instead. See
`CLAUDE.md` for this repo's working-mode rules (raw prototype, no test
suites unless asked, skip CI/lint ceremony) and `ECOSYSTEM.md` for how
this repo relates to the other six repos in the NEWAX ecosystem.

## Repo Structure & Naming Conventions

See `CONVENTIONS.md` for the ecosystem-wide rules on folder layout,
module/service naming, file naming, renaming, branches, worktrees,
commits, and merge strategy — identical across every NEWAX repo. This
repo's layout gaps against it (root-level debug logs, `erp-schema/`
naming, archive folders, `backend-rs/`'s module split) were resolved
in the 2026-11-14 audit — see `CONVENTIONS.md`'s "Resolved" section
for what moved where.

See `COMMUNICATION.md` for how this repo is meant to talk to nID and
the rest of the ecosystem (Module Gateway/gRPC, the NATS event bus,
per-ID ownership), how modules talk to each other inside this one
repo, and how third-party platforms (Shopify, Meta, TikTok) fit in.
None of the cross-repo pieces exist yet — see its audit note.

See `ecosystem-stack.md` for the ecosystem-wide technology stack and
architecture plan (backend, frontend, database, cache, package
manager) and its honest compliance audit against what's actually
running here — identical content to nID's copy except where this
repo's specifics (paths, compliance status) differ.

See `CONVENTIONS.md`'s **"Design anti-patterns — strictly forbidden,
no exceptions"** section for the ecosystem-wide list of banned
visual/copy clichés (gradients, drop shadows, Lucide icons, Inter/
Geist/Space Grotesk, em dashes, bento grids, fake testimonials, and
more — 30 rules total, identical across every NEWAX repo). These bind
every UI surface in this repo with no exceptions; read that section
before building or editing any UI.
