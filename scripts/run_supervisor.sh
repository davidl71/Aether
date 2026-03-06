#!/usr/bin/env bash
# Run the Go supervisor (single process managing all services from config).
# Run from project root; uses config/services.supervisor.json by default.
# Usage: ./scripts/run_supervisor.sh
#   SUPERVISOR_CONFIG and SUPERVISOR_ROOT are set so services run with cwd = project root.
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"

export SUPERVISOR_CONFIG="${SUPERVISOR_CONFIG:-${ROOT_DIR}/config/services.supervisor.json}"
export SUPERVISOR_ROOT="${ROOT_DIR}"

if ! command -v go >/dev/null 2>&1 || [ ! -f "${ROOT_DIR}/agents/go/cmd/supervisor/main.go" ]; then
  echo "Go or agents/go/cmd/supervisor not found. Use scripts/service_manager.sh start-all instead." >&2
  exit 1
fi

cd "${ROOT_DIR}/agents/go" && exec go run ./cmd/supervisor
