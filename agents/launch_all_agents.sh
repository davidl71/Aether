#!/usr/bin/env bash
# Launches per-agent setup scripts in parallel; helpful for fresh dev machines.
# Requires: poetry (backend), go, npm, xcodebuild depending on targets.
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"

echo "[info] Launching agent setup scripts in parallel..."

# Start NATS server if not running
if ! pgrep -f "nats-server" >/dev/null; then
  echo "[info] Starting NATS server..."
  (cd "$ROOT_DIR" && bash scripts/start_nats.sh) || echo "[warn] NATS server startup failed" >&2
fi

if command -v poetry >/dev/null 2>&1; then
  (cd "$ROOT_DIR" && bash agents/backend/scripts/setup.sh) &
  (cd "$ROOT_DIR" && bash agents/backend-mock/scripts/setup.sh) &
  (cd "$ROOT_DIR" && bash agents/backend-data/scripts/setup.sh) &
else
  echo "[warn] Poetry missing; backend agents skipped." >&2
fi

# No separate TUI agent setup: the active TUI is the Python/Textual app under python/tui.

if command -v npm >/dev/null 2>&1; then
  (cd "$ROOT_DIR" && bash agents/web/scripts/setup.sh) &
else
  echo "[warn] npm missing; web setup skipped." >&2
fi

wait || true
echo "[info] Agent setup scripts completed."
