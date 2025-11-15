#!/bin/bash
# List global documentation files for Cursor setup
# Usage: ./scripts/list_global_docs.sh

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

echo "=========================================="
echo "Cursor Global Docs - File Paths"
echo "=========================================="
echo ""
echo "Copy these paths into Cursor Settings → Features → Docs"
echo ""
echo "--- High-Priority Files (Must-Have) ---"
echo ""

# High-priority files
HIGH_PRIORITY=(
  "docs/API_DOCUMENTATION_INDEX.md"
  "docs/CODEBASE_ARCHITECTURE.md"
  "docs/COMMON_PATTERNS.md"
  "docs/AI_FRIENDLY_CODE.md"
  "docs/TWS_INTEGRATION_STATUS.md"
  "docs/BOX_SPREAD_COMPREHENSIVE_GUIDE.md"
  "docs/STATIC_ANALYSIS_ANNOTATIONS.md"
  "docs/IMPLEMENTATION_GUIDE.md"
)

for file in "${HIGH_PRIORITY[@]}"; do
  full_path="$PROJECT_ROOT/$file"
  if [ -f "$full_path" ]; then
    echo "$full_path"
  else
    echo "# MISSING: $full_path" >&2
  fi
done

echo ""
echo "--- External Documentation (Optional) ---"
echo ""

# External files
EXTERNAL=(
  "docs/external/TWS_API_QUICK_REFERENCE.md"
  "docs/external/ECLIENT_EWRAPPER_PATTERNS.md"
  "docs/external/CMake_PRESETS_GUIDE.md"
  "docs/external/CPP20_FEATURES.md"
)

for file in "${EXTERNAL[@]}"; do
  full_path="$PROJECT_ROOT/$file"
  if [ -f "$full_path" ]; then
    echo "$full_path"
  else
    echo "# MISSING: $full_path" >&2
  fi
done

echo ""
echo "=========================================="
echo "Relative paths (if Cursor supports them):"
echo "=========================================="
echo ""
echo "--- High-Priority Files ---"
for file in "${HIGH_PRIORITY[@]}"; do
  echo "$file"
done
echo ""
echo "--- External Documentation ---"
for file in "${EXTERNAL[@]}"; do
  echo "$file"
done
echo ""
echo "=========================================="
