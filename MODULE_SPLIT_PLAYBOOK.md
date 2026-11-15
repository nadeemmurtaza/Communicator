# Module Split Playbook (Communicator)

Step-by-step instructions for converting one `backend-rs` monolith
module (a `routes/<module>.rs` + `services/<module>.rs` pair) into a
real, independently-deployable `tonic` gRPC service — the pattern
`CONVENTIONS.md` calls "the crates/services split."

This file is **specific to this repo** (unlike `CONVENTIONS.md`,
`COMMUNICATION.md`, `ECOSYSTEM.md`, and `ecosystem-stack.md`, which
stay identical across `nID` and `Communicator` — see "Repo Structure &
Naming Conventions" in this repo's `CLAUDE.md`/`AGENT.md`). nID has
its own `MODULE_SPLIT_PLAYBOOK.md` for its `apps/rust-api` module
shape, which differs enough (folder-per-module vs. file-per-module,
`DbPool` vs. plain `PgPool`, workspace-inherited dependencies) that a
single shared playbook kept forcing "Communicator does X, nID does Y"
branches into every section. Don't copy this file into nID, and don't
copy nID's over this one — if a cross-repo-relevant convention changes
(port-block boundaries, the overall "two distinct roles" model), that
belongs in `CONVENTIONS.md`/`COMMUNICATION.md` instead, with just a
pointer here.

Follow this document top to bottom for every module you split. Do not
skip steps because a module "looks simple" — the steps that look
skippable (the coupling grep, the registration edits, the final
`cargo check`) are exactly the ones that silently break the build two
modules later if skipped.

---

## 0. Required reading, in this order

Read these once before your first split, and re-skim §3 and §10 of
`CONVENTIONS.md` before every module after that (they are the parts
that actually change your next action):

1. **`ECOSYSTEM.md`** — what this repo is, how it relates to the other
   six repos in the ecosystem, and the "one rule" for cross-repo
   identity ownership. Context only; you will not act on this file
   directly while splitting a module.
2. **`ecosystem-stack.md`** — the full technology stack decision and
   its compliance audit. The relevant part for this playbook is the
   Backend section's gRPC bullet and the "Module separation: started,
   not finished" audit line, which is what this playbook is closing
   out, module by module.
3. **`CONVENTIONS.md`** — the actual rulebook. Read in full once; the
   parts you will return to on every module are:
   - §1 folder layout (`crates/<module>/`, `services/<module>/`,
     `proto/communicator/<module>/v1/<module>.proto`)
   - §2 "one name, five places" (module name used identically as crate
     suffix, proto package, DB schema/table prefix, event-bus topic
     prefix, and folder name — see §2 below for how this playbook
     applies it to crate/service/proto folder names specifically,
     including the naming drift already present and what to do about
     it)
   - §3 backend separation (module separation is mandatory for all new
     work; old monolith routes/services are a migration backlog, not a
     template)
   - The **"crates/services split: in progress, tracked here"**
     section — the live table of every module already split, their
     ports, and the "Known shortcut, flagged" note about the
     `127.0.0.1:<port>` direct-dial stand-in. **You will add a row to
     this table for the module you just split, in both repos, in its
     own commit, before you consider the module done — see §12.**
4. **`COMMUNICATION.md`** — defines the "two distinct roles": Role 1
   is `tonic`+Protobuf for module-to-module calls (this is what you
   are building), Role 2 is the Pingora Module Gateway that will one
   day sit in front of all of it (you are not building this; the
   direct `127.0.0.1:<port>` dial is its temporary, explicitly-flagged
   stand-in). Also defines ID ownership if your module touches
   tenant/org/customer identity.

If any of these files disagree with this playbook on a point of
convention, **the other file wins** — update this playbook to match
and say so in your commit message.

---

## 1. Pick a module

### 1.1 Candidate shape

A real split candidate is a pair:
`backend-rs/src/routes/<module>.rs` (axum handlers) and
`backend-rs/src/services/<module>.rs` (business logic + types), wired
together by `use crate::services::<module>;` in the routes file and
registered as `pub mod <module>;` in both
`backend-rs/src/routes/mod.rs` and `backend-rs/src/services/mod.rs`,
with its routes mounted in `backend-rs/src/main.rs`.

### 1.2 The zero-coupling check (mandatory, every time)

Before you write a single new file, confirm the module has **no
in-process Rust calls into or out of another module's logic** (route
registration and shared DB tables are fine; direct function/struct
calls across module boundaries are the thing you're checking for):

```bash
# Look for the module calling another module's service
grep -n "crate::services::" backend-rs/src/services/<module>.rs | grep -v "crate::services::<module>"

# Check whether anything OTHER than the route file and main.rs
# constructs or calls into this module's service in-process. This is
# the check that caught reply-integration's coupling from
# reply_engine.rs — never skip it.
grep -rln "services::<module>\|<Module>Service::new" backend-rs/src --include="*.rs" \
  | grep -v "routes/<module>.rs\|services/<module>.rs\|routes/mod.rs\|services/mod.rs\|main.rs"
```

Three outcomes:

- **Nothing found anywhere** → clean, proceed with this playbook as
  written.
- **Found in the module's own route file only** → still clean (that's
  the expected caller); proceed.
- **Found somewhere else** (another service file, like
  `reply_engine.rs` calling `ReplyIntegrationService` directly) → this
  is real in-process coupling. You still split the module, but see
  §9 "Handling discovered in-process coupling" below instead of
  stopping.

### 1.3 Modules that are NOT split candidates

Some modules are pure shared libraries with no route handlers of
their own, called in-process by other services. Converting one of
these to a standalone gRPC service is wrong: it belongs inlined as a
library dependency, not behind a network call for every use. Check
that a route file is actually registered in `main.rs` before
starting — if it isn't, skip the module and note why in
`CONVENTIONS.md`'s tracking section (nID's `badges` module is the
documented example of this shape, even though it lives in the other
repo — the same reasoning applies here if you find a Communicator
module shaped like it).

### 1.4 Prefer the smallest clean candidate

Among the zero-coupling candidates, split the smallest one next
(`wc -l` the route+service files). This keeps each commit reviewable
and keeps the "convert one module, verify, commit" loop fast. Do not
batch multiple modules into one commit.

---

## 2. Naming — "one name, five places," and the drift to watch for

Per `CONVENTIONS.md` §2, the module's name should be identical
everywhere it appears. **In practice, this repo's own history
drifted**: the first few modules split used kebab-case for the
crate/service folder (`crates/kb-documents`, `services/kb-documents`,
`crates/embed-widget`, `crates/help-center`, `crates/scheduled-tasks`,
`crates/system-health`), while every module split afterward used
snake_case to match the proto package and Rust module conventions
(`crates/conversation_search`, `services/cx_qa_scoring`,
`crates/tone_of_voice`, `crates/ab_testing`,
`crates/compliance_dashboard`, `crates/unified_analytics`,
`crates/in_app_messaging`). **Snake_case is the rule going forward**
— it matches the proto package name (`newax.<module>.v1`, always
snake_case) and avoids a mismatch between the crate's directory name
and its module path. Do not "fix" the already-kebab-case folders as a
side effect of an unrelated module's split; that's a separate, scoped
cleanup if anyone decides to do it.

Concretely, for a module named `my_module`:

| Where | Spelling | Example |
|---|---|---|
| Proto package | `newax.my_module.v1` | always snake_case |
| Proto file path | `proto/communicator/my_module/v1/my_module.proto` | snake_case folder + filename |
| Crate folder | `crates/my_module/` | snake_case |
| Crate package name (`Cargo.toml` `name =`) | `newax-my-module` | kebab-case (Cargo convention) |
| Crate lib name (`Cargo.toml` `[lib] name =`) | `newax_my_module` | snake_case (this is the Rust `use` path) |
| Service folder | `services/my_module/` | snake_case |
| Service package/binary name | `svc-my-module` | kebab-case |
| `crates/proto` registration module | `pub mod my_module { pub mod v1 { ... } }` | snake_case |
| Env var prefix | `MY_MODULE_SERVICE_URL`, `MY_MODULE_PORT` | SCREAMING_SNAKE_CASE |

This repo's crates always set `[lib] name = "newax_my_module"`
explicitly in `Cargo.toml` — keep doing that; it's what makes
`use newax_my_module::...` work from the service binary and from
`backend-rs` itself (for the in-process-coupling case, §9).

---

## 3. Folder layout you will create

For a module named `my_module`, you create exactly these new paths
(nothing else — never touch files outside them except the specific
edits in §7 and §8):

```
proto/communicator/my_module/v1/my_module.proto   # new
crates/my_module/Cargo.toml                        # new
crates/my_module/src/lib.rs                         # new (moved logic)
services/my_module/Cargo.toml                       # new
services/my_module/build.rs                          # new
services/my_module/src/main.rs                        # new
```

Plus edits (never full rewrites) to three files that already exist:

```
crates/proto/src/lib.rs      # add one `pub mod my_module { pub mod v1 { ... } }` block
crates/proto/build.rs        # add one path to the PROTOS const
backend-rs/src/routes/<module>.rs   # rewritten as a thin gRPC client (§8)
```

And removal of the old in-process logic:

```
backend-rs/src/services/<module>.rs   # deleted (git often shows this as a
                                        # rename to crates/<module>/src/lib.rs
                                        # once both changes are staged together)
backend-rs/src/services/mod.rs         # `pub mod <module>;` line removed
```

You never need to touch `[workspace] members` in the root
`Cargo.toml` — it already uses the globs `"backend-rs"`, `"crates/*"`
and `"services/*"`, so a new crate/service directory is picked up
automatically the moment it has a `Cargo.toml` in it. Verify this is
still true before you start (`grep -A6 "\[workspace\]" Cargo.toml`);
if someone ever changes it to an explicit list, you will need to add
your two new entries there too.

---

## 4. Writing the proto file

### 4.1 Decide: fully-typed messages, or a JSON envelope?

Look at what the **route handlers** currently return, not the
service's internal types:

- If every handler does `Ok(Json(some_concrete_struct))` (the struct
  is a real `#[derive(Serialize)]` type, not `serde_json::Value`) →
  write fully-typed `.proto` messages mirroring those structs field
  for field. This is the common case (`cx_qa_scoring`,
  `tone_of_voice`, `ab_testing` all did this).
- If most handlers do `Ok(Json(serde_json::to_value(x)?))` or already
  return `Json<serde_json::Value>` — i.e. the original code itself
  treats the payload as dynamic JSON, not a fixed shape — use the
  pragmatic envelope instead:
  ```protobuf
  message JsonResponse {
    string json = 1;
  }
  message Empty {}
  ```
  and make every RPC that would otherwise need a dozen near-duplicate
  message types just return `JsonResponse`, with the service crate
  doing `serde_json::to_string(&real_struct)` and the client side
  doing `serde_json::from_str`. `kb_documents`, `embed_widget`,
  `help_center`, large parts of `monitoring`, `reply_integration`,
  `unified_analytics`, and `in_app_messaging` all used this — it is
  not a lesser pattern, it is the right one when the source was
  already dynamic. **Never invent a fixed shape the original code
  didn't have.**
- A module can mix both within one `.proto` file — type the RPCs whose
  source type was concrete, envelope the ones that weren't.
  `monitoring`'s `GetDashboardSummary` returns `JsonResponse` while
  every other RPC in that same service is fully typed.

### 4.2 Proto file skeleton

```protobuf
syntax = "proto3";

package newax.my_module.v1;

service MyModuleService {
  rpc DoThing(DoThingRequest) returns (DoThingResponse);
  // one rpc per route handler, same order as the handlers appear
  // in the route file — makes the main.rs diff easy to read against
  // the proto file side by side.
}

message Empty {}

// Prefer `optional` for every field that was `Option<T>` in the
// source Rust struct — proto3 only treats a field as "unset vs
// default" when you mark it `optional`; without it, you cannot tell
// an explicit empty string/0 from "the source had None".
message DoThingRequest {
  string required_field = 1;
  optional string optional_field = 2;
}

message DoThingResponse {
  string id = 1;
  // ... mirror the source struct's fields, in the source struct's
  // field order, 1-indexed, no gaps.
}
```

Rules that have held across every module split so far:

- One `.proto` file per module, one `service` block per module (never
  split one module's RPCs across two services).
- RPC names are PascalCase verbs matching the source function name
  (`get_summary` → `GetSummary`, `list_qa_rules` → `ListQaRules`).
- A `serde_json::Value` field inside an otherwise-typed message
  becomes a `string ..._json = N;` field (see `rules_json`,
  `config_json`, `metadata_json`, `forbidden_phrases_json` across
  existing protos) — never try to represent arbitrary JSON as a proto
  `map` or `Struct` type; stringify it and re-parse on both ends.
- A `Vec<String>` becomes `repeated string`; a `HashMap<String, f64>`
  becomes `map<string, double>` (used once, in `ab_testing`'s
  `VariantStats` — proto maps work fine when the value type is a
  scalar, just not when it needs to be `optional`).
- Request messages that exist purely to carry an ID take the pattern
  `message IdRequest { string id = 1; }` and are reused across every
  RPC in the module that just needs one ID — don't make a new
  single-field message per RPC.

---

## 5. Writing the crate (`crates/my_module/`)

### 5.1 `Cargo.toml`

```toml
[package]
name = "newax-my-module"
version = "0.1.0"
edition = "2021"

[lib]
name = "newax_my_module"
path = "src/lib.rs"

[dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"                                   # only if the module uses JSON values
sqlx = { version = "0.8", features = ["runtime-tokio", "postgres"] }
chrono = { version = "0.4", features = ["serde"] }  # only if the module uses timestamps
uuid = { version = "1", features = ["v4"] }         # only if the module generates IDs
anyhow = "1"                                        # only if the source used anyhow::Error
rand = "0.8"                                        # only if the source used rand (e.g. ab_testing)
```

Only list the dependencies the moved code actually uses — check the
source file's existing `use` statements, don't copy every dependency
this repo's crates happen to have elsewhere.

### 5.2 `src/lib.rs` — moving the logic

This is almost always a **near-verbatim copy** of the monolith's
`services/<module>.rs`, with at most one structural change: the
struct's database handle must end up being a plain pool.

Most Communicator services already take `&PgPool` or `Pool<Postgres>`
directly, not `&AppState` — check: does `fn new(pool: &PgPool)` or
`fn new(pool: Pool<Postgres>)` already exist? If so, copy the file
essentially unchanged. If the source instead has free functions
taking `&AppState` (e.g. `tone_of_voice`, `ab_testing` and
`monitoring` before their splits), change every `&AppState` parameter
to `&PgPool` and every `state.db` reference to the parameter name
directly — the function bodies themselves don't change.

Also:

- Keep any `#[cfg(test)] mod tests` block from the source file — it
  moves with the rest of the code unchanged.
- Preserve bugs. If the source code has an existing bug (e.g.
  `compliance_dashboard`'s `update_gdpr_request`, which builds a
  dynamic `UPDATE ... SET` query with N placeholders but only ever
  binds `binds[0]`), **do not fix it while migrating**. The module
  split must be behavior-preserving; file a separate note if you spot
  something worth fixing later, but fix it in its own commit, not
  bundled into the split.

---

## 6. Writing the service (`services/my_module/`)

### 6.1 `Cargo.toml`

```toml
[package]
name = "svc-my-module"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "svc-my-module"
path = "src/main.rs"

[dependencies]
newax-my-module = { path = "../../crates/my_module" }
tonic = "0.12"
prost = "0.13"
tokio = { version = "1", features = ["full"] }
sqlx = { version = "0.8", features = ["runtime-tokio", "postgres"] }
serde_json = "1"      # needed whenever main.rs itself calls serde_json (JSON envelope RPCs,
                       # or a `json_ok`/`parse_json` helper) — not just because the crate needs it
serde = { version = "1", features = ["derive"] }   # only if main.rs defines a generic
                                                      # `json_ok<T: serde::Serialize>` helper
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
dotenvy = "0.15"

[build-dependencies]
tonic-build = "0.12"
```

### 6.2 `build.rs`

```rust
fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure().compile_protos(
        &["../../proto/communicator/my_module/v1/my_module.proto"],
        &["../../proto"],
    )?;
    Ok(())
}
```

Note this is the **server-side** build script — it does *not* call
`.build_server(false)` (that flag belongs only to `crates/proto`'s
build script, §7, which generates client-only code for every other
module to depend on).

### 6.3 `src/main.rs` skeleton

```rust
use newax_my_module as logic;          // or `use newax_my_module::{Thing, MyModuleService};`
use sqlx::postgres::PgPoolOptions;
use tonic::{transport::Server, Request, Response, Status};
use tracing::info;

pub mod proto {
    tonic::include_proto!("newax.my_module.v1");
}

use proto::{
    my_module_service_server::{MyModuleService, MyModuleServiceServer},
    // ... every message type main.rs constructs or destructures
};

struct Svc {
    pool: sqlx::PgPool,
}

fn to_status<E: ToString>(e: E) -> Status {
    Status::internal(e.to_string())
}

// one conversion function per message type that needs mapping between
// the logic crate's plain struct and the generated proto struct —
// name them `<thing>_to_proto` / `<thing>_from_proto` consistently.

#[tonic::async_trait]
impl MyModuleService for Svc {
    async fn do_thing(&self, req: Request<proto::DoThingRequest>) -> Result<Response<proto::DoThingResponse>, Status> {
        let r = req.into_inner();
        let result = logic::MyModuleService::new(&self.pool).do_thing(&r.some_field).await.map_err(to_status)?;
        Ok(Response::new(do_thing_to_proto(result)))
    }
    // ... one method per rpc in the .proto service block, same order
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::try_new("info,svc_my_module=debug").unwrap())
        .init();

    let database_url = std::env::var("DATABASE_URL")
        .map_err(|_| "DATABASE_URL must be set (shared Communicator Postgres instance)")?;
    let pool = PgPoolOptions::new().max_connections(5).connect(&database_url).await?;

    let port: u16 = std::env::var("MY_MODULE_PORT").ok().and_then(|p| p.parse().ok()).unwrap_or(PORT);
    let addr = format!("0.0.0.0:{port}").parse()?;

    info!(port, "svc-my-module starting (gRPC)");
    Server::builder()
        .add_service(MyModuleServiceServer::new(Svc { pool }))
        .serve(addr)
        .await?;
    Ok(())
}
```

Every module split so far holds the raw pool in `Svc` and calls
`logic::MyModuleService::new(&self.pool)` once per request rather than
constructing it once at startup — keep doing that for consistency,
even though both work fine with `sqlx::PgPool`'s cheap-clone design.

### 6.4 Port allocation

Ports are a single incrementing counter **for this repo only** — nID
has its own, separate block starting at 50071; this repo's block
started at 50061 and is not shared with nID's numbering. **Check
`CONVENTIONS.md`'s tracking table for the highest Communicator port
already used and take the next integer** — do not leave gaps, and do
not reuse a port even if the module that used it was later renamed.

Env var name: `<MODULE_SCREAMING_SNAKE>_PORT` for the service's own
listen port, and `<MODULE_SCREAMING_SNAKE>_SERVICE_URL` for the
client-side override (used only by the route file's thin-client code
in §8, defaulting to `http://127.0.0.1:<port>`).

---

## 7. Registering the module in `crates/proto`

`crates/proto` is the one shared crate that generates **client-only**
stubs for every split module, so `backend-rs` (and any other module
that needs to call this one) depends on one pre-generated copy instead
of each caller regenerating its own.

Edit `crates/proto/src/lib.rs`, appending (never reordering existing
entries):

```rust
pub mod my_module {
    pub mod v1 {
        tonic::include_proto!("newax.my_module.v1");
    }
}
```

Edit `crates/proto/build.rs`, appending one path to the `PROTOS`
const (a `&[&str]` passed to one `compile_protos` call with
`.build_server(false)`):

```rust
"../../proto/communicator/my_module/v1/my_module.proto",
```

Do not add a new `tonic_build::configure()` call — there is exactly
one, covering every module's proto file in one compile pass.

---

## 8. Rewriting the route file

The route file (`backend-rs/src/routes/<module>.rs`) becomes the gRPC
client, with the **exact same public handler function names and
signatures** as before, so `main.rs`'s route registrations don't need
to change. Pattern:

```rust
//! <original doc comment>
//!
//! Thin gRPC client forwarding to svc-my-module; REST shape unchanged.

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use crate::db::AppState;   // or crate::AppState, whichever the original used

use newax_proto::my_module::v1::{
    my_module_service_client::MyModuleServiceClient,
    // ... every request/response message this file constructs
};

async fn client() -> Result<MyModuleServiceClient<tonic::transport::Channel>, (StatusCode, String)> {
    let url = std::env::var("MY_MODULE_SERVICE_URL").unwrap_or_else(|_| "http://127.0.0.1:<PORT>".to_string());
    MyModuleServiceClient::connect(url)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("my-module service unavailable: {e}")))
}

fn grpc_err(e: tonic::Status) -> (StatusCode, String) {
    (StatusCode::INTERNAL_SERVER_ERROR, e.message().to_string())
}

// Re-declare every response type the handlers return, with
// #[derive(Serialize)] (and Deserialize on request bodies), EXACTLY
// matching the field names/shape the old service module used — this
// is what keeps the REST contract byte-identical. If you used fully
// typed proto messages, write `_to_grpc`/`_from_grpc` conversion
// functions here, mirroring the ones in services/my_module/src/main.rs
// but in the opposite direction.

pub async fn get_summary(State(_state): State<AppState>) -> Result<Json<Summary>, (StatusCode, String)> {
    let mut c = client().await?;
    let resp = c.get_summary(Empty {}).await.map_err(grpc_err)?.into_inner();
    Ok(Json(/* converted resp */))
}
// ... one handler per original handler, same fn name, same signature,
// same behavior on error (same StatusCode mapping the original used —
// check whether the original distinguished 404 vs 400 vs 500 by
// message content and preserve that exact logic in the new
// grpc_err/not_found_err/bad_request_err helpers).
```

The `State(state)` parameter becomes `State(_state)` (still present,
now unused, because axum's router type still expects it — do not
remove the parameter, prefix it with `_` instead).

If the module used a JSON envelope (§4.1), the handler bodies collapse
to:

```rust
pub async fn get_summary(State(_state): State<AppState>) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let mut c = client().await?;
    let resp = c.get_summary(Empty {}).await.map_err(grpc_err)?.into_inner();
    Ok(Json(serde_json::from_str(&resp.json).unwrap_or_default()))
}
```

### 8.1 Delete the old in-process logic

The old `backend-rs/src/services/<module>.rs` is deleted outright
(its content already moved verbatim into `crates/<module>/src/lib.rs`
— `git mv` followed by edits, or a plain `rm` after copying, produces
the same result; git will often detect the rename on its own and show
it as `R` in `git status`). Remove its `pub mod <module>;` line from
`backend-rs/src/services/mod.rs`.

---

## 9. Handling discovered in-process coupling

If §1.2's grep found a real caller outside the module's own route
file (the `reply_integration` case — `reply_engine.rs` called
`ReplyIntegrationService` directly), do not let that block the split.
Instead:

1. Add the new crate (`crates/<module>`) as a normal Rust dependency of
   `backend-rs/Cargo.toml`, the same way `newax-proto` is already a
   dependency there.
2. Change the other caller's `use crate::services::<module>::...`
   import to `use newax_<module>::...` instead, and update the type
   path at each call site. The caller keeps calling the logic
   in-process — it does **not** go through gRPC — because this is an
   internal monolith call, not a cross-service call; routing it
   through gRPC would add latency and a new failure mode for zero
   benefit.
3. Document the resolution in `CONVENTIONS.md`'s tracking section (see
   the existing `reply-integration` paragraph there for the exact
   wording pattern) so the next person who hits this doesn't have to
   re-derive the same decision.

---

## 10. Verification (mandatory before every commit)

```bash
SQLX_OFFLINE=true cargo check --workspace
```

This must finish with `Finished` and **zero lines starting with
`error`** (warnings are fine and expected — the baseline is in the
hundreds and stays roughly flat; check the warning count didn't jump
unexpectedly, which usually means something didn't get wired up).

Never commit a module split that hasn't been checked this way. Never
batch two modules' changes into one `cargo check` pass and one
commit — if something fails, you need to know which module broke it.

---

## 11. Commit and push

One commit for the module split itself:

```
Split <module> out of backend-rs into a real gRPC service

<N> of ~<total> backend-rs modules now split. Port <port>, <fully
typed proto|JSON-envelope RPCs> (<one-line reason for the choice, e.g.
"all response shapes were already concrete structs" or "every route
already round-tripped through serde_json::Value">). Zero cross-module
coupling (<what you checked>); REST paths/payload shapes unchanged.
```

If §9 applied, add a paragraph naming the discovered coupling and how
it was resolved, matching the `reply-integration` commit's wording
pattern.

Push to the branch the session is already using
(`ccr-7fbf3c72-4tqu4w` as of this writing — confirm the current
branch name before pushing, it is session-assigned and will differ in
future sessions):

```bash
git push -u origin <branch>
```

---

## 12. Update `CONVENTIONS.md` in BOTH repos (separate commit)

This is not optional and not deferrable to "later" — do it in the
same work session, immediately after the module-split commit, as its
own commit. `CONVENTIONS.md` itself (unlike this playbook) stays
identical across both repos, so this is the one step in this document
that still touches nID's working copy:

1. In `CONVENTIONS.md`'s "crates/services split: in progress, tracked
   here" section:
   - Bump the "nineteen real modules split so far" count (in both the
     prose sentence and by adding a table row).
   - Add one row to the table: `| my-module | Communicator |
     \`crates/my_module\` | \`services/my_module\` | <port> |`.
   - Decrement the "~N more modules in Communicator's `backend-rs`"
     remaining count by one.
   - If §9 applied, add the discovered-coupling paragraph.
   - If §1.3 applied (skipped a non-candidate module), add a note
     about it instead of a table row, matching the existing `badges`
     note's wording.
2. Copy the edited `CONVENTIONS.md` into the nID checkout
   byte-for-byte:
   ```bash
   cp CONVENTIONS.md ../nID/CONVENTIONS.md
   diff CONVENTIONS.md ../nID/CONVENTIONS.md   # must print nothing
   ```
3. Commit and push in **both** repos, same message:
   ```
   Add <module> to CONVENTIONS.md's split-tracking table
   ```

The tracking table is the only source of truth for "what port is free
next" and "how many modules are left" — if it drifts out of sync with
reality, every subsequent split inherits the confusion. Treat updating
it as part of the module split, not a follow-up chore.

---

## 13. Final checklist

Copy this list into your own working notes for each module and tick
every line before calling the module done. Do not skip a line because
an earlier module "probably" got it right — verify this module,
every time.

- [ ] Confirmed the module has the standard candidate shape (§1.1),
      not a shared-library non-candidate (§1.3)
- [ ] Ran the zero-coupling grep (§1.2) and recorded the result
      (clean, or coupling found and resolved per §9)
- [ ] Chose fully-typed vs JSON-envelope per §4.1, based on what the
      *handlers* actually returned, not guessed
- [ ] `proto/communicator/<module>/v1/<module>.proto` written: one
      service block, RPC order matches handler order, every
      `Option<T>` field marked `optional`, every `serde_json::Value`
      field is a `string ..._json` field
- [ ] `crates/<module>/Cargo.toml` lists only the dependencies the
      moved code actually uses
- [ ] `crates/<module>/src/lib.rs` is the moved logic, taking a plain
      `PgPool` (not `AppState`), test module preserved, no bugs
      silently fixed
- [ ] `services/<module>/Cargo.toml`, `build.rs`, `src/main.rs`
      written; port taken from `CONVENTIONS.md`'s table + 1, not
      reused
- [ ] `crates/proto/src/lib.rs` and `crates/proto/build.rs` both
      updated, appended (not reordered)
- [ ] Route file rewritten as gRPC client, same handler
      names/signatures/status-code-mapping logic; old
      `services/<module>.rs` deleted; `services/mod.rs` line removed
- [ ] `cargo check --workspace` run with `SQLX_OFFLINE=true` and
      confirmed zero errors
- [ ] Module-split commit made and pushed to the session's branch
- [ ] `CONVENTIONS.md` updated in both repos (count, table row,
      remaining-count decrement, any coupling/skip note) and pushed as
      its own commit, with the two copies verified byte-identical via
      `diff`
