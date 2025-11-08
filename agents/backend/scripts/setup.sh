#!/usr/bin/env bash
set -euo pipefail

BACKEND_DIR="$(cd "$(dirname "$0")/.." && pwd)"
REPO_ROOT="$(cd "$BACKEND_DIR/.." && pwd)"

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

  wheel_path=""
  if [ -n "${NAUTILUS_TRADER_WHEEL:-}" ] && [ -f "$NAUTILUS_TRADER_WHEEL" ]; then
    wheel_path="$NAUTILUS_TRADER_WHEEL"
  else
    candidate="$(find "$REPO_ROOT" -maxdepth 3 -type f -name 'nautilus_trader-*.whl' -print -quit 2>/dev/null || true)"
    if [ -n "$candidate" ]; then
      wheel_path="$candidate"
    fi
  fi

  if [ -n "$wheel_path" ]; then
    echo "[info] Installing Nautilus Trader wheel: $wheel_path"
    pip install "$wheel_path"
  else
    echo "[warn] Nautilus Trader wheel not found; skipping prebuilt install. Set NAUTILUS_TRADER_WHEEL to a local wheel path." >&2
  fi

  pip install -e "$BACKEND_DIR/python"
else
  echo "[warn] python interpreter not found; skipping Python setup" >&2
fi

cargo fetch --manifest-path "$BACKEND_DIR/Cargo.toml"
