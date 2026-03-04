#!/usr/bin/env bash
# build_with_logging.sh - Build script with comprehensive logging
# Usage: ./scripts/build_with_logging.sh [preset]

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
cd "${PROJECT_ROOT}"

# Ensure third-party deps exist before configure/build
# shellcheck source=./include/ensure_third_party.sh
. "${SCRIPT_DIR}/include/ensure_third_party.sh"
ensure_third_party

# Log file in project directory
LOG_FILE="${PROJECT_ROOT}/logs/build_$(date +%Y%m%d_%H%M%S).log"
mkdir -p "${PROJECT_ROOT}/logs"

# Preset (default to macos-arm64-debug)
PRESET="${1:-macos-arm64-debug}"

# Function to log with timestamp
log() {
  echo "[$(date '+%Y-%m-%d %H:%M:%S')] $*" | tee -a "$LOG_FILE"
}

log "=== Build Script Started ==="
log "Preset: $PRESET"
log "Working directory: $PROJECT_ROOT"
log "CMake version: $(cmake --version 2>&1 || echo 'CMake not found')"
log "Ninja version: $(ninja --version 2>&1 || echo 'Ninja not found')"
log "Architecture: $(uname -m)"
log ""

# Redirect all output to both terminal and log file
exec > >(tee -a "$LOG_FILE") 2>&1

log "=== Configuring CMake ==="
if cmake --preset "$PRESET"; then
  log "CMake configuration successful"
else
  log "CMake configuration failed with exit code $?"
  exit 1
fi

log ""
log "=== Building ==="
if cmake --build --preset "$PRESET"; then
  log "Build successful"
else
  log "Build failed with exit code $?"
  exit 1
fi

log ""
log "=== Build Complete ==="
log "Log file: $LOG_FILE"
