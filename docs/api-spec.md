# ScholarVault — HTTP API Specification

Generated from the actual Axum routes registered in `repo/backend/src/router.rs`
and the handler signatures under `repo/backend/src/handlers/`.

## Conventions

- **Base URL:** `http://localhost:3000` (default `HOST`/`PORT` from `.env`).
- **Request bodies** are JSON unless noted; responses are JSON unless the
  endpoint explicitly returns a file stream.
- **All error responses** share this envelope (`backend/src/error.rs::AppError::into_response`):

  ```json
  { "code": "VALIDATION_ERROR", "message": "…", "timestamp": "2026-04-11T15:30:00+00:00" }
  ```

  Common codes: `AUTH_REQUIRED` (401), `CSRF_MISSING`/`CSRF_INVALID` (403),
  `FORBIDDEN` (403), `VALIDATION_ERROR` (400), `NOT_FOUND` (404),
  `CONFLICT` (409), `ACCOUNT_LOCKED` (423), `FILE_TOO_LARGE` (413),
  `INVALID_FILE_TYPE` (400), `MIME_MISMATCH` (400), `RATE_LIMIT` (429),
  `INTERNAL`/`DATABASE_ERROR` (500).

- **Rate limit:** every `/api/**` request is limited to 60/min per authenticated
  user (or per IP for anonymous requests). On overflow the response is HTTP 429
  with `Retry-After: 60`.

- **Security headers** on every response: `Strict-Transport-Security`,
  `Content-Security-Policy`, `X-Frame-Options: DENY`, `X-Content-Type-Options:
  nosniff`, `Referrer-Policy: strict-origin-when-cross-origin`,
  `Permissions-Policy`.

- **CSRF:** every `POST` / `PUT` / `PATCH` / `DELETE` request under `/api/**`
  (except `/api/auth/login`) requires both the `csrf_token` cookie and a
  matching `X-CSRF-Token` HTTP header. Missing header → 403 `CSRF_MISSING`,
  mismatch → 403 `CSRF_INVALID`.

- **Session:** `sv_session` cookie (opaque UUID, `HttpOnly`, `SameSite=Lax`)
  plus `csrf_token` cookie (not HttpOnly so the Leptos frontend can read it and
  attach the header). Both cookies are set on successful login and cleared on
  logout. Sessions expire after 8 hours.

## Authentication

### `POST /api/auth/login`

Anonymous. Runs the 5-fails-in-15-minutes lockout check before verifying the
password.

Request:
```json
{ "username": "admin", "password": "ScholarAdmin2024!" }
```

Response `200 OK`:
```json
{
  "id": "u-admin",
  "username": "admin",
  "role": "administrator",
  "full_name": "System Administrator",
  "csrf_token": "<64-hex>"
}
```
Sets cookies: `sv_session=<uuid>; HttpOnly; SameSite=Lax; Path=/` and
`csrf_token=<64-hex>; SameSite=Lax; Path=/`.

Errors: `401 AUTH_REQUIRED` (wrong credentials), `423 ACCOUNT_LOCKED` (too many
failures in the window), `400 VALIDATION_ERROR` (empty username / password).

### `POST /api/auth/logout`

Authenticated. Deletes the session row and clears both cookies. Returns
`{"success": true}`. Requires CSRF.

### `GET /api/auth/me`

Authenticated. Returns the current user with `csrf_token`:
```json
{ "id":"u-admin","username":"admin","role":"administrator","full_name":"…","email":null,"csrf_token":"…" }
```

### `POST /api/auth/refresh-csrf`

Authenticated + CSRF. Rotates the CSRF token, updates the session row, and
sets a fresh `csrf_token` cookie.

## Admin user management

### `GET /api/admin/users` — `RequireAdmin`
Returns `Vec<UserSummary>` (no password hashes).

### `POST /api/admin/users` — `RequireAdmin` + CSRF
```json
{ "username":"new", "password":"8+chars", "role":"content_curator", "full_name": "…", "email": "…" }
```
Valid roles: `administrator`, `content_curator`, `reviewer`, `finance_manager`,
`store_manager`. Returns the created `UserSummary` and writes an
`AuditAction::Create` row.

### `POST /api/admin/users/:id/role` — `RequireAdmin` + CSRF
```json
{ "role": "reviewer" }
```
Audit: `AuditAction::RoleChange`.

### `POST /api/admin/users/:id/active` — `RequireAdmin` + CSRF
```json
{ "active": false }
```

### `GET /api/admin/audit` — `RequireAdmin`
Returns the 200 most recent `audit_logs` rows (read-only; there is no write
endpoint).

## Knowledge module

All mutations require `RequireCurator` (ContentCurator + Administrator) + CSRF.

| Method | Path | Body / Query | Notes |
|--------|------|--------------|-------|
| `GET`  | `/api/knowledge/categories`          |   | All non-deleted rows. |
| `POST` | `/api/knowledge/categories`          | `{"name":"","parent_id":null,"description":null}` | |
| `GET`  | `/api/knowledge/categories/tree`     |   | Nested `CategoryNode[]` with per-category `kp_count`. |
| `PUT`  | `/api/knowledge/categories/:id`      | `UpdateCategoryInput` | Rejects parent moves that would create a cycle (`409 CONFLICT`). |
| `DELETE` | `/api/knowledge/categories/:id`    |   | Refuses if reference count > 0. |
| `GET`  | `/api/knowledge/categories/:id/references` | | `{direct_kp_count, child_category_count, indirect_question_count, total}` |
| `POST` | `/api/knowledge/categories/merge`    | `{"source_id":"","target_id":""}` | Cycle-checked merge; `409 CONFLICT` if target is a descendant of source. |
| `GET`  | `/api/knowledge/points`              | `?category_id=&difficulty_min=&difficulty_max=&discrimination_min=&discrimination_max=&tag=` | Combined filter, limit 500. |
| `POST` | `/api/knowledge/points`              | `CreateKnowledgePointInput` | Difficulty 1..5, discrimination -1..1. |
| `PUT`  | `/api/knowledge/points/:id`          | `UpdateKnowledgePointInput` | |
| `DELETE` | `/api/knowledge/points/:id`        |   | |
| `POST` | `/api/knowledge/points/bulk/preview` | `{"ids":[…],"changes":{…}}` | Returns `ConflictPreview[]`; rejects `ids.len() > 1000` with `400`. |
| `POST` | `/api/knowledge/points/bulk/apply`   | same | Hard-capped at 1000 ids; single transaction. |
| `GET`  | `/api/knowledge/questions`           | `?knowledge_point_id=&chapter=` | |
| `POST` | `/api/knowledge/questions`           | `CreateQuestionInput` | |
| `PUT`  | `/api/knowledge/questions/:id`       | `UpdateQuestionInput` | |
| `DELETE` | `/api/knowledge/questions/:id`     |   | Also removes `knowledge_question_links`. |
| `POST` | `/api/knowledge/questions/:id/link`  | `{"knowledge_point_id":""}` | |

## Outcome / IP module

Mutations require `RequireReviewer` (Reviewer + Administrator) + CSRF.

| Method | Path | Notes |
|--------|------|-------|
| `GET`  | `/api/outcomes` | 200 most recent (newest first). |
| `GET`  | `/api/outcomes/:id` | Returns `{outcome, contributors, evidence}` bundle. |
| `POST` | `/api/outcomes` | Body `CreateOutcomeInput` — `{type,title,abstract_snippet,certificate_number?}`. Runs Jaro-Winkler duplicate detection (title ≥ 0.85, abstract[:200] ≥ 0.80, exact certificate number) and returns `{outcome, duplicate_candidates}`. |
| `POST` | `/api/outcomes/:id/contributors` | Body `AddContributorInput`. Rejects additions that would push the running total over 100. |
| `DELETE` | `/api/outcomes/:id/contributors/:cid` | |
| `POST` | `/api/outcomes/:id/submit` | Refuses with `400 VALIDATION_ERROR` unless `SUM(share_percentage) == 100` exactly. |
| `POST` | `/api/outcomes/:id/approve` | Records `submitted_at → approved_at` cycle time in `approval_cycle_records`. |
| `POST` | `/api/outcomes/:id/reject` | Body `{ "reason": "…" }`. |
| `POST` | `/api/outcomes/:id/evidence` | **Multipart form-data**, one field `file`. Validators: `≤ 25 MB`, magic-number via `infer`, declared MIME must equal detected MIME, only `application/pdf`/`image/jpeg`/`image/png`. Returns the new `EvidenceFile`. Duplicate SHA-256 fingerprint → `409 CONFLICT`. Bytes are AES-256-GCM encrypted before they land in `<evidence_dir>/<outcome_id>/<file_id>`. |
| `GET`  | `/api/outcomes/:id/compare/:other_id` | Returns `{a, b, title_similarity, abstract_similarity}` with live Jaro-Winkler scores. |

## Store / Promotions

Management (product/promotion create/deactivate) requires `RequireStore` + CSRF.
Checkout and order listing require only an authenticated session.

| Method | Path | Notes |
|--------|------|-------|
| `GET`  | `/api/store/products` | Active products only. |
| `POST` | `/api/store/products` | `CreateProductInput`. |
| `GET`  | `/api/store/promotions` | Ordered by `priority DESC, created_at DESC`. |
| `POST` | `/api/store/promotions` | `CreatePromotionInput` — enforces `discount_type in ('percent','fixed')` and `effective_from < effective_until`. |
| `POST` | `/api/store/promotions/:id/deactivate` | |
| `POST` | `/api/store/checkout/preview` | Public best-offer dry run; no persistence. |
| `POST` | `/api/store/checkout` | Body `{items:[CartItem]}`. Runs the promotion engine (filter window → resolve exclusion groups → pick best → distribute proportionally), inserts an `orders` row + per-line `order_items` with JSON `promotion_trace`, returns `{order, result}`. Writes an `AuditAction::Checkout` row. |
| `GET`  | `/api/store/orders` | Current user's orders (most recent 100). |
| `GET`  | `/api/store/orders/:id` | `{order, items}`. |

## Analytics

Read endpoints require an authenticated session. Fund summary + CSV / PDF
exports require `RequireFinance` (Administrator + FinanceManager).

| Method | Path | Notes |
|--------|------|-------|
| `GET`  | `/api/analytics/members` | `MemberMetrics` — total, new, churned, 12-point series. |
| `GET`  | `/api/analytics/churn` | Churn rate over the last two snapshots. |
| `GET`  | `/api/analytics/events` | `EventSummary`. |
| `GET`  | `/api/analytics/funds` | `?period=2026-04` optional filter. Finance-gated. Includes `budget_cap=2500.00` and `over_budget` flag. |
| `GET`  | `/api/analytics/approval-cycles` | `{count, avg_minutes, median_minutes, slowest[]}`. |
| `POST` | `/api/analytics/export/csv` | Finance-gated. Body `{report_type: "fund"\|"members"\|"events", period: null}`. Returns `text/csv` attachment. Audit: `AuditAction::ExportReport`. |
| `POST` | `/api/analytics/export/pdf` | Finance-gated. Returns `application/pdf` attachment. |
| `POST` | `/api/analytics/reports/schedule` | Authenticated. Body `{report_type, format: "csv"\|"pdf", period: null}`. Generates the file inline, returns the record with `status="complete"` and a single-use `download_token`. |
| `GET`  | `/api/analytics/reports` | Current user's scheduled reports. |
| `GET`  | `/api/analytics/reports/:id/download/:token` | Streams the file with the correct MIME and clears the token from the row. Second call with the same token → `404 NOT_FOUND`. |

## Backup & Restore

Admin-only (`RequireAdmin`). Mutations require CSRF.

| Method | Path | Notes |
|--------|------|-------|
| `GET`  | `/api/backup/history` | 200 most recent `backup_records` rows. |
| `POST` | `/api/backup/run` | Builds a tar.gz of the SQLite file + evidence directory, AES-256-GCM encrypts it, writes to `<backup_dir>/<YYYYMMDD>-<type>-<short>.bin`, records the SHA-256. Type is `monthly` on the last calendar day, else `daily`. |
| `POST` | `/api/backup/:id/restore-sandbox` | Decrypts the bundle into a `tempdir`, runs SHA-256 verify + `PRAGMA integrity_check` + `SELECT COUNT(*) FROM users`, returns `SandboxValidationReport`. Live DB untouched. |
| `POST` | `/api/backup/:id/activate` | Re-runs sandbox validation and refuses (`409 CONFLICT`) unless `all_passed`. Marks `backup_records.restored_at`. |
| `POST` | `/api/backup/lifecycle-cleanup` | Applies the retention policy: daily > `daily_retention` days and monthly > `monthly_retention * 30` days are purged (file deleted, row status → `purged`). `preserve_financial` / `preserve_ip` flags honoured via bundle-path markers. Returns `{purged_daily, purged_monthly, preserved_financial, preserved_ip}`. |
| `GET`  | `/api/backup/policy` | Current `retention_policies` row. |

## Health

| Method | Path | Notes |
|--------|------|-------|
| `GET`  | `/healthz`        | Public. |
| `GET`  | `/api/healthz`    | Public. |

## Static asset fallback

Any request that does not match an API or health route is served by
`tower_http::services::ServeDir` from `$STATIC_DIR` (defaults to `/app/dist`).
Unknown paths fall back to `index.html`, which is what powers Leptos client-side
routing (`/login`, `/knowledge`, `/outcomes`, `/store`, `/analytics`, `/admin`).
