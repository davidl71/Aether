#!/usr/bin/env bash
# Run Risk-Free Rate service for extracting and comparing rates from box spreads
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/../.." && pwd)"
PYTHON_DIR="$ROOT_DIR/python"
SCRIPTS_DIR="${ROOT_DIR}/scripts"

# Load shared config functions
if [ -f "${SCRIPTS_DIR}/include/config.sh" ]; then
  # shellcheck source=../../scripts/include/config.sh
  source "${SCRIPTS_DIR}/include/config.sh"
else
  echo "Warning: config.sh not found, using default port 8004" >&2
  config_get_port() {
    echo "${2:-8004}"
  }
fi

cd "$PYTHON_DIR"

# Check for Python
if ! command -v python3 >/dev/null 2>&1; then
  echo "Error: python3 not found" >&2
  exit 1
fi

PYTHON_CMD="python3"

# Check if virtual environment exists
VENV_DIR="${PYTHON_DIR}/.venv"
if [ -d "${VENV_DIR}" ] && [ -f "${VENV_DIR}/bin/python" ]; then
  PYTHON_CMD="${VENV_DIR}/bin/python"
  echo "Using virtual environment Python: ${PYTHON_CMD}" >&2
else
  echo "Warning: Virtual environment not found, using system Python" >&2
fi

# Check if required packages are installed
MISSING_PACKAGES=()

if ! "${PYTHON_CMD}" -c "import fastapi" 2>/dev/null; then
  MISSING_PACKAGES+=("fastapi" "uvicorn[standard]")
fi

if [ ${#MISSING_PACKAGES[@]} -gt 0 ]; then
  echo "Installing missing packages: ${MISSING_PACKAGES[*]}..." >&2
  "${PYTHON_CMD}" -m pip install --quiet "${MISSING_PACKAGES[@]}" >&2
fi

# Check if integration module is available
if ! "${PYTHON_CMD}" -c "from integration.risk_free_rate_service import app" 2>/dev/null; then
  echo "Error: Cannot import risk_free_rate_service. Make sure you're in the python directory." >&2
  exit 1
fi

# Get port from config
RISK_FREE_RATE_PORT=$(config_get_port "risk_free_rate" 8004)

# Run the service
echo "Starting Risk-Free Rate service on port ${RISK_FREE_RATE_PORT}..." >&2
exec "${PYTHON_CMD}" -m uvicorn integration.risk_free_rate_service:app --host 127.0.0.1 --port "${RISK_FREE_RATE_PORT}" --reload
