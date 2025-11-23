#!/usr/bin/env bash
# Start Risk-Free Rate service in background (daemonized)
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

# Check if already running
if lsof -ti :${RISK_FREE_RATE_PORT} >/dev/null 2>&1; then
  PID=$(lsof -ti :${RISK_FREE_RATE_PORT})
  echo "[info] Risk-Free Rate service already running on port ${RISK_FREE_RATE_PORT} (PID: $PID)"
  echo "[info] URL: http://localhost:${RISK_FREE_RATE_PORT}/api/health"
  exit 0
fi

# Create logs directory
mkdir -p logs
LOG_FILE="${ROOT_DIR}/logs/risk-free-rate-service.log"

echo "[info] Starting Risk-Free Rate service on port ${RISK_FREE_RATE_PORT}..."

# Start in background and redirect output to log
./web/scripts/run-risk-free-rate-service.sh > "$LOG_FILE" 2>&1 &
SERVICE_PID=$!

# Disown the process
disown $SERVICE_PID 2>/dev/null || true

# Wait for service to start
sleep 4

# Check if service started successfully
if kill -0 "$SERVICE_PID" 2>/dev/null && curl -s http://localhost:${RISK_FREE_RATE_PORT}/api/health >/dev/null 2>&1; then
  echo "[info] Risk-Free Rate service started (PID: $SERVICE_PID)"
  echo "[info] URL: http://localhost:${RISK_FREE_RATE_PORT}/api/health"
  echo "[info] Log file: $LOG_FILE"
  echo "[info] To stop: ./scripts/stop_risk_free_rate_service.sh"
  echo "[info] To view logs: tail -f $LOG_FILE"
else
  echo "[error] Failed to start Risk-Free Rate service"
  echo "[error] Check log: $LOG_FILE"
  tail -20 "$LOG_FILE" 2>/dev/null || true
  exit 1
fi
