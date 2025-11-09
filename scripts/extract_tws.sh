#!/bin/bash
# extract_tws.sh - Extract TWS API to native/third_party directory

set -euo pipefail

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
readonly PROJECT_ROOT
TWS_DIR="${PROJECT_ROOT}/native/third_party/tws-api"
readonly TWS_DIR
DOWNLOADS="${HOME}/Downloads"
readonly DOWNLOADS

echo "════════════════════════════════════════════════════════════"
echo "  TWS API Extraction"
echo "════════════════════════════════════════════════════════════"
echo ""

# Find TWS API zip file
files=()
while IFS= read -r -d '' file; do
    files+=("${file}")
done < <(find "${DOWNLOADS}" -maxdepth 1 -name "twsapi*.zip" -print0 2>/dev/null)

if [ ${#files[@]} -eq 0 ]; then
    echo "❌ No TWS API zip files found"
    echo "Please run: ./scripts/check_tws_download.sh"
    exit 1
fi

# Get most recent file by modification time
LATEST=""
LATEST_MTIME=0
for candidate in "${files[@]}"; do
    mtime=$(stat -f %m "${candidate}" 2>/dev/null || stat -c %Y "${candidate}")
    if [ "${mtime}" -gt "${LATEST_MTIME}" ]; then
        LATEST_MTIME="${mtime}"
        LATEST="${candidate}"
    fi
done

echo "Found: ${LATEST}"
echo ""

# Create third-party directory
echo "Creating native third_party directory..."
mkdir -p "${TWS_DIR}"

# Check if already extracted
if [ -d "${TWS_DIR}/IBJts" ]; then
    echo "⚠️  Warning: TWS API already exists in native/third_party/"
    read -p "Overwrite? (y/N) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        echo "Cancelled"
        exit 0
    fi
    echo "Removing old installation..."
    rm -rf "${TWS_DIR:?}"/*
fi

# Extract
echo "Extracting TWS API..."
unzip -q "${LATEST}" -d "${TWS_DIR}"

# Verify extraction
EXPECTED_PATH="${TWS_DIR}/IBJts/source/cppclient/client"

if [ ! -d "${EXPECTED_PATH}" ]; then
    echo "❌ Extraction failed: Expected directory not found"
    echo "Expected: ${EXPECTED_PATH}"
    echo ""
    echo "Directory structure:"
    find "${TWS_DIR}" -type d -maxdepth 3
    exit 1
fi

echo "✅ Extraction successful"
echo ""

# Show what was extracted
echo "Extracted contents:"
ls -la "${TWS_DIR}"
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
        echo "  ✅ $(basename "${file}")"
    else
        echo "  ❌ $(basename "${file}") - NOT FOUND"
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
echo "Location: ${TWS_DIR}"
echo ""
echo "Next steps:"
echo "  1. Rebuild project: ./scripts/build_universal.sh"
echo "  2. Verify CMake detects API (should see 'TWS API found' message)"
echo "  3. Proceed to Step 3: Implement TWS client"
echo ""
echo "Documentation:"
echo "  - Implementation guide: docs/IMPLEMENTATION_GUIDE.md"
echo "  - Integration template: docs/TWS_INTEGRATION_TEMPLATE.cpp"
echo "  - TWS API docs: ${TWS_DIR}/IBJts/Guides/"
echo ""
