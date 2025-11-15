#!/bin/bash
# build_wasm.sh - Build C++ code to WebAssembly for web app

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
WASM_DIR="${PROJECT_ROOT}/native/wasm"
WEB_WASM_DIR="${PROJECT_ROOT}/web/public/wasm"

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}Building WASM module...${NC}"

# Check for Emscripten and source if needed
if [ -z "$EMSDK" ]; then
  # Try to find emsdk in common locations
  EMSDK_PATHS=(
    "${PROJECT_ROOT}/emsdk"
    "${HOME}/emsdk"
    "/usr/local/emsdk"
  )

  EMSDK_FOUND=false
  for path in "${EMSDK_PATHS[@]}"; do
    if [ -f "${path}/emsdk_env.sh" ]; then
      echo -e "${GREEN}Sourcing Emscripten from: ${path}${NC}"
      source "${path}/emsdk_env.sh"
      EMSDK_FOUND=true
      break
    fi
  done

  if [ "$EMSDK_FOUND" = false ] || [ -z "$EMSDK" ]; then
    echo -e "${RED}Error: Emscripten not found.${NC}"
    echo "Please install and activate emsdk:"
    echo "  git clone https://github.com/emscripten-core/emsdk.git"
    echo "  cd emsdk"
    echo "  ./emsdk install latest"
    echo "  ./emsdk activate latest"
    echo "  source ./emsdk_env.sh"
    echo ""
    echo "Or run this script after sourcing emsdk_env.sh"
    exit 1
  fi
fi

# Create build directory
cd "${WASM_DIR}"
mkdir -p build
cd build

# Configure with Emscripten
echo -e "${GREEN}Configuring CMake with Emscripten...${NC}"
emcmake cmake .. -DCMAKE_BUILD_TYPE=Release

# Build
echo -e "${GREEN}Building WASM module...${NC}"
cmake --build . --config Release

# Copy to web public directory
echo -e "${GREEN}Copying WASM files to web/public/wasm...${NC}"
mkdir -p "${WEB_WASM_DIR}"
cp box_spread_wasm.js "${WEB_WASM_DIR}/" || {
  echo -e "${RED}Error: box_spread_wasm.js not found${NC}"
  exit 1
}
cp box_spread_wasm.wasm "${WEB_WASM_DIR}/" || {
  echo -e "${RED}Error: box_spread_wasm.wasm not found${NC}"
  exit 1
}

echo -e "${GREEN}✅ WASM build complete!${NC}"
echo "   Files copied to: ${WEB_WASM_DIR}"
echo "   - box_spread_wasm.js"
echo "   - box_spread_wasm.wasm"
