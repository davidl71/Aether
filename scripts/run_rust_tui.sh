#!/bin/bash
# run_rust_tui.sh - Run the Rust TUI (Ratatui)
#
# This script runs the Rust TUI service which replaces the Python TUI.
# It subscribes to NATS for live trading state and renders a terminal UI.
#
# Usage:
#   ./scripts/run_rust_tui.sh
#
# Environment:
#   NATS_URL      NATS server URL (default: nats://localhost:4222)
#   BACKEND_ID    Snapshot subject suffix (default: ib)
#   REST_URL      REST fallback URL (default: read from config)
#   WATCHLIST     Comma-separated symbols to highlight (default: SPX,XSP,NDX)
#   TICK_MS       UI redraw interval ms (default: 250)
#
# Examples:
#   NATS_URL=nats://localhost:4222 BACKEND_ID=alpaca ./scripts/run_rust_tui.sh
#   TICK_MS=500 WATCHLIST=SPY,QQQ,MSTR ./scripts/run_rust_tui.sh

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
BACKEND_DIR="${PROJECT_ROOT}/agents/backend"

# Source config for port detection
# shellcheck source=scripts/include/config.sh
source "${SCRIPT_DIR}/include/config.sh"

# Get backend port from config (default 9090)
BACKEND_PORT=$(config_get_rust_backend_port 9090)

# Default values
NATS_URL="${NATS_URL:-nats://localhost:4222}"
BACKEND_ID="${BACKEND_ID:-ib}"
REST_URL="${REST_URL:-http://localhost:${BACKEND_PORT}}"
WATCHLIST="${WATCHLIST:-SPX,XSP,NDX}"
TICK_MS="${TICK_MS:-250}"
REST_POLL_MS="${REST_POLL_MS:-2000}"
REST_FALLBACK="${REST_FALLBACK:-true}"

# Check if NATS is running
check_nats() {
  if command -v nats &>/dev/null; then
    if ! nats -s "${NATS_URL}" server info &>/dev/null 2>&1; then
      echo "Warning: NATS server not reachable at ${NATS_URL}" >&2
      return 1
    fi
  elif command -v nc &>/dev/null; then
    local nats_host="${NATS_URL##*://}"
    local nats_port="${nats_host##*:}"
    if ! nc -z "${nats_host%:*}" "${nats_port:-4222}" 2>/dev/null; then
      echo "Warning: NATS server not reachable at ${NATS_URL}" >&2
      return 1
    fi
  fi
  return 0
}

# Build the TUI service if needed
build_tui() {
  if [[ ! -f "${BACKEND_DIR}/target/debug/tui_service" ]]; then
    echo "Building tui_service..."
    (cd "${BACKEND_DIR}" && cargo build -p tui_service)
  fi
}

# Run the Rust TUI
run_tui() {
  export NATS_URL
  export BACKEND_ID
  export REST_URL
  export WATCHLIST
  export TICK_MS
  export REST_POLL_MS
  export REST_FALLBACK

  echo "Starting Rust TUI..."
  echo "  NATS_URL:      ${NATS_URL}"
  echo "  BACKEND_ID:    ${BACKEND_ID}"
  echo "  REST_URL:      ${REST_URL}"
  echo "  WATCHLIST:     ${WATCHLIST}"
  echo "  TICK_MS:       ${TICK_MS}"
  echo "  REST_POLL_MS:  ${REST_POLL_MS}"
  echo "  REST_FALLBACK:  ${REST_FALLBACK}"
  echo ""

  cd "${BACKEND_DIR}"
  exec cargo run -p tui_service
}

# Main
main() {
  # Check for required tools
  if ! command -v cargo &>/dev/null; then
    echo "Error: cargo not found. Install Rust via rustup." >&2
    exit 1
  fi

  # Check NATS (warning only)
  check_nats || true

  # Build if needed
  build_tui

  # Run
  run_tui
}

main "$@"
