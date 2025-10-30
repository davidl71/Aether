#!/bin/bash
# build_universal.sh
# Build a universal (arm64 + x86_64) macOS binary for ib_box_spread using CMake.
# Usage:
#   ./build_universal.sh /path/to/IBJts/source/cppclient /path/to/libTwsApiCpp.dylib

set -euo pipefail

if [ $# -lt 2 ]; then
  echo "Usage: $0 <IBAPI_INCLUDE_DIR> <IBAPI_LIB_PATH>"
  exit 1
fi

IBAPI_INCLUDE_DIR="$1"
IBAPI_LIB="$2"

if [ ! -d "$IBAPI_INCLUDE_DIR" ]; then
  echo "IBAPI_INCLUDE_DIR not found: $IBAPI_INCLUDE_DIR" >&2
  exit 2
fi

if [ ! -f "$IBAPI_LIB" ]; then
  echo "IBAPI_LIB not found: $IBAPI_LIB" >&2
  exit 3
fi

BUILD_DIR="build-universal"
mkdir -p "$BUILD_DIR"
cd "$BUILD_DIR"

cmake -DCMAKE_BUILD_TYPE=Release               -DIBAPI_INCLUDE_DIR="$IBAPI_INCLUDE_DIR"               -DIBAPI_LIB="$IBAPI_LIB"               -DCMAKE_OSX_ARCHITECTURES="arm64;x86_64"               ..

cmake --build . --config Release

BIN_PATH="$(pwd)/ib_box_spread"
echo ""
echo "✅ Build complete: $BIN_PATH"
echo "Run with: $BIN_PATH --help"
