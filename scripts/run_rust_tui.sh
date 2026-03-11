#!/bin/bash
# run_rust_tui.sh - Run the Rust TUI (Ratatui)
#
# This script runs the Rust TUI service which replaces the Python TUI.
# It subscribes to NATS for live trading state and renders a terminal UI.
#
# Usage:
#   ./scripts/run_rust_tui.sh
#
# Environment overrides:
#   NATS_URL      Override NATS server URL
#   BACKEND_ID    Override snapshot subject suffix
#   REST_URL      Override REST fallback base URL
#   WATCHLIST     Override highlighted symbols
#   TICK_MS       Override UI redraw interval ms
#   REST_POLL_MS  Override REST polling interval ms
#   REST_FALLBACK Override REST fallback on/off
#   IB_BOX_SPREAD_CONFIG  Override shared config path
#
# Examples:
#   NATS_URL=nats://localhost:4222 BACKEND_ID=alpaca ./scripts/run_rust_tui.sh
#   TICK_MS=500 WATCHLIST=SPY,QQQ,MSTR ./scripts/run_rust_tui.sh

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
BACKEND_DIR="${PROJECT_ROOT}/agents/backend"

# Check if NATS is running
check_nats() {
  if [[ -z "${NATS_URL:-}" ]]; then
    return 0
  fi

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
  echo "Starting Rust TUI..."
  echo "  Shared config: ${IB_BOX_SPREAD_CONFIG:-auto-discovery}"
  echo "  NATS_URL:      ${NATS_URL:-<shared config / binary default>}"
  echo "  BACKEND_ID:    ${BACKEND_ID:-<shared config / binary default>}"
  echo "  REST_URL:      ${REST_URL:-<shared config / binary default>}"
  echo "  WATCHLIST:     ${WATCHLIST:-<shared config / binary default>}"
  echo "  TICK_MS:       ${TICK_MS:-<shared config / binary default>}"
  echo "  REST_POLL_MS:  ${REST_POLL_MS:-<shared config / binary default>}"
  echo "  REST_FALLBACK: ${REST_FALLBACK:-<shared config / binary default>}"
  echo ""

  cd "${PROJECT_ROOT}"
  exec "${BACKEND_DIR}/target/debug/tui_service"
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
