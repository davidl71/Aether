#!/usr/bin/env bash
# Start Rust backend service in background (daemonized)
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT_DIR"

# Load shared utilities
SCRIPTS_DIR="${ROOT_DIR}/scripts"
if [ -f "${SCRIPTS_DIR}/include/config.sh" ]; then
  source "${SCRIPTS_DIR}/include/config.sh"
fi

# Get ports from config (defaults: 8080 for REST, 50051 for gRPC)
RUST_BACKEND_REST_PORT=$(config_get ".services.rust_backend.rest_port" 8080)
RUST_BACKEND_GRPC_PORT=$(config_get ".services.rust_backend.grpc_port" 50051)

# Check if already running
if lsof -ti :${RUST_BACKEND_REST_PORT} >/dev/null 2>&1; then
  PID=$(lsof -ti :${RUST_BACKEND_REST_PORT})
  echo "[info] Rust backend already running on port ${RUST_BACKEND_REST_PORT} (PID: $PID)"
  echo "[info] REST API: http://localhost:${RUST_BACKEND_REST_PORT}/api/v1/snapshot"
  echo "[info] gRPC API: localhost:${RUST_BACKEND_GRPC_PORT}"
  exit 0
fi

# Check if Rust is available
if ! command -v cargo >/dev/null 2>&1; then
  echo "[error] Rust/Cargo not found. Install Rust toolchain first." >&2
  exit 1
fi

# Check if backend service exists
BACKEND_DIR="${ROOT_DIR}/agents/backend"
if [ ! -f "${BACKEND_DIR}/Cargo.toml" ]; then
  echo "[error] Backend service not found at ${BACKEND_DIR}" >&2
  exit 1
fi

# Create logs directory
mkdir -p logs
LOG_FILE="${ROOT_DIR}/logs/rust-backend.log"

echo "[info] Starting Rust backend service..."
echo "[info] REST API: http://localhost:${RUST_BACKEND_REST_PORT}"
echo "[info] gRPC API: localhost:${RUST_BACKEND_GRPC_PORT}"

# Start in background with ENABLE_SERVICE_CONTROL=true
cd "${BACKEND_DIR}"
(
  export ENABLE_SERVICE_CONTROL=true
  export RUST_LOG="${RUST_LOG:-info}"
  cargo run -p backend_service > "$LOG_FILE" 2>&1
) &
SERVICE_PID=$!

# Disown the process
disown $SERVICE_PID 2>/dev/null || true

# Wait for service to start
sleep 5

# Check if service started successfully
if kill -0 "$SERVICE_PID" 2>/dev/null && curl -s http://localhost:${RUST_BACKEND_REST_PORT}/health >/dev/null 2>&1; then
  echo "[info] Rust backend started (PID: $SERVICE_PID)"
  echo "[info] REST API: http://localhost:${RUST_BACKEND_REST_PORT}/api/v1/snapshot"
  echo "[info] gRPC API: localhost:${RUST_BACKEND_GRPC_PORT}"
  echo "[info] Service control: ENABLE_SERVICE_CONTROL=true"
  echo "[info] Log file: $LOG_FILE"
  echo "[info] To stop: ./scripts/stop_rust_backend.sh"
  echo "[info] To view logs: tail -f $LOG_FILE"
else
  echo "[error] Failed to start Rust backend"
  echo "[error] Check log: $LOG_FILE"
  tail -20 "$LOG_FILE" 2>/dev/null || true
  exit 1
fi
