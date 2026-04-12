#!/bin/sh
set -e

echo "========================================"
echo "  ScholarVault Test Suite"
echo "========================================"

if command -v cargo > /dev/null 2>&1; then
  CARGO="cargo"
else
  # No Rust toolchain on this host — delegate to the Docker test container.
  echo "cargo not found on host — running tests via Docker..."
  exec docker compose --profile test run --build test
fi

export SQLX_OFFLINE=true

BACKEND_UNIT_FAILED=0
BACKEND_API_FAILED=0
FRONTEND_UNIT_FAILED=0
FRONTEND_API_FAILED=0

echo ""
echo "--- Backend Unit Tests (backend/tests/unit_tests/) ---"
$CARGO test --package backend --test unit_tests \
  -- --test-threads=1 2>&1 || BACKEND_UNIT_FAILED=1
[ $BACKEND_UNIT_FAILED -eq 0 ] && echo "PASS Backend Unit Tests" \
                                || echo "FAIL Backend Unit Tests"

echo ""
echo "--- Backend API Tests (backend/tests/api_tests/) ---"
$CARGO test --package backend --test api_tests \
  -- --test-threads=1 2>&1 || BACKEND_API_FAILED=1
[ $BACKEND_API_FAILED -eq 0 ] && echo "PASS Backend API Tests" \
                               || echo "FAIL Backend API Tests"

echo ""
echo "--- Frontend Unit Tests (frontend/tests/unit_tests/) ---"
$CARGO test --package frontend --test unit_tests \
  -- --test-threads=1 2>&1 || FRONTEND_UNIT_FAILED=1
[ $FRONTEND_UNIT_FAILED -eq 0 ] && echo "PASS Frontend Unit Tests" \
                                 || echo "FAIL Frontend Unit Tests"

echo ""
echo "--- Frontend API Client Tests (frontend/tests/api_tests/) ---"
$CARGO test --package frontend --test api_tests \
  -- --test-threads=1 2>&1 || FRONTEND_API_FAILED=1
[ $FRONTEND_API_FAILED -eq 0 ] && echo "PASS Frontend API Tests" \
                                || echo "FAIL Frontend API Tests"

echo ""
echo "========================================"
TOTAL_FAILED=$((BACKEND_UNIT_FAILED + BACKEND_API_FAILED + FRONTEND_UNIT_FAILED + FRONTEND_API_FAILED))
if [ $TOTAL_FAILED -eq 0 ]; then
  echo "  ALL TESTS PASSED"
  exit 0
else
  echo "  SOME TESTS FAILED"
  echo "  Backend Unit:  $([ $BACKEND_UNIT_FAILED -eq 0 ] && echo PASS || echo FAIL)"
  echo "  Backend API:   $([ $BACKEND_API_FAILED -eq 0 ] && echo PASS || echo FAIL)"
  echo "  Frontend Unit: $([ $FRONTEND_UNIT_FAILED -eq 0 ] && echo PASS || echo FAIL)"
  echo "  Frontend API:  $([ $FRONTEND_API_FAILED -eq 0 ] && echo PASS || echo FAIL)"
  exit 1
fi
