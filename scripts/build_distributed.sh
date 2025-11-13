#!/bin/bash
# build_distributed.sh - Distributed build with sccache, distcc, or ccache

set -e

# Get script directory and project root
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

cd "${PROJECT_ROOT}"

# Detect caching tool (prefer sccache, fallback to ccache)
USE_SCCACHE=false
USE_CCACHE=false
USE_DISTCC=false

if command -v sccache &>/dev/null; then
  USE_SCCACHE=true
  echo "=== Distributed Build with sccache ==="
  echo "Note: sccache supports distributed caching natively (S3, Redis, etc.)"
elif command -v ccache &>/dev/null; then
  USE_CCACHE=true
  echo "=== Distributed Build with distcc + ccache ==="

  # Check for distcc
  if ! command -v distcc &>/dev/null; then
    echo "Warning: distcc not found. Install with: brew install distcc"
    echo "Continuing with ccache only (no distributed compilation)"
  else
    USE_DISTCC=true
  fi
else
  echo "Neither sccache nor ccache found. Installing ccache..."
  if [[ $OSTYPE == "darwin"* ]]; then
    brew install ccache
  else
    sudo apt-get install ccache
  fi
  USE_CCACHE=true
  echo "=== Distributed Build with distcc + ccache ==="

  if command -v distcc &>/dev/null; then
    USE_DISTCC=true
  fi
fi

# Configure caching tool
if [ "$USE_SCCACHE" = true ]; then
  echo "Configuring sccache..."
  export SCCACHE_DIR="${SCCACHE_DIR:-$HOME/.sccache}"
  export SCCACHE_CACHE_SIZE="${SCCACHE_CACHE_SIZE:-10G}"
  mkdir -p "$SCCACHE_DIR"
  echo "For distributed caching, configure S3/Redis backend:"
  echo "  export SCCACHE_BUCKET=your-bucket"
  echo "  export SCCACHE_REGION=us-east-1"
elif [ "$USE_CCACHE" = true ]; then
  # distcc hosts (customize for your network)
  if [ "$USE_DISTCC" = true ]; then
    if [ -z "$DISTCC_HOSTS" ]; then
      echo "Warning: DISTCC_HOSTS not set. Using localhost only."
      echo "To use remote machines, set: export DISTCC_HOSTS='localhost/4 remote-ip/8'"
      export DISTCC_HOSTS="localhost/4"
    fi
    echo "Using distcc hosts: $DISTCC_HOSTS"
    # Use distcc via ccache
    export CCACHE_PREFIX="distcc"
  fi

  echo "Configuring ccache..."
  ccache --max-size=10G
  ccache --set-config=compression=true
  ccache --set-config=compression_level=6
fi

# Build directory
BUILD_DIR="build-distributed"

# IBAPI paths (customize if needed)
IBAPI_INCLUDE_DIR="${IBAPI_INCLUDE_DIR:-$HOME/IBJts/source/cppclient}"
IBAPI_LIB="${IBAPI_LIB:-$HOME/IBJts/source/cppclient/libTwsApiCpp.dylib}"
TWS_API_BUILD_VENDOR="${TWS_API_BUILD_VENDOR:-OFF}"

# Configure CMake with appropriate caching tool
if [ "$USE_SCCACHE" = true ]; then
  echo "Configuring with sccache..."
  cmake -S . -B "$BUILD_DIR" \
    -DCMAKE_BUILD_TYPE=Release \
    -DENABLE_SCCACHE=ON \
    -DENABLE_LTO=ON \
    -DIBAPI_INCLUDE_DIR="$IBAPI_INCLUDE_DIR" \
    -DIBAPI_LIB="$IBAPI_LIB" \
    -DTWS_API_BUILD_VENDOR="$TWS_API_BUILD_VENDOR"
elif [ "$USE_CCACHE" = true ]; then
  if [ "$USE_DISTCC" = true ]; then
    echo "Configuring with ccache + distcc..."
    cmake -S . -B "$BUILD_DIR" \
      -DCMAKE_BUILD_TYPE=Release \
      -DENABLE_CCACHE=ON \
      -DENABLE_DISTCC=ON \
      -DENABLE_LTO=ON \
      -DIBAPI_INCLUDE_DIR="$IBAPI_INCLUDE_DIR" \
      -DIBAPI_LIB="$IBAPI_LIB" \
      -DTWS_API_BUILD_VENDOR="$TWS_API_BUILD_VENDOR"
  else
    echo "Configuring with ccache..."
    cmake -S . -B "$BUILD_DIR" \
      -DCMAKE_BUILD_TYPE=Release \
      -DENABLE_CCACHE=ON \
      -DENABLE_LTO=ON \
      -DIBAPI_INCLUDE_DIR="$IBAPI_INCLUDE_DIR" \
      -DIBAPI_LIB="$IBAPI_LIB" \
      -DTWS_API_BUILD_VENDOR="$TWS_API_BUILD_VENDOR"
  fi
fi

# Calculate optimal parallelism
if [[ $OSTYPE == "darwin"* ]]; then
  LOCAL_CORES=$(sysctl -n hw.ncpu)
else
  LOCAL_CORES=$(nproc)
fi

# Parse DISTCC_HOSTS to count total cores
TOTAL_CORES=$LOCAL_CORES
# Simple estimation: multiply by 2 for distributed
PARALLEL_JOBS=$((TOTAL_CORES * 2))

echo "Building with $PARALLEL_JOBS parallel jobs..."
make -j"$PARALLEL_JOBS" -C "$BUILD_DIR"

# Show statistics
echo ""
if [ "$USE_SCCACHE" = true ]; then
  echo "=== sccache Statistics ==="
  sccache --show-stats
elif [ "$USE_CCACHE" = true ]; then
  echo "=== ccache Statistics ==="
  ccache --show-stats

  if [ "$USE_DISTCC" = true ]; then
    echo ""
    echo "=== distcc Statistics ==="
    if command -v distcc &>/dev/null; then
      DISTCC_DIR=/tmp distcc --show-stats 2>/dev/null || echo "No distcc stats available"
    fi
  fi
fi

echo ""
bin_path="$BUILD_DIR/bin/ib_box_spread"
if [ -f "$bin_path" ]; then
  echo "✓ Build complete: $bin_path"
else
  echo "ℹ️  Build finished. Native CLI target was skipped (see CMake output for details)."
fi
