#!/usr/bin/env bash
# Wrapper to run exarp-go MCP server with PROJECT_ROOT set to this repo and CWD = project root.
# Usage: .cursor/mcp.json uses this script as the exarp-go command.
# Ensures exarp-go sees the correct project (e.g. .todo2 and task store).

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# Prefer env from mcp.json; otherwise use repo root relative to this script
if [[ -z "${PROJECT_ROOT:-}" ]]; then
  PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
fi
export PROJECT_ROOT

# Run from project root so exarp-go detects project and finds .todo2 / task store
cd "${PROJECT_ROOT}"

if command -v exarp-go &>/dev/null; then
  exec exarp-go "$@"
fi

# Fallback: common install paths (edit if your exarp-go is elsewhere)
for candidate in \
  "${HOME}/go/bin/exarp-go" \
  "${HOME}/Projects/exarp-go/bin/exarp-go" \
  "/usr/local/bin/exarp-go"; do
  if [[ -x "${candidate}" ]]; then
    exec "${candidate}" "$@"
  fi
done

echo "exarp-go not found. Install it or set PATH." >&2
exit 1
