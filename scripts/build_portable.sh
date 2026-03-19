#!/usr/bin/env bash
# build_portable.sh - Portable build wrapper for macOS (Intel, ARM) and Linux
#
# Usage: ./scripts/build_portable.sh [build|configure|clean|test|install] [--debug|--release]
#
# Detects OS and architecture and uses the matching CMake preset:
#   macOS arm64/aarch64  -> macos-arm64-{debug|release}
#   macOS x86_64         -> macos-x86_64-{debug|release}
#   Linux x86_64/amd64   -> linux-x64-{debug|release}
#   Linux arm64/aarch64 -> linux-aarch64-{debug|release}
#
# Set USE_NIX=1 to run inside the Nix dev shell (when flake.nix exists).
# Override preset with CMAKE_PRESET.
# Set BUILD_KEEP_GOING=1 to pass -k 0 to Ninja (continue past failures to surface more errors).

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=scripts/include/logging.sh
. "${SCRIPT_DIR}/include/logging.sh"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
# shellcheck source=scripts/with_nix.sh
. "${SCRIPT_DIR}/with_nix.sh"
run_with_nix_if_requested "$@"

# Detect platform: OS and arch
ARCH="${ARCH:-$(uname -m 2>/dev/null || echo unknown)}"
OS="${OS:-$(uname -s 2>/dev/null || echo unknown)}"

# Normalize arch
case "${ARCH}" in
x86_64 | amd64) ARCH="x86_64" ;;
arm64 | aarch64) ARCH="aarch64" ;;
*) ;;
esac

# Build type: --debug or default release (portable lowercase for preset names)
if [[ -n "${CMAKE_BUILD_TYPE:-}" ]]; then
  case "${CMAKE_BUILD_TYPE}" in
  [Dd]ebug) BUILD_TYPE="debug" ;;
  [Rr]elease) BUILD_TYPE="release" ;;
  *) BUILD_TYPE="release" ;;
  esac
else
  BUILD_TYPE="release"
fi
for arg in "$@"; do
  case "$arg" in
  --debug)
    BUILD_TYPE="debug"
    break
    ;;
  --release)
    BUILD_TYPE="release"
    break
    ;;
  esac
done

# Choose CMake preset by OS + arch
choose_preset() {
  case "${OS}" in
  Darwin)
    case "${ARCH}" in
    aarch64) echo "macos-arm64-${BUILD_TYPE}" ;;
    x86_64) echo "macos-x86_64-${BUILD_TYPE}" ;;
    *)
      log_error "Unsupported macOS arch: ${ARCH}"
      return 1
      ;;
    esac
    ;;
  Linux)
    case "${ARCH}" in
    x86_64) echo "linux-x64-${BUILD_TYPE}" ;;
    aarch64) echo "linux-aarch64-${BUILD_TYPE}" ;;
    *)
      log_error "Unsupported Linux arch: ${ARCH}"
      return 1
      ;;
    esac
    ;;
  *)
    log_error "Unsupported OS: ${OS}. Use CMAKE_PRESET= to override."
    return 1
    ;;
  esac
}

PRESET="${CMAKE_PRESET:-$(choose_preset)}"

if ! command -v cmake >/dev/null 2>&1; then
  log_error "cmake not found. Install CMake or use USE_NIX=1 with nix develop."
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

log_note "Platform: ${OS} ${ARCH} -> preset ${PRESET}"
log_note "Override with CMAKE_PRESET= or --debug/--release."

ensure_configured() {
  log_info "Configuring preset ${PRESET}..."
  cmake --preset "${PRESET}" >/dev/null
}

# Parse command (skip --debug/--release)
CMD="build"
while [[ $# -gt 0 ]]; do
  case "$1" in
  build | configure | clean | test | install | -h | --help)
    CMD="$1"
    shift
    break
    ;;
  --debug | --release) shift ;;
  *) shift ;;
  esac
done

case "${CMD}" in
-h | --help)
  cat <<EOF
Usage: $0 [build|configure|clean|test|install] [--debug|--release]

Portable across macOS (Intel/ARM) and Linux. Picks CMake preset by OS and arch.

Environment:
  CMAKE_PRESET   Override detected preset
  USE_NIX=1      Run inside Nix dev shell
  CMAKE_BUILD_TYPE   Debug or Release (default: Release)
EOF
  ;;
configure)
  ensure_configured
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
  if [[ -n "${BUILD_KEEP_GOING:-}" ]] && [[ "${BUILD_KEEP_GOING}" != "0" ]]; then
    cmake --build --preset "${PRESET}" -- -k 0 "$@"
  else
    cmake --build --preset "${PRESET}" "$@"
  fi
  ;;
esac
