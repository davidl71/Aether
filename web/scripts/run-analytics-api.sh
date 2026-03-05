#!/usr/bin/env bash
# Run unified Analytics API (calculations + risk-free rate) on port 8007.
# Set ANALYTICS_API_PORT or CALCULATIONS_API_PORT to override.
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/../.." && pwd)"
PYTHON_DIR="$ROOT_DIR/python"
SCRIPTS_DIR="${ROOT_DIR}/scripts"

if [ -f "${SCRIPTS_DIR}/include/config.sh" ]; then
  # shellcheck source=../../scripts/include/config.sh
  source "${SCRIPTS_DIR}/include/config.sh"
else
  config_get_port() { echo "${2:-8007}"; }
fi

cd "$PYTHON_DIR"

PYTHON_CMD="python3"
if [ -d "${PYTHON_DIR}/.venv" ] && [ -f "${PYTHON_DIR}/.venv/bin/python" ]; then
  PYTHON_CMD="${PYTHON_DIR}/.venv/bin/python"
fi

if ! "${PYTHON_CMD}" -c "from services.analytics_api import app" 2>/dev/null; then
  echo "Error: Cannot import analytics_api. Run from project root or set PYTHONPATH." >&2
  exit 1
fi

PORT=$(config_get_port "analytics" 8007)
PORT=${ANALYTICS_API_PORT:-${CALCULATIONS_API_PORT:-$PORT}}

echo "Starting Analytics API (calculations + risk-free rate) on port ${PORT}..." >&2
exec "${PYTHON_CMD}" -m uvicorn services.analytics_api:app --host 0.0.0.0 --port "${PORT}" --reload
