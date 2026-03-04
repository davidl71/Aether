#!/usr/bin/env bash
# Portable exarp-go runner: use working-dir build when inside exarp-go repo,
# otherwise global install, with fallback to working-dir (EXARP_GO_ROOT or sibling).
#
# Usage: .cursor/mcp.json uses this script as the exarp-go command.
# Ensures exarp-go sees the correct project via PROJECT_ROOT (e.g. .todo2 and task store).
#
# Resolution order:
#   1. If running within exarp-go working dir (CWD or EXARP_GO_ROOT is exarp-go repo):
#      use that repo's bin/exarp-go (or "go run ." if bin not built).
#   2. Else: use globally installed exarp-go (PATH).
#   3. Fallback: EXARP_GO_ROOT/bin/exarp-go, then PROJECT_ROOT/../exarp-go/bin, then common paths.
#
# Env: PROJECT_ROOT (project exarp-go serves); EXARP_GO_ROOT (exarp-go repo); EXARP_GO_VERBOSE=1 to log choice.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
if [[ -z "${PROJECT_ROOT:-}" ]]; then
  PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
fi
export PROJECT_ROOT

# Run from project root so exarp-go detects project and finds .todo2 / task store
cd "${PROJECT_ROOT}"

# Returns 0 if dir looks like exarp-go repo (go.mod with exarp-go module + main or bin)
is_exarp_go_repo() {
  local dir="${1:-.}"
  [[ -f "${dir}/go.mod" ]] || return 1
  grep -q 'exarp-go' "${dir}/go.mod" 2>/dev/null || return 1
  [[ -f "${dir}/main.go" ]] || [[ -d "${dir}/cmd" ]] || [[ -x "${dir}/bin/exarp-go" ]] || return 1
  return 0
}

# Print path to exarp-go repo root, or empty: walk up from start_dir looking for exarp-go repo
find_exarp_go_root() {
  local start_dir="${1:-$(pwd)}"
  local d
  d="$(cd "${start_dir}" 2>/dev/null && pwd)" || return 1
  while [[ -n "${d}" ]] && [[ "${d}" != "/" ]]; do
    if is_exarp_go_repo "${d}"; then
      echo "${d}"
      return 0
    fi
    d="$(dirname "${d}")"
  done
  return 1
}

# Resolve which exarp-go binary to run; set EXARP_GO_BIN (path or "exarp-go" for PATH) and optionally EXARP_GO_ROOT
resolve_exarp_go_bin() {
  local cwd_root
  local fallback_root

  # 1) Within exarp-go working dir: prefer that repo's build
  if [[ -n "${EXARP_GO_ROOT:-}" ]] && is_exarp_go_repo "${EXARP_GO_ROOT}"; then
    if [[ -x "${EXARP_GO_ROOT}/bin/exarp-go" ]]; then
      EXARP_GO_BIN="${EXARP_GO_ROOT}/bin/exarp-go"
      [[ -n "${EXARP_GO_VERBOSE:-}" ]] && echo "[exarp-go] using EXARP_GO_ROOT bin: ${EXARP_GO_BIN}" >&2
      return 0
    fi
    if command -v go &>/dev/null; then
      EXARP_GO_RUN_GO=1
      [[ -n "${EXARP_GO_VERBOSE:-}" ]] && echo "[exarp-go] using EXARP_GO_ROOT with go run: ${EXARP_GO_ROOT}" >&2
      return 0
    fi
  fi

  cwd_root="$(find_exarp_go_root "$(pwd)" 2>/dev/null)" || true
  if [[ -n "${cwd_root}" ]]; then
    if [[ -x "${cwd_root}/bin/exarp-go" ]]; then
      EXARP_GO_BIN="${cwd_root}/bin/exarp-go"
      export EXARP_GO_ROOT="${cwd_root}"
      [[ -n "${EXARP_GO_VERBOSE:-}" ]] && echo "[exarp-go] using CWD exarp-go repo bin: ${EXARP_GO_BIN}" >&2
      return 0
    fi
    if command -v go &>/dev/null; then
      EXARP_GO_ROOT="${cwd_root}"
      EXARP_GO_RUN_GO=1
      export EXARP_GO_ROOT
      [[ -n "${EXARP_GO_VERBOSE:-}" ]] && echo "[exarp-go] using CWD exarp-go repo with go run: ${EXARP_GO_ROOT}" >&2
      return 0
    fi
  fi

  # 2) Global install
  if command -v exarp-go &>/dev/null; then
    EXARP_GO_BIN="exarp-go"
    [[ -n "${EXARP_GO_VERBOSE:-}" ]] && echo "[exarp-go] using PATH exarp-go" >&2
    return 0
  fi

  # 3) Fallback: working dir (EXARP_GO_ROOT or sibling of PROJECT_ROOT)
  fallback_root=""
  if [[ -n "${EXARP_GO_ROOT:-}" ]] && [[ -x "${EXARP_GO_ROOT}/bin/exarp-go" ]]; then
    fallback_root="${EXARP_GO_ROOT}"
  elif [[ -x "${PROJECT_ROOT}/../exarp-go/bin/exarp-go" ]]; then
    fallback_root="$(cd "${PROJECT_ROOT}/../exarp-go" && pwd)"
  fi
  if [[ -n "${fallback_root}" ]]; then
    EXARP_GO_BIN="${fallback_root}/bin/exarp-go"
    export EXARP_GO_ROOT="${fallback_root}"
    [[ -n "${EXARP_GO_VERBOSE:-}" ]] && echo "[exarp-go] using fallback working dir: ${EXARP_GO_BIN}" >&2
    return 0
  fi

  for candidate in \
    "${HOME}/go/bin/exarp-go" \
    "${HOME}/Projects/exarp-go/bin/exarp-go" \
    "/usr/local/bin/exarp-go"; do
    if [[ -x "${candidate}" ]]; then
      EXARP_GO_BIN="${candidate}"
      [[ -n "${EXARP_GO_VERBOSE:-}" ]] && echo "[exarp-go] using fallback path: ${EXARP_GO_BIN}" >&2
      return 0
    fi
  done

  return 1
}

EXARP_GO_BIN=""
EXARP_GO_RUN_GO=""
if ! resolve_exarp_go_bin; then
  echo "exarp-go not found. Install it, set PATH, or set EXARP_GO_ROOT to repo (with bin/exarp-go built)." >&2
  exit 1
fi

if [[ -n "${EXARP_GO_ROOT:-}" ]]; then
  export EXARP_MIGRATIONS_DIR="${EXARP_GO_ROOT}/migrations"
fi

if [[ -n "${EXARP_GO_RUN_GO:-}" ]]; then
  cd "${EXARP_GO_ROOT}"
  exec go run . "$@"
fi

exec "${EXARP_GO_BIN}" "$@"
