#!/usr/bin/env bash
# Stop all services in reverse dependency order (delegates to service.sh)
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
SCRIPTS_DIR="${ROOT_DIR}/scripts"

exec "${SCRIPTS_DIR}/service.sh" stop-all
