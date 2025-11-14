#!/bin/bash
# test_tws_connection.sh - Test TWS API connection using the C++ test program
# This script builds and runs the TWS connection test

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  TWS API Connection Test Script"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Parse arguments
HOST="${1:-127.0.0.1}"
PORT="${2:-4002}" # Default to IB Gateway Paper Trading (4002) instead of TWS Paper (7497)
CLIENT_ID="${3:-999}"

echo "Configuration:"
echo "  Host: $HOST"
echo "  Port: $PORT"
echo "  Client ID: $CLIENT_ID"
echo ""

# Check if TWS/Gateway is running
echo "Checking if TWS/IB Gateway is running..."
if lsof -i :""""""$P"O""R""T" >/dev/null 2>&1; then
  echo "✓ Port $PORT is listening"
  echo ""
  echo "Process details:"
  lsof -i :""""""$P"O""R""T" | head -5
  echo ""
else
  echo "✗ Port $PORT is not listening"
  echo ""
  echo "Please start TWS or IB Gateway and ensure:"
  echo "  1. API is enabled: File → Global Configuration → API → Settings"
  echo "  2. Port $PORT matches your configuration"
  echo ""
  echo "Port reference:"
  echo "  4002 = IB Gateway Paper Trading (default)"
  echo "  4001 = IB Gateway Live Trading"
  echo "  7497 = TWS Paper Trading"
  echo "  7496 = TWS Live Trading"
  echo ""
  exit 1
fi

# Build the test program
echo "Building test program..."
cd "$PROJECT_ROOT"

# Check if build directory exists
if [ ! -d "build" ]; then
  echo "Build directory not found. Please build the project first:"
  echo "  cmake --preset macos-x86_64-debug"
  echo "  cmake --build --preset macos-x86_64-debug"
  exit 1
fi

# Try to build the test program (if it's in CMakeLists.txt)
# Otherwise, compile directly
if [ -f "native/src/tws_client.cpp" ]; then
  echo "Test program should be built with the main project"
  echo "If you need to build it separately, check CMakeLists.txt"
  echo ""
fi

# Try to find test binary or main app in build directory
# Check multiple possible locations
TEST_BINARY=$(find "$PROJECT_ROOT/build" -name "test_tws_connection" -type f -executable 2>/dev/null | head -1)
if [ -z "$TEST_BINARY" ]; then
  TEST_BINARY="$PROJECT_ROOT/build/test_tws_connection"
  [ ! -f "$TEST_BINARY" ] && TEST_BINARY=""
fi

MAIN_BINARY=$(find "$PROJECT_ROOT/build" "$PROJECT_ROOT/build-ramdisk" -name "ib_box_spread" -type f -executable 2>/dev/null | head -1)
if [ -z "$MAIN_BINARY" ]; then
  # Check common build locations (RAM disk first, then regular build)
  for path in "$PROJECT_ROOT/build-ramdisk/bin/ib_box_spread" \
    "$PROJECT_ROOT/build-ramdisk/macos-x86_64-debug/bin/ib_box_spread" \
    "$PROJECT_ROOT/build/macos-x86_64-debug/bin/ib_box_spread" \
    "$PROJECT_ROOT/build/macos-x86_64-debug/native/ib_box_spread" \
    "$PROJECT_ROOT/build/macos-universal-debug/bin/ib_box_spread" \
    "$PROJECT_ROOT/build/macos-universal-debug/native/ib_box_spread" \
    "$PROJECT_ROOT/build/native/ib_box_spread" \
    "$PROJECT_ROOT/build/ib_box_spread" \
    "$PROJECT_ROOT/build/bin/ib_box_spread"; do
    if [ -f "$path" ] && [ -x "$path" ]; then
      MAIN_BINARY="$path"
      break
    fi
  done
fi

# Create temporary config file for testing
TEMP_CONFIG=$(mktemp -t test_tws_config_XXXXXX.json)
cat >"$TEMP_CONFIG" <<EOF
{
  "tws": {
    "host": "$HOST",
    "port": $PORT,
    "client_id": $CLIENT_ID,
    "connection_timeout_ms": 30000,
    "auto_reconnect": false
  },
  "strategy": {
    "symbols": ["SPY"],
    "min_arbitrage_profit": 0.10,
    "min_roi_percent": 0.5,
    "max_position_size": 10000.0,
    "min_days_to_expiry": 30,
    "max_days_to_expiry": 90,
    "max_bid_ask_spread": 0.10,
    "min_volume": 100,
    "min_open_interest": 500
  },
  "risk": {
    "max_total_exposure": 50000.0,
    "max_positions": 10,
    "max_loss_per_position": 1000.0,
    "max_daily_loss": 2000.0,
    "position_size_percent": 0.1,
    "enable_stop_loss": true,
    "stop_loss_percent": 0.2
  },
  "logging": {
    "log_level": "debug",
    "log_to_console": true
  },
  "dry_run": true,
  "loop_delay_ms": 1000,
  "continue_on_error": false
}
EOF

if [ -f "$TEST_BINARY" ]; then
  echo "Running connection test program..."
  echo ""
  "$TEST_BINARY" "$HOST" "$PORT" "$CLIENT_ID"
  TEST_RESULT=$?
  rm -f "$TEMP_CONFIG"
  exit $TEST_RESULT
elif [ -n "$MAIN_BINARY" ]; then
  echo "⚠ Test binary not found, using main application for connection test..."
  echo ""
  echo "Running connection test with main application..."
  echo "This will attempt to connect and then exit..."
  echo ""

  # Run with timeout and capture output
  timeout 15 "$MAIN_BINARY" --config "$TEMP_CONFIG" --log-level debug 2>&1 |
    grep -E "(Connection|connected|Port|error|Error|✓|✗|━━━|connectAck|managedAccounts|nextValidId)" ||
    timeout 15 "$MAIN_BINARY" --config "$TEMP_CONFIG" --log-level debug 2>&1 | head -50

  TEST_RESULT=${PIPESTATUS[0]}
  rm -f "$TEMP_CONFIG"

  if [ """"""$TEST_RES"U""L""T" -eq 124 ]; then
    echo ""
    echo "⚠ Test timed out - connection may be hanging"
    exit 1
  fi
  exit """"""$TEST_RES"U""L""T"
else
  echo "⚠ Test binary not found: $TEST_BINARY"
  echo "⚠ Main application not found either"
  echo ""
  echo "Please build the project first:"
  echo "  cmake --preset macos-x86_64-debug"
  echo "  cmake --build --preset macos-x86_64-debug"
  echo ""
  rm -f "$TEMP_CONFIG"
  exit 1
fi
