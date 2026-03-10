#!/usr/bin/env bash

# Helper utilities for build scripts to emit structured logs compatible with
# Cursor’s AI agent, GitHub Actions annotations, and local troubleshooting.
# Expects scripts to source `logging.sh` first so log_* helpers are available.

if [[ -n "${__IB_BOX_BUILD_LOGGING_INCLUDED:-}" ]]; then
  # shellcheck disable=SC2317
  return 0 2>/dev/null || :
fi
__IB_BOX_BUILD_LOGGING_INCLUDED=1
# shellcheck source=./workspace_paths.sh
. "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/workspace_paths.sh"

setup_workspace_paths

__IB_BOX_BUILD_LOG_ROOT_DEFAULT="${PROJECT_ROOT}/build/logs"
BUILD_LOG_ROOT="${BUILD_LOG_ROOT:-${__IB_BOX_BUILD_LOG_ROOT_DEFAULT}}"

set_build_log_root() {
  local root="$1"
  BUILD_LOG_ROOT="${root}"
  mkdir -p "${BUILD_LOG_ROOT}"
}

build_log_path() {
  local name="$1"
  mkdir -p "${BUILD_LOG_ROOT}"
  printf '%s/%s.log' "${BUILD_LOG_ROOT}" "${name}"
}

run_logged() {
  local log_file="$1"
  shift
  local label="$1"
  shift

  mkdir -p "$(dirname "${log_file}")"
  log_info "${label}"

  local status
  set +e
  (
    set -o pipefail
    "$@" 2>&1 | tee "${log_file}"
  )
  status=${PIPESTATUS[0]}
  set -e

  if [[ "${status}" -ne 0 ]]; then
    log_error "::error ::${label} failed (see ${log_file})"
    return "${status}"
  fi

  log_note "${label} completed (log: ${log_file})"
}

