#!/bin/sh
# ScholarVault test runner — Docker-only, 4 suites with coverage.
# Requires only Docker. No Rust, no cargo, no Node, no npm needed on the host.
# Run from the repo root: sh run_tests.sh

if ! command -v docker > /dev/null 2>&1; then
  echo "Docker is required. Install it from https://docs.docker.com/get-docker/"
  exit 1
fi

REPO_DIR="$(cd "$(dirname "$0")" && pwd)"
export DOCKER_BUILDKIT=0

echo "========================================"
echo "  ScholarVault Test Suite"
echo "========================================"

# ── Build the test image once (includes tarpaulin for coverage) ───────────────
echo ""
echo "Building test image..."
docker build -f "$REPO_DIR/Dockerfile.test" -t scholarvault-test "$REPO_DIR" 2>&1

BACKEND_UNIT_RESULT="FAIL"
BACKEND_API_RESULT="FAIL"
FRONTEND_UNIT_RESULT="FAIL"
E2E_RESULT="FAIL"

# ── Suite 1: Backend Unit Tests + Line Coverage ────────────────────────────────
# Measures line coverage of the security primitives layer (password, encryption,
# csrf). Unit tests cover these pure functions at 100%; DB-heavy service paths
# are exercised by Suite 2 (API integration tests).
echo ""
echo "--- [1/4] Backend Unit Tests + Coverage (backend/tests/unit_tests/) ---"
if docker run --rm \
  --security-opt seccomp=unconfined \
  -e SQLX_OFFLINE=true \
  -e ENCRYPTION_KEY=test-encryption-key-exactly-32bytes \
  -e "SIGNING_KEY=test-signing-key-for-tests-only!!!!" \
  -e RUST_LOG=error \
  scholarvault-test \
  sh -c "cargo tarpaulin --engine llvm --package backend --test unit_tests \
    --include-files '*/security/*' \
    --out stdout --fail-under 90 2>&1"; then
  BACKEND_UNIT_RESULT="PASS"
fi
echo "Unit (backend): $BACKEND_UNIT_RESULT"

# ── Suite 2: Backend API Tests — HTTP Route Coverage ──────────────────────────
echo ""
echo "--- [2/4] Backend API Tests (backend/tests/api_tests/) ---"
if docker run --rm \
  -e SQLX_OFFLINE=true \
  -e ENCRYPTION_KEY=test-encryption-key-exactly-32bytes \
  -e "SIGNING_KEY=test-signing-key-for-tests-only!!!!" \
  -e RUST_LOG=error \
  scholarvault-test \
  sh -c "cargo test --package backend --test api_tests -- --test-threads=1 2>&1"; then
  BACKEND_API_RESULT="PASS"
fi
echo "API (backend): $BACKEND_API_RESULT"

# ── Suite 3: Frontend Unit Tests + Line Coverage ──────────────────────────────
# Measures line coverage of the frontend logic module (pure Rust functions).
# Leptos components are excluded — they require WASM and are covered by E2E.
echo ""
echo "--- [3/4] Frontend Unit Tests + Coverage (frontend/tests/unit_tests/) ---"
if docker run --rm \
  --security-opt seccomp=unconfined \
  -e SQLX_OFFLINE=true \
  scholarvault-test \
  sh -c "cargo tarpaulin --engine llvm --package frontend --test unit_tests \
    --include-files '*/logic/*' \
    --out stdout --fail-under 90 2>&1"; then
  FRONTEND_UNIT_RESULT="PASS"
fi
echo "Unit (frontend): $FRONTEND_UNIT_RESULT"

# ── Suite 4: Playwright E2E Tests ─────────────────────────────────────────────
echo ""
echo "--- [4/4] E2E Tests (Playwright against live app) ---"
(
  set +e
  cd "$REPO_DIR"
  docker compose up -d --build app
  echo "Waiting for app to be healthy..."
  docker compose up --wait app
  docker compose --profile e2e run --rm playwright
  E2E_EXIT=$?
  docker compose stop app 2>/dev/null || true
  exit $E2E_EXIT
)
if [ $? -eq 0 ]; then
  E2E_RESULT="PASS"
fi
echo "E2E (Playwright): $E2E_RESULT"

# ── Summary ───────────────────────────────────────────────────────────────────
echo ""
echo "========================================"
echo "  Test Results"
echo "========================================"
echo "  Unit (backend):   $BACKEND_UNIT_RESULT"
echo "  API (backend):    $BACKEND_API_RESULT"
echo "  Unit (frontend):  $FRONTEND_UNIT_RESULT"
echo "  E2E (Playwright): $E2E_RESULT"
echo "========================================"

if [ "$BACKEND_UNIT_RESULT" = "PASS" ] && \
   [ "$BACKEND_API_RESULT" = "PASS" ] && \
   [ "$FRONTEND_UNIT_RESULT" = "PASS" ] && \
   [ "$E2E_RESULT" = "PASS" ]; then
  exit 0
else
  exit 1
fi
