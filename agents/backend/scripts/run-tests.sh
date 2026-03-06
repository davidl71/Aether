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

if command -v python3 >/dev/null 2>&1; then
  # shellcheck disable=SC1091
  source "$BACKEND_DIR/.venv/bin/activate" || true
  python3 -m pytest "$BACKEND_DIR/python/tests"
else
  echo "[warn] python3 not found; skipping Python tests" >&2
fi
