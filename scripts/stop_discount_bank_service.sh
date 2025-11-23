#!/usr/bin/env bash
# Stop Discount Bank service
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT_DIR"

# Load shared utilities
SCRIPTS_DIR="${ROOT_DIR}/scripts"
if [ -f "${SCRIPTS_DIR}/include/config.sh" ]; then
  source "${SCRIPTS_DIR}/include/config.sh"
fi

# Get port from config (default: 8003)
DISCOUNT_BANK_PORT=$(config_get_port "discount_bank" 8003)

# Find and kill process on port
if lsof -ti :${DISCOUNT_BANK_PORT} >/dev/null 2>&1; then
  PID=$(lsof -ti :${DISCOUNT_BANK_PORT})
  echo "[info] Stopping Discount Bank service on port ${DISCOUNT_BANK_PORT} (PID: $PID)..."
  kill "$PID" 2>/dev/null || true
  sleep 1
  if kill -0 "$PID" 2>/dev/null; then
    echo "[warn] Process still running, force killing..."
    kill -9 "$PID" 2>/dev/null || true
  fi
  echo "[info] Discount Bank service stopped"
else
  echo "[info] Discount Bank service not running on port ${DISCOUNT_BANK_PORT}"
fi
