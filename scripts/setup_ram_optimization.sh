#!/usr/bin/env bash
# setup_ram_optimization.sh - RAM-based cache and temp optimization for development.
# Links ccache, sccache, pip, cargo, node caches and TMPDIR to a RAM disk.
# Usage: ./setup_ram_optimization.sh [enable|disable|status|redis]
# Used by workspace_ram_disk_manager.sh. See docs/RAM_OPTIMIZATION_GUIDE.md.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

# Cache RAM disk (matches workspace_ram_disk_manager.sh)
CACHE_RAMDISK_NAME="${CACHE_RAMDISK_NAME:-IBBoxSpreadDev}"
CACHE_RAMDISK_PATH="/Volumes/${CACHE_RAMDISK_NAME}"
CACHE_RAMDISK_SIZE_GB="${CACHE_RAMDISK_SIZE_GB:-12}"
CACHES_ROOT="${CACHE_RAMDISK_PATH}/caches"
ENV_FILE="${PROJECT_ROOT}/.ram-optimization-env"

log_info() { echo "ℹ️  $*"; }
log_ok() { echo "✓ $*"; }
log_err() { echo "✗ $*" >&2; }

usage() {
  cat <<EOF
Usage: $0 [command]

Commands:
  enable   - Create cache RAM disk, link caches + project .venv/node_modules, write .ram-optimization-env
  disable  - Remove symlinks and restore local cache dirs (RAM disk left mounted)
  status   - Show RAM disk and cache link status
  redis    - Configure sccache Redis backend and write to .ram-optimization-env

When enabled: Cargo target dir, project .venv, and node_modules (root + web) use ramdisk when absent.
After enable, run: source .ram-optimization-env

Environment:
  CACHE_RAMDISK_NAME   Volume name (default: IBBoxSpreadDev)
  CACHE_RAMDISK_SIZE_GB Size in GB (default: 12)
EOF
}

is_macos() {
  [ "$(uname -s)" = "Darwin" ]
}

create_cache_ramdisk() {
  if ! is_macos; then
    log_err "Cache RAM disk is supported only on macOS."
    return 1
  fi
  if [ -d "${CACHE_RAMDISK_PATH}" ] && df "${CACHE_RAMDISK_PATH}" >/dev/null 2>&1; then
    log_info "Cache RAM disk already mounted: ${CACHE_RAMDISK_PATH}"
    return 0
  fi
  local size_sectors
  size_sectors=$((CACHE_RAMDISK_SIZE_GB * 1024 * 2048))
  log_info "Creating ${CACHE_RAMDISK_SIZE_GB}GB cache RAM disk: ${CACHE_RAMDISK_PATH}"
  local dev
  dev=$(hdiutil attach -nomount "ram://${size_sectors}" | head -1 | awk '{print $1}')
  dev="${dev%%[[:space:]]*}"
  [ -z "${dev}" ] && { log_err "hdiutil attach failed"; return 1; }
  sleep 1
  diskutil eraseVolume HFS+ "${CACHE_RAMDISK_NAME}" "${dev}"
  log_ok "Cache RAM disk created: ${CACHE_RAMDISK_PATH}"
}

ensure_cache_dirs() {
  mkdir -p "${CACHES_ROOT}"/{ccache,sccache,pip,node,cargo-registry,cargo-git,cargo-target,tmp}
  mkdir -p "${CACHES_ROOT}/pip/__pycache__"
  # Optional project dirs on ramdisk (populated on first use)
  mkdir -p "${CACHES_ROOT}/venv"
  mkdir -p "${CACHES_ROOT}/node-modules-root"
  mkdir -p "${CACHES_ROOT}/node-modules-web"
}

# Move existing dir to ramdisk and symlink; or create dir on ramdisk and symlink.
link_cache() {
  local name="$1"
  local home_path="$2"
  local ramdisk_subdir="$3"
  local ramdisk_path="${CACHES_ROOT}/${ramdisk_subdir}"

  if [ -L "${home_path}" ]; then
    local dest
    dest=$(readlink "${home_path}" 2>/dev/null || true)
    if [ -n "${dest}" ] && [ -d "${ramdisk_path}" ] && [[ "${dest}" == *"${ramdisk_subdir}"* ]]; then
      log_info "${name}: already linked"
      return 0
    fi
    rm -f "${home_path}"
  fi

  if [ -d "${home_path}" ] && [ ! -L "${home_path}" ]; then
    log_info "${name}: moving existing data to RAM disk..."
    mkdir -p "${ramdisk_path}"
    rsync -a --ignore-errors "${home_path}/" "${ramdisk_path}/" 2>/dev/null || true
    rm -rf "${home_path}"
  fi

  if [ ! -e "${home_path}" ]; then
    mkdir -p "$(dirname "${home_path}")" 2>/dev/null || true
    ln -sfn "${ramdisk_path}" "${home_path}"
    log_ok "${name}: linked -> ${ramdisk_path}"
  fi
}

enable_caches() {
  ensure_cache_dirs

  # ccache
  if command -v ccache >/dev/null 2>&1; then
    link_cache "ccache" "${HOME}/.ccache" "ccache"
    ccache --max-size=4G 2>/dev/null || true
  else
    link_cache "ccache" "${HOME}/.ccache" "ccache"
  fi

  # sccache
  link_cache "sccache" "${HOME}/.sccache" "sccache"

  # pip
  mkdir -p "${HOME}/.cache"
  link_cache "pip" "${HOME}/.cache/pip" "pip"

  # Cargo registry and git
  if [ -d "${HOME}/.cargo" ] || mkdir -p "${HOME}/.cargo"; then
    link_cache "cargo/registry" "${HOME}/.cargo/registry" "cargo-registry"
    link_cache "cargo/git" "${HOME}/.cargo/git" "cargo-git"
  fi

  # Node: we don't symlink ~/.npm; we set env vars so npm/yarn use ramdisk
  mkdir -p "${CACHES_ROOT}/node"
}

# Optional: put this project's .venv and node_modules on ramdisk (only when absent).
enable_project_dirs() {
  # Root .venv -> ramdisk (so uv sync / python -m venv populates ramdisk)
  if [ ! -e "${PROJECT_ROOT}/.venv" ]; then
    ln -sfn "${CACHES_ROOT}/venv" "${PROJECT_ROOT}/.venv"
    log_ok "project .venv: linked -> ${CACHES_ROOT}/venv (run uv sync or python -m venv .venv)"
  else
    log_info "project .venv: already exists, not linked"
  fi
  # Root node_modules
  if [ ! -e "${PROJECT_ROOT}/node_modules" ]; then
    ln -sfn "${CACHES_ROOT}/node-modules-root" "${PROJECT_ROOT}/node_modules"
    log_ok "project node_modules: linked -> ${CACHES_ROOT}/node-modules-root"
  else
    log_info "project node_modules: already exists, not linked"
  fi
  # web/node_modules
  if [ ! -e "${PROJECT_ROOT}/web/node_modules" ]; then
    mkdir -p "${PROJECT_ROOT}/web"
    ln -sfn "${CACHES_ROOT}/node-modules-web" "${PROJECT_ROOT}/web/node_modules"
    log_ok "web/node_modules: linked -> ${CACHES_ROOT}/node-modules-web"
  else
    log_info "web/node_modules: already exists, not linked"
  fi
}

write_env_file() {
  cat > "${ENV_FILE}" <<EOF
# RAM optimization environment (generated by setup_ram_optimization.sh)
# Source this in your shell: source .ram-optimization-env
export CCACHE_DIR="${CACHES_ROOT}/ccache"
export CCACHE_MAXSIZE="4G"
export SCCACHE_DIR="${CACHES_ROOT}/sccache"
export SCCACHE_CACHE_SIZE="4G"
export CARGO_TARGET_DIR="${CACHES_ROOT}/cargo-target"
export PYTHONPYCACHEPREFIX="${CACHES_ROOT}/pip/__pycache__"
export pip_cache_dir="${CACHES_ROOT}/pip"
export npm_config_cache="${CACHES_ROOT}/node"
export YARN_CACHE_FOLDER="${CACHES_ROOT}/node"
export TMPDIR="${CACHES_ROOT}/tmp"
export TMP="${CACHES_ROOT}/tmp"
export TEMP="${CACHES_ROOT}/tmp"
EOF
  if [ -n "${SCCACHE_REDIS:-}" ]; then
    echo "export SCCACHE_REDIS=\"${SCCACHE_REDIS}\"" >> "${ENV_FILE}"
  fi
  log_ok "Wrote ${ENV_FILE}"
}

cmd_enable() {
  if ! is_macos; then
    log_err "RAM optimization is supported only on macOS."
    exit 1
  fi
  create_cache_ramdisk
  enable_caches
  enable_project_dirs
  write_env_file
  echo ""
  log_ok "RAM optimization enabled. Run: source .ram-optimization-env"
}

remove_link_and_restore_dir() {
  local name="$1"
  local home_path="$2"
  if [ -L "${home_path}" ]; then
    rm -f "${home_path}"
    mkdir -p "${home_path}"
    log_ok "${name}: symlink removed, local dir recreated"
  elif [ ! -d "${home_path}" ]; then
    mkdir -p "${home_path}"
    log_info "${name}: created local dir"
  fi
}

cmd_disable() {
  log_info "Disabling RAM optimization (removing symlinks)..."
  remove_link_and_restore_dir "ccache" "${HOME}/.ccache"
  remove_link_and_restore_dir "sccache" "${HOME}/.sccache"
  mkdir -p "${HOME}/.cache"
  remove_link_and_restore_dir "pip" "${HOME}/.cache/pip"
  if [ -d "${HOME}/.cargo" ]; then
    remove_link_and_restore_dir "cargo/registry" "${HOME}/.cargo/registry"
    remove_link_and_restore_dir "cargo/git" "${HOME}/.cargo/git"
  fi
  # Project dirs: remove symlinks only (do not create dirs; next enable or npm/uv will recreate)
  if [ -L "${PROJECT_ROOT}/.venv" ]; then
    rm -f "${PROJECT_ROOT}/.venv"
    log_ok "project .venv: symlink removed"
  fi
  if [ -L "${PROJECT_ROOT}/node_modules" ]; then
    rm -f "${PROJECT_ROOT}/node_modules"
    log_ok "project node_modules: symlink removed"
  fi
  if [ -L "${PROJECT_ROOT}/web/node_modules" ]; then
    rm -f "${PROJECT_ROOT}/web/node_modules"
    log_ok "web/node_modules: symlink removed"
  fi
  if [ -f "${ENV_FILE}" ]; then
    rm -f "${ENV_FILE}"
    log_ok "Removed ${ENV_FILE}"
  fi
  log_ok "RAM optimization disabled. Cache RAM disk left mounted at ${CACHE_RAMDISK_PATH}"
}

cmd_status() {
  echo ""
  echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
  echo "  RAM optimization status"
  echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
  echo ""

  if is_macos && [ -d "${CACHE_RAMDISK_PATH}" ] && df "${CACHE_RAMDISK_PATH}" >/dev/null 2>&1; then
    log_ok "Cache RAM disk: ${CACHE_RAMDISK_PATH}"
    df -h "${CACHE_RAMDISK_PATH}" | tail -1 | awk '{printf "  Size: %s  Used: %s  Avail: %s\n", $2, $3, $4}'
  else
    log_info "Cache RAM disk: not mounted"
  fi
  echo ""

  status_pair() {
    local label="$1"
    local path="$2"
    if [ -L "${path}" ]; then
      local dest
      dest=$(readlink "${path}" 2>/dev/null || echo "?")
      echo "  ${label}: linked -> ${dest}"
    elif [ -d "${path}" ]; then
      echo "  ${label}: local dir (not linked)"
    else
      echo "  ${label}: (absent)"
    fi
  }
  status_pair "ccache" "${HOME}/.ccache"
  status_pair "sccache" "${HOME}/.sccache"
  status_pair "pip" "${HOME}/.cache/pip"
  status_pair "cargo/registry" "${HOME}/.cargo/registry"
  status_pair "cargo/git" "${HOME}/.cargo/git"
  echo ""
  echo "  Non-C++ project dirs (when enabled, build/install on ramdisk):"
  status_pair "  CARGO_TARGET_DIR" "${CACHES_ROOT}/cargo-target"
  status_pair "  project .venv" "${PROJECT_ROOT}/.venv"
  status_pair "  project node_modules" "${PROJECT_ROOT}/node_modules"
  status_pair "  web/node_modules" "${PROJECT_ROOT}/web/node_modules"
  echo ""

  if [ -f "${ENV_FILE}" ]; then
    log_ok "Env file: ${ENV_FILE}"
  else
    log_info "Env file: not present"
  fi
  echo ""
}

cmd_redis() {
  local redis_url="${SCCACHE_REDIS:-redis://localhost:6379}"
  if [ -f "${ENV_FILE}" ]; then
    if grep -q 'SCCACHE_REDIS' "${ENV_FILE}"; then
      sed -i.bak "s|^export SCCACHE_REDIS=.*|export SCCACHE_REDIS=\"${redis_url}\"|" "${ENV_FILE}"
    else
      echo "export SCCACHE_REDIS=\"${redis_url}\"" >> "${ENV_FILE}"
    fi
    rm -f "${ENV_FILE}.bak" 2>/dev/null || true
  else
    mkdir -p "${PROJECT_ROOT}"
    echo "export SCCACHE_REDIS=\"${redis_url}\"" >> "${ENV_FILE}"
  fi
  log_ok "SCCACHE_REDIS=${redis_url} written to ${ENV_FILE}"
  echo ""
  echo "Restart sccache to use Redis:"
  echo "  sccache --stop-server"
  echo "  sccache --start-server"
  echo ""
  echo "Start Redis if needed: brew services start redis"
}

case "${1:-}" in
  enable)  cmd_enable ;;
  disable) cmd_disable ;;
  status)  cmd_status ;;
  redis)   cmd_redis ;;
  -h|--help) usage ;;
  *)
    log_err "Unknown command: ${1:-}"
    usage
    exit 1
    ;;
esac
