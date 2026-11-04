# Purging the fake rows the seeders already wrote

`purge_fake_rows.sql` deletes the fabricated demo/placeholder rows that the
(now removed) Python seeders and the still-present Rust seeders inserted into
real product tables of the SQLite database `backend-rs/communicator.db`.

Nothing has been executed. No database was opened, written to, or migrated while
this script was authored — every predicate comes from reading the seeders' own
literal values (Python code recovered with `git show HEAD:<path>`, Rust literals
read directly from `backend-rs/src/routes/*.rs`).

## Order of operations (important)

1. **Remove the Rust seeders first.** If you purge the rows while the Rust
   helpers still run, the tables refill themselves the next time a list endpoint
   is called. As of this writing the remaining row-writing seeders are:

   | File | Function | Fires on |
   | --- | --- | --- |
   | `backend-rs/src/routes/broadcasts.rs:859` | `seed_default_campaigns` | `POST /broadcasts/seed-defaults` (wired in `main.rs:182`) |
   | `backend-rs/src/routes/automation.rs:233` | `seed_default_automations` | `POST /automation-rules/seed-defaults` (wired in `main.rs:218`) |
   | `backend-rs/src/routes/canned_macros_sla.rs:40` | `ensure_default_canned` | `GET` canned responses |
   | `backend-rs/src/routes/canned_macros_sla.rs:485` | `ensure_default_macros` | `GET` macros |
   | `backend-rs/src/routes/canned_macros_sla.rs:1428` | `ensure_default_sla_policies` | `GET` SLA policies |
   | `backend-rs/src/routes/gateways_webhooks_stripe.rs:1563` | `seed_default_gateways_if_empty` | `GET` gateways |
   | `backend-rs/src/routes/gateways_webhooks_stripe.rs:1611` | `seed_default_webhooks_if_empty` | `GET` webhooks |
   | `backend-rs/src/routes/services_sequences.rs:29` | inline `if services.is_empty()` | `GET` services |
   | `backend-rs/src/routes/services_sequences.rs:140` | inline `if seqs.is_empty()` | `GET` sequences |
   | `backend-rs/src/routes/team_routing.rs:456` | `seed_default_teams` (called at `:516`) | `GET` teams (only if the `teams` table exists — see PART C) |

   These files are **not** in the ownership scope of the seeder-removal work, so
   they were left untouched on purpose.

2. **Stop the Rust server** (`communicator.db` is opened by the running process;
   writing while it is live risks a locked/WAL split-brain purge).

3. **Back the database up** — the rollback plan is this copy:

   ```sh
   cp backend-rs/communicator.db backend-rs/communicator.db.bak-before-purge
   ```

4. **Dry-run the predicates first** (read-only URI, cannot write):

   ```sh
   sqlite3 "file:backend-rs/communicator.db?mode=ro" \
     "SELECT 'broadcasts', COUNT(*) FROM broadcasts WHERE name IN
      ('Spring sale early access','New store opening','Loyalty points reminder')
      UNION ALL SELECT 'tickets', COUNT(*) FROM tickets WHERE ticket_number LIKE 'TCK-10%'
      UNION ALL SELECT 'canned', COUNT(*) FROM canned_responses WHERE short_code IN
      ('greet','pricing','hours','refund','followup','closing');"
   ```

   Any count that looks wrong (e.g. a campaign an operator genuinely reused)
   means you should delete that single statement from the script before applying.

5. **Apply** (from the repository root, `backend-rs/` one level up):

   ```sh
   sqlite3 backend-rs/communicator.db < backend-rs/sql/purge_fake_rows.sql
   ```

   PowerShell / cmd.exe equivalents:

   ```powershell
   Get-Content backend-rs\sql\purge_fake_rows.sql | sqlite3 backend-rs\communicator.db
   ```

   The script is wrapped in `BEGIN; ... COMMIT;`, so any statement that fails
   aborts the whole purge and leaves the database untouched.

6. **Verify.** Uncomment the `-- VERIFY` block at the bottom of the script (or
   paste it into the CLI) and run it. Every count in the first query must be `0`.
   The second query shows what is left in each purged table — that is now real,
   human-created data, and `0` is a perfectly valid answer.

   ```sh
   sqlite3 backend-rs/communicator.db < <(grep -v '^--' backend-rs/sql/purge_fake_rows.sql | tail -40)
   ```

7. **Rollback** (if anything looks wrong):

   ```sh
   cp backend-rs/communicator.db.bak-before-purge backend-rs/communicator.db
   ```

## What the script deliberately does *not* delete

- `role_definitions` system roles — the authz permission model, not demo data.
- The lazy `stripe_configs` default row (empty keys) and `tenants` rows written
  from a real `POST /organizations` body.
- Rows produced by still-live fabrication paths (simulated broadcast dispatch
  `meta_sim_%`, `sender_name='Campaign Broadcast'`, `inactivity_nudge` bot
  messages, synthetic `conv_%` gateway-test conversations, `Riley Brooks`
  default display names). These are listed in **PART B** as review-first
  commented statements, because a customer may treat them as real traffic.
- Tables the Rust schema never creates (`teams`, `team_members`,
  `team_memberships`, `knowledge_base_entries`, `knowledge_recommendations`,
  `knowledge_gaps`, `segments`, `contact_notes`, `drafts`). Their purge
  predicates are in **PART C**, commented out; enable them only after
  `sqlite3 backend-rs/communicator.db ".tables"` proves the table exists.

## Python / Postgres

The deleted Python seeders wrote to Postgres
(`postgresql+asyncpg://postgres:development-only@localhost:5432/communicator`),
not to this SQLite file, so the legacy database needs the same predicates. See
**PART D** of the script for the psql invocation and the table-name differences.
The Python seeders and the Rust seeders used identical literal values, which is
why one predicate list covers both.

## Known false-positive risks

- `automation_rules` is matched by both `trigger_keyword` **and** `name`, but an
  operator who rebuilt the same menu rules would still match. Check first.
- `ai_custom_policies.job_type` is `UNIQUE`: the DELETE would also remove a
  hand-written policy for those four job types. Verify before applying.
- `channel_gateway_connections` is additionally matched by credential literals
  containing `mock`; rename-only rows are safe, but a real credential that
  happens to contain one of those exact tokens would be deleted (never observed).
- `ticket_activities.author_name` includes the generic `'Sarah (Support)'` /
  `'David (Finance)'` display names — if a real agent shares one, scope that
  statement to the `ticket_id` join only.
