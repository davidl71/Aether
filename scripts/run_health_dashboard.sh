#!/usr/bin/env bash
# Run Health Dashboard (unified health JSON from NATS) on port 8011.
# Set HEALTH_DASHBOARD_PORT to override.
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
PYTHON_DIR="${ROOT_DIR}/python"
SCRIPTS_DIR="${ROOT_DIR}/scripts"

if [ -f "${SCRIPTS_DIR}/include/config.sh" ]; then
  # shellcheck source=scripts/include/config.sh
  source "${SCRIPTS_DIR}/include/config.sh"
else
  config_get_port() { echo "${2:-8011}"; }
fi

cd "$PYTHON_DIR"

PYTHON_CMD="python3"
if [ -d "${PYTHON_DIR}/.venv" ] && [ -f "${PYTHON_DIR}/.venv/bin/python" ]; then
  PYTHON_CMD="${PYTHON_DIR}/.venv/bin/python"
fi

if ! "${PYTHON_CMD}" -c "from services.health_dashboard import app" 2>/dev/null; then
  echo "Error: Cannot import health_dashboard. Run from project root or set PYTHONPATH." >&2
  exit 1
fi

PORT=$(config_get_port "health_dashboard" 8011)
PORT=${HEALTH_DASHBOARD_PORT:-$PORT}

echo "Starting Health Dashboard on port ${PORT}..." >&2
exec "${PYTHON_CMD}" -m uvicorn services.health_dashboard:app --host 0.0.0.0 --port "${PORT}" --reload
