#!/usr/bin/env bash
# Stop Tastytrade service
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT_DIR"

# Load shared utilities
SCRIPTS_DIR="${ROOT_DIR}/scripts"
if [ -f "${SCRIPTS_DIR}/include/config.sh" ]; then
  source "${SCRIPTS_DIR}/include/config.sh"
fi

# Get port from config (default: 8005)
TASTYTRADE_PORT=$(config_get_port "tastytrade" 8005)

# Find and kill process on port
if lsof -ti :${TASTYTRADE_PORT} >/dev/null 2>&1; then
  PID=$(lsof -ti :${TASTYTRADE_PORT})
  echo "[info] Stopping Tastytrade service on port ${TASTYTRADE_PORT} (PID: $PID)..."
  kill "$PID" 2>/dev/null || true
  sleep 1
  if kill -0 "$PID" 2>/dev/null; then
    echo "[warn] Process still running, force killing..."
    kill -9 "$PID" 2>/dev/null || true
  fi
  echo "[info] Tastytrade service stopped"
else
  echo "[info] Tastytrade service not running on port ${TASTYTRADE_PORT}"
fi
