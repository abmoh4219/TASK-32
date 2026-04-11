# Static Delivery Acceptance & Project Architecture Audit

Date: 2026-04-11  
Scope root: `./` (current working directory only)

## 1. Verdict

**Overall conclusion: Fail**

Primary reasons (static evidence):

- HTTPS is not enforced in-process, and secure cookies are optional-off by default (`backend/src/main.rs:99`, `backend/src/handlers/auth.rs:26-29`).
- Several core prompt requirements are only partially implemented (knowledge combined filters, backup bundle separation, retention-policy configurability).
- Authorization is inconsistent for executive analytics/report scheduling and outcome visibility boundaries.

## 2. Scope and Static Verification Boundary

### What was reviewed

- Documentation/config/manifests: `README.md:1-24`, `Cargo.toml:1-52`, `backend/Cargo.toml:1-55`, `frontend/Cargo.toml:1-38`, `run_tests.sh:1-66`, `.env.example`.
- Backend entrypoints, middleware, routing, handlers/services/models/migrations:
  - `backend/src/main.rs:1-146`
  - `backend/src/router.rs:1-236`
  - `backend/src/middleware/*.rs`
  - `backend/src/handlers/*.rs`
  - `backend/src/services/*.rs`
  - `backend/src/db/migrations/*.sql`
- Frontend page structure, API clients, logic helpers:
  - `frontend/src/app.rs:1-52`
  - `frontend/src/pages/**/*.rs`
  - `frontend/src/api/*.rs`
  - `frontend/src/logic/*.rs`
- Test suites (not executed):
  - `backend/tests/unit_tests/*.rs`
  - `backend/tests/api_tests/*.rs`
  - `frontend/tests/unit_tests/*.rs`
  - `frontend/tests/api_tests/*.rs`

### What was not reviewed

- Runtime behavior, browser rendering fidelity, container networking/volumes, performance under load, scheduler timing behavior under real clock conditions.

### What was intentionally not executed

- Project startup, tests, Docker, external services (per hard constraints).

### Claims requiring manual verification

- Browser-level visual quality and interaction states.
- Deployment-level HTTPS/TLS termination and cookie transport guarantees.
- Real restore activation behavior against a live persistent database.

## 3. Repository / Requirement Mapping Summary

### Prompt core goals and constraints (mapped)

- Offline knowledge + outcomes/IP + storefront + analytics portal with role-based access.
- Axum + Leptos + SQLite with security controls (CSRF, headers, lockout, encryption-at-rest, upload validation, rate limiting).
- Backup lifecycle with scheduled jobs, retention policy, sandbox validation and activation path.

### Main implementation areas mapped

- Auth/session/rbac: `backend/src/handlers/auth.rs`, `backend/src/services/auth_service.rs`, `backend/src/middleware/{session,csrf,require_role,rate_limit}.rs`.
- Domain modules:
  - Knowledge: `backend/src/handlers/knowledge.rs`, `backend/src/services/knowledge_service.rs`
  - Outcomes: `backend/src/handlers/outcomes.rs`, `backend/src/services/outcome_service.rs`
  - Store: `backend/src/handlers/store.rs`, `backend/src/services/store_service.rs`
  - Analytics: `backend/src/handlers/analytics.rs`, `backend/src/services/analytics_service.rs`
  - Backup: `backend/src/handlers/backup.rs`, `backend/src/services/{backup_service,backup_scheduler}.rs`
- Frontend flows: `frontend/src/pages/{knowledge,outcomes,store,analytics,admin}/**`.
- Static coverage evidence: `backend/tests/**`, `frontend/tests/**`.

## 4. Section-by-section Review

### 4.1 Hard Gates

#### 4.1.1 Documentation and static verifiability

- **Conclusion: Partial Pass**
- **Rationale:** Basic run/test docs are present and statically consistent with repo structure, but instructions are Docker-centric and minimal for non-Docker local verification.
- **Evidence:** `README.md:5`, `README.md:11`, `run_tests.sh:20-48`, `Cargo.toml:1-3`, `backend/src/main.rs:1-17`.
- **Manual verification note:** End-to-end startup/behavior still requires manual runtime verification.

#### 4.1.2 Material deviation from Prompt

- **Conclusion: Fail**
- **Rationale:** Multiple explicit prompt requirements are weakened or partially implemented (HTTPS enforcement, combined chapter+tags filtering behavior, backup bundle separation, admin-configurable retention update path).
- **Evidence:** `backend/src/main.rs:99`, `backend/src/handlers/knowledge.rs:241`, `backend/src/services/knowledge_service.rs:655-657`, `backend/src/services/backup_service.rs:111-115`, `backend/src/router.rs:198`, `backend/src/handlers/backup.rs:132-136`.

### 4.2 Delivery Completeness

#### 4.2.1 Core requirement coverage

- **Conclusion: Partial Pass**
- **Rationale:** Broad feature coverage exists across modules, but there are requirement-level gaps in security posture and specific business constraints.
- **Evidence:** Feature routing breadth `backend/src/router.rs:33-201`; gaps noted above and in Issues section.

#### 4.2.2 0→1 end-to-end deliverable shape

- **Conclusion: Pass**
- **Rationale:** Repo contains complete multi-crate structure, migrations, handlers/services, frontend pages, and test suites—this is not a snippet/demo-only drop.
- **Evidence:** `Cargo.toml:1-52`, `backend/src/db/migrations/0001_create_users.sql:1`, `frontend/src/app.rs:1-52`, `backend/tests/api_tests/mod.rs:1-9`, `frontend/tests/unit_tests/mod.rs:1-7`.

### 4.3 Engineering and Architecture Quality

#### 4.3.1 Structure and decomposition

- **Conclusion: Pass**
- **Rationale:** Clean modular split by middleware/handlers/services/models and matching frontend API/page logic separation.
- **Evidence:** `backend/src/lib.rs:7-16`, `backend/src/router.rs:1-27`, `frontend/src/pages/mod.rs:1-10`, `frontend/src/api/client.rs:1-18`.

#### 4.3.2 Maintainability/extensibility

- **Conclusion: Partial Pass**
- **Rationale:** Architecture is extensible, but policy rules are not consistently encoded at the same layer (some role checks in handlers only, some ownership in services, some endpoints broad by design), increasing future drift risk.
- **Evidence:** Store object check in handler `backend/src/handlers/store.rs:151-168`; report ownership in service `backend/src/services/analytics_service.rs:450`; broad analytics access `backend/src/handlers/analytics.rs:28-42,62`.

### 4.4 Engineering Details and Professionalism

#### 4.4.1 Error handling, logging, validation, API design

- **Conclusion: Partial Pass**
- **Rationale:** Error envelope and many validations are strong; logging/observability is relatively sparse in request-path diagnostics; anti-abuse backoff control is not evident beyond static rate limiting.
- **Evidence:** `backend/src/error.rs:14-100`, `backend/src/services/file_service.rs:18-53`, `backend/src/middleware/rate_limit.rs:62-88`, `backend/src/main.rs:41,84,94`.
- **Manual verification note:** Request-level telemetry sufficiency requires runtime logs under failure scenarios.

#### 4.4.2 Product/service professionalism

- **Conclusion: Partial Pass**
- **Rationale:** Product-grade scope is present, but blocker/high security and requirement-fit gaps prevent acceptance as production-ready.
- **Evidence:** Multi-domain coverage `backend/src/router.rs:33-201`; severe gaps listed in section 5.

### 4.5 Prompt Understanding and Requirement Fit

#### 4.5.1 Business objective and constraints fit

- **Conclusion: Partial Pass**
- **Rationale:** The team clearly targeted the right business domains and workflows, but missed several strict constraints from prompt wording (secure transport enforcement, combined filtering semantics, retention management, backup artifact split).
- **Evidence:** `backend/src/services/knowledge_service.rs:615-657`, `backend/src/services/backup_service.rs:111-115`, `backend/src/handlers/backup.rs:132-136`, `backend/src/main.rs:99`.

### 4.6 Aesthetics (frontend/full-stack)

#### 4.6.1 Visual/interaction quality

- **Conclusion: Cannot Confirm Statistically**
- **Rationale:** Static code indicates consistent design-system usage and hierarchical layout, but actual visual correctness and interaction quality require browser execution.
- **Evidence:** `frontend/src/components/layout/mod.rs:1-214`, `frontend/src/pages/login.rs:1-109`, `frontend/src/pages/analytics/dashboard.rs:1-155`, `style/main.scss` (present in repo tree).
- **Manual verification note:** Validate spacing/contrast/hover/focus/responsiveness manually.

## 5. Issues / Suggestions (Severity-Rated)

### Blocker / High

1. **Severity: Blocker**  
   **Title:** HTTPS is not enforced in-process, and secure cookie transport is optional/off by default  
   **Conclusion:** Fail  
   **Evidence:** `backend/src/main.rs:99` (plain `axum::serve`), `backend/src/handlers/auth.rs:26-29` (`cookies_secure` defaults false), cookie construction at `backend/src/handlers/auth.rs:77-85`.  
   **Impact:** Violates prompt’s “Security is enforced end-to-end with HTTPS”; risks session/CSRF token exposure in non-TLS deployment paths.  
   **Minimum actionable fix:** Enforce TLS at service boundary (or hard-fail without trusted TLS termination signal), and default cookies to `Secure=true` in production-safe baseline.

2. **Severity: High**  
   **Title:** Executive analytics/report scheduling endpoints are broadly accessible to any authenticated role  
   **Conclusion:** Fail  
   **Evidence:** `backend/src/handlers/analytics.rs:28-42,62` use `AuthenticatedUser` for `members/churn/events/approval_cycles`; `schedule_report` also uses `AuthenticatedUser` at `backend/src/handlers/analytics.rs:144-147`.  
   **Impact:** Role model and business boundary are weakened; non-executive roles can access/schedule executive analytics features.  
   **Minimum actionable fix:** Gate executive analytics/report scheduling with `RequireFinance` (and optionally `Administrator`) consistently.

3. **Severity: High**  
   **Title:** Outcome visibility is global for authenticated users (no object-level/user-scope isolation)  
   **Conclusion:** Fail  
   **Evidence:** `backend/src/handlers/outcomes.rs:23-25,31-33`; service query is global `SELECT * FROM outcomes ...` at `backend/src/services/outcome_service.rs:124-126`.  
   **Impact:** IP/outcome metadata can be over-exposed across users/roles where least-privilege isolation is expected.  
   **Minimum actionable fix:** Add policy-based scoping (creator/team/role) in list/get endpoints and data queries.

4. **Severity: High**  
   **Title:** Backup artifact design deviates from prompt requirement for separate versioned DB and file bundles  
   **Conclusion:** Fail  
   **Evidence:** Single tar bundle includes both DB and evidence in one record (`backend/src/services/backup_service.rs:111`, `backend/src/services/backup_service.rs:115`, `backend/src/services/backup_service.rs:130`).  
   **Impact:** Does not meet explicit deliverable semantics; complicates independent verification/restore lifecycle expected by prompt.  
   **Minimum actionable fix:** Produce independent versioned artifacts and metadata records for database and upload files.

5. **Severity: High**  
   **Title:** Retention policy is effectively read-only (admin-configurable update path missing)  
   **Conclusion:** Fail  
   **Evidence:** Only read endpoint exists (`backend/src/router.rs:198`, `backend/src/handlers/backup.rs:132-136`); service only reads default policy (`backend/src/services/backup_service.rs:320-322`); schema seeds one default row (`backend/src/db/migrations/0009_create_backup.sql:31`).  
   **Impact:** Fails prompt requirement for Administrator-configured retention policy.  
   **Minimum actionable fix:** Add authenticated admin mutation endpoint/service to update `retention_policies` with audit logging.

6. **Severity: High**  
   **Title:** Combined knowledge filter semantics are incomplete (chapter ignored, multi-tag collapsed to first tag)  
   **Conclusion:** Fail  
   **Evidence:** Handler hard-sets `chapter: None` and only single tag from query (`backend/src/handlers/knowledge.rs:236,241`); service marks chapter filter as no-op (`backend/src/services/knowledge_service.rs:655-657`); frontend sends only first tag (`frontend/src/pages/knowledge/knowledge_points.rs:24`).  
   **Impact:** Misses explicit prompt behavior for fast combined chapter+tags+difficulty+discrimination filtering.  
   **Minimum actionable fix:** Add chapter query parameter flow end-to-end and support multi-tag query serialization/handling.

### Medium / Low

7. **Severity: Medium**  
   **Title:** Anti-abuse exponential backoff for repeated invalid searches is not evident  
   **Conclusion:** Partial Fail  
   **Evidence:** Current anti-abuse implementation is static request rate limiting only (`backend/src/middleware/rate_limit.rs:62-88`); no invalid-search backoff logic appears in query handlers/services reviewed.  
   **Impact:** Prompt asks for basic anti-abuse controls beyond flat rate limits; repeated invalid search abuse may still be cheap.  
   **Minimum actionable fix:** Track invalid search bursts per actor and progressively delay/reject subsequent requests.

8. **Severity: Medium**  
   **Title:** Promotion effective-window input UX does not natively use required MM/DD/YYYY + 12-hour entry format  
   **Conclusion:** Partial Fail  
   **Evidence:** Input controls are `datetime-local` (`frontend/src/pages/store/promotions.rs:132,135`); formatting helper is display-only (`frontend/src/logic/promotion.rs:16-21`).  
   **Impact:** Requirement-fit gap in input UX semantics.  
   **Minimum actionable fix:** Provide explicit MM/DD/YYYY 12-hour input widgets/parsing validation at form level.

9. **Severity: Low**  
   **Title:** `preview_checkout` is unauthenticated while checkout/order endpoints are authenticated  
   **Conclusion:** Partial Fail  
   **Evidence:** `preview_checkout` has no `AuthenticatedUser` extractor (`backend/src/handlers/store.rs:126-133`), while checkout/list/get do (`backend/src/handlers/store.rs:107,137,145`).  
   **Impact:** Potentially inconsistent trust boundary and unnecessary surface exposure.  
   **Minimum actionable fix:** Align preview endpoint auth policy with checkout flow (or document rationale explicitly).

## 6. Security Review Summary

- **authentication entry points:** **Pass**  
   Evidence: auth routes and lockout/session logic exist (`backend/src/router.rs:35-38`, `backend/src/services/auth_service.rs:38-58,81-138`).

- **route-level authorization:** **Partial Pass**  
   Evidence: strong role gates in many mutation paths (`backend/src/handlers/backup.rs:39,46,66`; `backend/src/handlers/knowledge.rs:43,66,91`), but analytics/report scheduling uses broad `AuthenticatedUser` (`backend/src/handlers/analytics.rs:28-42,62,146`).

- **object-level authorization:** **Partial Pass**  
   Evidence: store order object check exists (`backend/src/handlers/store.rs:151-168`), but outcomes are globally listed/read for authenticated users (`backend/src/services/outcome_service.rs:124-126`).

- **function-level authorization:** **Partial Pass**  
   Evidence: extractor system is robust (`backend/src/middleware/require_role.rs:44-74`), but not consistently strict for all sensitive business functions.

- **tenant / user isolation:** **Fail**  
   Evidence: outcomes list/get are not user-scoped (`backend/src/services/outcome_service.rs:124-126`, `backend/src/handlers/outcomes.rs:23-33`).

- **admin / internal / debug protection:** **Pass**  
   Evidence: backup/admin endpoints require admin role (`backend/src/handlers/backup.rs:39,134`; `backend/src/handlers/auth.rs:188,197,231,258,280`).

## 7. Tests and Logging Review

### Unit tests

- **Conclusion: Pass**
- **Rationale:** Backend unit tests cover key business/security primitives: hashing, lockout policy assumptions, cycle detection, share totals, promotion resolution, file validation, backup lifecycle.
- **Evidence:** `backend/tests/unit_tests/auth_tests.rs:8-82`, `knowledge_tests.rs:23-165`, `outcome_tests.rs:39-219`, `store_tests.rs:45-167`, `analytics_tests.rs:23-176`, `backup_tests.rs:36-191`.

### API / integration tests

- **Conclusion: Partial Pass**
- **Rationale:** Good coverage for auth/CSRF/role gates and many happy-path flows, but missing/assertion-light coverage for high-risk policy boundaries (analytics role scoping, outcome isolation specifics, preview auth consistency).
- **Evidence:** `backend/tests/api_tests/auth_api.rs:11-133`, `knowledge_api.rs:41-149`, `store_api.rs:37-235`, `analytics_api.rs:30-223`, `outcome_api.rs:34-246`.

### Logging categories / observability

- **Conclusion: Partial Pass**
- **Rationale:** Structured tracing exists mostly for startup/scheduler, and immutable DB audit logging is broadly used for mutations; request-path diagnostic logging depth is limited.
- **Evidence:** `backend/src/main.rs:41,84,94`, `backend/src/services/backup_scheduler.rs:30-44`, `backend/src/services/audit_service.rs:26-56`.

### Sensitive-data leakage risk in logs / responses

- **Conclusion: Partial Pass**
- **Rationale:** No obvious plaintext password/PII logging found in reviewed code; however transport-level weakness (HTTP + optional secure cookie) remains a major leakage risk.
- **Evidence:** `backend/src/handlers/auth.rs:26-29,77-85`, `backend/src/main.rs:99`; sensitive model fields exist encrypted (`backend/src/models/user.rs:16-17`) and masking helpers exist (`backend/src/security/encryption.rs:83`, `frontend/src/logic/mask.rs:6`).

## 8. Test Coverage Assessment (Static Audit)

### 8.1 Test Overview

- **Unit tests exist:** yes (`backend/tests/unit_tests/mod.rs:1-7`, `frontend/tests/unit_tests/mod.rs:1-7`).
- **API/integration tests exist:** yes (`backend/tests/api_tests/mod.rs:1-9`, `frontend/tests/api_tests/mod.rs:1-6`).
- **Test frameworks:** Rust native `cargo test` / `tokio::test`.
- **Test entrypoints documented:** yes (`README.md:11`, `run_tests.sh:20-48`).

### 8.2 Coverage Mapping Table

| Requirement / Risk Point                           | Mapped Test Case(s) (`file:line`)                                                                        | Key Assertion / Fixture (`file:line`)                        | Coverage Assessment | Gap                                                     | Minimum Test Addition                                  |
| -------------------------------------------------- | -------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------ | ------------------- | ------------------------------------------------------- | ------------------------------------------------------ |
| Login + lockout (5 fails / 15 min)                 | `backend/tests/api_tests/auth_api.rs:11-44,107-133`                                                      | Valid 200, invalid 401, 6th 423                              | sufficient          | Lockout expiry window boundary not covered              | Add “wait/advance window then login succeeds” test     |
| CSRF on state-changing endpoints                   | `backend/tests/api_tests/auth_api.rs:46-105`                                                             | Missing token => 403; valid token logout => 200              | basically covered   | Cross-session token misuse path not explicit            | Add session A token on session B mutation test         |
| Knowledge bulk edit limit + cycle protection       | `backend/tests/api_tests/knowledge_api.rs:91-149`, `backend/tests/unit_tests/knowledge_tests.rs:58-110`  | 409 on cycle merge, 400 on >1000 ids                         | sufficient          | Chapter/tag combined filtering not tested               | Add API tests for chapter + multi-tag filter semantics |
| Outcome share total must equal 100                 | `backend/tests/api_tests/outcome_api.rs:89-138`, `backend/tests/unit_tests/outcome_tests.rs:39-104`      | Submit fails at 50%; submit succeeds at 100%                 | sufficient          | No user-isolation assertions                            | Add cross-user list/get visibility tests               |
| Evidence upload validation + duplicate fingerprint | `backend/tests/api_tests/outcome_api.rs:141-209`, `backend/tests/unit_tests/file_tests.rs:15-66`         | First upload 200, second same file 409                       | sufficient          | No malicious mixed-part edge tests                      | Add malformed multipart and spoofed MIME cases         |
| Promotion engine (priority/exclusion/best offer)   | `backend/tests/unit_tests/store_tests.rs:45-167`, `backend/tests/api_tests/store_api.rs:37-75`           | Best discount chosen; line trace populated                   | sufficient          | Preview endpoint auth policy not tested                 | Add unauthorized preview expectation test              |
| Analytics export/report token behavior             | `backend/tests/api_tests/analytics_api.rs:60-223`, `backend/tests/unit_tests/analytics_tests.rs:126-176` | CSV content type; token single-use; owner check for download | basically covered   | Role-scope for members/churn/events/schedule not tested | Add non-finance access tests for those endpoints       |
| Backup run/restore/cleanup                         | `backend/tests/api_tests/backup_api.rs:31-112`, `backend/tests/unit_tests/backup_tests.rs:53-191`        | Admin-only, hash validation, lifecycle purge                 | basically covered   | No tests for admin policy update (missing feature)      | Add policy update endpoint tests after implementation  |

### 8.3 Security Coverage Audit

- **authentication:** **sufficiently covered** (auth success/fail/lockout/CSRF).  
   Evidence: `backend/tests/api_tests/auth_api.rs:11-133`.

- **route authorization:** **insufficient** for analytics role boundaries.  
   Evidence: tests cover funds role gating (`backend/tests/api_tests/analytics_api.rs:30-58`) but not broad-access analytics endpoints.

- **object-level authorization:** **insufficient** in outcomes domain.  
   Evidence: no test asserts cross-user outcome visibility denial; current tests focus on create/submit/upload (`backend/tests/api_tests/outcome_api.rs:34-246`).

- **tenant / data isolation:** **insufficient** (outcomes global queries remain unchallenged by tests).  
   Evidence: service query global list (`backend/src/services/outcome_service.rs:124-126`).

- **admin / internal protection:** **basically covered** (backup admin role tests).  
   Evidence: `backend/tests/api_tests/backup_api.rs:53-81`.

### 8.4 Final Coverage Judgment

**Partial Pass**

Covered well: authentication, CSRF, many core happy paths, key validation rules.  
Coverage risk remaining: authorization scope and data-isolation semantics can still regress while the existing suite passes.

## 9. Final Notes

- This report is static-only; no runtime success is claimed.
- Findings are root-cause oriented and evidence-backed with file/line anchors.
- Most critical acceptance blockers are transport security enforcement and prompt-fit gaps in policy/backup/filter semantics.
