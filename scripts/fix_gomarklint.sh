#!/usr/bin/env bash
# Fix gomarklint issues: level-1 first headings -> level-2, and duplicate headings

set -euo pipefail

fix_first_heading_level() {
  local file="$1"
  if [[ "$(head -1 "$file")" =~ ^#\ [^#] ]]; then
    sed -i '' '1s/^# /## /' "$file"
    echo "Fixed first heading: $file"
  fi
}

fix_duplicate_headings() {
  local file="$1"
  local tmp
  tmp=$(mktemp)

  awk '
    /^##?##? / {
      if (match($0, /^(#+)/)) {
        level = RLENGTH
        heading = substr($0, level + 2)
        if (seen[heading]++) {
          $0 = sprintf("%-*s%s (%d)", level, substr($0,1,level), heading, seen[heading])
        }
      }
    }
    { print }
  ' "$file" >"$tmp" && mv "$tmp" "$file"
  echo "Fixed duplicate headings: $file"
}

export -f fix_first_heading_level
export -f fix_duplicate_headings

echo "=== Phase 1: Fixing first-heading-level issues ==="
gomarklint docs/ 2>/dev/null | grep "First heading should be level 2" |
  sed 's/:1: First heading.*//' | sort -u |
  while read -r f; do fix_first_heading_level "$f"; done

echo ""
echo "=== Phase 2: Fixing duplicate-heading issues ==="
gomarklint docs/ 2>/dev/null | grep "duplicate heading:" |
  sed 's/:[0-9]*: duplicate heading:.*//' | sort -u |
  while read -r f; do fix_duplicate_headings "$f"; done

echo ""
echo "=== Done. Run 'gomarklint docs/' to verify ==="
