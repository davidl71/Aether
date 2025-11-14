#!/usr/bin/env bash
# build_ramdisk.sh - Build project on RAM disk for faster compilation
# Usage: ./build_ramdisk.sh [build|clean|test|install|setup]
#
# This script:
# 1. Creates/sets up RAM disk if needed
# 2. Configures CMake to use RAM disk build directory
# 3. Builds the project on RAM disk
# 4. Optionally runs tests

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

# RAM disk configuration
RAMDISK_NAME="IBBoxSpreadBuild"
RAMDISK_PATH="/Volumes/${RAMDISK_NAME}"
RAMDISK_BUILD="${RAMDISK_PATH}/build"

# Use RAM disk build if available, otherwise fall back to regular build
if [ -d "${RAMDISK_PATH}" ] && [ -d "${RAMDISK_BUILD}" ]; then
  BUILD_DIR="${PROJECT_ROOT}/build-ramdisk"
  log_info() { echo "ℹ️  [RAM] $*"; }
else
  BUILD_DIR="${PROJECT_ROOT}/build"
  log_info() { echo "ℹ️  $*"; }
fi

DEFAULT_PRESET="macos-x86_64-debug"
PRESET="${CMAKE_PRESET:-${DEFAULT_PRESET}}"

# Source logging functions
if [ -f "${SCRIPT_DIR}/include/logging.sh" ]; then
  # shellcheck source=./include/logging.sh
  . "${SCRIPT_DIR}/include/logging.sh"
fi

function check_ramdisk() {
  if [ -d "${RAMDISK_PATH}" ] && [ -d "${RAMDISK_BUILD}" ]; then
    local available_space
    available_space=$(df -h "${RAMDISK_PATH}" | awk 'NR==2 {print $4}')
    log_info "Using RAM disk build: ${BUILD_DIR}"
    log_info "Available RAM disk space: ${available_space}"
    return 0
  else
    log_info "RAM disk not available, using regular build directory: ${BUILD_DIR}"
    log_info "Run './scripts/setup_ramdisk.sh create' to use RAM disk"
    return 1
  fi
}

function setup_ramdisk() {
  log_info "Setting up RAM disk..."
  "${SCRIPT_DIR}/setup_ramdisk.sh" create || {
    echo "Failed to create RAM disk, continuing with regular build"
    return 1
  }

  # Update BUILD_DIR to use RAM disk
  BUILD_DIR="${PROJECT_ROOT}/build-ramdisk"
}

function configure_cmake() {
  log_info "Configuring CMake with preset: ${PRESET}"
  log_info "Build directory: ${BUILD_DIR}"

  # Override binaryDir in preset to use RAM disk build directory
  local build_subdir
  build_subdir=$(echo "${PRESET}" | sed 's/macos-//')

  cmake --preset "${PRESET}" -B "${BUILD_DIR}" || {
    log_error "CMake configuration failed"
    return 1
  }

  log_success "CMake configured successfully"
}

function build_project() {
  log_info "Building project..."

  if [ ! -f "${BUILD_DIR}/build.ninja" ] && [ ! -f "${BUILD_DIR}/Makefile" ]; then
    log_info "Build files not found, configuring first..."
    configure_cmake
  fi

  cmake --build "${BUILD_DIR}" "$@" || {
    log_error "Build failed"
    return 1
  }

  log_success "Build completed successfully"
}

function run_tests() {
  log_info "Running tests..."

  if [ ! -d "${BUILD_DIR}" ]; then
    log_error "Build directory not found: ${BUILD_DIR}"
    log_info "Run './scripts/build_ramdisk.sh build' first"
    return 1
  fi

  ctest --test-dir "${BUILD_DIR}" --output-on-failure "$@" || {
    log_error "Tests failed"
    return 1
  }

  log_success "All tests passed"
}

function clean_build() {
  log_info "Cleaning build directory: ${BUILD_DIR}"

  if [ -d "${BUILD_DIR}" ]; then
    rm -rf "${BUILD_DIR:?}"/*
    log_success "Build directory cleaned"
  else
    log_info "Build directory does not exist"
  fi
}

function show_status() {
  echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
  echo "  RAM Disk Build Status"
  echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
  echo ""

  if check_ramdisk; then
    echo "RAM Disk: ✓ Mounted"
    df -h "${RAMDISK_PATH}" | tail -1 | awk '{printf "  Location: %s\n  Size: %s\n  Used: %s\n  Available: %s\n", $9, $2, $3, $4}'

    if [ -d "${BUILD_DIR}" ]; then
      echo ""
      echo "Build Directory: ✓ ${BUILD_DIR}"
      local build_size
      build_size=$(du -sh "${BUILD_DIR}" 2>/dev/null | cut -f1 || echo "0")
      echo "  Size: ${build_size}"
    fi
  else
    echo "RAM Disk: ✗ Not mounted"
    echo "  Run './scripts/setup_ramdisk.sh create' to enable RAM disk builds"
    echo ""
    echo "Using regular build directory: ${BUILD_DIR}"
  fi
  echo ""
}

function usage() {
  cat <<EOF
Usage: $0 [command] [options...]

Commands:
  setup     - Create and configure RAM disk for builds
  configure - Configure CMake (uses RAM disk if available)
  build     - Build the project (uses RAM disk if available)
  test      - Run tests (uses RAM disk build if available)
  clean     - Clean build directory
  status    - Show RAM disk and build status
  help      - Show this help message

Examples:
  # Setup RAM disk and build
  $0 setup
  $0 build

  # Or do it all at once
  $0 build

  # Run tests
  $0 test

  # Clean and rebuild
  $0 clean
  $0 build

Environment variables:
  CMAKE_PRESET - CMake preset to use (default: macos-x86_64-debug)
  RAMDISK_SIZE_GB - RAM disk size in GB (default: 8)

Notes:
  - Automatically uses RAM disk if available, falls back to regular build
  - RAM disk contents are lost on unmount or reboot
  - Recommended for faster iteration during development
EOF
}

# Check if we're using RAM disk
check_ramdisk || true

# Main command handler
case "${1:-build}" in
setup)
  setup_ramdisk
  configure_cmake
  ;;
configure)
  configure_cmake
  ;;
build)
  build_project "${@:2}"
  ;;
test)
  run_tests "${@:2}"
  ;;
clean)
  clean_build
  ;;
status)
  show_status
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
