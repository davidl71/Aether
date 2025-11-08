#!/usr/bin/env bash
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

if command -v go >/dev/null 2>&1; then
  (cd "$ROOT_DIR" && bash agents/tui/scripts/setup.sh) &
else
  echo "[warn] Go missing; TUI setup skipped." >&2
fi

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

