#!/usr/bin/env bash
# AI-friendly Rust build script with JSON output
#
# Usage:
#   ./scripts/build_rust_ai_friendly.sh [cargo args...]
#   ./scripts/build_rust_ai_friendly.sh --json-only [cargo args...]
#
# Output (stdout): JSON with success, duration, errors
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
RUST_DIR="${PROJECT_ROOT}/agents/backend"
LOG_DIR="${PROJECT_ROOT}/logs"
mkdir -p "${LOG_DIR}"
BUILD_LOG="${LOG_DIR}/rust_build_ai_friendly.log"

JSON_ONLY=false
if [[ "${1:-}" == "--json-only" ]]; then
  JSON_ONLY=true
  shift
fi

cd "${RUST_DIR}"

START="$(date +%s.%N)"
set +e
cargo build "$@" 2>&1 | tee "${BUILD_LOG}"
EXIT_CODE=${PIPESTATUS[0]}
set -e
END="$(date +%s.%N)"
DURATION="$(awk "BEGIN { printf \"%.2f\", ${END} - ${START} }")"

SUCCESS="false"
[[ ${EXIT_CODE} -eq 0 ]] && SUCCESS="true"

# Extract errors
if [[ -f "${BUILD_LOG}" ]]; then
  ERRORS=$(grep -E "^error|error\[" "${BUILD_LOG}" 2>/dev/null | head -20 || true)
  if [[ -n "${ERRORS}" ]]; then
    ERROR_JSON=$(echo "${ERRORS}" | jq -R -s -c 'split("\n") | map(select(length > 0))' 2>/dev/null || echo "[]")
  else
    ERROR_JSON="[]"
  fi
else
  ERROR_JSON="[]"
fi

if ${JSON_ONLY}; then
  echo "{\"success\":${SUCCESS},\"exit_code\":${EXIT_CODE},\"duration_sec\":${DURATION},\"log_path\":\"${BUILD_LOG}\",\"errors\":${ERROR_JSON}}"
else
  echo ""
  if [[ ${EXIT_CODE} -ne 0 ]]; then
    echo "Build failed. Log: ${BUILD_LOG}" >&2
    echo "Errors:" >&2
    echo "${ERRORS}" >&2
  else
    echo "Build succeeded in ${DURATION}s"
  fi
fi

exit ${EXIT_CODE}
