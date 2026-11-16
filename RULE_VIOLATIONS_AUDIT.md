# Rule Violations Audit (2026-11-15)

A point-in-time, breadth-first audit of both repos' actual code
against every rule in `CONVENTIONS.md`'s design anti-patterns,
`UX_UI_RULES.md`, `COMPLIANCE_AND_ACCESSIBILITY_RULES.md`,
`PRODUCTION_READINESS_RULES.md`, `SECURITY_RULES.md`,
`SAAS_UX_PATTERNS.md`, and `UNSLOP_DESIGN_RULES.md` — ~169 rules
total. This file is identical across both repos, same status as the
other rule files; update it whenever a violation gets fixed or a new
one is found, don't let it silently go stale.

**Method and limits:** this was a grep/signal pass, not an exhaustive
read of every file. A rule marked "unverifiable statically" means it
needs a rendered app, a manual flow walkthrough, or legal/content
review to actually check — not that it's assumed clean. A rule with
no finding either way in a repo this size may still have violations
this pass didn't surface; treat "no finding" as "not yet checked,"
not "confirmed clean," outside of the items explicitly listed under
"Confirmed followed."

---

## Communicator

### Violated

- **No Lucide icons** (design anti-pattern 2) — `apps/web/package.json`
  depends on `lucide`.
- **No Inter/Geist/Space Grotesk** (design anti-pattern 10) —
  `tokens.css`'s `--font-sans` is `'Inter'`. Already flagged in the
  file itself; not a new finding.
- **No drop shadows** (design anti-pattern 5) — `tokens.css`'s
  `--elevation-*` tokens are drop-shadow values. Already flagged in
  the file itself; not a new finding.
- **Privacy Policy / Terms of Service** (compliance 1–2) — no
  privacy/terms files or routes found anywhere in the repo.
- **Consent banner** (compliance 3) — no cookie/consent banner
  component found.
- **Real business details** (compliance 11) — no footer/about page
  states the operating entity's name/contact/address; the "footer"
  grep hits found are WhatsApp message-template footers, unrelated.
- **CDN** (production readiness 5) — no CDN configuration found.
- **Queue + background jobs** (production readiness 8) — no durable
  job-queue crate; async work is ad hoc.
- **Circuit breaker** (production readiness 17) — none found.
- **Monitoring + alerts** (production readiness 18) — no Sentry/
  Prometheus/Datadog dependency anywhere.
- **k6 load test** (production readiness 19) — no load-test script or
  directory.
### Fixed 2026-11-16 (Communicator, checked against a running backend)

- **Security headers** (security 17): `nosniff`, `X-Frame-Options`,
  `Referrer-Policy`, HSTS, a safe-subset CSP (`frame-ancestors`, `base-uri`,
  `object-src`) and `Permissions-Policy` on every response, the SPA pages and
  429s included (`backend-rs/src/hardening.rs`). A full `script-src` CSP still
  needs the frontend's inline scripts audited.
- **Lock down CORS** (security 8): methods and headers are an explicit list
  (the headers the web app sends), no longer `Any`.
- **Rate limiting** (production readiness 7 / security 5): per-client fixed
  window, 600 a minute by default (`RATE_LIMIT_PER_MINUTE`), `Retry-After` on
  429, health exempt, Cloudflare client IP honoured. It is in process memory,
  so with several instances it needs a shared store.
- **Timeouts** (production readiness 16): one shared reqwest client with a 5 s
  connect and 20 s total timeout replaces every default client. Retries and a
  circuit breaker are still absent.
- **Alt text** (compliance 16): the earlier finding was a false positive from a
  line-by-line grep; every `<img>` has `alt=` on the next line.
- Two empty directories (`crates/` and `services/in_app_messaging`) had no
  `Cargo.toml` and broke every `cargo` command through the workspace globs;
  removed.

### Confirmed followed

- **Health checks** — `GET /health` registered in `main.rs`.
- **Verify webhook signatures** — `services/signatures.rs` is actually
  imported and used in `routes/webhooks.rs` and `routes/live_proof.rs`.
- **Connection pooling** — `PgPoolOptions` used consistently across
  services.
- **Backup** — `docs/BACKUP.md` and `scripts/backup-databases.ps1`
  exist (whether failover is *tested*, per the rule's wording, is
  unverifiable statically).
- **Log admin actions** — `audit_log`/`AuditLog` referenced broadly
  across route files (`admin_system.rs`, `compliance_dashboard.rs`,
  `automation.rs`, `canned_macros_sla.rs`, `contacts.rs`, `otp.rs`,
  and more), not just the compliance-specific log.
- **Parameterized queries** — the known `compliance_dashboard::
  update_gdpr_request` bug (only `binds[0]` ever gets bound) is a
  correctness bug, not a SQL-injection violation: the query still
  uses `$n` placeholders and `.bind()`, never raw string
  interpolation of a user value.

### Unverifiable statically

Remove fake reviews / unsupported claims; color contrast ratio;
keyboard navigation; DB replication/sharding; age consent / parental
consent for children's data; hidden fees / dark patterns; file upload
type validation (upload routes exist — `omni_broadcast.rs`,
`erp_payments.rs`, `fin_vision.rs`, `commerce.rs`, `contacts.rs` —
but whether each validates real content vs. trusting client MIME
needs a per-file follow-up this pass didn't do); IDOR protection
(needs a per-endpoint ownership-check review, not conclusive from
spot checks at this repo's size); mobile-optimized layouts; skeleton
loader coverage (`Skeleton` found only in `ContactProfile.tsx` — other
pages' loading states need a broader per-page check).

---

## nID

### Violated

- **No Lucide icons** (design anti-pattern 2) — `lucide-react` (or
  equivalent) in `apps/profile-ui`, `apps/login-ui`,
  `apps/dashboard`, and `apps/verify-ui`'s `package.json`.
- **No Inter/Geist/Space Grotesk** / **no drop shadows** — same
  shared `tokens.css`, already flagged there.
- **Privacy Policy / Terms of Service** (compliance 1–2) —
  `apps/login-ui/app/privacy/page.tsx` (20 lines) and
  `app/terms/page.tsx` (19 lines) exist but are stub-sized —
  almost certainly placeholder, not real reviewed legal text.
  `apps/profile-ui` only has a verification-flow privacy note at a
  `.../verification/privacy` path, not a site-wide policy.
- **Consent banner** (compliance 3) — zero hits for consent/cookie-
  banner/GDPR anywhere in the repo.
- **Age consent** (compliance 6) — the `subjects` module has a
  `date_of_birth` field, but it's stored as verification data; no
  age-gate logic found at signup.
- **Parental consent for children's data** (compliance 7) — no
  parental-consent/COPPA logic found anywhere.
- **Data deletion request** (compliance 8) — zero hits for account-
  deletion/GDPR-deletion logic.
- **Unsubscribe link in emails** (compliance 9) — no email-sending
  module surfaced in this pass at all, so no unsubscribe mechanism
  either.
- **Real business details** (compliance 11) — no About/Contact/footer
  page found in nID's own apps.
- **Connection pooling, sized deliberately** (production readiness
  10) — `apps/rust-api/src/core/db/mod.rs:25` calls
  `PgPoolOptions::new().connect(database_url)` with no
  `.max_connections(...)` — a pool exists, but at sqlx's unconfigured
  default, not a deliberate size.
- **k6 load test** (production readiness 19) — none found.
- **Monitoring + alerts** (production readiness 18) — zero
  Prometheus/Sentry/Datadog dependencies in any `Cargo.toml`/
  `package.json`.
- **Row-level security** (security 2) — **partial, not repo-wide**:
  `apps/rust-api/migrations` has zero `.sql` files to check directly;
  the `set_config('app.tenant_id', ...)` + RLS pattern is only
  confirmed in `crates/subjects`, `verification_cases`, `sessions`,
  `brands`, and `organizations` — other tenant-scoped modules
  (`tenants`, `access`, others) show no RLS/tenant-context call found
  in this pass.
- **Lock down CORS** (security 8) — `apps/rust-api/src/main.rs:145`
  uses `CorsLayer::permissive()` — the exact wide-open configuration
  the rule bans, not an explicit origin allowlist.
- **Security headers** (security 17) — none found in `main.rs` or
  elsewhere.
- **Skeleton loaders** (SaaS UX pattern 2) — zero "Skeleton" component
  hits across `packages/` or any `apps/*` frontend.

### Confirmed followed

- **Health checks** — real `GET /health` endpoint at
  `apps/rust-api/src/modules/health.rs:14`.
- **Auth on every API route** — 18 of 19 `handlers.rs` files use
  `AuthenticatedApiKey`/`AccountSession`/`AuthenticatedActor`
  extractors; the one exception, `health.rs`, is the deliberate,
  visible public exception the rule itself allows.
- **Parameterized queries** — spot-checked `brands/service.rs:245`;
  the `format!()` there only interpolates a trusted constant column
  list, the actual value is bound as `$1`. No raw user-input
  string-built SQL found in this pass.
- **Log admin actions** — `audit_log_entry` writes found across
  `access`, `verification_cases`, `sessions`, `organizations`,
  `core/audit.rs`, and `crates/subjects` — broad coverage.

### Flagged (not a named rule, but adjacent — worth a second look)

- `tenants` table stores `webhook_secret_plaintext` and
  `billing_webhook_secret_plaintext` columns — the names imply the
  webhook signing secret itself is stored unhashed in the database.
  Not literally one of the 20 named security rules, but the same
  spirit as "verify webhook signatures" (security 10) — a leaked DB
  dump would hand over the signing secret directly.

### Unverifiable statically

All 20 `UX_UI_RULES.md` items (motion/feel requires a rendered app);
remaining `COMPLIANCE_AND_ACCESSIBILITY_RULES.md` items (per-form
audit, third-party SDK audit, dark patterns, hidden fees, fake
reviews, unsupported claims, alt text, contrast, keyboard nav);
most of `PRODUCTION_READINESS_RULES.md`'s Scale/Database items
(horizontal scaling, load balancer, auto scaling, CDN, Redis,
replication, sharding — infra-level, not visible in app code either
way); webhook signature *verification on the receiving side*
(security 10 — nID appears to send webhooks, a receiving handler
with signature verification wasn't located in this pass); storage
bucket config, HTTPS/deploy config (infra, not app code);
`UNSLOP_DESIGN_RULES.md`'s remaining 39 items (typography/animation/
layout vocabulary, all need visual inspection).

---

## What to do with this

This file is a snapshot, not a task tracker — it doesn't get a
checkbox workflow of its own. When a violation above gets fixed,
remove its bullet and note the fix in the fixing commit's message;
when a deeper pass finds something this breadth-first audit missed
(especially under "unverifiable statically" — IDOR, upload
validation, and skeleton-loader coverage were explicitly flagged as
needing a second, deeper pass even within what this audit could
reach), add it here the same way.
