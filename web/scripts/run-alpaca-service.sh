#!/usr/bin/env bash
# Run Alpaca service for PWA integration
# Supports 1Password integration for secure credential management
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/../.." && pwd)"
PYTHON_DIR="$ROOT_DIR/python"

# Function to read from 1Password or environment variable
# Supports both personal accounts (op signin) and service accounts (OP_SERVICE_ACCOUNT_TOKEN)
read_credential() {
  local op_secret="${1:-}"
  local env_var="${2:-}"
  local result=""

  # Try 1Password first if secret path is provided
  # The CLI automatically uses OP_SERVICE_ACCOUNT_TOKEN if set, otherwise uses signed-in session
  if [ -n "${op_secret:-}" ] && command -v op >/dev/null 2>&1; then
    result=$(op read "${op_secret}" 2>/dev/null | tr -d '[:space:]' || echo "")
    if [ -n "${result}" ]; then
      echo -n "${result}"
      return 0
    fi
  fi

  # Fall back to environment variable
  if [ -n "${env_var:-}" ]; then
    echo -n "${env_var}"
    return 0
  fi

  return 1
}

# Read credentials from 1Password or environment variables
# Supports both path format (op://Vault/Item/field) and UUID format
OP_API_KEY_SECRET="${OP_ALPACA_API_KEY_ID_SECRET:-}"
OP_API_SECRET_SECRET="${OP_ALPACA_API_SECRET_KEY_SECRET:-}"

# If OP_ALPACA_ITEM_UUID is set, use it to construct field references
# Format: op://<vault>/<uuid>/<field> or just <uuid> with field labels
if [ -n "${OP_ALPACA_ITEM_UUID:-}" ]; then
  # Try to auto-detect vault and field names
  if command -v op >/dev/null 2>&1; then
    ITEM_JSON=$(op item get "${OP_ALPACA_ITEM_UUID}" --format json 2>/dev/null || echo "")
    if [ -n "${ITEM_JSON}" ]; then
      VAULT_ID=$(echo "${ITEM_JSON}" | python3 -c "import sys, json; print(json.load(sys.stdin).get('vault', {}).get('id', ''))" 2>/dev/null || echo "")
      VAULT_NAME=$(echo "${ITEM_JSON}" | python3 -c "import sys, json; print(json.load(sys.stdin).get('vault', {}).get('name', ''))" 2>/dev/null || echo "")

      # Try to find the API key and secret fields by common names
      # Check for: API Key ID, username, api_key, key_id, etc.
      KEY_FIELD=$(echo "${ITEM_JSON}" | python3 -c "
import sys, json
data = json.load(sys.stdin)
fields = data.get('fields', [])
# Try common field names for API key
for name in ['API Key ID', 'api_key_id', 'API Key', 'api_key', 'username', 'key_id', 'Key ID']:
    for field in fields:
        if field.get('label', '').lower() == name.lower():
            print(name)
            sys.exit(0)
" 2>/dev/null || echo "")

      SECRET_FIELD=$(echo "${ITEM_JSON}" | python3 -c "
import sys, json
data = json.load(sys.stdin)
fields = data.get('fields', [])
# Try common field names for API secret
for name in ['API Secret Key', 'api_secret_key', 'API Secret', 'api_secret', 'credential', 'secret_key', 'Secret Key', 'password']:
    for field in fields:
        if field.get('label', '').lower() == name.lower() and field.get('type') == 'CONCEALED':
            print(name)
            sys.exit(0)
" 2>/dev/null || echo "")

      # Use custom field names if provided, otherwise use detected or defaults
      KEY_FIELD_NAME="${OP_ALPACA_KEY_FIELD_NAME:-${KEY_FIELD:-API Key ID}}"
      SECRET_FIELD_NAME="${OP_ALPACA_SECRET_FIELD_NAME:-${SECRET_FIELD:-API Secret Key}}"

      if [ -n "${VAULT_ID}" ]; then
        OP_API_KEY_SECRET="${OP_API_KEY_SECRET:-op://${VAULT_ID}/${OP_ALPACA_ITEM_UUID}/${KEY_FIELD_NAME}}"
        OP_API_SECRET_SECRET="${OP_API_SECRET_SECRET:-op://${VAULT_ID}/${OP_ALPACA_ITEM_UUID}/${SECRET_FIELD_NAME}}"
      elif [ -n "${VAULT_NAME}" ]; then
        OP_API_KEY_SECRET="${OP_API_KEY_SECRET:-op://${VAULT_NAME}/${OP_ALPACA_ITEM_UUID}/${KEY_FIELD_NAME}}"
        OP_API_SECRET_SECRET="${OP_API_SECRET_SECRET:-op://${VAULT_NAME}/${OP_ALPACA_ITEM_UUID}/${SECRET_FIELD_NAME}}"
      else
        OP_API_KEY_SECRET="${OP_API_KEY_SECRET:-op://Private/${OP_ALPACA_ITEM_UUID}/${KEY_FIELD_NAME}}"
        OP_API_SECRET_SECRET="${OP_API_SECRET_SECRET:-op://Private/${OP_ALPACA_ITEM_UUID}/${SECRET_FIELD_NAME}}"
      fi
    fi
  fi
fi

ALPACA_API_KEY_ID=$(read_credential "${OP_API_KEY_SECRET}" "${ALPACA_API_KEY_ID:-}" || echo "")
ALPACA_API_SECRET_KEY=$(read_credential "${OP_API_SECRET_SECRET}" "${ALPACA_API_SECRET_KEY:-}" || echo "")

# Check for required credentials
if [ -z "${ALPACA_API_KEY_ID}" ] || [ -z "${ALPACA_API_SECRET_KEY}" ]; then
  echo "Error: Alpaca credentials not set" >&2
  echo "" >&2
  echo "Option 1: Use 1Password (recommended):" >&2
  echo "  # Method A: Using item UUID (simplest):" >&2
  echo "  op signin" >&2
  echo "  export OP_ALPACA_ITEM_UUID='ldfc5jfigtmjvlg6ls4tgpgsuu'" >&2
  echo "  # Script will auto-detect field names" >&2
  echo "  # If auto-detection fails, specify field names:" >&2
  echo "  export OP_ALPACA_KEY_FIELD_NAME='username'  # or 'API Key ID', etc." >&2
  echo "  export OP_ALPACA_SECRET_FIELD_NAME='credential'  # or 'API Secret Key', etc." >&2
  echo "" >&2
  echo "  # Method B: Using full paths:" >&2
  echo "  op signin" >&2
  echo "  export OP_ALPACA_API_KEY_ID_SECRET='op://Vault/Item/API Key ID'" >&2
  echo "  export OP_ALPACA_API_SECRET_KEY_SECRET='op://Vault/Item/API Secret Key'" >&2
  echo "" >&2
  echo "  # Method C: Using UUID with explicit paths:" >&2
  echo "  export OP_ALPACA_API_KEY_ID_SECRET='op://Vault/<uuid>/API Key ID'" >&2
  echo "  export OP_ALPACA_API_SECRET_KEY_SECRET='op://Vault/<uuid>/API Secret Key'" >&2
  echo "" >&2
  echo "  # For service account (CI/CD):" >&2
  echo "  export OP_SERVICE_ACCOUNT_TOKEN='your_token'" >&2
  echo "  # Then use any of the methods above" >&2
  echo "" >&2
  echo "Option 2: Use environment variables:" >&2
  echo "  export ALPACA_API_KEY_ID=your_key_id" >&2
  echo "  export ALPACA_API_SECRET_KEY=your_secret_key" >&2
  echo "" >&2
  echo "Optional:" >&2
  echo "  export ALPACA_PAPER=1  # Use paper trading (default)" >&2
  echo "  export SYMBOLS=SPY,QQQ,IWM  # Comma-separated symbols (default: SPY,QQQ)" >&2
  exit 1
fi

# Export credentials for the Python service
export ALPACA_API_KEY_ID
export ALPACA_API_SECRET_KEY

cd "$PYTHON_DIR"

# Find Python and pip commands
PYTHON_CMD=""
PIP_CMD=""

# Try python3 first (most common on macOS/Linux)
if command -v python3 >/dev/null 2>&1; then
  PYTHON_CMD="python3"
  if command -v pip3 >/dev/null 2>&1; then
    PIP_CMD="pip3"
  fi
# Fall back to python
elif command -v python >/dev/null 2>&1; then
  PYTHON_CMD="python"
  if command -v pip >/dev/null 2>&1; then
    PIP_CMD="pip"
  fi
fi

if [ -z "${PYTHON_CMD}" ]; then
  echo "Error: Python not found. Please install Python 3." >&2
  exit 1
fi

# Check if required packages are installed
MISSING_PACKAGES=()

if ! "${PYTHON_CMD}" -c "import uvicorn" 2>/dev/null; then
  MISSING_PACKAGES+=("uvicorn" "fastapi")
fi

if ! "${PYTHON_CMD}" -c "from alpaca.trading.client import TradingClient" 2>/dev/null; then
  MISSING_PACKAGES+=("alpaca-py")
fi

if [ ${#MISSING_PACKAGES[@]} -gt 0 ]; then
  echo "Installing missing packages: ${MISSING_PACKAGES[*]}..." >&2
  if [ -z "${PIP_CMD}" ]; then
    # Try python -m pip as fallback
    if "${PYTHON_CMD}" -m pip --version >/dev/null 2>&1; then
      PIP_CMD="${PYTHON_CMD} -m pip"
    else
      echo "Error: pip not found. Please install pip or use: ${PYTHON_CMD} -m ensurepip" >&2
      exit 1
    fi
  fi
  ${PIP_CMD} install "${MISSING_PACKAGES[@]}" >&2
fi

# Check if integration module file exists
if [ ! -f "integration/alpaca_service.py" ]; then
  echo "Error: integration/alpaca_service.py not found. Current directory: $(pwd)" >&2
  exit 1
fi

# Check if port 8000 is already in use
check_port() {
  local port="${1:-8000}"
  if command -v lsof >/dev/null 2>&1; then
    lsof -ti ":${port}" >/dev/null 2>&1
  elif command -v netstat >/dev/null 2>&1; then
    netstat -an 2>/dev/null | grep -q ":${port}.*LISTEN"
  else
    # Fallback: try to connect
    "${PYTHON_CMD}" -c "import socket; s = socket.socket(); s.settimeout(0.1); result = s.connect_ex(('127.0.0.1', ${port})); s.close(); exit(0 if result == 0 else 1)" 2>/dev/null
  fi
}

# Check if Alpaca service is already running
if check_port 8000; then
  echo "Port 8000 is already in use. Checking if it's the Alpaca service..." >&2

  # Try to verify it's the Alpaca service by checking the health endpoint
  HEALTH_CHECK=$("${PYTHON_CMD}" -c "
import urllib.request
import json
import sys
try:
    with urllib.request.urlopen('http://127.0.0.1:8000/api/health', timeout=2) as response:
        data = json.loads(response.read().decode())
        if data.get('status') == 'ok':
            print('ALPACA_SERVICE')
        else:
            print('OTHER_SERVICE')
except Exception:
    print('OTHER_SERVICE')
" 2>/dev/null || echo "OTHER_SERVICE")

  if [ "${HEALTH_CHECK}" = "ALPACA_SERVICE" ]; then
    echo "✓ Alpaca service is already running on http://127.0.0.1:8000" >&2
    echo "  Using existing service. No need to start a new one." >&2
    echo "  Set VITE_API_URL=http://127.0.0.1:8000/api/snapshot in your web app" >&2
    echo "" >&2
    exit 0
  else
    echo "Error: Port 8000 is in use by another service (not Alpaca service)" >&2
    echo "  Please stop the service on port 8000 or use a different port:" >&2
    echo "  export ALPACA_SERVICE_PORT=8001" >&2
    echo "  (Note: Port configuration not yet implemented in this script)" >&2
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
    from integration.alpaca_service import app
    print('OK')
except Exception as e:
    print(f'ERROR: {e}')
finally:
    if os.path.exists(backup_py):
        os.rename(backup_py, init_py)
" 2>&1)

if echo "${IMPORT_TEST}" | grep -q "OK"; then
  echo "Starting Alpaca service on http://127.0.0.1:8000" >&2
  echo "Set VITE_API_URL=http://127.0.0.1:8000/api/snapshot in your web app" >&2
  echo "" >&2

  # Temporarily disable __init__.py to avoid dependency issues
  INIT_PY="integration/__init__.py"
  INIT_PY_BAK="integration/__init__.py.bak"
  if [ -f "${INIT_PY}" ]; then
    mv "${INIT_PY}" "${INIT_PY_BAK}"
    trap "mv '${INIT_PY_BAK}' '${INIT_PY}' 2>/dev/null || true" EXIT
  fi

  # Run the service with PYTHONPATH set
  export PYTHONPATH="${PYTHON_DIR}:${PYTHONPATH:-}"
  "${PYTHON_CMD}" -m uvicorn integration.alpaca_service:app --host 127.0.0.1 --port 8000 --reload
else
  echo "Error: Cannot import alpaca_service" >&2
  echo "${IMPORT_TEST}" >&2
  exit 1
fi
