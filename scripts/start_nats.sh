#!/usr/bin/env bash
# Start NATS server for development
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT_DIR"

# Check if NATS is installed
if ! command -v nats-server >/dev/null 2>&1; then
  echo "[error] NATS server not found. Run: ./scripts/install_nats.sh"
  exit 1
fi

# Check if already running
if pgrep -f "nats-server" >/dev/null; then
  echo "[info] NATS server is already running (PID: $(pgrep -f 'nats-server'))"
  echo "[info] Health check: http://localhost:8222/healthz"
  exit 0
fi

# Create logs directory if needed
mkdir -p logs

# Configuration file
CONFIG_FILE="${ROOT_DIR}/config/nats-server.conf"

if [[ ! -f "$CONFIG_FILE" ]]; then
  echo "[warn] Configuration file not found: $CONFIG_FILE"
  echo "[info] Starting NATS server with default configuration..."
  nats-server &
else
  echo "[info] Starting NATS server with config: $CONFIG_FILE"
  nats-server -c "$CONFIG_FILE" &
fi

# Wait a moment for server to start
sleep 1

# Check if server started successfully
if pgrep -f "nats-server" >/dev/null; then
  PID=$(pgrep -f "nats-server")
  echo "[info] NATS server started (PID: $PID)"
  echo "[info] Server URL: nats://localhost:4222"
  echo "[info] Monitoring: http://localhost:8222"
  echo "[info] Health check: http://localhost:8222/healthz"
  echo "[info] To stop: ./scripts/stop_nats.sh"
else
  echo "[error] Failed to start NATS server"
  exit 1
fi
