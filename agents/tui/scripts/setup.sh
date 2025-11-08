#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/../../.." && pwd)"

cd "$ROOT_DIR/tui"

if command -v go >/dev/null 2>&1; then
  go mod tidy
else
  echo "[warn] Go not found; install Go 1.21+ to build the TUI" >&2
fi

