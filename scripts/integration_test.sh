#!/usr/bin/env bash

# Integration Test Script for IBKR Box Spread Generator
# Tests TWS API connectivity and basic operations.
# Expectation: run after building the CLI (see scripts/build_universal.sh) and configuring native/config/config.json.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=./include/logging.sh
. "${SCRIPT_DIR}/include/logging.sh"

PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
DEFAULT_PRESET="macos-universal-release"
PRESET="${CMAKE_PRESET:-${DEFAULT_PRESET}}"

if ! command -v ctest >/dev/null 2>&1; then
  log_error "ctest not found. Install CMake/CTest before running integration wrapper."
  exit 1
fi

log_note "Running ctest preset '${PRESET}' (override with CMAKE_PRESET)."

(cd "${PROJECT_ROOT}" && cmake --preset "${PRESET}" >/dev/null)
(cd "${PROJECT_ROOT}" && ctest --preset "${PRESET}" --output-on-failure --label-regex "integration|Integration" "$@")
