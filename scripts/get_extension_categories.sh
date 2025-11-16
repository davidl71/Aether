#!/bin/bash
# Get extension categories from installed extensions
# Reads package.json files to extract category information

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Colors
CYAN='\033[0;36m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}Extracting extension categories...${NC}"
echo ""

# Check if Cursor CLI is available
if command -v cursor &> /dev/null; then
  CLI_CMD="cursor"
  EXT_DIR="$HOME/Library/Application Support/Cursor/extensions"
elif command -v code &> /dev/null; then
  CLI_CMD="code"
  EXT_DIR="$HOME/.vscode/extensions"
else
  echo "Error: Neither Cursor nor VS Code CLI found"
  exit 1
fi

# Get all installed extensions
ALL_EXTENSIONS=$($CLI_CMD --list-extensions 2>/dev/null | sort)

# VS Code extension categories (standard categories)
declare -a VSCODE_CATEGORIES=(
  "Azure"
  "Data Science"
  "Debuggers"
  "Education"
  "Extension Packs"
  "Formatters"
  "Keymaps"
  "Language Packs"
  "Linters"
  "Machine Learning"
  "Notebooks"
  "Other"
  "Programming Languages"
  "SCM Providers"
  "Snippets"
  "Testing"
  "Themes"
  "Visualization"
)

# Function to get extension category from package.json
get_extension_category() {
  local ext_id="$1"
  local ext_dir=$(find "$EXT_DIR" -maxdepth 2 -type d -name "${ext_id}-*" 2>/dev/null | head -1)

  if [ -z "$ext_dir" ]; then
    echo "unknown"
    return
  fi

  local package_json="${ext_dir}/package.json"
  if [ ! -f "$package_json" ]; then
    echo "unknown"
    return
  fi

  # Extract categories from package.json
  local categories=$(grep -A 10 '"categories"' "$package_json" 2>/dev/null | grep -o '"[^"]*"' | head -5 | tr -d '"' | tr '\n' ',' | sed 's/,$//')

  if [ -z "$categories" ]; then
    echo "uncategorized"
  else
    echo "$categories"
  fi
}

# Analyze extensions by category
declare -A CATEGORY_MAP

echo -e "${CYAN}Analyzing extension categories...${NC}"
echo ""

while IFS= read -r ext; do
  [ -z "$ext" ] && continue

  categories=$(get_extension_category "$ext")

  # Store in map (using comma as delimiter for multiple categories)
  IFS=',' read -ra CAT_ARRAY <<< "$categories"
  for cat in "${CAT_ARRAY[@]}"; do
    cat=$(echo "$cat" | xargs)  # trim whitespace
    if [ -n "$cat" ] && [ "$cat" != "unknown" ] && [ "$cat" != "uncategorized" ]; then
      if [ -z "${CATEGORY_MAP[$cat]}" ]; then
        CATEGORY_MAP[$cat]="$ext"
      else
        CATEGORY_MAP[$cat]="${CATEGORY_MAP[$cat]},$ext"
      fi
    fi
  done
done <<< "$ALL_EXTENSIONS"

# Display results
echo -e "${GREEN}Extensions by Category:${NC}"
echo ""

for category in "${!CATEGORY_MAP[@]}"; do
  extensions="${CATEGORY_MAP[$category]}"
  count=$(echo "$extensions" | tr ',' '\n' | wc -l | tr -d ' ')
  echo -e "${BLUE}${category}${NC} (${count}):"
  IFS=',' read -ra EXT_ARRAY <<< "$extensions"
  for ext in "${EXT_ARRAY[@]}"; do
    echo -e "  ${YELLOW}•${NC} $ext"
  done
  echo ""
done

# Check for potential redundancies within categories
echo -e "${CYAN}Checking for category-based redundancies...${NC}"
echo ""

REDUNDANT_CATEGORIES=0
for category in "${!CATEGORY_MAP[@]}"; do
  extensions="${CATEGORY_MAP[$category]}"
  count=$(echo "$extensions" | tr ',' '\n' | wc -l | tr -d ' ')

  if [ "$count" -gt 1 ]; then
    case "$category" in
      "Formatters"|"Linters"|"Themes"|"Debuggers"|"SCM Providers")
        echo -e "${YELLOW}⚠️  ${category} (${count} extensions)${NC}"
        IFS=',' read -ra EXT_ARRAY <<< "$extensions"
        for ext in "${EXT_ARRAY[@]}"; do
          echo -e "    ${YELLOW}•${NC} $ext"
        done
        echo ""
        REDUNDANT_CATEGORIES=$((REDUNDANT_CATEGORIES + 1))
        ;;
    esac
  fi
done

if [ $REDUNDANT_CATEGORIES -eq 0 ]; then
  echo -e "${GREEN}✓ No obvious category-based redundancies found${NC}"
fi
