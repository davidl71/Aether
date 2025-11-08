#!/usr/bin/env bash
set -euo pipefail

AGENT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
WORKSPACE_MANIFEST="$AGENT_DIR/Cargo.toml"

if ! command -v cargo >/dev/null 2>&1; then
  echo "[error] Rust toolchain missing. Install Rust via https://rustup.rs." >&2
  exit 1
fi

cargo fetch --manifest-path "$WORKSPACE_MANIFEST"
