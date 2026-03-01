#!/usr/bin/env bash
# Wrapper to run exarp-go MCP server with PROJECT_ROOT set to this repo.
# Usage: .cursor/mcp.json uses this script as the exarp-go command.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

export PROJECT_ROOT

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
