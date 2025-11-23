#!/usr/bin/env bash
# Run Alpaca service for PWA integration
# Supports 1Password integration for secure credential management
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

# shellcheck source=../../scripts/include/onepassword.sh
if [ -f "${SCRIPTS_DIR}/include/onepassword.sh" ]; then
  source "${SCRIPTS_DIR}/include/onepassword.sh"
fi

# Read credentials from 1Password or environment variables
# Supports both OAuth (preferred) and API key authentication
# Supports both path format (op://Vault/Item/field) and UUID format

# OAuth credentials (preferred)
OP_CLIENT_ID_SECRET="${OP_ALPACA_CLIENT_ID_SECRET:-}"
OP_CLIENT_SECRET_SECRET="${OP_ALPACA_CLIENT_SECRET_SECRET:-}"

# API key credentials (fallback)
OP_API_KEY_SECRET="${OP_ALPACA_API_KEY_ID_SECRET:-}"
OP_API_SECRET_SECRET="${OP_ALPACA_API_SECRET_KEY_SECRET:-}"

# If OP_ALPACA_ITEM_UUID is set, use it to construct field references
if [ -n "${OP_ALPACA_ITEM_UUID:-}" ]; then
  # Auto-detect fields from UUID
  KEY_FIELD=""
  SECRET_FIELD=""
  if op_detect_fields "${OP_ALPACA_ITEM_UUID}" "KEY_FIELD" "SECRET_FIELD"; then
    KEY_FIELD_NAME="${OP_ALPACA_KEY_FIELD_NAME:-${KEY_FIELD:-Client ID}}"
    SECRET_FIELD_NAME="${OP_ALPACA_SECRET_FIELD_NAME:-${SECRET_FIELD:-Client Secret}}"

    # Build secret paths for OAuth (only if not already set)
    if [ -z "${OP_CLIENT_ID_SECRET}" ] || [ -z "${OP_CLIENT_SECRET_SECRET}" ]; then
      op_build_secret_paths "${OP_ALPACA_ITEM_UUID}" "${KEY_FIELD_NAME}" "${SECRET_FIELD_NAME}" "OP_CLIENT_ID_SECRET" "OP_CLIENT_SECRET_SECRET"
    fi

    # Also try to build API key paths if OAuth not found
    if [ -z "${OP_API_KEY_SECRET}" ] || [ -z "${OP_API_SECRET_SECRET}" ]; then
      KEY_FIELD_NAME="${OP_ALPACA_KEY_FIELD_NAME:-${KEY_FIELD:-API Key ID}}"
      SECRET_FIELD_NAME="${OP_ALPACA_SECRET_FIELD_NAME:-${SECRET_FIELD:-API Secret Key}}"
      op_build_secret_paths "${OP_ALPACA_ITEM_UUID}" "${KEY_FIELD_NAME}" "${SECRET_FIELD_NAME}" "OP_API_KEY_SECRET" "OP_API_SECRET_SECRET"
    fi
  fi
fi

# Try OAuth credentials first
ALPACA_CLIENT_ID=$(read_credential "${OP_CLIENT_ID_SECRET}" "${ALPACA_CLIENT_ID:-}" || echo "")
ALPACA_CLIENT_SECRET=$(read_credential "${OP_CLIENT_SECRET_SECRET}" "${ALPACA_CLIENT_SECRET:-}" || echo "")

# Fall back to API keys if OAuth not available
if [ -z "${ALPACA_CLIENT_ID}" ] || [ -z "${ALPACA_CLIENT_SECRET}" ]; then
  ALPACA_API_KEY_ID=$(read_credential "${OP_API_KEY_SECRET}" "${ALPACA_API_KEY_ID:-}" || echo "")
  ALPACA_API_SECRET_KEY=$(read_credential "${OP_API_SECRET_SECRET}" "${ALPACA_API_SECRET_KEY:-}" || echo "")
fi

# Check for required credentials (either OAuth or API keys)
if [ -z "${ALPACA_CLIENT_ID}" ] || [ -z "${ALPACA_CLIENT_SECRET}" ]; then
  if [ -z "${ALPACA_API_KEY_ID}" ] || [ -z "${ALPACA_API_SECRET_KEY}" ]; then
    echo "Error: Alpaca credentials not set" >&2
    echo "" >&2
    echo "Option 1: Use OAuth (preferred):" >&2
    echo "  export ALPACA_CLIENT_ID=your_client_id" >&2
    echo "  export ALPACA_CLIENT_SECRET=your_client_secret" >&2
    echo "" >&2
    echo "Option 2: Use API Keys (fallback):" >&2
    echo "  export ALPACA_API_KEY_ID=your_key_id" >&2
    echo "  export ALPACA_API_SECRET_KEY=your_secret_key" >&2
    echo "" >&2
    echo "Option 3: Use 1Password (recommended):" >&2
    echo "  op signin" >&2
    echo "  export OP_ALPACA_ITEM_UUID='your-item-uuid'" >&2
    echo "  # Script will auto-detect field names" >&2
    echo "" >&2
    echo "Optional:" >&2
    echo "  export ALPACA_PAPER=1  # Use paper trading (default)" >&2
    echo "  export SYMBOLS=SPY,QQQ,IWM  # Comma-separated symbols (default: SPY,QQQ)" >&2
    exit 1
  fi
fi

# Export credentials for the Python service
export ALPACA_CLIENT_ID
export ALPACA_CLIENT_SECRET
export ALPACA_API_KEY_ID
export ALPACA_API_SECRET_KEY

cd "$PYTHON_DIR"

# Find Python command
find_python || exit 1

# Set up virtual environment
setup_venv "${PYTHON_DIR}" || exit 1

# Install required packages
install_python_packages "${VENV_PYTHON}" "uvicorn[standard]" "fastapi" "alpaca-py" || exit 1

# Use venv Python for all subsequent operations
PYTHON_CMD="${VENV_PYTHON}"

# Check if integration module file exists
if [ ! -f "integration/alpaca_service.py" ]; then
  echo "Error: integration/alpaca_service.py not found. Current directory: $(pwd)" >&2
  exit 1
fi

# Get Alpaca service port from config (default: 8000)
ALPACA_PORT=$(config_get_port "alpaca" 8000)

# Check if port is available and verify service identity
if ! check_port_with_service "${PYTHON_CMD}" "127.0.0.1" "${ALPACA_PORT}" "ALPACA_SERVICE" "Alpaca"; then
  exit 1
fi

# Test import before starting service
if ! test_python_import "${PYTHON_CMD}" "integration.alpaca_service" "app"; then
  echo "Error: Cannot import alpaca_service" >&2
  exit 1
fi

echo "Starting Alpaca service on http://127.0.0.1:${ALPACA_PORT}" >&2
echo "Set VITE_API_URL=http://127.0.0.1:${ALPACA_PORT}/api/snapshot in your web app" >&2
echo "" >&2

# Temporarily disable __init__.py to avoid dependency issues
disable_init_py "${PYTHON_DIR}" || exit 1

# Run the service with PYTHONPATH set
export PYTHONPATH="${PYTHON_DIR}:${PYTHONPATH:-}"
"${PYTHON_CMD}" -m uvicorn integration.alpaca_service:app --host 127.0.0.1 --port "${ALPACA_PORT}" --reload
