#!/bin/bash

# Integration Test Script for IBKR Box Spread Generator
# Tests TWS API connectivity and basic operations

set -e

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$PROJECT_ROOT"

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

# Check if binary exists
if [ ! -f "build/bin/ib_box_spread" ]; then
    log_error "Binary not found. Run ./scripts/build_universal.sh first"
    exit 1
fi

# Check if config exists
if [ ! -f "config/config.json" ]; then
    log_error "Configuration file not found: config/config.json"
    exit 1
fi

log_info "IBKR Box Spread Generator - Integration Tests"
echo "================================================"
echo ""

# Test 1: Configuration Validation
log_info "Test 1: Validating configuration..."
if build/bin/ib_box_spread --config config/config.json --validate > /dev/null 2>&1; then
    echo "  ✓ Configuration validation passed"
else
    log_error "Configuration validation failed"
    exit 1
fi
echo ""

# Test 2: Check TWS Connectivity
log_info "Test 2: Checking TWS/Gateway connectivity..."
TWS_PORT=$(grep -o '"port":[[:space:]]*[0-9]*' config/config.json | grep -o '[0-9]*')
if netstat -an | grep -q "\.${TWS_PORT}"; then
    echo "  ✓ TWS/Gateway detected on port ${TWS_PORT}"
    TWS_RUNNING=true
else
    log_warn "TWS/Gateway not detected on port ${TWS_PORT}"
    echo "    To start TWS paper trading:"
    echo "    1. Open Interactive Brokers Trader Workstation (TWS)"
    echo "    2. Log in with paper trading credentials"
    echo "    3. Enable API: File > Global Configuration > API > Settings"
    echo "    4. Check 'Enable ActiveX and Socket Clients'"
    echo "    5. Socket port should be ${TWS_PORT}"
    TWS_RUNNING=false
fi
echo ""

# Test 3: Dry Run Test
log_info "Test 3: Testing dry-run mode..."
timeout 5 build/bin/ib_box_spread --config config/config.json --dry-run 2>&1 | head -20 || true
echo ""

if [ "$TWS_RUNNING" = true ]; then
    log_info "Test 4: Testing TWS connection..."
    echo "  This will attempt to connect to TWS for 10 seconds..."
    timeout 10 build/bin/ib_box_spread --config config/config.json 2>&1 | grep -E "Connected|Connection|Error" | head -10 || true
    echo ""
    
    log_info "Integration tests completed with TWS connectivity"
else
    log_warn "Skipping TWS connection test (TWS not running)"
    echo ""
    log_info "Integration tests completed (partial - no TWS)"
fi

echo ""
echo "================================================"
log_info "Integration Test Summary"
echo "================================================"
echo "✓ Binary executable"
echo "✓ Configuration valid"
if [ "$TWS_RUNNING" = true ]; then
    echo "✓ TWS/Gateway running"
else
    echo "⚠ TWS/Gateway not running (manual test required)"
fi
echo ""
log_info "Next steps for full integration testing:"
echo "  1. Start TWS paper trading"
echo "  2. Run: ./scripts/integration_test.sh"
echo "  3. Monitor logs: tail -f logs/ib_box_spread.log"
echo ""
