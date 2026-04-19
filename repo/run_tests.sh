#!/bin/sh
# ScholarVault test runner — Docker-only, 4 suites with coverage.
# Requires only Docker. No Rust, no cargo, no Node, no npm needed on the host.
# Run from the repo root: sh run_tests.sh

if ! command -v docker > /dev/null 2>&1; then
  echo "Docker is required. Install it from https://docs.docker.com/get-docker/"
  exit 1
fi

REPO_DIR="$(cd "$(dirname "$0")" && pwd)"
export DOCKER_BUILDKIT=1

# On any exit (normal, error, or watchdog kill) stop named test containers.
cleanup() {
  kill "$WATCHDOG_PID" 2>/dev/null || true
  docker rm -f sv-suite1 sv-suite2 sv-suite3 2>/dev/null || true
}
trap cleanup EXIT

# Hard ceiling: kill the entire run after 120 minutes.
# kill -9 cannot be caught or queued — it fires even while blocked in docker run.
MAIN_PID=$$
( sleep 7200 && echo "TIMEOUT: test suite exceeded 120 minutes — forcing exit" && kill -9 "$MAIN_PID" ) &
WATCHDOG_PID=$!

echo "========================================"
echo "  ScholarVault Test Suite"
echo "========================================"

# ── Build the test image ──────────────────────────────────────────────────────
echo ""
echo "Building test image..."
docker build -f "$REPO_DIR/Dockerfile.test" -t scholarvault-test "$REPO_DIR" 2>&1

# Sanity-check: verify packages are actually baked into the image layer.
# If registry is missing the image was built with a cache-mount on cargo fetch
# and every docker run container would have to re-download 300+ crates.
echo "Verifying cargo registry is in image layer..."
if ! docker run --rm scholarvault-test sh -c "[ -d /usr/local/cargo/registry/src ]"; then
  echo "ERROR: /usr/local/cargo/registry/src missing from image."
  echo "Dockerfile.test must use plain 'RUN cargo fetch' (no --mount=type=cache)."
  echo "Force-remove the stale image and rebuild:"
  echo "  docker rmi scholarvault-test && sh run_tests.sh"
  exit 1
fi
echo "Registry OK — packages are in the image layer."

BACKEND_UNIT_RESULT="FAIL"
BACKEND_API_RESULT="FAIL"
FRONTEND_UNIT_RESULT="FAIL"
E2E_RESULT="FAIL"

# Shared named volume for compiled Rust artifacts — persists between suites so
# incremental compilation can reuse unchanged crates across containers.
TARGET_VOL="scholarvault-test-target"

# ── Suite 1: Backend Unit Tests + Line Coverage ────────────────────────────────
echo ""
echo "--- [1/4] Backend Unit Tests + Coverage (backend/tests/unit_tests/) ---"
if timeout 1200 docker run --rm \
  --name sv-suite1 \
  --security-opt seccomp=unconfined \
  -v "${TARGET_VOL}:/workspace/target" \
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

# ── Suite 2: Backend API Tests ─────────────────────────────────────────────────
echo ""
echo "--- [2/4] Backend API Tests (backend/tests/api_tests/) ---"
if timeout 1200 docker run --rm \
  --name sv-suite2 \
  -v "${TARGET_VOL}:/workspace/target" \
  -e SQLX_OFFLINE=true \
  -e ENCRYPTION_KEY=test-encryption-key-exactly-32bytes \
  -e "SIGNING_KEY=test-signing-key-for-tests-only!!!!" \
  -e RUST_LOG=error \
  scholarvault-test \
  sh -c "cargo test --package backend --test api_tests -- --test-threads=1 2>&1"; then
  BACKEND_API_RESULT="PASS"
fi
echo "API (backend): $BACKEND_API_RESULT"

# ── Suite 3: Frontend Unit Tests + Line Coverage ───────────────────────────────
echo ""
echo "--- [3/4] Frontend Unit Tests + Coverage (frontend/tests/unit_tests/) ---"
if timeout 1200 docker run --rm \
  --name sv-suite3 \
  --security-opt seccomp=unconfined \
  -v "${TARGET_VOL}:/workspace/target" \
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

  docker pull debian:bookworm-slim 2>/dev/null || true

  docker compose up -d --build app

  echo "Waiting for app to be healthy..."
  MAX_WAIT=450
  COUNT=0
  until docker compose ps app | grep -q "healthy"; do
    COUNT=$((COUNT + 1))
    if [ $COUNT -ge $MAX_WAIT ]; then
      echo "App failed to become healthy after ${MAX_WAIT} seconds"
      docker compose logs app | tail -50
      docker compose stop app 2>/dev/null || true
      exit 1
    fi
    sleep 1
  done
  echo "App is healthy after ${COUNT}s"

  docker compose --profile e2e run -T --rm playwright
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
