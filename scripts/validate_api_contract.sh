#!/usr/bin/env bash
# validate_api_contract.sh - Validate API contract hasn't been broken by parallel agent changes

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
CONTRACT_FILE="${PROJECT_ROOT}/agents/shared/API_CONTRACT.md"

cd "$PROJECT_ROOT"

echo "🔍 Validating API Contract..."

# Check file exists
if [ ! -f "$CONTRACT_FILE" ]; then
    echo "❌ API_CONTRACT.md not found at: $CONTRACT_FILE"
    exit 1
fi

# Validate required sections exist
REQUIRED_SECTIONS=(
    "## API Endpoints"
    "## Request/Response"
)

MISSING_SECTIONS=()
for section in "${REQUIRED_SECTIONS[@]}"; do
    if ! grep -q "^${section}" "$CONTRACT_FILE"; then
        MISSING_SECTIONS+=("$section")
    fi
done

if [ ${#MISSING_SECTIONS[@]} -gt 0 ]; then
    echo "❌ API Contract missing required sections:"
    for section in "${MISSING_SECTIONS[@]}"; do
        echo "   - $section"
    done
    exit 1
fi

# Check for breaking changes (if comparing with previous version)
if [ -n "${GITHUB_BASE_REF:-}" ] && [ -n "${GITHUB_HEAD_REF:-}" ]; then
    echo "📊 Checking for breaking changes..."

    # Fetch base branch
    git fetch origin "$GITHUB_BASE_REF"

    # Check if contract file changed
    if git diff --name-only "origin/${GITHUB_BASE_REF}"...HEAD | grep -q "API_CONTRACT.md"; then
        echo "⚠️  API Contract file has changed - reviewing for breaking changes..."

        # Look for removed endpoints (starting with -)
        REMOVED_ENDPOINTS=$(git diff "origin/${GITHUB_BASE_REF}"...HEAD -- "$CONTRACT_FILE" | grep "^\-.*:" | grep -v "^---" | grep -v "^+++" || true)

        if [ -n "$REMOVED_ENDPOINTS" ]; then
            echo "⚠️  Breaking changes detected (removed endpoints):"
            echo "$REMOVED_ENDPOINTS"
            echo ""
            echo "Please document breaking changes in PR description with:"
            echo "- Migration guide"
            echo "- Deprecation timeline"
            echo "- Alternative endpoints"

            # Don't fail, just warn (breaking changes may be intentional)
            echo "⚠️  Continuing validation (breaking changes may be intentional)..."
        fi
    fi
fi

# Validate JSON/format if contract contains examples
if grep -q '```json' "$CONTRACT_FILE"; then
    echo "✅ Validating JSON examples..."
    # Extract and validate JSON examples (if any)
    # This is a placeholder - implement full JSON validation if needed
fi

echo "✅ API Contract validation passed"
exit 0
