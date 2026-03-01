#!/bin/bash
# build_fast.sh - Fast build with sccache or ccache

set -e

# Get script directory and project root
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

cd "${PROJECT_ROOT}"

# Detect caching tool (prefer sccache, fallback to ccache)
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
  if [[ $OSTYPE == "darwin"* ]]; then
    brew install ccache
  else
    sudo apt-get install ccache
  fi
  USE_CCACHE=true
  echo "=== Fast Build with ccache ==="
fi

# Configure caching tool
if [ "$USE_SCCACHE" = true ]; then
  echo "Configuring sccache..."
  export SCCACHE_DIR="${SCCACHE_DIR:-$HOME/.sccache}"
  export SCCACHE_CACHE_SIZE="${SCCACHE_CACHE_SIZE:-10G}"
  mkdir -p "$SCCACHE_DIR"
elif [ "$USE_CCACHE" = true ]; then
  echo "Configuring ccache..."
  ccache --max-size=10G
  ccache --set-config=compression=true
  ccache --set-config=compression_level=6
fi

# Build directory
BUILD_DIR="build-fast"

# IBAPI paths - use third_party directory if available
TWS_API_INCLUDE_DIR="${TWS_API_INCLUDE_DIR:-native/third_party/tws-api/IBJts/source/cppclient/client}"
TWS_CLIENT_DIR="native/third_party/tws-api/IBJts/source/cppclient/client"
if [[ "$OSTYPE" == "darwin"* ]]; then
  TWS_LIB_EXT=".dylib"
else
  TWS_LIB_EXT=".so"
fi
TWS_LIB_PATH="${TWS_CLIENT_DIR}/build/lib/libtwsapi${TWS_LIB_EXT}"
# Check if TWS API library already exists, if so disable vendor build
if [ -f "$TWS_LIB_PATH" ]; then
  TWS_API_BUILD_VENDOR="OFF"
else
  TWS_API_BUILD_VENDOR="${TWS_API_BUILD_VENDOR:-ON}"
fi

# Build Intel Decimal library if needed
INTEL_LIB_PATH="native/third_party/IntelRDFPMathLib20U2/LIBRARY/libbid.a"
INTEL_DIR="native/third_party/IntelRDFPMathLib20U2/LIBRARY"
INTEL_BUILD_DIR="${INTEL_DIR}/build"

# If we will build TWS API and libbid.a already exists, force rebuild of Intel lib with -fPIC
# (required when linking static lib into shared lib; stale libbid.a may have been built without -fPIC)
if [ ! -f "$TWS_LIB_PATH" ] && [ -f "$INTEL_LIB_PATH" ]; then
  echo "Forcing rebuild of Intel Decimal library with -fPIC for TWS API shared library..."
  rm -f "$INTEL_LIB_PATH"
  rm -rf "$INTEL_BUILD_DIR"
  # Clean TWS API build so it reconfigures and links against the new libbid.a
  rm -rf "${TWS_CLIENT_DIR}/build"
fi

if [ ! -f "$INTEL_LIB_PATH" ]; then
  echo "Building Intel Decimal library (libbid.a)..."

  if [ -d "$INTEL_DIR" ]; then
    mkdir -p "$INTEL_BUILD_DIR"
    cmake -S "$INTEL_DIR" -B "$INTEL_BUILD_DIR" -DCMAKE_BUILD_TYPE=Release
    cmake --build "$INTEL_BUILD_DIR"

    if [ -f "$INTEL_LIB_PATH" ]; then
      echo "✓ Intel Decimal library built: $INTEL_LIB_PATH"
    else
      echo "⚠️  Warning: Intel Decimal library build may have failed, but continuing..."
    fi
  else
    echo "⚠️  Warning: Intel Decimal library source not found at $INTEL_DIR"
  fi
fi

# Build TWS API library if needed
TWS_CLIENT_DIR="native/third_party/tws-api/IBJts/source/cppclient/client"
if [[ "$OSTYPE" == "darwin"* ]]; then
  TWS_LIB_EXT=".dylib"
else
  TWS_LIB_EXT=".so"
fi
TWS_LIB_PATH="${TWS_CLIENT_DIR}/build/lib/libtwsapi${TWS_LIB_EXT}"
if [ ! -f "$TWS_LIB_PATH" ]; then
  echo "Building TWS API library (libtwsapi${TWS_LIB_EXT})..."

  if [ -d "$TWS_CLIENT_DIR" ]; then
    TWS_BUILD_DIR="${TWS_CLIENT_DIR}/build"
    mkdir -p "$TWS_BUILD_DIR"

    # Check if CMakeLists.txt exists in client directory
    if [ -f "${TWS_CLIENT_DIR}/CMakeLists.txt" ]; then
      cmake -S "$TWS_CLIENT_DIR" -B "$TWS_BUILD_DIR" -DCMAKE_BUILD_TYPE=Release
      cmake --build "$TWS_BUILD_DIR"

      if [ -f "$TWS_LIB_PATH" ]; then
        echo "✓ TWS API library built: $TWS_LIB_PATH"
      else
        echo "⚠️  Warning: TWS API library build may have failed, but continuing..."
      fi
    else
      echo "⚠️  Warning: TWS API CMakeLists.txt not found, will use ExternalProject build"
    fi
  else
    echo "⚠️  Warning: TWS API client directory not found at $TWS_CLIENT_DIR"
  fi
fi

# Configure CMake with appropriate caching tool
if [ "$USE_SCCACHE" = true ]; then
  echo "Configuring with sccache..."
  cmake -S . -B "$BUILD_DIR" \
    -DCMAKE_BUILD_TYPE=Release \
    -DENABLE_SCCACHE=ON \
    -DENABLE_LTO=ON \
    -DENABLE_NATIVE_CLI=ON \
    -DTWS_API_BUILD_VENDOR="$TWS_API_BUILD_VENDOR"
else
  echo "Configuring with ccache..."
  cmake -S . -B "$BUILD_DIR" \
    -DCMAKE_BUILD_TYPE=Release \
    -DENABLE_CCACHE=ON \
    -DENABLE_LTO=ON \
    -DENABLE_NATIVE_CLI=ON \
    -DTWS_API_BUILD_VENDOR="$TWS_API_BUILD_VENDOR"
fi

# Detect number of cores
if [[ $OSTYPE == "darwin"* ]]; then
  NUM_CORES=$(sysctl -n hw.ncpu)
else
  NUM_CORES=$(nproc)
fi

echo "Building with $NUM_CORES cores..."
echo "Building CLI target: ib_box_spread"
cmake --build "$BUILD_DIR" --target ib_box_spread -j"$NUM_CORES"

# Show caching statistics
echo ""
if [ "$USE_SCCACHE" = true ]; then
  echo "=== sccache Statistics ==="
  sccache --show-stats
elif [ "$USE_CCACHE" = true ]; then
  echo "=== ccache Statistics ==="
  ccache --show-stats
fi

bin_path="$BUILD_DIR/bin/ib_box_spread"
echo ""
if [ -f "$bin_path" ]; then
  echo "✓ Build complete: $bin_path"
else
  echo "ℹ️  Build finished. Native CLI target was skipped (see CMake output for details)."
fi
