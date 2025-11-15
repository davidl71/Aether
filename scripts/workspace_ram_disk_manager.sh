#!/usr/bin/env bash
# workspace_ram_disk_manager.sh - Workspace lifecycle RAM disk management
# Handles startup (create/pre-warm) and shutdown (save) for RAM disk
# Usage: ./workspace_ram_disk_manager.sh [startup|shutdown|save|status]

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

# RAM disk configuration
RAMDISK_NAME="IBBoxSpreadBuild"
RAMDISK_PATH="/Volumes/${RAMDISK_NAME}"
RAMDISK_BUILD="${RAMDISK_PATH}/build"
BUILD_RAMDISK_LINK="${PROJECT_ROOT}/build-ramdisk"

# Cache RAM disk configuration
CACHE_RAMDISK_NAME="IBBoxSpreadDev"
CACHE_RAMDISK_PATH="/Volumes/${CACHE_RAMDISK_NAME}"

# Backup location for saved builds
SAVED_BUILDS_DIR="${PROJECT_ROOT}/.saved-builds"

function log_info() {
  echo "ℹ️  $*"
}

function log_success() {
  echo "✓ $*"
}

function log_error() {
  echo "✗ $*" >&2
}

function check_ramdisk_mounted() {
  local ramdisk_path="$1"
  if [ -d "${ramdisk_path}" ] && mountpoint -q "${ramdisk_path}" 2>/dev/null || [ -d "${ramdisk_path}" ]; then
    return 0
  fi
  return 1
}

function create_build_ramdisk() {
  if check_ramdisk_mounted "${RAMDISK_PATH}"; then
    log_info "Build RAM disk already exists: ${RAMDISK_PATH}"
    return 0
  fi

  log_info "Creating build RAM disk..."
  "${SCRIPT_DIR}/setup_ramdisk.sh" create || {
    log_error "Failed to create build RAM disk"
    return 1
  }

  log_success "Build RAM disk created"
}

function create_cache_ramdisk() {
  if check_ramdisk_mounted "${CACHE_RAMDISK_PATH}"; then
    log_info "Cache RAM disk already exists: ${CACHE_RAMDISK_PATH}"
    return 0
  fi

  log_info "Creating cache RAM disk..."
  "${SCRIPT_DIR}/setup_ram_optimization.sh" enable || {
    log_error "Failed to create cache RAM disk"
    return 1
  }

  log_success "Cache RAM disk created"
}

function prewarm_ramdisk() {
  if ! check_ramdisk_mounted "${RAMDISK_PATH}"; then
    log_error "Build RAM disk not mounted: ${RAMDISK_PATH}"
    return 1
  fi

  log_info "Pre-warming build RAM disk..."

  # Create build directory structure
  mkdir -p "${RAMDISK_BUILD}"

  # Check if we have a saved build to restore
  local latest_save
  latest_save=$(ls -td "${SAVED_BUILDS_DIR}"/* 2>/dev/null | head -1 || true)

  if [ -n "${latest_save}" ] && [ -d "${latest_save}" ]; then
    log_info "Restoring saved build from: $(basename "${latest_save}")"
    rsync -a --progress "${latest_save}/" "${RAMDISK_BUILD}/" 2>/dev/null || {
      log_info "Build restore skipped (no saved build or error)"
    }
  fi

  # Pre-create common directories
  mkdir -p "${RAMDISK_BUILD}"/{bin,lib,CMakeFiles}

  log_success "Build RAM disk pre-warmed"
}

function prewarm_cache_ramdisk() {
  if ! check_ramdisk_mounted "${CACHE_RAMDISK_PATH}"; then
    log_error "Cache RAM disk not mounted: ${CACHE_RAMDISK_PATH}"
    return 1
  fi

  log_info "Pre-warming cache RAM disk..."

  # Ensure cache directories exist
  mkdir -p "${CACHE_RAMDISK_PATH}/caches"/{ccache,sccache,pip,node,cargo-registry,cargo-git}

  # Source environment if available
  if [ -f "${PROJECT_ROOT}/.ram-optimization-env" ]; then
    source "${PROJECT_ROOT}/.ram-optimization-env"
    log_success "Cache RAM disk environment loaded"
  fi

  log_success "Cache RAM disk pre-warmed"
}

function save_build_artifacts() {
  if ! check_ramdisk_mounted "${RAMDISK_PATH}"; then
    log_info "Build RAM disk not mounted, nothing to save"
    return 0
  fi

  if [ ! -d "${RAMDISK_BUILD}" ]; then
    log_info "No build directory on RAM disk, nothing to save"
    return 0
  fi

  log_info "Saving build artifacts from RAM disk..."

  # Create saved builds directory
  mkdir -p "${SAVED_BUILDS_DIR}"

  # Create timestamped backup
  local timestamp
  timestamp=$(date +%Y%m%d-%H%M%S)
  local save_path="${SAVED_BUILDS_DIR}/${timestamp}"

  # Save important build artifacts (not everything to save space/time)
  log_info "Saving to: ${save_path}"

  mkdir -p "${save_path}"

  # Save compile_commands.json (important for IDE)
  if [ -f "${RAMDISK_BUILD}/compile_commands.json" ]; then
    cp "${RAMDISK_BUILD}/compile_commands.json" "${save_path}/" 2>/dev/null || true
    log_info "Saved compile_commands.json"
  fi

  # Save CMakeCache.txt (preserves configuration)
  if [ -f "${RAMDISK_BUILD}/CMakeCache.txt" ]; then
    cp "${RAMDISK_BUILD}/CMakeCache.txt" "${save_path}/" 2>/dev/null || true
    log_info "Saved CMakeCache.txt"
  fi

  # Save binaries (most important)
  if [ -d "${RAMDISK_BUILD}/bin" ]; then
    rsync -a --progress "${RAMDISK_BUILD}/bin/" "${save_path}/bin/" 2>/dev/null || true
    log_info "Saved binaries"
  fi

  # Save libraries
  if [ -d "${RAMDISK_BUILD}/lib" ]; then
    rsync -a --progress "${RAMDISK_BUILD}/lib/" "${save_path}/lib/" 2>/dev/null || true
    log_info "Saved libraries"
  fi

  # Keep only last 3 saves to save disk space
  local saved_count
  saved_count=$(ls -d "${SAVED_BUILDS_DIR}"/* 2>/dev/null | wc -l | tr -d ' ')
  if [ "${saved_count}" -gt 3 ]; then
    log_info "Cleaning up old saves (keeping last 3)..."
    ls -dt "${SAVED_BUILDS_DIR}"/* 2>/dev/null | tail -n +4 | xargs rm -rf 2>/dev/null || true
  fi

  log_success "Build artifacts saved to: ${save_path}"
}

function save_cache_data() {
  if ! check_ramdisk_mounted "${CACHE_RAMDISK_PATH}"; then
    log_info "Cache RAM disk not mounted, nothing to save"
    return 0
  fi

  log_info "Saving cache data from RAM disk..."

  # Cache data is less critical (will be rebuilt), but save ccache/sccache stats
  local cache_backup="${PROJECT_ROOT}/.cache-backups"
  mkdir -p "${cache_backup}"

  # Save cache statistics (useful for analysis)
  if command -v ccache >/dev/null 2>&1; then
    ccache --show-stats > "${cache_backup}/ccache-stats-$(date +%Y%m%d).txt" 2>/dev/null || true
  fi

  if command -v sccache >/dev/null 2>&1; then
    sccache --show-stats > "${cache_backup}/sccache-stats-$(date +%Y%m%d).txt" 2>/dev/null || true
  fi

  log_success "Cache statistics saved"
}

function handle_startup() {
  log_info "Workspace startup: Initializing RAM disks..."

  # Create build RAM disk
  create_build_ramdisk || {
    log_error "Failed to create build RAM disk, continuing without it"
  }

  # Create cache RAM disk
  create_cache_ramdisk || {
    log_error "Failed to create cache RAM disk, continuing without it"
  }

  # Pre-warm build RAM disk (restore saved build if available)
  if check_ramdisk_mounted "${RAMDISK_PATH}"; then
    prewarm_ramdisk || {
      log_info "Pre-warming skipped (no saved build or error)"
    }
  fi

  # Pre-warm cache RAM disk
  if check_ramdisk_mounted "${CACHE_RAMDISK_PATH}"; then
    prewarm_cache_ramdisk || true
  fi

  # Source environment for cache optimization
  if [ -f "${PROJECT_ROOT}/.ram-optimization-env" ]; then
    log_info "To activate cache optimization, run: source .ram-optimization-env"
  fi

  log_success "Workspace RAM disk initialization complete"
  show_status
}

function handle_shutdown() {
  log_info "Workspace shutdown: Saving RAM disk data..."

  # Save build artifacts
  save_build_artifacts || {
    log_error "Failed to save build artifacts"
  }

  # Save cache statistics
  save_cache_data || true

  # Optionally unmount (commented out by default to keep RAM disk mounted)
  # Uncomment if you want to unmount on shutdown:
  # log_info "Unmounting RAM disks..."
  # "${SCRIPT_DIR}/setup_ramdisk.sh" unmount || true

  log_success "Workspace shutdown complete"
}

function show_status() {
  echo ""
  echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
  echo "  RAM Disk Status"
  echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
  echo ""

  # Build RAM disk
  if check_ramdisk_mounted "${RAMDISK_PATH}"; then
    log_success "Build RAM Disk: ✓ ${RAMDISK_PATH}"
    df -h "${RAMDISK_PATH}" | tail -1 | awk '{printf "  Size: %s, Used: %s, Available: %s\n", $2, $3, $4}'
    if [ -d "${RAMDISK_BUILD}" ]; then
      local build_size
      build_size=$(du -sh "${RAMDISK_BUILD}" 2>/dev/null | cut -f1 || echo "0")
      echo "  Build size: ${build_size}"
    fi
  else
    log_info "Build RAM Disk: ✗ Not mounted"
  fi

  echo ""

  # Cache RAM disk
  if check_ramdisk_mounted "${CACHE_RAMDISK_PATH}"; then
    log_success "Cache RAM Disk: ✓ ${CACHE_RAMDISK_PATH}"
    df -h "${CACHE_RAMDISK_PATH}" | tail -1 | awk '{printf "  Size: %s, Used: %s, Available: %s\n", $2, $3, $4}'
  else
    log_info "Cache RAM Disk: ✗ Not mounted"
  fi

  echo ""

  # Saved builds
  if [ -d "${SAVED_BUILDS_DIR}" ]; then
    local saved_count
    saved_count=$(ls -d "${SAVED_BUILDS_DIR}"/* 2>/dev/null | wc -l | tr -d ' ' || echo "0")
    if [ "${saved_count}" -gt 0 ]; then
      log_info "Saved builds: ${saved_count}"
      ls -lt "${SAVED_BUILDS_DIR}" 2>/dev/null | head -4 | tail -n +2 | awk '{printf "  %s (%s)\n", $9, $6" "$7" "$8}' || true
    else
      log_info "Saved builds: None"
    fi
  fi

  echo ""
}

function usage() {
  cat <<EOF
Usage: $0 [command]

Commands:
  startup   - Initialize RAM disks on workspace startup (create/pre-warm)
  shutdown  - Save RAM disk data on workspace shutdown
  save      - Save build artifacts now (without shutdown)
  status    - Show RAM disk status

Examples:
  # On workspace open (run automatically via task)
  $0 startup

  # Before closing workspace (run manually or via task)
  $0 shutdown

  # Save current build artifacts
  $0 save

  # Check status
  $0 status

EOF
}

# Main command handler
case "${1:-status}" in
startup)
  handle_startup
  ;;
shutdown)
  handle_shutdown
  ;;
save)
  save_build_artifacts
  save_cache_data
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
