#!/bin/bash
# TWS Connection Troubleshooting Script
#
# Usage:
#   ./scripts/test_tws_connection.sh           # Detect all IB ports (4001, 4002, 7496, 7497)
#   ./scripts/test_tws_connection.sh 7497     # Test single port (e.g. paper)

set -e

HOST="127.0.0.1"

# Color codes
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Standard IB/TWS API ports: Gateway 4001/4002, TWS 7496/7497
check_port() {
  local port=$1
  if lsof -iTCP:${port} -sTCP:LISTEN >/dev/null 2>&1; then
    echo -e "${GREEN}✓ Port ${port} is listening${NC}"
    return 0
  else
    echo -e "${RED}✗ Port ${port} is NOT listening${NC}"
    return 1
  fi
}

# No argument: detect and report all four ports
if [ $# -eq 0 ]; then
  echo "=================================================="
  echo "TWS/Gateway Port Detection"
  echo "=================================================="
  echo ""
  echo "Reference:"
  echo "  4002 = IB Gateway Paper Trading"
  echo "  4001 = IB Gateway Live Trading"
  echo "  7497 = TWS Paper Trading"
  echo "  7496 = TWS Live Trading"
  echo ""
  echo "Checking ports..."
  OPEN_COUNT=0
  check_port 4002 && OPEN_COUNT=$((OPEN_COUNT + 1)) || true
  check_port 4001 && OPEN_COUNT=$((OPEN_COUNT + 1)) || true
  check_port 7497 && OPEN_COUNT=$((OPEN_COUNT + 1)) || true
  check_port 7496 && OPEN_COUNT=$((OPEN_COUNT + 1)) || true
  echo ""
  if [ "$OPEN_COUNT" -eq 0 ]; then
    echo -e "${RED}No API ports are open. Start IB Gateway or TWS and enable API.${NC}"
    exit 1
  fi
  echo -e "${GREEN}${OPEN_COUNT} port(s) open. TWS/Gateway is reachable.${NC}"
  echo ""
  echo "To test a specific port (e.g. TCP connect): $0 <port>"
  exit 0
fi

# One argument: test single port (original behavior)
TEST_PORT=$1
echo "=================================================="
echo "TWS Connection Troubleshooting"
echo "=================================================="
echo ""
echo "Testing connection to: ${HOST}:${TEST_PORT}"
if [ "$TEST_PORT" = "7497" ] || [ "$TEST_PORT" = "4002" ]; then
  echo -e "${GREEN}Mode: Paper Trading (safe)${NC}"
else
  echo -e "${RED}Mode: LIVE TRADING${NC}"
fi
echo ""

echo "Test 1: Checking if TWS/Gateway is running..."
if ! check_port "$TEST_PORT"; then
  echo ""
  echo "Solution: Start IB Gateway or TWS first"
  echo ""
  exit 1
fi

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
