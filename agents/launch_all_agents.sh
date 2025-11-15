#!/usr/bin/env bash
# Launches per-agent setup scripts in parallel; helpful for fresh dev machines.
# Requires: poetry (backend), go, npm, xcodebuild depending on targets.
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"

echo "[info] Launching agent setup scripts in parallel..."

if command -v poetry >/dev/null 2>&1; then
  (cd "$ROOT_DIR" && bash agents/backend/scripts/setup.sh) &
  (cd "$ROOT_DIR" && bash agents/backend-mock/scripts/setup.sh) &
  (cd "$ROOT_DIR" && bash agents/backend-data/scripts/setup.sh) &
else
  echo "[warn] Poetry missing; backend agents skipped." >&2
fi

# C++ TUI is built as part of main CMake build - no separate setup needed
(cd "$ROOT_DIR" && bash agents/tui/scripts/setup.sh) &

if command -v npm >/dev/null 2>&1; then
  (cd "$ROOT_DIR" && bash agents/web/scripts/setup.sh) &
else
  echo "[warn] npm missing; web setup skipped." >&2
fi

if command -v xcodebuild >/dev/null 2>&1; then
  (cd "$ROOT_DIR" && bash agents/ipad/scripts/setup.sh) &
  (cd "$ROOT_DIR" && bash agents/desktop/scripts/setup.sh) &
else
  echo "[warn] Xcode not found; iPad/desktop setups skipped." >&2
fi

wait || true
echo "[info] Agent setup scripts completed."
