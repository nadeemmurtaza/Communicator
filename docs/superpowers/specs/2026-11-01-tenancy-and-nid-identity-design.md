# Communicator Tenancy + nID Identity — Design

> **Status:** draft (rev. 3 — nID holds authorization) · **Date:** 2026-11-01 · **Owner:** NEWAX Engineering
> **Scope:** project 0 (new work in the nID repo) + projects 1 and 2 in Communicator. See §1.

## 1. What this is and is not

This spec lands most of the multitenancy layer that already exists on
`feat/multi-tenant-automation-campaigns` onto `newax/finalization`, and makes **nID the authority
for authentication, tenant membership, and role**. Communicator stops deciding who may act for
which tenant and starts asking nID on every request. That requires new code in the nID repo,
specified in §4.1. "Most of": the branch's self-service organization creation is deleted rather
than merged, because it invents tenant identity a second time (D12).

**Non-goals (each gets its own spec later):**

- Creating the first tenant. After D12 a customer is onboarded by creating a `Tenant` in nID and
  inviting its staff; Communicator mirrors membership, it does not mint organisations.
- WhatsApp per-tenant number attachment and self-service automation pages. Needs this spec
  landed first; the `IntegrationSettings` singleton must become tenant-scoped before a second
  number can exist.
- Subscription and billing. nID ships no `/v1/billing/*` route and
  `nID/packages/payments/src/rapid-gateway.provider.ts` is `export {};`. Communicator's side of
  it — service credential, signed-webhook consumer, entitlement-gated pages — waits for that.
- Adopting `@newaxhq/*` packages. The `newaxhq` GitHub org has zero repositories
  (`gh api orgs/newaxhq/repos` → `[]`); 82 of 137 core-newax packages are `"private": true`;
  `nid-client` is `@newax/nid-client`, Node-22 TypeScript, UNLICENSED. A FastAPI service cannot
  import it and its endpoints (`/oauth/authorize`, `/api/v1/sessions/validate`) do not match the
  nID service that runs (`/v1/oauth/login-requests`, `/v1/oauth/token`). Nothing to consume.

## 2. Decisions

| # | Decision | Rationale |
|---|---|---|
| D1 | Integrate the branch with a **merge commit**, not a rebase | A rebase replays `c4d5e6f7a8b9` against nine tenant migrations, renumbers revision IDs across a 124-file chain, and re-triggers the add/add auth conflict 48 times. One merge resolves two alembic heads once and preserves both chains. |
| D2 | Keep `newax/finalization`'s `nid_client.py` / `nid_auth.py` as the base; **discard** the branch's copies, then port two fields the merged SPA depends on | They are add/add conflicts (+219/+111 vs +184/+90) — two independent implementations. The current-branch one calls the endpoints nID actually serves; the branch one's `validate_session` is `return _active_sessions.get(token)`. The SPA from the branch reads `data.dev_mode` (`AuthContext.tsx`), which the current-branch `LoginStartResponse` does not have, so taking one file wholesale silently changes the other branch's frontend behaviour — the merge is file-level plus a field-level port, and both `dev_mode` and the mock exchange end up gated on `nid_allow_mock_auth` only (D8). `GET /auth/me` differs in casing (`account_id` vs `accountId`) and **neither frontend calls it in either branch** (grepped), so its shape is a fresh decision here, not a conflict to preserve. |
| D3 | **nID is the authority for identity, tenant membership, and role.** Communicator stores no sessions and no membership table | Decided by the product owner. nID already holds `TenantStaffMembership` (`role`, `@@unique([tenantId, accountId])`) and `TenantActorGuard` already implements exactly this resolution — Communicator duplicating it in a local table would create a second source of truth that drifts. |
| D4 | Delegating a full `AccountSession` to a relying party is **rejected**. nID gains a new scoped, RP-bound, revocable session type | `AccountSession` is `{id, accountId, tokenHash, expiresAt, createdAt}` with no `clientId`, no scope column, and no `revokedAt` (`schema.prisma:605-615`), minted with `SESSION_TTL_DAYS = 30` (`accounts.service.ts:10,74`). Handing that to Communicator would be a 30-day unscoped credential to *all* of nID on behalf of the user. |
| D5 | nID gains one RP-facing **introspection** endpoint; Communicator calls it and caches the result for 60 s | Introspection is the standard OAuth shape for exactly this problem and keeps the authorization decision in nID. Per-request latency is bounded by the cache; correctness is bounded by the TTL. |
| D6 | nID's existing `X-Tenant-Id` convention wins; Communicator renames its branch's `X-Tenant-Key` | `TenantActorGuard` honours `X-Tenant-Id` (`tenant-actor.guard.ts:71-85`). Two spellings for the same concept across the ecosystem guarantees drift. |
| D7 | Communicator maps nID **role → product capability** locally, using the RBAC engine that already exists (`services/rbac_service.py`); nID does not gain a permissions engine | nID's role vocabulary is one unvalidated free-text default `'STAFF'` (`tenant-staff.service.ts:55-56`, `InviteStaffDto:17-18` has no enum) and zero permission strings; `TenantApiKey.scopes` exists but is enforced nowhere. Communicator is the opposite: `STANDARD_PERMISSIONS` already holds 22 keys, `RoleDefinition` and `check_member_permission` exist, and **nothing calls them on a request path** — only `routes/roles.py` imports the service. So D7 is not "invent an authz layer", it is "bind the one that already ships to nID's role", which is also the only way to avoid a second capability vocabulary. Who is an agent of which tenant is nID's; what an agent may do inside Communicator is Communicator's. **This is the one boundary this spec draws against a literal "all auth in nID" — overriding it means building a permissions system inside nID.** |
| D8 | Mock auth is gated on the explicit flag **only**; the `app_env == "development"` clause is deleted | `config.py:9` defaults `app_env="development"` and the branch mints a valid session for *any* code when that holds. A production deploy with defaults would accept forged logins. |
| D9 | Enforce auth route-file by route-file, with a written public allowlist, in the enforcement PR | 44 route files exist and 39 are tenant-blind. A runtime "enforcement on/off" flag ships a bypass; a per-file sweep with tests is the honest cost. |
| D10 | No migration-seeded tenant. A Communicator `tenants` row is **auto-provisioned**, idempotently, the first time one of its members is introspected | Rev. 2 said "keyed to a real nID tenant id", which a migration cannot know: `Tenant.id` is a deployment-specific cuid (`schema.prisma:86`). Tenant rows exist today only via `POST /api/v1/organizations` while everything defaults to `"newax"`, inserted by hand. Lazy upsert on `ON CONFLICT DO NOTHING` removes the hand-insert and keeps nID as the existence check. |
| D11 | nID work lands **before** Communicator's enforcement PR, not after | Communicator's enforcement PR depends on introspection responses. Shipping it against the current account-claims-only exchange would leave the escalation open behind a flag. |
| D12 | **Tenant provisioning belongs to nID.** The branch's `POST /organizations` self-service create is deleted, not adapted | It mints `tenant_key = _slugify(name)` and writes `owner_account_id` from the local session — a Communicator-invented key that exists in no nID `Tenant` row and grants no `TenantStaffMembership`. Under D3 every subsequent request for that "organization" is a 403. `GET /organizations` survives as a read-only projection of introspection memberships. |

## 3. Current state, with evidence

**nID, what exists.** `POST /v1/oauth/token` validates the client by
`constantTimeEqualHex(sha256Hex(clientSecret), client.clientSecretHash)` (`oauth.service.ts:122`),
enforces single-use via `codeConsumedAt` (:142), and returns **only**
`{account: {id, phone, email, phoneVerifiedAt}}` (:149-156) — no session token.
`approve()` writes `{status:'APPROVED', accountId}` and mints nothing (:74). The one-time code is
bound to clientId + redirectUri + accountId by the `OAuthLoginRequest` row; TTL is
`LOGIN_REQUEST_TTL_MINUTES = 10` (:6). `OAuthClient` is tenant-owned with `clientId @unique`,
`clientSecretHash`, `redirectUris String[]` (:622) — **no scopes, no TTL column**.
`GET /v1/accounts/me` is `AccountSessionGuard`; `AccountSessionGuard` proves a token by
`sha256Hex(token)` → `unscoped.accountSession.findUnique` → expiry check (`accounts.service.ts:81-92`),
with no revocation. `TenantActorGuard` is a true superset: API-key hash lookup, else
`resolveSession`, then `tenantStaffMembership.findMany({where:{accountId}})`, 403 on zero
memberships, honours/validates `X-Tenant-Id`, 400 when multi-tenant and no header
(`tenant-actor.guard.ts:29-89`). No endpoint returns "the tenants this account belongs to, with
roles" for a relying party.

**Communicator, tenancy:** none on `newax/finalization`. `models/integration_settings.py:10` pins
`SINGLETON_ID = 00000000-0000-0000-0000-000000000001` — one WhatsApp number and one business
profile per deployment, read by 11 call sites. `AutomationRule` has no tenant column and
`match_advanced_rule` (`services/whatsapp_automations.py:575-633`) selects every active rule
globally, so tenant A's inbound fires tenant B's rule. The only `tenant_id` in the tree is
`models/live_proof.py:38`, a free-text `String(64)` filled from a header
(`routes/live_proof.py:98-109`) — not a usable pattern.

**Communicator, identity:** `nid_client.py:161` reads `settings.environment`, which does not exist
on `Settings` — `AttributeError` on the exact path taken when nID is unreachable. Same at
`live_proof.py:275`. `nid_client.py:57-58` falls back to hardcoded `client_comm_default` /
`secret_comm_default`, which cannot authenticate anyway. Sessions live in `_pending_states` /
`_active_sessions` (:25-28) — both broken under more than one worker. `dev_auth_enabled`
(`config.py:55`) is referenced nowhere. No route has an auth dependency.

**Communicator, the login flow — broken on both branches, independently of tenancy.** Neither
`nid_client.py` can complete a real login. `start_login` calls `POST /v1/oauth/login-requests`
itself and returns `authorize_url = f"{self.base_url}/authorize/{poll_token}"`, but `base_url` is
`nid_base_url` (`config.py:64`, default `http://localhost:4000`) — the NestJS API, which serves no
`/authorize` route (only `@Controller('v1/oauth')`; grep for `authorize` in `apps/api/src` returns
guards, not a route). `/authorize` and `/authorize/[pollToken]` are pages in `apps/login-ui`, a
Next.js app on **:3300**. So the browser is sent to a 404 and no code is ever issued. The right
shape already exists: `login-ui/app/authorize/page.tsx:44-64` reads `client_id`, `redirect_uri` and
`state` from its own query string, creates the login request, and `…/[pollToken]/page.tsx:71-88`
claims the code and redirects to the stored `redirectUri` with `code` + `state`. **Communicator must
not call `login-requests` at all** — the pollToken is login-ui's, not the RP's — and there is no
config key for the login-ui origin today. Both branches also mint fake success on upstream failure:
`mock_poll_{state}` / `poll_fallback_{state}` authorize URLs (:104-113) and, at `nid_client.py:140`,
`data.get("access_token") or secrets.token_urlsafe(32)` — an nID rejection with no access token
yields a locally invented bearer. The mock account id is `f"acc_{hash(code) & …}"`, and Python's
`hash()` is salted per process, so even the dev path is unstable across workers.

The two files also differ on one check, and D2 keeps the weaker side: `nid_client.py:71-73` accepts
a caller-supplied `redirect_uri` if it `.startswith("http://localhost")` under the comment
"Validate redirect URI against configured allowed origins" — trivially satisfied by
`http://localhost.attacker.com`. The discarded branch copy is the one that got this right, with
`_is_allowed_redirect_uri` parsing scheme/host/port via `urlsplit`. §4.2 removes the parameter, so
nothing needs porting; recording it so nobody "restores the better version" instead.

**What the branch contributes:** `models/tenant.py` (`tenants` table, `tenant_key String(100)`
unique, `name`, `owner_account_id String(255) default ""`); `tenant_key` on integration_settings
(singleton removed), automation_rule (+ per-tenant `channel`), contact, lead, tag, segment,
broadcast, service, knowledge_base; tenant-and-channel-scoped `match_advanced_rule`;
`get_current_account_id` / `get_tenant_key`; `routes/organizations.py`; frontend Login /
AuthContext / ProtectedRoute / Organizations and `lib/api.ts`, which is where the bearer and `X-Tenant-Key` headers are actually attached to every
SPA request; nine migrations chained `b3e5f6a7c8d9 → … → 33fe48fddf99`.

**Defects to close in the same effort:**

1. Alembic splits into two heads after merge — `c4d5e6f7a8b9` and `33fe48fddf99` share parent
   `b3e5f6a7c8d9`. Needs an explicit merge revision.
2. `routes/automation.py` — all 10 endpoints take `Depends(get_db_session)` and nothing else.
   `routes/roles.py` is the same and worse: it manages the RBAC grants themselves, and
   `GET /roles/members/{member_id}/check` evaluates any member's permissions for any caller.
3. `seed_default_whatsapp_automations` — dedup is `select(AutomationRule.name)` with no tenant
   filter (`whatsapp_automations.py:1115`), so tenant two seeds zero rules; the seeder never
   sets `tenant_key`, so seeded rules land in the model default `"newax"`; the `count()` at
   :969-970 is computed and discarded.
4. `config.py` — both sides add a `nid_*` block; `nid_redirect_uri` (`:67`) and `public_url` (`:59`)
   both say `:8001` while uvicorn serves `:8000` (`README.md:30`) and the branch's Vite proxy sends
   `/api` to `:8000`. `nid_redirect_uri` additionally names the API path (`config.py:67`, read into
   `self.redirect_uri` at `nid_client.py:59`), not the SPA route that has to receive the code.
   Three ports, one intended meaning.
5. `app.py` / `models/__init__.py` conflict only in additive import and `include_router` blocks.
6. Zero tenancy tests; `tests/conftest.py` unchanged on both sides; no tenant fixtures.
7. Migration ordering smell: `tenant_key` columns arrive in `5a95ba8a58af`, `c41400fdcea1`,
   `7cd5381c8995` before `33fe48fddf99` creates `tenants`. Safe only because `tenant_key` has no
   FK. New tables take the FK; old ones do not, and the migration docstring says why.
8. `routes/organizations.py` is a second, divergent source of tenant truth: `POST` invents
   `tenant_key = _slugify(name)` with no nID counterpart, and `GET` enumerates by
   `Tenant.owner_account_id` rather than by nID membership. Both are deleted per D12; the branch's
   `owner_account_id String(255) default ""` keeps its column — §4.2 still needs local identity
   rows — but it stops being a filter and becomes display-only, set from the `OWNER` membership at
   auto-provision time.
9. `seed_default_whatsapp_automations`'s docstring promises 11 automations and names "Opt-Out
   Compliance (STOP)" and "Opt-In Re-engagement (START)" in that list; the `defaults` literal builds
   **10** (`whatsapp_automations.py:954-1124`, counted) and contains neither. `STOP`/`START` appear
   only in the Twilio-SMS path in `routes/inboxes.py:109-120`, never in a WhatsApp rule. §4.4 makes
   the seeder tenant-aware; it does not invent the two missing compliance rules, because opt-out
   handling is a product and carrier-compliance decision, not a tenancy mechanic. Recorded in §7.

## 4. Design

### 4.1 Project 0 — what gets built in nID

Follows nID's conventions: `apps/api/src/modules/<name>/` with controller/service/module/dto +
`*.spec.ts` (Jest, `apps/api/jest.config.cjs`); `prisma.forTenant()` for writes and
`prisma.unscoped` only where a table has an open-SELECT policy; RLS in the same migration that
creates a tenant-owned table (HOUSE-RULES rule 2); `timingSafeEqual` for every secret compare
(rule 4); env behind `core/config` and never logged (rule 6); done means
`pnpm typecheck && pnpm lint && pnpm test && pnpm db:validate` (rule 8). Nothing in §4.1 changes
how a login *starts* — `POST /v1/oauth/login-requests`, `approve` and `claim` already work and
`apps/login-ui` already drives them; only the token exchange and what a relying party may ask
afterwards are new.

**N1 — `OAuthClient` gains `scopes String[]` and `sessionTtlMinutes Int`.**
RP sessions must be bounded by what the client is allowed to do. Default scopes for
`communicator`: `identity:read`, `memberships:read`. Migration extends an existing bootstrap
table (open SELECT already in place, `20260917195555:208`).

**N2 — New table `RelyingPartySession`.**

```
RelyingPartySession
  id           TEXT pk (cuid)
  accountId    TEXT -> Account   ON DELETE CASCADE
  clientId     TEXT -> OAuthClient ON DELETE CASCADE
  tokenHash    TEXT @unique        -- sha256; raw token is returned once, never stored
  scopes       TEXT[]
  expiresAt    TIMESTAMP
  revokedAt    TIMESTAMP NULL      -- AccountSession has no equivalent; logout must be durable
  createdAt    TIMESTAMP
  @@index([accountId])
```

No `tenantId`: `Account` and `AccountSession` are deliberately non-tenant tables
(`schema.prisma:530-544`) because an account spans tenants. It gets the bootstrap shape —
open `FOR SELECT USING (true)` plus scoped insert/update/delete — because a token is resolved
before the tenant is known, exactly like `VerificationSession` (`001_verification_sessions:18`).

**N3 — `POST /v1/oauth/token` also mints an RP session.** Returns
`{account, session: {token, expiresAt, scopes}}`. Reuses `generateSessionToken` / `sha256Hex`
(already used at `accounts.service.ts:70`); TTL from `OAuthClient.sessionTtlMinutes`, defaulting
well below 30 days — 12 hours for `communicator`. Communicator then holds a token that is scoped,
short-lived and revocable, which is what makes D4's rejection of `AccountSession` a non-issue
instead of something the relying party has to work around.

**N4 — `POST /v1/oauth/introspect`, guarded by the RP's own client credential.**
Body `{token}`; server hashes, loads `RelyingPartySession`, checks expiry/revocation and that
the session's `clientId` equals the calling client, then resolves
`tenantStaffMembership.findMany({where:{accountId}})` joined to `Tenant` and returns:

```json
{
  "active": true,
  "accountId": "ckx…",
  "expiresAt": "2026-11-01T18:00:00Z",
  "scopes": ["identity:read", "memberships:read"],
  "memberships": [
    { "tenantId": "ckx…", "slug": "acme", "name": "Acme", "role": "STAFF" }
  ]
}
```

Response is `no-store`; the session row is `unscoped` read, writes via `forTenant`. Audit per
rule 3 is not required (no Subject PII), but the endpoint is rate-limited like checkout is in
the billing spec.

**N5 — `POST /v1/oauth/revoke`.** Sets `revokedAt`; idempotent; requires the client credential.
Communicator's logout calls it.

**N6 — Real role vocabulary.** `InviteStaffDto:17-18` accepts any string today and
`tenant-staff.service.ts:55-56` defaults to `'STAFF'`. Add an enum/const list
(`OWNER`, `ADMIN`, `AGENT`, `VIEWER`, `STAFF`) with class-validator enforcement at the DTO, and
reject unknown roles. Communicator's capability map (D7) depends on this being bounded —
otherwise one typo'd row silently grants or denies forever.

**Deliberately not built in nID:** permissions strings, per-resource authz rules, a policy
engine. D7 explains the boundary.

### 4.2 Communicator: login sequence

The branch's SPA already drives this flow and the shape is kept — `AuthContext.tsx` fetches
`start`, saves `state` in `sessionStorage`, navigates the whole browser, then fetches `callback`
and writes the token to `localStorage`. Communicator issues no 302 anywhere in this flow: the code
never did, and a 302 would strand the token in a top-level navigation the SPA cannot read.

```
SPA (browser)        Communicator API        nID login-ui :3300     nID API :4000
   | GET /auth/nid/start      |                     |                    |
   |------------------------->|  (no upstream call) |                    |
   | {authorize_url, state}   |                     |                    |
   |<-------------------------|                     |                    |
   | save state in sessionStorage                   |
   | window.location.href = {login_ui}/authorize?client_id&redirect_uri&state
   |--------------------------|-------------------->|                    |
   |                          |     POST /v1/oauth/login-requests        |
   |                          |                     |------------------->|
   |            user signs in and approves; login-ui polls, claims       |
   |   302 redirect_uri?code&state                  |                    |
   |<-------------------------|---------------------|                    |
   | GET /auth/nid/callback?code&state              |                    |
   |  (+ sessionStorage state check)                |                    |
   |------------------------->| POST /v1/oauth/token                     |
   |                          |---------------------------------------->|
   |                          | {account, session{token, expiresAt, scopes}}
   |                          |<----------------------------------------|
   | {token, account_id, email, memberships}        |                    |
   |<-------------------------|                     |                    |
   | every later fetch: Authorization: Bearer + X-Tenant-Id (lib/api.ts) |
```

Communicator stores **nothing** about the session: no `nid_sessions` table, no
`tenant_memberships` table, no `_active_sessions` dict. The SPA keeps nID's bearer and
`lib/api.ts` attaches it to every request. The callback does one introspection and returns its
memberships inside `LoginCallbackResponse`, because the SPA must pick a tenant before its first
business call and §4.3 step 3 answers "no header and more than one membership" with a 400.

Three things follow from the diagram:

- **It makes no nID call.** `authorize_url` is built from the new `NID_LOGIN_UI_URL` plus
  Communicator's own `client_id`, `redirect_uri` and `state`. The `POST /v1/oauth/login-requests`
  that both branches perform belongs to login-ui (`authorize/page.tsx:53`), and the `poll_token`
  field in `LoginStartResponse` is deleted with it. `mock_poll_` / `poll_fallback_` URLs go too:
  a login that cannot start says so.
- **`redirect_uri` is config-only.** The `?redirect_uri=` query param on `/auth/nid/start` is
  deleted, and with it `custom_redirect_uri` and the bypassable `startswith` compare in the file D2
  keeps (§3). Letting an anonymous caller name which registered URI receives the code buys nothing
  Communicator needs — its SPA callback route is fixed — and costs an attacker-controlled redirect
  surface.
- **That configured value currently points at the wrong place.** nID redirects the browser to
  `frontend/src/pages/NidCallback.tsx`, mounted at `/auth/nid/callback` in the branch's `App.tsx:47`,
  and *that page* fetches the API's `/api/v1/auth/nid/callback` with the code. `config.py:67` defaults
  `nid_redirect_uri` to the API path, so login-ui would hand the browser a JSON endpoint and
  `NidCallback` never runs. The default becomes `{public_url}/auth/nid/callback` — `public_url` being
  the SPA origin, `http://localhost:5173` in development (Vite proxies `/api` to :8000), and the API
  path stays an XHR target that is never a navigation destination.

`_pending_states` also goes: the OAuth `state` becomes an HMAC-signed compact value carrying the
nonce and Communicator's configured `redirect_uri`, verified with `hmac.compare_digest` on
callback. A server-side dict cannot survive more than one worker, and nID already validates the
redirect URI against `OAuthClient.redirectUris` (`oauth.service.ts:45`), so Communicator's job is
only to prove the callback matches a start this deployment issued. The SPA's `sessionStorage`
comparison stays — it answers a different question (did *this browser* start it) and is the branch's
own stated reason for keeping it tab-scoped.

`nid_accounts` stays, as a **reference** table, not an auth store:
`{id UUID pk, nid_account_id text unique, email, phone, display_name, last_login_at}`. It exists
because Communicator's own rows key people by local UUID — `Conversation.assigned_to_id`,
`Ticket.assigned_to_id`, `Approval.assigned_to_id` all point at `TeamMember.id`
(`models/team_member.py:16`) — and nID's cuid strings cannot be those FKs. So `TeamMember` gains a
nullable `nid_account_id` that resolves through this table, which is what D7's permission lookup
needs to find the caller at all. The row is upserted from introspection results and grants no
access by itself.

### 4.3 Communicator: per-request authorization

`require_tenant_member` in `api/dependencies.py`:

1. Read `Authorization: Bearer`; missing/malformed → 401.
2. `POST /v1/oauth/introspect` (client credential in `Authorization`, token in body). Cache keyed
   by sha256 of the token, TTL `NID_INTROSPECT_CACHE_TTL_SECONDS` default 60.
   `active:false` or expired → 401.
3. Read `X-Tenant-Id` — whose value is nID's `Tenant.id`, never a slug. Absent → the account's only
   membership if exactly one exists, otherwise 400. Never a silent `"newax"` fallback (that is the
   escalation).
4. Require `tenant_id ∈ memberships`; take role from the membership row. Not present → 403.
5. Yield `(tenant_key, role, account_id)`, where `tenant_key` is that same `Tenant.id` string —
   the branch's column name is kept so its nine migrations stay meaningful, and one value means
   one thing across both systems. Every query in the route filters by it.
6. nID unreachable with no cached result → 503 with `Retry-After`, **fail closed**. Cached read
   within TTL → serve, but writes require an entry no older than 15 s.

Communicator's `tenants` table then holds only product configuration for a tenant that exists in
nID — WhatsApp credentials, business profile, hours, automations — keyed by `tenant_key`. Step 4
passing means the tenant exists, so the row is created inside the same request on first sight
(`INSERT … ON CONFLICT (tenant_key) DO NOTHING`, D10); nothing is seeded and nothing is hand-inserted.
`"newax"` survives only as `DEFAULT_TENANT_KEY`, honoured when `app_env == "development"`, and
`Settings` refuses to boot if it is still the configured tenant while the app is not in development.

`GET /api/v1/auth/me` is the SPA's view of all this and must stop reading the deleted dict: it
becomes the introspection call itself, returning `{account_id, email, phone, memberships:
[{tenant_id, name, role}]}`. That response is what `Organizations.tsx` lists, what the tenant picker
switches, and what `GET /api/v1/organizations` projects (D12) — and the authorization
decision cannot disagree, because they are one read. Nothing in either branch's frontend calls
`/auth/me` today (D2), so `Organizations.tsx` gains that fetch in PR-2 instead of inheriting one.

Role → capability uses `rbac_service`, not a new module. PR-2 adds one dict from nID's N6 roles
(`OWNER`, `ADMIN`, `AGENT`, `VIEWER`, `STAFF`) to a `RoleDefinition` name that already exists
(`super_admin`, `admin`, `agent`, `restricted_agent`, `compliance_auditor`), resolves the caller's
`TeamMember` row through the `nid_accounts` link §4.2 adds, and asks
`rbac_service.check_member_permission(session, member.id, "conversations:reply")`. Routes ask for
permission keys; nothing else consults role. Three consequences worth stating:

- `routes/roles.py` has eight endpoints importing this service and **no auth dependency at all**,
  including `GET /roles/members/{member_id}/check?permission=…` (`:147-160`), which will happily
  evaluate another member's permissions for an anonymous caller. It is in the 44-file sweep like
  every other file, and its `member_id` path parameter is deleted — the endpoint answers for the
  caller only.
- Keys with no consumer today (`billing:view`, `billing:manage`, `sso:manage`) stay unconsumed. This
  spec does not add a billing or SSO route to justify them, and Communicator's `sso:manage` is a
  standing contradiction with ECOSYSTEM's "nID is the single identity provider" that §7 records.
- A caller whose nID role maps to a `RoleDefinition` with `custom_permissions` overrides on their
  `TeamMember` gets the union, per the existing service. Nothing in PR-2 changes that resolution;
  it only supplies the identity that was previously missing.

### 4.4 Automation tenancy

- `match_advanced_rule(session, ..., tenant_key)` — keep the branch's filter, delete the `"newax"`
  default so a missing tenant is a `TypeError` at the call site, not a silent cross-tenant read.
- Seeder dedup becomes `where(AutomationRule.tenant_key == tenant_key)` (the `select` at :1115);
  seeded rows carry `tenant_key`; the dead `count()` at :969-970 is removed; the docstring stops
  claiming eleven. Hardcoded copy inside the seeded rules (office address at :1077,
  `https://calendly.com` at :1064, "Welcome to our WhatsApp Center") reads the tenant's
  `IntegrationSettings` row — the same row that becomes per-tenant in this merge.
- `routes/automation.py`: all 10 endpoints take `require_tenant_member`; `PUT /{id}` and
  `DELETE /{id}` fetch `where(id == rule_id, tenant_key == tenant_key)` so another tenant's id
  returns 404, not 403-with-existence-proof.
- `run_inactivity_nudges` and `/scheduler/*` iterate per tenant and resolve the provider from that
  tenant's settings row, not the singleton.

### 4.5 Config and credential hygiene

- Delete the `client_comm_default` / `secret_comm_default` fallbacks; a missing
  `nid_client_id` / `nid_client_secret` raises at startup naming nID's client registration.
- Fix both `settings.environment` reads to `settings.app_env` (`nid_client.py:161`,
  `live_proof.py:275`), then delete the `app_env` clause per D8 — and delete its twin in the branch's
  `nid_auth.py`, `dev_mode = settings.app_env == "development" or settings.nid_allow_mock_auth`, so
  the flag is the only thing that enables mock login anywhere.
- New keys: `NID_LOGIN_UI_URL`, `NID_INTROSPECT_CACHE_TTL_SECONDS`,
  `NID_INTROSPECT_WRITE_MAX_AGE_SECONDS`. `nid_base_url`, `nid_client_id` and `nid_client_secret`
  already exist (`config.py:64-66`) — they need validation, not declaration. `public_url` defaults to
  `http://localhost:5173` (the SPA origin the Vite proxy assumes) and `nid_redirect_uri` is derived
  from it by a model validator, so the two cannot drift.
- `harvester_api_base_url` (`config.py:13`) defaults to `http://localhost:8000/api/v1` while
  `README.md:30` runs Communicator itself on the uvicorn default :8000 — with no `.env`, every
  Harvester call is a request to Communicator's own API. Out of tenancy scope, so this spec records
  it and does not silently pick Harvester's port; see §7.
- `.env.example` gains the currently undocumented block: `NID_*`, `ECOSYSTEM_EVENT_BUS_URL`,
  `ECOSYSTEM_GATEWAY_KEY`, `PUBLIC_URL`, `SALES_ALERT_WEBHOOK_URL`,
  `POSTMARK_INBOUND_WEBHOOK_SECRET`, `NEURA_CRYPTO_*`, and the `META_*` set.

### 4.6 Testing

`tests/conftest.py` (Communicator) gains fixtures: two tenants (`acme`, `globex`), an
`nid_account`, a fake nID introspection transport (httpx MockTransport) that can be told
active/expired/revoked/multi-membership, and `tenant_member` bindings. Minimum new coverage:

- **Login wiring:** `GET /auth/nid/start` answers with an `authorize_url` on the login-ui origin
  carrying `client_id`, `redirect_uri` and `state`, and issues **zero** requests to nID (asserted on
  the transport); `LoginStartResponse` no longer has `poll_token`.
- `/auth/me` and `GET /organizations` return the same membership set the introspection stub was told,
  proving D12's single-read claim rather than trusting two tables to agree.
- **Escalation regression (headline):** account with no `globex` membership sending
  `X-Tenant-Id: <globex's Tenant.id>` → 403. Today the same request succeeds.
- Unauthenticated request to a business route → 401; to each allowlisted path → succeeds.
- Expired, revoked, and garbage bearer each → 401; `active:false` → 401.
- nID unreachable: cached read within TTL served; write with a stale entry → 503; no cache → 503.
- Signed `state`: replaying the callback fails; a state issued for another redirect_uri fails.
- Automation isolation: A never matches B's rule; seeding A does not block seeding B; A cannot
  read/update/delete B's id.
- Auto-provision (D10): a first request for a membership with no local `tenants` row creates it, and
  ten concurrent first requests create exactly one.
- Capability mapping (D7): an account whose nID role is `VIEWER` is refused on a route asking
  `conversations:reply` and allowed on one asking `conversations:view`; `roles.py`'s
  member-check endpoint no longer answers for a member id the caller typed.
- Mock auth unset → a `test_`-prefixed code is rejected even when `app_env == "development"`.
- `alembic upgrade head` reaches one head; a test asserts `len(heads) == 1`.
- nID side: `RelyingPartySession` minted by exchange is rejected by `AccountSessionGuard`
  (proves an RP token cannot call the rest of nID), rejected by client B's introspect call,
  honoured after `revokedAt`, and unknown roles are refused by the DTO.

Existing 230 Communicator tests stay green; the four test files present on `newax/finalization`
and absent on the branch are kept.

## 5. Error handling

401 and 403 stay distinct — "we don't know you" versus "we know you and you may not". nID
unreachable during a *new* login is 503 naming the upstream; unreachable during a subsequent
request is a cache question, not a crash, and §4.3 step 6 fixes the answer. The vault loader keeps
continuing on *absent* credentials (development needs that) but a non-200 for *supplied*
credentials logs a warning instead of returning an empty dict silently.

## 6. Rollout order

1. **PR-N1 (nID):** `OAuthClient` scopes/TTL, `RelyingPartySession` + migration with bootstrap
   policies, session minted at token exchange, `introspect`, `revoke`, role enum. Ships dark —
   nothing consumes it yet.
2. **PR-1 (Communicator):** merge commit taking D2; alembic merge revision; assert one head;
   `alembic upgrade head` on a fresh DB. No behaviour change.
3. **PR-2 (Communicator):** one PR, because half of it 401s the product. Delete
   `_active_sessions` and `_pending_states`; signed `state`; login-ui `authorize_url` and the
   SPA-origin `nid_redirect_uri` (§4.5, §4.2); drop `login-requests`/`poll_token` and the
   caller-supplied `redirect_uri`; `nid_accounts` reference table; `require_tenant_member` +
   introspection cache; `X-Tenant-Key` → `X-Tenant-Id` in `dependencies.py` **and** `lib/api.ts`;
   `/auth/me` and `GET /organizations` re-based on introspection; `POST /organizations` deleted
   (D12); `TeamMember.nid_account_id` + the nID-role → `RoleDefinition` map and its use in
   `roles.py` (D7); config hygiene (§4.5); automation scoping and seeder (§4.4); frontend bearer
   wiring with `dev_mode` and `/auth/me` casing ported per D2. Splitting this ships an intermediate
   where the dict is gone, the guard is live, and the SPA still sends no bearer — every route 401.
4. **PR-3 (Communicator):** test sweep (§4.6) green, then `ruff check`, `ruff format --check`,
   typecheck, build. This is a gate on PR-2, not a separate deliverable.

PR-1 can merge today; it needs nothing from nID. PR-2 needs PR-N1 deployed, and a real login can
only be verified once PR-N1 is up, because today's `authorize_url` 404s (§3).

## 7. Risks

- **nID becomes a hard dependency for every authenticated request.** The 60 s cache bounds calls
  but not failure: nID down means new logins fail and cached principals expire into 503. Fail
  closed is the deliberate choice; the alternative is a local stale-allow window, which is a
  security decision nobody has made yet.
- **Cross-repo sequencing.** PR-N1 must ship and be deployed before PR-2 can pass CI. If PR-N1
  slips, PR-2 cannot merge; there is no interim path that does not reopen the escalation.
- **No endpoint today answers "which tenants for this account" for a relying party**, and
  `GET /v1/tenants/me` needs an nID `AccountSession` or tenant API key that a Communicator user
  does not hold. N4 is the fix, and it is new code in a repo this project does not own alone.
- A 44-route sweep touches files unmodified since the branch diverged; conflicts resolve in favour
  of adding the dependency, never dropping it.
- Removing the singleton rewrites 11 `get_settings_row` / `get_whatsapp_config` call sites to pass
  `tenant_key`; each needs a read, not a sed.
- The merge drags 3,155 deleted lines of branch rewrites. Regions `newax/finalization` also changed
  (live-proof, crypto-mesh) need per-file review — no blind `-X ours`.
- **D12 leaves no self-service onboarding.** Deleting `POST /organizations` means a new customer
  cannot create a workspace inside Communicator; somebody must create a `Tenant` in nID and invite
  the account's staff first. That is the correct owner, but it is a product gap this spec closes by
  naming it, not by solving it — and it is the thing most likely to make the reviewer say "so how
  does a customer sign up?"
- **Login now needs three origins to agree:** `OAuthClient.redirectUris` at nID, Communicator's
  `NID_REDIRECT_URI`, and the SPA route the browser lands on. Dev additionally needs nID API (:4000)
  *and* login-ui (:3300) running; with D8's mock gate narrowed, there is no way to click through
  Communicator's UI on a laptop that has neither.
- `harvester_api_base_url` collides with Communicator's own port under defaults (§4.5). Unfixed here
  because Harvester's real port is not knowable from this repo — it has no local checkout — and a
  default is not evidence. Needs one answer from the operator before §4.5's block is written into
  `.env.example`.
- **No WhatsApp opt-out exists** (defect 9). Meta's Cloud API requires honoring STOP-style
  unsubscribes, and the only opt-out keywords in the codebase belong to the Twilio SMS path. This
  spec does not add them, so after PR-2 the product still ships without one — now per tenant, which
  multiplies the decision (each tenant's own STOP list and quiet hours). Needs its own ticket,
  ideally before the second paying tenant.
- **Communicator's permission catalog still contains `sso:manage`** (`rbac_service.py:16-160`,
  alongside `billing:view`/`billing:manage`). ECOSYSTEM says nID owns SSO and Communicator owns no
  billing. D7 keeps the catalog because wiring it beats replacing it, but the three keys that
  contradict the ecosystem rules stay in the list, unused, after this spec. Deleting them is a
  visible product decision, not a cleanup PR-2 should make quietly.
- **Webhook header drift is unresolved and will bite project 5:** shipped code sends two headers
  `x-nid-timestamp` + `x-nid-signature` (plain hex), while the billing spec §9 documents one,
  `X-NID-Signature: t=…,v1=…`. Pin one in nID before any Python consumer verifies signatures.

## 8. Acceptance criteria

1. `alembic heads` reports one head; `alembic upgrade head` completes on a fresh database.
2. Clicking "Continue with nID" in a browser against a running nID API and login-ui reaches
   `NidCallback`, and the SPA holds a bearer. Communicator's code contains no call to
   `/v1/oauth/login-requests` and no `/authorize/{poll_token}` URL construction.
3. A login against a running nID yields an RP session whose token **cannot** call any non-OAuth
   nID endpoint, and **can** be introspected by Communicator and by no other client.
4. Communicator holds no session table and no membership table; `grep -rn "_active_sessions" src/`
   and `grep -rn "_pending_states" src/` return nothing.
5. Logging out revokes at nID: the token introspects `active:false` within one cache TTL.
6. No route under `/api/v1/` outside the enumerated allowlist returns data to an unauthenticated
   caller.
7. The §4.6 escalation test passes, and fails when the membership check is removed.
8. Two tenants hold distinct WhatsApp credentials and distinct automation rules; a message to A
   matches only A's rules; `seed-defaults` run once per tenant installs the same 10-rule set for
   both, and defect 9's docstring is corrected to stop claiming eleven.
9. `POST /api/v1/organizations` is gone, and the org list a person sees is exactly their
   introspected memberships.
10. nID: `pnpm typecheck && pnpm lint && pnpm test && pnpm db:validate` green. Communicator:
    `pytest`, `ruff check`, `ruff format --check`, typecheck and frontend build green.
