#!/usr/bin/env bash
# Run shellcheck on scripts and emit a single JSON line (for tools/AI).
# Usage: ./scripts/lint_shell_ai.sh
# Output: {"success":true|false,"exit_code":N,"linter":"shellcheck","log_path":"..."}
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
cd "${ROOT}"
mkdir -p logs
LOG="${ROOT}/logs/lint_shell_ai.log"

if ! command -v shellcheck >/dev/null 2>&1; then
  echo "{\"success\":false,\"exit_code\":1,\"linter\":\"shellcheck\",\"errors\":[\"shellcheck not found\"],\"log_path\":\"${LOG}\"}"
  exit 1
fi

set +e
shellcheck -x -S error -f gcc scripts/*.sh ansible/run-dev-setup.sh >"${LOG}" 2>&1
code=$?
set -e
success="false"
[[ $code -eq 0 ]] && success="true"
echo "{\"success\":${success},\"exit_code\":${code},\"linter\":\"shellcheck\",\"log_path\":\"${LOG}\"}"
exit "$code"
