#!/usr/bin/env bash
# Run a single exarp-go tool (lint, testing, security, etc.). Prefers native exarp-go
# from PATH, EXARP_GO_ROOT, or sibling repo (../mcp/exarp-go); falls back to in-repo
# scripts/run_exarp_go.sh if present.
# Usage: run_exarp_go_tool.sh <tool_name> [json_args]
# Example: run_exarp_go_tool.sh lint
#          run_exarp_go_tool.sh lint '{"path":"native/src"}'
#          run_exarp_go_tool.sh --list
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
export PROJECT_ROOT
PROJECT_HELPERS="${SCRIPT_DIR}/include/workspace_paths.sh"

if [[ -f "${PROJECT_HELPERS}" ]]; then
  # shellcheck source=/dev/null
  source "${PROJECT_HELPERS}"
fi

TOOL="${1:-}"
ARGS="${2:-{}}"

if [[ -z "${TOOL}" ]]; then
  echo "Usage: $0 <tool_name> [json_args]" >&2
  echo "Example: $0 lint" >&2
  echo "List tools: $0 --list" >&2
  exit 1
fi

sanitize_go_env() {
  # Local shells may export a stale GOROOT that breaks Go linters.
  if [[ -n "${GOROOT:-}" ]] && [[ ! -d "${GOROOT}" ]]; then
    unset GOROOT
  fi
}

# Resolve exarp-go: PATH -> EXARP_GO_ROOT -> sibling mcp/exarp-go -> in-repo run_exarp_go.sh
EXARP_GO_CMD=""
if command -v exarp-go &>/dev/null; then
  EXARP_GO_CMD="exarp-go"
elif [[ -n "${EXARP_GO_ROOT:-}" ]] && [[ -x "${EXARP_GO_ROOT}/bin/exarp-go" ]]; then
  EXARP_GO_CMD="${EXARP_GO_ROOT}/bin/exarp-go"
elif [[ -x "${PROJECT_ROOT}/../mcp/exarp-go/bin/exarp-go" ]]; then
  EXARP_GO_CMD="${PROJECT_ROOT}/../mcp/exarp-go/bin/exarp-go"
elif [[ -x "${PROJECT_ROOT}/../../mcp/exarp-go/bin/exarp-go" ]]; then
  EXARP_GO_CMD="${PROJECT_ROOT}/../../mcp/exarp-go/bin/exarp-go"
elif [[ -x "${SCRIPT_DIR}/run_exarp_go.sh" ]]; then
  # Fallback: in-repo portable runner (duplicate of exarp-go's script)
  if [[ "${TOOL}" == "--list" ]]; then
    exec "${SCRIPT_DIR}/run_exarp_go.sh" -list -quiet
  fi
  # Pass JSON args as single token to avoid any shell/flag splitting
  TMP_ARGS="$(mktemp -t exarp_args.XXXXXXXXXX)"
  printf '%s' "${ARGS}" > "${TMP_ARGS}"
  trap 'rm -f "${TMP_ARGS}"' EXIT
  EXARP_ARGS="$(cat "${TMP_ARGS}")"
  rm -f "${TMP_ARGS}"
  exec "${SCRIPT_DIR}/run_exarp_go.sh" -tool "${TOOL}" -args="${EXARP_ARGS}" -quiet
fi

if [[ -z "${EXARP_GO_CMD}" ]]; then
  echo "exarp-go not found. Install it (go install or build from exarp-go repo), set PATH or EXARP_GO_ROOT, or keep scripts/run_exarp_go.sh in this repo." >&2
  exit 1
fi

sanitize_go_env

if [[ "${TOOL}" == "--list" ]]; then
  exec "${EXARP_GO_CMD}" -list -quiet
fi

exec "${EXARP_GO_CMD}" -tool "${TOOL}" -args "${ARGS}" -quiet
