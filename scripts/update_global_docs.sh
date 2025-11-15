#!/bin/bash
# Automatically update and validate global documentation files for Cursor
# Usage: ./scripts/update_global_docs.sh [--check-only] [--sync]

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
CONFIG_FILE="$PROJECT_ROOT/.cursor/global-docs.json"
CHECK_ONLY=false
SYNC=false

# Parse arguments
while [[ $# -gt 0 ]]; do
  case $1 in
    --check-only)
      CHECK_ONLY=true
      shift
      ;;
    --sync)
      SYNC=true
      shift
      ;;
    *)
      echo "Unknown option: $1"
      echo "Usage: $0 [--check-only] [--sync]"
      exit 1
      ;;
  esac
done

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}Cursor Global Docs - Update Script${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""

# Check if config file exists
if [[ ! -f "$CONFIG_FILE" ]]; then
  echo -e "${RED}Error: Config file not found: $CONFIG_FILE${NC}"
  exit 1
fi

# Read config file
if ! command -v jq &> /dev/null; then
  echo -e "${YELLOW}Warning: jq not found. Install with: brew install jq${NC}"
  echo "Falling back to basic validation..."
  USE_JQ=false
else
  USE_JQ=true
fi

# Function to check if file exists
check_file() {
  local file_path="$1"
  local full_path="$PROJECT_ROOT/$file_path"

  if [[ -f "$full_path" ]]; then
    echo -e "${GREEN}✓${NC} $file_path"
    return 0
  else
    echo -e "${RED}✗${NC} $file_path ${RED}(MISSING)${NC}"
    return 1
  fi
}

# Function to get all files from config
get_all_files() {
  if [[ "$USE_JQ" == true ]]; then
    jq -r '.highPriority[], .external[], .secondary[] | .path' "$CONFIG_FILE" 2>/dev/null || {
      # Fallback: extract paths manually
      grep -o '"path": "[^"]*"' "$CONFIG_FILE" | sed 's/"path": "\(.*\)"/\1/'
    }
  else
    # Fallback: extract paths manually
    grep -o '"path": "[^"]*"' "$CONFIG_FILE" | sed 's/"path": "\(.*\)"/\1/'
  fi
}

# Validate all files
echo -e "${BLUE}Validating documentation files...${NC}"
echo ""

MISSING_FILES=()
ALL_FILES=()

while IFS= read -r file_path; do
  [[ -z "$file_path" ]] && continue
  ALL_FILES+=("$file_path")
  if ! check_file "$file_path"; then
    MISSING_FILES+=("$file_path")
  fi
done < <(get_all_files)

echo ""
echo -e "${BLUE}Summary:${NC}"
echo "  Total files: ${#ALL_FILES[@]}"
echo "  Found: $(( ${#ALL_FILES[@]} - ${#MISSING_FILES[@]} ))"
echo "  Missing: ${#MISSING_FILES[@]}"

if [[ ${#MISSING_FILES[@]} -gt 0 ]]; then
  echo ""
  echo -e "${RED}Missing files:${NC}"
  for file in "${MISSING_FILES[@]}"; do
    echo "  - $file"
  done
  exit 1
fi

echo ""
echo -e "${GREEN}All documentation files are present!${NC}"

# Generate paths list
echo ""
echo -e "${BLUE}Generating paths list...${NC}"
PATHS_FILE="$PROJECT_ROOT/.cursor/global-docs-paths.txt"
{
  echo "# Cursor Global Docs - File Paths"
  echo "# Generated: $(date)"
  echo "# Use these paths in Cursor Settings → Features → Docs"
  echo ""
  echo "--- High-Priority Files (Must-Have) ---"
  if [[ "$USE_JQ" == true ]]; then
    jq -r '.highPriority[] | "$PROJECT_ROOT/\(.path)"' "$CONFIG_FILE" | \
      sed "s|\$PROJECT_ROOT|$PROJECT_ROOT|g"
  else
    grep -A 1 '"highPriority"' "$CONFIG_FILE" | grep '"path"' | \
      sed 's/.*"path": "\(.*\)".*/'"$PROJECT_ROOT"'\/\1/'
  fi
  echo ""
  echo "--- External Documentation (Optional) ---"
  if [[ "$USE_JQ" == true ]]; then
    jq -r '.external[] | "$PROJECT_ROOT/\(.path)"' "$CONFIG_FILE" | \
      sed "s|\$PROJECT_ROOT|$PROJECT_ROOT|g"
  else
    grep -A 1 '"external"' "$CONFIG_FILE" | grep '"path"' | \
      sed 's/.*"path": "\(.*\)".*/'"$PROJECT_ROOT"'\/\1/'
  fi
} > "$PATHS_FILE"

echo -e "${GREEN}Paths list generated: $PATHS_FILE${NC}"

# Generate relative paths list
RELATIVE_PATHS_FILE="$PROJECT_ROOT/.cursor/global-docs-paths-relative.txt"
{
  echo "# Cursor Global Docs - Relative Paths"
  echo "# Generated: $(date)"
  echo ""
  echo "--- High-Priority Files ---"
  if [[ "$USE_JQ" == true ]]; then
    jq -r '.highPriority[] | .path' "$CONFIG_FILE"
  else
    grep -A 1 '"highPriority"' "$CONFIG_FILE" | grep '"path"' | \
      sed 's/.*"path": "\(.*\)".*/\1/'
  fi
  echo ""
  echo "--- External Documentation ---"
  if [[ "$USE_JQ" == true ]]; then
    jq -r '.external[] | .path' "$CONFIG_FILE"
  else
    grep -A 1 '"external"' "$CONFIG_FILE" | grep '"path"' | \
      sed 's/.*"path": "\(.*\)".*/\1/'
  fi
} > "$RELATIVE_PATHS_FILE"

echo -e "${GREEN}Relative paths list generated: $RELATIVE_PATHS_FILE${NC}"

# Update lastUpdated in config
if [[ "$USE_JQ" == true ]] && [[ "$CHECK_ONLY" == false ]]; then
  TODAY=$(date +%Y-%m-%d)
  jq ".lastUpdated = \"$TODAY\"" "$CONFIG_FILE" > "$CONFIG_FILE.tmp" && \
    mv "$CONFIG_FILE.tmp" "$CONFIG_FILE"
  echo -e "${GREEN}Config file updated with current date${NC}"
fi

# Sync instructions
if [[ "$SYNC" == true ]]; then
  echo ""
  echo -e "${YELLOW}Note: Cursor doesn't have a CLI for adding global docs yet.${NC}"
  echo -e "${YELLOW}Please manually add the files listed in: $PATHS_FILE${NC}"
  echo ""
  echo "To add in Cursor:"
  echo "1. Open Cursor Settings (Cmd+,)"
  echo "2. Navigate to Features → Docs"
  echo "3. Click 'Add Doc' or 'Add Folder'"
  echo "4. Copy paths from: $PATHS_FILE"
fi

echo ""
echo -e "${GREEN}Done!${NC}"
