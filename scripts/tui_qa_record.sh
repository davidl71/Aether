#!/usr/bin/env bash
# TUI QA recording and replay using asciinema or ttyrec.
#
# Prerequisites: NATS and backend_service running; TUI built (e.g. just run-tui once).
# See docs/runbooks/TUI_QA_RECORDING.md and docs/CLI_TUI_TOOLS_RECOMMENDATIONS.md.
#
# Usage:
#   ./scripts/tui_qa_record.sh record [output.cast]   # record with asciinema (default) or ttyrec
#   ./scripts/tui_qa_record.sh play <file.cast|file.ttyrec>
#
# Environment:
#   TUI_QA_RECORDER   asciinema (default) or ttyrec
#   TUI_QA_OUTPUT_DIR directory for default output path (default: docs/tui_qa)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
OUTPUT_DIR="${TUI_QA_OUTPUT_DIR:-${PROJECT_ROOT}/docs/tui_qa}"
RECORDER="${TUI_QA_RECORDER:-asciinema}"

# Default output path for record (asciinema: .cast, ttyrec: .ttyrec)
default_output() {
  local ext=".cast"
  [[ "$RECORDER" == "ttyrec" ]] && ext=".ttyrec"
  mkdir -p "$OUTPUT_DIR"
  echo "${OUTPUT_DIR}/tui_session_$(date +%Y%m%d_%H%M%S)${ext}"
}

cmd_record() {
  local out="${1:-$(default_output)}"
  mkdir -p "$(dirname "$out")"
  local tui_cmd="${PROJECT_ROOT}/scripts/run_rust_tui.sh"
  if [[ ! -x "$tui_cmd" ]]; then
    echo "Error: TUI script not found or not executable: $tui_cmd" >&2
    exit 1
  fi
  if [[ "$RECORDER" == "ttyrec" ]]; then
    if ! command -v ttyrec &>/dev/null; then
      echo "Error: ttyrec not found. Install with: brew install ttyrec" >&2
      exit 1
    fi
    echo "Recording with ttyrec to $out (quit TUI with 'q' to stop)..." >&2
    ttyrec "$out" "$tui_cmd"
  else
    if ! command -v asciinema &>/dev/null; then
      echo "Error: asciinema not found. Install with: brew install asciinema" >&2
      exit 1
    fi
    echo "Recording with asciinema to $out (quit TUI with 'q' to stop)..." >&2
    asciinema rec "$out" -c "$tui_cmd"
  fi
}

cmd_play() {
  local file="${1:?Usage: $0 play <file.cast|file.ttyrec>}"
  if [[ ! -f "$file" ]]; then
    echo "Error: file not found: $file" >&2
    exit 1
  fi
  case "$file" in
  *.cast)
    if ! command -v asciinema &>/dev/null; then
      echo "Error: asciinema not found. Install with: brew install asciinema" >&2
      exit 1
    fi
    asciinema play "$file"
    ;;
  *.ttyrec)
    if ! command -v ttyplay &>/dev/null; then
      echo "Error: ttyplay not found (part of ttyrec). Install with: brew install ttyrec" >&2
      exit 1
    fi
    ttyplay "$file"
    ;;
  *)
    echo "Error: unknown format (use .cast or .ttyrec): $file" >&2
    exit 1
    ;;
  esac
}

case "${1:-}" in
record)
  cmd_record "${2:-}"
  ;;
play)
  cmd_play "${2:-}"
  ;;
*)
  echo "Usage: $0 record [output.cast|output.ttyrec]"
  echo "       $0 play <file.cast|file.ttyrec>"
  echo ""
  echo "Env: TUI_QA_RECORDER=asciinema|ttyrec  TUI_QA_OUTPUT_DIR=dir"
  exit 1
  ;;
esac
