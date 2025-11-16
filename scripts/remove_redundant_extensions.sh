#!/bin/bash
# Script to remove redundant extensions based on redundancy analysis
# Includes safety checks and confirmation prompts

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
MAGENTA='\033[0;35m'
NC='\033[0m'

echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}  Remove Redundant Extensions${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}"
echo ""

# Check if Cursor CLI is available
if command -v cursor &> /dev/null; then
  CLI_CMD="cursor"
  IDE_NAME="Cursor"
elif command -v code &> /dev/null; then
  CLI_CMD="code"
  IDE_NAME="VS Code"
else
  echo -e "${RED}Error: Neither Cursor nor VS Code CLI found${NC}"
  exit 1
fi

# Get all installed extensions
ALL_EXTENSIONS=$($CLI_CMD --list-extensions 2>/dev/null | sort)

# Define redundant extensions by priority
declare -a HIGH_PRIORITY=(
  # C++ conflict - remove Microsoft version, keep Cursor's
  "ms-vscode.cpptools"
  
  # Rust legacy - redundant with rust-analyzer
  "rust-lang.rust"
  
  # Turbo - keep vercel, remove other
  "syntaxsyndicate.turbo-vsc"
  
  # Disabled extensions - can safely remove
  "ms-azuretools.vscode-containers"
  "ms-azuretools.vscode-docker"
  "ms-kubernetes-tools.vscode-kubernetes-tools"
  "golang.go"
  "neonxp.gotools"
  "shivamkumar.go-extras"
)

declare -a MEDIUM_PRIORITY=(
  # Documentation - keep one (removing doxygen, keeping doxdocgen)
  "bbenoist.doxygen"
  
  # Extension packs - may include individual extensions
  "pinage404.rust-extension-pack"
  "franneck94.vscode-c-cpp-dev-extension-pack"
  
  # C++ tools - optional helpers
  "llvm-vs-code-extensions.vscode-clangd"
  "franneck94.c-cpp-runner"
  "franneck94.vscode-c-cpp-config"
  "jbenden.c-cpp-flylint"
)

declare -a AI_ASSISTANTS=(
  "anthropic.claude-code"
  "amazonwebservices.amazon-q-vscode"
  "google.gemini-cli-vscode-ide-companion"
  "google.geminicodeassist"
  "openai.chatgpt"
  "fridaplatform.fridagpt"
)

declare -a MCP_EXTENSIONS=(
  "cjl.lsp-mcp"
  "daninemonic.mcp4humans"
  "interactive-mcp.interactive-mcp"
  "kirigaya.openmcp"
  "pimzino.agentic-tools-mcp-companion"
  "raz-labs.interactive-mcp"
)

# Function to check if extension is installed
is_installed() {
  local ext="$1"
  echo "$ALL_EXTENSIONS" | grep -q "^${ext}$"
}

# Function to remove extension
remove_extension() {
  local ext="$1"
  local reason="$2"
  
  echo -n "  Removing $ext... "
  if $CLI_CMD --uninstall-extension "$ext" 2>&1 | grep -v "ERROR:electron" > /dev/null; then
    echo -e "${GREEN}✓${NC}"
    return 0
  else
    echo -e "${YELLOW}⚠${NC} (may not be installed or has dependencies)"
    return 1
  fi
}

# Function to process extension list
process_extensions() {
  local category="$1"
  shift
  local extensions=("$@")
  local to_remove=()
  
  echo -e "${CYAN}Checking ${category}...${NC}"
  
  for ext in "${extensions[@]}"; do
    if is_installed "$ext"; then
      to_remove+=("$ext")
    fi
  done
  
  if [ ${#to_remove[@]} -eq 0 ]; then
    echo -e "  ${GREEN}✓${NC} No extensions to remove"
    echo ""
    return 0
  fi
  
  echo -e "  ${YELLOW}Found ${#to_remove[@]} extension(s) to remove:${NC}"
  for ext in "${to_remove[@]}"; do
    echo -e "    ${RED}•${NC} $ext"
  done
  
  return ${#to_remove[@]}
}

# Main removal function
remove_category() {
  local category="$1"
  shift
  local extensions=("$@")
  local to_remove=()
  
  for ext in "${extensions[@]}"; do
    if is_installed "$ext"; then
      to_remove+=("$ext")
    fi
  done
  
  if [ ${#to_remove[@]} -eq 0 ]; then
    return 0
  fi
  
  echo ""
  echo -e "${YELLOW}Removing ${category} (${#to_remove[@]} extension(s))...${NC}"
  
  local removed=0
  local failed=0
  
  for ext in "${to_remove[@]}"; do
    if remove_extension "$ext" ""; then
      removed=$((removed + 1))
    else
      failed=$((failed + 1))
    fi
  done
  
  echo -e "  ${GREEN}Removed: ${removed}${NC}"
  if [ $failed -gt 0 ]; then
    echo -e "  ${YELLOW}Failed: ${failed}${NC}"
  fi
  
  return $removed
}

# Show summary first
echo -e "${CYAN}Analyzing installed extensions...${NC}"
echo ""

TOTAL_REMOVABLE=0

# Count removable extensions
for ext in "${HIGH_PRIORITY[@]}" "${MEDIUM_PRIORITY[@]}"; do
  if is_installed "$ext"; then
    TOTAL_REMOVABLE=$((TOTAL_REMOVABLE + 1))
  fi
done

if [ $TOTAL_REMOVABLE -eq 0 ]; then
  echo -e "${GREEN}✓ No high/medium priority redundant extensions found${NC}"
  echo ""
  echo "You may still want to review:"
  echo "  - AI Assistants (8 extensions)"
  echo "  - MCP Extensions (7 extensions)"
  exit 0
fi

echo -e "${YELLOW}Found ${TOTAL_REMOVABLE} redundant extension(s) to remove${NC}"
echo ""

# Show what will be removed
echo -e "${BLUE}High Priority Removals:${NC}"
HIGH_COUNT=0
for ext in "${HIGH_PRIORITY[@]}"; do
  if is_installed "$ext"; then
    echo -e "  ${RED}•${NC} $ext"
    HIGH_COUNT=$((HIGH_COUNT + 1))
  fi
done

echo ""
echo -e "${BLUE}Medium Priority Removals:${NC}"
MEDIUM_COUNT=0
for ext in "${MEDIUM_PRIORITY[@]}"; do
  if is_installed "$ext"; then
    echo -e "  ${YELLOW}•${NC} $ext"
    MEDIUM_COUNT=$((MEDIUM_COUNT + 1))
  fi
done

echo ""
echo -e "${MAGENTA}AI Assistants (${#AI_ASSISTANTS[@]} available - manual review recommended):${NC}"
AI_COUNT=0
for ext in "${AI_ASSISTANTS[@]}"; do
  if is_installed "$ext"; then
    echo -e "  ${CYAN}•${NC} $ext"
    AI_COUNT=$((AI_COUNT + 1))
  fi
done

echo ""
echo -e "${MAGENTA}MCP Extensions (${#MCP_EXTENSIONS[@]} available - manual review recommended):${NC}"
MCP_COUNT=0
for ext in "${MCP_EXTENSIONS[@]}"; do
  if is_installed "$ext"; then
    echo -e "  ${CYAN}•${NC} $ext"
    MCP_COUNT=$((MCP_COUNT + 1))
  fi
done

echo ""
echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}  Removal Options${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}"
echo ""
echo "1. Remove HIGH priority only (${HIGH_COUNT} extensions)"
echo "2. Remove HIGH + MEDIUM priority (${HIGH_COUNT} + ${MEDIUM_COUNT} = $((HIGH_COUNT + MEDIUM_COUNT)) extensions)"
echo "3. Custom selection (interactive)"
echo "4. Show details only (no removal)"
echo "5. Exit"
echo ""
read -p "Select option [1-5]: " choice

case $choice in
  1)
    echo ""
    echo -e "${YELLOW}Removing HIGH priority extensions...${NC}"
    remove_category "High Priority" "${HIGH_PRIORITY[@]}"
    ;;
  2)
    echo ""
    echo -e "${YELLOW}Removing HIGH + MEDIUM priority extensions...${NC}"
    remove_category "High Priority" "${HIGH_PRIORITY[@]}"
    remove_category "Medium Priority" "${MEDIUM_PRIORITY[@]}"
    ;;
  3)
    echo ""
    echo -e "${CYAN}Custom Selection Mode${NC}"
    echo ""
    echo "High Priority extensions:"
    for i in "${!HIGH_PRIORITY[@]}"; do
      ext="${HIGH_PRIORITY[$i]}"
      if is_installed "$ext"; then
        echo "  [$((i+1))] $ext"
      fi
    done
    echo ""
    echo "Medium Priority extensions:"
    for i in "${!MEDIUM_PRIORITY[@]}"; do
      ext="${MEDIUM_PRIORITY[$i]}"
      if is_installed "$ext"; then
        echo "  [$((i+${#HIGH_PRIORITY[@]}+1))] $ext"
      fi
    done
    echo ""
    read -p "Enter extension numbers to remove (comma-separated, e.g., 1,3,5): " selections
    # Parse and remove selected extensions
    IFS=',' read -ra SELECTED <<< "$selections"
    for num in "${SELECTED[@]}"; do
      num=$((num - 1))
      if [ $num -lt ${#HIGH_PRIORITY[@]} ]; then
        ext="${HIGH_PRIORITY[$num]}"
        if is_installed "$ext"; then
          remove_extension "$ext" ""
        fi
      elif [ $num -lt $((${#HIGH_PRIORITY[@]} + ${#MEDIUM_PRIORITY[@]})) ]; then
        ext="${MEDIUM_PRIORITY[$((num - ${#HIGH_PRIORITY[@]}))]}"
        if is_installed "$ext"; then
          remove_extension "$ext" ""
        fi
      fi
    done
    ;;
  4)
    echo ""
    echo -e "${GREEN}Details shown above. No extensions removed.${NC}"
    exit 0
    ;;
  5)
    echo ""
    echo -e "${GREEN}Exiting without changes.${NC}"
    exit 0
    ;;
  *)
    echo -e "${RED}Invalid option. Exiting.${NC}"
    exit 1
    ;;
esac

echo ""
echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}  Summary${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}"
echo ""

# Verify final count
FINAL_COUNT=$($CLI_CMD --list-extensions 2>/dev/null | wc -l | tr -d ' ')
echo -e "Extensions remaining: ${CYAN}${FINAL_COUNT}${NC}"
echo ""

if [ $AI_COUNT -gt 0 ] || [ $MCP_COUNT -gt 0 ]; then
  echo -e "${YELLOW}Note:${NC} Consider manually reviewing:"
  if [ $AI_COUNT -gt 0 ]; then
    echo "  - AI Assistants (${AI_COUNT} installed) - keep 1-2 favorites"
  fi
  if [ $MCP_COUNT -gt 0 ]; then
    echo "  - MCP Extensions (${MCP_COUNT} installed) - keep 1-2 you use"
  fi
  echo ""
fi

echo -e "${GREEN}✓ Redundancy removal complete!${NC}"
echo ""
echo "Run ${CYAN}./scripts/check_extension_redundancy.sh${NC} to verify"

