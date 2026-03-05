#!/usr/bin/env bash
# Run TradeStation service for PWA integration
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
OP_CLIENT_ID_SECRET="${OP_TRADESTATION_CLIENT_ID_SECRET:-}"
OP_CLIENT_SECRET_SECRET="${OP_TRADESTATION_CLIENT_SECRET_SECRET:-}"

# Check for secret early: if 1Password refs are set but we can't read, exit before venv/pip/port work
if [ -n "${OP_CLIENT_ID_SECRET}" ] || [ -n "${OP_CLIENT_SECRET_SECRET}" ]; then
  TRADESTATION_CLIENT_ID=$(read_credential "${OP_CLIENT_ID_SECRET}" "${TRADESTATION_CLIENT_ID:-}" "Client ID" 2>/dev/null || echo "")
  TRADESTATION_CLIENT_SECRET=$(read_credential "${OP_CLIENT_SECRET_SECRET}" "${TRADESTATION_CLIENT_SECRET:-}" "Client Secret" 2>/dev/null || echo "")
  if [ -z "${TRADESTATION_CLIENT_ID}" ] || [ -z "${TRADESTATION_CLIENT_SECRET}" ]; then
    echo "Error: TradeStation 1Password refs set but could not read secrets." >&2
    echo "  Run 'op signin' or set TRADESTATION_CLIENT_ID and TRADESTATION_CLIENT_SECRET in the environment." >&2
    exit 1
  fi
  export TRADESTATION_CLIENT_ID
  export TRADESTATION_CLIENT_SECRET
  # Skip disabled-mode path; proceed to start service
else
  TRADESTATION_CLIENT_ID=$(read_credential "${OP_CLIENT_ID_SECRET}" "${TRADESTATION_CLIENT_ID:-}" "Client ID" || echo "")
  TRADESTATION_CLIENT_SECRET=$(read_credential "${OP_CLIENT_SECRET_SECRET}" "${TRADESTATION_CLIENT_SECRET:-}" "Client Secret" || echo "")
fi

# If no credentials (and we didn't exit above), allow disabled mode: start service, health only.
# Same as IB/Alpaca: service starts; /api/health returns status "disabled"; other endpoints return empty/error.
if [ -z "${TRADESTATION_CLIENT_ID}" ] || [ -z "${TRADESTATION_CLIENT_SECRET}" ]; then
  echo "TradeStation credentials not set; service will start in disabled mode (health only; TUI will show 'TradeStation: disabled')." >&2
  echo "To enable: set TRADESTATION_CLIENT_ID/TRADESTATION_CLIENT_SECRET or use 1Password (OP_TRADESTATION_*)" >&2
  export TRADESTATION_CLIENT_ID="${TRADESTATION_CLIENT_ID:-}"
  export TRADESTATION_CLIENT_SECRET="${TRADESTATION_CLIENT_SECRET:-}"
else
  export TRADESTATION_CLIENT_ID
  export TRADESTATION_CLIENT_SECRET
fi

cd "$PYTHON_DIR"

# Find Python command
find_python || exit 1

# Set up virtual environment
setup_venv "${PYTHON_DIR}" || exit 1

# Install required packages
install_python_packages "${VENV_PYTHON}" "uvicorn[standard]" "fastapi" || exit 1

# Use venv Python for all subsequent operations
PYTHON_CMD="${VENV_PYTHON}"

# Check if integration module is available
if ! test_python_import "${PYTHON_CMD}" "integration.tradestation_service" "app"; then
  echo "Error: Cannot import tradestation_service. Make sure you're in the python directory." >&2
  exit 1
fi

# Get TradeStation service port from config (default: 8001)
TRADESTATION_PORT=$(config_get_port "tradestation" 8001)

# Check if port is available and verify service identity
if ! check_port_with_service "${PYTHON_CMD}" "127.0.0.1" "${TRADESTATION_PORT}" "TRADESTATION_SERVICE" "TradeStation"; then
  exit 1
fi
if [ -n "${SERVICE_ALREADY_RUNNING:-}" ]; then
  exit 0
fi

echo "Starting TradeStation service on http://127.0.0.1:${TRADESTATION_PORT}" >&2
echo "Set VITE_API_URL=http://127.0.0.1:${TRADESTATION_PORT}/api/snapshot in your web app" >&2
echo "" >&2

# Run the service
"${PYTHON_CMD}" -m uvicorn integration.tradestation_service:app --host 127.0.0.1 --port "${TRADESTATION_PORT}" --reload
