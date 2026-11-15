# Schema reference (modeling notation only)

These `.prisma` files are **not** a live Prisma ORM schema and nothing
here is executed by Prisma Client — Communicator has no `@prisma/client`
dependency anywhere in this repo (confirmed, see the tech-stack
compliance audit in `CLAUDE.md`/`AGENT.md`). The `.prisma` syntax is
borrowed purely as a concise, readable schema-modeling notation for
the ERP domain model this repo's commerce features talk to.

The actual, executed database schema lives in
`backend-rs/migrations/*.sql` (SQLx/PostgreSQL). If a model here and a
migration disagree, the migration is correct and this directory is
stale — update it to match, don't assume the reverse.

This directory was previously `erp-schema/` at the repo root (plus a
duplicate copy under `backend-rs/erp-schema/`, now removed). Moved
here per `CONVENTIONS.md` so the `.prisma` extension doesn't imply a
live ORM dependency that doesn't exist.
