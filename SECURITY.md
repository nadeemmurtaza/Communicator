# Security Policy

## Supported Versions

Communicator is pre-1.0 and under active development. Security fixes are
applied to the `main` branch only -- there are no maintained release
branches yet.

| Version | Supported |
| ------- | --------- |
| `main`  | ✅ |

## Reporting a Vulnerability

Please **do not** open a public GitHub issue for a security vulnerability.

Report it privately instead:

- Preferred, if enabled for this repository: GitHub's private
  vulnerability reporting, under the repository's **Security** tab →
  **Report a vulnerability**.
- Otherwise: email dr.mnadeemmurtaza@gmail.com with details.

Please include, where possible:
- A description of the vulnerability and its potential impact.
- Steps to reproduce (a minimal proof of concept helps).
- Relevant logs, stack traces, or affected file/line references.

We aim to acknowledge reports within 5 business days and to follow up
with a timeline once the issue is confirmed.

## Scope

Communicator holds lead contact information (email addresses, phone
numbers) and message content on behalf of leads sourced from Harvester.
Areas of particular interest for security review:

- **Webhook authentication** (`src/communicator/webhooks/`,
  `src/communicator/api/routes/webhooks.py`) -- anything that would let an
  attacker forge an inbound message, replay a request, or bypass the
  Twilio signature / Postmark token checks.
- **The consent/compliance gate** (`src/communicator/compliance/optout.py`,
  `src/communicator/services/messaging.py`) -- anything that would let a
  message be sent to a contact who has opted out, or that would let
  consent state be spoofed via a forged inbound message.
- **Credential and secret handling** (`src/communicator/config.py`,
  `.env` / `.env.example`) -- provider API keys, webhook shared secrets,
  database credentials, and how they flow into logs or error messages.
- **Cross-service trust** (`src/communicator/harvester_client.py`) -- the
  boundary between Communicator and Harvester's API.

## Automated Scanning

This repository runs [CodeQL](.github/workflows/codeql.yml) and
[Semgrep](.github/workflows/semgrep.yml) on every pull request and on a
weekly schedule; findings surface as GitHub code scanning alerts.
[Dependabot](.github/dependabot.yml) keeps Python and GitHub Actions
dependencies patched on a weekly cadence.
