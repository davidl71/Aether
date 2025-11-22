#!/usr/bin/env bash
# Start TypeScript dev server in background (daemonized)
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT_DIR"

# Check if already running
if lsof -ti :5173 >/dev/null 2>&1; then
  PID=$(lsof -ti :5173)
  echo "[info] Dev server already running on port 5173 (PID: $PID)"
  echo "[info] URL: http://localhost:5173"
  exit 0
fi

# Create logs directory
mkdir -p logs
LOG_FILE="${ROOT_DIR}/logs/web-dev-server.log"

echo "[info] Starting TypeScript dev server..."
cd web

# Start in background and redirect output to log
npm run dev > "$LOG_FILE" 2>&1 &
DEV_PID=$!

# Disown the process
disown $DEV_PID 2>/dev/null || true

# Wait for server to start
sleep 3

# Check if server started successfully
if kill -0 "$DEV_PID" 2>/dev/null && curl -s http://localhost:5173 >/dev/null 2>&1; then
  echo "[info] Dev server started (PID: $DEV_PID)"
  echo "[info] URL: http://localhost:5173"
  echo "[info] Log file: $LOG_FILE"
  echo "[info] To stop: ./scripts/stop_web_dev.sh"
  echo "[info] To view logs: tail -f $LOG_FILE"
else
  echo "[error] Failed to start dev server"
  echo "[error] Check log: $LOG_FILE"
  tail -20 "$LOG_FILE" 2>/dev/null || true
  exit 1
fi
