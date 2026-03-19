#!/usr/bin/env bash
# build_universal.sh - x86_64 binary builder for IBKR Box Spread Generator
# Usage: ./build_universal.sh [build|clean|test|install]
# See README "Build" section for prerequisites; script assumes macOS with cmake + Xcode tools.
# Note: Currently builds x86_64 only. Universal binary support is in the wishlist (docs/WISHLIST.md).

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=scripts/include/logging.sh
. "${SCRIPT_DIR}/include/logging.sh"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
# shellcheck source=scripts/with_nix.sh
. "${SCRIPT_DIR}/with_nix.sh"
run_with_nix_if_requested "$@"
# Auto-detect architecture
ARCH=$(uname -m)
if [ "$ARCH" = "arm64" ] || [ "$ARCH" = "aarch64" ]; then
  DEFAULT_PRESET="macos-arm64-release"
else
  DEFAULT_PRESET="macos-x86_64-release"
fi
PRESET="${CMAKE_PRESET:-${DEFAULT_PRESET}}"

if ! command -v cmake >/dev/null 2>&1; then
  log_error "cmake not found. Install CMake before running this wrapper."
  exit 1
fi

cd "${PROJECT_ROOT}"

# Ensure third-party deps exist before configure/build
# shellcheck source=scripts/include/ensure_third_party.sh
. "${SCRIPT_DIR}/include/ensure_third_party.sh"
ensure_third_party

# Use all cores for Ninja when not set (see docs/BUILD_PARALLELIZATION_AND_MODULARITY.md)
# shellcheck source=scripts/include/set_parallel_level.sh
. "${SCRIPT_DIR}/include/set_parallel_level.sh"

log_note "Using CMake preset '${PRESET}'. Override with CMAKE_PRESET env var."

ensure_configured() {
  log_info "Configuring preset ${PRESET}..."
  cmake --preset "${PRESET}" >/dev/null
}

if [ "$#" -gt 0 ]; then
  command="$1"
  shift
else
  command="build"
fi

case "${command}" in
-h | --help)
  cat <<EOF
Usage: $0 [build|clean|test|install]

Environment:
  CMAKE_PRESET   Override the CMake preset (default: ${DEFAULT_PRESET})

This script is a thin wrapper around \`cmake --preset\`, \`cmake --build\`,
and \`ctest --preset\`.
EOF
  ;;
clean)
  ensure_configured
  cmake --build --preset "${PRESET}" --target clean "$@"
  ;;
test)
  ensure_configured
  ctest --preset "${PRESET}" "$@"
  ;;
install)
  ensure_configured
  cmake --install --preset "${PRESET}" "$@"
  ;;
build | *)
  ensure_configured
  cmake --build --preset "${PRESET}" "$@"
  ;;
esac
