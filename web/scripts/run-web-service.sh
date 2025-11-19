#!/usr/bin/env bash
# Run web PWA service (Vite dev server)
# Automatically configures connection to Alpaca service if available
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/../.." && pwd)"
WEB_DIR="$ROOT_DIR/web"

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

# Check if backend service is running
BACKEND_SERVICE_TYPE="none"
SERVICE_CHECK_RESULT=$(check_backend_service 8000 || echo $?)
case "${SERVICE_CHECK_RESULT}" in
  1)
    BACKEND_SERVICE_TYPE="alpaca"
    echo "✓ Alpaca service detected on http://127.0.0.1:8000" >&2
    ;;
  2)
    BACKEND_SERVICE_TYPE="ib"
    echo "✓ IB service detected on http://127.0.0.1:8000" >&2
    ;;
  0)
    BACKEND_SERVICE_TYPE="unknown"
    echo "✓ Backend service detected on http://127.0.0.1:8000" >&2
    ;;
  *)
    BACKEND_SERVICE_TYPE="none"
    ;;
esac

# Set up environment file
ENV_FILE=".env"
if [ ! -f "${ENV_FILE}" ] || ! grep -q "VITE_API_URL" "${ENV_FILE}" 2>/dev/null; then
  if [ "${BACKEND_SERVICE_TYPE}" != "none" ]; then
    if [ "${BACKEND_SERVICE_TYPE}" = "alpaca" ]; then
      echo "Configuring VITE_API_URL to connect to Alpaca service..." >&2
    elif [ "${BACKEND_SERVICE_TYPE}" = "ib" ]; then
      echo "Configuring VITE_API_URL to connect to IB service..." >&2
    else
      echo "Configuring VITE_API_URL to connect to backend service..." >&2
    fi
    if [ -f "${ENV_FILE}" ]; then
      # Append to existing .env
      echo "" >> "${ENV_FILE}"
      echo "VITE_API_URL=http://127.0.0.1:8000/api/snapshot" >> "${ENV_FILE}"
    else
      # Create new .env
      echo "VITE_API_URL=http://127.0.0.1:8000/api/snapshot" > "${ENV_FILE}"
    fi
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

# Check if Vite port is already in use
# If port is in use, exit - the launch script should handle port conflicts
VITE_PORT=5173
if check_port "${VITE_PORT}"; then
  echo "Error: Port ${VITE_PORT} is already in use." >&2
  echo "Another web service instance may be running." >&2
  echo "To stop it, run: ./web/scripts/launch-all-pwa-services.sh stop" >&2
  echo "Or manually: lsof -ti :${VITE_PORT} | xargs kill -9" >&2
  exit 1
fi

echo "Starting web service (PWA)..." >&2
if [ "${BACKEND_SERVICE_TYPE}" != "none" ]; then
  if [ "${BACKEND_SERVICE_TYPE}" = "alpaca" ]; then
    echo "  Connected to Alpaca service: http://127.0.0.1:8000" >&2
  elif [ "${BACKEND_SERVICE_TYPE}" = "ib" ]; then
    echo "  Connected to IB service: http://127.0.0.1:8000" >&2
  else
    echo "  Connected to backend service: http://127.0.0.1:8000" >&2
  fi
else
  echo "  Using static data (backend service not running)" >&2
fi
echo "" >&2

# Start the Vite dev server
npm run dev
