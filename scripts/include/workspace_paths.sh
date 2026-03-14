#!/usr/bin/env bash

# Shared workspace-local cache/temp/artifact path defaults for build and test scripts.

if [[ -n "${__IB_BOX_WORKSPACE_PATHS_INCLUDED:-}" ]]; then
  # shellcheck disable=SC2317
  return 0 2>/dev/null || true
fi
__IB_BOX_WORKSPACE_PATHS_INCLUDED=1

workspace_project_root() {
  if [[ -n "${PROJECT_ROOT:-}" ]]; then
    printf '%s\n' "${PROJECT_ROOT}"
    return 0
  fi

  local script_dir
  script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
  printf '%s\n' "$(cd "${script_dir}/../.." && pwd)"
}

setup_workspace_paths() {
  PROJECT_ROOT="${PROJECT_ROOT:-$(workspace_project_root)}"

  export WORKSPACE_CACHE_ROOT="${WORKSPACE_CACHE_ROOT:-${PROJECT_ROOT}/.cache}"
  export WORKSPACE_TMP_ROOT="${WORKSPACE_TMP_ROOT:-${WORKSPACE_CACHE_ROOT}/tmp}"
  export BUILD_ARTIFACT_ROOT="${BUILD_ARTIFACT_ROOT:-${PROJECT_ROOT}/build/test-artifacts}"

  export XDG_CACHE_HOME="${XDG_CACHE_HOME:-${WORKSPACE_CACHE_ROOT}}"
  export TMPDIR="${TMPDIR:-${WORKSPACE_TMP_ROOT}}"
  export UV_CACHE_DIR="${UV_CACHE_DIR:-${WORKSPACE_CACHE_ROOT}/uv}"
  export PIP_CACHE_DIR="${PIP_CACHE_DIR:-${WORKSPACE_CACHE_ROOT}/pip}"
  export SCCACHE_DIR="${SCCACHE_DIR:-${WORKSPACE_CACHE_ROOT}/sccache}"
  export CCACHE_DIR="${CCACHE_DIR:-${WORKSPACE_CACHE_ROOT}/ccache}"
  export GOCACHE="${GOCACHE:-${WORKSPACE_CACHE_ROOT}/go-build}"
  export GOMODCACHE="${GOMODCACHE:-${WORKSPACE_CACHE_ROOT}/go-mod}"
  export CARGO_HOME="${CARGO_HOME:-${WORKSPACE_CACHE_ROOT}/cargo}"
  export DISTCC_DIR="${DISTCC_DIR:-${WORKSPACE_CACHE_ROOT}/distcc}"

  mkdir -p \
    "${WORKSPACE_CACHE_ROOT}" \
    "${WORKSPACE_TMP_ROOT}" \
    "${BUILD_ARTIFACT_ROOT}" \
    "${UV_CACHE_DIR}" \
    "${PIP_CACHE_DIR}" \
    "${SCCACHE_DIR}" \
    "${CCACHE_DIR}" \
    "${GOCACHE}" \
    "${GOMODCACHE}" \
    "${CARGO_HOME}" \
    "${DISTCC_DIR}"
}
