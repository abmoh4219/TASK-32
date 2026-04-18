# Test Coverage Audit

## Scope & Method

- Audit mode: **static inspection only** (no execution, no builds, no containers).
- Evidence sources:
  - Endpoint source of truth: `backend/src/router.rs`
  - Backend API tests: `backend/tests/api_tests/*.rs`
  - Backend unit tests: `backend/tests/unit_tests/*.rs`
  - Frontend unit tests: `frontend/tests/unit_tests/*.rs`
  - Frontend E2E tests: `frontend/tests/e2e/*.spec.ts`
  - Test orchestration policy: `run_tests.sh`
  - README compliance: `README.md`

## Project Type Detection

- README top declaration: `# fullstack` (`README.md:1`).
- Structure confirms fullstack (`backend/` + `frontend/` + browser E2E tests).
- Effective audit mode: **fullstack**.

## Strict Definitions Applied

- Endpoint identity = **METHOD + fully resolved PATH** from router.
- Coverage counted only when tests send request to matching route paths (parameterized paths normalized).
- “True no-mock HTTP” requires real app/router execution path via HTTP layer.
- Direct service/DB-only tests are non-HTTP by definition.

## Backend Endpoint Inventory

Derived from `build_router` in `backend/src/router.rs`.

1. GET `/healthz`
2. POST `/api/auth/login`
3. POST `/api/auth/logout`
4. GET `/api/auth/me`
5. POST `/api/auth/refresh-csrf`
6. GET `/api/healthz`
7. GET `/api/knowledge/categories`
8. POST `/api/knowledge/categories`
9. GET `/api/knowledge/categories/tree`
10. PUT `/api/knowledge/categories/:id`
11. DELETE `/api/knowledge/categories/:id`
12. GET `/api/knowledge/categories/:id/references`
13. POST `/api/knowledge/categories/merge`
14. GET `/api/knowledge/points`
15. POST `/api/knowledge/points`
16. PUT `/api/knowledge/points/:id`
17. DELETE `/api/knowledge/points/:id`
18. POST `/api/knowledge/points/bulk/preview`
19. POST `/api/knowledge/points/bulk/apply`
20. GET `/api/knowledge/questions`
21. POST `/api/knowledge/questions`
22. PUT `/api/knowledge/questions/:id`
23. DELETE `/api/knowledge/questions/:id`
24. POST `/api/knowledge/questions/:id/link`
25. GET `/api/outcomes`
26. POST `/api/outcomes`
27. GET `/api/outcomes/:id`
28. POST `/api/outcomes/:id/contributors`
29. DELETE `/api/outcomes/:id/contributors/:cid`
30. POST `/api/outcomes/:id/submit`
31. POST `/api/outcomes/:id/approve`
32. POST `/api/outcomes/:id/reject`
33. POST `/api/outcomes/:id/evidence`
34. GET `/api/outcomes/:id/compare/:other_id`
35. GET `/api/store/products`
36. POST `/api/store/products`
37. GET `/api/store/promotions`
38. POST `/api/store/promotions`
39. POST `/api/store/promotions/:id/deactivate`
40. POST `/api/store/checkout`
41. POST `/api/store/checkout/preview`
42. GET `/api/store/orders`
43. GET `/api/store/orders/:id`
44. GET `/api/analytics/members`
45. GET `/api/analytics/churn`
46. GET `/api/analytics/events`
47. GET `/api/analytics/funds`
48. GET `/api/analytics/approval-cycles`
49. POST `/api/analytics/export/csv`
50. POST `/api/analytics/export/pdf`
51. POST `/api/analytics/reports/schedule`
52. GET `/api/analytics/reports`
53. GET `/api/analytics/reports/:id/download/:token`
54. GET `/api/backup/history`
55. POST `/api/backup/run`
56. POST `/api/backup/:id/restore-sandbox`
57. POST `/api/backup/:id/activate`
58. POST `/api/backup/lifecycle-cleanup`
59. GET `/api/backup/policy`
60. PUT `/api/backup/policy`
61. GET `/api/backup/schedule`
62. PUT `/api/backup/schedule`
63. GET `/api/admin/users`
64. POST `/api/admin/users`
65. POST `/api/admin/users/:id/role`
66. POST `/api/admin/users/:id/active`
67. GET `/api/admin/audit`

## API Test Mapping Table

Coverage evidence by domain (matching router signatures):

| Domain               | Endpoint set                                                                               | Covered | Type              | Primary evidence files                                                                                                                                                                                                                                                                                 |
| -------------------- | ------------------------------------------------------------------------------------------ | ------: | ----------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| Health               | `GET /healthz`, `GET /api/healthz`                                                         |     2/2 | true no-mock HTTP | `auth_api.rs` (`test_security_headers_present_on_representative_routes`)                                                                                                                                                                                                                               |
| Auth                 | login/logout/me/refresh-csrf                                                               |     4/4 | true no-mock HTTP | `auth_api.rs` (`test_login_valid_credentials_returns_200`, `test_csrf_valid_token_passes_logout`, `test_me_authenticated_returns_user_info`, `test_refresh_csrf_writes_audit_record`)                                                                                                                  |
| Knowledge categories | list/create/tree/update/delete/references/merge                                            |     7/7 | true no-mock HTTP | `knowledge_api.rs` (`test_get_categories_returns_seeded_tree`, `test_update_category_curator_succeeds`, `test_delete_category_not_found_returns_404`, `test_category_reference_count_returns_count`, `test_merge_cycle_returns_409`)                                                                   |
| Knowledge points     | list/create/update/delete/bulk preview/apply                                               |     6/6 | true no-mock HTTP | `knowledge_api.rs` (`test_create_knowledge_point_curator_succeeds`, `test_update_knowledge_point_curator_succeeds`, `test_delete_knowledge_point_curator_succeeds`, `test_bulk_preview_returns_conflicts`, `test_bulk_apply_oversize_returns_400`)                                                     |
| Questions            | list/create/update/delete/link                                                             |     5/5 | true no-mock HTTP | `knowledge_api.rs` (`test_create_question_curator_succeeds`, `test_update_question_not_found_returns_404`, `test_delete_question_not_found_returns_404`, `test_link_question_to_knowledge_point`)                                                                                                      |
| Outcomes             | list/create/get/add/remove/submit/approve/reject/evidence/compare                          |   10/10 | true no-mock HTTP | `outcome_api.rs` (`test_full_outcome_registration_flow`, `test_approve_outcome_reviewer_succeeds`, `test_reject_outcome_reviewer_succeeds`, `test_evidence_upload_duplicate_fingerprint_returns_409`, `test_outcomes_read_endpoints_reject_anonymous`)                                                 |
| Store                | products list/create, promotions list/create/deactivate, checkout/preview, orders list/get |     9/9 | true no-mock HTTP | `store_api.rs` (`test_create_product_store_manager_succeeds`, `test_store_manager_creates_promotion`, `test_full_checkout_flow_applies_best_offer`, `test_preview_checkout_requires_auth`, `test_list_orders_returns_own_orders_only`, `test_get_order_requires_auth_and_blocks_cross_user_access`)    |
| Analytics            | members/churn/events/funds/approval-cycles, csv/pdf exports, schedule/list/download        |   10/10 | true no-mock HTTP | `analytics_api.rs` (`test_exec_analytics_requires_finance_or_admin`, `test_export_csv_returns_text_csv`, `test_export_pdf_finance_returns_pdf`, `test_scheduled_report_creates_complete_record`, `test_list_reports_returns_own_reports`, `test_download_token_single_use_via_http`)                   |
| Backup               | history/run/restore/activate/cleanup/policy get+put/schedule get+put                       |     9/9 | true no-mock HTTP | `backup_api.rs` (`test_backup_run_creates_record`, `test_restore_sandbox_returns_validation_report`, `test_activate_backup_files_artifact_succeeds`, `test_lifecycle_cleanup_admin_succeeds`, `test_retention_policy_admin_can_update_others_cannot`, `test_backup_schedule_admin_update_and_persist`) |
| Admin                | users list/create, role change, active toggle, audit log                                   |     5/5 | true no-mock HTTP | `auth_api.rs` (`test_admin_list_users_returns_all_seeded`, `test_admin_create_user_encrypts_and_masks_pii`, `test_admin_change_role_succeeds`, `test_admin_set_active_deactivates_user`, `test_admin_audit_log_returns_records`)                                                                       |

### Endpoint Completeness

- Total endpoints: **67**
- Endpoints with HTTP evidence: **67**
- Endpoints without HTTP evidence: **0**

## API Test Classification

### 1) True No-Mock HTTP

Evidence:

- `backend/tests/api_tests/common.rs::setup_test_app` builds full app router via `build_router(state.clone())`.
- Requests executed through `oneshot(req)` against real routes.
- No transport/controller/service mocking frameworks detected in API tests.

### 2) HTTP with Mocking

- **None detected**.

### 3) Non-HTTP

- Backend unit tests (`backend/tests/unit_tests/*.rs`)
- Frontend unit tests (`frontend/tests/unit_tests/*.rs`)
- Non-HTTP cases inside API files:
  - `auth_api.rs::test_seed_users_all_present` (direct DB)
  - `backup_api.rs::test_restore_activation_applies_db_file_to_live_path` (service-level)

## Mock Detection

Pattern search targets: `jest.mock`, `vi.mock`, `sinon.stub`, `mockall`, `stub`, etc.

- Backend tests: no mocking framework usage found.
- Frontend tests: no mocking framework usage found.

Conclusion: API route tests remain classified as **true no-mock HTTP**.

## Coverage Summary

- Total endpoints: **67**
- Endpoints with HTTP tests: **67**
- Endpoints with true no-mock HTTP tests: **67**

Computed metrics:

- HTTP coverage = $\frac{67}{67} \times 100 = 100\%$
- True API coverage = $\frac{67}{67} \times 100 = 100\%$

## Unit Test Analysis

### Backend Unit Tests

Files:

- `backend/tests/unit_tests/auth_tests.rs`
- `backend/tests/unit_tests/knowledge_tests.rs`
- `backend/tests/unit_tests/outcome_tests.rs`
- `backend/tests/unit_tests/store_tests.rs`
- `backend/tests/unit_tests/analytics_tests.rs`
- `backend/tests/unit_tests/file_tests.rs`
- `backend/tests/unit_tests/backup_tests.rs`

Modules covered:

- Services: `knowledge_service`, `outcome_service`, `store_service`, `analytics_service`, `file_service`, `backup_service`
- Security primitives: `backend::security::password`, `encryption`, `csrf`

Important backend modules not clearly unit-tested directly:

- Middleware internals: `csrf`, `session`, `rate_limit`, `require_role`, `security_headers`
- Services without dedicated unit suite: `question_service`, `audit_service`, `abuse`, `backup_scheduler`

### Frontend Unit Tests (STRICT REQUIREMENT)

Detection checks:

1. Frontend test files exist: **Yes**
2. Tests target frontend modules: **Yes** (logic modules)
3. Framework evident: **Yes** (`#[test]` Rust harness)
4. Tests import frontend modules: **Yes** (`use frontend::logic::*`)

Frontend unit files:

- `frontend/tests/unit_tests/validation_tests.rs`
- `frontend/tests/unit_tests/mask_tests.rs`
- `frontend/tests/unit_tests/filter_tests.rs`
- `frontend/tests/unit_tests/promotion_tests.rs`

Frameworks/tools:

- Rust test harness (frontend crate)
- Playwright present for E2E (not unit)

Covered frontend modules:

- `frontend/src/logic/validation.rs`
- `frontend/src/logic/mask.rs`
- `frontend/src/logic/filter.rs`
- `frontend/src/logic/promotion.rs`

Important frontend modules not unit-tested:

- `frontend/src/pages/*`
- `frontend/src/components/*`
- `frontend/src/api/*`
- `frontend/src/app.rs`

**Mandatory Verdict: Frontend unit tests: PRESENT**

### Cross-Layer Observation

- Backend has complete route-level API test coverage.
- Frontend has good logic-level unit tests and E2E presence, but page/component/api-client unit coverage remains limited.
- Balance: backend-heavy at detailed unit depth, but overall system coverage is much stronger than before.

## API Observability Check

Strong:

- Backend API tests usually show method/path + request inputs + response assertions + side effects.

Weak:

- Some Playwright tests still use smoke-style assertions (`<500`, non-empty body), not strict contracts.

## Tests Check

### Success/failure/edge/auth

- Success paths: broad and strong across all major domains.
- Failure paths: robust (401/403/404/409/400/429) coverage.
- Edge cases: present (bulk limits, stale CSRF, duplicate evidence, lockout, ownership, single-use tokens).
- Auth/permissions: extensively covered and explicit.

### Assertion depth

- Backend API tests: generally meaningful.
- Frontend E2E tests: mixed (strong in some flows, smoke-level in others).

### `run_tests.sh` policy

- Docker-based orchestration (`docker build`, `docker run`, `docker compose`) → **OK**.
- No required host package-manager install steps in script.

## End-to-End Expectations

For fullstack, FE↔BE tests are expected.

- Present: Playwright suite exists and exercises UI + API probes.
- Caveat: one route drift still visible (`/api/analytics/fund-summary` in E2E vs `/api/analytics/funds` in router).

## Test Coverage Score (0–100)

**Score: 92 / 100**

### Score Rationale

Positive:

- 100% endpoint coverage by true no-mock HTTP tests (static evidence).
- Strong auth/permission and negative-path testing.
- Good backend unit breadth + frontend logic unit suite.

Deductions:

- Frontend unit tests do not yet cover pages/components/api-client modules.
- Some E2E assertions remain shallow.
- Minor E2E endpoint drift risk.

## Key Gaps

1. Frontend unit depth gap:
   - Add unit/component tests for `frontend/src/pages/*`, `frontend/src/components/*`, `frontend/src/api/*`.
2. E2E precision gap:
   - Replace generic “not 500” checks with explicit response contract assertions.
3. E2E route drift:
   - Update stale analytics path to match router.

## Confidence & Assumptions

- Confidence: **High** on endpoint inventory and backend route coverage.
- Confidence: **Medium-high** on frontend quality conclusions (static-only limitation).
- Assumptions:
  - Dynamic path tests map to parameterized router paths.
  - Static source evidence is authoritative for this strict audit.

## Test Coverage Audit Verdict

**PASS (with quality caveats)**

---

# README Audit

## README Location

- Required file exists at `README.md`.
- Result: **PASS**.

## Hard Gates

### Formatting

- Markdown is structured and readable.
- Result: **PASS**.

### Startup Instructions (fullstack/backend)

Required: include `docker-compose up`.

- Found:
  - `docker compose up --build`
  - `docker-compose up --build`
- Result: **PASS**.

### Access Method

Required for web/fullstack: URL + port.

- Found: `Open <http://localhost:3000>.`
- Result: **PASS**.

### Verification Method

Required: clear confirmation workflow.

- Found:
  - Role-based UI verification steps
  - API smoke checks via curl
- Result: **PASS**.

### Environment Rules (STRICT)

Disallowed runtime/manual installs and manual DB setup.

- README does not require `npm install`, `pip install`, `apt-get`, or manual DB setup.
- Workflow is Docker-contained.
- Result: **PASS**.

### Demo Credentials (auth exists)

Required: username/email + password + all roles.

- Provided for admin, curator, reviewer, finance, store.
- Result: **PASS**.

## Engineering Quality

### Strengths

- Clear type declaration (`fullstack`).
- Stack and architecture are explicit.
- Structure section matches repo layout and is practical.
- Verification/testing instructions are actionable.
- Security and role concepts are documented.

### Weaknesses

- README does not explicitly distinguish smoke checks vs contract-level E2E assertions.
- Could include note to keep E2E API paths synced with router definitions.

## High Priority Issues

- **None** (hard gates all pass).

## Medium Priority Issues

1. Clarify smoke vs strict contract validation in testing narrative.
2. Add route-sync note for E2E probes to avoid stale endpoint usage.

## Low Priority Issues

1. Optional consolidation of `docker compose` vs `docker-compose` examples for consistency.

## Hard Gate Failures

- **None**.

## README Verdict

**PASS**

---

## Final Combined Verdicts

- **Test Coverage Audit:** PASS (with quality caveats)
- **README Audit:** PASS

Overall strict-mode conclusion:

- Backend API route coverage is now complete by static evidence with true no-mock HTTP tests.
- README hard-gate compliance is fully satisfied.
- Remaining opportunities are quality-focused (frontend unit depth and stricter E2E contract assertions), not coverage/compliance blockers.
