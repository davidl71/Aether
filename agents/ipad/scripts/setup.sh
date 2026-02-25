#!/usr/bin/env bash
set -euo pipefail

if ! command -v xcodebuild >/dev/null 2>&1; then
  echo "[warn] xcodebuild not found; install Xcode command line tools for iPad development." >&2
  exit 0
fi

echo "[info] Xcode tooling detected. Configure your SwiftUI project under ios/."

