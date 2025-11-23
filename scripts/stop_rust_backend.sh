#!/usr/bin/env bash
# Stop Rust backend service
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"

# Load shared utilities
SCRIPTS_DIR="${ROOT_DIR}/scripts"
if [ -f "${SCRIPTS_DIR}/include/config.sh" ]; then
  source "${SCRIPTS_DIR}/include/config.sh"
fi

# Get port from config (default: 8080)
RUST_BACKEND_REST_PORT=$(config_get ".services.rust_backend.rest_port" 8080)

# Find and kill process
if lsof -ti :${RUST_BACKEND_REST_PORT} >/dev/null 2>&1; then
  PID=$(lsof -ti :${RUST_BACKEND_REST_PORT})
  echo "[info] Stopping Rust backend (PID: $PID)..."
  kill "$PID" 2>/dev/null || true
  sleep 2

  # Force kill if still running
  if kill -0 "$PID" 2>/dev/null; then
    echo "[warn] Process still running, force killing..."
    kill -9 "$PID" 2>/dev/null || true
  fi

  echo "[info] Rust backend stopped"
else
  echo "[info] Rust backend not running on port ${RUST_BACKEND_REST_PORT}"
fi

# Also kill any cargo processes running backend_service
pkill -f "cargo.*backend_service" 2>/dev/null || true
