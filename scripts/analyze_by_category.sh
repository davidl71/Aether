#!/bin/bash
# Comprehensive category-based extension analysis
# Uses VS Code's @category filter to group and analyze extensions

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
echo -e "${BLUE}  Category-Based Extension Analysis${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}"
echo ""

# Check if Cursor CLI is available
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
  IDE_NAME=$([[ "$CLI_CMD_AUTO" == "cursor" ]] && echo "Cursor" || echo "VS Code")
else
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
fi

# Common VS Code categories (use lowercase as per CLI)
declare -a CATEGORIES=(
  "formatters"
  "linters"
  "debuggers"
  "themes"
  "programming languages"
  "scm providers"
  "snippets"
  "other"
  "extension packs"
  "language packs"
  "keymaps"
  "notebooks"
  "data science"
  "machine learning"
  "testing"
  "visualization"
  "education"
  "azure"
  "ai"
  "chat"
)

echo -e "${CYAN}Analyzing extensions by category...${NC}"
echo ""

TOTAL_BY_CATEGORY=0
CATEGORIES_WITH_EXTENSIONS=0

# Analyze each category
for category in "${CATEGORIES[@]}"; do
  category_exts=$($CLI_CMD --list-extensions --category "$category" 2>/dev/null | sort || echo "")

  # Skip if category is invalid (error message in output)
  if echo "$category_exts" | grep -q "Invalid category"; then
    continue
  fi

  if [ -n "$category_exts" ]; then
    count=$(echo "$category_exts" | grep -v "Invalid category" | grep -c . || echo "0")
    if [ "$count" -gt 0 ]; then
      CATEGORIES_WITH_EXTENSIONS=$((CATEGORIES_WITH_EXTENSIONS + 1))
      TOTAL_BY_CATEGORY=$((TOTAL_BY_CATEGORY + count))

      # Capitalize first letter of each word for display
      category_display=$(echo "$category" | sed 's/\b\(.\)/\u\1/g')

      echo -e "${BLUE}${category_display}${NC} (${count} extension(s)):"
      while IFS= read -r ext; do
        [ -z "$ext" ] && continue
        # Skip error messages
        [[ "$ext" == *"Invalid category"* ]] && continue
        echo -e "  ${CYAN}•${NC} $ext"
      done <<< "$category_exts"
      echo ""

      # Provide recommendations for categories with multiple extensions
      if [ "$count" -gt 1 ]; then
        case "$category" in
          "formatters")
            echo -e "  ${YELLOW}⚠️  Multiple formatters - ensure they don't conflict${NC}"
            echo -e "  ${BLUE}Tip:${NC} Language-specific formatters (Black, ESLint) are usually fine"
            echo ""
            ;;
          "linters")
            echo -e "  ${GREEN}✓${NC} Multiple linters are usually fine (different languages)"
            echo ""
            ;;
          "debuggers")
            echo -e "  ${GREEN}✓${NC} Different debuggers for different targets (Python, C++, browser)"
            echo ""
            ;;
          "themes")
            echo -e "  ${GREEN}✓${NC} Multiple themes are fine - they don't conflict"
            echo ""
            ;;
          "extension packs")
            echo -e "  ${YELLOW}⚠️  Extension packs may include individual extensions${NC}"
            echo -e "  ${BLUE}Tip:${NC} Check if individual extensions are also installed"
            echo ""
            ;;
        esac
      fi
    fi
  fi
done

# Summary
echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}  Summary${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}"
echo ""

TOTAL_EXTENSIONS=$($CLI_CMD --list-extensions 2>/dev/null | wc -l | tr -d ' ')

echo -e "Total Extensions: ${CYAN}${TOTAL_EXTENSIONS}${NC}"
echo -e "Categories with Extensions: ${CYAN}${CATEGORIES_WITH_EXTENSIONS}${NC}"
echo -e "Extensions Categorized: ${CYAN}${TOTAL_BY_CATEGORY}${NC}"
echo ""

if [ $TOTAL_BY_CATEGORY -lt $TOTAL_EXTENSIONS ]; then
  uncategorized=$((TOTAL_EXTENSIONS - TOTAL_BY_CATEGORY))
  echo -e "${YELLOW}Note:${NC} ${uncategorized} extension(s) may not be categorized"
  echo "  (Some extensions may not have categories assigned)"
fi

echo ""
echo -e "${BLUE}Category-Based Redundancy Check:${NC}"
echo ""
echo "Categories with multiple extensions:"
echo "  • Formatters - Review for conflicts"
echo "  • Linters - Usually fine (different languages)"
echo "  • Debuggers - Usually fine (different targets)"
echo "  • Themes - Fine (no conflicts)"
echo ""
echo "Run ${CYAN}./scripts/check_extension_redundancy.sh${NC} for detailed redundancy analysis"
