#!/usr/bin/env bash
# Check extension build status
set -euo pipefail

cd "$(dirname "$0")"

echo "=== Extension Build Status ==="
echo ""

# Check dependencies
if [ -d "node_modules" ]; then
  echo "✓ Dependencies installed (node_modules exists)"
else
  echo "✗ Dependencies NOT installed"
  echo "  Run: npm install"
fi
echo ""

# Check compilation
if [ -f "out/extension.js" ]; then
  echo "✓ Compiled: out/extension.js exists"
  ls -lh out/extension.js
else
  echo "✗ NOT compiled: out/extension.js missing"
  echo "  Run: npm run compile"
fi
echo ""

# Check packaging
VSIX_FILE=$(ls -t *.vsix 2>/dev/null | head -1)
if [ -n "$VSIX_FILE" ] && [ -f "$VSIX_FILE" ]; then
  echo "✓ Packaged: $VSIX_FILE exists"
  ls -lh "$VSIX_FILE"
  echo ""
  echo "To install in Cursor:"
  echo "  1. Open Cursor"
  echo "  2. Cmd+Shift+P → 'Extensions: Install from VSIX...'"
  echo "  3. Select: $(pwd)/$VSIX_FILE"
else
  echo "✗ NOT packaged: VSIX file missing"
  echo "  Run: npm run package"
fi
echo ""

# Check for errors
if [ -f "npm-debug.log" ] || [ -f "yarn-error.log" ]; then
  echo "⚠ Build error logs found - check for issues"
fi
