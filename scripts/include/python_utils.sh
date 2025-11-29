#!/usr/bin/env bash
# python_utils.sh - Shared Python utility functions for service scripts
# Provides common functions for Python virtual environment management, package installation, etc.
#
# AI CONTEXT FOR AGENTS:
# =====================
# This file contains shared functions used by ALL service startup scripts:
#   - web/scripts/run-*.sh (Alpaca, IB, TradeStation, Discount Bank)
#   - scripts/start_alpaca_service.sh
#
# PURPOSE: Eliminate code duplication across service scripts (~95% reduction)
# PATTERN: Functions are sourced by service scripts, not executed directly
# DEPENDENCIES: Requires bash, Python 3, venv module (or uv for faster operation)
# TESTING: See spec/scripts/include/python_utils_spec.sh
#
# USAGE PATTERN IN SERVICE SCRIPTS:
#   source "${SCRIPTS_DIR}/include/python_utils.sh"
#   find_python || exit 1
#   setup_venv "${PYTHON_DIR}" || exit 1
#   install_python_packages "${VENV_PYTHON}" "fastapi" "uvicorn[standard]"
#
# GLOBAL VARIABLES SET BY FUNCTIONS:
#   - PYTHON_CMD: Path to Python interpreter (set by find_python)
#   - VENV_DIR: Path to virtual environment directory (set by setup_venv)
#   - ACTIVATE_PATH: Path to venv activation script (set by setup_venv)
#   - VENV_PYTHON: Path to venv Python executable (set by setup_venv)
#   - INIT_PY: Path to __init__.py file (set by disable_init_py)
#   - INIT_PY_BAK: Backup path for __init__.py (set by disable_init_py)
#
# ERROR HANDLING: All functions return 0 on success, 1 on failure
#                 Error messages are written to stderr
#                 Service scripts should check return codes and exit on failure

# Find Python command (python3 or python)
# Sets PYTHON_CMD variable and returns 0 if found, 1 if not found
#
# AI CONTEXT:
# - Searches for Python interpreter in PATH
# - Prefers python3 over python (modern systems)
# - Used by all service scripts before setting up virtual environments
# - Sets global PYTHON_CMD variable for use by other functions
#
# RETURN VALUES:
#   0: Python found, PYTHON_CMD is set
#   1: Python not found, error message written to stderr
#
# EXAMPLE:
#   find_python || exit 1
#   echo "Using Python: ${PYTHON_CMD}"
find_python() {
  PYTHON_CMD=""

  # Try python3 first (most common on macOS/Linux)
  if command -v python3 >/dev/null 2>&1; then
    PYTHON_CMD="python3"
  # Fall back to python
  elif command -v python >/dev/null 2>&1; then
    PYTHON_CMD="python"
  fi

  if [ -z "${PYTHON_CMD}" ]; then
    echo "Error: Python not found. Please install Python 3." >&2
    return 1
  fi

  return 0
}

# Set up virtual environment
# Usage: setup_venv <python_dir> [venv_dir]
# Sets VENV_DIR, ACTIVATE_PATH, and VENV_PYTHON variables
# Returns 0 on success, 1 on failure
#
# AI CONTEXT:
# - Creates Python virtual environment if it doesn't exist
# - Prefers `uv venv` when available (faster, modern tooling)
# - Falls back to `python3 -m venv` for compatibility
# - Activates existing venv if present
# - Updates pip and wheel in the venv
# - Sets global variables for use by other functions
# - REQUIRES: find_python() must be called first to set PYTHON_CMD
#
# PARAMETERS:
#   python_dir: Directory containing Python code (required)
#   venv_dir: Optional path to venv (defaults to ${python_dir}/.venv)
#
# GLOBAL VARIABLES SET:
#   VENV_DIR: Path to virtual environment directory
#   ACTIVATE_PATH: Path to venv/bin/activate script
#   VENV_PYTHON: Path to venv/bin/python executable
#   USE_UV: Set to "1" if uv is being used, empty otherwise
#
# SIDE EFFECTS:
#   - Creates .venv directory if it doesn't exist
#   - Sources venv/bin/activate (modifies current shell environment)
#   - Updates pip/wheel packages
#
# EXAMPLE:
#   find_python || exit 1
#   setup_venv "${PYTHON_DIR}" || exit 1
#   "${VENV_PYTHON}" -m pip install package
setup_venv() {
  local python_dir="${1:-}"
  local venv_dir="${2:-${python_dir}/.venv}"

  if [ -z "${python_dir}" ]; then
    echo "Error: python_dir required" >&2
    return 1
  fi

  VENV_DIR="${venv_dir}"
  ACTIVATE_PATH="${VENV_DIR}/bin/activate"
  USE_UV=""

  # Create virtual environment if it doesn't exist
  if [ ! -f "${ACTIVATE_PATH}" ]; then
    # Prefer uv if available (faster, modern tooling)
    if command -v uv >/dev/null 2>&1; then
      echo "Creating Python virtual environment with uv at ${VENV_DIR}..." >&2
      if uv venv "${VENV_DIR}" 2>/dev/null; then
        USE_UV="1"
      else
        echo "Warning: uv venv failed, falling back to standard venv" >&2
        USE_UV=""
      fi
    fi

    # Fallback to standard venv if uv not available or failed
    if [ -z "${USE_UV}" ] && [ ! -f "${ACTIVATE_PATH}" ]; then
      echo "Creating Python virtual environment with venv at ${VENV_DIR}..." >&2
      if ! "${PYTHON_CMD}" -m venv "${VENV_DIR}" 2>/dev/null; then
        echo "Error: Failed to create virtual environment. Please ensure venv module is available." >&2
        echo "  Try: ${PYTHON_CMD} -m ensurepip --upgrade" >&2
        return 1
      fi
    fi
  else
    echo "Using existing virtual environment at ${VENV_DIR}" >&2
    # Detect if existing venv was created with uv (check for uv-specific markers)
    # For now, assume standard venv unless we can detect otherwise
  fi

  # Activate virtual environment
  # shellcheck disable=SC1090
  source "${ACTIVATE_PATH}"

  # Update pip in virtual environment (only if not using uv)
  if [ -z "${USE_UV}" ]; then
    "${PYTHON_CMD}" -m pip install --quiet --upgrade pip wheel >/dev/null 2>&1 || true
  fi

  # Set venv Python path
  VENV_PYTHON="${VENV_DIR}/bin/python"

  return 0
}

# Install Python packages if missing
# Usage: install_python_packages <venv_python> <package1> [package2] ...
# Returns 0 on success, 1 on failure
#
# AI CONTEXT:
# - Checks if packages are installed before installing
# - Handles packages with extras like "uvicorn[standard]"
# - Special handling for uvicorn WebSocket support detection
# - Only installs missing packages (idempotent)
# - Used by all service scripts to ensure dependencies are available
#
# PARAMETERS:
#   venv_python: Path to venv Python executable (required)
#   package1, package2, ...: Package names to install (required, at least one)
#
# SPECIAL HANDLING:
#   - "uvicorn[standard]": Checks for uvicorn AND websockets/wsproto
#   - Packages with extras: Extracts base package name for import check
#   - Import names: Converts package names (with hyphens) to import names (with underscores)
#
# EXAMPLE:
#   install_python_packages "${VENV_PYTHON}" "fastapi" "uvicorn[standard]" "alpaca-py"
#
# NOTE: This function is idempotent - safe to call multiple times
install_python_packages() {
  local venv_python="${1:-}"
  shift

  if [ -z "${venv_python}" ] || [ $# -eq 0 ]; then
    echo "Error: venv_python and at least one package required" >&2
    return 1
  fi

  local missing_packages=()
  local package

  for package in "$@"; do
    # Check if package is installed
    # Handle packages with extras like "uvicorn[standard]"
    local base_package="${package%%[*}"

    # Special handling for uvicorn - check for websockets separately
    if [ "${base_package}" = "uvicorn" ]; then
      if ! "${venv_python}" -c "import uvicorn" 2>/dev/null; then
        missing_packages+=("${package}")
      elif ! "${venv_python}" -c "import websockets" 2>/dev/null && ! "${venv_python}" -c "import wsproto" 2>/dev/null; then
        # uvicorn installed but missing WebSocket support
        missing_packages+=("websockets")
      fi
    else
      # For other packages, try importing the base name
      local import_name="${base_package//-/_}"
      if ! "${venv_python}" -c "import ${import_name}" 2>/dev/null; then
        missing_packages+=("${package}")
      fi
    fi
  done

  if [ ${#missing_packages[@]} -gt 0 ]; then
    echo "Installing missing packages in virtual environment: ${missing_packages[*]}..." >&2
    # Prefer uv pip install if uv is available (faster)
    if command -v uv >/dev/null 2>&1; then
      uv pip install --quiet "${missing_packages[@]}" >&2 || return 1
    else
      "${venv_python}" -m pip install --quiet "${missing_packages[@]}" >&2 || return 1
    fi
  fi

  return 0
}

# Test Python module import
# Usage: test_python_import <venv_python> <module_path> [module_name]
# Example: test_python_import "${VENV_PYTHON}" "integration.alpaca_service" "app"
# Returns 0 if import succeeds, 1 if it fails
#
# AI CONTEXT:
# - Tests if a Python module can be imported before starting services
# - Temporarily disables __init__.py to avoid dependency issues
# - Used by service scripts to verify dependencies before startup
# - Helps catch import errors early with clear error messages
#
# PARAMETERS:
#   venv_python: Path to venv Python executable (required)
#   module_path: Python module path (e.g., "integration.alpaca_service") (required)
#   module_name: Name of object to import (defaults to "app")
#
# SIDE EFFECTS:
#   - Temporarily renames integration/__init__.py to integration/__init__.py.bak
#   - Restores original file after test (even on failure)
#
# EXAMPLE:
#   if ! test_python_import "${VENV_PYTHON}" "integration.alpaca_service" "app"; then
#     echo "Error: Cannot import alpaca_service" >&2
#     exit 1
#   fi
#
# NOTE: This function handles __init__.py issues that can cause import failures
#       when __init__.py has dependencies that aren't available
test_python_import() {
  local venv_python="${1:-}"
  local module_path="${2:-}"
  local module_name="${3:-app}"

  if [ -z "${venv_python}" ] || [ -z "${module_path}" ]; then
    echo "Error: venv_python and module_path required" >&2
    return 1
  fi

  local import_test
  import_test=$("${venv_python}" -c "
import sys
import os
# Temporarily disable __init__.py by renaming it
init_py = 'integration/__init__.py'
backup_py = 'integration/__init__.py.bak'
if os.path.exists(init_py):
    os.rename(init_py, backup_py)
try:
    sys.path.insert(0, '.')
    from ${module_path} import ${module_name}
    print('OK')
except Exception as e:
    print(f'ERROR: {e}')
finally:
    if os.path.exists(backup_py):
        os.rename(backup_py, init_py)
" 2>&1)

  if echo "${import_test}" | grep -q "OK"; then
    return 0
  else
    echo "${import_test}" >&2
    return 1
  fi
}

# Temporarily disable __init__.py to avoid dependency issues
# Usage: disable_init_py <python_dir>
# Sets INIT_PY and INIT_PY_BAK variables and creates trap
#
# AI CONTEXT:
# - Renames integration/__init__.py to avoid import dependency issues
# - Creates EXIT trap to restore file when script exits
# - Used by service scripts before running uvicorn
# - Prevents import errors when __init__.py has unavailable dependencies
#
# PARAMETERS:
#   python_dir: Directory containing integration/__init__.py (required)
#
# GLOBAL VARIABLES SET:
#   INIT_PY: Path to integration/__init__.py
#   INIT_PY_BAK: Path to backup file (integration/__init__.py.bak)
#
# SIDE EFFECTS:
#   - Renames integration/__init__.py if it exists
#   - Creates EXIT trap to restore file on script exit
#   - Trap ensures cleanup even if script is interrupted
#
# EXAMPLE:
#   disable_init_py "${PYTHON_DIR}" || exit 1
#   # Run uvicorn - __init__.py is disabled
#   # File will be restored automatically on exit
#
# NOTE: This is a workaround for __init__.py dependency issues
#       The trap ensures the file is restored even if the script crashes
disable_init_py() {
  local python_dir="${1:-}"

  if [ -z "${python_dir}" ]; then
    echo "Error: python_dir required" >&2
    return 1
  fi

  INIT_PY="${python_dir}/integration/__init__.py"
  INIT_PY_BAK="${python_dir}/integration/__init__.py.bak"

  if [ -f "${INIT_PY}" ]; then
    mv "${INIT_PY}" "${INIT_PY_BAK}"
    trap "mv '${INIT_PY_BAK}' '${INIT_PY}' 2>/dev/null || true" EXIT
  fi

  return 0
}
