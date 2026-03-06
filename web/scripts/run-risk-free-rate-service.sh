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

# Export 1Password token and OP_FRED_API_KEY_SECRET (op reference) so service can resolve FRED key from 1Password
if [ -f "${SCRIPTS_DIR}/setup_op_service_account.sh" ]; then
  # shellcheck source=../../scripts/setup_op_service_account.sh
  source "${SCRIPTS_DIR}/setup_op_service_account.sh" 2>/dev/null || true
  load_token 2>/dev/null && export OP_SERVICE_ACCOUNT_TOKEN || true
  if [[ -z "${OP_FRED_API_KEY_SECRET:-}" ]]; then
    FRED_VAULT="${OP_SETUP_VAULT:-$(read_vault_default 2>/dev/null)}"
    if [[ -n "${FRED_VAULT}" ]]; then
      export OP_FRED_API_KEY_SECRET="op://${FRED_VAULT}/FRED API/credential"
    fi
  fi
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
  MISSING_PACKAGES+=("fastapi")
fi
if ! "${PYTHON_CMD}" -c "import uvicorn" 2>/dev/null; then
  MISSING_PACKAGES+=("uvicorn[standard]")
fi
if ! "${PYTHON_CMD}" -c "import yfinance" 2>/dev/null; then
  MISSING_PACKAGES+=("yfinance")
fi

if [ ${#MISSING_PACKAGES[@]} -gt 0 ]; then
  # Try uv sync with rates extra first (includes yfinance fallback)
  if [ -f "${PYTHON_DIR}/pyproject.toml" ]; then
    echo "Running uv sync --extra rates to install dependencies..." >&2
    cd "${PYTHON_DIR}" && uv sync --extra rates 2>&1 || true
    cd - >/dev/null
  fi
  # Fallback to pip install if uv sync didn't work
  if ! "${PYTHON_CMD}" -c "import yfinance" 2>/dev/null; then
    echo "Installing missing packages: ${MISSING_PACKAGES[*]}..." >&2
    "${PYTHON_CMD}" -m pip install --quiet "${MISSING_PACKAGES[@]}" >&2
  fi
fi

# Check if integration module is available (show traceback on failure)
set +e
IMPORT_ERR=$("${PYTHON_CMD}" -c "from integration.risk_free_rate_service import app" 2>&1)
EXIT=$?
set -e
if [ "$EXIT" -ne 0 ]; then
  echo "Error: Cannot import risk_free_rate_service." >&2
  echo "Run from repo root and ensure deps: cd python && uv sync --extra dev --extra tui" >&2
  echo "" >&2
  echo "$IMPORT_ERR" >&2
  exit 1
fi

# Get port from config
RISK_FREE_RATE_PORT=$(config_get_port "risk_free_rate" 8004)

# Run the service
echo "Starting Risk-Free Rate service on port ${RISK_FREE_RATE_PORT}..." >&2
exec "${PYTHON_CMD}" -m uvicorn integration.risk_free_rate_service:app --host 127.0.0.1 --port "${RISK_FREE_RATE_PORT}" --reload
