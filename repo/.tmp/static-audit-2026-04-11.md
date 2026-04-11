# Static Delivery Acceptance & Project Architecture Audit

Date: 2026-04-11  
Scope root: `./` (current working directory only)

## 1. Verdict

**Overall conclusion: Fail**

Rationale (static, evidence-based): multiple material authorization defects allow unauthenticated or unauthorized access to sensitive resources (order details, outcomes/evidence, report download by token only), plus insecure default secret posture and test blind spots around those risks.

---

## 2. Scope and Static Verification Boundary

### What was reviewed

- Documentation and run/test instructions: `README.md:1-24`, `run_tests.sh:1-66`, `docker-compose.yml:1-56`, `.env.example:1-10`
- Backend entry/routing/security layers: `backend/src/main.rs:1-122`, `backend/src/router.rs:1-238`, `backend/src/middleware/*.rs`
- Core handlers/services/models/migrations: `backend/src/handlers/*.rs`, `backend/src/services/*.rs`, `backend/src/db/migrations/*.sql`
- Test suites (unit + API): `backend/tests/unit_tests/*.rs`, `backend/tests/api_tests/*.rs`
- Frontend structure/style samples: `frontend/src/app.rs:1-52`, `frontend/src/pages/**/*.rs` (selected), `style/main.scss:1-933`

### What was not reviewed

- Runtime behavior, live HTTP flows, browser rendering, Docker/container behavior, DB state under real deployment timing.
- Full frontend visual execution (no browser run).

### Intentionally not executed

- Project startup, Docker, tests, external services (per audit constraints).

### Claims requiring manual verification

- Real runtime startup success under actual environment and volumes.
- Browser-level aesthetics/interaction correctness.
- Network/TLS deployment controls (secure cookies over HTTPS, proxy behavior).

---

## 3. Repository / Requirement Mapping Summary

### Prompt extraction status

- Business prompt content is unresolved placeholder (`{prompt}`), so exact prompt-to-code semantic fit cannot be fully judged.
- Therefore requirement-fit conclusions are bounded to static repository intent inferred from docs and code comments.

### Inferred core flows from repository evidence

- Role-based portal with auth/session/CSRF + domain modules (knowledge, outcomes/IP, store/promotions, analytics/reports, backup/admin).
- Docker-based run/test workflow.
- SQLite persistence with migrations + seeded users/data.

### Main mapped implementation areas

- Auth/session/role extractors and route wiring.
- Domain handlers/services and ownership boundaries.
- Security middleware and defaults.
- Unit/API tests and logging/auditability.

---

## 4. Section-by-section Review

### 4.1 Hard Gates

#### 4.1.1 Documentation and static verifiability

- **Conclusion: Partial Pass**
- **Rationale:** Clear run/test commands exist and entrypoints are identifiable, but documentation is minimal and relies on Docker only; key security/runtime assumptions are not documented as verification steps.
- **Evidence:** `README.md:3-14`, `docker-compose.yml:13-29`, `Dockerfile:58-61`, `run_tests.sh:1-66`
- **Manual verification note:** runtime startup and end-to-end operation remain manual.

#### 4.1.2 Material deviation from Prompt

- **Conclusion: Cannot Confirm Statistically**
- **Rationale:** Prompt body is placeholder `{prompt}`; no authoritative business requirements available for strict mismatch determination.
- **Evidence:** User-provided audit input contains `{prompt}` placeholder (no concrete requirement text).

### 4.2 Delivery Completeness

#### 4.2.1 Coverage of explicit core requirements

- **Conclusion: Cannot Confirm Statistically**
- **Rationale:** Explicit authoritative requirements are unavailable (placeholder prompt).
- **Evidence:** Prompt missing concrete requirements.

#### 4.2.2 0→1 end-to-end deliverable vs partial/demo

- **Conclusion: Partial Pass**
- **Rationale:** Repository includes full workspace structure, backend/frontend/shared crates, migrations, tests, and docs; however security-critical authz defects materially reduce production-readiness.
- **Evidence:** `Cargo.toml:1-40`, `backend/Cargo.toml:1-53`, `backend/src/router.rs:1-238`, `backend/tests/api_tests/mod.rs:1-11`, `README.md:1-24`

### 4.3 Engineering and Architecture Quality

#### 4.3.1 Structure and module decomposition

- **Conclusion: Pass**
- **Rationale:** Reasonable separation by handlers/services/models/middleware and frontend pages/api/logic; no single-file pile-up.
- **Evidence:** `backend/src/{handlers,services,middleware,models}/`, `frontend/src/{pages,api,logic}/`, `backend/src/lib.rs:1-38`

#### 4.3.2 Maintainability/extensibility

- **Conclusion: Partial Pass**
- **Rationale:** Layering is generally maintainable, but core authorization policies are inconsistently applied at handler boundaries, creating systemic risk.
- **Evidence:** Protected handlers use role extractors (e.g., `backend/src/handlers/store.rs:23-31`) while neighboring sensitive reads omit them (`backend/src/handlers/store.rs:143-150`, `backend/src/handlers/outcomes.rs:23-41`).

### 4.4 Engineering Details and Professionalism

#### 4.4.1 Error handling, logging, validation, API design

- **Conclusion: Partial Pass**
- **Rationale:** Error envelope and status mapping are structured, and many validations exist; however observability is thin in request handlers and security defaults are weak.
- **Evidence:** `backend/src/error.rs:12-102`, validation examples in `backend/src/services/store_service.rs:83-157`, sparse tracing points largely startup/scheduler (`backend/src/main.rs:40,83,93`, `backend/src/services/backup_scheduler.rs:30-44`).

#### 4.4.2 Product/service quality vs demo

- **Conclusion: Partial Pass**
- **Rationale:** Product-like breadth exists (admin/backup/analytics/store/outcomes), but critical authorization weaknesses keep it below real-service quality.
- **Evidence:** Feature routes in `backend/src/router.rs:33-201`; defects in authz boundaries noted in Issues section.

### 4.5 Prompt Understanding and Requirement Fit

#### 4.5.1 Business-goal and constraint fit

- **Conclusion: Cannot Confirm Statistically**
- **Rationale:** No concrete prompt semantics were provided (placeholder), so fit cannot be authoritatively scored.
- **Evidence:** Missing prompt body.

### 4.6 Aesthetics (frontend/full-stack)

#### 4.6.1 Visual and interaction design quality

- **Conclusion: Cannot Confirm Statistically**
- **Rationale:** Strong static evidence of design system and component structure exists, but rendering/interaction quality requires browser execution.
- **Evidence:** `style/main.scss:1-933`, `frontend/src/pages/login.rs:1-102`, `frontend/src/pages/store/mod.rs:1-101`, `frontend/src/app.rs:1-52`
- **Manual verification note:** inspect rendered UI states (hover/focus/layout/responsiveness) in browser.

---

## 5. Issues / Suggestions (Severity-Rated)

### Blocker / High first

1. **Severity: Blocker**  
   **Title:** IDOR risk: order detail endpoint lacks auth and ownership checks  
   **Conclusion:** Fail  
   **Evidence:**

- Route is public GET endpoint (no route-level guard): `backend/src/router.rs:153`
- Handler has no `AuthenticatedUser` extractor: `backend/src/handlers/store.rs:143-150`
- Service fetches by `id` only (no `user_id` scope): `backend/src/services/store_service.rs:266-277`
- Tests do not cover `/api/store/orders/:id`: no matches under `backend/tests/**/*.rs` for this path; existing store API tests cover checkout/promotions only (`backend/tests/api_tests/store_api.rs:34-170`).
  **Impact:** Any caller with/guessing order ID can retrieve another user’s order and line items.  
  **Minimum actionable fix:** Require authenticated user in `get_order` and enforce ownership (or admin/store role exception) in query (`WHERE id=? AND user_id=?` for non-admin).

2. **Severity: High**  
   **Title:** Outcomes/evidence read endpoints are unauthenticated  
   **Conclusion:** Fail  
   **Evidence:**

- Handler signatures for list/get/compare have no auth extractor: `backend/src/handlers/outcomes.rs:23-41`, `backend/src/handlers/outcomes.rs:245-260`
- Routes are openly wired as GET: `backend/src/router.rs:99-103`, `backend/src/router.rs:128`
- Evidence files are included in `get_outcome` response payload: `backend/src/handlers/outcomes.rs:32-41`
  **Impact:** Sensitive outcome metadata/evidence listings may be exposed to anonymous users.  
  **Minimum actionable fix:** Add `AuthenticatedUser`/role extractor to read endpoints and, where needed, object-level policy checks before returning evidence/outcome details.

3. **Severity: High**  
   **Title:** Report download endpoint bypasses user binding (token-only access, no auth extractor)  
   **Conclusion:** Fail  
   **Evidence:**

- Handler has no authentication extractor: `backend/src/handlers/analytics.rs:173-188`
- Route exposed as public GET with path token: `backend/src/router.rs:180`
- Service validates only `id + token`, not creator/role: `backend/src/services/analytics_service.rs:431-460`
- In contrast, listing is user-scoped (`created_by`): `backend/src/services/analytics_service.rs:417-425`
  **Impact:** Token leakage (logs/referrers/sharing) can allow unauthorized report retrieval.  
  **Minimum actionable fix:** Require authenticated user in handler and validate report ownership/authorization in service before file read.

4. **Severity: High**  
   **Title:** Insecure default cryptographic secrets are hardcoded and shipped in docs/config  
   **Conclusion:** Fail  
   **Evidence:**

- Hardcoded fallback keys in runtime: `backend/src/main.rs:27-29`
- Same defaults in compose env fallbacks: `docker-compose.yml:23-24`
- Same defaults in `.env.example`: `.env.example:2-3`
  **Impact:** Deployments that keep defaults risk predictable key material and compromised confidentiality/integrity.  
  **Minimum actionable fix:** Remove insecure defaults; fail fast if secrets are missing/weak; document minimum entropy/rotation policy.

5. **Severity: Medium**  
   **Title:** Session/CSRF cookies omit explicit `Secure` attribute  
   **Conclusion:** Partial Fail  
   **Evidence:** cookie build sets `http_only` + `same_site` but no `.secure(true)`: `backend/src/handlers/auth.rs:68-74`, refresh path `backend/src/handlers/auth.rs:288-289`
   **Impact:** In non-TLS or misconfigured TLS-termination setups, cookie confidentiality can degrade.  
   **Minimum actionable fix:** Set `Secure` for sensitive cookies in production (or always with documented localhost override strategy).

6. **Severity: Medium**  
   **Title:** Security test coverage misses major authz/object-ownership paths  
   **Conclusion:** Fail (coverage gap)  
   **Evidence:**

- No API tests for `/api/store/orders/:id` authorization path (no matches in `backend/tests/**/*.rs`)
- No API tests for unauthenticated access to outcomes GET/list/compare (tests focus on create/submit/evidence upload path): `backend/tests/api_tests/outcome_api.rs:34-210`
- Report download tests verify token single-use only, not auth/ownership policy: `backend/tests/api_tests/analytics_api.rs:111-149`, `backend/tests/unit_tests/analytics_tests.rs:126-138`
  **Impact:** Severe defects can pass current test suite undetected.  
  **Minimum actionable fix:** Add negative and ownership tests for these endpoints (401/403 and cross-user access denial).

7. **Severity: Low**  
   **Title:** Migration runner can skip migrations if path missing and continue startup  
   **Conclusion:** Partial Fail  
   **Evidence:** `backend/src/db/mod.rs:39-44` (warn + return `Ok(())` when migration directory absent)
   **Impact:** Misconfiguration may produce partially initialized runtime with deferred failures.  
   **Minimum actionable fix:** In non-test environments, treat missing migration directory as startup error.

---

## 6. Security Review Summary

### authentication entry points

- **Conclusion: Pass (with caveats)**
- **Evidence:** Login/logout/me/refresh endpoints exist: `backend/src/router.rs:35-38`; lockout logic in service `backend/src/services/auth_service.rs:38-58`; CSRF middleware active on mutations `backend/src/router.rs:210-217`, `backend/src/middleware/csrf.rs:29-57`.
- **Reasoning:** Core auth/session controls are present.

### route-level authorization

- **Conclusion: Partial Pass**
- **Evidence:** Many mutating/admin routes enforce roles (`RequireAdmin`, `RequireStore`, `RequireCurator`, etc.) in handlers, e.g. `backend/src/handlers/backup.rs:35-39`, `backend/src/handlers/store.rs:23-31`; however some sensitive read routes lack auth (`backend/src/handlers/store.rs:143-150`, `backend/src/handlers/outcomes.rs:23-41`, `backend/src/handlers/analytics.rs:173-188`).
- **Reasoning:** Policy applied inconsistently.

### object-level authorization

- **Conclusion: Fail**
- **Evidence:** Order lookup by ID only: `backend/src/services/store_service.rs:266-277`; unauthenticated handler path `backend/src/handlers/store.rs:143-150`.
- **Reasoning:** No owner/tenant check at object read boundary.

### function-level authorization

- **Conclusion: Partial Pass**
- **Evidence:** Role extractor pattern exists (`backend/src/middleware/require_role.rs:26-74`) and is used broadly; missing on specific functions noted above.

### tenant / user isolation

- **Conclusion: Fail**
- **Evidence:** `list_orders` has user scoping path `backend/src/services/store_service.rs:248-264`, but `get_order_with_items` bypasses user scope `backend/src/services/store_service.rs:266-277`; report downloads not bound to owner `backend/src/services/analytics_service.rs:431-460`.

### admin / internal / debug protection

- **Conclusion: Partial Pass**
- **Evidence:** Backup/admin endpoints are admin-gated (`backend/src/handlers/backup.rs:35-39`, `backend/src/handlers/auth.rs:176-268`).
- **Reasoning:** Admin surfaces are guarded, but adjacent sensitive non-admin endpoints remain exposed.

---

## 7. Tests and Logging Review

### Unit tests

- **Conclusion: Pass (scope-limited)**
- **Evidence:** Unit suites exist for auth/knowledge/outcomes/store/analytics/file/backup: `backend/tests/unit_tests/mod.rs:1-11`.
- **Reasoning:** Good service-level coverage of business/validation algorithms.

### API / integration tests

- **Conclusion: Partial Pass**
- **Evidence:** API suites exist and cover auth/knowledge/outcomes/store/analytics/backup: `backend/tests/api_tests/mod.rs:1-11`.
- **Reasoning:** Coverage misses key authorization ownership scenarios (see §8).

### Logging categories / observability

- **Conclusion: Partial Pass**
- **Evidence:** Structured startup/scheduler/db logs exist: `backend/src/main.rs:40,83,93`, `backend/src/services/backup_scheduler.rs:30-44`, `backend/src/db/mod.rs:42,52`; mutable actions also write DB audit entries (e.g., `backend/src/handlers/store.rs:31-41`, `backend/src/handlers/outcomes.rs:59-69`).
- **Reasoning:** Operational tracing in request-path error/security events appears sparse; audit DB helps but does not replace runtime diagnostics.

### Sensitive-data leakage risk in logs/responses

- **Conclusion: Partial Pass**
- **Evidence:** `password_hash` is included in `User` model (`backend/src/models/user.rs:11`) but admin responses map to `UserSummary` excluding it (`backend/src/handlers/auth.rs:150-172`); audit logs hash payloads (`backend/src/services/audit_service` usage across handlers).
- **Reasoning:** No direct plaintext password logging found; however token-in-URL download design increases leakage surface (referer/log propagation risk).

---

## 8. Test Coverage Assessment (Static Audit)

### 8.1 Test Overview

- Unit tests: present (`backend/tests/unit_tests/mod.rs:1-11`)
- API tests: present (`backend/tests/api_tests/mod.rs:1-11`)
- Framework: Rust `cargo test` + `tokio::test` (`backend/tests/**/*`)
- Test entry points documented: `run_tests.sh:20-48`, `README.md:9-12`
- Documentation provides test command: yes (Docker profile test).

### 8.2 Coverage Mapping Table

| Requirement / Risk Point                                 | Mapped Test Case(s)                                                                                                                    | Key Assertion / Fixture / Mock                                               | Coverage Assessment | Gap                                  | Minimum Test Addition                                      |
| -------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------- | ------------------- | ------------------------------------ | ---------------------------------------------------------- |
| Login success/failure                                    | `backend/tests/api_tests/auth_api.rs:11-44`                                                                                            | `200` for valid, `401` for wrong password                                    | sufficient          | none major                           | add inactive-user case                                     |
| Lockout after repeated failures                          | `backend/tests/api_tests/auth_api.rs:107-133`                                                                                          | 6th attempt -> `423 LOCKED`                                                  | sufficient          | no window-expiry test                | add lockout expiry boundary                                |
| CSRF enforcement on mutation                             | `backend/tests/api_tests/auth_api.rs:46-105`                                                                                           | no token -> `403`; valid token allows logout                                 | basically covered   | no cross-session token mismatch test | add session-token mismatch test                            |
| Role gate for knowledge/store/backup                     | `knowledge_api.rs:49-89`, `store_api.rs:132-170`, `backup_api.rs:53-81`                                                                | allowed role `200`, disallowed role `403`                                    | basically covered   | not exhaustive across all routes     | add matrix for high-risk read routes                       |
| Order ownership / object auth (`/api/store/orders/:id`)  | none found for endpoint path                                                                                                           | N/A                                                                          | **missing**         | severe IDOR risk untested            | add cross-user order fetch test expecting `403/404`        |
| Outcome read auth (`/api/outcomes`, `/api/outcomes/:id`) | none for unauth/forbidden reads                                                                                                        | existing tests focus create/submit/evidence upload (`outcome_api.rs:34-210`) | **missing**         | sensitive read exposure undetected   | add unauthenticated GET tests (`401/403`)                  |
| Report download authorization binding                    | `analytics_api.rs:111-149`, `unit_tests/analytics_tests.rs:126-138`                                                                    | token single-use works; second call `404`                                    | insufficient        | no ownership/auth checks tested      | add cross-user token replay + unauth download denial tests |
| Validation and conflict paths                            | knowledge bulk limits `knowledge_api.rs:130-149`, outcomes shares `outcome_api.rs:89-138`, evidence duplicate `outcome_api.rs:141-209` | `400`/`409` asserted                                                         | sufficient          | none major                           | add boundary fuzz cases                                    |

### 8.3 Security Coverage Audit

- **authentication:** **Pass** (login failure/success/lockout covered) — `auth_api.rs:11-133`
- **route authorization:** **Partial Pass** (some role checks covered, not all sensitive reads) — `knowledge_api.rs`, `store_api.rs`, `backup_api.rs`
- **object-level authorization:** **Fail** (no tests for order ownership or cross-user object access)
- **tenant / data isolation:** **Fail** (no tests proving cross-user data isolation on order/report reads)
- **admin / internal protection:** **Pass (basic)** (backup admin-only tested) — `backup_api.rs:53-81`

### 8.4 Final Coverage Judgment

**Partial Pass**

Covered well: core auth happy path/failure, several role-gated mutations, multiple validation/conflict scenarios.  
Uncovered high-risk areas: object-level authorization and user-isolation on order details, outcomes reads, and report downloads. These gaps mean severe security defects can remain undetected while tests still pass.

---

## 9. Final Notes

- This audit is strictly static; no runtime success is claimed.
- The most material root causes are authorization-boundary inconsistencies and weak default secret posture.
- Prompt-fit scoring is constrained because authoritative prompt content was not provided (placeholder `{prompt}`).
