#!/usr/bin/env bash
# verify_toolchain.sh - Verify C++ toolchain (Xcode Command Line Tools)
# Usage: ./scripts/verify_toolchain.sh
#
# On macOS, C++ stdlib headers may be missing from CLT. This script checks
# and suggests: xcode-select --install
#
# See: native/CMakeLists.txt for automatic SDK header injection when CLT lacks headers.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
# shellcheck source=./with_nix.sh
. "${SCRIPT_DIR}/with_nix.sh"
run_with_nix_if_requested "$@"

echo "Checking C++ toolchain..."

# Test that clang++ can find standard library headers
if echo '#include <string>' | clang++ -x c++ -std=c++20 -fsyntax-only - 2>/dev/null; then
  echo "✓ C++ stdlib headers OK"
else
  echo "✗ C++ stdlib headers not found. clang++ cannot find <string>, <vector>, etc."
  echo ""
  echo "Fix: Run one of the following:"
  echo "  xcode-select --install"
  echo ""
  echo "If that fails, try:"
  echo "  sudo rm -rf /Library/Developer/CommandLineTools"
  echo "  xcode-select --install"
  echo ""
  echo "Then re-run this script to verify."
  exit 1
fi

# Verify CMake can configure
if command -v cmake &>/dev/null; then
  echo "✓ cmake found: $(cmake --version | head -1)"
else
  echo "✗ cmake not found. Install via: brew install cmake"
  exit 1
fi

# Verify ninja
if command -v ninja &>/dev/null; then
  echo "✓ ninja found: $(ninja --version)"
else
  echo "✗ ninja not found. Install via: brew install ninja"
  exit 1
fi

echo ""
echo "Toolchain verification passed."
