#!/usr/bin/env bash
# build_variant.sh - Parameterized build script (consolidates build_fast, build_distributed, build_with_logging)
# Usage: ./scripts/build_variant.sh [variant] [extra args...]
#
# Variants:
#   fast        - Fast build with sccache or ccache (default)
#   distributed - Distributed build with distcc + ccache (or sccache)
#   logging     - Build with timestamped log file
#   ramdisk     - Delegate to build_ramdisk.sh (setup/build/test/clean/status)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
# shellcheck source=./with_nix.sh
. "${SCRIPT_DIR}/with_nix.sh"
  run_with_nix_if_requested "$@"
cd "${PROJECT_ROOT}"

# Default preset from OS/arch (same logic as build_ai_friendly.sh)
detect_default_preset() {
  local arch os
  arch="$(uname -m 2>/dev/null || echo unknown)"
  os="$(uname -s 2>/dev/null || echo unknown)"
  case "${os}" in
    Darwin)
      if [[ "${arch}" == "arm64" || "${arch}" == "aarch64" ]]; then
        echo "macos-arm64-debug"
      else
        echo "macos-x86_64-debug"
      fi
      ;;
    Linux) echo "linux-x64-debug" ;;
    *) echo "macos-x86_64-debug" ;;
  esac
}

# Ensure third-party deps exist before configure/build
# shellcheck source=./include/ensure_third_party.sh
. "${SCRIPT_DIR}/include/ensure_third_party.sh"
ensure_third_party

# TWS API paths (new layout: tws-api/source/cppclient/client)
TWS_API_CLIENT_DIR="${TWS_API_CLIENT_DIR:-native/third_party/tws-api/source/cppclient/client}"
TWS_LIB_CANDIDATES=(
  "native/ibapi_cmake/build/lib/libTwsApiCpp.dylib"
  "${TWS_API_CLIENT_DIR}/build/lib/libtwsapi.dylib"
  "${TWS_API_CLIENT_DIR}/lib/libtwsapi.dylib"
)

# Find existing TWS lib or enable vendor build
TWS_API_BUILD_VENDOR="${TWS_API_BUILD_VENDOR:-ON}"
IBAPI_LIB_OVERRIDE=""
for cand in "${TWS_LIB_CANDIDATES[@]}"; do
  if [[ -f "$cand" ]]; then
    TWS_API_BUILD_VENDOR="OFF"
    IBAPI_LIB_OVERRIDE="$cand"
    break
  fi
done

# Build Intel Decimal if needed
INTEL_LIB="native/third_party/IntelRDFPMathLib20U2/LIBRARY/libbid.a"
if [[ ! -f "$INTEL_LIB" ]]; then
  echo "Building Intel Decimal (libbid.a)..."
  just build-intel-decimal 2>/dev/null || {
    cmake -S native/third_party/IntelRDFPMathLib20U2/LIBRARY \
      -B native/third_party/IntelRDFPMathLib20U2/LIBRARY/build -DCMAKE_BUILD_TYPE=Release
    cmake --build native/third_party/IntelRDFPMathLib20U2/LIBRARY/build
  }
fi

variant="${1:-fast}"
shift || true

case "$variant" in
  ramdisk)
    exec "${SCRIPT_DIR}/build_ramdisk.sh" "$@"
    ;;
  logging)
    PRESET="${1:-$(detect_default_preset)}"
    LOG_FILE="${PROJECT_ROOT}/logs/build_$(date +%Y%m%d_%H%M%S).log"
    mkdir -p "${PROJECT_ROOT}/logs"
    exec > >(tee -a "$LOG_FILE") 2>&1
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] === Build (logging) ==="
    cmake --preset "$PRESET"
    cmake --build --preset "$PRESET"
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] Log: $LOG_FILE"
    ;;
  fast)
    BUILD_DIR="build-fast"
    USE_SCCACHE=false
    USE_CCACHE=false
    if command -v sccache &>/dev/null; then
      USE_SCCACHE=true
      echo "=== Fast Build (sccache) ==="
    elif command -v ccache &>/dev/null; then
      USE_CCACHE=true
      echo "=== Fast Build (ccache) ==="
    else
      echo "=== Fast Build (no cache) ==="
    fi

    [[ "$USE_SCCACHE" = true ]] && export SCCACHE_DIR="${SCCACHE_DIR:-$HOME/.sccache}" && mkdir -p "$SCCACHE_DIR"
    [[ "$USE_CCACHE" = true ]] && ccache --max-size=10G 2>/dev/null || true

    CMAKE_EXTRA=()
    [[ -n "$IBAPI_LIB_OVERRIDE" ]] && CMAKE_EXTRA+=(-DIBAPI_LIB="$IBAPI_LIB_OVERRIDE")
    if [[ "$USE_SCCACHE" = true ]]; then
      cmake -S . -B "$BUILD_DIR" -G Ninja -DCMAKE_BUILD_TYPE=Release \
        -DENABLE_SCCACHE=ON -DENABLE_LTO=ON -DENABLE_NATIVE_CLI=ON -DTWS_API_BUILD_VENDOR="$TWS_API_BUILD_VENDOR" "${CMAKE_EXTRA[@]}"
    elif [[ "$USE_CCACHE" = true ]]; then
      cmake -S . -B "$BUILD_DIR" -G Ninja -DCMAKE_BUILD_TYPE=Release \
        -DENABLE_CCACHE=ON -DENABLE_LTO=ON -DENABLE_NATIVE_CLI=ON -DTWS_API_BUILD_VENDOR="$TWS_API_BUILD_VENDOR" "${CMAKE_EXTRA[@]}"
    else
      cmake -S . -B "$BUILD_DIR" -G Ninja -DCMAKE_BUILD_TYPE=Release \
        -DENABLE_LTO=ON -DENABLE_NATIVE_CLI=ON -DTWS_API_BUILD_VENDOR="$TWS_API_BUILD_VENDOR" "${CMAKE_EXTRA[@]}"
    fi

    NPROC=$(sysctl -n hw.ncpu 2>/dev/null || nproc)
    cmake --build "$BUILD_DIR" --target ib_box_spread -j"${NPROC}"
    [[ "$USE_SCCACHE" = true ]] && sccache --show-stats 2>/dev/null || true
    [[ "$USE_CCACHE" = true ]] && ccache --show-stats 2>/dev/null || true
    [[ -f "$BUILD_DIR/bin/ib_box_spread" ]] && echo "Build complete: $BUILD_DIR/bin/ib_box_spread"
    ;;
  distributed)
    BUILD_DIR="build-distributed"
    USE_DISTCC=false
    CMAKE_EXTRA=()
    [[ -n "$IBAPI_LIB_OVERRIDE" ]] && CMAKE_EXTRA+=(-DIBAPI_LIB="$IBAPI_LIB_OVERRIDE")
    if command -v sccache &>/dev/null; then
      echo "=== Distributed Build (sccache) ==="
      export SCCACHE_DIR="${SCCACHE_DIR:-$HOME/.sccache}"
      mkdir -p "$SCCACHE_DIR"
      cmake -S . -B "$BUILD_DIR" -G Ninja -DCMAKE_BUILD_TYPE=Release \
        -DENABLE_SCCACHE=ON -DENABLE_LTO=ON -DENABLE_NATIVE_CLI=ON -DTWS_API_BUILD_VENDOR="$TWS_API_BUILD_VENDOR" "${CMAKE_EXTRA[@]}"
    elif command -v ccache &>/dev/null; then
      command -v distcc &>/dev/null && USE_DISTCC=true
      [[ "$USE_DISTCC" = true ]] && export CCACHE_PREFIX="distcc" && export DISTCC_HOSTS="${DISTCC_HOSTS:-localhost/4}"
      echo "=== Distributed Build (ccache${USE_DISTCC:+ + distcc}) ==="
      ccache --max-size=10G 2>/dev/null || true
      if [[ "$USE_DISTCC" = true ]]; then
        cmake -S . -B "$BUILD_DIR" -G Ninja -DCMAKE_BUILD_TYPE=Release \
          -DENABLE_CCACHE=ON -DENABLE_DISTCC=ON -DENABLE_LTO=ON -DENABLE_NATIVE_CLI=ON -DTWS_API_BUILD_VENDOR="$TWS_API_BUILD_VENDOR" "${CMAKE_EXTRA[@]}"
      else
        cmake -S . -B "$BUILD_DIR" -G Ninja -DCMAKE_BUILD_TYPE=Release \
          -DENABLE_CCACHE=ON -DENABLE_LTO=ON -DENABLE_NATIVE_CLI=ON -DTWS_API_BUILD_VENDOR="$TWS_API_BUILD_VENDOR" "${CMAKE_EXTRA[@]}"
      fi
    else
      echo "=== Distributed Build (no cache) ==="
      cmake -S . -B "$BUILD_DIR" -G Ninja -DCMAKE_BUILD_TYPE=Release \
        -DENABLE_LTO=ON -DENABLE_NATIVE_CLI=ON -DTWS_API_BUILD_VENDOR="$TWS_API_BUILD_VENDOR" "${CMAKE_EXTRA[@]}"
    fi

    NPROC=$(sysctl -n hw.ncpu 2>/dev/null || nproc)
    PARALLEL=$((NPROC * 2))
    cmake --build "$BUILD_DIR" --target ib_box_spread -j"${PARALLEL}"
    [[ -f "$BUILD_DIR/bin/ib_box_spread" ]] && echo "✓ Build complete: $BUILD_DIR/bin/ib_box_spread"
    ;;
  *)
    echo "Usage: $0 [fast|distributed|logging|ramdisk] [args...]"
    echo "  fast        - sccache/ccache build (default)"
    echo "  distributed - distcc + ccache or sccache"
    echo "  logging     - build with log file"
    echo "  ramdisk     - delegate to build_ramdisk.sh"
    exit 1
    ;;
esac
