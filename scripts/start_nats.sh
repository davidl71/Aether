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
LOG_FILE="${ROOT_DIR}/logs/nats-server.log"

if [[ ! -f "$CONFIG_FILE" ]]; then
  echo "[warn] Configuration file not found: $CONFIG_FILE"
  echo "[info] Starting NATS server with default configuration..."
  nats-server > "$LOG_FILE" 2>&1 &
  NATS_PID=$!
else
  echo "[info] Starting NATS server with config: $CONFIG_FILE"
  nats-server -c "$CONFIG_FILE" > "$LOG_FILE" 2>&1 &
  NATS_PID=$!
fi

# Disown the process so it runs in background
disown $NATS_PID 2>/dev/null || true

# Wait a moment for server to start
sleep 2

# Check if server started successfully
if kill -0 "$NATS_PID" 2>/dev/null && pgrep -f "nats-server" >/dev/null; then
  echo "[info] NATS server started in background (PID: $NATS_PID)"
  echo "[info] Server URL: nats://localhost:4222"
  echo "[info] WebSocket URL: ws://localhost:8080"
  echo "[info] Monitoring: http://localhost:8222"
  echo "[info] Health check: http://localhost:8222/healthz"
  echo "[info] Log file: $LOG_FILE"
  echo "[info] To stop: ./scripts/stop_nats.sh"
  echo "[info] To view logs: tail -f $LOG_FILE"
else
  echo "[error] Failed to start NATS server"
  echo "[error] Check logs: $LOG_FILE"
  tail -20 "$LOG_FILE" 2>/dev/null || true
  exit 1
fi
