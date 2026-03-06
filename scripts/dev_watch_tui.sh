#!/usr/bin/env bash
# Watch TUI source and config; restart the Python TUI on change.
# Use for quick dev iteration: edit .py or config and the TUI restarts.
#
# Usage:
#   ./scripts/dev_watch_tui.sh
#     -> watches python/tui, python/integration, config; restarts TUI on change
#   ./scripts/dev_watch_tui.sh --dev
#     -> same but runs TUI with textual --dev (CSS live reload)
#   ./scripts/dev_watch_tui.sh -- mock
#     -> pass extra args to run_python_tui.sh (e.g. provider_type endpoint)
#
# Watchers (first available): watchfiles (Python, uv/pip), fswatch (macOS), inotifywait (Linux).
# Fallback: polls every 2s if none available.
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
SCRIPTS_DIR="${ROOT_DIR}/scripts"
TUI_SCRIPT="${SCRIPTS_DIR}/run_python_tui.sh"
WATCH_PATHS=(
  "${ROOT_DIR}/python/tui"
  "${ROOT_DIR}/python/integration"
  "${ROOT_DIR}/config/config.json"
  "${ROOT_DIR}/config/config.example.json"
)
# Legacy TUI config path
if [[ -n "${HOME:-}" ]]; then
  TUI_CONFIG="${HOME}/.config/ib_box_spread/tui_config.json"
  [[ -f "$TUI_CONFIG" ]] && WATCH_PATHS+=("$TUI_CONFIG")
fi

USE_DEV=false
EXTRA_ARGS=()
while [[ $# -gt 0 ]]; do
  case "$1" in
    --dev) USE_DEV=true; shift ;;
    --) shift; EXTRA_ARGS=("$@"); break ;;
    *) EXTRA_ARGS=("$@"); break ;;
  esac
done

RUN_CMD=("$TUI_SCRIPT")
[[ "$USE_DEV" == "true" ]] && RUN_CMD=("$TUI_SCRIPT" --dev)
RUN_CMD+=("${EXTRA_ARGS[@]}")

PID=""
start_tui() {
  stop_tui 2>/dev/null || true
  "${RUN_CMD[@]}" &
  PID=$!
  echo "[$(date '+%H:%M:%S')] TUI started (PID: $PID)"
}

stop_tui() {
  if [[ -n "$PID" ]] && kill -0 "$PID" 2>/dev/null; then
    kill "$PID" 2>/dev/null || true
    wait "$PID" 2>/dev/null || true
  fi
  PID=""
}

on_change() {
  echo "[$(date '+%H:%M:%S')] Change detected; restarting TUI..."
  start_tui
}

cleanup() {
  stop_tui
  exit 0
}
trap cleanup SIGINT SIGTERM

# Build fswatch/inotifywait args
WATCH_ARGS=()
for p in "${WATCH_PATHS[@]}"; do
  [[ -e "$p" ]] && WATCH_ARGS+=("$p")
done
if [[ ${#WATCH_ARGS[@]} -eq 0 ]]; then
  echo "No paths to watch (python/tui or config missing)." >&2
  exit 1
fi

start_tui

# Prefer Python watchfiles (no extra install on most setups; uv sync has it)
if "${PYTHON_CMD:-python3}" -c "import watchfiles" 2>/dev/null; then
  echo "Watching (watchfiles). Restart on change."
  export PYTHONPATH="${ROOT_DIR}/python${PYTHONPATH:+:${PYTHONPATH}}"
  exec "${PYTHON_CMD:-python3}" "${ROOT_DIR}/scripts/dev_watch_tui_runner.py" "${RUN_CMD[@]}"
fi

if command -v fswatch &>/dev/null; then
  echo "Watching ${WATCH_ARGS[*]} (fswatch). Restart on change."
  fswatch -o -r "${WATCH_ARGS[@]}" 2>/dev/null | while read -r; do on_change; done
elif command -v inotifywait &>/dev/null; then
  echo "Watching (inotifywait). Restart on change."
  while inotifywait -r -e modify,create,delete,move "${WATCH_ARGS[@]}" 2>/dev/null; do on_change; done
else
  echo "No watchfiles/fswatch/inotifywait. Polling every 2s. Install: uv sync (watchfiles) or brew install fswatch"
  _mtime() { stat -f %m "$1" 2>/dev/null || stat -c %Y "$1" 2>/dev/null; }
  LAST_MTIME=""
  while true; do
    CUR=""
    while IFS= read -r -d '' f; do
      t=$(_mtime "$f" 2>/dev/null)
      [[ -n "$t" && ( -z "$CUR" || "$t" -gt "$CUR" ) ]] && CUR=$t
    done < <(find "${ROOT_DIR}/python/tui" "${ROOT_DIR}/python/integration" -type f \( -name "*.py" -o -name "*.json" \) -print0 2>/dev/null)
    [[ -z "$CUR" ]] && CUR=$(date +%s)
    if [[ -n "$LAST_MTIME" && "$CUR" != "$LAST_MTIME" ]]; then
      on_change
    fi
    LAST_MTIME="$CUR"
    sleep 2
  done
fi
