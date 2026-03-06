#!/usr/bin/env bash
# with_nix.sh - Run the current script inside the Nix dev shell when USE_NIX=1
#
# Source this at the top of a script (after setting SCRIPT_DIR and PROJECT_ROOT), then:
#   run_with_nix_if_requested "$@"
#
# When USE_NIX=1 and nix is available and flake.nix exists, re-execs the script
# inside `nix develop` so cmake, ninja, ctest, uv, etc. come from the flake.
# Set NIX_DEV_SHELL=1 to skip (used internally to avoid re-entry).
#
# Usage in a script:
#   SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
#   PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
#   . "${SCRIPT_DIR}/with_nix.sh"
#   run_with_nix_if_requested "$@"
#   # rest of script runs with Nix toolchain when USE_NIX=1

run_with_nix_if_requested() {
  [[ "${USE_NIX:-0}" =~ ^(1|true|yes)$ ]] || return 0
  [[ -n "${NIX_DEV_SHELL:-}" ]] && return 0
  if ! command -v nix >/dev/null 2>&1; then
    echo "USE_NIX=1 but nix not found in PATH; running without Nix." >&2
    return 0
  fi
  if [[ ! -f "${PROJECT_ROOT:-.}/flake.nix" ]]; then
    echo "USE_NIX=1 but flake.nix not found; running without Nix." >&2
    return 0
  fi
  export NIX_DEV_SHELL=1
  exec nix develop "${PROJECT_ROOT}" \
    --extra-experimental-features 'nix-command flakes' \
    --command env NIX_DEV_SHELL=1 "$0" "$@"
}
