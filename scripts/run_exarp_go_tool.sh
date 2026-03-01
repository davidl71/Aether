#!/usr/bin/env bash
# Run a single exarp-go tool from scripts/CI/make. PROJECT_ROOT is set by run_exarp_go.sh.
# Usage: run_exarp_go_tool.sh <tool_name> [json_args]
# Example: run_exarp_go_tool.sh lint
#          run_exarp_go_tool.sh lint '{"path":"native/src"}'
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TOOL="${1:-}"
ARGS="${2:-{}}"

if [[ -z "${TOOL}" ]]; then
  echo "Usage: $0 <tool_name> [json_args]" >&2
  echo "Example: $0 lint" >&2
  echo "List tools: $0 --list" >&2
  exit 1
fi

if [[ "${TOOL}" == "--list" ]]; then
  exec "${SCRIPT_DIR}/run_exarp_go.sh" -list -quiet
fi

# -quiet for script/CI; optional -json for machine-readable
exec "${SCRIPT_DIR}/run_exarp_go.sh" -tool "${TOOL}" -args "${ARGS}" -quiet
