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
#   WATCHLIST     Override highlighted symbols
#   TICK_MS       Override UI redraw interval ms
#   SNAPSHOT_TTL_SECS  Override snapshot staleness seconds
#   IB_BOX_SPREAD_CONFIG  Override shared config path
#   RUST_TUI_VERBOSE  If set, print config before starting
#
# Examples:
#   NATS_URL=nats://localhost:4222 BACKEND_ID=ib ./scripts/run_rust_tui.sh
#   TICK_MS=500 WATCHLIST=SPY,QQQ,MSTR ./scripts/run_rust_tui.sh

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
BACKEND_DIR="${PROJECT_ROOT}/agents/backend"

# Resolve 1Password refs (e.g. OP_FRED_API_KEY_SECRET -> FRED_API_KEY) for Rust TUI
# Safe when op CLI or token is missing; see docs/platform/KEYS_FROM_1PASSWORD.md
# shellcheck source=scripts/include/onepassword.sh
if [[ -f "${SCRIPT_DIR}/include/onepassword.sh" ]]; then
  # shellcheck source=scripts/include/onepassword.sh
  source "${SCRIPT_DIR}/include/onepassword.sh"
  export_op_secrets_for_rust 2>/dev/null || true
fi

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

# Build the TUI service (always; cargo is incremental and only rebuilds changed crates)
build_tui() {
  (cd "${BACKEND_DIR}" && cargo build -q -p tui_service)
}

# Run the Rust TUI
run_tui() {
  if [[ -n "${RUST_TUI_VERBOSE:-}" ]]; then
    echo "Starting Rust TUI..."
    echo "  Shared config: ${IB_BOX_SPREAD_CONFIG:-auto-discovery}"
    echo "  NATS_URL:      ${NATS_URL:-<shared config / binary default>}"
    echo "  BACKEND_ID:    ${BACKEND_ID:-<shared config / binary default>}"
    echo "  WATCHLIST:     ${WATCHLIST:-<shared config / binary default>}"
    echo "  TICK_MS:       ${TICK_MS:-<shared config / binary default>}"
    echo "  SNAPSHOT_TTL_SECS: ${SNAPSHOT_TTL_SECS:-<shared config / binary default>}"
    echo ""
  fi

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
