# CLAUDE.md — ScholarVault Research & Commerce Operations Portal
# Task ID: TASK-32-W2
# Read SPEC.md + CLAUDE.md + PLAN.md before every single response. No exceptions.

## Read Order (mandatory, every response)
1. SPEC.md — source of truth
2. CLAUDE.md — this file
3. PLAN.md — current execution state

## Project Identity

- Name: ScholarVault Research & Commerce Operations Portal
- Task ID: TASK-32-W2
- Language: Rust (frontend + backend — 100% Rust)
- Frontend: Leptos 0.6.x (compiles to WASM via cargo-leptos) + TailwindCSS
- Backend: Axum 0.7.x
- Database: SQLite via SQLx 0.7.x (compile-time checked queries, migrations)
- Styling: TailwindCSS 3.x with golden-gradient dark SaaS theme
- Infrastructure: Docker + single docker-compose.yml

## QA Evaluation — TWO SIMULTANEOUS TESTS (both must pass — no exceptions)

> ⚠️ READ THIS BEFORE WRITING A SINGLE LINE OF CODE ⚠️
>
> TEST 1 — STATIC CODE AUDIT (GPT-5.3 reads every source file):
>   The auditor opens every .rs file and checks file:line evidence.
>   Security must be EXPLICITLY CODED — not just listed in Cargo.toml dependencies.
>   AuditService must have NO update/delete methods (auditor checks the impl block).
>   Business rules must be enforced at service layer (auditor traces through code).
>   Every feature from SPEC.md must be visible in code — not just mentioned in comments.
>
> TEST 2 — DOCKER RUNTIME MANUAL TESTING (human clicks through every page):
>   QA runs: docker compose up --build
>   QA opens: http://localhost:3000
>   QA logs in with ALL 5 credentials from README and clicks EVERY page for EVERY role.
>   Every form must submit to a real API. Every button must do something real.
>   Every page must load real data from SQLite. No stubs. No placeholders.
>   Any blank page, 500 error, broken form, or non-functional feature = FAIL.
>
> CONSEQUENCE: If code is correct but app is broken → FAIL.
>              If app runs but security is not explicitly coded → FAIL.
>              Both tests must pass simultaneously.

## Cargo Workspace Structure (strict — all code inside repo/)

```
TASK-32-W2/
├── SPEC.md
├── CLAUDE.md
├── PLAN.md
├── docs/
├── sessions/
├── metadata.json
└── repo/
    ├── Cargo.toml
    ├── Cargo.lock
    ├── .env.example
    ├── .gitignore
    ├── README.md
    ├── docker-compose.yml
    ├── Dockerfile
    ├── Dockerfile.test
    ├── run_tests.sh
    ├── tailwind.config.js
    ├── style/
    │   └── main.scss
    ├── backend/
    │   ├── Cargo.toml
    │   ├── src/
    │   │   ├── main.rs
    │   │   ├── router.rs
    │   │   ├── error.rs
    │   │   ├── middleware/
    │   │   │   ├── csrf.rs
    │   │   │   ├── rate_limit.rs
    │   │   │   ├── session.rs
    │   │   │   ├── require_role.rs
    │   │   │   └── security_headers.rs
    │   │   ├── handlers/
    │   │   │   ├── auth.rs
    │   │   │   ├── knowledge.rs
    │   │   │   ├── outcomes.rs
    │   │   │   ├── store.rs
    │   │   │   ├── analytics.rs
    │   │   │   ├── files.rs
    │   │   │   └── backup.rs
    │   │   ├── services/
    │   │   │   ├── auth_service.rs
    │   │   │   ├── knowledge_service.rs
    │   │   │   ├── outcome_service.rs
    │   │   │   ├── store_service.rs
    │   │   │   ├── analytics_service.rs
    │   │   │   ├── file_service.rs
    │   │   │   ├── backup_service.rs
    │   │   │   └── audit_service.rs
    │   │   ├── db/
    │   │   │   ├── mod.rs
    │   │   │   └── migrations/
    │   │   ├── models/
    │   │   └── security/
    │   │       ├── password.rs
    │   │       ├── encryption.rs
    │   │       └── csrf.rs
    │   └── tests/
    │       ├── unit_tests/            ← service-layer unit tests
    │       │   ├── mod.rs
    │       │   ├── auth_tests.rs
    │       │   ├── knowledge_tests.rs
    │       │   ├── outcome_tests.rs
    │       │   ├── store_tests.rs
    │       │   ├── analytics_tests.rs
    │       │   ├── file_tests.rs
    │       │   └── backup_tests.rs
    │       └── api_tests/             ← HTTP endpoint integration tests
    │           ├── mod.rs
    │           ├── auth_api.rs
    │           ├── knowledge_api.rs
    │           ├── outcome_api.rs
    │           ├── store_api.rs
    │           ├── analytics_api.rs
    │           └── backup_api.rs
    ├── frontend/
    │   ├── Cargo.toml
    │   ├── src/
    │   │   ├── main.rs
    │   │   ├── app.rs
    │   │   ├── components/
    │   │   │   ├── layout/
    │   │   │   ├── ui/
    │   │   │   └── charts/
    │   │   └── pages/
    │   │       ├── login.rs
    │   │       ├── dashboard.rs
    │   │       ├── knowledge/
    │   │       ├── outcomes/
    │   │       ├── store/
    │   │       ├── analytics/
    │   │       ├── audit/
    │   │       └── admin/
    │   └── tests/
    │       ├── unit_tests/            ← component logic + pure function tests
    │       │   ├── mod.rs
    │       │   ├── validation_tests.rs  ← form validation logic (share=100, date range, etc.)
    │       │   ├── promotion_tests.rs   ← discount calculation display logic
    │       │   ├── mask_tests.rs        ← sensitive field masking display logic
    │       │   └── filter_tests.rs      ← combined filter state logic
    │       └── api_tests/             ← frontend API client integration tests
    │           ├── mod.rs
    │           ├── auth_client_tests.rs  ← login/logout API client calls
    │           ├── knowledge_client_tests.rs
    │           ├── outcome_client_tests.rs
    │           └── store_client_tests.rs
    └── shared/
        ├── Cargo.toml
        └── src/
            └── lib.rs
```

## Test Architecture — Frontend + Backend Separation

```
BACKEND TESTS (backend/tests/)
├── unit_tests/     ← test individual service functions in isolation (no HTTP)
│   Purpose: verify business logic correctness — cycle detection, share validation,
│            file validation, promotion engine, backup retention, audit append-only
└── api_tests/      ← test HTTP endpoints with real SQLite test DB (Axum test client)
    Purpose: verify routes, middleware, auth, CSRF, rate limiting work end-to-end

FRONTEND TESTS (frontend/tests/)
├── unit_tests/     ← test pure Rust logic that lives in frontend (no DOM/WASM needed)
│   Purpose: verify form validation functions, display masking, filter state logic,
│            discount display calculation — all pure functions testable without browser
└── api_tests/      ← test frontend API client functions against a real backend
    Purpose: verify the gloo-net HTTP client calls serialize/deserialize correctly,
             that CSRF headers are attached, that error responses are handled properly
    Note: these tests run in a native Rust test environment (not WASM) using
          a mock or real backend endpoint, since WASM tests require wasm-pack
          which adds significant complexity. Pure function tests cover the WASM logic.
```

## Non-Negotiable Rules

1. **Read SPEC.md + CLAUDE.md + PLAN.md first.** Every response, no exceptions.
2. **One task at a time.** Complete exactly the current PLAN.md task.
3. **Mark [x] then continue.** Update PLAN.md and move immediately to next task.
4. **All code in repo/.** Never create files outside repo/.
5. **Static audit clarity.** Every security feature must have explicit, readable Rust code with doc comments. QA reads file:line references. A dependency in Cargo.toml is NOT proof of implementation — the actual code must exist.
6. **Every page must fully work in Docker.** QA logs in with all 5 credentials and clicks every single page. Every form submits to a real API. Every table shows real data from SQLite. Every button does something real. No stubs, no hardcoded mock data, no placeholder pages.
7. **Gorgeous UI is mandatory.** Golden-gradient dark SaaS theme throughout — QA inspects the browser UI as part of the evaluation.
8. **Every API route must be wired end-to-end.** Handler registered in router.rs → calls real service → service calls real SQLite → returns real response → Leptos page renders it. Every step of this chain must work.
8. **No unwrap() in non-test code.** Use ? operator and proper error propagation.
9. **SQLx compile-time queries.** Use sqlx::query! and sqlx::query_as! macros.
10. **Migrations only.** All schema via migration files — never ddl-auto in app code.
11. **Pause at phase boundaries only.** Only pause when entire phase checkpoint passes.
12. **Fix before proceeding.** Cargo compile errors fixed within same task.
13. **No hardcoded data in Leptos components.** All data from API calls.
14. **Audit log is append-only.** AuditService has ONLY a log() method — no update/delete. Enforced at type level.
15. **Test separation is strict.** unit_tests/ = isolated logic tests. api_tests/ = integration tests. Both frontend AND backend have both folders.
16. **Frontend tests are native Rust.** Frontend unit_tests test pure functions (no WASM runtime needed). Frontend api_tests test the HTTP client serialization logic.

## Tech Stack Specifics

### Rust Workspace Cargo.toml (repo/Cargo.toml)
```toml
[workspace]
members = ["backend", "frontend", "shared"]
resolver = "2"

[workspace.dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tokio = { version = "1", features = ["full"] }
anyhow = "1"
thiserror = "1"
chrono = { version = "0.4", features = ["serde"] }
uuid = { version = "1", features = ["v4", "serde"] }
tracing = "0.1"

# Backend
axum = { version = "0.7", features = ["multipart", "macros"] }
axum-extra = { version = "0.9", features = ["cookie", "typed-header"] }
sqlx = { version = "0.7", features = ["runtime-tokio-rustls", "sqlite", "migrate", "chrono", "uuid"] }
tower = "0.4"
tower-http = { version = "0.5", features = ["cors", "fs", "trace", "set-header", "compression-gzip"] }
argon2 = "0.5"
aes-gcm = "0.10"
rand = "0.8"
governor = "0.6"
sha2 = "0.10"
hex = "0.4"
infer = "0.15"
strsim = "0.11"
csv = "1"
printpdf = "0.7"
tokio-cron-scheduler = "0.9"
dashmap = "5"

# Frontend
leptos = { version = "0.6", features = ["csr"] }
leptos_router = "0.6"
leptos_meta = "0.6"
wasm-bindgen = "0.2"
web-sys = { version = "0.3", features = ["Window", "Document", "HtmlElement"] }
gloo-net = "0.5"
```

### Security Architecture (all explicitly coded with doc comments for static audit)

#### Argon2 Password Hashing (security/password.rs)
```rust
/// Hash a password using Argon2id with a randomly generated salt.
/// The output string embeds the salt and algorithm params for self-contained verification.
pub fn hash_password(password: &str) -> Result<String, AppError> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    Ok(argon2.hash_password(password.as_bytes(), &salt)
        .map_err(|e| AppError::Internal(e.to_string()))?
        .to_string())
}

/// Verify a plain password against its Argon2id hash.
pub fn verify_password(password: &str, hash: &str) -> Result<bool, AppError> {
    let parsed_hash = PasswordHash::new(hash)
        .map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}
```

#### AES-256-GCM Field Encryption (security/encryption.rs)
```rust
/// Encrypt a sensitive field with AES-256-GCM.
/// Each call generates a unique random nonce — same plaintext produces different ciphertext.
/// Output format: base64(nonce[12 bytes] + ciphertext)
pub fn encrypt_field(plaintext: &str, key: &[u8; 32]) -> Result<String, AppError> {
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
    let ciphertext = cipher.encrypt(&nonce, plaintext.as_bytes())
        .map_err(|_| AppError::Internal("Encryption failed".to_string()))?;
    Ok(base64::encode([nonce.as_slice(), &ciphertext].concat()))
}

/// Decrypt a field value previously encrypted with encrypt_field.
pub fn decrypt_field(encoded: &str, key: &[u8; 32]) -> Result<String, AppError> {
    let data = base64::decode(encoded)
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let (nonce_bytes, ciphertext) = data.split_at(12);
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
    let plaintext = cipher.decrypt(Nonce::from_slice(nonce_bytes), ciphertext)
        .map_err(|_| AppError::Internal("Decryption failed".to_string()))?;
    Ok(String::from_utf8(plaintext)
        .map_err(|e| AppError::Internal(e.to_string()))?)
}

/// Mask a sensitive value for UI display — show only last 4 characters.
/// Example: "9876543210" → "******3210"
pub fn mask_sensitive(value: &str) -> String {
    if value.len() <= 4 { return "*".repeat(value.len()); }
    format!("{}{}", "*".repeat(value.len() - 4), &value[value.len()-4..])
}
```

#### CSRF Protection (middleware/csrf.rs)
```rust
/// CSRF middleware — validates X-CSRF-Token header matches csrf_token session cookie.
/// Only applies to state-changing methods: POST, PUT, PATCH, DELETE.
/// Uses constant-time comparison to prevent timing-based attacks.
pub async fn csrf_middleware(
    cookies: Cookies, req: Request, next: Next,
) -> Result<Response, AppError> {
    if matches!(req.method(), &Method::GET | &Method::HEAD | &Method::OPTIONS) {
        return Ok(next.run(req).await);
    }
    let header_token = req.headers()
        .get("X-CSRF-Token")
        .and_then(|v| v.to_str().ok())
        .ok_or(AppError::CsrfMissing)?;
    let cookie_token = cookies.get("csrf_token")
        .map(|c| c.value().to_string())
        .ok_or(AppError::CsrfMissing)?;
    if !constant_time_eq(header_token.as_bytes(), cookie_token.as_bytes()) {
        return Err(AppError::CsrfInvalid);
    }
    Ok(next.run(req).await)
}
```

#### Rate Limiting (middleware/rate_limit.rs)
```rust
/// Rate limiting middleware — 60 requests/minute per authenticated user (keyed by user_id).
/// Returns HTTP 429 with Retry-After header when limit is exceeded.
/// Uses governor crate with in-memory keyed state.
pub struct RateLimitLayer {
    limiter: Arc<RateLimiter<String, DefaultKeyedStateStore<String>, DefaultClock>>,
}
impl RateLimitLayer {
    pub fn new() -> Self {
        Self { limiter: Arc::new(RateLimiter::keyed(
            Quota::per_minute(NonZeroU32::new(60).unwrap())
        ))}
    }
}
```

#### Account Lockout (services/auth_service.rs)
```rust
/// Check and enforce account lockout: 5 failed attempts within 15-minute sliding window.
/// The lockout expires automatically when the 15-minute window passes.
/// Checks both username-based AND IP-based attempt counts.
pub async fn check_lockout(&self, username: &str, ip: &str) -> Result<(), AppError> {
    let window = Utc::now() - Duration::minutes(15);
    let attempts = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM login_attempts
         WHERE (username = ? OR ip_address = ?)
         AND attempted_at > ? AND success = 0",
        username, ip, window
    ).fetch_one(&self.db).await?;
    if attempts >= 5 {
        return Err(AppError::AccountLocked {
            message: "Too many failed attempts. Try again in 15 minutes.".to_string()
        });
    }
    Ok(())
}
```

#### Immutable Audit Log (services/audit_service.rs)
```rust
/// Append-only audit log service.
/// This impl block intentionally has NO update() and NO delete() methods.
/// Every mutation in the system must call this service before returning.
pub struct AuditService { db: SqlitePool }

impl AuditService {
    /// Append one immutable audit record. This is the sole method on this service.
    pub async fn log(&self, actor_id: &str, action: AuditAction,
        entity_type: &str, entity_id: Option<&str>,
        before_hash: Option<String>, after_hash: Option<String>,
        ip_address: Option<&str>) -> Result<(), AppError> {
        let id = Uuid::new_v4().to_string();
        sqlx::query!(
            "INSERT INTO audit_logs
             (id, actor_id, action, entity_type, entity_id,
              before_hash, after_hash, ip_address, created_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
            id, actor_id, action.to_string(), entity_type, entity_id,
            before_hash, after_hash, ip_address, Utc::now().to_rfc3339()
        ).execute(&self.db).await?;
        Ok(())
    }
    pub fn compute_hash(data: &str) -> String {
        hex::encode(Sha256::digest(data.as_bytes()))
    }
    // NO update() — NO delete() — APPEND ONLY by design and type system enforcement
}
```

#### File Validation (services/file_service.rs)
```rust
/// Validate uploaded file offline using magic-number inspection + MIME check.
/// No external services — uses the infer crate to read file header bytes.
pub fn validate_file(bytes: &[u8], declared_mime: &str) -> Result<(), AppError> {
    if bytes.len() > 25 * 1024 * 1024 {
        return Err(AppError::FileTooLarge { size: bytes.len(), max: 25 * 1024 * 1024 });
    }
    let detected = infer::get(bytes).ok_or(AppError::UnknownFileType)?;
    if !["application/pdf", "image/jpeg", "image/png"].contains(&detected.mime_type()) {
        return Err(AppError::InvalidFileType(detected.mime_type().to_string()));
    }
    if detected.mime_type() != declared_mime {
        return Err(AppError::MimeMismatch {
            declared: declared_mime.to_string(),
            detected: detected.mime_type().to_string(),
        });
    }
    Ok(())
}
```

#### Knowledge Graph Cycle Detection (services/knowledge_service.rs)
```rust
/// DFS-based cycle detection for the category DAG.
/// Returns true if adding parent_id → child_id edge would create a cycle.
pub async fn check_would_create_cycle(
    &self, parent_id: &str, child_id: &str,
) -> Result<bool, AppError> {
    let mut visited = HashSet::new();
    let mut stack = vec![child_id.to_string()];
    while let Some(node) = stack.pop() {
        if node == parent_id { return Ok(true); } // would create cycle
        if !visited.insert(node.clone()) { continue; }
        stack.extend(self.get_direct_children(&node).await?);
    }
    Ok(false)
}
```

#### Promotion Discount Engine (services/store_service.rs)
```rust
/// Select and apply the best eligible promotion at checkout.
/// Step 1: Filter by effective time window.
/// Step 2: Within each mutual_exclusion_group, keep only highest-priority promotion.
/// Step 3: Apply the promotion producing the greatest total discount.
/// Returns per-line-item traceable discount details.
pub fn apply_best_promotion(
    cart_items: &[CartItem], promotions: &[Promotion],
) -> CheckoutResult {
    let now = Utc::now();
    let eligible: Vec<&Promotion> = promotions.iter()
        .filter(|p| p.is_active && p.effective_from <= now && p.effective_until >= now)
        .collect();
    let resolved = resolve_exclusion_groups(&eligible);
    let best = resolved.iter()
        .max_by_key(|p| ordered_float(calculate_total_discount(cart_items, p)));
    let line_items = cart_items.iter().map(|item| {
        let (discount, trace) = best
            .map(|p| (calculate_item_discount(item, p), Some(p.name.clone())))
            .unwrap_or((0.0, None));
        LineItemResult { item: item.clone(), discount_amount: discount, promotion_applied: trace }
    }).collect();
    CheckoutResult { line_items, best_promotion: best.map(|p| (*p).clone()) }
}
```

## UI Design Standards (Golden-Gradient Premium SaaS)

```css
:root {
  --bg-primary:   #0A0A0F;
  --bg-secondary: #12121A;
  --bg-card:      #1A1A28;
  --bg-hover:     #1F1F32;
  --gold-400:     #F5C518;
  --gold-500:     #E8A900;
  --gold-600:     #CC8800;
  --gradient-gold: linear-gradient(135deg, #F5C518 0%, #E8A900 50%, #CC8800 100%);
  --shadow-gold:  0 0 20px rgba(245,197,24,0.15);
  --text-primary:   #F0F0F5;
  --text-secondary: #A0A0B0;
  --text-gold:      #F5C518;
}
```

Every Leptos component follows: dark cards with gold left-border accent, gold gradient primary buttons, gold focus rings on inputs, gold shimmer skeleton loaders, gold active sidebar items. Every page has loading/empty/error states and role-gated buttons with "why blocked" tooltip.

## Docker Architecture (single docker-compose.yml)

```yaml
services:
  setup:
    image: alpine:3.18
    volumes: [".:/workspace"]
    command: >
      sh -c "[ ! -f /workspace/.env ] &&
             cp /workspace/.env.example /workspace/.env &&
             echo 'First run: .env created' || echo '.env exists'"
    restart: "no"

  app:
    build: { context: ., dockerfile: Dockerfile }
    ports: ["3000:3000"]
    env_file: [{ path: .env, required: false }]
    environment:
      DATABASE_URL: ${DATABASE_URL:-sqlite:///app/data/scholarvault.db}
      ENCRYPTION_KEY: ${ENCRYPTION_KEY:-scholarvault-aes256-key-32bytes!!}
      SIGNING_KEY: ${SIGNING_KEY:-scholarvault-jwt-signing-key-secret!}
      RUST_LOG: ${RUST_LOG:-info}
      HOST: 0.0.0.0
      PORT: 3000
    volumes: [app-data:/app/data, evidence-files:/app/evidence, backups:/app/backups]
    depends_on: { setup: { condition: service_completed_successfully } }

  test:
    profiles: [test]
    build: { context: ., dockerfile: Dockerfile.test }
    env_file: [{ path: .env, required: false }]
    environment:
      DATABASE_URL: sqlite:///tmp/scholarvault_test.db
      ENCRYPTION_KEY: test-encryption-key-exactly-32bytes
      SIGNING_KEY: test-signing-key-for-tests-only!!!!
      RUST_LOG: error
    command: ["sh", "run_tests.sh"]

volumes: { app-data: {}, evidence-files: {}, backups: {} }
```

## run_tests.sh (Docker-first, also runnable locally if Rust installed)

```bash
#!/bin/sh
set -e

echo "========================================"
echo "  ScholarVault Test Suite"
echo "========================================"

if command -v cargo > /dev/null 2>&1; then
  CARGO="cargo"
else
  echo "ERROR: cargo not found."
  echo "Run via Docker: docker compose --profile test run --build test"
  echo "Or install Rust toolchain: https://rustup.rs"
  exit 1
fi

BACKEND_UNIT_FAILED=0
BACKEND_API_FAILED=0
FRONTEND_UNIT_FAILED=0
FRONTEND_API_FAILED=0

echo ""
echo "--- Backend Unit Tests (backend/tests/unit_tests/) ---"
$CARGO test --package backend --test unit_tests \
  -- --test-threads=1 2>&1 || BACKEND_UNIT_FAILED=1
[ $BACKEND_UNIT_FAILED -eq 0 ] && echo "✅ Backend Unit Tests PASSED" \
                                || echo "❌ Backend Unit Tests FAILED"

echo ""
echo "--- Backend API Tests (backend/tests/api_tests/) ---"
$CARGO test --package backend --test api_tests \
  -- --test-threads=1 2>&1 || BACKEND_API_FAILED=1
[ $BACKEND_API_FAILED -eq 0 ] && echo "✅ Backend API Tests PASSED" \
                               || echo "❌ Backend API Tests FAILED"

echo ""
echo "--- Frontend Unit Tests (frontend/tests/unit_tests/) ---"
$CARGO test --package frontend --test unit_tests \
  -- --test-threads=1 2>&1 || FRONTEND_UNIT_FAILED=1
[ $FRONTEND_UNIT_FAILED -eq 0 ] && echo "✅ Frontend Unit Tests PASSED" \
                                 || echo "❌ Frontend Unit Tests FAILED"

echo ""
echo "--- Frontend API Client Tests (frontend/tests/api_tests/) ---"
$CARGO test --package frontend --test api_tests \
  -- --test-threads=1 2>&1 || FRONTEND_API_FAILED=1
[ $FRONTEND_API_FAILED -eq 0 ] && echo "✅ Frontend API Tests PASSED" \
                                || echo "❌ Frontend API Tests FAILED"

echo ""
echo "========================================"
TOTAL_FAILED=$((BACKEND_UNIT_FAILED + BACKEND_API_FAILED + FRONTEND_UNIT_FAILED + FRONTEND_API_FAILED))
if [ $TOTAL_FAILED -eq 0 ]; then
  echo "  ALL TESTS PASSED"
  exit 0
else
  echo "  SOME TESTS FAILED"
  echo "  Backend Unit:     $([ $BACKEND_UNIT_FAILED -eq 0 ] && echo PASS || echo FAIL)"
  echo "  Backend API:      $([ $BACKEND_API_FAILED -eq 0 ] && echo PASS || echo FAIL)"
  echo "  Frontend Unit:    $([ $FRONTEND_UNIT_FAILED -eq 0 ] && echo PASS || echo FAIL)"
  echo "  Frontend API:     $([ $FRONTEND_API_FAILED -eq 0 ] && echo PASS || echo FAIL)"
  exit 1
fi
```

## .env.example (committed to git)
```
DATABASE_URL=sqlite:///app/data/scholarvault.db
ENCRYPTION_KEY=scholarvault-aes256-key-32bytes!!
SIGNING_KEY=scholarvault-jwt-signing-key-secret!
RUST_LOG=info
HOST=0.0.0.0
PORT=3000
BACKUP_SCHEDULE=0 2 * * *
BACKUP_RETAIN_DAILY=30
BACKUP_RETAIN_MONTHLY=12
```

## .gitignore
```
/target
.env
*.db
*.db-wal
*.db-shm
node_modules/
dist/
.DS_Store
```

## README (minimal)
```markdown
# ScholarVault Research & Commerce Operations Portal

## Run
```bash
docker compose up --build
```
Open http://localhost:3000  (.env auto-created from .env.example on first run)

## Test
```bash
docker compose --profile test run --build test
```

## Stop
```bash
docker compose down
```

## Login
| Role | Username | Password |
|---|---|---|
| Administrator | admin | ScholarAdmin2024! |
| Content Curator | curator | Scholar2024! |
| Reviewer | reviewer | Scholar2024! |
| Finance Manager | finance | Scholar2024! |
| Store Manager | store | Scholar2024! |
```

## Open Questions & Clarifications (from business prompt only)

[ ] Category tree is a DAG — cycle detection (DFS) required on every node merge/migration
[ ] Contribution shares: integer whole-percentages, sum must equal exactly 100
[ ] Duplicate detection: Jaro-Winkler ≥ 0.85 (title), ≥ 0.80 (abstract snippet), exact (certificate number)
[ ] Best offer: highest-priority after mutual exclusion resolution, then largest absolute discount value
[ ] Account lockout: 5 failures in 15-min window → auto-expires when window passes
[ ] Backup monthly: last daily backup of each calendar month becomes the monthly archive
[ ] Restore sandbox: validates SHA-256 integrity + PRAGMA integrity_check + basic read before activation
[ ] Financial records: fund_transactions, orders, export_logs — preserved per retention policy
[ ] IP records: outcomes, evidence_files — preserved per retention policy
[ ] Approval cycle: submitted_at → approved_at or rejected_at timestamp diff
[ ] Discrimination bands: Poor <0.1, Acceptable 0.1–0.3, Good 0.3–0.5, Excellent >0.5
[ ] Anti-abuse backoff: in-memory DashMap, 3+ invalid searches → exponential delay 2^n seconds
