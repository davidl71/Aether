#!/bin/bash
# Daily Automation Script with Documentation Link Fixing and Exarp Checks
#
# This script runs daily automation tasks including:
# 1. Exarp documentation health check
# 2. Exarp Todo2 alignment analysis
# 3. Exarp duplicate task detection
# 4. Documentation link fixing
# 5. Documentation format validation
# 6. Shared TODO table synchronization
#
# Usage: ./scripts/daily_automation_with_link_fixing.sh [project_dir] [--dry-run]

set -euo pipefail

PROJECT_DIR="${1:-$(pwd)}"
DRY_RUN="${2:-}"

cd "$PROJECT_DIR"

echo "🚀 Starting daily automation tasks..."
echo "Project directory: $PROJECT_DIR"
if [ -n "$DRY_RUN" ]; then
    echo "Mode: DRY-RUN (no changes will be made)"
fi
echo ""

# Track failures for summary
FAILURES=0

# ============================================================================
# PHASE 1: Exarp Checks (via wrapper script)
# ============================================================================
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📦 Phase 1: Exarp Daily Automation Checks"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

EXARP_ARGS=("$PROJECT_DIR")
if [ -n "$DRY_RUN" ]; then
    EXARP_ARGS+=("--dry-run")
fi

if python3 scripts/exarp_daily_automation_wrapper.py "${EXARP_ARGS[@]}" 2>&1 | tee /tmp/exarp_automation.log; then
    echo "✅ Exarp automation checks completed"
else
    echo "⚠️  Exarp automation checks completed with warnings"
    FAILURES=$((FAILURES + 1))
fi
echo ""

# ============================================================================
# PHASE 2: Documentation Automation
# ============================================================================
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📚 Phase 2: Documentation Automation"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# 1. Fix documentation links
echo "📝 Task 1: Fixing documentation links..."
DOC_ARGS=("$PROJECT_DIR")
if [ -n "$DRY_RUN" ]; then
    DOC_ARGS+=("--dry-run")
else
    DOC_ARGS+=("--apply")
fi

if python3 scripts/exarp_fix_documentation_links.py "${DOC_ARGS[@]}" 2>&1 | tee /tmp/link_fix.log; then
    echo "✅ Documentation links fixed"
else
    echo "⚠️  Documentation link fixing completed with warnings"
    FAILURES=$((FAILURES + 1))
fi
echo ""

# 2. Validate documentation format
echo "📋 Task 2: Validating documentation format..."
if python3 scripts/exarp_validate_docs_format.py "$PROJECT_DIR" 2>&1 | tee /tmp/format_validation.log; then
    echo "✅ Documentation format validated"
else
    echo "⚠️  Documentation format validation found issues"
    FAILURES=$((FAILURES + 1))
fi
echo ""

# 3. Sync shared TODO table
echo "🔄 Task 3: Synchronizing shared TODO table..."
SYNC_ARGS=("$PROJECT_DIR")
if [ -n "$DRY_RUN" ]; then
    SYNC_ARGS+=("--dry-run")
else
    SYNC_ARGS+=("--apply")
fi

if python3 scripts/exarp_sync_shared_todo.py "${SYNC_ARGS[@]}" 2>&1 | tee /tmp/todo_sync.log; then
    echo "✅ Shared TODO table synchronized"
else
    echo "⚠️  Shared TODO synchronization completed with warnings"
    FAILURES=$((FAILURES + 1))
fi
echo ""

# ============================================================================
# Summary
# ============================================================================
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📊 Daily Automation Summary"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

if [ $FAILURES -eq 0 ]; then
    echo "✅ All tasks completed successfully!"
    EXIT_CODE=0
else
    echo "⚠️  Daily automation completed with $FAILURES warning(s)"
    EXIT_CODE=1
fi
echo ""
echo "Reports saved to:"
echo "  - /tmp/exarp_automation.log (Exarp checks)"
echo "  - /tmp/link_fix.log (Link fixing)"
echo "  - /tmp/format_validation.log (Format validation)"
echo "  - /tmp/todo_sync.log (TODO synchronization)"
echo ""

exit $EXIT_CODE
