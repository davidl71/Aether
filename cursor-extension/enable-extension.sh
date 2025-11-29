#!/usr/bin/env bash
# Enable Project Management Automation Cursor Extension
# Builds and provides instructions for installing the extension
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
cd "$SCRIPT_DIR"

# Colors for output
if [ -t 1 ] && command -v tput >/dev/null 2>&1; then
  RED=$(tput setaf 1)
  GREEN=$(tput setaf 2)
  YELLOW=$(tput setaf 3)
  BLUE=$(tput setaf 4)
  NC=$(tput sgr0)
else
  RED=''
  GREEN=''
  YELLOW=''
  NC=''
  BLUE=''
fi

echo "${BLUE}Building Project Management Automation Cursor Extension...${NC}"
echo ""

# Check if Node.js is available
if ! command -v node >/dev/null 2>&1; then
  echo "${RED}✗ Node.js is not installed${NC}" >&2
  echo "${YELLOW}  Install Node.js: https://nodejs.org/${NC}" >&2
  exit 1
fi

# Check if npm is available
if ! command -v npm >/dev/null 2>&1; then
  echo "${RED}✗ npm is not installed${NC}" >&2
  exit 1
fi

# Install dependencies
echo "${BLUE}Installing dependencies...${NC}"
if [ ! -d "node_modules" ]; then
  npm install
else
  echo "${GREEN}✓ Dependencies already installed${NC}"
fi
echo ""

# Compile TypeScript
echo "${BLUE}Compiling TypeScript...${NC}"
npm run compile
echo ""

# Package extension
echo "${BLUE}Packaging extension...${NC}"
npm run package
echo ""

# Find VSIX file
VSIX_FILE=$(ls -t *.vsix 2>/dev/null | head -1)

if [ -z "$VSIX_FILE" ]; then
  echo "${RED}✗ VSIX file not found${NC}" >&2
  exit 1
fi

VSIX_PATH="$SCRIPT_DIR/$VSIX_FILE"
echo "${GREEN}✓ Extension packaged: ${VSIX_FILE}${NC}"
echo ""

# Add to extensions.json if not already there
EXTENSIONS_JSON="$(cd "$SCRIPT_DIR/.." && pwd)/.vscode/extensions.json"
if [ -f "$EXTENSIONS_JSON" ]; then
  # Check if extension is already in recommendations
  if ! grep -q "project-management-automation" "$EXTENSIONS_JSON" 2>/dev/null; then
    echo "${BLUE}Adding extension to workspace recommendations...${NC}"
    # This is a simple approach - for proper JSON editing, you might want to use jq
    echo "${YELLOW}  Note: You may need to manually add to .vscode/extensions.json${NC}"
  else
    echo "${GREEN}✓ Extension already in workspace recommendations${NC}"
  fi
fi

echo ""
echo "${GREEN}✓ Extension build complete!${NC}"
echo ""
echo "${BLUE}To install in Cursor:${NC}"
echo ""
echo "1. Open Cursor"
echo "2. Press ${YELLOW}Cmd+Shift+P${NC} (Mac) or ${YELLOW}Ctrl+Shift+P${NC} (Windows/Linux)"
echo "3. Type: ${YELLOW}Extensions: Install from VSIX...${NC}"
echo "4. Select: ${YELLOW}${VSIX_PATH}${NC}"
echo ""
echo "${BLUE}Or install via command line:${NC}"
echo "  ${YELLOW}code --install-extension ${VSIX_PATH}${NC}"
echo ""
echo "${BLUE}After installation:${NC}"
echo "- Reload Cursor window: ${YELLOW}Cmd+Shift+P${NC} → ${YELLOW}Developer: Reload Window${NC}"
echo "- Check status bar for automation status indicator"
echo "- Test commands: ${YELLOW}Cmd+Shift+P${NC} → Type ${YELLOW}Project Automation${NC}"
