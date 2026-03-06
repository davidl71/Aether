#!/usr/bin/env bash
# Convenience script to run IB Client Portal Gateway
# Handles failures gracefully to avoid killing tmux sessions
# Ensures Java 11 is available (installs openjdk@11 via Homebrew on macOS if missing).
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

# --- Ensure Java 11 and set JAVA_HOME / PATH ---
ensure_java() {
  if command -v java &>/dev/null; then
    local ver
    ver=$(java -version 2>&1 | head -1) || true
    if [[ "$ver" == *"11"* ]] || [[ "$ver" == *"openjdk \"11"* ]]; then
      export JAVA_HOME="${JAVA_HOME:-$(dirname "$(dirname "$(command -v java)")")}"
      export PATH="${JAVA_HOME}/bin:${PATH}"
      return 0
    fi
  fi

  if [[ "$(uname -s)" == "Darwin" ]]; then
    if ! command -v brew &>/dev/null; then
      echo "Error: Java 11 not found and Homebrew not installed." >&2
      echo "Install Java 11: brew install openjdk@11" >&2
      echo "Or set JAVA_HOME and PATH to a Java 11 runtime." >&2
      exit 1
    fi
    if ! brew list openjdk@11 &>/dev/null; then
      echo "Java 11 not found. Installing openjdk@11 via Homebrew..." >&2
      brew install openjdk@11
    fi
    JAVA_HOME="$(brew --prefix openjdk@11)"
    export JAVA_HOME
    export PATH="${JAVA_HOME}/bin:${PATH}"
    return 0
  fi

  echo "Error: Java 11 is required but not found." >&2
  echo "macOS: brew install openjdk@11" >&2
  echo "Linux: sudo apt-get install openjdk-11-jdk  (or equivalent)" >&2
  echo "Then set JAVA_HOME and PATH, or run this script again." >&2
  exit 1
}

ensure_java

if [ ! -f "${SCRIPT_DIR}/bin/run.sh" ]; then
  echo "Error: Gateway not found at ${SCRIPT_DIR}" >&2
  echo "Run install-ib-gateway.sh first" >&2
  echo "" >&2
  echo "Press any key to continue..." >&2
  read -n 1 -s || true
  exit 1
fi

cd "${SCRIPT_DIR}"

# Find config file (prefer conf.yaml, fallback to conf.tws.yaml)
# Use relative path so the gateway's loader doesn't mangle absolute paths (..//...).
CONFIG_REL="root/conf.yaml"
if [ ! -f "${CONFIG_REL}" ]; then
  CONFIG_REL="root/conf.tws.yaml"
fi

if [ ! -f "${CONFIG_REL}" ]; then
  echo "Error: No config file found. Expected: root/conf.yaml or root/conf.tws.yaml" >&2
  echo "Press any key to continue..." >&2
  read -n 1 -s || true
  exit 1
fi

# Port: default 5001, override with IB_GATEWAY_PORT (e.g. if 5001 is in use or to match legacy 5000).
# Why 5000? Gateway can end up on 5000 if: (1) IB_GATEWAY_PORT=5000 is set in your env/shell,
# (2) the gateway was started by IB's native launcher (often defaults to 5000), or (3) root/conf.yaml
# or root/conf.tws.yaml was overwritten by an installer with listenPort: 5000. This script forces
# the port from IB_GATEWAY_PORT (default 5001) by writing root/conf.port.<port>.yaml when needed.
GATEWAY_PORT="${IB_GATEWAY_PORT:-5001}"

# If config has a different listenPort, write a variant under root/ so relative paths in config still work.
CONFIG_TO_USE="${CONFIG_REL}"
CURRENT_PORT=$(grep -E '^\s*listenPort:' "${CONFIG_REL}" | head -1 | sed -E 's/.*listenPort:\s*([0-9]+).*/\1/')
if [ -n "${CURRENT_PORT}" ] && [ "${CURRENT_PORT}" != "${GATEWAY_PORT}" ]; then
  PORT_CONFIG="${SCRIPT_DIR}/root/conf.port.${GATEWAY_PORT}.yaml"
  sed -E "s/^(\\s*listenPort:\\s*)[0-9]+/\\1${GATEWAY_PORT}/" "${CONFIG_REL}" > "${PORT_CONFIG}"
  CONFIG_TO_USE="root/conf.port.${GATEWAY_PORT}.yaml"
fi

# Warn if port is already in use.
if command -v lsof >/dev/null 2>&1; then
  if lsof -i ":${GATEWAY_PORT}" -sTCP:LISTEN -t >/dev/null 2>&1; then
    echo "⚠ Port ${GATEWAY_PORT} is already in use." >&2
    echo "  To see what is using it: lsof -i :${GATEWAY_PORT}" >&2
    echo "  On macOS, port 5000 is often used by AirPlay Receiver (System Settings → General → AirDrop & Handoff → AirPlay Receiver: Off)." >&2
    echo "  Or run on another port: IB_GATEWAY_PORT=5002 $(basename "$0")" >&2
    echo "  Then set IB_PORTAL_URL=https://localhost:5002/v1/portal and VITE_IB_GATEWAY_URL=https://localhost:5002 for the IB service and PWA." >&2
    echo "" >&2
  fi
fi

# Run gateway with config file and handle failures gracefully
if ! ./bin/run.sh "${CONFIG_TO_USE}" "$@"; then
  EXIT_CODE=$?
  echo "" >&2
  echo "⚠ IB Gateway exited with error code ${EXIT_CODE}" >&2
  echo "Press any key to continue..." >&2
  read -n 1 -s || true
  exit "${EXIT_CODE}"
fi
