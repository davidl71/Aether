#!/bin/bash
# Quick check of unwanted extensions from Cursor

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Check if Cursor CLI is available
if command -v cursor &> /dev/null; then
  CLI_CMD="cursor"
elif command -v code &> /dev/null; then
  CLI_CMD="code"
else
  echo "Error: Neither Cursor nor VS Code CLI found"
  exit 1
fi

# Get unwanted extensions from JSON
UNWANTED=$(jq -r '.unwantedRecommendations[]' "$PROJECT_ROOT/.vscode/extensions.json" 2>/dev/null)

# Get installed extensions
INSTALLED=$($CLI_CMD --list-extensions 2>/dev/null | sort)

echo "Checking for unwanted extensions..."
echo ""

FOUND=0
while IFS= read -r unwanted; do
  if echo "$INSTALLED" | grep -q "^${unwanted}$"; then
    echo "⚠️  $unwanted"
    FOUND=$((FOUND + 1))
  fi
done <<< "$UNWANTED"

if [ $FOUND -eq 0 ]; then
  echo "✓ No unwanted extensions found"
else
  echo ""
  echo "Found $FOUND unwanted extension(s)"
  echo "Action: Disable or uninstall these extensions"
fi
