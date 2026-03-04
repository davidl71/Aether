#!/usr/bin/env bash
# Run CMake build in AI-friendly mode: quiet (log to file) and emit a single JSON result.
# Use with BUILD_AI_FRIENDLY preset so compiler diagnostics are JSON when build fails.
#
# Usage:
#   ./scripts/build_ai_friendly.sh [build|configure] [<preset>]
#   ./scripts/build_ai_friendly.sh --json-only [<preset>]   # build and print only JSON to stdout
#
# Output (stdout): one JSON object
#   {"success": true, "exit_code": 0, "duration_sec": 12.3, "log_path": "...", "errors": []}
#   {"success": false, "exit_code": 1, "duration_sec": 5.2, "log_path": "...", "errors": ["..."]}
#
# Env: CMAKE_PRESET overrides default preset.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
LOG_DIR="${PROJECT_ROOT}/logs"
mkdir -p "${LOG_DIR}"
BUILD_LOG="${LOG_DIR}/build_ai_friendly.log"
JSON_ONLY=false

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

# If preset ends with -ai and exists, use it (enables BUILD_AI_FRIENDLY / JSON compiler diagnostics)
resolve_preset() {
  local p="$1"
  # Use as-is if it exists (e.g. macos-arm64-debug-ai when you want JSON diagnostics)
  if cmake --list-presets 2>/dev/null | grep -q "^\s*${p}\s"; then
    echo "${p}"
  else
    # Strip -ai suffix if preset not found so default still works before -ai presets are added
    case "${p}" in
      *-ai) echo "${p%-ai}" ;;
      *)    echo "${p}" ;;
    esac
  fi
}

ACTION="${1:-build}"
PRESET="${2:-${CMAKE_PRESET:-$(detect_default_preset)}}"
if [[ "${ACTION}" == "--json-only" ]]; then
  JSON_ONLY=true
  ACTION="${2:-build}"
  PRESET="${3:-${CMAKE_PRESET:-$(detect_default_preset)}}"
fi
PRESET="$(resolve_preset "${PRESET}")"

cd "${PROJECT_ROOT}"
START="$(date +%s.%N)"

run_build() {
  if [[ "${ACTION}" == "configure" ]]; then
    cmake --preset "${PRESET}" >> "${BUILD_LOG}" 2>&1
  else
    cmake --preset "${PRESET}" >> "${BUILD_LOG}" 2>&1 || true
    cmake --build --preset "${PRESET}" >> "${BUILD_LOG}" 2>&1
  fi
}

extract_errors() {
  local log="$1"
  if [[ ! -f "${log}" ]]; then
    echo "[]"
    return
  fi
  local errs
  errs="$(grep -E "error:|Error|fatal error:|FAILED:" "${log}" 2>/dev/null | head -50)"
  if [[ -z "${errs}" ]]; then
    errs="$(tail -20 "${log}" 2>/dev/null)"
  fi
  if [[ -z "${errs}" ]]; then
    echo "[]"
    return
  fi
  if command -v jq >/dev/null 2>&1; then
    echo "${errs}" | jq -R -s -c 'split("\n") | map(select(length > 0))'
  else
    echo "${errs}" | awk 'BEGIN { first=1; printf "[" }
      { gsub(/\\/,"\\\\"); gsub(/"/,"\\\""); gsub(/\t/," "); if (!first) printf ","; first=0; printf "\""; for(i=1;i<=NF;i++){if(i>1)printf " "; printf "%s",$i}; printf "\"" }
      END { printf "]" }'
  fi
}

exit_code=0
run_build || exit_code=$?
END="$(date +%s.%N)"
DURATION="$(awk "BEGIN { printf \"%.2f\", ${END} - ${START} }" 2>/dev/null || echo "0")"
ERRORS_JSON="$(extract_errors "${BUILD_LOG}")"

SUCCESS="false"
[[ ${exit_code} -eq 0 ]] && SUCCESS="true"

# Single-line JSON for easy parsing
if ${JSON_ONLY}; then
  echo "{\"success\":${SUCCESS},\"exit_code\":${exit_code},\"duration_sec\":${DURATION},\"log_path\":\"${BUILD_LOG}\",\"preset\":\"${PRESET}\",\"errors\":${ERRORS_JSON}}"
else
  echo "{\"success\":${SUCCESS},\"exit_code\":${exit_code},\"duration_sec\":${DURATION},\"log_path\":\"${BUILD_LOG}\",\"preset\":\"${PRESET}\",\"errors\":${ERRORS_JSON}}"
  if [[ ${exit_code} -ne 0 ]]; then
    echo "" 1>&2
    echo "Build failed. Log: ${BUILD_LOG}" 1>&2
    echo "Last 15 lines:" 1>&2
    tail -15 "${BUILD_LOG}" 1>&2
  fi
fi
exit ${exit_code}
