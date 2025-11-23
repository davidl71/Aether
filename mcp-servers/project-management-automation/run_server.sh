#!/bin/bash
# Wrapper script to run MCP server with virtual environment

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
VENV_PYTHON="${SCRIPT_DIR}/venv/bin/python3"
SERVER_SCRIPT="${SCRIPT_DIR}/server.py"

# Check if venv exists
if [ ! -f "$VENV_PYTHON" ]; then
    echo "Error: Virtual environment not found at ${SCRIPT_DIR}/venv" >&2
    exit 1
fi

# Run server with venv Python
exec "$VENV_PYTHON" "$SERVER_SCRIPT" "$@"
