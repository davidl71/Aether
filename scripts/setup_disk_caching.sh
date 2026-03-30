#!/usr/bin/env bash
# setup_disk_caching.sh - Disk-based cache setup (migrated from RAM disk).
# Replaces setup_ram_optimization.sh for disk-based development.
# Usage: ./setup_disk_caching.sh [enable|disable|status]
# Low free-space / target-size checks: ./scripts/disk_pressure.sh check | just disk-check

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

# Disk-based cache directories
CACHE_ROOT="${HOME}/.cache/aether"
CARGO_TARGET_DIR="${CACHE_ROOT}/cargo-target"
CCACHE_DIR="${CACHE_ROOT}/ccache"
SCCACHE_DIR="${CACHE_ROOT}/sccache"
PIP_CACHE_DIR="${CACHE_ROOT}/pip"
NPM_CACHE_DIR="${CACHE_ROOT}/npm"
YARN_CACHE_DIR="${CACHE_ROOT}/yarn"
ENV_FILE="${PROJECT_ROOT}/.disk-caching-env"

log_info() { echo "ℹ️  $*"; }
log_ok() { echo "✓ $*"; }
log_err() { echo "✗ $*" >&2; }

usage() {
  cat <<EOF
Usage: $0 [command]

Commands:
  enable   - Create disk-based cache directories, write .disk-caching-env
  disable  - Remove environment file (caches left in place)
  status   - Show cache directory status
  migrate  - Migrate from RAM disk to disk-based (copy data if exists)

Environment:
  Cache directories are created under: ${CACHE_ROOT}
EOF
}

ensure_cache_dirs() {
  mkdir -p "${CACHE_ROOT}"/{ccache,sccache,cargo-target,pip,npm,yarn}
  log_ok "Cache directories created under ${CACHE_ROOT}"
}

write_env_file() {
  cat >"${ENV_FILE}" <<EOF
# Disk-based caching environment (migrated from RAM disk)
# Source this in your shell: source .disk-caching-env
export CCACHE_DIR="${CCACHE_DIR}"
export CCACHE_MAXSIZE="4G"
export SCCACHE_DIR="${SCCACHE_DIR}"
export SCCACHE_CACHE_SIZE="4G"
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR}"
export PYTHONPYCACHEPREFIX="${PIP_CACHE_DIR}/__pycache__"
export pip_cache_dir="${PIP_CACHE_DIR}"
export npm_config_cache="${NPM_CACHE_DIR}"
export YARN_CACHE_FOLDER="${YARN_CACHE_DIR}"
EOF
  log_ok "Wrote ${ENV_FILE}"
}

cmd_enable() {
  ensure_cache_dirs
  write_env_file
  echo ""
  log_ok "Disk-based caching enabled. Run: source .disk-caching-env"
  echo ""
  log_info "To use in new shells, add to ~/.zshrc or ~/.bashrc:"
  echo "  source ${ENV_FILE}"
}

cmd_migrate() {
  local ramdisk_path="/Volumes/IBBoxSpreadDev"
  local ramdisk_caches="${ramdisk_path}/caches"

  log_info "Migrating from RAM disk to disk-based storage..."
  ensure_cache_dirs

  # Copy data from RAM disk if it exists and has content
  if [ -d "${ramdisk_caches}" ]; then
    local ramdisk_size
    ramdisk_size=$(du -sm "${ramdisk_caches}" 2>/dev/null | cut -f1 || echo "0")
    if [ "${ramdisk_size}" -gt 10 ]; then
      log_info "Copying ${ramdisk_size}MB from RAM disk..."

      # Copy each cache directory
      for dir in ccache sccache cargo-target pip npm yarn; do
        if [ -d "${ramdisk_caches}/${dir}" ]; then
          log_info "Copying ${dir}..."
          rsync -a --ignore-errors "${ramdisk_caches}/${dir}/" "${CACHE_ROOT}/${dir}/" 2>/dev/null || true
        fi
      done
      log_ok "Data migration complete"
    else
      log_info "RAM disk is empty or nearly empty, skipping data copy"
    fi
  fi

  write_env_file
  echo ""
  log_ok "Migration complete. Run: source .disk-caching-env"
  echo ""
  log_info "You can now unmount the RAM disk if desired:"
  echo "  diskutil unmount /Volumes/IBBoxSpreadDev"
}

cmd_disable() {
  if [ -f "${ENV_FILE}" ]; then
    rm -f "${ENV_FILE}"
    log_ok "Removed ${ENV_FILE}"
  fi
  log_info "Cache directories left in place at ${CACHE_ROOT}"
}

cmd_status() {
  echo ""
  echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
  echo "  Disk-based caching status"
  echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
  echo ""

  status_dir() {
    local label="$1"
    local path="$2"
    if [ -d "${path}" ]; then
      local size
      size=$(du -sh "${path}" 2>/dev/null | cut -f1 || echo "?")
      echo "  ${label}: ${size} (${path})"
    else
      echo "  ${label}: (not created)"
    fi
  }

  status_dir "ccache" "${CCACHE_DIR}"
  status_dir "sccache" "${SCCACHE_DIR}"
  status_dir "cargo-target" "${CARGO_TARGET_DIR}"
  status_dir "pip" "${PIP_CACHE_DIR}"
  status_dir "npm" "${NPM_CACHE_DIR}"
  status_dir "yarn" "${YARN_CACHE_DIR}"
  echo ""

  if [ -f "${ENV_FILE}" ]; then
    log_ok "Env file: ${ENV_FILE}"
  else
    log_info "Env file: not present (run: $0 enable)"
  fi
  echo ""

  # Show RAM disk status
  if [ -d "/Volumes/IBBoxSpreadDev" ] && df "/Volumes/IBBoxSpreadDev" >/dev/null 2>&1; then
    log_info "RAM disk still mounted: /Volumes/IBBoxSpreadDev"
    df -h "/Volumes/IBBoxSpreadDev" | tail -1 | awk '{printf "  Size: %s  Used: %s  Avail: %s\n", $2, $3, $4}'
  else
    log_ok "RAM disk: not mounted"
  fi
  echo ""
}

case "${1:-}" in
enable) cmd_enable ;;
migrate) cmd_migrate ;;
disable) cmd_disable ;;
status) cmd_status ;;
-h | --help) usage ;;
*)
  log_err "Unknown command: ${1:-}"
  usage
  exit 1
  ;;
esac
