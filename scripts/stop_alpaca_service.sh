#!/usr/bin/env bash
# Stop Alpaca service
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT_DIR"

# Load shared utilities
SCRIPTS_DIR="${ROOT_DIR}/scripts"
if [ -f "${SCRIPTS_DIR}/include/config.sh" ]; then
  source "${SCRIPTS_DIR}/include/config.sh"
fi

# Get port from config (default: 8000)
ALPACA_PORT=$(config_get_port "alpaca" 8000)

# Find and kill process on port
if lsof -ti :${ALPACA_PORT} >/dev/null 2>&1; then
  PID=$(lsof -ti :${ALPACA_PORT})
  echo "[info] Stopping Alpaca service on port ${ALPACA_PORT} (PID: $PID)..."
  kill "$PID" 2>/dev/null || true
  sleep 1
  if kill -0 "$PID" 2>/dev/null; then
    echo "[warn] Process still running, force killing..."
    kill -9 "$PID" 2>/dev/null || true
  fi
  echo "[info] Alpaca service stopped"
else
  echo "[info] Alpaca service not running on port ${ALPACA_PORT}"
fi
