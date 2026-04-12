# ScholarVault Static Delivery Acceptance & Architecture Audit

Date: 2026-04-12
Audit mode: **Static-only** (no runtime execution)

## 1. Verdict

- **Overall conclusion: Partial Pass**
- Rationale: The repository is substantial and implements most core domain areas (auth, roles, knowledge, outcomes, store, analytics, backups), but there are material gaps and risks including one **Blocker** (checkout trust boundary) and multiple **High** findings tied to prompt-fit/security/compliance expectations.

## 2. Scope and Static Verification Boundary

### What was reviewed

- Documentation, manifests, and test scripts (`README.md`, workspace/backend/frontend `Cargo.toml`, `run_tests.sh`)
- Backend entrypoints, router, middleware, handlers, services, migrations, and tests
- Frontend routing, major pages, API clients, logic helpers, and tests

### What was not reviewed

- Runtime behavior in a live browser/server/container
- External infra behavior (reverse proxy TLS termination, file-system permissions in deployment)
- Performance characteristics under load

### Intentionally not executed

- Project startup, Docker, tests, migrations at runtime, or API calls

### Claims requiring manual verification

- End-to-end UX quality and visual polish in-browser
- Effective HTTPS posture in deployed topology (proxy/TLS cert wiring)
- Operational scheduling behavior over real wall-clock time

## 3. Repository / Requirement Mapping Summary

- Prompt core goal (offline research + commerce ops portal) is mapped to:
  - Knowledge graph + question mapping (`backend/src/services/knowledge_service.rs`, `frontend/src/pages/knowledge/*`)
  - Outcomes/IP registration + duplicate checks + evidence (`backend/src/services/outcome_service.rs`, `backend/src/services/file_service.rs`, `frontend/src/pages/outcomes/*`)
  - Store promotions + checkout (`backend/src/services/store_service.rs`, `frontend/src/pages/store/*`)
  - Analytics/export/scheduled reports (`backend/src/services/analytics_service.rs`, `frontend/src/pages/analytics/*`)
  - Security middleware and role guards (`backend/src/router.rs`, `backend/src/middleware/*`)
  - Backup/restore/retention policy (`backend/src/services/backup_service.rs`, `backend/src/handlers/backup.rs`)

## 4. Section-by-section Review

### 1. Hard Gates

#### 1.1 Documentation and static verifiability

- **Conclusion: Pass**
- **Rationale:** Startup, config matrix, and test instructions are present and statically coherent with manifests/entrypoints.
- **Evidence:** `README.md:3`, `README.md:12`, `README.md:18`, `README.md:39`, `backend/Cargo.toml:10`, `backend/Cargo.toml:11`, `frontend/Cargo.toml:11`, `frontend/Cargo.toml:12`, `run_tests.sh:1`

#### 1.2 Material deviation from prompt

- **Conclusion: Partial Pass**
- **Rationale:** Most domains match the prompt, but there are significant deviations in risk-critical behavior (checkout trust boundary) and incomplete fit on analytics custom filtering and pre-submit side-by-side duplicate workflow.
- **Evidence:** `backend/src/services/store_service.rs:52`, `backend/src/services/store_service.rs:56`, `backend/src/services/store_service.rs:194`, `backend/src/services/store_service.rs:233`, `frontend/src/pages/outcomes/register.rs:50`, `frontend/src/pages/outcomes/register.rs:145`, `frontend/src/pages/outcomes/compare.rs:10`, `frontend/src/api/analytics.rs:133`, `frontend/src/pages/analytics/dashboard.rs:13`

### 2. Delivery Completeness

#### 2.1 Core explicit requirement coverage

- **Conclusion: Partial Pass**
- **Rationale:** Core modules exist and implement many explicit requirements (role auth, lockout, CSRF, evidence validation, backup retention, cycle prevention, bulk limits). Gaps remain for strict commerce integrity and some workflow semantics.
- **Evidence:**
  - Implemented: `backend/src/services/auth_service.rs:41`, `backend/src/security/password.rs:26`, `backend/src/middleware/csrf.rs:36`, `backend/src/services/file_service.rs:18`, `backend/src/services/file_service.rs:46`, `backend/src/services/knowledge_service.rs:21`, `backend/src/services/knowledge_service.rs:261`, `backend/src/services/outcome_service.rs:365`, `backend/src/services/backup_service.rs:85`
  - Gap indicators: `backend/src/services/store_service.rs:56`, `backend/src/services/store_service.rs:233`

#### 2.2 End-to-end deliverable vs partial demo

- **Conclusion: Pass**
- **Rationale:** Full multi-crate project with frontend/backend/shared, migrations, and extensive tests; not a single-file demo.
- **Evidence:** `Cargo.toml:1`, `Cargo.toml:2`, `backend/tests/api_tests/mod.rs:1`, `backend/tests/unit_tests/mod.rs:1`, `frontend/tests/unit_tests/mod.rs:1`

### 3. Engineering and Architecture Quality

#### 3.1 Module decomposition and structure

- **Conclusion: Pass**
- **Rationale:** Clean separation: middleware, handlers, services, models, and migrations; frontend also domain-sliced by page modules.
- **Evidence:** `backend/src/router.rs:31`, `backend/src/middleware/require_role.rs:70`, `backend/src/services/knowledge_service.rs:1`, `frontend/src/pages/mod.rs:1`

#### 3.2 Maintainability/extensibility

- **Conclusion: Partial Pass**
- **Rationale:** Generally maintainable; however, critical business invariants are not enforced at the strongest trust boundary in checkout path.
- **Evidence:** `backend/src/services/store_service.rs:52`, `backend/src/services/store_service.rs:316`, `backend/src/services/store_service.rs:233`

### 4. Engineering Details and Professionalism

#### 4.1 Error handling / logging / validation / API design

- **Conclusion: Partial Pass**
- **Rationale:** Error model and validation are strong overall; logging exists but mutation audit-hash completeness is inconsistent with strict requirement semantics.
- **Evidence:** `backend/src/error.rs:65`, `backend/src/error.rs:110`, `backend/src/services/file_service.rs:40`, `backend/src/services/audit_service.rs:38`, `backend/src/services/audit_service.rs:53`, `backend/src/handlers/auth.rs:370`, `backend/src/handlers/store.rs:37`

#### 4.2 Product-like organization vs demo

- **Conclusion: Pass**
- **Rationale:** Project resembles a real service with role-specific pages, exports, scheduling, backup lifecycle, and security middleware.
- **Evidence:** `backend/src/router.rs:35`, `backend/src/router.rs:193`, `frontend/src/app.rs:12`

### 5. Prompt Understanding and Requirement Fit

#### 5.1 Business goal and constraint fit

- **Conclusion: Partial Pass**
- **Rationale:** Strong alignment across domains, but notable shortfalls in strict commerce integrity and some required UX semantics (duplicate compare before submission, custom filtering exposure).
- **Evidence:** `backend/src/services/store_service.rs:194`, `backend/src/services/store_service.rs:233`, `frontend/src/pages/outcomes/register.rs:145`, `frontend/src/pages/outcomes/compare.rs:24`, `frontend/src/api/analytics.rs:133`, `frontend/src/pages/analytics/reports.rs:23`

### 6. Aesthetics (frontend)

#### 6.1 Visual/interaction quality

- **Conclusion: Cannot Confirm Statistically**
- **Rationale:** Rich style system and component/page structure are present, but visual rendering consistency and interaction feel require manual browser verification.
- **Evidence:** `style/main.scss:1`, `frontend/src/app.rs:18`, `frontend/src/pages/analytics/dashboard.rs:1`
- **Manual verification note:** Validate responsive layout, hierarchy, hover/focus states, and chart readability in-browser.

## 5. Issues / Suggestions (Severity-Rated)

### Blocker

1. **Severity: Blocker**  
   **Title:** Checkout trust boundary allows client-side price tampering  
   **Conclusion:** Fail  
   **Evidence:** `backend/src/services/store_service.rs:52`, `backend/src/services/store_service.rs:56`, `backend/src/services/store_service.rs:194`, `backend/src/services/store_service.rs:233`, `backend/src/handlers/store.rs:117`  
   **Impact:** Any authenticated user can submit arbitrary `unit_price` and potentially underpay or create inconsistent financial records; this breaks commerce integrity and auditability.  
   **Minimum actionable fix:** On checkout, ignore client `unit_price`/`product_name`; load authoritative product rows by `product_id`, validate active status and quantity, compute totals server-side, and persist server-derived pricing only.

### High

2. **Severity: High**  
   **Title:** Mutation audit trail does not consistently record real before/after hashes per strict requirement  
   **Conclusion:** Partial Fail  
   **Evidence:** `backend/src/services/audit_service.rs:38`, `backend/src/services/audit_service.rs:53`, `backend/src/handlers/store.rs:37`, `backend/src/handlers/store.rs:93`, `backend/src/handlers/auth.rs:370`  
   **Impact:** Audit entries may carry one-sided/sentinel hashes and some state changes (e.g., CSRF rotation) are not audited, weakening forensic guarantees required by prompt.  
   **Minimum actionable fix:** Enforce both hashes (or explicit canonical sentinel policy per mutation type) at call sites; add audit logging for all state-changing handlers including `refresh_csrf`; add tests asserting hash-pair presence for every mutation endpoint.

3. **Severity: High**  
   **Title:** Analytics custom-filter requirement is only partially surfaced in UI/API usage  
   **Conclusion:** Partial Fail  
   **Evidence:** `backend/src/services/analytics_service.rs:137`, `backend/src/services/analytics_service.rs:140`, `backend/src/services/analytics_service.rs:143`, `frontend/src/api/analytics.rs:133`, `frontend/src/pages/analytics/dashboard.rs:13`, `frontend/src/pages/analytics/reports.rs:23`  
   **Impact:** Prompt asks for custom filters across executive analytics and exports; current frontend path effectively uses unfiltered calls and sends `None` for advanced filters during scheduling flows.  
   **Minimum actionable fix:** Expose date/category/role filter controls in analytics UI; wire them through API client and scheduling/export requests; add API tests for filtered report generation semantics.

### Medium

4. **Severity: Medium**  
   **Title:** Duplicate-flag workflow is not tightly coupled to mandatory side-by-side compare before submission  
   **Conclusion:** Partial Fail  
   **Evidence:** `frontend/src/pages/outcomes/register.rs:50`, `frontend/src/pages/outcomes/register.rs:145`, `frontend/src/pages/outcomes/compare.rs:10`, `frontend/src/pages/outcomes/compare.rs:24`  
   **Impact:** Users see duplicate warnings but compare flow is separate/manual; this weakens prompt-fit for “side-by-side compare view before submission.”  
   **Minimum actionable fix:** From duplicate candidate list, provide direct compare actions and submission gating/acknowledgement step.

5. **Severity: Medium**  
   **Title:** Lockout query combines username OR IP, enabling broad IP-level denial scenarios  
   **Conclusion:** Suspected Risk  
   **Evidence:** `backend/src/services/auth_service.rs:44`, `backend/src/services/auth_service.rs:55`  
   **Impact:** Shared/NAT IP users may lock each other out after repeated failures; availability risk in offline institutional networks.  
   **Minimum actionable fix:** Track lockout primarily per account with separate IP abuse throttles; tune thresholds and add explicit tests for cross-user same-IP behavior.

## 6. Security Review Summary

- **Authentication entry points:** **Pass**  
  Evidence: login/logout/me/refresh-csrf routes and lockout+password hashing (`backend/src/router.rs:35`, `backend/src/services/auth_service.rs:41`, `backend/src/security/password.rs:26`).

- **Route-level authorization:** **Pass**  
  Evidence: role extractors and handler signatures enforce role gates (`backend/src/middleware/require_role.rs:70`, `backend/src/handlers/backup.rs:35`, `backend/src/handlers/analytics.rs:26`, `backend/src/handlers/knowledge.rs:47`).

- **Object-level authorization:** **Partial Pass**  
  Evidence: implemented for outcomes/orders/reports (`backend/src/handlers/outcomes.rs:37`, `backend/src/services/outcome_service.rs:142`, `backend/src/handlers/store.rs:166`, `backend/src/services/analytics_service.rs:479`). Core store checkout integrity remains broken by client-trusted pricing (`backend/src/services/store_service.rs:56`).

- **Function-level authorization:** **Pass**  
  Evidence: mutation handlers consistently include role/auth extractors (`backend/src/handlers/store.rs:31`, `backend/src/handlers/knowledge.rs:47`, `backend/src/handlers/backup.rs:41`).

- **Tenant / user data isolation:** **Cannot Confirm Statistically / Not Applicable (single-tenant design)**  
  Evidence: architecture appears single-organization local portal; user-level scoping exists in selected modules (`backend/src/services/outcome_service.rs:178`). No explicit tenant model present.

- **Admin / internal / debug endpoint protection:** **Pass**  
  Evidence: admin/backup endpoints gated by `RequireAdmin` and no obvious unguarded debug routes (`backend/src/router.rs:208`, `backend/src/router.rs:193`, `backend/src/handlers/backup.rs:35`).

## 7. Tests and Logging Review

- **Unit tests:** **Pass (risk-focused breadth is good)**  
  Evidence: backend unit suites across auth/knowledge/outcome/store/analytics/file/backup (`backend/tests/unit_tests/mod.rs:1`), frontend unit suites for filter/promotion/mask/validation (`frontend/tests/unit_tests/mod.rs:1`).

- **API / integration tests:** **Pass (backend strong), Partial (frontend API-client only)**  
  Evidence: backend API suites for auth/knowledge/outcome/store/analytics/backup (`backend/tests/api_tests/mod.rs:1`); frontend API tests are client-shape tests (`frontend/tests/api_tests/mod.rs:1`).

- **Logging categories / observability:** **Partial Pass**  
  Evidence: structured logs exist for internal errors, scheduler, session mismatches (`backend/src/error.rs:110`, `backend/src/services/backup_scheduler.rs:36`, `backend/src/middleware/session.rs:138`).

- **Sensitive-data leakage risk in logs / responses:** **Partial Pass**  
  Evidence: internal errors are masked in responses (`backend/src/error.rs:101`, `backend/src/error.rs:110`); no obvious plaintext password logging found. Manual verification still needed for production log sinks and redaction policy.

## 8. Test Coverage Assessment (Static Audit)

### 8.1 Test Overview

- Unit tests exist for backend and frontend (`backend/tests/unit_tests/mod.rs:1`, `frontend/tests/unit_tests/mod.rs:1`)
- API/integration tests exist for backend routes (`backend/tests/api_tests/mod.rs:1`)
- Frontend API tests exist but are client-layer focused (`frontend/tests/api_tests/mod.rs:1`)
- Test commands documented in README and script (`README.md:12`, `run_tests.sh:1`)
- Frameworks: Rust `cargo test` (Tokio async tests + Axum `oneshot` integration style)

### 8.2 Coverage Mapping Table

| Requirement / Risk Point                          | Mapped Test Case(s)                                                                                                                                                                                                                          | Key Assertion / Fixture / Mock                                      | Coverage Assessment | Gap                                                                                      | Minimum Test Addition                                                            |
| ------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------- | ------------------- | ---------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------- |
| Auth login + lockout 5/15                         | `backend/tests/api_tests/auth_api.rs:11`, `backend/tests/api_tests/auth_api.rs:108`                                                                                                                                                          | 6th attempt returns `LOCKED`                                        | sufficient          | Cross-user same-IP lockout not validated                                                 | Add same-IP multi-user lockout behavior tests                                    |
| CSRF enforcement                                  | `backend/tests/api_tests/auth_api.rs:48`, `backend/tests/api_tests/auth_api.rs:250`                                                                                                                                                          | Missing/stale CSRF returns 403                                      | sufficient          | No full matrix across all mutating endpoints                                             | Add parametrized CSRF tests across modules                                       |
| Role authorization (401/403)                      | `backend/tests/api_tests/knowledge_api.rs:60`, `backend/tests/api_tests/store_api.rs:174`, `backend/tests/api_tests/analytics_api.rs:72`, `backend/tests/api_tests/backup_api.rs:78`                                                         | Unauthorized/forbidden assertions                                   | sufficient          | None critical                                                                            | Keep regression suite                                                            |
| Knowledge DAG cycle + bulk 1000 cap               | `backend/tests/unit_tests/knowledge_tests.rs:27`, `backend/tests/unit_tests/knowledge_tests.rs:88`, `backend/tests/api_tests/knowledge_api.rs:126`, `backend/tests/api_tests/knowledge_api.rs:224`                                           | cycle conflict + oversize bulk 400                                  | sufficient          | Limited chapter-filter edge cases                                                        | Add chapter+tag combined corner cases                                            |
| Outcome 100% share + duplicates + evidence checks | `backend/tests/unit_tests/outcome_tests.rs:42`, `backend/tests/unit_tests/outcome_tests.rs:185`, `backend/tests/unit_tests/outcome_tests.rs:122`, `backend/tests/api_tests/outcome_api.rs:269`, `backend/tests/api_tests/outcome_api.rs:336` | share validation, duplicate detection, MIME/magic, dedup conflict   | sufficient          | No explicit “compare before submit” workflow test                                        | Add UI/API flow test that links duplicate candidates to compare action           |
| Store checkout correctness / promotions           | `backend/tests/api_tests/store_api.rs:33`, `backend/tests/unit_tests/store_tests.rs:77`                                                                                                                                                      | best-offer applied, line traces populated                           | **insufficient**    | No tampered `unit_price` negative test; server currently trusts client price             | Add failing test: client sends manipulated price, server must reject/recalculate |
| Analytics export/report token security            | `backend/tests/api_tests/analytics_api.rs:118`, `backend/tests/api_tests/analytics_api.rs:167`, `backend/tests/api_tests/analytics_api.rs:212`                                                                                               | CSV content-type, single-use token, ownership checks                | basically covered   | Custom-filter report generation not covered                                              | Add tests for date/category/role filtered exports/reports                        |
| Backup lifecycle + schedule + activation          | `backend/tests/api_tests/backup_api.rs:34`, `backend/tests/api_tests/backup_api.rs:166`, `backend/tests/api_tests/backup_api.rs:251`, `backend/tests/unit_tests/backup_tests.rs:68`                                                          | admin-only, schedule update, activation overwrite, cleanup behavior | sufficient          | Long-horizon scheduler runtime not testable statically                                   | Manual schedule-fire verification in ops checklist                               |
| Audit hash completeness for mutations             | `backend/tests/api_tests/outcome_api.rs:140`                                                                                                                                                                                                 | asserts no row has both hashes NULL                                 | **insufficient**    | Does not assert both before+after semantics per mutation type; missing endpoint coverage | Add cross-endpoint audit conformance tests (pair presence + expected sentinels)  |

### 8.3 Security Coverage Audit

- **authentication:** meaningfully covered (login, wrong password, lockout) — severe auth defects less likely to slip undetected.
- **route authorization:** well covered with multiple 401/403 endpoint tests.
- **object-level authorization:** covered for outcomes/orders/reports (`backend/tests/api_tests/outcome_api.rs:36`, `backend/tests/api_tests/store_api.rs:142`, `backend/tests/api_tests/analytics_api.rs:212`), but commerce integrity check (price authority) remains untested and currently vulnerable.
- **tenant/data isolation:** no tenant model; user-scoping tested in selected domains only.
- **admin/internal protection:** backup/admin access controls are tested (`backend/tests/api_tests/backup_api.rs:61`).

### 8.4 Final Coverage Judgment

- **Final Coverage Judgment: Partial Pass**
- Major risks covered: authn/authz basics, CSRF, knowledge DAG/bulk constraints, outcome share and evidence checks, backup lifecycle controls.
- Major uncovered/insufficient risks: checkout trust boundary (critical), strict audit-hash semantics across all mutations, and custom-filter analytics/report workflows. Tests could still pass while severe commerce integrity defects remain.

## 9. Final Notes

- This report is strictly static and evidence-based; no runtime success is claimed.
- The repository is close to a production-structured deliverable but should not be accepted without addressing the Blocker (checkout trust boundary) and High issues above.
