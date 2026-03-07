#!/usr/bin/env bash
# Run Tastytrade service for PWA integration
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/../.." && pwd)"
PYTHON_DIR="$ROOT_DIR/python"
SCRIPTS_DIR="${ROOT_DIR}/scripts"

# Load configuration functions
# shellcheck source=../../scripts/include/config.sh
if [ -f "${SCRIPTS_DIR}/include/config.sh" ]; then
  source "${SCRIPTS_DIR}/include/config.sh"
else
  echo "Warning: config.sh not found, using defaults" >&2
  # Fallback functions if config.sh not available
  config_get_port() {
    local service_name="${1:-}"
    local default_port="${2:-}"
    # Check environment variable
    local env_var_name
    env_var_name=$(echo "${service_name}" | tr '[:lower:]' '[:upper:]' | tr '_' ' ')
    env_var_name="${env_var_name}_PORT"
    env_var_name=$(echo "${env_var_name}" | tr ' ' '_')
    if [ -n "${!env_var_name:-}" ]; then
      echo "${!env_var_name}"
    else
      echo "${default_port}"
    fi
  }
  config_check_port_available() {
    local port="${1:-}"
    if command -v lsof >/dev/null 2>&1; then
      ! lsof -ti ":${port}" >/dev/null 2>&1
    elif command -v netstat >/dev/null 2>&1; then
      ! netstat -an 2>/dev/null | grep -q ":${port}.*LISTEN"
    else
      return 0
    fi
  }
fi

cd "$PYTHON_DIR"

# Find Python command
PYTHON_CMD=""

# Try python3 first (most common on macOS/Linux)
if command -v python3 >/dev/null 2>&1; then
  PYTHON_CMD="python3"
# Fall back to python
elif command -v python >/dev/null 2>&1; then
  PYTHON_CMD="python"
fi

if [ -z "${PYTHON_CMD}" ]; then
  echo "Error: Python not found. Please install Python 3." >&2
  exit 1
fi

# Set up virtual environment
VENV_DIR="${PYTHON_DIR}/.venv"
ACTIVATE_PATH="${VENV_DIR}/bin/activate"

# Create virtual environment if it doesn't exist
if [ ! -f "${ACTIVATE_PATH}" ]; then
  echo "Creating Python virtual environment at ${VENV_DIR}..." >&2
  "${PYTHON_CMD}" -m venv "${VENV_DIR}" || {
    echo "Error: Failed to create virtual environment. Please ensure venv module is available." >&2
    echo "  Try: ${PYTHON_CMD} -m ensurepip --upgrade" >&2
    exit 1
  }
else
  echo "Using existing virtual environment at ${VENV_DIR}" >&2
fi

# Activate virtual environment
# shellcheck disable=SC1090
source "${ACTIVATE_PATH}"

# Update pip in virtual environment only when venv has pip (skip for uv-created venvs)
if "${VENV_DIR}/bin/python" -m pip --version >/dev/null 2>&1; then
  "${VENV_DIR}/bin/python" -m pip install --quiet --upgrade pip wheel >/dev/null 2>&1 || true
fi

# Check if required packages are installed (using venv Python)
VENV_PYTHON="${VENV_DIR}/bin/python"
MISSING_PACKAGES=()

if ! "${VENV_PYTHON}" -c "import uvicorn" 2>/dev/null; then
  MISSING_PACKAGES+=("uvicorn[standard]" "fastapi")
fi

if ! "${VENV_PYTHON}" -c "import requests" 2>/dev/null; then
  MISSING_PACKAGES+=("requests")
fi

# Check for websockets (required for DXLink streaming and uvicorn WebSocket support)
if ! "${VENV_PYTHON}" -c "import websockets" 2>/dev/null; then
  MISSING_PACKAGES+=("websockets")
fi

if [ ${#MISSING_PACKAGES[@]} -gt 0 ]; then
  echo "Installing missing packages in virtual environment: ${MISSING_PACKAGES[*]}..." >&2
  if command -v uv >/dev/null 2>&1; then
    uv pip install --python "${VENV_PYTHON}" --quiet "${MISSING_PACKAGES[@]}" >&2
  else
    "${VENV_PYTHON}" -m pip install --quiet "${MISSING_PACKAGES[@]}" >&2
  fi
fi

# Use venv Python for all subsequent operations
PYTHON_CMD="${VENV_PYTHON}"

# Check if integration module file exists
if [ ! -f "integration/tastytrade_service.py" ]; then
  echo "Error: integration/tastytrade_service.py not found. Current directory: $(pwd)" >&2
  exit 1
fi

# Check for Tastytrade credentials
if [ -z "${TASTYTRADE_USERNAME:-}" ] && [ -z "${TASTYTRADE_PASSWORD:-}" ]; then
  echo "⚠ Warning: TASTYTRADE_USERNAME and TASTYTRADE_PASSWORD not set" >&2
  echo "  The service will start, but authentication will fail." >&2
  echo "" >&2
  echo "  Set credentials:" >&2
  echo "    export TASTYTRADE_USERNAME=your_username" >&2
  echo "    export TASTYTRADE_PASSWORD=your_password" >&2
  echo "" >&2
  echo "  Or configure in config/config.json:" >&2
  echo "    \"tastytrade\": {" >&2
  echo "      \"username\": \"your_username\"," >&2
  echo "      \"password\": \"your_password\"" >&2
  echo "    }" >&2
  echo "" >&2
fi

# Get Tastytrade service port from config (default: 8005)
TASTYTRADE_PORT=$(config_get_port "tastytrade" 8005)

# Check if Tastytrade service port is already in use
if ! config_check_port_available "${TASTYTRADE_PORT}"; then
  echo "Port ${TASTYTRADE_PORT} is already in use. Checking if it's the Tastytrade service..." >&2

  # Try to verify it's the Tastytrade service by checking the health endpoint
  # Accept status 'ok' or 'disabled' (e.g. missing credentials) as same service
  HEALTH_CHECK=$("${PYTHON_CMD}" -c "
import urllib.request
import json
import sys
try:
    with urllib.request.urlopen('http://127.0.0.1:${TASTYTRADE_PORT}/api/health', timeout=2) as response:
        data = json.loads(response.read().decode())
        if 'tastytrade_connected' in data:
            print('TASTYTRADE_SERVICE')
        else:
            print('OTHER_SERVICE')
except Exception:
    print('OTHER_SERVICE')
" 2>/dev/null || echo "OTHER_SERVICE")

  if [ "${HEALTH_CHECK}" = "TASTYTRADE_SERVICE" ]; then
    echo "✓ Tastytrade service is already running on http://127.0.0.1:${TASTYTRADE_PORT}" >&2
    echo "  Using existing service. No need to start a new one." >&2
    echo "  Set VITE_API_URL=http://127.0.0.1:${TASTYTRADE_PORT}/api/snapshot in your web app" >&2
    echo "" >&2
    exit 0
  else
    echo "Error: Port ${TASTYTRADE_PORT} is in use by another service (not Tastytrade service)" >&2
    echo "  Please stop the service on port ${TASTYTRADE_PORT} or use a different port:" >&2
    echo "  export TASTYTRADE_PORT=<different_port>" >&2
    exit 1
  fi
fi

# Check if we can import (this will fail if dependencies are missing, but that's OK)
# We'll handle the import error gracefully
IMPORT_TEST=$("${PYTHON_CMD}" -c "
import sys
import os
# Temporarily disable __init__.py by renaming it
init_py = 'integration/__init__.py'
backup_py = 'integration/__init__.py.bak'
if os.path.exists(init_py):
    os.rename(init_py, backup_py)
try:
    sys.path.insert(0, '.')
    from integration.tastytrade_service import app
    print('OK')
except Exception as e:
    print(f'ERROR: {e}')
finally:
    if os.path.exists(backup_py):
        os.rename(backup_py, init_py)
" 2>&1)

if echo "${IMPORT_TEST}" | grep -q "OK"; then
  echo "Starting Tastytrade service on http://127.0.0.1:${TASTYTRADE_PORT}" >&2
  echo "Set VITE_API_URL=http://127.0.0.1:${TASTYTRADE_PORT}/api/snapshot in your web app" >&2
  echo "" >&2
  echo "Optional environment variables:" >&2
  echo "  SYMBOLS=SPY,QQQ,IWM  # Comma-separated symbols (default: SPY,QQQ)" >&2
  echo "" >&2
  echo "  Authentication (OAuth preferred):" >&2
  echo "    TASTYTRADE_CLIENT_SECRET=your_client_secret  # OAuth client secret" >&2
  echo "    TASTYTRADE_REFRESH_TOKEN=your_refresh_token  # OAuth refresh token" >&2
  echo "" >&2
  echo "  Authentication (Session-based fallback):" >&2
  echo "    TASTYTRADE_USERNAME=your_username  # Tastytrade username" >&2
  echo "    TASTYTRADE_PASSWORD=your_password  # Tastytrade password" >&2
  echo "" >&2
  echo "  Environment:" >&2
  echo "    TASTYTRADE_SANDBOX=true  # Enable sandbox mode (uses api.cert.tastyworks.com)" >&2
  echo "    TASTYTRADE_BASE_URL=https://api.tastytrade.com  # Production API base URL" >&2
  echo "    TASTYTRADE_SANDBOX_BASE_URL=https://api.cert.tastyworks.com  # Sandbox API base URL" >&2
  echo "    TASTYTRADE_PORT=${TASTYTRADE_PORT}  # Service port (override config)" >&2
  echo "    SNAPSHOT_FILE_PATH=/path/to/snapshot.json  # Optional file output" >&2
  echo "" >&2
  if [ "${TASTYTRADE_SANDBOX:-}" = "true" ] || [ "${TASTYTRADE_SANDBOX:-}" = "1" ]; then
    echo "  ⚠ SANDBOX MODE ENABLED:" >&2
    echo "    - Sandbox resets every 24 hours (trades/positions cleared)" >&2
    echo "    - Quotes are 15-minutes delayed" >&2
    echo "    - Use sandbox user credentials" >&2
    echo "" >&2
  fi

  # Temporarily disable __init__.py to avoid dependency issues
  INIT_PY="integration/__init__.py"
  INIT_PY_BAK="integration/__init__.py.bak"
  if [ -f "${INIT_PY}" ]; then
    mv "${INIT_PY}" "${INIT_PY_BAK}"
    trap "mv '${INIT_PY_BAK}' '${INIT_PY}' 2>/dev/null || true" EXIT
  fi

  # Run the service with PYTHONPATH set
  # Use trap to handle failures gracefully (don't kill tmux)
  export PYTHONPATH="${PYTHON_DIR}:${PYTHONPATH:-}"
  if ! "${PYTHON_CMD}" -m uvicorn integration.tastytrade_service:app --host 127.0.0.1 --port "${TASTYTRADE_PORT}" --reload; then
    EXIT_CODE=$?
    echo "" >&2
    echo "⚠ Tastytrade service exited with error code ${EXIT_CODE}" >&2
    echo "Press any key to continue..." >&2
    read -n 1 -s || true
    exit "${EXIT_CODE}"
  fi
else
  echo "Error: Cannot import tastytrade_service" >&2
  echo "${IMPORT_TEST}" >&2
  echo "" >&2
  echo "Press any key to continue..." >&2
  read -n 1 -s || true
  exit 1
fi
