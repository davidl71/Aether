#!/usr/bin/env bash
set -euo pipefail

BACKEND_DIR="$(cd "$(dirname "$0")/.." && pwd)"

if command -v cargo >/dev/null 2>&1; then
  cargo fmt --all --manifest-path "$BACKEND_DIR/Cargo.toml"
  cargo clippy --all-targets --all-features --manifest-path "$BACKEND_DIR/Cargo.toml" -- -D warnings
  cargo test --all --manifest-path "$BACKEND_DIR/Cargo.toml"
else
  echo "[warn] cargo not found; skipping Rust tests" >&2
fi

echo "[info] No backend-local Python test package remains; skipping legacy Python test step."
