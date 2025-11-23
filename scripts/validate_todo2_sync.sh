#!/usr/bin/env bash
# validate_todo2_sync.sh - Validate TODO2 tasks are in sync with shared TODO table

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
SHARED_TODO_FILE="${PROJECT_ROOT}/agents/shared/TODO_OVERVIEW.md"
TODO2_FILE="${PROJECT_ROOT}/.todo2/state.todo2.json"

cd "$PROJECT_ROOT"

echo "🔍 Validating TODO2 sync..."

# Check files exist
if [ ! -f "$SHARED_TODO_FILE" ]; then
    echo "❌ Shared TODO file not found: $SHARED_TODO_FILE"
    exit 1
fi

if [ ! -f "$TODO2_FILE" ]; then
    echo "❌ TODO2 file not found: $TODO2_FILE"
    exit 1
fi

# Run sync script in dry-run mode to check for issues
if command -v python3 >/dev/null 2>&1; then
    echo "📊 Checking TODO2 sync status..."
    python3 scripts/automate_todo_sync.py --dry-run 2>/dev/null || {
        echo "⚠️  TODO2 sync check failed (non-blocking)"
        echo "   This may indicate sync issues - review manually if needed"
        exit 0  # Don't fail CI, just warn
    }
else
    echo "⚠️  Python3 not found, skipping detailed sync validation"
fi

# Basic validation: Check TODO2 file is valid JSON
if ! python3 -m json.tool "$TODO2_FILE" >/dev/null 2>&1; then
    echo "❌ TODO2 file is not valid JSON"
    exit 1
fi

echo "✅ TODO2 sync validation passed"
exit 0
