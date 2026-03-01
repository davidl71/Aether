#!/usr/bin/env bash
# Initialize exarp-go SQLite DB for this project using exarp-go's built-in CLI.
# Runs "exarp-go task sync" with PROJECT_ROOT and EXARP_MIGRATIONS_DIR set so the
# CLI's initializeDatabase() creates .todo2/todo2.db, runs migrations, and sync
# populates it from .todo2/state.todo2.json. Run once so MCP task_workflow works.
#
# Prereqs: exarp-go on PATH (or in EXARP_GO_ROOT/bin). No Go required.
# Usage: ./scripts/init_exarp_todo2_db.sh

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
EXARP_GO_ROOT="${EXARP_GO_ROOT:-$(cd "${PROJECT_ROOT}/../exarp-go" 2>/dev/null && pwd)}"

if [[ -z "${EXARP_GO_ROOT:-}" ]]; then
  echo "Warning: exarp-go repo not found at ../exarp-go. Set EXARP_GO_ROOT if it's elsewhere." >&2
fi

# Prefer exarp-go on PATH; otherwise use repo's bin if built
EXARP_GO_BIN=""
if command -v exarp-go &>/dev/null; then
  EXARP_GO_BIN="exarp-go"
elif [[ -n "${EXARP_GO_ROOT:-}" ]] && [[ -x "${EXARP_GO_ROOT}/bin/exarp-go" ]]; then
  EXARP_GO_BIN="${EXARP_GO_ROOT}/bin/exarp-go"
else
  echo "Error: exarp-go not found. Install it (e.g. 'cd exarp-go && go build -o bin/exarp-go .') or set PATH." >&2
  exit 1
fi

echo "Project root: ${PROJECT_ROOT}"
echo "Using: ${EXARP_GO_BIN}"
echo ""

export PROJECT_ROOT
if [[ -n "${EXARP_GO_ROOT:-}" ]]; then
  export EXARP_MIGRATIONS_DIR="${EXARP_GO_ROOT}/migrations"
fi

# Built-in init: CLI calls initializeDatabase() then task sync populates DB from JSON
"${EXARP_GO_BIN}" task sync

echo ""
echo "Done. Restart Cursor (or reload MCP) so exarp-go uses the new DB."
echo "MCP env already has EXARP_MIGRATIONS_DIR set in .cursor/mcp.json for future runs."
