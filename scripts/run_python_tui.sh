#!/bin/bash
# run_python_tui.sh - Run the Python TUI application
#
# This script runs the Python TUI, which replaces the C++ TUI for better
# performance and easier maintenance. It shares data models with the PWA.
#
# Usage:
#   ./scripts/run_python_tui.sh [provider_type] [endpoint]
#   ./scripts/run_python_tui.sh --dev [provider_type] [endpoint]   # Textual dev mode: CSS live reload
#
# Live refresh (restart TUI when Python source changes):
#   ./scripts/dev_watch_tui.sh [--dev] [-- mock]
#
# Environment:
#   UV_SYNC_QUIET=1  — run uv sync with --quiet (default: show uv output).
#
# Examples:
#   ./scripts/run_python_tui.sh mock
#   ./scripts/run_python_tui.sh rest http://localhost:8080/api/snapshot
#   ./scripts/run_python_tui.sh --dev rest http://localhost:8002/api/v1/snapshot

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

# Detect interactive session (stdin and stdout are TTYs)
is_interactive() {
  [[ -t 0 ]] && [[ -t 1 ]]
}

# Consider shared config "missing" only if no candidate config file exists (same logic as Python SharedConfigLoader).
# Checks: IB_BOX_SPREAD_CONFIG, ~/.config/ib_box_spread/config.json, project config/config.json and config.example.json.
config_missing() {
  local root="${PROJECT_ROOT}"
  local home="${HOME:-}"
  # Explicit env path
  if [[ -n "${IB_BOX_SPREAD_CONFIG:-}" ]]; then
    [[ -f "${IB_BOX_SPREAD_CONFIG}" ]] && return 1
  fi
  # Home config
  [[ -n "${home}" && -f "${home}/.config/ib_box_spread/config.json" ]] && return 1
  # macOS Application Support
  [[ "$(uname -s)" = "Darwin" && -n "${home}" && -f "${home}/Library/Application Support/ib_box_spread/config.json" ]] && return 1
  # Project config (repo)
  [[ -f "${root}/config/config.json" ]] && return 1
  [[ -f "${root}/config/config.example.json" ]] && return 1
  return 0
}

# Optional --dev: use textual run --dev for CSS live reload
DEV_MODE=false
if [[ "${1:-}" == "--dev" ]]; then
  DEV_MODE=true
  shift
fi

# Default values
PROVIDER_TYPE="${1:-mock}"
ENDPOINT="${2:-}"

# Activate virtual environment if it exists
if [ -d "${PROJECT_ROOT}/.venv" ]; then
    source "${PROJECT_ROOT}/.venv/bin/activate"
    PYTHON_CMD="${PROJECT_ROOT}/.venv/bin/python"
else
    PYTHON_CMD=""
fi

# Run uv sync by default (ensures deps including 1Password SDK and TUI deps are installed)
if command -v uv &>/dev/null; then
  (cd "${PROJECT_ROOT}/python" && uv sync --extra onepassword --extra tui ${UV_SYNC_QUIET:+--quiet} 2>/dev/null) || true
  if [ -x "${PROJECT_ROOT}/python/.venv/bin/python" ]; then
    PYTHON_CMD="${PROJECT_ROOT}/python/.venv/bin/python"
  fi
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

# If config is missing and we're in an interactive terminal, run full 1Password setup first
if config_missing && is_interactive; then
  echo "No shared config found (checked: config/config.json, config.example.json, ~/.config/ib_box_spread/config.json, IB_BOX_SPREAD_CONFIG). Starting interactive 1Password setup..."
  if "${SCRIPT_DIR}/setup_op_service_account.sh" setup-full; then
    # Export token into this shell so TUI can use op refs
    source "${SCRIPT_DIR}/setup_op_service_account.sh" 2>/dev/null || true
    if [[ -f "${PROJECT_ROOT}/config/config.example.json" ]] && ! [[ -f "${PROJECT_ROOT}/config/config.json" ]]; then
      echo "Tip: copy config/config.example.json to config/config.json and add op:// refs for secrets."
    fi
  fi
fi

# Export 1Password token into this shell if available (so TUI shows Auth: ready when token file exists)
if [[ -z "${OP_SERVICE_ACCOUNT_TOKEN:-}" ]]; then
  source "${SCRIPT_DIR}/setup_op_service_account.sh" 2>/dev/null || true
  load_token 2>/dev/null && export OP_SERVICE_ACCOUNT_TOKEN || true
fi
# Set FRED key to op reference when vault is known (so Benchmarks tab can use 1Password)
if [[ -z "${OP_FRED_API_KEY_SECRET:-}" ]] && [[ -n "${OP_SERVICE_ACCOUNT_TOKEN:-}" ]]; then
  FRED_VAULT="${OP_SETUP_VAULT:-$(read_vault_default 2>/dev/null)}"
  if [[ -n "${FRED_VAULT}" ]]; then
    export OP_FRED_API_KEY_SECRET="op://${FRED_VAULT}/FRED API/credential"
  fi
fi

# Use venv python if we have it, else try python3 / python from PATH
if [ -z "${PYTHON_CMD}" ]; then
    if command -v python3 &> /dev/null; then
        PYTHON_CMD=python3
    elif command -v python &> /dev/null; then
        PYTHON_CMD=python
    else
        echo "Error: python3 or python not found" >&2
        exit 1
    fi
fi

if [[ "$DEV_MODE" == "true" ]]; then
    if ! "${PYTHON_CMD}" -c "import textual" 2>/dev/null; then
        echo "Error: textual not found. Install with: pip install textual" >&2
        exit 1
    fi
    export PYTHONPATH="${PROJECT_ROOT}/python${PYTHONPATH:+:${PYTHONPATH}}"
    exec "${PYTHON_CMD}" -m textual run python/tui/app.py --dev
else
    exec "${PYTHON_CMD}" -m python.tui
fi
