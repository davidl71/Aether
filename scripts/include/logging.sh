#!/usr/bin/env bash

# Shared logging helpers for project shell scripts.
# Intentionally lightweight so other scripts can `source` this once; guarded to avoid double-loading.
# Usage:
#   SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
#   # shellcheck source=./include/logging.sh
#   . "${SCRIPT_DIR}/include/logging.sh"

if [[ -n "${__IB_BOX_SPREAD_LOGGING_INCLUDED:-}" ]]; then
  # shellcheck disable=SC2317
  return 0 2>/dev/null || :
fi
__IB_BOX_SPREAD_LOGGING_INCLUDED=1

if [[ -t 1 ]]; then
  __LOG_COLOR_INFO=$'\033[0;32m'
  __LOG_COLOR_WARN=$'\033[1;33m'
  __LOG_COLOR_ERROR=$'\033[0;31m'
  __LOG_COLOR_NOTE=$'\033[0;36m'
  __LOG_COLOR_RESET=$'\033[0m'
else
  __LOG_COLOR_INFO=''
  __LOG_COLOR_WARN=''
  __LOG_COLOR_ERROR=''
  __LOG_COLOR_NOTE=''
  __LOG_COLOR_RESET=''
fi

__ib_log_emit() {
  local level="$1"
  local color="$2"
  shift 2

  if [[ $# -eq 0 ]]; then
    return 0
  fi

  local message="$*"

  if [[ "$level" == "ERROR" ]]; then
    printf '%s[%s]%s %s\n' "$color" "$level" "$__LOG_COLOR_RESET" "$message" >&2
  else
    printf '%s[%s]%s %s\n' "$color" "$level" "$__LOG_COLOR_RESET" "$message"
  fi
}

log_info() {
  __ib_log_emit "INFO" "$__LOG_COLOR_INFO" "$@"
}

log_warn() {
  __ib_log_emit "WARN" "$__LOG_COLOR_WARN" "$@"
}

log_error() {
  __ib_log_emit "ERROR" "$__LOG_COLOR_ERROR" "$@"
}

log_note() {
  __ib_log_emit "NOTE" "$__LOG_COLOR_NOTE" "$@"
}

# Backwards-compatible aliases for legacy scripts.
log() {
  log_info "$@"
}

warn() {
  log_warn "$@"
}

err() {
  log_error "$@"
}


