#!/usr/bin/env bash
# Lint only staged files (e.g. after generation, before commit).
# Used by pre-commit hook "lint-staged"; can be run standalone:
#   ./scripts/lint_staged.sh
#
# Lints:
#   - Staged .py files: ruff check + ruff format (fix)
# Exit 0 if nothing to do or lint passes; non-zero if lint fails.
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

STAGED_PY="$(git diff --cached --name-only --diff-filter=ACM | grep -E '\.py$' || true)"
if [[ -z "${STAGED_PY}" ]]; then
  exit 0
fi

# Run from repo root so paths and config (pyproject.toml / ruff config) resolve
mapfile -t STAGED_PY_ARR <<<"$STAGED_PY"
if ! uv run ruff check "${STAGED_PY_ARR[@]}"; then
  echo "Fix ruff errors above, then re-stage and commit."
  exit 1
fi
if ! uv run ruff format "${STAGED_PY_ARR[@]}"; then
  echo "Re-stage formatted files and commit."
  exit 1
fi
# Re-stage in case ruff format changed files
if [[ -n "${STAGED_PY}" ]]; then
  echo "$STAGED_PY" | xargs git add
fi
exit 0
