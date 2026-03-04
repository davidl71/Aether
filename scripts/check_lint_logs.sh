#!/usr/bin/env bash
# Show lint log paths and optionally tail or cat them.
#
# Usage:
#   ./scripts/check_lint_logs.sh              # list paths and tail main log
#   ./scripts/check_lint_logs.sh --list       # list paths and metadata only
#   ./scripts/check_lint_logs.sh --tail N      # tail last N lines of main log (default 60)
#   ./scripts/check_lint_logs.sh --all        # tail both lint_ai_friendly and lint_shell_ai
#   ./scripts/check_lint_logs.sh --path       # print only paths (one per line)
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
LOG_DIR="${ROOT}/logs"

MAIN_LOG="${LOG_DIR}/lint_ai_friendly.log"
SHELL_LOG="${LOG_DIR}/lint_shell_ai.log"

list_logs() {
  echo "Lint log directory: ${LOG_DIR}"
  echo ""
  for log in "${MAIN_LOG}" "${SHELL_LOG}"; do
    name="$(basename "${log}")"
    if [[ -f "${log}" ]]; then
      n=$(wc -l < "${log}" 2>/dev/null || true); n=${n:-0}
      mtime="$(ls -l "${log}" 2>/dev/null | awk '{print $6, $7, $8}' || echo "?")"
      echo "  ${name}"
      echo "    path: ${log}"
      echo "    lines: ${n}  modified: ${mtime}"
    else
      echo "  ${name}"
      echo "    path: ${log}"
      echo "    (not found — run linters with --ai-friendly to create)"
    fi
    echo ""
  done
}

tail_main() {
  local n="${1:-60}"
  if [[ ! -f "${MAIN_LOG}" ]]; then
    echo "Log not found: ${MAIN_LOG}" 1>&2
    echo "Run: ./scripts/run_linters.sh --ai-friendly" 1>&2
    exit 1
  fi
  echo "=== Last ${n} lines of $(basename "${MAIN_LOG}") ==="
  tail -n "${n}" "${MAIN_LOG}"
}

tail_all() {
  for log in "${MAIN_LOG}" "${SHELL_LOG}"; do
    if [[ -f "${log}" ]]; then
      echo "=== $(basename "${log}") (last 40 lines) ==="
      tail -n 40 "${log}"
      echo ""
    fi
  done
}

print_paths() {
  echo "${MAIN_LOG}"
  echo "${SHELL_LOG}"
}

case "${1:-}" in
  --list)
    list_logs
    ;;
  --path)
    print_paths
    ;;
  --all)
    list_logs
    echo "---"
    tail_all
    ;;
  --tail)
    n="${2:-60}"
    list_logs
    echo "---"
    tail_main "${n}"
    ;;
  *)
    list_logs
    echo "---"
    tail_main "${1:-60}"
    ;;
esac
