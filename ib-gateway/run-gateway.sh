#!/usr/bin/env bash
# Convenience script to run IB Client Portal Gateway
# Handles failures gracefully to avoid killing tmux sessions
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

if [ ! -f "${SCRIPT_DIR}/bin/run.sh" ]; then
  echo "Error: Gateway not found at ${SCRIPT_DIR}" >&2
  echo "Run install-ib-gateway.sh first" >&2
  echo "" >&2
  echo "Press any key to continue..." >&2
  read -n 1 -s || true
  exit 1
fi

cd "${SCRIPT_DIR}"

# Set up Java environment (use Homebrew Java 17 if available)
if [ -d "/usr/local/opt/openjdk@17" ]; then
  export JAVA_HOME="/usr/local/opt/openjdk@17"
  export PATH="/usr/local/opt/openjdk@17/bin:$PATH"
elif [ -d "/opt/homebrew/opt/openjdk@17" ]; then
  # Apple Silicon Mac
  export JAVA_HOME="/opt/homebrew/opt/openjdk@17"
  export PATH="/opt/homebrew/opt/openjdk@17/bin:$PATH"
fi

# Verify Java is available
if ! command -v java >/dev/null 2>&1; then
  echo "Error: Java not found. Please install Java 17:" >&2
  echo "  brew install openjdk@17" >&2
  echo "" >&2
  echo "Press any key to continue..." >&2
  read -n 1 -s || true
  exit 1
fi

# Find config file (prefer conf.yaml, fallback to conf.tws.yaml)
CONFIG_FILE="${SCRIPT_DIR}/root/conf.yaml"
if [ ! -f "${CONFIG_FILE}" ]; then
  CONFIG_FILE="${SCRIPT_DIR}/root/conf.tws.yaml"
fi

# Normalize config file path to absolute path (fixes ..// issues)
if [ -f "${CONFIG_FILE}" ]; then
  # Use realpath if available, otherwise use cd/pwd
  if command -v realpath >/dev/null 2>&1; then
    CONFIG_FILE="$(realpath "${CONFIG_FILE}")"
  else
    CONFIG_FILE="$(cd "$(dirname "${CONFIG_FILE}")" && pwd)/$(basename "${CONFIG_FILE}")"
  fi
fi

if [ ! -f "${CONFIG_FILE}" ]; then
  echo "[ERROR] IB Gateway configuration file not found" >&2
  echo "[PATH] Searched locations:" >&2
  echo "  [PATH] ${SCRIPT_DIR}/root/conf.yaml" >&2
  echo "  [PATH] ${SCRIPT_DIR}/root/conf.tws.yaml" >&2
  echo "[CONTEXT] Gateway directory: [PATH] ${SCRIPT_DIR}" >&2
  echo "Press any key to continue..." >&2
  read -n 1 -s || true
  exit 1
fi

# Run gateway with config file and handle failures gracefully
if ! ./bin/run.sh "${CONFIG_FILE}" "$@"; then
  EXIT_CODE=$?
  echo "" >&2
  echo "[ERROR] IB Gateway exited with error code ${EXIT_CODE}" >&2
  echo "[PATH] Config file used: ${CONFIG_FILE}" >&2
  echo "[PATH] Gateway directory: ${SCRIPT_DIR}" >&2
  echo "[PATH] Log file: ${SCRIPT_DIR}/logs/gateway.log" >&2
  echo "Press any key to continue..." >&2
  read -n 1 -s || true
  exit "${EXIT_CODE}"
fi
