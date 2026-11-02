# Communicator Tenancy PR-1 (branch merge + single Alembic head) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Merge `feat/multi-tenant-automation-campaigns` into the current `newax/finalization` line so one tree carries both the LiveProof/crypto-mesh work and the tenancy/SPA work, with a single Alembic head and the existing test suite still passing unchanged.

**Architecture:** A `--no-ff` merge commit on a new branch cut from `newax/finalization`. Conflicts are resolved in favour of this branch's nID client/routes (spec D2) while taking the feature branch's tenancy models, migrations, SPA hosting, and `api/dependencies.py`. One empty Alembic merge revision reunites the two migration heads that diverged at `b3e5f6a7c8d9`. No endpoint behaviour, auth rule, or header name changes in this PR -- that is PR-2/PR-3.

**Tech Stack:** Python 3.11+ / FastAPI / SQLAlchemy 2.0 async / Alembic (async `env.py`) / pytest + pytest-asyncio (`asyncio_mode = "auto"`) / ruff (line-length 100) / Vite 8 + React 19 + TypeScript (`tsc -b && vite build`) / oxlint / npm only (no pnpm, no yarn) / PostgreSQL 16.

**Spec:** `docs/superpowers/specs/2026-11-01-tenancy-and-nid-identity-design.md` -- decisions D1 (merge, not rebase), D2 (keep this branch's `nid_client.py` / `nid_auth.py`, discard the feature copies, port `dev_mode` later), §5 PR-1 sequencing, §8 gates. Read the spec before starting; this plan argues from it.

**Companion plans:** PR-N1 is [nID Plan 1](file:///C:/Users/nommi/Downloads/scraper/nID/docs/superpowers/plans/2026-11-01-relying-party-sessions-and-introspection.md) and must deploy before **PR-2** can be verified end to end. PR-1 has no dependency on nID and can merge today.

---

## Global Constraints

Every task in this plan inherits these. They are copied from the spec and the repo config, verbatim where the source states them.

- **Merge, never rebase** (spec D1). The 48 feature-branch commits and the 14 local commits both stay in history as-authored; `git rebase`, `git reset --hard` and squash-merges are out.
- **Behaviour-neutral.** No route changes its status code, no dependency gains an auth check, no header is renamed, no column is dropped in this PR. The only intentional behaviour deltas are the three listed in "What PR-1 deliberately changes" at the bottom -- each is a *forced* consequence of the merge, not a design choice made here.
- **This branch's `nid_client.py` and `nid_auth.py` win; the feature branch's copies are discarded** (spec D2). The feature branch's `dev_mode` field on `/auth/nid/start` is *not* ported in PR-1 -- it belongs to PR-2's login rewrite.
- **Never commit credentials.** `META_WHATSAPP_ACCESS_TOKEN`, `META_WHATSAPP_PHONE_NUMBER_ID`, `META_WEBHOOK_VERIFY_TOKEN`, `META_APP_SECRET`, `NID_CLIENT_SECRET`, `NID_VAULT_SECRET_ID` live in `.env`, which is gitignored. This repo has **no `.env` right now**; do not create one and commit it. Check `git status --porcelain` for `.env` before every `git add`.
- **Dependency policy:** no new packages. The merge brings in exactly one new runtime dep, `defusedxml>=0.7,<1` (feature branch), next to this branch's `cryptography>=43.0,<45`; both sides' lines survive the auto-merge of `pyproject.toml`. `respx` is not available and must not be added -- outbound fakes in this repo are hand-written stubs or a monkeypatched singleton (house style, see `tests/test_nid_auth_and_crypto_mesh.py:76-83`).
- **Toolchain floor:** `requires-python = ">=3.11"`, ruff `target-version = "py311"`, ruff `line-length = 100`, ruff `ignore = ["B008"]`. Local Python here is 3.13 and local `pytest` is 9.1.1 / `pytest-asyncio` 1.4.0, both **outside** the `pyproject` pins -- that drift exists today and is not this PR's problem.
- **No typechecker exists in this repo** (no mypy, no pyright, no `[tool.mypy]`). Do not add one and do not write "run typecheck" as a gate. The Python gates are `ruff check src tests` and `ruff format --check src tests`; the TypeScript check is `tsc -b`, which runs inside `npm run build`.
- **Frontend: npm only.** `frontend/package-lock.json` is the only lockfile. Never run `pnpm install` or `yarn` here.
- **Do not touch the nID repo** (`C:\Users\nommi\Downloads\scraper\nID`) from this plan -- it holds another session's uncommitted `apps/login-ui/**` and `packages/payments/**` work.

## Environment reality (measured on this machine, 2026-11-01)

Verified, not assumed. Every command below was run and the stated output is real.

| Thing | Fact | Consequence for this plan |
|---|---|---|
| Postgres | 16.14 listening on `127.0.0.1:5432` with **trust** auth for existing roles; role `postgres` and databases `communicator`, `communicator_test` exist; role `communicator_test` exists, role `communicator` does **not** | `pytest` works out of the box (`test_database_url` default points at `127.0.0.1`). Alembic's default `database_url` points at **`localhost`**, which resolves to a *second* listener (`::1`) enforcing scram where that password fails. |
| `psql` | **not on PATH** | Never write a `psql -c "CREATE DATABASE ..."` step. Create databases with `asyncpg` (Task 4 shows how). |
| `alembic` CLI | `python -m alembic current` fails with `ModuleNotFoundError: No module named 'communicator'` because the package is not pip-installed here (CI does `pip install -e ".[dev]"`, which we are not reproducing) | Every alembic command in this plan is prefixed `PYTHONPATH=src`. |
| Local `communicator` DB | stamped `33fe48fddf99` -- the **feature branch's** head -- from someone's earlier run; `alembic current` on `newax/finalization` fails with "Can't locate revision identified by '33fe48fddf99'" | Do not trust it. Task 4 proves migrations on a **fresh** database instead. |
| `alembic check` | dirty on **both** sides: 32 `Detected …` lines on `newax/finalization`, 8 on the merged tree | It is not a gate in this repo and never was. CI only runs `alembic upgrade head`. Do not add a check step, do not "fix the drift". |
| `frontend/node_modules` | absent in the main checkout | `npm install` before `npm run build`. In the merge worktree it takes ~20s (warm npm cache). |
| Baseline suite | `230 passed in 89.41s` on `newax/finalization` @ `2e537ca` | That number is the post-merge expectation too -- verified: the merged tree also reports `230 passed`. |
| Baseline ruff | `All checks passed!` / `229 files already formatted` | Merged tree: `250 files already formatted`, still clean. |
| Baseline oxlint | not run on this branch's tree | Merged tree: `Found 47 warnings and 0 errors` -- **0 errors is the gate**; the 47 warnings are the feature branch's own and stay. |
| Working tree | `git status --porcelain` shows exactly ` M docs/superpowers/specs/2026-11-01-tenancy-and-nid-identity-design.md` | Task 1 commits it before branching. |
| Existing worktrees | `feat/multi-tenant-automation-campaigns` is checked out (clean) at `.claude/worktrees/scraper-reply-integration-654505` | Merging *from* it is fine -- git only refuses to check out the same branch twice. Do not delete that worktree. |

Set these once per shell for the whole plan:

```bash
PG_ADMIN="postgresql+asyncpg://postgres:postgres@127.0.0.1:5432/postgres"
```

---

## File structure

What this PR touches, and what each file is responsible for afterwards. Files marked *(auto)* merge without human input but are still verified.

| Path | Action | Responsibility after PR-1 |
|---|---|---|
| `src/communicator/api/app.py` | **resolve conflict** | App factory, lifespan vault load, router registration order, SPA hosting. The catch-all must stay last. |
| `src/communicator/services/reply_engine.py` | **resolve conflict** | Prompt assembly + reply generation. Keeps both the tenant-scoped KB/service lookups and the LiveProof refund-claim block. |
| `src/communicator/api/routes/nid_auth.py` | **resolve: keep ours** | `/auth/nid/start`, `/callback`, `/me` against this branch's client. |
| `src/communicator/services/nid_client.py` | **resolve: keep ours** + one-line fix | OAuth RP client; `_pending_states` / `_active_sessions` dicts (PR-2 deletes them). |
| `src/communicator/config.py` | **fix after auto-merge** | Settings. Auto-merge silently duplicates seven `nid_*` fields; one block must go. |
| `src/communicator/migrations/versions/d7f8a9b0c1d2_merge_live_proof_and_tenancy_heads.py` | **create** | Empty Alembic merge node. |
| `tests/test_nid_auth_and_crypto_mesh.py` | **modify** | Gains one regression test for the `settings.app_env` typo. |
| `pyproject.toml`, `src/communicator/models/__init__.py`, `api/dependencies.py`, `api/routes/{inboxes,webhooks}.py`, `frontend/**` | *(auto)* | Verified clean by Task 5, not hand-edited. |

Nothing else. If the merge reports a conflicted file outside this table, stop and re-read Step 2.3.

---

## Task 1: Commit the pending spec edit and open an isolated PR-1 worktree

Ten minutes. Ends with a clean worktree on a new branch, main checkout untouched.

- [ ] **Step 1: Confirm the starting state matches this plan**

```bash
cd /c/Users/nommi/Downloads/scraper/communicator
git rev-parse newax/finalization feat/multi-tenant-automation-campaigns
git status --porcelain
git merge-base newax/finalization feat/multi-tenant-automation-campaigns
```

Expected: `2e537ca066b3c0685dcdfd9bbd7deeea72a3024d` and `0b5ba07740311c7b075fffe980b0b18cf3703ccd`; one modified line (`docs/superpowers/specs/2026-11-01-tenancy-and-nid-identity-design.md`) and **nothing else** -- no untracked `.env`; `4fec3aab2d1549c279f0cb2677664c3c429dddef` as the merge base.

If any other file is dirty, that is someone else's in-progress work. Do not stash it, do not reset it. Add it to the handoff report and use a worktree from the *committed* ref (Step 3 already isolates you).

- [ ] **Step 2: Commit the spec on `newax/finalization`**

```bash
git add docs/superpowers/specs/2026-11-01-tenancy-and-nid-identity-design.md
git commit -m "docs(spec): correct tenancy design against the nID repo's real shape"
git status --porcelain   # expect empty
```

- [ ] **Step 3: Create the worktree and branch**

```bash
git worktree add -b feat/tenancy-pr1-merge ../communicator-tenancy-pr1 newax/finalization
cd /c/Users/nommi/Downloads/scraper/communicator-tenancy-pr1
git log --oneline -1   # expect the spec commit from Step 2
```

Expected: `HEAD is now at <sha> docs(spec): correct tenancy design...`. All later steps run in this directory.

**DoD:** `git worktree list` shows the new path on `feat/tenancy-pr1-merge`; the main checkout is still on `newax/finalization` with a clean tree.

---

## Task 2: Run the merge and resolve the four conflicts

Thirty minutes. Ends with every conflict resolved and the app importing. Do **not** commit yet -- Task 3 and 4 land inside the same uncommitted merge state, and Task 5 makes the single merge commit.

- [ ] **Step 1: Start the merge**

```bash
git merge --no-ff --no-commit feat/multi-tenant-automation-campaigns
git diff --name-only --diff-filter=U
```

Expected -- exactly these four, in this order, and `Automatic merge failed` on stderr:

```
src/communicator/api/app.py
src/communicator/api/routes/nid_auth.py
src/communicator/services/nid_client.py
src/communicator/services/reply_engine.py
```

Verified in a throwaway worktree on 2026-11-01 with this exact command. If you get a fifth file, the branch tips moved since this plan was written: run `git rev-parse` on both branches, diff against the SHAs in Task 1 Step 1, and re-derive that file's conflict instead of guessing.

- [ ] **Step 2: Take this branch's nID files wholesale (spec D2)**

```bash
git checkout --ours src/communicator/api/routes/nid_auth.py src/communicator/services/nid_client.py
git add src/communicator/api/routes/nid_auth.py src/communicator/services/nid_client.py
grep -c "<<<<<<<" src/communicator/api/routes/nid_auth.py src/communicator/services/nid_client.py
```

Expected: `0` for both. These are add/add conflicts -- both branches wrote the file from scratch after `4fec3aa`, so "ours" means "the whole file as `newax/finalization` wrote it". The feature branch's `dev_mode`, `LoginStartResponse` change, and its `nid_client` rewrite are all dropped here; PR-2 re-ports `dev_mode`.

Why this is safe: the only *other* callers of `nid_client` in the merged tree are `api/dependencies.py:32` and `api/routes/instagram_auth.py:54`, and both call `validate_session(token)`, which ours provides -- returning the same claims dict (`{"account_id", "phone", "email", "phone_verified", "access_token"`) that `get_current_account_id` indexes at `dependencies.py:37`. `routes/organizations.py` imports `get_current_account_id` and `Tenant`, never `nid_client`.

- [ ] **Step 3: Resolve `reply_engine.py` -- one region, keep both clauses**

The auto-merge already combined the imports (`import re`, `live_proof.create_live_proof_session`, `relationship_profile.{get_relationship_summary, render_relationship_context}`), `_ALERT_INTENTS`, `_render_knowledge_base(tenant_key=...)`, `_build_context`'s tenant scoping and relationship summary, and `generate_reply`'s LiveProof block. Only the prompt-template region conflicts. Replace it exactly:

Find:

```
<<<<<<< HEAD
If the customer asks for a return, refund, or reports damaged/defective merchandise, set intent to "refund_request" or "damaged_item".
=======
If a CUSTOMER RELATIONSHIP section is present, use it to respond with awareness of the relationship --
acknowledge what was previously discussed instead of restarting the conversation. Never state anything
about the relationship that isn't in that section.
>>>>>>> feat/multi-tenant-automation-campaigns
```

Replace with (both clauses survive; relationship first because it qualifies *how* to use context, refund second because it extends the intent enum below it):

```
If a CUSTOMER RELATIONSHIP section is present, use it to respond with awareness of the relationship --
acknowledge what was previously discussed instead of restarting the conversation. Never state anything
about the relationship that isn't in that section.
If the customer asks for a return, refund, or reports damaged/defective merchandise, set intent to "refund_request" or "damaged_item".
```

```bash
git add src/communicator/services/reply_engine.py
grep -c "<<<<<<<\|=======\|>>>>>>>" src/communicator/services/reply_engine.py
sed -n '73p;77p;101p' src/communicator/services/reply_engine.py
```

Expected: `0`, then the three surviving lines -- `_ALERT_INTENTS = ("buying_signal", "complaint", "escalate", "refund_request", "damaged_item")`, `async def _render_knowledge_base(session: AsyncSession, tenant_key: str = "newax") -> str:`, `tenant_key = contact.tenant_key or "newax"`. That is ours' intent tuple *and* the branch's tenancy scoping in one file.

- [ ] **Step 4: Resolve `app.py` -- three regions**

**Region 1 (file header).** Find:

```
<<<<<<< HEAD
from contextlib import asynccontextmanager

from fastapi import FastAPI
from fastapi.responses import RedirectResponse
=======
from pathlib import Path

from fastapi import FastAPI, HTTPException
from fastapi.responses import FileResponse
from fastapi.staticfiles import StaticFiles
>>>>>>> feat/multi-tenant-automation-campaigns
```

Replace with the union -- ours' lifespan import plus the branch's SPA-hosting imports. `RedirectResponse` is dropped because the `@app.get("/")` redirect it served is deleted by the merge (see "What PR-1 deliberately changes"); leaving it imported is an `F401` failure.

```
from contextlib import asynccontextmanager
from pathlib import Path

from fastapi import FastAPI, HTTPException
from fastapi.responses import FileResponse
from fastapi.staticfiles import StaticFiles
```

**Region 2 (route import list).** The auto-merge produced the 45-name union already; only `organizations` is contested. Find:

```
    nid_auth,
<<<<<<< HEAD
=======
    organizations,
>>>>>>> feat/multi-tenant-automation-campaigns
    outgoing_webhooks,
```

Replace with:

```
    nid_auth,
    organizations,
    outgoing_webhooks,
```

**Region 3 (registration tail).** This is the dangerous one. The auto-merge hoisted the branch's `app.include_router(nid_auth.router, prefix="/api/v1")` up into the main block (line 92 of the merged file), so ours' tail still contains a **second** `nid_auth` registration. Find from `<<<<<<< HEAD` to `>>>>>>> feat/multi-tenant-automation-campaigns` (ours' four includes, then the branch's `FRONTEND_DIST` + `spa()` block) and replace with ours' includes minus the duplicate, followed by the branch's SPA block **last**:

```python
# LiveProof verification portal and endpoints (WO-Verilo live camera return verification)
app.include_router(live_proof.router, prefix="/api/v1")
app.include_router(live_proof_ui.router)

# Zero-Trust Crypto Mesh Ingress
app.include_router(crypto_mesh.router, prefix="/api/v1")

# Built frontend (frontend/dist, from `npm run build`) -- served from the
# same origin as the API so the Instagram OAuth redirect_uri, webhook URLs
# etc. all point at one domain in production instead of needing a separate
# static host. Mounted/registered last so it never shadows an /api/v1/* route.
FRONTEND_DIST = Path(__file__).resolve().parents[3] / "frontend" / "dist"
if (FRONTEND_DIST / "assets").is_dir():
    app.mount("/assets", StaticFiles(directory=FRONTEND_DIST / "assets"), name="frontend-assets")


@app.get("/{full_path:path}", include_in_schema=False)
async def spa(full_path: str):
    """SPA fallback: any non-API, non-static path serves index.html and lets
    react-router handle the route client-side.

    index.html references the current build's hashed asset filenames, so it
    must never be cached (by the browser or by Cloudflare's edge) -- a
    cached index.html keeps pointing at a stale JS/CSS bundle after every
    deploy until the cache expires. The hashed files under /assets are safe
    to cache indefinitely since a new build always gets new filenames.
    """
    index = FRONTEND_DIST / "index.html"
    if not index.is_file():
        raise HTTPException(
            status_code=404, detail="Frontend build not found. Run `npm run build`."
        )
    return FileResponse(index, headers={"Cache-Control": "no-store, must-revalidate"})
```

`live_proof_ui.router` is included with **no prefix** (its routes are `/verify/proof/{session_id}`), so ordering it before the catch-all is what keeps the verification portal reachable. Registering the catch-all earlier turns every non-`/api/v1` path into `index.html`, including `/verify/proof/...` -- a silent breakage of a customer-facing page. Task 5 Step 2 asserts this.

```bash
git add src/communicator/api/app.py
grep -c "<<<<<<<\|>>>>>>>" src/communicator/api/app.py
grep -c "include_router(nid_auth.router" src/communicator/api/app.py
grep -c "include_router(crypto_mesh.router" src/communicator/api/app.py
```

Expected: `0`, `1`, `1`. (Each router exactly once -- the middle block also registers `nid_auth` at its auto-merged position, that's the one that stays.)

- [ ] **Step 5: Import smoke before anything else**

```bash
PYTHONPATH=src python -c "
from communicator.api.app import app
print('openapi paths:', len(app.openapi()['paths']))
print('conflicted files:', __import__('subprocess').run(['git','diff','--name-only','--diff-filter=U'],capture_output=True,text=True).stdout.strip() or 'none')
"
```

Expected: `openapi paths: 292`, `conflicted files: none`. A different path count means a router silently dropped out of the merged registration list -- diff `app.py` against `git show feat/multi-tenant-automation-campaigns:src/communicator/api/app.py` before continuing.

**DoD:** `git diff --name-only --diff-filter=U` empty; the four files contain no conflict markers; the app imports with 292 OpenAPI paths.

---

## Task 3: Fix the two things the auto-merge got wrong

Twenty minutes. Both are *silent* -- the merge "succeeds" and produces a broken or misleading tree. This is the reason PR-1 is a task list and not one `git merge -X ours`.

- [ ] **Step 1: See the damage first**

```bash
grep -n "nid_base_url\|nid_redirect_uri\|nid_allow_mock_auth" src/communicator/config.py
```

Expected on the merged tree -- **seven fields declared twice** (branch block at :61-72, ours at :89-95 + `nid_allow_mock_auth` at :104):

```
61:    nid_base_url: str = "http://localhost:4000"
68:    nid_redirect_uri: str = "http://localhost:8000/auth/nid/callback"
72:    nid_allow_mock_auth: bool = False
89:    nid_base_url: str = "http://localhost:4000"
92:    nid_redirect_uri: str = "http://localhost:8001/api/v1/auth/nid/callback"
104:    nid_allow_mock_auth: bool = False
```

Both branches added the same block at different line offsets, so git kept both copies and *no conflict was reported*. In a Python class body the later annotation wins, which is why this stays invisible until someone reads the file.

- [ ] **Step 2: Delete ours' duplicate block, keep the branch's**

Remove this block from `src/communicator/config.py` (it sits directly after `ecosystem_gateway_key: str = ""`):

```python
    # nID Single Identity Provider & Secrets Vault
    nid_base_url: str = "http://localhost:4000"
    nid_client_id: str = ""
    nid_client_secret: str = ""
    nid_redirect_uri: str = "http://localhost:8001/api/v1/auth/nid/callback"
    nid_vault_addr: str = "http://localhost:4000"
    nid_vault_role_id: str = ""
    nid_vault_secret_id: str = ""
```

and this single line from the crypto-mesh block (between `neura_crypto_auth_secret_hex` and `neura_relay_gateway_url`):

```python
    nid_allow_mock_auth: bool = False
```

The branch's block (`config.py:56-72`) stays, including its comment explaining that nID redirects the browser to the SPA route rather than the backend JSON endpoint. Reason for choosing it: PR-2's login rewrite needs exactly that shape (`NID_LOGIN_UI_URL` + SPA callback route), the two values differ only in an default that no test asserts (`grep -rn "nid_redirect_uri" tests/` is empty), and the surviving comment documents the architecture the spec picked. `nid_allow_mock_auth` keeps its `False` default either way, so D8's gating is unchanged.

- [ ] **Step 3: Verify one declaration each**

```bash
python -c "
import sys; sys.path.insert(0,'src')
from communicator.config import Settings
fields=[f for f in Settings.model_fields if f.startswith('nid_')]
print(fields)
print('declared twice:', [f for f in set(fields) if fields.count(f)>1] or 'none')
from communicator.config import settings
print('nid_redirect_uri =', settings.nid_redirect_uri)
"
```

Expected: `['nid_base_url', 'nid_client_id', 'nid_client_secret', 'nid_redirect_uri', 'nid_vault_addr', 'nid_vault_role_id', 'nid_vault_secret_id', 'nid_allow_mock_auth']`, `declared twice: none`, `nid_redirect_uri = http://localhost:8000/auth/nid/callback`.

- [ ] **Step 4: Write the failing test for the `settings.environment` typo**

`src/communicator/services/nid_client.py:161` reads `settings.environment`; the field is named `app_env` (`config.py:9`), so accessing it raises `AttributeError`. No test catches this today because every existing call passes a `test_`-prefixed code (`tests/test_nid_auth_and_crypto_mesh.py:39`) and Python short-circuits `or` before reaching line 161. In production -- nID down, real code, `nid_allow_mock_auth=False` -- the intended clean `NidAuthError` becomes a 500.

Add to the end of `tests/test_nid_auth_and_crypto_mesh.py`:

```python
@pytest.mark.asyncio
async def test_exchange_code_fails_cleanly_when_nid_unreachable(monkeypatch):
    """A real (non-test_) code with nID unreachable must raise NidAuthError.

    Before the app_env fix this raised AttributeError from
    settings.environment inside the mock-auth guard, i.e. the fallback
    path itself was unrunnable in production.
    """
    import time

    import communicator.services.nid_client as nid_client_module
    from communicator.config import settings
    from communicator.services.nid_client import NidAuthError

    monkeypatch.setattr(settings, "app_env", "production")
    monkeypatch.setattr(settings, "nid_allow_mock_auth", False)
    monkeypatch.setattr(nid_client_module.nid_client, "base_url", "http://127.0.0.1:1")
    nid_client_module._pending_states["state-real-code"] = {
        "poll_token": "poll-1",
        "redirect_uri": "http://localhost:5173/auth/nid/callback",
        "created_at": time.time(),
    }

    with pytest.raises(NidAuthError):
        await nid_client_module.nid_client.exchange_code(
            code="a-real-authorization-code", state="state-real-code"
        )
```

```bash
python -m pytest tests/test_nid_auth_and_crypto_mesh.py::test_exchange_code_fails_cleanly_when_nid_unreachable -q
```

Expected: **FAIL** with `AttributeError: 'Settings' object has no attribute 'environment'` inside `exchange_code`. If it passes, the merged `nid_client.py` is not ours' version -- re-run Task 2 Step 2.

- [ ] **Step 5: Fix the line**

`src/communicator/services/nid_client.py:161`:

```python
            or settings.app_env == "development"
```

(was `settings.environment == "development"`).

```bash
python -m pytest tests/test_nid_auth_and_crypto_mesh.py -q
grep -n "settings\.environment" src/communicator/services/nid_client.py
```

Expected: `4 passed` (the two existing tests, the crypto-mesh one, and the new one -- 3 existing + 1 new) and no matches for `settings.environment`.

- [ ] **Step 6: Confirm nothing is staged for commit yet, and note `.env` safety**

```bash
git add src/communicator/config.py src/communicator/services/nid_client.py tests/test_nid_auth_and_crypto_mesh.py
git status --short | grep -E "^\?\?" | grep -v "migrations/versions/d7f8a9b0c1d2" || echo "no unexpected untracked files"
```

Expected: `no unexpected untracked files`. `frontend/node_modules` and `frontend/dist` are gitignored (`frontend/.gitignore:10-11`) and must never appear here.

**DoD:** one declaration per `nid_*` field; the new test exists, failed before the fix and passes after.

---

## Task 4: Add the Alembic merge revision and prove it on a fresh database

Twenty minutes. Ends with `alembic upgrade head` succeeding from empty and `alembic heads` reporting one revision.

- [ ] **Step 1: Confirm the two heads exist post-merge**

```bash
PYTHONPATH=src python -m alembic heads
```

Expected (order may vary):

```
33fe48fddf99 (head)
c4d5e6f7a8b9 (head)
```

The chain: both sides forked at `b3e5f6a7c8d9`. Ours added one revision (`c4d5e6f7a8b9`, live_proof tables); the branch added nine (`29718cfde15d` → `3532b755459d` → `b72b9c04a734` → `a9562ba43aca` → `7cd5381c8995` → `5a95ba8a58af` → `c41400fdcea1` → `354b7c787957` → `33fe48fddf99`). Without a merge node every `alembic upgrade head` -- including CI's (`ci.yml:66`) -- fails with "Multiple head revisions".

- [ ] **Step 2: Write the merge revision**

Create `src/communicator/migrations/versions/d7f8a9b0c1d2_merge_live_proof_and_tenancy_heads.py`:

```python
"""merge live_proof and tenancy migration heads

Reunites the two Alembic heads that diverged from b3e5f6a7c8d9 when
`newax/finalization` (live_proof tables, c4d5e6f7a8b9) and
`feat/multi-tenant-automation-campaigns` (tenancy chain, 33fe48fddf99)
were merged in git. No schema work: neither side touches the other's
tables, so the merge point only has to exist for `alembic upgrade head`
to resolve to a single revision.

Revision ID: d7f8a9b0c1d2
Revises: c4d5e6f7a8b9, 33fe48fddf99
Create Date: 2026-11-01 14:00:00.000000

"""

from collections.abc import Sequence

revision: str = "d7f8a9b0c1d2"
down_revision: str | Sequence[str] | None = ("c4d5e6f7a8b9", "33fe48fddf99")
branch_labels: str | Sequence[str] | None = None
depends_on: str | Sequence[str] | None = None


def upgrade() -> None:
    pass


def downgrade() -> None:
    pass
```

Do **not** generate it with `alembic revision --autogenerate`: autogenerate cannot emit a two-parent merge, and it would also try to "fix" the pre-existing model/migration drift (32 `Detected …` lines on the base branch), which is out of scope. `upgrade()`/`downgrade()` stay empty because no table is shared between the two chains -- `c4d5e6f7a8b9` creates `live_proof_sessions`/`live_proof_evidences` and the tenancy chain only touches its own tables plus `tenant_key` columns on `services`, `knowledge_base_entries`, `automation_rules`, `tags`, `segments`, `leads`, `contacts`, `integration_settings`, `broadcasts`, `campaign_templates`.

- [ ] **Step 3: Assert a single head**

```bash
PYTHONPATH=src python -m alembic heads
```

Expected: `d7f8a9b0c1d2 (head)` and nothing else.

- [ ] **Step 4: Prove the full chain on a fresh database**

`psql` is not on PATH, so create the database with asyncpg. The DSN must name `127.0.0.1`, not `localhost` (see Environment reality).

```bash
PYTHONPATH=src python -c "
import asyncio, asyncpg
async def m():
    c = await asyncpg.connect(host='127.0.0.1', port=5432, user='postgres',
                              password='postgres', database='postgres', timeout=5)
    await c.execute('DROP DATABASE IF EXISTS communicator_pr1_check')
    await c.execute('CREATE DATABASE communicator_pr1_check')
    await c.close()
    print('fresh database created')
asyncio.run(m())
"
PYTHONPATH=src DATABASE_URL="postgresql+asyncpg://postgres:postgres@127.0.0.1:5432/communicator_pr1_check" \
  python -m alembic upgrade head 2>&1 | tail -4
```

Expected -- the last four lines of a 29-revision run, ending in the merge node (verified):

```
Running upgrade c41400fdcea1 -> 354b7c787957, add missing channeltype enum values
Running upgrade 354b7c787957 -> 33fe48fddf99, add tenants table
Running upgrade b3e5f6a7c8d9 -> c4d5e6f7a8b9, Add LiveProof return and fraud prevention session tables
Running upgrade c4d5e6f7a8b9, 33fe48fddf99 -> d7f8a9b0c1d2, merge live_proof and tenancy heads
```

Note the second-to-last line: alembic applies ours' chain *after* the branch's, from the shared `b3e5f6a7c8d9` ancestor. That is the merge node doing its job.

- [ ] **Step 5: Verify both sides' tables landed**

```bash
PYTHONPATH=src python -c "
import asyncio, asyncpg
async def m():
    c = await asyncpg.connect(host='127.0.0.1', port=5432, user='postgres',
                              password='postgres', database='communicator_pr1_check')
    tables = await c.fetch(\"select tablename from pg_tables where schemaname='public' order by 1\")
    names = {r[0] for r in tables}
    for expected in ('live_proof_sessions','live_proof_evidences','tenants','inquiries','customers','campaign_templates','discovery_flow_templates'):
        print(('OK  ' if expected in names else 'MISSING '), expected)
    cols = await c.fetch(\"select column_name from information_schema.columns where table_name='knowledge_base_entries'\")
    print('knowledge_base_entries has tenant_key:', 'tenant_key' in {r[0] for r in cols})
    ver = await c.fetch('select version_num from alembic_version')
    print('alembic_version rows:', [r[0] for r in ver])
    await c.close()
asyncio.run(m())
"
```

Expected: all seven `OK`, `knowledge_base_entries has tenant_key: True`, `alembic_version rows: ['d7f8a9b0c1d2']` (one row -- the merge node is the single stamp).

- [ ] **Step 6: Drop the scratch database**

```bash
PYTHONPATH=src python -c "
import asyncio, asyncpg
async def m():
    c = await asyncpg.connect(host='127.0.0.1', port=5432, user='postgres', password='postgres', database='postgres')
    await c.execute('DROP DATABASE communicator_pr1_check')
    await c.close(); print('dropped')
asyncio.run(m())
"
git add src/communicator/migrations/versions/d7f8a9b0c1d2_merge_live_proof_and_tenancy_heads.py
```

Expected: `dropped`. Only your own scratch database is removed -- never `communicator`, `communicator_test`, or the local `communicator` DB's feature-branch stamp.

**DoD:** `alembic heads` prints one revision; the empty-to-head run succeeds; both sides' tables exist in the fresh database.

---

## Task 5: Run every gate, then make the merge commit

Twenty minutes plus ~90s per test run. Ends with `feat/tenancy-pr1-merge` holding one merge commit that a reviewer can read as "the merge, plus the two silent fixes, plus the head".

- [ ] **Step 1: The suite**

```bash
python -m pytest -q 2>&1 | tail -3
```

Expected: `230 passed`. Verified on this exact merged tree on 2026-11-01 (`230 passed in 85.49s`) -- the same number as the pre-merge baseline, so a *lower* count means tests were lost in the merge and a *higher* one means you added a test outside Task 3. Any failure: fix it, but see Step 6 for what must **not** be "fixed" by touching auth.

- [ ] **Step 2: Route-shadowing regression (the merge's one silent-breakage risk)**

Run this before `npm run build` and again after -- both states are meaningful.

```bash
PYTHONPATH=src DATABASE_URL="postgresql+asyncpg://postgres:postgres@127.0.0.1:5432/communicator_pr1_nodb" python -c "
import asyncio, httpx
from communicator.api.app import app
async def m():
    async with httpx.AsyncClient(transport=httpx.ASGITransport(app=app), base_url='http://test') as c:
        for path in ['/api/v1/health','/docs','/','/verify/proof/00000000-0000-0000-0000-000000000000']:
            r = await c.get(path)
            print(f'{r.status_code} {path} {r.text[:60]!r}')
asyncio.run(m())
"
```

Expected with no `frontend/dist` present (verified):

```
200 /api/v1/health '{"status":"ok"}'
200 /docs '\n    <!DOCTYPE html>...'
404 / '{"detail":"Frontend build not found. Run `npm run build`."}'
422 /verify/proof/00000000-0000-0000-0000-000000000000 '{"detail":[{"type":"missing","loc":["query","token"],...'
```

The `/verify/proof/...` line is the assertion that matters: `422` means the request reached `live_proof_ui` and was rejected for the missing `token` query param. If it returns the SPA's `Frontend build not found` 404 instead, the catch-all is registered before `live_proof_ui` -- go back to Task 2 Step 4 Region 3 and re-order.

- [ ] **Step 3: Python lint and format, verbatim from CI**

```bash
ruff check src tests
ruff format --check src tests
```

Expected: `All checks passed!` and `250 files already formatted`. (250, not ours' 229 -- the merge adds the branch's route/service modules and tests.) If format complains, run `ruff format src tests`, then re-read the diff: the merge should not need reformatting, so a reformatted *hunk* is a hint you edited a region beyond Task 2's instructions.

- [ ] **Step 4: Frontend build (this is also the TypeScript gate -- `tsc -b` runs first)**

```bash
cd frontend && npm install --no-audit --no-fund 2>&1 | tail -2 && npm run build 2>&1 | tail -8 && npm run lint 2>&1 | tail -3
```

Expected (verified): `added 51 packages`; then `vite v8.3.4 building...`, `✓ 1994 modules transformed`, `dist/assets/index-*.js  1,306.86 kB`, `✓ built in 2.23s`; then `Found 47 warnings and 0 errors.` The chunk-size warning is the branch's own, not a failure. Gate = **0 errors** from both `tsc -b` and oxlint.

- [ ] **Step 5: Re-run the shadowing probe with `dist` present, then the suite once more**

```bash
PYTHONPATH=src python -c "
import asyncio, httpx
from communicator.api.app import app
async def m():
    async with httpx.AsyncClient(transport=httpx.ASGITransport(app=app), base_url='http://test') as c:
        for path in ['/','/login','/organizations','/api/v1/health','/verify/proof/00000000-0000-0000-0000-000000000000']:
            r = await c.get(path)
            print(f'{r.status_code} {path} [{r.headers.get(\"content-type\",\"\")[:20]}]')
asyncio.run(m())
"
cd .. && python -m pytest -q 2>&1 | tail -2
```

Expected (verified): `200 / [text/html]`, `200 /login [text/html]`, `200 /organizations [text/html]`, `200 /api/v1/health [application/json]`, `422 /verify/proof/... [application/json]`, then `230 passed`. The SPA now serves its routes and `/verify/proof` still bypasses it -- both orderings hold under the real build.

- [ ] **Step 6: Do not fix these (they are PR-2/PR-3 work, and fixing them here breaks the "behaviour-neutral" contract)**

- `/api/v1/auth/nid/start` returns an `authorize_url` of `http://localhost:4000/authorize/poll_fallback_<hex>` and logs `nID unreachable ...; using resilient fallback redirect`. Known, intentional today; PR-2's login rewrite replaces it.
- `GET /organizations` returns 401 without a Bearer token (it depends on `get_current_account_id`), while every other router is still unguarded. Expected mid-state.
- `get_tenant_key()` (`api/dependencies.py:52`) still hardcodes `or "newax"`. Deleted by PR-3's `require_tenant_member`.
- No endpoint gained or lost an auth dependency in this PR. If a test fails with 401 after you touched a dependency, you introduced enforcement -- revert that edit and record the failure instead.

- [ ] **Step 7: Commit the merge**

```bash
git status --short | grep -v "^A \|^M \|^D "   # only expected entries; no .env, no frontend/dist
git commit -m "$(cat <<'EOF'
Merge feat/multi-tenant-automation-campaigns into the tenancy line

Combines the LiveProof/crypto-mesh work on newax/finalization with the
multi-tenant + SPA-hosting work, so tenancy enforcement (PR-2/PR-3) has
one tree to land on.

Conflict resolutions:
- nid_auth.py / nid_client.py: kept this branch's versions (design D2);
  the feature branch's dev_mode is ported in PR-2.
- app.py: union of both router sets, SPA catch-all registered after
  live_proof_ui so /verify/proof/{session_id} stays reachable.
- reply_engine.py: keeps tenant-scoped KB/service lookups, the
  relationship-profile context and the LiveProof refund-claim block.

Silent auto-merge damage fixed in this commit:
- config.py declared seven nid_* settings twice; the duplicated block is
  removed, leaving the SPA-route nid_redirect_uri.
- nid_client.py read settings.environment (the field is app_env), so the
  nID-unreachable fallback raised AttributeError instead of NidAuthError;
  regression test added.
- New Alembic merge revision d7f8a9b0c1d2 unifies heads c4d5e6f7a8b9 and
  33fe48fddf99; alembic upgrade head verified on an empty database.

Gates: pytest 230 passed, ruff check + format clean, tsc -b + vite build
clean, oxlint 0 errors.
EOF
)"
git log --oneline -3
```

Expected: a merge commit with two parents (`git rev-list --parents -n1 HEAD` shows three SHAs), then the branch's 48 commits beneath it. Verify with:

```bash
git rev-list --parents -n1 HEAD
git log --oneline --first-parent -4
```

- [ ] **Step 8: Report, and stop before pushing**

Write down: post-merge test count, ruff result, build result, the single Alembic head, the 292 OpenAPI paths, and anything from Step 6 you saw change hands. Then **ask the user before** `git push -u origin feat/tenancy-pr1-merge` or opening a PR -- pushing is shared state and is not part of this plan's authorisation.

**DoD:** all five gates green with the numbers above; one merge commit on `feat/tenancy-pr1-merge`; nothing pushed without an explicit yes.

---

## What PR-1 deliberately changes

Three things move because the merge forces a choice, not because this plan wanted a behaviour change. Each is stated so a reviewer can veto it.

1. **`GET /` no longer redirects to `/docs`.** Ours had `@app.get("/") → RedirectResponse("/docs")`; the feature branch deleted it in favour of serving the built SPA from the API's own origin. The deletion wins because PR-2's login flow needs the SPA at root (`/login`, `/organizations`, `/auth/nid/callback` are react-router routes, and nID redirects the browser to the SPA). `/docs` itself still works -- FastAPI registers it before any router, verified `200` in Task 5 Step 2.
2. **`settings.nid_redirect_uri` default moves** from `http://localhost:8001/api/v1/auth/nid/callback` to `http://localhost:8000/auth/nid/callback`. No test asserts either value; neither is registered with nID yet (Plan 1 Task 6 registers `http://localhost:5173/auth/nid/callback`), and PR-2 rewrites the whole path.
3. **`_PROMPT_TEMPLATE` gains a CUSTOMER RELATIONSHIP clause ahead of the refund clause.** Prompt text is behaviour for the LLM, not for the tests; the two clauses are independent and both are kept.

## Follow-ups, explicitly out of scope for PR-1

Recorded here so they are not lost; none is merge damage and none blocks PR-2.

- `live_proof_sessions.session_token` has a **non-unique** index in migration `c4d5e6f7a8b9` while the model declares it unique (`ix_live_proof_sessions_session_token ... unique=False to unique=True` in `alembic check`). A duplicate token would make two verification sessions interchangeable at `/verify/proof/{id}`. Pre-existing on this branch; needs its own migration and its own test.
- `alembic check` is dirty on both sides (32 detected ops on the base tree) -- the repo has never kept autogenerate in sync and CI only runs `upgrade head`.
- `harvester_api_base_url` defaults to `http://localhost:8000/api/v1`, the same origin Communicator's own API serves on, and Harvester has no local checkout. Do not "fix" the port by guessing; spec §7 leaves it open on purpose.
- The merged SPA requires a session before rendering anything (`ProtectedRoute` → `/login` → `/api/v1/auth/nid/start`), and `/login` cannot complete without a reachable nID whose registered `redirect_uri` matches. So after PR-1, a fresh browser on the built frontend lands on a login screen that can't finish. That is exactly the intermediate state the spec predicted for PR-1 ("a real login can only be verified once PR-N1 is up"); PR-2 closes it.
- `Contact.tenant_key`/`tenant_id` and the `tenants` table now exist side by side with `LiveProofSession.tenant_id` defaulting to the string `"enterprise-default"` (`reply_engine.py`, ours' code). PR-3's tenancy sweep reconciles them.

## Done

- [ ] `git diff --name-only --diff-filter=U` is empty and no conflict marker exists anywhere in `src/` or `tests/`.
- [ ] `nid_auth.py` and `nid_client.py` are byte-identical to `newax/finalization`'s versions except the one-line `app_env` fix (`git diff newax/finalization -- src/communicator/api/routes/nid_auth.py` empty).
- [ ] `alembic heads` prints exactly one revision; `alembic upgrade head` succeeds against an empty database and leaves `alembic_version = ['d7f8a9b0c1d2']`.
- [ ] `python -m pytest -q` reports `230 passed`, `ruff check src tests` reports `All checks passed!`, `ruff format --check src tests` reports `250 files already formatted`, `npm run build` succeeds, `npm run lint` reports 0 errors.
- [ ] `/verify/proof/{session_id}` resolves to `live_proof_ui` (422), not the SPA catch-all, with and without `frontend/dist`.
- [ ] No `nid_*` field is declared twice in `config.py`.
- [ ] One merge commit with two parents on `feat/tenancy-pr1-merge`; nothing pushed or opened without the user's explicit approval.
