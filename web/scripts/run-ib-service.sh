#!/usr/bin/env bash
# Run IB (Interactive Brokers) service for PWA integration
# Uses IB Client Portal Gateway (must be running separately)
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/../.." && pwd)"
PYTHON_DIR="$ROOT_DIR/python"
SCRIPTS_DIR="${ROOT_DIR}/scripts"

# Load shared utility functions
# shellcheck source=../../scripts/include/config.sh
if [ -f "${SCRIPTS_DIR}/include/config.sh" ]; then
  source "${SCRIPTS_DIR}/include/config.sh"
fi

# shellcheck source=../../scripts/include/python_utils.sh
if [ -f "${SCRIPTS_DIR}/include/python_utils.sh" ]; then
  source "${SCRIPTS_DIR}/include/python_utils.sh"
else
  echo "Error: python_utils.sh not found" >&2
  exit 1
fi

# shellcheck source=../../scripts/include/service_utils.sh
if [ -f "${SCRIPTS_DIR}/include/service_utils.sh" ]; then
  source "${SCRIPTS_DIR}/include/service_utils.sh"
fi

cd "$PYTHON_DIR"

# Find Python command
find_python || exit 1

# Set up virtual environment
setup_venv "${PYTHON_DIR}" || exit 1

# Install required packages
install_python_packages "${VENV_PYTHON}" "uvicorn[standard]" "fastapi" "requests" || exit 1

# Use venv Python for all subsequent operations
PYTHON_CMD="${VENV_PYTHON}"

# Check if integration module file exists
if [ ! -f "integration/ib_service.py" ]; then
  echo "Error: integration/ib_service.py not found. Current directory: $(pwd)" >&2
  exit 1
fi

# Optional: Check if IB Client Portal Gateway is running
check_ib_gateway() {
  local portal_url="${1:-https://localhost:5000}"
  if command -v curl >/dev/null 2>&1; then
    # Check if gateway is accessible (ignore SSL errors)
    if curl -k -s --connect-timeout 2 "${portal_url}/sso/validate" >/dev/null 2>&1; then
      return 0
    fi
  else
    "${PYTHON_CMD}" -c "
import urllib.request
import ssl
import sys
try:
    ctx = ssl.create_default_context()
    ctx.check_hostname = False
    ctx.verify_mode = ssl.CERT_NONE
    with urllib.request.urlopen('${portal_url}/sso/validate', timeout=2, context=ctx) as response:
        sys.exit(0)
except Exception:
    sys.exit(1)
" 2>/dev/null && return 0
  fi
  return 1
}

IB_PORTAL_URL="${IB_PORTAL_URL:-https://localhost:5000/v1/portal}"
IB_GATEWAY_BASE="${IB_PORTAL_URL%/v1/portal}"

if ! check_ib_gateway "${IB_GATEWAY_BASE}"; then
  echo "⚠ Warning: IB Client Portal Gateway does not appear to be running at ${IB_GATEWAY_BASE}" >&2
  echo "  The service will start, but API calls will fail until the gateway is running." >&2
  echo "" >&2
  echo "  To start IB Client Portal Gateway:" >&2
  echo "  1. Download: https://download2.interactivebrokers.com/portal/clientportal.gw.zip" >&2
  echo "     Or visit: https://www.interactivebrokers.com/en/index.php?f=16457" >&2
  echo "  2. Extract the ZIP file" >&2
  echo "  3. Run the gateway application (usually bin/run.sh from the Gateway package)" >&2
  echo "  4. Log in via browser (usually opens automatically)" >&2
  echo "" >&2
  echo "  Documentation:" >&2
  echo "  - Setup Guide: https://www.interactivebrokers.com/campus/trading-lessons/launching-and-authenticating-the-gateway/" >&2
  echo "  - API Docs: https://interactivebrokers.github.io/cpwebapi/" >&2
  echo "  - IB API: https://www.interactivebrokers.com/en/trading/ib-api.php" >&2
  echo "" >&2
else
  echo "✓ IB Client Portal Gateway detected at ${IB_GATEWAY_BASE}" >&2
fi

# Get IB service port from config (default: 8002 to avoid conflict with Alpaca)
IB_PORT=$(config_get_port "ib" 8002)

# Check if port is available and verify service identity
# IB service has special health check that looks for 'ib_connected' in response
if ! config_check_port_available "${IB_PORT}"; then
  echo "Port ${IB_PORT} is already in use. Checking if it's the IB service..." >&2

  # Custom health check for IB service (checks for 'ib_connected' in response)
  IB_HEALTH_CHECK=$("${PYTHON_CMD}" -c "
import urllib.request
import json
import sys
try:
    with urllib.request.urlopen('http://127.0.0.1:${IB_PORT}/api/health', timeout=2) as response:
        data = json.loads(response.read().decode())
        if data.get('status') == 'ok' and 'ib_connected' in data:
            print('IB_SERVICE')
        else:
            print('OTHER_SERVICE')
except Exception:
    print('OTHER_SERVICE')
" 2>/dev/null || echo "OTHER_SERVICE")

  if [ "${IB_HEALTH_CHECK}" = "IB_SERVICE" ]; then
    echo "✓ IB service is already running on http://127.0.0.1:${IB_PORT}" >&2
    echo "  Using existing service. No need to start a new one." >&2
    echo "  Set VITE_API_URL=http://127.0.0.1:${IB_PORT}/api/snapshot in your web app" >&2
    echo "" >&2
    exit 0
  else
    echo "Error: Port ${IB_PORT} is in use by another service (not IB service)" >&2
    echo "  Please stop the service on port ${IB_PORT} or use a different port:" >&2
    echo "  export IB_PORT=<different_port>" >&2
    echo "  Or update config/config.json: services.ib.port" >&2
    exit 1
  fi
fi

# Test import before starting service
if ! test_python_import "${PYTHON_CMD}" "integration.ib_service" "app"; then
  echo "Error: Cannot import ib_service" >&2
  echo "" >&2
  echo "Press any key to continue..." >&2
  read -n 1 -s || true
  exit 1
fi

echo "Starting IB service on http://127.0.0.1:${IB_PORT}" >&2
echo "IB Client Portal URL: ${IB_PORTAL_URL}" >&2
echo "Set VITE_API_URL=http://127.0.0.1:${IB_PORT}/api/snapshot in your web app" >&2
echo "" >&2
echo "Optional environment variables:" >&2
echo "  SYMBOLS=SPY,QQQ,IWM  # Comma-separated symbols (default: SPY,QQQ)" >&2
echo "  IB_PORTAL_URL=${IB_PORTAL_URL}  # IB Client Portal URL" >&2
echo "  IB_PORT=${IB_PORT}  # Service port (override config)" >&2
echo "  SNAPSHOT_FILE_PATH=/path/to/snapshot.json  # Optional file output" >&2
echo "" >&2

# Temporarily disable __init__.py to avoid dependency issues
disable_init_py "${PYTHON_DIR}" || exit 1

# Run the service with PYTHONPATH set
# Use trap to handle failures gracefully (don't kill tmux)
export PYTHONPATH="${PYTHON_DIR}:${PYTHONPATH:-}"
if ! "${PYTHON_CMD}" -m uvicorn integration.ib_service:app --host 127.0.0.1 --port "${IB_PORT}" --reload; then
  EXIT_CODE=$?
  echo "" >&2
  echo "⚠ IB service exited with error code ${EXIT_CODE}" >&2
  echo "Press any key to continue..." >&2
  read -n 1 -s || true
  exit "${EXIT_CODE}"
fi
