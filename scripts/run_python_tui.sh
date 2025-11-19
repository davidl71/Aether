#!/bin/bash
# run_python_tui.sh - Run the Python TUI application
#
# This script runs the Python TUI, which replaces the C++ TUI for better
# performance and easier maintenance. It shares data models with the PWA.
#
# Usage:
#   ./scripts/run_python_tui.sh [provider_type] [endpoint]
#
# Examples:
#   ./scripts/run_python_tui.sh mock
#   ./scripts/run_python_tui.sh rest http://localhost:8080/api/snapshot
#   ./scripts/run_python_tui.sh file web/public/data/snapshot.json

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

# Default values
PROVIDER_TYPE="${1:-mock}"
ENDPOINT="${2:-}"

# Activate virtual environment if it exists
if [ -d "${PROJECT_ROOT}/.venv" ]; then
    source "${PROJECT_ROOT}/.venv/bin/activate"
fi

# Set environment variables based on provider type
export TUI_BACKEND="${PROVIDER_TYPE}"

if [ "${PROVIDER_TYPE}" = "rest" ] && [ -n "${ENDPOINT}" ]; then
    export TUI_API_URL="${ENDPOINT}"
elif [ "${PROVIDER_TYPE}" = "file" ] && [ -n "${ENDPOINT}" ]; then
    export TUI_SNAPSHOT_FILE="${ENDPOINT}"
fi

# Run the Python TUI
cd "${PROJECT_ROOT}"

# Try python3 first, fall back to python
if command -v python3 &> /dev/null; then
    PYTHON_CMD=python3
elif command -v python &> /dev/null; then
    PYTHON_CMD=python
else
    echo "Error: python3 or python not found" >&2
    exit 1
fi

${PYTHON_CMD} -m python.tui
