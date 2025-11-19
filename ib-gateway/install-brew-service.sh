#!/usr/bin/env bash
# Install IB Gateway as a brew service
# This creates a launchd plist and registers it with brew services
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PLIST_SOURCE="${SCRIPT_DIR}/com.davidl71.ib-gateway.plist"
PLIST_NAME="com.davidl71.ib-gateway"
BREW_SERVICES_DIR="${HOME}/Library/LaunchAgents"

# Check if brew is available
if ! command -v brew >/dev/null 2>&1; then
  echo "[ERROR] Homebrew is not installed" >&2
  echo "[INFO] Install Homebrew: https://brew.sh" >&2
  exit 1
fi

# Detect Java installation
JAVA_HOME=""
if [ -d "/opt/homebrew/opt/openjdk@17" ]; then
  JAVA_HOME="/opt/homebrew/opt/openjdk@17"
elif [ -d "/usr/local/opt/openjdk@17" ]; then
  JAVA_HOME="/usr/local/opt/openjdk@17"
fi

if [ -z "${JAVA_HOME}" ]; then
  echo "[ERROR] Java 17 not found" >&2
  echo "[INFO] Install Java 17: brew install openjdk@17" >&2
  exit 1
fi

# Check if plist source exists
if [ ! -f "${PLIST_SOURCE}" ]; then
  echo "[ERROR] Plist template not found: ${PLIST_SOURCE}" >&2
  exit 1
fi

# Create brew services directory if it doesn't exist
mkdir -p "${BREW_SERVICES_DIR}"

# Generate plist with actual paths
PLIST_DEST="${BREW_SERVICES_DIR}/${PLIST_NAME}.plist"
sed -e "s|GATEWAY_DIR|${SCRIPT_DIR}|g" \
    -e "s|JAVA_HOME_PLACEHOLDER|${JAVA_HOME}|g" \
    "${PLIST_SOURCE}" > "${PLIST_DEST}"

echo "[INFO] Installed plist to: [PATH] ${PLIST_DEST}" >&2
echo "[INFO] Service name: ${PLIST_NAME}" >&2
echo "" >&2
echo "[INFO] To start the service:" >&2
echo "  brew services start ${PLIST_NAME}" >&2
echo "" >&2
echo "[INFO] To stop the service:" >&2
echo "  brew services stop ${PLIST_NAME}" >&2
echo "" >&2
echo "[INFO] To restart the service:" >&2
echo "  brew services restart ${PLIST_NAME}" >&2
echo "" >&2
echo "[INFO] To check service status:" >&2
echo "  brew services list | grep ${PLIST_NAME}" >&2
