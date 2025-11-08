#!/usr/bin/env bash
set -euo pipefail

BACKEND_DIR="$(cd "$(dirname "$0")/.." && pwd)"

if ! command -v cargo >/dev/null 2>&1; then
  echo "[error] Rust toolchain missing. Install Rust via https://rustup.rs." >&2
  exit 1
fi

PYTHON_BIN="${PYTHON_BIN:-}"
if [ -z "$PYTHON_BIN" ]; then
  if command -v python3.11 >/dev/null 2>&1; then
    PYTHON_BIN="python3.11"
  elif command -v python3 >/dev/null 2>&1; then
    PYTHON_BIN="python3"
  fi
fi

if [ -n "$PYTHON_BIN" ]; then
  "$PYTHON_BIN" -m venv "$BACKEND_DIR/.venv"
  # shellcheck disable=SC1091
  source "$BACKEND_DIR/.venv/bin/activate"
  pip install --upgrade pip
  pip install -e "$BACKEND_DIR/python"
else
  echo "[warn] python interpreter not found; skipping Python setup" >&2
fi

cargo fetch --manifest-path "$BACKEND_DIR/Cargo.toml"
