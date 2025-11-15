#!/usr/bin/env bash
# setup_ram_optimization.sh - Optimize development workflow using RAM disks
# Usage: ./setup_ram_optimization.sh [enable|disable|status]
#
# This script optimizes development workflow by:
# 1. Moving compiler caches (ccache/sccache) to RAM disk
# 2. Moving Python cache to RAM disk
# 3. Moving Rust cargo cache to RAM disk (if applicable)
# 4. Moving Node.js cache to RAM disk (if applicable)
# 5. Setting up Redis for distributed sccache (optional)
#
# Benefits:
# - Faster cache access (RAM vs SSD: 100x+ faster)
# - Reduced disk I/O and wear
# - Faster builds, tests, and development iterations

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

# RAM disk configuration
RAMDISK_NAME="IBBoxSpreadDev"
RAMDISK_PATH="/Volumes/${RAMDISK_NAME}"
RAMDISK_SIZE_GB="${RAMDISK_SIZE_GB:-12}" # Default 12GB for caches + build

# Cache directories to move to RAM
CCACHE_DIR="${CCACHE_DIR:-$HOME/.ccache}"
SCCACHE_DIR="${SCCACHE_DIR:-$HOME/.sccache}"
PYTHON_CACHE_DIR="${PYTHON_CACHE_DIR:-$HOME/.cache/pip}"
RUST_CACHE_DIR="${RUST_CACHE_DIR:-$HOME/.cargo}"
NODE_CACHE_DIR="${NODE_CACHE_DIR:-$HOME/.cache/node}"
CARGO_CACHE_DIR="${CARGO_CACHE_DIR:-$HOME/.cargo/registry}"
CARGO_GIT_CACHE_DIR="${CARGO_GIT_CACHE_DIR:-$HOME/.cargo/git}"

# Backup location for original caches (on disk)
BACKUP_DIR="${HOME}/.cache-backups"

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

  # Calculate sector count (1GB = 2097152 sectors, 1 sector = 512 bytes)
  local sectors=$((RAMDISK_SIZE_GB * 2097152))

  # Create RAM disk
  local device
  device=$(hdiutil attach -nomount "ram://${sectors}" 2>/dev/null)
  if [ -z "${device}" ]; then
    log_error "Failed to create RAM disk"
    return 1
  fi

  # Format as APFS
  diskutil eraseDisk APFS "${RAMDISK_NAME}" "${device}" >/dev/null 2>&1 || {
    diskutil eraseDisk HFS+ "${RAMDISK_NAME}" "${device}" >/dev/null 2>&1
  }

  if [ -d "${RAMDISK_PATH}" ]; then
    log_success "RAM disk created at ${RAMDISK_PATH}"

    # Create directory structure
    mkdir -p "${RAMDISK_PATH}/caches/ccache"
    mkdir -p "${RAMDISK_PATH}/caches/sccache"
    mkdir -p "${RAMDISK_PATH}/caches/pip"
    mkdir -p "${RAMDISK_PATH}/caches/node"
    mkdir -p "${RAMDISK_PATH}/caches/cargo-registry"
    mkdir -p "${RAMDISK_PATH}/caches/cargo-git"
    mkdir -p "${RAMDISK_PATH}/tmp"

    log_success "Cache directories created on RAM disk"
    return 0
  else
    log_error "RAM disk created but path not found: ${RAMDISK_PATH}"
    return 1
  fi
}

function backup_and_link_cache() {
  local original_dir="$1"
  local ramdisk_dir="$2"
  local cache_name="$3"

  if [ ! -d "${RAMDISK_PATH}" ]; then
    log_error "RAM disk not mounted: ${RAMDISK_PATH}"
    return 1
  fi

  # Create backup directory
  mkdir -p "${BACKUP_DIR}"

  # If original directory exists and is not already a symlink
  if [ -d "${original_dir}" ] && [ ! -L "${original_dir}" ]; then
    log_info "Backing up ${cache_name} cache..."
    local backup_path="${BACKUP_DIR}/$(basename "${original_dir}").$(date +%Y%m%d-%H%M%S).tar.gz"
    tar -czf "${backup_path}" -C "$(dirname "${original_dir}")" "$(basename "${original_dir}")" 2>/dev/null || true
    log_success "Backed up to ${backup_path}"

    # Move original to backup location temporarily
    mv "${original_dir}" "${BACKUP_DIR}/$(basename "${original_dir}").original" 2>/dev/null || true
  fi

  # Copy existing cache to RAM disk if it exists in backup
  if [ -d "${BACKUP_DIR}/$(basename "${original_dir}").original" ]; then
    log_info "Copying ${cache_name} cache to RAM disk..."
    cp -R "${BACKUP_DIR}/$(basename "${original_dir}").original"/* "${ramdisk_dir}/" 2>/dev/null || true
  fi

  # Create symlink
  if [ ! -e "${original_dir}" ]; then
    ln -sf "${ramdisk_dir}" "${original_dir}"
    log_success "Linked ${original_dir} -> ${ramdisk_dir}"
  elif [ -L "${original_dir}" ]; then
    log_info "${original_dir} already linked"
  else
    log_error "${original_dir} exists but is not a symlink"
  fi
}

function enable_ram_optimization() {
  log_info "Enabling RAM-based optimization..."

  # Create RAM disk if needed
  create_ramdisk || return 1

  # Create backup directory
  mkdir -p "${BACKUP_DIR}"

  # Setup compiler caches
  log_info "Setting up compiler caches..."

  # ccache
  if command -v ccache >/dev/null 2>&1; then
    backup_and_link_cache "${CCACHE_DIR}" "${RAMDISK_PATH}/caches/ccache" "ccache"
    ccache --max-size=4G
    ccache --set-config=compression=true
    ccache --set-config=compression_level=6
  else
    log_info "ccache not found, skipping"
  fi

  # sccache
  if command -v sccache >/dev/null 2>&1; then
    backup_and_link_cache "${SCCACHE_DIR}" "${RAMDISK_PATH}/caches/sccache" "sccache"
    export SCCACHE_DIR="${RAMDISK_PATH}/caches/sccache"
    export SCCACHE_CACHE_SIZE="4G"
    sccache --stop-server >/dev/null 2>&1 || true
  else
    log_info "sccache not found, skipping"
  fi

  # Python cache
  log_info "Setting up Python cache..."
  backup_and_link_cache "${PYTHON_CACHE_DIR}" "${RAMDISK_PATH}/caches/pip" "pip"
  export PYTHONPYCACHEPREFIX="${RAMDISK_PATH}/caches/pip/__pycache__"

  # Node.js cache
  if command -v npm >/dev/null 2>&1 || command -v node >/dev/null 2>&1; then
    log_info "Setting up Node.js cache..."
    mkdir -p "${RAMDISK_PATH}/caches/node"
    backup_and_link_cache "${NODE_CACHE_DIR}" "${RAMDISK_PATH}/caches/node" "node"
    export npm_config_cache="${RAMDISK_PATH}/caches/node"
  else
    log_info "Node.js not found, skipping"
  fi

  # Rust cargo cache (if Rust is installed)
  if command -v cargo >/dev/null 2>&1; then
    log_info "Setting up Rust cargo cache..."

    # Cargo registry cache
    if [ -d "${CARGO_CACHE_DIR}" ]; then
      backup_and_link_cache "${CARGO_CACHE_DIR}" "${RAMDISK_PATH}/caches/cargo-registry" "cargo-registry"
    fi

    # Cargo git cache
    if [ -d "${CARGO_GIT_CACHE_DIR}" ]; then
      backup_and_link_cache "${CARGO_GIT_CACHE_DIR}" "${RAMDISK_PATH}/caches/cargo-git" "cargo-git"
    fi

    export CARGO_HOME="${RUST_CACHE_DIR}"
  else
    log_info "Rust not found, skipping"
  fi

  # Create environment file for persistence
  cat > "${PROJECT_ROOT}/.ram-optimization-env" <<EOF
# RAM Optimization Environment Variables
# Source this file to enable RAM-based caches:
#   source .ram-optimization-env

export CCACHE_DIR="${RAMDISK_PATH}/caches/ccache"
export SCCACHE_DIR="${RAMDISK_PATH}/caches/sccache"
export SCCACHE_CACHE_SIZE="4G"
export PYTHONPYCACHEPREFIX="${RAMDISK_PATH}/caches/pip/__pycache__"
export npm_config_cache="${RAMDISK_PATH}/caches/node"
export TMPDIR="${RAMDISK_PATH}/tmp"
export TMP="${RAMDISK_PATH}/tmp"
export TEMP="${RAMDISK_PATH}/tmp"
EOF

  log_success "RAM optimization enabled!"
  log_info "Source .ram-optimization-env to activate in new shells:"
  echo "  source .ram-optimization-env"
}

function disable_ram_optimization() {
  log_info "Disabling RAM-based optimization..."

  # Restore original caches from backup
  mkdir -p "${BACKUP_DIR}"

  for cache_name in ccache sccache pip node cargo-registry cargo-git; do
    local original_var="${cache_name^^}_DIR"
    local original_dir="${!original_var:-}"

    if [ -z "${original_dir}" ]; then
      continue
    fi

    if [ -L "${original_dir}" ]; then
      log_info "Restoring ${cache_name} cache..."
      rm -f "${original_dir}"

      # Restore from backup if available
      local backup_original="${BACKUP_DIR}/$(basename "${original_dir}").original"
      if [ -d "${backup_original}" ]; then
        mv "${backup_original}" "${original_dir}"
        log_success "Restored ${original_dir}"
      else
        mkdir -p "${original_dir}"
        log_info "Created empty ${original_dir}"
      fi
    fi
  done

  # Remove environment file
  if [ -f "${PROJECT_ROOT}/.ram-optimization-env" ]; then
    rm -f "${PROJECT_ROOT}/.ram-optimization-env"
    log_success "Removed .ram-optimization-env"
  fi

  log_success "RAM optimization disabled"
}

function show_status() {
  echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
  echo "  RAM Optimization Status"
  echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
  echo ""

  # Check RAM disk
  if [ -d "${RAMDISK_PATH}" ]; then
    log_success "RAM Disk: ✓ Mounted"
    df -h "${RAMDISK_PATH}" | tail -1 | awk '{printf "  Location: %s\n  Size: %s\n  Used: %s\n  Available: %s\n", $9, $2, $3, $4}'
    echo ""

    # Check cache directories
    log_info "Cache Directories:"
    [ -d "${RAMDISK_PATH}/caches/ccache" ] && du -sh "${RAMDISK_PATH}/caches/ccache" 2>/dev/null | awk '{printf "  ccache:    %s\n", $1}' || echo "  ccache:    (empty)"
    [ -d "${RAMDISK_PATH}/caches/sccache" ] && du -sh "${RAMDISK_PATH}/caches/sccache" 2>/dev/null | awk '{printf "  sccache:   %s\n", $1}' || echo "  sccache:   (empty)"
    [ -d "${RAMDISK_PATH}/caches/pip" ] && du -sh "${RAMDISK_PATH}/caches/pip" 2>/dev/null | awk '{printf "  pip:       %s\n", $1}' || echo "  pip:       (empty)"
    [ -d "${RAMDISK_PATH}/caches/node" ] && du -sh "${RAMDISK_PATH}/caches/node" 2>/dev/null | awk '{printf "  node:      %s\n", $1}' || echo "  node:      (empty)"
    [ -d "${RAMDISK_PATH}/caches/cargo-registry" ] && du -sh "${RAMDISK_PATH}/caches/cargo-registry" 2>/dev/null | awk '{printf "  cargo:     %s\n", $1}' || echo "  cargo:     (empty)"
  else
    log_info "RAM Disk: ✗ Not mounted"
    echo "  Run './scripts/setup_ram_optimization.sh enable' to create"
  fi

  echo ""
  log_info "Cache Links:"
  [ -L "${CCACHE_DIR}" ] && echo "  ✓ ${CCACHE_DIR} -> $(readlink "${CCACHE_DIR}")" || echo "  ✗ ${CCACHE_DIR} (not linked)"
  [ -L "${SCCACHE_DIR}" ] && echo "  ✓ ${SCCACHE_DIR} -> $(readlink "${SCCACHE_DIR}")" || echo "  ✗ ${SCCACHE_DIR} (not linked)"
  [ -L "${PYTHON_CACHE_DIR}" ] && echo "  ✓ ${PYTHON_CACHE_DIR} -> $(readlink "${PYTHON_CACHE_DIR}")" || echo "  ✗ ${PYTHON_CACHE_DIR} (not linked)"

  if [ -f "${PROJECT_ROOT}/.ram-optimization-env" ]; then
    echo ""
    log_success "Environment file exists: .ram-optimization-env"
    log_info "Source it to activate in new shells: source .ram-optimization-env"
  fi
}

function setup_redis_sccache() {
  if ! command -v sccache >/dev/null 2>&1; then
    log_error "sccache not found. Install with: brew install sccache"
    return 1
  fi

  if ! command -v redis-server >/dev/null 2>&1; then
    log_error "Redis not found. Install with: brew install redis"
    return 1
  fi

  log_info "Setting up Redis for distributed sccache..."

  # Check if Redis is running
  if ! redis-cli ping >/dev/null 2>&1; then
    log_info "Starting Redis server..."
    brew services start redis || redis-server --daemonize yes
    sleep 2
  fi

  # Configure sccache to use Redis
  export SCCACHE_REDIS="redis://localhost:6379"
  sccache --stop-server >/dev/null 2>&1 || true
  sccache --start-server

  log_success "sccache configured to use Redis backend"
  log_info "Redis URL: ${SCCACHE_REDIS}"
  log_info "To use across machines, configure Redis with: redis-cli CONFIG SET bind 0.0.0.0"
}

function usage() {
  cat <<EOF
Usage: $0 [command]

Commands:
  enable        - Enable RAM-based optimization (creates RAM disk and links caches)
  disable       - Disable RAM-based optimization (restores original caches)
  status        - Show current RAM optimization status
  redis         - Setup Redis backend for distributed sccache
  help          - Show this help message

Environment variables:
  RAMDISK_SIZE_GB - Size of RAM disk in GB (default: 12)

Examples:
  # Enable RAM optimization (creates 12GB RAM disk)
  $0 enable

  # Enable with custom size
  RAMDISK_SIZE_GB=16 $0 enable

  # Check status
  $0 status

  # Setup Redis for distributed caching
  $0 redis

  # Disable (restores original caches)
  $0 disable

Performance Benefits:
  - ccache/sccache: 100x+ faster cache access
  - Python pip cache: Faster package installs
  - Node.js cache: Faster npm/yarn operations
  - Rust cargo cache: Faster crate compilation
  - Build artifacts: Use ./scripts/build_ramdisk.sh

Notes:
  - RAM disk contents are lost on unmount or reboot
  - Original caches are backed up before linking
  - Source .ram-optimization-env in new shells to activate
EOF
}

# Main command handler
case "${1:-help}" in
enable)
  enable_ram_optimization
  ;;
disable)
  disable_ram_optimization
  ;;
status)
  show_status
  ;;
redis)
  setup_redis_sccache
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
