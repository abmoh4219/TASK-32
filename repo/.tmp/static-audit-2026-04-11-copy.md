# Static Delivery Acceptance & Project Architecture Audit

Date: 2026-04-11  
Scope root: `./` (current working directory only)

## 1. Verdict

**Overall conclusion: Fail**

Primary blockers/high-risk findings:

- Backup “activation” does not perform restore; it only re-validates and timestamps (`backend/src/services/backup_service.rs:290-298`).
- Contributor deletion is not scoped to `outcome_id`, allowing cross-record deletion by contributor id (`backend/src/handlers/outcomes.rs:115-121`, `backend/src/services/outcome_service.rs:300-302`).
- Admin cannot configure backup schedule via API (prompt requires admin-configurable schedule; implementation is env/startup only) (`backend/src/main.rs:87-88`, `backend/src/router.rs:188-200`).

## 2. Scope and Static Verification Boundary

### What was reviewed

- Docs/manifests/config: `README.md:5-11`, `.env.example:1-14`, `Cargo.toml` workspace, `run_tests.sh:1-66`.
- Backend architecture and behavior: `backend/src/main.rs`, `router.rs`, `middleware/*.rs`, `handlers/*.rs`, `services/*.rs`, `db/migrations/*.sql`.
- Frontend architecture and requirement-fit surfaces: `frontend/src/app.rs`, `frontend/src/pages/**`, `frontend/src/api/**`, `style/main.scss`.
- Test assets (static audit only): `backend/tests/**`, `frontend/tests/**`.

### What was not reviewed

- Runtime behavior, browser rendering/runtime interactions, Docker/network orchestration, and timing/clock behavior under real deployment conditions.

### What was intentionally not executed

- Project startup, Docker, tests, and external services (per audit constraints).

### Claims requiring manual verification

- True HTTPS termination chain and certificate setup in deployment.
- End-to-end restore behavior in a live environment with real persisted DB/file artifacts.
- UI visual fidelity/accessibility and interaction feedback under browser execution.

## 3. Repository / Requirement Mapping Summary

- **Prompt core objective mapped:** offline ScholarVault portal across knowledge management, outcomes/IP registration, storefront promotions/checkout, analytics/export/reporting, and backup lifecycle.
- **Implementation areas mapped:**
  - Auth/security/middleware: `backend/src/router.rs:35-38`, `backend/src/middleware/require_role.rs:70-78`, `backend/src/services/auth_service.rs:41-55`.
  - Domain modules: knowledge, outcomes, store, analytics, backup (`backend/src/router.rs:40-200`).
  - Frontend role pages and APIs: `frontend/src/app.rs:19-27`, `frontend/src/pages/**`, `frontend/src/api/**`.
  - Persistence: SQL migrations for users/knowledge/outcomes/store/analytics/backup.

## 4. Section-by-section Review

### 4.1 Hard Gates

#### 4.1.1 Documentation and static verifiability

- **Conclusion:** Pass
- **Rationale:** Startup/test/config entry points are documented and statically consistent with repository structure and scripts.
- **Evidence:** `README.md:5`, `README.md:11`, `.env.example:1-14`, `run_tests.sh:20-48`.
- **Manual verification note:** Runtime correctness still requires manual execution.

#### 4.1.2 Material deviation from Prompt

- **Conclusion:** Fail
- **Rationale:** Core prompt semantics are materially weakened in backup/restore lifecycle and object-level mutation safety.
- **Evidence:** `backend/src/services/backup_service.rs:290-298`, `backend/src/main.rs:87-88`, `backend/src/handlers/outcomes.rs:115-121`, `backend/src/services/outcome_service.rs:300-302`.

### 4.2 Delivery Completeness

#### 4.2.1 Core requirement coverage

- **Conclusion:** Partial Pass
- **Rationale:** Most core domains are implemented (knowledge/outcomes/store/analytics/security/backups), but several explicit constraints are not fully satisfied (admin-config backup scheduling, real activation restore semantics, robust retention preservation semantics).
- **Evidence:** `backend/src/router.rs:40-200`, `backend/src/services/backup_service.rs:290-298`, `backend/src/services/backup_service.rs:342-346`, `backend/src/main.rs:87-88`.

#### 4.2.2 0→1 end-to-end deliverable shape

- **Conclusion:** Pass
- **Rationale:** Multi-crate full-stack structure with migrations, APIs, frontend pages, and test suites is present; not a fragment/demo-only drop.
- **Evidence:** `Cargo.toml:1-3`, `backend/src/db/migrations/0001_create_users.sql:1`, `backend/tests/api_tests/mod.rs:1-9`, `frontend/tests/api_tests/mod.rs:1-8`.

### 4.3 Engineering and Architecture Quality

#### 4.3.1 Structure and module decomposition

- **Conclusion:** Pass
- **Rationale:** Clear separation across middleware, handlers, services, models, and frontend API/page layers.
- **Evidence:** `backend/src/lib.rs:7-16`, `backend/src/router.rs:1-33`, `frontend/src/pages/mod.rs:5-12`, `frontend/src/api/client.rs:1-18`.

#### 4.3.2 Maintainability and extensibility

- **Conclusion:** Partial Pass
- **Rationale:** Generally maintainable, but policy enforcement is mixed across layers and includes brittle heuristics (retention preservation by filename substring).
- **Evidence:** `backend/src/services/backup_service.rs:342-346`, `backend/src/handlers/store.rs:160`, `backend/src/services/analytics_service.rs:450`.

### 4.4 Engineering Details and Professionalism

#### 4.4.1 Error handling, logging, validation, API design

- **Conclusion:** Partial Pass
- **Rationale:** Strong typed error envelope and extensive validations exist; request-path observability is relatively thin and some security controls are heuristic.
- **Evidence:** `backend/src/error.rs:14-46`, `backend/src/error.rs:61-75`, `backend/src/services/file_service.rs:39-50`, `backend/src/main.rs:47`, `backend/src/services/backup_scheduler.rs:30-31`.

#### 4.4.2 Product/service realism

- **Conclusion:** Partial Pass
- **Rationale:** Product-like scope and module breadth are present, but blocker/high defects prevent delivery acceptance as prompt-compliant.
- **Evidence:** `backend/src/router.rs:40-200`, `frontend/src/app.rs:19-27`.

### 4.5 Prompt Understanding and Requirement Fit

#### 4.5.1 Business goal and implicit constraints fit

- **Conclusion:** Partial Pass
- **Rationale:** Team understood core business areas, but several strict prompt semantics are not fully met (backup schedule control and activation semantics, robust object-level mutation checks).
- **Evidence:** `backend/src/main.rs:87-88`, `backend/src/router.rs:188-200`, `backend/src/services/backup_service.rs:290-298`, `backend/src/services/outcome_service.rs:300-302`.

### 4.6 Aesthetics (frontend/full-stack)

#### 4.6.1 Visual and interaction design quality

- **Conclusion:** Cannot Confirm Statistically
- **Rationale:** Design system and page hierarchy are statically consistent, but rendered quality/feedback/responsiveness must be checked in-browser.
- **Evidence:** `style/main.scss:1`, `frontend/src/components/layout/mod.rs:68-111`, `frontend/src/pages/login.rs:43-107`.
- **Manual verification note:** Required for contrast, spacing, hover/focus states, and responsive behavior.

## 5. Issues / Suggestions (Severity-Rated)

### Blocker / High

1. **Severity:** Blocker  
   **Title:** Restore activation is functionally a no-op (no DB/file activation restore)  
   **Conclusion:** Fail  
   **Evidence:** `backend/src/services/backup_service.rs:290-298`  
   **Impact:** Violates prompt requirement for one-click restore activation after sandbox validation; system can report “activated” without restoring data.  
   **Minimum actionable fix:** Implement actual activation flow that atomically applies validated DB/files artifacts to live storage (with rollback guard and audit).

2. **Severity:** High  
   **Title:** Outcome contributor deletion is not bound to path outcome id (object-level mutation integrity gap)  
   **Conclusion:** Fail  
   **Evidence:** `backend/src/handlers/outcomes.rs:115-121`, `backend/src/services/outcome_service.rs:300-302`  
   **Impact:** A reviewer/admin with a contributor id could delete a contributor row unrelated to the path outcome, violating object-level boundary expectations.  
   **Minimum actionable fix:** Delete by `(contributor_id, outcome_id)` and return conflict/not-found on mismatch; add explicit authorization check for target outcome ownership/privilege.

3. **Severity:** High  
   **Title:** Backup schedule is not admin-configurable via application API  
   **Conclusion:** Fail  
   **Evidence:** `backend/src/main.rs:87-88`, `backend/src/router.rs:188-200`  
   **Impact:** Prompt requires admin-set schedule (default 2:00 AM). Current schedule is env/startup bound and not manageable through role-governed product flows.  
   **Minimum actionable fix:** Add admin API/UI for schedule updates persisted in DB, with scheduler reload and audit log.

4. **Severity:** High  
   **Title:** Retention preservation logic for financial/IP records relies on filename substring heuristics  
   **Conclusion:** Fail  
   **Evidence:** `backend/src/services/backup_service.rs:342-346`, `backend/src/services/backup_service.rs:101`, `backend/src/services/backup_service.rs:141`  
   **Impact:** Policy can silently fail to preserve required audited financial/IP records if filenames do not contain heuristic substrings.  
   **Minimum actionable fix:** Preserve by structured metadata/classification flags stored in DB, not path substring matching.

### Medium / Low

5. **Severity:** Medium  
   **Title:** Rate-limit keying trusts `X-Forwarded-For` directly; spoof risk in non-trusted proxy topologies  
   **Conclusion:** Suspected Risk  
   **Evidence:** `backend/src/middleware/rate_limit.rs:72`, `backend/src/middleware/rate_limit.rs:77`  
   **Impact:** Caller-controlled headers may weaken anti-abuse controls depending on deployment edge trust configuration.  
   **Minimum actionable fix:** Only trust forwarded headers behind explicit trusted-proxy config; otherwise use socket address.

6. **Severity:** Medium  
   **Title:** End-to-end enforcement of phone/ID encrypted-at-rest + masked UI is only partially evidenced  
   **Conclusion:** Cannot Confirm Statistically  
   **Evidence:** `backend/src/models/user.rs:16-17`, `backend/src/security/encryption.rs:83`, `backend/src/services/file_service.rs:91`  
   **Impact:** Prompt explicitly requires these sensitive fields to be encrypted and masked; static evidence shows primitives/columns, but not complete user-flow enforcement.  
   **Minimum actionable fix:** Add explicit service/handler/UI flows proving storage encryption + masked presentation for these fields, with tests.

## 6. Security Review Summary

- **authentication entry points:** **Pass**  
  Evidence: Auth routes and lockout logic exist (`backend/src/router.rs:35-38`, `backend/src/services/auth_service.rs:41-55`, `backend/src/services/auth_service.rs:90`).

- **route-level authorization:** **Pass**  
  Evidence: Role extractors and route usage are broadly present (`backend/src/middleware/require_role.rs:70-78`, `backend/src/handlers/backup.rs:33`, `backend/src/handlers/knowledge.rs:40`).

- **object-level authorization:** **Partial Pass**  
  Evidence: Positive controls exist (`backend/src/handlers/store.rs:160`, `backend/src/services/outcome_service.rs:135-145`), but contributor deletion scoping flaw remains (`backend/src/services/outcome_service.rs:300-302`).

- **function-level authorization:** **Partial Pass**  
  Evidence: Extractor-based role enforcement is systematic, but not sufficient to prevent object-level misuse in all mutations (`backend/src/handlers/outcomes.rs:115-121`).

- **tenant / user data isolation:** **Pass** (single-tenant local app context)  
  Evidence: User-scoped reads enforced in key modules (`backend/src/services/outcome_service.rs:128-145`, `backend/src/handlers/store.rs:160`).  
  Note: Multi-tenant isolation is **Not Applicable** for this repository model.

- **admin / internal / debug endpoint protection:** **Pass**  
  Evidence: Admin-only endpoints guarded (`backend/src/handlers/backup.rs:33`, `backend/src/handlers/auth.rs:188`, `backend/src/handlers/auth.rs:280`).

## 7. Tests and Logging Review

- **Unit tests:** **Pass**  
  Evidence: Domain/security unit tests across auth/knowledge/outcome/store/analytics/backup (`backend/tests/unit_tests/mod.rs:4-10`, `backend/tests/unit_tests/outcome_tests.rs:44`, `backend/tests/unit_tests/knowledge_tests.rs:66`).

- **API / integration tests:** **Pass**  
  Evidence: Auth, CSRF, role gates, object access, duplicates, backup, and analytics download ownership are covered (`backend/tests/api_tests/auth_api.rs:48`, `backend/tests/api_tests/store_api.rs:136`, `backend/tests/api_tests/analytics_api.rs:211`, `backend/tests/api_tests/backup_api.rs:117`).

- **Logging categories / observability:** **Partial Pass**  
  Evidence: Structured logs present mainly in startup/scheduler paths (`backend/src/main.rs:47`, `backend/src/services/backup_scheduler.rs:30-31`). Request-level telemetry is limited.

- **Sensitive-data leakage risk in logs / responses:** **Partial Pass**  
  Evidence: Audit logs store hashes rather than payload snapshots (`backend/src/services/audit_service.rs:57-61`), but explicit log-redaction policy/tests are not present (Cannot Confirm Statistically).

## 8. Test Coverage Assessment (Static Audit)

### 8.1 Test Overview

- Unit and API/integration tests both exist for backend and frontend client/logic layers.
- Framework: Rust `cargo test` (`tokio::test` for async API/service tests).
- Entry points: `backend/tests/unit_tests/mod.rs:4-10`, `backend/tests/api_tests/mod.rs:4-9`, `frontend/tests/unit_tests/mod.rs:4-7`, `frontend/tests/api_tests/mod.rs:4-7`.
- Documented commands exist: `README.md:11`, `run_tests.sh:20-48`.
- Static-only boundary: tests were **not executed** in this audit.

### 8.2 Coverage Mapping Table

| Requirement / Risk Point                         | Mapped Test Case(s)                                                                                     | Key Assertion / Fixture                                          | Coverage Assessment | Gap                                                    | Minimum Test Addition                                             |
| ------------------------------------------------ | ------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------- | ------------------- | ------------------------------------------------------ | ----------------------------------------------------------------- |
| Login + lockout (5 in 15m)                       | `backend/tests/api_tests/auth_api.rs:12`, `:110`                                                        | 200 on valid login; `423 LOCKED` after failures                  | sufficient          | None material                                          | Add IP-rotation bypass test                                       |
| CSRF enforcement on mutation                     | `backend/tests/api_tests/auth_api.rs:48`                                                                | POST without token returns 403                                   | sufficient          | None material                                          | Add token mismatch negative test                                  |
| Knowledge cycle prevention + 1000 bulk cap       | `backend/tests/unit_tests/knowledge_tests.rs:66`, `:88`; `backend/tests/api_tests/knowledge_api.rs:174` | Cycle conflict + oversize bulk rejected                          | sufficient          | None material                                          | Add parent migration orphan reference scenario                    |
| Combined knowledge filtering                     | `backend/tests/api_tests/knowledge_api.rs:135`                                                          | multi-tag CSV + difficulty filter result correctness             | basically covered   | Chapter filter depth not deeply tested                 | Add chapter+tags+difficulty+discrimination combined matrix test   |
| Outcome share total exactly 100                  | `backend/tests/api_tests/outcome_api.rs:92`; `backend/tests/unit_tests/outcome_tests.rs:44`, `:73`      | submit fails unless sum=100                                      | sufficient          | None material                                          | Add concurrent contributor update race test                       |
| Duplicate detection + evidence dedup/file limits | `backend/tests/unit_tests/outcome_tests.rs:157`, `:185`; `backend/tests/api_tests/outcome_api.rs:143`   | 25MB reject, similarity candidate, duplicate fingerprint 409     | sufficient          | No side-by-side compare assertion                      | Add compare endpoint semantic assertions                          |
| Store promo application + object access control  | `backend/tests/api_tests/store_api.rs:75`, `:136`, `:212`                                               | zero discount without promos; order access 401/403; preview auth | sufficient          | No tie-break behavior test                             | Add equal-discount tie priority deterministic test                |
| Analytics export/download security               | `backend/tests/api_tests/analytics_api.rs:117`, `:168`, `:211`                                          | CSV content type; single-use token; ownership check              | sufficient          | Limited custom-filter export checks                    | Add period/filter integrity test per report type                  |
| Backup split artifacts + sandbox validation      | `backend/tests/api_tests/backup_api.rs:117`, `:82`; `backend/tests/unit_tests/backup_tests.rs:110`      | db/files artifact split; sandbox report                          | basically covered   | Activation behavior not validated against real restore | Add integration test proving live DB/file replacement on activate |

### 8.3 Security Coverage Audit

- **authentication:** Meaningfully covered (`backend/tests/api_tests/auth_api.rs:12`, `:110`) — severe auth regressions likely detected.
- **route authorization:** Broadly covered via 403 checks (`backend/tests/api_tests/knowledge_api.rs:71`, `backend/tests/api_tests/store_api.rs:249`, `backend/tests/api_tests/backup_api.rs:162`).
- **object-level authorization:** Partially covered (`backend/tests/api_tests/store_api.rs:136`, `backend/tests/api_tests/outcome_api.rs:212`) but missed contributor-delete scoping defect.
- **tenant/data isolation:** Basically covered for scoped reads (outcomes/orders), but single-tenant architecture limits tenant-style coverage relevance.
- **admin/internal protection:** Covered (`backend/tests/api_tests/backup_api.rs:162`).

Conclusion: tests are strong enough to catch many severe defects, but at least one high-impact object-level mutation flaw still slips through, so severe defects can remain undetected.

### 8.4 Final Coverage Judgment

**Partial Pass**

Covered major risks: auth/lockout, CSRF, many role checks, core business validations, duplicate checks, export/download security, backup artifact split.  
Uncovered/insufficiently covered risks: restore activation semantics, contributor deletion scoping, retention policy preservation semantics under real archival classifications.

## 9. Final Notes

- This is a strict static audit only; no runtime success claims are made.
- Conclusions above are traceable to cited `file:line` evidence.
- Acceptance is blocked primarily by backup activation semantics and object-level mutation integrity gaps, not by superficial style concerns.
