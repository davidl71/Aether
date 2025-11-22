#!/usr/bin/env bash
# activate_python_env.sh - Activate Python 3.12 virtual environment for PyO3 compatibility
# Usage: source ./scripts/activate_python_env.sh
#        or: . ./scripts/activate_python_env.sh

set -euo pipefail

# Handle both sourced and executed cases
if [ -n "${BASH_SOURCE[0]:-}" ]; then
  SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
else
  SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
fi
BACKEND_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"
VENV_DIR="${BACKEND_DIR}/.venv"
PYTHON312="/usr/local/bin/python3.12"

# Check if Python 3.12 is available
if [ ! -f "${PYTHON312}" ]; then
  echo "❌ Python 3.12 not found at ${PYTHON312}" >&2
  echo "   Install Python 3.12: brew install python@3.12" >&2
  return 1 2>/dev/null || exit 1
fi

# Create venv if it doesn't exist
if [ ! -d "${VENV_DIR}" ]; then
  echo "Creating Python 3.12 virtual environment..."
  "${PYTHON312}" -m venv "${VENV_DIR}"
  echo "✅ Virtual environment created at ${VENV_DIR}"
fi

# Activate virtual environment
source "${VENV_DIR}/bin/activate"

# Set PyO3 to use this Python
export PYO3_PYTHON="${VENV_DIR}/bin/python"
export PYO3_PYTHON_VERSION="3.12"

# Verify Python version
PYTHON_VERSION=$(python --version 2>&1 | cut -d' ' -f2 | cut -d'.' -f1,2)
if [ "${PYTHON_VERSION}" != "3.12" ]; then
  echo "⚠️  Warning: Python version is ${PYTHON_VERSION}, expected 3.12" >&2
else
  echo "✅ Python ${PYTHON_VERSION} activated"
  echo "✅ PyO3 configured: PYO3_PYTHON=${PYO3_PYTHON}"
fi

# Install Python dependencies if pyproject.toml exists
if [ -f "${BACKEND_DIR}/python/pyproject.toml" ]; then
  if [ ! -f "${VENV_DIR}/.deps_installed" ]; then
    echo "Installing Python dependencies..."
    pip install --upgrade pip > /dev/null 2>&1
    pip install -e "${BACKEND_DIR}/python" > /dev/null 2>&1
    touch "${VENV_DIR}/.deps_installed"
    echo "✅ Python dependencies installed"
  fi
fi

echo ""
echo "Environment ready for Rust builds with PyO3"
echo "Run: cargo check -p backend_service"
