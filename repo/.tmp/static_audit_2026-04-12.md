# ScholarVault Static Delivery Acceptance & Architecture Audit

Date: 2026-04-12
Reviewer mode: Static-only (no runtime execution)

## 1. Verdict

**Overall conclusion: Fail**

Rationale (static): material Prompt/security deviations remain in delivered behavior, notably object-level authorization gaps on outcome mutations, incomplete immutable-audit hash coverage, and missing reviewer evidence-upload UX flow required by the Prompt.

---

## 2. Scope and Static Verification Boundary

### What was reviewed

- Documentation and manifests: `README.md`, workspace/backend/frontend Cargo manifests, migration files, shell test harness.
- Backend entrypoints/router/middleware/services/handlers/models/migrations.
- Frontend Leptos routes/pages/API clients/styles.
- Backend + frontend unit/API test sources.

### What was not reviewed

- Runtime behavior under real browser/network/container/process conditions.
- DB/file-system effects at execution time.
- Docker orchestration behavior.

### Intentionally not executed

- Project start, tests, docker compose, external services (per instruction).

### Claims requiring manual verification

- End-to-end HTTPS transport in actual deployment (cert/proxy/termination behavior).
- Browser interaction details (actual CSV/PDF download UX, visual rendering fidelity).
- Real upload/download filesystem permissions and concurrency behavior.

---

## 3. Repository / Requirement Mapping Summary

Prompt core mapped areas:

- Knowledge graph + filtering + bulk edits: `backend/src/services/knowledge_service.rs`, `frontend/src/pages/knowledge/*`.
- Outcomes/IP registration + duplicates + evidence: `backend/src/services/outcome_service.rs`, `backend/src/services/file_service.rs`, `frontend/src/pages/outcomes/*`.
- Store promotions/checkout best-offer: `backend/src/services/store_service.rs`, `frontend/src/pages/store/*`.
- Analytics dashboards/export/scheduled reports: `backend/src/services/analytics_service.rs`, `frontend/src/pages/analytics/*`.
- Security (auth/CSRF/rate-limit/headers/roles): `backend/src/middleware/*`, `backend/src/services/auth_service.rs`, `backend/src/router.rs`.
- Backups/retention/sandbox restore: `backend/src/services/backup_service.rs`, backup migrations.

---

## 4. Section-by-section Review

### 1. Hard Gates

#### 1.1 Documentation and static verifiability

- **Conclusion: Partial Pass**
- **Rationale:** Startup/test instructions exist and entrypoints are statically consistent, but docs are narrow (Docker-first) and do not clearly document non-container verification path despite local `cargo` test harness existing.
- **Evidence:** `README.md:5`, `README.md:11`, `backend/src/main.rs:1`, `backend/src/router.rs:32`, `run_tests.sh:33`, `run_tests.sh:40`
- **Manual verification note:** Runtime setup correctness still needs manual execution.

#### 1.2 Material deviation from Prompt

- **Conclusion: Fail**
- **Rationale:** Required reviewer evidence attachment flow is not implemented in the registration UI/client path; dashboard route is placeholder text rather than business dashboard.
- **Evidence:** `frontend/src/pages/outcomes/register.rs:37`, `frontend/src/pages/outcomes/register.rs:67`, `frontend/src/pages/outcomes/register.rs:92`, `frontend/src/api/outcomes.rs:94`, `frontend/src/api/outcomes.rs:117`, `frontend/src/app.rs:23`, `frontend/src/app.rs:48`

### 2. Delivery Completeness

#### 2.1 Core explicit requirements coverage

- **Conclusion: Partial Pass**
- **Rationale:** Many core requirements are implemented (bulk cap, DAG checks, duplicate detection, contribution=100%, promotion engine, scheduled reports, retention policy), but critical gaps remain (UI evidence upload, strict audit hash completeness, outcome mutation object-level authorization).
- **Evidence:** `backend/src/services/knowledge_service.rs:21`, `backend/src/services/knowledge_service.rs:261`, `backend/src/services/outcome_service.rs:190`, `backend/src/services/outcome_service.rs:333`, `backend/src/services/store_service.rs:288`, `backend/src/services/analytics_service.rs:368`, `backend/src/services/backup_service.rs:451`, `backend/src/handlers/outcomes.rs:93`, `backend/src/handlers/outcomes.rs:136`

#### 2.2 Basic end-to-end deliverable from 0→1

- **Conclusion: Partial Pass**
- **Rationale:** Repo is complete multi-crate application with backend/frontend/shared, migrations, tests, and docs; however, missing required flow details and security constraints prevent full acceptance.
- **Evidence:** `Cargo.toml:1`, `backend/Cargo.toml:1`, `frontend/Cargo.toml:1`, `backend/src/db/migrations/0001_create_users.sql:1`, `backend/tests/api_tests/mod.rs:1`, `frontend/tests/unit_tests/mod.rs:1`

### 3. Engineering and Architecture Quality

#### 3.1 Structure and module decomposition

- **Conclusion: Pass**
- **Rationale:** Clear layered architecture (router/handlers/services/middleware/models), separate frontend API/page logic, scoped modules by business domain.
- **Evidence:** `backend/src/router.rs:32`, `backend/src/handlers/knowledge.rs:1`, `backend/src/services/knowledge_service.rs:1`, `backend/src/middleware/require_role.rs:1`, `frontend/src/pages/mod.rs:1`, `frontend/src/api/client.rs:1`

#### 3.2 Maintainability and extensibility

- **Conclusion: Partial Pass**
- **Rationale:** Service-layer decomposition and reusable pure functions/tests are strong; but security/business invariants are unevenly enforced across mutation paths.
- **Evidence:** `backend/src/services/store_service.rs:288`, `backend/src/services/analytics_service.rs:368`, `backend/src/services/audit_service.rs:24`, `backend/src/handlers/outcomes.rs:93`, `backend/src/services/outcome_service.rs:245`

### 4. Engineering Details and Professionalism

#### 4.1 Error handling/logging/validation/API design

- **Conclusion: Partial Pass**
- **Rationale:** Strong typed errors and response sanitization exist; validation appears extensive in many flows; logging categories exist. But high-severity authorization and audit-invariant defects remain.
- **Evidence:** `backend/src/error.rs:67`, `backend/src/error.rs:110`, `backend/src/services/file_service.rs:18`, `backend/src/services/file_service.rs:50`, `backend/src/main.rs:47`, `backend/src/services/backup_scheduler.rs:36`, `backend/src/handlers/outcomes.rs:93`

#### 4.2 Product/service readiness vs demo shape

- **Conclusion: Partial Pass**
- **Rationale:** Most modules are product-like; however one top-level route remains placeholder and some required UX flow pieces are absent.
- **Evidence:** `frontend/src/app.rs:23`, `frontend/src/app.rs:36`, `frontend/src/app.rs:48`

### 5. Prompt Understanding and Requirement Fit

#### 5.1 Business-goal fit and constraints fidelity

- **Conclusion: Fail**
- **Rationale:** Core intent mostly understood, but key explicit constraints are not fully met: strict privilege-escalation checks on every mutation and immutable audit records with before/after hashes for every mutation are not consistently enforced; reviewer evidence attachment flow is incomplete in UI.
- **Evidence:** `backend/src/handlers/outcomes.rs:93`, `backend/src/services/outcome_service.rs:245`, `backend/src/handlers/auth.rs:151`, `backend/src/handlers/backup.rs:101`, `backend/src/handlers/knowledge.rs:398`, `frontend/src/pages/outcomes/register.rs:37`

### 6. Aesthetics (frontend/full-stack)

#### 6.1 Visual/interaction design quality

- **Conclusion: Pass (Static), Manual Verification Required**
- **Rationale:** Theming, hierarchy, spacing, interactive states, and role-specific shells are well structured in static code/CSS. Actual rendering fidelity and cross-browser behavior require manual check.
- **Evidence:** `style/main.scss:1`, `style/main.scss:174`, `style/main.scss:426`, `style/main.scss:452`, `frontend/src/components/layout/mod.rs:1`, `frontend/src/pages/analytics/dashboard.rs:1`

---

## 5. Issues / Suggestions (Severity-Rated)

### High

1. **Outcome mutation endpoints lack object-level ownership/assignment authorization**

- **Conclusion:** Fail
- **Evidence:** `backend/src/handlers/outcomes.rs:93`, `backend/src/handlers/outcomes.rs:136`, `backend/src/handlers/outcomes.rs:162`, `backend/src/services/outcome_service.rs:245`, `backend/src/services/outcome_service.rs:324`
- **Impact:** Any user with Reviewer role can modify/submit/approve/reject outcomes by ID, violating strict privilege boundary expectations and increasing unauthorized data/process manipulation risk.
- **Minimum actionable fix:** Add scoped authorization checks for all outcome mutations (creator/contributor/workflow-assignee/admin policy), at service layer and handler layer; return `403` on violations.

2. **Immutable audit requirement not met: before/after hashes are optional and often omitted on mutations**

- **Conclusion:** Fail
- **Evidence:** `backend/src/services/audit_service.rs:35`, `backend/src/handlers/auth.rs:151`, `backend/src/handlers/backup.rs:101`, `backend/src/handlers/knowledge.rs:398`
- **Impact:** Fails Prompt requirement for immutable mutation trail including before/after hashes; weakens forensic integrity.
- **Minimum actionable fix:** Enforce non-null before/after hash policy for all mutation actions (or explicit standardized sentinel for create/delete where one side is absent), and add guard tests ensuring no mutation logs with both hashes missing.

3. **Reviewer evidence upload flow is missing from primary registration UX/API client path**

- **Conclusion:** Fail
- **Evidence:** `frontend/src/pages/outcomes/register.rs:37`, `frontend/src/pages/outcomes/register.rs:67`, `frontend/src/pages/outcomes/register.rs:92`, `frontend/src/api/outcomes.rs:94`, `frontend/src/api/outcomes.rs:117`, `backend/src/handlers/outcomes.rs:206`
- **Impact:** Prompt-required “attach evidence files (PDF/JPG/PNG)” during registration workflow is not fully delivered in frontend.
- **Minimum actionable fix:** Add file input + multipart upload action to registration step (or explicit post-create evidence step before submit), and expose corresponding frontend API helper.

### Medium

4. **CSV/PDF direct export controls in frontend are method/payload-mismatched with backend**

- **Conclusion:** Partial Fail
- **Evidence:** `backend/src/router.rs:164`, `backend/src/handlers/analytics.rs:73`, `backend/src/handlers/analytics.rs:76`, `frontend/src/pages/analytics/reports.rs:106`, `frontend/src/pages/analytics/reports.rs:114`, `frontend/src/api/analytics.rs:152`
- **Impact:** Frontend anchors issue GET navigation while backend expects POST JSON (`report_type`, `period`), likely breaking direct export UX.
- **Minimum actionable fix:** Replace anchor links with POST request/submit mechanism (or add backend GET equivalents with query params) and validate download behavior.

5. **Hardcoded seeded credentials are documented and globally known**

- **Conclusion:** Risk
- **Evidence:** `README.md:23`, `README.md:24`, `README.md:25`, `README.md:26`, `README.md:27`, `backend/src/db/migrations/0010_seed_users.sql:1`
- **Impact:** Elevated risk if deployed beyond controlled local/dev environment.
- **Minimum actionable fix:** Gate seeding to explicit dev/test mode, force first-login password rotation, and remove real credential table from default README.

6. **Top-level `/dashboard` remains placeholder rather than business dashboard**

- **Conclusion:** Partial Fail
- **Evidence:** `frontend/src/app.rs:23`, `frontend/src/app.rs:36`, `frontend/src/app.rs:48`
- **Impact:** Weakens perceived end-to-end completeness for a central route.
- **Minimum actionable fix:** Route `/dashboard` to role-aware dashboard landing with real metrics/actions.

---

## 6. Security Review Summary

- **Authentication entry points — Pass**
  - Evidence: `backend/src/router.rs:35`, `backend/src/handlers/auth.rs:64`, `backend/src/services/auth_service.rs:41`, `backend/src/services/auth_service.rs:55`
  - Notes: Username/password with lockout policy statically implemented.

- **Route-level authorization — Partial Pass**
  - Evidence: `backend/src/middleware/require_role.rs:70`, `backend/src/middleware/require_role.rs:78`, `backend/src/handlers/backup.rs:33`, `backend/src/handlers/analytics.rs:24`
  - Notes: Strong role extractors; however route-level role checks are insufficient without object-level checks on certain mutations.

- **Object-level authorization — Fail**
  - Evidence: Positive example `backend/src/handlers/store.rs:166`, `backend/src/services/outcome_service.rs:120`; gap `backend/src/handlers/outcomes.rs:93`, `backend/src/services/outcome_service.rs:245`
  - Notes: Some reads are scoped; key outcome mutations are not scoped per object ownership/assignment.

- **Function-level authorization — Partial Pass**
  - Evidence: `backend/src/handlers/knowledge.rs:44`, `backend/src/handlers/store.rs:31`, `backend/src/handlers/outcomes.rs:93`
  - Notes: Most mutators require role extractors; fine-grained mutation constraints are incomplete in outcomes flow.

- **Tenant/user data isolation — Partial Pass**
  - Evidence: `backend/src/handlers/store.rs:147`, `backend/src/services/analytics_service.rs:422`, `backend/src/services/outcome_service.rs:143`
  - Notes: Isolation present in several paths; mutation-scope gaps still allow cross-record actions by same role.

- **Admin/internal/debug endpoint protection — Pass**
  - Evidence: `backend/src/router.rs:211`, `backend/src/router.rs:222`, `backend/src/handlers/backup.rs:33`, `backend/src/handlers/auth.rs:210`
  - Notes: Admin routes are role-gated; no obvious open debug endpoints found.

---

## 7. Tests and Logging Review

- **Unit tests — Pass (static existence and relevance)**
  - Evidence: `backend/tests/unit_tests/mod.rs:1`, `frontend/tests/unit_tests/mod.rs:1`, `backend/tests/unit_tests/outcome_tests.rs:1`, `frontend/tests/unit_tests/promotion_tests.rs:1`

- **API/integration tests — Partial Pass**
  - Evidence: `backend/tests/api_tests/mod.rs:1`, `backend/tests/api_tests/auth_api.rs:10`, `backend/tests/api_tests/outcome_api.rs:37`, `backend/tests/api_tests/analytics_api.rs:1`
  - Notes: Good breadth; notable gap around approve/reject object-level authorization tests.

- **Logging categories/observability — Partial Pass**
  - Evidence: `backend/src/main.rs:47`, `backend/src/services/backup_scheduler.rs:36`, `backend/src/error.rs:110`
  - Notes: Structured logging present; no centralized request audit correlation ID observed statically.

- **Sensitive-data leakage risk in logs/responses — Partial Pass**
  - Evidence: `backend/src/error.rs:110`, `backend/src/error.rs:113`, `backend/src/handlers/auth.rs:189`
  - Notes: Internal/DB errors are sanitized in responses; manual verification required for runtime log sinks and redaction policy completeness.

---

## 8. Test Coverage Assessment (Static Audit)

### 8.1 Test Overview

- Unit tests exist: backend and frontend (`backend/tests/unit_tests/mod.rs:1`, `frontend/tests/unit_tests/mod.rs:1`).
- API/integration tests exist: backend API suite (`backend/tests/api_tests/mod.rs:1`).
- Frameworks observed: Rust test harness + `tokio::test` + Axum `oneshot` style integration (`backend/tests/api_tests/auth_api.rs:10`).
- Test entry points documented via shell harness (`run_tests.sh:33`, `run_tests.sh:40`).
- README test command documented (Docker profile) (`README.md:11`).

### 8.2 Coverage Mapping Table

| Requirement / Risk Point                           | Mapped Test Case(s)                                                                                                                             | Key Assertion / Fixture / Mock                                                 | Coverage Assessment | Gap                                           | Minimum Test Addition                                                                                   |
| -------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------ | ------------------- | --------------------------------------------- | ------------------------------------------------------------------------------------------------------- |
| Auth login + lockout 5/15                          | `backend/tests/api_tests/auth_api.rs:10`, `backend/tests/api_tests/auth_api.rs:108`                                                             | 200 on valid login; 6th attempt => `LOCKED`                                    | sufficient          | none major                                    | add lockout expiry window test                                                                          |
| CSRF required for mutations                        | `backend/tests/api_tests/auth_api.rs:47`, `backend/tests/api_tests/auth_api.rs:61`                                                              | 403 without token; success with valid token                                    | sufficient          | none major                                    | add cross-route CSRF matrix                                                                             |
| Knowledge DAG cycle + bulk cap                     | `backend/tests/api_tests/knowledge_api.rs:106`, `backend/tests/api_tests/knowledge_api.rs:213`; `backend/tests/unit_tests/knowledge_tests.rs:1` | merge cycle returns 409; bulk >1000 returns 400                                | sufficient          | none major                                    | add orphan-reference migration case                                                                     |
| Outcome contribution total=100                     | `backend/tests/api_tests/outcome_api.rs:91`, `backend/tests/unit_tests/outcome_tests.rs:37`                                                     | submit fails when total != 100                                                 | sufficient          | none major                                    | add boundary case with contributor edits                                                                |
| Evidence duplicate fingerprint                     | `backend/tests/api_tests/outcome_api.rs:136`                                                                                                    | second upload -> 409                                                           | basically covered   | no UI flow coverage                           | add frontend workflow test with evidence before submit                                                  |
| Store checkout best offer                          | `backend/tests/api_tests/store_api.rs:34`, `backend/tests/unit_tests/store_tests.rs:1`                                                          | discount applied, line trace populated                                         | sufficient          | none major                                    | add equal-discount tie-breaker test                                                                     |
| Analytics scheduled report + single-use token      | `backend/tests/api_tests/analytics_api.rs:111`, `backend/tests/api_tests/analytics_api.rs:139`                                                  | second download 404                                                            | sufficient          | none major                                    | add token replay after ownership change                                                                 |
| Backup schedule/policy + restore                   | `backend/tests/api_tests/backup_api.rs:164`, `backend/tests/api_tests/backup_api.rs:228`                                                        | schedule persisted; activate restore overwrites live file                      | sufficient          | none major                                    | add concurrent restore safety test                                                                      |
| Outcome object-level mutation authorization        | (no dedicated test found)                                                                                                                       | endpoints exist `backend/src/router.rs:116`, `backend/src/router.rs:120`       | missing             | severe security gap can survive current tests | add tests where reviewer A attempts mutate reviewer B outcome (add/submit/approve/reject) expecting 403 |
| Audit before/after hash presence on every mutation | (no dedicated invariant test found)                                                                                                             | service allows nullable hash fields `backend/src/services/audit_service.rs:35` | missing             | Prompt requires strict hash traceability      | add invariant test: every mutating API writes non-empty before/after according to action semantics      |

### 8.3 Security Coverage Audit

- **authentication:** covered well (login, wrong password, lockout, CSRF bootstrap) — sufficient.
- **route authorization:** covered for multiple modules (knowledge/store/analytics/backup) — basically covered.
- **object-level authorization:** partially covered (store/order read, outcome read scope) but **not** for outcome mutations — insufficient.
- **tenant/data isolation:** partially covered (report download ownership, order ownership) — insufficient for outcome mutation paths.
- **admin/internal protection:** covered for backup/admin paths — basically covered.

### 8.4 Final Coverage Judgment

**Partial Pass**

Major risks covered: auth/CSRF/lockout, knowledge constraints, promotion engine, scheduled report token, backup core flows.

Critical uncovered risks: outcome mutation object-level authorization and audit-hash completeness invariants. Current test suites could still pass while severe authorization/forensics defects remain.

---

## 9. Final Notes

- This report is static-only and evidence-traceable; no runtime success is claimed.
- Manual verification remains required for deployment HTTPS behavior, browser download behavior, and full UI runtime interactions.
- Primary acceptance blockers are security/requirement-fit defects, not project structure depth.
