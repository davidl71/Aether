#!/bin/bash
# LiveVol Quick Test Script
# Tests LiveVol API access and discovers quoted spread endpoints

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}LiveVol API Quick Test${NC}"
echo "================================"
echo ""

# Check if credentials are provided
if [ -z "$LIVEVOL_CLIENT_ID" ] || [ -z "$LIVEVOL_CLIENT_SECRET" ]; then
    echo -e "${YELLOW}⚠️  Credentials not found in environment variables${NC}"
    echo ""
    echo "Please provide credentials:"
    echo "  Option 1: Environment variables"
    echo "    export LIVEVOL_CLIENT_ID='your_client_id'"
    echo "    export LIVEVOL_CLIENT_SECRET='your_client_secret'"
    echo ""
    echo "  Option 2: Command line arguments"
    echo "    $0 --client-id YOUR_ID --client-secret YOUR_SECRET"
    echo ""

    # Check for command line arguments
    if [ "$1" == "--client-id" ] && [ -n "$2" ] && [ "$3" == "--client-secret" ] && [ -n "$4" ]; then
        export LIVEVOL_CLIENT_ID="$2"
        export LIVEVOL_CLIENT_SECRET="$4"
        echo -e "${GREEN}✅ Using credentials from command line${NC}"
    else
        echo -e "${RED}❌ No credentials provided. Exiting.${NC}"
        exit 1
    fi
else
    echo -e "${GREEN}✅ Credentials found in environment${NC}"
fi

echo ""
echo "Testing LiveVol API..."
echo ""

# Check if Python script exists
if [ ! -f "scripts/livevol_api_explorer.py" ]; then
    echo -e "${RED}❌ scripts/livevol_api_explorer.py not found${NC}"
    exit 1
fi

# Check if requests is installed
if ! python3 -c "import requests" 2>/dev/null; then
    echo -e "${YELLOW}⚠️  'requests' library not found. Installing...${NC}"
    pip3 install requests --quiet
fi

# Run the exploration script
echo -e "${GREEN}Running LiveVol API exploration...${NC}"
echo ""

python3 scripts/livevol_api_explorer.py \
    --client-id "$LIVEVOL_CLIENT_ID" \
    --client-secret "$LIVEVOL_CLIENT_SECRET" \
    --output-dir docs/livevol_exploration

echo ""
echo -e "${GREEN}✅ Exploration complete!${NC}"
echo ""
echo "Check results in: docs/livevol_exploration/"
echo "  - exploration_report_*.md (summary)"
echo "  - exploration_results.json (full results)"
echo "  - quoted_spread_tests.json (quoted spread tests)"
echo "  - qsb_tests.json (QSB instrument tests)"
echo ""
