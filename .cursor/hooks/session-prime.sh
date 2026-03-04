#!/usr/bin/env bash
# Cursor sessionStart hook: run exarp-go session prime for THIS project and inject as additional_context.
# When a new Composer conversation starts, Cursor runs this script (cwd = project root), passes JSON on stdin,
# and expects stdout: JSON with "additional_context" (string) and "continue" (bool).
#
# Uses this repo's scripts/run_exarp_go.sh so exarp-go sees PROJECT_ROOT = this repo (tasks, .todo2, etc.).

set -e
# Consume stdin (Cursor sends sessionStart payload)
INPUT=$(cat 2>/dev/null || true)

PROJECT_ROOT="${PROJECT_ROOT:-$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)}"
export PROJECT_ROOT
RUNNER="${PROJECT_ROOT}/scripts/run_exarp_go.sh"

PRIME_JSON=""
if [[ -x "$RUNNER" ]]; then
  RAW=$("$RUNNER" -tool session -args '{"action":"prime","include_tasks":true,"include_hints":true,"compact":true,"include_cli_command":false}' -json -quiet 2>/dev/null) || true
  if [[ -n "$RAW" ]]; then
    if command -v jq >/dev/null 2>&1; then
      PRIME_JSON=$(printf '%s' "$RAW" | jq -r '.[0].text // empty' 2>/dev/null) || true
    fi
  fi
fi

ADDITIONAL=""
if [[ -n "$PRIME_JSON" ]] && command -v jq >/dev/null 2>&1; then
  STATUS_CTX=$(printf '%s' "$PRIME_JSON" | jq -r '.status_context // ""')
  STATUS_LABEL=$(printf '%s' "$PRIME_JSON" | jq -r '.status_label // ""')
  SUGGESTED=$(printf '%s' "$PRIME_JSON" | jq -r '.suggested_next[0].content // ""')
  CLI_SUG=$(printf '%s' "$PRIME_JSON" | jq -r '.cursor_cli_suggestion // ""')
  HAS_HANDOFF=$(printf '%s' "$PRIME_JSON" | jq -r 'if .handoff_alert != null then "1" else "" end')
  LINES=()
  [[ -n "$STATUS_LABEL" ]] && LINES+=("Session: $STATUS_LABEL")
  [[ -n "$STATUS_CTX" ]] && LINES+=("Context: $STATUS_CTX")
  [[ -n "$HAS_HANDOFF" ]] && LINES+=("Review handoff from previous developer before starting.")
  [[ -n "$SUGGESTED" ]] && LINES+=("Suggested next: $SUGGESTED")
  [[ -n "$CLI_SUG" ]] && LINES+=("CLI: $CLI_SUG")
  ADDITIONAL=$(IFS=$'\n'; echo "${LINES[*]}")
fi

if [[ -z "$ADDITIONAL" ]]; then
  ADDITIONAL="exarp-go session prime unavailable (install exarp-go, set PATH or EXARP_GO_ROOT, and ensure jq in PATH for rich context)."
fi

if command -v jq >/dev/null 2>&1; then
  jq -n --arg ctx "$ADDITIONAL" '{ "additional_context": $ctx, "continue": true }'
else
  ESCAPED=$(printf '%s' "$ADDITIONAL" | sed 's/\\/\\\\/g; s/"/\\"/g; s/\n/\\n/g')
  printf '{"additional_context":"%s","continue":true}\n' "$ESCAPED"
fi
