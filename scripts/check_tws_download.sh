#!/bin/bash
# check_tws_download.sh - Verify TWS API download

set -euo pipefail

echo "════════════════════════════════════════════════════════════"
echo "  TWS API Download Checker"
echo "════════════════════════════════════════════════════════════"
echo ""

# Check Downloads folder for TWS API
DOWNLOADS="${HOME}/Downloads"
echo "Checking ${DOWNLOADS} for TWS API..."
echo ""

# Find TWS API zip files
FILES=$(find "${DOWNLOADS}" -maxdepth 1 -name "twsapi*.zip" 2>/dev/null || true)

if [ -z "${FILES}" ]; then
    echo "❌ No TWS API zip files found in Downloads folder"
    echo ""
    echo "Please download TWS API from:"
    echo "  https://interactivebrokers.github.io/"
    echo ""
    echo "Look for: TWS API for Mac/Unix"
    echo "File name: twsapi_macunix.XXX.zip"
    echo ""
    exit 1
fi

echo "✅ Found TWS API file(s):"
echo ""

# List found files with details
ls -lh ${FILES}
echo ""

# Get the most recent file
LATEST=$(ls -t ${FILES} | head -1)
echo "Most recent: ${LATEST}"
echo ""

# Check file size (should be ~10-20 MB)
SIZE=$(stat -f%z "${LATEST}" 2>/dev/null || stat -c%s "${LATEST}")
SIZE_MB=$((SIZE / 1024 / 1024))

if [ ${SIZE_MB} -lt 5 ]; then
    echo "⚠️  Warning: File seems small (${SIZE_MB} MB)"
    echo "Expected size: 10-20 MB"
    echo "The download may be incomplete."
    echo ""
    exit 1
fi

echo "✅ File size looks good: ${SIZE_MB} MB"
echo ""

# Test if it's a valid zip
echo "Testing zip file integrity..."
if unzip -t "${LATEST}" > /dev/null 2>&1; then
    echo "✅ Zip file is valid"
else
    echo "❌ Zip file appears corrupted"
    echo "Please re-download"
    exit 1
fi

echo ""
echo "════════════════════════════════════════════════════════════"
echo "  ✅ Download Verified!"
echo "════════════════════════════════════════════════════════════"
echo ""
echo "File ready to extract: ${LATEST}"
echo ""
echo "Next step:"
echo "  Run: ./scripts/extract_tws.sh"
echo ""
