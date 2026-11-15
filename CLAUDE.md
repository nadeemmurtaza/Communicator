# Working Mode: Raw Prototype

This project is being built as a raw prototype. It will be finalized,
formatted, and linted later in Cursor. This is a standing rule for all
future work in this repo, not a one-off instruction.

When building anything here, act as a senior software architect and:

- Focus 100% on core business logic, API routing, database schemas, and
  overall architectural structure.
- Do not spend time on inline comments, strict formatting, or defensive
  boilerplate (input validation, exhaustive error handling, edge-case
  guards) unless it's critical to the core logic actually working.
- When generating multi-file structures, output them plainly (file path +
  full contents) so they're easy to copy into a local directory.
- Skip writing test suites and chasing CI/lint green unless explicitly
  asked for. Formatting and linting happen later, in Cursor.

## Git workflow

Still commit and push to a branch as work is produced, so nothing is
lost -- but skip the ceremony:

- No detailed PR descriptions, test plans, or verification write-ups
  unless asked.
- Don't write test suites just to prove something works.
- Don't chase CI/lint failures on prototype work -- that's Cursor's job
  later.

This applies going forward from here. It does not retroactively change
already-merged or already-reviewed work (e.g. the Phase 0-2 foundation
and the CI/CodeQL/Semgrep/Dependabot/SECURITY.md setup already in
place) -- those stay as they are.

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
more, plus a "teach by doing, not by lecturing" product rule — 31
rules total, identical across every NEWAX repo). These bind
every UI surface in this repo with no exceptions; read that section
before building or editing any UI.

See `UX_UI_RULES.md` for the 20 binding rules on how motion,
feedback, and layout should actually behave (Disney's 12 animation
principles adapted for interfaces, plus Fitts's/Hick's/Miller's/
Doherty/Postel's/Jakob's laws and hit-area/frequency-of-use rules) —
identical across every NEWAX repo, same binding status as
`CONVENTIONS.md`. Read it before building or editing any interactive
UI, alongside the anti-patterns section above.

See `COMPLIANCE_AND_ACCESSIBILITY_RULES.md` for 18 binding legal,
anti-dark-pattern, and accessibility requirements (Privacy Policy,
Terms of Service, consent banner, form data-minimization audits,
third-party SDK audits, no dark patterns/hidden fees/fake reviews/
unsupported claims, alt text, color contrast, keyboard navigation,
business details, age/parental consent, unsubscribe links, font/image
licensing, data deletion requests) — identical across every NEWAX
repo. These are launch blockers, not nice-to-haves; read it before
any surface that collects data, sends email, charges money, or
renders UI is considered shippable.

See `PRODUCTION_READINESS_RULES.md` (scale/database/reliability, 20
rules), `SECURITY_RULES.md` (20 security holes to close), and
`SAAS_UX_PATTERNS.md` (20 expected SaaS UX patterns) — all identical
across every NEWAX repo, same binding status as the files above.
Together they're what keeps a service that works fine in dev from
falling over, getting breached, or feeling unfinished the moment it
has real users.

See `UNSLOP_DESIGN_RULES.md` for 40 more rules on not looking like
generic AI-generated design: 20 concrete fixes (delete filler badges,
specific headlines, consistent icons, nested-radius math, and more)
plus 20 vocabulary terms this repo's UI work must get right (kerning,
optical centre, tabular numbers, safe area, progressive disclosure,
shared element transitions, and more), each tied to the exact rule it
implies here. Cross-referenced against, not duplicating, the files
above — read it alongside them before any UI work.
