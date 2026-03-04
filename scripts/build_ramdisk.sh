#!/usr/bin/env bash
# build_ramdisk.sh - Thin wrapper: RAM disk setup + build_portable with -ramdisk preset (macOS).
#
# Usage: ./scripts/build_ramdisk.sh [setup|configure|build|test|clean|status|help]
#
# Delegates to setup_ramdisk.sh (create/status) and build_portable.sh (configure/build/test/clean).
# Uses CMake preset macos-<arch>-debug-ramdisk when build-ramdisk is available (see setup_ramdisk.sh).
# On non-macOS or when ramdisk is not set up, falls back to build_portable without -ramdisk.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
cd "${PROJECT_ROOT}"

# Resolve ramdisk preset (macOS only; preset exists in CMakePresets.json).
ramdisk_preset() {
  if [[ "$(uname -s)" != "Darwin" ]]; then
    return 1
  fi
  local arch
  arch="$(uname -m 2>/dev/null || echo unknown)"
  case "${arch}" in
    arm64|aarch64) echo "macos-arm64-debug-ramdisk" ;;
    x86_64)        echo "macos-x86_64-debug-ramdisk" ;;
    *)             return 1 ;;
  esac
}

# Use -ramdisk preset only if build-ramdisk link/dir exists (ramdisk was set up).
use_ramdisk_preset() {
  [[ -e "${PROJECT_ROOT}/build-ramdisk" ]] && ramdisk_preset || return 1
}

usage() {
  cat <<EOF
Usage: $0 [command]

Commands:
  setup     - Create RAM disk and link build-ramdisk, then configure (macOS)
  configure - Configure CMake (uses -ramdisk preset if build-ramdisk exists)
  build     - Build (delegates to build_portable.sh)
  test      - Run tests (delegates to build_portable.sh)
  clean     - Clean build (delegates to build_portable.sh)
  status    - Show RAM disk status (delegates to setup_ramdisk.sh status)
  help      - This message

Environment:
  CMAKE_PRESET  Override preset (e.g. macos-arm64-debug-ramdisk)
  USE_NIX=1     Passed through to build_portable.sh
EOF
}

CMD="${1:-build}"
case "${CMD}" in
  setup)
    "${SCRIPT_DIR}/setup_ramdisk.sh" create
    PRESET="$(use_ramdisk_preset)" || PRESET="$(ramdisk_preset)"
    export CMAKE_PRESET="${PRESET}"
    exec "${SCRIPT_DIR}/build_portable.sh" configure
    ;;
  configure|build|test|clean)
    PRESET="$(use_ramdisk_preset)" || PRESET=""
    if [[ -n "${PRESET}" ]]; then
      export CMAKE_PRESET="${PRESET}"
    fi
    exec "${SCRIPT_DIR}/build_portable.sh" "${CMD}" "${@:2}"
    ;;
  status)
    exec "${SCRIPT_DIR}/setup_ramdisk.sh" status
    ;;
  help|--help|-h)
    usage
    exit 0
    ;;
  *)
    echo "Unknown command: ${CMD}" >&2
    usage
    exit 1
    ;;
esac
