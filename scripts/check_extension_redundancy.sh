#!/bin/bash
# Check installed extensions for redundancy, duplicates, and overlapping functionality

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Colors
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
GREEN='\033[0;32m'
NC='\033[0m'

echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}  Extension Redundancy Analysis${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}"
echo ""

# Parse optional --ide override
CLI_CMD_AUTO=""
while [[ $# -gt 0 ]]; do
  case "$1" in
    --ide)
      case "${2:-}" in
        cursor) CLI_CMD_AUTO="cursor" ;;
        code) CLI_CMD_AUTO="code" ;;
        *) echo -e "${RED}--ide must be 'cursor' or 'code'${NC}"; exit 2 ;;
      esac; shift 2 ;;
    *) echo -e "${YELLOW}Ignoring unknown arg:${NC} $1"; shift ;;
  esac
done

# Detect CLI, honor --ide if provided
if [[ -n "$CLI_CMD_AUTO" ]]; then
  CLI_CMD="$CLI_CMD_AUTO"
else
  if command -v cursor &> /dev/null; then
    CLI_CMD="cursor"
  elif command -v code &> /dev/null; then
    CLI_CMD="code"
  else
    echo -e "${RED}Error: Neither Cursor nor VS Code CLI found${NC}"
    exit 1
  fi
fi

# Get all installed extensions
ALL_EXTENSIONS=$($CLI_CMD --list-extensions 2>/dev/null | sort)

echo -e "${CYAN}Analyzing extensions for redundancy...${NC}"
echo ""

# Define extension groups that might have redundancy (using functions instead of associative arrays)
check_group() {
  local group_name="$1"
  shift
  local group_exts=("$@")
  local found_exts=()

  for ext in "${group_exts[@]}"; do
    if echo "$ALL_EXTENSIONS" | grep -q "^${ext}$"; then
      found_exts+=("$ext")
    fi
  done

  if [ ${#found_exts[@]} -gt 1 ]; then
    echo -e "${YELLOW}⚠️  ${group_name} (${#found_exts[@]} extensions)${NC}"
    for ext in "${found_exts[@]}"; do
      echo -e "   ${CYAN}•${NC} $ext"
    done
    echo ""
    return ${#found_exts[@]}
  fi
  return 0
}

# Check extensions by category using CLI
echo -e "${CYAN}Checking extensions by category...${NC}"
echo ""

CATEGORY_REDUNDANCIES=0

# Get valid categories (use lowercase as per CLI output)
VALID_CATEGORIES=("formatters" "linters" "debuggers" "themes")

# Check for category-based redundancies
for category in "${VALID_CATEGORIES[@]}"; do
  category_exts=$($CLI_CMD --list-extensions --category "$category" 2>/dev/null | sort || echo "")
  if [ -n "$category_exts" ]; then
    count=$(echo "$category_exts" | grep -c . || echo "0")
    if [ "$count" -gt 1 ]; then
      echo -e "${YELLOW}⚠️  ${category} Category (${count} extensions)${NC}"
      while IFS= read -r ext; do
        [ -z "$ext" ] && continue
        echo -e "   ${CYAN}•${NC} $ext"
      done <<< "$category_exts"

      # Capitalize first letter for display
      category_display=$(echo "$category" | sed 's/^./\U&/')

      case "$category" in
        "formatters")
          echo -e "   ${BLUE}Recommendation:${NC} Review if multiple formatters conflict"
          echo -e "   ${BLUE}Note:${NC} Language-specific formatters (Black, ESLint) are usually fine"
          echo -e "   ${BLUE}Action:${NC} Ensure formatters don't conflict (e.g., Prettier vs ESLint)"
          ;;
        "linters")
          echo -e "   ${BLUE}Recommendation:${NC} Multiple linters are usually fine (different languages)"
          echo -e "   ${BLUE}Note:${NC} ESLint (JS/TS), ShellCheck (shell), markdownlint (markdown) are complementary"
          ;;
        "debuggers")
          echo -e "   ${BLUE}Recommendation:${NC} Different debuggers for different targets"
          echo -e "   ${BLUE}Note:${NC} Python debugger, C++ debugger, browser debuggers are complementary"
          ;;
        "themes")
          echo -e "   ${BLUE}Recommendation:${NC} Multiple themes are fine - they don't conflict"
          ;;
      esac
      echo ""
      CATEGORY_REDUNDANCIES=$((CATEGORY_REDUNDANCIES + 1))
    fi
  fi
done

# Check for duplicates within groups
REDUNDANCIES_FOUND=0

# C++ Tools
if check_group "C++ Tools" "ms-vscode.cpptools" "anysphere.cpptools" "llvm-vs-code-extensions.vscode-clangd" "franneck94.c-cpp-runner" "franneck94.vscode-c-cpp-config" "franneck94.vscode-c-cpp-dev-extension-pack" "jbenden.c-cpp-flylint" "vadimcn.vscode-lldb"; then
  echo -e "   ${BLUE}Recommendation:${NC} Keep ${GREEN}anysphere.cpptools${NC} (Cursor's version) or ${GREEN}ms-vscode.cpptools${NC}, remove others"
  echo ""
  REDUNDANCIES_FOUND=$((REDUNDANCIES_FOUND + 1))
fi

# Python Tools
if check_group "Python Tools" "ms-python.python" "ms-python.vscode-pylance" "ms-python.black-formatter" "ms-python.debugpy" "anysphere.cursorpyright" "guyskk.language-cython"; then
  echo -e "   ${BLUE}Recommendation:${NC} Keep ${GREEN}ms-python.python${NC} + ${GREEN}ms-python.vscode-pylance${NC}, ${GREEN}anysphere.cursorpyright${NC} is Cursor's version"
  echo ""
  REDUNDANCIES_FOUND=$((REDUNDANCIES_FOUND + 1))
fi

# CMake Tools
if check_group "CMake Tools" "ms-vscode.cmake-tools" "cheshirekow.cmake-format" "kylinideteam.cmake-intellisence" "twxs.cmake"; then
  echo -e "   ${BLUE}Recommendation:${NC} Keep ${GREEN}ms-vscode.cmake-tools${NC} (main tool), others are optional helpers"
  echo ""
  REDUNDANCIES_FOUND=$((REDUNDANCIES_FOUND + 1))
fi

# Rust Tools
if check_group "Rust Tools" "rust-lang.rust-analyzer" "rust-lang.rust" "pinage404.rust-extension-pack" "serayuzgur.crates" "washan.cargo-appraiser"; then
  echo -e "   ${BLUE}Recommendation:${NC} Keep ${GREEN}rust-lang.rust-analyzer${NC} (essential), others are optional"
  echo ""
  REDUNDANCIES_FOUND=$((REDUNDANCIES_FOUND + 1))
fi

# MCP Extensions
if check_group "MCP Extensions" "yutengjing.vscode-mcp" "yutengjing.vscode-mcp-bridge" "cjl.lsp-mcp" "daninemonic.mcp4humans" "interactive-mcp.interactive-mcp" "kirigaya.openmcp" "pimzino.agentic-tools-mcp-companion" "raz-labs.interactive-mcp"; then
  echo -e "   ${BLUE}Recommendation:${NC} Review which MCP extensions you actually use - many overlap"
  echo ""
  REDUNDANCIES_FOUND=$((REDUNDANCIES_FOUND + 1))
fi

# AI Assistants
if check_group "AI Assistants" "github.copilot" "github.copilot-chat" "anthropic.claude-code" "amazonwebservices.codewhisperer-for-command-line-companion" "amazonwebservices.amazon-q-vscode" "google.gemini-cli-vscode-ide-companion" "google.geminicodeassist" "openai.chatgpt" "fridaplatform.fridagpt" "continue.continue"; then
  echo -e "   ${BLUE}Recommendation:${NC} You have 8+ AI assistants - consider keeping 1-2 favorites"
  echo ""
  REDUNDANCIES_FOUND=$((REDUNDANCIES_FOUND + 1))
fi

# Prompt Tools
if check_group "Prompt Tools" "backnotprop.prompt-tower" "prompttower.prompttower" "pascalx.sketchprompt"; then
  echo -e "   ${BLUE}Recommendation:${NC} Multiple prompt tools - keep one that works best for you"
  echo ""
  REDUNDANCIES_FOUND=$((REDUNDANCIES_FOUND + 1))
fi

# Jupyter/Notebook
if check_group "Jupyter/Notebook" "ms-toolsai.jupyter" "ms-toolsai.jupyter-renderers" "ms-toolsai.vscode-jupyter-cell-tags" "ms-toolsai.vscode-jupyter-slideshow"; then
  echo -e "   ${BLUE}Recommendation:${NC} Core Jupyter extension includes renderers - others may be redundant"
  echo ""
  REDUNDANCIES_FOUND=$((REDUNDANCIES_FOUND + 1))
fi

# Docker/Containers
if check_group "Docker/Containers" "ms-azuretools.vscode-containers" "ms-azuretools.vscode-docker" "ms-kubernetes-tools.vscode-kubernetes-tools"; then
  echo -e "   ${BLUE}Recommendation:${NC} All disabled - can be uninstalled if not needed"
  echo ""
  REDUNDANCIES_FOUND=$((REDUNDANCIES_FOUND + 1))
fi

# Ansible
if check_group "Ansible" "redhat.ansible" "mattiasbaake.vscode-snippets-for-ansible" "jborean.ansibug"; then
  echo -e "   ${BLUE}Recommendation:${NC} Keep ${GREEN}redhat.ansible${NC} if needed, others are optional"
  echo ""
  REDUNDANCIES_FOUND=$((REDUNDANCIES_FOUND + 1))
fi

# Go Tools
if check_group "Go Tools" "golang.go" "neonxp.gotools" "shivamkumar.go-extras"; then
  echo -e "   ${BLUE}Recommendation:${NC} All disabled - can be uninstalled (Go not used in project)"
  echo ""
  REDUNDANCIES_FOUND=$((REDUNDANCIES_FOUND + 1))
fi

# Markdown
if check_group "Markdown" "yzhang.markdown-all-in-one" "davidanson.vscode-markdownlint"; then
  echo -e "   ${BLUE}Recommendation:${NC} Both are useful - keep both (one for editing, one for linting)"
  echo ""
  REDUNDANCIES_FOUND=$((REDUNDANCIES_FOUND + 1))
fi

# Debuggers
if check_group "Debuggers" "firefox-devtools.vscode-firefox-debug" "ms-edgedevtools.vscode-edge-devtools" "vadimcn.vscode-lldb"; then
  echo -e "   ${BLUE}Recommendation:${NC} Different debuggers for different targets - keep as needed"
  echo ""
  REDUNDANCIES_FOUND=$((REDUNDANCIES_FOUND + 1))
fi

# Documentation
if check_group "Documentation" "bbenoist.doxygen" "cschlosser.doxdocgen"; then
  echo -e "   ${BLUE}Recommendation:${NC} Both for C++ documentation - keep one that works better"
  echo ""
  REDUNDANCIES_FOUND=$((REDUNDANCIES_FOUND + 1))
fi

# Turbo
if check_group "Turbo" "syntaxsyndicate.turbo-vsc" "vercel.turbo-vsc"; then
  echo -e "   ${BLUE}Recommendation:${NC} Two Turbo extensions - likely redundant, keep one"
  echo ""
  REDUNDANCIES_FOUND=$((REDUNDANCIES_FOUND + 1))
fi

# Check for extension packs that might include individual extensions
echo -e "${CYAN}Checking for extension packs...${NC}"
echo ""

PACK_EXTENSIONS=(
  "franneck94.vscode-c-cpp-dev-extension-pack"
  "pinage404.rust-extension-pack"
)

for pack in "${PACK_EXTENSIONS[@]}"; do
  if echo "$ALL_EXTENSIONS" | grep -q "^${pack}$"; then
    echo -e "${YELLOW}⚠️  Extension Pack Found:${NC} $pack"
    echo -e "   ${BLUE}Note:${NC} Extension packs include multiple extensions"
    echo -e "   ${BLUE}Recommendation:${NC} Check if individual extensions are also installed separately"
    echo ""
  fi
done

# Check for similar functionality
echo -e "${CYAN}Checking for similar functionality...${NC}"
echo ""

# C++ IntelliSense alternatives
if echo "$ALL_EXTENSIONS" | grep -q "ms-vscode.cpptools" && echo "$ALL_EXTENSIONS" | grep -q "anysphere.cpptools"; then
  echo -e "${YELLOW}⚠️  C++ IntelliSense Redundancy${NC}"
  echo -e "   ${CYAN}•${NC} ms-vscode.cpptools (Microsoft)"
  echo -e "   ${CYAN}•${NC} anysphere.cpptools (Cursor's version)"
  echo -e "   ${BLUE}Recommendation:${NC} Keep ${GREEN}anysphere.cpptools${NC} (Cursor-optimized) or ${GREEN}ms-vscode.cpptools${NC}, not both"
  echo ""
  REDUNDANCIES_FOUND=$((REDUNDANCIES_FOUND + 1))
fi

# Python language servers
if echo "$ALL_EXTENSIONS" | grep -q "ms-python.vscode-pylance" && echo "$ALL_EXTENSIONS" | grep -q "anysphere.cursorpyright"; then
  echo -e "${YELLOW}⚠️  Python Language Server Redundancy${NC}"
  echo -e "   ${CYAN}•${NC} ms-python.vscode-pylance (Microsoft Pylance)"
  echo -e "   ${CYAN}•${NC} anysphere.cursorpyright (Cursor's Pyright)"
  echo -e "   ${BLUE}Recommendation:${NC} Keep ${GREEN}anysphere.cursorpyright${NC} (Cursor-optimized) or ${GREEN}ms-python.vscode-pylance${NC}, not both"
  echo ""
  REDUNDANCIES_FOUND=$((REDUNDANCIES_FOUND + 1))
fi

# Summary
echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}  Summary${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}"
echo ""

TOTAL_COUNT=$(echo "$ALL_EXTENSIONS" | wc -l | tr -d ' ')

TOTAL_REDUNDANCIES=$((REDUNDANCIES_FOUND + CATEGORY_REDUNDANCIES))

if [ $TOTAL_REDUNDANCIES -eq 0 ]; then
  echo -e "${GREEN}✓ No major redundancies found${NC}"
else
  echo -e "${YELLOW}Found potential redundancies: ${REDUNDANCIES_FOUND} functional + ${CATEGORY_REDUNDANCIES} category-based${NC}"
  echo ""
  echo -e "${BLUE}Recommendations:${NC}"
  echo "1. Review each group above and remove redundant extensions"
  echo "2. Keep Cursor-specific versions (anysphere.*) when available"
  echo "3. Remove disabled extensions completely if not needed"
  echo "4. Consolidate AI assistants to 1-2 favorites"
  echo "5. Review MCP extensions - many may overlap"
fi

echo ""
echo -e "Total Extensions: ${CYAN}${TOTAL_COUNT}${NC}"
echo ""
echo "For detailed analysis, see the groups above."
