#!/usr/bin/env bash

# Integration Test Script for IBKR Box Spread Generator
# Tests TWS API connectivity and basic operations.
# Expectation: run after building the CLI (see scripts/build_universal.sh) and configuring native/config/config.json.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=./include/logging.sh
. "${SCRIPT_DIR}/include/logging.sh"

PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
DEFAULT_PRESET="macos-x86_64-release"
PRESET="${CMAKE_PRESET:-${DEFAULT_PRESET}}"
BUILD_DIR="${PROJECT_ROOT}/build/${PRESET}"
LOG_DIR="${PROJECT_ROOT}/build/integration_logs"
mkdir -p "${LOG_DIR}"
timestamp="$(date +%Y%m%d-%H%M%S)"
CTEST_LOG="${LOG_DIR}/ctest_${PRESET}_${timestamp}.log"
VALIDATION_LOG="${LOG_DIR}/validate_${timestamp}.log"

if ! command -v ctest >/dev/null 2>&1; then
  log_error "ctest not found. Install CMake/CTest before running integration wrapper."
  exit 1
fi

log_note "Running ctest preset '${PRESET}' (override with CMAKE_PRESET)."

(cd "${PROJECT_ROOT}" && cmake --preset "${PRESET}") | tee "${CTEST_LOG}"

(cd "${PROJECT_ROOT}" && cmake --build --preset "${PRESET}" -j"$(sysctl -n hw.ncpu)") | tee -a "${CTEST_LOG}"

(cd "${PROJECT_ROOT}" && ctest --preset "${PRESET}" --output-on-failure --label-regex "integration|Integration" "$@") | tee -a "${CTEST_LOG}"

CLI_BIN="${BUILD_DIR}/bin/ib_box_spread"
if [ ! -x "${CLI_BIN}" ]; then
  log_error "CLI binary not found at ${CLI_BIN}. Did the build succeed?"
  exit 1
fi

EXAMPLE_CONFIG="${PROJECT_ROOT}/config/config.example.json"
if [ ! -f "${EXAMPLE_CONFIG}" ]; then
  log_error "Config example missing at ${EXAMPLE_CONFIG}"
  exit 1
fi

TEMP_CONFIG="$(mktemp "${LOG_DIR}/config_XXXXXX.json")"
cleanup() {
  rm -f "${TEMP_CONFIG}"
}
trap cleanup EXIT

log_note "Generating sanitized config (based on example) -> ${TEMP_CONFIG}"
if ! "${CLI_BIN}" --init-config "${TEMP_CONFIG}" >/dev/null; then
  log_error "Failed to generate sanitized config"
  exit 1
fi

CONFIG_FILE="${TEMP_CONFIG}"

log_note "Validating example configuration with --validate (output -> ${VALIDATION_LOG})"
if ! "${CLI_BIN}" --config "${CONFIG_FILE}" --validate | tee "${VALIDATION_LOG}"; then
  log_error "CLI validation run failed. Check ${VALIDATION_LOG}"
  exit 1
fi

if ! grep -q "Configuration validation successful" "${VALIDATION_LOG}"; then
  log_error "Expected validation success message not found in ${VALIDATION_LOG}"
  exit 1
fi

MOCK_RUN_TIMEOUT="${MOCK_TWS_TIMEOUT:-8}"
MOCK_RUN_LOG="${LOG_DIR}/mock_run_${timestamp}.log"
if command -v timeout >/dev/null 2>&1; then
  log_note "Running CLI in mock TWS mode for ${MOCK_RUN_TIMEOUT}s (log -> ${MOCK_RUN_LOG})"
  set +e
  (
    cd "${PROJECT_ROOT}" || exit 1
    timeout "${MOCK_RUN_TIMEOUT}" "${CLI_BIN}" --config "${CONFIG_FILE}" --mock-tws --dry-run --log-level warn
  ) >"${MOCK_RUN_LOG}" 2>&1
  mock_status=$?
  set -e
  if [ "${mock_status}" -ne 0 ] && [ "${mock_status}" -ne 124 ]; then
    log_error "Mock TWS run failed (exit code ${mock_status}). See ${MOCK_RUN_LOG}"
    exit 1
  fi
  if [ "${mock_status}" -eq 124 ]; then
    log_note "Mock TWS run reached timeout (expected)."
  else
    log_note "Mock TWS run completed successfully before timeout."
  fi
else
  log_warn "timeout command not found; skipping mock TWS execution"
fi

log_note "Integration checks complete. Logs saved to ${LOG_DIR}"
