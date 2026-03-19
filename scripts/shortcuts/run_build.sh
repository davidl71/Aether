#!/usr/bin/env bash
# Run a CMake build with a sensible default preset.
# Intended for use from macOS Shortcuts “Run Shell Script”.
#
# Usage:
#   ./scripts/shortcuts/run_build.sh [configure|build] [<cmake_preset>]
# Env:
#   CMAKE_PRESET - override preset (falls back to platform default)
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
LOG_DIR="${PROJECT_ROOT}/logs"
mkdir -p "${LOG_DIR}"
BUILD_LOG="${LOG_DIR}/build_latest.log"

detect_default_preset() {
  local arch os
  arch="$(uname -m 2>/dev/null || echo unknown)"
  os="$(uname -s 2>/dev/null || echo unknown)"
  case "${os}" in
  Darwin)
    if [[ "${arch}" == "arm64" || "${arch}" == "aarch64" ]]; then
      echo "macos-arm64-debug"
    else
      echo "macos-x86_64-debug"
    fi
    ;;
  Linux)
    echo "linux-x64-debug"
    ;;
  MINGW* | MSYS* | CYGWIN* | Windows_NT)
    echo "windows-x64-debug"
    ;;
  *)
    echo "macos-arm64-debug"
    ;;
  esac
}

ACTION="${1:-build}"
PRESET="${2:-${CMAKE_PRESET:-$(detect_default_preset)}}"

cd "${PROJECT_ROOT}"
# Use all cores for build when not set (so -j not needed)
# shellcheck source=scripts/include/set_parallel_level.sh
. "${SCRIPT_DIR}/../include/set_parallel_level.sh"
{
  echo "=== Build Runner ==="
  echo "Timestamp: $(date -Iseconds)"
  echo "Action: ${ACTION}"
  echo "Preset: ${PRESET}"
  echo ""

  if [[ "${ACTION}" == "configure" ]]; then
    cmake --preset "${PRESET}"
  else
    # Ensure configured then build
    cmake --preset "${PRESET}" >/dev/null 2>&1 || true
    cmake --build --preset "${PRESET}"
  fi
} 2>&1 | tee "${BUILD_LOG}"

echo ""
echo "Log written to ${BUILD_LOG}"
