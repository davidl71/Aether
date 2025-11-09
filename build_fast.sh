#!/bin/bash
# build_fast.sh - Fast build with ccache

set -e

echo "=== Fast Build with ccache ==="

# Install ccache if not present
if ! command -v ccache &> /dev/null; then
    echo "ccache not found. Installing..."
    if [[ "$OSTYPE" == "darwin"* ]]; then
        brew install ccache
    else
        sudo apt-get install ccache
    fi
fi

# Configure ccache
echo "Configuring ccache..."
ccache --max-size=10G
ccache --set-config=compression=true
ccache --set-config=compression_level=6

# Build directory
BUILD_DIR="build-fast"

# IBAPI paths (customize if needed)
IBAPI_INCLUDE_DIR="${IBAPI_INCLUDE_DIR:-$HOME/IBJts/source/cppclient}"
IBAPI_LIB="${IBAPI_LIB:-$HOME/IBJts/source/cppclient/libTwsApiCpp.dylib}"
TWS_API_BUILD_VENDOR="${TWS_API_BUILD_VENDOR:-OFF}"

echo "Configuring with ccache..."
cmake -S . -B "$BUILD_DIR" \
  -DCMAKE_BUILD_TYPE=Release \
  -DENABLE_CCACHE=ON \
  -DENABLE_LTO=ON \
  -DIBAPI_INCLUDE_DIR="$IBAPI_INCLUDE_DIR" \
  -DIBAPI_LIB="$IBAPI_LIB" \
  -DTWS_API_BUILD_VENDOR="$TWS_API_BUILD_VENDOR"

# Detect number of cores
if [[ "$OSTYPE" == "darwin"* ]]; then
    NUM_CORES=$(sysctl -n hw.ncpu)
else
    NUM_CORES=$(nproc)
fi

echo "Building with $NUM_CORES cores..."
make -j"$NUM_CORES" -C "$BUILD_DIR"

# Show ccache statistics
echo ""
echo "=== ccache Statistics ==="
ccache --show-stats

bin_path="$BUILD_DIR/bin/ib_box_spread"
echo ""
if [ -f "$bin_path" ]; then
  echo "✓ Build complete: $bin_path"
else
  echo "ℹ️  Build finished. Native CLI target was skipped (see CMake output for details)."
fi

