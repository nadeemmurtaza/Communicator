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
commits, and merge strategy — identical across every NEWAX repo. It
also lists this repo's current gaps against that layout (root-level
debug logs, `erp-schema/*.prisma`, archive folders mixed into `src/`)
so they aren't mistaken for the target shape.

See `COMMUNICATION.md` for how this repo is meant to talk to nID and
the rest of the ecosystem (Module Gateway/gRPC, the NATS event bus,
per-ID ownership), how modules talk to each other inside this one
repo, and how third-party platforms (Shopify, Meta, TikTok) fit in.
None of the cross-repo pieces exist yet — see its audit note.

## NEWAX Ecosystem — Technology Stack & Architecture Plan

> Applies to every repo in the ecosystem (nID, Communicator, core-newax,
> erp-1, newcom, harvester, AegisDesk). Optimization order, in priority:
> **1) lowest RAM, 2) minimum garbage/allocation, 3) lowest battery drain
> (mobile), 4) maximum throughput/latency, 5) maximum security.**
> Speed-to-market is explicitly *not* in this list — every choice below
> trades dev velocity for runtime efficiency on purpose.

### Backend

- **Language:** Rust. No exceptions for new services — it's already the
  standard in this ecosystem (`backend-rs/`, `crates/`, `Cargo.toml`
  everywhere). No GC, no runtime, predictable allocation.
- **HTTP/RPC framework:** `tonic` (gRPC) + `tonic-web`/Connect handler on
  the same binary. Async via `tokio`.
- **Schema:** Protobuf (`.proto`) is the single source of truth for every
  API contract, shared across all services and all client platforms.
  Codegen, not hand-written DTOs, on every side.
- **Wire protocols by consumer** (one schema, three transports from the
  same Rust binary — no separate proxy process):
  - Native clients (iOS, Android, Windows): real gRPC over HTTP/2,
    persistent connections — fewer TLS handshakes, lower radio/battery
    cost on mobile.
  - Browser clients: **Connect protocol** (buf.build), not plain
    grpc-web. Connect speaks Protobuf-or-JSON over plain HTTP/1.1 or
    HTTP/2 directly from the Rust server — no Envoy/grpc-web proxy, no
    second schema, no extra RAM-hungry process to run and secure.
  - Service-to-service inside the ecosystem: gRPC via the Module Gateway
    in `core-newax` only. No repo calls another repo's API directly —
    see each repo's `ECOSYSTEM.md`.
- **Database:** PostgreSQL. Row-level security, mature TLS/auditing, no
  surprises. Not Mongo/MySQL unless a specific service proves a
  document-shaped workload needs it.
- **ORM / query layer:** `SQLx`, compile-time checked raw SQL. No Prisma
  (no real Rust support — the community client is unmaintained — and its
  Node-side engine/connection-pool model is exactly the RAM and GC
  overhead this ecosystem is optimizing away). `SeaORM` is the only
  fallback, and only where SQLx's lack of ORM ergonomics is a real
  blocker, not a convenience preference.
- **Caching, two tiers, add the second only when actually needed:**
  - In-process: `moka` (Rust, concurrent LRU/TTL) inside the service —
    zero network hop, zero extra process, lowest RAM for a
    single-instance deployment.
  - Shared/distributed: Redis, only once there are multiple backend
    instances that must agree on cached state. Don't stand up Redis for
    a single-instance service — that's pure RAM/ops cost with no payoff.
- **Security baseline (by construction, not bolted on):**
  - Memory safety from Rust itself — no buffer overflows/UAF class of
    bugs.
  - SQL injection closed by SQLx's compile-time query verification, not
    by a runtime sanitizer.
  - mTLS between services wherever traffic crosses a process boundary
    (Module Gateway, event bus, any Connect/gRPC endpoint not fronting a
    public browser).
  - Identity is centralized in **nID** (OAuth sessions, MFA/2FA, secrets
    vault) — no repo reinvents login or stores its own secrets; see
    `ECOSYSTEM.md`.

### Frontend — per platform, same Rust core, different native shell

The core strategy is **Shared Core, Thin Native UI**, extended to the web
via WebAssembly so there is still exactly one logic implementation, not
"mobile+desktop in Rust, web in a second stack":

- **Core logic module:** Rust, compiled natively for iOS/Android/Windows
  and to **WASM** for the browser. Business rules, caching rules, and
  networking (the gRPC/Connect client) live here once, bound into every
  platform via `UniFFI` (native) or `wasm-bindgen` (web). This is the
  piece that keeps behavior identical across platforms and keeps the
  footprint small everywhere.

#### Design system (shared global CSS tokens)

Canonical token file: `packages/design-tokens/tokens.css` (package
`@newax/design-tokens`), imported as `@import
"@newax/design-tokens/tokens.css";` at the top of
`apps/web/src/index.css`. This is the exact same package name, file
name, and folder (`packages/design-tokens/`) as nID — not a local
file this repo invented independently. Same palette, typography,
spacing, layout, radius, elevation and motion tokens as every other
NEWAX repo. Until `core-newax` hosts this as a real shared package
(the correct long-term home, per this ecosystem's "build once, reuse
everywhere" rule), the package is duplicated byte-for-byte across
repos: edit one, copy to the others, never fork the values or let one
repo's copy drift into a loose file while another's stays a package.
Components read `--color-*`, `--text-*`, `--space-*`, `--radius-*`,
`--elevation-*`, `--layer-*`, `--motion-*` — never a literal value.

- **iOS:** Swift/SwiftUI shell, calling the Rust core via UniFFI. ARC,
  no GC pauses, no bridge overhead.
- **Android:** Kotlin/Jetpack Compose shell, same UniFFI bridge into the
  same Rust core.
- **Windows:** C#/WinUI 3 as the default; if the C# GC proves to be a
  real RAM/latency cost in profiling, prototype a Rust-native UI
  (`slint` or `egui`) to remove the last non-Rust layer on desktop.
- **Web:** Svelte (compiles away — no virtual-DOM diffing garbage at
  runtime, unlike React) driving the Rust-core-as-WASM module in-browser
  directly, talking Connect to the backend. This is deliberately *not*
  Flutter: Flutter's Dart runtime carries its own garbage collector,
  which directly conflicts with this ecosystem's "minimum garbage"
  requirement — Impeller fixes rendering jank, not allocation pressure.
  A single cross-platform Dart/Flutter codebase is the faster-to-ship
  choice and was considered and rejected here specifically because
  runtime efficiency outranks speed-to-market for this project.

#### Package manager: pnpm, everywhere, no exception

Both repos' JS/TS tooling runs on **pnpm workspaces** (`pnpm-workspace.yaml`
at the repo root, `apps/*` + `packages/*`), not npm. This was a real
divergence — Communicator used npm, nID used pnpm — and it's resolved
in pnpm's favor:

- **Lowest RAM/disk, by construction:** pnpm keeps one global
  content-addressable store and hard-links packages into each
  `node_modules`, instead of npm's copy-per-project. Less disk churn,
  less duplicate install garbage across `apps/*`/`packages/*`.
- **No phantom dependencies:** pnpm's non-flat `node_modules` means
  code can only import what its own `package.json` actually declares
  — a correctness property npm's flat hoisting doesn't give you.
- **Real monorepo primitives:** `workspace:*` protocol, `pnpm -r
  --filter`, and content-addressable linking are built for exactly
  the `apps/*` + `packages/*` shape both repos use — npm workspaces
  support the shape but not the efficiency.
- Communicator's root `package.json` and `pnpm-workspace.yaml` mirror
  nID's exactly; `apps/web/package.json` depends on
  `"@newax/design-tokens": "workspace:*"`, same as nID's apps. Run
  `pnpm install` at the repo root, not `npm install`.

#### Tech-stack compliance audit (2026-11-14)

Honest status against the plan above, not papered over:

- **Backend wire protocol: not compliant.** `backend-rs` is Axum +
  plain REST/JSON today — no `tonic`/`prost`/`.proto` anywhere in this
  repo. The gRPC/Connect plan above is the target for new cross-repo
  communication (see `COMMUNICATION.md`) and for a future deliberate
  migration, not a description of what's running. Don't assume any
  existing endpoint speaks gRPC.
- **Database: compliant.** SQLx + PostgreSQL, as planned.
  `erp-schema/*.prisma` is schema-*modeling* only — confirmed no
  `@prisma/client` dependency anywhere in this repo.
- **Cache: not implemented.** No `moka`, no Redis, in this repo today.
  There is currently no caching layer at all, not a non-compliant one.
- **Frontend: not compliant, and not a quick fix.** `apps/web` is
  React + Vite + Tailwind, not Svelte, and talks REST, not Connect.
  This is ~150 existing pages, not a prototype stub — re-platforming
  it to Svelte/WASM-core/Connect would be a ground-up rewrite, not an
  audit fix, and isn't something to do without an explicit decision to
  spend that effort. Until that decision is made, treat the
  Svelte/WASM plan as the target for *new* client surfaces (iOS,
  Android, Windows, or a second web app), not a mandate to rewrite
  this one.

### Why this combination and not the obvious alternatives

- **Not Prisma** — wrong language ecosystem for a Rust backend, and its
  Node-side engine is the opposite of low-RAM/low-GC.
- **Not Flutter** — Dart's GC conflicts with the explicit
  minimum-garbage requirement; only reconsider this if speed-to-market
  is explicitly re-prioritized above runtime efficiency.
- **Not grpc-web via an Envoy proxy** — adds a second always-on process,
  more RAM, more attack surface, and a translation hop Connect avoids by
  serving browsers straight from the same Rust binary that serves gRPC.
- **Shared Rust core via UniFFI/WASM, not four separate reimplementations**
  — one schema (Protobuf), one logic implementation (Rust), four thin
  native shells. The cost is real: this is more upfront engineering than
  one Flutter codebase. That tradeoff is intentional given this
  ecosystem's stated priorities (RAM, GC, battery, security over
  dev speed).
