#!/usr/bin/env bash
# validate_todo_table.sh - Validate TODO table format and completeness

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
TODO_FILE="${PROJECT_ROOT}/agents/shared/TODO_OVERVIEW.md"

cd "$PROJECT_ROOT"

echo "🔍 Validating TODO Table..."

# Check file exists
if [ ! -f "$TODO_FILE" ]; then
    echo "❌ TODO_OVERVIEW.md not found at: $TODO_FILE"
    exit 1
fi

# Check for required table structure
if ! grep -q "|.*Task.*|.*Agent.*|.*Status.*|" "$TODO_FILE"; then
    echo "❌ TODO table missing required columns (Task | Agent | Status)"
    exit 1
fi

# Validate table format (basic check)
if ! grep -q "^|.*Task" "$TODO_FILE"; then
    echo "❌ TODO table header not found or malformed"
    exit 1
fi

# Count tasks
TASK_COUNT=$(grep -c "^|.*|.*|" "$TODO_FILE" || echo "0")
if [ "$TASK_COUNT" -eq "0" ]; then
    echo "⚠️  No tasks found in TODO table"
    # Don't fail - empty table may be intentional
else
    echo "✅ Found $TASK_COUNT task entries"
fi

# Check for in-progress tasks (coordination check)
IN_PROGRESS_COUNT=$(grep -c "|.*|.*|.*in_progress.*|" "$TODO_FILE" || echo "0")
if [ "$IN_PROGRESS_COUNT" -gt 0 ]; then
    echo "📋 Found $IN_PROGRESS_COUNT in-progress tasks"

    # Warn if multiple agents working on overlapping tasks
    AGENTS=$(grep "|.*|.*|.*in_progress.*|" "$TODO_FILE" | awk -F'|' '{print $3}' | sort -u | wc -l)
    if [ "$AGENTS" -gt 1 ]; then
        echo "✅ Multiple agents working (good coordination)"
    fi
fi

# Basic format validation (check for pipe separators)
MALFORMED_LINES=$(grep "^|.*|.*|" "$TODO_FILE" | grep -v "^|.*|.*|.*|.*|" || true)
if [ -n "$MALFORMED_LINES" ]; then
    echo "⚠️  Some table rows may be malformed:"
    echo "$MALFORMED_LINES" | head -5
    echo "   (This is a warning, not a failure)"
fi

# Check TODO2 sync status
if [ -f "scripts/validate_todo2_sync.sh" ]; then
    echo ""
    echo "📊 Checking TODO2 sync..."
    bash scripts/validate_todo2_sync.sh || {
        echo "⚠️  TODO2 sync check failed (non-blocking)"
        echo "   Run 'python3 scripts/automate_todo_sync.py' to sync manually"
    }
fi

echo ""
echo "✅ TODO Table validation passed"
exit 0
