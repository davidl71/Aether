#!/usr/bin/env bash
# Stop NATS server
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT_DIR"

# Find NATS server process
PID=$(pgrep -f "nats-server" || true)

if [[ -z "$PID" ]]; then
  echo "[info] NATS server is not running"
  exit 0
fi

echo "[info] Stopping NATS server (PID: $PID)..."

# Try graceful shutdown first
kill "$PID" 2>/dev/null || true

# Wait for process to stop
for i in {1..10}; do
  if ! pgrep -f "nats-server" >/dev/null; then
    echo "[info] NATS server stopped successfully"
    exit 0
  fi
  sleep 0.5
done

# Force kill if still running
if pgrep -f "nats-server" >/dev/null; then
  echo "[warn] Force killing NATS server..."
  kill -9 "$PID" 2>/dev/null || true
  sleep 1

  if pgrep -f "nats-server" >/dev/null; then
    echo "[error] Failed to stop NATS server"
    exit 1
  else
    echo "[info] NATS server force stopped"
  fi
fi
