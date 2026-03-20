#!/usr/bin/env bash

# check_javascript.sh - Check JavaScript files for syntax errors and silent issues
# Uses Node.js --check flag for syntax validation

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

info() {
  printf '\n\033[1m==> %s\033[0m\n' "$1"
}

warn() {
  printf '\033[33m[warn]\033[0m %s\n' "$1"
}

err() {
  printf '\033[31m[error]\033[0m %s\n' "$1" >&2
}

check_js_syntax() {
  if ! command -v node >/dev/null 2>&1; then
    warn "Skipping JavaScript syntax check (node not found)"
    return 0
  fi

  info "Checking JavaScript syntax (Node.js --check)"

  local errors=0
  local checked=0

  # Find all .js files (excluding node_modules, dist, etc.)
  while IFS= read -r -d '' file; do
    # Skip if in ignored directories
    case "${file}" in
    */node_modules/* | */dist/* | */dev-dist/* | */build/* | */coverage/* | */\.git/* | */.venv/* | */venv/* | */emsdk/* | */ib-gateway/* | */python/* | */native/*)
      continue
      ;;
    esac

    checked=$((checked + 1))
    if ! node --check "${file}" 2>/dev/null; then
      err "Syntax error in: ${file}"
      errors=$((errors + 1))
    fi
  done < <(find "${ROOT_DIR}" -type f -name "*.js" -not -path "*/node_modules/*" -not -path "*/dist/*" -not -path "*/dev-dist/*" -not -path "*/build/*" -not -path "*/coverage/*" -not -path "*/.git/*" -print0 2>/dev/null || true)

  if [ "${checked}" -eq 0 ]; then
    warn "No JavaScript files found to check"
    return 0
  fi

  if [ "${errors}" -gt 0 ]; then
    err "Found ${errors} JavaScript file(s) with syntax errors"
    return 1
  fi

  info "✅ All ${checked} JavaScript file(s) passed syntax check"
  return 0
}

check_js_syntax
