#!/usr/bin/env bash
# Uninstall IB Gateway brew service
set -euo pipefail

PLIST_NAME="com.davidl71.ib-gateway"
BREW_SERVICES_DIR="${HOME}/Library/LaunchAgents"
PLIST_DEST="${BREW_SERVICES_DIR}/${PLIST_NAME}.plist"

# Check if brew is available
if ! command -v brew >/dev/null 2>&1; then
  echo "[ERROR] Homebrew is not installed" >&2
  exit 1
fi

# Stop service if running
if brew services list | grep -q "${PLIST_NAME}.*started"; then
  echo "[INFO] Stopping service..." >&2
  brew services stop "${PLIST_NAME}" 2>/dev/null || true
fi

# Remove plist file
if [ -f "${PLIST_DEST}" ]; then
  echo "[INFO] Removing plist file: [PATH] ${PLIST_DEST}" >&2
  rm -f "${PLIST_DEST}"
  echo "[INFO] Service uninstalled" >&2
else
  echo "[INFO] Service not installed (plist not found)" >&2
fi
