#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT_DEFAULT="$(cd "${SCRIPT_DIR}/.." && pwd)"
export PROJECT_ROOT="${PROJECT_ROOT:-${PROJECT_ROOT_DEFAULT}}"

sanitize_go_env() {
  if [[ -n "${GOROOT:-}" ]] && [[ ! -d "${GOROOT}" ]]; then
    unset GOROOT
  fi
}

resolve_exarp_go() {
  local candidate=""

  if command -v exarp-go >/dev/null 2>&1; then
    candidate="$(command -v exarp-go)"
  elif [[ -n "${EXARP_GO_ROOT:-}" ]] && [[ -x "${EXARP_GO_ROOT}/bin/exarp-go" ]]; then
    candidate="${EXARP_GO_ROOT}/bin/exarp-go"
  elif [[ -x "${PROJECT_ROOT}/../mcp/exarp-go/bin/exarp-go" ]]; then
    candidate="${PROJECT_ROOT}/../mcp/exarp-go/bin/exarp-go"
  elif [[ -x "${PROJECT_ROOT}/../../mcp/exarp-go/bin/exarp-go" ]]; then
    candidate="${PROJECT_ROOT}/../../mcp/exarp-go/bin/exarp-go"
  elif [[ -x "${HOME}/go/bin/exarp-go" ]]; then
    candidate="${HOME}/go/bin/exarp-go"
  elif [[ -x "${HOME}/Projects/exarp-go/bin/exarp-go" ]]; then
    candidate="${HOME}/Projects/exarp-go/bin/exarp-go"
  elif [[ -x "/usr/local/bin/exarp-go" ]]; then
    candidate="/usr/local/bin/exarp-go"
  fi

  if [[ -z "${candidate}" ]]; then
    echo "exarp-go not found. Install it on PATH or set EXARP_GO_ROOT." >&2
    exit 1
  fi

  if [[ -z "${EXARP_MIGRATIONS_DIR:-}" ]] && [[ -n "${EXARP_GO_ROOT:-}" ]] && [[ -d "${EXARP_GO_ROOT}/migrations" ]]; then
    export EXARP_MIGRATIONS_DIR="${EXARP_GO_ROOT}/migrations"
  fi

  if [[ "${EXARP_GO_VERBOSE:-0}" == "1" ]]; then
    echo "[exarp-go] PROJECT_ROOT=${PROJECT_ROOT}" >&2
    echo "[exarp-go] using ${candidate}" >&2
  fi

  printf '%s\n' "${candidate}"
}

sanitize_go_env
EXARP_GO_BIN="$(resolve_exarp_go)"

# When Cursor (or another host) runs this script as MCP with no args, ensure exarp-go
# runs with project root as cwd so it finds .todo2/, config, etc. If cwd is already
# PROJECT_ROOT (e.g. via mcp.json "cwd"), this is a no-op.
if [[ "$#" -eq 0 ]] && [[ -n "${PROJECT_ROOT:-}" ]] && [[ -d "${PROJECT_ROOT}" ]]; then
  cd "${PROJECT_ROOT}"
fi

# MCP mode: no args + non-TTY stdin. Avoid git-hook guard in exarp-go so it starts the server.
if [[ "$#" -eq 0 ]]; then
  unset GIT_HOOK
fi

exec "${EXARP_GO_BIN}" "$@"
