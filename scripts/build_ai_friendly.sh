#!/usr/bin/env bash
# Run CMake build in AI-friendly mode: quiet (log to file) and emit a single JSON result.
# Use with BUILD_AI_FRIENDLY preset so compiler diagnostics are JSON when build fails.
# When build-ramdisk is set up (e.g. ./scripts/setup_ramdisk.sh create), uses the -ramdisk
# preset automatically so the build runs on the ramdisk.
#
# Usage:
#   ./scripts/build_ai_friendly.sh [build|configure] [<preset>]
#   ./scripts/build_ai_friendly.sh --json-only [<preset>]   # build and print only JSON to stdout
#   BUILD_KEEP_GOING=1 ./scripts/build_ai_friendly.sh       # continue past failures to surface more errors (-k 0)
#
# Output (stdout): one JSON object
#   {"success": true, "exit_code": 0, "duration_sec": 12.3, "log_path": "...", "errors": []}
#   {"success": false, "exit_code": 1, "duration_sec": 5.2, "log_path": "...", "errors": ["..."]}
#
# Env: CMAKE_PRESET overrides default preset.
# Env: CLEAN_TWS_API=1 (or unset) — on macOS, remove TWS API sub-build before building so it
#      reconfigures with SDK C++ and client staging (avoids stale "mutex/string not found" and
#      Protobuf version mismatch). Set CLEAN_TWS_API=0 to skip (faster rebuilds when TWS is already ok).
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
      # Unknown OS: prefer arch from uname -m to avoid assuming Apple Silicon
      if [[ "${arch}" == "x86_64" || "${arch}" == "amd64" ]]; then
        echo "macos-x86_64-debug"
      elif [[ "${arch}" == "arm64" || "${arch}" == "aarch64" ]]; then
        echo "macos-arm64-debug"
      else
        echo "macos-x86_64-debug"
      fi
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

# If build-ramdisk is set up (symlink or dir), prefer the -ramdisk preset when it exists
prefer_ramdisk_if_setup() {
  local p="$1"
  [[ -z "${p}" ]] && return
  local ramdisk_dir="${PROJECT_ROOT}/build-ramdisk"
  if [[ -d "${ramdisk_dir}" ]] && [[ -w "${ramdisk_dir}" ]]; then
    local ramdisk_preset="${p}-ramdisk"
    if cmake --list-presets 2>/dev/null | grep -q "^\s*${ramdisk_preset}\s"; then
      echo "${ramdisk_preset}"
      return
    fi
  fi
  echo "${p}"
}

ACTION="${1:-build}"
PRESET="${2:-${CMAKE_PRESET:-$(detect_default_preset)}}"
if [[ "${ACTION}" == "--json-only" ]]; then
  JSON_ONLY=true
  ACTION="${2:-build}"
  PRESET="${3:-${CMAKE_PRESET:-$(detect_default_preset)}}"
fi
PRESET="$(resolve_preset "${PRESET}")"
PRESET="$(prefer_ramdisk_if_setup "${PRESET}")"

cd "${PROJECT_ROOT}"
# Use all cores for Ninja when not set (see docs/BUILD_PARALLELIZATION_AND_MODULARITY.md)
# shellcheck source=./include/set_parallel_level.sh
. "${SCRIPT_DIR}/include/set_parallel_level.sh"

# On macOS, force TWS API and optionally Catch2 to reconfigure so SDK/staging and Catch2 version are current
if [[ "$(uname -s)" == "Darwin" ]]; then
  if [[ "${PRESET}" == *-ramdisk ]]; then
    BUILD_DIR="${PROJECT_ROOT}/build-ramdisk"
  else
    BUILD_DIR="${PROJECT_ROOT}/build/${PRESET}"
  fi
else
  BUILD_DIR="${PROJECT_ROOT}/build/${PRESET}"
fi
if [[ "${ACTION}" == "build" ]] && [[ "$(uname -s)" == "Darwin" ]] && [[ -d "${BUILD_DIR}" ]]; then
  if [[ "${CLEAN_TWS_API:-1}" != "0" ]]; then
    rm -rf "${BUILD_DIR}/native/twsapi_external-prefix" "${BUILD_DIR}/tws_api_vendor_build"
  fi
  # Clear Catch2 FetchContent cache so next configure uses GIT_TAG v3.5.4 (valid VERSION)
  if [[ -d "${BUILD_DIR}/_deps/catch2-src" ]]; then
    if grep -q 'VERSION 3.5.2-develop' "${BUILD_DIR}/_deps/catch2-src/CMakeLists.txt" 2>/dev/null; then
      rm -rf "${BUILD_DIR}/_deps/catch2-"*
    fi
  fi
fi
START="$(date +%s.%N)"

run_build() {
  if [[ "${ACTION}" == "configure" ]]; then
    cmake --preset "${PRESET}" >> "${BUILD_LOG}" 2>&1
  else
    cmake --preset "${PRESET}" >> "${BUILD_LOG}" 2>&1 || true
    if [[ -n "${BUILD_KEEP_GOING:-}" ]] && [[ "${BUILD_KEEP_GOING}" != "0" ]]; then
      # Pass -k 0 to Ninja: continue building after failures to detect more issues
      cmake --build --preset "${PRESET}" -- -k 0 >> "${BUILD_LOG}" 2>&1
    else
      cmake --build --preset "${PRESET}" >> "${BUILD_LOG}" 2>&1
    fi
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
