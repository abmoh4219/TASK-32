# fullstack

ScholarVault Research & Commerce Operations Portal — a full-stack Rust application
(Leptos/WASM frontend + Axum backend + SQLite) that manages academic knowledge,
research outcomes, a digital store, analytics, and administrative operations with
a golden-gradient dark SaaS UI.

## Architecture & Tech Stack

| Layer | Technology |
|-------|-----------|
| Frontend | Rust + [Leptos 0.6](https://leptos.dev) compiled to WebAssembly via Trunk |
| Backend | Rust + [Axum 0.7](https://github.com/tokio-rs/axum) HTTP framework |
| Database | [SQLite](https://sqlite.org) via SQLx 0.7 with compile-time checked queries and migration files |
| Styling | TailwindCSS 3.x with a golden-gradient dark SaaS theme |
| Security | Argon2id password hashing, AES-256-GCM field encryption, CSRF protection, rate limiting, account lockout |
| Infrastructure | Docker multi-stage build (WASM + Axum binary → slim runtime) |

## Project Structure

```
repo/
├── Cargo.toml              # Rust workspace (backend + frontend + shared)
├── Cargo.lock
├── Dockerfile              # Multi-stage: wasm-builder → backend-builder → runtime
├── Dockerfile.test         # Test runner image (cargo test suites)
├── docker-compose.yml      # App + test + Playwright services
├── run_tests.sh            # Docker-only test orchestrator (4 suites)
├── README.md
├── index.html              # Trunk HTML entry point
├── style/main.scss         # TailwindCSS + custom golden-gradient theme
├── tailwind.config.js
├── backend/
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs         # Server startup, TLS, env validation
│       ├── router.rs       # All Axum routes (40+ endpoints)
│       ├── error.rs        # AppError enum, HTTP status mapping
│       ├── handlers/       # auth, knowledge, outcomes, store, analytics, backup, files
│       ├── services/       # auth, knowledge, outcome, store, analytics, audit, file, backup
│       ├── models/         # user, knowledge, outcome, store, analytics, backup, audit
│       ├── middleware/     # csrf, rate_limit, session, require_role, security_headers
│       ├── security/       # password (Argon2id), encryption (AES-256-GCM), csrf
│       └── db/
│           ├── mod.rs
│           └── migrations/ # 0001–0012 SQL migration + seed files
├── frontend/
│   ├── Cargo.toml
│   └── src/
│       ├── app.rs          # Leptos root component + routing
│       ├── pages/          # login, dashboard, knowledge, outcomes, store, analytics, admin
│       ├── components/     # layout, ui primitives, charts
│       ├── api/            # gloo-net HTTP client modules per domain
│       └── logic/          # validation, mask, filter, promotion (pure Rust, unit-testable)
├── shared/
│   └── src/lib.rs          # Types shared between backend and frontend
├── backend/tests/
│   ├── unit_tests/         # Service-layer isolation tests (no HTTP)
│   └── api_tests/          # HTTP integration tests (Axum test client + real SQLite)
└── frontend/tests/
    ├── unit_tests/         # Pure-function logic tests (no WASM required)
    └── e2e/                # Playwright browser tests
```

## Prerequisites

- [Docker](https://docs.docker.com/get-docker/) and Docker Compose v2

No Rust toolchain, Node.js, wasm-pack, or any other tool required on the host.
Everything compiles and runs inside Docker containers.

## Running the Application

```bash
docker compose up --build
```

```bash
docker-compose up --build
```

Open <http://localhost:3000>.

On first run the `setup` service auto-generates a `.env` with random 48-byte secrets
from `.env.example`. Subsequent runs reuse the existing `.env`.

To stop:

```bash
docker compose down
```

## Testing

Run all four test suites (backend unit + backend API + frontend unit + Playwright E2E):

```bash
chmod +x run_tests.sh
./run_tests.sh
```

The script requires only Docker — no Rust, cargo, Node, or npm needed on the host.

### What each suite tests

| # | Suite | Technology | What it covers |
|---|-------|------------|----------------|
| 1 | Backend Unit | `cargo test` | Service-layer logic in isolation: Argon2 hashing, AES encryption, cycle detection, bulk-edit limits, discount engine, backup retention |
| 2 | Backend API | `cargo test` | Every HTTP endpoint with real SQLite: auth, CSRF, rate limiting, RBAC, PII encryption, audit hashing |
| 3 | Frontend Unit | `cargo test` | Pure Rust logic: form validation, field masking, filter state, promotion display formatting |
| 4 | E2E | Playwright | Browser flows: login for all roles, navigation, API role-gating, checkout trust boundary |

## Seeded Credentials

| Role | Username | Password | What this role can do |
|------|----------|----------|-----------------------|
| Administrator | `admin` | `ScholarAdmin2024!` | Full access: user management, audit log (read-only), backups, retention policy, schedule |
| Content Curator | `curator` | `Scholar2024!` | Manage knowledge categories (DAG), knowledge points, question bank, bulk edit |
| Reviewer | `reviewer` | `Scholar2024!` | Register research outcomes, upload evidence files, manage contributors, submit for approval |
| Finance Manager | `finance` | `Scholar2024!` | View analytics dashboard, fund summary, approval cycles, export CSV/PDF reports |
| Store Manager | `store` | `Scholar2024!` | Manage products, create promotions, run checkout, view order history |

## Verification

After `docker compose up --build`, open <http://localhost:3000> and verify each role:

### Administrator (`admin` / `ScholarAdmin2024!`)
1. Log in — you are redirected to `/admin`
2. **Backup & Restore** tab — click **Run Backup** → a new row appears in the history table
3. **Users** tab — click **Create User** → new user appears in the list with masked phone/national_id
4. **Audit Log** tab — every mutation is listed with before/after hashes; no edit/delete buttons present

### Content Curator (`curator` / `Scholar2024!`)
1. Log in — redirected to `/knowledge`
2. **Category Tree** tab — create a new category; merge two categories (cycle attempt is blocked with 409)
3. **Knowledge Points** tab — create a point; use bulk-edit preview (≤1000 IDs)
4. **Question Bank** tab — create a question; link it to a knowledge point

### Reviewer (`reviewer` / `Scholar2024!`)
1. Log in — redirected to `/outcomes`
2. **Register New** tab — fill in title, type, contributors (shares must sum to 100), upload a PDF
3. **My Outcomes** tab — submit the draft; status changes to `submitted`
4. **Compare** tab — select two outcomes for side-by-side diff

### Finance Manager (`finance` / `Scholar2024!`)
1. Log in — redirected to `/analytics`
2. **Dashboard** tab — member metrics, churn rate, fund summary (with $2,500 cap flag), approval cycles
3. Click **Export CSV** → file downloads; click **Export PDF** → file downloads
4. **Scheduled Reports** tab — schedule a report; it appears with `complete` status

### Store Manager (`store` / `Scholar2024!`)
1. Log in — redirected to `/store`
2. **Promotions** tab — create a percentage promotion with a time window
3. **Checkout** tab — add a product, click **Preview** → best-offer promotion applied; click **Checkout**
4. **Orders** tab — completed order appears with server-resolved price (client-side tampering ignored)

### API smoke tests

```bash
# Health check
curl -s http://localhost:3000/api/healthz

# Login
curl -s -X POST http://localhost:3000/api/auth/login \
  -H 'Content-Type: application/json' \
  -d '{"username":"admin","password":"ScholarAdmin2024!"}'
```
