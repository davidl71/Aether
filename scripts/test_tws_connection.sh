#!/bin/bash
# TWS Connection Troubleshooting Script

set -e

echo "=================================================="
echo "TWS Connection Troubleshooting"
echo "=================================================="
echo ""

HOST="127.0.0.1"
TEST_PORT=${1:-7497}

# Color codes
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo "Testing connection to: ${HOST}:${TEST_PORT}"
if [ "$TEST_PORT" = "7497" ]; then
    echo -e "${GREEN}Mode: Paper Trading (safe)${NC}"
else
    echo -e "${RED}Mode: LIVE TRADING${NC}"
fi
echo ""

# Test 1: Check if port is listening
echo "Test 1: Checking if TWS/Gateway is running..."
if lsof -iTCP:${TEST_PORT} -sTCP:LISTEN >/dev/null 2>&1; then
    echo -e "${GREEN}✓ Port ${TEST_PORT} is listening${NC}"
else
    echo -e "${RED}✗ Port ${TEST_PORT} is NOT listening${NC}"
    echo ""
    echo "Solution: Start IB Gateway or TWS first"
    echo ""
    exit 1
fi

# Test 2: Test TCP connection
echo "Test 2: Testing TCP connection..."
if nc -zv ${HOST} ${TEST_PORT} 2>&1 | grep -q succeeded; then
    echo -e "${GREEN}✓ TCP connection successful${NC}"
else
    echo -e "${RED}✗ TCP connection failed${NC}"
    exit 1
fi

echo ""
echo -e "${GREEN}All tests passed! TWS is ready.${NC}"
echo ""
echo "Next steps:"
echo "1. Run: ./ib_box_spread --config config/config.example.json --dry-run --log-level debug"
echo "2. Accept connection in TWS popup (if prompted)"
echo "3. Check logs for 'Received nextValidId' message"
