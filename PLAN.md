# PLAN.md — ScholarVault Execution Plan
# Task ID: TASK-32-W2
# [ ] = pending  [x] = complete
# Rule: Complete ALL tasks in a phase without stopping. Pause ONLY at phase boundaries.
# Fix Rust compile errors within the same task before marking [x].
# CRITICAL: QA reads code (static) AND runs Docker (runtime). Both must be perfect.
# TEST RULE: Backend AND Frontend each have unit_tests/ and api_tests/ folders.

---

## PHASE 0 — Cargo Workspace, Docker Infrastructure & Project Foundation
> Goal: Workspace compiles, all Docker files correct, test folder skeleton created
> Complete all tasks continuously, then pause. Wait for "proceed".

- [x] 0.1 Create repo/.gitignore (content from CLAUDE.md — .env excluded, .env.example NOT excluded)
- [x] 0.2 Create repo/.env.example (committed to git, content from CLAUDE.md)
- [x] 0.3 Create repo/README.md (minimal format from CLAUDE.md)
- [x] 0.4 Create repo/Cargo.toml — workspace root, members=["backend","frontend","shared"], all [workspace.dependencies] from CLAUDE.md
- [x] 0.5 Create repo/shared/Cargo.toml + repo/shared/src/lib.rs — shared DTOs: UserRole, AuditAction, OutcomeType (Paper/Patent/CompetitionResult/SoftwareCopyright), PromotionType, DifficultyLevel, ApiResponse<T>, ErrorResponse. All: #[derive(Debug,Clone,Serialize,Deserialize,PartialEq)]
- [x] 0.6 Create repo/backend/Cargo.toml — all backend deps from CLAUDE.md workspace deps
- [x] 0.7 Create repo/frontend/Cargo.toml — leptos, leptos_router, leptos_meta, gloo-net, wasm-bindgen, web-sys, shared crate
- [x] 0.8 Create repo/tailwind.config.js — golden dark SaaS theme from CLAUDE.md, content paths for .rs files
- [x] 0.9 Create repo/style/main.scss — TailwindCSS directives + CSS custom properties from CLAUDE.md
- [x] 0.10 Create repo/Dockerfile — multi-stage: wasm-builder (install cargo-leptos + wasm32 target + build frontend), backend-builder (cargo build --release -p backend), runtime (debian-slim, copy binary + site, expose 3000)
- [x] 0.11 Create repo/Dockerfile.test — rust:1.75-slim, COPY workspace, RUN cargo fetch, chmod +x run_tests.sh, CMD ["sh","run_tests.sh"]
- [x] 0.12 Create repo/docker-compose.yml — single file, exact content from CLAUDE.md (setup + app + test[profile:test])
- [x] 0.13 Create repo/run_tests.sh — exact content from CLAUDE.md (4 suites: backend unit, backend api, frontend unit, frontend api), chmod +x
- [x] 0.14 Create backend/src/main.rs — Axum entry: load env, SQLite pool with WAL mode (PRAGMA journal_mode=WAL + busy_timeout=5000), run migrations, build router, bind HOST:PORT
- [x] 0.15 Create backend/src/error.rs — AppError enum (Auth, CsrfMissing, CsrfInvalid, RateLimit, Validation(String), NotFound, Forbidden, Conflict, Internal(String), AccountLocked{message:String}, FileTooLarge{size:usize,max:usize}, InvalidFileType(String), MimeMismatch{declared:String,detected:String}, UnknownFileType) + axum IntoResponse returning proper HTTP status + JSON {code,message,timestamp}
- [x] 0.16 Create frontend/src/main.rs — Leptos CSR mount_to_body; frontend/src/app.rs — root App + leptos_router Routes
- [x] 0.17 Create backend test folder skeleton:
       backend/tests/unit_tests/mod.rs — declares: mod auth_tests; mod knowledge_tests; mod outcome_tests; mod store_tests; mod analytics_tests; mod file_tests; mod backup_tests;
       backend/tests/unit_tests/auth_tests.rs — placeholder #[test] fn placeholder_auth() {}
       backend/tests/unit_tests/knowledge_tests.rs — placeholder
       backend/tests/unit_tests/outcome_tests.rs — placeholder
       backend/tests/unit_tests/store_tests.rs — placeholder
       backend/tests/unit_tests/analytics_tests.rs — placeholder
       backend/tests/unit_tests/file_tests.rs — placeholder
       backend/tests/unit_tests/backup_tests.rs — placeholder
       backend/tests/api_tests/mod.rs — declares: mod auth_api; mod knowledge_api; mod outcome_api; mod store_api; mod analytics_api; mod backup_api;
       backend/tests/api_tests/auth_api.rs — placeholder #[test] fn placeholder_auth_api() {}
       [same placeholder pattern for all api_tests submodules]
- [x] 0.18 Create frontend test folder skeleton:
       frontend/tests/unit_tests/mod.rs — declares: mod validation_tests; mod promotion_tests; mod mask_tests; mod filter_tests;
       frontend/tests/unit_tests/validation_tests.rs — placeholder #[test] fn placeholder_validation() {}
       frontend/tests/unit_tests/promotion_tests.rs — placeholder
       frontend/tests/unit_tests/mask_tests.rs — placeholder
       frontend/tests/unit_tests/filter_tests.rs — placeholder
       frontend/tests/api_tests/mod.rs — declares: mod auth_client_tests; mod knowledge_client_tests; mod outcome_client_tests; mod store_client_tests;
       frontend/tests/api_tests/auth_client_tests.rs — placeholder #[test] fn placeholder_auth_client() {}
       [same placeholder pattern for all api_tests submodules]
- [x] 0.19 Verify: cargo build --workspace succeeds with zero errors. Fix all compile errors before marking done.

**Phase 0 checkpoint: cargo build --workspace succeeds with zero errors. All Docker files created. docker compose up --build starts the app at http://localhost:3000 (login page visible). Both backend/ and frontend/ have tests/unit_tests/ and tests/api_tests/ folders.**

---

## PHASE 1 — Database Schema (SQLite Migrations + Models)
> Goal: All tables created via migrations, SQLx models compile, seed data enables immediate login
> Complete all tasks continuously, then pause. Wait for "proceed".

- [x] 1.1 Create backend/src/db/mod.rs — SqlitePool init with WAL pragma, run_migrations() using sqlx::migrate!("./migrations"), connection options with busy_timeout
- [x] 1.2 V001__create_users.sql: users (id TEXT PK, username TEXT UNIQUE NOT NULL, password_hash TEXT NOT NULL, role TEXT NOT NULL, is_active INT DEFAULT 1, phone_encrypted TEXT, national_id_encrypted TEXT, created_at TEXT, updated_at TEXT)
- [x] 1.3 V002__create_login_tracking.sql: login_attempts (id TEXT PK, username TEXT, ip_address TEXT, attempted_at TEXT, success INT), sessions (id TEXT PK, user_id TEXT, csrf_token TEXT, expires_at TEXT, created_at TEXT)
- [x] 1.4 V003__create_knowledge.sql: categories (id TEXT PK, name TEXT NOT NULL, parent_id TEXT NULL, level INT, description TEXT, created_by TEXT, created_at TEXT, updated_at TEXT, deleted_at TEXT), knowledge_points (id TEXT PK, category_id TEXT, title TEXT, content TEXT, difficulty INT CHECK(difficulty BETWEEN 1 AND 5), discrimination REAL CHECK(discrimination BETWEEN -1.0 AND 1.0), tags TEXT — JSON array, created_by TEXT, created_at TEXT, updated_at TEXT)
- [x] 1.5 V004__create_question_bank.sql: questions (id TEXT PK, knowledge_point_id TEXT, question_text TEXT, question_type TEXT, options TEXT — JSON, correct_answer TEXT, explanation TEXT, chapter TEXT, created_by TEXT, created_at TEXT), knowledge_question_links (knowledge_point_id TEXT, question_id TEXT, PRIMARY KEY(knowledge_point_id, question_id))
- [x] 1.6 V005__create_outcomes.sql: outcomes (id TEXT PK, type TEXT CHECK(type IN ('paper','patent','competition_result','software_copyright')), title TEXT NOT NULL, abstract_snippet TEXT, certificate_number TEXT, status TEXT DEFAULT 'draft', created_by TEXT, created_at TEXT, updated_at TEXT), outcome_contributors (id TEXT PK, outcome_id TEXT, user_id TEXT, share_percentage INT, role_in_work TEXT), evidence_files (id TEXT PK, outcome_id TEXT, filename TEXT, mime_type TEXT, stored_path TEXT, file_size INT, sha256_fingerprint TEXT UNIQUE, uploaded_by TEXT, uploaded_at TEXT)
- [x] 1.7 V006__create_store.sql: products (id TEXT PK, name TEXT, description TEXT, price REAL, stock_quantity INT, is_active INT, created_by TEXT, created_at TEXT), promotions (id TEXT PK, name TEXT, discount_value REAL, discount_type TEXT CHECK(discount_type IN ('percent','fixed')), effective_from TEXT, effective_until TEXT, mutual_exclusion_group TEXT, priority INT DEFAULT 0, is_active INT, created_by TEXT, created_at TEXT), orders (id TEXT PK, user_id TEXT, status TEXT, subtotal REAL, discount_applied REAL, total REAL, created_at TEXT), order_items (id TEXT PK, order_id TEXT, product_id TEXT, quantity INT, unit_price REAL, discount_amount REAL, promotion_applied TEXT, promotion_trace TEXT — JSON)
- [x] 1.8 V007__create_analytics.sql: member_snapshots (id TEXT PK, snapshot_date TEXT, total_members INT, new_members INT, churned_members INT, created_at TEXT), event_participation (id TEXT PK, event_name TEXT, event_date TEXT, participant_count INT, category TEXT, created_at TEXT), fund_transactions (id TEXT PK, type TEXT CHECK(type IN ('income','expense')), amount REAL, category TEXT, description TEXT, budget_period TEXT, recorded_by TEXT, created_at TEXT), approval_cycle_records (id TEXT PK, entity_type TEXT, entity_id TEXT, submitted_at TEXT, approved_at TEXT, approver_id TEXT, cycle_time_minutes INT), scheduled_reports (id TEXT PK, report_type TEXT, filters TEXT, status TEXT DEFAULT 'pending', file_path TEXT, download_token TEXT UNIQUE, created_by TEXT, created_at TEXT, completed_at TEXT)
- [x] 1.9 V008__create_audit.sql: audit_logs (id TEXT PK, actor_id TEXT NOT NULL, action TEXT NOT NULL, entity_type TEXT, entity_id TEXT, before_hash TEXT, after_hash TEXT, ip_address TEXT, created_at TEXT NOT NULL — NO updated_at, append-only)
- [x] 1.10 V009__create_backup.sql: backup_records (id TEXT PK, backup_type TEXT CHECK(backup_type IN ('daily','monthly')), bundle_path TEXT, sha256_hash TEXT, status TEXT DEFAULT 'complete', size_bytes INT, created_at TEXT, expires_at TEXT, restored_at TEXT)
- [x] 1.11 V010__seed_users.sql: INSERT 5 users with pre-computed Argon2id hashes for ScholarAdmin2024! (admin) and Scholar2024! (curator, reviewer, finance, store). Compute real hashes — not placeholder strings.
- [x] 1.12 Create all SQLx model structs in backend/src/models/ — user.rs, knowledge.rs, outcome.rs, store.rs, analytics.rs, audit.rs, backup.rs. All structs: #[derive(sqlx::FromRow, serde::Serialize, serde::Deserialize, Debug, Clone)]
- [x] 1.13 Verify: cargo build --package backend succeeds — sqlx compile-time query checking passes.

**Phase 1 checkpoint: cargo build --package backend succeeds. All 10 migration files are syntactically valid SQL.**

---

## PHASE 2 — Auth, Security Middleware & CSRF
> Goal: Login works in Docker for all 5 roles, lockout enforced, CSRF active, rate limit active — all clearly coded
> Complete all tasks continuously, then pause. Wait for "proceed".

- [x] 2.1 Create backend/src/security/password.rs — hash_password() + verify_password() using Argon2, exactly as in CLAUDE.md spec with doc comments
- [x] 2.2 Create backend/src/security/encryption.rs — encrypt_field() + decrypt_field() + mask_sensitive() using AES-256-GCM, exactly as in CLAUDE.md spec with doc comments
- [x] 2.3 Create backend/src/middleware/csrf.rs — csrf_middleware() axum middleware, exactly as in CLAUDE.md spec with doc comments
- [x] 2.4 Create backend/src/middleware/security_headers.rs — Tower middleware adding: Strict-Transport-Security, Content-Security-Policy, X-Frame-Options: DENY, X-Content-Type-Options: nosniff, Referrer-Policy: strict-origin
- [x] 2.5 Create backend/src/middleware/rate_limit.rs — RateLimitLayer using governor crate (60 req/min/user keyed by user_id), returns 429 + Retry-After header, exactly as in CLAUDE.md spec
- [x] 2.6 Create backend/src/services/auth_service.rs — login(), logout(), check_lockout(), record_attempt(), get_current_user() — exactly as in CLAUDE.md spec with doc comments
- [x] 2.7 Create backend/src/services/audit_service.rs — AuditService with ONLY log() + compute_hash() — NO update/delete methods — exactly as in CLAUDE.md with doc comment explaining append-only design
- [x] 2.8 Create backend/src/middleware/session.rs — extract session from cookie, load CurrentUser from DB, attach to request extensions
- [x] 2.9 Create backend/src/middleware/require_role.rs — axum extractor that checks CurrentUser.role against required roles, returns 403 if mismatch
- [x] 2.10 Create backend/src/handlers/auth.rs — POST /api/auth/login, POST /api/auth/logout, GET /api/auth/me, POST /api/auth/refresh-csrf
- [x] 2.11 Create backend/src/router.rs — register all routes with middleware layers (security headers on all, csrf on state-changing, rate_limit on /api/**, require_role per route group)
- [x] 2.12 Create frontend/src/pages/login.rs — Leptos login page: golden-gradient dark card centered on screen, username + password inputs with gold focus ring, primary login button with loading spinner, error display (locked account message with countdown), CSRF token included in POST headers
- [x] 2.13 Fill in backend unit tests (backend/tests/unit_tests/auth_tests.rs):
       test_password_hash_and_verify(), test_wrong_password_fails_verify(), test_mask_sensitive_short_value(), test_mask_sensitive_long_value(), test_encrypt_decrypt_roundtrip(), test_encrypt_different_nonce_each_time(), test_lockout_check_threshold_5()
- [x] 2.14 Fill in backend API tests (backend/tests/api_tests/auth_api.rs):
       test_login_valid_credentials_returns_200(), test_login_wrong_password_returns_401(), test_post_without_csrf_returns_403(), test_csrf_valid_token_passes(), test_logout_invalidates_session()
- [x] 2.15 Fill in frontend unit tests (frontend/tests/unit_tests/mask_tests.rs):
       test_mask_phone_shows_last_4(), test_mask_id_shows_last_4(), test_mask_short_value_all_stars(), test_mask_exactly_4_chars_unchanged()
- [x] 2.16 Verify: cargo build --workspace succeeds.

**Phase 2 checkpoint: docker compose up --build → all 5 credentials from README log in successfully and redirect to correct dashboards. POST without CSRF token returns 403. Rate limit returns 429 on 61st request. Login page renders with golden theme.**

---

## PHASE 3 — Knowledge Management Module
> Goal: Category DAG with cycle detection, knowledge points, question bank, bulk edit 1000, combined filters
> Complete all tasks continuously, then pause. Wait for "proceed".

- [x] 3.1 Create backend/src/services/knowledge_service.rs: create_category(), update_category(), delete_category(), get_tree() (recursive), check_would_create_cycle() DFS exactly as CLAUDE.md, get_reference_count(node_id) → {direct_kp_count, child_category_count, indirect_question_count, total}, merge_nodes(source_id, target_id) → validates no cycle + no orphans → moves all relations → audit log
- [x] 3.2 Add to knowledge_service: create_knowledge_point(), update_knowledge_point(), bulk_update(ids: Vec<String>, changes: BulkUpdate) — validates len ≤ 1000 (error if > 1000), preview_bulk_conflicts(ids, changes) → returns Vec<ConflictPreview> before applying, apply_bulk_update(); filter_knowledge_points(FilterParams{category_id, tags, difficulty_min, difficulty_max, discrimination_min, discrimination_max, chapter})
- [x] 3.3 Create backend/src/services/question_service.rs: create_question(), update_question(), delete_question(), link_to_kp(question_id, kp_id), unlink_from_kp(), filter_questions(FilterParams)
- [x] 3.4 Create backend/src/handlers/knowledge.rs — thin handlers delegating to services, all mutations call audit_service.log() with before/after hash
- [x] 3.5 Register knowledge routes in router.rs: CRUD guarded by ContentCurator + Administrator roles
- [x] 3.6 Create frontend/src/pages/knowledge/category_tree.rs — collapsible DAG tree, golden node icons, reference count badges on each node, "Merge" button opens modal showing cycle check result + {kp_count, subcategory_count, question_count} before confirming
- [x] 3.7 Create frontend/src/pages/knowledge/knowledge_points.rs — data table with filter sidebar (chapter select, tags multi-select chips, difficulty slider 1-5, discrimination band presets), bulk select with "max 1,000 records" indicator, bulk edit modal with conflict preview (shows fields that differ before applying)
- [x] 3.8 Create frontend/src/pages/knowledge/question_bank.rs — question list, filters, link/unlink modal
- [x] 3.9 Fill in backend/tests/unit_tests/knowledge_tests.rs:
       test_cycle_detection_direct_cycle(), test_cycle_detection_indirect_cycle(), test_cycle_detection_no_cycle(), test_bulk_update_exactly_1000_succeeds(), test_bulk_update_1001_returns_error(), test_merge_blocks_when_orphan_created(), test_reference_count_includes_all_types()
- [x] 3.10 Fill in backend/tests/api_tests/knowledge_api.rs:
       test_create_category_success(), test_create_category_curator_role_allowed(), test_create_category_reviewer_role_forbidden(), test_merge_cycle_returns_409(), test_bulk_edit_preview_returns_conflicts()
- [x] 3.11 Fill in frontend/tests/unit_tests/filter_tests.rs:
       test_filter_state_combines_difficulty_and_tags(), test_filter_state_clears_correctly(), test_discrimination_band_preset_sets_correct_range()

**Phase 3 checkpoint: QA logs in as curator → navigates to Knowledge section → category tree renders real data → create/merge operations work → knowledge points table loads and filters work → all pages functional in browser.**

---

## PHASE 4 — Outcome/IP Registration Module
> Goal: All 4 outcome types, evidence upload (magic-number check), share=100 enforced, duplicate detection
> Complete all tasks continuously, then pause. Wait for "proceed".

- [ ] 4.1 Create backend/src/services/outcome_service.rs: create_outcome(dto) → run duplicate detection using strsim::jaro_winkler on title (≥0.85), abstract_snippet first 200 chars (≥0.80), exact certificate_number match → return CreateOutcomeResult{outcome_id, duplicate_candidates: Vec<DuplicateCandidate{id,title,similarity_score}>}
- [ ] 4.2 Add: add_contributor(outcome_id, user_id, share_percentage, role) → validates total shares ≤ 100 after addition; submit_outcome(id) → validates SUM(share_percentage) == 100 exactly (else Err(Validation("Contribution shares must total exactly 100%"))); approve_outcome(id, approver_id); reject_outcome(id, reason); record_approval_cycle(entity_id, submitted_at, approved_at, approver_id) → computes cycle_time_minutes
- [ ] 4.3 Create backend/src/services/file_service.rs: upload_evidence(outcome_id, bytes, filename, declared_mime) → validate_file() exactly as CLAUDE.md (magic-number via infer + MIME mismatch + 25MB limit) → compute SHA-256 fingerprint → check uniqueness (SELECT by fingerprint → reject if exists) → encrypt bytes with AES-256-GCM → store to /app/evidence/{outcome_id}/ → save metadata → audit log
- [ ] 4.4 Create backend/src/handlers/outcomes.rs — POST /api/outcomes (returns duplicate_candidates), POST /api/outcomes/:id/contributors, DELETE /api/outcomes/:id/contributors/:id, POST /api/outcomes/:id/evidence (multipart), POST /api/outcomes/:id/submit, GET /api/outcomes/:id/compare/:other_id (side-by-side diff)
- [ ] 4.5 Register outcome routes: Reviewer + Administrator roles for write operations
- [ ] 4.6 Create frontend/src/pages/outcomes/register.rs — multi-step form: Step 1 (type select with descriptive cards for paper/patent/competition_result/software_copyright), Step 2 (title + abstract with real-time duplicate check → amber warning banner "X similar outcomes found" + "View Comparison" link if ≥0.85), Step 3 (evidence drag-drop upload, progress bar, MIME error inline), Step 4 (contributor allocation: name search, share % input, running total bar turns green at 100 / red if over or under)
- [ ] 4.7 Create frontend/src/pages/outcomes/compare.rs — side-by-side golden split-card layout, fields highlighted amber where different, similarity score displayed at top
- [ ] 4.8 Fill in backend/tests/unit_tests/outcome_tests.rs:
       test_share_validation_exactly_100_passes(), test_share_validation_99_fails(), test_share_validation_101_fails(), test_file_pdf_magic_number_accepted(), test_file_jpeg_magic_number_accepted(), test_file_wrong_magic_number_rejected(), test_file_exceeds_25mb_rejected(), test_file_mime_mismatch_rejected(), test_fingerprint_prevents_duplicate_upload(), test_duplicate_detection_above_title_threshold(), test_duplicate_detection_below_threshold()
- [ ] 4.9 Fill in backend/tests/api_tests/outcome_api.rs:
       test_full_outcome_registration_flow(), test_submit_without_100_percent_returns_400(), test_evidence_upload_duplicate_fingerprint_returns_409()
- [ ] 4.10 Fill in frontend/tests/unit_tests/validation_tests.rs:
       test_share_total_display_100_shows_green(), test_share_total_display_99_shows_red(), test_share_total_display_101_shows_red(), test_date_range_start_before_end_valid(), test_date_range_start_after_end_invalid()

**Phase 4 checkpoint: QA logs in as reviewer → navigates to Outcomes → multi-step registration form works end-to-end → evidence upload accepts valid files and rejects invalid → share percentage enforced at exactly 100% → duplicate warning appears correctly → all outcome pages work in browser.**

---

## PHASE 5 — Store & Promotions Module
> Goal: Promotions with time windows, mutual exclusion, best-offer engine, traceable line items
> Complete all tasks continuously, then pause. Wait for "proceed".

- [ ] 5.1 Create backend/src/services/store_service.rs: create_promotion(), update_promotion(), deactivate_promotion(); apply_best_promotion(cart_items, user_id) exactly as CLAUDE.md (time window filter → mutual exclusion resolution keeping highest priority per group → select greatest discount → return CheckoutResult with per-line LineItemResult{item, discount_amount, promotion_applied: Option<String>})
- [ ] 5.2 Add: create_order(user_id, cart_items) → calls apply_best_promotion → saves order + order_items with promotion_trace JSON → audit log; get_orders(), get_order(id)
- [ ] 5.3 Create backend/src/handlers/store.rs — product CRUD, promotion CRUD, POST /api/store/checkout, GET /api/store/orders
- [ ] 5.4 Register: StoreManager + Administrator for promotion/product management; authenticated for checkout
- [ ] 5.5 Create frontend/src/pages/store/promotions.rs — promotion table, create/edit form with datetime display in MM/DD/YYYY 12-hour format (stored as ISO 8601 internally, converted at UI layer), mutual exclusion group text input, priority number input, effective window preview badge
- [ ] 5.6 Create frontend/src/pages/store/checkout.rs — cart, "Apply Best Offer" button, discount breakdown per line item with promotion name + discount amount, order total summary
- [ ] 5.7 Fill in backend/tests/unit_tests/store_tests.rs:
       test_best_offer_selects_highest_priority(), test_mutual_exclusion_one_per_group(), test_two_exclusion_groups_each_gets_best(), test_expired_promotion_not_applied(), test_future_promotion_not_applied(), test_promotion_at_boundary_applied(), test_line_item_trace_contains_promotion_name()
- [ ] 5.8 Fill in backend/tests/api_tests/store_api.rs:
       test_full_checkout_flow(), test_checkout_no_eligible_promotions(), test_store_manager_creates_promotion(), test_reviewer_cannot_create_promotion()
- [ ] 5.9 Fill in frontend/tests/unit_tests/promotion_tests.rs:
       test_discount_display_percent_type(), test_discount_display_fixed_type(), test_promotion_time_format_mm_dd_yyyy(), test_total_savings_calculation()

**Phase 5 checkpoint: QA logs in as store manager → navigates to Store → creates a promotion → checkout applies best offer → discount shown per line item with promotion name → time window respected → all store pages functional in browser.**

---

## PHASE 6 — Analytics, Dashboards & Scheduled Reports
> Goal: All 4 dashboard metrics with real data, CSV/PDF export, scheduled reports with download tokens
> Complete all tasks continuously, then pause. Wait for "proceed".

- [ ] 6.1 Create backend/src/services/analytics_service.rs: get_member_metrics(from, to), get_churn_rate(from, to) → (churned / prior_total) as percentage, get_event_participation(filters), get_fund_summary(period) → {total_income, total_expense, net, budget_cap: 2500.00, over_budget: bool}, get_approval_cycle_stats(from, to) → {avg_minutes, median_minutes, slowest: Vec<...>}
- [ ] 6.2 Add: generate_csv(report_type, filters) → Vec<u8>; generate_pdf(report_type, filters) → Vec<u8> using printpdf crate; schedule_report(type, filters, user_id) → creates pending record + tokio::spawn background generation → sets status=complete + download_token UUID when done; get_report_download(report_id, token) → validates single-use token → streams file → clears token
- [ ] 6.3 Create backend/src/handlers/analytics.rs — GET /api/analytics/members, /events, /funds, /approval-cycles, POST /api/analytics/export/csv (FinanceManager+Admin), POST /api/analytics/export/pdf, POST /api/analytics/reports/schedule, GET /api/analytics/reports/:id/download/:token
- [ ] 6.4 Register: FinanceManager + Administrator for fund/export endpoints; all authenticated for general metrics
- [ ] 6.5 Create frontend/src/components/charts/ — Leptos components rendering chart data via JavaScript interop (canvas element + chart initialization via web_sys eval or JS shim): line_chart.rs (member growth over time), bar_chart.rs (income/expense stacked + budget cap line), histogram.rs (approval cycle time distribution)
- [ ] 6.6 Create frontend/src/pages/analytics/dashboard.rs — role-aware dashboard: 4 golden metric cards at top, date-range picker + filters, charts section, "Export CSV" + "Export PDF" buttons (only visible to FinanceManager/Administrator), "Schedule Report" button
- [ ] 6.7 Create frontend/src/pages/analytics/reports.rs — scheduled reports list with status badges (pending/processing/complete), download button when complete (calls download token endpoint), single-use token clears after download
- [ ] 6.8 Fill in backend/tests/unit_tests/analytics_tests.rs:
       test_churn_rate_calculation_formula(), test_fund_summary_over_budget_flag(), test_fund_summary_under_budget_no_flag(), test_csv_output_has_correct_headers(), test_approval_cycle_average_calculation()
- [ ] 6.9 Fill in backend/tests/api_tests/analytics_api.rs:
       test_fund_summary_finance_manager_allowed(), test_fund_summary_curator_forbidden(), test_export_csv_creates_download(), test_scheduled_report_creates_pending_record(), test_download_token_single_use()

**Phase 6 checkpoint: QA logs in as finance → navigates to Analytics → dashboard loads real data charts → fund summary shows budget cap → CSV export downloads a real file → scheduled report saves and shows download button when ready → all analytics pages functional in browser.**

---

## PHASE 7 — File Management & Backup/Restore System
> Goal: Encrypted evidence storage, backup scheduler, 30+12 retention, restore sandbox, lifecycle cleanup
> Complete all tasks continuously, then pause. Wait for "proceed".

- [ ] 7.1 Create backend/src/services/backup_service.rs: run_backup() → copy SQLite file → tar+gzip evidence dir → encrypt both bundles with AES-256-GCM → save to /app/backups/{date}/ → determine type (daily vs monthly: last day of month = monthly) → record in backup_records with sha256_hash; get_backup_history()
- [ ] 7.2 Add: restore_to_sandbox(backup_id) → decrypt + extract to /tmp/restore-sandbox/ → run PRAGMA integrity_check → verify SHA-256 hash → run SELECT COUNT(*) FROM users as basic read test → return SandboxValidationReport{integrity_ok, hash_ok, read_test_ok, all_passed}; activate_restore(backup_id) → replaces live DB from sandbox (requires admin confirmation + all_passed=true)
- [ ] 7.3 Add: apply_lifecycle_cleanup(retention_policy) → delete daily backups older than 30 days → delete monthly backups older than 12 months → SKIP records where entity references financial or IP data per retention_policy → delete files + mark status='purged' in backup_records
- [ ] 7.4 Create backend/src/services/backup_scheduler.rs — tokio-cron-scheduler job reading BACKUP_SCHEDULE env var (default "0 2 * * *"), calls run_backup() on schedule
- [ ] 7.5 Create backend/src/handlers/backup.rs — POST /api/backup/run (Admin only), GET /api/backup/history, POST /api/backup/:id/restore-sandbox, POST /api/backup/:id/activate, POST /api/backup/lifecycle-cleanup
- [ ] 7.6 Register backup routes with Administrator role only
- [ ] 7.7 Create frontend/src/pages/admin/backup.rs — backup history table (type/date/size/status badges), "Run Backup Now" button, restore button per row → opens sandbox validation report modal (shows integrity/hash/read test pass/fail) → "Activate Restore" button (only enabled if all_passed=true), lifecycle cleanup button with preview of what would be purged
- [ ] 7.8 Fill in backend/tests/unit_tests/backup_tests.rs:
       test_daily_backup_type_mid_month(), test_monthly_backup_type_last_day_of_month(), test_lifecycle_cleanup_removes_old_daily(), test_lifecycle_cleanup_removes_old_monthly(), test_lifecycle_cleanup_preserves_financial_records(), test_lifecycle_cleanup_preserves_ip_records(), test_restore_sandbox_sha256_verification()
- [ ] 7.9 Fill in backend/tests/api_tests/backup_api.rs:
       test_backup_run_creates_record(), test_backup_admin_only(), test_curator_cannot_access_backup(), test_restore_sandbox_returns_validation_report()

**Phase 7 checkpoint: QA logs in as admin → navigates to Backup → Run Backup Now creates a record in the history table → Restore button opens sandbox validation modal with real pass/fail results → all backup pages functional in browser.**

---

## PHASE 8 — Complete Frontend UI (All Pages, Golden Theme, Role Dashboards)
> Goal: Every page renders beautifully, every role has proper dashboard, app works end-to-end in Docker
> QA manually inspects browser — must look stunning.
> Complete all tasks continuously, then pause. Wait for "proceed".

- [ ] 8.1 Create frontend/src/components/layout/sidebar.rs — 240px dark sidebar, ScholarVault logo with golden gradient text, role-based nav items (gold active state + left border accent), role badge at bottom (gold bordered pill), logout button
- [ ] 8.2 Create frontend/src/components/layout/topbar.rs — sticky top bar, page title, breadcrumbs, user avatar + role badge, notification bell with count badge
- [ ] 8.3 Create frontend/src/components/ui/ complete set:
       button.rs: Primary(gold gradient, black text), Secondary(gold border), Danger(red), Ghost — all with loading spinner
       card.rs: dark bg-card, gold left-border accent, hover shadow-gold, glassmorphism option
       badge.rs: role badges (gold gradient), status badges (colored glow), count badges
       modal.rs: dark overlay, escape-to-close, focus trap
       table.rs: alternating dark rows, gold header underlines, sticky header, sort indicators, pagination
       form.rs: dark inputs, gold focus ring, floating labels, inline error messages
       skeleton.rs: gold shimmer animation (#F5C518 → transparent → #F5C518)
       empty_state.rs: centered icon + heading + body + action button
- [ ] 8.4 Create frontend/src/pages/dashboard.rs — role-aware: Administrator sees system KPIs (user count, pending approvals, backup status, unread alerts); ContentCurator sees knowledge stats; Reviewer sees pending outcomes; FinanceManager sees fund summary card; StoreManager sees active promotions + recent orders
- [ ] 8.5 Create frontend/src/pages/admin/users.rs — user management table, create user form (username, password with strength indicator, role select with role descriptions), deactivate/activate toggle, role change with confirmation modal
- [ ] 8.6 Create frontend/src/pages/admin/audit.rs — immutable audit log table (actor/action/entity/before-hash/after-hash/timestamp), filter by actor/action/date, no edit/delete UI anywhere on this page, badge showing "IMMUTABLE" on audit log header
- [ ] 8.7 Final pass — all Leptos components use #[component] macro correctly. All API calls use gloo_net. All reactive state uses create_signal/create_resource.
- [ ] 8.8 Final pass — verify golden theme consistent: CSS variables from CLAUDE.md used everywhere, no hardcoded hex colors in component styles
- [ ] 8.9 Final pass — verify every page: loading skeleton (skeleton.rs), empty state (empty_state.rs), error state with retry, role-gated buttons with disabled tooltip "X permission required"
- [ ] 8.10 Fill in frontend/tests/api_tests/auth_client_tests.rs:
        test_login_request_includes_csrf_header(), test_login_response_deserializes_correctly(), test_401_response_triggers_redirect_to_login()
- [ ] 8.11 Fill in frontend/tests/api_tests/knowledge_client_tests.rs:
        test_knowledge_point_dto_serializes_correctly(), test_filter_params_serialize_to_query_string()
- [ ] 8.12 Fill in frontend/tests/api_tests/outcome_client_tests.rs:
        test_create_outcome_request_serializes_correctly(), test_duplicate_candidate_response_deserializes()
- [ ] 8.13 Fill in frontend/tests/api_tests/store_client_tests.rs:
        test_checkout_request_serializes_cart_items(), test_checkout_response_deserializes_line_items()
- [ ] 8.14 Verify: cargo build --workspace succeeds — all Leptos components compile, all backend handlers compile

**Phase 8 checkpoint: cargo build --workspace succeeds. docker compose up --build → QA logs in with all 5 credentials in sequence → each role sees a different dashboard with real data → every page in the sidebar navigates without errors → golden theme visually consistent → no blank pages, no 500 errors, no placeholder content.**

---

## PHASE 9 — Test Suite Completion, Docker Verification & Static Audit Readiness
> Goal: All 4 test suites pass via Docker, no unwrap() in prod code, static audit clean
> Complete all tasks continuously, then pause. Wait for "proceed".

- [ ] 9.1 Audit: grep -r "unwrap()" backend/src/ frontend/src/ | grep -v "#\[cfg(test\|tests/" → must be zero results. Fix all with ? operator.
- [ ] 9.2 Audit: grep -r "println!\|dbg!" backend/src/ frontend/src/ | grep -v "tests/" → must be zero. Replace with tracing::info!/warn!/error!
- [ ] 9.3 Audit: grep -r "UPDATE audit_logs\|DELETE audit_logs" backend/src/ → must be zero (append-only enforced)
- [ ] 9.4 Audit: verify every POST/PUT/PATCH/DELETE handler file calls audit_service.log() — check each handler file manually
- [ ] 9.5 Audit: verify AuditService impl block has ONLY log() and compute_hash() — no update/delete methods
- [ ] 9.6 Write final missing tests to complete all 4 test suites:
       backend/tests/unit_tests/file_tests.rs: test_pdf_magic_bytes_accepted(), test_jpeg_magic_bytes_accepted(), test_png_magic_bytes_accepted(), test_exe_magic_bytes_rejected(), test_file_with_pdf_extension_but_exe_magic_rejected(), test_sha256_fingerprint_consistent()
       backend/tests/api_tests/analytics_api.rs: test_export_rate_limit_enforced()
       frontend/tests/unit_tests/filter_tests.rs: test_combined_difficulty_and_discrimination_filter()
- [ ] 9.7 Run: docker compose --profile test run --build test → shows 4 test suites, fix ALL failures until output shows "ALL TESTS PASSED" + exit code 0
- [ ] 9.8 Run: docker compose up --build → verify:
       app at http://localhost:3000 — login page renders with golden theme
       all 5 role logins redirect to correct dashboards
       no browser console errors
- [ ] 9.9 Final static audit readiness check:
       - Every security impl has doc comments with security intent (not one-liners)
       - Every service module has a //! module doc comment explaining its purpose
       - No TODO or unimplemented!() in non-test code: grep -r "TODO\|unimplemented!" backend/src/ frontend/src/ | grep -v "tests/" → zero
       - Migration files all syntactically valid SQL
       - README matches actual Docker commands and login credentials

**Phase 9 checkpoint: docker compose --profile test run test → "ALL TESTS PASSED" exit 0. docker compose up --build → golden login at :3000. 4 test suites all shown in output.**

---

## PHASE 10 — Documentation Generation
> Final phase — generate docs from actual code. No pause needed.

- [ ] 10.1 Create docs/design.md from actual implemented code:
       - ASCII architecture (Browser WASM ↔ Axum ↔ SQLite + evidence dir + backup dir)
       - Cargo workspace: 3 crates, their responsibilities and dependencies
       - Docker: 3-stage Dockerfile flow, single docker-compose services, .env.example auto-copy
       - Database: all 10 migration tables with key columns and constraints
       - Security architecture: Argon2 flow, AES-256-GCM field encryption, CSRF validation, rate limiting, account lockout (15-min window), audit append-only design
       - Knowledge DAG: DFS cycle detection algorithm, reference counting, bulk edit 1000-record limit
       - Outcome dedup pipeline: Jaro-Winkler similarity → candidate list → side-by-side compare
       - Promotion engine: time window filter → mutual exclusion resolution → best-discount selection → per-line trace
       - Analytics: member churn formula, fund budget cap logic, scheduled report download token lifecycle
       - Backup: daily/monthly classification, AES encryption, sandbox restore validation steps, retention lifecycle
       - Test architecture: why 4 test suites (backend unit, backend api, frontend unit, frontend api), what each covers

- [ ] 10.2 Create docs/api-spec.md from actual implemented code:
       - Every Axum route: method, path, role required, request shape (JSON fields or multipart), response shape, error codes
       - Auth endpoints: session cookie + csrf_token cookie behavior
       - File upload: multipart spec, validation rules (25MB, PDF/JPEG/PNG only, magic-number check)
       - Checkout endpoint: cart input + CheckoutResult with per-line promotion trace
       - Analytics export: CSV column format, PDF structure, scheduled report token lifecycle
       - Backup endpoints: restore sandbox flow, activate flow
       - Standard AppError JSON format: {code: String, message: String, timestamp: String}
       - CSRF: which endpoints require X-CSRF-Token header
       - Rate limiting: which endpoints are limited, 429 response format with Retry-After

---

## Execution Notes for Claude

- Complete ALL tasks in a phase without stopping between tasks
- Mark [x] immediately and continue — never pause mid-phase
- Fix Rust compile errors within the same task before marking [x]
- Only pause when entire phase checkpoint passes
- At each pause: brief summary (files created, checkpoint result)
- Wait for "proceed" before next phase
- Test rule: every phase that adds a service must also fill in the corresponding tests in BOTH unit_tests/ and api_tests/ for BOTH backend/ and frontend/ where applicable
- No unwrap() rule: enforce from Phase 0 onwards — catch in Phase 9 audit
- Golden UI: every Leptos component uses CSS variables from CLAUDE.md — no hardcoded colors
