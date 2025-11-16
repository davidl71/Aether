#!/bin/bash
# Cleanup remaining extensions: AI assistants, MCP, and disabled language extensions

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
echo -e "${BLUE}  Cleanup Remaining Extensions${NC}"
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

# Get all installed extensions
ALL_EXTENSIONS=$($CLI_CMD --list-extensions 2>/dev/null | sort)

# Step 1: AI Assistants
declare -a AI_ASSISTANTS=(
  "anthropic.claude-code"
  "amazonwebservices.amazon-q-vscode"
  "amazonwebservices.codewhisperer-for-command-line-companion"
  "google.gemini-cli-vscode-ide-companion"
  "google.geminicodeassist"
  "openai.chatgpt"
  "fridaplatform.fridagpt"
  "continue.continue"
)

# Step 2: MCP Extensions
declare -a MCP_EXTENSIONS=(
  "yutengjing.vscode-mcp"
  "yutengjing.vscode-mcp-bridge"
  "cjl.lsp-mcp"
  "daninemonic.mcp4humans"
  "interactive-mcp.interactive-mcp"
  "kirigaya.openmcp"
  "pimzino.agentic-tools-mcp-companion"
  "raz-labs.interactive-mcp"
)

# Step 3: Disabled Language Extensions
declare -a DISABLED_LANGUAGES=(
  "redhat.java"
  "shopify.ruby-lsp"
)

# Function to check if extension is installed
is_installed() {
  local ext="$1"
  echo "$ALL_EXTENSIONS" | grep -q "^${ext}$"
}

# Function to remove extension
remove_extension() {
  local ext="$1"
  echo -n "  Removing $ext... "
  if $CLI_CMD --uninstall-extension "$ext" 2>&1 | grep -v "ERROR:electron" > /dev/null; then
    echo -e "${GREEN}✓${NC}"
    return 0
  else
    echo -e "${YELLOW}⚠${NC} (may not be installed or has dependencies)"
    return 1
  fi
}

# Step 1: AI Assistants
echo -e "${CYAN}Step 1: AI Assistants${NC}"
echo ""

AI_FOUND=()
for ext in "${AI_ASSISTANTS[@]}"; do
  if is_installed "$ext"; then
    AI_FOUND+=("$ext")
  fi
done

if [ ${#AI_FOUND[@]} -eq 0 ]; then
  echo -e "  ${GREEN}✓${NC} No AI assistants found"
else
  echo -e "  ${YELLOW}Found ${#AI_FOUND[@]} AI assistant(s):${NC}"
  for ext in "${AI_FOUND[@]}"; do
    echo -e "    ${CYAN}•${NC} $ext"
  done
  echo ""
  echo -e "  ${BLUE}Recommendation:${NC} Keep 1-2 favorites (e.g., continue.continue + one other)"
  echo ""
  read -p "  Remove all except continue.continue? [y/N]: " remove_ai
  if [[ "$remove_ai" =~ ^[Yy]$ ]]; then
    echo ""
    for ext in "${AI_FOUND[@]}"; do
      if [ "$ext" != "continue.continue" ]; then
        remove_extension "$ext"
      fi
    done
  else
    echo -e "  ${YELLOW}⚠${NC} Skipping AI assistant removal"
  fi
fi

echo ""

# Step 2: MCP Extensions
echo -e "${CYAN}Step 2: MCP Extensions${NC}"
echo ""

MCP_FOUND=()
for ext in "${MCP_EXTENSIONS[@]}"; do
  if is_installed "$ext"; then
    MCP_FOUND+=("$ext")
  fi
done

if [ ${#MCP_FOUND[@]} -eq 0 ]; then
  echo -e "  ${GREEN}✓${NC} No MCP extensions found"
else
  echo -e "  ${YELLOW}Found ${#MCP_FOUND[@]} MCP extension(s):${NC}"
  for ext in "${MCP_FOUND[@]}"; do
    echo -e "    ${CYAN}•${NC} $ext"
  done
  echo ""
  echo -e "  ${BLUE}Recommendation:${NC} Keep 1-2 that you actually use"
  echo ""
  read -p "  Remove all except yutengjing.vscode-mcp-bridge? [y/N]: " remove_mcp
  if [[ "$remove_mcp" =~ ^[Yy]$ ]]; then
    echo ""
    for ext in "${MCP_FOUND[@]}"; do
      if [ "$ext" != "yutengjing.vscode-mcp-bridge" ]; then
        remove_extension "$ext"
      fi
    done
  else
    echo -e "  ${YELLOW}⚠${NC} Skipping MCP extension removal"
  fi
fi

echo ""

# Step 3: Disabled Language Extensions
echo -e "${CYAN}Step 3: Disabled Language Extensions${NC}"
echo ""

DISABLED_FOUND=()
for ext in "${DISABLED_LANGUAGES[@]}"; do
  if is_installed "$ext"; then
    DISABLED_FOUND+=("$ext")
  fi
done

if [ ${#DISABLED_FOUND[@]} -eq 0 ]; then
  echo -e "  ${GREEN}✓${NC} No disabled language extensions found"
else
  echo -e "  ${YELLOW}Found ${#DISABLED_FOUND[@]} disabled extension(s):${NC}"
  for ext in "${DISABLED_FOUND[@]}"; do
    echo -e "    ${RED}•${NC} $ext (disabled, not used in project)"
  done
  echo ""
  read -p "  Remove disabled language extensions? [Y/n]: " remove_disabled
  if [[ ! "$remove_disabled" =~ ^[Nn]$ ]]; then
    echo ""
    for ext in "${DISABLED_FOUND[@]}"; do
      remove_extension "$ext"
    done
  else
    echo -e "  ${YELLOW}⚠${NC} Skipping disabled extension removal"
  fi
fi

echo ""
echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}  Summary${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}"
echo ""

FINAL_COUNT=$($CLI_CMD --list-extensions 2>/dev/null | wc -l | tr -d ' ')
echo -e "Extensions remaining: ${CYAN}${FINAL_COUNT}${NC}"
echo ""
echo -e "${GREEN}✓ Cleanup complete!${NC}"
