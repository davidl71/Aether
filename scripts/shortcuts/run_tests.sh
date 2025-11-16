#!/usr/bin/env bash
# Run ctest with a preset. Intended for macOS Shortcuts “Run Shell Script”.
#
# Usage:
#   ./scripts/shortcuts/run_tests.sh [<ctest_preset>]
# Env:
#   CTEST_PRESET - override test preset (falls back to platform default)
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
LOG_DIR="${PROJECT_ROOT}/logs"
mkdir -p "${LOG_DIR}"
TEST_LOG="${LOG_DIR}/tests_latest.log"

detect_default_preset() {
  local arch os
  arch="$(uname -m 2>/dev/null || echo unknown)"
  os="$(uname -s 2>/dev/null || echo unknown)"
  case "${os}" in
    Darwin)
      if [[ "${arch}" == "arm64" || "${arch}" == "aarch64" ]]; then
        echo "macos-arm64-debug"
      else
        echo "macos-x86_64-debug"
      fi
      ;;
    Linux)
      echo "linux-x64-debug"
      ;;
    MINGW*|MSYS*|CYGWIN*|Windows_NT)
      echo "windows-x64-debug"
      ;;
    *)
      echo "macos-arm64-debug"
      ;;
  esac
}

PRESET="${1:-${CTEST_PRESET:-$(detect_default_preset)}}"

cd "${PROJECT_ROOT}"
{
  echo "=== Test Runner ==="
  echo "Timestamp: $(date -Iseconds)"
  echo "Preset: ${PRESET}"
  echo ""

  # Ensure configured build dir exists
  cmake --preset "${PRESET}" >/dev/null 2>&1 || true
  ctest --preset "${PRESET}" --output-on-failure
} 2>&1 | tee "${TEST_LOG}"

echo ""
echo "Log written to ${TEST_LOG}"
