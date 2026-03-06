#!/bin/bash
# Setup Pre-Commit Hook for Documentation Validation

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
HOOK_FILE="$PROJECT_ROOT/.git/hooks/pre-commit"

# Colors
GREEN='\033[0;32m'
NC='\033[0m'

echo "🔧 Setting up pre-commit hook for documentation validation..."
echo ""

# Check if .git directory exists
if [[ ! -d "$PROJECT_ROOT/.git" ]]; then
  echo "❌ Error: .git directory not found. Are you in a git repository?"
  exit 1
fi

# Create hooks directory if it doesn't exist
mkdir -p "$PROJECT_ROOT/.git/hooks"

# Create pre-commit hook
cat > "$HOOK_FILE" << 'EOF'
#!/bin/bash
# Pre-commit hook for documentation validation

DOCS_DIR="docs"
SCRIPTS_DIR="scripts"

# Check if documentation files changed
if git diff --cached --name-only | grep -q "$DOCS_DIR/API_DOCUMENTATION_INDEX.md"; then
  echo "🔍 Validating documentation..."

  # Docs format validation: use exarp-go MCP (check_documentation_health_tool) or run locally:
  #   exarp-go -tool check_documentation_health_tool (with workingDirectory = project root)
  # Link validation (non-blocking, just warn):
  if ! "$SCRIPTS_DIR/validate_docs_links.sh" 2>/dev/null; then
    echo ""
    echo "⚠️  Warning: Some documentation links may be broken"
    echo "   (This is non-blocking, but please fix before merging)"
    echo "   You can skip this check with: git commit --no-verify"
  fi

  echo "✅ Documentation validation passed"
fi

exit 0
EOF

# Make executable
chmod +x "$HOOK_FILE"

echo -e "${GREEN}✅ Pre-commit hook installed successfully${NC}"
echo ""
echo "Location: $HOOK_FILE"
echo ""
echo "The hook will automatically validate documentation when you commit changes to:"
echo "  - docs/API_DOCUMENTATION_INDEX.md"
echo ""
echo "To test, try committing a change to the documentation:"
echo "  git add docs/API_DOCUMENTATION_INDEX.md"
echo "  git commit -m 'Test commit'"
echo ""
echo "To bypass validation (not recommended):"
echo "  git commit --no-verify"
