#!/bin/bash
# Quick test for live position retrieval
# Uses a random high client ID to avoid conflicts

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Set library path
export DYLD_LIBRARY_PATH=native/ibapi_cmake/build/lib:native/third_party/tws-api/IBJts/source/cppclient/client/build/lib

cd "$(dirname "$0")/.."

# Generate random client ID to avoid conflicts
CLIENT_ID=$((RANDOM + 10000))

echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}  IBKR Live Position Retrieval Test${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""
echo -e "${YELLOW}Using unique client ID: ${CLIENT_ID}${NC}"
echo ""
echo -e "${YELLOW}Checking IB Gateway status...${NC}"

# Check if port 4001 is open
if ! lsof -i :4001 | grep -q LISTEN; then
  echo -e "${RED}Error: IB Gateway not listening on port 4001${NC}"
  echo "Please start IB Gateway (Live Trading) and enable API"
  exit 1
fi

echo -e "${GREEN}✓ IB Gateway is running on port 4001${NC}"
echo ""

# Check API settings
echo -e "${YELLOW}Important: Verify these API settings in IB Gateway:${NC}"
echo "  1. Configure → Settings → API → Settings"
echo "  2. ✓ Enable ActiveX and Socket Clients"
echo "  3. ✓ Allow connections from localhost"
echo "  4. Socket port: 4001"
echo "  5. (Optional) Trusted IPs: 127.0.0.1"
echo "  6. (Optional) Read-Only API (safer for testing)"
echo ""
echo -e "${YELLOW}Press Enter to continue or Ctrl+C to cancel...${NC}"
read -r

echo -e "${GREEN}Running test with client ID ${CLIENT_ID}...${NC}"
echo ""

# Run the test - note: this will auto-detect and use port 4001
# The test program itself handles port detection and connection
./native/build_native/bin/test_positions_live

echo ""
echo -e "${GREEN}✓ Test complete${NC}"
