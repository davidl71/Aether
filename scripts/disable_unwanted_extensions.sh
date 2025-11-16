#!/bin/bash
# Script to disable unwanted extensions in Cursor using CLI

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m'

echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}  Disabling Unwanted Extensions${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}"
echo ""

# Check if Cursor CLI is available
if command -v cursor &> /dev/null; then
  CLI_CMD="cursor"
elif command -v code &> /dev/null; then
  CLI_CMD="code"
else
  echo -e "${RED}Error: Neither Cursor nor VS Code CLI found${NC}"
  exit 1
fi

# Get unwanted extensions from JSON
UNWANTED=$(jq -r '.unwantedRecommendations[]' "$PROJECT_ROOT/.vscode/extensions.json" 2>/dev/null | grep -v "^//" | grep -v "^$")

# Get installed extensions
INSTALLED=$($CLI_CMD --list-extensions 2>/dev/null | sort)

echo -e "${CYAN}Checking for unwanted extensions to disable...${NC}"
echo ""

# Find unwanted extensions that are installed
TO_DISABLE=()
while IFS= read -r unwanted; do
  [ -z "$unwanted" ] && continue
  # Skip comments
  [[ "$unwanted" =~ ^[[:space:]]*// ]] && continue
  if echo "$INSTALLED" | grep -q "^${unwanted}$"; then
    TO_DISABLE+=("$unwanted")
  fi
done <<< "$UNWANTED"

if [ ${#TO_DISABLE[@]} -eq 0 ]; then
  echo -e "${GREEN}✓ No unwanted extensions found to disable${NC}"
  exit 0
fi

echo -e "${YELLOW}Found ${#TO_DISABLE[@]} unwanted extension(s) to disable:${NC}"
echo ""

for ext in "${TO_DISABLE[@]}"; do
  echo -e "  ${RED}•${NC} $ext"
done

echo ""
echo -e "${CYAN}Disabling extensions...${NC}"
echo ""

# Disable each extension
DISABLED=0
FAILED=0

for ext in "${TO_DISABLE[@]}"; do
  echo -n "Disabling $ext... "
  if $CLI_CMD --disable-extension "$ext" 2>/dev/null; then
    echo -e "${GREEN}✓${NC}"
    DISABLED=$((DISABLED + 1))
  else
    echo -e "${RED}✗${NC} (may need to uninstall instead)"
    FAILED=$((FAILED + 1))
  fi
done

echo ""
echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}  Results${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}"
echo ""
echo -e "${GREEN}✓ Disabled: ${DISABLED}${NC}"
if [ $FAILED -gt 0 ]; then
  echo -e "${YELLOW}⚠ Failed: ${FAILED}${NC}"
  echo ""
  echo "Some extensions may need to be uninstalled instead of disabled."
  echo "You can uninstall them manually in Cursor Extensions view (Cmd+Shift+X)"
fi

echo ""
echo -e "${CYAN}Verifying...${NC}"
echo ""

# Verify by checking again
REMAINING=$($CLI_CMD --list-extensions 2>/dev/null | grep -E "^($(IFS='|'; echo "${TO_DISABLE[*]}"))$" | wc -l | tr -d ' ')

if [ "$REMAINING" -eq 0 ]; then
  echo -e "${GREEN}✓ All unwanted extensions have been disabled!${NC}"
else
  echo -e "${YELLOW}⚠ ${REMAINING} extension(s) are still enabled${NC}"
  echo "   They may need to be uninstalled instead of disabled"
fi

echo ""
echo "Run ${CYAN}./scripts/quick_extension_check.sh${NC} to verify"
