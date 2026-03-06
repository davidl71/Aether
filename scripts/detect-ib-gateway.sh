#!/usr/bin/env bash
# Detect if IB Gateway is running
# Returns 0 if running, 1 if not running

set -euo pipefail

# Colors
if [ -t 1 ] && command -v tput >/dev/null 2>&1; then
  GREEN=$(tput setaf 2)
  RED=$(tput setaf 1)
  NC=$(tput sgr0)
else
  GREEN=''
  RED=''
  NC=''
fi

GATEWAY_RUNNING=false

# Check gateway port (default 5001)
GATEWAY_PORT="${IB_GATEWAY_PORT:-5001}"
if lsof -ti ":${GATEWAY_PORT}" >/dev/null 2>&1; then
  # Check if it's actually the gateway by testing the API
  if curl -k -s --connect-timeout 2 "https://localhost:${GATEWAY_PORT}/sso/validate" >/dev/null 2>&1; then
    GATEWAY_RUNNING=true
  fi
fi

# Also check for gateway process
if pgrep -f "clientportal.gw\|ibgroup.web.core\|GatewayStart" >/dev/null 2>&1; then
  GATEWAY_RUNNING=true
fi

# Output results
if [ "$GATEWAY_RUNNING" = true ]; then
  echo "${GREEN}✓ IB Gateway is running${NC}"
  if [ "${1:-}" = "--verbose" ]; then
    echo "  Port ${GATEWAY_PORT}: In use"
    echo "  API endpoint: Responding"
    lsof -ti ":${GATEWAY_PORT}" 2>/dev/null | while read -r pid; do
      echo "  Process PID: $pid"
      ps -p "$pid" -o comm= 2>/dev/null | head -1 | xargs echo "  Process:"
    done
  fi
  exit 0
else
  echo "${RED}✗ IB Gateway is not running${NC}"
  if [ "${1:-}" = "--verbose" ]; then
    echo "  Port ${GATEWAY_PORT}: Free"
    echo "  API endpoint: Not responding"
    echo "  Process: Not found"
  fi
  exit 1
fi
