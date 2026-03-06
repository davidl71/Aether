#!/bin/bash
# IBKR Connection Diagnostic Script
# Shows exactly what's happening with your IB Gateway connection

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m'

export DYLD_LIBRARY_PATH=native/ibapi_cmake/build/lib:native/third_party/tws-api/IBJts/source/cppclient/client/build/lib

cd "$(dirname "$0")/.."

# Find test_diagnostic_connect in common build locations
DIAG_BIN=""
for candidate in \
  "build/bin/test_diagnostic_connect" \
  "build/macos-arm64-debug/bin/test_diagnostic_connect" \
  "build/macos-x86_64-debug/bin/test_diagnostic_connect" \
  "native/build_native/bin/test_diagnostic_connect"; do
  if [ -f "$candidate" ]; then
    DIAG_BIN="$candidate"
    break
  fi
done

echo -e "${CYAN}╔════════════════════════════════════════════════════════════════╗${NC}"
echo -e "${CYAN}║        IBKR Connection Diagnostic Tool                        ║${NC}"
echo -e "${CYAN}╚════════════════════════════════════════════════════════════════╝${NC}"
echo ""

# Check prerequisites (optional: script can still do port checks without binary)
echo -e "${BLUE}[1/5] Checking prerequisites...${NC}"
if [ -n "$DIAG_BIN" ]; then
  echo -e "${GREEN}✓ Diagnostic test binary found: $DIAG_BIN${NC}"
else
  echo -e "${YELLOW}⚠ Diagnostic test binary not built (optional)${NC}"
  echo "  Port check will run; for full connection test build first:"
  echo "  just build  # or: cmake --build build --target test_diagnostic_connect"
fi

# Check if Gateway is running
echo ""
echo -e "${BLUE}[2/5] Checking IB Gateway process...${NC}"
if pgrep -fi "ibgateway|java.*tws" >/dev/null; then
  echo -e "${GREEN}✓ IB Gateway/TWS is running${NC}"
else
  echo -e "${YELLOW}⚠  Cannot find IB Gateway process${NC}"
  echo "  If Gateway is running, this may be a false negative"
fi

# Check ports
echo ""
echo -e "${BLUE}[3/5] Checking API ports...${NC}"
PAPER_PORT_OPEN=false
LIVE_PORT_OPEN=false

if lsof -i :4002 | grep -q LISTEN; then
  echo -e "${GREEN}✓ Port 4002 (IB Gateway Paper Trading) is OPEN${NC}"
  PAPER_PORT_OPEN=true
fi

if lsof -i :4001 | grep -q LISTEN; then
  echo -e "${GREEN}✓ Port 4001 (IB Gateway Live Trading) is OPEN${NC}"
  LIVE_PORT_OPEN=true
fi

if lsof -i :7497 | grep -q LISTEN; then
  echo -e "${GREEN}✓ Port 7497 (TWS Paper Trading) is OPEN${NC}"
  PAPER_PORT_OPEN=true
fi

if lsof -i :7496 | grep -q LISTEN; then
  echo -e "${GREEN}✓ Port 7496 (TWS Live Trading) is OPEN${NC}"
  LIVE_PORT_OPEN=true
fi

if ! $PAPER_PORT_OPEN && ! $LIVE_PORT_OPEN; then
  echo -e "${RED}✗ No API ports are open${NC}"
  echo ""
  echo "  Please enable API in IB Gateway:"
  echo "  Configure → Settings → API → Settings"
  echo "  Check: ☑ Enable ActiveX and Socket Clients"
  exit 1
fi

# Show what we'll test
echo ""
echo -e "${BLUE}[4/5] Configuration${NC}"
if $LIVE_PORT_OPEN; then
  echo "  Testing: Port 4001 (Live Trading)"
  TEST_CLIENT_ID=777
else
  echo "  Testing: Port 7497 or 4002 (Paper Trading)"
  TEST_CLIENT_ID=777
fi
echo "  Client ID: $TEST_CLIENT_ID"

# Run diagnostic (only if binary exists)
echo ""
echo -e "${BLUE}[5/5] Running connection diagnostic...${NC}"
if [ -z "$DIAG_BIN" ]; then
  echo -e "${YELLOW}Skipped (build test_diagnostic_connect for full handshake test).${NC}"
  echo ""
  echo -e "${GREEN}Port check passed. Use: ./scripts/test_tws_connection.sh <port> to test TCP.${NC}"
  exit 0
fi

echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""
echo -e "${YELLOW}👀 Watch for these key messages:${NC}"
echo "  • connectAck received ←  Initial handshake"
echo "  • managedAccounts received ← Gateway accepts connection"
echo "  • nextValidId received ← Ready for trading"
echo ""
echo -e "${YELLOW}If you see 'Connection closed by TWS' after connectAck:${NC}"
echo "  → Gateway is rejecting because 'auto-accept' is disabled"
echo "  → Check IB Gateway window for approval dialog"
echo ""
echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""
sleep 2

# Run the test and capture output
OUTPUT_FILE=$(mktemp)
if ./"$DIAG_BIN" "$TEST_CLIENT_ID" 2>&1 | tee "$OUTPUT_FILE"; then
  :
else
  :
fi

echo ""
echo -e "${CYAN}╔════════════════════════════════════════════════════════════════╗${NC}"
echo -e "${CYAN}║        Diagnostic Results                                     ║${NC}"
echo -e "${CYAN}╚════════════════════════════════════════════════════════════════╝${NC}"
echo ""

# Analyze output
CONNECT_ACK=$(grep -c "connectAck received" "$OUTPUT_FILE" || echo "0")
MANAGED_ACCOUNTS=$(grep -c "managedAccounts received" "$OUTPUT_FILE" || echo "0")
NEXT_VALID_ID=$(grep -c "nextValidId received" "$OUTPUT_FILE" || echo "0")
CONNECTION_CLOSED=$(grep -c "Connection closed by TWS" "$OUTPUT_FILE" || echo "0")

echo "Callbacks received:"
if [ "$CONNECT_ACK" -gt 0 ]; then
  echo -e "  ${GREEN}✓ connectAck${NC}"
else
  echo -e "  ${RED}✗ connectAck${NC}"
fi

if [ "$MANAGED_ACCOUNTS" -gt 0 ]; then
  echo -e "  ${GREEN}✓ managedAccounts${NC}"
else
  echo -e "  ${RED}✗ managedAccounts${NC}"
fi

if [ "$NEXT_VALID_ID" -gt 0 ]; then
  echo -e "  ${GREEN}✓ nextValidId${NC}"
else
  echo -e "  ${RED}✗ nextValidId${NC}"
fi

echo ""

# Provide specific diagnosis
if [ "$CONNECT_ACK" -gt 0 ] && [ "$CONNECTION_CLOSED" -gt 0 ] && [ "$MANAGED_ACCOUNTS" -eq 0 ]; then
  echo -e "${RED}╔════════════════════════════════════════════════════════════════╗${NC}"
  echo -e "${RED}║  DIAGNOSIS: Gateway Rejecting Connection                     ║${NC}"
  echo -e "${RED}╚════════════════════════════════════════════════════════════════╝${NC}"
  echo ""
  echo "Your IB Gateway is actively rejecting API connections."
  echo ""
  echo -e "${YELLOW}SOLUTION:${NC}"
  echo "  1. Open IB Gateway"
  echo "  2. Click: Configure → Settings"
  echo "  3. Navigate to: API → Settings"
  echo "  4. Enable these checkboxes:"
  echo "     ☑ Enable ActiveX and Socket Clients"
  echo "     ☑ Accept incoming connection requests automatically"
  echo "  5. Add to 'Trusted IPs': 127.0.0.1"
  echo "  6. Click OK"
  echo "  7. Restart IB Gateway"
  echo ""
  echo -e "${CYAN}After making these changes, run this script again.${NC}"

elif [ "$NEXT_VALID_ID" -gt 0 ]; then
  echo -e "${GREEN}╔════════════════════════════════════════════════════════════════╗${NC}"
  echo -e "${GREEN}║  SUCCESS: Connection Fully Established!                      ║${NC}"
  echo -e "${GREEN}╚════════════════════════════════════════════════════════════════╝${NC}"
  echo ""
  echo "Your IBKR connection is working perfectly!"
  echo ""
  echo -e "${CYAN}Next steps:${NC}"
  echo "  • Test position retrieval:"
  echo "    ./native/build_native/bin/test_positions_live"
  echo ""
  echo "  • Run the main CLI:"
  echo "    ./native/build_native/bin/ib_box_spread"

elif [ "$CONNECT_ACK" -eq 0 ]; then
  echo -e "${RED}╔════════════════════════════════════════════════════════════════╗${NC}"
  echo -e "${RED}║  DIAGNOSIS: Cannot Reach API                                  ║${NC}"
  echo -e "${RED}╚════════════════════════════════════════════════════════════════╝${NC}"
  echo ""
  echo "The TWS API is not responding to connection attempts."
  echo ""
  echo "Possible causes:"
  echo "  • API not enabled in Gateway settings"
  echo "  • Wrong port number"
  echo "  • Firewall blocking connection"
  echo ""
  echo "Check: Configure → Settings → API → Settings"
  echo "Ensure: ☑ Enable ActiveX and Socket Clients"

else
  echo -e "${YELLOW}╔════════════════════════════════════════════════════════════════╗${NC}"
  echo -e "${YELLOW}║  DIAGNOSIS: Partial Connection                                ║${NC}"
  echo -e "${YELLOW}╚════════════════════════════════════════════════════════════════╝${NC}"
  echo ""
  echo "Connection started but didn't complete."
  echo "Review the diagnostic output above for details."
fi

rm -f "$OUTPUT_FILE"

echo ""
