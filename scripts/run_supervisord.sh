#!/usr/bin/env bash
# Run Python supervisord with config/supervisord.conf (process manager for all backends).
# Requires: pip install supervisord  (or  apt install supervisor  for system-wide).
#
# Usage:
#   ./scripts/run_supervisord.sh          # start supervisord in foreground
#   ./scripts/run_supervisord.sh -d       # daemonize (if supervisord supports -d)
#
# Control once running:
#   supervisorctl -c config/supervisord.conf status
#   supervisorctl -c config/supervisord.conf start ib
#   supervisorctl -c config/supervisord.conf stop alpaca
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
CONFIG="${PROJECT_ROOT}/config/supervisord.conf"

export PROJECT_ROOT

if [ ! -f "$CONFIG" ]; then
  echo "Error: config not found: $CONFIG" >&2
  exit 1
fi

if ! command -v supervisord >/dev/null 2>&1; then
  echo "supervisord not found. Install with: pip install supervisord  or  apt install supervisor" >&2
  exit 1
fi

cd "$PROJECT_ROOT"
exec supervisord -c "$CONFIG" "$@"
