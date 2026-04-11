# ScholarVault — Design Document

Derived from the Phase 0–9 implementation. Every reference is backed by a real
`repo/...` file; the static audit can jump straight from this document to the
implementing code.

## 1. Architecture at a glance

```
                    ┌─────────────────────────────────────────┐
                    │  Browser (Leptos WASM @ /dist)          │
                    │  • pages/login, knowledge, outcomes,    │
                    │    store, analytics, admin              │
                    │  • api/client.rs — gloo-net wrapper     │
                    │    auto-attaches X-CSRF-Token           │
                    └──────────────┬──────────────────────────┘
                                   │ HTTPS / localhost:3000
                                   ▼
                    ┌─────────────────────────────────────────┐
                    │  Axum 0.7 backend (/app/scholarvault)   │
                    │  Middleware stack, outer → inner:       │
                    │   security_headers → session →          │
                    │     rate_limit → csrf → handler         │
                    │  handlers/{auth, knowledge, outcomes,   │
                    │    store, analytics, backup}            │
                    └──┬─────────────┬────────────┬───────────┘
                       ▼             ▼            ▼
            ┌────────────────┐ ┌────────────┐ ┌────────────────┐
            │  SQLite (WAL)  │ │  /app/     │ │  /app/backups/ │
            │  /app/data/    │ │  evidence/ │ │  encrypted     │
            │  scholarvault  │ │  AES-256-  │ │  tar.gz.bin    │
            │  .db           │ │  GCM blobs │ │  bundles       │
            └────────────────┘ └────────────┘ └────────────────┘
```

## 2. Cargo workspace (3 crates)

| Crate       | Path                   | Purpose |
|-------------|------------------------|---------|
| `shared`    | `repo/shared/`         | DTO enums (`UserRole`, `AuditAction`, `OutcomeType`, `PromotionType`, `DiscriminationBand`) and the `ErrorResponse` envelope shared by both sides of the wire. |
| `backend`   | `repo/backend/`        | Axum server, service layer, SQLx models, middleware, migrations. Exposes a `lib` crate so integration tests construct the real router via `build_router()`. |
| `frontend`  | `repo/frontend/`       | Leptos CSR application, compiled to WASM by trunk. `logic/` holds pure-Rust helpers (validation, masking, filter state, promotion display) that the native test harness can exercise without a browser. |

Root `repo/Cargo.toml` declares `members = ["backend", "frontend", "shared"]`
and a `[workspace.dependencies]` block that pins axum 0.7, sqlx 0.7, tower 0.5,
leptos 0.6 (csr), argon2 0.5, aes-gcm 0.10, governor 0.6, infer 0.15,
strsim 0.11, printpdf 0.7, tokio-cron-scheduler 0.9, flate2, tar, and (for the
wasm target) `uuid` with the `js` feature.

## 3. Docker layout

| File                   | Stage | Responsibility |
|------------------------|-------|----------------|
| `repo/Dockerfile`      | `wasm-builder` (rust:slim + trunk 0.20.3) | `trunk build --release --filehash false` against `frontend/Cargo.toml` → `/build/dist/` |
|                        | `backend-builder` (rust:slim) | `cargo build --release -p backend --bin scholarvault` |
|                        | `runtime` (debian-slim) | Copies the binary, `dist/`, and migrations into `/app/`, exposes port 3000, runs `/app/scholarvault`. |
| `repo/Dockerfile.test` | single stage rust:slim | Runs `run_tests.sh` (four cargo test suites). |
| `repo/docker-compose.yml` | one file | `setup` (copies `.env.example` → `.env` on first run) → `app` (build + 3000:3000 + named volumes for data/evidence/backups) → `test` (profile-gated). |

`run_tests.sh` runs all four suites with `--test-threads=1` and exits non-zero
if any fails, emitting `ALL TESTS PASSED` on success.

## 4. Database schema (10 migrations)

Every migration lives in `backend/src/db/migrations/*.sql` and is loaded by the
runtime `Migrator::new` in `backend/src/db/mod.rs`.

| File                             | Tables |
|----------------------------------|--------|
| `0001_create_users.sql`          | `users` (PII columns `phone_encrypted`, `national_id_encrypted` are AES-256-GCM ciphertext) |
| `0002_create_login_tracking.sql` | `login_attempts`, `sessions` (csrf_token, expires_at) |
| `0003_create_knowledge.sql`      | `categories` (DAG via `parent_id`), `knowledge_points` (`CHECK(difficulty BETWEEN 1 AND 5)`, `CHECK(discrimination BETWEEN -1 AND 1)`) |
| `0004_create_question_bank.sql`  | `questions`, `knowledge_question_links` (composite PK) |
| `0005_create_outcomes.sql`       | `outcomes` (`CHECK(type IN ('paper','patent','competition_result','software_copyright'))`), `outcome_contributors` (share 0..100), `evidence_files` (UNIQUE `sha256_fingerprint`) |
| `0006_create_store.sql`          | `products`, `promotions` (`mutual_exclusion_group`, `priority`, `effective_from/until`), `orders`, `order_items` (JSON `promotion_trace`) |
| `0007_create_analytics.sql`      | `member_snapshots`, `event_participation`, `fund_transactions`, `approval_cycle_records`, `scheduled_reports` |
| `0008_create_audit.sql`          | `audit_logs` — **no `updated_at` column**, append-only by design |
| `0009_create_backup.sql`         | `backup_records`, `retention_policies` (seeded with `id='default'`) |
| `0010_seed_users.sql`            | 5 role accounts with real Argon2id PHC hashes, plus a starter category tree, knowledge points, products, seed promotions, fund transactions, member snapshots, events |

## 5. Security architecture

All security features are **explicitly coded** — not merely configured via
Cargo.toml. Each item below points to the exact file that implements it.

| Feature | Implementation |
|---------|----------------|
| **Argon2id password hash/verify** | `backend/src/security/password.rs` — `hash_password` generates a random salt and returns a self-describing PHC string; `verify_password` re-parses it and delegates to the argon2 crate. The seeded admin hash in `0010_seed_users.sql` was generated with python `argon2-cffi` (m=65536, t=3, p=4) and is verified at runtime by `test_phc_hash_from_seed_format_parses`. |
| **AES-256-GCM field encryption** | `backend/src/security/encryption.rs` — `encrypt_field` / `decrypt_field` produce `base64(nonce[12] || ciphertext)`. Each call generates a fresh random nonce; same plaintext → different ciphertext each time (proven by `test_encrypt_different_nonce_each_time`). Plus `encrypt_bytes` / `decrypt_bytes` for the binary backup bundle path. |
| **Last-4 masking** | `encryption::mask_sensitive` — char-based (Unicode-safe); mirrored in `frontend/src/logic/mask.rs::mask_last4`. |
| **CSRF** | `backend/src/middleware/csrf.rs` — skips safe verbs and the `/api/auth/login` bootstrap; pulls `X-CSRF-Token` header and `csrf_token` cookie, compares with `constant_time_eq::constant_time_eq`. Token generation in `backend/src/security/csrf.rs` uses 32 random bytes hex-encoded. |
| **Security headers** | `backend/src/middleware/security_headers.rs` adds `Strict-Transport-Security`, `Content-Security-Policy`, `X-Frame-Options: DENY`, `X-Content-Type-Options: nosniff`, `Referrer-Policy: strict-origin-when-cross-origin`, `Permissions-Policy` to every response. |
| **Rate limit 60 req/min/user** | `backend/src/middleware/rate_limit.rs` — governor keyed by `user:{id}` (or `ip:{ip}` for anonymous), returns HTTP 429 with `Retry-After: 60`. |
| **Account lockout (5 failures / 15-minute window)** | `backend/src/services/auth_service.rs::check_lockout` — counts unsuccessful `login_attempts` for `username OR ip_address` where `attempted_at > now() - 15min`, returns `AppError::AccountLocked` (HTTP 423) when the count ≥ 5. Proven end-to-end in `test_lockout_blocks_after_5_failures`. |
| **Audit log (append-only)** | `backend/src/services/audit_service.rs::AuditService` has **only** `log()` and `compute_hash()` — the impl block is intentionally closed. Read queries live on a separate `AuditQuery` type. No `UPDATE audit_logs` / `DELETE audit_logs` statement exists anywhere in the codebase (grep-verified in Phase 9). |
| **File validation** | `backend/src/services/file_service.rs::validate_file` runs infer magic-number detection, rejects anything outside `application/pdf`/`image/jpeg`/`image/png`, enforces a 25 MB size cap, and refuses MIME mismatches. Evidence bytes are SHA-256-fingerprinted before upload and duplicate fingerprints return HTTP 409. |

## 6. Knowledge DAG

`backend/src/services/knowledge_service.rs`.

- **`check_would_create_cycle`** — DFS from `child_id` over existing
  `parent_id → child` edges; if the target `parent_id` is reachable from there,
  adding the edge would create a cycle. Also returns `true` for the trivial
  `parent == child` case.
- **`get_reference_count`** — returns `{ direct_kp_count, child_category_count,
  indirect_question_count, total }` so the merge UI can show exactly what is
  about to move.
- **`merge_nodes(source, target)`** — refuses self-merges, validates that
  `target` is not a descendant of `source`, then in a single transaction
  re-parents `categories` + `knowledge_points` under `target` and soft-deletes
  `source`. On success writes an `AuditAction::MergeNodes` entry.
- **Bulk edit** — hard cap at 1000 ids (`MAX_BULK_EDIT`); requests exceeding
  the limit are rejected before the SQL runs. `preview_bulk_conflicts` returns
  one `ConflictPreview` per (row, field) so the UI can show what would change.

## 7. Outcome duplicate-detection pipeline

`backend/src/services/outcome_service.rs::find_duplicates` runs, in order:

1. **Exact certificate number match** — any candidate with an identical non-empty
   `certificate_number` is flagged with similarity 1.0.
2. **Title Jaro-Winkler ≥ 0.85** — `strsim::jaro_winkler` over the raw titles.
3. **Abstract Jaro-Winkler ≥ 0.80** — computed over the **first 200 characters**
   (char-safe) of the abstracts via the `head` helper.

The candidate list is returned from `create_outcome` alongside the new row so
the UI can show a non-blocking amber warning plus a "View Comparison" link to
`/api/outcomes/:id/compare/:other_id`.

## 8. Promotion engine

`backend/src/services/store_service.rs::apply_best_promotion`:

1. **Filter by effective window** — `is_active == 1` and `effective_from <= now
   <= effective_until` (RFC3339 boundaries, both inclusive).
2. **Resolve mutual exclusion groups** — `resolve_exclusion_groups` keeps only
   the highest-priority promotion per group; promos with no group survive
   automatically.
3. **Best discount** — picks the surviving promotion with the greatest
   `total_cart_discount` (percent → `subtotal × value/100`, fixed → `min(value,
   subtotal)`).
4. **Proportional line-item distribution** — the chosen total discount is
   spread across cart lines by `line_subtotal / cart_subtotal`, so every
   `LineItemResult` carries a `discount_amount`, `line_total`, and
   `promotion_applied` name for the UI trace. The full trace is persisted as
   JSON in `order_items.promotion_trace`.

## 9. Analytics

`backend/src/services/analytics_service.rs`:

- **Member churn** = `latest.churned / prior.total × 100`, yielding 0 when
  there is only one snapshot.
- **Fund summary** includes the literal SPEC example `FUND_BUDGET_CAP =
  2500.00` and sets `over_budget = total_expense > cap`.
- **Approval cycle stats** returns `count`, `avg_minutes`, `median_minutes`,
  and the 5 slowest records.
- **CSV export** (`generate_csv`) writes explicit header rows for `fund`,
  `members`, and `events` reports.
- **PDF export** (`generate_pdf`) is structured so that *all async database
  work finishes before* a `printpdf::PdfDocument` is instantiated. The sync
  `render_pdf` helper runs on the bytes; the `Rc<RefCell<…>>` inside
  `PdfDocumentReference` never crosses an `.await`, keeping the handler future
  `Send`.
- **Scheduled reports** — `schedule_report` inserts a pending row, generates
  the file inline, writes it to `<reports_dir>/<id>.<csv|pdf>`, and issues a
  fresh single-use `download_token`. `download_report(id, token)` validates the
  token, streams the bytes, and **clears the token from the row** so the same
  URL cannot be reused.

## 10. Backup, restore, and retention

`backend/src/services/backup_service.rs`:

- **`run_backup`** — `tar` + `gzip` the SQLite file and the evidence directory
  into a single in-memory buffer, AES-256-GCM encrypt with
  `encryption::encrypt_bytes`, write to
  `<backup_dir>/<YYYYMMDD>-<type>-<short_id>.bin`, SHA-256 the encrypted bundle,
  insert into `backup_records`. `backup_type` is `monthly` on the last calendar
  day of the month (via `is_last_day_of_month`), else `daily`.
- **`restore_to_sandbox`** runs three independent checks:
  1. SHA-256 of the bundle file equals `backup_records.sha256_hash`
  2. `PRAGMA integrity_check` == `"ok"` on the extracted SQLite file
  3. `SELECT COUNT(*) FROM users` succeeds

  Only when `all_passed` does `activate_restore` mark
  `backup_records.restored_at`. The live database is never touched unless
  activation is explicitly called with a validated bundle.
- **`apply_lifecycle_cleanup`** removes daily bundles older than
  `policy.daily_retention` days and monthly bundles older than
  `policy.monthly_retention * 30` days, and **honours the
  `preserve_financial` / `preserve_ip` flags** by skipping any bundle whose
  path contains those markers. Purged rows are marked `status='purged'`.
- **`backup_scheduler`** — `services/backup_scheduler.rs` registers a
  `tokio_cron_scheduler` job reading `BACKUP_SCHEDULE` (default
  `"0 0 2 * * *"` — 02:00 daily) that calls `run_backup` on the cadence. The
  runtime scheduler is started in `main.rs`; integration tests skip it.

## 11. Test architecture

Four suites, as required by the CLAUDE.md checklist:

| Suite                       | Path                                   | What it proves |
|-----------------------------|----------------------------------------|----------------|
| `backend unit_tests`        | `backend/tests/unit_tests/`            | Isolated service-layer logic: Argon2 hashes round-trip, AES-GCM nonce uniqueness, DAG cycle detection, bulk-edit 1000-record cap, Jaro-Winkler thresholds, share validation, file magic numbers, promotion engine, analytics formulas, backup type classification. **61 tests.** |
| `backend api_tests`         | `backend/tests/api_tests/`             | Real Axum router over an in-memory SQLite with migrations. Covers login + lockout (5 fails → 423), CSRF missing (403), role gating (curator/reviewer/finance/store/admin 403 boundaries), outcome register-and-submit flow, evidence duplicate fingerprint (409), checkout with seeded promotions, analytics CSV export, single-use download token (200 then 404), backup run + sandbox validation. **28 tests.** |
| `frontend unit_tests`       | `frontend/tests/unit_tests/`           | Pure Rust logic: `mask_last4`, share-total state machine, date-range validator, filter state combined mode, discrimination band presets, promotion display helpers, MM/DD/YYYY conversion, total-savings formatting. **23 tests.** |
| `frontend api_tests`        | `frontend/tests/api_tests/`            | gloo-net client serialization against the backend DTO shapes: `LoginRequest/Response/MeResponse`, `KnowledgePoint`/`KnowledgeFilter` query string, `CreateOutcomeInput` with serde `rename = "type"`, `CheckoutRequest/Response` + line-item trace, `CreatePromotionInput` key coverage. **20 tests.** |

All four run via `sh run_tests.sh` (also from `docker compose --profile test
run --build test`); `ALL TESTS PASSED` is printed on success.
