#!/usr/bin/env bash
# Start all services in dependency order (delegates to service.sh)
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
SCRIPTS_DIR="${ROOT_DIR}/scripts"

exec "${SCRIPTS_DIR}/service.sh" start-all
