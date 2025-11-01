#!/bin/bash
# extract_tws.sh - Extract TWS API to vendor directory

set -euo pipefail

readonly PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
readonly VENDOR_DIR="${PROJECT_ROOT}/vendor/tws-api"
readonly DOWNLOADS="${HOME}/Downloads"

echo "════════════════════════════════════════════════════════════"
echo "  TWS API Extraction"
echo "════════════════════════════════════════════════════════════"
echo ""

# Find TWS API zip file
FILES=$(find "${DOWNLOADS}" -maxdepth 1 -name "twsapi*.zip" 2>/dev/null || true)

if [ -z "${FILES}" ]; then
    echo "❌ No TWS API zip files found"
    echo "Please run: ./scripts/check_tws_download.sh"
    exit 1
fi

# Get most recent file
LATEST=$(ls -t ${FILES} | head -1)
echo "Found: ${LATEST}"
echo ""

# Create vendor directory
echo "Creating vendor directory..."
mkdir -p "${VENDOR_DIR}"

# Check if already extracted
if [ -d "${VENDOR_DIR}/IBJts" ]; then
    echo "⚠️  Warning: TWS API already exists in vendor/"
    read -p "Overwrite? (y/N) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        echo "Cancelled"
        exit 0
    fi
    echo "Removing old installation..."
    rm -rf "${VENDOR_DIR:?}"/*
fi

# Extract
echo "Extracting TWS API..."
unzip -q "${LATEST}" -d "${VENDOR_DIR}"

# Verify extraction
EXPECTED_PATH="${VENDOR_DIR}/IBJts/source/cppclient/client"

if [ ! -d "${EXPECTED_PATH}" ]; then
    echo "❌ Extraction failed: Expected directory not found"
    echo "Expected: ${EXPECTED_PATH}"
    echo ""
    echo "Directory structure:"
    find "${VENDOR_DIR}" -type d -maxdepth 3
    exit 1
fi

echo "✅ Extraction successful"
echo ""

# Show what was extracted
echo "Extracted contents:"
ls -la "${VENDOR_DIR}"
echo ""

# Verify critical files exist
echo "Checking for critical files..."
REQUIRED_FILES=(
    "${EXPECTED_PATH}/EClient.h"
    "${EXPECTED_PATH}/EWrapper.h"
    "${EXPECTED_PATH}/Contract.h"
    "${EXPECTED_PATH}/Order.h"
)

ALL_FOUND=true
for file in "${REQUIRED_FILES[@]}"; do
    if [ -f "${file}" ]; then
        echo "  ✅ $(basename ${file})"
    else
        echo "  ❌ $(basename ${file}) - NOT FOUND"
        ALL_FOUND=false
    fi
done

echo ""

if [ "${ALL_FOUND}" = false ]; then
    echo "❌ Some required files are missing"
    echo "The extraction may have failed or the archive structure is different"
    exit 1
fi

echo "════════════════════════════════════════════════════════════"
echo "  ✅ TWS API Installed Successfully!"
echo "════════════════════════════════════════════════════════════"
echo ""
echo "Location: ${VENDOR_DIR}"
echo ""
echo "Next steps:"
echo "  1. Rebuild project: ./scripts/build_universal.sh"
echo "  2. Verify CMake detects API (should see 'TWS API found' message)"
echo "  3. Proceed to Step 3: Implement TWS client"
echo ""
echo "Documentation:"
echo "  - Implementation guide: docs/IMPLEMENTATION_GUIDE.md"
echo "  - Integration template: docs/TWS_INTEGRATION_TEMPLATE.cpp"
echo "  - TWS API docs: ${VENDOR_DIR}/IBJts/Guides/"
echo ""
