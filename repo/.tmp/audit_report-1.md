# ScholarVault Delivery Acceptance & Architecture Audit (Static-Only)

Date: 2026-04-12

## 1. Verdict

- **Overall conclusion: Partial Pass**

The repository is substantial and maps strongly to the prompt across backend domains (auth, knowledge, outcomes, store, analytics, backup) with meaningful tests. However, there are material gaps/deviations (notably transport-security implementation evidence and certificate-number duplicate logic semantics), plus a few medium architecture/requirement-fit issues.

## 2. Scope and Static Verification Boundary

### What was reviewed

- Project docs and manifests: `README.md:1`, `Cargo.toml:1`, `backend/Cargo.toml:1`, `frontend/Cargo.toml:1`, `docker-compose.yml:1`, `run_tests.sh:1`
- Backend routing/entry/security/domain services/models/migrations
- Frontend routing/pages/API clients/styles
- Backend + frontend tests (static inspection only)

### What was not reviewed

- Runtime behavior in browser, network stack, Docker/runtime orchestration, TLS termination setup, filesystem permissions under real deployment.

### Intentionally not executed

- No project run, no tests execution, no Docker, no external services.

### Claims requiring manual verification

- Real HTTPS end-to-end path (certificates/proxy wiring) in deployment.
- Runtime UX completeness and visual polish under real browser rendering.
- Scheduler execution timing behavior in real wall-clock operation.

## 3. Repository / Requirement Mapping Summary

### Prompt core goal (condensed)

Build an offline ScholarVault portal with role-based operations across:

- Knowledge graph + question linkage + bulk/filter workflows
- Outcome/IP registration + evidence + duplicate checks + contribution integrity
- Store promotions/checkout best-offer traceability
- Executive analytics + exports + scheduled reports
- Security (authz, CSRF, rate-limit, audit, encryption) and backup/retention lifecycle

### Main implementation areas mapped

- Backend Axum surface in `backend/src/router.rs:34` onward
- Service-layer core logic in `backend/src/services/*`
- Persistence/constraints in `backend/src/db/migrations/*.sql`
- Leptos UI in `frontend/src/pages/**/*`
- Tests in `backend/tests/**/*` and `frontend/tests/**/*`

## 4. Section-by-section Review

### 4.1 Hard Gates

#### 1.1 Documentation and static verifiability

- **Conclusion: Partial Pass**
- **Rationale:** Basic run/test commands exist, but docs are minimal and Docker-centric, with limited static guidance for non-Docker/local verification, config matrix, and feature verification paths.
- **Evidence:** `README.md:5`, `README.md:11`, `README.md:7`, `run_tests.sh:26`, `run_tests.sh:33`, `run_tests.sh:40`, `run_tests.sh:47`
- **Manual verification note:** Local non-Docker verification path is under-documented; reviewer must infer from scripts/manifests.

#### 1.2 Material deviation from prompt

- **Conclusion: Partial Pass**
- **Rationale:** Core domains are implemented and routed, but at least one explicit requirement is not implemented as stated (certificate-number _similarity_), and transport-security requirement is only partially evidenced statically.
- **Evidence:** `backend/src/router.rs:34`, `backend/src/services/outcome_service.rs:209`, `backend/src/main.rs:120`, `backend/src/main.rs:123`

### 4.2 Delivery Completeness

#### 2.1 Core explicit requirements coverage

- **Conclusion: Partial Pass**
- **Rationale:** Most core flows exist (knowledge DAG/filter/bulk, outcomes + evidence, store promotions/checkout, analytics export/reports, backups/retention). Gaps: certificate-number similarity semantics and constrained analytics filter surface.
- **Evidence:**
  - Knowledge: `backend/src/services/knowledge_service.rs:476`, `backend/src/services/knowledge_service.rs:555`
  - Outcomes: `backend/src/services/outcome_service.rs:300`, `backend/src/services/file_service.rs:18`, `backend/src/services/file_service.rs:39`
  - Store: `backend/src/services/store_service.rs:262`, `backend/src/services/store_service.rs:307`
  - Analytics: `backend/src/handlers/analytics.rs:48`, `backend/src/services/analytics_service.rs:134`
  - Backup: `backend/src/services/backup_service.rs:451`, `backend/src/db/migrations/0009_create_backup.sql:23`

#### 2.2 End-to-end deliverable (0→1) vs partial demo

- **Conclusion: Pass**
- **Rationale:** Full multi-crate structure, routes, services, migrations, frontend pages, and tests are present.
- **Evidence:** `Cargo.toml:1`, `backend/src/router.rs:34`, `frontend/src/app.rs:15`, `backend/tests/api_tests/mod.rs:6`

### 4.3 Engineering and Architecture Quality

#### 3.1 Module decomposition and structure

- **Conclusion: Pass**
- **Rationale:** Clear separation across handlers/services/models/middleware/migrations and frontend pages/API/logic/components.
- **Evidence:** `backend/src/lib.rs:7`, `backend/src/handlers/mod.rs:4`, `frontend/src/pages/mod.rs:4`

#### 3.2 Maintainability/extensibility

- **Conclusion: Partial Pass**
- **Rationale:** Generally maintainable; however, a few implementation details reduce reliability (e.g., bulk-update affected-count logic uses max instead of sum).
- **Evidence:** `backend/src/services/knowledge_service.rs:518`, `backend/src/services/knowledge_service.rs:533`, `backend/src/services/knowledge_service.rs:545`

### 4.4 Engineering Details and Professionalism

#### 4.1 Error handling, logging, validation, API quality

- **Conclusion: Partial Pass**
- **Rationale:** Validation/error envelopes are good; logging exists but appears sparse for broad operational observability. Security-critical checks are mostly present (CSRF/rate-limit/authz).
- **Evidence:** `backend/src/error.rs:110`, `backend/src/middleware/csrf.rs:55`, `backend/src/middleware/rate_limit.rs:138`, `backend/src/services/file_service.rs:39`, `backend/src/services/auth_service.rs:41`
- **Manual verification note:** Need runtime log review under load to confirm observability sufficiency.

#### 4.2 Product-grade vs demo-grade

- **Conclusion: Partial Pass**
- **Rationale:** Backend is product-like; frontend has one unresolved placeholder route (`/dashboard`) and a stub page module.
- **Evidence:** `frontend/src/app.rs:23`, `frontend/src/app.rs:36`, `frontend/src/pages/dashboard.rs:1`

### 4.5 Prompt Understanding and Requirement Fit

#### 5.1 Business goal/scenario fit

- **Conclusion: Partial Pass**
- **Rationale:** Strong fit across major workflows, but some requirement semantics are narrowed (certificate duplicate logic; analytics filtering scope).
- **Evidence:** `backend/src/services/outcome_service.rs:209`, `backend/src/services/outcome_service.rs:220`, `backend/src/services/outcome_service.rs:231`, `backend/src/handlers/analytics.rs:49`, `backend/src/services/analytics_service.rs:137`

### 4.6 Aesthetics (frontend)

#### 6.1 Visual and interaction quality

- **Conclusion: Partial Pass**
- **Rationale:** Static evidence shows consistent themed design, hierarchy, spacing, and component states; however, runtime rendering/usability cannot be confirmed statically.
- **Evidence:** `style/main.scss:1`, `style/main.scss:108`, `style/main.scss:315`, `frontend/src/components/layout/mod.rs:96`
- **Manual verification note:** Browser QA needed for responsive behavior, hover/interaction fidelity, and accessibility checks.

## 5. Issues / Suggestions (Severity-Rated)

### Blocker / High

1. **High — End-to-end HTTPS not statically implemented at app listener layer**

- **Conclusion:** Fail
- **Evidence:** `backend/src/main.rs:120`, `backend/src/main.rs:123`, `backend/src/main.rs:201`, `backend/src/middleware/security_headers.rs:16`
- **Impact:** Prompt explicitly requires HTTPS end-to-end; current server binds plain TCP and depends on environment/proxy policy.
- **Minimum actionable fix:** Add first-class TLS serving path (or hard-required trusted TLS proxy mode) with explicit production config and documentation.

2. **High — Duplicate detection uses exact certificate match, not similarity check**

- **Conclusion:** Fail
- **Evidence:** `backend/src/services/outcome_service.rs:209`, `backend/src/services/outcome_service.rs:220`, `backend/src/services/outcome_service.rs:231`
- **Impact:** Prompt requests similarity checks on title/abstract snippet/certificate number; certificate path is exact-only.
- **Minimum actionable fix:** Implement certificate similarity heuristic (e.g., normalized Levenshtein/Jaro-Winkler) with threshold and test coverage.

### Medium

3. **Medium — Analytics filter surface is narrow (period-only) vs “custom filters” expectation**

- **Conclusion:** Partial Fail
- **Evidence:** `backend/src/handlers/analytics.rs:48`, `backend/src/handlers/analytics.rs:49`, `backend/src/services/analytics_service.rs:134`, `backend/src/services/analytics_service.rs:137`, `frontend/src/api/analytics.rs:109`
- **Impact:** Limits business reporting flexibility promised in prompt.
- **Minimum actionable fix:** Add structured filter DTOs (date range, category, role/source dimensions) for dashboard/export/report endpoints.

4. **Medium — Placeholder dashboard route/page remains**

- **Conclusion:** Partial Fail
- **Evidence:** `frontend/src/app.rs:23`, `frontend/src/app.rs:36`, `frontend/src/pages/dashboard.rs:1`
- **Impact:** Reduces perceived delivery completeness of UI navigation map.
- **Minimum actionable fix:** Implement a real dashboard page or remove/redirect placeholder route.

5. **Medium (Suspected Risk) — Session loading does not bind to IP/user-agent**

- **Conclusion:** Suspected Risk
- **Evidence:** `backend/src/middleware/session.rs:63`, `backend/src/middleware/session.rs:76`
- **Impact:** Stolen session-cookie replay risk may be higher in offline/shared-network environments.
- **Minimum actionable fix:** Validate session context (IP/UA binding policy with tolerance) and rotate/invalidate suspicious sessions.

6. **Medium — Bulk update affected-count logic is inaccurate**

- **Conclusion:** Fail
- **Evidence:** `backend/src/services/knowledge_service.rs:518`, `backend/src/services/knowledge_service.rs:533`, `backend/src/services/knowledge_service.rs:545`
- **Impact:** API returns misleading updated counts; audit hashes based on counts may be less trustworthy.
- **Minimum actionable fix:** Sum rows affected per statement/chunk (with dedupe strategy), do not use `max`.

### Low

7. **Low — Documentation depth is thin for acceptance reproducibility**

- **Conclusion:** Partial Fail
- **Evidence:** `README.md:5`, `README.md:11`, `README.md:7`
- **Impact:** Slower/manual reviewer onboarding.
- **Minimum actionable fix:** Expand README with config matrix, non-Docker run path, role/endpoint verification checklist, and security deployment notes.

## 6. Security Review Summary

- **Authentication entry points:** **Pass**
  - Evidence: `backend/src/router.rs:36`, `backend/src/services/auth_service.rs:41`, `backend/src/services/auth_service.rs:55`
  - Notes: Lockout threshold and session issuance exist.

- **Route-level authorization:** **Pass**
  - Evidence: `backend/src/middleware/require_role.rs:70`, `backend/src/middleware/require_role.rs:78`, `backend/src/router.rs:34`

- **Object-level authorization:** **Partial Pass**
  - Evidence: `backend/src/handlers/outcomes.rs:34`, `backend/src/handlers/store.rs:166`, `backend/src/services/analytics_service.rs:450`
  - Notes: Good in outcomes/order/report download; session context binding is a residual risk.

- **Function-level authorization:** **Pass**
  - Evidence: `backend/src/handlers/auth.rs:242`, `backend/src/handlers/backup.rs:34`, `backend/src/handlers/store.rs:40`

- **Tenant/user data isolation:** **Partial Pass**
  - Evidence: `backend/src/services/outcome_service.rs:144`, `backend/src/services/analytics_service.rs:421`, `backend/src/handlers/store.rs:147`
  - Notes: Isolation mostly present, but store list-orders policy may be business-constraining for store manager workflows.

- **Admin/internal/debug protection:** **Pass**
  - Evidence: `backend/src/router.rs:198`, `backend/src/router.rs:214`, `backend/tests/api_tests/backup_api.rs:62`

## 7. Tests and Logging Review

- **Unit tests:** **Pass**
  - Evidence: `backend/tests/unit_tests/mod.rs:4`, `backend/tests/unit_tests/mod.rs:10`, `frontend/tests/unit_tests/mod.rs:4`

- **API/integration tests:** **Pass**
  - Evidence: `backend/tests/api_tests/mod.rs:6`, `backend/tests/api_tests/mod.rs:11`, `backend/tests/api_tests/analytics_api.rs:211`, `backend/tests/api_tests/store_api.rs:136`

- **Logging categories / observability:** **Partial Pass**
  - Evidence: `backend/src/main.rs:47`, `backend/src/error.rs:110`, `backend/src/services/backup_scheduler.rs:36`
  - Notes: Logging exists for startup/errors/scheduler; broader request/business telemetry appears limited.

- **Sensitive-data leakage risk in logs/responses:** **Partial Pass**
  - Evidence: `backend/src/error.rs:104`, `backend/src/handlers/auth.rs:198`, `backend/src/handlers/auth.rs:217`, `backend/tests/api_tests/auth_api.rs:274`
  - Notes: API response masking is implemented/tested; manual runtime audit still needed for log redaction completeness.

## 8. Test Coverage Assessment (Static Audit)

### 8.1 Test Overview

- Unit + API/integration tests exist for backend and frontend.
- Framework: Rust `cargo test` with async integration tests (`tokio`).
- Test entry points:
  - Backend unit: `backend/tests/unit_tests/mod.rs:4`
  - Backend API: `backend/tests/api_tests/mod.rs:6`
  - Frontend unit: `frontend/tests/unit_tests/mod.rs:4`
  - Frontend API-client: `frontend/tests/api_tests/mod.rs:4`
- Test command documentation exists in script: `run_tests.sh:26`, `run_tests.sh:33`, `run_tests.sh:40`, `run_tests.sh:47`

### 8.2 Coverage Mapping Table

| Requirement / Risk Point                             | Mapped Test Case(s)                                                                              | Key Assertion / Fixture                                        | Coverage          | Gap                                  | Minimum Test Addition                                            |
| ---------------------------------------------------- | ------------------------------------------------------------------------------------------------ | -------------------------------------------------------------- | ----------------- | ------------------------------------ | ---------------------------------------------------------------- |
| Auth login + lockout                                 | `backend/tests/api_tests/auth_api.rs:14`, `:108`                                                 | 200 on valid login; LOCKED after 5 fails                       | sufficient        | None major                           | Add lockout window-expiry test                                   |
| CSRF enforcement                                     | `backend/tests/api_tests/auth_api.rs:45`, `:139`                                                 | 403 missing token; session-bound mismatch 403                  | sufficient        | None major                           | Add PATCH/DELETE matrix by route                                 |
| Knowledge cycle/merge safety                         | `backend/tests/api_tests/knowledge_api.rs:103`; `backend/tests/unit_tests/knowledge_tests.rs:52` | 409 on cycle merge                                             | sufficient        | None major                           | Add orphan-reference migration edge test                         |
| Bulk edit cap + preview                              | `backend/tests/api_tests/knowledge_api.rs:123`, `:209`                                           | preview conflicts; >1000 returns 400                           | basically covered | inaccurate count bug untested        | Add assertion on exact affected count across multi-chunk updates |
| Outcomes share=100 submit gate                       | `backend/tests/api_tests/outcome_api.rs:151`, `:211`                                             | submit 200 at 100%; 400 when !=100                             | sufficient        | None major                           | Add boundary tests for 0/100 contributor updates                 |
| Evidence validation + duplicate fingerprint          | `backend/tests/api_tests/outcome_api.rs:263`; `backend/tests/unit_tests/file_tests.rs:17`        | magic/MIME/size behavior + 409 duplicate upload                | sufficient        | None major                           | Add encrypted-on-disk artifact inspection test                   |
| Object-level authorization (outcomes/orders/reports) | `backend/tests/api_tests/outcome_api.rs:435`; `store_api.rs:136`; `analytics_api.rs:211`         | cross-user access denied                                       | sufficient        | None major                           | Add admin-bypass regression matrix                               |
| Backup run/restore/schedule/policy                   | `backend/tests/api_tests/backup_api.rs:30`, `:87`, `:149`, `:290`                                | backup records, sandbox validation, schedule/policy admin-only | basically covered | scheduler runtime cadence not proven | Add deterministic scheduler trigger harness                      |
| Analytics export/report token single-use             | `backend/tests/api_tests/analytics_api.rs:98`, `:149`                                            | CSV response shape; token invalid after first use              | sufficient        | custom filter semantics not covered  | Add tests for expanded filter DTOs after implementation          |
| Frontend business-logic helpers                      | `frontend/tests/unit_tests/filter_tests.rs:5`, `promotion_tests.rs:26`                           | filter + MM/DD/YYYY 12h conversion                             | basically covered | UI interaction/state wiring untested | Add wasm UI/component interaction tests                          |

### 8.3 Security Coverage Audit

- **Authentication:** covered meaningfully (strong) — `backend/tests/api_tests/auth_api.rs:14`, `:108`
- **Route authorization:** covered (strong) — `knowledge_api.rs:86`, `store_api.rs:235`, `backup_api.rs:74`
- **Object-level authorization:** covered (strong) — `outcome_api.rs:17`, `store_api.rs:136`, `analytics_api.rs:211`
- **Tenant/data isolation:** covered for key surfaces, but not exhaustive — `outcome_api.rs:435`, `analytics_api.rs:211`
- **Admin/internal protection:** covered — `backup_api.rs:62`, `analytics_api.rs:33`

Residual severe defects that could still slip:

- Transport security deployment misconfiguration (not exercised by tests)
- Certificate-number similarity semantics mismatch (tests do not assert similarity behavior)

### 8.4 Final Coverage Judgment

- **Partial Pass**

Major authz/CSRF/object-access/security paths are well covered statically by tests. However, uncovered or insufficiently covered areas (transport-security implementation semantics, certificate similarity semantics, and some business-fit filter breadth) mean tests could still pass while important requirement defects remain.

## 9. Final Notes

- This report is static-only and intentionally avoids runtime claims.
- High-severity findings focus on explicit prompt-fit and security semantics, not style preferences.
- Root-cause consolidation was applied to avoid duplicate symptom inflation.
