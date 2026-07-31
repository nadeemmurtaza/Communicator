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
