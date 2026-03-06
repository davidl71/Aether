#!/usr/bin/env bash
# Run JupyterLab service for interactive analysis and backtesting
# Follows the same pattern as other service scripts (alpaca, ib, etc.)
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
PYTHON_DIR="$ROOT_DIR/python"
SCRIPTS_DIR="${ROOT_DIR}/scripts"

# Load shared utility functions
# shellcheck source=scripts/include/config.sh
if [ -f "${SCRIPTS_DIR}/include/config.sh" ]; then
  source "${SCRIPTS_DIR}/include/config.sh"
fi

# shellcheck source=scripts/include/python_utils.sh
if [ -f "${SCRIPTS_DIR}/include/python_utils.sh" ]; then
  source "${SCRIPTS_DIR}/include/python_utils.sh"
else
  echo "Error: python_utils.sh not found" >&2
  exit 1
fi

# shellcheck source=scripts/include/service_utils.sh
if [ -f "${SCRIPTS_DIR}/include/service_utils.sh" ]; then
  source "${SCRIPTS_DIR}/include/service_utils.sh"
fi

cd "$PYTHON_DIR"

# Find Python command
find_python || exit 1

# Set up virtual environment
setup_venv "${PYTHON_DIR}" || exit 1

# Install required packages
install_python_packages "${VENV_PYTHON}" "jupyterlab>=4.0.0" || exit 1

# Use venv Python for all subsequent operations
PYTHON_CMD="${VENV_PYTHON}"

# Check if integration module file exists
if [ ! -f "integration/jupyterlab_service.py" ]; then
  echo "Error: integration/jupyterlab_service.py not found. Current directory: $(pwd)" >&2
  exit 1
fi

# Get JupyterLab service port from config (default: 8888)
JUPYTERLAB_PORT=$(config_get_port "jupyterlab" 8888)

# Check if port is available
if ! check_port_with_service "${PYTHON_CMD}" "127.0.0.1" "${JUPYTERLAB_PORT}" "JUPYTERLAB" "JupyterLab"; then
  exit 1
fi

# Test import before starting service
if ! test_python_import "${PYTHON_CMD}" "integration.jupyterlab_service" "main"; then
  echo "Error: Cannot import jupyterlab_service" >&2
  exit 1
fi

# Set notebook directory (default: project root/notebooks)
NOTEBOOK_DIR="${JUPYTERLAB_DIR:-${ROOT_DIR}/notebooks}"
mkdir -p "${NOTEBOOK_DIR}"

# Export environment variables for the service
export JUPYTERLAB_PORT="${JUPYTERLAB_PORT}"
export JUPYTERLAB_DIR="${NOTEBOOK_DIR}"

echo "Starting JupyterLab service on http://127.0.0.1:${JUPYTERLAB_PORT}" >&2
echo "Notebook directory: ${NOTEBOOK_DIR}" >&2
echo "Access token will be displayed in the output" >&2
echo "" >&2

# Temporarily disable __init__.py to avoid dependency issues
disable_init_py "${PYTHON_DIR}" || exit 1

# Run the service with PYTHONPATH set
export PYTHONPATH="${PYTHON_DIR}:${PYTHONPATH:-}"
"${PYTHON_CMD}" -m integration.jupyterlab_service
