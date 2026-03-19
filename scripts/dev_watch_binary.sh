#!/usr/bin/env bash
# Watch build output and run a command when the binary (or build dir) changes.
# Use for quick dev iteration: rebuild triggers restart of the CLI or a service.
#
# Usage:
#   ./scripts/dev_watch_binary.sh
#     -> watches build/; on change prints a reminder to restart
#   ./scripts/dev_watch_binary.sh -- './scripts/service.sh restart ib'
#     -> on change runs that command
#   ./scripts/dev_watch_binary.sh --path build/macos-arm64-debug/bin/ib_box_spread -- 'just build && ./scripts/service.sh restart ib'
#
# Requires: fswatch (macOS: brew install fswatch) or inotifywait (Linux: inotify-tools)
# Fallback: polls every 2s if no watcher available.
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
WATCH_PATH="${ROOT_DIR}/build"
ON_CHANGE_CMD=""
POLL_INTERVAL=2

while [[ $# -gt 0 ]]; do
  case "$1" in
  --path=*)
    WATCH_PATH="${1#--path=}"
    shift
    ;;
  --path)
    WATCH_PATH="$2"
    shift 2
    ;;
  --poll)
    POLL_INTERVAL="${2:-2}"
    shift 2
    ;;
  --)
    shift
    ON_CHANGE_CMD=("$@")
    break
    ;;
  *)
    ON_CHANGE_CMD=("$@")
    break
    ;;
  esac
done

# Resolve to binary path if it exists, else keep as dir
if [[ -f "$WATCH_PATH" ]]; then
  WATCH_TARGET="$WATCH_PATH"
elif [[ -d "$WATCH_PATH" ]]; then
  WATCH_TARGET="$WATCH_PATH"
else
  # Try common binary locations
  for cand in \
    "${ROOT_DIR}/build/macos-arm64-debug/bin/ib_box_spread" \
    "${ROOT_DIR}/build/macos-x86_64-debug/bin/ib_box_spread" \
    "${ROOT_DIR}/build/bin/ib_box_spread" \
    "${ROOT_DIR}/build"; do
    if [[ -e "$cand" ]]; then
      WATCH_TARGET="$cand"
      break
    fi
  done
  if [[ -z "${WATCH_TARGET:-}" ]]; then
    WATCH_TARGET="${ROOT_DIR}/build"
  fi
fi

if [[ ${#ON_CHANGE_CMD[@]} -eq 0 ]]; then
  ON_CHANGE_CMD=(echo "Binary/build changed. Restart with: ./scripts/service.sh restart ib  (or run your dev command)")
fi

run_on_change() {
  echo "[$(date '+%H:%M:%S')] Change detected under ${WATCH_TARGET}"
  "${ON_CHANGE_CMD[@]}" || true
}

if command -v fswatch &>/dev/null; then
  echo "Watching ${WATCH_TARGET} (fswatch). Run command on change."
  fswatch -o -r "$WATCH_TARGET" 2>/dev/null | while read -r; do run_on_change; done
elif command -v inotifywait &>/dev/null; then
  echo "Watching ${WATCH_TARGET} (inotifywait). Run command on change."
  while inotifywait -r -e modify,create,delete,move "$WATCH_TARGET" 2>/dev/null; do run_on_change; done
else
  echo "No fswatch/inotifywait found. Polling every ${POLL_INTERVAL}s. Install fswatch (macOS) or inotify-tools (Linux) for event-based watch."
  _mtime() {
    if [[ -f "$1" ]]; then
      stat -f %m "$1" 2>/dev/null || stat -c %Y "$1" 2>/dev/null
    fi
  }
  LAST_MTIME=""
  while true; do
    CUR=""
    if [[ -f "$WATCH_TARGET" ]]; then
      CUR=$(_mtime "$WATCH_TARGET")
    else
      for f in "$WATCH_TARGET"/bin/ib_box_spread "$WATCH_TARGET"/*/bin/ib_box_spread; do
        [[ -f "$f" ]] && CUR=$(_mtime "$f") && break
      done
    fi
    [[ -z "$CUR" ]] && CUR=$(date +%s)
    if [[ -n "$LAST_MTIME" && "$CUR" != "$LAST_MTIME" ]]; then
      run_on_change
    fi
    LAST_MTIME="$CUR"
    sleep "$POLL_INTERVAL"
  done
fi
