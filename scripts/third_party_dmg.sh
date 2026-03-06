#!/usr/bin/env bash
# third_party_dmg.sh - Create and use a read-only compressed DMG for third-party trees (macOS).
# Reduces disk read volume and avoids writes to vendor trees during builds.
# Usage: ./third_party_dmg.sh [create|mount|unmount|status]
# Env: USE_THIRD_PARTY_DMG=1 to prefer DMG in ensure_third_party / build (mount if needed).
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
NATIVE_TP="${PROJECT_ROOT}/native/third_party"
ORIG_DIR="${NATIVE_TP}/.orig"
DMG_PATH="${PROJECT_ROOT}/.dmg/ThirdParty.dmg"
VOLUME_NAME="IBBoxSpreadThirdParty"
MOUNT_POINT="/Volumes/${VOLUME_NAME}"

usage() {
  cat <<EOF
Usage: $0 [create|mount|unmount|status]

  create  - Pack tws-api and IntelRDFPMathLib* from native/third_party into a
            read-only compressed DMG at .dmg/ThirdParty.dmg (macOS only).
  mount   - Attach DMG and symlink native/third_party/tws-api and Intel* to
            the mounted volume (moves existing dirs to .orig first).
  unmount - Restore .orig dirs, remove symlinks, detach DMG.
  status  - Show whether DMG exists, is mounted, and symlinks active.

Environment:
  USE_THIRD_PARTY_DMG=1  Build scripts and ensure_third_party will mount
                         the DMG when available (see ensure_third_party.sh).
EOF
}

is_macos() {
  [[ "$(uname -s)" == "Darwin" ]]
}

dmg_exists() {
  [[ -f "${DMG_PATH}" ]]
}

is_mounted() {
  [[ -d "${MOUNT_POINT}" ]] && df "${MOUNT_POINT}" >/dev/null 2>&1
}

is_active() {
  [[ -L "${NATIVE_TP}/tws-api" ]] && [[ "$(readlink "${NATIVE_TP}/tws-api")" == "${MOUNT_POINT}/tws-api" ]]
}

# Copy only the trees needed for the native build (no cache, no nautilus).
prepare_staging() {
  local staging="$1"
  rm -rf "${staging}"
  mkdir -p "${staging}"
  local need_one=false
  if [[ -d "${NATIVE_TP}/tws-api" ]]; then
    cp -a "${NATIVE_TP}/tws-api" "${staging}/"
    need_one=true
  fi
  if [[ -d "${NATIVE_TP}/IntelRDFPMathLib20U4" ]]; then
    cp -a "${NATIVE_TP}/IntelRDFPMathLib20U4" "${staging}/"
    need_one=true
  fi
  if [[ -d "${NATIVE_TP}/IntelRDFPMathLib20U2" ]]; then
    cp -a "${NATIVE_TP}/IntelRDFPMathLib20U2" "${staging}/"
    need_one=true
  fi
  if [[ "$need_one" != true ]]; then
    echo "No tws-api or IntelRDFPMathLib* under native/third_party. Run ./scripts/fetch_third_party.sh first." >&2
    return 1
  fi
  return 0
}

cmd_create() {
  if ! is_macos; then
    echo "DMG creation is supported only on macOS." >&2
    return 1
  fi
  local staging
  staging="$(mktemp -d "${TMPDIR:-/tmp}/ib_third_party_dmg.XXXXXX")"
  if ! prepare_staging "${staging}"; then
    rm -rf "${staging}"
    return 1
  fi
  mkdir -p "$(dirname "${DMG_PATH}")"
  # Read-only compressed (UDZO); volume name so mount point is /Volumes/IBBoxSpreadThirdParty
  hdiutil create -srcfolder "${staging}" -volname "${VOLUME_NAME}" -ov -format UDZO \
    -imagekey zlib-level=9 "${DMG_PATH}"
  rm -rf "${staging}"
  echo "Created: ${DMG_PATH}"
}

move_orig() {
  mkdir -p "${ORIG_DIR}"
  for name in tws-api IntelRDFPMathLib20U4 IntelRDFPMathLib20U2; do
    if [[ -d "${NATIVE_TP}/${name}" ]] && [[ ! -L "${NATIVE_TP}/${name}" ]]; then
      rm -rf "${ORIG_DIR:?}/${name}"
      mv "${NATIVE_TP}/${name}" "${ORIG_DIR}/"
    fi
  done
}

restore_orig() {
  for name in tws-api IntelRDFPMathLib20U4 IntelRDFPMathLib20U2; do
    if [[ -L "${NATIVE_TP}/${name}" ]]; then
      rm -f "${NATIVE_TP}/${name}"
    fi
    if [[ -d "${ORIG_DIR}/${name}" ]]; then
      mv "${ORIG_DIR}/${name}" "${NATIVE_TP}/"
    fi
  done
  rmdir "${ORIG_DIR}" 2>/dev/null || true
}

cmd_mount() {
  if ! is_macos; then
    echo "DMG mount is supported only on macOS." >&2
    return 1
  fi
  if ! dmg_exists; then
    echo "DMG not found: ${DMG_PATH}. Run: $0 create" >&2
    return 1
  fi
  if is_active; then
    echo "Third-party DMG already active (symlinks in place)."
    return 0
  fi
  if ! is_mounted; then
    hdiutil attach "${DMG_PATH}" -readonly -nobrowse || {
      echo "Failed to mount ${DMG_PATH}" >&2
      return 1
    }
  fi
  move_orig
  for name in tws-api IntelRDFPMathLib20U4 IntelRDFPMathLib20U2; do
    if [[ -d "${MOUNT_POINT}/${name}" ]]; then
      ln -sfn "${MOUNT_POINT}/${name}" "${NATIVE_TP}/${name}"
    fi
  done
  echo "Mounted and active: ${MOUNT_POINT} -> native/third_party (symlinks)"
}

cmd_unmount() {
  if ! is_macos; then
    echo "DMG unmount is supported only on macOS." >&2
    return 1
  fi
  if is_active; then
    restore_orig
  fi
  if is_mounted; then
    hdiutil detach "${MOUNT_POINT}" || true
    echo "Unmounted: ${MOUNT_POINT}"
  else
    echo "DMG not mounted."
  fi
}

cmd_status() {
  echo "DMG path:    ${DMG_PATH}"
  echo "DMG exists:  $(dmg_exists && echo yes || echo no)"
  echo "Mounted:     $(is_mounted && echo "yes (${MOUNT_POINT})" || echo no)"
  echo "Symlinks:    $(is_active && echo "active" || echo "inactive")"
  if [[ -d "${ORIG_DIR}" ]]; then
    echo "Backup:      ${ORIG_DIR} (present)"
  fi
}

case "${1:-}" in
  create)  cmd_create ;;
  mount)   cmd_mount ;;
  unmount) cmd_unmount ;;
  status)  cmd_status ;;
  -h|--help) usage ;;
  *)
    echo "Usage: $0 [create|mount|unmount|status]" >&2
    exit 1
    ;;
esac
