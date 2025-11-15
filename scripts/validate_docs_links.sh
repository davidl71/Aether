#!/bin/bash
# Validate Documentation Links
# Checks for broken URLs in API documentation files

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
DOCS_DIR="$PROJECT_ROOT/docs"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Counters
TOTAL_LINKS=0
BROKEN_LINKS=0
SKIPPED_LINKS=0

# Files to check
FILES=(
  "$DOCS_DIR/API_DOCUMENTATION_INDEX.md"
  "$DOCS_DIR/API_DOCUMENTATION_SUMMARY.md"
  "$DOCS_DIR/indices"/*.md
)

# URLs to skip (known issues, local files, etc.)
SKIP_PATTERNS=(
  "docs/"           # Local documentation links
  "mailto:"         # Email links
  "#"               # Anchor links
  "github.com.*blob" # GitHub blob links (may require auth)
)

echo "🔍 Validating documentation links..."
echo ""

# Function to check if URL should be skipped
should_skip() {
  local url="$1"
  for pattern in "${SKIP_PATTERNS[@]}"; do
    if [[ "$url" == *"$pattern"* ]]; then
      return 0
    fi
  done
  return 1
}

# Function to validate a single URL
validate_url() {
  local url="$1"
  local file="$2"
  local line="$3"

  ((TOTAL_LINKS++))

  if should_skip "$url"; then
    ((SKIPPED_LINKS++))
    return 0
  fi

  # Use curl to check URL (with timeout and follow redirects)
  local http_code
  http_code=$(curl -s -o /dev/null -w "%{http_code}" --max-time 10 --location "$url" 2>/dev/null || echo "000")

  if [[ "$http_code" =~ ^[23][0-9][0-9]$ ]]; then
    return 0  # Success (2xx or 3xx)
  else
    echo -e "${RED}❌ Broken link${NC}: $url"
    echo "   File: $file (line $line)"
    echo "   HTTP Code: $http_code"
    echo ""
    ((BROKEN_LINKS++))
    return 1
  fi
}

# Extract URLs from markdown files
extract_urls() {
  local file="$1"
  local line_num=0

  while IFS= read -r line; do
    ((line_num++))

    # Match markdown links: [text](url)
    local link_pattern='\[([^\]]+)\]\(([^)]+)\)'
    while [[ $line =~ $link_pattern ]]; do
      local url="${BASH_REMATCH[2]}"
      validate_url "$url" "$file" "$line_num"
      line="${line#*]}"
    done

    # Match angle bracket URLs: <url>
    while [[ $line =~ \<([^>]+)\> ]]; do
      local url="${BASH_REMATCH[1]}"
      if [[ "$url" =~ ^https?:// ]]; then
        validate_url "$url" "$file" "$line_num"
      fi
      line="${line#*>}"
    done
  done < "$file"
}

# Process all files
for file in "${FILES[@]}"; do
  if [[ ! -f "$file" ]]; then
    continue
  fi

  echo "Checking: $(basename "$file")"
  extract_urls "$file"
done

# Summary
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📊 Validation Summary:"
echo "   Total links checked: $TOTAL_LINKS"
echo "   Skipped (local/email): $SKIPPED_LINKS"
echo "   Valid links: $((TOTAL_LINKS - BROKEN_LINKS - SKIPPED_LINKS))"
if [[ $BROKEN_LINKS -eq 0 ]]; then
  echo -e "   ${GREEN}Broken links: $BROKEN_LINKS ✅${NC}"
  exit 0
else
  echo -e "   ${RED}Broken links: $BROKEN_LINKS ❌${NC}"
  exit 1
fi
