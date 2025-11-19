#!/bin/bash
# start_alpaca_service.sh - Start the Alpaca FastAPI service for TUI/PWA integration
#
# Usage:
#   ./scripts/start_alpaca_service.sh
#
# Environment variables:
#   ALPACA_API_KEY_ID - Alpaca API key (required, or use 1Password)
#   ALPACA_API_SECRET_KEY - Alpaca API secret (required, or use 1Password)
#   ALPACA_PAPER - Set to "1" for paper trading (default: "1")
#   SYMBOLS - Comma-separated symbols to monitor (default: "SPY,QQQ")
#   SNAPSHOT_FILE_PATH - Optional: path to write snapshot JSON for TUI file polling
#   PORT - Server port (overrides config, default: 8000)
#   HOST - Server host (default: 127.0.0.1)
#   OP_ALPACA_ITEM_UUID - 1Password item UUID for credentials (optional)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
PYTHON_DIR="${PROJECT_ROOT}/python"

# Load shared utility functions
# shellcheck source=./include/config.sh
if [ -f "${SCRIPT_DIR}/include/config.sh" ]; then
  source "${SCRIPT_DIR}/include/config.sh"
fi

# shellcheck source=./include/python_utils.sh
if [ -f "${SCRIPT_DIR}/include/python_utils.sh" ]; then
  source "${SCRIPT_DIR}/include/python_utils.sh"
else
  echo "Error: python_utils.sh not found" >&2
  exit 1
fi

# shellcheck source=./include/service_utils.sh
if [ -f "${SCRIPT_DIR}/include/service_utils.sh" ]; then
  source "${SCRIPT_DIR}/include/service_utils.sh"
fi

# shellcheck source=./include/onepassword.sh
if [ -f "${SCRIPT_DIR}/include/onepassword.sh" ]; then
  source "${SCRIPT_DIR}/include/onepassword.sh"
fi

# Default values
export ALPACA_PAPER="${ALPACA_PAPER:-1}"
export SYMBOLS="${SYMBOLS:-SPY,QQQ}"
export HOST="${HOST:-127.0.0.1}"

# Get port from config with environment variable override
PORT=$(config_get_port "alpaca" 8000)
export PORT

# Read credentials from 1Password or environment variables
OP_API_KEY_SECRET="${OP_ALPACA_API_KEY_ID_SECRET:-}"
OP_API_SECRET_SECRET="${OP_ALPACA_API_SECRET_KEY_SECRET:-}"

# If OP_ALPACA_ITEM_UUID is set, use it to construct field references
if [ -n "${OP_ALPACA_ITEM_UUID:-}" ]; then
  # Auto-detect fields from UUID
  KEY_FIELD=""
  SECRET_FIELD=""
  if op_detect_fields "${OP_ALPACA_ITEM_UUID}" "KEY_FIELD" "SECRET_FIELD"; then
    KEY_FIELD_NAME="${OP_ALPACA_KEY_FIELD_NAME:-${KEY_FIELD:-API Key ID}}"
    SECRET_FIELD_NAME="${OP_ALPACA_SECRET_FIELD_NAME:-${SECRET_FIELD:-API Secret Key}}"

    # Build secret paths (only if not already set)
    if [ -z "${OP_API_KEY_SECRET}" ] || [ -z "${OP_API_SECRET_SECRET}" ]; then
      op_build_secret_paths "${OP_ALPACA_ITEM_UUID}" "${KEY_FIELD_NAME}" "${SECRET_FIELD_NAME}" "OP_API_KEY_SECRET" "OP_API_SECRET_SECRET"
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
  echo "  op signin" >&2
  echo "  export OP_ALPACA_ITEM_UUID='your_item_uuid'" >&2
  echo "  # Or use explicit paths:" >&2
  echo "  export OP_ALPACA_API_KEY_ID_SECRET='op://Vault/Item/API Key ID'" >&2
  echo "  export OP_ALPACA_API_SECRET_KEY_SECRET='op://Vault/Item/API Secret Key'" >&2
  echo "" >&2
  echo "Option 2: Use environment variables:" >&2
  echo "  export ALPACA_API_KEY_ID=your_key_id" >&2
  echo "  export ALPACA_API_SECRET_KEY=your_secret_key" >&2
  exit 1
fi

# Export credentials for the Python service
export ALPACA_API_KEY_ID
export ALPACA_API_SECRET_KEY

# Optional: Set snapshot file path for TUI file polling
if [[ -n "${SNAPSHOT_FILE_PATH:-}" ]]; then
  echo "Snapshot file path: $SNAPSHOT_FILE_PATH"
  # Create directory if it doesn't exist
  mkdir -p "$(dirname "$SNAPSHOT_FILE_PATH")"
fi

# Find Python command
find_python || exit 1

# Set up virtual environment
setup_venv "${PYTHON_DIR}" || exit 1

# Install required packages
install_python_packages "${VENV_PYTHON}" "uvicorn[standard]" "fastapi" "alpaca-py" || exit 1

# Use venv Python for all subsequent operations
PYTHON_CMD="${VENV_PYTHON}"

# Check if port is available and verify service identity
if ! check_port_with_service "${PYTHON_CMD}" "${HOST}" "${PORT}" "ALPACA_SERVICE" "Alpaca"; then
  exit 1
fi

echo "Starting Alpaca FastAPI service..."
echo "  Host: $HOST"
echo "  Port: $PORT"
echo "  Symbols: $SYMBOLS"
echo "  Paper Trading: $ALPACA_PAPER"
echo ""
echo "Endpoints:"
echo "  Health: http://$HOST:$PORT/api/health"
echo "  Snapshot: http://$HOST:$PORT/api/snapshot"
echo "  Account: http://$HOST:$PORT/api/account"
echo "  Positions: http://$HOST:$PORT/api/positions"
echo "  Orders: http://$HOST:$PORT/api/orders"
echo ""

# Change to Python directory
cd "$PYTHON_DIR"

# Check if integration module file exists
if [ ! -f "integration/alpaca_service.py" ]; then
  echo "Error: integration/alpaca_service.py not found. Current directory: $(pwd)" >&2
  exit 1
fi

# Temporarily disable __init__.py to avoid dependency issues
disable_init_py "${PYTHON_DIR}" || exit 1

# Run the service with PYTHONPATH set
export PYTHONPATH="${PYTHON_DIR}:${PYTHONPATH:-}"
exec "${PYTHON_CMD}" -m uvicorn integration.alpaca_service:app \
  --host "$HOST" \
  --port "$PORT" \
  --reload
