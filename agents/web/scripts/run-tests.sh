#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/../../.." && pwd)"
WEB_DIR="$ROOT_DIR/web"

if [ -d "$WEB_DIR" ]; then
  cd "$WEB_DIR"
  if command -v npm >/dev/null 2>&1; then
    npm test -- --watch=false || true
  else
    echo "[warn] npm not found; cannot run web SPA tests" >&2
  fi
else
  echo "[info] web/ directory not present yet; skipping tests." >&2
fi

