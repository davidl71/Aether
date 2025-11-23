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

# Setup Python 3.12 environment for PyO3 compatibility
PYTHON312="/usr/local/bin/python3.12"
VENV_DIR="${BACKEND_DIR}/.venv"

if [ ! -f "${PYTHON312}" ]; then
  echo "[error] Python 3.12 not found at ${PYTHON312}" >&2
  echo "[error] Install with: brew install python@3.12" >&2
  exit 1
fi

# Create venv if it doesn't exist
if [ ! -d "${VENV_DIR}" ]; then
  echo "[info] Creating Python 3.12 virtual environment..."
  "${PYTHON312}" -m venv "${VENV_DIR}"
fi

# Set PyO3 environment variables
export PYO3_PYTHON="${VENV_DIR}/bin/python"
export PYO3_PYTHON_VERSION="3.12"
echo "[info] Using Python 3.12: ${PYO3_PYTHON}"

# Create logs directory
mkdir -p logs
LOG_FILE="${ROOT_DIR}/logs/rust-backend.log"

echo "[info] Starting Rust backend service..."
echo "[info] REST API: http://localhost:${RUST_BACKEND_REST_PORT}"
echo "[info] gRPC API: localhost:${RUST_BACKEND_GRPC_PORT}"
echo "[info] Python: ${PYO3_PYTHON:-$(which python3)}"

# Start in background with ENABLE_SERVICE_CONTROL=true
cd "${BACKEND_DIR}"
(
  export ENABLE_SERVICE_CONTROL=true
  export RUST_LOG="${RUST_LOG:-info}"
  # Preserve Python environment variables for PyO3
  export PYO3_PYTHON="${PYO3_PYTHON}"
  export PYO3_PYTHON_VERSION="${PYO3_PYTHON_VERSION}"
  # Add venv Python to PATH for cargo build scripts
  export PATH="${VENV_DIR}/bin:${PATH}"
  cargo run -p backend_service > "$LOG_FILE" 2>&1
) &
SERVICE_PID=$!

# Disown the process
disown $SERVICE_PID 2>/dev/null || true

# Wait for service to start (longer for first-time compilation)
echo "[info] Waiting for service to start (this may take longer on first build)..."
MAX_WAIT=60
WAIT_COUNT=0
HEALTHY=false

while [ $WAIT_COUNT -lt $MAX_WAIT ]; do
  if kill -0 "$SERVICE_PID" 2>/dev/null; then
    if curl -s http://localhost:${RUST_BACKEND_REST_PORT}/health >/dev/null 2>&1; then
      HEALTHY=true
      break
    fi
  else
    # Process died, check logs
    echo "[error] Backend process died during startup" >&2
    tail -30 "$LOG_FILE" 2>/dev/null || true
    exit 1
  fi
  sleep 2
  WAIT_COUNT=$((WAIT_COUNT + 2))
  echo "[info] Still waiting... (${WAIT_COUNT}s/${MAX_WAIT}s)"
done

# Check if service started successfully
if [ "$HEALTHY" = true ]; then
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
