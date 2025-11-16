#!/bin/bash
# Compare built-in extensions with third-party extensions for redundancies

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
echo -e "${BLUE}  Built-in vs Third-Party Extension Comparison${NC}"
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

# Known built-in extensions (these come with Cursor/VS Code)
# Built-in extensions typically have publisher "vscode" or are core functionality
declare -a BUILTIN_PATTERNS=(
  "vscode."
  "ms-vscode."
  "anysphere."  # Cursor-specific built-ins
)

# Common built-in functionality (for reference, not used in array due to bash version compatibility)
# Built-in features include: TypeScript/JavaScript, JSON, HTML, CSS, Markdown, XML, YAML,
# Shell scripts, Git integration, Search, Terminal, Debugging, Snippets, Emmet, Formatting, IntelliSense

# Third-party extensions that might duplicate built-in functionality
# Format: extension_id|reason
POTENTIAL_REDUNDANCIES=(
  "yzhang.markdown-all-in-one|Built-in markdown support exists (but adds useful features)"
  "davidanson.vscode-markdownlint|Built-in markdown support exists (linting adds value)"
  "redhat.vscode-yaml|Built-in YAML support exists (schema validation adds value)"
  "eamodio.gitlens|Built-in Git exists (GitLens adds advanced features)"
  "donjayamanne.githistory|Built-in Git exists (history viewer adds features)"
)

# Check for built-in extensions
echo -e "${CYAN}Analyzing built-in vs third-party extensions...${NC}"
echo ""

# Identify likely built-in extensions
BUILTIN_FOUND=()
THIRD_PARTY=()

while IFS= read -r ext; do
  [ -z "$ext" ] && continue

  is_builtin=false
  for pattern in "${BUILTIN_PATTERNS[@]}"; do
    if [[ "$ext" == ${pattern}* ]]; then
      is_builtin=true
      break
    fi
  done

  if [ "$is_builtin" = true ]; then
    BUILTIN_FOUND+=("$ext")
  else
    THIRD_PARTY+=("$ext")
  fi
done <<< "$ALL_EXTENSIONS"

echo -e "${GREEN}Built-in Extensions (${#BUILTIN_FOUND[@]}):${NC}"
for ext in "${BUILTIN_FOUND[@]}"; do
  echo -e "  ${CYAN}•${NC} $ext"
done

echo ""
echo -e "${YELLOW}Third-Party Extensions (${#THIRD_PARTY[@]}):${NC}"
for ext in "${THIRD_PARTY[@]}"; do
  echo -e "  ${MAGENTA}•${NC} $ext"
done

echo ""
echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}  Potential Redundancies${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}"
echo ""

REDUNDANCIES_FOUND=0

# Check for known redundancies
for ext in "${THIRD_PARTY[@]}"; do
  for redundancy in "${POTENTIAL_REDUNDANCIES[@]}"; do
    redundancy_ext="${redundancy%%|*}"
    redundancy_reason="${redundancy#*|}"
    if [ "$ext" = "$redundancy_ext" ]; then
      echo -e "${YELLOW}⚠️  $ext${NC}"
      echo -e "   ${BLUE}Note:${NC} $redundancy_reason"
      echo ""
      REDUNDANCIES_FOUND=$((REDUNDANCIES_FOUND + 1))
      break
    fi
  done
done

# Check for language support redundancies
echo -e "${CYAN}Checking language support redundancies...${NC}"
echo ""

# Python - check if both built-in and third-party exist
if echo "$ALL_EXTENSIONS" | grep -q "ms-python.python"; then
  echo -e "${GREEN}✓${NC} Python: Using ${CYAN}ms-python.python${NC} (recommended, not built-in)"
else
  echo -e "${YELLOW}⚠${NC} Python: No Python extension found (built-in has basic support)"
fi

# C++ - check for conflicts
if echo "$ALL_EXTENSIONS" | grep -q "anysphere.cpptools"; then
  echo -e "${GREEN}✓${NC} C++: Using ${CYAN}anysphere.cpptools${NC} (Cursor's built-in, recommended)"
elif echo "$ALL_EXTENSIONS" | grep -q "ms-vscode.cpptools"; then
  echo -e "${YELLOW}⚠${NC} C++: Using ${CYAN}ms-vscode.cpptools${NC} (consider anysphere.cpptools for Cursor)"
fi

# TypeScript/JavaScript - built-in has excellent support
if echo "$ALL_EXTENSIONS" | grep -q "dbaeumer.vscode-eslint"; then
  echo -e "${GREEN}✓${NC} TypeScript/JS: Built-in support + ${CYAN}ESLint${NC} (complementary, not redundant)"
fi

# Markdown - check for redundancy
if echo "$ALL_EXTENSIONS" | grep -q "yzhang.markdown-all-in-one"; then
  echo -e "${YELLOW}⚠${NC} Markdown: Built-in support exists, but ${CYAN}markdown-all-in-one${NC} adds useful features"
  echo -e "   ${BLUE}Verdict:${NC} Keep (adds value beyond built-in)"
fi

if echo "$ALL_EXTENSIONS" | grep -q "davidanson.vscode-markdownlint"; then
  echo -e "${GREEN}✓${NC} Markdown: ${CYAN}markdownlint${NC} adds linting (complementary to built-in)"
fi

# YAML - check for redundancy
if echo "$ALL_EXTENSIONS" | grep -q "redhat.vscode-yaml"; then
  echo -e "${YELLOW}⚠${NC} YAML: Built-in support exists, but ${CYAN}redhat.vscode-yaml${NC} adds schema validation"
  echo -e "   ${BLUE}Verdict:${NC} Keep (adds value beyond built-in)"
fi

# Git - check for redundancy
if echo "$ALL_EXTENSIONS" | grep -q "eamodio.gitlens"; then
  echo -e "${GREEN}✓${NC} Git: Built-in Git + ${CYAN}GitLens${NC} (adds advanced features, not redundant)"
fi

echo ""
echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}  Analysis Summary${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}"
echo ""

TOTAL_COUNT=$(echo "$ALL_EXTENSIONS" | wc -l | tr -d ' ')

echo -e "Total Extensions: ${CYAN}${TOTAL_COUNT}${NC}"
echo -e "Built-in: ${GREEN}${#BUILTIN_FOUND[@]}${NC}"
echo -e "Third-party: ${MAGENTA}${#THIRD_PARTY[@]}${NC}"
echo ""

if [ $REDUNDANCIES_FOUND -eq 0 ]; then
  echo -e "${GREEN}✓ No major redundancies found${NC}"
  echo ""
  echo "Most third-party extensions add value beyond built-in functionality:"
  echo "  - Language servers (Python, C++, Rust) enhance built-in support"
  echo "  - Formatters (Black, ESLint) improve on basic formatting"
  echo "  - Linters add code quality checks"
  echo "  - Git tools (GitLens) add advanced Git features"
else
  echo -e "${YELLOW}Found ${REDUNDANCIES_FOUND} potential redundancy(ies)${NC}"
  echo ""
  echo "Review the extensions listed above to determine if they're truly redundant"
  echo "or if they add value beyond built-in functionality."
fi

echo ""
echo -e "${BLUE}Key Insight:${NC}"
echo "Built-in extensions provide basic functionality. Third-party extensions"
echo "typically enhance rather than duplicate. True redundancies are rare."
echo ""
echo "Examples of complementary (not redundant) extensions:"
echo "  • Built-in Git + GitLens = Enhanced Git features"
echo "  • Built-in Markdown + markdown-all-in-one = Better editing"
echo "  • Built-in YAML + redhat.vscode-yaml = Schema validation"
