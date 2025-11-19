#!/usr/bin/env bash
# Run Discount Bank service for PWA integration
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
install_python_packages "${VENV_PYTHON}" "fastapi" "uvicorn[standard]" "pydantic" || exit 1

# Use venv Python for all subsequent operations
PYTHON_CMD="${VENV_PYTHON}"

# Get Discount Bank service port from config (default: 8003)
DISCOUNT_BANK_PORT=$(config_get_port "discount_bank" 8003)

# Check if port is available (with basic check, no health endpoint for this service)
if ! config_check_port_available "${DISCOUNT_BANK_PORT}"; then
  echo "Warning: Port ${DISCOUNT_BANK_PORT} is already in use. Service may not start." >&2
  echo "  To use a different port, set DISCOUNT_BANK_PORT environment variable" >&2
  echo "  Or update config/config.json: services.discount_bank.port" >&2
fi

# Set default file path if not provided
if [ -z "${DISCOUNT_BANK_FILE_PATH:-}" ]; then
  export DISCOUNT_BANK_FILE_PATH="${HOME}/Downloads/DISCOUNT.dat"
  echo "Using default file path: ${DISCOUNT_BANK_FILE_PATH}" >&2
  echo "  To use a different path, set DISCOUNT_BANK_FILE_PATH environment variable." >&2
fi

# Run the service
echo "Starting Discount Bank service on port ${DISCOUNT_BANK_PORT}..." >&2
echo "  File path: ${DISCOUNT_BANK_FILE_PATH}" >&2
echo "  Health: http://localhost:${DISCOUNT_BANK_PORT}/api/health" >&2
echo "  Balance: http://localhost:${DISCOUNT_BANK_PORT}/api/balance" >&2
echo "" >&2

exec "${PYTHON_CMD}" -m uvicorn integration.discount_bank_service:app \
  --host 0.0.0.0 \
  --port "${DISCOUNT_BANK_PORT}" \
  --reload
