#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/../../.." && pwd)"

# C++ TUI is built as part of main CMake build
# No separate setup needed - TUI is built with ENABLE_TUI=ON

echo "[info] C++ TUI is built as part of main CMake build"
echo "[info] Build with: cmake --build build --target ib_box_spread_tui"
