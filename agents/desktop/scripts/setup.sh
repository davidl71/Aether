#!/usr/bin/env bash
set -euo pipefail

if command -v xcodebuild >/dev/null 2>&1; then
  echo "[info] macOS SwiftUI toolchain available."
elif command -v npm >/dev/null 2>&1; then
  echo "[info] Node.js available for Electron-based desktop app."
else
  echo "[warn] Neither Xcode nor Node.js detected; install required tooling for desktop development." >&2
fi

