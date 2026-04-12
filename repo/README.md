# ScholarVault Research & Commerce Operations Portal

## Quick start (Docker — recommended)

```bash
docker compose up --build
```

Open <http://localhost:3000>. On first run the `setup` service auto-generates
a `.env` with random secrets from `.env.example`.

## Run tests

```bash
docker compose --profile test run --build test
```

## Run without Docker (requires Rust toolchain)

```bash
# 1. Install Rust: https://rustup.rs
# 2. Install trunk for WASM frontend: cargo install trunk
# 3. Copy config:
cp .env.example .env
# 4. Edit .env — set real ENCRYPTION_KEY / SIGNING_KEY (≥32 bytes each)
# 5. Build frontend:
trunk build --release --filehash false
# 6. Run backend:
cd backend && cargo run --release
# 7. Open http://localhost:3000
```

## Stop

```bash
docker compose down
```

## Configuration matrix

| Variable | Required | Default (Docker) | Description |
|----------|----------|-------------------|-------------|
| `APP_ENV` | no | `dev` | `dev`/`local`/`test` = HTTP; any other value requires TLS config |
| `ENCRYPTION_KEY` | **yes** | auto-generated | AES-256 key ≥32 bytes; rejects known insecure defaults |
| `SIGNING_KEY` | **yes** | auto-generated | Session signing key ≥32 bytes |
| `COOKIE_SECURE` | no | `false` | Set `true` in production (HTTPS) |
| `TRUSTED_TLS_PROXY` | no | unset | Set `true` when a reverse proxy terminates TLS |
| `TLS_CERT_PATH` | no | unset | PEM cert chain path for in-process TLS |
| `TLS_KEY_PATH` | no | unset | PEM private key path for in-process TLS |
| `TRUSTED_PROXY_HEADERS` | no | unset | Set `true` to trust `X-Forwarded-For` for rate limiting |
| `DATABASE_URL` | no | `sqlite:///app/data/scholarvault.db` | SQLite connection string |
| `BACKUP_SCHEDULE` | no | `0 0 2 * * *` | Fallback cron if DB schedule row missing |
| `BACKUP_RETAIN_DAILY` | no | 30 | Days to keep daily backups |
| `BACKUP_RETAIN_MONTHLY` | no | 12 | Months to keep monthly backups |

## Transport security modes

| Mode | Config | Use case |
|------|--------|----------|
| **Dev HTTP** | `APP_ENV=dev` | Local Docker / CI |
| **Trusted proxy** | `TRUSTED_TLS_PROXY=true` | Nginx / Caddy / ALB terminates TLS |
| **In-process TLS** | `TLS_CERT_PATH` + `TLS_KEY_PATH` | Self-hosted without proxy |

The backend **hard-fails at startup** in production mode if none of these is configured.

## Login credentials (dev/test only)

> In non-dev environments (`APP_ENV=production`) seed accounts are
> automatically deactivated at startup. Create real admin users first.

| Role | Username | Password |
|------|----------|----------|
| Administrator | admin | ScholarAdmin2024! |
| Content Curator | curator | Scholar2024! |
| Reviewer | reviewer | Scholar2024! |
| Finance Manager | finance | Scholar2024! |
| Store Manager | store | Scholar2024! |

## Verification checklist

After startup, verify each role can access their expected pages:

- [ ] **Admin** → `/admin` (users, audit log, backups, schedule, retention policy)
- [ ] **Curator** → `/knowledge` (categories, knowledge points, question bank + linking)
- [ ] **Reviewer** → `/outcomes` (register, contributors, evidence upload, submit)
- [ ] **Finance** → `/analytics` (dashboard metrics, fund summary, CSV/PDF export)
- [ ] **Store** → `/store` (products, promotions, checkout)

Key API smoke tests:

```bash
# Health check
curl -s http://localhost:3000/api/healthz

# Login
curl -s -X POST http://localhost:3000/api/auth/login \
  -H 'Content-Type: application/json' \
  -d '{"username":"admin","password":"ScholarAdmin2024!"}'
```
