#!/usr/bin/env bash
# audit_dead_code.sh — Find Python files with zero production callers.
# Usage: ./scripts/audit_dead_code.sh [python/integration | python/services | python]
set -uo pipefail
cd "$(git rev-parse --show-toplevel)" || exit 1

SEARCH_DIR="${1:-python/integration}"
echo "=== Dead code audit: $SEARCH_DIR — $(date) ==="
echo ""

echo "--- CONFIRMED DEAD (0 callers anywhere) ---"
for f in "$SEARCH_DIR"/*.py; do
  [[ -f "$f" ]] || continue
  name=$(basename "$f" .py)
  [[ "$name" == "__init__" ]] && continue
  prod=$(grep -rn "from.*${name} import\|from.*\.${name} import\|import ${name}\b" \
    python/ agents/ scripts/ web/ \
    --include="*.py" --include="*.go" --include="*.sh" \
    --exclude-dir=.venv --exclude-dir=__pycache__ 2>/dev/null \
    | grep -v "^${f}:" | grep -cv "/test_\|/tests/")
  tests=$(grep -rn "from.*${name} import\|from.*\.${name} import\|import ${name}\b" \
    python/tests/ --include="*.py" --exclude-dir=__pycache__ 2>/dev/null | wc -l)
  size=$(wc -l < "$f")
  [[ "$prod" -eq 0 && "$tests" -eq 0 ]] && echo "  $f  [$size lines]"
done

echo ""
echo "--- TEST-ONLY (only referenced from tests, no production callers) ---"
for f in "$SEARCH_DIR"/*.py; do
  [[ -f "$f" ]] || continue
  name=$(basename "$f" .py)
  [[ "$name" == "__init__" ]] && continue
  prod=$(grep -rn "from.*${name} import\|from.*\.${name} import\|import ${name}\b" \
    python/ agents/ scripts/ web/ \
    --include="*.py" --include="*.go" --include="*.sh" \
    --exclude-dir=.venv --exclude-dir=__pycache__ 2>/dev/null \
    | grep -v "^${f}:" | grep -cv "/test_\|/tests/")
  tests=$(grep -rn "from.*${name} import\|from.*\.${name} import\|import ${name}\b" \
    python/tests/ --include="*.py" --exclude-dir=__pycache__ 2>/dev/null | wc -l)
  size=$(wc -l < "$f")
  [[ "$prod" -eq 0 && "$tests" -gt 0 ]] && echo "  $f  [$size lines, $tests test refs]"
done

echo ""
echo "--- ALIVE (has production callers) ---"
for f in "$SEARCH_DIR"/*.py; do
  [[ -f "$f" ]] || continue
  name=$(basename "$f" .py)
  [[ "$name" == "__init__" ]] && continue
  prod=$(grep -rn "from.*${name} import\|from.*\.${name} import\|import ${name}\b" \
    python/ agents/ scripts/ web/ \
    --include="*.py" --include="*.go" --include="*.sh" \
    --exclude-dir=.venv --exclude-dir=__pycache__ 2>/dev/null \
    | grep -v "^${f}:" | grep -cv "/test_\|/tests/")
  size=$(wc -l < "$f")
  [[ "$prod" -gt 0 ]] && echo "  $f  [$size lines, $prod refs]"
done

echo ""
echo "=== done ==="
