#!/usr/bin/env bash
# build_fast.sh - Fast build using CMake presets with sccache or ccache
# Uses CMakePresets.json (macos-arm64-release-sccache, macos-x86_64-release-sccache,
# linux-x64-release-sccache, or -ccache variants). CMake builds TWS API and Intel
# decimal via ExternalProject when needed; no manual vendor build here.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

cd "${PROJECT_ROOT}"

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

# Prefer sccache, fallback to ccache
USE_SCCACHE=false
USE_CCACHE=false

if command -v sccache &>/dev/null; then
  USE_SCCACHE=true
  echo "=== Fast Build with sccache ==="
elif command -v ccache &>/dev/null; then
  USE_CCACHE=true
  echo "=== Fast Build with ccache ==="
else
  echo "Neither sccache nor ccache found. Installing ccache..."
  if [[ "${OSTYPE:-}" == "darwin"* ]]; then
    brew install ccache
  else
    sudo apt-get install -y ccache
  fi
  USE_CCACHE=true
  echo "=== Fast Build with ccache ==="
fi

# Configure cache tool
if [ "$USE_SCCACHE" = true ]; then
  export SCCACHE_DIR="${SCCACHE_DIR:-$HOME/.sccache}"
  export SCCACHE_CACHE_SIZE="${SCCACHE_CACHE_SIZE:-10G}"
  mkdir -p "$SCCACHE_DIR"
elif [ "$USE_CCACHE" = true ]; then
  ccache --max-size=10G
  ccache --set-config=compression=true
  ccache --set-config=compression_level=6
fi

PRESET="${SUFFIX}-release-sccache"
[ "$USE_CCACHE" = true ] && PRESET="${SUFFIX}-release-ccache"

# Ensure preset exists (CMake will error with a clear message if not)
cmake --preset "${PRESET}"
echo "Building with preset: ${PRESET}"
if [[ "${OSTYPE:-}" == "darwin"* ]]; then
  NUM_JOBS=$(sysctl -n hw.ncpu 2>/dev/null || echo 4)
else
  NUM_JOBS=$(nproc 2>/dev/null || echo 4)
fi
cmake --build --preset "${PRESET}" --target ib_box_spread -j"${NUM_JOBS}"

# Show cache statistics
echo ""
if [ "$USE_SCCACHE" = true ]; then
  echo "=== sccache Statistics ==="
  sccache --show-stats
elif [ "$USE_CCACHE" = true ]; then
  echo "=== ccache Statistics ==="
  ccache --show-stats
fi

BUILD_DIR="build/${PRESET}"
BIN_PATH="${BUILD_DIR}/bin/ib_box_spread"
echo ""
if [ -f "${BIN_PATH}" ]; then
  echo "✓ Build complete: ${BIN_PATH}"
else
  echo "ℹ Build finished. Check CMake output for target status."
fi
