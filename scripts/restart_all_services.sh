#!/usr/bin/env bash
# Restart all services: stop in reverse order, then start in dependency order (delegates to service.sh)
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
SCRIPTS_DIR="${ROOT_DIR}/scripts"

exec "${SCRIPTS_DIR}/service.sh" restart-all
