#!/usr/bin/env bash
# setup_ramdisk.sh - Create and manage RAM disk for faster builds
# Usage: ./setup_ramdisk.sh [create|mount|unmount|status|cleanup]
#
# RAM disk improves build performance by:
# - Faster I/O operations
# - Reduced disk wear
# - Temporary builds don't clutter your SSD

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

# RAM disk configuration
RAMDISK_SIZE_GB="${RAMDISK_SIZE_GB:-8}" # Default 8GB, override with env var
RAMDISK_NAME="IBBoxSpreadBuild"
RAMDISK_PATH="/Volumes/${RAMDISK_NAME}"
BUILD_PATH_ON_RAMDISK="${RAMDISK_PATH}/build"

# Calculate sector count (1GB = 2097152 sectors, 1 sector = 512 bytes)
RAMDISK_SECTORS=$((RAMDISK_SIZE_GB * 2097152))

function log_info() {
  echo "ℹ️  $*"
}

function log_success() {
  echo "✓ $*"
}

function log_error() {
  echo "✗ $*" >&2
}

function create_ramdisk() {
  if [ -d "${RAMDISK_PATH}" ]; then
    log_info "RAM disk already exists at ${RAMDISK_PATH}"
    return 0
  fi

  log_info "Creating ${RAMDISK_SIZE_GB}GB RAM disk: ${RAMDISK_NAME}..."

  # Create RAM disk
  DEVICE=$(hdiutil attach -nomount "ram://${RAMDISK_SECTORS}" 2>/dev/null)
  if [ -z "${DEVICE}" ]; then
    log_error "Failed to create RAM disk"
    return 1
  fi

  # Format as APFS (faster than HFS+ on modern macOS)
  diskutil eraseDisk APFS "${RAMDISK_NAME}" "${DEVICE}" >/dev/null 2>&1 || {
    # Fallback to HFS+ if APFS fails
    diskutil eraseDisk HFS+ "${RAMDISK_NAME}" "${DEVICE}" >/dev/null 2>&1
  }

  if [ -d "${RAMDISK_PATH}" ]; then
    log_success "RAM disk created at ${RAMDISK_PATH}"

    # Create build directory structure
    mkdir -p "${BUILD_PATH_ON_RAMDISK}"
    log_success "Build directory created: ${BUILD_PATH_ON_RAMDISK}"

    # Create symlink from project to RAM disk build
    if [ ! -L "${PROJECT_ROOT}/build-ramdisk" ]; then
      ln -sf "${BUILD_PATH_ON_RAMDISK}" "${PROJECT_ROOT}/build-ramdisk"
      log_success "Symlink created: ${PROJECT_ROOT}/build-ramdisk -> ${BUILD_PATH_ON_RAMDISK}"
    fi

    return 0
  else
    log_error "RAM disk created but path not found: ${RAMDISK_PATH}"
    return 1
  fi
}

function unmount_ramdisk() {
  if [ ! -d "${RAMDISK_PATH}" ]; then
    log_info "RAM disk not mounted: ${RAMDISK_PATH}"
    return 0
  fi

  log_info "Unmounting RAM disk: ${RAMDISK_NAME}..."

  # Remove symlink
  if [ -L "${PROJECT_ROOT}/build-ramdisk" ]; then
    rm -f "${PROJECT_ROOT}/build-ramdisk"
    log_success "Symlink removed"
  fi

  # Unmount disk
  diskutil unmount "${RAMDISK_PATH}" >/dev/null 2>&1 || {
    # Force unmount if needed
    diskutil unmount force "${RAMDISK_PATH}" >/dev/null 2>&1
  }

  log_success "RAM disk unmounted"
}

function ramdisk_status() {
  if [ -d "${RAMDISK_PATH}" ]; then
    log_success "RAM disk is mounted at: ${RAMDISK_PATH}"
    df -h "${RAMDISK_PATH}" | tail -1
    if [ -d "${BUILD_PATH_ON_RAMDISK}" ]; then
      echo ""
      log_info "Build directory size:"
      du -sh "${BUILD_PATH_ON_RAMDISK}" 2>/dev/null || echo "  (empty)"
    fi
    return 0
  else
    log_info "RAM disk is not mounted"
    return 1
  fi
}

function cleanup_ramdisk() {
  unmount_ramdisk

  # Find and detach any orphaned RAM disk volumes
  diskutil list | grep -q "${RAMDISK_NAME}" && {
    log_info "Cleaning up orphaned RAM disk volumes..."
    diskutil unmountDisk force /dev/disk* 2>/dev/null || true
  }

  log_success "Cleanup complete"
}

function usage() {
  cat <<EOF
Usage: $0 [command]

Commands:
  create    - Create a new RAM disk for builds
  unmount   - Unmount the RAM disk (builds will be lost!)
  status    - Show RAM disk status and usage
  cleanup   - Unmount and cleanup any orphaned RAM disks
  help      - Show this help message

Environment variables:
  RAMDISK_SIZE_GB - Size of RAM disk in GB (default: 8)

Examples:
  # Create 8GB RAM disk (default)
  $0 create

  # Create 16GB RAM disk
  RAMDISK_SIZE_GB=16 $0 create

  # Check status
  $0 status

  # Unmount when done (all builds will be lost!)
  $0 unmount

Notes:
  - RAM disk contents are lost on unmount or reboot
  - Recommended size: 8-16GB depending on project size
  - Use build-ramdisk/ directory for CMake builds
  - Regular build/ directory remains on disk
EOF
}

# Main command handler
case "${1:-help}" in
create)
  create_ramdisk
  echo ""
  log_info "Next steps:"
  echo "  1. Configure CMake: cmake --preset macos-x86_64-debug -B build-ramdisk"
  echo "  2. Build: cmake --build build-ramdisk"
  echo "  3. Test: Use build-ramdisk/bin/ib_box_spread"
  ;;
unmount)
  unmount_ramdisk
  ;;
status)
  ramdisk_status
  ;;
cleanup)
  cleanup_ramdisk
  ;;
help | --help | -h)
  usage
  ;;
*)
  log_error "Unknown command: $1"
  usage
  exit 1
  ;;
esac
