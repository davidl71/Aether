#!/usr/bin/env bash
# build_ramdisk.sh - RAM disk setup + build_portable with -ramdisk preset (macOS) or /dev/shm (Linux).
#
# Usage: ./scripts/build_ramdisk.sh [setup|configure|build|test|clean|status|help]
#
# Delegates to setup_ramdisk.sh (create/status) on macOS and build_portable.sh (configure/build/test/clean).
# On Linux, uses /dev/shm if available (no setup needed - just creates build-ramdisk symlink).
# On macOS, uses setup_ramdisk.sh to create a RAM disk and links build-ramdisk.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
cd "${PROJECT_ROOT}"

DEVSHM_PATH="/dev/shm"
RAMDISK_LINK="${PROJECT_ROOT}/build-ramdisk"

# Detect if we're on Linux with /dev/shm available
use_devshm() {
  [[ "$(uname -s)" == "Linux" ]] && [[ -d "${DEVSHM_PATH}" ]] && [[ -w "${DEVSHM_PATH}" ]]
}

# Detect if we're on macOS (ramdisk setup via setup_ramdisk.sh)
use_macos_ramdisk() {
  [[ "$(uname -s)" == "Darwin" ]]
}

# Resolve ramdisk preset (macOS only; preset exists in CMakePresets.json).
ramdisk_preset() {
  if [[ "$(uname -s)" != "Darwin" ]]; then
    return 1
  fi
  local arch
  arch="$(uname -m 2>/dev/null || echo unknown)"
  case "${arch}" in
  arm64 | aarch64) echo "macos-arm64-debug-ramdisk" ;;
  x86_64) echo "macos-x86_64-debug-ramdisk" ;;
  *) return 1 ;;
  esac
}

# Linux preset for /dev/shm builds
devshm_preset() {
  local arch
  arch="$(uname -m 2>/dev/null || echo unknown)"
  case "${arch}" in
  x86_64) echo "linux-x64-debug" ;;
  aarch64) echo "linux-aarch64-debug" ;;
  *) return 1 ;;
  esac
}

# Check if build-ramdisk link/dir exists (ramdisk was set up)
build_ramdisk_exists() {
  [[ -e "${PROJECT_ROOT}/build-ramdisk" ]]
}

# Setup /dev/shm for Linux
setup_devshm() {
  if ! use_devshm; then
    echo "Error: /dev/shm not available or not writable on Linux" >&2
    return 1
  fi

  local devshm_build="${DEVSHM_PATH}/build"
  mkdir -p "${devshm_build}"

  if [[ -L "${RAMDISK_LINK}" ]]; then
    echo "Link already exists: ${RAMDISK_LINK}"
  elif [[ -e "${RAMDISK_LINK}" ]]; then
    echo "Warning: ${RAMDISK_LINK} exists but is not a symlink" >&2
  else
    ln -sfn "${devshm_build}" "${RAMDISK_LINK}"
    echo "Linked: ${RAMDISK_LINK} -> ${devshm_build}"
  fi

  echo "Build directory: ${devshm_build}"
  echo "Available space:"
  df -h "${DEVSHM_PATH}" | tail -1
}

# Show status
show_status() {
  if use_macos_ramdisk; then
    exec "${SCRIPT_DIR}/setup_ramdisk.sh" status
  elif use_devshm; then
    if build_ramdisk_exists; then
      echo "build-ramdisk: linked to ${DEVSHM_PATH}/build"
      ls -la "${RAMDISK_LINK}" 2>/dev/null || true
    else
      echo "build-ramdisk: not linked (run: $0 setup)"
    fi
    echo "/dev/shm status:"
    df -h "${DEVSHM_PATH}" | tail -1
  else
    echo "RAM disk not available on this platform"
  fi
}

usage() {
  cat <<EOF
Usage: $0 [command]

Commands:
  setup     - Create RAM disk and link build-ramdisk (macOS: setup_ramdisk.sh; Linux: /dev/shm)
  configure - Configure CMake (uses -ramdisk preset on macOS, linux-x64-debug on Linux if build-ramdisk exists)
  build     - Build (delegates to build_portable.sh)
  test      - Run tests (delegates to build_portable.sh)
  clean     - Clean build (delegates to build_portable.sh)
  status    - Show RAM disk status (macOS: setup_ramdisk.sh; Linux: /dev/shm)
  help      - This message

Environment:
  CMAKE_PRESET  Override preset (e.g. macos-arm64-debug-ramdisk, linux-x64-debug)
  USE_NIX=1     Passed through to build_portable.sh
EOF
}

CMD="${1:-build}"
case "${CMD}" in
setup)
  if use_macos_ramdisk; then
    "${SCRIPT_DIR}/setup_ramdisk.sh" create
    PRESET="$(ramdisk_preset)" || PRESET=""
  elif use_devshm; then
    setup_devshm
    PRESET="$(devshm_preset)" || PRESET=""
  else
    echo "Error: RAM disk setup not supported on this platform" >&2
    exit 1
  fi
  if [[ -n "${PRESET:-}" ]]; then
    export CMAKE_PRESET="${PRESET}"
  fi
  exec "${SCRIPT_DIR}/build_portable.sh" configure
  ;;
configure | build | test | clean)
  PRESET=""
  if build_ramdisk_exists; then
    if use_macos_ramdisk; then
      PRESET="$(ramdisk_preset)" || true
    elif use_devshm; then
      PRESET="$(devshm_preset)" || true
    fi
  fi
  if [[ -n "${PRESET}" ]]; then
    export CMAKE_PRESET="${PRESET}"
  fi
  exec "${SCRIPT_DIR}/build_portable.sh" "${CMD}" "${@:2}"
  ;;
status)
  show_status
  ;;
help | --help | -h)
  usage
  exit 0
  ;;
*)
  echo "Unknown command: ${CMD}" >&2
  usage
  exit 1
  ;;
esac
