#!/usr/bin/env bash
# Stop TypeScript dev server
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT_DIR"

# Find dev server process
PID=$(lsof -ti :5173 2>/dev/null || true)

if [[ -z "$PID" ]]; then
  echo "[info] Dev server is not running"
  exit 0
fi

echo "[info] Stopping dev server (PID: $PID)..."

# Try graceful shutdown first
kill "$PID" 2>/dev/null || true

# Wait for processes to stop
for i in {1..10}; do
  if ! lsof -ti :5173 >/dev/null 2>&1; then
    echo "[info] Dev server stopped successfully"
    exit 0
  fi
  sleep 0.5
done

# Force kill if still running
REMAINING_PIDS=$(lsof -ti :5173 2>/dev/null || true)
if [[ -n "$REMAINING_PIDS" ]]; then
  echo "[warn] Force killing dev server..."
  for PID in $REMAINING_PIDS; do
    kill -9 "$PID" 2>/dev/null || true
  done
  sleep 1

  if lsof -ti :5173 >/dev/null 2>&1; then
    echo "[error] Failed to stop dev server"
    exit 1
  else
    echo "[info] Dev server force stopped"
  fi
fi
