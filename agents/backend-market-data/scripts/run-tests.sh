#!/usr/bin/env bash
set -euo pipefail

AGENT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
MANIFEST="$AGENT_DIR/Cargo.toml"

if ! command -v cargo >/dev/null 2>&1; then
  echo "[error] cargo not found; install Rust toolchain" >&2
  exit 1
fi

cargo fmt --all --manifest-path "$MANIFEST"
cargo clippy --all-targets --manifest-path "$MANIFEST" -- -D warnings
cargo test --manifest-path "$MANIFEST"
