#!/bin/bash
# build_distributed.sh - Distributed build with distcc and ccache

set -e

echo "=== Distributed Build with distcc + ccache ==="

# Check for required tools
if ! command -v distcc &> /dev/null; then
    echo "Error: distcc not found. Install with: brew install distcc"
    exit 1
fi

if ! command -v ccache &> /dev/null; then
    echo "ccache not found. Installing..."
    if [[ "$OSTYPE" == "darwin"* ]]; then
        brew install ccache
    else
        sudo apt-get install ccache
    fi
fi

# distcc hosts (customize for your network)
if [ -z "$DISTCC_HOSTS" ]; then
    echo "Warning: DISTCC_HOSTS not set. Using localhost only."
    echo "To use remote machines, set: export DISTCC_HOSTS='localhost/4 remote-ip/8'"
    export DISTCC_HOSTS="localhost/4"
fi

echo "Using distcc hosts: $DISTCC_HOSTS"

# Use distcc via ccache
export CCACHE_PREFIX="distcc"

# Configure ccache
echo "Configuring ccache..."
ccache --max-size=10G
ccache --set-config=compression=true
ccache --set-config=compression_level=6

# Build directory
BUILD_DIR="build-distributed"

# IBAPI paths (customize if needed)
IBAPI_INCLUDE_DIR="${IBAPI_INCLUDE_DIR:-$HOME/IBJts/source/cppclient}"
IBAPI_LIB="${IBAPI_LIB:-$HOME/IBJts/source/cppclient/libTwsApiCpp.dylib}"

echo "Configuring with ccache + distcc..."
cmake -S . -B "$BUILD_DIR" \
  -DCMAKE_BUILD_TYPE=Release \
  -DENABLE_CCACHE=ON \
  -DENABLE_DISTCC=ON \
  -DENABLE_LTO=ON \
  -DIBAPI_INCLUDE_DIR="$IBAPI_INCLUDE_DIR" \
  -DIBAPI_LIB="$IBAPI_LIB"

# Calculate optimal parallelism
if [[ "$OSTYPE" == "darwin"* ]]; then
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
echo "=== ccache Statistics ==="
ccache --show-stats

echo ""
echo "=== distcc Statistics ==="
if command -v distcc &> /dev/null; then
    DISTCC_DIR=/tmp distcc --show-stats 2>/dev/null || echo "No distcc stats available"
fi

echo ""
echo "✓ Build complete: $BUILD_DIR/bin/ib_box_spread"

