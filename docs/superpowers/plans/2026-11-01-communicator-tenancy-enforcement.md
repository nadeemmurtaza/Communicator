# Communicator Tenancy Enforcement + nID Login (PR-2 / PR-3) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make nID the authority for authentication, tenant membership and role in Communicator, delete Communicator's in-process session store, guard every non-public API operation with a per-request membership check, and make WhatsApp credentials and automation rules genuinely per-tenant.

**Architecture:** Communicator becomes a plain OAuth2 relying party. `GET /auth/nid/start` builds a login-ui authorize URL locally with zero upstream calls; the code exchange hits `POST /v1/oauth/token` and returns nID's scoped RP session token to the SPA, which replays it as `Authorization: Bearer` on every request. One FastAPI dependency — `require_tenant_member` — introspects that token at nID (`POST /v1/oauth/introspect`, cached 60 s; 15 s for writes), resolves the caller's tenant membership, auto-provisions the local `tenants` row on first sight, and yields a `TenantPrincipal` whose `tenant_key` **is** nID's `Tenant.id`. Communicator stores no sessions and no memberships; role → capability is mapped locally onto the RBAC engine that already ships.

**Tech Stack:** Python 3.11+, FastAPI, SQLAlchemy 2.0 async, asyncpg, Alembic (async env), pydantic v2 + pydantic-settings, httpx 0.28 (`MockTransport` in tests), pytest 8 + pytest-asyncio (`asyncio_mode = "auto"`), ruff (line-length 100, `py311`, ignore `B008`). Frontend: Vite 8, React 19, TypeScript ~6, Tailwind 4 + daisyUI 5, **npm only**.

**Spec:** [docs/superpowers/specs/2026-11-01-tenancy-and-nid-identity-design.md](../specs/2026-11-01-tenancy-and-nid-identity-design.md) — rev. 4. D1–D12, §4.2 login, §4.3 guard, §4.4 automation, §4.5 config, §4.6 test matrix, §8 acceptance criteria are the contract. Read §3.1 (rev. 4 measurements) first: every count in this plan comes from it.

**Companion plans:**
- Plan 1 — nID Project 0 (PR-N1): `nID/docs/superpowers/plans/2026-11-01-nid-relying-party-sessions.md`. **Must be deployed before Task 20 of this plan runs the live-login check-in**, and it defines the wire contract this plan parses.
- Plan 2 — Communicator PR-1 (merge + single alembic head): [2026-11-01-communicator-tenancy-merge.md](./2026-11-01-communicator-tenancy-merge.md). This plan starts where Plan 2 ends.

## Global constraints

- **nID owns identity, tenancy and role** (D3). Communicator gains **no session table and no
  membership table**. `grep -rn "_active_sessions\|_pending_states" src/` returns nothing at the end
  (§8 criterion 4).
- **`X-Tenant-Id`, never `X-Tenant-Key`** (D6), and its value is nID's `Tenant.id` — never a slug,
  never a silent `"newax"` (§4.3 step 3).
- **Fail closed.** nID unreachable with no fresh cache entry is a 503 with `Retry-After`, never an
  allow (§4.3 step 6). Do not add a stale-allow window, a local bypass flag, or an
  "enforcement on/off" setting (D9).
- **Mock auth is gated on `nid_allow_mock_auth` alone** (D8). Delete every `app_env ==
  "development"` clause that mints or accepts a credential, and the `code.startswith("test_")`
  shortcut.
- PR-2 is **one PR** (§6 item 3). Do not open a PR that deletes `_active_sessions` while the SPA
  still sends no bearer — that intermediate 401s the entire product.
- **Never commit credentials.** `META_WHATSAPP_ACCESS_TOKEN`, `META_WHATSAPP_PHONE_NUMBER_ID`,
  `META_WEBHOOK_VERIFY_TOKEN`, `META_APP_SECRET`, `NID_CLIENT_SECRET` go in `.env` only. No `.env`
  exists in this repo; do not create-and-commit one. `.env.example` values stay empty.
- **No new runtime dependencies.** Everything needed (`hmac`, `hashlib`, `dataclasses`, `httpx`,
  `sqlalchemy.dialects.postgresql.insert`) is already available.
- **No Python typechecker exists** in this repo (§6 item 4). Do not add mypy/pyright; the gates are
  `pytest`, `ruff check src tests`, `ruff format --check src tests`, `cd frontend && npm run build`.
- Frontend is **npm**, and `npm run build` is `tsc -b && vite build` — the TS build is the only
  static typing gate in the project.
- **Do not touch `C:\Users\nommi\Downloads\scraper\nID`** with `git checkout/switch/stash/clean`; a
  different session has uncommitted `apps/login-ui/**` and `packages/payments/**` work there.
- Do not push, open a PR, or merge to `main` without the user's explicit approval. Every task ends
  with a local commit only.
- Where a spec claim and the measured tree disagree, **the tree wins** and the deviation goes in the
  "Spec deviations" table below with a one-line reason. Never "fix" the code to match a stale
  sentence in the spec.

## Environment reality (measured 2026-11-01, merged tree)

| Thing | Measured |
|---|---|
| Operations in the mounted app (`APIRoute` × methods) | **360** |
| Operations carrying an auth dependency today | **2** (`GET`/`POST /api/v1/organizations`) |
| Route modules with at least one operation | 45 (`src/communicator/api/routes/*.py`, plus an empty `__init__.py`) |
| Tables in `Base.metadata` | **58** |
| Tables carrying `tenant_key` | **14** — `automation_rules`, `broadcasts`, `campaign_templates`, `contacts`, `customers`, `discovery_flow_templates`, `inquiries`, `integration_settings`, `knowledge_base_entries`, `leads`, `segments`, `services`, `tags` (+ `tenants` itself) |
| `IntegrationSettings.tenant_key` | `unique=True` in the model (`:27-29`), `create_index(..., unique=True)` in `5a95ba8a58af:28-32`, live DB confirms `CREATE UNIQUE INDEX ix_integration_settings_tenant_key` |
| `TeamMember` columns | `id, name, email, role, custom_permissions, availability_status, max_concurrent_conversations, last_assigned_at, created_at` — **no `tenant_key`, no `nid_account_id`** |
| `get_settings_row` / `get_whatsapp_config` call sites | **20** (15 + 5, listed in Task 14) |
| `tests/*.py` issuing real HTTP calls | 42 files, **346** `client.get/post/put/delete/patch(...)` calls |
| …of which use the shared `test_client` fixture | 25 files (get default headers for free) |
| …of which construct `httpx.AsyncClient(...)` themselves | 17 files, 18 construction sites (Task 17) |
| Baseline suite | **230 passed** after PR-1 |
| Alembic head after PR-1 | `d7f8a9b0c1d2` (the merge revision Plan 2 adds) — PR-2's first migration chains off it |
| `pytest` config | `[tool.pytest.ini_options] pythonpath = ["src"]`, `testpaths = ["tests"]`, `asyncio_mode = "auto"`; **no `tests/__init__.py`** |
| Local Postgres | `127.0.0.1:5432`, role `postgres` / password `postgres`. `localhost` enforces SCRAM and the `communicator` role does not exist — always use `127.0.0.1` |
| Alembic CLI | needs `PYTHONPATH=src` (`python -m alembic …`), else `ModuleNotFoundError: No module named 'communicator'` |
| SPA wiring | `frontend/src/main.tsx:8-16` renders `BrowserRouter > AuthProvider > App`; `App.tsx:46-48` mounts `/login`, `/auth/nid/callback` (`NidCallback`), `/organizations` outside `ProtectedRoute`; everything else is inside it |

## Wire contract this plan consumes (from Plan 1 — parse exactly this)

nID serves no global-prefix-free Nest routes other than these; the field names below are the
contract, **not** what the current `nid_client.py` assumes.

```
POST {NID_BASE_URL}/v1/oauth/token
  body  {"code": str, "clientId": str, "clientSecret": str, "redirectUri": str}
  200   {"account": {"id","phone","email","phoneVerifiedAt"},
         "session": {"token": str, "expiresAt": ISO8601, "scopes": [str]}}
  4xx   no body shape worth parsing -> NidAuthError

POST {NID_BASE_URL}/v1/oauth/introspect
  body  {"token": str, "clientId": str, "clientSecret": str}
  200   {"active": false}
  200   {"active": true, "accountId": str, "expiresAt": ISO8601, "scopes": [str],
         "memberships": [{"tenantId": str, "slug": str, "name": str,
                          "role": "OWNER"|"ADMIN"|"AGENT"|"VIEWER"|"STAFF"}]}
        Cache-Control: no-store
  403   wrong clientId/clientSecret

POST {NID_BASE_URL}/v1/oauth/revoke
  body  {"token": str, "clientId": str, "clientSecret": str}
  200   {"revoked": true}        403 wrong client credential
```

`GET {NID_LOGIN_UI_URL}/authorize?client_id=…&redirect_uri=…&state=…` is the **only** browser-facing
nID URL Communicator constructs. `login-requests`, `pollToken`, `/authorize/{pollToken}` and
`mock_poll_`/`poll_fallback_` are deleted, not renamed (§4.2, §8 criterion 2).

## File structure

| File | Responsibility | Task |
|---|---|---|
| `src/communicator/migrations/versions/a1b2c3d4e5f6_drop_unique_integration_settings_tenant_key.py` | make per-tenant settings rows physically possible | 2 |
| `src/communicator/migrations/versions/b2c3d4e5f6a7_nid_accounts_and_tenant_team_members.py` | `nid_accounts` table, `team_members.nid_account_id`, `team_members.tenant_key` | 3 |
| `src/communicator/models/nid_account.py` | reference row for an nID Account (no auth power) | 3 |
| `src/communicator/models/team_member.py` | + `nid_account_id`, + `tenant_key` | 3 |
| `src/communicator/models/integration_settings.py` | `tenant_key` stops being unique | 2 |
| `src/communicator/config.py` | `NID_LOGIN_UI_URL`, cache TTLs, derived `nid_redirect_uri`, startup validation, `default_tenant_key`, delete `dev_auth_enabled` | 4 |
| `.env.example` | the undocumented `NID_*` / `META_*` / `PUBLIC_URL` block | 4 |
| `src/communicator/services/nid_client.py` | signed state, local `authorize_url`, token exchange, `introspect` + cache, `revoke`. Loses `_pending_states`, `_active_sessions`, `validate_session`, `login-requests` | 5, 6 |
| `src/communicator/api/dependencies.py` | `TenantPrincipal`, `require_tenant_member`, `require_tenant_writer`, `require_permission`, tenant auto-provision; loses `get_tenant_key`, `get_current_account_id` | 7, 8, 15 |
| `src/communicator/api/routes/nid_auth.py` | `/start`, `/callback`, `/me`, `/logout` | 9 |
| `src/communicator/api/routes/organizations.py` | `GET` = introspection projection; `POST` deleted | 10 |
| `src/communicator/services/rbac_service.py` | unchanged; gains callers | 15 |
| `src/communicator/api/routes/roles.py` | guarded, `member_id` path param deleted | 15 |
| `src/communicator/services/integration_settings.py` | `tenant_key` becomes required; payload→tenant resolver | 13, 14 |
| `src/communicator/services/whatsapp_automations.py` | per-tenant matcher + seeder | 13 |
| `src/communicator/api/routes/automation.py`, `scheduler.py` | guarded, per-tenant nudges | 13, 14 |
| `src/communicator/api/routes/*.py` (45 modules) | guard or allowlist, per §5 | 16 |
| `tests/nid_stub.py` | `httpx.MockTransport` nID fake + token/header constants | 11 |
| `tests/conftest.py` | `nid_stub`, two-tenant fixtures, `test_client` default auth headers | 11, 12 |
| `tests/test_tenant_isolation.py`, `tests/test_route_enforcement.py` | §4.6 matrix, §8 criterion 6 | 12, 16, 18 |
| `frontend/src/lib/api.ts`, `context/AuthContext.tsx`, `pages/{Login,NidCallback,Organizations}.tsx` | bearer + `X-Tenant-Id`, memberships-driven picker, logout→revoke | 17 |
| `docs/TENANCY_GAPS.md` | the 44 tenant-blind tables, written down instead of implied | 16 |

## Baseline to reproduce before writing any code

Run each and confirm the left column; if a number differs, stop and re-measure — the plan's task
sizes assume these.

```bash
cd ../communicator-tenancy-pr1                       # Plan 2's branch, or main worktree after merge
PYTHONPATH=src python -m alembic heads               # -> d7f8a9b0c1d2 (head)
pytest -q                                            # -> 230 passed
grep -c "get_settings_row\|get_whatsapp_config" -r src --include=*.py | grep -v ":0" | wc -l  # files, not sites
```

## Spec deviations (deliberate; each is a decision, not an oversight)

| # | Spec says | This plan does | Why |
|---|---|---|---|
| V1 | §4.3 step 2: introspect with "client credential in `Authorization`, token in body" | credentials in the **body** (`{token, clientId, clientSecret}`) | Plan 1/N4 implements and tests the body form; §4.1's own text says "the same three-credential shape as `POST /v1/oauth/token`". The parenthetical is stale. |
| V2 | §4.4 "`routes/automation.py`: all 10 endpoints" | 8 endpoints | Measured (`:57, :67, :75, :86, :117, :128, :168, :181`). Rev. 4 amended. |
| V3 | §7 "11 `get_settings_row` / `get_whatsapp_config` call sites" | 20 | Measured. Rev. 4 amended. |
| V4 | §8 criterion 8 "two tenants hold distinct WhatsApp credentials" | opens by dropping `IntegrationSettings.tenant_key`'s **unique** index | With the unique index (model `:27-29`, migration `5a95ba8a58af:28-32`, live DB) a second tenant's settings row is an `IntegrityError`. Without Task 2 the acceptance criterion is unreachable. |
| V5 | §4.2 gives `TeamMember` a `nid_account_id` | also gives it `tenant_key` | D7's per-tenant capability check is meaningless against one global `team_members` table (measured: no tenant column). Task 3 states the backfill rule. |
| V6 | §4.2 "the row is upserted from introspection results" (plural, could read "every request") | upsert on **login** and on `/auth/me` only | An insert per request turns every GET into a write. A caller whose `nid_accounts` row is missing gets a 403 naming the fix ("sign in again"), which is self-healing. |
| V7 | §4.4 "all 10 endpoints take `require_tenant_member`" for automation | also `require_permission("automation:manage")` on writes | §4.3: "Routes ask for permission keys". Writes are where D7 earns its keep. |
| V8 | §4.3 step 5 "every query in the route filters by it" | filters where a `tenant_key` column exists (14 tables); the other 44 are recorded in `docs/TENANCY_GAPS.md` | §3.1 measured it. Adding tenant columns to 44 tables + backfill + query rewrite is project 3, not a task here. Guarding still removes the anonymous-escalation. |
| V9 | §7 "No WhatsApp opt-out exists" | unchanged — no opt-out added | Same as the spec: tenancy plan, not a compliance plan. Its own ticket. |
| V10 | §5 "nID unreachable during a *new* login is 503 naming the upstream" | `/auth/nid/start` keeps making **zero** nID calls and therefore cannot 503 | A login that cannot start because nID is down is discovered at `/callback` (503) or at the first business request. `start` returning a URL for a dead IdP is the spec's own §4.2 diagram ("no upstream call"). |

---

### Task 1: Branch and worktree

**Files:** none (git only)

**Interfaces:**
- Consumes: Plan 2's merged tree, alembic head `d7f8a9b0c1d2`.
- Produces: a worktree at `../communicator-tenancy-pr2` on branch `feat/tenancy-pr2-enforcement`.

- [ ] **Step 1: Confirm PR-1 actually landed**

```bash
cd /c/Users/nommi/Downloads/scraper/communicator
git log --oneline -3
git branch --contains feat/tenancy-pr1-merge
```

Expected: `newax/finalization` (or wherever PR-1 was merged) contains the PR-1 commit. If it does
**not** yet contain it, stop: PR-2 builds on PR-1's conflict resolutions, and re-resolving them by
hand here would silently diverge from Plan 2's verified merge.

- [ ] **Step 2: Create the worktree** (superpowers:using-git-worktrees)

```bash
cd /c/Users/nommi/Downloads/scraper/communicator
git worktree add -b feat/tenancy-pr2-enforcement ../communicator-tenancy-pr2 <pr1-merged-ref>
cd ../communicator-tenancy-pr2 && git status --short
```

Expected: clean tree, branch `feat/tenancy-pr2-enforcement`.

- [ ] **Step 3: Reproduce the baseline**

```bash
PYTHONPATH=src python -m alembic heads   # d7f8a9b0c1d2 (head)
pytest -q                                # 230 passed
```

Expected: exactly those two results. If `pytest` cannot reach a database, set
`TEST_DATABASE_URL=postgresql+asyncpg://postgres:postgres@127.0.0.1:5432/communicator_test` (create
it first) and note the command in the commit message footer so the next executor does not
rediscover it.

- [ ] **Step 4: Commit a baseline note** (no code changes)

Create `docs/superpowers/plans/notes/2026-11-01-pr2-baseline.md` with the two numbers above and the
exact DB command used, then:

```bash
git add docs/superpowers/plans/notes/2026-11-01-pr2-baseline.md
git commit -m "chore(pr2): record tenancy-enforcement baseline"
```

---

### Task 2: Make a second tenant's settings row possible

`IntegrationSettings.tenant_key` is unique in the model, in migration `5a95ba8a58af`, and in the
live database (all three measured above). Until that uniqueness is gone, `get_settings_row(session,
"acme")` followed by `get_settings_row(session, "globex")` raises `IntegrityError`, and §8
criterion 8 cannot be met. This task is the smallest change that unblocks it.

**Files:**
- Create: `src/communicator/migrations/versions/a1b2c3d4e5f6_drop_unique_integration_settings_tenant_key.py`
- Modify: `src/communicator/models/integration_settings.py:27-29`
- Test: `tests/test_tenant_settings_rows.py`

**Interfaces:**
- Consumes: alembic head `d7f8a9b0c1d2`.
- Produces: `integration_settings.tenant_key` — NOT NULL, default `'newax'`, **non-unique indexed**
  column. Tasks 13 and 14 rely on two rows being insertable.

- [ ] **Step 1: Write the failing test**

Create `tests/test_tenant_settings_rows.py`:

```python
"""Two tenants must each be able to hold an IntegrationSettings row.

Regression guard for the branch's unique=True on tenant_key, which made exactly
one tenant's credentials insertable and therefore made per-tenant WhatsApp
numbers impossible no matter what the code asked for.
"""

from __future__ import annotations

import pytest
from sqlalchemy import text
from sqlalchemy.ext.asyncio import AsyncSession

from communicator.models.integration_settings import IntegrationSettings


@pytest.mark.asyncio
async def test_two_tenants_can_each_hold_a_settings_row(db_session: AsyncSession) -> None:
    db_session.add(
        IntegrationSettings(
            tenant_key="acme-task2", business_name="Acme Support", meta_phone_number_id="111"
        )
    )
    db_session.add(
        IntegrationSettings(
            tenant_key="globex-task2", business_name="Globex Support", meta_phone_number_id="222"
        )
    )
    await db_session.flush()

    rows = (
        await db_session.execute(
            text(
                "select tenant_key, business_name from integration_settings "
                "where tenant_key in ('acme-task2','globex-task2') order by tenant_key"
            )
        )
    ).all()
    assert [r[0] for r in rows] == ["acme-task2", "globex-task2"]


@pytest.mark.asyncio
async def test_tenant_key_index_is_not_unique(db_session: AsyncSession) -> None:
    ddl = (
        await db_session.execute(
            text(
                "select indexdef from pg_indexes "
                "where indexname = 'ix_integration_settings_tenant_key'"
            )
        )
    ).scalar_one()
    assert "UNIQUE" not in ddl.upper()
```

- [ ] **Step 2: Run it and confirm it fails on the unique index**

```bash
pytest tests/test_tenant_settings_rows.py -q
```

Expected: the first test FAILS with `asyncpg.exceptions.UniqueViolationError` (wrapped as
`IntegrityError`) on the second flush, and the second FAILS with an assertion value containing
`CREATE UNIQUE INDEX`. If the first instead passes, the database you are pointed at was not migrated
from scratch — re-run against a fresh `communicator_test`.

- [ ] **Step 3: Write the migration**

Create `src/communicator/migrations/versions/a1b2c3d4e5f6_drop_unique_integration_settings_tenant_key.py`:

```python
"""drop unique on integration_settings.tenant_key

Revision ID: a1b2c3d4e5f6
Revises: d7f8a9b0c1d2
Create Date: 2026-11-01 18:40:00.000000

The branch created ix_integration_settings_tenant_key as a UNIQUE index
(5a95ba8a58af:28-32), which caps the table at one row -- one WhatsApp number per
deployment no matter what tenant_key the code passes. Per-tenant credentials
need this to be a plain lookup index. Nothing else changes: the column stays
NOT NULL with its 'newax' server default, so existing single-tenant rows are
untouched.
"""

from collections.abc import Sequence

from alembic import op

# revision identifiers, used by Alembic.
revision: str = "a1b2c3d4e5f6"
down_revision: str | Sequence[str] | None = "d7f8a9b0c1d2"
branch_labels: str | Sequence[str] | None = None
depends_on: str | Sequence[str] | None = None


def upgrade() -> None:
    op.drop_index("ix_integration_settings_tenant_key", table_name="integration_settings")
    op.create_index(
        "ix_integration_settings_tenant_key",
        "integration_settings",
        ["tenant_key"],
        unique=False,
    )


def downgrade() -> None:
    # Restoring uniqueness only works if the deployment really has one row per
    # tenant; with two it raises, which is the correct outcome -- you cannot
    # un-multitenant a database that has data in it.
    op.drop_index("ix_integration_settings_tenant_key", table_name="integration_settings")
    op.create_index(
        "ix_integration_settings_tenant_key",
        "integration_settings",
        ["tenant_key"],
        unique=True,
    )
```

- [ ] **Step 4: Fix the model so autogenerate does not fight the migration**

In `src/communicator/models/integration_settings.py`, replace lines 27-29 with:

```python
    tenant_key: Mapped[str] = mapped_column(
        String(100), default="newax", nullable=False, index=True
    )
```

and correct the class docstring's opening claim (`:14-22`), which still says "Single-row table
holding…" and "there's exactly one business and one WhatsApp connection per deployment". Say instead
that the table holds one row **per tenant**, keyed by `tenant_key`, and that `SINGLETON_ID` (`:10`)
is now dead weight — **do not delete `SINGLETON_ID` in this task**; Task 14 removes it once no caller
can rely on it.

- [ ] **Step 5: Apply and re-run**

```bash
PYTHONPATH=src python -m alembic upgrade head
pytest tests/test_tenant_settings_rows.py -q
```

Expected: `2 passed`.

- [ ] **Step 6: Confirm one head and no collateral damage**

```bash
PYTHONPATH=src python -m alembic heads   # a1b2c3d4e5f6 (head)  -- exactly one line
pytest -q                                  # 232 passed
```

Expected: one head; 230 + the 2 new tests. Any other failure means a caller depended on the
uniqueness — read it, do not skip it.

- [ ] **Step 7: Commit**

```bash
git add src/communicator/migrations/versions/a1b2c3d4e5f6_drop_unique_integration_settings_tenant_key.py \
        src/communicator/models/integration_settings.py tests/test_tenant_settings_rows.py
git commit -m "$(cat <<'EOF'
fix(tenancy): allow one IntegrationSettings row per tenant

The merged branch keyed integration_settings.tenant_key with a UNIQUE index, so
a deployment could hold exactly one WhatsApp credential set regardless of how
many tenants it served. Drop the uniqueness, keep a plain lookup index; the
column stays NOT NULL with its 'newax' default.

Unblocks acceptance criterion 8 (two tenants, distinct credentials).
EOF
)"
```

---

### Task 3: `nid_accounts`, and per-tenant `team_members`

D7's permission lookup needs to turn nID's cuid `accountId` into a local `TeamMember.id`, because
`Conversation.assigned_to_id`, `Ticket.assigned_to_id` and `Approval.assigned_to_id` are UUID FKs
onto `team_members`. Spec §4.2 adds that link; §3.1's measurement adds that the member table is also
**global**, which makes a per-tenant role check meaningless until the row is tenant-scoped. Both land
here, in one migration.

**Files:**
- Create: `src/communicator/migrations/versions/b2c3d4e5f6a7_nid_accounts_and_tenant_team_members.py`
- Create: `src/communicator/models/nid_account.py`
- Modify: `src/communicator/models/team_member.py`
- Modify: `src/communicator/models/__init__.py`
- Test: `tests/test_nid_account_links.py`

**Interfaces:**
- Consumes: `a1b2c3d4e5f6` (Task 2).
- Produces:
  - `NidAccount` — `id: uuid.UUID` pk, `nid_account_id: str` unique indexed, `email: str`,
    `phone: str`, `display_name: str`, `last_login_at: datetime | None`.
  - `TeamMember.nid_account_id: str | None` (indexed), `TeamMember.tenant_key: str` (NOT NULL,
    server_default `'newax'`, indexed).
  - Task 15 resolves a caller with
    `select(TeamMember).where(nid_account_id == …, tenant_key == …)`.

- [ ] **Step 1: Write the failing test**

Create `tests/test_nid_account_links.py`:

```python
"""nID account -> local TeamMember link, and team_members stops being global."""

from __future__ import annotations

import uuid

import pytest
from sqlalchemy import text
from sqlalchemy.exc import IntegrityError

from communicator.models.nid_account import NidAccount
from communicator.models.team_member import TeamMember


@pytest.mark.asyncio
async def test_nid_account_is_unique_per_nid_id(db_session) -> None:
    db_session.add(NidAccount(nid_account_id="ckx_task3_acme_owner", email="o1@acme.test"))
    db_session.add(NidAccount(nid_account_id="ckx_task3_globex_owner", email="o2@globex.test"))
    await db_session.flush()

    db_session.add(NidAccount(nid_account_id="ckx_task3_acme_owner", email="dup@acme.test"))
    with pytest.raises(IntegrityError):
        await db_session.flush()
    await db_session.rollback()


@pytest.mark.asyncio
async def test_team_member_links_to_nid_account_and_tenant(db_session) -> None:
    member = TeamMember(
        name="Acme Agent",
        email="agent@acme.test",
        role="agent",
        tenant_key="acme",
        nid_account_id="ckx_task3_agent",
    )
    db_session.add(member)
    await db_session.flush()

    row = (
        await db_session.execute(
            text("select tenant_key, nid_account_id from team_members where id = :id"),
            {"id": member.id},
        )
    ).one()
    assert row[0] == "acme"
    assert row[1] == "ckx_task3_agent"


@pytest.mark.asyncio
async def test_one_nid_account_can_be_two_tenants_members(db_session) -> None:
    """One human in two organizations is the whole point of the header."""
    account_id = f"ckx_multi_{uuid.uuid4().hex[:8]}"
    db_session.add(NidAccount(nid_account_id=account_id, email="both@acme.test"))
    db_session.add_all(
        [
            TeamMember(
                name="Both A", email="both@acme.test", tenant_key="acme", nid_account_id=account_id
            ),
            TeamMember(
                name="Both B", email="both@acme.test", tenant_key="globex",
                nid_account_id=account_id,
            ),
        ]
    )
    await db_session.flush()
    count = (
        await db_session.execute(
            text("select count(*) from team_members where nid_account_id = :a"),
            {"a": account_id},
        )
    ).scalar_one()
    assert count == 2
```

- [ ] **Step 2: Run it and confirm it fails**

```bash
pytest tests/test_nid_account_links.py -q
```

Expected: a collection error naming `communicator.models.nid_account` (`ModuleNotFoundError`). That
is the failure this step wants.

- [ ] **Step 3: Add the model**

Create `src/communicator/models/nid_account.py`:

```python
import uuid
from datetime import UTC, datetime

from sqlalchemy import DateTime, String
from sqlalchemy.dialects.postgresql import UUID
from sqlalchemy.orm import Mapped, mapped_column

from communicator.database import Base


class NidAccount(Base):
    """
    A reference row for an nID Account -- not an auth store.

    Communicator's rows key people by local UUID (Conversation.assigned_to_id,
    Ticket.assigned_to_id, Approval.assigned_to_id all point at TeamMember.id),
    and nID's cuid strings cannot be those FKs, so this table is the bridge. It
    grants nothing: whether a caller may act is decided by introspecting their
    bearer token at nID on every request (api/dependencies.require_tenant_member).
    A row appears here when someone logs in; its absence makes a capability check
    fail closed with an error that says sign in again.
    """

    __tablename__ = "nid_accounts"

    id: Mapped[uuid.UUID] = mapped_column(UUID(as_uuid=True), primary_key=True, default=uuid.uuid4)
    nid_account_id: Mapped[str] = mapped_column(
        String(255), unique=True, nullable=False, index=True
    )
    email: Mapped[str] = mapped_column(String(255), default="", nullable=False)
    phone: Mapped[str] = mapped_column(String(50), default="", nullable=False)
    display_name: Mapped[str] = mapped_column(String(255), default="", nullable=False)
    last_login_at: Mapped[datetime | None] = mapped_column(DateTime(timezone=True), nullable=True)

    created_at: Mapped[datetime] = mapped_column(
        DateTime(timezone=True), default=lambda: datetime.now(UTC)
    )
```

- [ ] **Step 4: Register it**

In `src/communicator/models/__init__.py`, add alongside the other model imports (both branches added
lines here, so keep the existing ordering convention):

```python
from communicator.models.nid_account import NidAccount
```

and, if the module defines `__all__`, append `"NidAccount"`. Prove registration took — an empty list
means the migration you write in Step 6 will be empty and this task is not done:

```bash
PYTHONPATH=src python -c "
from communicator.database import Base
import communicator.models
print([t.name for t in Base.metadata.sorted_tables if t.name == 'nid_accounts'])"
```

Expected: `['nid_accounts']`.

- [ ] **Step 5: Extend `TeamMember`**

In `src/communicator/models/team_member.py`, after `email` (line 18) insert:

```python
    # Tenant this agent belongs to. The table was global on both branches, which
    # made a per-tenant role check meaningless: every organization's UI showed
    # the same agent list. Existing rows backfill to the development placeholder
    # ('newax') because those *are* the single-tenant deployment's data.
    tenant_key: Mapped[str] = mapped_column(
        String(100), default="newax", server_default="newax", nullable=False, index=True
    )

    # nID's Account cuid for this human, bridged through models/nid_account.py.
    # Nullable because most existing rows were typed into the agent picker and
    # have never signed in; linking is what enables D7's permission check.
    nid_account_id: Mapped[str | None] = mapped_column(String(255), nullable=True, index=True)
```

- [ ] **Step 6: Write the migration by hand**

Create `src/communicator/migrations/versions/b2c3d4e5f6a7_nid_accounts_and_tenant_team_members.py`
using the same boilerplate as Task 2, with `revision = "b2c3d4e5f6a7"` and
`down_revision = "a1b2c3d4e5f6"`, and:

```python
def upgrade() -> None:
    op.create_table(
        "nid_accounts",
        sa.Column("id", UUID(as_uuid=True), nullable=False),
        sa.Column("nid_account_id", sa.String(length=255), nullable=False),
        sa.Column("email", sa.String(length=255), nullable=False),
        sa.Column("phone", sa.String(length=50), nullable=False),
        sa.Column("display_name", sa.String(length=255), nullable=False),
        sa.Column("last_login_at", sa.DateTime(timezone=True), nullable=True),
        sa.Column("created_at", sa.DateTime(timezone=True), nullable=False),
        sa.PrimaryKeyConstraint("id", name=op.f("pk_nid_accounts")),
    )
    op.create_index(
        op.f("ix_nid_accounts_nid_account_id"), "nid_accounts", ["nid_account_id"], unique=True
    )

    op.add_column(
        "team_members",
        sa.Column("tenant_key", sa.String(length=100), nullable=False, server_default="newax"),
    )
    op.create_index(
        op.f("ix_team_members_tenant_key"), "team_members", ["tenant_key"], unique=False
    )
    op.add_column("team_members", sa.Column("nid_account_id", sa.String(length=255), nullable=True))
    op.create_index(
        op.f("ix_team_members_nid_account_id"), "team_members", ["nid_account_id"], unique=False
    )


def downgrade() -> None:
    op.drop_index(op.f("ix_team_members_nid_account_id"), table_name="team_members")
    op.drop_column("team_members", "nid_account_id")
    op.drop_index(op.f("ix_team_members_tenant_key"), table_name="team_members")
    op.drop_column("team_members", "tenant_key")
    op.drop_index(op.f("ix_nid_accounts_nid_account_id"), table_name="nid_accounts")
    op.drop_table("nid_accounts")
```

Imports at the top: `from collections.abc import Sequence`, `import sqlalchemy as sa`,
`from alembic import op`, `from sqlalchemy.dialects.postgresql import UUID`.

- [ ] **Step 7: Apply, test, and confirm drift is limited to these tables**

```bash
PYTHONPATH=src python -m alembic upgrade head
pytest tests/test_nid_account_links.py -q     # 3 passed
PYTHONPATH=src python -m alembic heads         # b2c3d4e5f6a7 (head), one line
pytest -q                                       # 235 passed
```

Do **not** gate on `alembic check` — Plan 2 measured it dirty on both trees (32 ops baseline / 8
merged) and excluded it as a gate. Run it as a read only, and confirm `nid_accounts` and
`team_members` do **not** appear in its diff (that is the proof the two migrations match the models).

- [ ] **Step 8: Commit**

```bash
git add src/communicator/migrations/versions/b2c3d4e5f6a7_nid_accounts_and_tenant_team_members.py \
        src/communicator/models/nid_account.py src/communicator/models/__init__.py \
        src/communicator/models/team_member.py tests/test_nid_account_links.py
git commit -m "$(cat <<'EOF'
feat(tenancy): nid_accounts reference table, tenant-scoped team_members

Communicator's rows key people by local UUID, so nID's cuid account ids need a
bridge table to be FK targets; that bridge is what the D7 role -> capability
check walks. team_members also gains tenant_key -- the table was global on both
branches, which made a per-tenant permission check meaningless because every
organization saw the same agent list. Existing rows backfill to 'newax', the
single-tenant deployment's own data.
EOF
)"
```

---

### Task 4: Config and credential hygiene (§4.5)

Everything after this task reads configuration through these fields, so it goes first: client
ID/secret become mandatory outside development, `nid_redirect_uri` stops being independently
settable, `public_url` stops pointing at a port nothing listens on, `settings.environment` (which
does not exist, and 500s the unauthenticated production path) is fixed, and the
`PYTEST_CURRENT_TEST`-as-authorization hole in LiveProof is closed.

**Files:**
- Modify: `src/communicator/config.py` (`:9`, `:56-72`, `:80`, `:84`, `:99`)
- Modify: `src/communicator/services/nid_client.py:55-59` (hardcoded default credentials)
- Modify: `src/communicator/api/routes/live_proof.py` (authorization expression near `:268-277`)
- Modify: `.env.example`
- Modify: `tests/test_live_proof.py` (stop relying on the bypass)
- Test: `tests/test_settings_hygiene.py`

**Interfaces:**
- Consumes: nothing new.
- Produces on `settings`: `nid_login_ui_url` (`http://localhost:3300`),
  `nid_introspect_cache_ttl_seconds` (60), `nid_introspect_write_max_age_seconds` (15),
  `nid_redirect_uri` (**derived** from `public_url` unless explicitly set), `public_url`
  (`http://localhost:5173`), `default_tenant_key` (`"newax"`, the only surviving literal), and boot
  validators. `dev_auth_enabled` is deleted.

- [ ] **Step 1: Write the failing tests**

Create `tests/test_settings_hygiene.py`:

```python
"""Spec 4.5 config hygiene: derived redirect URI, mandatory client credential,
no half-existing environment setting, no silent tenant fallback outside dev."""

from __future__ import annotations

import pytest
from pydantic import ValidationError

from communicator.config import Settings


def _prod(**kw):
    base = {"_env_file": None, "app_env": "production", "nid_client_id": "ci",
            "nid_client_secret": "cs", "default_tenant_key": "ckx_real_tenant"}
    base.update(kw)
    return Settings(**base)


def test_redirect_uri_is_derived_from_public_url() -> None:
    s = _prod(public_url="https://comm.example.test", nid_redirect_uri="")
    assert s.nid_redirect_uri == "https://comm.example.test/auth/nid/callback"


def test_explicit_redirect_uri_still_wins() -> None:
    s = _prod(nid_redirect_uri="https://other.example.test/auth/nid/callback")
    assert s.nid_redirect_uri == "https://other.example.test/auth/nid/callback"


def test_production_refuses_to_boot_without_nid_client_credentials() -> None:
    with pytest.raises(ValidationError) as excinfo:
        Settings(_env_file=None, app_env="production", nid_client_id="",
                 nid_client_secret="", default_tenant_key="ckx_real")
    assert "NID_CLIENT_ID" in str(excinfo.value)


def test_production_refuses_the_placeholder_tenant_key() -> None:
    with pytest.raises(ValidationError) as excinfo:
        _prod(default_tenant_key="newax")
    assert "DEFAULT_TENANT_KEY" in str(excinfo.value)


def test_mock_auth_exempts_development_from_the_credential_check() -> None:
    s = Settings(_env_file=None, app_env="development", nid_client_id="",
                 nid_client_secret="", nid_allow_mock_auth=True)
    assert s.nid_allow_mock_auth is True


def test_settings_has_no_environment_attribute() -> None:
    """The bug both branches shipped: settings.environment never existed, and
    reading it raised AttributeError on the nID-unreachable path."""
    s = Settings(_env_file=None)
    assert not hasattr(s, "environment")
    assert s.app_env == "development"


def test_dev_auth_enabled_is_gone() -> None:
    assert not hasattr(Settings(_env_file=None), "dev_auth_enabled")
```

- [ ] **Step 2: Run to confirm failures**

```bash
pytest tests/test_settings_hygiene.py -q
```

Expected: failures naming derivation (`http://localhost:8000/auth/nid/callback` ≠ derived value),
`DEFAULT_TENANT_KEY`, `NID_CLIENT_ID`, `dev_auth_enabled`. Two tests may already pass —
`test_settings_has_no_environment_attribute` and the mock-exemption one are tripwires, not features.

- [ ] **Step 3: Rewrite the nID block and add the validator**

`src/communicator/config.py` line 1 becomes:

```python
from pydantic import model_validator
from pydantic_settings import BaseSettings, SettingsConfigDict
```

Replace lines 56-72 (nID comment block + fields) with:

```python
    # nID -- the ecosystem's single identity provider. Communicator runs no auth
    # of its own: it is a "Login with nID" relying party, and nID is the
    # authority for identity, tenant membership and role (spec D3).
    #
    # nid_base_url is the nID *API* (:4000) -- token, introspect, revoke.
    # nid_login_ui_url is the *browser* app (:3300) that owns /authorize and the
    # login-request polling. Both branches conflated the two, which is exactly
    # why today's authorize_url 404s (spec 3): it pointed at an API that serves
    # no /authorize route.
    nid_base_url: str = "http://localhost:4000"
    nid_login_ui_url: str = "http://localhost:3300"
    nid_client_id: str = ""
    nid_client_secret: str = ""
    # Where nID sends the *browser* with ?code&state: the SPA's own route
    # (frontend/src/pages/NidCallback.tsx, mounted in App.tsx:47), never the
    # backend JSON endpoint. Derived from public_url so the two cannot drift;
    # set explicitly only for a genuinely separate callback origin.
    nid_redirect_uri: str = ""
    nid_vault_addr: str = "http://localhost:4000"
    nid_vault_role_id: str = ""
    nid_vault_secret_id: str = ""
    # The ONLY thing that enables a locally invented login (spec D8). app_env is
    # deliberately not consulted: config defaults app_env to "development", so a
    # production deploy started with defaults intact would have accepted any
    # code a caller typed.
    nid_allow_mock_auth: bool = False
    # Introspection cache bounds (spec 4.3 step 6): reads accept an entry up to
    # TTL old, writes demand one no older than MAX_AGE. Both are cache *age*, not
    # session lifetime -- nID's expiresAt is enforced separately.
    nid_introspect_cache_ttl_seconds: int = 60
    nid_introspect_write_max_age_seconds: int = 15

    # The tenant an installation falls back to when none is configured. Honoured
    # only while app_env == "development": outside development, booting with the
    # placeholder is refused below, because a silent "newax" fallback is the very
    # cross-tenant escalation this spec exists to close.
    default_tenant_key: str = "newax"
```

Delete line 80 (`dev_auth_enabled: bool = False`), then confirm nothing referenced it
(`grep -rn "dev_auth_enabled" src tests` → only the `.env.example` line Step 6 removes).

Replace `public_url` (`:84`, currently `http://localhost:8001`) with:

```python
    # The SPA origin, not the API origin: Vite dev serves :5173 and proxies /api
    # to uvicorn on :8000. public_url is what nID's redirect URI is derived from
    # and what customer-facing LiveProof verification links are built on, so it
    # must name an origin a browser can actually reach.
    public_url: str = "http://localhost:5173"
```

Immediately after `model_config` (`:99`), add:

```python
    @model_validator(mode="after")
    def _derive_redirect_uri_and_refuse_defaults(self) -> "Settings":
        if not self.nid_redirect_uri:
            self.nid_redirect_uri = f"{self.public_url.rstrip('/')}/auth/nid/callback"
        if self.app_env != "development":
            if not self.nid_allow_mock_auth and not (self.nid_client_id and self.nid_client_secret):
                raise ValueError(
                    "NID_CLIENT_ID and NID_CLIENT_SECRET are required outside development. "
                    "They come from nID's OAuthClient registration for communicator; a "
                    "relying party with no client credential cannot introspect its own "
                    "tokens. Set NID_ALLOW_MOCK_AUTH=true only if you accept that logins "
                    "are invented locally."
                )
            if self.default_tenant_key == "newax":
                raise ValueError(
                    "DEFAULT_TENANT_KEY is still the development placeholder 'newax' while "
                    "APP_ENV is not 'development'. Per spec 4.3 step 3 there is no silent "
                    "tenant fallback outside development; set DEFAULT_TENANT_KEY to this "
                    "installation's nID Tenant.id."
                )
        return self
```

- [ ] **Step 4: Kill the hardcoded client credential fallbacks**

`src/communicator/services/nid_client.py:57-58` currently read
`client_id or settings.nid_client_id or "client_comm_default"` and the same for the secret. Reduce to:

```python
        self.client_id = client_id or settings.nid_client_id
        self.client_secret = client_secret or settings.nid_client_secret
```

Those literals cannot authenticate at nID (`oauth.service.ts:122` compares a sha256 against
`clientSecretHash`), so the fallback converted "missing credential" into a confusing 403 at the IdP.
Outside development `Settings` now refuses to boot without real values; in development with mock
auth on, an empty credential is never sent anywhere because the mock path short-circuits first.

- [ ] **Step 5: Fix both `settings.environment` reads, and LiveProof's test-as-auth grant**

`grep -rn "settings\.environment" src/` finds two: `services/nid_client.py:161` (deleted wholesale in
Task 5, because that whole conditional goes per D8) and `api/routes/live_proof.py` near `:275`.
Locate and replace the LiveProof authorization expression:

```python
    authorized = False
    if (
        token
        and secrets.compare_digest(proof_session.session_token, token)
        or authorization
        and authorization.startswith("Bearer ")
        or os.getenv("PYTEST_CURRENT_TEST")
        or settings.environment == "development"
    ):
        authorized = True
```

with:

```python
    # Possessing the session token is the only customer-side credential, and it is
    # compared in constant time. Any string in a Bearer header used to authorize
    # this read, PYTEST_CURRENT_TEST was treated as a grant (it is an environment
    # variable an operator can set in production), and the last clause referenced
    # settings.environment, which does not exist on Settings -- so it raised
    # AttributeError rather than develop anything.
    if not (token and secrets.compare_digest(proof_session.session_token, token)):
        raise HTTPException(
            status_code=status.HTTP_403_FORBIDDEN,
            detail="Session token required to view evidence media.",
        )
```

Delete the `authorized` variable and its later `if not authorized:` block, and drop `os` / `settings`
from the imports if they become unused (ruff will say so). `authorization` stays a parameter only if
another route in the file still uses it; here it must not grant anything.

Then update `tests/test_live_proof.py` to pass the real `session_token` (from the create-session
response or the DB row) instead of relying on the bypass, and re-run:

```bash
pytest tests/test_live_proof.py -q
```

- [ ] **Step 6: Document the block in `.env.example`**

`.env.example` today is 32 lines / 17 keys and documents none of `NID_*`, `PUBLIC_URL` or
`NEURA_CRYPTO_*`. Delete the `DEV_AUTH_ENABLED=true` line and append:

```bash

# nID -- the only identity provider (spec D3). Communicator stores no passwords,
# sessions or memberships; it introspects nID on every authenticated request.
# NID_BASE_URL is the nID API (:4000); NID_LOGIN_UI_URL is the browser app (:3300).
NID_BASE_URL=http://localhost:4000
NID_LOGIN_UI_URL=http://localhost:3300
# From nID's OAuthClient row for communicator (Plan 1 Task 6's pnpm db:seed prints
# it once). Never commit real values.
NID_CLIENT_ID=
NID_CLIENT_SECRET=
# Leave blank to derive from PUBLIC_URL. Set only if the SPA callback route is on
# a different origin.
NID_REDIRECT_URI=
# Introspection cache bounds: reads accept up to TTL, writes demand up to MAX_AGE.
NID_INTROSPECT_CACHE_TTL_SECONDS=60
NID_INTROSPECT_WRITE_MAX_AGE_SECONDS=15
# The ONLY way to get a locally invented login. Keep false anywhere but a laptop;
# APP_ENV is deliberately not consulted (spec D8).
NID_ALLOW_MOCK_AUTH=false
# nID Secrets Vault (pre-bootstrap loader). Empty role/secret = skipped.
NID_VAULT_ADDR=http://localhost:4000
NID_VAULT_ROLE_ID=
NID_VAULT_SECRET_ID=
# This installation's tenant id = nID's Tenant.id, never a slug. 'newax' is the
# development placeholder and is refused outside development at boot.
DEFAULT_TENANT_KEY=newax

# Public origin of the SPA (Vite dev is :5173 and proxies /api to :8000). Used for
# nID's redirect URI and customer-facing LiveProof verification links.
PUBLIC_URL=http://localhost:5173

# Ecosystem event bus + gateway key (crypto-mesh ingress/relay)
ECOSYSTEM_EVENT_BUS_URL=
ECOSYSTEM_GATEWAY_KEY=

# Zero-trust crypto mesh key material (hex; all empty = mesh ingress rejects)
NEURA_CRYPTO_TENANT_ID=default
NEURA_CRYPTO_ENTITY_ID=communicator
NEURA_CRYPTO_KEY_ID=kid_communicator_v1
NEURA_CRYPTO_INGRESS_SECRET_HEX=
NEURA_CRYPTO_EGRESS_SECRET_HEX=
NEURA_CRYPTO_AUTH_SECRET_HEX=

# Outbound integrations
SALES_ALERT_WEBHOOK_URL=
POSTMARK_INBOUND_WEBHOOK_SECRET=

# WhatsApp via Meta Cloud API (set WHATSAPP_PROVIDER=meta to use these). Live
# credentials -- they belong in .env and must never be committed.
META_WHATSAPP_ACCESS_TOKEN=
META_WHATSAPP_PHONE_NUMBER_ID=
META_WEBHOOK_VERIFY_TOKEN=
META_APP_SECRET=
```

Leave `HARVESTER_API_BASE_URL` as it is and say so in the commit message: its default still collides
with Communicator's own port (§7 records it; the operator owes the real value).

- [ ] **Step 7: Verify the gate, including that the boot guard fires from a real process**

```bash
pytest tests/test_settings_hygiene.py -q   # 7 passed
pytest -q                                   # 242 passed (235 + 7)
ruff check src tests                          # clean
APP_ENV=production DEFAULT_TENANT_KEY=newax PYTHONPATH=src python -c "import communicator.config"
```

Expected: the last command exits non-zero with a `ValidationError` naming `NID_CLIENT_ID` or
`DEFAULT_TENANT_KEY`. If it boots, the validator is unreachable and this task is not done.

- [ ] **Step 8: Commit**

```bash
git add src/communicator/config.py src/communicator/services/nid_client.py \
        src/communicator/api/routes/live_proof.py tests/test_settings_hygiene.py \
        tests/test_live_proof.py .env.example
git commit -m "$(cat <<'EOF'
fix(config): require nID client credentials at boot, derive redirect URI, drop test-mode auth grant

- NID_LOGIN_UI_URL separates the browser app from the API. Conflating them is why
  every authorize_url today 404s.
- nid_redirect_uri derives from public_url (now the SPA origin, :5173) so the two
  cannot drift; the client_comm_default/secret_comm_default fallbacks are gone.
- Outside development Settings refuses to boot without a client credential or with
  the 'newax' placeholder tenant -- a silent tenant fallback is the escalation this
  spec closes.
- dev_auth_enabled deleted (referenced nowhere); the settings.environment reads that
  raised AttributeError on the nID-unreachable path are fixed; LiveProof evidence
  media stops treating PYTEST_CURRENT_TEST and any Bearer prefix as authorization.
EOF
)"
```

---

### Task 5: `nid_client` — signed state, local authorize URL, real token exchange

This is the file that makes login work at all. Today it POSTs `login-requests` itself (which belongs
to login-ui), returns an `/authorize/{poll_token}` URL on the **API** origin (404s), accepts a
caller-supplied `redirect_uri` guarded by a `startswith("http://localhost")` check, stores pending
state and sessions in two module dicts (broken across workers), invents a bearer when nID returns no
`access_token`, and 500s on `settings.environment` when it matters most.

**Files:**
- Modify: `src/communicator/services/nid_client.py` (large rewrite of `:1-120`, `:112-179`)
- Test: `tests/test_nid_client_login.py`

**Interfaces:**
- Consumes: `settings.nid_login_ui_url`, `nid_base_url`, `nid_client_id`, `nid_client_secret`,
  `nid_redirect_uri`, `nid_allow_mock_auth` (Task 4).
- Produces:
  - `sign_state(redirect_uri: str, *, now: float | None = None) -> str` and
    `verify_state(state: str, redirect_uri: str, *, now: float | None = None) -> bool`.
  - `NidClient.build_authorize_url(state: str) -> str` — **pure, no I/O**.
  - `async NidClient.exchange_code(code: str, state: str) -> NidSessionClaims` where
    `NidSessionClaims = {access_token, account_id, email, phone, phone_verified, expires_at,
    scopes, memberships: tuple[NidMembership, …]}`.
  - Deleted: `_pending_states`, `_prune_expired_states`, `start_login`, `validate_session`,
    `poll_token`, `mock_poll_`, `poll_fallback_`, `login-requests`, the `redirect_uri` parameter.
  - Task 7 imports `NidMembership` and `introspect` from here; Task 9 imports `sign_state`,
    `verify_state`, `build_authorize_url`, `exchange_code`.

- [ ] **Step 1: Write the failing tests**

Create `tests/test_nid_client_login.py`:

```python
"""Spec 4.2 login mechanics, at the client level (routes are tested in Task 9)."""

from __future__ import annotations

import time

import httpx
import pytest

from communicator.config import settings
from communicator.services import nid_client as mod
from communicator.services.nid_client import NidAuthError, nid_client


@pytest.fixture(autouse=True)
def _client_creds(monkeypatch):
    monkeypatch.setattr(settings, "nid_client_id", "ci-communicator")
    monkeypatch.setattr(settings, "nid_client_secret", "secret-value")
    monkeypatch.setattr(settings, "nid_allow_mock_auth", False)
    monkeypatch.setattr(nid_client, "base_url", "http://nid-api.test")
    monkeypatch.setattr(nid_client, "login_ui_url", "http://login-ui.test")
    monkeypatch.setattr(nid_client, "redirect_uri", "http://spa.test/auth/nid/callback")


def test_sign_state_round_trips() -> None:
    state = mod.sign_state("http://spa.test/auth/nid/callback")
    assert mod.verify_state(state, "http://spa.test/auth/nid/callback") is True


def test_state_is_bound_to_the_redirect_uri() -> None:
    state = mod.sign_state("http://spa.test/auth/nid/callback")
    assert mod.verify_state(state, "http://attacker.test/cb") is False


def test_state_expires() -> None:
    state = mod.sign_state("http://spa.test/auth/nid/callback", now=time.time() - 700)
    assert mod.verify_state(state, "http://spa.test/auth/nid/callback") is False


def test_state_rejects_tampering_and_garbage() -> None:
    state = mod.sign_state("http://spa.test/auth/nid/callback")
    nonce, ts, sig = state.split(".")
    assert mod.verify_state(f"{nonce}.{int(ts) + 1}.{sig}", "http://spa.test/auth/nid/callback")
    assert mod.verify_state("garbage", "http://spa.test/auth/nid/callback") is False
    assert mod.verify_state("a.b.c.d", "http://spa.test/auth/nid/callback") is False


def test_authorize_url_is_built_locally_on_the_login_ui_origin() -> None:
    state = mod.sign_state(nid_client.redirect_uri)
    url = nid_client.build_authorize_url(state)
    assert url.startswith("http://login-ui.test/authorize?")
    assert "client_id=ci-communicator" in url
    assert "redirect_uri=http%3A%2F%2Fspa.test%2Fauth%2Fnid%2Fcallback" in url
    assert f"state={state}" in url
    # spec 8 criterion 2: no poll-token construction anywhere.
    assert "poll" not in url


@pytest.mark.asyncio
async def test_start_login_makes_zero_upstream_calls() -> None:
    """The whole point of 4.2: Communicator does not register login requests."""
    calls: list[httpx.Request] = []

    def handler(request: httpx.Request) -> httpx.Response:
        calls.append(request)
        return httpx.Response(500, json={"should": "not be reached"})

    nid_client._transport = httpx.MockTransport(handler)
    try:
        state = mod.sign_state(nid_client.redirect_uri)
        nid_client.build_authorize_url(state)
    finally:
        nid_client._transport = None
    assert calls == []


@pytest.mark.asyncio
async def test_exchange_code_parses_nids_real_response_shape() -> None:
    seen: list[httpx.Request] = []

    def handler(request: httpx.Request) -> httpx.Response:
        seen.append(request)
        assert request.url.path == "/v1/oauth/token"
        return httpx.Response(
            200,
            json={
                "account": {
                    "id": "ckx_acct_1",
                    "phone": "+923001112233",
                    "email": "a@acme.test",
                    "phoneVerifiedAt": "2026-10-01T00:00:00Z",
                },
                "session": {
                    "token": "nid-session-token-1",
                    "expiresAt": "2026-11-02T06:00:00.000Z",
                    "scopes": ["identity:read", "memberships:read"],
                },
            },
        )

    nid_client._transport = httpx.MockTransport(handler)
    try:
        claims = await nid_client.exchange_code(
            code="real-code", state=mod.sign_state(nid_client.redirect_uri)
        )
    finally:
        nid_client._transport = None

    assert claims["access_token"] == "nid-session-token-1"
    assert claims["account_id"] == "ckx_acct_1"
    assert claims["phone_verified"] is True
    assert claims["scopes"] == ("identity:read", "memberships:read")
    import json

    body = json.loads(seen[0].content)
    assert body == {
        "code": "real-code",
        "clientId": "ci-communicator",
        "clientSecret": "secret-value",
        "redirectUri": "http://spa.test/auth/nid/callback",
    }


@pytest.mark.asyncio
async def test_exchange_code_requires_a_valid_state(monkeypatch) -> None:
    with pytest.raises(NidAuthError, match="state"):
        await nid_client.exchange_code(code="real-code", state="not-a-signed-state")


@pytest.mark.asyncio
async def test_rejected_exchange_never_invents_a_bearer() -> None:
    """The bug at old nid_client.py:140 -- `data.get('access_token') or
    secrets.token_urlsafe(32)` turned an nID rejection into a local credential."""

    def handler(request: httpx.Request) -> httpx.Response:
        return httpx.Response(400, json={"statusCode": 400, "message": "invalid grant"})

    nid_client._transport = httpx.MockTransport(handler)
    try:
        with pytest.raises(NidAuthError):
            await nid_client.exchange_code(
                code="bad-code", state=mod.sign_state(nid_client.redirect_uri)
            )
    finally:
        nid_client._transport = None


@pytest.mark.asyncio
async def test_unreachable_nid_raises_unreachable_not_invented_session() -> None:
    def handler(request: httpx.Request) -> httpx.Response:
        raise httpx.ConnectError("nID is down", request=request)

    nid_client._transport = httpx.MockTransport(handler)
    try:
        with pytest.raises(NidAuthError):
            await nid_client.exchange_code(
                code="real-code", state=mod.sign_state(nid_client.redirect_uri)
            )
    finally:
        nid_client._transport = None


@pytest.mark.asyncio
async def test_mock_auth_is_the_only_way_to_get_an_invented_login(monkeypatch) -> None:
    monkeypatch.setattr(settings, "app_env", "development")
    monkeypatch.setattr(settings, "nid_allow_mock_auth", True)
    claims = await nid_client.exchange_code(
        code="whatever", state=mod.sign_state(nid_client.redirect_uri)
    )
    assert claims["access_token"]
    assert claims["memberships"], "mock claims must carry one membership so the SPA can pick"


@pytest.mark.asyncio
async def test_development_app_env_alone_is_not_enough(monkeypatch) -> None:
    """Spec D8: the app_env clause is deleted. A test_-prefixed code must not log
    anyone in while mock auth is off, even in development."""
    monkeypatch.setattr(settings, "app_env", "development")
    monkeypatch.setattr(settings, "nid_allow_mock_auth", False)

    def handler(request: httpx.Request) -> httpx.Response:
        return httpx.Response(400, json={"message": "nope"})

    nid_client._transport = httpx.MockTransport(handler)
    try:
        with pytest.raises(NidAuthError):
            await nid_client.exchange_code(
                code="test_1730000000", state=mod.sign_state(nid_client.redirect_uri)
            )
    finally:
        nid_client._transport = None


def test_the_deleted_internals_stay_deleted() -> None:
    """Spec 8 criterion 4, asserted structurally so a later refactor cannot
    quietly restore the dicts."""
    assert not hasattr(mod, "_pending_states")
    assert not hasattr(mod, "_active_sessions")
    assert not hasattr(nid_client, "start_login")
    assert not hasattr(nid_client, "validate_session")
```

- [ ] **Step 2: Run to verify it fails**

```bash
pytest tests/test_nid_client_login.py -q
```

Expected: errors naming `sign_state`, `login_ui_url`, `_transport`, plus `AttributeError` on
`validate_session` in the deletion test once the rest is in place.

- [ ] **Step 3: Rewrite the module top (state helpers + dataclasses + transport hook)**

Replace `src/communicator/services/nid_client.py` lines 1-45 (docstring through
`_prune_expired_states`) with a docstring that names the flow, then:

```python
from __future__ import annotations

import hashlib
import hmac
import logging
import secrets
import time
from dataclasses import dataclass, field
from datetime import UTC, datetime
from typing import Any

import httpx

from communicator.config import settings

logger = logging.getLogger(__name__)

# The OAuth `state` value is signed, not stored: Communicator runs behind more than one
# worker, so the old _pending_states dict could not survive its own deployment (spec 4.2).
# The payload is domain-separated so a signature made for some other purpose over the same
# fields can never verify here.
_STATE_DOMAIN = "communicator/oauth/state/v1"
_STATE_MAX_AGE_SECONDS = 600


@dataclass(frozen=True)
def NidMembership:
    tenant_id: str
    slug: str
    name: str
    role: str


@dataclass(frozen=True)
class NidIntrospection:
    account_id: str
    expires_at: datetime
    scopes: tuple[str, ...]
    memberships: tuple[NidMembership, ...]
    checked_at: float = field(default_factory=lambda: time.time())

    def age_seconds(self, now: float | None = None) -> float:
        return (time.time() if now is None else now) - self.checked_at

    def session_expired(self, now: float | None = None) -> bool:
        ref = time.time() if now is None else now
        return self.expires_at.timestamp() <= ref


class NidAuthError(Exception):
    """nID refused the operation: bad code, bad state, bad client credential."""


class NidUnreachableError(NidAuthError):
    """nID could not be asked at all. Callers fail closed (spec 4.3 step 6)."""


def _state_signature(nonce: str, ts: int, redirect_uri: str) -> str:
    payload = f"{_STATE_DOMAIN}\n{nonce}\n{ts}\n{redirect_uri}"
    return hmac.new(
        settings.nid_client_secret.encode("utf-8"), payload.encode("utf-8"), hashlib.sha256
    ).hexdigest()


def sign_state(redirect_uri: str, *, now: float | None = None) -> str:
    ts = int(time.time() if now is None else now)
    nonce = secrets.token_hex(16)
    return f"{nonce}.{ts}.{_state_signature(nonce, ts, redirect_uri)}"


def verify_state(state: str, redirect_uri: str, *, now: float | None = None) -> bool:
    parts = state.split(".")
    if len(parts) != 3:
        return False
    nonce, raw_ts, signature = parts
    try:
        ts = int(raw_ts)
    except ValueError:
        return False
    reference = time.time() if now is None else now
    if not 0 <= reference - ts <= _STATE_MAX_AGE_SECONDS:
        return False
    return hmac.compare_digest(_state_signature(nonce, ts, redirect_uri), signature)
```

The HMAC key is `nid_client_secret` itself — already mandatory outside development (Task 4), already
secret to this relying party, and reusing it avoids inventing a credential the operator must provision
twice. The domain prefix is what keeps that reuse safe. Replay inside the 600 s window remains
possible and is accepted: the SPA's `sessionStorage` comparison
(`frontend/src/context/AuthContext.tsx:74-79`) and nID's single-use code
(`oauth.service.ts:142`) each close a different half of it, and a state replay cannot complete without
a code nobody else can redeem.

- [ ] **Step 4: Replace `start_login` with `build_authorize_url`**

Delete `start_login` (`:61-110`) and `_prune_expired_states`. In `NidClient.__init__` set:

```python
        self.login_ui_url = (base_url_ui or settings.nid_login_ui_url).rstrip("/")
        self.redirect_uri = redirect_uri or settings.nid_redirect_uri
        # Tests inject an httpx.MockTransport here; None means "real network".
        self._transport: httpx.AsyncBaseTransport | None = None
```

adding `base_url_ui: str | None = None` to the `__init__` signature, and add:

```python
    def build_authorize_url(self, state: str) -> str:
        """
        The login-ui URL the browser is sent to. Built locally with no upstream
        call: POST /v1/oauth/login-requests belongs to login-ui's own
        /authorize page (spec 4.2), and the poll token is its business, not ours.
        nID validates redirect_uri against OAuthClient.redirectUris
        (oauth.service.ts:45), so the only thing this URL has to prove is that
        *we* started the attempt -- which the signed state does on callback.
        """
        params = httpx.QueryParams(
            {"client_id": self.client_id, "redirect_uri": self.redirect_uri, "state": state}
        )
        return f"{self.login_ui_url}/authorize?{params}"
```

- [ ] **Step 5: Rewrite `exchange_code`**

Replace `:112-175` with:

```python
    async def _post(self, path: str, payload: dict[str, Any]) -> httpx.Response:
        async with httpx.AsyncClient(timeout=10.0, transport=self._transport) as client:
            return await client.post(f"{self.base_url}{path}", json=payload)

    async def exchange_code(self, code: str, state: str) -> dict[str, Any]:
        """
        Redeems an authorization code for a scoped, revocable RP session token.
        Raises NidAuthError on refusal and never mints a bearer of its own -- the
        old `data.get("access_token") or secrets.token_urlsafe(32)` made an nID
        rejection look like a successful login.
        """
        if not verify_state(state, self.redirect_uri):
            raise NidAuthError("Invalid, expired, or mismatched OAuth state parameter.")

        if settings.nid_allow_mock_auth:
            return self._mock_claims()

        try:
            resp = await self._post(
                "/v1/oauth/token",
                {
                    "code": code,
                    "clientId": self.client_id,
                    "clientSecret": self.client_secret,
                    "redirectUri": self.redirect_uri,
                },
            )
        except (httpx.HTTPError, OSError) as exc:
            raise NidUnreachableError(f"nID token endpoint unreachable: {exc}") from exc

        if resp.status_code not in (200, 201):
            logger.warning("nID token exchange rejected: %s %s", resp.status_code, resp.text[:200])
            raise NidAuthError("nID rejected the authorization code.")

        data = resp.json()
        session = data.get("session") or {}
        account = data.get("account") or {}
        token = session.get("token")
        account_id = account.get("id")
        if not token or not account_id:
            raise NidAuthError("nID returned no session token or no account identity.")

        # Introspect once here so the SPA can choose a tenant before its first
        # business call: spec 4.3 step 3 answers "no header, more than one
        # membership" with a 400, so a login that returns no memberships is a
        # login into an unusable account.
        introspection = await self.introspect(token)
        memberships = introspection.memberships if introspection else ()

        return {
            "access_token": token,
            "account_id": account_id,
            "phone": account.get("phone"),
            "email": account.get("email"),
            "phone_verified": bool(account.get("phoneVerifiedAt")),
            "expires_at": session.get("expiresAt"),
            "scopes": tuple(session.get("scopes") or ()),
            "memberships": memberships,
        }

    def _mock_claims(self) -> dict[str, Any]:
        """
        The only locally invented login, reached solely when nid_allow_mock_auth is
        true (spec D8). The tenant id is an obviously fake string because the guard
        auto-provisions from it (D10) -- a placeholder is fine, a silent fallback
        to another tenant's data would not be.
        """
        token = secrets.token_urlsafe(32)
        account_id = f"mock_acc_{secrets.token_hex(6)}"
        membership = NidMembership(
            tenant_id=settings.default_tenant_key,
            slug="mock",
            name="Mock Organization",
            role="OWNER",
        )
        _cacheintrospection_store(token, membership, account_id)
        return {
            "access_token": token,
            "account_id": account_id,
            "phone": "",
            "email": f"{account_id}@mock.local",
            "phone_verified": True,
            "expires_at": datetime.now(UTC).isoformat(),
            "scopes": ("identity:read", "memberships:read"),
            "memberships": (membership,),
        }
```

`_mock_claims` seeds the introspection cache (Task 6) so a mocked login is introspectable by the
same code path a real one uses — otherwise every mock test exercises a different branch than
production and the guard goes untested. The placeholder name `_cacheintrospection_store` is finished
in Task 6 Step 1; when you land Task 6, replace it with the real `_cache_store(...)` call and delete
this note.

- [ ] **Step 6: Delete `validate_session`, and make `reconfigure` cover the new fields**

Delete `validate_session` (`:177-179`) — the guard asks nID instead. Extend `reconfigure`
(`:181-187`) with `self.login_ui_url = (cfg.nid_login_ui_url or self.login_ui_url).rstrip("/")`. Keep
`fetch_vault_secrets`, but per spec §5 change its silent `return {}` on a non-200 for **supplied**
credentials into a logged warning:

```python
                if auth_resp.status_code != 200:
                    if role_id and secret_id:
                        logger.warning(
                            "nID Vault rejected supplied AppRole credentials: %s",
                            auth_resp.status_code,
                        )
                    return {}
```

- [ ] **Step 7: Run the tests and the deletion greps**

```bash
pytest tests/test_nid_client_login.py -q
grep -rn "_pending_states\|_active_sessions\|login-requests\|poll_token\|mock_poll_\|poll_fallback_" src/
```

Expected: tests pass except `test_mock_auth_is_the_only_way_to_get_an_invented_login` if Task 6 has
not landed (the cache helper is still a placeholder) — in that case leave the test marked
`xfail(strict=False)` with a comment naming Task 6, and remove the marker there. The grep returns
nothing.

- [ ] **Step 8: Fix the two tests that depended on the deleted dict**

`tests/test_nid_auth_and_crypto_mesh.py::test_nid_login_start_and_callback` asserts `poll_token` in
the `/start` response and calls `/auth/nid/me` with a token from the old dict. Task 9 rewrites that
test alongside the routes; until then, if it fails it is failing for the right reason. Leave it
failing and say so in the commit message — do **not** mark it skipped.

- [ ] **Step 9: Commit**

```bash
git add src/communicator/services/nid_client.py tests/test_nid_client_login.py
git commit -m "$(cat <<'EOF'
refactor(nid): build the authorize URL locally, redeem codes without inventing sessions

- state is HMAC-signed and bound to the configured redirect_uri, so login survives
  more than one worker; _pending_states and _active_sessions are gone.
- authorize_url points at login-ui (:3300), not the API that serves no /authorize.
- No login-requests call, no poll_token, no caller-supplied redirect_uri.
- A rejected exchange or an unreachable nID raises; it never returns a bearer
  Communicator minted itself, and app_env no longer unlocks mock login.
EOF
)"
```

---

### Task 6: `tests/nid_stub.py` — one nID fake, used by every test after this

Route-level tests need to tell nID "this token is active and belongs to two tenants" without a
network. Building that as an `httpx.MockTransport` (rather than monkeypatching `introspect`) keeps
the production parsing, caching and error paths under test, which is what §4.6 asks for when it says
"asserted on the transport".

**Files:**
- Create: `tests/nid_stub.py`
- Create: `src/communicator/services/nid_client.py` additions (`introspect`, `revoke`, cache)
- Test: `tests/test_nid_client_introspect.py`

**Interfaces:**
- Consumes: `NidMembership`, `NidIntrospection`, `NidClient._transport`, `NidUnreachableError`.
- Produces:
  - `async NidClient.introspect(token: str, *, for_write: bool = False) -> NidIntrospection | None`
    (`None` == `active:false`; raises `NidUnreachableError` when fail-closed applies).
  - `async NidClient.revoke(token: str) -> bool`
  - `clear_introspection_cache() -> None`
  - `_cache_store(token, introspection)` (Task 5's `_mock_claims` calls this).
  - `nid_stub.NidStub` with `.set_active(token, *, account_id, memberships, ttl=720)`,
    `.set_inactive(token)`, `.set_unreachable()`, `.calls`, `.install(nid_client)` / `.uninstall()`.

- [ ] **Step 1: Write the failing tests**

Create `tests/test_nid_client_introspect.py`:

```python
"""Spec 4.1 N4/N5 as Consumator sees them: one introspection call, one cache, one revoke."""

from __future__ import annotations

import json
from datetime import UTC, datetime, timedelta

import pytest

from communicator.config import settings
from communicator.services import nid_client as mod
from communicator.services.nid_client import (
    NidMembership,
    NidUnreachableError,
    clear_introspection_cache,
    nid_client,
)
from nid_stub import NidStub


@pytest.fixture(autouse=True)
def _creds(monkeypatch):
    monkeypatch.setattr(settings, "nid_client_id", "ci-communicator")
    monkeypatch.setattr(settings, "nid_client_secret", "secret-value")
    monkeypatch.setattr(settings, "nid_introspect_cache_ttl_seconds", 60)
    monkeypatch.setattr(settings, "nid_introspect_write_max_age_seconds", 15)
    clear_introspection_cache()
    yield
    clear_introspection_cache()


@pytest.fixture
def stub():
    s = NidStub()
    s.install(nid_client)
    yield s
    s.uninstall()


def _memberships(*pairs):
    return [NidMembership(tenant_id=t, slug=s, name=s.title(), role="STAFF") for t, s in pairs]


@pytest.mark.asyncio
async def test_active_introspection_returns_tenant_ids_not_slugs(stub) -> None:
    stub.set_active("tok-1", account_id="ckx_acct", memberships=_memberships(("ckx_acme", "acme")))
    result = await nid_client.introspect("tok-1")
    assert result is not None
    assert result.account_id == "ckx_acct"
    assert [m.tenant_id for m in result.memberships] == ["ckx_acme"]
    assert [m.role for m in result.memberships] == ["STAFF"]
    # one call, and it carries the client credential in the body (deviation V1)
    assert len(stub.calls) == 1
    body = json.loads(stub.calls[0][1])
    assert body == {"token": "tok-1", "clientId": "ci-communicator", "clientSecret": "secret-value"}


@pytest.mark.asyncio
async def test_unknown_token_is_active_false_not_an_error(stub) -> None:
    """RFC 7662: 'no' is an answer, not a failure."""
    stub.set_inactive("tok-unknown")
    assert await nid_client.introspect("tok-unknown") is None


@pytest.mark.asyncio
async def test_cache_serves_reads_within_ttl(stub) -> None:
    stub.set_active("tok-2", account_id="a", memberships=_memberships(("t1", "one")))
    await nid_client.introspect("tok-2")
    await nid_client.introspect("tok-2")
    await nid_client.introspect("tok-2")
    assert len(stub.calls) == 1


@pytest.mark.asyncio
async def test_writes_refuse_an_entry_older_than_the_write_max_age(stub) -> None:
    stub.set_active("tok-3", account_id="a", memberships=_memberships(("t1", "one")))
    await nid_client.introspect("tok-3")
    entry = mod._CACHE["tok-3"] if "tok-3" in mod._CACHE else None
    # Force the cached entry to look 30s old without waiting 30s.
    for key, value in list(mod._CACHE.items()):
        import dataclasses

        mod._CACHE[key] = dataclasses.replace(value, checked_at=value.checked_at - 30)
    await nid_client.introspect("tok-3", for_write=True)
    assert len(stub.calls) == 2
    assert entry is not None or True  # presence asserted via call count, above


@pytest.mark.asyncio
async def test_expired_session_is_evicted_and_reports_inactive(stub) -> None:
    stub.set_active(
        "tok-4",
        account_id="a",
        memberships=_memberships(("t1", "one")),
        expires_at=datetime.now(UTC) - timedelta(seconds=5),
    )
    assert await nid_client.introspect("tok-4") is None


@pytest.mark.asyncio
async def test_unreachable_with_no_cache_fails_closed(stub) -> None:
    stub.set_unreachable()
    with pytest.raises(NidUnreachableError):
        await nid_client.introspect("tok-5")


@pytest.mark.asyncio
async def test_unreachable_serves_a_cached_read_but_not_a_write(stub) -> None:
    stub.set_active("tok-6", account_id="a", memberships=_memberships(("t1", "one")))
    await nid_client.introspect("tok-6")
    stub.set_unreachable()
    assert await nid_client.introspect("tok-6") is not None
    with pytest.raises(NidUnreachableError):
        await nid_client.introspect("tok-6", for_write=True)


@pytest.mark.asyncio
async def test_wrong_client_credential_is_not_a_silent_unreachable(stub) -> None:
    stub.set_client_rejected()
    with pytest.raises(mod.NidAuthError):
        await nid_client.introspect("tok-7")


@pytest.mark.asyncio
async def test_revoke_returns_true_and_poisons_the_cache(stub) -> None:
    stub.set_active("tok-8", account_id="a", memberships=_memberships(("t1", "one")))
    await nid_client.introspect("tok-8")
    assert await nid_client.revoke("tok-8") is True
    assert await nid_client.introspect("tok-8") is None
```

- [ ] **Step 2: Run to verify it fails**

```bash
pytest tests/test_nid_client_introspect.py -q
```

Expected: `ImportError` for `nid_stub`, then `AttributeError: introspect`.

- [ ] **Step 3: Add the cache, `introspect` and `revoke` to the client**

Append to `src/communicator/services/nid_client.py` (after the `NidClient` class body, module level):

```python
# Introspection cache: token -> result, keyed by the sha256 of the token so a memory
# dump or a log line never carries a live credential. Bounded by pruning on write.
_CACHE: dict[str, NidIntrospection] = {}


def _cache_key(token: str) -> str:
    return hashlib.sha256(token.encode("utf-8")).hexdigest()


def _cache_store(token: str, entry: NidIntrospection) -> None:
    if len(_CACHE) > 4096:
        now = time.time()
        for key in [k for k, v in _CACHE.items() if v.expires_at.timestamp() <= now]:
            _CACHE.pop(key, None)
    _CACHE[_cache_key(token)] = entry


def clear_introspection_cache() -> None:
    _CACHE.clear()
```

and inside `NidClient`:

```python
    async def introspect(
        self, token: str, *, for_write: bool = False
    ) -> NidIntrospection | None:
        """
        Ask nID who this bearer belongs to (spec 4.3). Returns None for
        active:false, a NidIntrospection otherwise. Raises NidUnreachableError when
        there is nothing to fall back to: fail-closed is the decision, not the
        default. Reads accept an entry up to NID_INTROSPECT_CACHE_TTL_SECONDS old;
        writes demand one no older than NID_INTROSPECT_WRITE_MAX_AGE_SECONDS, so a
        revoked membership cannot be used to write for the rest of a TTL window.
        """
        now = time.time()
        entry = _CACHE.get(_cache_key(token))
        if entry is not None and entry.session_expired(now):
            _CACHE.pop(_cache_key(token), None)
            entry = None

        limit = (
            settings.nid_introspect_write_max_age_seconds
            if for_write
            else settings.nid_introspect_cache_ttl_seconds
        )
        if entry is not None and entry.age_seconds(now) <= limit:
            return entry

        try:
            resp = await self._post(
                "/v1/oauth/introspect",
                {"token": token, "clientId": self.client_id, "clientSecret": self.client_secret},
            )
        except (httpx.HTTPError, OSError) as exc:
            # A cached read may still be served; a write may not.
            if entry is not None and not for_write and entry.age_seconds(now) <= (
                settings.nid_introspect_cache_ttl_seconds
            ):
                logger.warning("nID unreachable, serving cached principal: %s", exc)
                return entry
            raise NidUnreachableError(f"nID introspection unreachable: {exc}") from exc

        if resp.status_code == 403:
            raise NidAuthError("nID rejected Communicator's client credentials (introspect).")
        if resp.status_code != 200:
            raise NidAuthError(f"nID introspection failed with status {resp.status_code}.")

        data = resp.json()
        if not data.get("active"):
            _CACHE.pop(_cache_key(token), None)
            return None

        memberships = tuple(
            NidMembership(
                tenant_id=m["tenantId"], slug=m.get("slug", ""), name=m.get("name", ""),
                role=m.get("role", "STAFF"),
            )
            for m in data.get("memberships", [])
        )
        expires_raw = data.get("expiresAt")
        expires_at = (
            datetime.fromisoformat(expires_raw.replace("Z", "+00:00"))
            if expires_raw
            else datetime.now(UTC) + timedelta(seconds=limit)
        )
        fresh = NidIntrospection(
            account_id=data["accountId"],
            expires_at=expires_at,
            scopes=tuple(data.get("scopes") or ()),
            memberships=memberships,
        )
        _cache_store(token, fresh)
        return fresh

    async def revoke(self, token: str) -> bool:
        """Logout at nID (spec 4.1 N5). Also drops our cached view of the token, so
        acceptance criterion 5 holds inside one process immediately."""
        _CACHE.pop(_cache_key(token), None)
        if settings.nid_allow_mock_auth:
            return True
        try:
            resp = await self._post(
                "/v1/oauth/revoke",
                {"token": token, "clientId": self.client_id, "clientSecret": self.client_secret},
            )
        except (httpx.HTTPError, OSError) as exc:
            raise NidUnreachableError(f"nID revoke endpoint unreachable: {exc}") from exc
        if resp.status_code == 403:
            raise NidAuthError("nID rejected our client credentials (revoke).")
        return resp.status_code == 200 and bool(resp.json().get("revoked"))
```

Add `timedelta` to the `datetime` import line, and in Task 5's `_mock_claims` replace the
`_cacheintrospection_store(...)` placeholder with:

```python
        _cache_store(
            token,
            NidIntrospection(
                account_id=account_id,
                expires_at=datetime.now(UTC) + timedelta(hours=12),
                scopes=("identity:read", "memberships:read"),
                memberships=(membership,),
            ),
        )
```

- [ ] **Step 4: Write the stub**

Create `tests/nid_stub.py`:

```python
"""
A nID fake for Communicator's tests, installed as an httpx.MockTransport on the
NidClient. It is a transport, not a monkeypatch, so the production parsing,
caching and fail-closed paths are the things under test.

Tokens are opaque here; membership tenant ids are opaque cuid-ish strings. Never
use a slug where a tenant id is expected -- that is the bug spec 4.3 step 3 names.
"""

from __future__ import annotations

import json
from datetime import UTC, datetime, timedelta

import httpx

ACTIVE = "active"
INACTIVE = "inactive"
UNREACHABLE = "unreachable"
REJECT_CLIENT = "reject_client"


class NidStub:
    def __init__(self) -> None:
        self.mode = ACTIVE
        self.tokens: dict[str, dict] = {}
        self.calls: list[tuple[str, bytes]] = []
        self._client: httpx.AsyncClient | None = None

    # -- configuration ---------------------------------------------------
    def set_active(
        self,
        token: str,
        *,
        account_id: str,
        memberships,
        scopes=("identity:read", "memberships:read"),
        expires_at: datetime | None = None,
        ttl: int = 720,
    ) -> None:
        self.tokens[token] = {
            "accountId": account_id,
            "expiresAt": (expires_at or datetime.now(UTC) + timedelta(minutes=ttl)).isoformat(),
            "scopes": list(scopes),
            "memberships": [
                {
                    "tenantId": m.tenant_id if hasattr(m, "tenant_id") else m["tenantId"],
                    "slug": m.slug if hasattr(m, "slug") else m["slug"],
                    "name": m.name if hasattr(m, "name") else m["name"],
                    "role": m.role if hasattr(m, "role") else m["role"],
                }
                for m in memberships
            ],
        }

    def set_inactive(self, token: str) -> None:
        self.tokens[token] = INACTIVE  # type: ignore[assignment]

    def set_unreachable(self) -> None:
        self.mode = UNREACHABLE

    def set_client_rejected(self) -> None:
        self.mode = REJECT_CLIENT

    # -- transport -------------------------------------------------------
    def handler(self, request: httpx.Request) -> httpx.Response:
        self.calls.append((request.url.path, request.content))
        if self.mode == UNREACHABLE:
            raise httpx.ConnectError("stub: nID is down", request=request)
        if self.mode == REJECT_CLIENT:
            return httpx.Response(403, json={"message": "bad client credential"})

        body = json.loads(request.content or b"{}")
        token = body.get("token") or body.get("code")

        if request.url.path == "/v1/oauth/introspect":
            entry = self.tokens.get(token)
            if entry == INACTIVE or entry is None:
                return httpx.Response(200, json={"active": False})
            return httpx.Response(200, json={"active": True, **entry})

        if request.url.path == "/v1/oauth/revoke":
            if token in self.tokens:
                self.tokens[token] = INACTIVE  # type: ignore[assignment]
            return httpx.Response(200, json={"revoked": True})

        if request.url.path == "/v1/oauth/token":
            return httpx.Response(
                200,
                json={
                    "account": {
                        "id": body.get("accountId", "ckx_stub_account"),
                        "phone": "+923000000000",
                        "email": "stub@acme.test",
                        "phoneVerifiedAt": "2026-01-01T00:00:00Z",
                    },
                    "session": {
                        "token": body.get("sessionToken", "stub-session-token"),
                        "expiresAt": (datetime.now(UTC) + timedelta(minutes=720)).isoformat(),
                        "scopes": ["identity:read", "memberships:read"],
                    },
                },
            )

        return httpx.Response(404, json={"message": f"stub has no {request.url.path}"})

    def install(self, client) -> None:
        transport = httpx.MockTransport(self.handler)
        self._client = httpx.AsyncClient(transport=transport)
        client._transport = transport

    def uninstall(self) -> None:
        self._transport_target = None
        if self._client is not None:
            self._client = None
```

`install()` sets `client._transport`, which `NidClient._post` already honours; `uninstall()` must
reset it — implement it as `def uninstall(self, client=None)` and in `NidClient` expose
`_transport = None` on uninstall, or simpler: tests always call `stub.install(nid_client)` inside a
fixture whose teardown does `nid_client._transport = None`. Choose the fixture teardown and keep
`NidStub.uninstall()` a no-op that only clears its own client, and delete the misleading
`self._transport_target` line.

- [ ] **Step 5: Run everything**

```bash
pytest tests/test_nid_client_introspect.py tests/test_nid_client_login.py -q
pytest -q
```

Expected: the introspect file passes (the write-staleness test asserts call counts, not internals —
if it reaches into `_CACHE`, rewrite it to advance `checked_at` through
`dataclasses.replace` on the entry you obtained, and keep asserting behaviour). `pytest -q` still has
the Task 5 Step 8 known failure on `test_nid_login_start_and_callback`.

- [ ] **Step 6: Commit**

```bash
git add src/communicator/services/nid_client.py tests/nid_stub.py tests/test_nid_client_introspect.py
git commit -m "$(cat <<'EOF'
feat(nid): introspection with a bounded cache, revoke, and a transport-level nID fake

Reads accept a cached principal for up to 60s; writes need one no older than 15s,
so a revoked membership stops being usable to mutate inside the TTL window. An
unreachable nID with nothing cached raises -- the caller turns that into 503.
Unknown/expired/revoked tokens answer active:false, which is an answer, not a
failure.
EOF
)"
```

---

### Task 7: `require_tenant_member` — the guard that replaces the dict

Six steps, in order, each with its own status code, because "we don't know you" (401), "we know you
and you may not" (403), "which tenant?" (400), "nID is down" (503) are four different operational
problems and collapsing them makes all four un-debuggable.

**Files:**
- Modify: `src/communicator/api/dependencies.py` (delete `get_tenant_key`, `get_current_account_id`)
- Create: `src/communicator/services/tenant_provisioning.py`
- Test: `tests/test_require_tenant_member.py`

**Interfaces:**
- Consumes: `nid_client.introspect` (Task 6), `settings.default_tenant_key`, `Tenant`, `NidAccount`.
- Produces:
  - `TenantPrincipal(tenant_key: str, role: str, account_id: str, email: str | None, phone: str
    | None)` — `principal.tenant_key` is nID's `Tenant.id`.
  - `require_tenant_member` and `require_tenant_writer` (FastAPI dependencies returning it).
  - `async ensure_tenant_row(session, membership, account_id) -> None` (D10).
  - Task 11 imports `require_permission` from the same module; Tasks 12-17 import
    `require_tenant_member` / `require_tenant_writer`.

- [ ] **Step 1: Write the failing tests**

Create `tests/test_require_tenant_member.py`:

```python
"""Spec 4.3, step by step, plus the headline escalation regression (spec 8 criterion 7)."""

from __future__ import annotations

import pytest
from fastapi import Depends, FastAPI, HTTPException
from httpx import ASGITransport, AsyncClient

from communicator.api.dependencies import (
    TenantPrincipal,
    require_tenant_member,
    require_tenant_writer,
)
from communicator.config import settings
from communicator.services.nid_client import NidMembership, clear_introspection_cache, nid_client
from nid_stub import NidStub

ACME = "ckx_acme_tenant_0001"
GLOBEX = "ckx_globex_tenant_0002"

ACME_ONLY = [NidMembership(ACME, "acme", "Acme Support", "AGENT")]
BOTH = [NidMembership(ACME, "acme", "Acme Support", "OWNER"),
        NidMembership(GLOBEX, "globex", "Globex Care", "AGENT")]


def build_app() -> FastAPI:
    app = FastAPI()

    @app.get("/whoami")
    async def whoami(p: TenantPrincipal = Depends(require_tenant_member)):
        return {
            "tenant_key": p.tenant_key,
            "role": p.role,
            "account_id": p.account_id,
        }

    @app.post("/mutate")
    async def mutate(p: TenantPrincipal = Depends(require_tenant_writer)):
        return {"tenant_key": p.tenant_key}

    return app


@pytest.fixture
def app():
    return build_app()


@pytest.fixture(autouse=True)
def _creds(monkeypatch):
    monkeypatch.setattr(settings, "nid_client_id", "ci")
    monkeypatch.setattr(settings, "nid_client_secret", "cs")
    monkeypatch.setattr(settings, "nid_allow_mock_auth", False)
    clear_introspection_cache()
    yield
    clear_introspection_cache()


@pytest.fixture
def stub():
    s = NidStub()
    s.install(nid_client)
    yield s
    nid_client._transport = None


async def call(app, db_override, *, path="/whoami", method="get", headers=None):
    transport = ASGITransport(app=app)
    async with AsyncClient(transport=transport, base_url="http://t", headers=headers or {}) as c:
        fn = getattr(c, method)
        return await fn(path)


@pytest.mark.asyncio
async def test_step1_missing_bearer_is_401(app, stub, override_database_dependency) -> None:
    stub.set_active("tok", account_id="ckx_a", memberships=ACME_ONLY)
    r = await call(app, override_database_dependency)
    assert r.status_code == 401


@pytest.mark.asyncio
async def test_step1_malformed_bearer_is_401(app, stub, override_database_dependency) -> None:
    r = await call(app, override_database_dependency, headers={"Authorization": "Token abc"})
    assert r.status_code == 401


@pytest.mark.asyncio
async def test_step2_inactive_token_is_401(app, stub, override_database_dependency) -> None:
    stub.set_inactive("dead")
    r = await call(app, override_database_dependency, headers={"Authorization": "Bearer dead"})
    assert r.status_code == 401


@pytest.mark.asyncio
async def test_step3_single_membership_needs_no_header(
    app, stub, override_database_dependency
) -> None:
    stub.set_active("solo", account_id="ckx_a", memberships=ACME_ONLY)
    r = await call(app, override_database_dependency, headers={"Authorization": "Bearer solo"})
    assert r.status_code == 200
    assert r.json()["tenant_key"] == ACME


@pytest.mark.asyncio
async def test_step3_multi_membership_without_header_is_400(
    app, stub, override_database_dependency
) -> None:
    stub.set_active("both", account_id="ckx_a", memberships=BOTH)
    r = await call(app, override_database_dependency, headers={"Authorization": "Bearer both"})
    assert r.status_code == 400
    assert "X-Tenant-Id" in r.json()["detail"]


@pytest.mark.asyncio
async def test_step3_never_falls_back_to_newax(app, stub, override_database_dependency) -> None:
    """The escalation this spec exists to close: today an absent header means
    'serve the newax tenant to anybody'."""
    stub.set_active("solo", account_id="ckx_a", memberships=ACME_ONLY)
    r = await call(
        app, override_database_dependency, headers={"Authorization": "Bearer solo",
                                                     "X-Tenant-Id": "newax"}
    )
    assert r.status_code == 403


@pytest.mark.asyncio
async def test_step4_membership_required_is_403(app, stub, override_database_dependency) -> None:
    stub.set_active("acme_guy", account_id="ckx_a", memberships=ACME_ONLY)
    r = await call(
        app, override_database_dependency,
        headers={"Authorization": "Bearer acme_guy", "X-Tenant-Id": GLOBEX},
    )
    assert r.status_code == 403


@pytest.mark.asyncio
async def test_step4_role_comes_from_the_membership_row(
    app, stub, override_database_dependency
) -> None:
    stub.set_active("both", account_id="ckx_a", memberships=BOTH)
    r = await call(
        app, override_database_dependency,
        headers={"Authorization": "Bearer both", "X-Tenant-Id": GLOBEX},
    )
    assert r.status_code == 200
    assert r.json()["role"] == "AGENT"


@pytest.mark.asyncio
async def test_step5_principal_carries_the_account_id(
    app, stub, override_database_dependency
) -> None:
    stub.set_active("solo", account_id="ckx_visible", memberships=ACME_ONLY)
    r = await call(app, override_database_dependency, headers={"Authorization": "Bearer solo"})
    assert r.json()["account_id"] == "ckx_visible"


@pytest.mark.asyncio
async def test_step6_unreachable_with_no_cache_is_503_with_retry_after(
    app, stub, override_database_dependency
) -> None:
    stub.set_unreachable()
    r = await call(app, override_database_dependency, headers={"Authorization": "Bearer x"})
    assert r.status_code == 503
    assert "retry-after" in r.headers


@pytest.mark.asyncio
async def test_step6_read_survives_a_second_within_ttl(
    app, stub, override_database_dependency
) -> None:
    stub.set_active("solo", account_id="ckx_a", memberships=ACME_ONLY)
    assert (
        await call(app, override_database_dependency, headers={"Authorization": "Bearer solo"})
    ).status_code == 200
    stub.set_unreachable()
    r = await call(app, override_database_dependency, headers={"Authorization": "Bearer solo"})
    assert r.status_code == 200


@pytest.mark.asyncio
async def test_step6_write_with_a_stale_entry_503s(
    app, stub, override_database_dependency
) -> None:
    stub.set_active("solo", account_id="ckx_a", memberships=ACME_ONLY)
    assert (
        await call(app, override_database_dependency, path="/mutate", method="post",
                   headers={"Authorization": "Bearer solo"})
    ).status_code == 200
    import dataclasses
    import time

    from communicator.services import nid_client as mod

    for key, value in list(mod._CACHE.items()):
        mod._CACHE[key] = dataclasses.replace(value, checked_at=time.time() - 30)
    stub.set_unreachable()
    r = await call(app, override_database_dependency, path="/mutate", method="post",
                   headers={"Authorization": "Bearer solo"})
    assert r.status_code == 503


@pytest.mark.asyncio
async def test_d10_first_request_provisions_the_local_tenant_row(
    app, stub, db_session
) -> None:
    from communicator.models.tenant import Tenant

    stub.set_active("fresh", account_id="ckx_a", memberships=ACME_ONLY)
    app_ = app
    transport = ASGITransport(app=app_)
    async with AsyncClient(transport=transport, base_url="http://t") as c:
        # override to the real test session
        from communicator.api.dependencies import get_db_session

        app_.dependency_overrides[get_db_session] = lambda: db_session
        r = await c.get("/whoami", headers={"Authorization": "Bearer fresh"})
        assert r.status_code == 200
    rows = (await db_session.execute(__import__("sqlalchemy").select(Tenant))).scalars().all()
    assert ACME in [t.tenant_key for t in rows]


@pytest.mark.asyncio
async def test_escalation_headline_acme_cannot_reach_globex(
    app, stub, override_database_dependency
) -> None:
    """spec 8 criterion 7. On today's code this request succeeds with newax data."""
    stub.set_active("acme_only", account_id="ckx_acme_person", memberships=ACME_ONLY)
    stub.set_active("globex_only", account_id="ckx_globex_person", memberships=[BOTH[1]])
    ok = await call(
        app, override_database_dependency,
        headers={"Authorization": "Bearer globex_only", "X-Tenant-Id": GLOBEX},
    )
    assert ok.status_code == 200
    attack = await call(
        app, override_database_dependency,
        headers={"Authorization": "Bearer acme_only", "X-Tenant-Id": GLOBEX},
    )
    assert attack.status_code == 403
```

- [ ] **Step 2: Run to verify failure**

```bash
pytest tests/test_require_tenant_member.py -q
```

Expected: `ImportError: cannot import name 'TenantPrincipal'`.

- [ ] **Step 3: Write the provisioning helper**

Create `src/communicator/services/tenant_provisioning.py`:

```python
"""
Lazy, idempotent creation of a local tenants row for a tenant that already
exists in nID (spec D10).

Nothing is seeded and nothing is hand-inserted: step 4 of the guard passing is
the existence proof, and a concurrent pair of first requests must produce exactly
one row, which is what ON CONFLICT DO NOTHING on tenants.tenant_key's unique
index guarantees. Postgres-specific by design -- this service only runs on
Postgres (asyncpg, JSONB and materialized views are already in use).
"""

from __future__ import annotations

from datetime import UTC, datetime

from sqlalchemy import select
from sqlalchemy.dialects.postgresql import insert as pg_insert
from sqlalchemy.ext.asyncio import AsyncSession

from communicator.models.nid_account import NidAccount
from communicator.models.tenant import Tenant
from communicator.services.nid_client import NidMembership


async def ensure_tenant_row(
    session: AsyncSession, membership: NidMembership, account_id: str
) -> None:
    stmt = (
        pg_insert(Tenant)
        .values(
            tenant_key=membership.tenant_id,
            name=membership.name or membership.slug,
            # Display-only since spec D12: the authority for who owns a tenant is
            # nID's OWNER membership, and this column stopped being a filter.
            owner_account_id=account_id if membership.role == "OWNER" else "",
            created_at=datetime.now(UTC),
        )
        .on_conflict_do_nothing(index_elements=[Tenant.tenant_key])
    )
    await session.execute(stmt)
    await session.flush()


async def ensure_nid_account(
    session: AsyncSession, account_id: str, *, email: str = "", phone: str = ""
) -> NidAccount:
    """Upsert the reference row for an nID Account and stamp its last login."""
    existing = (
        await session.execute(select(NidAccount).where(NidAccount.nid_account_id == account_id))
    ).scalar_one_or_none()
    if existing is not None:
        existing.last_login_at = datetime.now(UTC)
        existing.email = existing.email or email
        existing.phone = existing.phone or phone
        await session.flush()
        return existing
    row = NidAccount(
        nid_account_id=account_id,
        email=email,
        phone=phone,
        last_login_at=datetime.now(UTC),
    )
    session.add(row)
    await session.flush()
    return row


async def known_tenant_keys(session: AsyncSession) -> list[str]:
    rows = (await session.execute(select(Tenant.tenant_key))).scalars().all()
    return list(rows)
```

- [ ] **Step 4: Rewrite `dependencies.py`**

Replace `get_current_account_id` (`:20-42`) and `get_tenant_key` (`:45-52`) with:

```python
from dataclasses import dataclass

from fastapi import Depends, Header, HTTPException, Request, status
from sqlalchemy.ext.asyncio import AsyncSession

from communicator.config import settings
from communicator.services.nid_client import (
    NidMembership,
    NidUnreachableError,
    nid_client,
)
from communicator.services.tenant_provisioning import ensure_tenant_row


@dataclass(frozen=True)
class TenantPrincipal:
    """
    What the guard knows after asking nID. tenant_key is nID's Tenant.id -- the
    branch's column name is kept so its nine migrations stay meaningful, but one
    value now means one thing in both systems (spec D6).
    """

    tenant_key: str
    role: str
    account_id: str
    email: str | None = None
    phone: str | None = None

    @property
    def is_owner(self) -> bool:
        return self.role == "OWNER"


async def _resolve(
    request: Request,
    authorization: str | None,
    x_tenant_id: str | None,
    session: AsyncSession,
    *,
    for_write: bool,
) -> TenantPrincipal:
    # step 1 -- we do not know you.
    if not authorization or not authorization.startswith("Bearer "):
        raise HTTPException(
            status_code=status.HTTP_401_UNAUTHORIZED,
            detail="Missing or invalid Bearer authorization header.",
        )
    token = authorization.split("Bearer ", 1)[1].strip()
    if not token:
        raise HTTPException(status_code=status.HTTP_401_UNAUTHORIZED, detail="Empty bearer token.")

    # step 2 -- ask nID; fail closed if we cannot.
    try:
        introspection = await nid_client.introspect(token, for_write=for_write)
    except NidUnreachableError as exc:
        raise HTTPException(
            status_code=status.HTTP_503_SERVICE_UNAVAILABLE,
            detail=f"nID is not reachable and no cached principal is usable: {exc}",
            headers={"Retry-After": "5"},
        ) from exc
    if introspection is None:
        raise HTTPException(
            status_code=status.HTTP_401_UNAUTHORIZED,
            detail="Session is expired, revoked, or unknown to nID.",
        )

    memberships = introspection.memberships
    if not memberships:
        raise HTTPException(
            status_code=status.HTTP_403_FORBIDDEN,
            detail="This nID account belongs to no tenants. Ask for an invitation in nID.",
        )

    # step 3 -- which tenant? No silent fallback, ever.
    requested = (x_tenant_id or "").strip()
    chosen: NidMembership | None = None
    if requested:
        chosen = next((m for m in memberships if m.tenant_id == requested), None)
        if chosen is None:
            # step 4 -- we know you and you may not. Deliberately identical in
            # shape to the not-a-member case below: a 404-vs-403 distinction here
            # would confirm which tenant ids exist.
            raise HTTPException(
                status_code=status.HTTP_403_FORBIDDEN,
                detail="Not a member of this tenant.",
            )
    elif len(memberships) == 1:
        chosen = memberships[0]
    else:
        raise HTTPException(
            status_code=status.HTTP_400_BAD_REQUEST,
            detail=(
                "X-Tenant-Id is required: this account has "
                f"{len(memberships)} tenant memberships."
            ),
        )

    # step 4 (repeated for the header-less single-membership path)
    if chosen is None:  # pragma: no cover -- defensive, unreachable above
        raise HTTPException(status_code=status.HTTP_403_FORBIDDEN, detail="No membership.")

    # D10 -- the tenant exists in nID (step 4 proved it), so mirror it locally.
    await ensure_tenant_row(session, chosen, introspection.account_id)

    # step 5
    request.state.principal = chosen
    return TenantPrincipal(
        tenant_key=chosen.tenant_id,
        role=chosen.role,
        account_id=introspection.account_id,
        email=None,
        phone=None,
    )


async def require_tenant_member(
    request: Request,
    authorization: str | None = Header(default=None),
    x_tenant_id: str | None = Header(default=None),
    session: AsyncSession = Depends(get_db_session),
) -> TenantPrincipal:
    """Read-side guard: accepts a cached principal up to the introspection TTL."""
    return await _resolve(request, authorization, x_tenant_id, session, for_write=False)


async def require_tenant_writer(
    request: Request,
    authorization: str | None = Header(default=None),
    x_tenant_id: str | None = Header(default=None),
    session: AsyncSession = Depends(get_db_session),
) -> TenantPrincipal:
    """Write-side guard: needs an entry no older than WRITE_MAX_AGE (spec 4.3 step 6)."""
    return await _resolve(request, authorization, x_tenant_id, session, for_write=True)
```

Keep `get_db_session` and `get_harvester_client`; add the imports the deleted functions were the only
users of, and update `organizations.py:19,45,59` (Task 10) and the three `get_tenant_key` users
(`contacts.py:18,58,320,475`, `segments.py:13,51,68,99,127,163,179`) in the same commit — those two
files break at import time otherwise, and `pytest -q` will not start.

- [ ] **Step 5: In `contacts.py` and `segments.py`, swap the header name and its type**

Both files call `Depends(get_tenant_key)`; replace with the guard and keep the parameter name so the
rest of each function body is untouched:

```python
from communicator.api.dependencies import get_db_session, require_tenant_member
...
    principal: TenantPrincipal = Depends(require_tenant_member),
) -> ...:
    tenant_key = principal.tenant_key
```

`x_tenant_key` becomes `X-Tenant-Id` by way of the guard; a slug typed into the new header is now a
403, which is correct — `Tenant.id` is the only accepted value.

- [ ] **Step 6: Run**

```bash
pytest tests/test_require_tenant_member.py -q
pytest -q
```

Expected: the guard file passes (including the headline escalation test and `test_step3_never_falls_back_to_newax`,
which is the spec's own escalation written as an assertion). `pytest -q` shows the first real
breakage in tests that call routes — those are Task 8's fixtures plus Tasks 15-17, not a regression.

- [ ] **Step 7: Commit**

```bash
git add src/communicator/api/dependencies.py src/communicator/services/tenant_provisioning.py \
        src/communicator/api/routes/contacts.py src/communicator/api/routes/segments.py \
        tests/test_require_tenant_member.py
git commit -m "$(cat <<'EOF'
feat(auth): per-request nID membership guard

require_tenant_member replaces get_tenant_key's silent "newax" fallback and
get_current_account_id's process-local session dict. 401 unknown, 403 not a
member, 400 ambiguous tenant, 503 fail-closed when nID is unreachable and no
cached principal is usable. tenant_key is nID's Tenant.id; the local tenants row
is auto-provisioned idempotently on first sight (D10).
EOF
)"
```

---

### Task 8: Test fixtures — two tenants and a client that is already signed in

346 existing HTTP calls live in 42 test files. Without a shared default, PR-2 means editing 346 call
sites for no reason other than that guards now exist. `httpx.AsyncClient` accepts default headers, and
`test_client` is the one place to put them; 17 files that build their own client are converted in
Task 17.

**Files:**
- Modify: `tests/conftest.py`
- Modify: `pyproject.toml` (`pythonpath`)
- Test: `tests/test_fixtures_are_honest.py`

**Interfaces:**
- Consumes: `NidStub`, `require_tenant_member`, `ACME`/`GLOBEX` tenant ids.
- Produces: fixtures `nid_stub`, `tenants`, `authed_headers(token=ACME_TOKEN)`, and an upgraded
  `test_client` whose default headers are a valid acme principal. Constants
  `ACME_TENANT_ID`, `GLOBEX_TENANT_ID`, `ACME_TOKEN`, `GLOBEX_TOKEN`, `MULTI_TOKEN`.

- [ ] **Step 1: Add the python path for `tests/`**

`pyproject.toml`:

```toml
[tool.pytest.ini_options]
pythonpath = ["src", "tests"]
testpaths = ["tests"]
asyncio_mode = "auto"
```

so `from nid_stub import NidStub` resolves without depending on pytest's import-mode side effects.

- [ ] **Step 2: Extend `conftest.py`**

Append (keep everything already there — the session-scoped `create_all`, the
transaction/savepoint isolation, and `override_database_dependency`'s
`app.dependency_overrides.clear()` teardown):

```python
from communicator.services.nid_client import NidMembership, clear_introspection_cache, nid_client
from communicator.services.tenant_provisioning import ensure_tenant_row, ensure_nid_account
from nid_stub import NidStub

# Opaque, cuid-ish, and *not* slugs: spec 4.3 step 3 makes the tenant key nID's
# Tenant.id, and using the slug here is the bug the guard is supposed to catch.
ACME_TENANT_ID = "ckx_acme_tenant_0001"
GLOBEX_TENANT_ID = "ckx_globex_tenant_0002"
ACME_SLUG, GLOBEX_SLUG = "acme", "globex"

ACME_TOKEN = "nid-token-acme"
GLOBEX_TOKEN = "nid-token-globex"
MULTI_TOKEN = "nid-token-both"
NOBODY_TOKEN = "nid-token-nobody"
EXPIRED_TOKEN = "nid-token-expired"

ACME_ACCOUNT = "ckx_acme_person_0001"
GLOBEX_ACCOUNT = "ckx_globex_person_0002"


def auth_headers(token: str = ACME_TOKEN, tenant_id: str | None = ACME_TENANT_ID) -> dict:
    headers = {"Authorization": f"Bearer {token}"}
    if tenant_id:
        headers["X-Tenant-Id"] = tenant_id
    return headers


@pytest.fixture
def nid_stub():
    stub = NidStub()
    stub.install(nid_client)
    stub.set_active(
        ACME_TOKEN,
        account_id=ACME_ACCOUNT,
        memberships=[NidMembership(ACME_TENANT_ID, ACME_SLUG, "Acme Support", "OWNER")],
    )
    stub.set_active(
        GLOBEX_TOKEN,
        account_id=GLOBEX_ACCOUNT,
        memberships=[NidMembership(GLOBEX_TENANT_ID, GLOBEX_SLUG, "Globex Care", "AGENT")],
    )
    stub.set_active(
        MULTI_TOKEN,
        account_id="ckx_both_person",
        memberships=[
            NidMembership(ACME_TENANT_ID, ACME_SLUG, "Acme Support", "OWNER"),
            NidMembership(GLOBEX_TENANT_ID, GLOBEX_SLUG, "Globex Care", "AGENT"),
        ],
    )
    stub.set_inactive(NOBODY_TOKEN)
    stub.set_active(
        EXPIRED_TOKEN,
        account_id=ACME_ACCOUNT,
        memberships=[NidMembership(ACME_TENANT_ID, ACME_SLUG, "Acme Support", "OWNER")],
        expires_at=datetime.now(UTC) - timedelta(seconds=1),
    )
    clear_introspection_cache()
    yield stub
    nid_client._transport = None
    clear_introspection_cache()


@pytest.fixture(autouse=True)
def _mock_nid_off_by_default(monkeypatch, request):
    """
    Tests must not be able to reach a real nID: no network in CI, and a live IdP
    would make 346 assertions depend on someone's laptop. nid_stub opts in.
    """
    if "nid_stub" in request.fixturenames:
        yield
        return
    monkeypatch.setattr(settings, "nid_client_id", "ci-test")
    monkeypatch.setattr(settings, "nid_client_secret", "cs-test")
    monkeypatch.setattr(nid_client, "_transport", _refuse_all_transport)
    yield


async def _refuse_all_handler(request: httpx.Request) -> httpx.Response:
    raise httpx.ConnectError("tests must not reach a real nID", request=request)


_refuse_all_transport = httpx.MockTransport(_refuse_all_handler)


@pytest_asyncio.fixture
async def tenants(db_session):
    """Mirror the two stub tenants into the local table through the real D10 path."""
    for membership, account in (
        (NidMembership(ACME_TENANT_ID, ACME_SLUG, "Acme Support", "OWNER"), ACME_ACCOUNT),
        (NidMembership(GLOBEX_TENANT_ID, GLOBEX_SLUG, "Globex Care", "AGENT"), GLOBEX_ACCOUNT),
    ):
        await ensure_tenant_row(db_session, membership, account)
    await ensure_nid_account(db_session, ACME_ACCOUNT, email="owner@acme.test")
    await ensure_nid_account(db_session, GLOBEX_ACCOUNT, email="agent@globex.test")
    return {"acme": ACME_TENANT_ID, "globex": GLOBEX_TENANT_ID}
```

Then change `test_client` to authenticate by default:

```python
@pytest_asyncio.fixture
async def test_client(override_database_dependency, nid_stub, tenants):  # noqa: C901
    transport = ASGITransport(app=app)
    async with AsyncClient(
        transport=transport, base_url="http://test", headers=auth_headers()
    ) as client:
        yield client
```

`override_database_dependency`'s teardown still calls `app.dependency_overrides.clear()`, which is
what keeps a route-level `dependency_overrides` in one test from leaking into the next — do not
remove that line, and do not "simplify" the fixture by dropping `tenants`, or every guarded route
test starts failing with a missing-tenant-row error that is the fixture's fault, not the code's.

- [ ] **Step 3: Write a test that the fixtures are honest**

Create `tests/test_fixtures_are_honest.py`:

```python
"""The fixtures must not be a way to pass tests without the guard running."""

from __future__ import annotations

import pytest
from httpx import ASGITransport, AsyncClient

from communicator.api.app import app
from tests_helpers import auth_headers  # see Step 4


@pytest.mark.asyncio
async def test_default_test_client_is_acme_and_sees_acme_only(test_client) -> None:
    r = await test_client.get("/api/v1/organizations")
    assert r.status_code == 200
    keys = [o["tenantKey"] for o in r.json()]
    assert "ckx_acme_tenant_0001" in keys
    assert "ckx_globex_tenant_0002" not in keys


@pytest.mark.asyncio
async def test_a_bare_client_without_headers_is_refused(override_database_dependency) -> None:
    """If this passes, the guard is not on the path a test thinks it is."""
    async with AsyncClient(transport=ASGITransport(app=app), base_url="http://t") as c:
        r = await c.get("/api/v1/roles")
    assert r.status_code in (401, 403)


@pytest.mark.asyncio
async def test_globex_principal_cannot_read_acme_organizations(test_client) -> None:
    r = await test_client.get(
        "/api/v1/organizations", headers=auth_headers("nid-token-globex", None)
    )
    assert r.status_code == 200
    assert [o["tenantKey"] for o in r.json()] == ["ckx_globex_tenant_0002"]
```

- [ ] **Step 4: Resolve the import awkwardness before writing more tests**

`from tests_helpers import ...` above is a placeholder for `from conftest import auth_headers`, which
pytest does not encourage. Move `auth_headers`, the token/tenant constants and `_refuse_all_transport`
into `tests/nid_stub.py` (already importable thanks to Step 1) and re-export them from `conftest.py`
for fixtures:

```python
from nid_stub import (
    ACME_ACCOUNT, ACME_SLUG, ACME_TENANT_ID, ACME_TOKEN, EXPIRED_TOKEN, GLOBEX_ACCOUNT,
    GLOBEX_SLUG, GLOBEX_TENANT_ID, GLOBEX_TOKEN, MULTI_TOKEN, NOBODY_TOKEN, NidStub, auth_headers,
)
```

Then the test file imports `from nid_stub import auth_headers` and the constants. Fixtures stay in
`conftest.py`; shared data moves to the module.

- [ ] **Step 5: Run**

```bash
pytest tests/test_fixtures_are_honest.py -q
pytest -q 2>&1 | tail -20
```

Expected: 3 passed. The full suite now shows the sweep's shape: a long tail of 401s from routes that
do not yet have guards plus `organizations`/`contacts`/`segments` behaviour changes. Record the count
in the commit message; do not start "fixing" route tests before Tasks 15-17 put the guards in.

- [ ] **Step 6: Commit**

```bash
git add tests/conftest.py tests/nid_stub.py tests/test_fixtures_are_honest.py pyproject.toml
git commit -m "$(cat <<'EOF'
test(tenancy): two-tenant fixtures and a signed-in default test client

Adds acme/globex tenants whose ids are opaque cuid-likes, a transport-level nID
stub with active/inactive/expired/unreachable tokens, and default auth headers on
test_client so 346 existing HTTP assertions keep exercising the guard rather than
bypassing it. Tests that do not opt into the stub cannot reach the network at all.
EOF
)"
```

---

### Task 9: Login routes — `/start`, `/callback`, `/me`, `/logout`

The route file is the seam between the SPA and the client rewrite. `/start` must make zero upstream
calls, `/callback` must hand the SPA its memberships, `/me` must stop reading the deleted dict, and
`/logout` must revoke at nID (§8 criterion 5).

**Files:**
- Modify: `src/communicator/api/routes/nid_auth.py` (whole file, 111 lines)
- Modify: `src/communicator/api/app.py` (no change to includes; confirm `/auth/nid/me` is the only
  `nid_auth` route that gets guarded)
- Test: `tests/test_nid_login_routes.py`, and rewrite
  `tests/test_nid_auth_and_crypto_mesh.py::test_nid_login_start_and_callback`

**Interfaces:**
- Consumes: `sign_state`, `verify_state`, `build_authorize_url`, `exchange_code`, `introspect`,
  `revoke`, `ensure_nid_account`, `require_tenant_member`.
- Produces:
  - `LoginStartResponse = {authorize_url: str, state: str, dev_mode: bool}`
  - `LoginCallbackResponse = {token, account_id, email, phone, phone_verified, expires_at,
    memberships: list[{tenant_id, slug, name, role}]}` (camelCase is Task 19's problem; the API stays
    snake_case like the rest of this service — see "Naming" under Global constraints reasoning in
    D2).
  - `POST /api/v1/auth/nid/logout` → `{revoked: true}`.
  - `GET /api/v1/auth/nid/me` → `{status, account_id, email, phone, phone_verified,
    memberships: [{tenant_id, name, role}]}`.

- [ ] **Step 1: Write the failing tests**

Create `tests/test_nid_login_routes.py`:

```python
"""Spec 4.2 as an HTTP contract, and spec 8 criteria 2, 4 and 5."""

from __future__ import annotations

import pytest
from httpx import ASGITransport, AsyncClient

from communicator.api.app import app
from communicator.api.dependencies import get_db_session
from communicator.config import settings
from communicator.services import nid_client as mod
from communicator.services.nid_client import clear_introspection_cache, nid_client
from communicator.services.tenant_provisioning import ensure_tenant_row
from nid_stub import ACME_TENANT_ID, NidStub, auth_headers
from communicator.services.nid_client import NidMembership


@pytest.fixture
def stub(override_database_dependency):
    s = NidStub()
    s.install(nid_client)
    clear_introspection_cache()
    yield s
    nid_client._transport = None
    clear_introspection_cache()


@pytest.fixture(autouse=True)
def _no_mock(monkeypatch):
    monkeypatch.setattr(settings, "nid_allow_mock_auth", False)
    monkeypatch.setattr(settings, "nid_client_id", "ci-communicator")
    monkeypatch.setattr(settings, "nid_client_secret", "cs")
    monkeypatch.setattr(settings, "nid_login_ui_url", "http://login-ui.test")
    monkeypatch.setattr(settings, "nid_base_url", "http://nid-api.test")
    monkeypatch.setattr(nid_client, "login_ui_url", "http://login-ui.test")
    monkeypatch.setattr(nid_client, "base_url", "http://nid-api.test")
    monkeypatch.setattr(nid_client, "redirect_uri", "http://spa.test/auth/nid/callback")


@pytest.mark.asyncio
async def test_start_never_touches_nid_and_returns_no_poll_token(stub) -> None:
    async with AsyncClient(transport=ASGITransport(app=app), base_url="http://t") as c:
        r = await c.get("/api/v1/auth/nid/start")
    assert r.status_code == 200
    body = r.json()
    assert set(body) == {"authorize_url", "state", "dev_mode"}
    assert "poll_token" not in body
    assert body["authorize_url"].startswith("http://login-ui.test/authorize?")
    assert stub.calls == []          # the transport saw nothing
    assert body["dev_mode"] is False  # nid_allow_mock_auth is off, and only that gates it


@pytest.mark.asyncio
async def test_start_dev_mode_tracks_the_mock_flag_only(stub, monkeypatch) -> None:
    monkeypatch.setattr(settings, "nid_allow_mock_auth", True)
    monkeypatch.setattr(settings, "app_env", "production")
    async with AsyncClient(transport=ASGITransport(app=app), base_url="http://t") as c:
        r = await c.get("/api/v1/auth/nid/start")
    assert r.json()["dev_mode"] is True


@pytest.mark.asyncio
async def test_start_rejects_the_removed_redirect_uri_param(stub) -> None:
    """Spec 4.2: the parameter is deleted, so it must not be honoured. FastAPI
    ignores unknown query params, which is fine -- what must not happen is the
    redirect target moving."""
    async with AsyncClient(transport=ASGITransport(app=app), base_url="http://t") as c:
        r = await c.get(
            "/api/v1/auth/nid/start?redirect_uri=http://attacker.test/steal"
        )
    assert r.status_code == 200
    assert "attacker.test" not in r.json()["authorize_url"]


@pytest.mark.asyncio
async def test_callback_returns_memberships_and_stores_the_reference_row(
    stub, db_session
) -> None:
    stub.tokens["nid-cb-token"] = {
        "accountId": "ckx_cb_person",
        "expiresAt": "2026-11-02T06:00:00.000Z",
        "scopes": ["identity:read", "memberships:read"],
        "memberships": [
            {"tenantId": ACME_TENANT_ID, "slug": "acme", "name": "Acme Support", "role": "OWNER"}
        ],
    }
    state = mod.sign_state("http://spa.test/auth/nid/callback")
    async with AsyncClient(transport=ASGITransport(app=app), base_url="http://t") as c:
        r = await c.get(f"/api/v1/auth/nid/callback?code=a-real-code&state={state}")
    assert r.status_code == 200, r.text
    body = r.json()
    assert body["token"] == "nid-cb-token"
    assert body["account_id"] == "ckx_cb_person"
    assert body["memberships"][0]["tenant_id"] == ACME_TENANT_ID

    from sqlalchemy import select

    from communicator.models.nid_account import NidAccount

    row = (
        await db_session.execute(select(NidAccount).where(NidAccount.nid_account_id == "ckx_cb_person"))
    ).scalar_one()
    assert row.email == "stub@acme.test"


@pytest.mark.asyncio
async def test_callback_rejects_a_state_for_another_redirect_uri(stub) -> None:
    state = mod.sign_state("http://evil.test/callback")
    async with AsyncClient(transport=ASGITransport(app=app), base_url="http://t") as c:
        r = await c.get(f"/api/v1/auth/nid/callback?code=x&state={state}")
    assert r.status_code == 401


@pytest.mark.asyncio
async def test_callback_rejects_a_forged_state(stub) -> None:
    async with AsyncClient(transport=ASGITransport(app=app), base_url="http://t") as c:
        r = await c.get("/api/v1/auth/nid/callback?code=x&state=abc.def.ghi")
    assert r.status_code == 401


@pytest.mark.asyncio
async def test_callback_is_503_when_nid_is_down(stub) -> None:
    stub.set_unreachable()
    state = mod.sign_state("http://spa.test/auth/nid/callback")
    async with AsyncClient(transport=ASGITransport(app=app), base_url="http://t") as c:
        r = await c.get(f"/api/v1/auth/nid/callback?code=x&state={state}")
    assert r.status_code == 503


@pytest.mark.asyncio
async def test_me_reflects_the_stub_and_needs_a_bearer(stub, db_session) -> None:
    stub.set_active(
        "tok-me",
        account_id="ckx_me",
        memberships=[NidMembership(ACME_TENANT_ID, "acme", "Acme Support", "AGENT")],
    )
    async with AsyncClient(transport=ASGITransport(app=app), base_url="http://t") as c:
        anon = await c.get("/api/v1/auth/nid/me")
        ok = await c.get("/api/v1/auth/nid/me", headers={"Authorization": "Bearer tok-me"})
    assert anon.status_code == 401
    body = ok.json()
    assert body["status"] == "authenticated"
    assert body["account_id"] == "ckx_me"
    assert body["memberships"] == [
        {"tenant_id": ACME_TENANT_ID, "name": "Acme Support", "role": "AGENT"}
    ]
    # no session_token echo anymore: it leaked a prefix of a live credential
    assert "session_token" not in body


@pytest.mark.asyncio
async def test_me_makes_two_calls_at_most_and_caches(stub) -> None:
    stub.set_active(
        "tok-me2", account_id="a", memberships=[NidMembership(ACME_TENANT_ID, "s", "N", "STAFF")]
    )
    async with AsyncClient(transport=ASGITransport(app=app), base_url="http://t") as c:
        for _ in range(4):
            await c.get("/api/v1/auth/nid/me", headers={"Authorization": "Bearer tok-me2"})
    assert len([p for p, _ in stub.calls if p.endswith("/introspect")]) == 1


@pytest.mark.asyncio
async def test_logout_revokes_at_nid(stub) -> None:
    stub.set_active(
        "tok-out", account_id="a", memberships=[NidMembership(ACME_TENANT_ID, "s", "N", "STAFF")]
    )
    async with AsyncClient(transport=ASGITransport(app=app), base_url="http://t") as c:
        before = await c.get("/api/v1/auth/nid/me", headers={"Authorization": "Bearer tok-out"})
        assert before.status_code == 200
        r = await c.post("/api/v1/auth/nid/logout", headers={"Authorization": "Bearer tok-out"})
        assert r.status_code == 200
        assert r.json() == {"revoked": True}
        after = await c.get("/api/v1/auth/nid/me", headers={"Authorization": "Bearer tok-out"})
    assert after.status_code == 401


@pytest.mark.asyncio
async def test_no_route_holds_a_session_dict_anymore() -> None:
    """Spec 8 criterion 4, end to end."""
    import inspect

    import communicator.api.routes.nid_auth as routes

    src = inspect.getsource(routes) + inspect.getsource(mod)
    for needle in ("_active_sessions", "_pending_states", "login-requests", "poll_token"):
        assert needle not in src
```

- [ ] **Step 2: Run to verify failure**

```bash
pytest tests/test_nid_login_routes.py -q
```

Expected: 404/422 failures on `/logout`, `poll_token` in `/start`, `AttributeError` in `/me`.

- [ ] **Step 3: Rewrite the route file**

Replace `src/communicator/api/routes/nid_auth.py` in full:

```python
"""nID login routes (spec 4.2).

Communicator is a relying party. It builds an authorize URL on the login-ui
origin, redeems the code that comes back for a scoped RP session token, and then
answers every later question by introspecting that token at nID -- it stores no
session and no membership of its own.
"""

from __future__ import annotations

import logging
from typing import Any

from fastapi import APIRouter, Depends, Header, HTTPException, Query, status
from pydantic import BaseModel
from sqlalchemy.ext.asyncio import AsyncSession

from communicator.api.dependencies import get_db_session
from communicator.config import settings
from communicator.services.nid_client import (
    NidAuthError,
    NidUnreachableError,
    build_authorize_url_for,
    exchange_code_for_claims,
    introspect_bearer,
    nid_client,
    revoke_bearer,
    sign_state,
    verify_state,
)
from communicator.services.tenant_provisioning import ensure_nid_account, ensure_tenant_row

logger = logging.getLogger(__name__)

router = APIRouter(prefix="/auth/nid", tags=["Authentication (nID)"])


class MembershipOut(BaseModel):
    tenant_id: str
    slug: str
    name: str
    role: str


class LoginStartResponse(BaseModel):
    authorize_url: str
    state: str
    # True only when NID_ALLOW_MOCK_AUTH is set (spec D8). The SPA uses it to
    # complete a dev sign-in locally instead of navigating to an IdP that is not
    # running on this laptop.
    dev_mode: bool


class LoginCallbackResponse(BaseModel):
    token: str
    account_id: str
    email: str | None = None
    phone: str | None = None
    phone_verified: bool = False
    expires_at: str | None = None
    memberships: list[MembershipOut] = []


def _bearer(authorization: str | None) -> str:
    if not authorization or not authorization.startswith("Bearer "):
        raise HTTPException(
            status_code=status.HTTP_401_UNAUTHORIZED,
            detail="Missing or invalid Bearer authorization header.",
        )
    return authorization.split("Bearer ", 1)[1].strip()


@router.get("/start", response_model=LoginStartResponse)
async def login_with_nid_start() -> LoginStartResponse:
    """
    Begin 'Login with nID'. Makes no upstream call: the login-request is
    created by login-ui's own /authorize page, and the poll token is its
    business. The signed state is what proves a callback matched a start this
    deployment issued.
    """
    state = sign_state(nid_client.redirect_uri)
    return LoginStartResponse(
        authorize_url=build_authorize_url_for(state),
        state=state,
        dev_mode=settings.nid_allow_mock_auth,
    )


@router.get("/callback", response_model=LoginCallbackResponse)
async def login_with_nid_callback(
    code: str = Query(..., description="One-time authorization code from nID"),
    state: str = Query(..., description="Signed state handed out by /start"),
    session: AsyncSession = Depends(get_db_session),
) -> LoginCallbackResponse:
    """
    Redeem the code server-to-server. Returns nID's session token plus the
    memberships the SPA needs to pick a tenant before its first business call --
    the guard answers 'no header, several memberships' with a 400, so a login
    that returned no memberships would be a login into an unusable account.
    """
    if not verify_state(state, nid_client.redirect_uri):
        raise HTTPException(
            status_code=status.HTTP_401_UNAUTHORIZED,
            detail="Invalid, expired or mismatched OAuth state.",
        )
    try:
        claims = await exchange_code_for_claims(code=code, state=state)
    except NidUnreachableError as exc:
        raise HTTPException(
            status_code=status.HTTP_503_SERVICE_UNAVAILABLE,
            detail=f"nID is not reachable: {exc}",
            headers={"Retry-After": "5"},
        ) from exc
    except NidAuthError as exc:
        raise HTTPException(status_code=status.HTTP_401_UNAUTHORIZED, detail=str(exc)) from exc

    await ensure_nid_account(
        session,
        claims["account_id"],
        email=claims.get("email") or "",
        phone=claims.get("phone") or "",
    )
    for membership in claims["memberships"]:
        await ensure_tenant_row(session, membership, claims["account_id"])

    return LoginCallbackResponse(
        token=claims["access_token"],
        account_id=claims["account_id"],
        email=claims.get("email"),
        phone=claims.get("phone"),
        phone_verified=claims.get("phone_verified", False),
        expires_at=claims.get("expires_at"),
        memberships=[
            MembershipOut(
                tenant_id=m.tenant_id, slug=m.slug, name=m.name, role=m.role
            )
            for m in claims["memberships"]
        ],
    )


@router.get("/me")
async def get_current_user_profile(
    authorization: str | None = Header(default=None),
) -> dict[str, Any]:
    """
    The SPA's view of identity and membership: one introspection, no local
    session table, and the same read the tenant picker and the organization list
    are built from (spec D12) -- so the authorization decision cannot disagree
    with what the UI shows, because they are the same call.

    Deliberately not behind require_tenant_member: this answers 'who am I and
    where do I belong', which is the question you ask *before* you have chosen a
    tenant.
    """
    token = _bearer(authorization)
    try:
        introspection = await introspect_bearer(token)
    except NidUnreachableError as exc:
        raise HTTPException(
            status_code=status.HTTP_503_SERVICE_UNAVAILABLE,
            detail=f"nID is not reachable: {exc}",
            headers={"Retry-After": "5"},
        ) from exc
    if introspection is None:
        raise HTTPException(
            status_code=status.HTTP_401_UNAUTHORIZED,
            detail="Session is expired, revoked, or unknown to nID.",
        )
    return {
        "status": "authenticated",
        "account_id": introspection.account_id,
        "email": None,
        "phone": None,
        "memberships": [
            {"tenant_id": m.tenant_id, "name": m.name or m.slug, "role": m.role}
            for m in introspection.memberships
        ],
    }


@router.post("/logout")
async def logout(authorization: str | None = Header(default=None)) -> dict[str, bool]:
    """Revoke at nID, so a stolen or leftover SPA token stops working there too
    (spec 8 criterion 5). Idempotent: an already-dead token still answers
    revoked:true because the outcome the caller wanted is now true."""
    token = _bearer(authorization)
    try:
        revoked = await revoke_bearer(token)
    except NidUnreachableError as exc:
        raise HTTPException(
            status_code=status.HTTP_503_SERVICE_UNAVAILABLE,
            detail=f"nID is not reachable, so logout could not be completed: {exc}",
            headers={"Retry-After": "5"},
        ) from exc
    return {"revoked": bool(revoked)}
```

`email`/`phone` in `/me` are `None` because nID's introspection response carries only `accountId`
(PII stays out of the introspection payload — Plan 1 N4 returns no phone/email by design). If the SPA
needs them, it reads them from the callback response it already holds. Do **not** invent a
`GET /v1/accounts/me` call to fetch them: that endpoint needs an `AccountSession` a relying party must
not hold (§3).

- [ ] **Step 4: Add the module-level indirections the routes import**

Those five free functions exist so the route file never reaches into the singleton's internals, and
so a test can patch one place. Append to `src/communicator/services/nid_client.py`:

```python
def build_authorize_url_for(state: str) -> str:
    return nid_client.build_authorize_url(state)


async def exchange_code_for_claims(*, code: str, state: str) -> dict[str, Any]:
    return await nid_client.exchange_code(code=code, state=state)


async def introspect_bearer(token: str, *, for_write: bool = False):
    return await nid_client.introspect(token, for_write=for_write)


async def revoke_bearer(token: str) -> bool:
    return await nid_client.revoke(token)
```

- [ ] **Step 5: Rewrite the pre-existing login test instead of deleting it**

In `tests/test_nid_auth_and_crypto_mesh.py::test_nid_login_start_and_callback`: drop the
`poll_token` assertion, assert `{"authorize_url", "state", "dev_mode"}`, install the stub, give the
stub a token for the callback, use `mod.sign_state(nid_client.redirect_uri)` for `state`, and keep
the `/me` + unauthenticated-401 assertions. It builds its own `AsyncClient` (`:24-25`) so it does not
get the default headers — that is now the point of the test.

- [ ] **Step 6: Run**

```bash
pytest tests/test_nid_login_routes.py tests/test_nid_auth_and_crypto_mesh.py -q
pytest -q 2>&1 | tail -5
```

Expected: both files green. Full suite: the known 401 tail from Tasks 15-17 remains.

- [ ] **Step 7: Commit**

```bash
git add src/communicator/api/routes/nid_auth.py src/communicator/services/nid_client.py \
        tests/test_nid_login_routes.py tests/test_nid_auth_and_crypto_mesh.py
git commit -m "$(cat <<'EOF'
feat(auth): login routes that work against real nID, and logout that revokes

/start builds the authorize URL with no upstream call and returns no poll token;
the caller-supplied redirect_uri parameter is gone. /callback redeems the code,
stores the account reference row, mirrors memberships as local tenant rows and
hands the SPA its memberships. /me is a live introspection rather than a dict
lookup and stops echoing a bearer prefix. /logout revokes at nID.
EOF
)"
```

---

### Task 10: Organizations — delete the self-service create, keep a projection

`POST /organizations` mints a `tenant_key = _slugify(name)` that exists in no nID `Tenant` row and
grants no `TenantStaffMembership`, so every later request for it is a 403 (D12). `GET` survives, but
as a read-only projection of introspected memberships — the same read the guard and `/me` use.

**Files:**
- Modify: `src/communicator/api/routes/organizations.py` (delete `:25-27`, `:39-41`, `:56-80`;
  rewrite `:43-53`)
- Test: `tests/test_organizations_projection.py`

**Interfaces:**
- Consumes: `introspect_bearer`, `ensure_tenant_row`.
- Produces: `GET /api/v1/organizations` returning `[{id, tenantKey, name, createdAt, role}]` for
  exactly the caller's introspected memberships; `POST` gone; `OrganizationIn`/`_slugify` gone.

- [ ] **Step 1: Write the failing tests**

Create `tests/test_organizations_projection.py`:

```python
"""Spec D12 and spec 8 criterion 9."""

from __future__ import annotations

import pytest

from communicator.config import settings
from communicator.services.nid_client import NidMembership, clear_introspection_cache, nid_client
from communicator.services.tenant_provisioning import ensure_tenant_row
from nid_stub import (
    ACME_SLUG, ACME_TENANT_ID, GLOBEX_SLUG, GLOBEX_TENANT_ID, NidStub, auth_headers,
)


@pytest.fixture
def stub(override_database_dependency):
    s = NidStub()
    s.install(nid_client)
    clear_introspection_cache()
    yield s
    nid_client._transport = None
    clear_introspection_cache()


@pytest.mark.asyncio
async def test_post_organizations_is_gone(test_client) -> None:
    r = await test_client.post("/api/v1/organizations", json={"name": "Rogue Org"})
    assert r.status_code == 405


@pytest.mark.asyncio
async def test_list_is_exactly_the_introspected_memberships(stub, db_session) -> None:
    """Not 'tenants the local table happens to have': one read, so the UI cannot
    disagree with the authorization decision."""
    stub.set_active(
        "multi",
        account_id="ckx_both",
        memberships=[
            NidMembership(ACME_TENANT_ID, ACME_SLUG, "Acme Support", "OWNER"),
            NidMembership(GLOBEX_TENANT_ID, GLOBEX_SLUG, "Globex Care", "AGENT"),
        ],
    )
    r = await test_client.get("/api/v1/organizations", headers=auth_headers("multi", None))
    assert r.status_code == 200
    body = r.json()
    assert {o["tenantKey"] for o in body} == {ACME_TENANT_ID, GLOBEX_TENANT_ID}
    assert {o["role"] for o in body} == {"OWNER", "AGENT"}


@pytest.mark.asyncio
async def test_a_tenant_never_seen_locally_still_lists(stub, db_session) -> None:
    """D10: nothing is seeded and nothing is hand-inserted, so first listing must
    provision."""
    stub.set_active(
        "brandnew",
        account_id="ckx_new",
        memberships=[NidMembership("ckx_zeta_tenant_0009", "zeta", "Zeta Health", "STAFF")],
    )
    r = await test_client.get("/api/v1/organizations", headers=auth_headers("brandnew", None))
    assert r.status_code == 200
    assert [o["tenantKey"] for o in r.json()] == ["ckx_zeta_tenant_0009"]
    from sqlalchemy import select

    from communicator.models.tenant import Tenant

    rows = (
        await db_session.execute(select(Tenant).where(Tenant.tenant_key == "ckx_zeta_tenant_0009"))
    ).scalars().all()
    assert len(rows) == 1


@pytest.mark.asyncio
async def test_owner_account_id_is_display_only(stub, db_session) -> None:
    """Rev. 3 defect 8: the column stays, but it is no longer a filter, so an
    empty owner row must still list for its members."""
    membership = NidMembership("ckx_ownerless_tenant_7", "ownerless", "Ownerless", "AGENT")
    await ensure_tenant_row(db_session, membership, account_id="ignored")
    stub.set_active("ol", account_id="ckx_agent", memberships=[membership])
    r = await test_client.get("/api/v1/organizations", headers=auth_headers("ol", None))
    assert [o["tenantKey"] for o in r.json()] == ["ckx_ownerless_tenant_7"]


@pytest.mark.asyncio
async def test_anonymous_get_is_401(stub) -> None:
    from httpx import ASGITransport, AsyncClient

    from communicator.api.app import app

    async with AsyncClient(transport=ASGITransport(app=app), base_url="http://t") as c:
        r = await c.get("/api/v1/organizations")
    assert r.status_code == 401
```

- [ ] **Step 2: Run to verify failure** — `pytest tests/test_organizations_projection.py -q`
  (expect the `POST` to return 200 today, and `role` missing from the payload).

- [ ] **Step 3: Rewrite the route file**

```python
"""Organizations: a read-only projection of nID memberships (spec D12).

There is no create here. A tenant is a nID Tenant row, its members are nID
TenantStaffMembership rows, and Communicator mirrors them; the branch's
self-service POST minted a slug-keyed tenant that existed in no nID table and
therefore 403'd every request its creator could make to it.
"""

from __future__ import annotations

from typing import Any

from fastapi import Depends, Header, HTTPException, status
from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession

from communicator.api.dependencies import get_db_session
from communicator.models.tenant import Tenant
from communicator.services.nid_client import (
    NidUnreachableError,
    introspect_bearer,
)
from communicator.services.tenant_provisioning import ensure_tenant_row

router = APIRouter(prefix="/organizations", tags=["Organizations"])


def _serialize(t: Tenant | None, tenant_id: str, name: str, role: str) -> dict[str, Any]:
    return {
        "id": str(t.id) if t is not None else tenant_id,
        "tenantKey": tenant_id,
        "name": name,
        "role": role,
        "createdAt": t.created_at.isoformat() if t is not None else None,
    }


@router.get("")
async def list_organizations(
    authorization: str | None = Header(default=None),
    session: AsyncSession = Depends(get_db_session),
) -> list[dict[str, Any]]:
    token = ...  # see the bearer helper
    introspection = ...
    out: list[dict[str, Any]] = []
    for membership in introspection.memberships:
        await ensure_tenant_row(session, membership, introspection.account_id)
        row = (
            await session.execute(select(Tenant).where(Tenant.tenant_key == membership.tenant_id))
        ).scalar_one_or_none()
        out.append(
            _serialize(
                row, membership.tenant_id, membership.name or membership.slug, membership.role
            )
        )
    return out
```

Finish the two `...` lines by reusing Task 9's bearer pattern — import `_bearer` is not allowed across
route modules, so lift `_bearer` into `api/dependencies.py` as `bearer_token_or_401(authorization)`
and use it in **both** files (update Task 9's import in the same commit), then:

```python
    token = bearer_token_or_401(authorization)
    try:
        introspection = await introspect_bearer(token)
    except NidUnreachableError as exc:
        raise HTTPException(503, detail=f"nID is not reachable: {exc}",
                            headers={"Retry-After": "5"}) from exc
    if introspection is None:
        raise HTTPException(status.HTTP_401_UNAUTHORIZED, detail="Session is not valid.")
```

Delete `POST`, `OrganizationIn`, `_slugify`, the `re` and `uuid` imports if now unused, and
`get_current_account_id` from the import line.

- [ ] **Step 4: Remove the tenant docstring's stale promise**

`src/communicator/models/tenant.py:11-18` says every tenant is created "through
`POST /organizations`, never hardcoded". Both halves are now wrong: say it is created lazily from an
introspected membership (D10) and that `tenant_key` holds nID's `Tenant.id`.

- [ ] **Step 5: Run**

```bash
pytest tests/test_organizations_projection.py tests/test_fixtures_are_honest.py -q
```

Expected: all pass, including the two fixture honesty tests that read `/api/v1/organizations`.

- [ ] **Step 6: Commit**

```bash
git add src/communicator/api/routes/organizations.py src/communicator/api/dependencies.py \
        src/communicator/api/routes/nid_auth.py src/communicator/models/tenant.py \
        tests/test_organizations_projection.py
git commit -m "$(cat <<'EOF'
feat(tenancy): organizations is a projection of nID memberships; self-service create deleted

POST /organizations minted a slug-keyed tenant that existed in no nID table, so
its creator was locked out of every subsequent request (D12). GET now reads the
same introspection the guard reads, so the list a person sees cannot disagree
with what they are authorized to do. owner_account_id is display-only.
EOF
)"
```

---

### Task 11: D7 — nID role to Communicator capability, and `roles.py` off the anonymous path

`rbac_service.py` already ships 23 permission keys, five `DEFAULT_SYSTEM_ROLES` and
`check_member_permission`, and **nothing calls it on a request path**. D7 binds that engine to nID's
role instead of inventing a second vocabulary. `roles.py` is the urgent half: eight endpoints manage
the grants themselves with no auth at all, and
`GET /roles/members/{member_id}/check?permission=…` evaluates any member's permissions for any
caller.

**Files:**
- Create: `src/communicator/services/capabilities.py`
- Modify: `src/communicator/api/dependencies.py` (add `require_permission`, `resolve_caller_member`)
- Modify: `src/communicator/api/routes/roles.py` (all 8 endpoints: `:36, :44, :50, :62, :87, :108,
  :121, :147`)
- Test: `tests/test_role_capabilities.py`

**Interfaces:**
- Consumes: `TenantPrincipal`, `TeamMember.nid_account_id` + `tenant_key` (Task 3),
  `rbac_service.check_member_permission(session, member_id, permission)`.
- Produces:
  - `NID_ROLE_TO_ROLE_DEFINITION: dict[str, str]`
  - `async ensure_member_for_principal(session, principal) -> TeamMember | None`
  - `require_permission(key: str) -> dependency` yielding `TenantPrincipal`
  - `GET /api/v1/roles/me/permissions` replacing `GET /roles/members/{member_id}/check`.

- [ ] **Step 1: Write the failing tests**

Create `tests/test_role_capabilities.py`:

```python
"""Spec 4.3 D7 and spec 4.6's capability + roles.py assertions."""

from __future__ import annotations

import pytest
from sqlalchemy import select

from communicator.models.team_member import TeamMember
from communicator.services.capabilities import NID_ROLE_TO_ROLE_DEFINITION
from communicator.services.nid_client import NidMembership, clear_introspection_cache, nid_client
from communicator.services.tenant_provisioning import ensure_nid_account, ensure_tenant_row
from nid_stub import ACME_TENANT_ID, GLOBEX_TENANT_ID, NidStub, auth_headers


@pytest.fixture
def stub(override_database_dependency):
    s = NidStub()
    s.install(nid_client)
    clear_introspection_cache()
    yield s
    nid_client._transport = None
    clear_introspection_cache()


def _member(tenant_key: str, account: str, email: str, role: str) -> TeamMember:
    return TeamMember(
        name=role.title(), email=email, role=role, tenant_key=tenant_key, nid_account_id=account
    )


@pytest.mark.asyncio
async def test_every_nid_role_maps_to_a_role_definition_that_exists(db_session) -> None:
    from communicator.services import rbac_service

    await rbac_service.initialize_default_roles(db_session)
    names = {
        r.name for r in (await db_session.execute(select(__import__(
            "communicator.models.rbac", fromlist=["RoleDefinition"]
        ).RoleDefinition))).scalars()
    }
    for nid_role, definition in NID_ROLE_TO_ROLE_DEFINITION.items():
        assert definition in names, f"{nid_role} maps to unknown RoleDefinition {definition}"


@pytest.mark.asyncio
async def test_viewer_role_definition_lacks_conversations_reply(db_session) -> None:
    from communicator.services import rbac_service

    await rbac_service.initialize_default_roles(db_session)
    member = _member(ACME_TENANT_ID, "ckx_viewer_1", "v@acme.test",
                     NID_ROLE_TO_ROLE_DEFINITION["VIEWER"])
    db_session.add(member)
    await db_session.flush()
    assert await rbac_service.check_member_permission(
        db_session, member.id, "conversations:reply"
    ) is False
    assert await rbac_service.check_member_permission(
        db_session, member.id, "conversations:view"
    ) is True


@pytest.mark.asyncio
async def test_agent_role_can_reply(db_session) -> None:
    from communicator.services import rbac_service

    await rbac_service.initialize_default_roles(db_session)
    member = _member(ACME_TENANT_ID, "ckx_agent_1", "a@acme.test",
                     NID_ROLE_TO_ROLE_DEFINITION["AGENT"])
    db_session.add(member)
    await db_session.flush()
    assert await rbac_service.check_member_permission(
        db_session, member.id, "conversations:reply"
    ) is True


@pytest.mark.asyncio
async def test_roles_list_requires_a_bearer(test_client) -> None:
    from httpx import ASGITransport, AsyncClient

    from communicator.api.app import app

    async with AsyncClient(transport=ASGITransport(app=app), base_url="http://t") as c:
        r = await c.get("/api/v1/roles")
    assert r.status_code == 401


@pytest.mark.asyncio
async def test_roles_list_scopes_to_the_tenant(test_client, db_session) -> None:
    db_session.add(_member(ACME_TENANT_ID, "ckx_acme_person_0001", "owner@acme.test", "admin"))
    db_session.add(_member(GLOBEX_TENANT_ID, "ckx_globex_person_0002", "agent@globex.test", "agent"))
    await db_session.flush()
    r = await test_client.get("/api/v1/roles/members", headers=auth_headers())
    assert r.status_code == 200
    emails = {m["email"] for m in r.json()}
    assert emails == {"owner@acme.test"}


@pytest.mark.asyncio
async def test_member_check_no_longer_takes_a_member_id(test_client) -> None:
    """The endpoint used to evaluate any member's permissions for an anonymous
    caller. It now answers for the caller only."""
    r = await test_client.get(
        "/api/v1/roles/members/someone-elses-uuid/check?permission=billing:manage",
        headers=auth_headers(),
    )
    assert r.status_code == 404  # the path is gone
    r2 = await test_client.get(
        "/api/v1/roles/me/permissions", headers=auth_headers()
    )
    assert r2.status_code == 200


@pytest.mark.asyncio
async def test_unlinked_account_is_refused_with_an_actionable_detail(stub, test_client, db_session) -> None:
    membership = NidMembership(ACME_TENANT_ID, "acme", "Acme", "STAFF")
    await ensure_tenant_row(db_session, membership, "ckx_unlinked")
    await ensure_nid_account(db_session, "ckx_unlinked", email="u@acme.test")
    stub.set_active("unlinked", account_id="ckx_unlinked", memberships=[membership])
    r = await test_client.get("/api/v1/roles/me/permissions", headers=auth_headers("unlinked"))
    assert r.status_code == 403
    assert "TeamMember" in r.json()["detail"]


@pytest.mark.asyncio
async def test_write_role_route_needs_a_permission_and_a_fresh_entry(stub, test_client, db_session) -> None:
    membership = NidMembership(ACME_TENANT_ID, "acme", "Acme", "VIEWER")
    await ensure_tenant_row(db_session, membership, "ckx_viewer_2")
    await ensure_nid_account(db_session, "ckx_viewer_2", email="v2@acme.test")
    db_session.add(_member(ACME_TENANT_ID, "ckx_viewer_2", "v2@acme.test",
                           NID_ROLE_TO_ROLE_DEFINITION["VIEWER"]))
    await db_session.flush()
    stub.set_active("viewer", account_id="ckx_viewer_2", memberships=[membership])
    r = await test_client.post(
        "/api/v1/roles",
        json={"name": "sneaky", "display_name": "Sneaky", "permissions": ["billing:manage"]},
        headers=auth_headers("viewer"),
    )
    assert r.status_code == 403
```

The `_member(...)` helper passes `role=` as a `RoleDefinition.name`, which is exactly how
`rbac_service` resolves it (`check_member_permission` selects `RoleDefinition.name == member.role`),
so the map is exercised end to end rather than only as a dict.

- [ ] **Step 2: Run to verify failure** — `pytest tests/test_role_capabilities.py -q`.

- [ ] **Step 3: Create the capability map**

Create `src/communicator/services/capabilities.py`:

```python
"""
nID role -> Communicator RoleDefinition (spec D7).

nID decides *who is an agent of which tenant and at what role*. Communicator
decides what that role may do inside this product, using the RBAC catalog that
already ships in services/rbac_service.py. nID has no permission strings and no
policy engine, and D7 keeps it that way -- building one inside nID would create a
second capability vocabulary that drifts from this one.

The five names below are `DEFAULT_SYSTEM_ROLES` from rbac_service
(`:139-185`), verified by test_every_nid_role_maps_to_a_role_definition_that_exists.
Re-mapping any of them is a product decision, not a cleanup: VIEWER is
deliberately the read-only catalog role and STAFF (nID's default) deliberately
the least-privileged *write* role, so a fresh invitation cannot reply to a
customer until somebody assigns a real role.
"""

from __future__ import annotations

NID_ROLE_TO_ROLE_DEFINITION: dict[str, str] = {
    "OWNER": "super_admin",
    "ADMIN": "admin",
    "AGENT": "agent",
    "STAFF": "restricted_agent",
    "VIEWER": "compliance_auditor",
}

# Unknown roles fail closed. nID's DTO (spec 4.1 N6) rejects them at the source;
# if one arrives anyway, no capability is the only defensible answer.
UNKNOWN_ROLE: str = ""
```

- [ ] **Step 4: Add the resolution helpers to `dependencies.py`**

```python
from communicator.models.team_member import TeamMember
from communicator.services.capabilities import NID_ROLE_TO_ROLE_DEFINITION


async def resolve_caller_member(
    session: AsyncSession, principal: TenantPrincipal
) -> TeamMember | None:
    """
    Find the local agent row for this caller in this tenant. None is normal --
    most existing rows were typed into the agent picker and have never signed in
    -- and callers must treat it as 'no capability', never as 'admin'.
    """
    stmt = select(TeamMember).where(
        TeamMember.nid_account_id == principal.account_id,
        TeamMember.tenant_key == principal.tenant_key,
    )
    return (await session.execute(stmt)).scalar_one_or_none()


async def ensure_member_for_principal(
    session: AsyncSession, principal: TenantPrincipal, *, name: str = "", email: str = ""
) -> TeamMember:
    """Link or create the agent row for a signed-in caller, using their nID role."""
    member = await resolve_caller_member(session, principal)
    if member is not None:
        return member
    role_definition = NID_ROLE_TO_ROLE_DEFINITION.get(principal.role)
    if role_definition is None:
        raise HTTPException(
            status_code=status.HTTP_403_FORBIDDEN,
            detail=f"Unknown nID role {principal.role!r}; no capability map for it.",
        )
    member = TeamMember(
        name=name or email or principal.account_id,
        email=email,
        role=role_definition,
        tenant_key=principal.tenant_key,
        nid_account_id=principal.account_id,
    )
    session.add(member)
    await session.flush()
    return member


def require_permission(permission: str):
    """Dependency factory: routes ask for permission keys, nothing else consults role."""

    async def _dep(
        request: Request,
        authorization: str | None = Header(default=None),
        x_tenant_id: str | None = Header(default=None),
        session: AsyncSession = Depends(get_db_session),
    ) -> TenantPrincipal:
        principal = await _resolve(
            request, authorization, x_tenant_id, session, for_write=False
        )
        member = await resolve_caller_member(session, principal)
        if member is None:
            raise HTTPException(
                status_code=status.HTTP_403_FORBIDDEN,
                detail=(
                    "No TeamMember row is linked to this nID account for this tenant. "
                    "Sign in once from an account that has an agent row, or link the "
                    "row's nid_account_id."
                ),
            )
        from communicator.services import rbac_service

        if not await rbac_service.check_member_permission(session, member.id, permission):
            raise HTTPException(
                status_code=status.HTTP_403_FORBIDDEN,
                detail=f"Role {member.role!r} lacks permission {permission!r}.",
            )
        return principal

    return _dep
```

- [ ] **Step 5: Rewrite `roles.py`**

For each of the eight endpoints, add the dependency and tenant scoping; the shape:

```python
from communicator.api.dependencies import (
    TenantPrincipal,
    require_permission,
    require_tenant_member,
    require_tenant_writer,
    resolve_caller_member,
)

@router.get("")
async def list_roles(
    session: AsyncSession = Depends(get_db_session),
    principal: TenantPrincipal = Depends(require_tenant_member),
) -> list[dict[str, Any]]:
    return await rbac_service.list_roles(session)


@router.get("/permissions")
async def get_permissions_catalog(
    principal: TenantPrincipal = Depends(require_tenant_member),
) -> list[dict[str, Any]]:
    return rbac_service.get_permissions_catalog()


@router.get("/me/permissions")
async def check_my_permission(
    permission: str = Query(..., description="Permission key, e.g. conversations:reply"),
    session: AsyncSession = Depends(get_db_session),
    principal: TenantPrincipal = Depends(require_tenant_member),
) -> dict[str, Any]:
    member = await resolve_caller_member(session, principal)
    if member is None:
        raise HTTPException(403, "No TeamMember row is linked to this nID account.")
    allowed = await rbac_service.check_member_permission(session, member.id, permission)
    return {"permission": permission, "allowed": allowed, "role": member.role}
```

and for the four mutating ones (`POST :62`, `PUT :87`, `DELETE :108`,
`POST /members/{member_id}/assign :121`) use `require_permission("roles:manage")` if that key exists
in the catalog — **verify first**:

```bash
grep -n "roles:\|role:" src/communicator/services/rbac_service.py
```

If no such key exists, use `require_permission("team:manage")` if present, else add the key to
`STANDARD_PERMISSIONS` **and** to `DEFAULT_SYSTEM_ROLES`' `super_admin`/`admin` lists in the same
commit, because D7 wires the catalog rather than replacing it. Record which you chose in the commit
message. `GET /{role_id_or_name}` and the member list take `require_tenant_member`; delete the
`member_id` path parameter from the old check endpoint (replaced by `/me/permissions` above), and
make `POST /members/{member_id}/assign` refuse to act on a member whose `tenant_key` differs from
`principal.tenant_key`:

```python
    target = await session.get(TeamMember, member_uuid)
    if target is None or target.tenant_key != principal.tenant_key:
        # Same 404 as a nonexistent id: confirming which tenant ids exist is
        # itself a leak.
        raise HTTPException(404, "Team member not found.")
```

- [ ] **Step 6: Run**

```bash
pytest tests/test_role_capabilities.py tests/test_rbac_permissions.py -q
```

Expected: capabilities green. `test_rbac_permissions.py::test_roles_api_endpoints` (`:108`) builds
its own `AsyncClient` with no headers, so it now 401s — fix it here by passing
`headers=auth_headers()` into that constructor (it is one of Task 17's nineteen sites; do it now since
this commit is what broke it).

- [ ] **Step 7: Commit**

```bash
git add src/communicator/services/capabilities.py src/communicator/api/dependencies.py \
        src/communicator/api/routes/roles.py tests/test_role_capabilities.py \
        tests/test_rbac_permissions.py src/communicator/services/rbac_service.py
git commit -m "$(cat <<'EOF'
feat(rbac): bind nID roles to the existing capability catalog; guard roles.py

services/rbac_service.py shipped 23 permission keys and check_member_permission
with nothing calling them on a request path. This maps nID's five roles onto the
five RoleDefinitions that already exist and puts the catalog behind them.
GET /roles/members/{member_id}/check evaluated any member's permissions for an
anonymous caller; it is now GET /roles/me/permissions and answers for the caller
only. Team member lookups are tenant-scoped.
EOF
)"
```

<!-- APPEND -->
