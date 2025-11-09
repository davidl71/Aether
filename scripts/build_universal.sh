#!/usr/bin/env bash
# build_universal.sh - Universal binary builder for IBKR Box Spread Generator
# Usage: ./build_universal.sh [build|clean|test|install]
# See README “Build” section for prerequisites; script assumes macOS with cmake + Xcode tools.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=./include/logging.sh
. "${SCRIPT_DIR}/include/logging.sh"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
DEFAULT_PRESET="macos-universal-release"
PRESET="${CMAKE_PRESET:-${DEFAULT_PRESET}}"

if ! command -v cmake >/dev/null 2>&1; then
  log_error "cmake not found. Install CMake before running this wrapper."
  exit 1
fi

cd "${PROJECT_ROOT}"

log_note "Using CMake preset '${PRESET}'. Override with CMAKE_PRESET env var."

ensure_configured() {
  log_info "Configuring preset ${PRESET}..."
  cmake --preset "${PRESET}" >/dev/null
}

if [ "$#" -gt 0 ]; then
  command="$1"
  shift
else
  command="build"
fi

case "${command}" in
  -h|--help)
    cat <<EOF
Usage: $0 [build|clean|test|install]

Environment:
  CMAKE_PRESET   Override the CMake preset (default: ${DEFAULT_PRESET})

This script is a thin wrapper around \`cmake --preset\`, \`cmake --build\`,
and \`ctest --preset\`.
EOF
    ;;
  clean)
    ensure_configured
    cmake --build --preset "${PRESET}" --target clean "$@"
    ;;
  test)
    ensure_configured
    ctest --preset "${PRESET}" "$@"
    ;;
  install)
    ensure_configured
    cmake --install --preset "${PRESET}" "$@"
    ;;
  build|*)
    ensure_configured
    cmake --build --preset "${PRESET}" "$@"
    ;;
esac
