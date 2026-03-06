#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/../../.." && pwd)"
DESKTOP_DIR="$ROOT_DIR/desktop"

if [ -d "$DESKTOP_DIR" ]; then
  cd "$DESKTOP_DIR"

  shopt -s nullglob
  xcode_projects=(*.xcodeproj)
  shopt -u nullglob

  if [ ${#xcode_projects[@]} -gt 0 ] && command -v xcodebuild >/dev/null 2>&1; then
    xcodebuild test || true
  elif [ -f "Package.swift" ] && command -v swift >/dev/null 2>&1; then
    swift test || true
  else
    echo "[warn] No supported desktop test runner detected; initialize Xcode or Swift toolchains." >&2
  fi
else
  echo "[info] desktop/ project not present yet; add implementation before running tests." >&2
fi
