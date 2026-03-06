#!/usr/bin/env bash
# setup_ramdisk.sh - Create/manage RAM disk for build-ramdisk workflow (macOS).
# Usage: ./setup_ramdisk.sh [create|status|unmount]
# Used by build_ramdisk.sh and workspace_ram_disk_manager.sh.
# Environment: RAMDISK_SIZE_GB (default 8), RAMDISK_NAME (default IBBoxSpreadBuild).

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

RAMDISK_NAME="${RAMDISK_NAME:-IBBoxSpreadBuild}"
RAMDISK_PATH="/Volumes/${RAMDISK_NAME}"
RAMDISK_SIZE_GB="${RAMDISK_SIZE_GB:-8}"
RAMDISK_BUILD="${RAMDISK_PATH}/build"
BUILD_RAMDISK_LINK="${PROJECT_ROOT}/build-ramdisk"

usage() {
  cat <<EOF
Usage: $0 [create|status|unmount]

  create   - Create RAM disk at ${RAMDISK_PATH}, create build dir, link build-ramdisk
  status   - Show whether RAM disk is mounted
  unmount  - Unmount RAM disk (data is lost)

Environment:
  RAMDISK_SIZE_GB  Size in GB (default: 8)
  RAMDISK_NAME     Volume name (default: IBBoxSpreadBuild)
EOF
}

create() {
  if [ "$(uname -s)" != "Darwin" ]; then
    echo "RAM disk setup is supported only on macOS." >&2
    return 1
  fi

  if [ -d "${RAMDISK_PATH}" ] && df "${RAMDISK_PATH}" >/dev/null 2>&1; then
    echo "RAM disk already mounted: ${RAMDISK_PATH}"
    mkdir -p "${RAMDISK_BUILD}"
    link_build_dir
    return 0
  fi

  local size_sectors
  size_sectors=$((RAMDISK_SIZE_GB * 1024 * 2048))
  echo "Creating ${RAMDISK_SIZE_GB}GB RAM disk: ${RAMDISK_PATH}"
  local dev
  dev=$(hdiutil attach -nomount "ram://${size_sectors}" | head -1 | awk '{print $1}')
  dev="${dev%%[[:space:]]*}"
  [ -z "${dev}" ] && { echo "hdiutil attach failed" >&2; return 1; }
  sleep 1
  diskutil eraseVolume HFS+ "${RAMDISK_NAME}" "${dev}"
  mkdir -p "${RAMDISK_BUILD}"
  link_build_dir
  echo "Created: ${RAMDISK_PATH} (build: ${RAMDISK_BUILD})"
}

link_build_dir() {
  if [ -L "${BUILD_RAMDISK_LINK}" ]; then
    return 0
  fi
  if [ -e "${BUILD_RAMDISK_LINK}" ]; then
    echo "Note: ${BUILD_RAMDISK_LINK} exists and is not a link; not overwriting." >&2
    return 0
  fi
  ln -sfn "${RAMDISK_BUILD}" "${BUILD_RAMDISK_LINK}"
  echo "Linked: ${BUILD_RAMDISK_LINK} -> ${RAMDISK_BUILD}"
}

status() {
  if [ "$(uname -s)" != "Darwin" ]; then
    echo "RAM disk status is supported only on macOS."
    exit 0
  fi
  if [ -d "${RAMDISK_PATH}" ] && df "${RAMDISK_PATH}" >/dev/null 2>&1; then
    echo "RAM disk: mounted at ${RAMDISK_PATH}"
    df -h "${RAMDISK_PATH}" | tail -1
    [ -d "${RAMDISK_BUILD}" ] && echo "Build dir: ${RAMDISK_BUILD}"
  else
    echo "RAM disk: not mounted (run: $0 create)"
  fi
}

unmount() {
  if [ "$(uname -s)" != "Darwin" ]; then
    echo "Unmount is supported only on macOS." >&2
    return 1
  fi
  if [ ! -d "${RAMDISK_PATH}" ]; then
    echo "RAM disk not mounted." >&2
    return 0
  fi
  rm -f "${BUILD_RAMDISK_LINK}" 2>/dev/null || true
  hdiutil detach "${RAMDISK_PATH}" || true
  echo "Unmounted: ${RAMDISK_PATH}"
}

case "${1:-}" in
  create)   create ;;
  status)   status ;;
  unmount)  unmount ;;
  -h|--help) usage ;;
  *)
    echo "Usage: $0 [create|status|unmount]" >&2
    exit 1
    ;;
esac
