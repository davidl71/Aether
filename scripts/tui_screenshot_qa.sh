#!/usr/bin/env bash
# Capture a TUI screenshot for QA/sanity.
# Writes to build/qa/tui/ by default (gitignored). Override with TUI_QA_SCREENSHOT_DIR.
#
# Usage:
#   ./scripts/tui_screenshot_qa.sh
#   TUI_QA_SCREENSHOT_DIR=qa/artifacts ./scripts/tui_screenshot_qa.sh

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

cd "${PROJECT_ROOT}"

if [ -d "${PROJECT_ROOT}/.venv" ]; then
    # shellcheck source=/dev/null
    source "${PROJECT_ROOT}/.venv/bin/activate"
fi

# Default output under build/ so it's gitignored; can override
export TUI_QA_SCREENSHOT_DIR="${TUI_QA_SCREENSHOT_DIR:-build/qa/tui}"

echo "Capturing TUI screenshot -> ${TUI_QA_SCREENSHOT_DIR}"
saved="$(uv run python -m python.tui.qa_screenshot "$@")"
if [ -n "${saved}" ] && [ -f "${saved}" ]; then
    echo "Screenshot saved: ${saved}"
else
    echo "Failed to capture screenshot." >&2
    exit 1
fi
