#!/usr/bin/env bash
# Summarize a file and create a macOS Notes entry with the result.
# Usage (from macOS Shortcuts “Run Shell Script”):
#   ./scripts/shortcuts/summarize_file_to_notes.sh "<absolute-path-to-file>"
set -euo pipefail

FILE_PATH="${1:-}"
if [[ -z "${FILE_PATH}" || ! -f "${FILE_PATH}" ]]; then
  echo "Usage: $0 <file-path>" >&2
  exit 1
fi

SUMMARY="(MLX tool not available - python/tools deleted)"
if [[ -z "${SUMMARY}" ]]; then
  SUMMARY="(No summary generated)"
fi

TITLE="Summary: $(basename "${FILE_PATH}")"
BODY="${SUMMARY//$'\n'/'\n'}"

# Target folder and tags
FOLDER_NAME="IB Automation"
AUTO_TAGS="[AUTO] #ib-automation #auto-generated"
NOTE_BODY="${AUTO_TAGS}\n\n${BODY}"

# Create a new note under FOLDER_NAME (ensure folder exists) via AppleScript
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
