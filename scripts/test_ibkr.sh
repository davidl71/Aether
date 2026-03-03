#!/bin/bash
# Helper script to test IBKR connection and positions
# Usage: ./scripts/test_ibkr.sh [connection|positions]

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Set library path for TWS API
export DYLD_LIBRARY_PATH=native/ibapi_cmake/build/lib:native/third_party/tws-api/IBJts/source/cppclient/client/build/lib

# Change to repo root if not already there
cd "$(dirname "$0")/.."

# Check if binaries exist
if [ ! -f "native/build_native/bin/test_tws_connection" ]; then
  echo -e "${RED}Error: test_tws_connection not found${NC}"
  echo "Please build first: ninja -C native/build_native test_tws_connection"
  exit 1
fi

if [ ! -f "native/build_native/bin/test_positions_live" ]; then
  echo -e "${RED}Error: test_positions_live not found${NC}"
  echo "Please build first: ninja -C native/build_native test_positions_live"
  exit 1
fi

# Function to show instructions
show_instructions() {
  echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
  echo -e "${BLUE}  IBKR Connection Setup Instructions${NC}"
  echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
  echo ""
  echo -e "${YELLOW}Before running the test:${NC}"
  echo "  1. Start IB Gateway or TWS"
  echo "  2. Enable API connections:"
  echo "     - Go to: Global Configuration → API → Settings"
  echo "     - Check: 'Enable ActiveX and Socket Clients'"
  echo "     - Check: 'Read-Only API' (optional, for safety)"
  echo "     - Note your port number:"
  echo "       • Paper Trading: 4002 (Gateway) or 7497 (TWS)"
  echo "       • Live Trading:  4001 (Gateway) or 7496 (TWS)"
  echo "  3. When prompted, click 'Accept' to allow the connection"
  echo ""
  echo -e "${YELLOW}Connection will timeout if not accepted within 30 seconds${NC}"
  echo ""
}

# Default to connection test
TEST_TYPE="${1:-connection}"

case "$TEST_TYPE" in
connection | conn | c)
  show_instructions
  echo -e "${GREEN}Running connection test...${NC}"
  echo ""
  ./native/build_native/bin/test_tws_connection "$@"
  ;;

positions | pos | p)
  show_instructions
  echo -e "${GREEN}Running positions retrieval test...${NC}"
  echo ""
  ./native/build_native/bin/test_positions_live "$@"
  ;;

*)
  echo -e "${RED}Unknown test type: $TEST_TYPE${NC}"
  echo ""
  echo "Usage: $0 [connection|positions]"
  echo ""
  echo "Examples:"
  echo "  $0 connection          # Test basic connection"
  echo "  $0 positions           # Test position retrieval"
  echo "  $0                     # Default: connection test"
  exit 1
  ;;
esac
