#!/usr/bin/env bash
# Draft a plan from the clipboard and create a macOS Note.
# Usage: Run from macOS Shortcuts without arguments.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"

SPEC="$(pbpaste || true)"
if [[ -z "${SPEC// /}" ]]; then
  echo "Clipboard is empty." >&2
  exit 1
fi

PLAN="(MLX tool not available - python/tools deleted)"
if [[ -z "${PLAN}" ]]; then
  PLAN="(No plan generated)"
fi

TITLE="Plan: $(date +%Y-%m-%d_%H-%M)"
BODY="${PLAN//$'\n'/'\n'}"

# Target folder and tags
FOLDER_NAME="IB Automation"
AUTO_TAGS="[AUTO] #ib-automation #auto-generated"
NOTE_BODY="${AUTO_TAGS}\n\n${BODY}"

osascript <<OSA
tell application "Notes"
  activate
  set targetAccount to first account
  set targetFolder to missing value
  repeat with f in folders of targetAccount
    if name of f is "${FOLDER_NAME}" then
      set targetFolder to f
      exit repeat
    end if
  end repeat
  if targetFolder is missing value then
    set targetFolder to make new folder at targetAccount with properties {name:"${FOLDER_NAME}"}
  end if
  set newNote to make new note at targetFolder with properties {name:"${TITLE}", body:"${NOTE_BODY}"}
  return id of newNote
end tell
OSA

echo "Created note '${TITLE}' in Notes (folder: ${FOLDER_NAME})."
