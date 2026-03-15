#!/usr/bin/env bash
# Run Rust tests (and optionally ShellSpec). Intended for macOS Shortcuts "Run Shell Script".
# C++ ctest was removed with native build.
#
# Usage:
#   ./scripts/shortcuts/run_tests.sh
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
LOG_DIR="${PROJECT_ROOT}/logs"
mkdir -p "${LOG_DIR}"
TEST_LOG="${LOG_DIR}/tests_latest.log"

cd "${PROJECT_ROOT}"
{
  echo "=== Test Runner ==="
  echo "Timestamp: $(date -Iseconds)"
  echo ""

  echo "--- Rust (agents/backend) ---"
  (cd agents/backend && cargo test 2>&1) || exit 1

  if command -v shellspec >/dev/null 2>&1; then
    echo ""
    echo "--- ShellSpec ---"
    shellspec --format documentation 2>&1 || true
  fi
} 2>&1 | tee "${TEST_LOG}"

echo ""
echo "Log written to ${TEST_LOG}"
