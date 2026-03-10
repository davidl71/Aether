#!/usr/bin/env bash
# Run web PWA service (Vite dev server)
# Automatically configures connection to Alpaca service if available
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/../.." && pwd)"
WEB_DIR="$ROOT_DIR/web"

# Load shared configuration if available
SCRIPTS_DIR_ROOT="${ROOT_DIR}/scripts"
if [ -f "${SCRIPTS_DIR_ROOT}/include/config.sh" ]; then
  # shellcheck source=../../scripts/include/config.sh
  source "${SCRIPTS_DIR_ROOT}/include/config.sh"
fi

# Helper function to get service port (with fallback to config_get_port if available)
get_service_port() {
  local service_name="${1:-}"
  local default_port="${2:-}"

  if command -v config_get_port >/dev/null 2>&1; then
    config_get_port "${service_name}" "${default_port}"
  else
    echo "${default_port}"
  fi
}

cd "$WEB_DIR"

# Check if Node.js is available
if ! command -v node >/dev/null 2>&1; then
  echo "Error: Node.js is not installed or not in PATH" >&2
  echo "Please install Node.js: https://nodejs.org/" >&2
  exit 1
fi

# Check if npm is available
if ! command -v npm >/dev/null 2>&1; then
  echo "Error: npm is not installed or not in PATH" >&2
  exit 1
fi

# Check if node_modules exists, install if needed
if [ ! -d "node_modules" ]; then
  echo "Installing npm dependencies..." >&2
  npm install
fi

# Check if backend service is running (Alpaca or IB)
check_backend_service() {
  local port="${1:-8000}"
  if command -v python3 >/dev/null 2>&1; then
    python3 -c "
import urllib.request
import json
import sys
try:
    with urllib.request.urlopen('http://127.0.0.1:${port}/api/health', timeout=1) as response:
        data = json.loads(response.read().decode())
        if data.get('status') == 'ok':
            # Check if it's Alpaca or IB service
            if 'alpaca_connected' in data:
                sys.exit(1)  # Alpaca service
            elif 'ib_connected' in data:
                sys.exit(2)  # IB service
            else:
                sys.exit(0)  # Unknown but OK
        else:
            sys.exit(1)
except Exception:
    sys.exit(1)
" 2>/dev/null
  else
    return 1
  fi
}

# Check if port 5173 (default Vite port) is in use
check_port() {
  local port="${1:-5173}"
  if command -v lsof >/dev/null 2>&1; then
    lsof -ti ":${port}" >/dev/null 2>&1
  elif command -v netstat >/dev/null 2>&1; then
    netstat -an 2>/dev/null | grep -q ":${port}.*LISTEN"
  else
    # Fallback: try to connect
    node -e "const net = require('net'); const s = new net.Socket(); s.setTimeout(100); s.on('connect', () => { s.destroy(); process.exit(0); }); s.on('timeout', () => { s.destroy(); process.exit(1); }); s.on('error', () => { process.exit(1); }); s.connect(${port}, '127.0.0.1');" 2>/dev/null
  fi
}

# Get service ports from config
ALPACA_PORT=$(get_service_port "alpaca" 8000)
IB_PORT=$(get_service_port "ib" 8002)
DISCOUNT_BANK_PORT=$(get_service_port "discount_bank" 8003)
RISK_FREE_RATE_PORT=$(get_service_port "risk_free_rate" 8004)
TASTYTRADE_PORT=$(get_service_port "tastytrade" 8005)
# Rust backend uses nested config path, so use config_get directly
if command -v config_get >/dev/null 2>&1; then
  RUST_BACKEND_REST_PORT=$(config_get ".services.rust_backend.rest_port" 8080)
else
  RUST_BACKEND_REST_PORT=8080
fi

# Check if backend service is running (try Alpaca first, then IB)
BACKEND_SERVICE_TYPE="none"
BACKEND_PORT="${ALPACA_PORT}"
SERVICE_CHECK_RESULT=$(check_backend_service "${ALPACA_PORT}" || echo $?)
case "${SERVICE_CHECK_RESULT}" in
  1)
    BACKEND_SERVICE_TYPE="alpaca"
    BACKEND_PORT="${ALPACA_PORT}"
    echo "✓ Alpaca service detected on http://127.0.0.1:${ALPACA_PORT}" >&2
    ;;
  2)
    BACKEND_SERVICE_TYPE="ib"
    BACKEND_PORT="${ALPACA_PORT}"  # IB also uses Alpaca port when running
    echo "✓ IB service detected on http://127.0.0.1:${ALPACA_PORT}" >&2
    ;;
  0)
    BACKEND_SERVICE_TYPE="unknown"
    BACKEND_PORT="${ALPACA_PORT}"
    echo "✓ Backend service detected on http://127.0.0.1:${ALPACA_PORT}" >&2
    ;;
  *)
    # Try IB port as fallback
    IB_CHECK_RESULT=$(check_backend_service "${IB_PORT}" || echo $?)
    if [ "${IB_CHECK_RESULT}" = "2" ]; then
      BACKEND_SERVICE_TYPE="ib"
      BACKEND_PORT="${IB_PORT}"
      echo "✓ IB service detected on http://127.0.0.1:${IB_PORT}" >&2
    else
      BACKEND_SERVICE_TYPE="none"
    fi
    ;;
esac

# Set up environment file with port configuration
ENV_FILE=".env"
ENV_UPDATED=false

# Function to update or add env var in .env file
update_env_var() {
  local var_name="${1}"
  local var_value="${2}"

  if [ -f "${ENV_FILE}" ]; then
    # Remove existing entry if present
    grep -v "^${var_name}=" "${ENV_FILE}" > "${ENV_FILE}.tmp" 2>/dev/null || true
    mv "${ENV_FILE}.tmp" "${ENV_FILE}" 2>/dev/null || true
    # Append new value
    echo "${var_name}=${var_value}" >> "${ENV_FILE}"
  else
    echo "${var_name}=${var_value}" > "${ENV_FILE}"
  fi
  ENV_UPDATED=true
}

# Update VITE_API_URL if backend service is detected
if [ ! -f "${ENV_FILE}" ] || ! grep -q "VITE_API_URL" "${ENV_FILE}" 2>/dev/null; then
  if [ "${BACKEND_SERVICE_TYPE}" != "none" ]; then
    if [ "${BACKEND_SERVICE_TYPE}" = "alpaca" ]; then
      echo "Configuring VITE_API_URL to connect to Alpaca service..." >&2
    elif [ "${BACKEND_SERVICE_TYPE}" = "ib" ]; then
      echo "Configuring VITE_API_URL to connect to IB service..." >&2
    else
      echo "Configuring VITE_API_URL to connect to backend service..." >&2
    fi
    update_env_var "VITE_API_URL" "http://127.0.0.1:${BACKEND_PORT}/api/snapshot"
  else
    echo "⚠ Backend service not detected. Using static JSON data." >&2
    echo "  To use live data, start a backend service:" >&2
    echo "  ./web/scripts/run-alpaca-service.sh  # For Alpaca" >&2
    echo "  ./web/scripts/run-ib-service.sh      # For Interactive Brokers" >&2
  fi
else
  # .env exists and has VITE_API_URL
  CURRENT_URL=$(grep "^VITE_API_URL=" "${ENV_FILE}" 2>/dev/null | cut -d'=' -f2- | tr -d '"' || echo "")
  if [ -n "${CURRENT_URL}" ]; then
    echo "Using existing VITE_API_URL: ${CURRENT_URL}" >&2
  fi
fi

# Set port configuration as environment variables for frontend
update_env_var "VITE_ALPACA_PORT" "${ALPACA_PORT}"
update_env_var "VITE_IB_PORT" "${IB_PORT}"
update_env_var "VITE_DISCOUNT_BANK_PORT" "${DISCOUNT_BANK_PORT}"
update_env_var "VITE_RISK_FREE_RATE_PORT" "${RISK_FREE_RATE_PORT}"
update_env_var "VITE_TASTYTRADE_PORT" "${TASTYTRADE_PORT}"
update_env_var "VITE_RUST_BACKEND_REST_PORT" "${RUST_BACKEND_REST_PORT}"

if [ "${ENV_UPDATED}" = true ]; then
  echo "Updated ${ENV_FILE} with port configuration" >&2
fi

# Get Vite port from config
VITE_PORT=$(get_service_port "web" 5173)

# Check if Vite port is already in use
# If port is in use, exit - the launch script should handle port conflicts
if check_port "${VITE_PORT}"; then
  echo "Error: Port ${VITE_PORT} is already in use." >&2
  echo "Another web service instance may be running." >&2
  echo "To stop it, run: ./web/scripts/launch-all-pwa-services.sh stop" >&2
  echo "Or manually: lsof -ti :${VITE_PORT} | xargs kill -9" >&2
  exit 1
fi

echo "Starting web service (PWA) on port ${VITE_PORT}..." >&2
if [ "${BACKEND_SERVICE_TYPE}" != "none" ]; then
  if [ "${BACKEND_SERVICE_TYPE}" = "alpaca" ]; then
    echo "  Connected to Alpaca service: http://127.0.0.1:${ALPACA_PORT}" >&2
  elif [ "${BACKEND_SERVICE_TYPE}" = "ib" ]; then
    echo "  Connected to IB service: http://127.0.0.1:${BACKEND_PORT}" >&2
  else
    echo "  Connected to backend service: http://127.0.0.1:${BACKEND_PORT}" >&2
  fi
else
  echo "  Using static data (backend service not running)" >&2
fi
echo "" >&2

# Start the Vite dev server
npm run dev
