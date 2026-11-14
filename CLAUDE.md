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
