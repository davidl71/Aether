#!/bin/bash
# Daily Automation Script with Documentation Link Fixing
#
# This script runs daily automation tasks including:
# 1. Documentation link fixing
# 2. Documentation format validation
# 3. Shared TODO table synchronization
#
# Usage: ./scripts/daily_automation_with_link_fixing.sh [project_dir]

set -e

PROJECT_DIR="${1:-$(pwd)}"
cd "$PROJECT_DIR"

echo "🚀 Starting daily automation tasks..."
echo "Project directory: $PROJECT_DIR"
echo ""

# 1. Fix documentation links
echo "📝 Task 1: Fixing documentation links..."
if python3 scripts/exarp_fix_documentation_links.py "$PROJECT_DIR" --apply 2>&1 | tee /tmp/link_fix.log; then
    echo "✅ Documentation links fixed"
else
    echo "⚠️  Documentation link fixing completed with warnings"
fi
echo ""

# 2. Validate documentation format
echo "📋 Task 2: Validating documentation format..."
if python3 scripts/exarp_validate_docs_format.py "$PROJECT_DIR" 2>&1 | tee /tmp/format_validation.log; then
    echo "✅ Documentation format validated"
else
    echo "⚠️  Documentation format validation found issues"
fi
echo ""

# 3. Sync shared TODO table
echo "🔄 Task 3: Synchronizing shared TODO table..."
if python3 scripts/exarp_sync_shared_todo.py "$PROJECT_DIR" --apply 2>&1 | tee /tmp/todo_sync.log; then
    echo "✅ Shared TODO table synchronized"
else
    echo "⚠️  Shared TODO synchronization completed with warnings"
fi
echo ""

echo "✅ Daily automation complete!"
echo ""
echo "Reports saved to:"
echo "  - /tmp/link_fix.log"
echo "  - /tmp/format_validation.log"
echo "  - /tmp/todo_sync.log"
