#!/bin/bash
# Comprehensive extension analyzer - checks all installed extensions
# and categorizes them for workspace vs global management

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
echo -e "${BLUE}  Comprehensive VS Code Extension Analysis${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}"
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

# Get all installed extensions
echo -e "${CYAN}Fetching installed extensions from ${IDE_NAME}...${NC}"
ALL_EXTENSIONS=$($CLI_CMD --list-extensions 2>/dev/null | sort || echo "")
TOTAL_COUNT=$(echo "$ALL_EXTENSIONS" | grep -c . || echo "0")

if [ "$TOTAL_COUNT" -eq 0 ]; then
  echo -e "${YELLOW}No extensions found via CLI.${NC}"
  echo "This might mean:"
  echo "  1. Extensions are installed but CLI can't access them"
  echo "  2. VS Code is not properly configured"
  echo ""
  echo "Please check VS Code Extensions view manually (Cmd+Shift+X)"
  exit 1
fi

echo -e "${GREEN}Found ${TOTAL_COUNT} installed extension(s)${NC}"
echo ""

# Load workspace recommendations and unwanted extensions
WORKSPACE_RECOMMENDED=$(jq -r '.recommendations[]' "$PROJECT_ROOT/.vscode/extensions.json" 2>/dev/null || echo "")
UNWANTED=$(jq -r '.unwantedRecommendations[]' "$PROJECT_ROOT/.vscode/extensions.json" 2>/dev/null || echo "")

# Define categories
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

GLOBAL_SAFE=(
  "editorconfig.editorconfig"
  "redhat.vscode-yaml"
  "eamodio.gitlens"
  "yzhang.markdown-all-in-one"
  "davidanson.vscode-markdownlint"
  "streetsidesoftware.code-spell-checker"
  "usernamehw.errorlens"
)

# Language extensions that should be workspace-only
LANGUAGE_EXTENSIONS=(
  # C/C++
  "ms-vscode.cpptools"
  "ms-vscode.cmake-tools"
  "ms-vscode.cpptools-extension-pack"
  # Python
  "ms-python.python"
  "ms-python.vscode-pylance"
  "ms-python.black-formatter"
  "ms-python.isort"
  "ms-python.autopep8"
  "ms-python.flake8"
  # Rust
  "rust-lang.rust-analyzer"
  # TypeScript/JavaScript
  "dbaeumer.vscode-eslint"
  "esbenp.prettier-vscode"
  # Swift
  "sswg.swift-lang"
  # Go
  "golang.go"
  # Java
  "vscjava.vscode-java-pack"
  "vscjava.vscode-java-debug"
  "vscjava.vscode-java-test"
  "vscjava.vscode-maven"
  "vscjava.vscode-gradle"
  "redhat.java"
  # C#
  "ms-dotnettools.csharp"
  "ms-dotnettools.csdevkit"
  # Dart
  "dart-code.dart"
  "dart-code.flutter"
  # PHP
  "bmewburn.vscode-intelephense-client"
  "felixfbecker.php-intellisense"
  # Ruby
  "shopify.ruby-lsp"
  "rebornix.ruby"
  # Kotlin
  "fwcd.kotlin"
  "mathiasfrohlich.kotlin"
)

# AI/Assistant extensions
AI_EXTENSIONS=(
  "github.copilot"
  "github.copilot-chat"
  "amazonwebservices.codewhisperer-for-command-line-companion"
  "prompttower.prompttower"
)

# Initialize counters
WORKSPACE_ISSUES=0
UNWANTED_FOUND=0
UNCATEGORIZED=0
AI_FOUND=0

# Arrays to store results
WORKSPACE_ISSUE_LIST=()
UNWANTED_LIST=()
UNCATEGORIZED_LIST=()
AI_LIST=()
SAFE_GLOBAL_LIST=()
WORKSPACE_OK_LIST=()

echo -e "${YELLOW}═══════════════════════════════════════════════════════════${NC}"
echo -e "${YELLOW}  Analysis Results${NC}"
echo -e "${YELLOW}═══════════════════════════════════════════════════════════${NC}"
echo ""

# Analyze each extension
while IFS= read -r ext; do
  [ -z "$ext" ] && continue

  # Check if workspace-only
  is_workspace_only=false
  for ws_ext in "${WORKSPACE_ONLY[@]}"; do
    if [ "$ext" = "$ws_ext" ]; then
      is_workspace_only=true
      break
    fi
  done

  # Check if unwanted
  is_unwanted=false
  while IFS= read -r unwanted_ext; do
    if [ "$ext" = "$unwanted_ext" ]; then
      is_unwanted=true
      break
    fi
  done <<< "$UNWANTED"

  # Check if safe global
  is_safe_global=false
  for safe_ext in "${GLOBAL_SAFE[@]}"; do
    if [ "$ext" = "$safe_ext" ]; then
      is_safe_global=true
      break
    fi
  done

  # Check if language extension
  is_language=false
  for lang_ext in "${LANGUAGE_EXTENSIONS[@]}"; do
    if [ "$ext" = "$lang_ext" ]; then
      is_language=true
      break
    fi
  done

  # Check if AI extension
  is_ai=false
  for ai_ext in "${AI_EXTENSIONS[@]}"; do
    if [ "$ext" = "$ai_ext" ]; then
      is_ai=true
      break
    fi
  done

  # Categorize
  if [ "$is_unwanted" = true ]; then
    UNWANTED_LIST+=("$ext")
    UNWANTED_FOUND=$((UNWANTED_FOUND + 1))
  elif [ "$is_workspace_only" = true ]; then
    WORKSPACE_OK_LIST+=("$ext")
  elif [ "$is_safe_global" = true ]; then
    SAFE_GLOBAL_LIST+=("$ext")
  elif [ "$is_ai" = true ]; then
    AI_LIST+=("$ext")
    AI_FOUND=$((AI_FOUND + 1))
  elif [ "$is_language" = true ]; then
    WORKSPACE_ISSUE_LIST+=("$ext")
    WORKSPACE_ISSUES=$((WORKSPACE_ISSUES + 1))
  else
    UNCATEGORIZED_LIST+=("$ext")
    UNCATEGORIZED=$((UNCATEGORIZED + 1))
  fi
done <<< "$ALL_EXTENSIONS"

# Print results
if [ ${#UNWANTED_LIST[@]} -gt 0 ]; then
  echo -e "${RED}⚠️  UNWANTED EXTENSIONS (${UNWANTED_FOUND})${NC}"
  echo -e "${RED}   These should be DISABLED - not used in this project${NC}"
  for ext in "${UNWANTED_LIST[@]}"; do
    echo -e "   ${RED}•${NC} $ext"
  done
  echo ""
fi

if [ ${#WORKSPACE_ISSUE_LIST[@]} -gt 0 ]; then
  echo -e "${YELLOW}⚠️  LANGUAGE EXTENSIONS (${WORKSPACE_ISSUES})${NC}"
  echo -e "${YELLOW}   Should be workspace-only to prevent conflicts${NC}"
  for ext in "${WORKSPACE_ISSUE_LIST[@]}"; do
    echo -e "   ${YELLOW}•${NC} $ext"
  done
  echo ""
fi

if [ ${#AI_LIST[@]} -gt 0 ]; then
  echo -e "${CYAN}🤖 AI/ASSISTANT EXTENSIONS (${AI_FOUND})${NC}"
  echo -e "${CYAN}   Consider workspace-only for project-specific configs${NC}"
  for ext in "${AI_LIST[@]}"; do
    echo -e "   ${CYAN}•${NC} $ext"
  done
  echo ""
fi

if [ ${#SAFE_GLOBAL_LIST[@]} -gt 0 ]; then
  echo -e "${GREEN}✓ SAFE GLOBAL EXTENSIONS (${#SAFE_GLOBAL_LIST[@]})${NC}"
  for ext in "${SAFE_GLOBAL_LIST[@]}"; do
    echo -e "   ${GREEN}•${NC} $ext"
  done
  echo ""
fi

if [ ${#WORKSPACE_OK_LIST[@]} -gt 0 ]; then
  echo -e "${GREEN}✓ WORKSPACE-ONLY (CORRECTLY CONFIGURED) (${#WORKSPACE_OK_LIST[@]})${NC}"
  for ext in "${WORKSPACE_OK_LIST[@]}"; do
    echo -e "   ${GREEN}•${NC} $ext"
  done
  echo ""
fi

if [ ${#UNCATEGORIZED_LIST[@]} -gt 0 ]; then
  echo -e "${MAGENTA}? UNCATEGORIZED EXTENSIONS (${UNCATEGORIZED})${NC}"
  echo -e "${MAGENTA}   Review these manually${NC}"
  for ext in "${UNCATEGORIZED_LIST[@]}"; do
    echo -e "   ${MAGENTA}•${NC} $ext"
  done
  echo ""
fi

# Summary
echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}  Summary${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}"
echo ""
echo -e "Total Extensions: ${CYAN}${TOTAL_COUNT}${NC}"
echo -e "Unwanted: ${RED}${UNWANTED_FOUND}${NC}"
echo -e "Language Extensions (should be workspace-only): ${YELLOW}${WORKSPACE_ISSUES}${NC}"
echo -e "AI Extensions: ${CYAN}${AI_FOUND}${NC}"
echo -e "Safe Global: ${GREEN}${#SAFE_GLOBAL_LIST[@]}${NC}"
echo -e "Workspace-Only (OK): ${GREEN}${#WORKSPACE_OK_LIST[@]}${NC}"
echo -e "Uncategorized: ${MAGENTA}${UNCATEGORIZED}${NC}"
echo ""

# Recommendations
if [ $UNWANTED_FOUND -gt 0 ] || [ $WORKSPACE_ISSUES -gt 0 ]; then
  echo -e "${YELLOW}═══════════════════════════════════════════════════════════${NC}"
  echo -e "${YELLOW}  Recommendations${NC}"
  echo -e "${YELLOW}═══════════════════════════════════════════════════════════${NC}"
  echo ""

  if [ $UNWANTED_FOUND -gt 0 ]; then
    echo -e "${RED}1. Disable ${UNWANTED_FOUND} unwanted extension(s):${NC}"
    echo "   Open Extensions (Cmd+Shift+X) → Search → Gear icon → Disable"
    echo ""
  fi

  if [ $WORKSPACE_ISSUES -gt 0 ]; then
    echo -e "${YELLOW}2. Make ${WORKSPACE_ISSUES} language extension(s) workspace-only:${NC}"
    echo "   Open Extensions → Gear icon → 'Disable' globally"
    echo "   Then enable only in this workspace"
    echo ""
  fi

  echo "See docs/EXTENSION_MANAGEMENT.md for detailed guidance"
  echo ""
fi

# Export full list for review
OUTPUT_FILE="$PROJECT_ROOT/.vscode/extensions_analysis.txt"
echo "$ALL_EXTENSIONS" > "$OUTPUT_FILE"
echo -e "${BLUE}Full extension list saved to: ${OUTPUT_FILE}${NC}"
