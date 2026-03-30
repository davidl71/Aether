#!/usr/bin/env bash
# disk_pressure.sh — Low free-space detection + safe Rust target prune (dry-run default).
#
# Free-space thresholds (measured on the filesystem containing each path, via df -Pk):
#   AETHER_DISK_FREE_WARN_MB   default 5120  (warn if available < this)
#   AETHER_DISK_FREE_CRIT_MB   default 1024  (critical if available < this)
#
# Optional target-dir size signal (agents/backend/target or CARGO_TARGET_DIR):
#   AETHER_RUST_TARGET_WARN_MB  default 0 (disabled). e.g. 10240 warns if tree > ~10GiB.
#
# Paths checked: repo root (.) and, if set, CARGO_TARGET_DIR (may differ from in-tree target).
#
# Exit codes (check): 0 = ok, 1 = warn, 2 = critical (lowest free among checked paths).
#
# Prune (rust workspace cache only): default prints plan + sizes; never deletes without --apply.
#   ./scripts/disk_pressure.sh prune-rust-target           # dry-run
#   ./scripts/disk_pressure.sh prune-rust-target --apply    # runs: cd agents/backend && cargo clean
#
# Git hooks: optional non-blocking pre-push snippet (do not block pushes by default):
#   ./scripts/disk_pressure.sh check || true
#
# Related: scripts/setup_disk_caching.sh, just sweep-dry / clean-rust.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
BACKEND_DIR="${PROJECT_ROOT}/agents/backend"

# Defaults (MiB)
: "${AETHER_DISK_FREE_WARN_MB:=5120}"
: "${AETHER_DISK_FREE_CRIT_MB:=1024}"
: "${AETHER_RUST_TARGET_WARN_MB:=0}"

log() { printf '%s\n' "$*"; }
log_err() { printf '%s\n' "$*" >&2; }

usage() {
  cat <<'EOF'
Usage: disk_pressure.sh check [--json] | prune-rust-target [--apply]

  check [--json]     Report free space for repo root and CARGO_TARGET_DIR (if set).
                     With --json: one JSON object on stdout (schema aether.disk_pressure/v1);
                     human lines go to stderr only when not using --json.
  prune-rust-target  Dry-run: show target dir size and exact cargo clean command.
                     With --apply: run cargo clean from agents/backend (uses CARGO_TARGET_DIR if set).

Environment (see header in script for defaults):
  AETHER_DISK_FREE_WARN_MB  AETHER_DISK_FREE_CRIT_MB  AETHER_RUST_TARGET_WARN_MB
EOF
}

# Return free KiB on filesystem for path (POSIX df -Pk, awk last field parsing).
free_kib_for_path() {
  local path="$1"
  local p
  p=$(cd "${path}" 2>/dev/null && pwd -P) || return 1
  df -Pk "${p}" 2>/dev/null | awk 'END {print $4}'
}

human_free_kib() {
  awk -v k="$1" 'BEGIN {
    if (k >= 1073741824) printf "%.2f TiB", k/1073741824
    else if (k >= 1048576) printf "%.2f GiB", k/1048576
    else printf "%d MiB", int(k/1024)
  }'
}

# Prints diagnostic lines to stderr (unless json_mode); echoes only severity 0|1|2 to stdout for capture.
# Globals: json_mode (0|1), json_parts accumulates mount lines for final JSON.
# $1 label $2 path
check_one_mount() {
  local label="$1"
  local path="$2"
  local free_k avail_mb status human sev

  if [ ! -e "${path}" ]; then
    if [ "${json_mode}" -eq 0 ]; then
      printf '%s\n' "  ${label}: path missing (skip): ${path}" >&2
    fi
    _check_one_mount_sev=0
    return 0
  fi

  free_k=$(free_kib_for_path "${path}") || {
    if [ "${json_mode}" -eq 0 ]; then
      log_err "  ${label}: df failed for ${path}"
    fi
    _check_one_mount_sev=0
    return 0
  }
  avail_mb=$((free_k / 1024))
  human=$(human_free_kib "${free_k}")

  status=ok
  sev=0
  if [ "${avail_mb}" -lt "${AETHER_DISK_FREE_CRIT_MB}" ]; then
    status=critical
    sev=2
  elif [ "${avail_mb}" -lt "${AETHER_DISK_FREE_WARN_MB}" ]; then
    status=warn
    sev=1
  fi

  if [ "${json_mode}" -eq 1 ]; then
    json_parts="${json_parts}{\"label\":\"${label}\",\"path\":\"${path}\",\"free_mib\":${avail_mb},\"free_human\":\"${human}\",\"status\":\"${status}\"},"
  else
    printf '%s\n' "  ${label}: ${avail_mb} MiB free (~${human}) status=${status} path=${path}" >&2
  fi
  _check_one_mount_sev=${sev}
}

cmd_check() {
  local worst=0
  local r t
  json_mode=0
  json_parts=""
  if [ "${1:-}" = "--json" ]; then
    json_mode=1
  fi

  if [ "${json_mode}" -eq 0 ]; then
    log "Disk pressure check (warn below ${AETHER_DISK_FREE_WARN_MB} MiB, critical below ${AETHER_DISK_FREE_CRIT_MB} MiB free)"
  fi

  check_one_mount "project_root" "${PROJECT_ROOT}"
  r=${_check_one_mount_sev}
  worst=$((r > worst ? r : worst))

  if [ -n "${CARGO_TARGET_DIR:-}" ]; then
    check_one_mount "CARGO_TARGET_DIR" "${CARGO_TARGET_DIR}"
    t=${_check_one_mount_sev}
    worst=$((t > worst ? t : worst))
  fi

  local rust_target_warn=0
  local rust_target_json=""
  # Optional: large target tree warning (size on disk, not free space)
  if [ "${AETHER_RUST_TARGET_WARN_MB}" -gt 0 ] 2>/dev/null; then
    local td
    td="${CARGO_TARGET_DIR:-${BACKEND_DIR}/target}"
    if [ -d "${td}" ]; then
      local sz_k sz_mb
      sz_k=$(du -sk "${td}" 2>/dev/null | awk '{print $1}') || sz_k=0
      sz_mb=$((sz_k / 1024))
      if [ "${json_mode}" -eq 0 ]; then
        log "  rust_target_size: ${sz_mb} MiB (threshold ${AETHER_RUST_TARGET_WARN_MB} MiB) dir=${td}"
      fi
      if [ "${sz_mb}" -ge "${AETHER_RUST_TARGET_WARN_MB}" ]; then
        rust_target_warn=1
        if [ "${json_mode}" -eq 0 ]; then
          log "  rust_target_size: WARN — target tree exceeds AETHER_RUST_TARGET_WARN_MB (consider: just sweep-dry / just disk-prune-rust-dry)"
        fi
        worst=$((worst > 1 ? worst : 1))
      fi
      rust_target_json="\"rust_target_mib\":${sz_mb},\"rust_target_path\":\"${td}\",\"rust_target_warn\":${rust_target_warn}"
    fi
  fi

  if [ "${json_mode}" -eq 1 ]; then
    local mounts_json sev_name
    mounts_json="[${json_parts%,}]"
    case "${worst}" in
      0) sev_name="ok" ;;
      1) sev_name="warn" ;;
      *) sev_name="critical" ;;
    esac
    printf '{"schema":"aether.disk_pressure/v1","severity":%s,"severity_name":"%s","warn_free_mib":%s,"crit_free_mib":%s,"rust_target_threshold_mib":%s' \
      "${worst}" "${sev_name}" "${AETHER_DISK_FREE_WARN_MB}" "${AETHER_DISK_FREE_CRIT_MB}" "${AETHER_RUST_TARGET_WARN_MB}"
    if [ -n "${rust_target_json}" ]; then
      printf ',%s' "${rust_target_json}"
    fi
    printf ',"mounts":%s}\n' "${mounts_json}"
  fi

  exit "${worst}"
}

resolve_target_display() {
  if [ -n "${CARGO_TARGET_DIR:-}" ]; then
    printf '%s' "${CARGO_TARGET_DIR}"
  else
    printf '%s' "${BACKEND_DIR}/target"
  fi
}

cmd_prune_rust_target() {
  local apply=0
  if [ "${1:-}" = "--apply" ]; then
    apply=1
  fi

  local td
  td=$(resolve_target_display)

  log "Rust workspace target prune"
  if [ -d "${td}" ]; then
    local sz_k sz_mb
    sz_k=$(du -sk "${td}" 2>/dev/null | awk '{print $1}') || sz_k=0
    sz_mb=$((sz_k / 1024))
    log "  Current target dir size: ~${sz_mb} MiB"
    log "  Path: ${td}"
  else
    log "  Target dir not present: ${td} (nothing to clean)"
    return 0
  fi

  log ""
  log "  Exact command (from repo root):"
  log "    (cd \"${BACKEND_DIR}\" && cargo clean)"
  log ""
  if [ "${apply}" -eq 1 ]; then
    log "  Applying cargo clean ..."
    (cd "${BACKEND_DIR}" && cargo clean)
    log "  Done."
  else
    log "[DRY-RUN] No files removed. To run cargo clean:"
    log "    ./scripts/disk_pressure.sh prune-rust-target --apply"
    log "    # or: just disk-prune-rust-apply"
  fi
}

main() {
  case "${1:-}" in
    check)
      shift
      cmd_check "$@"
      ;;
    prune-rust-target) cmd_prune_rust_target "${2:-}" ;;
    -h | --help) usage ;;
    *)
      log_err "Unknown command: ${1:-}"
      usage
      exit 1
      ;;
  esac
}

main "$@"
