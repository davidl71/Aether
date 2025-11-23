#!/usr/bin/env bash
# Stop Risk-Free Rate service
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT_DIR"

# Load shared utilities
SCRIPTS_DIR="${ROOT_DIR}/scripts"
if [ -f "${SCRIPTS_DIR}/include/config.sh" ]; then
  source "${SCRIPTS_DIR}/include/config.sh"
fi

# Get port from config (default: 8004)
RISK_FREE_RATE_PORT=$(config_get_port "risk_free_rate" 8004)

# Find and kill process on port
if lsof -ti :${RISK_FREE_RATE_PORT} >/dev/null 2>&1; then
  PID=$(lsof -ti :${RISK_FREE_RATE_PORT})
  echo "[info] Stopping Risk-Free Rate service on port ${RISK_FREE_RATE_PORT} (PID: $PID)..."
  kill "$PID" 2>/dev/null || true
  sleep 1
  if kill -0 "$PID" 2>/dev/null; then
    echo "[warn] Process still running, force killing..."
    kill -9 "$PID" 2>/dev/null || true
  fi
  echo "[info] Risk-Free Rate service stopped"
else
  echo "[info] Risk-Free Rate service not running on port ${RISK_FREE_RATE_PORT}"
fi
