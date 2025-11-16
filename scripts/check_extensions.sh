#!/bin/bash
# Check VS Code extension status and provide recommendations
# for workspace vs global extension management

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}VS Code Extension Management Check${NC}"
echo "=================================="
echo ""

# Check if Cursor CLI is available (prefer Cursor over VS Code)
if command -v cursor &> /dev/null; then
  CLI_CMD="cursor"
  IDE_NAME="Cursor"
elif command -v code &> /dev/null; then
  CLI_CMD="code"
  IDE_NAME="VS Code"
else
  echo -e "${RED}Error: Neither Cursor nor VS Code CLI found${NC}"
  echo "Install it: Cmd+Shift+P → 'Shell Command: Install code command in PATH'"
  exit 1
fi

# Get workspace-recommended extensions
WORKSPACE_EXTENSIONS=$(jq -r '.recommendations[]' "$PROJECT_ROOT/.vscode/extensions.json" 2>/dev/null || echo "")

# Get globally installed extensions
GLOBAL_EXTENSIONS=$($CLI_CMD --list-extensions 2>/dev/null || echo "")

# Workspace-only extensions (should be disabled globally)
WORKSPACE_ONLY=(
  "ms-vscode.cpptools"
  "ms-vscode.cmake-tools"
  "ms-python.python"
  "ms-python.vscode-pylance"
  "ms-python.black-formatter"
  "rust-lang.rust-analyzer"
  "dbaeumer.vscode-eslint"
  "timonwong.shellcheck"
  "sswg.swift-lang"
  "yutengjing.vscode-mcp"
  "prompttower.prompttower"
)

# Safe global extensions
GLOBAL_SAFE=(
  "editorconfig.editorconfig"
  "redhat.vscode-yaml"
  "eamodio.gitlens"
  "yzhang.markdown-all-in-one"
  "davidanson.vscode-markdownlint"
  "streetsidesoftware.code-spell-checker"
  "usernamehw.errorlens"
)

echo -e "${YELLOW}Checking workspace-only extensions...${NC}"
echo ""

WORKSPACE_ISSUES=0
for ext in "${WORKSPACE_ONLY[@]}"; do
  if echo "$GLOBAL_EXTENSIONS" | grep -q "^${ext}$"; then
    echo -e "${RED}⚠️  ${ext}${NC}"
    echo -e "   ${YELLOW}Found globally enabled - should be workspace-only${NC}"
    echo -e "   ${BLUE}Action: Disable globally, enable in workspace only${NC}"
    WORKSPACE_ISSUES=$((WORKSPACE_ISSUES + 1))
  else
    echo -e "${GREEN}✓${NC} ${ext}"
  fi
done

echo ""
echo -e "${YELLOW}Checking safe global extensions...${NC}"
echo ""

GLOBAL_OK=0
for ext in "${GLOBAL_SAFE[@]}"; do
  if echo "$GLOBAL_EXTENSIONS" | grep -q "^${ext}$"; then
    echo -e "${GREEN}✓${NC} ${ext} (global is OK)"
    GLOBAL_OK=$((GLOBAL_OK + 1))
  else
    echo -e "${BLUE}○${NC} ${ext} (not installed, optional)"
  fi
done

echo ""
echo "=================================="
echo -e "${BLUE}Summary${NC}"
echo "=================================="

if [ $WORKSPACE_ISSUES -eq 0 ]; then
  echo -e "${GREEN}✓ All workspace-only extensions are properly configured${NC}"
else
  echo -e "${YELLOW}⚠️  Found ${WORKSPACE_ISSUES} workspace-only extension(s) enabled globally${NC}"
  echo ""
  echo "To fix:"
  echo "1. Open VS Code Extensions view (Cmd+Shift+X)"
  echo "2. For each extension listed above:"
  echo "   - Click the gear icon"
  echo "   - Select 'Disable (Workspace)' or 'Disable' globally"
  echo "   - Re-enable only in this workspace"
  echo ""
  echo "See docs/EXTENSION_MANAGEMENT.md for detailed guidance"
fi

echo ""
echo -e "${GREEN}✓ ${GLOBAL_OK} safe global extension(s) found${NC}"

# Check for other potentially conflicting extensions
echo ""
echo -e "${YELLOW}Checking for other potentially conflicting extensions...${NC}"

UNWANTED=(
  "golang.go"
  "vscjava.vscode-java-pack"
  "ms-dotnettools.csharp"
  "dart-code.dart"
  # Enterprise/Mainframe extensions
  "halcyontechltd.code-for-ibmi"
  "codefori.vscode-ibmi"
  "ibm.zopeneditor"
  "broadcomMFD.code4z"
  "broadcomMFD.cobol-language-support"
  "bitlang.cobol"
  "MicroFocus.cobol"
  "rocketsoftware.rocket-enterprise-extension"
)

CONFLICTS=0
for ext in "${UNWANTED[@]}"; do
  if echo "$GLOBAL_EXTENSIONS" | grep -q "^${ext}$"; then
    echo -e "${YELLOW}⚠️  ${ext}${NC} (not used in this project)"
    CONFLICTS=$((CONFLICTS + 1))
  fi
done

if [ $CONFLICTS -eq 0 ]; then
  echo -e "${GREEN}✓ No unwanted extensions found${NC}"
else
  echo -e "${YELLOW}Consider disabling ${CONFLICTS} unused extension(s) to improve performance${NC}"
fi

echo ""
echo "For more information, see: docs/EXTENSION_MANAGEMENT.md"
