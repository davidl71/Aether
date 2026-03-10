#!/usr/bin/env bash
# build_distributed.sh - Distributed build using CMake presets with distcc+ccache or sccache
# Uses CMakePresets.json (-distcc, -sccache, or -ccache presets). Prefers distcc+ccache
# when distcc is available; otherwise same logic as build_fast.sh. Uses cmake --build.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
# shellcheck source=./include/workspace_paths.sh
. "${SCRIPT_DIR}/include/workspace_paths.sh"

setup_workspace_paths

cd "${PROJECT_ROOT}"

# Ensure third-party deps exist before configure/build
# shellcheck source=scripts/include/ensure_third_party.sh
. "${SCRIPT_DIR}/include/ensure_third_party.sh"
ensure_third_party

# Detect platform for preset selection
detect_preset_suffix() {
  local arch os
  arch="$(uname -m 2>/dev/null || echo unknown)"
  os="$(uname -s 2>/dev/null || echo unknown)"
  case "${os}" in
    Darwin)
      if [[ "${arch}" == "arm64" || "${arch}" == "aarch64" ]]; then
        echo "macos-arm64"
      else
        echo "macos-x86_64"
      fi
      ;;
    Linux)
      echo "linux-x64"
      ;;
    *)
      echo ""
      ;;
  esac
}

SUFFIX="$(detect_preset_suffix)"
if [ -z "${SUFFIX}" ]; then
  echo "Unsupported platform: $(uname -s) $(uname -m). Use a CMake preset directly." >&2
  exit 1
fi

# Prefer distcc+ccache, then sccache, then ccache
USE_DISTCC=false
USE_SCCACHE=false
USE_CCACHE=false

if command -v ccache &>/dev/null && command -v distcc &>/dev/null; then
  USE_CCACHE=true
  USE_DISTCC=true
  echo "=== Distributed Build with distcc + ccache ==="
  if [ -z "${DISTCC_HOSTS:-}" ]; then
    echo "Warning: DISTCC_HOSTS not set. Using localhost only."
    echo "To use remote machines: export DISTCC_HOSTS='localhost/4 remote-ip/8'"
    export DISTCC_HOSTS="localhost/4"
  else
    echo "Using distcc hosts: ${DISTCC_HOSTS}"
  fi
  export CCACHE_PREFIX="distcc"
elif command -v sccache &>/dev/null; then
  USE_SCCACHE=true
  echo "=== Distributed Build with sccache ==="
  echo "For distributed caching, configure S3/Redis: export SCCACHE_BUCKET=... or SCCACHE_REDIS=..."
elif command -v ccache &>/dev/null; then
  USE_CCACHE=true
  echo "=== Distributed Build with ccache ==="
else
  echo "Neither sccache nor ccache found. Installing ccache..."
  if [[ "${OSTYPE:-}" == "darwin"* ]]; then
    brew install ccache
  else
    sudo apt-get install -y ccache
  fi
  USE_CCACHE=true
  echo "=== Distributed Build with ccache ==="
fi

# Configure cache tool
if [ "$USE_SCCACHE" = true ]; then
  export SCCACHE_CACHE_SIZE="${SCCACHE_CACHE_SIZE:-10G}"
  mkdir -p "$SCCACHE_DIR"
elif [ "$USE_CCACHE" = true ]; then
  export CCACHE_DIR="${CCACHE_DIR}"
  ccache --max-size=10G
  ccache --set-config=compression=true
  ccache --set-config=compression_level=6
fi

# Choose preset
PRESET="${SUFFIX}-release-distcc"
[ "$USE_DISTCC" = false ] && PRESET="${SUFFIX}-release-sccache"
[ "$USE_SCCACHE" = false ] && [ "$USE_CCACHE" = true ] && [ "$USE_DISTCC" = false ] && PRESET="${SUFFIX}-release-ccache"

cmake --preset "${PRESET}"
echo "Building with preset: ${PRESET}"

# Parallelism: for distcc, use more jobs (local + remote)
if [[ "${OSTYPE:-}" == "darwin"* ]]; then
  LOCAL_CORES=$(sysctl -n hw.ncpu 2>/dev/null || echo 4)
else
  LOCAL_CORES=$(nproc 2>/dev/null || echo 4)
fi
PARALLEL_JOBS=$((LOCAL_CORES * 2))
[ "$USE_DISTCC" = true ] || PARALLEL_JOBS=$LOCAL_CORES

cmake --build --preset "${PRESET}" --target ib_box_spread -j"${PARALLEL_JOBS}"

# Statistics
echo ""
if [ "$USE_SCCACHE" = true ]; then
  echo "=== sccache Statistics ==="
  sccache --show-stats
elif [ "$USE_CCACHE" = true ]; then
  echo "=== ccache Statistics ==="
  ccache --show-stats
  if [ "$USE_DISTCC" = true ] && command -v distcc &>/dev/null; then
    echo ""
    echo "=== distcc Statistics ==="
    distcc --show-stats 2>/dev/null || echo "No distcc stats available"
  fi
fi

BUILD_DIR="build/${PRESET}"
BIN_PATH="${BUILD_DIR}/bin/ib_box_spread"
echo ""
if [ -f "${BIN_PATH}" ]; then
  echo "✓ Build complete: ${BIN_PATH}"
else
  echo "ℹ Build finished. Check CMake output for target status."
fi
