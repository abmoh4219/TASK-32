# SPEC.md — ScholarVault Research & Commerce Operations Portal
# Task ID: TASK-32-W2
# Single source of truth. All decisions trace back to this file.

## Original Business Prompt (Verbatim — Do Not Modify)

Construct a ScholarVault Research & Commerce Operations Portal supporting offline management of organizational knowledge assets, outcome/IP registration, and an on-premise storefront with promotions and analytics. Users sign in with a local username and password and are assigned roles as Administrator, Content Curator, Reviewer, Finance Manager, and Store Manager. In the Leptos-based web UI, Curators maintain a multi-level category tree and knowledge-point library, link knowledge points to questions in the question bank, and use fast combined filters across chapter, tags, difficulty (1–5), and discrimination bands to locate and update items. Bulk edits support up to 1,000 records per action with preview of conflicts; node merge/migration blocks changes when it would create cycles or orphan referenced nodes, and the UI shows reference counts before saving. Reviewers register outcomes by type (paper, patent, competition result, software copyright), attach evidence files (PDF/JPG/PNG) and allocate contribution shares that must total exactly 100%; duplicates are flagged during entry using title, abstract snippet, and certificate number similarity checks, with a side-by-side compare view before submission. Store Managers configure offline promotions with effective windows (MM/DD/YYYY, 12-hour time), mutual exclusion groups, and priority; at checkout the UI automatically applies the best offer and displays traceable discount details per line item. Executives view visual dashboards for member scale and churn, event participation, fund income/expense versus budget (for example a $2,500.00 monthly cap), and approval cycle time, with custom filters, CSV/PDF export, and scheduled report generation saved for later download when the user returns.
The backend uses Axum to expose REST-style endpoints consumed by the Leptos frontend, with SQLite for durable local persistence of users, roles, audit trails, category/knowledge-point graphs, question mappings, outcomes, evidence metadata, promotions, orders, and analytics snapshots. Security is enforced end-to-end with HTTPS, secure response headers, CSRF tokens for state-changing requests, XSS-safe output encoding, and strict privilege-escalation checks on every mutation, writing an immutable audit record including actor, timestamp, and before/after hashes. Authentication uses salted password hashing and account lockout after 5 failed attempts in 15 minutes; sensitive fields such as phone numbers and IDs are encrypted at rest and masked in the UI except for last 4 digits. File uploads are inspected offline using MIME/type validation plus magic-number checks and size limits (25 MB per file), rejecting mismatches and storing evidence in an encrypted local directory with content fingerprints to prevent duplicates. APIs apply rate limiting (for example 60 requests/minute per user) and basic anti-abuse controls like exponential backoff on repeated invalid searches. Backups run on a schedule set by Admin (default nightly at 2:00 AM), creating separate versioned bundles for the SQLite database and uploaded files; the system retains 30 daily versions and 12 monthly archives, supports one-click restore into a validation sandbox before activation, and applies lifecycle cleanup to purge expired archives while preserving audited financial and IP records according to an Administrator-configured retention policy.

## Project Metadata

- Task ID: TASK-32-W2
- Project Type: fullstack
- Language: Rust (both frontend and backend)
- Frontend: Leptos (Rust → WASM) + TailwindCSS + shadcn-inspired components
- Backend: Axum (Rust HTTP framework)
- Database: SQLite (via SQLx with migrations)
- Infrastructure: Docker + docker-compose (single file)
- Build: cargo-leptos for WASM compilation

> PRIORITY RULE: Original business prompt takes absolute priority over metadata.
> Metadata supports the prompt — never overrides it.

## Roles (all 5 must be implemented with distinct permissions AND work in the running app)

| Role | Key Responsibilities |
|---|---|
| Administrator | Full access, user/role management, backup/restore config, retention policy, audit log |
| Content Curator | Category tree, knowledge-point library, question bank, bulk edits, node merge |
| Reviewer | Outcome/IP registration, evidence upload, contribution shares, duplicate detection |
| Finance Manager | Fund income/expense tracking, budget management, financial dashboards, CSV/PDF export |
| Store Manager | Promotion configuration, mutual exclusion groups, checkout discount engine |

## Core Modules (all 8 must be fully implemented AND fully functional in the running Docker app)

1. **Auth & Security** — Local login, salted Argon2 hashing, account lockout (5 attempts/15min), CSRF tokens on all mutations, secure headers (HSTS, CSP, X-Frame-Options), XSS-safe output, sensitive field encryption at rest (AES-256-GCM), last-4-digit masking in UI, rate limiting (60 req/min/user), exponential backoff on repeated invalid searches
2. **Knowledge Management** — Multi-level category tree (DAG with cycle detection), knowledge-point library with tags/difficulty(1-5)/discrimination bands, question bank with knowledge-point linking, combined fast filters, bulk edit (up to 1,000 records) with conflict preview, node merge/migration with orphan detection and reference count display
3. **Outcome/IP Registration** — Register by type (paper/patent/competition/software copyright), evidence file upload (PDF/JPG/PNG, 25MB max, MIME + magic-number validation), contribution share allocation (must total exactly 100%), duplicate detection (title + abstract snippet + certificate number similarity), side-by-side compare view, encrypted evidence storage with content fingerprints
4. **Store & Promotions** — Promotion config with effective windows (MM/DD/YYYY 12-hour time), mutual exclusion groups, priority ordering, automatic best-offer selection at checkout, traceable discount details per line item
5. **Analytics & Dashboards** — Member scale/churn, event participation, fund income/expense vs budget ($2,500 cap example), approval cycle time, custom filters, CSV/PDF export, scheduled report generation saved for later download
6. **Audit Log** — Immutable records for every mutation: actor, timestamp, before/after SHA-256 hashes, privilege escalation checks on every write
7. **File Management** — MIME + magic-number validation offline, encrypted local storage directory, content fingerprint deduplication, 25MB size limit
8. **Backup & Restore** — Admin-configurable schedule (default 2:00 AM nightly), separate versioned bundles (SQLite DB + files), 30 daily + 12 monthly archives, one-click restore into validation sandbox before activation, lifecycle cleanup with configurable retention policy, preserve financial and IP records per policy

## QA Evaluation — TWO SIMULTANEOUS TESTS (both must pass — no exceptions)

### TEST 1 — Static Code Audit (automated AI reads every source file)

The QA AI auditor opens every source file and checks for file:line evidence.
This test CANNOT be passed by having working code alone — the code must ALSO be:
- Clearly structured with explicit security implementations (not just Cargo.toml deps)
- Self-evidently correct when read — security logic visible at the code level
- Free from stubs, TODOs, and unimplemented!() in production code
- Modular with clear separation of concerns across files

What the static audit specifically checks:
1. README startup/test instructions are clear and consistent with actual code structure
2. All 8 modules have real implementations — not placeholder functions
3. Security: Argon2 hashing, AES-256-GCM encryption, CSRF middleware, rate limiter, lockout — all explicitly coded with readable method bodies and doc comments
4. AuditService impl block has NO update() or delete() methods — append-only enforced at type level
5. Business rules enforced at service layer (not just frontend validation)
6. Test files exist with meaningful assertions — not empty test functions
7. No hardcoded display data in Leptos components — all from real API calls

### TEST 2 — Docker Runtime Manual Testing (human clicks through every page)

The QA team runs EXACTLY these commands and tests:
git clone <repo>
cd repo
docker compose up --build
→ Opens http://localhost:3000
→ Logs in with all 5 credentials from README
→ Clicks through EVERY page for EVERY role
→ Tests EVERY feature end-to-end
→ Verifies data persists in SQLite across operations
→ Verifies no blank pages, 500 errors, or broken functionality

What the manual test specifically checks:
1. `docker compose up --build` builds and starts with no errors — zero manual setup
2. Login page loads with golden-gradient dark SaaS UI
3. All 5 credentials log in and route to correct role-specific dashboards
4. **As Administrator:** create a user → change a role → run a backup → view audit log → configure retention policy
5. **As Content Curator:** create a category → add a knowledge point → link a question → test bulk edit → test node merge with reference count display
6. **As Reviewer:** register an outcome (paper) → upload evidence file → allocate contribution shares → verify 100% enforcement → submit → see duplicate warning if applicable
7. **As Finance Manager:** view analytics dashboard with real charts → export CSV → schedule a report → download report when ready
8. **As Store Manager:** create a promotion with time window → test mutual exclusion → complete a checkout → verify discount trace per line item
9. Every form submits to the real API — no forms that look functional but do nothing
10. Every table shows real data from SQLite — no hardcoded placeholder rows
11. Every page has loading state, empty state, and error state visually handled
12. Golden-gradient dark SaaS UI theme is consistent across all pages

### Definition of Pass

PASS = Test 1 PASSES AND Test 2 PASSES simultaneously.

FAIL scenarios (any one of these = overall FAIL):
- Code compiles but any page in the Docker app is broken, blank, or shows an error
- App runs beautifully but security is only in Cargo.toml — not explicitly coded
- Any of the 5 role logins does not work or routes to wrong page
- Any feature listed in Core Modules is a stub or placeholder
- AuditService has an update() or delete() method anywhere in the codebase
- Any form submits but does nothing (no API call, no data persisted)
- Any table shows hardcoded mock data instead of real SQLite data
- Tests exist but are empty functions with no assertions

## Non-Negotiable Delivery Rules

- Single `docker-compose.yml` — one file only (app service + test service with profile)
- `docker compose up --build` → app at http://localhost:3000 with no manual steps
- `docker compose --profile test run --build test` → runs `run_tests.sh` via Docker (4 suites)
- `run_tests.sh` — Docker-first; also runnable locally if Rust toolchain present
- `run_tests.sh` uses system `cargo` — NOT a wrapper script that may not exist
- `.env.example` committed to git — auto-copied to `.env` by Docker setup service on first run
- `.env` in `.gitignore` — `.env.example` is NOT ignored
- Minimal README: Run / Test / Stop / Login credentials only
- All code inside `repo/`
- No manual setup of any kind required after `git clone`
