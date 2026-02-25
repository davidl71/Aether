#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/../../.." && pwd)"

WEB_DIR="$ROOT_DIR/web"

if [ -d "$WEB_DIR" ]; then
  if [ -s "$HOME/.nvm/nvm.sh" ]; then
    # shellcheck disable=SC1090
    . "$HOME/.nvm/nvm.sh"
    nvm use --lts >/dev/null 2>&1 || nvm install --lts
  fi
  cd "$WEB_DIR"
  if command -v npm >/dev/null 2>&1; then
    npm install
  else
    echo "[warn] npm not found; install Node.js to work on the web SPA" >&2
  fi
else
  echo "[info] web/ directory not present yet; create SPA scaffold before running setup." >&2
fi
