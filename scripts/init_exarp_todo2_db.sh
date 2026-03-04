#!/usr/bin/env bash
# Initialize exarp-go SQLite DB for this project using exarp-go's built-in CLI.
# Runs "exarp-go task sync" with PROJECT_ROOT and EXARP_MIGRATIONS_DIR set so the
# CLI's initializeDatabase() creates .todo2/todo2.db, runs migrations, and sync
# populates it from .todo2/state.todo2.json. Run once so MCP task_workflow works.
#
# Uses same resolution as run_exarp_go.sh: global exarp-go, then EXARP_GO_ROOT/bin,
# then PROJECT_ROOT/../exarp-go/bin. Set EXARP_GO_ROOT if exarp-go repo is elsewhere.
# Usage: ./scripts/init_exarp_todo2_db.sh

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
export PROJECT_ROOT

echo "Project root: ${PROJECT_ROOT}"
EXARP_GO_VERBOSE=1 "${SCRIPT_DIR}/run_exarp_go.sh" task sync
echo ""
echo "Done. Restart Cursor (or reload MCP) so exarp-go uses the new DB."
