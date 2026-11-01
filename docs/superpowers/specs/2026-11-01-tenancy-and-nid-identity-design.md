# Communicator Tenancy + nID Identity — Design

> **Status:** draft · **Date:** 2026-11-01 · **Owner:** NEWAX Engineering
> **Scope:** projects 1 and 2 of the decomposed WhatsApp/nID work. See §1 Non-goals.

## 1. What this is and is not

This spec does two things: it lands the multitenancy layer that already exists on
`feat/multi-tenant-automation-campaigns` onto `newax/finalization`, and it replaces that
branch's demo-grade identity code with a session and authorization model that can face the
internet. It is a prerequisite for per-tenant WhatsApp numbers and self-service automation.

**Non-goals (each gets its own spec later):**

- WhatsApp per-tenant number attachment and self-service automation pages. Needs this spec
  landed first; the `IntegrationSettings` singleton must become tenant-scoped before a second
  number can exist.
- Subscription and billing. nID ships no `/v1/billing/*` endpoint and its Rapid Gateway
  provider is an empty module (`nID/packages/payments/src/rapid-gateway.provider.ts` is
  `export {};`). The payment step cannot be integrated before it exists in nID.
- Adopting `@newaxhq/*` packages. The `newaxhq` GitHub org has zero repositories
  (`gh api orgs/newaxhq/repos` → `[]`); 82 of 137 core-newax packages are `"private": true`;
  `nid-client` is `@newax/nid-client`, Node-22+ TypeScript, UNLICENSED. A FastAPI service
  cannot import it, and its endpoints (`/oauth/authorize`, `/oauth/token`,
  `/api/v1/sessions/validate`) do not match the nID service that actually runs
  (`/v1/oauth/login-requests`, `/v1/oauth/token`). Nothing to consume.

## 2. Decisions

| # | Decision | Rationale |
|---|---|---|
| D1 | Integrate the branch with a **merge commit**, not a rebase | A rebase replays `c4d5e6f7a8b9` against nine tenant migrations and renumbers revision IDs across a 124-file chain, re-triggering the add/add auth conflict 48 times. One merge commit resolves two alembic heads once and preserves both chains. |
| D2 | Keep `newax/finalization`'s `nid_client.py` and `nid_auth.py`; **discard** the branch's copies | `nid_client.py` and `nid_auth.py` are add/add conflicts (+219/+111 vs +184/+90) — two independent implementations. The current-branch version calls the endpoints nID actually serves. The branch version's `validate_session` is `return _active_sessions.get(token)`. |
| D3 | **nID authenticates. Communicator authorizes.** | nID's code exchange returns only `{account: {id, phone, email, phoneVerifiedAt}}` (`nID/apps/api/src/modules/oauth/oauth.service.ts:145-152`) and issues no session token. ECOSYSTEM.md gives nID identity and no business data, so tenant membership is Communicator's record, not nID's. |
| D4 | Sessions become a **database table** with hashed tokens and expiry; the in-memory dict is deleted | A dict is lost on restart and is invisible to other workers. nID's own `AccountSession` uses `tokenHash @unique` + `expiresAt` (`schema.prisma:605-615`); mirror that shape. |
| D5 | Tenancy is derived from a **membership lookup against the authenticated account**, never from a header alone | Today `get_tenant_key` returns `x_tenant_key or "newax"` with zero binding to the account (`feat/...:api/dependencies.py:45-52`). Any caller can name another tenant — horizontal privilege escalation. |
| D6 | Mock auth is gated on the explicit flag **only**; the `app_env == "development"` clause is deleted | `config.py:9` defaults `app_env="development"` and the branch mints a valid session for *any* code when that is true. A production deploy with defaults would accept forged logins. |
| D7 | Enforce auth **route-file by route-file**, in one PR, with a written public allowlist | 44 route files exist and 39 are tenant-blind. A runtime "enforcement on/off" flag would ship a bypass; a per-file sweep with tests is the honest cost. |
| D8 | The `newax` tenant is created by **migration**, not by hand | Tenant rows today exist only via `POST /api/v1/organizations` (`organizations.py:77`) and everything else defaults to `"newax"`, which must currently be inserted manually. |

## 3. Current state, with evidence

**Tenancy on `newax/finalization`:** none. `models/integration_settings.py:10` pins
`SINGLETON_ID = 00000000-0000-0000-0000-000000000001` — one WhatsApp number and one business
profile per deployment, read by 11 call sites. `AutomationRule` has no tenant column and
`match_advanced_rule` (`services/whatsapp_automations.py:575-633`) selects every active rule
globally, so tenant A's inbound message fires tenant B's rule. The only `tenant_id` in the
tree is `models/live_proof.py:38` — a free-text `String(64)` filled from a request header
(`routes/live_proof.py:98-109`), not a usable pattern.

**Identity on `newax/finalization`:** `nid_client.py:161` reads `settings.environment`, which
does not exist on `Settings` — an `AttributeError` on the exact path taken when nID is
unreachable. Same bug at `live_proof.py:275`. `nid_client.py:57-58` falls back to hardcoded
`client_comm_default` / `secret_comm_default`. Sessions live in `_active_sessions` (:28).
`dev_auth_enabled` (`config.py:55`) is declared and referenced nowhere — dead. No route in
`api/app.py` has an auth dependency; `api/dependencies.py` provides only a DB session.

**What the branch contributes:** `models/tenant.py` (`tenant_key` unique `String(100)`, `name`,
`owner_account_id`); `tenant_key` on integration_settings (singleton removed), automation_rule
(+ per-tenant `channel`), contact, lead, tag, segment, broadcast, service, knowledge_base;
tenant-and-channel-scoped `match_advanced_rule`; `get_current_account_id` / `get_tenant_key`
dependencies; `routes/organizations.py`; frontend Login / AuthContext / ProtectedRoute /
Organizations; nine migrations chained `b3e5f6a7c8d9 → … → 33fe48fddf99`.

**Known defects to fix in the same effort:**

1. Alembic splits into two heads after merge — `c4d5e6f7a8b9` and `33fe48fddf99` share parent
   `b3e5f6a7c8d9`. Needs an explicit merge revision.
2. `routes/automation.py` — all 10 endpoints take `Depends(get_db_session)` and nothing else.
   No auth, no tenant filter.
3. `seed_default_whatsapp_automations` — dedup is `select(AutomationRule.name)` with no tenant
   filter (`whatsapp_automations.py:1124-1125`), so tenant two seeds zero rules; the seeder
   never sets `tenant_key`, so every seeded rule lands in the model default `"newax"`; and the
   `count()` at :978-979 is computed and discarded.
4. `config.py` — both sides add a `nid_*` block; `nid_redirect_uri` disagrees (`:8001` vs
   `:8000`).
5. `app.py` and `models/__init__.py` conflict only in additive import/include blocks.
6. Zero tenancy tests; `tests/conftest.py` unchanged on both sides; no tenant fixtures.
7. Migration ordering smell: `tenant_key` columns are added by `5a95ba8a58af`, `c41400fdcea1`
   and `7cd5381c8995` before `33fe48fddf99` creates `tenants`. It works only because
   `tenant_key` carries no FK. Keep it that way and say so in the migration docstring.

## 4. Design

### 4.1 Data model

```
nid_accounts
  id              UUID pk
  nid_account_id  text unique not null      -- "acc_..." from nID
  email           text null
  phone           text null
  phone_verified  bool not null default false
  created_at, last_login_at

nid_sessions
  id              UUID pk
  token_hash      text unique not null      -- sha256 of the bearer token; raw never stored
  nid_account_id  UUID fk -> nid_accounts on delete cascade
  expires_at      timestamptz not null
  created_at, revoked_at timestamptz null

tenant_memberships
  id              UUID pk
  tenant_key      text not null fk -> tenants.tenant_key (unique)
  nid_account_id  UUID fk -> nid_accounts on delete cascade
  role            text not null default 'member'
  created_at
  unique (tenant_key, nid_account_id)
```

The FK is safe here because `tenants.tenant_key` is unique (`models/tenant.py:23`) and both
tables are created in this work; the branch's *existing* `tenant_key` columns stay FK-free
because they are added by earlier migrations, before `tenants` exists. New tables take the FK,
old ones do not, and the migration docstring says so.

`nid_accounts` and `tenant_memberships` follow the pattern ECOSYSTEM.md points at as the
reference (`esp/nid_auth.go`: one nID Account to one local row via `nid_account_id`).
`tenant_memberships` is the table that closes D5's escalation: it is the only thing that can
turn "an authenticated account" into "an authorized actor for this tenant".

`Tenant.owner_account_id` stays as-is but changes type from `text` (default `""`) to a
nullable FK to `nid_accounts.id`, so an owner is always a resolvable principal.

### 4.2 Login sequence

```
browser            Communicator                     nID
   |  GET /auth/nid/start?redirect_uri=...              |
   |------------------->|  POST /v1/oauth/login-requests|
   |                    |------------------------------>|
   |  302 authorize_url  (pollToken)                    |
   |<-------------------|                               |
   |  user approves in nID login-ui                     |
   |  GET /auth/nid/callback?code&state                 |
   |------------------->|  POST /v1/oauth/token         |
   |                    |------------------------------>|
   |                    |  {account:{id,phone,email,phoneVerifiedAt}}
   |                    |<------------------------------|
   |                    |  upsert nid_accounts
   |                    |  insert nid_sessions (token_hash, expires_at)
   |  bearer in JSON body (no cookie)                   |
   |<-------------------|                               |
   |  every later call: Authorization: Bearer <token>   |
   |------------------->|  sha256 -> SELECT nid_sessions |
                        |  -> account -> memberships    |
```

Communicator mints its own bearer because nID hands back no session token. The token is
opaque, random (`secrets.token_urlsafe(32)`), stored only as a sha256 hash, and expires;
`expires_at` is checked on every request and `revoked_at` makes logout durable across
restarts and workers. State remains single-use (the existing `pop` at `nid_client.py:118` is
correct) and TTL-pruned.

### 4.3 Request authorization

One dependency, `require_tenant_member`, replaces every bare `get_db_session` on business
routes:

1. Read `Authorization: Bearer`. Reject missing/malformed → 401.
2. Hash, look up `nid_sessions`, reject expired/revoked → 401.
3. Read `X-Tenant-Key` (the header name the frontend already sends,
   `frontend/src/lib/api.ts:21`). Absent means the account's only membership if exactly one
   exists, otherwise 400 — not a silent `"newax"` fallback.
4. Look up `tenant_memberships(tenant_key, account)`. No row → 403. **This is the step that
   does not exist today and is the reason the escalation is open.**
5. Yield `(tenant_key, account_id)` to the route; every query in that route filters on
   `tenant_key`.

Public paths, unauthenticated and enumerated rather than pattern-matched:

- `/api/v1/webhooks/**` — authenticate by their own means already present in code:
  Twilio signature (`webhooks.py:42`), Meta app-secret HMAC, Postmark `?token=`.
- `/api/v1/widget/**`, `/api/v1/prechat/**`, the LiveProof portal UI — customer-facing
  surfaces whose visitors hold no nID session. They take per-conversation or per-verification
  tokens instead.
- `/api/v1/health`, `/docs`, `/openapi.json`, `/redoc`.
- `/api/v1/auth/nid/**` — the login flow itself.

Everything under `/api/v1/` not on that list requires a tenant membership. That includes
`automation-rules`, `conversations`, `contacts`, `drafts`, `sequences`, `broadcasts`,
`inboxes`, `teams`, `knowledge-base`, `reports`, `settings`, `scheduler`.

Frontend: `request()` in `lib/api.ts` attaches the bearer from stored session and sends
`X-Tenant-Key`; a 401 clears it and returns to Login. `ProtectedRoute` and AuthContext arrive
with the branch already, so this is wiring rather than new components.

### 4.4 Automation tenancy

- `match_advanced_rule(session, ..., tenant_key)` — keep the branch's filter; delete the
  `"newax"` default so a missing tenant is a `ProgrammingError`, not a silent cross-tenant read.
- Seeder dedup becomes `where(AutomationRule.tenant_key == tenant_key)`; seeded rows carry
  `tenant_key`; the discarded `count()` at :978-979 is removed. The hardcoded copy inside the
  seeded rules (office address at :1077, `https://calendly.com`, "Welcome to our WhatsApp
  Center", generic service text) moves to reading the tenant's `IntegrationSettings` row, which
  is exactly the row that becomes per-tenant in this merge.
- `routes/automation.py`: all 10 endpoints take `require_tenant_member`; `PUT /{id}` and
  `DELETE /{id}` fetch with `where(id == rule_id, tenant_key == tenant_key)` so another
  tenant's rule id returns 404, not 403-with-existence-proof.
- `run_inactivity_nudges` and `/scheduler/*` iterate per tenant and resolve the provider from
  that tenant's settings row rather than the singleton.

### 4.5 Credential and config hygiene

- Delete the `client_comm_default` / `secret_comm_default` fallbacks. A missing
  `nid_client_id` or `nid_client_secret` raises at startup with a message naming nID's client
  registration. Silent placeholder credentials cannot authenticate anyway
  (`oauth.service.ts:121-124` compares a hash) — failing loudly is strictly better than a
  403 per login.
- Fix both `settings.environment` reads to `settings.app_env` (`nid_client.py:161`,
  `live_proof.py:275`), then delete the `app_env` clause entirely per D6.
- `.env.example` gains the currently undocumented block: `NID_BASE_URL`, `NID_CLIENT_ID`,
  `NID_CLIENT_SECRET`, `NID_REDIRECT_URI`, `NID_VAULT_ADDR`, `NID_VAULT_ROLE_ID`,
  `NID_VAULT_SECRET_ID`, `NID_ALLOW_MOCK_AUTH`, `ECOSYSTEM_EVENT_BUS_URL`,
  `ECOSYSTEM_GATEWAY_KEY`, `PUBLIC_URL`, `SALES_ALERT_WEBHOOK_URL`,
  `POSTMARK_INBOUND_WEBHOOK_SECRET`, `NEURA_CRYPTO_*`, and the existing `META_*` set.
- Set `public_url` default to the port the app actually listens on (`http://localhost:8000`,
  per README) instead of `8001`, and build `nid_redirect_uri` from it via a model validator so
  the two cannot drift. Both are wrong today: `config.py:59` is `:8001` and `config.py:67` is
  `:8001/api/v1/auth/nid/callback`, while `uvicorn` serves `:8000`.

### 4.6 Testing

`tests/conftest.py` gains fixtures: `tenant` (two of them, `acme` and `globex`),
`nid_account`, `nid_session` returning a raw bearer alongside its row, and
`tenant_member` binding account→tenant. New tests, at minimum:

- **Escalation regression (the headline test):** account with no `globex` membership presenting
  `X-Tenant-Key: globex` gets 403. Before this spec's change the same request succeeds.
- Unauthenticated request to a business route gets 401; to each allowlisted path it succeeds.
- Expired session, revoked session, and garbage bearer each get 401.
- OAuth state is single-use: replaying the callback with the same state fails.
- Automation isolation: `match_advanced_rule` for tenant A never returns tenant B's rule;
  seeding for A does not block seeding for B; A cannot read, update or delete B's rule id.
- Mock auth: with `NID_ALLOW_MOCK_AUTH` unset, a `test_`-prefixed code is rejected even when
  `app_env == "development"`.
- Session durability: a session created, then reloaded through a fresh client, still validates
  (proves the dict is gone).
- `alembic upgrade head` runs to a single head; a test asserts `len(heads) == 1`.

Existing 230 tests must stay green. The four test files present on `newax/finalization` but
absent on the branch are kept.

## 5. Error handling

`NidAuthError` maps to 401 for session problems and 403 for authorization problems — they are
not the same failure and the response must not blur them. nID being unreachable during a
*new* login is a 503 naming the upstream; nID being unreachable during a *subsequent request*
is not an error at all, because validation is local (D4). The vault loader keeps its
existing behaviour of continuing on absent credentials, since boot-time secrets are optional
in development, but a vault that returns a non-200 for supplied credentials logs a warning
rather than silently yielding an empty dict.

## 6. Rollout order

1. Merge commit; resolve the two add/add auth files by taking D2.
2. Alembic merge revision; assert one head; `alembic upgrade head` locally.
3. Delete `_active_sessions`; add `nid_accounts`, `nid_sessions`, `tenant_memberships` +
   migration; migration-seed the `newax` tenant.
4. Config hygiene (§4.5) and the `app_env` crash fix.
5. `require_tenant_member` + membership check; wire the frontend bearer.
6. Automation scoping and seeder fix (§4.4).
7. Test sweep (§4.6) to green, then `ruff check`, `ruff format`, typecheck, build.

Steps 3-6 are **one PR**: once `_active_sessions` is deleted and `require_tenant_member`
exists, every business route returns 401 until the frontend sends a bearer, so splitting them
would ship a broken intermediate state. Steps 1-2 are a separate preceding PR (merge plus
migration reconciliation) that changes no behaviour. Step 7 is not a PR — it is the verification
gate PR 2 must pass before merge. The branch's frontend Login/AuthContext arrives in PR 1 and is
switched on in PR 2.

## 7. Risks

- **Cross-repo gap:** no nID endpoint answers "which tenants does this account belong to" for
  a relying party. `GET /v1/accounts/me` (AccountSessionGuard) returns the account;
  `GET /v1/tenants/me` needs an nID session or tenant API key that a Communicator user does
  not hold. D3 sidesteps this by keeping membership local, which is correct per ECOSYSTEM.md,
  but it means Communicator owns invites and no one else can answer "can this person act for
  this tenant".
- A 44-route sweep will touch files nobody has reviewed since the branch diverged; conflicts
  with the branch's own edits to those routes are resolved in favour of adding the dependency,
  not in favour of dropping it.
- Removing the singleton is a schema change against a live row keyed on `SINGLETON_ID`; the
  branch already rewrote it, and its 11 call sites must all be re-checked to pass `tenant_key`.
- The merge drags in 3,155 deleted lines of rewrites from the branch. Anything `newax/finalization`
  changed in those same regions (live-proof, crypto-mesh) needs a per-file read, not a
  blind `-X ours`.

## 8. Acceptance criteria

1. `alembic heads` reports exactly one head; `alembic upgrade head` completes on a fresh DB.
2. A login against a running nID produces a `nid_accounts` row, a `nid_sessions` row, and a
   bearer that survives an app restart.
3. No route under `/api/v1/` other than the enumerated allowlist returns data to an
   unauthenticated caller.
4. The escalation test in §4.6 passes, and fails when the membership check is removed.
5. Two tenants hold distinct WhatsApp credentials and distinct automation rules; a message to
   tenant A matches only A's rules.
6. `seed-defaults` run twice, once per tenant, installs 11 rules for each.
7. `pytest`, `ruff check`, `ruff format --check` and the frontend build are green.
