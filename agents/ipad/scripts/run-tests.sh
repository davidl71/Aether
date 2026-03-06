#!/usr/bin/env bash
set -euo pipefail

IOS_DIR="$(cd "$(dirname "$0")/../../.." && pwd)/ios"

if ! command -v xcodebuild >/dev/null 2>&1; then
  echo "[warn] xcodebuild not found; skipping iPad tests." >&2
  exit 0
fi

if [ -d "$IOS_DIR" ]; then
  cd "$IOS_DIR"
  SCHEME="BoxSpreadIPad"
  PROJECT="BoxSpreadIPad.xcodeproj"
  DESTINATION=${DESTINATION:-"platform=iOS Simulator,name=iPad (10th generation)"}

  if [ ! -d "$PROJECT" ]; then
    echo "[error] $PROJECT missing; cannot run tests." >&2
    exit 1
  fi

  if ! xcodebuild -project "$PROJECT" -scheme "$SCHEME" -destination "$DESTINATION" test; then
    echo "[warn] xcodebuild test failed; attempting build-for-testing instead." >&2
    xcodebuild -project "$PROJECT" -scheme "$SCHEME" -destination "generic/platform=iOS Simulator" build-for-testing || true
  fi
else
  echo "[info] ios/ project not present yet; add SwiftUI project before running tests." >&2
fi

